//! Lossless LaTeX block edits. This crate intentionally has no dependencies.
use std::ops::Range;
mod objects;
pub use objects::*;
pub mod document;
pub mod literal_text;
pub mod spacing;
pub mod adapters;
pub mod templates;
mod selection_hints;
pub use selection_hints::resolve_selection_hint;

pub type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub id: String,
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub size: f64,
    pub content: String,
    pub range: Range<usize>,
    pub arguments: Vec<Range<usize>>,
    pub line: usize,
}

#[derive(Clone, Debug)]
pub enum Change {
    Page(usize), X(f64), Y(f64), Width(f64), Size(f64), Content(String),
}

fn command(s: &str, start: usize) -> (&str, usize) {
    let b = s.as_bytes();
    let mut end = start + 1;
    if end < b.len() && (b[end].is_ascii_alphabetic() || b[end] == b'@') {
        while end < b.len() && (b[end].is_ascii_alphabetic() || b[end] == b'@') { end += 1; }
    } else if end < b.len() {
        end += s[end..].chars().next().map(char::len_utf8).unwrap_or(0);
    }
    (&s[start..end], end)
}

fn comment_end(s: &str, i: usize) -> usize {
    s[i..].find('\n').map_or(s.len(), |n| i + n + 1)
}

fn skip_space(s: &str, mut i: usize) -> usize {
    let b = s.as_bytes();
    while i < b.len() {
        if b[i].is_ascii_whitespace() { i += 1; }
        else if b[i] == b'%' { i = comment_end(s, i); }
        else { break; }
    }
    i
}

fn argument(s: &str, start: usize) -> Result<(Range<usize>, usize)> {
    let start = skip_space(s, start);
    let b = s.as_bytes();
    if b.get(start) != Some(&b'{') { return Err("Expected a braced argument".into()); }
    let mut depth = 1;
    let mut i = start + 1;
    while i < b.len() {
        match b[i] {
            b'\\' => { i = command(s, i).1; continue; }
            b'%' => { i = comment_end(s, i); continue; }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 { return Ok((start + 1..i, i + 1)); }
            }
            _ => (),
        }
        i += 1;
    }
    Err("Unclosed LaTeX argument".into())
}

fn skip_literal(s: &str, cmd: &str, mut end: usize) -> Option<usize> {
    if cmd == "\\verb" {
        if s.as_bytes().get(end) == Some(&b'*') { end += 1; }
        let delimiter = s[end..].chars().next()?;
        if delimiter.is_whitespace() { return None; }
        let begin = end + delimiter.len_utf8();
        return Some(s[begin..].find(delimiter).map_or(s.len(), |n| begin + n + delimiter.len_utf8()));
    }
    if cmd == "\\begin" {
        let (arg, after) = argument(s, end).ok()?;
        let env = &s[arg];
        if ["verbatim", "verbatim*", "Verbatim", "lstlisting", "minted"].contains(&env) {
            let closing = format!("\\end{{{env}}}");
            return Some(s[after..].find(&closing).map_or(s.len(), |n| after + n + closing.len()));
        }
    }
    None
}

pub fn blocks(s: &str) -> Vec<Block> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if s.as_bytes()[i] == b'%' { i = comment_end(s, i); continue; }
        if s.as_bytes()[i] != b'\\' { i += 1; continue; }
        let (cmd, end) = command(s, i);
        if let Some(after) = skip_literal(s, cmd, end) { i = after; continue; }
        if cmd != "\\StudioBlock" { i = end; continue; }
        let parsed = (|| -> Result<Block> {
            let mut args = Vec::new();
            let mut cursor = end;
            for _ in 0..7 { let (range, after) = argument(s, cursor)?; args.push(range); cursor = after; }
            let id = &s[args[0].clone()];
            if id.is_empty() || id.len() > 64 || !id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-') {
                return Err("Invalid block ID".into());
            }
            let page = s[args[1].clone()].trim().parse::<usize>().map_err(|_| "Invalid page")?;
            let number = |n: usize| -> Result<f64> {
                let value = s[args[n].clone()].trim().parse::<f64>().map_err(|_| "Invalid dimension")?;
                if !value.is_finite() { return Err("Invalid dimension".into()); }
                Ok(value)
            };
            let (x, y, width, size) = (number(2)?, number(3)?, number(4)?, number(5)?);
            if page == 0 || width <= 0.0 || size <= 0.0 { return Err("Invalid block dimensions".into()); }
            Ok(Block { id: id.into(), page, x, y, width, size, content: s[args[6].clone()].into(),
                range: i..cursor, arguments: args, line: s[..i].bytes().filter(|c| *c == b'\n').count() + 1 })
        })();
        if let Ok(block) = parsed { i = block.range.end; result.push(block); }
        else { i = end; }
    }
    result
}

