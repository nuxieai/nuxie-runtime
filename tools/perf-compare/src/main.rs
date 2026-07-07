use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

fn main() {
    match run() {
        Ok(()) => {}
        Err(error) => {
            eprintln!("perf-compare error: {error}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1).collect())?;
    match &options.mode {
        Mode::Single(target) => run_single(&options, target),
        Mode::Corpus(corpus) => run_corpus(&options, corpus),
    }
}

fn run_single(options: &Options, target: &RunTarget) -> Result<(), String> {
    let cpp = measure_runner("cpp", &options.cpp_runner, target, options)?;
    let rust = measure_runner("rust", &options.rust_runner, target, options)?;
    let ratio = rust.median.as_secs_f64() / cpp.median.as_secs_f64();

    println!("perf-compare file={}", target.file.display());
    println!(
        "perf-compare samples={} iterations={} warmups={}",
        target.samples, options.iterations, options.warmups
    );
    print_measurements("cpp", cpp);
    print_measurements("rust", rust);
    println!("rust_over_cpp={ratio:.3}");
    check_max_ratio(ratio, options.max_ratio)
}

fn run_corpus(options: &Options, corpus: &Path) -> Result<(), String> {
    let entries = parse_corpus(corpus)?;
    let corpus_dir = corpus
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let mut targets = entries
        .into_iter()
        .filter(|entry| entry.status == Status::Exact)
        .map(|entry| RunTarget::from_corpus_entry(&entry, &options.rive_runtime_dir, &corpus_dir))
        .collect::<Vec<_>>();
    if let Some(limit) = options.corpus_limit {
        targets.truncate(limit);
    }
    if targets.is_empty() {
        return Err(format!(
            "corpus {} has no selected exact entries",
            corpus.display()
        ));
    }

    let mut cpp_sum = Duration::ZERO;
    let mut rust_sum = Duration::ZERO;
    let mut segments = 0usize;
    println!("perf-compare corpus={}", corpus.display());
    println!(
        "perf-compare entries={} iterations={} warmups={}",
        targets.len(),
        options.iterations,
        options.warmups
    );
    for target in &targets {
        let cpp = measure_runner("cpp", &options.cpp_runner, target, options)?;
        let rust = measure_runner("rust", &options.rust_runner, target, options)?;
        let ratio = rust.median.as_secs_f64() / cpp.median.as_secs_f64();
        cpp_sum += cpp.median;
        rust_sum += rust.median;
        segments += target.segment_count;
        println!(
            "entry id={} segments={} cpp_median_ms={:.3} rust_median_ms={:.3} rust_over_cpp={ratio:.3}",
            target.id,
            target.segment_count,
            millis(cpp.median),
            millis(rust.median)
        );
    }

    let aggregate_ratio = rust_sum.as_secs_f64() / cpp_sum.as_secs_f64();
    println!(
        "aggregate entries={} segments={} cpp_median_ms_sum={:.3} rust_median_ms_sum={:.3} rust_over_cpp={aggregate_ratio:.3}",
        targets.len(),
        segments,
        millis(cpp_sum),
        millis(rust_sum)
    );
    check_max_ratio(aggregate_ratio, options.max_ratio)
}

fn print_measurements(label: &str, measurements: Measurements) {
    println!(
        "{label} median_ms={:.3} min_ms={:.3} max_ms={:.3}",
        millis(measurements.median),
        millis(measurements.min),
        millis(measurements.max)
    );
}

fn check_max_ratio(ratio: f64, max_ratio: Option<f64>) -> Result<(), String> {
    let Some(max_ratio) = max_ratio else {
        return Ok(());
    };
    if ratio <= max_ratio {
        println!("perf-threshold ok rust_over_cpp={ratio:.3} max_ratio={max_ratio:.3}");
        Ok(())
    } else {
        Err(format!(
            "perf threshold failed: rust_over_cpp={ratio:.3} max_ratio={max_ratio:.3}"
        ))
    }
}

#[derive(Debug, Clone)]
struct Options {
    cpp_runner: PathBuf,
    rust_runner: PathBuf,
    rive_runtime_dir: PathBuf,
    mode: Mode,
    iterations: usize,
    warmups: usize,
    corpus_limit: Option<usize>,
    max_ratio: Option<f64>,
}

#[derive(Debug, Clone)]
enum Mode {
    Single(RunTarget),
    Corpus(PathBuf),
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut cpp_runner = env::var_os("GOLDEN_RUNNER")
            .map(PathBuf::from)
            .unwrap_or_else(default_cpp_runner);
        let mut rust_runner = env::var_os("RUST_GOLDEN_RUNNER")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target/debug/rust-golden-runner"));
        let mut rive_runtime_dir = env::var_os("RIVE_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
        let mut file = None;
        let mut corpus = None;
        let mut artboard = None;
        let mut state_machine = None;
        let mut input_script = None;
        let mut samples = "0".to_owned();
        let mut sample_count = 1usize;
        let mut iterations = 5usize;
        let mut warmups = 0usize;
        let mut corpus_limit = None;
        let mut max_ratio = None;

        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            let mut value = |option: &str| -> Result<String, String> {
                index += 1;
                args.get(index)
                    .cloned()
                    .ok_or_else(|| format!("{option} requires a value"))
            };

            match arg.as_str() {
                "--cpp-runner" => cpp_runner = PathBuf::from(value(arg)?),
                "--rust-runner" => rust_runner = PathBuf::from(value(arg)?),
                "--rive-runtime-dir" => rive_runtime_dir = PathBuf::from(value(arg)?),
                "--file" => file = Some(PathBuf::from(value(arg)?)),
                "--corpus" => corpus = Some(PathBuf::from(value(arg)?)),
                "--corpus-limit" => {
                    corpus_limit = Some(parse_positive_usize(&value(arg)?, "--corpus-limit")?)
                }
                "--max-ratio" => max_ratio = Some(parse_ratio(&value(arg)?)?),
                "--artboard" => artboard = Some(value(arg)?),
                "--state-machine" => state_machine = Some(value(arg)?),
                "--input-script" => input_script = Some(PathBuf::from(value(arg)?)),
                "--samples" => {
                    let parsed = parse_samples_csv(&value(arg)?)?;
                    sample_count = parsed.count;
                    samples = parsed.csv;
                }
                "--iterations" => iterations = parse_positive_usize(&value(arg)?, "--iterations")?,
                "--warmups" => {
                    warmups = value(arg)?
                        .parse::<usize>()
                        .map_err(|_| "--warmups must be a non-negative integer".to_owned())?;
                }
                "--help" | "-h" => {
                    println!(
                        "usage: perf-compare (--file <path> | --corpus corpus.toml) [--samples 0,0.5] [--iterations N] [--warmups N] [--max-ratio N] [--cpp-runner path] [--rust-runner path]"
                    );
                    std::process::exit(0);
                }
                other if !other.starts_with('-') && file.is_none() && corpus.is_none() => {
                    file = Some(PathBuf::from(other));
                }
                other => return Err(format!("unknown option: {other}")),
            }
            index += 1;
        }

        let mode = match (file, corpus) {
            (Some(file), None) => Mode::Single(RunTarget {
                id: "single".to_owned(),
                file,
                artboard,
                state_machine,
                input_script,
                samples,
                segment_count: sample_count,
            }),
            (None, Some(corpus)) => Mode::Corpus(corpus),
            (Some(_), Some(_)) => {
                return Err("choose either --file or --corpus, not both".to_owned());
            }
            (None, None) => return Err("missing --file <path> or --corpus <path>".to_owned()),
        };

        Ok(Self {
            cpp_runner,
            rust_runner,
            rive_runtime_dir,
            mode,
            iterations,
            warmups,
            corpus_limit,
            max_ratio,
        })
    }
}

#[derive(Debug, Clone)]
struct RunTarget {
    id: String,
    file: PathBuf,
    artboard: Option<String>,
    state_machine: Option<String>,
    input_script: Option<PathBuf>,
    samples: String,
    segment_count: usize,
}

impl RunTarget {
    fn from_corpus_entry(entry: &CorpusEntry, rive_runtime_dir: &Path, corpus_dir: &Path) -> Self {
        Self {
            id: entry.id.clone(),
            file: resolve_asset_path(&entry.path, rive_runtime_dir),
            artboard: entry.artboard.clone(),
            state_machine: entry.state_machine.clone(),
            input_script: entry
                .input_script
                .as_deref()
                .map(|path| resolve_script_path(path, corpus_dir)),
            samples: samples_csv(&entry.samples),
            segment_count: entry.samples.len(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Measurements {
    min: Duration,
    median: Duration,
    max: Duration,
}

fn measure_runner(
    label: &str,
    runner: &Path,
    target: &RunTarget,
    options: &Options,
) -> Result<Measurements, String> {
    for warmup in 0..options.warmups {
        run_once(label, runner, target, warmup + 1, true)?;
    }

    let mut measurements = Vec::with_capacity(options.iterations);
    for iteration in 0..options.iterations {
        measurements.push(run_once(label, runner, target, iteration + 1, false)?);
    }
    Ok(measurements_summary(measurements))
}

fn run_once(
    label: &str,
    runner: &Path,
    target: &RunTarget,
    iteration: usize,
    warmup: bool,
) -> Result<Duration, String> {
    let mut command = runner_command(runner, target);
    let start = Instant::now();
    let output = command
        .output()
        .map_err(|error| format!("failed to run {label} runner {}: {error}", runner.display()))?;
    let elapsed = start.elapsed();
    if !output.status.success() {
        return Err(format!(
            "{label} runner {} exited with {} for {}\n{}",
            runner.display(),
            output.status,
            target.id,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !output.stdout.starts_with(b"rive-golden-stream-v1\n") {
        let kind = if warmup { "warmup" } else { "iteration" };
        return Err(format!(
            "{label} runner {} did not emit a rive-golden stream for {} on {kind} {iteration}",
            runner.display(),
            target.id
        ));
    }
    Ok(elapsed)
}

fn runner_command(runner: &Path, target: &RunTarget) -> Command {
    let mut command = Command::new(runner);
    command.arg("--file").arg(&target.file);
    if let Some(artboard) = &target.artboard {
        command.arg("--artboard").arg(artboard);
    }
    if let Some(state_machine) = &target.state_machine {
        command.arg("--state-machine").arg(state_machine);
    }
    if let Some(input_script) = &target.input_script {
        command.arg("--input-script").arg(input_script);
    }
    command.arg("--samples").arg(&target.samples);
    command
}

fn measurements_summary(mut measurements: Vec<Duration>) -> Measurements {
    measurements.sort();
    let min = measurements[0];
    let max = measurements[measurements.len() - 1];
    let median = measurements[measurements.len() / 2];
    Measurements { min, median, max }
}

#[derive(Debug, Clone)]
struct ParsedSamples {
    csv: String,
    count: usize,
}

fn parse_samples_csv(value: &str) -> Result<ParsedSamples, String> {
    let samples = value
        .split(',')
        .map(|part| {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                return Err("empty sample in --samples".to_owned());
            }
            trimmed
                .parse::<f32>()
                .map_err(|_| format!("invalid sample {trimmed}"))?;
            Ok(trimmed.to_owned())
        })
        .collect::<Result<Vec<_>, String>>()?;
    if samples.is_empty() {
        return Err("--samples must include at least one sample".to_owned());
    }
    Ok(ParsedSamples {
        count: samples.len(),
        csv: samples.join(","),
    })
}

#[derive(Debug, Clone)]
struct CorpusEntry {
    id: String,
    path: String,
    artboard: Option<String>,
    state_machine: Option<String>,
    input_script: Option<String>,
    samples: Vec<f32>,
    status: Status,
}

impl CorpusEntry {
    fn new() -> Self {
        Self {
            id: String::new(),
            path: String::new(),
            artboard: None,
            state_machine: None,
            input_script: None,
            samples: vec![0.0],
            status: Status::NotYet,
        }
    }

    fn validate(&self, line: usize) -> Result<(), String> {
        if self.id.is_empty() {
            return Err(format!("entry before line {line} is missing id"));
        }
        if self.path.is_empty() {
            return Err(format!("entry {} is missing path", self.id));
        }
        if self.samples.is_empty() {
            return Err(format!(
                "entry {} must include at least one sample",
                self.id
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Exact,
    UnsupportedFeature,
    Diverges,
    NotYet,
}

impl Status {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "exact" => Ok(Self::Exact),
            "unsupported-feature" => Ok(Self::UnsupportedFeature),
            "diverges" => Ok(Self::Diverges),
            "not-yet" => Ok(Self::NotYet),
            other => Err(format!("unknown corpus status: {other}")),
        }
    }
}

fn parse_corpus(path: &Path) -> Result<Vec<CorpusEntry>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let mut entries = Vec::new();
    let mut current = None::<CorpusEntry>;

    for (index, raw_line) in text.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if line == "[[file]]" {
            if let Some(entry) = current.take() {
                entry.validate(line_number)?;
                entries.push(entry);
            }
            current = Some(CorpusEntry::new());
            continue;
        }

        let Some(entry) = current.as_mut() else {
            return Err(format!("line {line_number}: expected [[file]] before keys"));
        };
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("line {line_number}: expected key = value"))?;
        match key.trim() {
            "id" => entry.id = parse_string(value.trim(), line_number)?,
            "path" => entry.path = parse_string(value.trim(), line_number)?,
            "artboard" => entry.artboard = Some(parse_string(value.trim(), line_number)?),
            "state_machine" => entry.state_machine = Some(parse_string(value.trim(), line_number)?),
            "input_script" => entry.input_script = Some(parse_string(value.trim(), line_number)?),
            "samples" => entry.samples = parse_float_array(value.trim(), line_number)?,
            "status" => entry.status = Status::parse(&parse_string(value.trim(), line_number)?)?,
            _ => {}
        }
    }

    if let Some(entry) = current.take() {
        entry.validate(text.lines().count() + 1)?;
        entries.push(entry);
    }

    Ok(entries)
}

fn parse_string(value: &str, line: usize) -> Result<String, String> {
    let bytes = value.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'"' || bytes[bytes.len() - 1] != b'"' {
        return Err(format!("line {line}: expected quoted string"));
    }
    Ok(value[1..value.len() - 1].replace("\\\"", "\""))
}

fn parse_float_array(value: &str, line: usize) -> Result<Vec<f32>, String> {
    let Some(inner) = value
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    else {
        return Err(format!("line {line}: expected float array"));
    };
    let mut values = Vec::new();
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        values.push(
            part.parse::<f32>()
                .map_err(|error| format!("line {line}: invalid float {part}: {error}"))?,
        );
    }
    Ok(values)
}

fn resolve_asset_path(path: &str, rive_runtime_dir: &Path) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        rive_runtime_dir.join(path)
    }
}

fn resolve_script_path(path: &str, corpus_dir: &Path) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        corpus_dir.join(path)
    }
}

fn samples_csv(samples: &[f32]) -> String {
    samples
        .iter()
        .map(|sample| sample.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn parse_positive_usize(value: &str, option: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| format!("{option} must be a positive integer"))?;
    if parsed == 0 {
        return Err(format!("{option} must be greater than 0"));
    }
    Ok(parsed)
}

fn parse_ratio(value: &str) -> Result<f64, String> {
    let parsed = value
        .parse::<f64>()
        .map_err(|_| "--max-ratio must be a finite positive number".to_owned())?;
    if !parsed.is_finite() || parsed <= 0.0 {
        return Err("--max-ratio must be a finite positive number".to_owned());
    }
    Ok(parsed)
}

fn default_cpp_runner() -> PathBuf {
    let os = match env::consts::OS {
        "macos" => "macosx",
        "windows" => "windows",
        _ => "linux",
    };
    PathBuf::from(format!(
        "tools/golden-runner/build/{os}/bin/debug/rive_golden_runner"
    ))
}

fn millis(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_required_file_and_optional_values() {
        let options = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--samples".to_owned(),
            "0, 0.5".to_owned(),
            "--iterations".to_owned(),
            "3".to_owned(),
            "--warmups".to_owned(),
            "2".to_owned(),
            "--artboard".to_owned(),
            "Main".to_owned(),
        ])
        .expect("parse options");

        let Mode::Single(target) = options.mode else {
            panic!("expected single target");
        };
        assert_eq!(target.file, PathBuf::from("fixture.riv"));
        assert_eq!(target.samples, "0,0.5");
        assert_eq!(target.segment_count, 2);
        assert_eq!(options.iterations, 3);
        assert_eq!(options.warmups, 2);
        assert_eq!(target.artboard.as_deref(), Some("Main"));
    }

    #[test]
    fn parses_corpus_mode_and_threshold() {
        let options = Options::parse(vec![
            "--corpus".to_owned(),
            "corpus.toml".to_owned(),
            "--corpus-limit".to_owned(),
            "7".to_owned(),
            "--max-ratio".to_owned(),
            "1.5".to_owned(),
        ])
        .expect("parse options");

        assert!(matches!(options.mode, Mode::Corpus(_)));
        assert_eq!(options.corpus_limit, Some(7));
        assert_eq!(options.max_ratio, Some(1.5));
    }

    #[test]
    fn rejects_zero_iterations() {
        let error = Options::parse(vec![
            "--file".to_owned(),
            "fixture.riv".to_owned(),
            "--iterations".to_owned(),
            "0".to_owned(),
        ])
        .unwrap_err();
        assert!(error.contains("greater than 0"));
    }

    #[test]
    fn parses_exact_corpus_entries() {
        let path = env::temp_dir().join(format!("perf-compare-corpus-{}.toml", std::process::id()));
        fs::write(
            &path,
            r#"
[[file]]
id = "first"
path = "tests/unit_tests/assets/first.riv"
samples = [0.0, 0.25]
status = "exact"

[[file]]
id = "second"
path = "tests/unit_tests/assets/second.riv"
input_script = "tests/input_scripts/click.txt"
status = "unsupported-feature"
"#,
        )
        .expect("write corpus");

        let entries = parse_corpus(&path).expect("parse corpus");
        fs::remove_file(path).ok();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].id, "first");
        assert_eq!(entries[0].samples, vec![0.0, 0.25]);
        assert_eq!(entries[0].status, Status::Exact);
        assert_eq!(
            entries[1].input_script.as_deref(),
            Some("tests/input_scripts/click.txt")
        );
    }

    #[test]
    fn summarizes_measurements_by_sorted_median() {
        let summary = measurements_summary(vec![
            Duration::from_millis(30),
            Duration::from_millis(10),
            Duration::from_millis(20),
        ]);
        assert_eq!(summary.min, Duration::from_millis(10));
        assert_eq!(summary.median, Duration::from_millis(20));
        assert_eq!(summary.max, Duration::from_millis(30));
    }
}
