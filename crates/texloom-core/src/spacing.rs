//! A narrow adapter for an existing explicit LaTeX line-break length.
//! Editing spacing permits reflow; it is distinct from drawing displacement.
use crate::{command, comment_end, skip_space, validate_movable_text, Result};
use std::{collections::HashSet, ops::Range};

#[derive(Clone, Debug)]
pub struct Parameter {
    pub range: Range<usize>,
    pub value: Option<f64>,
    pub unit: String,
    pub source: String,
    pub reason: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Edit {
    pub id: usize,
    pub value: String,
}

/// Return the bracket contents and the full command-tail end. Whitespace,
/// comments and the optional star remain outside every numeric replacement.
pub(crate) fn break_argument(source: &str, end: usize) -> Result<Option<(Range<usize>, usize)>> {
    let mut i = end;
    if source.as_bytes().get(i) == Some(&b'*') {
        i += 1;
    }
    i = skip_space(source, i);
    if source.as_bytes().get(i) != Some(&b'[') {
        return Ok(None);
    }
    let close = source[i + 1..]
        .find(']')
        .map(|n| i + 1 + n)
        .ok_or("Unclosed line-break spacing")?;
    Ok(Some((i + 1..close, close + 1)))
}
fn decimal(s: &str) -> Result<f64> {
    let s = s.trim();
    if s.is_empty() || s.len() > 64 {
        return Err("Enter a decimal spacing value".into());
    }
    let digits = s.strip_prefix(['+', '-']).unwrap_or(s);
    if !digits.bytes().any(|b| b.is_ascii_digit())
        || digits.bytes().any(|b| !b.is_ascii_digit() && b != b'.')
        || digits.bytes().filter(|b| *b == b'.').count() > 1
    {
        return Err("Use a decimal number, without commands or expressions".into());
    }
    s.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .ok_or("Spacing must be finite".into())
}
fn factor(unit: &str) -> Option<f64> {
    match unit {
        "bp" => Some(1.0),
        "pt" => Some(72.0 / 72.27),
        "mm" => Some(72.0 / 25.4),
        "cm" => Some(72.0 / 2.54),
        "in" => Some(72.0),
        _ => None,
    }
}
fn parameter(source: &str, range: Range<usize>) -> Parameter {
    let raw = &source[range.clone()];
    let start = range.start + raw.len() - raw.trim_start().len();
    let text = raw.trim();
    let numeric = text
        .bytes()
        .take_while(|b| b.is_ascii_digit() || matches!(b, b'+' | b'-' | b'.'))
        .count();
    let number = &text[..numeric];
    let unit = text[numeric..].trim();
    let parsed = decimal(number)
        .ok()
        .filter(|v| factor(unit).is_some_and(|scale| (v * scale).abs() <= 400.0));
    Parameter {
        range: start..start + numeric,
        value: parsed,
        unit: unit.into(),
        source: raw.into(),
        reason: if parsed.is_some() {
            None
        } else {
            Some("This length is defined by LaTeX or exceeds the supported range. Edit it in source mode.".into())
        },
    }
}
pub fn describe(fragment: &str, document: &str) -> Result<Vec<Parameter>> {
    if fragment.len() > 4_000_000 || document.len() > 4_000_000 {
        return Err("Text exceeds the supported limit".into());
    }
    validate_movable_text(fragment, document)?;
    let mut result = Vec::new();
    let mut i = 0;
    while i < fragment.len() {
        match fragment.as_bytes()[i] {
            b'%' => {
                i = comment_end(fragment, i);
                continue;
            }
            b'\\' => {
                let (cmd, end) = command(fragment, i);
                i = end;
                if cmd == "\\\\" {
                    if let Some((range, after)) = break_argument(fragment, end)? {
                        if result.len() >= 64 {
                            return Err("Too many spacing parameters in this selection".into());
                        }
                        result.push(parameter(fragment, range));
                        i = after;
                    }
                }
                continue;
            }
            _ => i += 1,
        }
    }
    Ok(result)
}
pub fn apply(fragment: &str, expected: &str, document: &str, edits: &[Edit]) -> Result<String> {
    if fragment != expected {
        return Err("The selected content changed. Reopen its editor.".into());
    }
    if edits.len() > 64 {
        return Err("Too many spacing edits".into());
    }
    let parameters = describe(fragment, document)?;
    let mut seen = HashSet::new();
    let mut patches = Vec::new();
    for edit in edits {
        if !seen.insert(edit.id) {
            return Err("Duplicate spacing edit".into());
        }
        let p = parameters
            .get(edit.id)
            .ok_or("The spacing parameter no longer exists")?;
        let old = p
            .value
            .ok_or("This spacing expression requires source editing")?;
        let value = decimal(&edit.value)?;
        if (value * factor(&p.unit).ok_or("Unsupported spacing unit")?).abs() > 400.0 {
            return Err("Keep explicit line spacing within 400 PDF points".into());
        }
        if value != old {
            patches.push((p.range.clone(), value.to_string()))
        }
    }
    patches.sort_by_key(|(r, _)| std::cmp::Reverse(r.start));
    let mut result = fragment.to_string();
    for (range, value) in patches {
        result.replace_range(range, &value)
    }
    validate_movable_text(&result, document)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    const DOC: &str =
        "\\documentclass{article}\n\\newcommand{\\Gap}{2pt}\n\\begin{document}X\\end{document}";
    #[test]
    fn only_numeric_parameters_change_and_every_other_byte_survives() {
        let source =
            "🙂 \\textbf{Name}\\\\*[ +02.00 pt ]% keep\r\nDepartment\\\\[3mm]\r\nUniversity";
        assert_eq!(
            apply(
                source,
                source,
                DOC,
                &[
                    Edit {
                        id: 0,
                        value: "6.5".into()
                    },
                    Edit {
                        id: 1,
                        value: "4".into()
                    }
                ]
            )
            .unwrap(),
            source.replace("+02.00", "6.5").replace("3mm", "4mm")
        );
        assert_eq!(
            apply(
                source,
                source,
                DOC,
                &[Edit {
                    id: 0,
                    value: "2".into()
                }]
            )
            .unwrap(),
            source
        );
    }
    #[test]
    fn comments_escapes_and_default_breaks_are_not_parameters() {
        let model = describe(
            "% \\\\[99pt]\nA\\\\\nB \\textbackslash{}[8pt]\\\\ %keep\n [2bp]C",
            DOC,
        )
        .unwrap();
        assert_eq!(model.len(), 1);
        assert_eq!(model[0].value, Some(2.0));
    }
    #[test]
    fn macro_font_relative_and_glue_lengths_are_read_only() {
        for length in ["\\Gap", "1em", "2pt plus 1pt"] {
            let s = format!("A\\\\[{length}]B");
            let p = describe(&s, DOC).unwrap();
            assert_eq!(p[0].value, None);
            assert!(apply(
                &s,
                &s,
                DOC,
                &[Edit {
                    id: 0,
                    value: "4".into()
                }]
            )
            .is_err());
        }
    }
    #[test]
    fn stale_duplicate_nonfinite_injected_and_excessive_values_fail() {
        let s = "A\\\\[2pt]B";
        assert!(apply(s, "old", DOC, &[]).is_err());
        for value in ["NaN", "inf", "1e3", "2pt", "3}\\input{x}", "10000"] {
            assert!(
                apply(
                    s,
                    s,
                    DOC,
                    &[Edit {
                        id: 0,
                        value: value.into()
                    }]
                )
                .is_err(),
                "{value}"
            )
        }
        assert!(apply(
            s,
            s,
            DOC,
            &[
                Edit {
                    id: 0,
                    value: "3".into()
                },
                Edit {
                    id: 0,
                    value: "4".into()
                }
            ]
        )
        .is_err());
        assert!(apply(
            s,
            s,
            DOC,
            &[Edit {
                id: 2,
                value: "3".into()
            }]
        )
        .is_err());
    }
    #[test]
    fn negative_and_zero_spacing_keep_the_existing_unit() {
        let s = "A\\\\[-2mm]B";
        assert_eq!(
            apply(
                s,
                s,
                DOC,
                &[Edit {
                    id: 0,
                    value: "0".into()
                }]
            )
            .unwrap(),
            "A\\\\[0mm]B"
        )
    }
    #[test]
    fn no_parameter_is_invented_for_a_default_line_break() {
        assert!(describe("A\\\\B", DOC).unwrap().is_empty())
    }
}
