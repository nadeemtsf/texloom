//! Lossless LaTeX source edits. This crate intentionally has no dependencies.
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

/// Append a paragraph to the main document and return its exact source range.
pub fn append_plain_paragraph(source: &str, text: &str) -> Result<(String, Range<usize>)> {
    let text = text.trim();
    if text.is_empty() || text.len() > 256_000 ||
        text.chars().any(|ch| ch.is_control() || matches!(ch, '\u{2028}' | '\u{2029}'))
    {
        return Err("Enter one nonempty paragraph on a single line".into());
    }
    let (_, _, end) = document_bounds(source)?;
    let newline = if source.find('\n')
        .is_some_and(|offset| source[..offset].ends_with('\r')) { "\r\n" } else { "\n" };
    let content = escape_text(text);
    let start = end + 2 * newline.len();
    let range = start..start + content.len();
    let mut updated = source.to_string();
    updated.insert_str(end, &format!("{newline}{newline}{content}{newline}{newline}"));
    Ok((updated, range))
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
    #[test] fn plain_text_escapes_special_characters() { assert_eq!(escape_text("50% & a_b"), "50\\% \\& a\\_b"); }
}
