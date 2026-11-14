use super::*;
use std::{
    fs::File,
    mem::{size_of, zeroed},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle},
    },
    ptr::{null, null_mut},
    thread,
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::*,
    Security::SECURITY_ATTRIBUTES,
    System::{JobObjects::*, Pipes::CreatePipe, Threading::*},
};

struct Handle(HANDLE);
impl Handle {
    fn checked(h: HANDLE) -> io::Result<Self> {
        if h.is_null() || h == INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(h))
        }
    }
}
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
struct Attributes {
    _storage: Vec<usize>,
    pointer: LPPROC_THREAD_ATTRIBUTE_LIST,
}
impl Attributes {
    fn new() -> io::Result<Self> {
        unsafe {
            let mut bytes = 0;
            InitializeProcThreadAttributeList(null_mut(), 2, 0, &mut bytes);
            if bytes == 0 {
                return Err(io::Error::last_os_error());
            }
            let mut storage = vec![0usize; bytes.div_ceil(size_of::<usize>())];
            let pointer = storage.as_mut_ptr().cast();
            check(InitializeProcThreadAttributeList(pointer, 2, 0, &mut bytes))?;
            Ok(Self {
                _storage: storage,
                pointer,
            })
        }
    }
    fn set<T>(&mut self, key: u32, values: &[T]) -> io::Result<()> {
        unsafe {
            check(UpdateProcThreadAttribute(
                self.pointer,
                0,
                key as usize,
                values.as_ptr().cast(),
                std::mem::size_of_val(values),
                null_mut(),
                null(),
            ))
        }
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        unsafe {
            DeleteProcThreadAttributeList(self.pointer);
        }
    }
}
fn check(result: i32) -> io::Result<()> {
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
fn wide(s: &std::ffi::OsStr) -> io::Result<Vec<u16>> {
    let mut text: Vec<_> = s.encode_wide().collect();
    if text.contains(&0) {
        return Err(io::Error::other("NUL in process argument"));
    }
    text.push(0);
    Ok(text)
}
// Windows argv quoting: double backslashes before quotes and before the closing quote.
fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    let mut slashes = 0;
    for c in s.chars() {
        if c == '\\' {
            slashes += 1;
        } else {
            out.push_str(&"\\".repeat(if c == '"' { slashes * 2 + 1 } else { slashes }));
            slashes = 0;
            out.push(c);
        }
    }
    out.push_str(&"\\".repeat(slashes * 2));
    out.push('"');
    out
}
pub(super) fn run(spec: &CommandSpec<'_>, cancel: &Cancellation) -> io::Result<Output> {
    unsafe {
        // Assignment occurs inside CreateProcess, before child code can spawn descendants.
        // No unsupervised process is launched if Job Objects are unavailable.
        let job = Handle::checked(CreateJobObjectW(null(), null()))?;
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = zeroed();
        limits.BasicLimitInformation.LimitFlags =
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_JOB_MEMORY;
        limits.JobMemoryLimit = spec.memory_bytes;
        check(SetInformationJobObject(
            job.0,
            JobObjectExtendedLimitInformation,
            (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ))?;
        let sa = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: null_mut(),
            bInheritHandle: 1,
        };
        let (mut read, mut write) = (null_mut(), null_mut());
        check(CreatePipe(&mut read, &mut write, &sa, 0))?;
        let read = Handle::checked(read)?;
        let write = Handle::checked(write)?;
        check(SetHandleInformation(read.0, HANDLE_FLAG_INHERIT, 0))?;
        let stdin = File::open("NUL")?;
        let input = stdin.as_raw_handle() as HANDLE;
        check(SetHandleInformation(
            input,
            HANDLE_FLAG_INHERIT,
            HANDLE_FLAG_INHERIT,
        ))?;
        let handles = [input, write.0];
        let jobs = [job.0];
        let mut attrs = Attributes::new()?;
        attrs.set(PROC_THREAD_ATTRIBUTE_HANDLE_LIST, &handles)?;
        attrs.set(PROC_THREAD_ATTRIBUTE_JOB_LIST, &jobs)?;
        let mut info: STARTUPINFOEXW = zeroed();
        info.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        info.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        info.StartupInfo.hStdInput = input;
        info.StartupInfo.hStdOutput = write.0;
        info.StartupInfo.hStdError = write.0;
        info.lpAttributeList = attrs.pointer;
        let program = spec.program.canonicalize()?;
        let program_w = wide(program.as_os_str())?;
        let args = std::iter::once(quote(&spec.program.to_string_lossy()))
            .chain(spec.args.iter().map(|x| quote(x)))
            .collect::<Vec<_>>()
            .join(" ");
        let mut command = wide(std::ffi::OsStr::new(&args))?;
        let directory = wide(spec.directory.as_os_str())?;
        let mut environment: BTreeMap<String, (std::ffi::OsString, std::ffi::OsString)> =
            std::env::vars_os()
                .map(|(k, v)| (k.to_string_lossy().to_uppercase(), (k, v)))
                .collect();
        for (k, v) in spec.env {
            environment.insert(k.to_uppercase(), (k.into(), v.into()));
        }
        let mut env = Vec::new();
        for (_, (k, v)) in environment {
            let entry = wide(&std::ffi::OsString::from(format!(
                "{}={}",
                k.to_string_lossy(),
                v.to_string_lossy()
            )))?;
            env.extend(entry);
        }
        env.push(0);
        let mut process: PROCESS_INFORMATION = zeroed();
        check(CreateProcessW(
            program_w.as_ptr(),
            command.as_mut_ptr(),
            null(),
            null(),
            1,
            CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT | EXTENDED_STARTUPINFO_PRESENT,
            env.as_ptr().cast(),
            directory.as_ptr(),
            &info.StartupInfo,
            &mut process,
        ))?;
        let thread_handle = process.hThread;
        let process = Handle::checked(process.hProcess)?;
        let _thread = Handle::checked(thread_handle)?;
        drop(write);
        drop(stdin);
        drop(attrs);
        let pipe = File::from_raw_handle(read.0);
        std::mem::forget(read);
        let reader = thread::spawn(move || Capture::read(pipe));
        let started = Instant::now();
        let mut timed_out = false;
        let mut cancelled = false;
        let mut wait_error = None;
        loop {
            let result = WaitForSingleObject(process.0, 25);
            if result == WAIT_OBJECT_0 {
                break;
            }
            if result == WAIT_FAILED {
                wait_error = Some(io::Error::last_os_error());
                break;
            }
            cancelled = cancel.cancelled();
            timed_out = started.elapsed() >= spec.timeout;
            if cancelled || timed_out {
                break;
            }
        }
        // Also removes orphaned descendants holding inherited stdout open after root exit.
        TerminateJobObject(job.0, 1);
        WaitForSingleObject(process.0, 5000);
        let mut code = 1;
        let got_code = GetExitCodeProcess(process.0, &mut code);
        drop(job);
        let capture = reader
            .join()
            .map_err(|_| io::Error::other("Compiler output reader failed"))?;
        if let Some(error) = wait_error {
            return Err(error);
        }
        check(got_code)?;
        let (text, truncated) = capture.finish();
        Ok(Output {
            success: code == 0 && !cancelled && !timed_out,
            cancelled,
            timed_out,
            truncated,
            text,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quotes_paths_quotes_and_trailing_slashes() {
        assert_eq!(quote("a b"), "\"a b\"");
        assert_eq!(quote("a\"b"), "\"a\\\"b\"");
        assert_eq!(quote("x\\"), "\"x\\\\\"");
        assert_eq!(quote(""), "\"\"");
    }
}
