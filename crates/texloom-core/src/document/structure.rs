//! Read-only, lossless first slice of the visual editor's source projection.
//! A lexical candidate is never permission to edit or evidence of PDF identity.
use super::{EditError, EditResult, Revision, Session};
use std::{ops::Range, sync::Arc};

const MAX_STEPS: usize = 200_000;
const MAX_SCOPES: usize = 128;
const MAX_REGIONS: usize = 10_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationKind {
    ReplaceText,
    DeleteContent,
    ReorderContent,
    AdjustSpacing,
    ApplyFormatting,
    InsertText,
    SplitParagraph,
    JoinParagraphs,
    RemovePositioning,
    DeleteAsset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpaqueReason {
    OutsideBody,
    UnsupportedSyntax,
    IncompleteStructure,
    DynamicSyntax,
    BudgetExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionKind {
    PlainParagraph,
    ParagraphBreak,
    Trivia,
    Opaque(OpaqueReason),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnavailableReason {
    OpaqueSource,
    NotContent,
    NoLocalParameter,
    NotPositioned,
    AssetLifecycleRequired,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readiness {
    /// Still needs an operation-specific planner, context/dependency checks,
    /// source/PDF identity and the applicable verification. Not a permission.
    CandidateRequiringPlanAndProof,
    Unavailable(UnavailableReason),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRegion {
    range: Range<usize>,
    kind: RegionKind,
}
impl SourceRegion {
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub fn kind(&self) -> RegionKind {
        self.kind
    }
    pub fn readiness(&self, operation: OperationKind) -> Readiness {
        use UnavailableReason::*;
        if operation == OperationKind::DeleteAsset {
            return Readiness::Unavailable(AssetLifecycleRequired);
        }
        match self.kind {
            RegionKind::Opaque(_) => Readiness::Unavailable(OpaqueSource),
            RegionKind::Trivia | RegionKind::ParagraphBreak => Readiness::Unavailable(NotContent),
            RegionKind::PlainParagraph => match operation {
                OperationKind::AdjustSpacing => Readiness::Unavailable(NoLocalParameter),
                OperationKind::RemovePositioning => Readiness::Unavailable(NotPositioned),
                _ => Readiness::CandidateRequiringPlanAndProof,
            },
        }
    }
}

pub struct SourceProjection<'a> {
    owner: Arc<()>,
    file: &'a str,
    revision: Revision,
    source: &'a str,
    regions: Vec<SourceRegion>,
}
impl<'a> SourceProjection<'a> {
    pub fn file(&self) -> &str {
        self.file
    }
    pub fn revision(&self) -> Revision {
        self.revision
    }
    pub fn source(&self) -> &'a str {
        self.source
    }
    pub fn regions(&self) -> &[SourceRegion] {
        &self.regions
    }
    pub fn text(&self, index: usize) -> Option<&'a str> {
        self.regions
            .get(index)
            .and_then(|r| self.source.get(r.range.clone()))
    }
    /// Snapshot-local identity, deliberately not serialized or accepted by IPC.
    pub fn entity_reference(&self, index: usize) -> Option<EntityReference> {
        let region = self.regions.get(index)?;
        if region.kind != RegionKind::PlainParagraph {
            return None;
        }
        Some(EntityReference {
            owner: self.owner.clone(),
            file: self.file.into(),
            revision: self.revision,
            range: region.range.clone(),
            expected: self.source[region.range.clone()].into(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct EntityReference {
    owner: Arc<()>,
    file: String,
    revision: Revision,
    range: Range<usize>,
    expected: String,
}

pub struct ResolvedEntity<'a> {
    file: &'a str,
    range: Range<usize>,
    text: &'a str,
}
impl ResolvedEntity<'_> {
    pub fn file(&self) -> &str {
        self.file
    }
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
    pub fn text(&self) -> &str {
        self.text
    }
}

impl Session {
    pub fn source_projection(&self, file: &str) -> EditResult<SourceProjection<'_>> {
        let (file, source) = self
            .sources
            .get_key_value(file)
            .ok_or_else(|| EditError::MissingFile(file.into()))?;
        Ok(SourceProjection {
            owner: self.owner.clone(),
            file,
            revision: self.revision,
            source,
            regions: project(source),
        })
    }
    /// Resolves source identity only; it grants no mutation or render authority.
    pub fn resolve_entity_reference(
        &self,
        reference: &EntityReference,
    ) -> EditResult<ResolvedEntity<'_>> {
        if !Arc::ptr_eq(&self.owner, &reference.owner) {
            return Err(EditError::ForeignSession);
        }
        if self.revision != reference.revision {
            return Err(EditError::StaleRevision);
        }
        let (file, source) = self
            .sources
            .get_key_value(&reference.file)
            .ok_or_else(|| EditError::MissingFile(reference.file.clone()))?;
        let text = source
            .get(reference.range.clone())
            .ok_or(EditError::InvalidRange)?;
        if text != reference.expected {
            return Err(EditError::ExpectedTextMismatch);
        }
        Ok(ResolvedEntity {
            file,
            range: reference.range.clone(),
            text,
        })
    }
}

fn opaque(source: &str, reason: OpaqueReason) -> Vec<SourceRegion> {
    if source.is_empty() {
        Vec::new()
    } else {
        vec![SourceRegion {
            range: 0..source.len(),
            kind: RegionKind::Opaque(reason),
        }]
    }
}

struct Boundary {
    range: Range<usize>,
    kind: RegionKind,
}

struct Envelope {
    body: Range<usize>,
    blocked: Vec<Range<usize>>,
    boundaries: Vec<Boundary>,
}

/// Bounded lexical forms in the preamble only. This preserves their bytes and
/// does not establish package/font semantics, compilation or visual authority.
/// Unknown options, computed arguments and general macro definitions stay opaque.
fn preamble_form(source: &str, command: &str, end: usize) -> Option<Result<usize, OpaqueReason>> {
    fn argument(source: &str, at: usize) -> Result<(&str, usize), OpaqueReason> {
        let (range, end) =
            crate::argument(source, at).map_err(|_| OpaqueReason::IncompleteStructure)?;
        Ok((&source[range], end))
    }
    fn font_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 128
            && name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b' ' | b'-'))
    }
    let parsed = || -> Result<usize, OpaqueReason> {
        let unsupported = OpaqueReason::UnsupportedSyntax;
        match command {
            "\\setmainfont" => {
                let (name, after) = argument(source, end)?;
                if !font_name(name) {
                    return Err(unsupported);
                }
                // fontspec also permits trailing options; they are outside this subset.
                if source[crate::skip_space(source, after)..].starts_with('[') {
                    return Err(unsupported);
                }
                Ok(after)
            }
            "\\setdefaultlanguage" => {
                let (language, after) = argument(source, end)?;
                if !["english", "arabic", "hebrew"].contains(&language) {
                    return Err(unsupported);
                }
                Ok(after)
            }
            "\\newfontfamily" => {
                let at = crate::skip_space(source, end);
                if !source[at..].starts_with('\\') {
                    return Err(unsupported);
                }
                let (switch, after_switch) = crate::command(source, at);
                let options = match switch {
                    "\\arabicfont" => "[Script=Arabic]",
                    "\\hebrewfont" => "[Script=Hebrew]",
                    _ => return Err(unsupported),
                };
                let at = crate::skip_space(source, after_switch);
                if !source[at..].starts_with(options) {
                    return Err(unsupported);
                }
                let (name, after) = argument(source, at + options.len())?;
                if !font_name(name) || source[crate::skip_space(source, after)..].starts_with('[') {
                    return Err(unsupported);
                }
                Ok(after)
            }
            "\\setlength" => {
                let (register, after) = argument(source, end)?;
                if !["\\parindent", "\\parskip"].contains(&register) {
                    return Err(unsupported);
                }
                let (dimension, after) = argument(source, after)?;
                if dimension.len() > 32 {
                    return Err(unsupported);
                }
                let number = ["pt", "bp", "mm", "cm"]
                    .iter()
                    .find_map(|unit| dimension.strip_suffix(unit));
                let valid = number.is_some_and(|s| {
                    !s.is_empty()
                        && s.bytes().all(|c| c.is_ascii_digit() || c == b'.')
                        && s.bytes().filter(|&c| c == b'.').count() <= 1
                        && s.parse::<f64>().is_ok_and(f64::is_finite)
                });
                if !valid {
                    return Err(unsupported);
                }
                Ok(after)
            }
            _ => unreachable!(),
        }
    };
    [
        "\\setmainfont",
        "\\setdefaultlanguage",
        "\\newfontfamily",
        "\\setlength",
    ]
    .contains(&command)
    .then(parsed)
}
// Recognize standalone flow commands without consuming the surrounding prose.
fn standalone_boundary(
    source: &str,
    command: &str,
    start: usize,
    after_command: usize,
) -> Option<Range<usize>> {
    let line_start = source[..start].rfind('\n').map_or(0, |offset| offset + 1);
    if !source[line_start..start].bytes().all(|b| matches!(b, b' ' | b'\t')) {
        return None;
    }
    let mut lines = source[after_command..].split('\n');
    let remainder = lines.next()?;
    let line_end = after_command + remainder.len();
    if matches!(command, "\\newpage" | "\\clearpage" | "\\par") {
        return remainder.chars().all(ascii_space).then_some(line_start..line_end);
    }

    fn literal_title(title: &str) -> bool {
        !title.trim().is_empty()
            && !title.chars().any(|c| {
                c.is_control() || matches!(c, '\\' | '{' | '}' | '%' | '$' | '&' | '#' | '_' | '^' | '~')
            })
    }

    let starred = remainder.strip_prefix('*');
    let mut arguments = starred.unwrap_or(remainder).trim_start_matches(ascii_space);
    if starred.is_none() {
        if let Some(optional) = arguments.strip_prefix('[') {
            let (short, rest) = optional.split_once(']')?;
            if !literal_title(short) || short.contains('[') {
                return None;
            }
            arguments = rest.trim_start_matches(ascii_space);
        }
    }
    let (title, suffix) = arguments.strip_prefix('{')?.split_once('}')?;
    if !literal_title(title) {
        return None;
    }
    fn literal_label(line: &str) -> bool {
        line.trim_matches(ascii_space)
            .strip_prefix("\\label")
            .and_then(|text| text.trim_start_matches(ascii_space).strip_prefix('{'))
            .and_then(|text| text.strip_suffix('}'))
            .is_some_and(|key| {
                !key.is_empty()
                    && key.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'-' | b'_' | b'.' | b'/')
                    })
            })
    }

    let suffix = suffix.trim_matches(ascii_space);
    if !suffix.is_empty() {
        return literal_label(suffix).then_some(line_start..line_end);
    }
    // An immediately following literal label belongs to this heading. Leave
    // anything else to the normal scanner, including an invalid label.
    let mut boundary_end = line_end;
    if let Some(next) = lines.next() {
        if literal_label(next) {
            boundary_end += 1 + next.len();
        }
    }
    Some(line_start..boundary_end)
}

