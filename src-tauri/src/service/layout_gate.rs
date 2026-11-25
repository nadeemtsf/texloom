//! Native visual-operation authority. The UI proposes an operation; Rust
//! replans it against canonical bytes, compiles it, and owns the approval.
//! No IPC value can manufacture an Approval or turn a selection probe into one.
use super::*;
use texloom_build::{
    inspection_worker::{CheckedInspection, Worker},
    scheduler::{Cancellation, Lease},
    snapshot::Fingerprint,
    verification::{self, Inspection, Operation},
};
type Sources = BTreeMap<String, String>;

#[derive(Serialize,Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[cfg_attr(feature="typegen",derive(ts_rs::TS))]
pub enum VisualOperation {
    Adapter { request: super::AdapterRequest },
    Text {
        request: ObjectEditRequest,
    },

}

pub(super) struct Plan {
    baseline: Arc<Build>,
    policy: Policy,
}
enum Policy {
    Semantic,
    Move {
        marker: String,
        dx: f64,
        dy: f64,
    },
    Reflow {
        marker: String,
    },
    Detach {
        marker: String,
        dx: f64,
        dy: f64,
        zero: Sources,
    },
    Reorder {
        original_order: Vec<String>,
        new_order: Vec<usize>,
        before_probe: Sources,
        after_probe: Sources,
    },
    DirectText {
        marker: String,
        before_probe: Option<Sources>,
        after_probe: Sources,
        displacement: Option<(f64, f64)>,
    },
}
/// Retained only inside the pending native transaction. Artifacts are pinned
/// against cache eviction until commit/discard. Not serializable or cloneable.
pub(super) struct Approval {
    build_id: String,
    build: Arc<Build>,
    baseline: Arc<Build>,
    private_builds: Vec<Arc<Build>>,
    cancel: Cancellation,
}
impl Approval {
    pub(super) fn check(&self, project: &str, base: u64, build_id: &str) -> Result<()> {
        if self.build_id != build_id
            || self.build.project_id != project
            || self.baseline.project_id != project
            || self.baseline.revision != base
            || self.build.revision != base.checked_add(1).ok_or("Revision exhausted")?
            || !self.build.eligible
        {
            return Err("The visual approval belongs to a different transaction".into());
        }
        self.cancel.check()?;
        self.build.cancel.check()?;
        for build in std::iter::once(&self.baseline)
            .chain(&self.private_builds)
            .chain(std::iter::once(&self.build))
        {
            build.receipt.verify()?;
        }
        self.cancel.check()
    }
}

