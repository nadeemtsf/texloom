//! Visual operations prepare source candidates; they never save a project.
use super::*;
use std::time::Instant;

#[derive(Deserialize)]
pub struct SelectionPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize)]
pub struct ResolvedSelection {
    pub schema: u32,
    pub unit_id: String,
    pub build_id: String,
    pub revision: u64,
    pub file: String,
    pub expected_source: String,
    pub start: usize,
    pub end: usize,
    pub first_line: usize,
    pub last_line: usize,
    pub kind: String,
    pub content: String,
    pub plain_text: Option<String>,
    pub rectangles: Vec<SourceRectangle>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spacing_model_and_object_edit_change_only_the_explicit_length() {
        let content = "\\textbf{Name}\\\\[2pt]\r\nDepartment\\\\[3mm]\r\nUniversity\n";
        let model = describe_text_edit(content, content).unwrap();
        assert_eq!(model.spacing.len(), 2);
        assert_eq!(model.spacing[0].value, Some(2.0));
        let source=format!("\\documentclass{{article}}\n\\usepackage{{texloom-objects}}\n\\begin{{document}}\nBefore\n\\TexloomObject{{sig}}{{0}}{{0}}{{{content}}}\nAfter\n\\end{{document}}");
        let request:ObjectEditRequest=serde_json::from_value(json!({"sources":{"main.tex":source},"main":"main.tex","file":"main.tex","id":"sig","action":"edit","expected":content,"spacing_edits":[{"id":0,"value":"6"}]})).unwrap();
        assert_eq!(
            plan_object_edit(request).unwrap().sources["main.tex"],
            source.replacen("[2pt]", "[6pt]", 1)
        );
    }
    #[test]
    fn combined_text_and_spacing_keep_parameter_identity_and_refuse_stale_input() {
        let content = "A \\textbf{B}\\\\[2pt]C";
        let text = [LiteralEditRequest {
            id: 1,
            text: "D&E".into(),
        }];
        let gap = [SpacingEditRequest {
            id: 0,
            value: "5".into(),
        }];
        assert_eq!(
            preview_edit_parameters(content, content, content, &text, &gap).unwrap(),
            "A \\textbf{D\\&E}\\\\[5pt]C"
        );
        assert!(preview_edit_parameters(content, "old", content, &text, &gap).is_err());
    }
    #[test]
    fn protected_object_edits_preserve_every_command_and_unrelated_source_byte() {
        let source="\\documentclass{article}\r\n\\usepackage{texloom-objects}\r\n\\begin{document}\r\nBefore\r\n\\TexloomObject{name}{0}{0}{A \\textbf{B} C\n}\r\nAfter\r\n\\end{document}";
        let request: ObjectEditRequest=serde_json::from_value(json!({"sources":{"main.tex":source},"main":"main.tex","file":"main.tex","action":"edit","id":"name","expected":"A \\textbf{B} C\n","literal_edits":[{"id":1,"text":"D&E"}]})).unwrap();
        let plan = plan_object_edit(request).unwrap();
        assert_eq!(
            plan.sources["main.tex"],
            source.replace("\\textbf{B}", "\\textbf{D\\&E}")
        );
    }
    #[test]
    fn protected_models_refuse_stale_content_and_keep_macro_calls_locked() {
        let document="\\documentclass{article}\n\\newcommand{\\Person}{Name}\n\\begin{document}Text\\end{document}";
        let model = describe_text_edit("Hello \\Person \\textbf{world}", document).unwrap();
        assert_eq!(model.runs.len(), 2);
        assert!(model.protected.contains(&"\\Person".to_string()));
        assert_eq!(
            preview_text_edit(
                "A \\textbf{B}",
                document,
                &[LiteralEditRequest {
                    id: 1,
                    text: "C".into()
                }]
            )
            .unwrap(),
            "A \\textbf{C}"
        );
        let source = format!("{document}\\TexloomObject{{name}}{{0}}{{0}}{{A}}");
        let request: ObjectEditRequest=serde_json::from_value(json!({"sources":{"main.tex":source},"main":"main.tex","file":"main.tex","action":"edit","id":"name","expected":"old","literal_edits":[{"id":0,"text":"B"}]})).unwrap();
        assert!(plan_object_edit(request).err().unwrap().contains("changed"));
    }
    #[test]
    fn visual_ranges_use_the_editor_projection_without_rewriting_windows_source() {
        let source="\u{feff}\\documentclass{article}\r\n\\begin{document}\r\nBefore.\r\n\r\nDr. Example\\\\[2pt]\r\nDepartment\r\n\\end{document}\r\n";
        let projection = texloom_core::document::projection::EditorProjection::new(source);
        let start = source.find("Dr. Example").unwrap();
        let end = start + "Dr. Example".len();
        let request = ObjectEditRequest {
            sources: BTreeMap::from([("main.tex".into(), source.into())]),
            main: "main.tex".into(),
            file: "main.tex".into(),
            action: "wrap".into(),
            id: "windows".into(),
            selection: Some([
                projection.editor_offset(start).unwrap(),
                projection.editor_offset(end).unwrap(),
            ]),
            expected: Some("Dr. Example".into()),
            dx: Some(0.0),
            dy: Some(0.0),
            content: None,
            plain: false,
            literal_edits: None,
            spacing_edits: None,
        };
        let plan = plan_object_edit(request).unwrap();
        let result = &plan.sources["main.tex"];
        assert!(
            result.contains("\\TexloomObject{windows}{0}{0}{Dr. Example\n}\\\\[2pt]\r\nDepartment")
        );
        let detached = texloom_core::unwrap_text_object(result, "windows").unwrap();
        assert_eq!(
            detached,
            texloom_core::ensure_objects_package(source).unwrap()
        );
        assert!(result.starts_with('\u{feff}'));
    }
}

