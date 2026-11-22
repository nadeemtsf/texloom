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
        self.backend.documents.get_mut().unwrap().clear();
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn sources(source: &str) -> Sources {
    BTreeMap::from([("main.tex".into(), source.into())])
}
fn document() -> &'static str {
    "\\documentclass{article}\n\\usepackage{texloom-objects}\n\\begin{document}\nBefore\n\n\\TexloomObject{sig}{0}{0}{First\\\\[2pt]Second}\n\nNeighbor\n\\end{document}\n"
}
fn result(id: &str) -> CompileResult {
    CompileResult {
        ok: true,
        build_id: id.into(),
        revision: 1,
        pdf: vec![],
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
fn a_private_selection_wrapper_cannot_be_planned_as_a_saved_visual_move() {
    let source = "\\documentclass{article}\n\\begin{document}\nFirst\n\\end{document}";
    let f = Fixture::new(source);
    let current = f.backend.document_snapshot("project").unwrap();
    let (_, baseline) = f.build(&current.sources, 0, b"PDF", None);
    let start = source.find("First").unwrap();
    let mut request = f.request("wrap");
    request.selection = Some([start, start + "First".len()]);
    request.expected = Some("First".into());
    request.dx = Some(0.0);
    request.dy = Some(0.0);
    let probe = visual::plan_object_edit(request.clone()).unwrap().sources;
    assert!(probe.contains_key("texloom-objects.sty"));
    assert!(Plan::prepare(
        baseline, &current.sources, "main.tex", &probe, VisualOperation::Text { request },
    ).is_err());
}

fn prepare_test_verifier(f: &mut Fixture) {
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
}

#[test]
#[ignore = "requires the explicitly prepared Windows worker and PDFium"]
fn native_approvals_are_one_shot_cancelled_and_rechecked_before_history() {
    let mut f = Fixture::new(document());
    prepare_test_verifier(&mut f);
    let current = f.backend.document_snapshot("project").unwrap();
    let (baseline, _) = f.build(&current.sources, 0, &simple_pdf(0, 0, 180), None);
    let request = f.request("move");
    let proposed = visual::plan_object_edit(request.clone()).unwrap().sources;
    let candidate_pdf = simple_pdf(3, 8, 180);
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
    let (build_id, build) = f.build(&proposed, 1, &candidate_pdf, Some(&lease));
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
    fs::write(build.directory.join(".out/preview.pdf"), &candidate_pdf).unwrap();
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
    let (build_id, _) = f.build(&proposed, 1, &candidate_pdf, Some(&lease));
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
    prepare_test_verifier(&mut f);
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

#[test]
fn direct_text_plans_keep_packages_out_of_the_candidate_and_refuse_stale_text() {
    let source = "\u{feff}\\documentclass{article}\r\n\\begin{document}\r\nFirst.\r\n\r\nA \\textbf{word}.\r\n\\end{document}\r\n";
    let f = Fixture::new(source);
    let mut request = f.request("replace_source");
    request.sources.insert("custom.sty".into(), "% Keep me".into());
    let projection = EditorProjection::new(source);
    let content = "A \\textbf{word}.";
    let start = source.find(content).unwrap();
    request.selection = Some([
        projection.editor_offset(start).unwrap(),
        projection.editor_offset(start + content.len()).unwrap(),
    ]);
    request.expected = Some(content.into());
    request.dx = None;
    request.dy = None;
    request.literal_edits = Some(vec![LiteralEditRequest { id: 1, text: "phrase & text".into() }]);
    let plan = visual::plan_direct_text(&request).unwrap();
    assert_eq!(plan.sources["main.tex"], source.replace("word", "phrase \\& text"));
    assert_eq!(plan.sources["custom.sty"], "% Keep me");
    assert_eq!(plan.sources.len(), 2);
    assert!(plan.before_probe.unwrap()["main.tex"].contains("\\TexloomObject"));
    assert!(plan.after_probe["main.tex"].contains("\\TexloomObject"));
    request.expected = Some("Stale text".into());
    assert!(visual::plan_direct_text(&request).is_err());
    request.expected = Some(content.into());
    request.dx = Some(1.0);
    assert!(visual::plan_direct_text(&request).is_err());
}

#[test]
#[ignore = "requires installed pdfLaTeX, XeLaTeX, LuaLaTeX and the pinned Windows verifier"]
fn direct_text_real_engine_workflow() {
    let source = "\\documentclass{article}\n\\pagestyle{empty}\n\\begin{document}\nFirst paragraph.\n\nA \\textbf{word}.\n\nLast paragraph.\n\\end{document}\n";
    for engine in ["pdflatex", "xelatex", "lualatex"] {
        let mut f = Fixture::new(source);
        prepare_test_verifier(&mut f);
        // Use the normal project-open path so Save has a disk fingerprint.
        let path = f.root.join("project/main.tex").to_string_lossy().into_owned();
        let project = f.backend.open_project_with_profile(
            Some(path.clone()), Some(ProjectProfile { schema: 1, main: "main.tex".into(), engine: engine.into() }),
        ).unwrap();
        let id = &project.id;
        let mut expected = source.to_owned();
        for (base, action) in [(0, "replace_source"), (1, "append_paragraph")] {
            let current = f.backend.document_snapshot(id).unwrap();
            let baseline = f.backend.compile_document(id, base, None, None, base * 2 + 1).unwrap();
            assert!(baseline.ok && baseline.verification_eligible, "{engine}: {}", baseline.log);
            let selected = "A \\textbf{word}.";
            let start = source.find(selected).unwrap();
            let request: ObjectEditRequest = if action == "replace_source" {
                expected = expected.replace("\\textbf{word}", "\\textbf{phrase}");
                serde_json::from_value(json!({
                    "sources": current.sources, "main":"main.tex", "file":"main.tex",
                    "action":action, "id":"direct-edit", "selection":[start,start+selected.len()],
                    "expected":selected, "literal_edits":[{"id":1,"text":"phrase"}],
                })).unwrap()
            } else {
                expected = expected.replace("\\end{document}", "\n\nAn \\& sign.\n\n\\end{document}");
                serde_json::from_value(json!({
                    "sources":current.sources,"main":"main.tex","file":"main.tex",
                    "action":action,"id":"direct-append","content":"An & sign.","plain":true,
                })).unwrap()
            };
            let planned = visual::plan_object_edit(request.clone()).unwrap();
            assert_eq!(planned.sources, sources(&expected), "{engine} {action}");
            let prepared = f.backend.document_prepare_visual(
                id, base, planned.sources, &baseline.build_id, Some(VisualOperation::Text { request }),
            ).unwrap();
            let result = f.backend.compile_document(id, base, Some(&prepared.token), None, base * 2 + 2)
                .unwrap_or_else(|error| panic!("{engine} {action}: {error}"));
            assert!(result.ok && result.verification_eligible, "{engine} {action}: {}", result.log);
            assert!(result.objects.is_empty(), "{engine} {action} published a private marker");
            let compiled = f.backend.layout_build(&result.build_id).unwrap();
            let rendered = f.backend.inspect_layout(&compiled, None, &Cancellation::default()).unwrap();
            let text: String = rendered.view.glyphs.iter().map(|glyph| glyph.text.as_str()).collect();
            let text: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
            assert!(text.contains("Aphrase."), "{engine} {action}: {text}");
            if base == 1 { assert!(text.contains("An&sign."), "{engine}: {text}"); }
            assert_eq!(fs::read_to_string(&path).unwrap(), source);
            f.backend.document_commit(id, &prepared.token, &result.build_id).unwrap();
        }
        let undone = f.backend.document_history(id, 2, false).unwrap();
        assert_eq!(undone.sources, sources(&source.replace("\\textbf{word}", "\\textbf{phrase}")));
        let redone = f.backend.document_history(id, undone.revision, true).unwrap();
        assert_eq!(redone.sources, sources(&expected));
        f.backend.save_document(id, redone.revision).unwrap();
        let reopened = f.backend.open_project(Some(path)).unwrap();
        assert_eq!(reopened.sources, sources(&expected));
        // Invoke the installed engine directly from the saved project. The
        // app's private build snapshots and bundled files are outside this path.
        let saved_pdf = f.root.join("external");
        fs::create_dir(&saved_pdf).unwrap();
        let checked = Command::new(find_program(engine).unwrap())
            .args(["-no-shell-escape", "-halt-on-error", "-interaction=nonstopmode"])
            .arg(format!("-output-directory={}", saved_pdf.display()))
            .arg("main.tex").current_dir(f.root.join("project")).output().unwrap();
        assert!(checked.status.success(), "{engine}: {}", String::from_utf8_lossy(&checked.stdout));
        assert!(saved_pdf.join("main.pdf").is_file(), "{engine}");
        println!("{engine}: approved edit and insertion, Undo/Redo, Save/reopen and external build");
    }
}

#[test]
#[ignore = "requires installed pdfLaTeX, XeLaTeX, LuaLaTeX and the pinned Windows verifier"]
fn direct_drag_real_engine_workflow() {
    let source = "\\documentclass[12pt]{article}\n\\begin{document}\nThe PDF starts here on the first page.\n\nThis neighboring paragraph should stay put.\n\n\\newpage\nThis text belongs to the second page.\n\n\\newpage\nThe cursor should take you to this sentence on page three.\n\\end{document}\n";
    let content = "The PDF starts here on the first page.";
    for engine in ["pdflatex", "xelatex", "lualatex"] {
        let mut f = Fixture::new(source);
        prepare_test_verifier(&mut f);
        let path = f.root.join("project/main.tex").to_string_lossy().into_owned();
        let project = f.backend.open_project_with_profile(
            Some(path.clone()), Some(ProjectProfile { schema: 1, main: "main.tex".into(), engine: engine.into() }),
        ).unwrap();
        let id = &project.id;
        let mut expected = source.to_owned();
        for (base, dx, dy, total_dx, total_dy) in [(0, 88, 70, 88, 70), (1, -15, 5, 73, 75)] {
            let current = f.backend.document_snapshot(id).unwrap();
            let baseline = f.backend.compile_document(id, base, None, None, base * 2 + 1).unwrap();
            assert!(baseline.ok && baseline.verification_eligible, "{engine}: {}", baseline.log);
            let unit=texloom_core::resolve_text_unit(&expected, 3).unwrap();
            assert_eq!(unit.content, content);
            let request: ObjectEditRequest = serde_json::from_value(json!({
                "sources": current.sources, "main":"main.tex", "file":"main.tex",
                "action":"move_source", "id":"drag-test", "selection":[unit.range.start,unit.range.end],
                "expected": content, "dx": dx, "dy": dy,
            })).unwrap();
            expected = source.replace(content, &format!(
                "\\raisebox{{-{total_dy}bp}}[\\height][\\depth]{{\\kern{total_dx}bp\\relax {content}\\kern-{total_dx}bp\\relax}}"
            ));
            let planned = visual::plan_object_edit(request.clone()).unwrap();
            assert_eq!(planned.sources, sources(&expected), "{engine}");
            let prepared = f.backend.document_prepare_visual(
                id, base, planned.sources, &baseline.build_id, Some(VisualOperation::Text { request }),
            ).unwrap();
            let result = f.backend.compile_document(id, base, Some(&prepared.token), None, base * 2 + 2)
                .unwrap_or_else(|error| panic!("{engine} move {base}: {error}"));
            assert!(result.ok && result.verification_eligible, "{engine}: {}", result.log);
            assert!(result.objects.is_empty(), "{engine} published a private marker");
            assert_eq!(fs::read_to_string(&path).unwrap(), source);
            f.backend.document_commit(id, &prepared.token, &result.build_id).unwrap();
        }
        let undone = f.backend.document_history(id, 2, false).unwrap();
        assert!(undone.sources["main.tex"].contains("\\raisebox{-70bp}"));
        let redone = f.backend.document_history(id, undone.revision, true).unwrap();
        assert_eq!(redone.sources, sources(&expected));
        f.backend.save_document(id, redone.revision).unwrap();
        assert_eq!(f.backend.open_project(Some(path)).unwrap().sources, sources(&expected));
        let checked = Command::new(find_program(engine).unwrap())
            .args(["-no-shell-escape", "-halt-on-error", "-interaction=nonstopmode", "main.tex"])
            .current_dir(f.root.join("project")).output().unwrap();
        assert!(checked.status.success(), "{engine}: {}", String::from_utf8_lossy(&checked.stdout));
        println!("{engine}: two source-native drags and history, Save/reopen, external compile");
    }
}

fn reorder_request(source: &str, first: &str, second: &str, before: bool) -> ObjectEditRequest {
    let range = |text: &str| {
        let start = source.find(text).unwrap();
        [start, start + text.len()]
    };
    serde_json::from_value(json!({
        "sources": sources(source), "main":"main.tex", "file":"main.tex",
        "action":"reorder_paragraph", "id":"drag-reorder", "selection":range(first),
        "neighbor":range(second), "expected":first, "before_neighbor":before,
    })).unwrap()
}

#[test]
fn reorder_rotates_a_plain_run_and_rejects_structural_and_noop_destinations() {
    let source = "\\documentclass{article}\n\\begin{document}\nOne.\n\nTwo.\n\n\nThree.\n\\end{document}\n";
    let plan = visual::plan_paragraph_reorder(&reorder_request(source, "One.", "Three.", false)).unwrap();
    assert_eq!(plan.sources["main.tex"], source.replace("One.\n\nTwo.\n\n\nThree.", "Two.\n\nThree.\n\n\nOne."));
    assert_eq!(plan.new_order, vec![1, 2, 0]);
    let plan = visual::plan_paragraph_reorder(&reorder_request(source, "One.", "Three.", true)).unwrap();
    assert_eq!(plan.new_order, vec![1, 0, 2]);
    let plan = visual::plan_paragraph_reorder(&reorder_request(source, "Three.", "One.", false)).unwrap();
    assert_eq!(plan.new_order, vec![0, 2, 1]);
    assert!(visual::plan_paragraph_reorder(&reorder_request(source, "One.", "Two.", true)).is_err());
    let separated = source.replace("\n\nTwo.", "\n\n\\section{Boundary}\n\nTwo.");
    assert!(visual::plan_paragraph_reorder(&reorder_request(&separated, "One.", "Three.", false)).is_err());
    let duplicate = source.replace("Three.", "Two.");
    let request = reorder_request(&duplicate, "One.", "Two.", false);
    let mut later = request.clone();
    let last = duplicate.rfind("Two.").unwrap();
    later.neighbor = Some([last, last + "Two.".len()]);
    let plan = visual::plan_paragraph_reorder(&later).unwrap();
    assert_eq!(plan.new_order, vec![1, 2, 0]);
}

#[test]
#[ignore = "requires the installed TeX engines and the pinned Windows verifier"]
fn verified_reorder_real_engine_workflow() {
    let long = "First paragraph wraps across PDF lines. ".repeat(5).trim_end().to_owned();
    let middle = "This intervening paragraph stays ahead of the destination.";
    let short = "The destination paragraph is shorter.";
    let source = format!("\\documentclass[12pt]{{article}}\n\\begin{{document}}\n{long}\n\n{middle}\n\n{short}\n\nA neighboring paragraph stays here.\n\\end{{document}}\n");
    let swapped = format!("\\documentclass[12pt]{{article}}\n\\begin{{document}}\n{middle}\n\n{short}\n\n{long}\n\nA neighboring paragraph stays here.\n\\end{{document}}\n");
    for engine in ["pdflatex", "xelatex", "lualatex"] {
        let mut f = Fixture::new(&source);
        prepare_test_verifier(&mut f);
        let path = f.root.join("project/main.tex").to_string_lossy().into_owned();
        let project = f.backend.open_project_with_profile(Some(path.clone()), Some(ProjectProfile {
            schema: 1, main: "main.tex".into(), engine: engine.into(),
        })).unwrap();
        let id = &project.id;
        for (base, input, expected, neighbor, before) in [(0, &source, &swapped, short, false), (1, &swapped, &source, middle, true)] {
            let baseline = f.backend.compile_document(id, base, None, None, base * 2 + 1).unwrap();
            assert!(baseline.ok && baseline.verification_eligible, "{engine}: {}", baseline.log);
            let request = reorder_request(input, &long, neighbor, before);
            let planned = visual::plan_object_edit(request.clone()).unwrap();
            assert_eq!(planned.sources, sources(expected), "{engine}");
            let prepared = f.backend.document_prepare_visual(
                id, base, planned.sources, &baseline.build_id, Some(VisualOperation::Text { request }),
            ).unwrap();
            let result = f.backend.compile_document(id, base, Some(&prepared.token), None, base * 2 + 2)
                .unwrap_or_else(|error| panic!("{engine} reorder {base}: {error}"));
            assert!(result.ok && result.verification_eligible, "{engine}: {}", result.log);
            assert!(result.objects.is_empty());
            assert_eq!(f.backend.document_commit(id, &prepared.token, &result.build_id).unwrap().sources,sources(expected));
        }
        let undone = f.backend.document_history(id, 2, false).unwrap();
        assert_eq!(undone.sources, sources(&swapped));
        let redone = f.backend.document_history(id, undone.revision, true).unwrap();
        assert_eq!(redone.sources, sources(&source));
        let swapped_again = f.backend.document_history(id, redone.revision, false).unwrap();
        f.backend.save_document(id, swapped_again.revision).unwrap();
        assert_eq!(f.backend.open_project(Some(path)).unwrap().sources, sources(&swapped));
        let result = Command::new(find_program(engine).unwrap())
            .args(["-no-shell-escape", "-halt-on-error", "-interaction=nonstopmode", "main.tex"])
            .current_dir(f.root.join("project")).output().unwrap();
        assert!(result.status.success(), "{engine}: {}", String::from_utf8_lossy(&result.stdout));
        println!("{engine}: reordered a wrapped paragraph in both directions and verified history, Save and external compile");
    }
}

#[test]
#[ignore = "requires the installed TeX engines and the pinned Windows verifier"]
fn first_paragraph_on_a_later_page_can_be_selected_and_reordered() {
    let leading = "Delta starts the second page.";
    let next = "Echo is the next paragraph.";
    let source = format!(
        "\\documentclass[12pt]{{article}}\n\\usepackage[textheight=48pt]{{geometry}}\n\\begin{{document}}\n\
         Alpha opens the first page.\n\nBravo is next.\n\nCharlie ends the first page.\n\n\
         {leading}\n\n{next}\n\nFoxtrot ends the document.\n\\end{{document}}\n"
    );
    let expected = source.replace(&format!("{leading}\n\n{next}"), &format!("{next}\n\n{leading}"));
    for engine in ["pdflatex", "xelatex", "lualatex"] {
        let mut f = Fixture::new(&source);
        prepare_test_verifier(&mut f);
        let path = f.root.join("project/main.tex").to_string_lossy().into_owned();
        let project = f.backend.open_project_with_profile(Some(path), Some(ProjectProfile {
            schema: 1, main: "main.tex".into(), engine: engine.into(),
        })).unwrap();
        let id = &project.id;
        let baseline = f.backend.compile_document(id, 0, None, None, 1).unwrap();
        assert!(baseline.ok && baseline.verification_eligible, "{engine}: {}", baseline.log);
        let request = reorder_request(&source, leading, next, false);
        let mut probe = request.clone();
        probe.action = "probe_paragraph".into();
        probe.neighbor = None;
        let probe_sources = visual::plan_object_edit(probe).unwrap().sources;
        let prepared = f.backend.document_prepare_visual(id, 0, probe_sources, &baseline.build_id, None).unwrap();
        let selected = f.backend.compile_document(id, 0, Some(&prepared.token), None, 2).unwrap();
        assert!(selected.ok && selected.verification_eligible, "{engine}: {}", selected.log);
        let marked = f.backend.layout_build(&selected.build_id).unwrap();
        let baseline_build = f.backend.layout_build(&baseline.build_id).unwrap();
        let cancel = Cancellation::default();
        let baseline_pdf = f.backend.inspect_layout(&baseline_build, None, &cancel).unwrap();
        let probe_pdf = f.backend.inspect_layout(&marked, None, &cancel).unwrap();
        verification::compare(&baseline_pdf.view, &probe_pdf.view, Operation::Identity).unwrap();
        let probe_pdf = f.backend.inspect_layout(&marked, Some("Texloom_drag-reorder"), &cancel).unwrap();
        assert_eq!(reorder_bounds(&probe_pdf.view).unwrap().0, 1, "{engine}: selected page");
        let marked_text = probe_pdf.view.glyphs.iter().filter(|glyph| glyph.target)
            .flat_map(|glyph| glyph.text.chars()).filter(|ch| !ch.is_whitespace()).collect::<String>();
        assert_eq!(marked_text, leading.replace(' ', ""), "{engine}: selected glyphs");
        assert!(f.backend.document_commit(id, &prepared.token, &selected.build_id).is_err());
        f.backend.document_discard(id, &prepared.token).unwrap();

        let planned = visual::plan_object_edit(request.clone()).unwrap();
        assert_eq!(planned.sources, sources(&expected), "{engine}");
        let prepared = f.backend.document_prepare_visual(
            id, 0, planned.sources, &baseline.build_id, Some(VisualOperation::Text { request }),
        ).unwrap();
        let reordered = f.backend.compile_document(id, 0, Some(&prepared.token), None, 3)
            .unwrap_or_else(|error| panic!("{engine} later-page reorder: {error}"));
        assert!(reordered.ok && reordered.verification_eligible, "{engine}: {}", reordered.log);
        assert_eq!(
            f.backend.document_commit(id, &prepared.token, &reordered.build_id).unwrap().sources,
            sources(&expected),
        );
        println!("{engine}: selected the first paragraph on page two and reordered it below its neighbor");
    }
}
