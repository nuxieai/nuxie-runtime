use nuxie_render_stream::RenderStream;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct Corpus {
    file: Vec<RivEntry>,
}

#[derive(Debug, Deserialize)]
struct RivEntry {
    id: String,
    path: PathBuf,
    #[serde(default)]
    artboard: Option<String>,
    #[serde(default)]
    state_machine: Option<String>,
    #[serde(default)]
    input_script: Option<PathBuf>,
    #[serde(default = "default_samples")]
    samples: Vec<f32>,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    status: String,
}

fn default_samples() -> Vec<f32> {
    vec![0.0]
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::parse()?;
    let corpus: Corpus = toml::from_str(&fs::read_to_string(&options.corpus)?)?;
    let corpus_dir = options.corpus.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(&options.stream_dir)?;
    fs::create_dir_all(&options.reference_dir)?;
    let mut failures = Vec::new();
    let mut gates = Vec::new();
    let mut frame_count = 0usize;

    for (index, entry) in corpus.file.iter().enumerate() {
        let runner = if entry
            .features
            .iter()
            .any(|feature| feature == "scripted-runner-only")
        {
            &options.scripted_runner
        } else {
            &options.runner
        };
        let asset = if entry.path.is_absolute() {
            entry.path.clone()
        } else {
            options.rive_runtime_dir.join(&entry.path)
        };
        let mut command = Command::new(runner);
        command.args(["--file", path_str(&asset)?]);
        if let Some(artboard) = &entry.artboard {
            command.args(["--artboard", artboard]);
        }
        if let Some(state_machine) = &entry.state_machine {
            command.args(["--state-machine", state_machine]);
        }
        command.args([
            "--samples",
            &entry
                .samples
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        ]);
        if let Some(input_script) = &entry.input_script {
            command.args(["--input-script", path_str(&corpus_dir.join(input_script))?]);
        }
        let output = command.output()?;
        if !output.status.success() {
            if entry.status == "unsupported-feature"
                && entry
                    .features
                    .iter()
                    .any(|feature| feature.starts_with("import-error:"))
            {
                gates.push(format!(
                    "{}\t{}",
                    entry.id,
                    entry
                        .features
                        .iter()
                        .find(|feature| feature.starts_with("import-error:"))
                        .expect("checked above")
                ));
                continue;
            }
            failures.push(format!(
                "{}: runner failed: {}",
                entry.id,
                String::from_utf8_lossy(&output.stderr)
            ));
            continue;
        }
        let stdout = String::from_utf8(output.stdout)?;
        let Some(start) = stdout.find("rive-golden-stream-v1\n") else {
            failures.push(format!("{}: runner emitted no stream", entry.id));
            continue;
        };
        let stream_text = &stdout[start..];
        let stream = RenderStream::parse(stream_text)?;
        let stream_path = options.stream_dir.join(format!("{}.rive-stream", entry.id));
        fs::write(&stream_path, stream_text)?;

        for frame in 0..stream.frames.len() {
            // The upstream Metal backend explicitly leaves MSAA flush unimplemented.
            for mode in ["clockwise-atomic"] {
                let reference = options
                    .reference_dir
                    .join(format!("{}-frame-{frame}-{mode}.png", entry.id));
                let replay = Command::new(&options.replay)
                    .args(["--stream", path_str(&stream_path)?])
                    .args(["--output", path_str(&reference)?])
                    .args(["--backend", "ffi-metal"])
                    .args(["--frame", &frame.to_string()])
                    .args(["--mode", mode])
                    .status()?;
                if !replay.success() {
                    failures.push(format!(
                        "{} frame {frame} mode {mode}: reference replay failed",
                        entry.id
                    ));
                    break;
                }
                frame_count += 1;
            }
        }
        if (index + 1) % 10 == 0 {
            println!(
                "riv-capture progress={}/{} frames={frame_count}",
                index + 1,
                corpus.file.len()
            );
        }
    }

    println!(
        "riv-capture files={} frames={} gates={} failures={}",
        corpus.file.len(),
        frame_count,
        gates.len(),
        failures.len()
    );
    fs::write(options.stream_dir.join("gated.txt"), gates.join("\n"))?;
    if !failures.is_empty() {
        for failure in failures {
            eprintln!("{failure}");
        }
        return Err("one or more .riv entries failed capture".into());
    }
    Ok(())
}

struct Options {
    corpus: PathBuf,
    rive_runtime_dir: PathBuf,
    runner: PathBuf,
    scripted_runner: PathBuf,
    replay: PathBuf,
    stream_dir: PathBuf,
    reference_dir: PathBuf,
}

impl Options {
    fn parse() -> Result<Self, Box<dyn Error>> {
        let mut corpus = PathBuf::from("corpus.toml");
        let mut rive_runtime_dir = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
        let mut runner =
            PathBuf::from("tools/golden-runner/build/macosx/bin/release/rive_golden_runner");
        let mut scripted_runner = PathBuf::from(
            "tools/golden-runner/build/macosx/bin/release/rive_golden_runner_scripted",
        );
        let mut replay = PathBuf::from("target/debug/renderer-replay");
        let mut stream_dir = PathBuf::from("fixtures/renderer/streams/riv");
        let mut reference_dir = PathBuf::from("fixtures/renderer/reference/metal/riv");
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            let mut value = || args.next().ok_or("missing option value");
            match arg.as_str() {
                "--corpus" => corpus = PathBuf::from(value()?),
                "--rive-runtime-dir" => rive_runtime_dir = PathBuf::from(value()?),
                "--runner" => runner = PathBuf::from(value()?),
                "--scripted-runner" => scripted_runner = PathBuf::from(value()?),
                "--replay" => replay = PathBuf::from(value()?),
                "--stream-dir" => stream_dir = PathBuf::from(value()?),
                "--reference-dir" => reference_dir = PathBuf::from(value()?),
                _ => return Err(format!("unknown argument `{arg}`").into()),
            }
        }
        Ok(Self {
            corpus,
            rive_runtime_dir,
            runner,
            scripted_runner,
            replay,
            stream_dir,
            reference_dir,
        })
    }
}

fn path_str(path: &Path) -> Result<&str, Box<dyn Error>> {
    path.to_str()
        .ok_or_else(|| "path is not valid UTF-8".into())
}
