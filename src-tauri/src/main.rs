#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    #[cfg(windows)]
    match std::env::args_os()
        .nth(1)
        .as_deref()
        .and_then(std::ffi::OsStr::to_str)
    {
        Some("--check-latex-for-setup") => {
            std::process::exit(texloom_desktop_lib::check_latex_for_setup(false));
        }
        Some("--prepare-latex-for-setup") => {
            std::process::exit(texloom_desktop_lib::check_latex_for_setup(true));
        }
        _ => {}
    }
    texloom_desktop_lib::run();
}
