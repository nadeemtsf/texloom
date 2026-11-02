use super::*;
use serde_json::json;
use texloom_build::snapshot::Receipt;

struct Fixture {
    root: PathBuf,
    backend: Backend,
}
impl Fixture {
    fn new(source: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("texloom-layout-gate-{}", Uuid::new_v4().simple()));
        let project = root.join("project");
        fs::create_dir_all(&project).unwrap();
        fs::write(root.join("compiler"), b"test compiler identity").unwrap();
        fs::write(project.join("main.tex"), source).unwrap();
        // Source-receipt checks must also handle unchanged binary assets.
        fs::write(project.join("image.png"), b"fixture asset").unwrap();
        let backend = Backend::new(root.join("app"));
        backend.documents.lock().unwrap().insert(
            "project".into(),
            document::DocumentSlot::new(sources(source), "main.tex".into()).unwrap(),
        );
        backend
            .projects
            .lock()
            .unwrap()
            .insert("project".into(), project);
        Self { root, backend }
    }
    fn build(
        &self,
        sources: &Sources,
        revision: u64,
        pdf: &[u8],
        job: Option<&Lease>,
    ) -> (String, Arc<Build>) {
        let id = Uuid::new_v4().simple().to_string();
        let mut receipt = Receipt::capture(
            &self.root.join("project"),
            &self.root.join(&id),
            "main.tex",
            sources,
            &BTreeMap::from([
                ("studio.sty".into(), STYLE.into()),
                ("texloom-objects.sty".into(), OBJECT_STYLE.into()),
            ]),
            &self.root.join("compiler"),
        )
        .unwrap();
        fs::write(receipt.directory.join(".out/preview.pdf"), pdf).unwrap();
        receipt.seal_outputs().unwrap();
        let build = Arc::new(Build {
            directory: receipt.directory.clone(),
            compiler: receipt.compiler.path.clone(),
            project_id: "project".into(),
            main: "main.tex".into(),
            revision,
            receipt,
            eligible: true,
            purpose: "visual".into(),
            cancel: job.map(|l| l.cancel.clone()).unwrap_or_default(),
            sequence: job.map(|l| l.id).unwrap_or(0),
            bytes: pdf.len() as u64,
        });
        self.backend
            .builds
            .lock()
            .unwrap()
            .insert(id.clone(), build.clone());
        (id, build)
    }
    fn request(&self, action: &str) -> ObjectEditRequest {
        serde_json::from_value(
            json!({"sources":self.backend.document_snapshot("project").unwrap().sources,
            "main":"main.tex","file":"main.tex","action":action,"id":"sig","dx":3,"dy":8}),
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn sources(source: &str) -> Sources {
    BTreeMap::from([("main.tex".into(), source.into())])
}
fn document() -> &'static str {
    "\\documentclass{article}\n\\usepackage{texloom-objects}\n\\begin{document}\nBefore\n\n\\TexloomObject{sig}{0}{0}{First\\\\[2pt]Second}\n\nNeighbor\n\\end{document}\n"
}
fn legacy(source: &str) -> TransformRequest {
    serde_json::from_value(
        json!({"source":source,"action":"patch","id":"block","changes":{"x":20}}),
    )
    .unwrap()
}
fn result(id: &str) -> CompileResult {
    CompileResult {
        ok: true,
        build_id: id.into(),
        revision: 1,
        pdf: vec![],
        blocks: vec![],
        objects: vec![],
        log: String::new(),
        engine: "pdflatex".into(),
        seconds: 0.,
        verification_eligible: true,
        passes: 2,
        diagnostics: vec![],
        job_id: 1,
    }
}
fn compile_request(sources: Sources) -> CompileRequest {
    CompileRequest {
        project_id: "project".into(),
        main: "main.tex".into(),
        sources,
        engine: "auto".into(),
        revision: 1,
        purpose: "visual".into(),
    }
}

#[test]
fn native_plan_rejects_forged_candidates_sources_profiles_and_baselines() {
    let f = Fixture::new(document());
    let current = f.backend.document_snapshot("project").unwrap();
    let (baseline, _) = f.build(&current.sources, 0, b"PDF", None);
    let request = f.request("move");
    let proposed = visual::plan_object_edit(request.clone()).unwrap().sources;
    assert!(f
        .backend
        .document_prepare_visual(
            "project",
            0,
            proposed.clone(),
            &baseline,
            Some(VisualOperation::Text {
                request: request.clone()
            })
        )
        .is_ok());
    let mut forged = proposed.clone();
    forged.get_mut("main.tex").unwrap().push_str("% unrelated");
    assert!(f
        .backend
        .document_prepare_visual(
            "project",
            0,
            forged,
            &baseline,
            Some(VisualOperation::Text {
                request: request.clone()
            })
        )
        .is_err());
    let mut stale = request.clone();
    stale
        .sources
        .get_mut("main.tex")
        .unwrap()
        .push_str("% stale");
    assert!(f
        .backend
        .document_prepare_visual(
            "project",
            0,
            proposed.clone(),
            &baseline,
            Some(VisualOperation::Text { request: stale })
        )
        .is_err());
    let mut wrong_main = request;
    wrong_main.main = "other.tex".into();
    assert!(f
        .backend
        .document_prepare_visual(
            "project",
            0,
            proposed.clone(),
            &baseline,
            Some(VisualOperation::Text {
                request: wrong_main
            })
        )
        .is_err());
    assert!(f
        .backend
        .document_prepare_visual("project", 1, proposed.clone(), &baseline, None)
        .is_err());
    let (unrelated, _) = f.build(&sources("Unrelated source"), 0, b"PDF", None);
    assert!(f
        .backend
        .document_prepare_visual(
            "project",
            0,
            proposed,
            &unrelated,
            Some(VisualOperation::Text {
                request: f.request("move")
            })
        )
        .is_err());
    assert_eq!(
        f.backend.document_snapshot("project").unwrap().sources,
        current.sources
    );
    assert!(!f.backend.document_snapshot("project").unwrap().can_undo);
}

#[test]
fn selection_probes_cannot_commit_even_with_an_eligible_compiled_pdf() {
    let f = Fixture::new(document());
    let original = f.backend.document_snapshot("project").unwrap();
    let proposed = visual::plan_object_edit(f.request("move")).unwrap().sources;
    let pending = f
        .backend
        .document_prepare("project", 0, proposed.clone())
        .unwrap();
    let (build, _) = f.build(&proposed, 1, b"PDF", None);
    assert!(f
        .backend
        .document_commit("project", &pending.token, &build)
        .err()
        .unwrap()
        .contains("no native"));
    assert_eq!(
        f.backend.document_snapshot("project").unwrap().sources,
        original.sources
    );
    assert!(!f.backend.document_snapshot("project").unwrap().can_undo);
}

#[test]
fn wrap_plan_derives_zero_identity_and_delta_from_the_emitted_source() {
    let source="\\documentclass{article}\n\\begin{document}\nBefore\n\nFirst\\\\[2pt]Second\n\nNeighbor\n\\end{document}";
    let f = Fixture::new(source);
    let current = f.backend.document_snapshot("project").unwrap();
    let (_, baseline) = f.build(&current.sources, 0, b"PDF", None);
    let start = source.find("First").unwrap();
    let end = start + "First\\\\[2pt]Second".len();
    let mut request = f.request("wrap");
    request.selection = Some([start, end]);
    request.expected = Some(source[start..end].into());
    request.dx = Some(3.1256);
    request.dy = Some(8.0);
    let proposed = visual::plan_object_edit(request.clone()).unwrap().sources;
    let plan = Plan::prepare(
        baseline,
        &current.sources,
        "main.tex",
        &proposed,
        VisualOperation::Text { request },
    )
    .unwrap();
    let Policy::Move {
        dx,
        dy,
        zero: Some(zero),
        ..
    } = plan.policy
    else {
        panic!("Expected guarded wrap")
    };
    let next = object(&proposed, "main.tex", "sig").unwrap();
    assert_eq!(dx, next.dx);
    assert_eq!(dy, next.dy);
    let zero = object(&zero, "main.tex", "sig").unwrap();
    assert_eq!((zero.dx, zero.dy), (0., 0.));
    assert_eq!(zero.content, next.content);
}

#[test]
fn native_approvals_are_one_shot_cancelled_and_rechecked_before_history() {
    let source="\\documentclass{article}\n\\usepackage{studio}\n\\begin{document}\n\\StudioBlock{block}{1}{10}{40}{100}{12}{First}\n\\end{document}";
    let f = Fixture::new(source);
    let current = f.backend.document_snapshot("project").unwrap();
    let (baseline, _) = f.build(&current.sources, 0, b"PDF", None);
    let request = legacy(source);
    let mut proposed = current.sources.clone();
    proposed.insert("main.tex".into(), transform(request.clone()).unwrap());
    let prepared = f
        .backend
        .document_prepare_visual(
            "project",
            0,
            proposed.clone(),
            &baseline,
            Some(VisualOperation::Legacy {
                file: "main.tex".into(),
                request,
            }),
        )
        .unwrap();
    let lease = f
        .backend
        .scheduler
        .request("project")
        .unwrap()
        .acquire()
        .unwrap();
    let (build_id, build) = f.build(&proposed, 1, b"VERIFIED PDF", Some(&lease));
    let plan = f.backend.documents.lock().unwrap()["project"].pending[&prepared.token]
        .plan
        .clone()
        .unwrap();
    let approval = f
        .backend
        .approve_layout(
            &plan,
            &compile_request(proposed.clone()),
            &result(&build_id),
            &lease,
        )
        .unwrap();
    assert!(approval.check("other", 0, &build_id).is_err());
    assert!(approval.check("project", 1, &build_id).is_err());
    assert!(approval.check("project", 0, "different-build").is_err());
    fs::write(build.directory.join(".out/preview.pdf"), b"TAMPERED").unwrap();
    assert!(approval.check("project", 0, &build_id).is_err());
    fs::write(build.directory.join(".out/preview.pdf"), b"VERIFIED PDF").unwrap();
    lease.cancel.cancel();
    assert!(approval.check("project", 0, &build_id).is_err());
    drop(lease);
    let lease = f
        .backend
        .scheduler
        .request("project")
        .unwrap()
        .acquire()
        .unwrap();
    assert!(f
        .backend
        .approve_layout(
            &plan,
            &compile_request(proposed.clone()),
            &result(&build_id),
            &lease
        )
        .is_err());
    let (build_id, _) = f.build(&proposed, 1, b"VERIFIED PDF", Some(&lease));
    let approval = f
        .backend
        .approve_layout(
            &plan,
            &compile_request(proposed.clone()),
            &result(&build_id),
            &lease,
        )
        .unwrap();
    f.backend
        .documents
        .lock()
        .unwrap()
        .get_mut("project")
        .unwrap()
        .pending
        .get_mut(&prepared.token)
        .unwrap()
        .approval = Some(approval);
    assert!(f
        .backend
        .document_commit("project", &prepared.token, "wrong-build")
        .is_err());
    let committed = f
        .backend
        .document_commit("project", &prepared.token, &build_id)
        .unwrap();
    assert_eq!(committed.sources, proposed);
    assert!(committed.can_undo);
    assert!(f
        .backend
        .document_commit("project", &prepared.token, &build_id)
        .is_err());
    assert_eq!(
        f.backend
            .document_history("project", 1, false)
            .unwrap()
            .sources,
        current.sources
    );
    assert_eq!(
        f.backend
            .document_history("project", 2, true)
            .unwrap()
            .sources,
        proposed
    );
}

fn simple_pdf(dx: i32, dy: i32, neighbor: i32) -> Vec<u8> {
    let stream=format!("q 1 0 0 1 {dx} {} cm /Texloom_sig BMC BT /F1 12 Tf 20 150 Td (First) Tj ET BT /F1 12 Tf 20 130 Td (Second) Tj ET EMC Q BT /F1 12 Tf {neighbor} 30 Td (Neighbor) Tj ET",-dy);
    let objects=["<< /Type /Catalog /Pages 2 0 R >>".into(),"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".into(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 300 200] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".into(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".into(),format!("<< /Length {} >>\nstream\n{stream}\nendstream",stream.len())];
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = vec![0];
    for (i, o) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{o}\nendobj\n", i + 1).as_bytes());
    }
    let xref = pdf.len();
    pdf.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
    for offset in offsets.iter().skip(1) {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len()
        )
        .as_bytes(),
    );
    pdf
}

