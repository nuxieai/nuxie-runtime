use anyhow::{Context, Result};
use rive_binary::read_runtime_file;
use rive_graph::GraphFile;
use serde::Serialize;
use std::path::PathBuf;

fn main() -> Result<()> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .context("usage: graph-inspect <file.riv>")?;
    let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    let runtime =
        read_runtime_file(&bytes).with_context(|| format!("importing {}", path.display()))?;
    let graph = GraphFile::from_runtime_file(&runtime)
        .with_context(|| format!("building graph for {}", path.display()))?;

    let summary = Summary {
        path: path.display().to_string(),
        graph,
    };

    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

#[derive(Serialize)]
struct Summary {
    path: String,
    graph: GraphFile,
}