fn envelope(source: &str) -> Result<Envelope, OpaqueReason> {
    let mut i = 0;
    let mut steps = 0;
    let mut groups = Vec::new();
    let mut envs: Vec<(&str, usize)> = Vec::new();
    let mut blocked = Vec::new();
    let mut boundaries = Vec::new();
    let mut begin = None;
    let mut finish = None;
    while i < source.len() {
        steps += 1;
        if steps > MAX_STEPS {
            return Err(OpaqueReason::BudgetExceeded);
        }
        match source.as_bytes()[i] {
            b'%' => {
                let end = crate::comment_end(source, i);
                blocked.push(i..end);
                i = end;
                continue;
            }
            b'\\' => {
                let (command, end) = crate::command(source, i);
                if [
                    "\\catcode",
                    "\\def",
                    "\\gdef",
                    "\\edef",
                    "\\xdef",
                    "\\let",
                    "\\futurelet",
                    "\\newcommand",
                    "\\renewcommand",
                    "\\providecommand",
                    "\\DeclareRobustCommand",
                    "\\csname",
                    "\\endcsname",
                    "\\expandafter",
                    "\\else",
                    "\\fi",
                    "\\makeatletter",
                    "\\ExplSyntaxOn",
                    "\\directlua",
                    "\\endinput",
                ]
                .contains(&command)
                    || command.starts_with("\\if")
                {
                    return Err(OpaqueReason::DynamicSyntax);
                }
                if begin.is_none() && groups.is_empty() && envs.is_empty() {
                    if let Some(parsed) = preamble_form(source, command, end) {
                        let after = parsed?;
                        blocked.push(i..after);
                        i = after;
                        continue;
                    }
                }
                // Exact kernel spellings emitted by escape_text. Recognize
                // only their empty terminator group, not arbitrary arguments
                // or command expansion. They still need context/render proof.
                if ["\\textbackslash", "\\textasciitilde", "\\textasciicircum"].contains(&command) {
                    if source[end..].starts_with("{}") {
                        i = end + 2;
                        continue;
                    }
                    return Err(OpaqueReason::UnsupportedSyntax);
                }
                if let Some(after) = crate::skip_literal(source, command, end) {
                    blocked.push(i..after);
                    i = after;
                    continue;
                }
                if command == "\\begin" || command == "\\end" {
                    let (arg, after) = crate::argument(source, end)
                        .map_err(|_| OpaqueReason::IncompleteStructure)?;
                    let name = &source[arg];
                    if name.is_empty()
                        || !name
                            .bytes()
                            .all(|c| c.is_ascii_alphanumeric() || c == b'*' || c == b'@')
                    {
                        return Err(OpaqueReason::DynamicSyntax);
                    }
                    if !groups.is_empty() {
                        return Err(OpaqueReason::UnsupportedSyntax);
                    }
                    if command == "\\begin" {
                        if envs.len() >= MAX_SCOPES {
                            return Err(OpaqueReason::BudgetExceeded);
                        }
                        if name == "document" {
                            if begin.is_some() || !envs.is_empty() {
                                return Err(OpaqueReason::IncompleteStructure);
                            }
                            begin = Some(after);
                        }
                        envs.push((name, i));
                    } else {
                        let (open, start) = envs.pop().ok_or(OpaqueReason::IncompleteStructure)?;
                        if open != name {
                            return Err(OpaqueReason::IncompleteStructure);
                        }
                        if name == "document" {
                            finish = Some(i);
                        } else {
                            blocked.push(start..after);
                        }
                    }
                    i = after;
                    continue;
                }
                if matches!(
                    command,
                    "\\part" | "\\chapter" | "\\section" | "\\subsection" | "\\subsubsection"
                        | "\\newpage" | "\\clearpage" | "\\par"
                )
                    && finish.is_none()
                    && groups.is_empty()
                    && matches!(envs.as_slice(), [("document", _)])
                {
                    let boundary = standalone_boundary(source, command, i, end)
                        .ok_or(OpaqueReason::UnsupportedSyntax)?;
                    i = boundary.end;
                    let kind = if command == "\\par" {
                        RegionKind::ParagraphBreak
                    } else {
                        RegionKind::Opaque(OpaqueReason::UnsupportedSyntax)
                    };
                    boundaries.push(Boundary { range: boundary, kind });
                    continue;
                }
                // First slice recognizes literal prose, not markup expansion.
                if !["\\%", "\\&", "\\$", "\\#", "\\_", "\\{", "\\}", "\\ "].contains(&command) {
                    if ["\\[", "\\]", "\\(", "\\)"].contains(&command) {
                        return Err(OpaqueReason::UnsupportedSyntax);
                    }
                    // An unknown command can alter context after its own line.
                    // Do not infer safe following paragraphs from whitespace.
                    if ![
                        "\\documentclass",
                        "\\usepackage",
                        "\\pagestyle",
                        "\\textbf",
                        "\\textit",
                        "\\textsl",
                        "\\textsc",
                        "\\textsf",
                        "\\textrm",
                        "\\texttt",
                        "\\emph",
                        "\\underline",
                        "\\mbox",
                        "\\textcolor",
                    ]
                    .contains(&command)
                    {
                        return Err(OpaqueReason::DynamicSyntax);
                    }
                    blocked.push(i..end);
                }
                i = end;
                continue;
            }
            b'{' => {
                if groups.len() >= MAX_SCOPES {
                    return Err(OpaqueReason::BudgetExceeded);
                }
                groups.push(i);
            }
            b'}' => {
                let start = groups.pop().ok_or(OpaqueReason::IncompleteStructure)?;
                blocked.push(start..i + 1);
            }
            b'$' => return Err(OpaqueReason::UnsupportedSyntax),
            // TeX's ^^ preprocessing can manufacture control characters before
            // tokenization. This lexical subset must not guess its meaning.
            b'^' => return Err(OpaqueReason::DynamicSyntax),
            b'&' | b'#' | b'_' | b'~' => blocked.push(i..i + 1),
            _ => (),
        }
        i += source[i..].chars().next().unwrap().len_utf8();
    }
    if !groups.is_empty() || !envs.is_empty() {
        return Err(OpaqueReason::IncompleteStructure);
    }
    let body = match (begin, finish) {
        (Some(a), Some(b)) if a <= b => a..b,
        _ => return Err(OpaqueReason::IncompleteStructure),
    };
    blocked.sort_unstable_by_key(|r| r.start);
    let mut merged: Vec<Range<usize>> = Vec::new();
    for range in blocked {
        if let Some(last) = merged.last_mut().filter(|last| range.start <= last.end) {
            last.end = last.end.max(range.end);
        } else {
            merged.push(range);
        }
    }
    Ok(Envelope {
        body,
        blocked: merged,
        boundaries,
    })
}

