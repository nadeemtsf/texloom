//! Source-preserving text objects and deliberately conservative selection units.
use std::ops::Range;
use super::{argument, blocks, command, comment_end, document_bounds, has_package, number, skip_literal, skip_space, validate_fragment, Result};

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
    let mut objects = Vec::<TextObject>::new();
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
        if objects.iter().any(|o| o.id == id) { return Err("Object identifiers must be unique in each source file".into()); }
        let dimension = |n: usize| -> Result<f64> {
            let value = source[args[n].clone()].trim().parse::<f64>().map_err(|_| "Invalid object offset")?;
            number(value, false)?; Ok(value)
        };
        let content = source[args[3].clone()].to_string();
        if content.contains("\\TexloomObject") { return Err("Nested text objects are not supported".into()); }
        objects.push(TextObject { id: id.into(), dx: dimension(1)?, dy: dimension(2)?, content,
            range: i..cursor, arguments: args, line: source[..i].bytes().filter(|c| *c == b'\n').count() + 1 });
        i = cursor;
    }
    Ok(objects)
}

fn simple_definitions(source: &str) -> Vec<(String, String)> {
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
                    if ["\\begin", "\\end", "\\section", "\\subsection", "\\chapter", "\\part", "\\caption", "\\label", "\\ref", "\\pageref", "\\cite", "\\href", "\\url", "\\hyperlink", "\\hyperref", "\\footnote", "\\marginpar", "\\includegraphics", "\\newcommand", "\\renewcommand", "\\def", "\\gdef", "\\write", "\\special", "\\TexloomObject", "\\StudioBlock", "\\enlargethispage", "\\vspace", "\\hspace", "\\fontsize", "\\selectfont", "\\color", "\\bfseries", "\\itshape", "\\centering", "\\raggedright", "\\input", "\\include", "\\usepackage", "\\documentclass"].contains(&cmd) {
                        return Err(format!("{cmd} needs source editing or a dedicated layout adapter"));
                    }
                    let basic = ["\\textbf", "\\textit", "\\textsl", "\\textsc", "\\textsf", "\\textrm", "\\texttt", "\\emph", "\\underline", "\\textcolor", "\\mbox", "\\par", "\\\\", "\\%", "\\&", "\\$", "\\#", "\\_", "\\{", "\\}", "\\ ", "\\,", "\\;", "\\:", "\\!", "\\-", "\\'", "\\\"", "\\`", "\\^", "\\~", "\\c", "\\v", "\\H", "\\u", "\\b", "\\d", "\\r", "\\oe", "\\OE", "\\ae", "\\AE", "\\ss", "\\o", "\\O", "\\textbackslash", "\\textasciitilde", "\\textasciicircum", "\\LaTeX", "\\TeX", "\\ldots", "\\dots"];
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
    text.is_empty() || text.starts_with('%') || ["\\begin", "\\end", "\\section", "\\subsection", "\\chapter", "\\enlargethispage", "\\vspace", "\\noindent", "\\TexloomObject", "\\StudioBlock"].iter().any(|prefix| text.starts_with(prefix))
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
    if blocks(source).iter().any(|block| block.range.contains(&lines[index].start)) || text_objects(source)?.iter().any(|object| object.range.contains(&lines[index].start)) {
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

/// Plain editing is offered only when each logical line has a lossless style template.
pub fn plain_text_preview(fragment: &str, document: &str) -> Option<String> {
    let definitions=simple_definitions(document);
    forced_segments(fragment).ok()?.iter().map(|(range,_)|plain_segment(&fragment[range.clone()],&definitions,None,0)).collect::<Result<Vec<_>>>().ok().map(|lines|lines.join("\n"))
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
    #[test] fn soft_wrapped_prose_is_one_unit() { let u = resolve_text_unit(DOC, 5).unwrap(); assert_eq!(u.kind,"paragraph"); assert_eq!((u.first_line,u.last_line),(4,5)); }
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
