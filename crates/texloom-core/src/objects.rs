//! Source-preserving text objects and deliberately conservative selection units.
use std::{collections::HashSet, ops::Range};
use super::{argument, command, comment_end, document_bounds, has_package, number, skip_literal, skip_space, validate_fragment, Result};

#[derive(Clone, Debug)]
pub struct TextObject {
    pub id: String,
    pub dx: f64,
    pub dy: f64,
    pub content: String,
    pub range: Range<usize>,
    pub arguments: Vec<Range<usize>>,
    pub line: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SourceUnit {
    pub range: Range<usize>,
    pub first_line: usize,
    pub last_line: usize,
    pub kind: &'static str,
    pub content: String,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}

pub fn text_objects(source: &str) -> Result<Vec<TextObject>> {
    let mut objects = Vec::new();
    let mut identifiers = HashSet::new();
    let mut line = 1;
    let mut counted_to = 0;
    let mut i = 0;
    while i < source.len() {
        if source.as_bytes()[i] == b'%' { i = comment_end(source, i); continue; }
        if source.as_bytes()[i] != b'\\' { i += 1; continue; }
        let (cmd, end) = command(source, i);
        if let Some(after) = skip_literal(source, cmd, end) { i = after; continue; }
        if cmd != "\\TexloomObject" { i = end; continue; }
        let mut cursor = end;
        let mut args = Vec::new();
        for _ in 0..4 { let (range, next) = argument(source, cursor)?; args.push(range); cursor = next; }
        let id = source[args[0].clone()].trim();
        if !valid_id(id) { return Err("An object has an invalid identifier".into()); }
        if !identifiers.insert(id) { return Err("Object identifiers must be unique in each source file".into()); }
        let dimension = |n: usize| -> Result<f64> {
            let value = source[args[n].clone()].trim().parse::<f64>().map_err(|_| "Invalid object offset")?;
            number(value, false)?; Ok(value)
        };
        let content = source[args[3].clone()].to_string();
        if content.contains("\\TexloomObject") { return Err("Nested text objects are not supported".into()); }
        line += source.as_bytes()[counted_to..i].iter().filter(|&&c| c == b'\n').count();
        counted_to = i;
        objects.push(TextObject { id: id.into(), dx: dimension(1)?, dy: dimension(2)?, content,
            range: i..cursor, arguments: args, line });
        i = cursor;
    }
    Ok(objects)
}

pub(crate) fn simple_definitions(source: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let limit = document_bounds(source).map(|(end, _, _)| end).unwrap_or(0);
    let mut i = 0;
    while i < limit {
        if source.as_bytes()[i] == b'%' { i = comment_end(source, i); continue; }
        if source.as_bytes()[i] != b'\\' { i += 1; continue; }
        let (cmd, end) = command(source, i);
        i = end;
        if !["\\newcommand", "\\renewcommand", "\\providecommand"].contains(&cmd) { continue; }
        if let Ok((name, next)) = argument(source, end) {
            if let Ok((body, next)) = argument(source, next) {
                let name = source[name].trim();
                if name.starts_with('\\') && !source[body.clone()].contains('#') {
                    found.push((name.to_string(), source[body].to_string()));
                }
                i = next;
            }
        }
    }
    found
}

/// Movable text excludes operations with external state, annotations or page scope.
pub fn validate_movable_text(fragment: &str, document: &str) -> Result<()> {
    validate_fragment(fragment)?;
    let mut normalized = std::borrow::Cow::Borrowed(fragment);
    for raw in line_ranges(fragment).into_iter().rev() {
        let row = &fragment[raw.clone()];
        let end = break_start(row).unwrap_or_else(|| visible_line_end(row));
        let range = trim_range(fragment, raw.start..raw.start + end);
        if let Some(position) = standard_text_position(&fragment[range.clone()]) {
            let content = &fragment[range.start + position.content.start..range.start + position.content.end];
            normalized.to_mut().replace_range(range, content);
        }
    }
    let fragment = normalized.as_ref();
    let definitions = simple_definitions(document);
    fn inspect(s: &str, definitions: &[(String, String)], chain: &mut Vec<String>) -> Result<()> {
        let mut i = 0;
        while i < s.len() {
            match s.as_bytes()[i] {
                b'%' => { i = comment_end(s, i); continue; }
                b'$' | b'&' => return Err("Select ordinary text; math and table cells need a dedicated layout adapter".into()),
                b'\\' => {
                    let (cmd, end) = command(s, i);
                    if skip_literal(s, cmd, end).is_some() { return Err("Literal/code environments cannot be moved as ordinary text".into()); }
                    if ["\\begin", "\\end", "\\section", "\\subsection", "\\chapter", "\\part", "\\caption", "\\label", "\\ref", "\\pageref", "\\cite", "\\href", "\\url", "\\hyperlink", "\\hyperref", "\\footnote", "\\marginpar", "\\includegraphics", "\\newcommand", "\\renewcommand", "\\def", "\\gdef", "\\write", "\\special", "\\TexloomObject", "\\enlargethispage", "\\vspace", "\\hspace", "\\fontsize", "\\selectfont", "\\color", "\\bfseries", "\\itshape", "\\centering", "\\raggedright", "\\input", "\\include", "\\usepackage", "\\documentclass"].contains(&cmd) {
                        return Err(format!("{cmd} needs source editing or a dedicated layout adapter"));
                    }
                    let basic = ["\\textbf", "\\textit", "\\textsl", "\\textsc", "\\textsf", "\\textrm", "\\texttt", "\\emph", "\\underline", "\\textcolor", "\\mbox", "\\par", "\\\\", "\\%", "\\&", "\\$", "\\#", "\\_", "\\{", "\\}", "\\ ", "\\,", "\\;", "\\:", "\\!", "\\-", "\\'", "\\\"", "\\`", "\\^", "\\~", "\\c", "\\v", "\\H", "\\u", "\\b", "\\d", "\\r", "\\oe", "\\OE", "\\ae", "\\AE", "\\ss", "\\o", "\\O", "\\textbackslash", "\\textasciitilde", "\\textasciicircum", "\\textless", "\\textgreater", "\\textquotesingle", "\\textasciigrave", "\\LaTeX", "\\TeX", "\\ldots", "\\dots"];
                    if !basic.contains(&cmd) {
                        let body = definitions.iter().rev().find(|(name, _)| name == cmd).map(|(_, text)| text)
                            .ok_or_else(|| format!("The effect of {cmd} is unknown. Edit this selection in the source editor."))?;
                        if chain.len() >= 8 || chain.iter().any(|name| name == cmd) { return Err("Recursive text macros cannot be moved safely".into()); }
                        chain.push(cmd.into()); inspect(body, definitions, chain)?; chain.pop();
                    }
                    i = end; continue;
                }
                _ => (),
            }
            i += 1;
        }
        Ok(())
    }
    inspect(fragment, &definitions, &mut Vec::new())
}

fn line_ranges(source: &str) -> Vec<Range<usize>> {
    let mut start = 0;
    let mut lines = Vec::new();
    for line in source.split_inclusive('\n') {
        let end = start + line.trim_end_matches(['\r', '\n']).len();
        lines.push(start..end); start += line.len();
    }
    if source.is_empty() || source.ends_with('\n') { lines.push(start..start); }
    lines
}

fn visible_line_end(s: &str) -> usize {
    let mut i = 0;
    while i < s.len() {
        if s.as_bytes()[i] == b'%' { return i; }
        if s.as_bytes()[i] == b'\\' { i = command(s, i).1; } else { i += 1; }
    }
    s.len()
}

fn trim_range(source: &str, range: Range<usize>) -> Range<usize> {
    let text = &source[range.clone()];
    let start = range.start + text.len() - text.trim_start().len();
    start..(range.start + text.trim_end().len()).max(start)
}

fn break_start(line: &str) -> Option<usize> {
    let end = visible_line_end(line);
    let line = line[..end].trim_end();
    let mut found = None;
    let mut i = 0;
    while i < line.len() {
        if line.as_bytes()[i] != b'\\' { i += 1; continue; }
        let (cmd, next) = command(line, i);
        if cmd == "\\\\" {
            let mut after = skip_space(line, next);
            if line.as_bytes().get(after) == Some(&b'[') {
                after = line[after..].find(']').map(|n| after + n + 1)?;
            }
            if line[after..].trim().is_empty() { found = Some(i); }
        }
        i = next;
    }
    found
}

fn boundary(line: &str) -> bool {
    let text = line.trim_start();
    if text.starts_with('\\') {
        let (name, _) = command(text, 0);
        if super::page_flow_command(name) || matches!(name, "\\part" | "\\subsubsection" | "\\label" | "\\par") {
            return true;
        }
    }
    text.is_empty() || text.starts_with('%') || ["\\begin", "\\end", "\\section", "\\subsection", "\\chapter", "\\enlargethispage", "\\vspace", "\\noindent", "\\TexloomObject"].iter().any(|prefix| text.starts_with(prefix))
}

pub fn resolve_text_unit(source: &str, line: usize) -> Result<SourceUnit> {
    resolve_text_unit_in_document(source, line, source)
}

pub fn resolve_text_unit_in_document(source: &str, line: usize, document: &str) -> Result<SourceUnit> {
    let lines = line_ranges(source);
    let index = line.checked_sub(1).filter(|n| *n < lines.len()).ok_or("The mapped source line is outside this file")?;
    let raw = &source[lines[index].clone()];
    if boundary(raw) { return Err("This PDF point maps to structural source. Select the text itself or use source editing.".into()); }
    if let Ok((_, body, end)) = document_bounds(source) {
        if lines[index].start < body || lines[index].end > end { return Err("Generated text or macro definitions in the preamble cannot be moved here".into()); }
    }
    if text_objects(source)?.iter().any(|object| object.range.contains(&lines[index].start)) {
        return Err("Select the existing object's frame to edit it".into());
    }
    let forced = break_start(raw);
    let previous_forced = index > 0 && break_start(&source[lines[index - 1].clone()]).is_some();
    let (first, last, range, kind) = if forced.is_some() || previous_forced {
        let end = lines[index].start + forced.unwrap_or_else(|| visible_line_end(raw));
        (index, index, trim_range(source, lines[index].start..end), "line")
    } else {
        let mut first = index;
        let mut last = index;
        while first > 0 && !boundary(&source[lines[first - 1].clone()]) && break_start(&source[lines[first - 1].clone()]).is_none() { first -= 1; }
        while last + 1 < lines.len() && !boundary(&source[lines[last + 1].clone()]) && break_start(&source[lines[last].clone()]).is_none() { last += 1; }
        (first, last, trim_range(source, lines[first].start..lines[last].end), "paragraph")
    };
    let range = standard_text_position(&source[range.clone()])
        .map_or(range.clone(), |position| range.start + position.content.start..range.start + position.content.end);
    if range.is_empty() { return Err("There is no editable text at this source location".into()); }
    let content = source[range.clone()].to_string();
    validate_movable_text(&content, document)?;
    Ok(SourceUnit { range, first_line: first + 1, last_line: last + 1, kind, content })
}

pub fn combine_text_units(source: &str, units: &[SourceUnit]) -> Result<SourceUnit> {
    combine_text_units_in_document(source, units, source)
}

pub fn combine_text_units_in_document(source: &str, units: &[SourceUnit], document: &str) -> Result<SourceUnit> {
    let mut units = units.to_vec();
    units.sort_by_key(|u| u.range.start);
    units.dedup_by(|a, b| a.range == b.range);
    if units.len() > 1 {
        for unit in &mut units {
            if let Some((wrapper, _)) = source_line_position(source, &unit.range) {
                unit.range = wrapper;
            }
        }
    }
    let first = units.first().ok_or("Select at least one text line")?;
    let last = units.last().unwrap();
    for pair in units.windows(2) {
        if pair[0].range.end > pair[1].range.start { return Err("Overlapping source units cannot form a group".into()); }
        let gap = &source[pair[0].range.end..pair[1].range.start];
        let mut i = 0;
        while i < gap.len() {
            i = skip_space(gap, i);
            if i == gap.len() { break; }
            if !gap[i..].starts_with("\\\\") { return Err("Select adjacent lines; unselected source occurs between these items".into()); }
            i = skip_space(gap, i + 2);
            if gap.as_bytes().get(i) == Some(&b'[') { i = gap[i..].find(']').map(|n| i + n + 1).ok_or("Unclosed line spacing")?; }
        }
    }
    let range = first.range.start..last.range.end;
    let content = source[range.clone()].to_string();
    validate_movable_text(&content, document)?;
    Ok(SourceUnit { range, first_line: first.first_line, last_line: last.last_line,
        kind: if units.len() > 1 { "group" } else { first.kind }, content })
}

struct StandardTextPosition {
    content: Range<usize>,
    dx: f64,
    dy: f64,
}

// Recognize only the exact balanced form emitted by position_text_range.
// Other raisebox constructions retain their own source semantics.
fn standard_text_position(source: &str) -> Option<StandardTextPosition> {
    let offset = source.strip_prefix("\\raisebox{")?;
    let (lift, rest) = offset.split_once("bp}[\\height][\\depth]{\\kern")?;
    let (shift, rest) = rest.split_once("bp\\relax ")?;
    let (content, back) = rest.rsplit_once("\\kern")?;
    let back = back.strip_suffix("bp\\relax}")?;
    let dx = shift.parse::<f64>().ok()?;
    let dy = -lift.parse::<f64>().ok()?;
    if number(dx, false).ok()?.as_str() != shift
        || number(-dy, false).ok()?.as_str() != lift
        || number(-dx, false).ok()?.as_str() != back
    {
        return None;
    }
    let start = source.len() - rest.len();
    Some(StandardTextPosition { content: start..start + content.len(), dx, dy })
}

fn source_line_position(source: &str, range: &Range<usize>) -> Option<(Range<usize>, StandardTextPosition)> {
    let first = source[..range.start].rfind('\n').map_or(0, |index| index + 1);
    let last = source[range.end..].find('\n').map_or(source.len(), |index| range.end + index);
    let raw_line = &source[first..last];
    let text_end = break_start(raw_line).unwrap_or_else(|| visible_line_end(raw_line));
    let line = trim_range(source, first..first + text_end);
    standard_text_position(&source[line.clone()])
        .filter(|position| line.start + position.content.start == range.start
            && line.start + position.content.end == range.end)
        .map(|position| (line, position))
}

/// Move adjacent explicit lines independently so TeX keeps the original breaks.
pub fn position_text_group(
    source: &str, range: Range<usize>, dx: f64, dy: f64, document: &str,
) -> Result<(String, Range<usize>, f64, f64)> {
    let fragment = source.get(range.clone()).ok_or("Invalid group selection")?;
    let segments = forced_segments(fragment)?;
    if !(2..=16).contains(&segments.len()) || segments.iter().any(|(line, _)| line.is_empty()) {
        return Err("Free dragging a group requires adjacent explicit LaTeX line breaks".into());
    }
    let mut updated = source.to_owned();
    let mut applied = None;
    for (line, _) in segments.into_iter().rev() {
        let line = range.start + line.start..range.start + line.end;
        let inner = standard_text_position(&source[line.clone()]).map_or(line.clone(), |position| {
            line.start + position.content.start..line.start + position.content.end
        });
        let (candidate, _, step_x, step_y) = position_text_range(&updated, inner, dx, dy, document)?;
        if applied.is_some_and(|(x, y): (f64, f64)| (x - step_x).abs() > 0.00001 || (y - step_y).abs() > 0.00001) {
            return Err("The selected lines cannot move by the same offset".into());
        }
        applied = Some((step_x, step_y));
        updated = candidate;
    }
    let end = updated.len() - (source.len() - range.end);
    let (step_x, step_y) = applied.ok_or("Empty group")?;
    Ok((updated, range.start..end, step_x, step_y))
}

/// Use a LaTeX box whose width and height/depth match the original text.
/// The caller must prove those dimensions and the rendered translation.
pub fn position_text_range(
    source: &str, range: Range<usize>, dx: f64, dy: f64, document: &str,
) -> Result<(String, Range<usize>, f64, f64)> {
    let content = source.get(range.clone()).ok_or("Invalid source selection")?;
    if content.is_empty() || content.contains(['\r', '\n']) || visible_line_end(content) != content.len() || content.contains("\\\\") || content.contains("\\par") {
        return Err("Free dragging supports one complete text line at a time".into());
    }
    validate_movable_text(content, document)?;
    let existing = source_line_position(source, &range);
    let (replace, old_dx, old_dy) = match existing {
        Some((line, position)) => (line, position.dx, position.dy),
        None => (range, 0.0, 0.0),
    };
    let round_offset = |value: f64| {
        let rounded = (value * 100.0).round() / 100.0;
        if rounded == 0.0 { 0.0 } else { rounded }
    };
    let target_dx = round_offset(old_dx + dx);
    let target_dy = round_offset(old_dy + dy);
    if target_dx == 0.0 && target_dy == 0.0 {
        let start = replace.start;
        let mut updated = source.to_owned();
        updated.replace_range(replace, content);
        return Ok((updated, start..start + content.len(), -old_dx, -old_dy));
    }
    let prefix = format!("\\raisebox{{{}bp}}[\\height][\\depth]{{\\kern{}bp\\relax ",
        number(-target_dy, false)?, number(target_dx, false)?);
    let suffix = format!("\\kern{}bp\\relax}}", number(-target_dx, false)?);
    let start = replace.start + prefix.len();
    let mut updated = source.to_owned();
    updated.replace_range(replace, &format!("{prefix}{content}{suffix}"));
    Ok((updated, start..start + content.len(), target_dx - old_dx, target_dy - old_dy))
}

pub fn ensure_objects_package(source: &str) -> Result<String> {
    let (preamble, _, _) = document_bounds(source)?;
    if has_package(&source[..preamble], "texloom-objects") { return Ok(source.into()); }
    let mut updated = source.to_string();
    updated.insert_str(preamble, "\\usepackage{texloom-objects}\n"); Ok(updated)
}

pub fn wrap_text_object(source: &str, range: Range<usize>, id: &str, dx: f64, dy: f64, document: &str) -> Result<String> {
    if !valid_id(id) { return Err("Invalid object identifier".into()); }
    let existing = text_objects(source)?;
    if existing.iter().any(|o| o.id == id || o.range.start < range.end && o.range.end > range.start) { return Err("Duplicate or nested text objects are not supported".into()); }
    let content = source.get(range.clone()).filter(|s| !s.trim().is_empty()).ok_or("Invalid source selection")?;
    validate_movable_text(content, document)?;
    let replacement = format!("\\TexloomObject{{{id}}}{{{}}}{{{}}}{{{}\n}}", number(dx, false)?, number(dy, false)?, content);
    let mut updated = source.to_string(); updated.replace_range(range, &replacement);
    text_objects(&updated)?; Ok(updated)
}

pub fn update_text_object(source: &str, id: &str, dx: Option<f64>, dy: Option<f64>, content: Option<&str>, document: &str) -> Result<String> {
    let object = text_objects(source)?.into_iter().find(|o| o.id == id).ok_or("The selected object no longer exists")?;
    let mut patches = Vec::new();
    if let Some(value) = dx { patches.push((object.arguments[1].clone(), number(value, false)?)); }
    if let Some(value) = dy { patches.push((object.arguments[2].clone(), number(value, false)?)); }
    if let Some(text) = content { validate_movable_text(text, document)?; patches.push((object.arguments[3].clone(), format!("{}\n", text.trim_end()))); }
    patches.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    let mut updated = source.to_string();
    for (range, text) in patches { updated.replace_range(range, &text); }
    text_objects(&updated)?; Ok(updated)
}

pub fn unwrap_text_object(source: &str, id: &str) -> Result<String> {
    let object = text_objects(source)?.into_iter().find(|o| o.id == id).ok_or("The selected object no longer exists")?;
    let mut updated = source.to_string();
    // Exactly one newline is added by the wrapper so trailing comments remain safe.
    let content = object.content.strip_suffix('\n').unwrap_or(&object.content);
    updated.replace_range(object.range, content); Ok(updated)
}

fn forced_segments(source: &str) -> Result<Vec<(Range<usize>, String)>> {
    let mut result=Vec::new(); let mut start=0; let mut i=0; let mut depth=0usize;
    while i<source.len() {
        match source.as_bytes()[i] {
            b'%' => { i=comment_end(source,i);continue; }
            b'{' => depth+=1,
            b'}' => depth=depth.checked_sub(1).ok_or("Unbalanced text")?,
            b'\\' => {
                let (cmd,end)=command(source,i);
                if cmd=="\\\\" && depth==0 {
                    let mut after=skip_space(source,end);
                    if source.as_bytes().get(after)==Some(&b'[') { after=source[after..].find(']').map(|n|after+n+1).ok_or("Unclosed line spacing")?; }
                    result.push((trim_range(source,start..i),source[i..after].into()));start=after;i=after;continue;
                }
                i=end;continue;
            }
            _=>(),
        }
        i+=1;
    }
    result.push((trim_range(source,start..source.len()),String::new()));Ok(result)
}

fn plain_segment(source: &str, definitions: &[(String,String)], replacement: Option<&str>, depth: usize) -> Result<String> {
    if depth>8 { return Err("Text macro nesting is too deep".into()); }
    let source=source.trim();
    if source.starts_with('\\') {
        let (cmd,end)=command(source,0);
        if ["\\textbf","\\textit","\\textsl","\\textsc","\\textsf","\\textrm","\\texttt","\\emph","\\underline"].contains(&cmd) {
            let (range,after)=argument(source,end)?;
            if !source[after..].trim().is_empty() { return Err("Mixed inline formatting should be edited as LaTeX".into()); }
            let inner=plain_segment(&source[range],definitions,replacement,depth+1)?;
            return Ok(if replacement.is_some(){format!("{cmd}{{{inner}}}")}else{inner});
        }
        if source[end..].trim().is_empty() {
            if let Some((_,body))=definitions.iter().rev().find(|(name,_)|name==cmd) {return plain_segment(body,definitions,replacement,depth+1)}
        }
    }
    if source.chars().any(|c| matches!(c,'\\'|'{'|'}'|'%'|'$'|'&'|'#'|'_'|'~'|'^')) { return Err("Formatted text should be edited as LaTeX so its commands are preserved".into()); }
    Ok(replacement.map(super::escape_text).unwrap_or_else(||source.split_whitespace().collect::<Vec<_>>().join(" ")))
}

/// Definitions from the main document used to describe plain editable text.
pub struct PlainTextPreview {
    definitions: Vec<(String, String)>,
}

impl PlainTextPreview {
    pub fn new(document: &str) -> Self {
        Self { definitions: simple_definitions(document) }
    }

