use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

const REPORT_SCHEMA: &str = "rive-renderer-perf-v1";
const RUNNER_PROTOCOL: &str = "rive-renderer-perf-runner-v1";
const MANIFEST_SCHEMA: &str = "rive-renderer-perf-scenes-v1";
const SAMPLES_PER_RUNNER: usize = 7;

fn main() {
    if let Err(error) = run() {
        eprintln!("r4-timing-compare error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = Options::parse(env::args().skip(1))?;
    let a_first = load_report("A first", &options.a_first)?;
    let b_first = load_report("B first", &options.b_first)?;
    let b_second = load_report("B second", &options.b_second)?;
    let a_second = load_report("A second", &options.a_second)?;
    validate_trace([&a_first, &b_first, &b_second, &a_second])?;

    let comparison = Comparison::new(&a_first, &b_first, &b_second, &a_second)?;
    let rendered = serde_json::to_string_pretty(&comparison)
        .map(|json| format!("{json}\n"))
        .map_err(|error| format!("failed to encode comparison: {error}"))?;
    if let Some(output) = options.output {
        std::fs::write(&output, rendered)
            .map_err(|error| format!("failed to write {}: {error}", output.display()))?;
    } else {
        print!("{rendered}");
    }

    check_limit(
        "post-tail B worst-scene renderer/C++ timing",
        comparison.b_worst_scene_ratio,
        options.max_renderer_ratio,
    )?;
    check_limit(
        "normalized B/A candidate timing",
        comparison.normalized_b_over_a,
        options.max_b_over_a,
    )?;
    check_limit(
        "C++ control drift",
        comparison.cpp_control_drift,
        options.max_control_drift,
    )?;
    check_limit(
        "normalized A repeat drift",
        comparison.normalized_a_repeat_drift,
        options.max_repeat_drift,
    )?;
    check_limit(
        "normalized B repeat drift",
        comparison.normalized_b_repeat_drift,
        options.max_repeat_drift,
    )?;
    Ok(())
}

fn check_limit(name: &str, actual: f64, maximum: f64) -> Result<(), String> {
    if !maximum.is_finite() || maximum <= 0.0 {
        return Err(format!(
            "limit for {name} must be finite and greater than zero"
        ));
    }
    if actual > maximum {
        return Err(format!("{name} failed: {actual:.6} exceeds {maximum:.6}"));
    }
    Ok(())
}

fn load_report(label: &str, path: &Path) -> Result<Report, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {label} report {}: {error}", path.display()))?;
    let report: Report = serde_json::from_str(&contents).map_err(|error| {
        format!(
            "invalid {REPORT_SCHEMA} JSON in {label} report {}: {error}",
            path.display()
        )
    })?;
    report.validate(label)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    schema: String,
    runner_protocol: String,
    release: bool,
    profile: String,
    debug: bool,
    samples_per_runner: usize,
    manifest_schema: String,
    scenes: Vec<Scene>,
    aggregate: Aggregate,
}

impl Report {
    fn validate(self, label: &str) -> Result<Self, String> {
        if self.schema != REPORT_SCHEMA
            || self.runner_protocol != RUNNER_PROTOCOL
            || !self.release
            || self.profile != "release"
            || self.debug
            || self.samples_per_runner != SAMPLES_PER_RUNNER
            || self.manifest_schema != MANIFEST_SCHEMA
        {
            return Err(format!(
                "{label} report does not declare the fenced renderer-perf-v1 protocol"
            ));
        }
        if self.scenes.len() != 16 {
            return Err(format!("{label} report must contain the 16 fenced scenes"));
        }

        let mut ids = std::collections::BTreeSet::new();
        let mut baseline_sum = 0_u64;
        let mut candidate_sum = 0_u64;
        let mut worst: Option<(&str, f64)> = None;
        for scene in &self.scenes {
            scene.validate(label)?;
            if !ids.insert(&scene.id) {
                return Err(format!("{label} report repeats scene {}", scene.id));
            }
            baseline_sum = baseline_sum
                .checked_add(scene.baseline.min_of_medians_ns)
                .ok_or_else(|| format!("{label} report baseline timing overflow"))?;
            candidate_sum = candidate_sum
                .checked_add(scene.candidate.min_of_medians_ns)
                .ok_or_else(|| format!("{label} report candidate timing overflow"))?;
            // `Iterator::max_by` used by renderer-perf keeps the final item on
            // ties, so mirror that rule when checking its reported aggregate.
            if worst.is_none_or(|(_, ratio)| scene.ratio >= ratio) {
                worst = Some((&scene.id, scene.ratio));
            }
        }
        let aggregate_ratio = ratio(candidate_sum, baseline_sum)?;
        let (worst_id, worst_ratio) = worst.expect("nonempty scenes checked above");
        if self.aggregate.baseline_min_of_medians_ns_sum != baseline_sum
            || self.aggregate.candidate_min_of_medians_ns_sum != candidate_sum
            || !same_number(self.aggregate.ratio, aggregate_ratio)
            || self.aggregate.worst_scene != worst_id
            || !same_number(self.aggregate.worst_ratio, worst_ratio)
        {
            return Err(format!(
                "{label} report aggregate is inconsistent with its scenes"
            ));
        }
        Ok(self)
    }
}

fn validate_trace(reports: [&Report; 4]) -> Result<(), String> {
    let reference = reports[0];
    for report in reports.into_iter().skip(1) {
        for (expected, actual) in reference.scenes.iter().zip(&report.scenes) {
            if expected.id != actual.id
                || expected.stream != actual.stream
                || expected.frame != actual.frame
                || expected.mode != actual.mode
                || expected.width != actual.width
                || expected.height != actual.height
                || expected.adapter_selection != actual.adapter_selection
                || !expected.selected_adapter.matches(&actual.selected_adapter)
                || !expected.timing.matches(&actual.timing)
            {
                return Err(
                    "ABBA reports do not describe the same fenced workload and adapter".to_owned(),
                );
            }
        }
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Scene {
    id: String,
    stream: String,
    frame: u32,
    mode: String,
    width: u32,
    height: u32,
    adapter_selection: String,
    selected_adapter: Adapter,
    timing: Timing,
    baseline: TimingSummary,
    candidate: TimingSummary,
    structural: Structural,
    ratio: f64,
}

impl Scene {
    fn validate(&self, label: &str) -> Result<(), String> {
        if self.id.trim().is_empty()
            || self.stream.trim().is_empty()
            || self.frame != 0
            || !matches!(self.mode.as_str(), "clockwise-atomic" | "msaa")
            || self.width != 1024
            || self.height != 1024
            || self.adapter_selection != "high-performance"
        {
            return Err(format!("{label} report has an invalid fenced scene"));
        }
        self.selected_adapter.validate(label)?;
        self.timing.validate(label)?;
        self.baseline.validate(label)?;
        self.candidate.validate(label)?;
        let expected = ratio(
            self.candidate.min_of_medians_ns,
            self.baseline.min_of_medians_ns,
        )?;
        if !same_number(self.ratio, expected) {
            return Err(format!(
                "{label} report scene {} has an inconsistent ratio",
                self.id
            ));
        }
        let _ = (
            self.structural.logical_flushes,
            self.structural.draws,
            self.structural.atomic_strategy_partitions,
        );
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Adapter {
    backend: String,
    name: String,
    vendor: String,
    device: String,
    driver: String,
}

impl Adapter {
    fn validate(&self, label: &str) -> Result<(), String> {
        if [
            &self.backend,
            &self.name,
            &self.vendor,
            &self.device,
            &self.driver,
        ]
        .iter()
        .any(|field| field.trim().is_empty())
        {
            return Err(format!("{label} report has an incomplete adapter identity"));
        }
        Ok(())
    }

    fn matches(&self, other: &Self) -> bool {
        self.backend == other.backend
            && self.name == other.name
            && self.vendor == other.vendor
            && self.device == other.device
            && self.driver == other.driver
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Timing {
    warmup_frames: u32,
    measured_frames: u32,
    scope: String,
    gpu_completion: String,
}

impl Timing {
    fn validate(&self, label: &str) -> Result<(), String> {
        if self.warmup_frames != 10
            || self.measured_frames != 100
            || self.scope != "submit-to-gpu-complete"
            || self.gpu_completion != "wait-for-completion-each-frame"
        {
            return Err(format!("{label} report has an unfenced timing method"));
        }
        Ok(())
    }

    fn matches(&self, other: &Self) -> bool {
        self.warmup_frames == other.warmup_frames
            && self.measured_frames == other.measured_frames
            && self.scope == other.scope
            && self.gpu_completion == other.gpu_completion
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TimingSummary {
    sample_medians_ns: Vec<u64>,
    min_of_medians_ns: u64,
    p50_ns: u64,
    p95_ns: u64,
    spread_ns: u64,
}

impl TimingSummary {
    fn validate(&self, label: &str) -> Result<(), String> {
        if self.sample_medians_ns.len() != SAMPLES_PER_RUNNER
            || self.sample_medians_ns.iter().any(|sample| *sample == 0)
        {
            return Err(format!("{label} report has invalid timing samples"));
        }
        let mut samples = self.sample_medians_ns.clone();
        samples.sort_unstable();
        let min = samples[0];
        let p50 = samples[(samples.len() - 1) / 2];
        let p95 = samples[((samples.len() * 95).div_ceil(100)).saturating_sub(1)];
        let spread = samples[samples.len() - 1] - min;
        if self.min_of_medians_ns != min
            || self.p50_ns != p50
            || self.p95_ns != p95
            || self.spread_ns != spread
        {
            return Err(format!("{label} report has inconsistent timing summary"));
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Structural {
    logical_flushes: u64,
    draws: u64,
    atomic_strategy_partitions: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Aggregate {
    baseline_min_of_medians_ns_sum: u64,
    candidate_min_of_medians_ns_sum: u64,
    ratio: f64,
    worst_scene: String,
    worst_ratio: f64,
}

#[derive(Serialize)]
struct LegValues {
    a_first: f64,
    b_first: f64,
    b_second: f64,
    a_second: f64,
}

#[derive(Serialize)]
struct Comparison {
    schema: &'static str,
    candidate_ns: LegValues,
    cpp_control_ns: LegValues,
    candidate_over_cpp: LegValues,
    normalized_a_average: f64,
    normalized_b_average: f64,
    normalized_b_over_a: f64,
    b_worst_scene_ratio: f64,
    cpp_control_drift: f64,
    normalized_a_repeat_drift: f64,
    normalized_b_repeat_drift: f64,
}

impl Comparison {
    fn new(
        a_first: &Report,
        b_first: &Report,
        b_second: &Report,
        a_second: &Report,
    ) -> Result<Self, String> {
        let candidate_ns = LegValues {
            a_first: a_first.aggregate.candidate_min_of_medians_ns_sum as f64,
            b_first: b_first.aggregate.candidate_min_of_medians_ns_sum as f64,
            b_second: b_second.aggregate.candidate_min_of_medians_ns_sum as f64,
            a_second: a_second.aggregate.candidate_min_of_medians_ns_sum as f64,
        };
        let cpp_control_ns = LegValues {
            a_first: a_first.aggregate.baseline_min_of_medians_ns_sum as f64,
            b_first: b_first.aggregate.baseline_min_of_medians_ns_sum as f64,
            b_second: b_second.aggregate.baseline_min_of_medians_ns_sum as f64,
            a_second: a_second.aggregate.baseline_min_of_medians_ns_sum as f64,
        };
        let candidate_over_cpp = LegValues {
            a_first: candidate_ns.a_first / cpp_control_ns.a_first,
            b_first: candidate_ns.b_first / cpp_control_ns.b_first,
            b_second: candidate_ns.b_second / cpp_control_ns.b_second,
            a_second: candidate_ns.a_second / cpp_control_ns.a_second,
        };
        let normalized_a_average =
            average_number(candidate_over_cpp.a_first, candidate_over_cpp.a_second);
        let normalized_b_average =
            average_number(candidate_over_cpp.b_first, candidate_over_cpp.b_second);
        let normalized_b_over_a = normalized_b_average / normalized_a_average;
        let b_worst_scene_ratio = b_first
            .aggregate
            .worst_ratio
            .max(b_second.aggregate.worst_ratio);
        let cpp_control_drift = spread([
            cpp_control_ns.a_first,
            cpp_control_ns.b_first,
            cpp_control_ns.b_second,
            cpp_control_ns.a_second,
        ])?;
        let normalized_a_repeat_drift =
            spread([candidate_over_cpp.a_first, candidate_over_cpp.a_second])?;
        let normalized_b_repeat_drift =
            spread([candidate_over_cpp.b_first, candidate_over_cpp.b_second])?;
        Ok(Self {
            schema: "rive-r4-timing-comparison-v2",
            candidate_ns,
            cpp_control_ns,
            candidate_over_cpp,
            normalized_a_average,
            normalized_b_average,
            normalized_b_over_a,
            b_worst_scene_ratio,
            cpp_control_drift,
            normalized_a_repeat_drift,
            normalized_b_repeat_drift,
        })
    }
}

fn average_number(left: f64, right: f64) -> f64 {
    (left + right) / 2.0
}

fn spread<const N: usize>(values: [f64; N]) -> Result<f64, String> {
    let mut minimum = f64::INFINITY;
    let mut maximum = f64::NEG_INFINITY;
    for value in values {
        if !value.is_finite() || value <= 0.0 {
            return Err("timing comparison values must be finite and greater than zero".to_owned());
        }
        minimum = minimum.min(value);
        maximum = maximum.max(value);
    }
    Ok(maximum / minimum)
}

fn ratio(numerator: u64, denominator: u64) -> Result<f64, String> {
    if denominator == 0 {
        return Err("cannot calculate a timing ratio with zero baseline".to_owned());
    }
    Ok(numerator as f64 / denominator as f64)
}

fn same_number(actual: f64, expected: f64) -> bool {
    actual.is_finite() && (actual - expected).abs() <= 1e-12
}

struct Options {
    a_first: PathBuf,
    b_first: PathBuf,
    b_second: PathBuf,
    a_second: PathBuf,
    max_b_over_a: f64,
    max_renderer_ratio: f64,
    max_control_drift: f64,
    max_repeat_drift: f64,
    output: Option<PathBuf>,
}

impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut a_first = None;
        let mut b_first = None;
        let mut b_second = None;
        let mut a_second = None;
        let mut max_b_over_a = None;
        let mut max_renderer_ratio = None;
        let mut max_control_drift = None;
        let mut max_repeat_drift = None;
        let mut output = None;
        let mut args = args.into_iter();
        while let Some(argument) = args.next() {
            let mut value = |flag| next_value(&mut args, flag);
            match argument.as_str() {
                "--a-first" => a_first = Some(PathBuf::from(value("--a-first")?)),
                "--b-first" => b_first = Some(PathBuf::from(value("--b-first")?)),
                "--b-second" => b_second = Some(PathBuf::from(value("--b-second")?)),
                "--a-second" => a_second = Some(PathBuf::from(value("--a-second")?)),
                "--max-b-over-a" => max_b_over_a = Some(parse_number(value("--max-b-over-a")?)?),
                "--max-renderer-ratio" => {
                    max_renderer_ratio = Some(parse_number(value("--max-renderer-ratio")?)?)
                }
                "--max-control-drift" => {
                    max_control_drift = Some(parse_number(value("--max-control-drift")?)?)
                }
                "--max-repeat-drift" => {
                    max_repeat_drift = Some(parse_number(value("--max-repeat-drift")?)?)
                }
                "--output" => output = Some(PathBuf::from(value("--output")?)),
                "--help" | "-h" => return Err(usage().to_owned()),
                _ => return Err(format!("unknown argument {argument}\n{}", usage())),
            }
        }
        Ok(Self {
            a_first: a_first.ok_or_else(|| "--a-first is required".to_owned())?,
            b_first: b_first.ok_or_else(|| "--b-first is required".to_owned())?,
            b_second: b_second.ok_or_else(|| "--b-second is required".to_owned())?,
            a_second: a_second.ok_or_else(|| "--a-second is required".to_owned())?,
            max_b_over_a: max_b_over_a.ok_or_else(|| "--max-b-over-a is required".to_owned())?,
            max_renderer_ratio: max_renderer_ratio
                .ok_or_else(|| "--max-renderer-ratio is required".to_owned())?,
            max_control_drift: max_control_drift
                .ok_or_else(|| "--max-control-drift is required".to_owned())?,
            max_repeat_drift: max_repeat_drift
                .ok_or_else(|| "--max-repeat-drift is required".to_owned())?,
            output,
        })
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn parse_number(value: String) -> Result<f64, String> {
    value
        .parse()
        .map_err(|_| format!("expected a number, got {value}"))
}

fn usage() -> &'static str {
    "usage: r4-timing-compare --a-first report --b-first report --b-second report --a-second report --max-renderer-ratio N --max-b-over-a N --max-control-drift N --max-repeat-drift N [--output path]"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(control_ns: u64, candidate_ns: u64) -> Report {
        Report {
            schema: REPORT_SCHEMA.to_owned(),
            runner_protocol: RUNNER_PROTOCOL.to_owned(),
            release: true,
            profile: "release".to_owned(),
            debug: false,
            samples_per_runner: SAMPLES_PER_RUNNER,
            manifest_schema: MANIFEST_SCHEMA.to_owned(),
            scenes: Vec::new(),
            aggregate: Aggregate {
                baseline_min_of_medians_ns_sum: control_ns,
                candidate_min_of_medians_ns_sum: candidate_ns,
                ratio: candidate_ns as f64 / control_ns as f64,
                worst_scene: String::new(),
                worst_ratio: 0.0,
            },
        }
    }

    #[test]
    fn comparison_normalizes_each_candidate_leg_against_cpp() {
        let a_first = report(100, 100);
        let b_first = report(80, 96);
        let b_second = report(120, 180);
        let a_second = report(200, 300);

        let comparison = Comparison::new(&a_first, &b_first, &b_second, &a_second).unwrap();

        assert!((comparison.candidate_over_cpp.a_first - 1.0).abs() < 1e-12);
        assert!((comparison.candidate_over_cpp.b_first - 1.2).abs() < 1e-12);
        assert!((comparison.candidate_over_cpp.b_second - 1.5).abs() < 1e-12);
        assert!((comparison.candidate_over_cpp.a_second - 1.5).abs() < 1e-12);
        assert!((comparison.normalized_a_average - 1.25).abs() < 1e-12);
        assert!((comparison.normalized_b_average - 1.35).abs() < 1e-12);
        assert!((comparison.normalized_b_over_a - 1.08).abs() < 1e-12);
        assert!((comparison.cpp_control_drift - 2.5).abs() < 1e-12);
    }

    #[test]
    fn comparison_does_not_average_away_control_instability() {
        let a_first = report(100, 100);
        let b_first = report(200, 100);
        let b_second = report(100, 100);
        let a_second = report(200, 100);

        let comparison = Comparison::new(&a_first, &b_first, &b_second, &a_second).unwrap();

        assert_eq!(comparison.cpp_control_drift, 2.0);
    }

    #[test]
    fn comparison_does_not_average_away_variant_instability() {
        let a_first = report(100, 100);
        let b_first = report(100, 200);
        let b_second = report(100, 100);
        let a_second = report(100, 200);

        let comparison = Comparison::new(&a_first, &b_first, &b_second, &a_second).unwrap();

        assert_eq!(comparison.normalized_b_over_a, 1.0);
        assert_eq!(comparison.normalized_a_repeat_drift, 2.0);
        assert_eq!(comparison.normalized_b_repeat_drift, 2.0);
    }
}