#[test]
#[ignore = "requires the explicitly prepared Windows worker and PDFium"]
fn independent_native_movement_approval_refuses_neighbor_drift_and_preserves_history() {
    let mut f = Fixture::new(document());
    let program = PathBuf::from(
        std::env::var_os("TEXLOOM_PDFIUM_TEST_WORKER").expect("Set checked worker path"),
    );
    let library =
        PathBuf::from(std::env::var_os("TEXLOOM_PDFIUM_TEST_DLL").expect("Set pinned DLL path"));
    f.backend.verifier = Some(
        Worker::new(
            program.clone(),
            Fingerprint::file(&program).unwrap().sha256,
            library,
            f.root.join("evidence"),
        )
        .map_err(fail),
    );
    let current = f.backend.document_snapshot("project").unwrap();
    let (baseline, _) = f.build(&current.sources, 0, &simple_pdf(0, 0, 180), None);
    let request = f.request("move");
    let proposed = visual::plan_object_edit(request.clone()).unwrap().sources;
    let prepared = f
        .backend
        .document_prepare_visual(
            "project",
            0,
            proposed.clone(),
            &baseline,
            Some(VisualOperation::Text { request }),
        )
        .unwrap();
    let lease = f
        .backend
        .scheduler
        .request("project")
        .unwrap()
        .acquire()
        .unwrap();
    let plan = f.backend.documents.lock().unwrap()["project"].pending[&prepared.token]
        .plan
        .clone()
        .unwrap();
    let (bad, _) = f.build(&proposed, 1, &simple_pdf(3, 8, 181), Some(&lease));
    assert!(f
        .backend
        .approve_layout(
            &plan,
            &compile_request(proposed.clone()),
            &result(&bad),
            &lease
        )
        .is_err());
    assert!(f
        .backend
        .document_commit("project", &prepared.token, &bad)
        .is_err());
    assert_eq!(
        f.backend.document_snapshot("project").unwrap().sources,
        current.sources
    );
    assert!(!f.backend.document_snapshot("project").unwrap().can_undo);
    let (good, _) = f.build(&proposed, 1, &simple_pdf(3, 8, 180), Some(&lease));
    let approval = f
        .backend
        .approve_layout(
            &plan,
            &compile_request(proposed.clone()),
            &result(&good),
            &lease,
        )
        .unwrap();
    f.backend
        .documents
        .lock()
        .unwrap()
        .get_mut("project")
        .unwrap()
        .pending
        .get_mut(&prepared.token)
        .unwrap()
        .approval = Some(approval);
    assert_eq!(
        f.backend
            .document_commit("project", &prepared.token, &good)
            .unwrap()
            .sources,
        proposed
    );
    assert_eq!(
        f.backend
            .document_history("project", 1, false)
            .unwrap()
            .sources,
        current.sources
    );
    assert_eq!(fs::read_dir(f.root.join("evidence")).unwrap().count(), 0);
}

