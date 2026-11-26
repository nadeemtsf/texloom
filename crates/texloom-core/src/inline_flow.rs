use crate::{
    argument, command, document_bounds, escape_text, objects::simple_definitions, skip_literal,
    Result,
};
use std::{collections::BTreeSet, ops::Range};

const TAB: &str = r"\mbox{\ \ \ \ }";
const EMPTY_LINE: &str = r"\mbox{}";

#[derive(Clone, Debug)]
struct Piece {
    source: Range<usize>,
    text: String,
    style: Option<usize>,
}

#[derive(Debug)]
pub struct InlineFlow {
    pub range: Range<usize>,
    pub text: String,
    pieces: Vec<Piece>,
    hard_lines: bool,
}

fn line_break(source: &str, end: usize) -> Option<usize> {
    let mut cursor = end;
    if source.as_bytes().get(cursor) == Some(&b'*') {
        cursor += 1;
    }
    if let Some(options) = source.get(cursor..)?.strip_prefix('[') {
        let close = options.find(']')?;
        let length = &options[..close];
        if length.contains(['\n', '\r', '{', '}', '%', '\\']) {
            return None;
        }
        cursor += close + 2;
    }
    let remaining = source.get(cursor..)?.trim_start_matches([' ', '\t']);
    let newline = if remaining.starts_with("\r\n") {
        2
    } else if remaining.starts_with('\n') {
        1
    } else {
        0
    };
    Some(if newline == 0 {
        cursor
    } else {
        source.len() - remaining[newline..].trim_start_matches([' ', '\t']).len()
    })
}

fn push(pieces: &mut Vec<Piece>, source: Range<usize>, text: impl Into<String>) {
    pieces.push(Piece {
        source,
        text: text.into(),
        style: None,
    });
}

fn parse(
    source: &str,
    range: Range<usize>,
    definitions: &[(String, String)],
    pieces: &mut Vec<Piece>,
    depth: usize,
) -> Option<()> {
    if depth > 16 {
        return None;
    }
    let mut cursor = range.start;
    while cursor < range.end {
        let rest = &source[cursor..range.end];
        let ch = rest.chars().next()?;
        if rest.starts_with("{}") {
            cursor += 2;
            continue;
        }
        let mut end = cursor + ch.len_utf8();
        if ch == '\\' {
            let (name, after) = command(source, cursor);
            if after > range.end {
                return None;
            }
            end = after;
            let text = match name {
                "\\\\" => {
                    end = line_break(&source[..range.end], after)?;
                    "\n"
                }
                "\\textbf" | "\\textit" | "\\textsl" | "\\textsc" | "\\textsf" | "\\textrm"
                | "\\texttt" | "\\emph" | "\\underline" => {
                    let (inner, after_arg) = argument(source, after).ok()?;
                    if after_arg > range.end || source[after..inner.start].contains('%') {
                        return None;
                    }
                    let first_piece = pieces.len();
                    parse(source, inner, definitions, pieces, depth + 1)?;
                    for piece in &mut pieces[first_piece..] {
                        piece.style.get_or_insert(cursor);
                    }
                    cursor = after_arg;
                    continue;
                }
                "\\mbox" if rest.starts_with(TAB) => {
                    end = cursor + TAB.len();
                    "\t"
                }
                "\\mbox" if rest.starts_with(EMPTY_LINE) => {
                    end = cursor + EMPTY_LINE.len();
                    ""
                }
                "\\%" => "%",
                "\\&" => "&",
                "\\$" => "$",
                "\\#" => "#",
                "\\_" => "_",
                "\\{" => "{",
                "\\}" => "}",
                "\\ " => " ",
                "\\textbackslash" | "\\textasciitilde" | "\\textasciicircum" | "\\textless"
                | "\\textgreater" | "\\textquotesingle" | "\\textasciigrave"
                    if source[after..range.end].starts_with("{}") =>
                {
                    end += 2;
                    match name {
                        "\\textbackslash" => "\\",
                        "\\textasciitilde" => "~",
                        "\\textless" => "<",
                        "\\textgreater" => ">",
                        "\\textquotesingle" => "'",
                        "\\textasciigrave" => "`",
                        _ => "^",
                    }
                }
                _ => {
                    let (_, body) = definitions.iter().rev().find(|(key, _)| key == name)?;
                    if body.is_empty()
                        || body.trim() != body
                        || body
                            .chars()
                            .any(|c| c.is_control() || "\\{}%&#_$~^".contains(c))
                    {
                        return None;
                    }
                    end = range.end
                        - source[after..range.end]
                            .trim_start_matches([' ', '\t', '\r', '\n'])
                            .len();
                    body
                }
            };
            if end > range.end {
                return None;
            }
            push(pieces, cursor..end, text);
        } else if ch.is_ascii_whitespace() {
            end = range.end
                - rest
                    .trim_start_matches(|c: char| c.is_ascii_whitespace())
                    .len();
            let text = if source[cursor..end]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                > 1
            {
                "\n"
            } else {
                " "
            };
            push(pieces, cursor..end, text);
        } else {
            if ch.is_control() || "{}%&#_$~^".contains(ch) {
                return None;
            }
            push(pieces, cursor..end, ch);
        }
        cursor = end;
    }
    Some(())
}

