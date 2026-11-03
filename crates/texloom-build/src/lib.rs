//! Build and durable recovery infrastructure, independent of Tauri and the UI.
pub mod engine;
mod bibliography;
pub mod project_io;
pub mod inspection_worker;
pub mod persistence;
pub mod process;
pub mod recovery;
pub mod scheduler;
pub mod snapshot;
pub mod verification;
