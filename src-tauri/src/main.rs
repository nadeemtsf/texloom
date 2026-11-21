#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    #[cfg(windows)]
    if std::env::args_os().nth(1).as_deref()
        == Some(std::ffi::OsStr::new("--check-latex-for-setup"))
    {
        std::process::exit(texloom_desktop_lib::check_latex_for_setup());
    }
    texloom_desktop_lib::run();
}