fn brace_delta(line: &str) -> isize {
    let mut depth = 0;
    let mut cursor = 0;
    while cursor < line.len() {
        match line.as_bytes()[cursor] {
            b'%' => break,
            b'\\' => {
                cursor = command(line, cursor).1;
                continue;
            }
            b'{' => depth += 1,
            b'}' => depth -= 1,
            _ => (),
        }
        cursor += 1;
    }
    depth
}

fn append_scope(
    source: &str,
    range: Range<usize>,
    defs: &[(String, String)],
    hard_lines: bool,
    flows: &mut Vec<InlineFlow>,
) {
    let mut run: Option<InlineFlow> = None;
    let mut cursor = range.start;
    let mut block_start = cursor;
    let mut depth = 0;
    for line in source[range].split_inclusive('\n') {
        let end = cursor + line.len();
        depth += brace_delta(line);
        cursor = end;
        if depth > 0 {
            continue;
        }
        depth = 0;
        let start = block_start;
        let block = &source[start..end];
        block_start = end;
        if block.trim().is_empty() {
            continue;
        }
        // Consecutive physical lines share a paragraph unless a blank source line separates them.
        let leading = block.len() - block.trim_start().len();
        let trimmed = block.trim_end();
        let escaped_space = block[trimmed.len()..].starts_with(' ')
            && trimmed
                .bytes()
                .rev()
                .take_while(|byte| *byte == b'\\')
                .count()
                % 2
                == 1;
        let end = start + trimmed.len() + usize::from(escaped_space);
        let start = start + leading;
        let mut pieces = Vec::new();
        if parse(source, start..end, defs, &mut pieces, 0).is_none() || pieces.is_empty() {
            flows.extend(run.take());
            continue;
        }
        let text: String = pieces.iter().map(|piece| piece.text.as_str()).collect();
        if let Some(mut previous) = run.take() {
            let gap = &source[previous.range.end..start];
            if gap.trim().is_empty() {
                let follows_break = previous.text.ends_with('\n')
                    && previous
                        .pieces
                        .last()
                        .is_some_and(|piece| piece.text == "\n");
                if follows_break {
                    if let Some(last) = previous.pieces.last_mut() {
                        last.source.end = start;
                    }
                } else {
                    let separator = if gap.bytes().filter(|&b| b == b'\n').count() > 1 {
                        "\n"
                    } else {
                        " "
                    };
                    push(&mut previous.pieces, previous.range.end..start, separator);
                    previous.text.push_str(separator);
                }
                previous.range.end = end;
                previous.pieces.extend(pieces);
                previous.text.push_str(&text);
                run = Some(previous);
                continue;
            }
            flows.push(previous);
        }
        run = Some(InlineFlow {
            range: start..end,
            text,
            pieces,
            hard_lines,
        });
    }
    flows.extend(run);
}

