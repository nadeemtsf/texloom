use super::*;
use std::{
    os::unix::process::CommandExt,
    process::{Command, Stdio},
    thread,
    time::Instant,
};
pub(super) fn run(spec: &CommandSpec<'_>, cancel: &Cancellation) -> io::Result<Output> {
    let mut command = Command::new(spec.program);
    command
        .args(spec.args)
        .current_dir(spec.directory)
        .envs(spec.env)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    let memory = spec.memory_bytes as libc::rlim_t;
    unsafe {
        command.pre_exec(move || {
            let limit = libc::rlimit {
                rlim_cur: memory,
                rlim_max: memory,
            };
            if libc::setrlimit(libc::RLIMIT_AS, &limit) != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command.spawn()?;
    let out = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("Missing compiler stdout"))?;
    let err = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("Missing compiler stderr"))?;
    let out = thread::spawn(move || Capture::read(out));
    let err = thread::spawn(move || Capture::read(err));
    let started = Instant::now();
    let mut timed_out = false;
    let mut cancelled = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status.success()),
            Ok(None) => {}
            Err(e) => break Err(e),
        }
        cancelled = cancel.cancelled();
        timed_out = started.elapsed() >= spec.timeout;
        if cancelled || timed_out {
            break Ok(false);
        }
        thread::sleep(Duration::from_millis(25));
    };
    unsafe {
        libc::kill(-(child.id() as i32), libc::SIGKILL);
    }
    let _ = child.wait();
    let (mut text, a) = out
        .join()
        .map_err(|_| io::Error::other("Output reader failed"))?
        .finish();
    let (error, b) = err
        .join()
        .map_err(|_| io::Error::other("Error reader failed"))?
        .finish();
    text.push_str(&error);
    Ok(Output {
        success: status? && !timed_out && !cancelled,
        cancelled,
        timed_out,
        truncated: a || b,
        text,
    })
}
