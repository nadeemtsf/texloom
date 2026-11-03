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
    /// Existing positioned StudioBlock controls have a separate compatibility
    /// contract. They do not authorize arbitrary source replacements.
    Legacy {
        file: String,
        request: TransformRequest,
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
        zero: Option<Sources>,
    },
    Reflow {
        marker: String,
        zero: Option<Sources>,
    },
    Detach {
        marker: String,
        dx: f64,
        dy: f64,
        zero: Sources,
    },
    Legacy,
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
        .entry("studio.sty".into())
        .or_insert_with(|| STYLE.into());
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
                let planned = visual::plan_object_edit(request.clone())?;
                let policy = match request.action.as_str() {
                    "wrap" => {
                        // Privately prove the same wrapper and package at zero
                        // offset before authorizing either movement or reflow.
                        let mut zero_request = request.clone();
                        zero_request.dx = Some(0.0);
                        zero_request.dy = Some(0.0);
                        zero_request.content = None;
                        zero_request.literal_edits = None;
                        zero_request.spacing_edits = None;
                        let zero = visual::plan_object_edit(zero_request)?.sources;
                        let original = object(&zero, &request.file, &request.id)?;
                        let next = object(&planned.sources, &request.file, &request.id)?;
                        if original.content == next.content {
                            Policy::Move {
                                marker,
                                dx: next.dx,
                                dy: next.dy,
                                zero: Some(zero),
                            }
                        } else {
                            if next.dx != 0.0 || next.dy != 0.0 {
                                return Err(
                                    "Edit the text and move it in separate operations".into()
                                );
                            }
                            Policy::Reflow {
                                marker,
                                zero: Some(zero),
                            }
                        }
                    }
                    "move" => {
                        let old = object(before, &request.file, &request.id)?;
                        let next = object(&planned.sources, &request.file, &request.id)?;
                        Policy::Move {
                            marker,
                            dx: next.dx - old.dx,
                            dy: next.dy - old.dy,
                            zero: None,
                        }
                    }
                    "edit" => Policy::Reflow { marker, zero: None },
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
            VisualOperation::Legacy { file, request } => {
                relative(&file)?;
                if before.get(&file) != Some(&request.source) {
                    return Err("The positioned-block source changed".into());
                }
                let mut sources = before.clone();
                sources.insert(file, transform(request)?);
                (sources, Policy::Legacy)
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
    fn private_layout_build(
        &self,
        request: &CompileRequest,
        sources: Sources,
        lease: &Lease,
    ) -> Result<Arc<Build>> {
        let mut request = request.clone();
        request.sources = sources;
        request.purpose = "verification".into();
        let result = self.compile(request, lease)?;
        if !result.ok || !result.verification_eligible {
            return Err("The private layout probe did not compile cleanly".into());
        }
        self.layout_build(&result.build_id)
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
            Policy::Legacy => { /* Native source plan plus existing PDF.js block acceptance. */ }
            Policy::Move { marker, zero, .. } | Policy::Reflow { marker, zero } => {
                let tagged = if let Some(sources) = zero {
                    let probe = self.private_layout_build(request, sources.clone(), lease)?;
                    same_recipe(&plan.baseline, &probe)?;
                    let original = self.inspect_layout(&plan.baseline, None, &lease.cancel)?;
                    let probe_view = self.inspect_layout(&probe, None, &lease.cancel)?;
                    verification::compare(&original.view, &probe_view.view, Operation::Identity)
                        .map_err(fail)?;
                    private_builds.push(probe.clone());
                    probe
                } else {
                    plan.baseline.clone()
                };
                let old = self.inspect_layout(&tagged, Some(marker), &lease.cancel)?;
                let next = self.inspect_layout(&build, Some(marker), &lease.cancel)?;
                match &plan.policy {
                    Policy::Move { dx, dy, .. } => {
                        let prediction=self.predict_layout(&tagged,marker,*dx,*dy,&lease.cancel)?;
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
