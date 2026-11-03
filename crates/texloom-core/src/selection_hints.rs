//! Bounded source candidates for engines whose SyncTeX points at a minipage's
//! closing box. These are hints only: the caller must compile a private marker
//! probe and prove exact selected runs and unchanged page appearance.
use crate::{combine_text_units_in_document, resolve_text_unit_in_document, Result, SourceUnit};

pub fn resolve_selection_hint(
    source: &str,
    lines: &[usize],
    document: &str,
    candidate: usize,
) -> Result<SourceUnit> {
    if lines.is_empty() || lines.len() > 64 || candidate >= 16 {
        return Err("Selection hint exceeds its candidate limit".into());
    }
    let mut unique_lines = lines.to_vec();
    unique_lines.sort_unstable();
    unique_lines.dedup();
    let normal = unique_lines
        .iter()
        .map(|line| resolve_text_unit_in_document(source, *line, document))
        .collect::<Result<Vec<_>>>();
    if let Ok(units) = &normal {
        if candidate != 0 {
            return Err("No further selection hints".into());
        }
        return combine_text_units_in_document(source, units, document);
    }
    let rows = source.lines().collect::<Vec<_>>();
    let hint = lines[0]
        .checked_sub(1)
        .filter(|i| *i < rows.len())
        .ok_or("Invalid source hint line")?;
    let mut closing = hint;
    while closing > 0 && hint - closing < 2 && rows[closing].trim().is_empty() {
        closing -= 1;
    }
    if lines.iter().any(|line| *line != hint + 1) || rows[closing].trim() != "\\end{minipage}" {
        return Err(normal.unwrap_err());
    }
    let mut depth = 1usize;
    let mut opening = None;
    for i in (closing.saturating_sub(128)..closing).rev() {
        let row = rows[i].trim();
        if row == "\\end{minipage}" {
            depth += 1;
        }
        if row.starts_with("\\begin{minipage}") || row.starts_with("\\noindent\\begin{minipage}") {
            depth -= 1;
            if depth == 0 {
                opening = Some(i);
                break;
            }
        }
    }
    let opening = opening.ok_or("The minipage exceeds the bounded selection-hint window")?;
    let mut units: Vec<SourceUnit> = Vec::new();
    for line in opening + 2..=closing {
        if let Ok(unit) = resolve_text_unit_in_document(source, line, document) {
            if !units.iter().any(|old| old.range == unit.range) {
                units.push(unit);
            }
        }
        if units.len() > 16 {
            return Err("This minipage has too many units for a safe bounded probe".into());
        }
    }
    let mut candidates = Vec::new();
    for first in 0..units.len() {
        for last in first..(first + lines.len()).min(units.len()) {
            if let Ok(unit) = combine_text_units_in_document(source, &units[first..=last], document)
            {
                candidates.push((
                    lines.len().abs_diff(last - first + 1),
                    std::cmp::Reverse(unit.last_line),
                    unit,
                ));
            }
        }
    }
    candidates.sort_by_key(|(difference, end, _)| (*difference, *end));
    candidates
        .into_iter()
        .nth(candidate)
        .map(|(_, _, unit)| unit)
        .ok_or("No further selection hints".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str = "\\begin{document}\nOutside.\n\\noindent\\begin{minipage}{\\linewidth}\nBest regards,\\\\[10mm]\n\\textbf{Name}\\\\[2pt]\nDepartment\\\\[2pt]\nUniversity\n\\end{minipage}\nFollowing.\n\\end{document}";
    #[test]
    fn closing_box_hint_has_bounded_candidates_without_changing_structural_resolver() {
        assert!(resolve_text_unit_in_document(SOURCE, 8, SOURCE).is_err());
        let group = resolve_selection_hint(SOURCE, &[8, 8, 8], SOURCE, 0).unwrap();
        assert_eq!((group.first_line, group.last_line), (5, 7));
        let after_box = SOURCE.replace("\\end{minipage}\n", "\\end{minipage}\n\n");
        assert_eq!(
            resolve_selection_hint(&after_box, &[9, 9, 9], &after_box, 0)
                .unwrap()
                .range,
            group.range
        );
        assert_eq!(
            resolve_selection_hint(SOURCE, &[8], SOURCE, 2)
                .unwrap()
                .content,
            "\\textbf{Name}"
        );
        assert!(resolve_selection_hint(SOURCE, &[3], SOURCE, 0).is_err());
        assert!(resolve_selection_hint(SOURCE, &[8, 7], SOURCE, 0).is_err());
        assert!(resolve_selection_hint(SOURCE, &[8], SOURCE, 16).is_err());
    }
    #[test]
    fn normal_hints_keep_exact_units_and_do_not_offer_unrelated_alternatives() {
        let normal = resolve_selection_hint(SOURCE, &[5], SOURCE, 0).unwrap();
        assert_eq!(normal.content, "\\textbf{Name}");
        assert!(resolve_selection_hint(SOURCE, &[5], SOURCE, 1).is_err());
        for line in 0..30 {
            for candidate in 0..18 {
                let _ = resolve_selection_hint(SOURCE, &[line], SOURCE, candidate);
            }
        }
    }
}