fn page_flow_command(cmd: &str) -> bool {
    matches!(cmd, "\\newpage" | "\\clearpage" | "\\cleardoublepage" | "\\pagebreak" | "\\nopagebreak")
}

pub fn validate_fragment(s: &str) -> Result<()> {
    let mut envs: Vec<String> = Vec::new();
    let mut braces = 0usize;
    let mut i = 0;
    while i < s.len() {
        match s.as_bytes()[i] {
            b'%' => { i = comment_end(s, i); continue; }
            b'\\' => {
                let (cmd, end) = command(s, i);
                if let Some(after) = skip_literal(s, cmd, end) { i = after; continue; }
                if page_flow_command(cmd) || ["\\input", "\\include", "\\usepackage", "\\documentclass", "\\AddToHook"].contains(&cmd) {
                    return Err("Page and project commands must remain in normal document flow".into());
                }
                if cmd == "\\begin" || cmd == "\\end" {
                    let (range, after) = argument(s, end)?;
                    let name = &s[range];
                    if ["document", "figure", "figure*", "table", "table*", "longtable", "filecontents"].contains(&name) {
                        return Err("This environment cannot be placed inside a movable block".into());
                    }
                    if cmd == "\\begin" { envs.push(name.into()); }
                    else if envs.pop().as_deref() != Some(name) {
                        return Err("Select a complete environment, including its matching begin and end commands".into());
                    }
                    i = after; continue;
                }
                i = end; continue;
            }
            b'{' => braces += 1,
            b'}' => { braces = braces.checked_sub(1).ok_or("Select complete content with balanced braces")?; }
            _ => (),
        }
        i += 1;
    }
    if braces != 0 { return Err("Select complete content with balanced braces".into()); }
    if !envs.is_empty() { return Err("Select a complete environment, including its matching begin and end commands".into()); }
    Ok(())
}

fn number(value: f64, positive: bool) -> Result<String> {
    if !value.is_finite() || value.abs() >= 10000.0 || (positive && value <= 0.0) {
        return Err("Invalid dimension".into());
    }
    Ok(format!("{value:.2}").trim_end_matches('0').trim_end_matches('.').to_string())
}

pub fn patch_block(source: &str, id: &str, changes: &[Change]) -> Result<String> {
    let found: Vec<_> = blocks(source).into_iter().filter(|b| b.id == id).collect();
    if found.len() != 1 { return Err("Block is missing or its ID is duplicated".into()); }
    let block = &found[0];
    let mut edits: Vec<(Range<usize>, String)> = Vec::new();
    for change in changes {
        let (index, value) = match change {
            Change::Page(p) if *p > 0 => (1, p.to_string()),
            Change::Page(_) => return Err("Page must be positive".into()),
            Change::X(n) => (2, number(*n, false)?), Change::Y(n) => (3, number(*n, false)?),
            Change::Width(n) => (4, number(*n, true)?), Change::Size(n) => (5, number(*n, true)?),
            Change::Content(text) => { validate_fragment(text)?; (6, text.clone()) },
        };
        if edits.iter().any(|(r, _)| *r == block.arguments[index]) { return Err("A field may only be changed once".into()); }
        edits.push((block.arguments[index].clone(), value));
    }
    edits.sort_by_key(|(r, _)| std::cmp::Reverse(r.start));
    let mut updated = source.to_string();
    for (range, value) in edits { updated.replace_range(range, &value); }
    let current = blocks(&updated).into_iter().find(|b| b.id == id).ok_or("The edit would break the block")?;
    for change in changes { if let Change::Content(text) = change { if current.content != *text { return Err("The edit would break the block's braces or comments".into()); } } }
    Ok(updated)
}

