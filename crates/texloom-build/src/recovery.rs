//! Unsaved source/history recovery, separate from explicit project Save.
//! Framed, hash-chained accepted mutations are flushed before protection is
//! acknowledged. Recovery never writes the original project or trusts builds.
use crate::{
    persistence::DiskSnapshot,
    snapshot::{is_source, relative, Fingerprint},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use texloom_core::document::{
    recovery::Checkpoint, FileChange, Origin, Patch, Revision, Selection, Session, TypingGroup,
};

type Result<T> = std::result::Result<T, String>;
const OWNER: &[u8] = b"Texloom unsaved source recovery v1\n";
const DOMAIN: &[u8] = b"Texloom recovery frame v1\0";
const MAX_FRAME: u64 = 224 * 1024 * 1024;
const MAX_GENERATION: u64 = 256 * 1024 * 1024;
const MAX_STORE: u64 = 2 * 1024 * 1024 * 1024;
const MAX_DRAFTS: usize = 32;
const MAX_FILES: usize = 64;
const ROTATE_BYTES: u64 = 16 * 1024 * 1024;
const ROTATE_EVENTS: u64 = 1024;
static NEXT: AtomicU64 = AtomicU64::new(0);

fn fail(e: impl std::fmt::Display) -> String {
    e.to_string()
}
fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.into())
    }
}
fn checked(path: &Path, directory: bool) -> Result<()> {
    let meta = fs::symlink_metadata(path).map_err(fail)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        require(
            meta.file_attributes() & 0x400 == 0,
            "Reparse point in recovery path",
        )?;
    }
    require(
        !meta.file_type().is_symlink()
            && meta.is_dir() == directory
            && (directory || meta.is_file()),
        "Invalid recovery path type",
    )
}
fn read_small(path: &Path, limit: u64) -> Result<Vec<u8>> {
    checked(path, false)?;
    let file = File::open(path).map_err(fail)?;
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes).map_err(fail)?;
    require(
        bytes.len() as u64 <= limit,
        "Recovery evidence exceeds byte budget",
    )?;
    Ok(bytes)
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<File> {
    let mut options = OpenOptions::new();
    options.read(true).write(true).create_new(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(1); // readers only while this writer owns the file
    }
    let mut file = options.open(path).map_err(fail)?;
    file.write_all(bytes).map_err(fail)?;
    file.sync_all().map_err(fail)?;
    Ok(file)
}
fn valid_id(id: &str) -> bool {
    id.len() == 32
        && id
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}
fn identity() -> String {
    Fingerprint::bytes(
        format!(
            "{}:{:?}:{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )
        .as_bytes(),
    )
    .sha256[..32]
        .into()
}
fn generation_name(number: u64) -> String {
    format!("{number:016x}.wal")
}
fn generation_number(name: &str) -> Option<u64> {
    let digits = name.strip_suffix(".wal")?;
    if digits.len() != 16
        || !digits
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return None;
    }
    u64::from_str_radix(digits, 16).ok().filter(|n| *n > 0)
}
fn lock(directory: &Path, create: bool) -> Result<File> {
    checked(directory, true)?;
    let path = directory.join("owner.lock");
    if create {
        drop(write_new(&path, OWNER)?);
    }
    require(
        read_small(&path, 128)? == OWNER,
        "Unknown recovery owner marker",
    )?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .map_err(fail)?;
    file.try_lock()
        .map_err(|_| "This recovery draft is open in another Texloom instance".to_string())?;
    Ok(file)
}
fn generations(directory: &Path) -> Result<Vec<(u64, PathBuf)>> {
    checked(directory, true)?;
    let mut out = Vec::new();
    let mut count = 0;
    for entry in fs::read_dir(directory).map_err(fail)? {
        let entry = entry.map_err(fail)?;
        count += 1;
        require(count <= MAX_FILES, "Recovery draft file retention is full")?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name == "owner.lock" {
            continue;
        }
        let number =
            generation_number(&name).ok_or("Unexpected recovery file; data left in place")?;
        checked(&entry.path(), false)?;
        out.push((number, entry.path()));
    }
    out.sort_by_key(|(number, _)| *number);
    Ok(out)
}
fn valid_source(file: &str) -> Result<()> {
    require(
        is_source(&relative(file)?),
        "Invalid source filename in recovery data",
    )
}
fn validate_disk(disk: &DiskSnapshot) -> Result<()> {
    require(
        disk.root.is_absolute() && disk.files.len() <= 1200,
        "Invalid recovery project identity",
    )?;
    for (name, value) in &disk.files {
        valid_source(name)?;
        if let Some(hash) = value {
            require(
                hash.bytes <= 4_000_000
                    && hash.sha256.len() == 64
                    && hash
                        .sha256
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
                "Invalid saved-file fingerprint",
            )?;
        }
    }
    Ok(())
}
fn validate_session(session: &Session) -> Result<()> {
    for name in session.sources().keys() {
        valid_source(name)?;
    }
    require(
        relative(&session.profile().main)?
            .extension()
            .and_then(|s| s.to_str())
            == Some("tex"),
        "Invalid recovery main file",
    )
}

