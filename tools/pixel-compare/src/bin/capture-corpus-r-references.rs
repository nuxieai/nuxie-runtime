use pixel_compare::{ReferenceIdentity, RgbaImage, validate_reference_identities};
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct Manifest {
    entry: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
struct Entry {
    id: String,
    stream: PathBuf,
    reference: PathBuf,
    status: String,
    #[serde(default)]
    frame: usize,
    mode: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::parse()?;
    let manifest: Manifest = toml::from_str(&fs::read_to_string(&options.manifest)?)?;
    validate_reference_identities(manifest.entry.iter().map(|entry| ReferenceIdentity {
        id: &entry.id,
        stream: &entry.stream,
        frame: entry.frame,
        mode: &entry.mode,
        reference: &entry.reference,
    }))?;
    require_supported_mode(&options.mode)?;
    let mut captured = 0usize;

    for entry in manifest.entry.iter().filter(|entry| {
        entry.status == options.status
            && entry.mode == options.mode
            && options
                .id
                .as_ref()
                .is_none_or(|selected| selected == &entry.id)
    }) {
        if let Some(parent) = entry.reference.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = temporary_path(&entry.reference, captured)?;
        let _ = fs::remove_file(&temporary);
        let replay = Command::new(&options.replay)
            .args(["--stream", path_str(&entry.stream)?])
            .args(["--output", path_str(&temporary)?])
            .args(["--backend", "ffi-metal"])
            .args(["--frame", &entry.frame.to_string()])
            .args(["--mode", &entry.mode])
            .status()?;
        if !replay.success() {
            let _ = fs::remove_file(&temporary);
            return Err(format!("C++ reference replay failed for {}", entry.id).into());
        }
        RgbaImage::read_png(&temporary).map_err(|error| {
            let _ = fs::remove_file(&temporary);
            format!(
                "C++ reference replay produced no valid PNG for {}: {error}",
                entry.id
            )
        })?;
        fs::rename(&temporary, &entry.reference)?;
        captured += 1;
        println!(
            "captured {}: frame={} mode={} reference={}",
            entry.id,
            entry.frame,
            entry.mode,
            entry.reference.display()
        );
    }

    if options.id.is_some() && captured == 0 {
        return Err("no matching manifest entry was selected".into());
    }
    println!("renderer-reference-capture captured={captured}");
    Ok(())
}

struct Options {
    manifest: PathBuf,
    replay: PathBuf,
    status: String,
    mode: String,
    id: Option<String>,
}

impl Options {
    fn parse() -> Result<Self, Box<dyn Error>> {
        let mut manifest = PathBuf::from("corpus-r.toml");
        let mut replay = PathBuf::from("target/debug/renderer-replay");
        let mut status = "exact".to_owned();
        let mut mode = "clockwise-atomic".to_owned();
        let mut id = None;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--manifest" => manifest = PathBuf::from(args.next().ok_or(usage())?),
                "--replay" => replay = PathBuf::from(args.next().ok_or(usage())?),
                "--status" => status = args.next().ok_or(usage())?,
                "--mode" => mode = args.next().ok_or(usage())?,
                "--id" => id = Some(args.next().ok_or(usage())?),
                _ => return Err(format!("unknown argument `{arg}`\n{}", usage()).into()),
            }
        }
        Ok(Self {
            manifest,
            replay,
            status,
            mode,
            id,
        })
    }
}

fn path_str(path: &Path) -> Result<&str, Box<dyn Error>> {
    path.to_str()
        .ok_or_else(|| "path is not valid UTF-8".into())
}

fn temporary_path(reference: &Path, index: usize) -> Result<PathBuf, Box<dyn Error>> {
    let name = reference
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("reference has no UTF-8 file name")?;
    Ok(reference.with_file_name(format!(".{name}.{}.{}.capture", std::process::id(), index)))
}

fn require_supported_mode(mode: &str) -> Result<(), Box<dyn Error>> {
    if mode == "clockwise-atomic" {
        Ok(())
    } else {
        Err("C++ Metal reference capture only supports clockwise-atomic mode".into())
    }
}

fn usage() -> &'static str {
    "usage: capture-corpus-r-references [--manifest FILE] [--replay FILE] [--status STATUS] [--mode MODE] [--id ID]"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_msaa_capture_before_launching_replay() {
        let error = require_supported_mode("msaa").unwrap_err();
        assert!(error.to_string().contains("only supports clockwise-atomic"));
    }
}
