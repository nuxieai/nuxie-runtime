use perf_compare::renderer_counter::{
    check_counter_parity, render_json, render_markdown, run_counter_compare,
};
use perf_compare::renderer_perf::{ReportProvenance, SubprocessRunner, load_manifest};
use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("perf-counter-compare error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1))?;
    let baseline_source_id = options
        .baseline_source_id
        .or_else(|| env::var("RENDERER_COUNTER_BASELINE_SOURCE_ID").ok())
        .ok_or_else(|| {
            "--baseline-source-id is required (or set RENDERER_COUNTER_BASELINE_SOURCE_ID)"
                .to_owned()
        })?;
    let candidate_source_id = options
        .candidate_source_id
        .or_else(|| env::var("RENDERER_COUNTER_CANDIDATE_SOURCE_ID").ok())
        .ok_or_else(|| {
            "--candidate-source-id is required (or set RENDERER_COUNTER_CANDIDATE_SOURCE_ID)"
                .to_owned()
        })?;
    let working_directory = options
        .manifest
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let generator = env::current_exe()
        .map_err(|error| format!("failed to locate perf-counter-compare executable: {error}"))?;
    let provenance = ReportProvenance::from_files(
        &options.manifest,
        &options.baseline_runner,
        &options.candidate_runner,
        &generator,
        baseline_source_id.clone(),
        candidate_source_id.clone(),
    )?;
    let manifest = load_manifest(&options.manifest)?;
    let mut baseline =
        SubprocessRunner::new(options.baseline_runner.clone(), working_directory.clone());
    let mut candidate = SubprocessRunner::new(options.candidate_runner.clone(), working_directory);
    let report = run_counter_compare(&manifest, &mut baseline, &mut candidate, provenance)?;
    report.provenance.verify_files(
        &options.manifest,
        &options.baseline_runner,
        &options.candidate_runner,
        &generator,
        &baseline_source_id,
        &candidate_source_id,
    )?;

    std::fs::write(&options.json, render_json(&report)?)
        .map_err(|error| format!("failed to write {}: {error}", options.json.display()))?;
    std::fs::write(&options.markdown, render_markdown(&report))
        .map_err(|error| format!("failed to write {}: {error}", options.markdown.display()))?;

    println!(
        "perf-counter-compare scenes={} variants={} excesses={} timing=directional-only",
        manifest.scene.len(),
        report.scenes.len(),
        report.ranked_excesses.len(),
    );
    println!("perf-counter-compare json={}", options.json.display());
    println!(
        "perf-counter-compare markdown={}",
        options.markdown.display()
    );
    check_counter_parity(&report)?;
    println!("perf-counter-compare counter-parity=pass");
    Ok(())
}

struct Options {
    manifest: PathBuf,
    baseline_runner: PathBuf,
    candidate_runner: PathBuf,
    baseline_source_id: Option<String>,
    candidate_source_id: Option<String>,
    json: PathBuf,
    markdown: PathBuf,
}

impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("renderer-scenes.toml");
        let mut baseline_runner = None;
        let mut candidate_runner = None;
        let mut baseline_source_id = None;
        let mut candidate_source_id = None;
        let mut json = PathBuf::from("rive-renderer-work-counters.json");
        let mut markdown = PathBuf::from("rive-renderer-work-counters.md");
        let mut args = args.into_iter();
        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--manifest" => manifest = PathBuf::from(next_value(&mut args, "--manifest")?),
                "--baseline-runner" => {
                    baseline_runner =
                        Some(PathBuf::from(next_value(&mut args, "--baseline-runner")?))
                }
                "--candidate-runner" => {
                    candidate_runner =
                        Some(PathBuf::from(next_value(&mut args, "--candidate-runner")?))
                }
                "--baseline-source-id" => {
                    baseline_source_id = Some(next_value(&mut args, "--baseline-source-id")?)
                }
                "--candidate-source-id" => {
                    candidate_source_id = Some(next_value(&mut args, "--candidate-source-id")?)
                }
                "--json" => json = PathBuf::from(next_value(&mut args, "--json")?),
                "--markdown" => markdown = PathBuf::from(next_value(&mut args, "--markdown")?),
                "--help" | "-h" => return Err(usage().to_owned()),
                _ => return Err(format!("unknown argument {argument}\n{}", usage())),
            }
        }
        Ok(Self {
            manifest,
            baseline_runner: baseline_runner
                .ok_or_else(|| format!("--baseline-runner is required\n{}", usage()))?,
            candidate_runner: candidate_runner
                .ok_or_else(|| format!("--candidate-runner is required\n{}", usage()))?,
            baseline_source_id,
            candidate_source_id,
            json,
            markdown,
        })
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{flag} requires a value\n{}", usage()))
}

fn usage() -> &'static str {
    "usage: perf-counter-compare [--manifest path] --baseline-runner path --candidate-runner path --baseline-source-id id --candidate-source-id id [--json path] [--markdown path]"
}