fn environment_end(source: &str, mut cursor: usize, name: &str) -> Option<(usize, usize)> {
    if ["verbatim", "verbatim*", "Verbatim", "lstlisting", "minted"].contains(&name) {
        let closing = format!("\\end{{{name}}}");
        return source[cursor..].find(&closing).map(|offset| {
            let close = cursor + offset;
            (close, close + closing.len())
        });
    }
    let mut nesting = 0;
    while cursor < source.len() {
        let offset = source[cursor..].find(['\\', '%'])?;
        cursor += offset;
        if source.as_bytes()[cursor] == b'%' {
            cursor += source[cursor..].find('\n')? + 1;
            continue;
        }
        let (cmd, next) = command(source, cursor);
        if let Some(after) = skip_literal(source, cmd, next) {
            cursor = after;
            continue;
        }
        if matches!(cmd, "\\begin" | "\\end") {
            if let Ok((arg, after)) = argument(source, next) {
                if &source[arg] == name {
                    if cmd == "\\begin" {
                        nesting += 1;
                    } else if nesting == 0 {
                        return Some((cursor, after));
                    } else {
                        nesting -= 1;
                    }
                }
                cursor = after;
                continue;
            }
        }
        cursor = next;
    }
    None
}

pub fn inline_flows(source: &str) -> Vec<InlineFlow> {
    let Ok((_, start, end)) = document_bounds(source) else {
        return Vec::new();
    };
    flows_in(source,start..end,&simple_definitions(source))
}

fn flows_in(source: &str, range: Range<usize>, definitions: &[(String,String)]) -> Vec<InlineFlow> {
    let Range { start, end }=range;
    let mut flows = Vec::new();
    let mut scope = start;
    let mut cursor = start;
    // Skip unsupported environments as a unit, including text on internal lines.
    while cursor < end {
        let rest = &source[cursor..end];
        let ch = rest.chars().next().unwrap();
        if ch == '%' {
            cursor = rest.find('\n').map_or(end, |length| cursor + length + 1);
            continue;
        }
        if ch != '\\' {
            cursor += ch.len_utf8();
            continue;
        }
        let (name, next) = command(source, cursor);
        if name != "\\begin" {
            cursor = next;
            continue;
        }
        let Ok((env, body)) = argument(source, next) else {
            break;
        };
        let env = &source[env];
        if body > end {
            break;
        }
        let Some((close, after)) = environment_end(&source[..end], body, env) else {
            break;
        };
        let prelude = source[scope..cursor].trim_end();
        let before = prelude
            .strip_suffix(r"\noindent")
            .map_or(cursor, |prefix| scope + prefix.len());
        append_scope(source, scope..before, definitions, false, &mut flows);
        if env == "minipage" {
            let args = source[body..close].trim_start();
            let mut width = close - args.len();
            let mut valid = true;
            for _ in 0..3 {
                if source[width..close].starts_with('[') {
                    let Some(after) = source[width..close].find(']') else {
                        valid = false;
                        break;
                    };
                    width += after + 1;
                    width = close - source[width..close].trim_start().len();
                }
            }
            if valid {
                if let Ok((_, inner)) = argument(source, width) {
                    if inner <= close {
                        append_scope(source, inner..close, definitions, true, &mut flows);
                    }
                }
            }
        }
        cursor = after;
        scope = cursor;
    }
    if cursor == end {
        append_scope(source, scope..end, definitions, false, &mut flows);
    }
    let fits=|flow: &InlineFlow| !flow.text.trim().is_empty()
        && flow.text.len()<=4096 && flow.text.split('\n').count()<=24;
    flows.into_iter().flat_map(|flow| {
        if fits(&flow) { vec![flow] }
        else { inline_paragraphs(source,&flow).into_iter().filter(fits).collect() }
    }).collect()
}

pub fn inline_paragraphs(source: &str, flow: &InlineFlow) -> Vec<InlineFlow> {
    if flow.hard_lines { return Vec::new(); }
    let cuts:Vec<_>=flow.pieces.iter().enumerate().filter(|(_,piece)|
        piece.text=="\n" && source[piece.source.clone()].trim().is_empty()).collect();
    if cuts.is_empty() {return Vec::new()}
    let (mut start,mut first)=(flow.range.start,0);
    cuts.into_iter().map(|(index,piece)|(index,piece.source.start,piece.source.end))
        .chain(std::iter::once((flow.pieces.len(),flow.range.end,flow.range.end)))
        .filter_map(|(last,end,next)|{
        let range=start..end;
        let pieces=&flow.pieces[first..last];
        start=next;
        first=last+1;
        let text: String=pieces.iter().map(|piece|piece.text.as_str()).collect();
        (!text.trim().is_empty() && text.len()<=4096 && !text.contains('\n'))
            .then(||InlineFlow{range,text,pieces:pieces.to_vec(),hard_lines:false})
    }).collect()
}

