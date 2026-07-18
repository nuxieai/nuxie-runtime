use perf_compare::renderer_perf::{
    ReportProvenance, SubprocessRunner, check_threshold, load_manifest, render_json,
    render_markdown, run_benchmark,
};
use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("renderer-perf error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1))?;
    let manifest = load_manifest(&options.manifest)?;
    if options.validate_only {
        println!(
            "renderer-perf manifest={} scenes={} variants={} status=valid",
            options.manifest.display(),
            manifest.scene.len(),
            manifest.scene.len() * manifest.modes.len(),
        );
        return Ok(());
    }

    let baseline_runner = options
        .baseline_runner
        .ok_or_else(|| "--baseline-runner is required unless --validate-only is set".to_owned())?;
    let candidate_runner = options
        .candidate_runner
        .ok_or_else(|| "--candidate-runner is required unless --validate-only is set".to_owned())?;
    let baseline_source_id = options
        .baseline_source_id
        .or_else(|| env::var("RENDERER_PERF_BASELINE_SOURCE_ID").ok())
        .ok_or_else(|| {
            "--baseline-source-id is required when running benchmarks (or set RENDERER_PERF_BASELINE_SOURCE_ID for fixed harnesses)".to_owned()
        })?;
    let candidate_source_id = options
        .candidate_source_id
        .or_else(|| env::var("RENDERER_PERF_CANDIDATE_SOURCE_ID").ok())
        .ok_or_else(|| {
            "--candidate-source-id is required when running benchmarks (or set RENDERER_PERF_CANDIDATE_SOURCE_ID for fixed harnesses)".to_owned()
        })?;
    let max_ratio = options
        .max_ratio
        .ok_or_else(|| "--max-ratio is required when running benchmarks".to_owned())?;
    let working_directory = options
        .manifest
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let generator = env::current_exe()
        .map_err(|error| format!("failed to locate renderer-perf executable: {error}"))?;
    let provenance = ReportProvenance::from_files(
        &options.manifest,
        &baseline_runner,
        &candidate_runner,
        &generator,
        baseline_source_id.clone(),
        candidate_source_id.clone(),
    )?;
    let mut baseline = SubprocessRunner::new(baseline_runner.clone(), working_directory.clone());
    let mut candidate = SubprocessRunner::new(candidate_runner.clone(), working_directory);
    let report = run_benchmark(&manifest, &mut baseline, &mut candidate, provenance)?;
    report.provenance.verify_files(
        &options.manifest,
        &baseline_runner,
        &candidate_runner,
        &generator,
        &baseline_source_id,
        &candidate_source_id,
    )?;

    std::fs::write(&options.json, render_json(&report)?)
        .map_err(|error| format!("failed to write {}: {error}", options.json.display()))?;
    std::fs::write(&options.markdown, render_markdown(&report))
        .map_err(|error| format!("failed to write {}: {error}", options.markdown.display()))?;
    println!("renderer-perf json={}", options.json.display());
    println!("renderer-perf markdown={}", options.markdown.display());

    check_threshold(&report, max_ratio)?;
    println!("renderer-perf threshold=pass max_ratio={max_ratio:.6}");
    Ok(())
}

struct Options {
    manifest: PathBuf,
    baseline_runner: Option<PathBuf>,
    candidate_runner: Option<PathBuf>,
    json: PathBuf,
    markdown: PathBuf,
    max_ratio: Option<f64>,
    baseline_source_id: Option<String>,
    candidate_source_id: Option<String>,
    validate_only: bool,
}

impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut options = Self {
            manifest: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("renderer-scenes.toml"),
            baseline_runner: None,
            candidate_runner: None,
            json: PathBuf::from("rive-renderer-perf.json"),
            markdown: PathBuf::from("rive-renderer-perf.md"),
            max_ratio: None,
            baseline_source_id: None,
            candidate_source_id: None,
            validate_only: false,
        };
        let mut args = args.into_iter();
        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--manifest" => {
                    options.manifest = PathBuf::from(next_value(&mut args, "--manifest")?)
                }
                "--baseline-runner" => {
                    options.baseline_runner =
                        Some(PathBuf::from(next_value(&mut args, "--baseline-runner")?))
                }
                "--candidate-runner" => {
                    options.candidate_runner =
                        Some(PathBuf::from(next_value(&mut args, "--candidate-runner")?))
                }
                "--json" => options.json = PathBuf::from(next_value(&mut args, "--json")?),
                "--markdown" => {
                    options.markdown = PathBuf::from(next_value(&mut args, "--markdown")?)
                }
                "--max-ratio" => {
                    let value = next_value(&mut args, "--max-ratio")?;
                    options.max_ratio = Some(
                        value
                            .parse()
                            .map_err(|_| format!("--max-ratio must be a number, got {value}"))?,
                    );
                }
                "--candidate-source-id" => {
                    options.candidate_source_id =
                        Some(next_value(&mut args, "--candidate-source-id")?)
                }
                "--baseline-source-id" => {
                    options.baseline_source_id =
                        Some(next_value(&mut args, "--baseline-source-id")?)
                }
                "--validate-only" => options.validate_only = true,
                "--help" | "-h" => return Err(usage().to_owned()),
                _ => return Err(format!("unknown argument {argument}\n{}", usage())),
            }
        }
        Ok(options)
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{flag} requires a value\n{}", usage()))
}

fn usage() -> &'static str {
    "usage: renderer-perf [--manifest path] --validate-only\n       renderer-perf [--manifest path] --baseline-runner path --candidate-runner path --baseline-source-id id --candidate-source-id id --max-ratio N [--json path] [--markdown path]"
}
