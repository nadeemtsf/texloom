//! Editing a literal run never serializes or rewrites its command scaffold.
use crate::{argument, command, comment_end, escape_text, validate_movable_text, Result};
use std::{collections::HashSet, ops::Range};

#[derive(Clone, Debug, PartialEq)]
pub struct LiteralRun {
    pub range: Range<usize>,
    pub text: String,
    pub styles: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct TextModel {
    pub runs: Vec<LiteralRun>,
    pub protected: Vec<String>,
}
#[derive(Clone, Debug)]
pub struct LiteralEdit {
    pub id: usize,
    pub text: String,
}
fn decode(s: &str) -> String {
    let mut chars = s.chars();
    let mut out = String::new();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(next)
            }
        } else {
            out.push(c)
        }
    }
    out
}
fn literal(s: &str, range: Range<usize>, styles: &[String], model: &mut TextModel) -> Result<()> {
    let raw = &s[range.clone()];
    let text = raw.trim();
    if text.is_empty() {
        return Ok(());
    }
    let start = range.start + raw.len() - raw.trim_start().len();
    if model.runs.len() >= 128 {
        return Err("This text has too many independently editable runs. Use LaTeX mode.".into());
    }
    model.runs.push(LiteralRun {
        range: start..start + text.len(),
        text: decode(text),
        styles: styles.to_vec(),
    });
    Ok(())
}
fn scan(
    s: &str,
    range: Range<usize>,
    styles: &[String],
    depth: usize,
    model: &mut TextModel,
) -> Result<()> {
    if depth > 32 {
        return Err("Text formatting nesting exceeds the supported limit".into());
    }
    let mut i = range.start;
    let mut start = i;
    while i < range.end {
        let c = s[i..].chars().next().ok_or("Invalid text boundary")?;
        if c == '\\' {
            let (cmd, end) = command(s, i);
            if ["\\%", "\\&", "\\$", "\\#", "\\_", "\\{", "\\}"].contains(&cmd) {
                i = end;
                continue;
            }
            literal(s, start..i, styles, model)?;
            let style = match cmd {
                "\\textbf" => Some("Bold"),
                "\\textit" | "\\emph" => Some("Italic"),
                "\\textsl" => Some("Slanted"),
                "\\textsc" => Some("Small caps"),
                "\\textsf" => Some("Sans serif"),
                "\\textrm" => Some("Roman"),
                "\\texttt" => Some("Monospace"),
                "\\underline" => Some("Underlined"),
                "\\mbox" => Some("Unbroken"),
                "\\textcolor" => Some("Coloured"),
                _ => None,
            };
            if model.protected.len() >= 1024 {
                return Err("Too many protected text commands. Use source editing.".into());
            }
            model.protected.push(cmd.into());
            if let Some(style) = style {
                let cursor = if cmd == "\\textcolor" {
                    argument(s, end)?.1
                } else {
                    end
                };
                let (body, after) = argument(s, cursor)?;
                if after > range.end {
                    return Err("Formatting argument escaped its source range".into());
                }
                let mut nested = styles.to_vec();
                nested.push(style.into());
                scan(s, body, &nested, depth + 1, model)?;
                i = after;
            } else {
                i = end;
                // Control sequences, their explicit arguments and optional
                // forced-break spacing remain protected, including macros.
                while s.as_bytes().get(i) == Some(&b'{') {
                    i = argument(s, i)?.1;
                }
                if cmd == "\\\\" && s.as_bytes().get(i) == Some(&b'[') {
                    i = s[i..]
                        .find(']')
                        .map(|n| i + n + 1)
                        .ok_or("Unclosed line spacing")?;
                }
            }
            start = i;
            continue;
        }
        if c == '{' {
            literal(s, start..i, styles, model)?;
            let (body, after) = argument(s, i)?;
            if after > range.end {
                return Err("Group escaped its source range".into());
            }
            scan(s, body, styles, depth + 1, model)?;
            i = after;
            start = i;
            continue;
        }
        if c == '%' || c == '~' {
            literal(s, start..i, styles, model)?;
            i = if c == '%' { comment_end(s, i) } else { i + 1 };
            if model.protected.len() >= 1024 {
                return Err("Too many protected text commands. Use source editing.".into());
            }
            model.protected.push(
                if c == '%' {
                    "Comment"
                } else {
                    "Nonbreaking space"
                }
                .into(),
            );
            start = i;
            continue;
        }
        i += c.len_utf8();
    }
    literal(s, start..range.end, styles, model)
}
pub fn describe(fragment: &str, document: &str) -> Result<TextModel> {
    if fragment.len() > 4_000_000 || document.len() > 4_000_000 {
        return Err("Text exceeds the supported limit".into());
    }
    validate_movable_text(fragment, document)?;
    let mut model = TextModel {
        runs: Vec::new(),
        protected: Vec::new(),
    };
    scan(fragment, 0..fragment.len(), &[], 0, &mut model)?;
    Ok(model)
}
pub fn apply(
    fragment: &str,
    expected: &str,
    document: &str,
    edits: &[LiteralEdit],
) -> Result<String> {
    if fragment != expected {
        return Err("The selected content changed. Reopen its editor.".into());
    }
    let model = describe(fragment, document)?;
    let mut seen = HashSet::new();
    let mut patches = Vec::new();
    if edits.len() > 128 || edits.iter().map(|e| e.text.len()).sum::<usize>() > 4_000_000 {
        return Err("Too many text edits".into());
    }
    for edit in edits {
        if !seen.insert(edit.id) {
            return Err("Duplicate literal edit".into());
        }
        let run = model
            .runs
            .get(edit.id)
            .ok_or("The text run no longer exists")?;
        if edit.text.len() > 4_000_000 || edit.text.contains('\0') {
            return Err("Invalid literal text".into());
        }
        if edit.text == run.text {
            continue;
        }
        let text = edit.text.replace("\r\n", "\n").replace('\r', "\n");
        let mut replacement = escape_text(&text);
        if fragment[run.range.clone()].contains("\r\n") {
            replacement = replacement.replace('\n', "\r\n")
        }
        patches.push((run.range.clone(), replacement));
    }
    patches.sort_by_key(|(range, _)| std::cmp::Reverse(range.start));
    let mut result = fragment.to_string();
    for (range, text) in patches {
        result.replace_range(range, &text)
    }
    if result.len() > 4_000_000 {
        return Err("Edited text exceeds the supported limit".into());
    }
    validate_movable_text(&result, document)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    const DOC:&str="\\documentclass{article}\n\\newcommand{\\Name}{Example}\n\\begin{document}\nText\n\\end{document}";
    #[test]
    fn nested_formatting_is_protected_while_words_remain_editable() {
        let s = "An \\textbf{office \\textit{file}} in \\textcolor{red}{Paris}.";
        let m = describe(s, DOC).unwrap();
        assert_eq!(
            m.runs.iter().map(|r| r.text.as_str()).collect::<Vec<_>>(),
            vec!["An", "office", "file", "in", "Paris", "."]
        );
        assert_eq!(m.runs[2].styles, vec!["Bold", "Italic"]);
        let result = apply(
            s,
            s,
            DOC,
            &[
                LiteralEdit {
                    id: 2,
                    text: "document".into(),
                },
                LiteralEdit {
                    id: 4,
                    text: "Lyon".into(),
                },
            ],
        )
        .unwrap();
        assert_eq!(
            result,
            "An \\textbf{office \\textit{document}} in \\textcolor{red}{Lyon}."
        );
    }
    #[test]
    fn macros_comments_breaks_and_spacing_are_untouched() {
        let s = "Hello \\Name % keep this comment\r\n\\\\[2pt]\r\nDepartment";
        let m = describe(s, DOC).unwrap();
        assert_eq!(m.runs.len(), 2);
        assert_eq!(m.runs[1].text, "Department");
        assert_eq!(
            apply(
                s,
                s,
                DOC,
                &[LiteralEdit {
                    id: 1,
                    text: "Faculty".into()
                }]
            )
            .unwrap(),
            s.replace("Department", "Faculty")
        );
    }
    #[test]
    fn escaped_plain_text_roundtrips_without_rewriting_an_unchanged_run() {
        let s = "R\\&D \\textbf{A\\_B}";
        let m = describe(s, DOC).unwrap();
        assert_eq!(m.runs[0].text, "R&D");
        assert_eq!(
            apply(
                s,
                s,
                DOC,
                &[LiteralEdit {
                    id: 0,
                    text: "R&D".into()
                }]
            )
            .unwrap(),
            s
        );
        assert_eq!(
            apply(
                s,
                s,
                DOC,
                &[LiteralEdit {
                    id: 1,
                    text: "C_{D}%".into()
                }]
            )
            .unwrap(),
            "R\\&D \\textbf{C\\_\\{D\\}\\%}"
        );
    }
    #[test]
    fn stale_duplicate_or_missing_run_edits_are_refused() {
        let edit = LiteralEdit {
            id: 0,
            text: "B".into(),
        };
        assert!(apply("A", "old", DOC, &[edit.clone()]).is_err());
        assert!(apply("A", "A", DOC, &[edit.clone(), edit]).is_err());
        assert!(apply(
            "A",
            "A",
            DOC,
            &[LiteralEdit {
                id: 4,
                text: "B".into()
            }]
        )
        .is_err());
    }
    #[test]
    fn unicode_byte_ranges_and_windows_line_endings_remain_exact() {
        let s = "🙂 قبل \\textbf{école\r\nParis} بعد";
        assert_eq!(
            apply(
                s,
                s,
                DOC,
                &[LiteralEdit {
                    id: 1,
                    text: "école\nLyon".into()
                }]
            )
            .unwrap(),
            "🙂 قبل \\textbf{école\r\nLyon} بعد"
        );
    }
    #[test]
    fn unsupported_structure_and_deep_nesting_fail_explicitly() {
        for s in ["\\href{x}{y}", "\\unknown{X}", "\\textbf{unclosed"] {
            assert!(describe(s, DOC).is_err())
        }
        assert!(describe(&format!("{}A{}", "{".repeat(40), "}".repeat(40)), DOC).is_err());
    }
    #[test]
    fn deletion_keeps_the_formatting_container() {
        assert_eq!(
            apply(
                "A \\textbf{B} C",
                "A \\textbf{B} C",
                DOC,
                &[LiteralEdit {
                    id: 1,
                    text: String::new()
                }]
            )
            .unwrap(),
            "A \\textbf{} C"
        )
    }
}