/// Recover a contiguous sequence of complete paragraphs from one parsed flow.
/// The requested edges must be paragraph edges, never the middle of a token.
pub fn inline_paragraph_range(source: &str, flow: &InlineFlow, range: Range<usize>) -> Option<InlineFlow> {
    let paragraphs = inline_paragraphs(source, flow);
    let first = paragraphs.iter().position(|part| part.range.start == range.start)?;
    let last = paragraphs.iter().position(|part| part.range.end == range.end)?;
    let parts = paragraphs.get(first..=last)?;
    let pieces: Vec<_> = flow.pieces.iter()
        .filter(|piece| piece.source.start >= range.start && piece.source.end <= range.end)
        .cloned().collect();
    let text: String = pieces.iter().map(|piece| piece.text.as_str()).collect();
    // A skipped or oversized paragraph must not disappear inside a selection.
    let joined = parts.iter().map(|part| part.text.as_str()).collect::<Vec<_>>().join("\n");
    (text == joined && text.len() <= 4096 && parts.len() <= 24)
        .then_some(InlineFlow { range, text, pieces, hard_lines: false })
}

fn literal_inputs<'a>(source: &str, range: Range<usize>, sources: &'a crate::document::Sources) -> Vec<(usize,&'a str)> {
    let mut inputs=Vec::new();
    let mut cursor=range.start;
    let body=&source[..range.end];
    let (mut groups,mut conditions)=(0usize,0usize);
    while cursor<range.end {
        match body.as_bytes()[cursor] {
            b'%' => { cursor=crate::comment_end(body,cursor); continue; }
            b'{' => groups+=1,
            b'}' => groups=groups.saturating_sub(1),
            b'\\' => {
                let start=cursor;
                let (name,after)=command(body,cursor);
                cursor=after;
                if let Some(after)=skip_literal(body,name,after) {cursor=after;continue}
                if name.starts_with("\\if") { conditions+=1; continue; }
                if name=="\\fi" { conditions=conditions.saturating_sub(1); continue; }
                if groups!=0 || conditions!=0 { continue; }
                if name=="\\begin" {
                    let Ok((env,inner))=argument(body,after) else { break };
                    let Some((_,end))=environment_end(body,inner,&body[env]) else { break };
                    cursor=end;
                } else if matches!(name,"\\input"|"\\include") {
                    let Ok((path,end))=argument(body,after) else { continue };
                    let path=&body[path];
                    cursor=end;
                    let line_start=body[..start].rfind('\n').map_or(0,|offset|offset+1);
                    let remainder=body[end..].split('\n').next().unwrap_or("").trim();
                    if !body[line_start..start].trim().is_empty() ||
                        (!remainder.is_empty()&&!remainder.starts_with('%')) {continue}
                    if !path.split('/').all(|part| !matches!(part,""|"."|"..") &&
                        part.bytes().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'_'|b'-'|b'.'))) {continue}
                    let file=if path.ends_with(".tex") {path.to_owned()} else {format!("{path}.tex")};
                    if let Some((file,_))=sources.get_key_value(&file) {inputs.push((start,file.as_str()))}
                }
                continue;
            }
            _=>(),
        }
        cursor+=1;
    }
    inputs
}

/// Literal project-local includes contribute their own byte ranges in document
/// order. A reused input and its descendants have no single editable occurrence.
pub fn project_inline_flows<'a>(sources: &'a crate::document::Sources, main: &str) -> Vec<(&'a str,InlineFlow)> {
    fn walk<'a>(file:&'a str,range:Range<usize>,sources:&'a crate::document::Sources,defs:&[(String,String)],
                seen:&mut BTreeSet<&'a str>,reused:&mut BTreeSet<&'a str>,out:&mut Vec<(&'a str,InlineFlow)>) {
        if !seen.insert(file) {reused.insert(file);return}
        let source=&sources[file];
        let mut flows=flows_in(source,range.clone(),defs).into_iter().peekable();
        for (position,input) in literal_inputs(source,range,sources) {
            while flows.peek().is_some_and(|flow|flow.range.start<position) {
                out.push((file,flows.next().unwrap()));
            }
            walk(input,0..sources[input].len(),sources,defs,seen,reused,out);
        }
        out.extend(flows.map(|flow|(file,flow)));
    }
    let Some((main,document))=sources.get_key_value(main) else {return Vec::new()};
    let Ok((_,start,end))=document_bounds(document) else {return Vec::new()};
    let mut seen=BTreeSet::new();
    let mut reused=BTreeSet::new();
    let mut out=Vec::new();
    walk(main,start..end,sources,&simple_definitions(document),&mut seen,&mut reused,&mut out);
    let mut pending:Vec<_>=reused.iter().copied().collect();
    while let Some(file)=pending.pop() {
        for (_,child) in literal_inputs(&sources[file],0..sources[file].len(),sources) {
            if reused.insert(child) {pending.push(child)}
        }
    }
    out.retain(|(file,_)|!reused.contains(file));
    out
}