struct LimitedBytes(Vec<u8>);
impl Write for LimitedBytes {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.0.len() as u64 + bytes.len() as u64 > MAX_FRAME {
            return Err(std::io::Error::other("Recovery record exceeds byte budget"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn encode(value: &impl Serialize) -> Result<Vec<u8>> {
    let mut bytes = LimitedBytes(Vec::new());
    serde_json::to_writer(&mut bytes, value).map_err(fail)?;
    Ok(bytes.0)
}
fn digest(previous: &[u8; 32], prefix: &[u8; 16], payload: &[u8]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(DOMAIN);
    hash.update(previous);
    hash.update(prefix);
    hash.update(payload);
    hash.finalize().into()
}
fn frame(payload: &[u8], previous: &[u8; 32]) -> Result<(Vec<u8>, [u8; 32])> {
    require(
        payload.len() as u64 <= MAX_FRAME,
        "Recovery record exceeds byte budget",
    )?;
    let length = payload.len() as u64;
    let mut prefix = [0; 16];
    prefix[..8].copy_from_slice(&length.to_le_bytes());
    prefix[8..].copy_from_slice(&(!length).to_le_bytes());
    let checksum = digest(previous, &prefix, payload);
    let mut bytes = Vec::with_capacity(payload.len() + 48);
    bytes.extend_from_slice(&prefix);
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(&checksum);
    Ok((bytes, checksum))
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Seal {
    generation: u64,
    fingerprint: Fingerprint,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Header {
    schema: u32,
    id: String,
    generation: u64,
    disk: DiskSnapshot,
    checkpoint: Checkpoint,
    previous: Vec<Seal>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Mutation {
    Patch {
        patch: Patch,
        origin: Origin,
        group: Option<TypingGroup>,
    },
    Undo,
    Redo,
    Selection {
        selection: Selection,
    },
    Saved {
        disk: DiskSnapshot,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    base: u64,
    revision: u64,
    mutation: Mutation,
}

/// App-managed store. No draft is evicted automatically to make room.
pub struct Store {
    root: PathBuf,
}
pub struct Journal {
    directory: PathBuf,
    id: String,
    _lock: File,
    file: File,
    generation: u64,
    digest: [u8; 32],
    hash: Sha256,
    bytes: u64,
    tail_bytes: u64,
    events: u64,
    revision: u64,
    project: PathBuf,
    previous: Vec<Seal>,
    poisoned: bool,
}
pub struct Recovered {
    pub session: Session,
    pub disk: DiskSnapshot,
    pub warning: Option<String>,
}
#[derive(Clone, Serialize)]
pub struct Draft {
    pub id: String,
    pub project: String,
    pub main: String,
    pub revision: u64,
    pub source_bytes: u64,
    pub files: usize,
    pub warning: Option<String>,
}
#[derive(Default, Serialize)]
pub struct Inventory {
    pub drafts: Vec<Draft>,
    pub errors: Vec<String>,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        if !path.exists() {
            fs::create_dir_all(path).map_err(fail)?;
        }
        checked(path, true)?;
        let root = path.canonicalize().map_err(fail)?;
        let marker = root.join("owner.txt");
        if !marker.exists() {
            // Refuse to claim a pre-existing directory containing unknown data.
            require(
                fs::read_dir(&root).map_err(fail)?.next().is_none(),
                "Recovery folder is not owned by Texloom",
            )?;
            drop(write_new(&marker, OWNER)?);
        }
        require(
            read_small(&marker, 128)? == OWNER,
            "Unknown recovery store marker",
        )?;
        Ok(Self { root })
    }
    fn directories(&self) -> Result<Vec<PathBuf>> {
        checked(&self.root, true)?;
        require(
            read_small(&self.root.join("owner.txt"), 128)? == OWNER,
            "Recovery store changed",
        )?;
        let mut directories = Vec::new();
        for entry in fs::read_dir(&self.root).map_err(fail)? {
            let entry = entry.map_err(fail)?;
            if entry.file_name() == "owner.txt" || entry.file_name() == "store.lock" {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            require(
                valid_id(&name),
                "Unexpected recovery store entry; data left in place",
            )?;
            checked(&entry.path(), true)?;
            directories.push(entry.path());
            require(
                directories.len() <= MAX_DRAFTS,
                "Recovery draft retention is full; review earlier drafts",
            )?;
        }
        directories.sort();
        Ok(directories)
    }
    fn budget_lock(&self) -> Result<File> {
        let path = self.root.join("store.lock");
        if !path.exists() {
            match write_new(&path, OWNER) {
                Ok(file) => drop(file),
                Err(error) if path.exists() => {
                    let _ = error;
                }
                Err(error) => return Err(error),
            }
        }
        require(
            read_small(&path, 128)? == OWNER,
            "Recovery store lock changed",
        )?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(fail)?;
        file.try_lock().map_err(|_| {
            "Recovery storage is busy in another Texloom instance; Save remains available"
                .to_string()
        })?;
        Ok(file)
    }
    fn budget(&self, additional: u64) -> Result<()> {
        let mut total = additional;
        for directory in self.directories()? {
            for (_, path) in generations(&directory)? {
                total = total
                    .checked_add(fs::metadata(path).map_err(fail)?.len())
                    .ok_or("Recovery byte budget overflow")?;
                require(
                    total <= MAX_STORE,
                    "Recovery storage is full; save your work and review earlier drafts",
                )?;
            }
        }
        require(total <= MAX_STORE, "Recovery storage is full")
    }
    pub fn begin(&self, disk: &DiskSnapshot, session: &Session) -> Result<Journal> {
        validate_disk(disk)?;
        validate_session(session)?;
        let _budget = self.budget_lock()?;
        require(
            self.directories()?.len() < MAX_DRAFTS,
            "Recovery draft retention is full; review earlier drafts",
        )?;
        self.budget(MAX_GENERATION)?;
        let id = identity();
        let directory = self.root.join(&id);
        let header = Header {
            schema: 1,
            id: id.clone(),
            generation: 1,
            disk: disk.clone(),
            checkpoint: session.checkpoint(),
            previous: vec![],
        };
        let (bytes, digest) = frame(&encode(&header)?, &[0; 32])?;
        fs::create_dir(&directory).map_err(fail)?;
        let owner = lock(&directory, true)?;
        let file = write_new(&directory.join(generation_name(1)), &bytes)?;
        let mut hash = Sha256::new();
        hash.update(&bytes);
        Ok(Journal {
            directory,
            id,
            _lock: owner,
            file,
            generation: 1,
            digest,
            hash,
            bytes: bytes.len() as u64,
            tail_bytes: 0,
            events: 0,
            revision: session.revision().0,
            project: disk.root.clone(),
            previous: vec![],
            poisoned: false,
        })
    }
    pub fn inventory(&self) -> Result<Inventory> {
        let mut inventory = Inventory::default();
        checked(&self.root, true)?;
        require(
            read_small(&self.root.join("owner.txt"), 128)? == OWNER,
            "Recovery store changed",
        )?;
        let mut directories = Vec::new();
        for (index, entry) in fs::read_dir(&self.root).map_err(fail)?.enumerate() {
            if index >= MAX_DRAFTS + 16 {
                inventory.errors.push(
                    "Recovery inventory entry budget exceeded; additional data was left in place"
                        .into(),
                );
                break;
            }
            let entry = entry.map_err(fail)?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name == "owner.txt" || name == "store.lock" {
                continue;
            }
            if valid_id(&name)
                && checked(&entry.path(), true).is_ok()
                && directories.len() < MAX_DRAFTS
            {
                directories.push(entry.path());
            } else {
                inventory
                    .errors
                    .push(format!("Unexpected recovery entry retained: {name}"));
            }
        }
        directories.sort();
        for directory in directories {
            let id = directory
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let owner = match lock(&directory, false) {
                Ok(file) => file,
                Err(error) if error.contains("another Texloom instance") => continue,
                Err(error) => {
                    inventory.errors.push(format!("{id}: {error}"));
                    continue;
                }
            };
            match load(&directory, &id) {
                Ok(recovered) => inventory.drafts.push(Draft {
                    id,
                    project: recovered.disk.root.to_string_lossy().into_owned(),
                    main: recovered.session.profile().main.clone(),
                    revision: recovered.session.revision().0,
                    source_bytes: recovered
                        .session
                        .sources()
                        .values()
                        .map(|s| s.len() as u64)
                        .sum(),
                    files: recovered.session.sources().len(),
                    warning: recovered.warning,
                }),
                Err(error) => inventory.errors.push(format!("{id}: {error}")),
            }
            drop(owner);
        }
        Ok(inventory)
    }
    pub fn recover(&self, id: &str, project: &Path) -> Result<Recovered> {
        require(valid_id(id), "Invalid recovery draft ID")?;
        let directory = self.root.join(id);
        let _owner = lock(&directory, false)?;
        let recovered = load(&directory, id)?;
        require(
            recovered.disk.root == project,
            "Recovery draft belongs to a different project",
        )?;
        Ok(recovered)
    }
    /// Explicit discard removes only recognized files with unchanged fingerprints.
    /// Unknown/corrupt bytes are retained for inspection, never recursively removed.
    pub fn discard(&self, id: &str, project: &Path) -> Result<()> {
        require(valid_id(id), "Invalid recovery draft ID")?;
        let _budget = self.budget_lock()?;
        let directory = self.root.join(id);
        let owner = lock(&directory, false)?;
        let recovered = load(&directory, id)?;
        require(
            recovered.disk.root == project,
            "Recovery draft belongs to a different project",
        )?;
        let mut receipts = Vec::new();
        for (number, path) in generations(&directory)? {
            let mut file = File::open(&path).map_err(fail)?;
            let mut consumed = 0;
            match read_frame(&mut file, &[0; 32], &mut consumed)? {
                FrameRead::Complete(payload, mut digest) => {
                    let header: Header = serde_json::from_slice(&payload).map_err(fail)?;
                    require(
                        header.schema == 1
                            && header.id == id
                            && header.generation == number
                            && header.disk.root == project,
                        "Unknown recovery generation; data left in place",
                    )?;
                    let mut count = 0;
                    loop {
                        match read_frame(&mut file, &digest, &mut consumed)? {
                            FrameRead::Complete(payload, next) => {
                                count += 1;
                                require(
                                    count <= ROTATE_EVENTS + 1,
                                    "Recovery event budget exceeded",
                                )?;
                                let _: Record = serde_json::from_slice(&payload).map_err(fail)?;
                                digest = next;
                            }
                            FrameRead::End | FrameRead::Partial => break,
                        }
                    }
                }
                // A partial checkpoint has no complete project binding. It can
                // still be inspected manually; never delete it as a known draft.
                _ => {
                    return Err(
                        "Incomplete checkpoint retained; recovery bytes were left in place".into(),
                    )
                }
            }
            receipts.push((path.clone(), Fingerprint::file(&path)?));
        }
        for (path, fingerprint) in receipts {
            checked(&path, false)?;
            require(
                Fingerprint::file(&path)? == fingerprint,
                "Recovery bytes changed before discard",
            )?;
            fs::remove_file(path).map_err(fail)?;
        }
        drop(owner);
        require(
            read_small(&directory.join("owner.lock"), 128)? == OWNER,
            "Recovery owner changed",
        )?;
        fs::remove_file(directory.join("owner.lock")).map_err(fail)?;
        fs::remove_dir(directory).map_err(fail)
    }
}

impl Journal {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn protected_revision(&self) -> Option<u64> {
        (!self.poisoned).then_some(self.revision)
    }
    pub fn checkpoint(
        &mut self,
        store: &Store,
        session: &Session,
        disk: &DiskSnapshot,
    ) -> Result<()> {
        let result = (|| {
            require(
                self.directory.parent() == Some(&store.root) && disk.root == self.project,
                "Recovery project/store changed",
            )?;
            let _budget = store.budget_lock()?;
            self.rotate(store, session, disk)
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    /// Called after canonical acceptance. Failure must be surfaced as recovery
    /// degradation, not misreported as a rejected source edit.
    pub fn record(
        &mut self,
        store: &Store,
        base: u64,
        mutation: Mutation,
        session: &Session,
        disk: &DiskSnapshot,
    ) -> Result<()> {
        let result = (|| {
            require(
                self.directory.parent() == Some(&store.root),
                "Recovery journal belongs to a different store",
            )?;
            require(
                disk.root == self.project,
                "Recovery project identity changed",
            )?;
            let _budget = store.budget_lock()?;
            let changes_revision = matches!(
                &mutation,
                Mutation::Patch { .. } | Mutation::Undo | Mutation::Redo
            );
            require(
                session.revision().0
                    == base
                        .checked_add(u64::from(changes_revision))
                        .ok_or("Recovery revision exhausted")?,
                "Recovery mutation revision mismatch",
            )?;
            if let Mutation::Patch { patch, .. } = &mutation {
                require(
                    patch.base == Revision(base),
                    "Recovery patch revision mismatch",
                )?;
            }
            if self.poisoned || self.events >= ROTATE_EVENTS || self.tail_bytes >= ROTATE_BYTES {
                return self.rotate(store, session, disk);
            }
            require(base == self.revision, "Recovery mutation revision gap")?;
            let revision = session.revision().0;
            let record = Record {
                base,
                revision,
                mutation,
            };
            let (bytes, digest) = frame(&encode(&record)?, &self.digest)?;
            if self.bytes + bytes.len() as u64 > MAX_GENERATION {
                return self.rotate(store, session, disk);
            }
            store.budget(bytes.len() as u64)?;
            require(
                self.file.metadata().map_err(fail)?.len() == self.bytes,
                "Recovery file changed while editing",
            )?;
            self.file.write_all(&bytes).map_err(fail)?;
            self.file.sync_all().map_err(fail)?;
            self.hash.update(&bytes);
            self.digest = digest;
            self.bytes += bytes.len() as u64;
            self.tail_bytes += bytes.len() as u64;
            self.events += 1;
            self.revision = revision;
            Ok(())
        })();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn rotate(&mut self, store: &Store, session: &Session, disk: &DiskSnapshot) -> Result<()> {
        validate_disk(disk)?;
        validate_session(session)?;
        store.budget(MAX_GENERATION)?;
        let files = generations(&self.directory)?;
        require(
            files.len() + 1 < MAX_FILES,
            "Recovery generation retention is full",
        )?;
        let generation = files
            .last()
            .map(|(n, _)| *n)
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("Recovery generation exhausted")?;
        let previous = if self.poisoned {
            Vec::new()
        } else {
            vec![Seal {
                generation: self.generation,
                fingerprint: Fingerprint {
                    bytes: self.bytes,
                    sha256: format!("{:x}", self.hash.clone().finalize()),
                },
            }]
        };
        let header = Header {
            schema: 1,
            id: self.id.clone(),
            generation,
            disk: disk.clone(),
            checkpoint: session.checkpoint(),
            previous,
        };
        let (bytes, digest) = frame(&encode(&header)?, &[0; 32])?;
        let file = write_new(&self.directory.join(generation_name(generation)), &bytes)?;
        let old_previous = std::mem::replace(&mut self.previous, header.previous);
        self.file = file;
        self.generation = generation;
        self.digest = digest;
        self.hash = Sha256::new();
        self.hash.update(&bytes);
        self.bytes = bytes.len() as u64;
        self.tail_bytes = 0;
        self.events = 0;
        self.revision = session.revision().0;
        self.poisoned = false;
        for seal in old_previous {
            let path = self.directory.join(generation_name(seal.generation));
            // Deletion is optional; the new flushed checkpoint is authoritative.
            if checked(&path, false).is_ok()
                && Fingerprint::file(&path).ok().as_ref() == Some(&seal.fingerprint)
            {
                let _ = fs::remove_file(path);
            }
        }
        Ok(())
    }
}

enum FrameRead {
    Complete(Vec<u8>, [u8; 32]),
    End,
    Partial,
}
fn read_frame(file: &mut File, previous: &[u8; 32], consumed: &mut u64) -> Result<FrameRead> {
    let mut prefix = [0; 16];
    let first = file.read(&mut prefix).map_err(fail)?;
    if first == 0 {
        return Ok(FrameRead::End);
    }
    if first < prefix.len() {
        match file.read_exact(&mut prefix[first..]) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(FrameRead::Partial)
            }
            Err(e) => return Err(fail(e)),
        }
    }
    let length = u64::from_le_bytes(prefix[..8].try_into().unwrap());
    require(
        u64::from_le_bytes(prefix[8..].try_into().unwrap()) == !length && length <= MAX_FRAME,
        "Corrupt recovery record length",
    )?;
    *consumed = consumed
        .checked_add(length + 48)
        .ok_or("Recovery size overflow")?;
    require(
        *consumed <= MAX_GENERATION,
        "Recovery generation exceeds byte budget",
    )?;
    let mut payload = vec![0; length as usize];
    let mut checksum = [0; 32];
    for buffer in [&mut payload[..], &mut checksum[..]] {
        match file.read_exact(buffer) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                return Ok(FrameRead::Partial)
            }
            Err(e) => return Err(fail(e)),
        }
    }
    require(
        checksum == digest(previous, &prefix, &payload),
        "Recovery checksum mismatch; data left in place",
    )?;
    Ok(FrameRead::Complete(payload, checksum))
}
fn apply(record: Record, session: &mut Session, disk: &mut DiskSnapshot) -> Result<()> {
    require(
        record.base == session.revision().0,
        "Recovery record is out of order",
    )?;
    let changed = match record.mutation {
        Mutation::Patch {
            patch,
            origin,
            group,
        } => {
            require(
                patch.base == Revision(record.base),
                "Recovery patch base mismatch",
            )?;
            if let (Origin::Source, None, [FileChange::Edit { file, edits }]) =
                (origin, &patch.profile, patch.files.as_slice())
            {
                // The journal carries data, not prepared authority. Revalidate
                // expected bytes, limits and the current replay owner/base.
                let candidate = session
                    .prepare_source_edit(patch.base, file, edits.clone())
                    .map_err(fail)?;
                require(candidate.changed(), "Unexpected no-op recovery patch")?;
                session.commit_source_edit(candidate, group).map_err(fail)?;
            } else {
                let candidate = session.prepare(patch).map_err(fail)?;
                require(candidate.changed(), "Unexpected no-op recovery patch")?;
                session.commit(candidate, origin, group).map_err(fail)?;
            }
            true
        }
        Mutation::Undo => {
            require(session.undo().map_err(fail)?, "Invalid recovery undo")?;
            true
        }
        Mutation::Redo => {
            require(session.redo().map_err(fail)?, "Invalid recovery redo")?;
            true
        }
        Mutation::Selection { selection } => {
            session.set_selection(selection).map_err(fail)?;
            false
        }
        Mutation::Saved { disk: saved } => {
            validate_disk(&saved)?;
            require(
                saved.root == disk.root,
                "Recovery Save changed project identity",
            )?;
            *disk = saved;
            false
        }
    };
    require(
        record.revision
            == record
                .base
                .checked_add(u64::from(changed))
                .ok_or("Recovery revision exhausted")?
            && record.revision == session.revision().0,
        "Recovery revision mismatch",
    )?;
    validate_session(session)
}
fn load(directory: &Path, id: &str) -> Result<Recovered> {
    let generations = generations(directory)?;
    let mut chosen = None;
    let mut interrupted_header = false;
    for (number, path) in generations.iter().rev() {
        require(
            fs::metadata(path).map_err(fail)?.len() <= MAX_GENERATION,
            "Recovery generation exceeds byte budget",
        )?;
        let mut file = File::open(path).map_err(fail)?;
        let mut consumed = 0;
        match read_frame(&mut file, &[0; 32], &mut consumed)? {
            FrameRead::Complete(payload, digest) => {
                let header: Header = serde_json::from_slice(&payload).map_err(fail)?;
                require(
                    header.schema == 1
                        && header.id == id
                        && header.generation == *number
                        && header.previous.len() <= 1
                        && header.previous.iter().all(|p| p.generation < *number),
                    "Invalid recovery checkpoint identity",
                )?;
                validate_disk(&header.disk)?;
                chosen = Some((header, file, digest, consumed));
                break;
            }
            FrameRead::End | FrameRead::Partial => {
                interrupted_header = true;
            }
        }
    }
    let (header, mut file, mut digest, mut consumed) =
        chosen.ok_or("No complete recovery checkpoint; data left in place")?;
    let mut replay = header.checkpoint.replay().map_err(fail)?;
    validate_session(replay.session())?;
    let mut disk = header.disk;
    let mut partial = false;
    let mut count = 0;
    loop {
        match read_frame(&mut file, &digest, &mut consumed)? {
            FrameRead::Complete(payload, next) => {
                count += 1;
                require(
                    count <= ROTATE_EVENTS + 1,
                    "Recovery event retention exceeds its budget",
                )?;
                apply(
                    serde_json::from_slice(&payload).map_err(fail)?,
                    replay.session_mut(),
                    &mut disk,
                )?;
                digest = next;
            }
            FrameRead::Partial => {
                partial = true;
                break;
            }
            FrameRead::End => break,
        }
    }
    Ok(Recovered { session: replay.finish(), disk, warning: (partial || interrupted_header).then(||
        "An interrupted final recovery record was excluded. The last complete verified state was restored; the original recovery bytes are retained.".into()) })
}

#[cfg(test)]
mod tests;