#[test]
#[ignore = "requires the explicitly packaged Windows verifier resources"]
fn packaged_worker_is_bound_to_the_program_hash_embedded_in_the_desktop() {
    let f = Fixture::new(document());
    let program = PathBuf::from(
        std::env::var_os("TEXLOOM_PDFIUM_TEST_WORKER").expect("Set checked worker path"),
    );
    let library =
        PathBuf::from(std::env::var_os("TEXLOOM_PDFIUM_TEST_DLL").expect("Set pinned DLL path"));
    let resources = f.root.join("resources");
    let verifier = resources.join("verification");
    fs::create_dir_all(&verifier).unwrap();
    fs::copy(&program, verifier.join("pdfium_worker.exe")).unwrap();
    fs::copy(&library, verifier.join("pdfium.dll")).unwrap();
    assert_eq!(
        option_env!("TEXLOOM_WORKER_SHA256"),
        Some(Fingerprint::file(&program).unwrap().sha256.as_str())
    );
    let backend = Backend::new(f.root.join("packaged-app")).with_verification_resources(resources);
    let (_, build) = f.build(&sources(document()), 0, &simple_pdf(0, 0, 180), None);
    let checked = backend
        .inspect_layout(&build, Some("Texloom_sig"), &Cancellation::default())
        .unwrap();
    assert!(checked.view.glyphs.iter().any(|g| g.target));
    drop(checked);
    use std::io::Write;
    fs::OpenOptions::new()
        .append(true)
        .open(verifier.join("pdfium_worker.exe"))
        .unwrap()
        .write_all(b"changed executable")
        .unwrap();
    assert!(backend
        .inspect_layout(&build, Some("Texloom_sig"), &Cancellation::default())
        .is_err());
    assert_eq!(f.backend.document_snapshot("project").unwrap().revision, 0);
}

#[test]
fn matching_recipe_normalizes_only_the_checked_private_output_directory() {
    let f = Fixture::new(document());
    let (_, a) = f.build(&sources(document()), 0, b"PDF", None);
    let (_, b) = f.build(&sources(document()), 1, b"PDF", None);
    let mut a = (*a).clone();
    let mut b = (*b).clone();
    a.receipt.environment.insert(
        "TEXMFOUTPUT".into(),
        a.directory.join(".out").to_string_lossy().into(),
    );
    b.receipt.environment.insert(
        "TEXMFOUTPUT".into(),
        b.directory.join(".out").to_string_lossy().into(),
    );
    assert!(same_recipe(&a, &b).is_ok());
    b.receipt
        .environment
        .insert("TEXINPUTS".into(), "different-fonts-and-packages".into());
    assert!(same_recipe(&a, &b).is_err());
    b.receipt.environment.remove("TEXINPUTS");
    b.receipt
        .environment
        .insert("TEXMFOUTPUT".into(), f.root.to_string_lossy().into());
    assert!(same_recipe(&a, &b).is_err());
}