#[derive(Clone, Deserialize)]
pub struct ObjectEditRequest {
    pub sources: BTreeMap<String, String>,
    pub main: String,
    pub file: String,
    pub action: String,
    pub id: String,
    #[serde(default)]
    pub selection: Option<[usize; 2]>,
    #[serde(default)]
    pub expected: Option<String>,
    #[serde(default)]
    pub dx: Option<f64>,
    #[serde(default)]
    pub dy: Option<f64>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub plain: bool,
    #[serde(default)]
    pub literal_edits: Option<Vec<LiteralEditRequest>>,
    #[serde(default)]
    pub spacing_edits: Option<Vec<SpacingEditRequest>>,
}

#[derive(Clone, Deserialize)]
pub struct SpacingEditRequest {
    pub id: usize,
    pub value: String,
}
#[derive(Serialize)]
pub struct SpacingView {
    pub id: usize,
    pub value: Option<f64>,
    pub unit: String,
    pub source: String,
    pub reason: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct LiteralEditRequest {
    pub id: usize,
    pub text: String,
}
#[derive(Serialize)]
pub struct LiteralRunView {
    pub id: usize,
    pub text: String,
    pub styles: Vec<String>,
}
#[derive(Serialize)]
pub struct TextEditModel {
    pub runs: Vec<LiteralRunView>,
    pub protected: Vec<String>,
    pub spacing: Vec<SpacingView>,
}
pub fn describe_text_edit(content: &str, document: &str) -> Result<TextEditModel> {
    let model = texloom_core::literal_text::describe(content, document)?;
    Ok(TextEditModel {
        runs: model
            .runs
            .into_iter()
            .enumerate()
            .map(|(id, run)| LiteralRunView {
                id,
                text: run.text,
                styles: run.styles,
            })
            .collect(),
        protected: model.protected,
        spacing: texloom_core::spacing::describe(content, document)?
            .into_iter()
            .enumerate()
            .map(|(id, p)| SpacingView {
                id,
                value: p.value,
                unit: p.unit,
                source: p.source,
                reason: p.reason,
            })
            .collect(),
    })
}
fn literal_edits(edits: &[LiteralEditRequest]) -> Vec<texloom_core::literal_text::LiteralEdit> {
    edits
        .iter()
        .map(|e| texloom_core::literal_text::LiteralEdit {
            id: e.id,
            text: e.text.clone(),
        })
        .collect()
}
#[cfg(test)]
fn preview_text_edit(
    content: &str,
    document: &str,
    edits: &[LiteralEditRequest],
) -> Result<String> {
    preview_edit_parameters(content, content, document, edits, &[])
}
pub fn preview_edit_parameters(
    content: &str,
    expected: &str,
    document: &str,
    edits: &[LiteralEditRequest],
    spacing: &[SpacingEditRequest],
) -> Result<String> {
    let text =
        texloom_core::literal_text::apply(content, expected, document, &literal_edits(edits))?;
    let changes = spacing
        .iter()
        .map(|e| texloom_core::spacing::Edit {
            id: e.id,
            value: e.value.clone(),
        })
        .collect::<Vec<_>>();
    texloom_core::spacing::apply(&text, &text, document, &changes)
}

#[derive(Serialize)]
pub struct ObjectPlan {
    pub sources: BTreeMap<String, String>,
    pub object_id: String,
    pub file: String,
    pub operation: String,
}

pub fn plan_object_edit(mut request: ObjectEditRequest) -> Result<ObjectPlan> {
    relative(&request.main)?;
    relative(&request.file)?;
    let source = request
        .sources
        .get(&request.file)
        .ok_or("Source file is not in this project")?;
    let document = request
        .sources
        .get(&request.main)
        .ok_or("Main document is missing")?;
    if source.len() > 4_000_000 || request.sources.len() > MAX_FILES {
        return Err("Project source exceeds supported limits".into());
    }
    let updated = match request.action.as_str() {
        "wrap" => {
            let [start, end] = request.selection.ok_or("Select text before moving it")?;
            let projection = texloom_core::document::projection::EditorProjection::new(source);
            let range = projection.source_offset(start).map_err(fail)?
                ..projection.source_offset(end).map_err(fail)?;
            if source.get(range.clone()) != request.expected.as_deref() {
                return Err("The selected source has changed; select it again".into());
            }
            let wrapped = texloom_core::wrap_text_object(
                source,
                range,
                &request.id,
                request.dx.unwrap_or(0.0),
                request.dy.unwrap_or(0.0),
                document,
            )?;
            if request.literal_edits.is_some() || request.spacing_edits.is_some() {
                let content = request
                    .expected
                    .as_deref()
                    .ok_or("Missing expected content")?;
                let updated = preview_edit_parameters(
                    content,
                    content,
                    document,
                    request.literal_edits.as_deref().unwrap_or(&[]),
                    request.spacing_edits.as_deref().unwrap_or(&[]),
                )?;
                texloom_core::update_text_object(
                    &wrapped,
                    &request.id,
                    None,
                    None,
                    Some(&updated),
                    document,
                )?
            } else if let Some(value) = request.content.as_deref() {
                let object = texloom_core::text_objects(&wrapped)?
                    .into_iter()
                    .find(|o| o.id == request.id)
                    .ok_or("Object is missing")?;
                let content = if request.plain {
                    texloom_core::replace_plain_text(&object.content, value, document)?
                } else {
                    value.into()
                };
                texloom_core::update_text_object(
                    &wrapped,
                    &request.id,
                    None,
                    None,
                    Some(&content),
                    document,
                )?
            } else {
                wrapped
            }
        }
        "move" => texloom_core::update_text_object(
            source,
            &request.id,
            request.dx,
            request.dy,
            None,
            document,
        )?,
        "edit" => {
            if request.literal_edits.is_some() || request.spacing_edits.is_some() {
                let object = texloom_core::text_objects(source)?
                    .into_iter()
                    .find(|o| o.id == request.id)
                    .ok_or("Object is missing")?;
                let updated = preview_edit_parameters(
                    &object.content,
                    request
                        .expected
                        .as_deref()
                        .ok_or("Missing expected content")?,
                    document,
                    request.literal_edits.as_deref().unwrap_or(&[]),
                    request.spacing_edits.as_deref().unwrap_or(&[]),
                )?;
                let mut source = source.clone();
                source.replace_range(object.arguments[3].clone(), &updated);
                source
            } else {
                let value = request
                    .content
                    .as_deref()
                    .ok_or("Enter the object's text")?;
                let content = if request.plain {
                    let object = texloom_core::text_objects(source)?
                        .into_iter()
                        .find(|o| o.id == request.id)
                        .ok_or("Object is missing")?;
                    texloom_core::replace_plain_text(&object.content, value, document)?
                } else {
                    value.into()
                };
                texloom_core::update_text_object(
                    source,
                    &request.id,
                    None,
                    None,
                    Some(&content),
                    document,
                )?
            }
        }
        "detach" => texloom_core::unwrap_text_object(source, &request.id)?,
        _ => return Err("Unsupported text-object operation".into()),
    };
    request.sources.insert(request.file.clone(), updated);
    let main = request
        .sources
        .get(&request.main)
        .ok_or("Main document is missing")?;
    request
        .sources
        .insert(request.main, texloom_core::ensure_objects_package(main)?);
    if let Some(existing) = request.sources.get("texloom-objects.sty") {
        if existing != OBJECT_STYLE {
            return Err("This project has a different texloom-objects.sty. Keep its version consistent before visual editing.".into());
        }
    } else {
        request
            .sources
            .insert("texloom-objects.sty".into(), OBJECT_STYLE.into());
    }
    let mut ids = std::collections::HashSet::new();
    for (name, source) in &request.sources {
        if name.ends_with(".tex") {
            for object in texloom_core::text_objects(source)? {
                if !ids.insert(object.id) {
                    return Err("Text object IDs must be unique across the project".into());
                }
            }
        }
    }
    Ok(ObjectPlan {
        sources: request.sources,
        object_id: request.id,
        file: request.file,
        operation: request.action,
    })
}

impl Backend {
    pub fn resolve_text_selection(
        &self,
        project: &str,
        revision: u64,
        id: &str,
        page: usize,
        points: Vec<SelectionPoint>,
    ) -> Result<ResolvedSelection> {
        if points.is_empty() || points.len() > 64 {
            return Err("Select between one and 64 visible text lines".into());
        }
        self.verify_project_build(project, revision, id, true)?;
        let current = self.document_snapshot(project)?;
        if current.revision != revision {
            return Err("Compile the current source before selecting layout text".into());
        }
        let started = Instant::now();
        let build = self
            .builds
            .lock()
            .map_err(fail)?
            .get(id)
            .cloned()
            .ok_or("Compile the document first")?;
        let document =
            fs::read_to_string(build.directory.join(relative(&build.main)?)).map_err(fail)?;
        let mut file: Option<String> = None;
        let mut source = String::new();
        let mut units = Vec::new();
        for point in points {
            if started.elapsed() > Duration::from_secs(15) {
                return Err(
                    "Selection mapping exceeded its time limit. Use source editing.".into(),
                );
            }
            let location = self.inverse(id, page, point.x, point.y)?;
            if file.as_ref().is_some_and(|name| name != &location.file) {
                return Err("A text group must come from one source file".into());
            }
            if file.is_none() {
                source = fs::read_to_string(build.directory.join(relative(&location.file)?))
                    .map_err(fail)?;
                if current.sources.get(&location.file) != Some(&source) {
                    return Err("The mapped source is not a current editable project file".into());
                }
                file = Some(location.file.clone());
            }
            let unit =
                texloom_core::resolve_text_unit_in_document(&source, location.line, &document)?;
            if units
                .iter()
                .any(|u: &texloom_core::SourceUnit| u.range == unit.range)
            {
                continue;
            }
            units.push(unit);
        }
        // Validation uses the main preamble for simple macros in included files.
        let unit = texloom_core::combine_text_units_in_document(&source, &units, &document)?;
        texloom_core::validate_movable_text(&unit.content, &document)?;
        let plain_text = texloom_core::plain_text_preview(&unit.content, &document);
        if started.elapsed() > Duration::from_secs(15) {
            return Err("Selection mapping exceeded its time limit. Use source editing.".into());
        }
        self.verify_project_build(project, revision, id, true)?;
        if self.document_snapshot(project)?.revision != revision {
            return Err("The source changed during selection mapping".into());
        }
        let file = file.ok_or("No source file was found")?;
        let projection = texloom_core::document::projection::EditorProjection::new(&source);
        Ok(ResolvedSelection {
            schema: 1,
            unit_id: format!("{id}:{file}:{}:{}", unit.range.start, unit.range.end),
            build_id: id.into(),
            revision: build.revision,
            file,
            start: projection.editor_offset(unit.range.start).map_err(fail)?,
            end: projection.editor_offset(unit.range.end).map_err(fail)?,
            expected_source: source,
            first_line: unit.first_line,
            last_line: unit.last_line,
            kind: unit.kind.into(),
            content: unit.content,
            plain_text,
            rectangles: Vec::new(),
        })
    }
}
