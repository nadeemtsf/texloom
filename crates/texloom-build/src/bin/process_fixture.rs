//! Development-only subprocess used by lifecycle tests; never packaged.
use std::{io::Write, process::Command, time::Duration};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--version") => {
            let mode = std::fs::read_to_string("compiler-fixture.sty").unwrap_or_default();
            if mode.trim() == "truncated" {
                let mut out = std::io::stdout().lock();
                out.write_all(b"MiKTeX fixture-first\n").unwrap();
                for _ in 0..128 {
                    out.write_all(&[b'x'; 8192]).unwrap();
                }
                out.write_all(b"\nfixture-last\n").unwrap();
            } else {
                std::fs::create_dir_all(".out").unwrap();
                std::fs::write(".out/preview.pdf", b"%PDF-untrusted-preflight-output").unwrap();
                println!("MiKTeX fixture-first\nAccess is denied.");
                for _ in 0..128 {
                    println!("Function: RegOpenKeyExW; Result: 5; synthetic initialization trace");
                }
                println!("fixture-last");
                std::process::exit(7);
            }
        }
        Some("echo") => print!("{}", args.get(2).unwrap()),
        Some("flood") => {
            let mut out = std::io::stdout().lock();
            out.write_all(b"FIRST").unwrap();
            for _ in 0..512 {
                out.write_all(&[b'x'; 8192]).unwrap();
            }
            out.write_all(b"LAST").unwrap();
        }
        Some("tree") => {
            let child = Command::new(std::env::current_exe().unwrap())
                .args(["heartbeat", &args[2]])
                .spawn()
                .unwrap();
            println!("child={}", child.id());
            std::io::stdout().flush().unwrap();
            loop {
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        Some("heartbeat") => loop {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&args[2])
                .unwrap();
            file.write_all(b".").unwrap();
            std::thread::sleep(Duration::from_millis(20));
        },
        _ => std::process::exit(7),
    }
}
