use anyhow::{Context, Result};
use rive_binary::read_runtime_file;
use serde::Serialize;
use std::path::PathBuf;

fn main() -> Result<()> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .context("usage: riv-inspect <file.riv>")?;
    let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    let file =
        read_runtime_file(&bytes).with_context(|| format!("importing {}", path.display()))?;

    let summary = Summary {
        path: path.display().to_string(),
        major_version: file.header.major_version,
        minor_version: file.header.minor_version,
        file_id: file.header.file_id,
        object_count: file.object_count(),
        known_object_count: file.known_object_count(),
        imported_object_count: file.imported_object_count(),
        objects: file.objects,
        import_statuses: file.import_statuses,
    };

    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

#[derive(Serialize)]
struct Summary {
    path: String,
    major_version: u64,
    minor_version: u64,
    file_id: u64,
    object_count: usize,
    known_object_count: usize,
    imported_object_count: usize,
    objects: Vec<Option<rive_binary::RuntimeObject>>,
    import_statuses: Vec<rive_binary::RuntimeImportStatus>,
}