fn object(sources: &Sources, file: &str, id: &str) -> Result<texloom_core::TextObject> {
    texloom_core::text_objects(sources.get(file).ok_or("Source file is missing")?)?
        .into_iter()
        .find(|o| o.id == id)
        .ok_or("Text object is missing".into())
}
fn source_receipt(build: &Build, sources: &Sources, main: &str) -> Result<()> {
    let mut expected = sources.clone();
    expected
        .entry("texloom-objects.sty".into())
        .or_insert_with(|| OBJECT_STYLE.into());
    let mut expected = expected
        .iter()
        .map(|(n, s)| (n.clone(), Fingerprint::bytes(s.as_bytes())))
        .collect::<BTreeMap<_, _>>();
    expected.extend(
        build
            .receipt
            .assets
            .iter()
            .map(|(n, a)| (n.clone(), a.fingerprint.clone())),
    );
    if build.main != main || build.receipt.inputs != expected {
        return Err("That PDF was compiled from different source bytes".into());
    }
    Ok(())
}
impl Plan {
    pub(super) fn prepare(
        baseline: Arc<Build>,
        before: &Sources,
        main: &str,
        proposed: &Sources,
        operation: VisualOperation,
    ) -> Result<Self> {
        source_receipt(&baseline, before, main)?;
        let (expected, policy) = match operation {
            VisualOperation::Adapter { request } => {
                if request.sources != *before || request.main != main { return Err("The adapter request does not match the current document".into()); }
                (super::plan_adapter(request)?.sources, Policy::Semantic)
            }
            VisualOperation::Text { request } => {
                if request.sources != *before || request.main != main {
                    return Err("The visual request does not match the current document".into());
                }
                let marker = format!("Texloom_{}", request.id);
                if request.action == "reorder_paragraph" {
                    let plan = visual::plan_paragraph_reorder(&request)?;
                    (plan.sources, Policy::Reorder {
                        original_order: plan.original_order, new_order: plan.new_order,
                        before_probe: plan.before_probe, after_probe: plan.after_probe,
                    })
                } else if matches!(request.action.as_str(), "replace_source" | "append_paragraph" | "move_source") {
                    let plan = visual::plan_direct_text(&request)?;
                    (plan.sources, Policy::DirectText {
                        marker,
                        before_probe: plan.before_probe,
                        after_probe: plan.after_probe,
                        displacement: plan.displacement,
                    })
                } else {
                    let planned = visual::plan_object_edit(request.clone())?;
                    let policy = match request.action.as_str() {
                        "move" => {
                            let old = object(before, &request.file, &request.id)?;
                            let next = object(&planned.sources, &request.file, &request.id)?;
                            Policy::Move {
                                marker,
                                dx: next.dx - old.dx,
                                dy: next.dy - old.dy,
                            }
                        }
                        "edit" => Policy::Reflow { marker },
                        "detach" => {
                            let old = object(before, &request.file, &request.id)?;
                            let mut zero_request = request.clone();
                            zero_request.action = "move".into();
                            zero_request.dx = Some(0.0);
                            zero_request.dy = Some(0.0);
                            let zero = visual::plan_object_edit(zero_request)?.sources;
                            Policy::Detach {
                                marker,
                                dx: -old.dx,
                                dy: -old.dy,
                                zero,
                            }
                        }
                        _ => return Err("Unsupported text operation".into()),
                    };
                    (planned.sources, policy)
                }
            }
        };
        if expected != *proposed {
            return Err("The candidate differs from the native operation plan".into());
        }
        if expected == *before {
            return Err("This visual operation does not change the source".into());
        }
        Ok(Self { baseline, policy })
    }
}

fn same_recipe(a: &Build, b: &Build) -> Result<()> {
    let environment = |build: &Build| -> Result<BTreeMap<String, String>> {
        let mut env = build.receipt.environment.clone();
        if let Some(output) = env.remove("TEXMFOUTPUT") {
            if output != build.directory.join(".out").to_string_lossy() {
                return Err("Compiler output environment escaped its build".into());
            }
        }
        for key in ["BIBINPUTS","BSTINPUTS"] {if let Some(value)=env.remove(key){if value!=texloom_build::engine::bibliography_search_path(&build.directory){return Err("Bibliography search path escaped its build".into())}}}
        Ok(env)
    };
    if a.project_id != b.project_id
        || a.main != b.main
        || a.receipt.compiler.path != b.receipt.compiler.path
        || a.receipt.compiler.fingerprint != b.receipt.compiler.fingerprint
        || a.receipt.engine_version != b.receipt.engine_version
        || a.receipt.arguments != b.receipt.arguments
        || a.receipt.auxiliary_tools.keys().collect::<Vec<_>>() != b.receipt.auxiliary_tools.keys().collect::<Vec<_>>()
        || a.receipt.auxiliary_tools.iter().any(|(name,tool)|b.receipt.auxiliary_tools.get(name).is_none_or(|other|tool.binary.path!=other.binary.path||tool.binary.fingerprint!=other.binary.fingerprint||tool.version!=other.version))
        || environment(a)? != environment(b)?
    {
        return Err("The compiler configuration changed during verification".into());
    }
    Ok(())
}
fn reflow(old: &Inspection, new: &Inspection) -> Result<()> {
    // Reflow may change page/line positions, but must not silently change the
    // content of unrelated text. Membership comes from inspected mark handles.
    let outside = |v: &Inspection| {
        let mut counts = BTreeMap::new();
        for g in v.glyphs.iter().filter(|g| !g.target) {
            *counts.entry(g.text.clone()).or_insert(0usize) += 1;
        }
        counts
    };
    if outside(old) != outside(new) {
        return Err("Text outside the edited object changed".into());
    }
    if old.graphics.len() != new.graphics.len() {
        return Err("A text edit changed non-text object count".into());
    }
    if !new.glyphs.iter().any(|g| g.target) {
        return Err("The edited text has no visible target".into());
    }
    Ok(())
}

