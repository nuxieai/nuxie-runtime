use perf_compare::renderer_perf_parity_gate::{
    check_threshold, evaluate_report_files, render_json, render_markdown,
};
use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(error) = run() {
        eprintln!("renderer-perf-parity-gate error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1))?;
    let report = evaluate_report_files(&options.reports, options.max_ratio)?;
    std::fs::write(&options.json, render_json(&report)?)
        .map_err(|error| format!("failed to write {}: {error}", options.json.display()))?;
    std::fs::write(&options.markdown, render_markdown(&report))
        .map_err(|error| format!("failed to write {}: {error}", options.markdown.display()))?;
    println!("renderer-perf-parity-gate json={}", options.json.display());
    println!(
        "renderer-perf-parity-gate markdown={}",
        options.markdown.display()
    );
    check_threshold(&report)?;
    println!(
        "renderer-perf-parity-gate threshold=pass max_ratio={:.6}",
        options.max_ratio
    );
    Ok(())
}

struct Options {
    reports: Vec<PathBuf>,
    max_ratio: f64,
    json: PathBuf,
    markdown: PathBuf,
}

impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut reports = Vec::new();
        let mut max_ratio = 1.0;
        let mut json = PathBuf::from("rive-renderer-perf-parity-gate.json");
        let mut markdown = PathBuf::from("rive-renderer-perf-parity-gate.md");
        let mut args = args.into_iter();
        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--report" => reports.push(PathBuf::from(next_value(&mut args, "--report")?)),
                "--max-ratio" => {
                    let value = next_value(&mut args, "--max-ratio")?;
                    max_ratio = value
                        .parse()
                        .map_err(|_| format!("--max-ratio must be a number, got {value}"))?;
                }
                "--json" => json = PathBuf::from(next_value(&mut args, "--json")?),
                "--markdown" => markdown = PathBuf::from(next_value(&mut args, "--markdown")?),
                "--help" | "-h" => return Err(usage().to_owned()),
                _ => return Err(format!("unknown argument {argument}\n{}", usage())),
            }
        }
        Ok(Self {
            reports,
            max_ratio,
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
    "usage: renderer-perf-parity-gate --report run1.json --report run2.json --report run3.json --report run4.json --report run5.json [--max-ratio N] [--json path] [--markdown path]"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_ratio_defaults_to_one_and_reports_are_repeatable() {
        let options = Options::parse([
            "--report".to_owned(),
            "one.json".to_owned(),
            "--report".to_owned(),
            "two.json".to_owned(),
        ])
        .expect("options must parse");

        assert_eq!(options.max_ratio, 1.0);
        assert_eq!(
            options.reports,
            [PathBuf::from("one.json"), PathBuf::from("two.json")]
        );
    }
}
