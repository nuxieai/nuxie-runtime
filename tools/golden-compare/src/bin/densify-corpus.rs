use anyhow::{Context, Result, bail};
use nuxie_binary::{RuntimeFile, RuntimeLinearAnimation, RuntimeObject, read_runtime_file};
use std::collections::{BTreeSet, VecDeque};
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("densify-corpus error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let options = Options::parse(env::args().skip(1).collect())?;
    let corpus_text = std::fs::read_to_string(&options.corpus)
        .with_context(|| format!("failed to read {}", options.corpus.display()))?;
    let corpus_dir = options
        .corpus
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut output = String::with_capacity(corpus_text.len() + 4096);
    let mut densified = 0usize;
    let mut chunks = corpus_text.split("[[file]]");
    output.push_str(chunks.next().unwrap_or_default());
    for body in chunks {
        let chunk = format!("[[file]]{body}");

        let entry = Entry::parse(&chunk)?;
        let replacement = proposed_samples(&entry, &options.rive_runtime_dir, &corpus_dir)?;
        if let Some(samples) = replacement {
            let old = &entry.samples_line;
            let new = format!(
                "samples = [0.0, {}, {}]",
                format_time(samples.0),
                format_time(samples.1)
            );
            output.push_str(&chunk.replacen(old, &new, 1));
            println!(
                "{}: midpoint={} boundary={}",
                entry.id,
                format_time(samples.0),
                format_time(samples.1)
            );
            densified += 1;
        } else {
            output.push_str(&chunk);
        }
    }

    println!("densify-corpus summary: densified={densified}");
    if options.write {
        std::fs::write(&options.corpus, output)
            .with_context(|| format!("failed to write {}", options.corpus.display()))?;
    }
    Ok(())
}

#[derive(Debug)]
struct Options {
    corpus: PathBuf,
    rive_runtime_dir: PathBuf,
    write: bool,
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self> {
        let mut corpus = PathBuf::from("corpus.toml");
        let mut rive_runtime_dir = env::var_os("RIVE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
        let mut write = false;
        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            let mut value = |option: &str| -> Result<String> {
                index += 1;
                args.get(index)
                    .cloned()
                    .with_context(|| format!("{option} requires a value"))
            };
            match arg.as_str() {
                "--corpus" => corpus = PathBuf::from(value(arg)?),
                "--rive-runtime-dir" => rive_runtime_dir = PathBuf::from(value(arg)?),
                "--write" => write = true,
                "--help" | "-h" => {
                    println!(
                        "usage: densify-corpus [--corpus corpus.toml] [--rive-runtime-dir DIR] [--write]"
                    );
                    std::process::exit(0);
                }
                other => bail!("unknown option: {other}"),
            }
            index += 1;
        }
        Ok(Self {
            corpus,
            rive_runtime_dir,
            write,
        })
    }
}

#[derive(Debug)]
struct Entry {
    id: String,
    path: String,
    artboard: Option<String>,
    samples: Vec<f32>,
    samples_line: String,
    samples_t0_only: bool,
    has_linear_animation: bool,
}

impl Entry {
    fn parse(chunk: &str) -> Result<Self> {
        let mut id = None;
        let mut path = None;
        let mut artboard = None;
        let mut samples = Vec::new();
        let mut samples_line = None;
        let mut has_linear_animation = false;
        for raw_line in chunk.lines() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "id" => id = Some(parse_string(value.trim())?),
                "path" => path = Some(parse_string(value.trim())?),
                "artboard" => artboard = Some(parse_string(value.trim())?),
                "samples" => {
                    samples = parse_samples(value.trim())?;
                    samples_line = Some(line.to_owned());
                }
                "features" => has_linear_animation = value.contains("type-key:31:LinearAnimation"),
                _ => {}
            }
        }
        Ok(Self {
            id: id.context("corpus entry is missing id")?,
            path: path.context("corpus entry is missing path")?,
            artboard,
            samples_t0_only: samples.as_slice() == [0.0],
            samples,
            samples_line: samples_line.context("corpus entry is missing samples")?,
            has_linear_animation,
        })
    }
}