fn document_bounds(s: &str) -> Result<(usize, usize, usize)> {
    let mut begin = None;
    let mut finish = None;
    let mut i = 0;
    while i < s.len() {
        if s.as_bytes()[i] == b'%' { i = comment_end(s, i); continue; }
        if s.as_bytes()[i] != b'\\' { i += 1; continue; }
        let (cmd, end) = command(s, i);
        if let Some(after) = skip_literal(s, cmd, end) { i = after; continue; }
        if cmd == "\\begin" || cmd == "\\end" {
            if let Ok((range, after)) = argument(s, end) {
                if &s[range] == "document" {
                    if cmd == "\\begin" { begin = Some((i, after)); } else { finish = Some(i); }
                }
                i = after; continue;
            }
        }
        i = end;
    }
    match (begin, finish) {
        (Some((start, body)), Some(end)) if body <= end => Ok((start, body, end)),
        _ => Err("The main source must contain a complete document environment".into()),
    }
}

fn has_package(s: &str, package: &str) -> bool {
    let mut i = 0;
    while i < s.len() {
        if s.as_bytes()[i] == b'%' { i = comment_end(s, i); continue; }
        if s.as_bytes()[i] != b'\\' { i += 1; continue; }
        let (cmd, end) = command(s, i);
        if cmd == "\\usepackage" {
            let mut pos = skip_space(s, end);
            if s.as_bytes().get(pos) == Some(&b'[') {
                if let Some(n) = s[pos..].find(']') { pos += n + 1; }
            }
            if let Ok((range, after)) = argument(s, pos) {
                if s[range].split(',').any(|p| p.trim() == package) { return true; }
                i = after; continue;
            }
        }
        i = end;
    }
    false
}

#[derive(Clone, Debug)]
pub struct NewBlock {
    pub id: String, pub page: usize, pub x: f64, pub y: f64,
    pub width: f64, pub size: f64, pub content: String,
}

pub fn insert_block(source: &str, block: &NewBlock, selection: Option<Range<usize>>) -> Result<String> {
    if block.id.is_empty() || block.id.len() > 64 || !block.id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-') { return Err("Invalid block ID".into()); }
    if blocks(source).iter().any(|b| b.id == block.id) { return Err("Duplicate block ID".into()); }
    if block.page == 0 { return Err("Page must be positive".into()); }
    let (x, y, width, size) = (number(block.x, false)?, number(block.y, false)?, number(block.width, true)?, number(block.size, true)?);
    let (_, body, end) = document_bounds(source)?;
    let mut updated = source.to_string();
    let content = if let Some(range) = selection {
        if range.start < body || range.end > end || range.start >= range.end || !source.is_char_boundary(range.start) || !source.is_char_boundary(range.end) {
            return Err("Select complete content inside the document body".into());
        }
        let text = source[range.clone()].to_string(); validate_fragment(&text)?;
        updated.replace_range(range, ""); text
    } else { validate_fragment(&block.content)?; block.content.clone() };
    let (preamble, _, _) = document_bounds(&updated)?;
    if !has_package(&updated[..preamble], "studio") { updated.insert_str(preamble, "\\usepackage{studio}\n"); }
    let (_, _, end) = document_bounds(&updated)?;
    // A terminating newline prevents a trailing comment swallowing the closing brace.
    let separator = if content.ends_with('\n') { "" } else { "\n" };
    let text = format!("\n\\StudioBlock{{{}}}{{{}}}{{{x}}}{{{y}}}{{{width}}}{{{size}}}{{{content}{separator}}}\n", block.id, block.page);
    updated.insert_str(end, &text);
    if !blocks(&updated).iter().any(|b| b.id == block.id) { return Err("The selection does not form a complete movable block".into()); }
    Ok(updated)
}

pub fn utf16_to_byte(s: &str, offset: usize) -> Result<usize> {
    let mut count = 0;
    for (byte, ch) in s.char_indices() {
        if count == offset { return Ok(byte); }
        count += ch.len_utf16();
        if count > offset { return Err("Select complete Unicode characters".into()); }
    }
    if count == offset { Ok(s.len()) } else { Err("Selection is outside the source".into()) }
}