// Compare the two independently marked occurrences. Counts retain repeated
// glyphs and the ordering check uses nonoverlapping physical line bounds.
fn reorder_bounds(view: &Inspection) -> Result<(usize, f64, f64)> {
    let mut glyphs = view.glyphs.iter().filter(|glyph| glyph.target);
    let first = glyphs.next().ok_or("The paragraph has no visible target")?;
    let mut top = first.bounds.y;
    let mut bottom = top + first.bounds.height;
    for glyph in glyphs {
        if glyph.page != first.page {
            return Err("Reordering across pages needs a separate verification policy".into());
        }
        top = top.min(glyph.bounds.y);
        bottom = bottom.max(glyph.bounds.y + glyph.bounds.height);
    }
    Ok((first.page, top, bottom))
}
fn verify_reorder_identity(before: &Inspection, after: &Inspection) -> Result<()> {
    reflow(before, after)?;
    let counts = |view: &Inspection| {
        let mut counts = BTreeMap::new();
        for glyph in view.glyphs.iter().filter(|glyph| glyph.target) {
            *counts.entry(glyph.text.clone()).or_insert(0usize) += 1;
        }
        counts
    };
    if counts(before) != counts(after) || before.pages.len() != after.pages.len()
        || before.pages.iter().zip(&after.pages).any(|(a,b)| {
            a.width != b.width || a.height != b.height || a.annotations != 0 || b.annotations != 0
        })
    {
        return Err("Reordering changed visible text or page structure".into());
    }
    Ok(())
}
fn verify_reorder_order(order: impl IntoIterator<Item = (usize, f64, f64)>) -> Result<()> {
    let mut order = order.into_iter();
    let Some((page, _, mut bottom)) = order.next() else {
        return Err("The PDF has no reordering evidence".into());
    };
    for (next_page, next_top, next_bottom) in order {
        if next_page != page || bottom > next_top {
            return Err("The PDF does not show the paragraphs in the expected order on one page".into());
        }
        bottom = next_bottom;
    }
    Ok(())
}

fn inline_stops(glyphs: &[verification::Glyph], page: usize, text: &str) -> Result<Vec<f64>> {
    const UNSUPPORTED: &str = "This line has no reliable character positions. Edit it in the source editor.";
    if text.is_empty()
        || text.len() > 160
        || text.trim() != text
        || text.contains("  ")
        || text.contains("--")
        || !text.bytes().all(|byte| byte.is_ascii_alphanumeric() || b" .,!?;:-".contains(&byte))
    {
        return Err(UNSUPPORTED.into());
    }

    let characters: Vec<_> = text.char_indices().filter(|(_, character)| *character != ' ').collect();
    let marked: Vec<_> = glyphs.iter().filter(|glyph| glyph.target).collect();
    if marked.len() != characters.len()
        || marked.iter().zip(&characters).any(|(glyph, (_, character))| {
            glyph.page != page || !glyph.text.starts_with(*character) || glyph.text.len() != 1
                || glyph.bounds.width <= 0.0 || glyph.bounds.height <= 0.0
        })
    {
        return Err(UNSUPPORTED.into());
    }

    let mut stops = Vec::with_capacity(text.len() + 1);
    stops.push(marked[0].bounds.x);
    for (pair, offsets) in marked.windows(2).zip(characters.windows(2)) {
        let a = pair[0].bounds;
        let b = pair[1].bounds;
        let right = a.x + a.width;
        let a_center = a.x + a.width / 2.0;
        let b_center = b.x + b.width / 2.0;
        let between = (right + b.x) / 2.0;
        if b.x + b.width <= right || a_center >= between || between >= b_center {
            return Err(UNSUPPORTED.into());
        }
        // All letters must share roughly the same baseline, even if their
        // ascent and descent differ. The PDF.js proof also requires one line.
        if (a.y + a.height - b.y - b.height).abs() > a.height.max(b.height) / 2.0 {
            return Err(UNSUPPORTED.into());
        }
        if offsets[1].0 > offsets[0].0 + 1 {
            stops.extend([right, b.x]);
        } else {
            stops.push(between);
        }
    }
    let last = marked[marked.len() - 1].bounds;
    stops.push(last.x + last.width);
    if stops.iter().any(|x| !x.is_finite() || *x < 0.0)
        || stops.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(UNSUPPORTED.into());
    }
    Ok(stops)
}

