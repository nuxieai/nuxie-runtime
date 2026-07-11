use pixel_compare::{artifact, compare, RgbaImage, Tolerance};
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