pub fn byte_to_utf16(s: &str, byte: usize) -> Result<usize> {
    s.get(..byte).map(|p| p.encode_utf16().count()).ok_or_else(|| "Invalid character boundary".into())
}

pub fn escape_text(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => result.push_str("\\textbackslash{}"), '{' => result.push_str("\\{"), '}' => result.push_str("\\}"),
            '%' => result.push_str("\\%"), '&' => result.push_str("\\&"), '#' => result.push_str("\\#"),
            '_' => result.push_str("\\_"), '$' => result.push_str("\\$"), '~' => result.push_str("\\textasciitilde{}"),
            '^' => result.push_str("\\textasciicircum{}"), _ => result.push(ch),
        }
    }
    result.replace("\n\n", "\n\\par\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str = "\\documentclass{article}\n\\usepackage{studio}\n\\begin{document}\n\\null\n\\StudioBlock{title}{1}{62}{100}{300}{20}{Hello \\textbf{world}.}\n\\end{document}\n";
    fn new_block() -> NewBlock { NewBlock { id: "new".into(), page: 1, x: 50.0, y: 80.0, width: 300.0, size: 12.0, content: "New text".into() } }
    #[test] fn preserves_every_other_character() {
        let changed = patch_block(SOURCE, "title", &[Change::X(84.0), Change::Y(136.0)]).unwrap();
        assert_eq!(changed, SOURCE.replace("{62}{100}", "{84}{136}"));
    }
    #[test] fn comments_and_verbatim_are_not_blocks() {
        let literal = "\\StudioBlock{fake}{1}{1}{1}{1}{1}{Text}";
        let s = format!("% {literal}\n\\begin{{verbatim}}{literal}\\end{{verbatim}}\n\\verb|{literal}|\n{SOURCE}");
        assert_eq!(blocks(&s).len(), 1);
    }
    #[test] fn duplicate_ids_are_rejected() { assert!(patch_block(&format!("{SOURCE}{SOURCE}"), "title", &[Change::X(1.0)]).is_err()); }
    #[test] fn a_closing_environment_cannot_be_moved_on_its_own() { assert!(validate_fragment("\\end{center}").is_err()); }
    #[test] fn incomplete_and_mismatched_environments_are_rejected() {
        for s in ["\\begin{center}text", "\\begin{center}text\\end{quote}", "text}"] { assert!(validate_fragment(s).is_err()); }
    }
    #[test] fn complete_nested_environments_are_supported() { assert!(validate_fragment("\\begin{center}\\begin{tabular}{ll}A & B\\\\C & D\\end{tabular}\\end{center}").is_ok()); }
    #[test] fn escaped_braces_and_comments_are_supported() { assert!(validate_fragment("\\{Text\\} \\%\n% } {\nNext").is_ok()); }
    #[test] fn unicode_offsets_are_lossless() {
        let s = "A🧵العربية";
        assert_eq!(utf16_to_byte(s, 3).unwrap(), 5);
        assert_eq!(byte_to_utf16(s, 5).unwrap(), 3);
        assert!(utf16_to_byte(s, 2).is_err());
    }
    #[test] fn only_selected_text_leaves_flow() {
        let source = "\\documentclass{article}\n% \\usepackage{studio}\n\\begin{document}\nFirst paragraph.\n\nSecond paragraph.\n\\end{document}";
        let start = source.find("First").unwrap();
        let changed = insert_block(source, &new_block(), Some(start..start + 16)).unwrap();
        assert_eq!(changed.matches("First paragraph.").count(), 1);
        assert!(changed.contains("Second paragraph."));
        assert_eq!(changed.matches("\\usepackage{studio}").count(), 2);
    }
    #[test] fn visual_edits_cannot_insert_page_commands() { assert!(patch_block(SOURCE, "title", &[Change::Content("\\newpage text".into())]).is_err()); }
    #[test] fn numeric_values_must_be_finite() { assert!(patch_block(SOURCE, "title", &[Change::X(f64::NAN)]).is_err()); }
    #[test] fn plain_text_escapes_special_characters() { assert_eq!(escape_text("50% & a_b"), "50\\% \\& a\\_b"); }
}
