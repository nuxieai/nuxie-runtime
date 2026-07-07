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
    let cpp = measure_runner("cpp", &options.cpp_runner, &options)?;
    let rust = measure_runner("rust", &options.rust_runner, &options)?;
    let ratio = rust.median.as_secs_f64() / cpp.median.as_secs_f64();

    println!("perf-compare file={}", options.file.display());
    println!(
        "perf-compare samples={} iterations={} warmups={}",
        options.samples, options.iterations, options.warmups
    );
    println!(
        "cpp median_ms={:.3} min_ms={:.3} max_ms={:.3}",
        millis(cpp.median),
        millis(cpp.min),
        millis(cpp.max)
    );
    println!(
        "rust median_ms={:.3} min_ms={:.3} max_ms={:.3}",
        millis(rust.median),
        millis(rust.min),
        millis(rust.max)
    );
    println!("rust_over_cpp={ratio:.3}");
    Ok(())
}

#[derive(Debug, Clone)]
struct Options {
    cpp_runner: PathBuf,
    rust_runner: PathBuf,
    file: PathBuf,
    artboard: Option<String>,
    state_machine: Option<String>,
    input_script: Option<PathBuf>,
    samples: String,
    iterations: usize,
    warmups: usize,
}

impl Options {
    fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut cpp_runner = env::var_os("GOLDEN_RUNNER")
            .map(PathBuf::from)
            .unwrap_or_else(default_cpp_runner);
        let mut rust_runner = env::var_os("RUST_GOLDEN_RUNNER")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target/debug/rust-golden-runner"));
        let mut file = None;
        let mut artboard = None;
        let mut state_machine = None;
        let mut input_script = None;
        let mut samples = "0".to_owned();
        let mut iterations = 5usize;
        let mut warmups = 0usize;

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
                "--file" => file = Some(PathBuf::from(value(arg)?)),
                "--artboard" => artboard = Some(value(arg)?),
                "--state-machine" => state_machine = Some(value(arg)?),
                "--input-script" => input_script = Some(PathBuf::from(value(arg)?)),
                "--samples" => samples = parse_samples_csv(&value(arg)?)?,
                "--iterations" => {
                    iterations = value(arg)?
                        .parse::<usize>()
                        .map_err(|_| "--iterations must be a positive integer".to_owned())?;
                    if iterations == 0 {
                        return Err("--iterations must be greater than 0".to_owned());
                    }
                }
                "--warmups" => {
                    warmups = value(arg)?
                        .parse::<usize>()
                        .map_err(|_| "--warmups must be a non-negative integer".to_owned())?;
                }
                "--help" | "-h" => {
                    println!(
                        "usage: perf-compare --file <path> [--samples 0,0.5] [--iterations N] [--warmups N] [--cpp-runner path] [--rust-runner path]"
                    );
                    std::process::exit(0);
                }
                other if !other.starts_with('-') && file.is_none() => {
                    file = Some(PathBuf::from(other));
                }
                other => return Err(format!("unknown option: {other}")),
            }
            index += 1;
        }

        Ok(Self {
            cpp_runner,
            rust_runner,
            file: file.ok_or_else(|| "missing --file <path>".to_owned())?,
            artboard,
            state_machine,
            input_script,
            samples,
            iterations,
            warmups,
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct Measurements {
    min: Duration,
    median: Duration,
    max: Duration,
}

fn measure_runner(label: &str, runner: &Path, options: &Options) -> Result<Measurements, String> {
    for warmup in 0..options.warmups {
        run_once(label, runner, options, warmup + 1, true)?;
    }

    let mut measurements = Vec::with_capacity(options.iterations);
    for iteration in 0..options.iterations {
        measurements.push(run_once(label, runner, options, iteration + 1, false)?);
    }
    Ok(measurements_summary(measurements))
}

fn run_once(
    label: &str,
    runner: &Path,
    options: &Options,
    iteration: usize,
    warmup: bool,
) -> Result<Duration, String> {
    let mut command = runner_command(runner, options);
    let start = Instant::now();
    let output = command
        .output()
        .map_err(|error| format!("failed to run {label} runner {}: {error}", runner.display()))?;
    let elapsed = start.elapsed();
    if !output.status.success() {
        return Err(format!(
            "{label} runner {} exited with {}\n{}",
            runner.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if !output.stdout.starts_with(b"rive-golden-stream-v1\n") {
        let kind = if warmup { "warmup" } else { "iteration" };
        return Err(format!(
            "{label} runner {} did not emit a rive-golden stream on {kind} {iteration}",
            runner.display()
        ));
    }
    Ok(elapsed)
}

fn runner_command(runner: &Path, options: &Options) -> Command {
    let mut command = Command::new(runner);
    command.arg("--file").arg(&options.file);
    if let Some(artboard) = &options.artboard {
        command.arg("--artboard").arg(artboard);
    }
    if let Some(state_machine) = &options.state_machine {
        command.arg("--state-machine").arg(state_machine);
    }
    if let Some(input_script) = &options.input_script {
        command.arg("--input-script").arg(input_script);
    }
    command.arg("--samples").arg(&options.samples);
    command
}

fn measurements_summary(mut measurements: Vec<Duration>) -> Measurements {
    measurements.sort();
    let min = measurements[0];
    let max = measurements[measurements.len() - 1];
    let median = measurements[measurements.len() / 2];
    Measurements { min, median, max }
}

fn parse_samples_csv(value: &str) -> Result<String, String> {
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
    Ok(samples.join(","))
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

        assert_eq!(options.file, PathBuf::from("fixture.riv"));
        assert_eq!(options.samples, "0,0.5");
        assert_eq!(options.iterations, 3);
        assert_eq!(options.warmups, 2);
        assert_eq!(options.artboard.as_deref(), Some("Main"));
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
