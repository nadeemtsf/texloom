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
            RegionKind::Trivia => Readiness::Unavailable(NotContent),
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

struct Envelope {
    body: Range<usize>,
    blocked: Vec<Range<usize>>,
}
fn envelope(source: &str) -> Result<Envelope, OpaqueReason> {
    let mut i = 0;
    let mut steps = 0;
    let mut groups = Vec::new();
    let mut envs: Vec<(&str, usize)> = Vec::new();
    let mut blocked = Vec::new();
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
        .filter(|last| last.kind == kind && last.range.end == range.start)
    {
        last.range.end = range.end;
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
    for line in source[view.body.clone()].split_inclusive('\n') {
        let end = offset + line.len();
        if line.chars().all(ascii_space) {
            if chunk < offset {
                chunks.push(chunk..offset);
            }
            chunks.push(offset..end);
            chunk = end;
        }
        offset = end;
        if chunks.len() > MAX_REGIONS {
            return opaque(source, OpaqueReason::BudgetExceeded);
        }
    }
    if chunk < view.body.end {
        chunks.push(chunk..view.body.end);
    }
    for raw in chunks {
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
            append(
                &mut regions,
                source,
                start..end,
                if blocked {
                    RegionKind::Opaque(OpaqueReason::UnsupportedSyntax)
                } else {
                    RegionKind::PlainParagraph
                },
            );
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
