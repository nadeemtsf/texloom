//! SyncTeX CLI queries are hints, never authority to mutate source. Incomplete,
//! failed or ambiguous results must not become guessed object coordinates.
use super::*;
use texloom_build::{
    process::{self, CommandSpec},
    scheduler::Cancellation,
    snapshot::Fingerprint,
};

impl Backend {
    pub(super) fn check_mapping_snapshot(&self, build: &Build) -> Result<()> {
        build.receipt.verify()?;
        if !build
            .receipt
            .outputs
            .contains_key(".out/preview.synctex.gz")
        {
            return Err(
                "The build has no verified SyncTeX file. Recompile before mapping source.".into(),
            );
        }
        let documents = self.documents.lock().map_err(fail)?;
        let slot = documents
            .get(&build.project_id)
            .ok_or("This project has closed")?;
        if slot.revision() != build.revision {
            return Err("The source changed. Compile the current version before mapping.".into());
        }
        Ok(())
    }
}

pub(super) fn run(command: Command, cancel: &Cancellation) -> Result<String> {
    // Reading an accepted immutable artifact has its own lifetime. A later
    // private probe may have cancelled the old compiler job without changing
    // the published source/PDF. Callers recheck artifact/source identity.
    let program = PathBuf::from(command.get_program());
    let before = Fingerprint::file(&program)?;
    let args = command
        .get_args()
        .map(|v| v.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let environment = command
        .get_envs()
        .filter_map(|(k, v)| {
            v.map(|v| {
                (
                    k.to_string_lossy().into_owned(),
                    v.to_string_lossy().into_owned(),
                )
            })
        })
        .collect();
    let directory = command
        .get_current_dir()
        .map(Path::to_path_buf)
        .unwrap_or(std::env::current_dir().map_err(fail)?);
    let output = process::run(
        &CommandSpec {
            program: &program,
            args: &args,
            directory: &directory,
            env: &environment,
            timeout: Duration::from_secs(10),
            memory_bytes: 2 * 1024 * 1024 * 1024,
        },
        cancel,
    )
    .map_err(fail)?;
    cancel.check()?;
    if Fingerprint::file(&program)? != before {
        return Err("SyncTeX changed during the query; compile again".into());
    }
    accepted_output(output)
}
fn accepted_output(output: process::Output) -> Result<String> {
    if output.cancelled {
        return Err("SyncTeX mapping query was cancelled".into());
    }
    if output.timed_out {
        return Err("SyncTeX mapping query exceeded its time limit".into());
    }
    if output.truncated {
        return Err("SyncTeX output was incomplete or truncated; mapping was refused".into());
    }
    if !output.success {
        let detail = output
            .text
            .lines()
            .take(4)
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(1000)
            .collect::<String>();
        return Err(format!("SyncTeX mapping query failed. {detail}"));
    }
    Ok(output.text)
}
fn field<'a>(output: &'a str, name: &str) -> Result<&'a str> {
    let mut values = output
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(key, _)| key.trim() == name)
        .map(|(_, value)| value.trim());
    let value = values
        .next()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| format!("SyncTeX mapping is missing {name}"))?;
    if values.next().is_some() {
        return Err("SyncTeX returned ambiguous mapping records. Select supported text in the PDF or edit its source.".into());
    }
    Ok(value)
}
pub(super) fn inverse(output: &str) -> Result<(&str, usize)> {
    let input = field(output, "Input")?;
    let line = field(output, "Line")?
        .parse::<usize>()
        .ok()
        .filter(|line| *line > 0)
        .ok_or("SyncTeX returned an invalid source line")?;
    Ok((input, line))
}
pub(super) fn forward(output: &str) -> Result<SourceRectangle> {
    let page = field(output, "Page")?
        .parse::<usize>()
        .ok()
        .filter(|page| *page > 0)
        .ok_or("SyncTeX returned an invalid page number")?;
    let number = |name: &str| -> Result<f64> {
        field(output, name)?
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite())
            .ok_or_else(|| format!("SyncTeX returned an invalid {name}"))
    };
    let x = number("h")?;
    let baseline = number("v")?;
    let width = number("W")?;
    let height = number("H")?;
    let y = baseline - height;
    if width <= 0.0 || height <= 0.0 || !y.is_finite() {
        return Err("SyncTeX returned an empty or invalid source rectangle".into());
    }
    Ok(SourceRectangle {
        page,
        x,
        y,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn forward_requires_real_complete_geometry_and_does_not_enlarge_or_clamp_it() {
        let mapped = forward("Page:2\nh:10\nv:5\nW:3.25\nH:8\nx:99\ny:99\n").unwrap();
        assert_eq!(
            (mapped.page, mapped.x, mapped.y, mapped.width, mapped.height),
            (2, 10.0, -3.0, 3.25, 8.0)
        );
        for text in [
            "Page:1",
            "Page:1\nx:2\ny:3\nW:4\nH:5",
            "Page:1\nh:2\nv:3\nW:4",
            "Page:1\nh:NaN\nv:3\nW:4\nH:5",
            "Page:1\nh:2\nv:3\nW:-1\nH:5",
            "Page:1\nh:2\nv:3\nW:4\nH:0",
            "Page:1.5\nh:2\nv:3\nW:4\nH:5",
            "Page:1\nh:2\nv:3\nW:4\nH:5\nPage:2",
            "Page:1\nh:2\nv:3\nW:4\nH:5\nh:2",
        ] {
            assert!(forward(text).is_err(), "{text}");
        }
    }
    #[test]
    fn inverse_refuses_missing_zero_or_ambiguous_records_instead_of_combining_them() {
        assert_eq!(
            inverse("Input:C:/project/main.tex\nLine:17\n").unwrap(),
            ("C:/project/main.tex", 17)
        );
        for text in [
            "Input:main.tex",
            "Input:main.tex\nLine:0",
            "Input:main.tex\nInput:chapter.tex\nLine:8",
            "Input:main.tex\nLine:2\nLine:3",
            "Input:\nLine:2",
        ] {
            assert!(inverse(text).is_err());
        }
    }
    #[test]
    fn nonzero_timeout_and_truncated_outputs_cannot_become_usable_mapping_hints() {
        let text = "Page:1\nh:2\nv:3\nW:4\nH:5";
        for (success, timed_out, truncated, cancelled) in [
            (false, false, false, false),
            (true, true, false, false),
            (true, false, true, false),
            (true, false, false, true),
        ] {
            let output = process::Output {
                success,
                timed_out,
                truncated,
                cancelled,
                text: text.into(),
            };
            assert!(accepted_output(output).is_err());
        }
    }

    #[cfg(windows)]
    #[test]
    fn actual_nonzero_and_truncated_processes_cannot_supply_partial_mapping_evidence() {
        let program = PathBuf::from(std::env::var_os("SystemRoot").unwrap())
            .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        for script in [
            "Write-Output 'Page:1';Write-Output 'h:2';Write-Output 'v:3';Write-Output 'W:4';Write-Output 'H:5';exit 7",
            "Write-Output ('x' * 180000);Write-Output 'Page:1';Write-Output 'h:2';Write-Output 'v:3';Write-Output 'W:4';Write-Output 'H:5';exit 0",
        ] {
            let mut command=Command::new(&program);command.args(["-NoProfile","-NonInteractive","-Command",script]);
            assert!(run(command,&Cancellation::default()).is_err());
        }
    }

    #[test]
    fn mapping_requires_a_sealed_current_trace_and_remains_readable_after_a_probe_supersedes_the_old_job(
    ) {
        let root = std::env::temp_dir().join(format!(
            "texloom-mapping-receipt-{}",
            Uuid::new_v4().simple()
        ));
        fs::create_dir_all(root.join("project")).unwrap();
        fs::write(
            root.join("project/main.tex"),
            "\\documentclass{article}\nText",
        )
        .unwrap();
        let backend = Backend::new(root.join("app"));
        let project = backend
            .open_project(Some(
                root.join("project/main.tex").to_string_lossy().into_owned(),
            ))
            .unwrap();
        let mut receipt = texloom_build::snapshot::Receipt::capture(
            &root.join("project"),
            &root.join("build"),
            "main.tex",
            &project.sources,
            &BTreeMap::new(),
            &std::env::current_exe().unwrap(),
        )
        .unwrap();
        fs::write(
            receipt.directory.join(".out/preview.pdf"),
            b"%PDF-receipt-only fixture",
        )
        .unwrap();
        receipt.seal_outputs().unwrap();
        let cancel = Cancellation::default();
        cancel.cancel();
        let mut build = Build {
            directory: receipt.directory.clone(),
            compiler: receipt.compiler.path.clone(),
            project_id: project.id.clone(),
            main: project.main.clone(),
            revision: 0,
            receipt,
            eligible: true,
            purpose: "preview".into(),
            cancel,
            sequence: 1,
            bytes: 1,
        };
        assert!(
            backend.check_mapping_snapshot(&build).is_err(),
            "An unsealed trace must never be queried"
        );
        let trace = build.directory.join(".out/preview.synctex.gz");
        fs::write(&trace, b"Receipt-only trace fixture").unwrap();
        build.receipt.seal_outputs().unwrap();
        assert!(
            backend.check_mapping_snapshot(&build).is_ok(),
            "The prior compiler lease is not the lifetime of an accepted artifact"
        );
        fs::write(&trace, b"TAMPERED").unwrap();
        assert!(backend.check_mapping_snapshot(&build).is_err());
        fs::write(&trace, b"Receipt-only trace fixture").unwrap();
        backend
            .document_edit(
                &project.id,
                0,
                "main.tex",
                vec![EditorChange {
                    offset: 0,
                    length: 0,
                    text: "% edit\n".into(),
                }],
                None,
            )
            .unwrap();
        assert!(
            backend.check_mapping_snapshot(&build).is_err(),
            "A changed canonical revision must invalidate mapping"
        );
        drop(backend);
        fs::remove_dir_all(root).unwrap();
    }
}
