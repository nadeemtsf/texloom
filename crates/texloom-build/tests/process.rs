use std::{
    collections::BTreeMap,
    path::Path,
    time::{Duration, Instant},
};
use texloom_build::{
    process::{self, CommandSpec},
    scheduler::Cancellation,
};
fn run(args: &[String], cancel: &Cancellation, timeout: Duration) -> process::Output {
    process::run(
        &CommandSpec {
            program: Path::new(env!("CARGO_BIN_EXE_process_fixture")),
            args,
            directory: &std::env::current_dir().unwrap(),
            env: &BTreeMap::new(),
            timeout,
            memory_bytes: 2 * 1024 * 1024 * 1024,
        },
        cancel,
    )
    .unwrap()
}
#[test]
fn native_argv_preserves_unicode_quotes_spaces_and_trailing_slashes() {
    for text in ["hello world", "a\"b", r"C:\a b\", "مرحبا 😀", ""] {
        let result = run(
            &["echo".into(), text.into()],
            &Cancellation::default(),
            Duration::from_secs(5),
        );
        assert!(result.success, "{}", result.text);
        assert_eq!(result.text, text);
    }
}
#[test]
fn large_child_output_is_fully_drained_without_deadlock() {
    let output = run(
        &["flood".into()],
        &Cancellation::default(),
        Duration::from_secs(5),
    );
    assert!(output.success);
    assert!(output.truncated);
    assert!(output.text.starts_with("FIRST"));
    assert!(output.text.ends_with("LAST"));
}
#[test]
fn cancellation_and_timeout_stop_descendants_and_close_their_pipes() {
    for timeout_case in [false, true] {
        let path = std::env::temp_dir().join(format!(
            "texloom-heartbeat-{}-{}-{}",
            std::process::id(),
            timeout_case,
            Instant::now().elapsed().as_nanos()
        ));
        let cancel = Cancellation::default();
        let worker_cancel = cancel.clone();
        let file = path.clone();
        let start = Instant::now();
        let worker = std::thread::spawn(move || {
            run(
                &["tree".into(), file.to_string_lossy().into()],
                &worker_cancel,
                if timeout_case {
                    Duration::from_secs(5)
                } else {
                    Duration::from_secs(10)
                },
            )
        });
        while !path.exists() && !worker.is_finished() && start.elapsed() < Duration::from_secs(8) {
            std::thread::sleep(Duration::from_millis(20));
        }
        if !path.exists() {
            cancel.cancel();
            let result = worker.join().unwrap();
            panic!("Descendant did not start (timeout case={timeout_case}, timed_out={}, cancelled={}): {}",result.timed_out,result.cancelled,result.text);
        }
        let cancel_started = Instant::now();
        if !timeout_case {
            cancel.cancel();
        }
        let result = worker.join().unwrap();
        if !timeout_case {
            assert!(
                cancel_started.elapsed() < Duration::from_secs(2),
                "Cancellation did not finish promptly"
            );
        }
        assert!(!result.success);
        assert_eq!(result.timed_out, timeout_case);
        assert_eq!(result.cancelled, !timeout_case);
        let size = std::fs::metadata(&path).unwrap().len();
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(
            std::fs::metadata(&path).unwrap().len(),
            size,
            "Descendant survived job cleanup"
        );
        std::fs::remove_file(path).unwrap();
        assert!(start.elapsed() < Duration::from_secs(12));
    }
}