fn proposed_samples(
    entry: &Entry,
    rive_runtime_dir: &Path,
    corpus_dir: &Path,
) -> Result<Option<(f32, f32)>> {
    if !entry.has_linear_animation {
        return Ok(None);
    }
    if !entry.samples_t0_only {
        let repairs_missing_zero = entry.samples.len() == 2
            && entry.samples.iter().all(|sample| *sample > 0.0)
            && (entry.samples[1] - entry.samples[0] * 2.0).abs() < 0.000_002;
        return Ok(repairs_missing_zero.then(|| (entry.samples[0], entry.samples[1])));
    }
    let path = resolve_asset_path(&entry.path, rive_runtime_dir, corpus_dir);
    let bytes = std::fs::read(&path)
        .with_context(|| format!("failed to read {} for {}", path.display(), entry.id))?;
    let runtime = read_runtime_file(&bytes)
        .with_context(|| format!("failed to import {} for {}", path.display(), entry.id))?;
    let artboard_index = selected_artboard_index(&runtime, entry.artboard.as_deref())
        .with_context(|| format!("selected artboard was not found for {}", entry.id))?;

    let mut animations = runtime.artboard_linear_animations(artboard_index);
    // A selected artboard can be animated solely by a nested simple animation
    // whose timeline belongs to a library artboard. Follow only authored
    // NestedArtboard references from the selected scene; unrelated artboards
    // in the same file must not choose its sample boundary.
    if animations.is_empty() {
        animations = nested_artboard_linear_animations(&runtime, artboard_index);
    }

    let mut looping = Vec::new();
    let mut finite = Vec::new();
    for animation in animations {
        let Some(boundary) = animation_boundary_seconds(animation.object) else {
            continue;
        };
        finite.push(boundary);
        if animation.object.uint_property("loopValue").unwrap_or(0) != 0 {
            looping.push(boundary);
        }
    }
    let candidates = if looping.is_empty() {
        &finite
    } else {
        &looping
    };
    let Some(boundary) = candidates.iter().copied().reduce(f32::min) else {
        return Ok(None);
    };
    Ok(Some((boundary / 2.0, boundary)))
}

fn nested_artboard_linear_animations(
    runtime: &RuntimeFile,
    root_artboard_index: usize,
) -> Vec<RuntimeLinearAnimation<'_>> {
    let mut animations = Vec::new();
    let mut visited = BTreeSet::from([root_artboard_index]);
    let mut pending = VecDeque::from([root_artboard_index]);

    while let Some(artboard_index) = pending.pop_front() {
        let Some(slots) = runtime.artboard_local_object_slots(artboard_index) else {
            continue;
        };
        for nested_index in slots.into_iter().flatten().filter_map(|object| {
            (object.type_name == "NestedArtboard")
                .then(|| object.uint_property("artboardId"))
                .flatten()
                .and_then(|index| usize::try_from(index).ok())
        }) {
            if !visited.insert(nested_index) {
                continue;
            }
            animations.extend(runtime.artboard_linear_animations(nested_index));
            pending.push_back(nested_index);
        }
    }
    animations
}

fn animation_boundary_seconds(animation: &RuntimeObject) -> Option<f32> {
    let fps = animation.uint_property("fps")? as f32;
    let speed = animation.double_property("speed")?.abs();
    if !fps.is_finite() || fps <= 0.0 || !speed.is_finite() || speed <= 0.0 {
        return None;
    }
    let duration_frames = if animation.bool_property("enableWorkArea").unwrap_or(false) {
        let start = animation.uint_property("workStart")? as f32;
        let end = animation.uint_property("workEnd")? as f32;
        end - start
    } else {
        animation.uint_property("duration")? as f32
    };
    let boundary = duration_frames / fps / speed;
    (boundary.is_finite() && boundary > 0.0).then_some(boundary)
}

fn selected_artboard_index(runtime: &RuntimeFile, artboard: Option<&str>) -> Option<usize> {
    match artboard {
        Some(name) => runtime
            .artboards()
            .iter()
            .position(|candidate| candidate.string_property("name").unwrap_or_default() == name),
        None => runtime.default_artboard().map(|_| 0),
    }
}

fn resolve_asset_path(path: &str, rive_runtime_dir: &Path, corpus_dir: &Path) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else if path.starts_with("fixtures/") {
        corpus_dir.join(path)
    } else {
        rive_runtime_dir.join(path)
    }
}

fn parse_string(value: &str) -> Result<String> {
    let Some(value) = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    else {
        bail!("expected quoted string, found {value}");
    };
    Ok(value.replace("\\\"", "\"").replace("\\\\", "\\"))
}

fn parse_samples(value: &str) -> Result<Vec<f32>> {
    let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        bail!("expected sample array, found {value}");
    };
    inner
        .split(',')
        .map(|sample| {
            sample
                .trim()
                .parse::<f32>()
                .with_context(|| format!("invalid sample {sample}"))
        })
        .collect()
}

fn format_time(value: f32) -> String {
    let mut value = format!("{value:.6}");
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.push('0');
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_t0_animated_entry_without_reformatting_toml() {
        let entry = Entry::parse(
            r#"[[file]]
id = "animated"
path = "tests/unit_tests/assets/animated.riv"
artboard = "main"
samples = [0.0]
status = "exact"
features = ["type-key:31:LinearAnimation"]
"#,
        )
        .unwrap();
        assert_eq!(entry.id, "animated");
        assert_eq!(entry.artboard.as_deref(), Some("main"));
        assert!(entry.samples_t0_only);
        assert!(entry.has_linear_animation);
    }

    #[test]
    fn sample_format_is_stable_and_keeps_a_decimal() {
        assert_eq!(format_time(0.5), "0.5");
        assert_eq!(format_time(1.0), "1.0");
        assert_eq!(format_time(1.0 / 3.0), "0.333333");
    }
}
