use crate::renderer_perf::{
    ESTIMATOR, MANIFEST_SCHEMA, Mode, PAIR_ORDER, REPORT_SCHEMA, RUNNER_PROTOCOL, Report,
    ReportProvenance, SAMPLE_COUNT, SampleOrder, SceneReport, TimingMethod,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::PathBuf;

pub const PARITY_GATE_SCHEMA: &str = "rive-renderer-perf-parity-gate-v1";
pub const PARITY_GATE_ESTIMATOR: &str = "equal-order-ratio-of-median-sums-v1";
pub const REQUIRED_REPORT_COUNT: usize = 5;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ModeValues<T> {
    pub overall: T,
    pub clockwise_atomic: T,
    pub msaa: T,
}

#[derive(Clone, Debug, Serialize)]
pub struct GateRun {
    pub report: String,
    pub report_sha256: String,
    pub ratios: ModeValues<f64>,
    pub strict_selected_diagnostic: ModeValues<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ParityGateReport {
    pub schema: &'static str,
    pub estimator: &'static str,
    pub max_ratio: f64,
    pub run_count: usize,
    pub provenance: ReportProvenance,
    pub runs: Vec<GateRun>,
    pub medians: ModeValues<f64>,
    pub pass_counts: ModeValues<usize>,
    pub strict_selected_diagnostic_medians: ModeValues<f64>,
    pub passed: bool,
}

pub fn evaluate_report_files(
    paths: &[PathBuf],
    max_ratio: f64,
) -> Result<ParityGateReport, String> {
    if paths.len() != REQUIRED_REPORT_COUNT {
        return Err(format!(
            "renderer parity gate requires exactly {REQUIRED_REPORT_COUNT} independent reports, got {}",
            paths.len()
        ));
    }
    if !max_ratio.is_finite() || max_ratio <= 0.0 {
        return Err("--max-ratio must be a finite value greater than zero".to_owned());
    }

    let mut canonical_paths = HashSet::new();
    let mut semantic_hashes = HashSet::new();
    let mut reports = Vec::with_capacity(REQUIRED_REPORT_COUNT);
    for path in paths {
        let canonical = std::fs::canonicalize(path)
            .map_err(|error| format!("failed to resolve report {}: {error}", path.display()))?;
        if !canonical_paths.insert(canonical) {
            return Err(format!("duplicate report path alias: {}", path.display()));
        }
        let contents = std::fs::read(path)
            .map_err(|error| format!("failed to read report {}: {error}", path.display()))?;
        let content_hash = format!("{:x}", Sha256::digest(&contents));
        let report = serde_json::from_slice::<Report>(&contents)
            .map_err(|error| format!("invalid renderer-perf report {}: {error}", path.display()))?;
        let canonical = serde_json::to_vec(&report).map_err(|error| {
            format!(
                "failed to canonicalize renderer-perf report {}: {error}",
                path.display()
            )
        })?;
        let semantic_hash = format!("{:x}", Sha256::digest(canonical));
        if !semantic_hashes.insert(semantic_hash) {
            return Err(format!(
                "duplicate report contents do not count as independent runs: {}",
                path.display()
            ));
        }
        reports.push(LoadedReport {
            path: path.display().to_string(),
            sha256: content_hash,
            report,
        });
    }
    validate_report_collection(&reports)?;
    let provenance = reports[0].report.provenance.clone();
    let runs = reports
        .iter()
        .map(|loaded| {
            Ok(GateRun {
                report: loaded.path.clone(),
                report_sha256: loaded.sha256.clone(),
                ratios: mode_values(&loaded.report, equal_order_ratio)?,
                strict_selected_diagnostic: mode_values(&loaded.report, strict_selected_ratio)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let medians = median_run_values(&runs, |run| run.ratios);
    let strict_selected_diagnostic_medians =
        median_run_values(&runs, |run| run.strict_selected_diagnostic);
    let pass_counts = ModeValues {
        overall: runs
            .iter()
            .filter(|run| run.ratios.overall <= max_ratio)
            .count(),
        clockwise_atomic: runs
            .iter()
            .filter(|run| run.ratios.clockwise_atomic <= max_ratio)
            .count(),
        msaa: runs
            .iter()
            .filter(|run| run.ratios.msaa <= max_ratio)
            .count(),
    };
    let passed = medians.overall <= max_ratio
        && medians.clockwise_atomic <= max_ratio
        && medians.msaa <= max_ratio;
    Ok(ParityGateReport {
        schema: PARITY_GATE_SCHEMA,
        estimator: PARITY_GATE_ESTIMATOR,
        max_ratio,
        run_count: runs.len(),
        provenance,
        runs,
        medians,
        pass_counts,
        strict_selected_diagnostic_medians,
        passed,
    })
}

pub fn render_json(report: &ParityGateReport) -> Result<String, String> {
    serde_json::to_string_pretty(report)
        .map(|json| format!("{json}\n"))
        .map_err(|error| format!("failed to render parity gate JSON: {error}"))
}

pub fn render_markdown(report: &ParityGateReport) -> String {
    let provenance = &report.provenance;
    let mut markdown = format!(
        "# Rive Renderer Performance Parity Gate\n\nSchema: `{}`  \nEstimator: `{}`  \nMaximum ratio: `{:.6}`  \nVerdict: **{}**\n\n## Provenance\n\n- Manifest SHA-256: `{}`\n- Baseline runner SHA-256: `{}`\n- Candidate runner SHA-256: `{}`\n- Generator SHA-256: `{}`\n- Baseline source identity: `{}`\n- Candidate source identity: `{}`\n\n| run | source report | report SHA-256 | overall | clockwise-atomic | MSAA | strict overall | strict clockwise-atomic | strict MSAA |\n| ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |\n",
        report.schema,
        report.estimator,
        report.max_ratio,
        if report.passed { "PASS" } else { "FAIL" },
        provenance.manifest_sha256,
        provenance.baseline_runner_sha256,
        provenance.candidate_runner_sha256,
        provenance.generator_sha256,
        provenance.baseline_source_id,
        provenance.candidate_source_id,
    );
    for (index, run) in report.runs.iter().enumerate() {
        markdown.push_str(&format!(
            "| {} | {} | `{}` | {:.6} | {:.6} | {:.6} | {:.6} | {:.6} | {:.6} |\n",
            index + 1,
            run.report.replace('|', "\\|"),
            run.report_sha256,
            run.ratios.overall,
            run.ratios.clockwise_atomic,
            run.ratios.msaa,
            run.strict_selected_diagnostic.overall,
            run.strict_selected_diagnostic.clockwise_atomic,
            run.strict_selected_diagnostic.msaa,
        ));
    }
    markdown.push_str(&format!(
        "\n## Gate result\n\n| mode | median ratio | passing runs | threshold |\n| --- | ---: | ---: | ---: |\n| overall | {:.6} | {}/{} | {:.6} |\n| clockwise-atomic | {:.6} | {}/{} | {:.6} |\n| MSAA | {:.6} | {}/{} | {:.6} |\n\nStrict selected diagnostic (non-gating) medians: overall `{:.6}`, clockwise-atomic `{:.6}`, MSAA `{:.6}`.\n",
        report.medians.overall,
        report.pass_counts.overall,
        report.run_count,
        report.max_ratio,
        report.medians.clockwise_atomic,
        report.pass_counts.clockwise_atomic,
        report.run_count,
        report.max_ratio,
        report.medians.msaa,
        report.pass_counts.msaa,
        report.run_count,
        report.max_ratio,
        report.strict_selected_diagnostic_medians.overall,
        report
            .strict_selected_diagnostic_medians
            .clockwise_atomic,
        report.strict_selected_diagnostic_medians.msaa,
    ));
    markdown
}

pub fn check_threshold(report: &ParityGateReport) -> Result<(), String> {
    if report.passed {
        return Ok(());
    }
    let failed = [
        ("overall", report.medians.overall),
        ("clockwise-atomic", report.medians.clockwise_atomic),
        ("msaa", report.medians.msaa),
    ]
    .into_iter()
    .filter(|(_, ratio)| *ratio > report.max_ratio)
    .map(|(mode, ratio)| format!("{mode}={ratio:.6}"))
    .collect::<Vec<_>>()
    .join(", ");
    Err(format!(
        "renderer parity threshold failed: {failed} exceed {:.6}",
        report.max_ratio
    ))
}

struct LoadedReport {
    path: String,
    sha256: String,
    report: Report,
}

fn validate_report_collection(reports: &[LoadedReport]) -> Result<(), String> {
    let reference_provenance = &reports[0].report.provenance;
    validate_provenance(reference_provenance, &reports[0].path)?;
    let mut reference_adapter = None;
    let mut reference_structural = None;
    for (report_index, loaded) in reports.iter().enumerate() {
        if &loaded.report.provenance != reference_provenance {
            return Err(format!(
                "report {} provenance mismatch across independent runs",
                loaded.path
            ));
        }
        validate_provenance(&loaded.report.provenance, &loaded.path)?;
        validate_report(
            &loaded.report,
            &loaded.path,
            report_index,
            &mut reference_adapter,
            &mut reference_structural,
        )?;
    }
    Ok(())
}

fn validate_provenance(provenance: &ReportProvenance, label: &str) -> Result<(), String> {
    for (name, hash) in [
        ("manifest", &provenance.manifest_sha256),
        ("baseline runner", &provenance.baseline_runner_sha256),
        ("candidate runner", &provenance.candidate_runner_sha256),
        ("generator", &provenance.generator_sha256),
    ] {
        if hash.len() != 64
            || !hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(format!("report {label} has an invalid {name} SHA-256"));
        }
    }
    if provenance.baseline_source_id.trim().is_empty() {
        return Err(format!(
            "report {label} has an empty baseline source identity"
        ));
    }
    if provenance.candidate_source_id.trim().is_empty() {
        return Err(format!(
            "report {label} has an empty candidate source identity"
        ));
    }
    Ok(())
}

fn validate_report(
    report: &Report,
    label: &str,
    _report_index: usize,
    reference_adapter: &mut Option<crate::renderer_perf::AdapterIdentity>,
    reference_structural: &mut Option<Vec<crate::renderer_perf::StructuralMetrics>>,
) -> Result<(), String> {
    if report.schema != REPORT_SCHEMA
        || report.runner_protocol != RUNNER_PROTOCOL
        || report.estimator != ESTIMATOR
        || report.pair_order != PAIR_ORDER
        || !report.release
        || report.profile != "release"
        || report.debug
        || report.samples_per_runner != SAMPLE_COUNT
        || report.manifest_schema != MANIFEST_SCHEMA
    {
        return Err(format!(
            "report {label} does not declare the fixed renderer-perf protocol"
        ));
    }
    let expected_scene_count =
        crate::renderer_perf::REQUIRED_SCENES.len() * crate::renderer_perf::REQUIRED_MODES.len();
    if report.scenes.len() != expected_scene_count {
        return Err(format!(
            "report {label} must contain the exact {expected_scene_count} fixed scene variants in order"
        ));
    }

    let mut structural = Vec::with_capacity(expected_scene_count);
    for (scene_index, scene) in report.scenes.iter().enumerate() {
        let source_scene = scene_index / crate::renderer_perf::REQUIRED_MODES.len();
        let mode_index = scene_index % crate::renderer_perf::REQUIRED_MODES.len();
        let (expected_scene_id, expected_stream) =
            crate::renderer_perf::REQUIRED_SCENES[source_scene];
        let expected_mode = crate::renderer_perf::REQUIRED_MODES[mode_index];
        let expected_id = format!(
            "{expected_scene_id}-{}",
            crate::renderer_perf::mode_name(expected_mode)
        );
        if scene.id != expected_id
            || scene.stream != expected_stream
            || scene.frame != 0
            || scene.mode != expected_mode
            || scene.width != 1024
            || scene.height != 1024
            || scene.adapter_selection != "high-performance"
        {
            return Err(format!(
                "report {label} scene {} does not match the fixed scene identity/order",
                scene_index + 1
            ));
        }
        if scene.selected_adapter.backend != "metal"
            || [
                &scene.selected_adapter.name,
                &scene.selected_adapter.vendor,
                &scene.selected_adapter.device,
                &scene.selected_adapter.driver,
            ]
            .iter()
            .any(|field| field.trim().is_empty())
        {
            return Err(format!(
                "report {label} scene {} does not identify a complete Metal adapter",
                scene.id
            ));
        }
        match reference_adapter {
            Some(expected) if expected != &scene.selected_adapter => {
                return Err(format!(
                    "report {label} scene {} adapter mismatch across independent runs",
                    scene.id
                ));
            }
            None => *reference_adapter = Some(scene.selected_adapter.clone()),
            _ => {}
        }
        if scene.timing != TimingMethod::required() {
            return Err(format!(
                "report {label} scene {} does not use the fixed timing method",
                scene.id
            ));
        }
        validate_scene_timings(scene, label, scene_index)?;
        structural.push(scene.structural);
    }
    match reference_structural {
        Some(expected) if expected != &structural => {
            return Err(format!(
                "report {label} structural mismatch across independent runs"
            ));
        }
        None => *reference_structural = Some(structural),
        _ => {}
    }
    validate_aggregate(report, label)
}

fn validate_scene_timings(
    scene: &SceneReport,
    label: &str,
    scene_index: usize,
) -> Result<(), String> {
    if scene.sample_orders.len() != SAMPLE_COUNT
        || scene.baseline.sample_medians_ns.len() != SAMPLE_COUNT
        || scene.candidate.sample_medians_ns.len() != SAMPLE_COUNT
    {
        return Err(format!(
            "report {label} scene {} must contain exactly {SAMPLE_COUNT} paired samples",
            scene.id
        ));
    }
    for (sample, &order) in scene.sample_orders.iter().enumerate() {
        let expected = if (scene_index + sample).is_multiple_of(2) {
            SampleOrder::CppThenCandidate
        } else {
            SampleOrder::CandidateThenCpp
        };
        if order != expected {
            return Err(format!(
                "report {label} scene {} has an invalid alternating sample order",
                scene.id
            ));
        }
    }
    validate_timing_summary(&scene.baseline, label, &scene.id, "baseline")?;
    validate_timing_summary(&scene.candidate, label, &scene.id, "candidate")?;

    let selected_index = scene
        .baseline
        .sample_medians_ns
        .iter()
        .enumerate()
        .min_by_key(|(_, timing)| *timing)
        .map(|(index, _)| index)
        .ok_or_else(|| format!("report {label} scene {} has no samples", scene.id))?;
    let cpp_control_ns = scene.baseline.sample_medians_ns[selected_index];
    let candidate_ns = scene.candidate.sample_medians_ns[selected_index];
    let expected_ratio = candidate_ns as f64 / cpp_control_ns as f64;
    if scene.control_selected_pair.sample_index != selected_index + 1
        || scene.control_selected_pair.cpp_control_ns != cpp_control_ns
        || scene.control_selected_pair.candidate_ns != candidate_ns
        || !same_number(
            scene.control_selected_pair.candidate_over_cpp,
            expected_ratio,
        )
    {
        return Err(format!(
            "report {label} scene {} has an inconsistent selected-min diagnostic",
            scene.id
        ));
    }
    Ok(())
}

fn validate_timing_summary(
    summary: &crate::renderer_perf::TimingSummary,
    label: &str,
    scene: &str,
    runner: &str,
) -> Result<(), String> {
    if summary.sample_medians_ns.contains(&0) {
        return Err(format!(
            "report {label} scene {scene} {runner} contains a zero timing"
        ));
    }
    let mut sorted = summary.sample_medians_ns.clone();
    sorted.sort_unstable();
    let min = sorted[0];
    let p50 = sorted[(sorted.len() - 1) / 2];
    let p95_index = ((sorted.len() * 95).div_ceil(100)).saturating_sub(1);
    let p95 = sorted[p95_index];
    let spread = sorted[sorted.len() - 1] - min;
    if summary.min_of_medians_ns != min
        || summary.p50_ns != p50
        || summary.p95_ns != p95
        || summary.spread_ns != spread
    {
        return Err(format!(
            "report {label} scene {scene} has an inconsistent {runner} timing summary"
        ));
    }
    Ok(())
}

fn validate_aggregate(report: &Report, label: &str) -> Result<(), String> {
    let cpp_sum = report.scenes.iter().try_fold(0_u64, |sum, scene| {
        sum.checked_add(scene.control_selected_pair.cpp_control_ns)
            .ok_or_else(|| format!("report {label} selected baseline timing overflow"))
    })?;
    let candidate_sum = report.scenes.iter().try_fold(0_u64, |sum, scene| {
        sum.checked_add(scene.control_selected_pair.candidate_ns)
            .ok_or_else(|| format!("report {label} selected candidate timing overflow"))
    })?;
    let ratio = candidate_sum as f64 / cpp_sum as f64;
    let mut worst = &report.scenes[0];
    for scene in &report.scenes[1..] {
        if scene.control_selected_pair.candidate_over_cpp
            > worst.control_selected_pair.candidate_over_cpp
        {
            worst = scene;
        }
    }
    if report.aggregate.cpp_control_selected_ns_sum != cpp_sum
        || report.aggregate.candidate_paired_ns_sum != candidate_sum
        || !same_number(report.aggregate.candidate_over_cpp, ratio)
        || report.aggregate.worst_scene.id != worst.id
        || report.aggregate.worst_scene.sample_index != worst.control_selected_pair.sample_index
        || report.aggregate.worst_scene.cpp_control_ns != worst.control_selected_pair.cpp_control_ns
        || report.aggregate.worst_scene.candidate_ns != worst.control_selected_pair.candidate_ns
        || !same_number(
            report.aggregate.worst_scene.candidate_over_cpp,
            worst.control_selected_pair.candidate_over_cpp,
        )
    {
        return Err(format!(
            "report {label} has an inconsistent selected-min aggregate diagnostic"
        ));
    }
    Ok(())
}

fn same_number(actual: f64, expected: f64) -> bool {
    actual.is_finite()
        && expected.is_finite()
        && (actual - expected).abs() <= f64::EPSILON * 8.0 * expected.abs().max(1.0)
}

fn mode_values(
    report: &Report,
    estimator: fn(&[&SceneReport]) -> Result<f64, String>,
) -> Result<ModeValues<f64>, String> {
    let all = report.scenes.iter().collect::<Vec<_>>();
    let clockwise_atomic = report
        .scenes
        .iter()
        .filter(|scene| scene.mode == Mode::ClockwiseAtomic)
        .collect::<Vec<_>>();
    let msaa = report
        .scenes
        .iter()
        .filter(|scene| scene.mode == Mode::Msaa)
        .collect::<Vec<_>>();
    Ok(ModeValues {
        overall: estimator(&all)?,
        clockwise_atomic: estimator(&clockwise_atomic)?,
        msaa: estimator(&msaa)?,
    })
}

fn equal_order_ratio(scenes: &[&SceneReport]) -> Result<f64, String> {
    let mut ratios = Vec::with_capacity(2);
    for order in [SampleOrder::CppThenCandidate, SampleOrder::CandidateThenCpp] {
        let mut baseline_sum = 0.0;
        let mut candidate_sum = 0.0;
        for scene in scenes {
            let indices = scene
                .sample_orders
                .iter()
                .enumerate()
                .filter_map(|(index, &sample_order)| (sample_order == order).then_some(index))
                .collect::<Vec<_>>();
            let baseline = indices
                .iter()
                .map(|&index| scene.baseline.sample_medians_ns[index])
                .collect::<Vec<_>>();
            let candidate = indices
                .iter()
                .map(|&index| scene.candidate.sample_medians_ns[index])
                .collect::<Vec<_>>();
            baseline_sum += median_u64(&baseline)?;
            candidate_sum += median_u64(&candidate)?;
        }
        if baseline_sum == 0.0 {
            return Err("cannot calculate ratio with zero baseline".to_owned());
        }
        ratios.push(candidate_sum / baseline_sum);
    }
    Ok((ratios[0] + ratios[1]) / 2.0)
}

fn strict_selected_ratio(scenes: &[&SceneReport]) -> Result<f64, String> {
    let baseline_sum = scenes.iter().try_fold(0_u64, |sum, scene| {
        sum.checked_add(scene.control_selected_pair.cpp_control_ns)
            .ok_or_else(|| "strict selected baseline timing overflow".to_owned())
    })?;
    let candidate_sum = scenes.iter().try_fold(0_u64, |sum, scene| {
        sum.checked_add(scene.control_selected_pair.candidate_ns)
            .ok_or_else(|| "strict selected candidate timing overflow".to_owned())
    })?;
    if baseline_sum == 0 {
        return Err("cannot calculate ratio with zero baseline".to_owned());
    }
    Ok(candidate_sum as f64 / baseline_sum as f64)
}

fn median_u64(values: &[u64]) -> Result<f64, String> {
    if values.is_empty() {
        return Err("cannot calculate median of an empty sample order".to_owned());
    }
    let mut values = values.to_vec();
    values.sort_unstable();
    let middle = values.len() / 2;
    if values.len().is_multiple_of(2) {
        Ok((values[middle - 1] as f64 + values[middle] as f64) / 2.0)
    } else {
        Ok(values[middle] as f64)
    }
}

fn median_run_values(
    runs: &[GateRun],
    value: impl Fn(&GateRun) -> ModeValues<f64>,
) -> ModeValues<f64> {
    ModeValues {
        overall: median_f64(runs.iter().map(|run| value(run).overall)),
        clockwise_atomic: median_f64(runs.iter().map(|run| value(run).clockwise_atomic)),
        msaa: median_f64(runs.iter().map(|run| value(run).msaa)),
    }
}

fn median_f64(values: impl Iterator<Item = f64>) -> f64 {
    let mut values = values.collect::<Vec<_>>();
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer_perf::{
        AdapterIdentity, Aggregate, ControlSelectedPair, GpuCompletion, MANIFEST_SCHEMA, Mode,
        PAIR_ORDER, REPORT_SCHEMA, RUNNER_PROTOCOL, SAMPLE_COUNT, SampleOrder, SceneReport,
        StructuralMetrics, TimingMethod, TimingScope, TimingSummary, WorstScene,
    };
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_ID: AtomicU64 = AtomicU64::new(0);

    struct TempReports {
        directory: PathBuf,
    }

    impl TempReports {
        fn new() -> Self {
            let directory = std::env::temp_dir().join(format!(
                "rive-renderer-parity-gate-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                TEMP_ID.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&directory).unwrap();
            Self { directory }
        }

        fn write(&self, reports: &[Report]) -> Vec<PathBuf> {
            reports
                .iter()
                .enumerate()
                .map(|(index, report)| {
                    let path = self.directory.join(format!("report-{}.json", index + 1));
                    std::fs::write(&path, serde_json::to_string_pretty(report).unwrap()).unwrap();
                    path
                })
                .collect()
        }
    }

    impl Drop for TempReports {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.directory).unwrap();
        }
    }

    fn valid_report() -> Report {
        let mut scenes = Vec::new();
        for (scene_number, (scene_id, stream)) in
            crate::renderer_perf::REQUIRED_SCENES.iter().enumerate()
        {
            for &mode in &crate::renderer_perf::REQUIRED_MODES {
                let scene_index = scenes.len();
                let sample_orders = (0..SAMPLE_COUNT)
                    .map(|sample| {
                        if (scene_index + sample) % 2 == 0 {
                            SampleOrder::CppThenCandidate
                        } else {
                            SampleOrder::CandidateThenCpp
                        }
                    })
                    .collect::<Vec<_>>();
                let (baseline_samples, candidate_samples) = samples_by_order(&sample_orders);
                let baseline = summary(baseline_samples);
                let candidate = summary(candidate_samples);
                let selected_index = baseline
                    .sample_medians_ns
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, value)| *value)
                    .unwrap()
                    .0;
                let cpp_control_ns = baseline.sample_medians_ns[selected_index];
                let candidate_ns = candidate.sample_medians_ns[selected_index];
                scenes.push(SceneReport {
                    id: format!("{scene_id}-{}", crate::renderer_perf::mode_name(mode)),
                    stream: (*stream).to_owned(),
                    frame: 0,
                    mode,
                    width: 1024,
                    height: 1024,
                    adapter_selection: "high-performance".to_owned(),
                    selected_adapter: adapter(),
                    timing: fixed_timing(),
                    baseline,
                    candidate,
                    sample_orders,
                    control_selected_pair: ControlSelectedPair {
                        sample_index: selected_index + 1,
                        cpp_control_ns,
                        candidate_ns,
                        candidate_over_cpp: candidate_ns as f64 / cpp_control_ns as f64,
                    },
                    structural: StructuralMetrics {
                        logical_flushes: (scene_number + 1) as u64,
                        draws: (scene_number + 10) as u64,
                        atomic_strategy_partitions: usize::from(mode == Mode::ClockwiseAtomic)
                            as u64,
                    },
                });
            }
        }
        let cpp_sum = scenes
            .iter()
            .map(|scene| scene.control_selected_pair.cpp_control_ns)
            .sum();
        let candidate_sum = scenes
            .iter()
            .map(|scene| scene.control_selected_pair.candidate_ns)
            .sum();
        let mut worst = &scenes[0];
        for scene in &scenes[1..] {
            if scene.control_selected_pair.candidate_over_cpp
                > worst.control_selected_pair.candidate_over_cpp
            {
                worst = scene;
            }
        }
        let worst_scene = WorstScene {
            id: worst.id.clone(),
            sample_index: worst.control_selected_pair.sample_index,
            cpp_control_ns: worst.control_selected_pair.cpp_control_ns,
            candidate_ns: worst.control_selected_pair.candidate_ns,
            candidate_over_cpp: worst.control_selected_pair.candidate_over_cpp,
        };
        Report {
            schema: REPORT_SCHEMA.to_owned(),
            runner_protocol: RUNNER_PROTOCOL.to_owned(),
            estimator: crate::renderer_perf::ESTIMATOR.to_owned(),
            pair_order: PAIR_ORDER.to_owned(),
            release: true,
            profile: "release".to_owned(),
            debug: false,
            samples_per_runner: SAMPLE_COUNT,
            manifest_schema: MANIFEST_SCHEMA.to_owned(),
            provenance: provenance(),
            scenes,
            aggregate: Aggregate {
                cpp_control_selected_ns_sum: cpp_sum,
                candidate_paired_ns_sum: candidate_sum,
                candidate_over_cpp: candidate_sum as f64 / cpp_sum as f64,
                worst_scene,
            },
        }
    }

    fn samples_by_order(orders: &[SampleOrder]) -> (Vec<u64>, Vec<u64>) {
        let mut cpp_index = 0;
        let mut candidate_index = 0;
        let mut baseline = Vec::new();
        let mut candidate = Vec::new();
        for order in orders {
            match order {
                SampleOrder::CppThenCandidate => {
                    let baseline_values = [100, 200, 200, 300];
                    let candidate_values = [100, 300, 300, 500];
                    baseline.push(baseline_values[cpp_index]);
                    candidate.push(candidate_values[cpp_index]);
                    cpp_index += 1;
                }
                SampleOrder::CandidateThenCpp => {
                    baseline.push(100);
                    candidate.push(50);
                    candidate_index += 1;
                }
            }
        }
        let _ = candidate_index;
        (baseline, candidate)
    }

    fn summary(samples: Vec<u64>) -> TimingSummary {
        let mut sorted = samples.clone();
        sorted.sort_unstable();
        TimingSummary {
            sample_medians_ns: samples,
            min_of_medians_ns: sorted[0],
            p50_ns: sorted[3],
            p95_ns: sorted[6],
            spread_ns: sorted[6] - sorted[0],
        }
    }

    fn fixed_timing() -> TimingMethod {
        TimingMethod {
            warmup_frames: 10,
            measured_frames: 100,
            scope: TimingScope::SubmitToGpuComplete,
            gpu_completion: GpuCompletion::WaitForCompletionEachFrame,
        }
    }

    fn adapter() -> AdapterIdentity {
        AdapterIdentity {
            backend: "metal".to_owned(),
            name: "Test Metal GPU".to_owned(),
            vendor: "Test Vendor".to_owned(),
            device: "Test Device".to_owned(),
            driver: "1.0".to_owned(),
        }
    }

    fn provenance() -> ReportProvenance {
        ReportProvenance {
            manifest_sha256: "11".repeat(32),
            baseline_runner_sha256: "22".repeat(32),
            candidate_runner_sha256: "33".repeat(32),
            generator_sha256: "44".repeat(32),
            baseline_source_id: "git:baseline".to_owned(),
            candidate_source_id: "git:test+dirty-sha256:test".to_owned(),
        }
    }

    fn write_five(report: Report) -> (TempReports, Vec<PathBuf>) {
        let directory = TempReports::new();
        let reports = distinct_reports(report);
        let paths = directory.write(&reports);
        (directory, paths)
    }

    fn distinct_reports(report: Report) -> Vec<Report> {
        (1..=REQUIRED_REPORT_COUNT)
            .map(|factor| scaled_report(report.clone(), factor as u64))
            .collect()
    }

    fn scaled_report(mut report: Report, factor: u64) -> Report {
        for scene in &mut report.scenes {
            scale_summary(&mut scene.baseline, factor);
            scale_summary(&mut scene.candidate, factor);
            scene.control_selected_pair.cpp_control_ns *= factor;
            scene.control_selected_pair.candidate_ns *= factor;
        }
        report.aggregate.cpp_control_selected_ns_sum *= factor;
        report.aggregate.candidate_paired_ns_sum *= factor;
        report.aggregate.worst_scene.cpp_control_ns *= factor;
        report.aggregate.worst_scene.candidate_ns *= factor;
        report
    }

    fn scale_summary(summary: &mut TimingSummary, factor: u64) {
        for timing in &mut summary.sample_medians_ns {
            *timing *= factor;
        }
        summary.min_of_medians_ns *= factor;
        summary.p50_ns *= factor;
        summary.p95_ns *= factor;
        summary.spread_ns *= factor;
    }

    fn scale_candidate_mode(report: &mut Report, mode: Mode, numerator: u64, denominator: u64) {
        for scene in report.scenes.iter_mut().filter(|scene| scene.mode == mode) {
            scale_summary_ratio(&mut scene.candidate, numerator, denominator);
            scene.control_selected_pair.candidate_ns =
                scene.control_selected_pair.candidate_ns * numerator / denominator;
            scene.control_selected_pair.candidate_over_cpp =
                scene.control_selected_pair.candidate_ns as f64
                    / scene.control_selected_pair.cpp_control_ns as f64;
        }
        recompute_aggregate(report);
    }

    fn scale_summary_ratio(summary: &mut TimingSummary, numerator: u64, denominator: u64) {
        for timing in &mut summary.sample_medians_ns {
            *timing = *timing * numerator / denominator;
        }
        summary.min_of_medians_ns = summary.min_of_medians_ns * numerator / denominator;
        summary.p50_ns = summary.p50_ns * numerator / denominator;
        summary.p95_ns = summary.p95_ns * numerator / denominator;
        summary.spread_ns = summary.spread_ns * numerator / denominator;
    }

    fn recompute_aggregate(report: &mut Report) {
        let cpp_sum = report
            .scenes
            .iter()
            .map(|scene| scene.control_selected_pair.cpp_control_ns)
            .sum();
        let candidate_sum = report
            .scenes
            .iter()
            .map(|scene| scene.control_selected_pair.candidate_ns)
            .sum();
        let mut worst = &report.scenes[0];
        for scene in &report.scenes[1..] {
            if scene.control_selected_pair.candidate_over_cpp
                > worst.control_selected_pair.candidate_over_cpp
            {
                worst = scene;
            }
        }
        report.aggregate = Aggregate {
            cpp_control_selected_ns_sum: cpp_sum,
            candidate_paired_ns_sum: candidate_sum,
            candidate_over_cpp: candidate_sum as f64 / cpp_sum as f64,
            worst_scene: WorstScene {
                id: worst.id.clone(),
                sample_index: worst.control_selected_pair.sample_index,
                cpp_control_ns: worst.control_selected_pair.cpp_control_ns,
                candidate_ns: worst.control_selected_pair.candidate_ns,
                candidate_over_cpp: worst.control_selected_pair.candidate_over_cpp,
            },
        };
    }

    #[test]
    fn estimator_equal_weights_the_two_launch_orders_after_summing_scene_medians() {
        let (_directory, paths) = write_five(valid_report());

        let gate = evaluate_report_files(&paths, 1.0).expect("valid reports must evaluate");

        assert_eq!(gate.medians.overall, 1.0);
        assert_eq!(gate.medians.clockwise_atomic, 1.0);
        assert_eq!(gate.medians.msaa, 1.0);
        assert!(gate.passed);
    }

    #[test]
    fn copied_report_contents_do_not_count_as_independent_runs() {
        let directory = TempReports::new();
        let paths = directory.write(&vec![valid_report(); REQUIRED_REPORT_COUNT]);

        let error = evaluate_report_files(&paths, 1.0).expect_err("replays must be rejected");

        assert!(error.contains("duplicate report contents"), "{error}");
    }

    #[test]
    fn reserialized_copy_does_not_count_as_an_independent_run() {
        let directory = TempReports::new();
        let mut reports = distinct_reports(valid_report());
        reports[1] = reports[0].clone();
        let paths = directory.write(&reports);
        std::fs::write(&paths[1], serde_json::to_string(&reports[1]).unwrap()).unwrap();

        let error = evaluate_report_files(&paths, 1.0).expect_err("reserialized replay must fail");

        assert!(error.contains("duplicate report contents"), "{error}");
    }

    #[test]
    fn path_aliases_do_not_count_as_independent_runs() {
        let (_directory, mut paths) = write_five(valid_report());
        paths[4] = paths[0].clone();

        let error = evaluate_report_files(&paths, 1.0).expect_err("path aliases must fail");

        assert!(error.contains("duplicate report path alias"), "{error}");
    }

    #[test]
    fn exactly_five_reports_are_required() {
        let directory = TempReports::new();
        let reports = (1..=4)
            .map(|factor| scaled_report(valid_report(), factor))
            .collect::<Vec<_>>();
        let paths = directory.write(&reports);

        let error = evaluate_report_files(&paths, 1.0).expect_err("four reports must fail");

        assert!(error.contains("exactly 5 independent reports"), "{error}");
    }

    #[test]
    fn threshold_fails_when_only_the_msaa_median_exceeds_it() {
        let mut report = valid_report();
        scale_candidate_mode(&mut report, Mode::ClockwiseAtomic, 1, 2);
        scale_candidate_mode(&mut report, Mode::Msaa, 6, 5);
        let (_directory, paths) = write_five(report);

        let gate = evaluate_report_files(&paths, 1.0).expect("reports must evaluate");

        assert!(gate.medians.overall < 1.0);
        assert!(gate.medians.clockwise_atomic < 1.0);
        assert!(gate.medians.msaa > 1.0);
        assert_eq!(gate.pass_counts.overall, 5);
        assert_eq!(gate.pass_counts.clockwise_atomic, 5);
        assert_eq!(gate.pass_counts.msaa, 0);
        assert!(!gate.passed);
    }

    #[test]
    fn threshold_gates_the_median_run_and_records_individual_pass_counts() {
        let directory = TempReports::new();
        let factors = [(4, 5), (9, 10), (1, 1), (11, 10), (6, 5)];
        let reports = factors
            .into_iter()
            .enumerate()
            .map(|(index, (numerator, denominator))| {
                let mut report = valid_report();
                scale_candidate_mode(&mut report, Mode::ClockwiseAtomic, numerator, denominator);
                scaled_report(report, index as u64 + 1)
            })
            .collect::<Vec<_>>();
        let paths = directory.write(&reports);

        let gate = evaluate_report_files(&paths, 1.0).expect("reports must evaluate");

        assert_eq!(gate.medians.overall, 1.0);
        assert_eq!(gate.medians.clockwise_atomic, 1.0);
        assert_eq!(gate.medians.msaa, 1.0);
        assert_eq!(gate.pass_counts.overall, 3);
        assert_eq!(gate.pass_counts.clockwise_atomic, 3);
        assert_eq!(gate.pass_counts.msaa, 5);
        assert!(gate.passed);
    }

    #[test]
    fn provenance_must_match_across_all_five_reports() {
        let directory = TempReports::new();
        let mut reports = distinct_reports(valid_report());
        reports[4].provenance.generator_sha256 = "55".repeat(32);
        let paths = directory.write(&reports);

        let error = evaluate_report_files(&paths, 1.0).expect_err("mixed provenance must fail");

        assert!(error.contains("provenance mismatch"), "{error}");
    }

    #[test]
    fn metal_adapter_must_match_across_scenes_and_runs() {
        let directory = TempReports::new();
        let mut reports = distinct_reports(valid_report());
        reports[3].scenes[7].selected_adapter.device = "different-device".to_owned();
        let paths = directory.write(&reports);

        let error = evaluate_report_files(&paths, 1.0).expect_err("mixed adapters must fail");

        assert!(error.contains("adapter mismatch"), "{error}");
    }

    #[test]
    fn fixed_scene_identity_and_order_are_required() {
        let directory = TempReports::new();
        let mut reports = distinct_reports(valid_report());
        reports[2].scenes.swap(0, 1);
        let paths = directory.write(&reports);

        let error = evaluate_report_files(&paths, 1.0).expect_err("scene reorder must fail");

        assert!(error.contains("fixed scene identity/order"), "{error}");
    }

    #[test]
    fn alternating_sample_order_is_required() {
        let directory = TempReports::new();
        let mut reports = distinct_reports(valid_report());
        reports[1].scenes[0].sample_orders[0] = SampleOrder::CandidateThenCpp;
        let paths = directory.write(&reports);

        let error = evaluate_report_files(&paths, 1.0).expect_err("bad sample order must fail");

        assert!(error.contains("alternating sample order"), "{error}");
    }

    #[test]
    fn structural_metrics_must_match_across_runs() {
        let directory = TempReports::new();
        let mut reports = distinct_reports(valid_report());
        reports[4].scenes[0].structural.draws += 1;
        let paths = directory.write(&reports);

        let error = evaluate_report_files(&paths, 1.0).expect_err("mixed structure must fail");

        assert!(error.contains("structural mismatch"), "{error}");
    }

    #[test]
    fn malformed_sample_vectors_are_rejected_without_indexing_them() {
        let directory = TempReports::new();
        let mut reports = distinct_reports(valid_report());
        reports[0].scenes[0].candidate.sample_medians_ns.pop();
        let paths = directory.write(&reports);

        let error = evaluate_report_files(&paths, 1.0).expect_err("short samples must fail");

        assert!(error.contains("exactly 7 paired samples"), "{error}");
    }

    #[test]
    fn report_protocol_and_fixed_timing_are_required() {
        let directory = TempReports::new();
        let mut reports = distinct_reports(valid_report());
        reports[0].schema = "not-the-renderer-schema".to_owned();
        let paths = directory.write(&reports);
        let error = evaluate_report_files(&paths, 1.0).expect_err("bad schema must fail");
        assert!(error.contains("fixed renderer-perf protocol"), "{error}");

        let directory = TempReports::new();
        let mut reports = distinct_reports(valid_report());
        reports[0].scenes[0].timing.measured_frames = 99;
        let paths = directory.write(&reports);
        let error = evaluate_report_files(&paths, 1.0).expect_err("bad timing must fail");
        assert!(error.contains("fixed timing method"), "{error}");
    }

    #[test]
    fn json_and_markdown_record_each_run_diagnostics_and_provenance() {
        let (_directory, paths) = write_five(valid_report());
        let gate = evaluate_report_files(&paths, 1.0).expect("reports must evaluate");

        let json: serde_json::Value =
            serde_json::from_str(&render_json(&gate).expect("JSON must render")).unwrap();
        let markdown = render_markdown(&gate);

        assert_eq!(json["schema"], PARITY_GATE_SCHEMA);
        assert_eq!(
            json["runs"].as_array().unwrap().len(),
            REQUIRED_REPORT_COUNT
        );
        assert_eq!(json["runs"][0]["report_sha256"].as_str().unwrap().len(), 64);
        assert_eq!(json["provenance"]["baseline_source_id"], "git:baseline");
        assert_eq!(
            json["provenance"]["candidate_source_id"],
            "git:test+dirty-sha256:test"
        );
        assert!(markdown.contains("Strict selected diagnostic (non-gating)"));
        assert!(markdown.contains("git:test+dirty-sha256:test"));
    }
}
