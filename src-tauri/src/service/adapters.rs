use super::*;

#[derive(Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct AdapterParameter {
    pub key: String,
    pub label: String,
    pub value: String,
    pub reason: Option<String>,
}
#[derive(Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct AdapterControl {
    pub id: usize,
    pub kind: String,
    pub label: String,
    pub line: usize,
    pub parameters: Vec<AdapterParameter>,
    pub reason: Option<String>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct AdapterRequest {
    pub sources: BTreeMap<String, String>,
    pub main: String,
    pub file: String,
    pub expected_source: String,
    pub control: Option<usize>,
    pub selection: Option<[usize; 2]>,
    pub key: String,
    pub value: String,
}
#[derive(Serialize)]
#[cfg_attr(feature = "typegen", derive(ts_rs::TS))]
pub struct AdapterPlan {
    pub sources: BTreeMap<String, String>,
    pub file: String,
}

pub fn describe(source: &str) -> Result<Vec<AdapterControl>> {
    if source.len() > 4_000_000 {
        return Err("Source exceeds the control inspection limit".into());
    }
    Ok(texloom_core::adapters::describe(source)
        .into_iter()
        .map(|c| AdapterControl {
            id: c.id,
            kind: c.kind,
            label: c.label,
            line: c.line,
            reason: c.reason,
            parameters: c
                .parameters
                .into_iter()
                .map(|p| AdapterParameter {
                    key: p.key,
                    label: p.label,
                    value: p.value,
                    reason: p.reason,
                })
                .collect(),
        })
        .collect())
}
pub fn plan(request: AdapterRequest) -> Result<AdapterPlan> {
    relative(&request.file)?;
    relative(&request.main)?;
    let source = request
        .sources
        .get(&request.file)
        .ok_or("The source file is missing")?;
    let document = request
        .sources
        .get(&request.main)
        .ok_or("The main source is missing")?;
    let updated = if request.key == "insert_image" {
        if request.control.is_some() {
            return Err("Image insertion requires a source cursor".into());
        }
        let [_, end] = request
            .selection
            .ok_or("Place the source cursor in the document body")?;
        let projection = texloom_core::document::projection::EditorProjection::new(source);
        texloom_core::adapters::insert_image(
            source,
            &request.expected_source,
            projection.source_offset(end).map_err(fail)?,
            &request.value,
        )?
    } else if request.key == "style" {
        if request.control.is_some() {
            return Err("Text styling requires an explicit source selection".into());
        }
        let [start, end] = request.selection.ok_or("Select the source text to style")?;
        let projection = texloom_core::document::projection::EditorProjection::new(source);
        let range = projection.source_offset(start).map_err(fail)?
            ..projection.source_offset(end).map_err(fail)?;
        texloom_core::adapters::style(
            source,
            &request.expected_source,
            document,
            range,
            &request.value,
        )?
    } else {
        if request.selection.is_some() {
            return Err("Parameter changes do not accept replacement ranges".into());
        }
        texloom_core::adapters::apply(
            source,
            &request.expected_source,
            request.control.ok_or("Choose a source control")?,
            &request.key,
            &request.value,
        )?
    };
    let image = request.key == "insert_image";
    let mut sources = request.sources;
    sources.insert(request.file.clone(), updated);
    if image {
        let main = sources.get(&request.main).ok_or("Main source is missing")?;
        let updated = texloom_core::adapters::ensure_graphics_package(main)?;
        sources.insert(request.main, updated);
    }
    Ok(AdapterPlan {
        sources,
        file: request.file,
    })
}
