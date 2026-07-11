use pixel_compare::{
    artifact, compare, validate_reference_identities, ReferenceIdentity, RgbaImage, Tolerance,
};
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
    max_channel_delta: u8,
    max_different_pixels: u64,
    gated: Option<String>,
    mode: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::parse()?;
    let manifest: Manifest = toml::from_str(&fs::read_to_string(&options.manifest)?)?;
    validate_reference_identity(&std::env::current_dir()?, &manifest.entry)?;
    fs::create_dir_all(&options.output_dir)?;
    let mut exact = 0usize;
    let mut diverges = 0usize;
    let mut gated = 0usize;

    for entry in &manifest.entry {
        if entry.status == "gated" {
            gated += 1;
            println!(
                "gated {}: {}",
                entry.id,
                entry.gated.as_deref().unwrap_or("no diagnostic")
            );
            continue;
        }
        let actual = options.output_dir.join(format!("{}.png", entry.id));
        let replay = Command::new(&options.replay)
            .args(["--stream", path_str(&entry.stream)?])
            .args(["--output", path_str(&actual)?])
            .args(["--backend", &options.backend])
            .args(["--frame", &entry.frame.to_string()])
            .args(["--mode", &entry.mode])
            .status()?;
        if !replay.success() {
            return Err(format!("renderer replay failed for {}", entry.id).into());
        }
        let expected_image = RgbaImage::read_png(&entry.reference)?;
        let actual_image = RgbaImage::read_png(&actual)?;
        let report = compare(
            &expected_image,
            &actual_image,
            Tolerance {
                max_channel_delta: entry.max_channel_delta,
                max_different_pixels: entry.max_different_pixels,
            },
        )?;
        if report.within_tolerance {
            exact += 1;
            println!(
                "exact {}: different-pixels={} max-channel-delta={}",
                entry.id, report.different_pixels, report.max_channel_delta
            );
        } else {
            diverges += 1;
            let artifact_path = options.output_dir.join(format!("{}-diff.png", entry.id));
            artifact(&expected_image, &actual_image)?.write_png(&artifact_path)?;
            println!(
                "diverges {}: different-pixels={} max-channel-delta={} artifact={}",
                entry.id,
                report.different_pixels,
                report.max_channel_delta,
                artifact_path.display()
            );
        }
    }

    println!(
        "renderer-corpus exact={exact} diverges={diverges} gated={gated} total={}",
        manifest.entry.len()
    );
    if options.expect_all_fail {
        if exact != 0 {
            return Err(format!("stub baseline unexpectedly passed {exact} entries").into());
        }
    } else if manifest
        .entry
        .iter()
        .filter(|entry| entry.status == "exact")
        .count()
        != exact
        || diverges != 0
    {
        return Err("renderer corpus ratchet failed".into());
    }
    Ok(())
}

fn validate_reference_identity(base: &Path, entries: &[Entry]) -> Result<(), String> {
    validate_reference_identities(
        base,
        entries.iter().map(|entry| ReferenceIdentity {
            id: &entry.id,
            stream: &entry.stream,
            frame: entry.frame,
            mode: &entry.mode,
            reference: &entry.reference,
        }),
    )
}

struct Options {
    manifest: PathBuf,
    replay: PathBuf,
    backend: String,
    output_dir: PathBuf,
    expect_all_fail: bool,
}

impl Options {
    fn parse() -> Result<Self, Box<dyn Error>> {
        let mut manifest = PathBuf::from("corpus-r.toml");
        let mut replay = PathBuf::from("target/debug/renderer-replay");
        let mut backend = "rust-wgpu".to_owned();
        let mut output_dir = PathBuf::from("target/renderer-corpus");
        let mut expect_all_fail = false;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--manifest" => manifest = PathBuf::from(args.next().ok_or(usage())?),
                "--replay" => replay = PathBuf::from(args.next().ok_or(usage())?),
                "--backend" => backend = args.next().ok_or(usage())?,
                "--output-dir" => output_dir = PathBuf::from(args.next().ok_or(usage())?),
                "--expect-all-fail" => expect_all_fail = true,
                _ => return Err(format!("unknown argument `{arg}`\n{}", usage()).into()),
            }
        }
        Ok(Self {
            manifest,
            replay,
            backend,
            output_dir,
            expect_all_fail,
        })
    }
}

fn path_str(path: &Path) -> Result<&str, Box<dyn Error>> {
    path.to_str()
        .ok_or_else(|| "path is not valid UTF-8".into())
}

fn usage() -> &'static str {
    "usage: corpus-r [--manifest FILE] [--replay FILE] [--backend stub|rust-wgpu|ffi-metal] [--output-dir DIR] [--expect-all-fail]"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, mode: &str, reference: &str) -> Entry {
        Entry {
            id: id.to_owned(),
            stream: PathBuf::from("fixture.rive-stream"),
            reference: PathBuf::from(reference),
            status: "gated".to_owned(),
            frame: 0,
            max_channel_delta: 2,
            max_different_pixels: 0,
            gated: Some("algorithm-core".to_owned()),
            mode: mode.to_owned(),
        }
    }

    #[test]
    fn rejects_reference_reuse_across_modes() {
        let entries = [
            entry("atomic", "clockwise-atomic", "shared.png"),
            entry("msaa", "msaa", "shared.png"),
        ];
        let error = validate_reference_identity(Path::new("/repo"), &entries).unwrap_err();
        assert!(error.contains("keyed by stream, frame, and mode"));
    }

    #[test]
    fn accepts_mode_specific_reference_paths() {
        let entries = [
            entry("atomic", "clockwise-atomic", "atomic.png"),
            entry("msaa", "msaa", "msaa.png"),
        ];
        validate_reference_identity(Path::new("/repo"), &entries).unwrap();
    }

    #[test]
    fn rejects_lexical_aliases_of_the_same_reference() {
        let entries = [
            entry("atomic", "clockwise-atomic", "alias/shared.png"),
            entry("msaa", "msaa", "alias/sub/../shared.png"),
        ];
        validate_reference_identity(Path::new("/repo"), &entries).unwrap_err();
    }

    #[test]
    fn rejects_absolute_and_relative_aliases() {
        let entries = [
            entry("one", "clockwise-atomic", "fixtures/shared.png"),
            entry("two", "msaa", "/repo/fixtures/shared.png"),
        ];
        validate_reference_identity(Path::new("/repo"), &entries).unwrap_err();
    }
}
