//! Development-only export/check. Does not open the app, projects or a renderer.
fn main() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    let types = root.join("web/generated/document-ipc.d.ts");
    let fixtures = root.join("tests/fixtures/document-ipc.json");
    let declarations = texloom_desktop_lib::protocol_bindings::declarations();
    let data = texloom_desktop_lib::protocol_bindings::fixtures();
    match std::env::args().nth(1).as_deref() {
        Some("--write") => {
            std::fs::create_dir_all(types.parent().unwrap()).unwrap();
            std::fs::write(types, declarations).unwrap();
            std::fs::write(
                fixtures,
                serde_json::to_string_pretty(&data).unwrap() + "\n",
            )
            .unwrap();
            println!("Generated document DTO types and Serde fixtures.");
        }
        Some("--check") => {
            assert_eq!(
                std::fs::read_to_string(types)
                    .unwrap()
                    .replace("\r\n", "\n"),
                declarations
            );
            let current: serde_json::Value =
                serde_json::from_slice(&std::fs::read(fixtures).unwrap()).unwrap();
            assert_eq!(current, data);
            println!("Document DTO types and Serde fixtures match Rust.");
        }
        _ => panic!("Pass --write or --check"),
    }
}
