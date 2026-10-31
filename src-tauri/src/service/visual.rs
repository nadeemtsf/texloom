//! Visual operations prepare source candidates; they never save a project.
use super::*;

#[derive(Deserialize)]
pub struct SelectionPoint { pub x: f64, pub y: f64 }

#[derive(Serialize)]
pub struct ResolvedSelection {
    pub build_id: String, pub revision: u64, pub file: String,
    pub expected_source: String, pub start: usize, pub end: usize,
    pub first_line: usize, pub last_line: usize,
    pub kind: String, pub content: String, pub plain_text: Option<String>, pub rectangles: Vec<SourceRectangle>,
}

#[derive(Deserialize)]
pub struct ObjectEditRequest {
    pub sources: BTreeMap<String, String>, pub main: String, pub file: String,
    pub action: String, pub id: String,
    #[serde(default)] pub selection: Option<[usize; 2]>,
    #[serde(default)] pub expected: Option<String>,
    #[serde(default)] pub dx: Option<f64>,
    #[serde(default)] pub dy: Option<f64>,
    #[serde(default)] pub content: Option<String>,
    #[serde(default)] pub plain: bool,
}

#[derive(Serialize)]
pub struct ObjectPlan {
    pub sources: BTreeMap<String, String>, pub object_id: String,
    pub file: String, pub operation: String,
}

pub fn plan_object_edit(mut request: ObjectEditRequest) -> Result<ObjectPlan> {
    relative(&request.main)?; relative(&request.file)?;
    let source = request.sources.get(&request.file).ok_or("Source file is not in this project")?;
    let document = request.sources.get(&request.main).ok_or("Main document is missing")?;
    if source.len() > 4_000_000 || request.sources.len() > MAX_FILES { return Err("Project source exceeds supported limits".into()); }
    let updated = match request.action.as_str() {
        "wrap" => {
            let [start, end] = request.selection.ok_or("Select text before moving it")?;
            let range = texloom_core::utf16_to_byte(source, start)?..texloom_core::utf16_to_byte(source, end)?;
            if source.get(range.clone()) != request.expected.as_deref() { return Err("The selected source has changed; select it again".into()); }
            let wrapped=texloom_core::wrap_text_object(source, range, &request.id, request.dx.unwrap_or(0.0), request.dy.unwrap_or(0.0), document)?;
            if let Some(value)=request.content.as_deref() {
                let object=texloom_core::text_objects(&wrapped)?.into_iter().find(|o|o.id==request.id).ok_or("Object is missing")?;
                let content=if request.plain {texloom_core::replace_plain_text(&object.content,value,document)?}else{value.into()};
                texloom_core::update_text_object(&wrapped,&request.id,None,None,Some(&content),document)?
            } else {wrapped}
        }
        "move" => texloom_core::update_text_object(source, &request.id, request.dx, request.dy, None, document)?,
        "edit" => {
            let value=request.content.as_deref().ok_or("Enter the object's text")?;
            let content=if request.plain {
                let object=texloom_core::text_objects(source)?.into_iter().find(|o|o.id==request.id).ok_or("Object is missing")?;
                texloom_core::replace_plain_text(&object.content,value,document)?
            } else {value.into()};
            texloom_core::update_text_object(source,&request.id,None,None,Some(&content),document)?
        }
        "detach" => texloom_core::unwrap_text_object(source, &request.id)?,
        _ => return Err("Unsupported text-object operation".into()),
    };
    request.sources.insert(request.file.clone(), updated);
    let main = request.sources.get(&request.main).ok_or("Main document is missing")?;
    request.sources.insert(request.main, texloom_core::ensure_objects_package(main)?);
    if let Some(existing) = request.sources.get("texloom-objects.sty") {
        if existing != OBJECT_STYLE { return Err("This project has a different texloom-objects.sty. Keep its version consistent before visual editing.".into()); }
    } else { request.sources.insert("texloom-objects.sty".into(), OBJECT_STYLE.into()); }
    let mut ids = std::collections::HashSet::new();
    for (name, source) in &request.sources {
        if name.ends_with(".tex") {
            for object in texloom_core::text_objects(source)? {
                if !ids.insert(object.id) { return Err("Text object IDs must be unique across the project".into()); }
            }
        }
    }
    Ok(ObjectPlan { sources: request.sources, object_id: request.id, file: request.file, operation: request.action })
}

