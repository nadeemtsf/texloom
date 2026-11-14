//! Experimental Windows inspection process. Not yet exposed through app IPC.
#[cfg(windows)]
#[path = "pdfium_worker/mod.rs"]
mod worker;
fn main() {
    #[cfg(windows)]
    if let Err(error) = worker::run() {
        eprintln!("PDFium worker refused: {error}");
        std::process::exit(1);
    }
    #[cfg(not(windows))]
    {
        eprintln!("This inspection prototype supports Windows x64 only");
        std::process::exit(1);
    }
}