impl Backend {
    /// Resource directory is obtained from the native Tauri host, never IPC.
    pub fn with_verification_resources(mut self, resources: PathBuf) -> Self {
        self.verifier = Some((|| {
            let hash = option_env!("TEXLOOM_WORKER_SHA256").filter(|s| !s.is_empty())
                .ok_or("This build has no packaged layout verifier. Rebuild the complete desktop package.")?;
            let root = resources.join("verification");
            Worker::new(
                root.join("pdfium_worker.exe"),
                hash.into(),
                root.join("pdfium.dll"),
                self.root
                    .join("inspections")
                    .join(Uuid::new_v4().simple().to_string()),
            )
        })());
        self
    }
    pub(super) fn layout_build(&self, id: &str) -> Result<Arc<Build>> {
        self.builds
            .lock()
            .map_err(fail)?
            .get(id)
            .cloned()
            .ok_or("The compiled artifact expired".into())
    }
    pub(super) fn plan_visual(
        &self,
        project: &str,
        revision: u64,
        baseline: &str,
        before: &Sources,
        main: &str,
        proposed: &Sources,
        operation: VisualOperation,
    ) -> Result<Plan> {
        let build = self.layout_build(baseline)?;
        if build.project_id != project || build.revision != revision || !build.eligible {
            return Err("Compile the current document before visual editing".into());
        }
        build.receipt.verify()?;
        Plan::prepare(build, before, main, proposed, operation)
    }
    fn inspect_layout(
        &self,
        build: &Build,
        marker: Option<&str>,
        cancel: &Cancellation,
    ) -> Result<CheckedInspection> {
        build.receipt.verify()?;
        let sha = &build
            .receipt
            .outputs
            .get(".out/preview.pdf")
            .ok_or("PDF receipt is missing")?
            .sha256;
        self.verifier
            .as_ref()
            .ok_or("Layout verification is unavailable in this build")?
            .as_ref()
            .map_err(Clone::clone)?
            .inspect(
                &build.directory.join(".out/preview.pdf"),
                sha,
                marker,
                cancel,
            )
    }
    pub fn inline_caret_stops(&self, request: crate::build_protocol::InlineCaretRequest) -> Result<Vec<f64>> {
        let current = self.document_snapshot(&request.project_id)?;
        if current.revision != request.revision
            || request.marker.len() != 18
            || !request.marker.strip_prefix("selection-").is_some_and(|suffix| suffix.bytes().all(|byte| byte.is_ascii_hexdigit()))
        {
            return Err("Select the text again after compiling the current document".into());
        }
        let before = self.layout_build(&request.build_id)?;
        let probe = self.layout_build(&request.probe_build_id)?;
        let probe_revision = request.revision.checked_add(1).ok_or("Invalid selection revision")?;
        self.verify_project_build(&request.project_id, request.revision, &request.build_id, true)?;
        self.verify_project_build(&request.project_id, probe_revision, &request.probe_build_id, true)?;
        if probe.purpose != "visual" {
            return Err("Select the text again to verify its character positions".into());
        }
        same_recipe(&before, &probe)?;
        let marker = format!("Texloom_{}", request.marker);
        let checked = self.inspect_layout(&probe, Some(&marker), &probe.cancel)?;
        let page = request.page.checked_sub(1).ok_or("Invalid PDF page")?;
        inline_stops(&checked.view.glyphs, page, &request.text)
    }
    fn private_layout_build(
        &self,
        request: &CompileRequest,
        sources: Sources,
        lease: &Lease,
    ) -> Result<Arc<Build>> {
        let request = CompileRequest {
            project_id: request.project_id.clone(),
            main: request.main.clone(),
            sources,
            engine: request.engine.clone(),
            revision: request.revision,
            purpose: "verification".into(),
        };
        let result = self.compile(request, lease)?;
        if !result.ok || !result.verification_eligible {
            return Err("The private layout probe did not compile cleanly".into());
        }
        self.layout_build(&result.build_id)
    }
    fn equivalent_text_probe(
        &self,
        request: &CompileRequest,
        sources: &Sources,
        unmarked: &Build,
        lease: &Lease,
    ) -> Result<Arc<Build>> {
        let probe = self.private_layout_build(request, sources.clone(), lease)?;
        same_recipe(unmarked, &probe)?;
        let original = self.inspect_layout(unmarked, None, &lease.cancel)?;
        let marked = self.inspect_layout(&probe, None, &lease.cancel)?;
        verification::compare(&original.view, &marked.view, Operation::Identity).map_err(fail)?;
        Ok(probe)
    }
    fn predict_layout(&self,build:&Build,marker:&str,dx:f64,dy:f64,cancel:&Cancellation)->Result<CheckedInspection>{
        let sha=&build.receipt.outputs.get(".out/preview.pdf").ok_or("PDF receipt is missing")?.sha256;
        self.verifier.as_ref().ok_or("Layout verification is unavailable in this build")?.as_ref().map_err(Clone::clone)?.predict_displacement(&build.directory.join(".out/preview.pdf"),sha,marker,dx,dy,cancel)
    }
    pub(super) fn approve_layout(
        &self,
        plan: &Plan,
        request: &CompileRequest,
        result: &CompileResult,
        lease: &Lease,
    ) -> Result<Approval> {
        lease.cancel.check()?;
        let build = self.layout_build(&result.build_id)?;
        if !result.ok
            || !result.verification_eligible
            || !build.eligible
            || build.project_id != request.project_id
            || build.revision != request.revision
            || build.sequence != lease.id
            || build.purpose != "visual"
        {
            return Err("The build does not belong to this visual job".into());
        }
        build.cancel.check()?;
        same_recipe(&plan.baseline, &build)?;
        source_receipt(&build, &request.sources, &request.main)?;
        let mut private_builds = Vec::new();
        match &plan.policy {
            Policy::Semantic => {
                // Parameter/style edits explicitly permit reflow. The exact
                // source plan is rederived natively; no displacement claim is
                // made. Require independent readable page/paint evidence.
                let view = self.inspect_layout(&build, None, &lease.cancel)?;
                verification::validate(&view.view).map_err(fail)?;
            }
            Policy::Reorder { original_order, new_order, before_probe, after_probe } => {
                let before = self.equivalent_text_probe(request, before_probe, &plan.baseline, lease)?;
                let after = self.equivalent_text_probe(request, after_probe, &build, lease)?;
                let compare_marker = |marker: &str| -> Result<_> {
                    let old = self.inspect_layout(&before, Some(marker), &lease.cancel)?;
                    let next = self.inspect_layout(&after, Some(marker), &lease.cancel)?;
                    verify_reorder_identity(&old.view, &next.view)?;
                    let old_bounds = reorder_bounds(&old.view)?;
                    let new_bounds = reorder_bounds(&next.view)?;
                    if old_bounds.0 != new_bounds.0 {
                        return Err("This reorder would move the paragraph to another page".into());
                    }
                    Ok((old_bounds, new_bounds))
                };
                let bounds = original_order.iter().map(|marker| compare_marker(marker))
                    .collect::<Result<Vec<_>>>()?;
                verify_reorder_order(bounds.iter().map(|(old, _)| *old))?;
                verify_reorder_order(new_order.iter().map(|index| bounds[*index].1))?;
                private_builds.extend([before, after]);
            }
            Policy::DirectText { marker, before_probe, after_probe, displacement } => {
                let baseline = if let Some(sources) = before_probe {
                    let probe = self.equivalent_text_probe(request, sources, &plan.baseline, lease)?;
                    private_builds.push(probe.clone());
                    probe
                } else {
                    plan.baseline.clone()
                };
                let candidate = self.equivalent_text_probe(request, after_probe, &build, lease)?;
                let before = self.inspect_layout(
                    &baseline, before_probe.as_ref().map(|_| marker.as_str()), &lease.cancel,
                )?;
                let after = self.inspect_layout(&candidate, Some(marker), &lease.cancel)?;
                if let Some((dx, dy)) = displacement {
                    let prediction = self.predict_layout(&baseline, marker, *dx, *dy, &lease.cancel)?;
                    verification::compare_prediction(&before.view, &after.view, *dx, *dy, &prediction.view)
                        .map_err(fail)?;
                } else {
                    reflow(&before.view, &after.view)?;
                }
                private_builds.push(candidate);
            }
            Policy::Move { marker, .. } | Policy::Reflow { marker } => {
                let tagged = &plan.baseline;
                let old = self.inspect_layout(tagged, Some(marker), &lease.cancel)?;
                let next = self.inspect_layout(&build, Some(marker), &lease.cancel)?;
                match &plan.policy {
                    Policy::Move { dx, dy, .. } => {
                        let prediction=self.predict_layout(tagged,marker,*dx,*dy,&lease.cancel)?;
                        verification::compare_prediction(
                            &old.view,
                            &next.view,
                            *dx,*dy,&prediction.view,
                        )
                        .map_err(fail)?;
                    }
                    Policy::Reflow { .. } => reflow(&old.view, &next.view)?,
                    _ => unreachable!(),
                }
            }
            Policy::Detach {
                marker,
                dx,
                dy,
                zero,
            } => {
                let probe = self.private_layout_build(request, zero.clone(), lease)?;
                same_recipe(&plan.baseline, &probe)?;
                let old = self.inspect_layout(&plan.baseline, Some(marker), &lease.cancel)?;
                let next = self.inspect_layout(&probe, Some(marker), &lease.cancel)?;
                let prediction=self.predict_layout(&plan.baseline,marker,*dx,*dy,&lease.cancel)?;
                verification::compare_prediction(
                    &old.view,
                    &next.view,
                    *dx,*dy,&prediction.view,
                )
                .map_err(fail)?;
                drop(old);
                drop(next);
                let unwrapped = self.inspect_layout(&build, None, &lease.cancel)?;
                let wrapped = self.inspect_layout(&probe, None, &lease.cancel)?;
                verification::compare(&unwrapped.view, &wrapped.view, Operation::Identity)
                    .map_err(fail)?;
                private_builds.push(probe);
            }
        }
        let approval = Approval {
            build_id: result.build_id.clone(),
            build,
            baseline: plan.baseline.clone(),
            private_builds,
            cancel: lease.cancel.clone(),
        };
        approval.check(
            &request.project_id,
            plan.baseline.revision,
            &result.build_id,
        )?;
        Ok(approval)
    }
}

#[cfg(test)]
mod tests;