fn ascii_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n')
}
fn append(regions: &mut Vec<SourceRegion>, source: &str, range: Range<usize>, kind: RegionKind) {
    if range.is_empty() {
        return;
    }
    let kind = if source[range.clone()].chars().all(ascii_space) {
        RegionKind::Trivia
    } else {
        kind
    };
    if let Some(last) = regions
        .last_mut()
        .filter(|last| {
            last.range.end == range.start
                && (last.kind == kind || matches!((last.kind, kind),
                    (RegionKind::Trivia, RegionKind::ParagraphBreak)
                    | (RegionKind::ParagraphBreak, RegionKind::Trivia)))
        })
    {
        last.range.end = range.end;
        if kind == RegionKind::ParagraphBreak {
            last.kind = kind;
        }
    } else {
        regions.push(SourceRegion { range, kind });
    }
}

fn project(source: &str) -> Vec<SourceRegion> {
    let view = match envelope(source) {
        Ok(view) => view,
        Err(reason) => return opaque(source, reason),
    };
    let mut regions = Vec::new();
    append(
        &mut regions,
        source,
        0..view.body.start,
        RegionKind::Opaque(OpaqueReason::OutsideBody),
    );
    let mut chunk = view.body.start;
    let mut offset = chunk;
    let mut blocked_index = 0;
    let mut chunks = Vec::new();
    let mut boundaries = view.boundaries.into_iter().peekable();
    for line in source[view.body.clone()].split_inclusive('\n') {
        let end = offset + line.len();
        if let Some(boundary) = boundaries.peek().filter(|boundary| boundary.range.start <= offset) {
            if boundary.range.start == offset && chunk < offset {
                chunks.push((chunk..offset, None));
                chunk = offset;
            }
            if boundary.range.end <= end {
                chunks.push((chunk..end, Some(boundary.kind)));
                chunk = end;
                boundaries.next();
            }
        } else if line.chars().all(ascii_space) {
            if chunk < offset {
                chunks.push((chunk..offset, None));
            }
            chunks.push((offset..end, None));
            chunk = end;
        }
        offset = end;
        if chunks.len() > MAX_REGIONS {
            return opaque(source, OpaqueReason::BudgetExceeded);
        }
    }
    if chunk < view.body.end {
        chunks.push((chunk..view.body.end, None));
    }
    for (raw, boundary_kind) in chunks {
        let text = &source[raw.clone()];
        let start = raw.start + text.len() - text.trim_start_matches(ascii_space).len();
        let end = (raw.start + text.trim_end_matches(ascii_space).len()).max(start);
        append(&mut regions, source, raw.start..start, RegionKind::Trivia);
        if start < end {
            while blocked_index < view.blocked.len() && view.blocked[blocked_index].end <= start {
                blocked_index += 1;
            }
            let blocked = view
                .blocked
                .get(blocked_index)
                .is_some_and(|r| r.start < end);
            let kind = if blocked {
                RegionKind::Opaque(OpaqueReason::UnsupportedSyntax)
            } else {
                boundary_kind.unwrap_or(RegionKind::PlainParagraph)
            };
            append(&mut regions, source, start..end, kind);
        }
        append(&mut regions, source, end..raw.end, RegionKind::Trivia);
        if regions.len() > MAX_REGIONS {
            return opaque(source, OpaqueReason::BudgetExceeded);
        }
    }
    append(
        &mut regions,
        source,
        view.body.end..source.len(),
        RegionKind::Opaque(OpaqueReason::OutsideBody),
    );
    if regions.len() > MAX_REGIONS {
        return opaque(source, OpaqueReason::BudgetExceeded);
    }
    regions
}
