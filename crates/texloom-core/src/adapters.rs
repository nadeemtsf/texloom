//! Parameter adapters retain surrounding tokens and refuse unknown grammars.
use super::{argument, command, comment_end, skip_literal, skip_space, Result};
use std::{collections::BTreeSet, ops::Range};

#[derive(Clone, Debug)]
pub struct Parameter {
    pub key: String,
    pub label: String,
    pub value: String,
    pub reason: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Control {
    pub id: usize,
    pub kind: String,
    pub label: String,
    pub line: usize,
    pub parameters: Vec<Parameter>,
    pub reason: Option<String>,
}
#[derive(Clone)]
enum Slot {
    Value(Range<usize>),
    Column(Range<usize>),
    Image {
        options: Option<Range<usize>>,
        after: usize,
        key: String,
    },
    Float {
        options: Option<Range<usize>>,
        after: usize,
    },
}
struct Parsed {
    control: Control,
    slots: Vec<Slot>,
}

fn optional(s: &str, at: usize) -> Result<Option<(Range<usize>, usize)>> {
    let start = skip_space(s, at);
    if s.as_bytes().get(start) != Some(&b'[') {
        return Ok(None);
    }
    let mut i = start + 1;
    let mut depth = 0;
    while i < s.len() {
        match s.as_bytes()[i] {
            b'\\' => {
                i = command(s, i).1;
                continue;
            }
            b'%' => {
                i = comment_end(s, i);
                continue;
            }
            b'{' => depth += 1,
            b'}' => {
                if depth == 0 {
                    return Err("Unbalanced option braces".into());
                }
                depth -= 1
            }
            b']' if depth == 0 => return Ok(Some((start + 1..i, i + 1))),
            _ => (),
        }
        i += 1;
    }
    Err("Unclosed optional argument".into())
}
fn options(s: &str, range: Range<usize>) -> Result<Vec<(Range<usize>, String, String)>> {
    let mut out = Vec::new();
    let mut begin = range.start;
    let mut i = begin;
    let mut depth = 0;
    while i <= range.end {
        if i == range.end || s.as_bytes()[i] == b',' && depth == 0 {
            let text = s[begin..i].trim();
            if !text.is_empty() {
                let (key, value) = text.split_once('=').unwrap_or((text, "true"));
                out.push((begin..i, key.trim().into(), value.trim().into()));
            }
            begin = i + 1;
        } else {
            match s.as_bytes()[i] {
                b'\\' => {
                    i = command(s, i).1;
                    continue;
                }
                b'{' => depth += 1,
                b'}' => {
                    if depth == 0 {
                        return Err("Unbalanced image options".into());
                    }
                    depth -= 1
                }
                b'%' => return Err("Commented image options require source editing".into()),
                _ => (),
            }
        }
        i += 1;
    }
    if depth != 0 {
        return Err("Unbalanced image options".into());
    }
    Ok(out)
}
fn dimension(value: &str, negative: bool, font: bool) -> bool {
    let value = value.trim();
    let end = value
        .find(|c: char| !(c.is_ascii_digit() || matches!(c, '.' | '+' | '-')))
        .unwrap_or(value.len());
    let Ok(number) = value[..end].parse::<f64>() else {
        return false;
    };
    if !number.is_finite() || !negative && number <= 0.0 {
        return false;
    }
    let unit = value[end..].trim();
    let multiplier = match unit {
        "mm" => 72.0 / 25.4,
        "cm" => 72.0 / 2.54,
        "in" => 72.0,
        "pt" => 72.0 / 72.27,
        "bp" => 1.0,
        "pc" => 12.0 * 72.0 / 72.27,
        "" if font => 1.0,
        "em" | "ex" | "\\linewidth" | "\\textwidth" | "\\columnwidth" | "\\textheight" => {
            return !font && number.abs() <= 20.0
        }
        _ => return false,
    };
    number.abs() * multiplier <= if font { 400.0 } else { 14400.0 }
}
fn parameter(key: &str, label: &str, value: &str, editable: bool) -> Parameter {
    Parameter {
        key: key.into(),
        label: label.into(),
        value: value.into(),
        reason: (!editable).then(|| "This expression requires source editing.".into()),
    }
}
fn scan(source: &str) -> Vec<Parsed> {
    let mut found = Vec::new();
    let mut i = 0;
    let newlines = source
        .bytes()
        .enumerate()
        .filter_map(|(i, b)| (b == b'\n').then_some(i))
        .collect::<Vec<_>>();
    let line = |start| newlines.partition_point(|i| *i < start) + 1;
    while i < source.len() {
        if found.len() >= 2000 {
            break;
        }
        if source.as_bytes()[i] == b'%' {
            i = comment_end(source, i);
            continue;
        }
        if source.as_bytes()[i] != b'\\' {
            i += 1;
            continue;
        }
        let start = i;
        let (cmd, end) = command(source, i);
        i = end;
        if let Some(next) = skip_literal(source, cmd, end) {
            i = next;
            continue;
        }
        let parsed=(||->Result<Option<(String,String,Vec<Parameter>,Vec<Slot>,Option<String>,usize)>>{
            if cmd=="\\geometry" {
                let (range,next)=argument(source,end)?;let values=options(source,range.clone())?;let mut keys=BTreeSet::new();
                for (_,key,_) in &values {if !keys.insert(key){return Err("Duplicate page-layout option".into())}}
                let mut params=Vec::new();let mut slots=Vec::new();
                for (key,label) in [("margin","All margins"),("left","Left margin"),("right","Right margin"),("top","Top margin"),("bottom","Bottom margin")] {
                    let value=values.iter().find(|v|v.1==key).map(|v|v.2.as_str()).unwrap_or("");
                    params.push(parameter(key,label,value,value.is_empty()||dimension(value,false,false)));slots.push(Slot::Image{options:Some(range.clone()),after:end,key:key.into()});
                }
                return Ok(Some(("layout".into(),"Page margins".into(),params,slots,None,next)))
            }
            if ["\\linespread","\\setstretch"].contains(&cmd) {
                let (range,next)=argument(source,end)?;let value=&source[range.clone()];
                let editable=value.trim().parse::<f64>().is_ok_and(|v|v.is_finite()&&(0.5..=3.0).contains(&v));
                return Ok(Some(("typography".into(),"Line spacing factor".into(),vec![parameter("factor","Factor",value,editable)],vec![Slot::Value(range)],None,next)))
            }
            if cmd=="\\includegraphics" {
                let after=if source.as_bytes().get(end)==Some(&b'*'){end+1}else{end};
                let opt=optional(source,after)?; let (file,next)=argument(source,opt.as_ref().map_or(after,|v|v.1))?;
                let opts=opt.as_ref().map(|v|v.0.clone()); let values=if let Some(r)=&opts{options(source,r.clone())?}else{vec![]};
                let mut params=Vec::new();let mut slots=Vec::new();let mut keys=BTreeSet::new();
                for (_,key,_) in &values {if !keys.insert(key){return Err("Duplicate image option".into())}}
                for (key,label) in [("width","Width"),("height","Height"),("scale","Scale"),("keepaspectratio","Keep proportions")] {
                    let value=values.iter().find(|v|v.1==key).map(|v|v.2.as_str()).unwrap_or(if key=="keepaspectratio"{"false"}else{""});
                    let editable=value.is_empty()||match key{"scale"=>value.parse::<f64>().is_ok_and(|v|v.is_finite()&&v>0.0&&v<=100.0),"keepaspectratio"=>["true","false"].contains(&value),_=>dimension(value,false,false)};
                    params.push(parameter(key,label,value,editable)); slots.push(Slot::Image{options:opts.clone(),after,key:key.into()});
                }
                return Ok(Some(("image".into(),format!("Image: {}",&source[file]),params,slots,None,next)))
            }
            if ["\\vspace","\\hspace","\\setlength","\\fontsize"].contains(&cmd) {
                let after=if source.as_bytes().get(end)==Some(&b'*'){end+1}else{end};
                let (first,next)=argument(source,after)?;
                let mut params=Vec::new();let mut slots=Vec::new();let mut finish=next;
                if cmd=="\\fontsize" {
                    let (leading,last)=argument(source,next)?;finish=last;
                    for (key,label,range) in [("size","Font size",first),("leading","Line height",leading)] {let value=&source[range.clone()];params.push(parameter(key,label,value,dimension(value,false,true)));slots.push(Slot::Value(range));}
                } else {
                    let value=if cmd=="\\setlength" {
                        if !["\\parskip","\\parindent","\\baselineskip","\\tabcolsep","\\abovecaptionskip","\\belowcaptionskip","\\columnsep","\\textwidth","\\textheight","\\headheight","\\footskip","\\oddsidemargin","\\evensidemargin","\\topmargin"].contains(&source[first.clone()].trim()){return Ok(None)}
                        let (value,last)=argument(source,next)?;finish=last;value
                    }else{first};
                    let text=&source[value.clone()];params.push(parameter("length","Length",text,dimension(text,true,false)));slots.push(Slot::Value(value));
                }
                return Ok(Some((if cmd=="\\fontsize"{"typography"}else{"spacing"}.into(),cmd.trim_start_matches('\\').into(),params,slots,None,finish)))
            }
            if cmd=="\\begin" {
                let (env,next)=argument(source,end)?;let env=&source[env];
                if ["figure","figure*","table","table*"].contains(&env) {
                    let opt=optional(source,next)?;let value=opt.as_ref().map(|v|source[v.0.clone()].trim()).unwrap_or("");
                    return Ok(Some(("float".into(),format!("{env} placement"),vec![parameter("placement","Placement",value,placement(value))],vec![Slot::Float{options:opt.as_ref().map(|v|v.0.clone()),after:next}],None,opt.map_or(next,|v|v.1))))
                }
                if ["tabular","tabular*","tabularx"].contains(&env) {
                    let mut cursor=next;let mut params=Vec::new();let mut slots=Vec::new();
                    if env!="tabular" {let (width,last)=argument(source,cursor)?;cursor=last;let value=&source[width.clone()];params.push(parameter("width","Table width",value,dimension(value,false,false)));slots.push(Slot::Value(width));}
                    if let Some((_,last))=optional(source,cursor)?{cursor=last;}
                    let (columns,last)=argument(source,cursor)?;let mut at=columns.start;let mut number=0;let mut reason=None;
                    while at<columns.end {
                        if number>=64 {reason=Some("Tables with more than 64 columns require source editing.".into());break}
                        let c=source.as_bytes()[at];
                        if c.is_ascii_whitespace()||c==b'|' {at+=1;continue}
                        if matches!(c,b'@'|b'!'|b'>'|b'<'){let (_,last)=argument(source,at+1)?;at=last;continue}
                        let begin=at;at+=1;number+=1;
                        let value=if matches!(c,b'p'|b'm'|b'b'){let (dim,last)=argument(source,at)?;at=last;source[dim].to_string()}else if matches!(c,b'l'|b'c'|b'r'|b'X'){String::new()}else{reason=Some("Repeated or custom column types require source editing.".into());break};
                        params.push(parameter(&format!("column_{number}"),&format!("Column {number} width"),&value,value.is_empty()||dimension(&value,false,false)));slots.push(Slot::Column(begin..at));
                    }
                    return Ok(Some(("table".into(),format!("{env}: {number} columns"),params,slots,reason,last)))
                }
            } Ok(None)
        })();
        match parsed {
            Ok(Some((kind, label, parameters, slots, reason, next))) => {
                found.push(Parsed {
                    control: Control {
                        id: found.len(),
                        kind,
                        label: label.chars().take(160).collect(),
                        line: line(start),
                        parameters,
                        reason,
                    },
                    slots,
                });
                i = next;
            }
            Err(reason) => {
                found.push(Parsed {
                    control: Control {
                        id: found.len(),
                        kind: "unsupported".into(),
                        label: cmd.into(),
                        line: line(start),
                        parameters: vec![],
                        reason: Some(reason),
                    },
                    slots: vec![],
                });
                // A malformed braced/optional argument has no trustworthy end.
                // Do not repeatedly rescan the rest of a large incomplete file.
                break;
            }
            _ => (),
        }
    }
    found
}
fn placement(value: &str) -> bool {
    value.is_empty()
        || value.len() <= 6
            && value.chars().all(|c| "htbp!H".contains(c))
            && (!value.contains('H') || value == "H")
}
pub fn describe(source: &str) -> Vec<Control> {
    scan(source).into_iter().map(|p| p.control).collect()
}

/// One explicit parameter change. `expected` is the complete canonical file.
pub fn apply(source: &str, expected: &str, id: usize, key: &str, value: &str) -> Result<String> {
    if source != expected {
        return Err("The source changed. Review its controls again.".into());
    }
    let mut parsed = scan(source);
    if id >= parsed.len() {
        return Err("That source control no longer exists".into());
    }
    let parsed = parsed.swap_remove(id);
    if let Some(reason) = parsed.control.reason {
        return Err(reason);
    }
    let index = parsed
        .control
        .parameters
        .iter()
        .position(|p| p.key == key)
        .ok_or("Unknown source parameter")?;
    let parameter = &parsed.control.parameters[index];
    if let Some(reason) = &parameter.reason {
        return Err(reason.clone());
    }
    let value = value.trim();
    let valid = match key {
        "placement" => placement(value),
        "keepaspectratio" => ["true", "false"].contains(&value),
        "scale" => {
            value.is_empty()
                || value
                    .parse::<f64>()
                    .is_ok_and(|v| v.is_finite() && v > 0.0 && v <= 100.0)
        }
        "factor" => value
            .parse::<f64>()
            .is_ok_and(|v| v.is_finite() && (0.5..=3.0).contains(&v)),
        "size" | "leading" => dimension(value, false, true),
        _ => {
            value.is_empty() && ["image", "layout"].contains(&parsed.control.kind.as_str())
                || dimension(value, parsed.control.kind == "spacing", false)
        }
    };
    if !valid {
        return Err("Enter a supported numeric length, scale or placement.".into());
    }
    let (range, replacement) = match &parsed.slots[index] {
        Slot::Value(range) => (range.clone(), value.into()),
        Slot::Column(range) => {
            let kind = source.as_bytes()[range.start];
            let kind = if matches!(kind, b'p' | b'm' | b'b') {
                kind as char
            } else {
                'p'
            };
            (range.clone(), format!("{kind}{{{value}}}"))
        }
        Slot::Float { options, after } => {
            if let Some(range) = options {
                (range.clone(), value.into())
            } else {
                (*after..*after, format!("[{value}]"))
            }
        }
        Slot::Image {
            options: opt,
            after,
            key,
        } => {
            if let Some(range) = opt {
                let values = options(source, range.clone())?;
                if let Some((n, (slot, _, _))) =
                    values.iter().enumerate().find(|(_, v)| v.1 == *key)
                {
                    if value.is_empty() {
                        let deletion = if values.len() == 1 {
                            slot.clone()
                        } else if n + 1 < values.len() {
                            slot.start..values[n + 1].0.start
                        } else {
                            values[n - 1].0.end..slot.end
                        };
                        (deletion, String::new())
                    } else {
                        (slot.clone(), format!("{key}={value}"))
                    }
                } else if value.is_empty() {
                    return Ok(source.into());
                } else {
                    (
                        range.end..range.end,
                        format!("{}{key}={value}", if values.is_empty() { "" } else { "," }),
                    )
                }
            } else if value.is_empty() {
                return Ok(source.into());
            } else {
                (*after..*after, format!("[{key}={value}]"))
            }
        }
    };
    if source[range.clone()] == replacement {
        return Ok(source.into());
    }
    let mut result = source.to_string();
    result.replace_range(range, &replacement);
    Ok(result)
}

pub fn style(
    source: &str,
    expected: &str,
    document: &str,
    range: Range<usize>,
    style: &str,
) -> Result<String> {
    if source != expected
        || range.start >= range.end
        || !source.is_char_boundary(range.start)
        || !source.is_char_boundary(range.end)
        || range.end > source.len()
    {
        return Err("The selected source changed or has invalid boundaries".into());
    }
    let fragment = &source[range.clone()];
    super::validate_movable_text(fragment, document)?;
    let command = match style {
        "bold" => "textbf",
        "italic" => "textit",
        "underline" => "underline",
        "serif" => "textrm",
        "sans" => "textsf",
        "mono" => "texttt",
        _ => return Err("Unknown text style".into()),
    };
    let mut result = source.to_string();
    result.replace_range(range, &format!("\\{command}{{{fragment}\n}}"));
    Ok(result)
}
pub fn ensure_graphics_package(source: &str) -> Result<String> {
    let (preamble, _, _) = super::document_bounds(source)?;
    if super::has_package(&source[..preamble], "graphicx") {
        return Ok(source.into());
    }
    let mut result = source.to_string();
    result.insert_str(preamble, "\\usepackage{graphicx}\n");
    Ok(result)
}
pub fn insert_image(source: &str, expected: &str, at: usize, asset: &str) -> Result<String> {
    if source != expected
        || !source.is_char_boundary(at)
        || !asset.starts_with("images/")
        || asset
            .split('/')
            .any(|s| s.is_empty() || s == ".." || s == ".")
        || !asset
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/._-".contains(&b))
    {
        return Err("The image insertion source or asset changed".into());
    }
    if let Ok((_, body, end)) = super::document_bounds(source) {
        if at < body || at > end {
            return Err("Place the source cursor inside the document body".into());
        }
    }
    let mut result = source.to_string();
    result.insert_str(
        at,
        &format!("\n\\includegraphics[width=0.5\\linewidth,keepaspectratio]{{{asset}}}\n"),
    );
    Ok(result)
}
pub fn utf8_declarations(source: &str) -> Result<String> {
    let mut patches = Vec::new();
    let mut i = 0;
    while i < source.len() {
        if source.as_bytes()[i] == b'%' {
            i = comment_end(source, i);
            continue;
        }
        if source.as_bytes()[i] != b'\\' {
            i += 1;
            continue;
        }
        let (cmd, end) = command(source, i);
        i = end;
        if let Some(last) = skip_literal(source, cmd, end) {
            i = last;
            continue;
        }
        if cmd == "\\inputencoding" {
            let (range, last) = argument(source, end)?;
            patches.push(range);
            i = last;
        } else if ["\\usepackage", "\\RequirePackage"].contains(&cmd) {
            let opt = optional(source, end)?;
            if let Some((range, last)) = opt {
                if let Ok((packages, after)) = argument(source, last) {
                    if source[packages].split(',').any(|p| p.trim() == "inputenc") {
                        if source[range.clone()].contains(',') {
                            return Err("Review the complex inputenc declaration before converting this source".into());
                        }
                        patches.push(range);
                    }
                    i = after;
                }
            }
        }
    }
    let mut result = source.to_string();
    for range in patches.into_iter().rev() {
        result.replace_range(range, "utf8");
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_and_unicode_control_inputs_are_bounded_and_never_panic() {
        let pieces = [
            "\\includegraphics",
            "{",
            "}",
            "[",
            "]",
            "%",
            "\n",
            "\\",
            "😀",
            "é",
            "\u{200f}",
            "\\begin",
            "\\vspace",
            "\\verb|",
            "\\end",
            "\\geometry",
            "\\fontsize",
            "3mm",
        ];
        let mut seed = 17u64;
        for _ in 0..3000 {
            let mut source = String::new();
            for _ in 0..40 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                source.push_str(pieces[(seed >> 32) as usize % pieces.len()]);
            }
            let original = source.clone();
            let described = describe(&source);
            assert_eq!(source, original);
            assert!(described.len() <= 2000);
            for control in described.iter().take(2) {
                if let Some(parameter) = control.parameters.first() {
                    let _ = apply(&source, &source, control.id, &parameter.key, "2mm");
                }
            }
        }
        let source = "\\vspace{".repeat(20000);
        let controls = describe(&source);
        assert_eq!(controls.len(), 1);
        assert!(controls[0].reason.is_some());
    }
    #[test]
    fn page_margins_and_line_spacing_change_only_explicit_source_parameters() {
        let source="\u{feff}\\geometry{paper=a4paper, margin = 25mm,includehead}% keep\r\n\\linespread{1.1}\r\n\\setlength{\\columnsep}{8mm}";
        assert_eq!(
            apply(source, source, 0, "margin", "20mm").unwrap(),
            source.replace(" margin = 25mm", "margin=20mm")
        );
        assert_eq!(
            apply(source, source, 0, "left", "18mm").unwrap(),
            source.replace(",includehead}", ",includehead,left=18mm}")
        );
        assert_eq!(
            apply(source, source, 1, "factor", "1.3").unwrap(),
            source.replace("{1.1}", "{1.3}")
        );
        assert_eq!(
            apply(source, source, 2, "length", "10mm").unwrap(),
            source.replace("{8mm}", "{10mm}")
        );
        for factor in ["0", "4", "NaN", "1}\\input{x}"] {
            assert!(apply(source, source, 1, "factor", factor).is_err());
        }
        assert!(describe("\\geometry{margin=20mm,margin=25mm}")[0]
            .reason
            .is_some());
    }
    #[test]
    fn image_options_keep_other_tokens_and_change_only_the_requested_parameter() {
        let s="\u{feff}% header\r\n\\includegraphics[trim={1 2 3 4}, width = 2cm,clip]{a.png}\r\nAfter";
        let changed = apply(s, s, 0, "width", "35mm").unwrap();
        assert_eq!(changed, s.replace(" width = 2cm", "width=35mm"));
        assert_eq!(
            apply(s, s, 0, "height", "4cm").unwrap(),
            s.replace(",clip]", ",clip,height=4cm]")
        );
        assert_eq!(
            apply(s, s, 0, "width", "").unwrap(),
            s.replace(" width = 2cm,", "")
        );
        let bare = "\\includegraphics{a.png}";
        assert_eq!(
            apply(bare, bare, 0, "width", "0.5\\linewidth").unwrap(),
            "\\includegraphics[width=0.5\\linewidth]{a.png}"
        );
    }
    #[test]
    fn table_columns_preserve_rules_modifiers_and_other_columns() {
        let s = "\\begin{tabular}{|l|p{2cm}|>{\\bfseries}r|}A&B&C\\\\\\end{tabular}";
        assert_eq!(
            apply(s, s, 0, "column_2", "35mm").unwrap(),
            s.replace("p{2cm}", "p{35mm}")
        );
        assert_eq!(
            apply(s, s, 0, "column_1", "1in").unwrap(),
            s.replace("|l|", "|p{1in}|")
        );
        let repeated = "\\begin{tabular}{*{2}{l}}A&B\\end{tabular}";
        assert!(apply(repeated, repeated, 0, "column_1", "2cm").is_err());
    }
    #[test]
    fn explicit_spacing_fonts_and_float_placement_are_bounded_and_lossless() {
        for (s, key, value, expected) in [
            (
                "A\\vspace*{2mm}%keep\r\nB",
                "length",
                "-3pt",
                "A\\vspace*{-3pt}%keep\r\nB",
            ),
            ("\\fontsize{12}{14}X", "size", "16", "\\fontsize{16}{14}X"),
            (
                "\\begin{figure}X\\end{figure}",
                "placement",
                "htbp",
                "\\begin{figure}[htbp]X\\end{figure}",
            ),
        ] {
            assert_eq!(apply(s, s, 0, key, value).unwrap(), expected)
        }
        let s = "\\hspace{2mm}";
        for value in ["NaNmm", "1mm}\\input{secret}", "999999in", ""] {
            assert!(apply(s, s, 0, "length", value).is_err())
        }
    }
    #[test]
    fn comments_literals_stale_sources_and_duplicate_keys_never_become_edits() {
        let s="% \\includegraphics{x}\n\\verb|\\vspace{2pt}|\n\\begin{verbatim}\\fontsize{1}{2}\\end{verbatim}";
        assert!(describe(s).is_empty());
        let s = "\\includegraphics[width=2cm,width=3cm]{x}";
        assert!(describe(s)[0].reason.is_some());
        assert!(apply("\\vspace{2mm}", "old", 0, "length", "3mm").is_err());
        let s = "\\vspace{\\CustomGap}";
        assert!(describe(s)[0].parameters[0].reason.is_some());
    }
    #[test]
    fn styling_preserves_unicode_and_outside_bytes_and_refuses_structural_source() {
        let s = "Before café after";
        let start = s.find("café").unwrap();
        let end = start + "café".len();
        assert_eq!(
            style(s, s, s, start..end, "bold").unwrap(),
            "Before \\textbf{café\n} after"
        );
        assert!(style("\\input{x}", "\\input{x}", "", 0..9, "bold").is_err());
    }
    #[test]
    fn encoding_declarations_and_image_insertion_keep_unrelated_tokens() {
        let source="\u{feff}\\documentclass{article}\r\n\\usepackage[latin1]{inputenc}\r\n% \\inputencoding{latin1}\r\n\\begin{document}\r\nCafe\r\n\\end{document}";
        assert_eq!(
            utf8_declarations(source).unwrap(),
            source.replacen("[latin1]", "[utf8]", 1)
        );
        let at = source.find("Cafe").unwrap();
        let with = insert_image(source, source, at, "images/image-1.png").unwrap();
        assert!(with.contains(
            "\\includegraphics[width=0.5\\linewidth,keepaspectratio]{images/image-1.png}"
        ));
        assert!(ensure_graphics_package(&with)
            .unwrap()
            .contains("\\usepackage{graphicx}"));
        assert!(insert_image(source, source, 0, "images/x.png").is_err());
        assert!(insert_image(source, source, at, "images/../secret.png").is_err());
    }
}
