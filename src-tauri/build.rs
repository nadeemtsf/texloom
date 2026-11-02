use sha2::{Digest, Sha256};
fn main() {
    let worker = "resources/verification/pdfium_worker.exe";
    println!("cargo:rerun-if-changed={worker}");
    // Embed the trusted build artifact's identity. The runtime never accepts
    // an executable path or expected hash from a webview request/sidecar.
    let hash = std::fs::read(worker)
        .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
        .unwrap_or_default();
    println!("cargo:rustc-env=TEXLOOM_WORKER_SHA256={hash}");
    tauri_build::build()
}