    /// Every logical line must have a lossless style template.
    pub fn render(&self, fragment: &str) -> Option<String> {
        let segments = forced_segments(fragment).ok()?;
        let lines = segments.iter()
            .map(|(range, _)| plain_segment(&fragment[range.clone()], &self.definitions, None, 0))
            .collect::<Result<Vec<_>>>().ok()?;
        Some(lines.join("\n"))
    }
}

pub fn plain_text_preview(fragment: &str, document: &str) -> Option<String> {
    PlainTextPreview::new(document).render(fragment)
}

pub fn replace_plain_text(fragment: &str, value: &str, document: &str) -> Result<String> {
    let definitions=simple_definitions(document);let segments=forced_segments(fragment)?;
    let values:Vec<_>=if segments.len()==1 {vec![value]}else{value.split('\n').collect()};
    if values.len()!=segments.len() {return Err("Keep the same number of lines, or use LaTeX mode to change line structure".into());}
    let mut result=String::new();
    for ((range,separator),value) in segments.iter().zip(values) {
        result.push_str(&plain_segment(&fragment[range.clone()],&definitions,Some(value),0)?);result.push_str(separator);
        if !separator.is_empty(){result.push('\n')}
    }
    validate_movable_text(&result,document)?;Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    const DOC: &str = "\\documentclass{article}\n\\newcommand{\\Person}{Dr. Example}\n\\begin{document}\nFirst paragraph.\nIt wraps over source lines.\n\n\\begin{minipage}{\\linewidth}\nBest regards,\\\\[10mm]\n\\textbf{\\Person}\\\\[2pt]\nExample Department\\\\[2pt]\nExample University\n\\end{minipage}\n\nFollowing paragraph.\n\\end{document}";
    #[test]
    fn repeat_positioning_preserves_forced_breaks_comments_and_original_bytes() {
        let source = "\\begin{document}\r\n\\textbf{Name}\\\\[2pt] % Keep this\r\nDepartment\r\n\\end{document}\r\n";
        let original = resolve_text_unit(source, 2).unwrap();
        let (moved, range, dx, dy) = position_text_range(source, original.range, 3.126, -4.5, source).unwrap();
        assert_eq!((dx, dy), (3.13, -4.5));
        assert_eq!(
            moved,
            source.replace("\\textbf{Name}", "\\raisebox{4.5bp}[\\height][\\depth]{\\kern3.13bp\\relax \\textbf{Name}\\kern-3.13bp\\relax}"),
        );
        assert_eq!(resolve_text_unit(&moved, 2).unwrap().range, range);
        let (restored, _, _, _) = position_text_range(&moved, range, -3.13, 4.5, &moved).unwrap();
        assert_eq!(restored, source);
    }

    #[test]
    fn positioning_refuses_multi_line_ranges_and_nonfinite_offsets() {
        let source = "First\nSecond";
        assert!(position_text_range(source, 0..source.len(), 10.0, 0.0, source).is_err());
        assert!(position_text_range(source, 0..5, f64::INFINITY, 0.0, source).is_err());
    }

    #[test]
    fn explicit_line_group_preserves_breaks_and_can_be_moved_again() {
        let source = "Opening.\\\\[2pt]\r\nSecond.\r\n";
        let range = 0..source.trim_end().len();
        let (moved, span, _, _) = position_text_group(source, range, 6.0, 12.0, source).unwrap();
        let units = [resolve_text_unit(&moved, 1).unwrap(), resolve_text_unit(&moved, 2).unwrap()];
        assert_eq!(combine_text_units(&moved, &units).unwrap().range, span);
        assert!(moved.contains("\\\\[2pt]\r\n"));
        let (restored, _, _, _) = position_text_group(&moved, span, -6.0, -12.0, &moved).unwrap();
        assert_eq!(restored, source);
        assert!(position_text_group("wrapped\ntext", 0..12, 1.0, 2.0, source).is_err());
    }

    #[test] fn soft_wrapped_prose_is_one_unit() { let u = resolve_text_unit(DOC, 5).unwrap(); assert_eq!(u.kind,"paragraph"); assert_eq!((u.first_line,u.last_line),(4,5)); }
    #[test] fn page_breaks_bound_adjacent_paragraphs_without_becoming_movable_content() {
        for page_break in ["\\newpage", "\\clearpage", "\\cleardoublepage", "\\pagebreak", "\\pagebreak[2]", "\\nopagebreak[4]"] {
            let source = format!("\\documentclass{{article}}\r\n\\begin{{document}}\r\nBefore page.\r\n{page_break}\r\nAfter page.\r\n\\end{{document}}\r\n");
            let before = resolve_text_unit(&source, 3).unwrap();
            let after = resolve_text_unit(&source, 5).unwrap();
            assert_eq!(before.content, "Before page.");assert_eq!(after.content, "After page.");
            assert!(resolve_text_unit(&source, 4).is_err());
            assert!(combine_text_units(&source, &[before.clone(), after]).is_err());
            let wrapped = wrap_text_object(&source, before.range, "before-page", 0.0, 0.0, &source).unwrap();
            assert_eq!(unwrap_text_object(&wrapped, "before-page").unwrap(), source);
        }
        let inline = "\\documentclass{article}\n\\begin{document}\nText.\\newpage\nAfter.\n\\end{document}";
        assert!(resolve_text_unit(inline, 3).is_err(), "inline structural content must still refuse");
    }
    #[test]
    fn standalone_structure_stays_outside_adjacent_text_selections() {
        for structure in [r"\part{Title}", r"\subsubsection*{Title}", r"\label{sec:title}", r"\par"] {
            let source = format!("\\documentclass{{report}}\n\\begin{{document}}\nBefore.\n{structure}\nAfter.\n\\end{{document}}");
            let before = resolve_text_unit(&source, 3).unwrap();
            let after = resolve_text_unit(&source, 5).unwrap();
            assert_eq!(before.content, "Before.");
            assert_eq!(after.content, "After.");
            assert!(resolve_text_unit(&source, 4).is_err());
        }
    }
    #[test] fn signature_line_keeps_its_break_outside() { let u = resolve_text_unit(DOC,9).unwrap(); assert_eq!(u.content,"\\textbf{\\Person}"); assert_eq!(u.kind,"line"); }
    #[test] fn last_signature_line_is_independent() { assert_eq!(resolve_text_unit(DOC,11).unwrap().content,"Example University"); }
    #[test] fn adjacent_signature_lines_combine() { let a=resolve_text_unit(DOC,9).unwrap();let b=resolve_text_unit(DOC,10).unwrap(); let c=resolve_text_unit(DOC,11).unwrap(); let group=combine_text_units(DOC,&[c,a,b]).unwrap();assert_eq!(group.kind,"group");assert!(group.content.contains("\\\\[2pt]")); }
    #[test] fn group_cannot_swallow_an_unselected_line() { assert!(combine_text_units(DOC,&[resolve_text_unit(DOC,9).unwrap(),resolve_text_unit(DOC,11).unwrap()]).is_err()); }
    #[test] fn preamble_and_structural_lines_are_rejected() { for line in [1,2,3,7,12,15] { assert!(resolve_text_unit(DOC,line).is_err(),"line {line}"); } }
    #[test] fn wrapper_preserves_unrelated_text_and_can_detach() { let u=resolve_text_unit(DOC,10).unwrap();let s=wrap_text_object(DOC,u.range,"name",10.0,5.0,DOC).unwrap();assert_eq!(unwrap_text_object(&s,"name").unwrap(),DOC);assert!(s.contains("}\\\\[2pt]")); }
    #[test] fn object_patch_only_changes_requested_offsets() { let u=resolve_text_unit(DOC,10).unwrap();let s=wrap_text_object(DOC,u.range,"name",0.0,0.0,DOC).unwrap();let moved=update_text_object(&s,"name",Some(15.0),Some(8.0),None,DOC).unwrap();assert_eq!(moved,s.replace("{name}{0}{0}","{name}{15}{8}")); }
    #[test] fn comments_and_literal_commands_are_not_objects() { assert!(text_objects("% \\TexloomObject{o}{0}{0}{Fake}\n\\verb|\\TexloomObject{o}{0}{0}{Fake}|").unwrap().is_empty()); }
    #[test] fn duplicate_ids_and_nested_objects_are_rejected() { for s in ["\\TexloomObject{a}{0}{0}{x}\\TexloomObject{a}{0}{0}{y}","\\TexloomObject{a}{0}{0}{\\TexloomObject{b}{0}{0}{x}}"] { assert!(text_objects(s).is_err()); } }
    #[test] fn local_text_macro_is_supported_but_unknown_macro_is_not() { assert!(validate_movable_text("\\textbf{\\Person}",DOC).is_ok());assert!(validate_movable_text("\\Unknown{X}",DOC).is_err()); }
    #[test] fn annotations_and_external_side_effects_are_rejected() { for s in ["\\href{https://example.com}{x}","\\ref{x}","\\footnote{x}","\\input{x}","$a+b$","\\begin{minipage}{2cm}X\\end{minipage}"] { assert!(validate_movable_text(s,DOC).is_err(),"{s}"); } }
    #[test] fn wrapper_keeps_a_trailing_comment_safe() { let s="Text % comment";let wrapped=wrap_text_object(s,0..s.len(),"comment",0.0,0.0,DOC).unwrap();assert_eq!(text_objects(&wrapped).unwrap()[0].content,"Text % comment\n"); }
    #[test] fn nonfinite_and_excessive_offsets_are_rejected() { let u=resolve_text_unit(DOC,10).unwrap();for value in [f64::NAN,f64::INFINITY,10000.0] { assert!(wrap_text_object(DOC,u.range.clone(),"bad",value,0.0,DOC).is_err()); } }
    #[test] fn recursive_or_effectful_macros_are_rejected() { let s="\\documentclass{article}\n\\newcommand{\\A}{\\A}\n\\newcommand{\\B}{\\footnote{X}}\n\\begin{document}X\\end{document}";for name in ["\\A","\\B"] { assert!(validate_movable_text(name,s).is_err()); } }
    #[test] fn package_is_inserted_only_once() { let s=ensure_objects_package(DOC).unwrap();assert_eq!(ensure_objects_package(&s).unwrap(),s); }
    #[test] fn plain_name_edit_keeps_bold_and_changes_only_the_call() { let before="\\textbf{\\Person}";assert_eq!(plain_text_preview(before,DOC).unwrap(),"Dr. Example");assert_eq!(replace_plain_text(before,"Dr. Someone Else",DOC).unwrap(),"\\textbf{Dr. Someone Else}"); }
    #[test] fn plain_group_edit_retains_spacing_and_each_line_style() { let source="\\textbf{\\Person}\\\\[2pt]\nDepartment\\\\[4pt]\nUniversity";let changed=replace_plain_text(source,"New Name\nNew Department\nNew University",DOC).unwrap();assert!(changed.contains("\\textbf{New Name}\\\\[2pt]"));assert!(changed.contains("New Department\\\\[4pt]")); }
    #[test] fn plain_edit_does_not_silently_drop_mixed_formatting() { assert!(plain_text_preview("Hello \\textbf{world}",DOC).is_none());assert!(replace_plain_text("Hello \\textbf{world}","Different text",DOC).is_err()); }
    #[test] fn plain_group_line_count_is_checked() { assert!(replace_plain_text("A\\\\[2pt]B","Only one line",DOC).is_err()); }
    #[test] fn included_text_can_use_the_main_preamble() { assert!(resolve_text_unit_in_document("\\textbf{\\Person}\\\\[2pt]\nDepartment",1,DOC).is_ok()); }
}