#[test]
fn project_flows_preserve_source_order_and_refuse_reused_subtrees() {
    let mut sources=crate::document::Sources::from([
        ("main.tex".into(), "\\begin{document}\nBefore.\n\\input{sections/chapter}\nAfter.\n\\end{document}\n".into()),
        ("sections/chapter.tex".into(), "\\section{Chapter}\r\nFirst paragraph.\r\n\r\nSecond paragraph.\r\n\\input{sections/detail.tex}\r\n".into()),
        ("sections/detail.tex".into(), "A separate source.\n".into()),
        ("unused.tex".into(), "This source is not included.\n".into()),
    ]);
    let flows=project_inline_flows(&sources,"main.tex");
    assert_eq!(flows.iter().map(|(file,flow)|(*file,flow.text.as_str())).collect::<Vec<_>>(),vec![
        ("main.tex","Before."),("sections/chapter.tex","First paragraph.\nSecond paragraph."),
        ("sections/detail.tex","A separate source."),("main.tex","After."),
    ]);
    let parts=inline_paragraphs(&sources["sections/chapter.tex"],&flows[1].1);
    assert_eq!(&sources["sections/chapter.tex"][parts[1].range.clone()],"Second paragraph.");
    let main=sources.get_mut("main.tex").unwrap();
    *main=main.replace("After.","\\input{sections/chapter}\nAfter.");
    let files:Vec<_>=project_inline_flows(&sources,"main.tex").into_iter().map(|(file,_)|file).collect();
    assert_eq!(files,vec!["main.tex","main.tex"]);
}

#[test]
fn adjacent_paragraphs_can_join_in_a_partial_flow() {
    let source = "\\begin{document}\r\nEarlier.\r\n\r\nFirst.\r\n\r\nSecond.\r\n\r\nLater.\r\n\\end{document}";
    let flow = &inline_flows(source)[0];
    let start = source.find("First.").unwrap();
    let end = source.find("Second.").unwrap() + "Second.".len();
    assert!(inline_paragraph_range(source, flow, start + 1..end).is_none());
    let joined = inline_paragraph_range(source, flow, start..end).unwrap();
    assert_eq!(joined.text, "First.\nSecond.");
    let (updated, _) = edit_inline_flow(source, &joined, "First.Second.").unwrap();
    assert_eq!(updated, source.replace("First.\r\n\r\nSecond.", "First.Second."));
}

fn encode(text: &str, hard_lines: bool, line_ending: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    let mut previous = None;
    while let Some(ch) = chars.next() {
        match ch {
            '\n' => {
                if matches!(previous, Some('\n')) {
                    result.push_str(EMPTY_LINE);
                }
                if hard_lines {
                    result.push_str(r"\\");
                } else {
                    result.push_str(line_ending);
                }
                result.push_str(line_ending);
            }
            '\t' => result.push_str(TAB),
            ' ' if previous.is_none() || previous == Some(' ') || chars.peek().is_none() => {
                result.push_str(r"\ ")
            }
            '<' => result.push_str(r"\textless{}"),
            '>' => result.push_str(r"\textgreater{}"),
            '\'' => result.push_str(r"\textquotesingle{}"),
            '`' => result.push_str(r"\textasciigrave{}"),
            '-' => {
                // An empty group prevents TeX's punctuation ligatures (e.g. double hyphen).
                result.push_str("{}");
                result.push(ch);
                result.push_str("{}");
            }
            _ => result.push_str(&escape_text(&ch.to_string())),
        }
        previous = Some(ch);
    }
    result
}

