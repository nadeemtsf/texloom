//! Development-only subprocess used by lifecycle tests; never packaged.
use std::{io::Write,process::Command,time::Duration};
fn main() {
    let args:Vec<_>=std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("echo")=>print!("{}",args.get(2).unwrap()),
        Some("flood")=>{let mut out=std::io::stdout().lock();out.write_all(b"FIRST").unwrap();
            for _ in 0..512 {out.write_all(&[b'x';8192]).unwrap();}out.write_all(b"LAST").unwrap();},
        Some("tree")=>{
            let child=Command::new(std::env::current_exe().unwrap()).args(["heartbeat",&args[2]]).spawn().unwrap();
            println!("child={}",child.id());std::io::stdout().flush().unwrap();
            loop {std::thread::sleep(Duration::from_millis(20));}
        },
        Some("heartbeat")=>loop {
            let mut file=std::fs::OpenOptions::new().create(true).append(true).open(&args[2]).unwrap();
            file.write_all(b".").unwrap();std::thread::sleep(Duration::from_millis(20));
        },
        _=>std::process::exit(7),
    }
}
