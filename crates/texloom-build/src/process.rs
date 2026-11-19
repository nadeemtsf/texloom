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
    pub timeout: Option<Duration>,
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
                    let first = n.min(Self::HALF - c.first.len());
                    c.first.extend_from_slice(&buffer[..first]);
                    let tail = &buffer[first..n];
                    // Each read fits in the tail buffer, so overflow comes from older bytes.
                    let excess = (c.last.len() + tail.len()).saturating_sub(Self::HALF);
                    c.last.drain(..excess);
                    c.last.extend(tail);
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
    fn output_retains_the_exact_prefix_and_tail_at_the_capture_limit() {
        let limit = Capture::HALF * 2;
        for length in [limit - 1, limit, limit + 1, 1_000_000] {
            let bytes: Vec<_> = (0..length).map(|i| b'a' + (i % 26) as u8).collect();
            let (text, truncated) = Capture::read(bytes.as_slice()).finish();
            assert_eq!(truncated, length > limit);
            if truncated {
                let expected = [
                    &bytes[..Capture::HALF],
                    b"\n[compiler output truncated or interrupted]\n",
                    &bytes[length - Capture::HALF..],
                ].concat();
                assert_eq!(text.as_bytes(), expected);
            } else {
                assert_eq!(text.as_bytes(), bytes);
            }
        }
    }
}