fn differences<'a>(previous: &str, draft: &'a str) -> Vec<(Range<usize>, &'a str)> {
    let front: usize = previous
        .chars()
        .zip(draft.chars())
        .take_while(|(a, b)| a == b)
        .map(|(ch, _)| ch.len_utf8())
        .sum();
    let back: usize = previous[front..]
        .chars()
        .rev()
        .zip(draft[front..].chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(ch, _)| ch.len_utf8())
        .sum();
    let before: Vec<_> = previous[front..previous.len() - back]
        .char_indices()
        .collect();
    let after: Vec<_> = draft[front..draft.len() - back].char_indices().collect();

    // A bounded longest common subsequence keeps untouched source tokens between
    // disjoint edits, including the original macro uses and exact break spelling.
    let stride = after.len() + 1;
    let mut lengths = vec![0u16; (before.len() + 1) * stride];
    for (i, &(_, left)) in before.iter().enumerate().rev() {
        for (j, &(_, right)) in after.iter().enumerate().rev() {
            let slot = i * stride + j;
            lengths[slot] = if left == right {
                lengths[slot + stride + 1] + 1
            } else {
                lengths[slot + stride].max(lengths[slot + 1])
            };
        }
    }
    let left_offset = |i: usize| {
        before
            .get(i)
            .map_or(previous.len() - back, |entry| front + entry.0)
    };
    let right_offset = |i: usize| {
        after
            .get(i)
            .map_or(draft.len() - back, |entry| front + entry.0)
    };
    let (mut i, mut j) = (0, 0);
    let (mut left, mut right) = (0, 0);
    let mut edits = Vec::new();
    while i < before.len() || j < after.len() {
        if i < before.len() && j < after.len() && before[i].1 == after[j].1 {
            if i != left || j != right {
                edits.push((
                    left_offset(left)..left_offset(i),
                    &draft[right_offset(right)..right_offset(j)],
                ));
            }
            i += 1;
            j += 1;
            (left, right) = (i, j);
        } else if i < before.len()
            && (j == after.len() || lengths[(i + 1) * stride + j] >= lengths[i * stride + j + 1])
        {
            i += 1;
        } else {
            j += 1;
        }
    }
    if i != left || j != right {
        edits.push((
            left_offset(left)..left_offset(i),
            &draft[right_offset(right)..right_offset(j)],
        ));
    }
    edits
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum ProtectedSpan {
    Style(usize),
    Token(usize),
}

fn protected_spans(
    source: &str,
    flow: &InlineFlow,
) -> Vec<(Option<ProtectedSpan>, Option<ProtectedSpan>, Range<usize>)> {
    let mut at = 0;
    flow.pieces
        .iter()
        .map(|piece| {
            let next = at + piece.text.len();
            let span = (
                piece.style.map(ProtectedSpan::Style),
                (source.as_bytes()[piece.source.start] == b'\\' || piece.text == "\n")
                    .then_some(ProtectedSpan::Token(piece.source.start)),
                at..next,
            );
            at = next;
            span
        })
        .collect()
}

fn protected_edits(
    spans: &[(Option<ProtectedSpan>, Option<ProtectedSpan>, Range<usize>)],
    edits: impl IntoIterator<Item = Range<usize>>,
) -> BTreeSet<ProtectedSpan> {
    let mut protected = BTreeSet::new();
    for edit in edits {
        for (style, token, span) in spans {
            let insertion = edit.is_empty()
                && (span.contains(&edit.start) || span.end == edit.start && !span.is_empty());
            if insertion || span.start < edit.end && edit.start < span.end {
                protected.extend(style);
                protected.extend(token);
                if edit.is_empty() {
                    break;
                }
            }
            if span.start > edit.end || span.start == edit.end && !edit.is_empty() {
                break;
            }
        }
    }
    protected
}

fn ambiguous_protected_edit(
    source: &str,
    flow: &InlineFlow,
    draft: &str,
    edits: &[(Range<usize>, &str)],
) -> bool {
    let spans = protected_spans(source, flow);
    if !spans
        .iter()
        .any(|(style, token, _)| style.is_some() || token.is_some())
    {
        return false;
    }
    let reverse_before: String = flow.text.chars().rev().collect();
    let reverse_after: String = draft.chars().rev().collect();
    let alternate = differences(&reverse_before, &reverse_after);
    let len = flow.text.len();
    let forward = protected_edits(&spans, edits.iter().map(|(range, _)| range.clone()));
    let reverse = protected_edits(
        &spans,
        alternate
            .into_iter()
            .rev()
            .map(|(range, _)| len - range.end..len - range.start),
    );
    forward != reverse
}

fn separates_control_word(source: &str, range: &Range<usize>) -> bool {
    let letter = |byte: u8| byte.is_ascii_alphabetic() || byte == b'@';
    if !source
        .as_bytes()
        .get(range.end)
        .is_some_and(|byte| letter(*byte))
    {
        return false;
    }
    let prefix = &source.as_bytes()[..range.start];
    let name = prefix
        .iter()
        .rev()
        .take_while(|byte| letter(**byte))
        .count();
    name > 0
        && prefix[..prefix.len() - name]
            .iter()
            .rev()
            .take_while(|byte| **byte == b'\\')
            .count()
            % 2
            == 1
}

/// Applies display edits while leaving source-only formatting and untouched separators in place.
pub fn edit_inline_flow(
    source: &str,
    flow: &InlineFlow,
    draft: &str,
) -> Result<(String, Range<usize>)> {
    if draft.len() > 4096
        || draft.split('\n').count() > 24
        || draft
            .chars()
            .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\t'))
        || draft == flow.text
    {
        return Err("Enter supported inline text".into());
    }
    let edits = differences(&flow.text, draft);
    if ambiguous_protected_edit(source, flow, draft, &edits) {
        return Err("The edit matches more than one formatted passage. Select more text or edit the source.".into());
    }
    let line_ending = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut patches = Vec::new();
    let mut at = 0;
    let mut edit_index = 0;
    for piece in &flow.pieces {
        let next = at + piece.text.len();
        let original = &source[piece.source.clone()];
        let mut value = String::new();
        let mut written = 0;
        let mut changed = false;
        let mut preserve_boundary = false;
        let mut starts_with_empty_line = false;
        let mut ends_with_empty_line = false;
        while let Some((range, insert)) = edits.get(edit_index) {
            let boundary_insertion = range.is_empty() && range.start == next && next > at;
            if range.start >= next && !boundary_insertion {
                break;
            }
            let left = range.start.saturating_sub(at);
            let right = range.end.saturating_sub(at).min(piece.text.len());
            preserve_boundary = boundary_insertion
                && !changed
                && !(original.starts_with('\\') && piece.text.chars().count() > 1);
            if !preserve_boundary {
                value.push_str(&piece.text[written..left]);
            }
            if range.start >= at {
                starts_with_empty_line |= (range.start == 0
                    || flow.text[..range.start].ends_with('\n'))
                    && insert.starts_with('\n');
                ends_with_empty_line |= range.end == flow.text.len() && insert.ends_with('\n');
                value.push_str(insert);
            }
            written = right;
            changed = true;
            if range.end > next {
                break;
            }
            edit_index += 1;
        }
        if changed {
            value.push_str(&piece.text[written..]);
            let mut replacement = String::new();
            if preserve_boundary {
                replacement.push_str(original);
            }
            if starts_with_empty_line {
                replacement.push_str(EMPTY_LINE);
            }
            replacement.push_str(&encode(&value, flow.hard_lines, line_ending));
            if ends_with_empty_line {
                replacement.push_str(EMPTY_LINE);
            }
            if replacement.is_empty() && separates_control_word(source, &piece.source) {
                replacement.push_str("{}");
            }
            patches.push((piece.source.clone(), replacement));
        }
        at = next;
    }
    if edit_index != edits.len() {
        return Err("The inline selection is stale".into());
    }
    let mut updated = source.to_owned();
    for (range, replacement) in patches.into_iter().rev() {
        updated.replace_range(range, &replacement);
    }
    if draft.is_empty() {
        updated.insert_str(flow.range.start, EMPTY_LINE);
    }
    let end = flow.range.end + updated.len() - source.len();
    Ok((updated, flow.range.start..end))
}
#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = include_str!("../../../resources/main.tex");

    fn update(source: &str, prefix: &str, text: &str) -> String {
        let flow = inline_flows(source)
            .into_iter()
            .find(|flow| flow.text.starts_with(prefix))
            .unwrap();
        edit_inline_flow(source, &flow, text).unwrap().0
    }

    #[test]
    fn shipped_signature_is_one_flow_and_only_edited_macro_use_expands() {
        let flow = inline_flows(EXAMPLE)
            .into_iter()
            .find(|flow| flow.text.starts_with("Best"))
            .unwrap();
        assert_eq!(flow.text, "Best regards,\nDr. Example Researcher\nHead of the Example Department\nExample University");
        let expected = EXAMPLE.replace(r"\textbf{\PersonName}", r"\textbf{Dr. Sample Researcher}");
        let draft = flow.text.replace("Dr. Example", "Dr. Sample");
        assert_eq!(
            edit_inline_flow(EXAMPLE, &flow, &draft).unwrap().0,
            expected
        );
    }

    #[test]
    fn disjoint_edits_leave_macro_and_separators_intact() {
        let source = EXAMPLE.replace("\r\n", "\n").replace('\n', "\r\n");
        let flow = inline_flows(&source)
            .into_iter()
            .find(|flow| flow.text.starts_with("Best"))
            .unwrap();
        let draft = flow
            .text
            .replace("Best", "Kind")
            .replace("University", "College");
        let updated = edit_inline_flow(&source, &flow, &draft).unwrap().0;
        assert_eq!(
            updated,
            source
                .replace("Best", "Kind")
                .replace("University", "College")
        );
        let joined = draft.replace("Researcher\nHead", "ResearcherHead");
        assert_eq!(
            update(&updated, "Kind", &joined),
            updated.replace("\\\\[2pt]\r\nHead", "Head")
        );
    }

    #[test]
    fn deleting_a_break_does_not_accidentally_extend_a_control_word() {
        let source = EXAMPLE.replace(r"\textbf{\PersonName}", r"\PersonName");
        let flow = inline_flows(&source)
            .into_iter()
            .find(|flow| flow.text.starts_with("Best"))
            .unwrap();
        let draft = flow.text.replace("Researcher\nHead", "ResearcherHead");
        let updated = edit_inline_flow(&source, &flow, &draft).unwrap().0;
        assert!(updated.contains(r"\PersonName{}Head"));
        assert_eq!(
            inline_flows(&updated)
                .into_iter()
                .find(|flow| flow.text.starts_with("Best"))
                .unwrap()
                .text,
            draft
        );
    }

    #[test]
    fn blank_lines_tabs_and_literal_characters_round_trip() {
        let source =
            "\\documentclass{article}\r\n\\begin{document}\r\nText.\r\n\\end{document}\r\n";
        let draft = "Text.\n\nCafé\t&_%\\#{}^~\n";
        let updated = update(source, "Text", draft);
        assert_eq!(inline_flows(&updated)[0].text, draft);
        assert!(
            updated.contains(r"\&\_\%\textbackslash{}\#\{\}\textasciicircum{}\textasciitilde{}")
        );
        assert_eq!(
            update(source, "Text", ""),
            source.replace("Text.", r"\mbox{}")
        );
    }

    #[test]
    fn a_trailing_space_remains_in_the_projected_flow() {
        let source = "\\documentclass{article}\n\\begin{document}\nText.\n\\end{document}\n";
        let updated = update(source, "Text", "Text. ");
        assert_eq!(inline_flows(&updated)[0].text, "Text. ");
    }

    #[test]
    fn ambiguous_repeated_letters_preserve_formatting_and_macro_provenance() {
        let preamble = "\\documentclass{article}\n\\newcommand{\\A}{a}\n\\begin{document}\n";
        for body in [r"\textbf{a}a", r"\A{}\A"] {
            let source = format!("{preamble}{body}\n\\end{{document}}");
            let flow = &inline_flows(&source)[0];
            assert_eq!(flow.text, "aa");
            assert!(edit_inline_flow(&source, flow, "a")
                .unwrap_err()
                .contains("more than one formatted passage"));
        }
        let source = format!("{preamble}aaa\n\\end{{document}}");
        assert_eq!(update(&source, "aaa", "aa"), source.replace("aaa", "aa"));
    }

    #[test]
    fn unsupported_regions_bound_flows() {
        let source = r"\documentclass{article}
\begin{document}
Before

\begin{verbatim}
literal text
\end{verbatim}

A \textbf{bold} \emph{term}.

\section{A heading}
After
\end{document}";
        let texts: Vec<_> = inline_flows(source)
            .into_iter()
            .map(|flow| flow.text)
            .collect();
        assert_eq!(texts, ["Before", "A bold term.", "After"]);
    }
}
