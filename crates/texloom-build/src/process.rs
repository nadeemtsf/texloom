//! Bounded output is retained while pipes continue draining to EOF.
use crate::scheduler::Cancellation;
use std::{
    collections::BTreeMap,
    io::{self, Read},
    path::Path,
    time::Duration,
};
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

pub struct Output {
    pub success: bool,
    pub cancelled: bool,
    pub timed_out: bool,
    pub truncated: bool,
    pub text: String,
}
pub struct CommandSpec<'a> {
    pub program: &'a Path,
    pub args: &'a [String],
    pub directory: &'a Path,
    pub env: &'a BTreeMap<String, String>,
    pub timeout: Duration,
    pub memory_bytes: usize,
}
pub fn run(spec: &CommandSpec<'_>, cancel: &Cancellation) -> io::Result<Output> {
    cancel.check().map_err(io::Error::other)?;
    #[cfg(windows)]
    {
        windows::run(spec, cancel)
    }
    #[cfg(unix)]
    {
        unix::run(spec, cancel)
    }
}
struct Capture {
    first: Vec<u8>,
    last: std::collections::VecDeque<u8>,
    total: usize,
    failed: bool,
}
impl Capture {
    const HALF: usize = 64 * 1024;
    fn read(mut stream: impl Read) -> Self {
        let mut c = Self {
            first: Vec::new(),
            last: Default::default(),
            total: 0,
            failed: false,
        };
        let mut buffer = [0; 8192];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    c.total = c.total.saturating_add(n);
                    for b in &buffer[..n] {
                        if c.first.len() < Self::HALF {
                            c.first.push(*b);
                        } else {
                            if c.last.len() == Self::HALF {
                                c.last.pop_front();
                            }
                            c.last.push_back(*b);
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => {
                    c.failed = true;
                    break;
                }
            }
        }
        c
    }
    fn finish(self) -> (String, bool) {
        let truncated = self.failed || self.total > Self::HALF * 2;
        let mut bytes = self.first;
        if truncated {
            bytes.extend_from_slice(b"\n[compiler output truncated or interrupted]\n");
        }
        bytes.extend(self.last);
        (String::from_utf8_lossy(&bytes).into_owned(), truncated)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_failures_never_masquerade_as_complete_logs() {
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("read failed"))
            }
        }
        assert!(Capture::read(Broken).finish().1);
    }
    #[test]
    fn oversized_output_is_drained_and_retains_both_ends() {
        let mut bytes = vec![b'x'; 1_000_000];
        bytes[..5].copy_from_slice(b"FIRST");
        let n = bytes.len();
        bytes[n - 4..].copy_from_slice(b"LAST");
        let (text, truncated) = Capture::read(bytes.as_slice()).finish();
        assert!(truncated);
        assert!(text.starts_with("FIRST"));
        assert!(text.ends_with("LAST"));
        assert!(text.len() < 132_000);
    }
}