impl Backend {
    pub fn resolve_text_selection(&self, id: &str, page: usize, points: Vec<SelectionPoint>) -> Result<ResolvedSelection> {
        if points.is_empty() || points.len() > 64 { return Err("Select between one and 64 visible text lines".into()); }
        let build = self.builds.lock().map_err(fail)?.get(id).cloned().ok_or("Compile the document first")?;
        let document = fs::read_to_string(build.directory.join(relative(&build.main)?)).map_err(fail)?;
        let mut file: Option<String> = None;
        let mut source = String::new();
        let mut units = Vec::new();
        let mut rectangles = Vec::new();
        for point in points {
            let location = self.inverse(id, page, point.x, point.y)?;
            if file.as_ref().is_some_and(|name| name != &location.file) { return Err("A text group must come from one source file".into()); }
            if file.is_none() {
                source = fs::read_to_string(build.directory.join(relative(&location.file)?)).map_err(fail)?;
                file = Some(location.file.clone());
            }
            let unit = texloom_core::resolve_text_unit_in_document(&source, location.line, &document)?;
            if units.iter().any(|u: &texloom_core::SourceUnit| u.range == unit.range) { continue; }
            for line in unit.first_line..=unit.last_line {
                rectangles.extend(self.source_rectangles(id, &location.file, line)?);
            }
            units.push(unit);
        }
        // Validation uses the main preamble for simple macros in included files.
        let unit = texloom_core::combine_text_units_in_document(&source, &units, &document)?;
        texloom_core::validate_movable_text(&unit.content, &document)?;
        if rectangles.iter().any(|r| r.page != page) { return Err("This text crosses a page boundary and cannot be moved as one object".into()); }
        let plain_text=texloom_core::plain_text_preview(&unit.content,&document);
        Ok(ResolvedSelection { build_id: id.into(), revision: build.revision, file: file.ok_or("No source file was found")?,
            start: texloom_core::byte_to_utf16(&source, unit.range.start)?, end: texloom_core::byte_to_utf16(&source, unit.range.end)?,
            expected_source: source, first_line: unit.first_line, last_line: unit.last_line,
            kind: unit.kind.into(), content: unit.content, plain_text, rectangles })
    }

    fn source_rectangles(&self, id: &str, name: &str, line: usize) -> Result<Vec<SourceRectangle>> {
        let build = self.builds.lock().map_err(fail)?.get(id).cloned().ok_or("Compile the document first")?;
        let sibling = build.compiler.with_file_name(if cfg!(windows) { "synctex.exe" } else { "synctex" });
        let binary = if sibling.is_file() { sibling } else { find_program("synctex").ok_or("SyncTeX is unavailable")? };
        let mut command = Command::new(binary);
        command.current_dir(&build.directory).arg("view").arg("-i")
            .arg(format!("{line}:0:{}", build.directory.join(relative(name)?).display()))
            .arg("-o").arg(build.directory.join(".out/preview.pdf"));
        let (_, output) = run_command(command, Duration::from_secs(10))?;
        let mut records: Vec<HashMap<&str, f64>> = Vec::new();
        let mut current = HashMap::new();
        for record in output.lines() {
            if record.starts_with("Output:") && !current.is_empty() { records.push(current); current = HashMap::new(); }
            if let Some((key, value)) = record.split_once(':') {
                if let Ok(number) = value.trim().parse::<f64>() { current.insert(key.trim(), number); }
            }
        }
        if !current.is_empty() { records.push(current); }
        let result: Vec<_> = records.into_iter().filter_map(|r| {
            let page = r.get("Page").copied()? as usize;
            let x = r.get("h").or_else(|| r.get("x")).copied()?;
            let baseline = r.get("v").or_else(|| r.get("y")).copied()?;
            let height = r.get("H").copied().unwrap_or(12.0).max(1.0);
            Some(SourceRectangle { page, x, y: baseline - height, width: r.get("W").copied().unwrap_or(30.0).max(1.0), height })
        }).collect();
        if result.is_empty() { return Err("The complete selected text could not be mapped to the PDF".into()); }
        Ok(result)
    }
}
