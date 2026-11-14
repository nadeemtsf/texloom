//! Project ZIPs and explicit encoding conversion; originals are read-only inputs.
use crate::snapshot::{self, Fingerprint};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read, Write},
    path::Path,
};
pub type Files = BTreeMap<String, Vec<u8>>;
type Result<T> = std::result::Result<T, String>;
const MAX_BYTES: u64 = 60 * 1024 * 1024;
pub const PROFILE: &str = "texloom-project.json";
fn fail(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn portable_name(name: &str) -> Result<()> {
    snapshot::relative(name)?;
    for part in name.split('/') {
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if part.len() > 200
            || part.trim_end_matches(['.', ' ']) != part
            || part
                .chars()
                .any(|c| c.is_control() || "<>\"|?*".contains(c))
            || [
                "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
                "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
                "LPT9",
            ]
            .contains(&stem.as_str())
        {
            return Err("A project filename is not portable to Windows".into());
        }
        if part.starts_with('.') {
            return Err("Hidden project files are not imported or exported".into());
        }
    }
    Ok(())
}
fn allowed(name: &str) -> bool {
    name == PROFILE
        || name.ends_with(&format!("/{PROFILE}"))
        || snapshot::is_source(Path::new(name))
        || snapshot::is_asset(Path::new(name))
}
fn generated(name: &str) -> bool {
    name.starts_with("__MACOSX/")
        || name.split('/').any(|s| s == ".DS_Store")
        || [
            "aux", "log", "toc", "out", "bcf", "blg", "lof", "lot", "nav", "snm",
        ]
        .contains(
            &Path::new(name)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or(""),
        )
        || name.ends_with(".synctex.gz")
        || name.ends_with(".run.xml")
}
pub fn read_archive(path: &Path) -> Result<Files> {
    snapshot::check_plain(path)?;
    if fs::metadata(path).map_err(fail)?.len() > MAX_BYTES {
        return Err("Project ZIP exceeds 60 MiB".into());
    }
    read_archive_bytes(&fs::read(path).map_err(fail)?)
}
pub fn read_archive_bytes(bytes: &[u8]) -> Result<Files> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err("Project ZIP exceeds 60 MiB".into());
    }
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(fail)?;
    if archive.len() > 1200 {
        return Err("Project ZIP has too many entries".into());
    }
    let mut files = Files::new();
    let mut seen = BTreeSet::new();
    let mut total = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(fail)?;
        let name = std::str::from_utf8(entry.name_raw())
            .map_err(|_| "ZIP filenames must use UTF-8")?
            .to_string();
        let trimmed = name.trim_end_matches('/');
        if generated(trimmed) {
            continue;
        }
        portable_name(trimmed)?;
        if !seen.insert(trimmed.to_lowercase()) {
            return Err("ZIP contains duplicate or case-colliding names".into());
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err("ZIP links are not supported".into());
        }
        if entry.is_dir() {
            continue;
        }
        if !allowed(&name) {
            return Err(format!("Unsupported project ZIP entry: {name}"));
        }
        if ![
            zip::CompressionMethod::Stored,
            zip::CompressionMethod::Deflated,
        ]
        .contains(&entry.compression())
        {
            return Err("Use a stored or deflated project ZIP".into());
        }
        total = total
            .checked_add(entry.size())
            .ok_or("Project ZIP size overflow")?;
        if total > MAX_BYTES
            || files.len() >= 600
            || snapshot::is_source(Path::new(&name)) && entry.size() > 4_000_000
        {
            return Err("Project ZIP exceeds source/file/size limits".into());
        }
        let declared = entry.size();
        let mut data = Vec::new();
        Read::by_ref(&mut entry)
            .take(declared + 1)
            .read_to_end(&mut data)
            .map_err(fail)?;
        if data.len() as u64 != declared {
            return Err("ZIP entry length does not match its declaration".into());
        }
        files.insert(name, data);
    }
    let names = files
        .keys()
        .map(|n| n.to_lowercase())
        .collect::<BTreeSet<_>>();
    for name in &names {
        for parent in Path::new(name).ancestors().skip(1) {
            if names.contains(&parent.to_string_lossy().replace('\\', "/")) {
                return Err("ZIP contains a file where another entry needs a folder".into());
            }
        }
    }
    // Common ZIP packaging adds one or more enclosing directories. Strip only
    // a shared directory when no file exists at that level.
    loop {
        if files.is_empty() || files.keys().any(|n| !n.contains('/')) {
            break;
        }
        let prefix = files
            .keys()
            .next()
            .unwrap()
            .split('/')
            .next()
            .unwrap()
            .to_string()
            + "/";
        if !files.keys().all(|n| n.starts_with(&prefix)) {
            break;
        }
        files = files
            .into_iter()
            .map(|(n, b)| (n[prefix.len()..].to_string(), b))
            .collect();
    }
    if !files.keys().any(|n| n.ends_with(".tex")) {
        return Err("Project ZIP has no .tex source".into());
    }
    Ok(files)
}
pub fn archive_bytes(files: &Files) -> Result<Vec<u8>> {
    if files.len() > 600 || files.values().map(|b| b.len() as u64).sum::<u64>() > MAX_BYTES {
        return Err("Project export exceeds 600 files or 60 MiB".into());
    }
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut seen = BTreeSet::new();
    for (name, data) in files {
        portable_name(name)?;
        if !allowed(name) || !seen.insert(name.to_lowercase()) {
            return Err("Invalid project export entry".into());
        }
        writer
            .start_file(
                name,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated),
            )
            .map_err(fail)?;
        writer.write_all(data).map_err(fail)?;
    }
    let bytes = writer.finish().map_err(fail)?.into_inner();
    if read_archive_bytes(&bytes)? != *files {
        return Err("Project ZIP verification did not reproduce the source bytes".into());
    }
    Ok(bytes)
}
pub fn fingerprint(files: &Files) -> String {
    let mut text = String::new();
    for (name, data) in files {
        text.push_str(&format!(
            "{}:{name}:{}:{}\n",
            name.len(),
            data.len(),
            Fingerprint::bytes(data).sha256
        ))
    }
    Fingerprint::bytes(text.as_bytes()).sha256
}
pub fn write_new_tree(root: &Path, files: &Files) -> Result<()> {
    if files.len() > 600
        || files.values().map(|b| b.len() as u64).sum::<u64>() > MAX_BYTES
        || files
            .iter()
            .any(|(n, b)| snapshot::is_source(Path::new(n)) && b.len() > 4_000_000)
    {
        return Err("Converted/imported project exceeds its limits".into());
    }
    if root.exists() {
        return Err("A new project folder is required".into());
    }
    fs::create_dir(root).map_err(fail)?;
    for (name, data) in files {
        portable_name(name)?;
        let target = root.join(snapshot::relative(name)?);
        let parent = target.parent().ok_or("Missing project parent")?;
        fs::create_dir_all(parent).map_err(fail)?;
        let mut f = fs::File::create_new(&target).map_err(fail)?;
        f.write_all(data).map_err(fail)?;
        f.sync_all().map_err(fail)?;
    }
    Ok(())
}
pub fn utf8(bytes: &[u8]) -> bool {
    std::str::from_utf8(bytes).is_ok() && !bytes.contains(&0)
}
pub fn detected(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xff, 0xfe]) {
        Some("utf-16le")
    } else if bytes.starts_with(&[0xfe, 0xff]) {
        Some("utf-16be")
    } else if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        Some("utf-8")
    } else {
        None
    }
}
pub fn decode(bytes: &[u8], label: &str) -> Result<String> {
    if let Some(actual) = detected(bytes) {
        if actual != label {
            return Err("Choose the encoding indicated by the file's byte-order mark".into());
        }
    }
    let bytes = match detected(bytes) {
        Some("utf-8") => &bytes[3..],
        Some(_) => &bytes[2..],
        None => bytes,
    };
    if label == "iso-8859-1" {
        if bytes.contains(&0) {
            return Err("The selected encoding leaves NUL bytes in the source".into());
        }
        return Ok(bytes.iter().map(|b| char::from(*b)).collect());
    }
    if ![
        "utf-8",
        "windows-1252",
        "windows-1256",
        "windows-1255",
        "utf-16le",
        "utf-16be",
        "shift_jis",
        "gb18030",
        "big5",
    ]
    .contains(&label)
    {
        return Err("Choose a supported source encoding".into());
    }
    let encoding =
        encoding_rs::Encoding::for_label(label.as_bytes()).ok_or("Unsupported source encoding")?;
    let (text, errors) = encoding.decode_without_bom_handling(bytes);
    if errors || text.contains('\0') {
        return Err("That encoding cannot decode the file without replacement characters".into());
    }
    Ok(text.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn raw_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes) in entries {
            writer
                .start_file(
                    *name,
                    zip::write::SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored),
                )
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }
    #[test]
    fn actual_zip_entries_refuse_traversal_collisions_ancestors_links_and_corruption() {
        for entries in [
            vec![("../main.tex", b"Text".as_slice())],
            vec![("main.tex", b"A".as_slice()), ("MAIN.tex", b"B".as_slice())],
            vec![
                ("main.tex", b"A".as_slice()),
                ("main.tex/part.tex", b"B".as_slice()),
            ],
        ] {
            assert!(read_archive_bytes(&raw_zip(&entries)).is_err());
        }
        let mut bytes = raw_zip(&[("main.tex", b"Unique bytes")]);
        let offset = bytes
            .windows(12)
            .position(|s| s == b"Unique bytes")
            .unwrap();
        bytes[offset] ^= 1;
        assert!(read_archive_bytes(&bytes).is_err(), "CRC must be checked");
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .add_symlink(
                "main.tex",
                "../outside.tex",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        assert!(read_archive_bytes(&writer.finish().unwrap().into_inner()).is_err());
    }
    #[test]
    fn shared_zip_wrapper_is_removed_but_generated_outputs_are_not_project_sources() {
        let bytes = raw_zip(&[
            ("Outer/Inner/main.tex", b"Source"),
            ("Outer/Inner/image.png", &[0, 255]),
            ("Outer/Inner/main.aux", b"Generated"),
        ]);
        let files = read_archive_bytes(&bytes).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files["main.tex"], b"Source");
        assert_eq!(files["image.png"], [0, 255]);
    }
    #[test]
    fn zip_roundtrips_unsaved_unicode_bom_crlf_binary_assets_and_profile() {
        let files = Files::from([
            ("main.tex".into(), b"\xef\xbb\xbfDocument\r\n".to_vec()),
            ("chapters/name.tex".into(), "café\r\n".as_bytes().to_vec()),
            ("image.png".into(), vec![0, 1, 255]),
            (PROFILE.into(), b"{}".to_vec()),
        ]);
        assert_eq!(
            read_archive_bytes(&archive_bytes(&files).unwrap()).unwrap(),
            files
        );
    }
    #[test]
    fn zip_refuses_escaping_alias_reserved_hidden_and_case_colliding_paths() {
        for name in [
            "../main.tex",
            "/main.tex",
            "a/../main.tex",
            "C:/main.tex",
            "a\\main.tex",
            "CON.tex",
            "main.tex ",
            ".env",
        ] {
            assert!(portable_name(name).is_err(), "{name}");
        }
        let files = Files::from([("main.tex".into(), vec![]), ("MAIN.TEX".into(), vec![])]);
        assert!(archive_bytes(&files).is_err());
    }
    #[test]
    fn encoding_conversion_is_explicit_and_refuses_malformed_unicode() {
        assert_eq!(decode(b"caf\xe9", "windows-1252").unwrap(), "café");
        assert_eq!(decode(&[0xff, 0xfe, b'A', 0], "utf-16le").unwrap(), "A");
        assert!(decode(&[0xff, 0xfe, b'A'], "utf-16le").is_err());
        assert!(decode(&[0xff, 0xfe, b'A', 0], "windows-1252").is_err());
        assert!(!utf8(&[b'A', 0]));
    }
}
