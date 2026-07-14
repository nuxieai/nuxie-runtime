use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const MANIFEST_SCHEMA: &str = "rive-renderer-perf-scenes-v1";
pub const RUNNER_PROTOCOL: &str = "rive-renderer-perf-runner-v1";
pub const REPORT_SCHEMA: &str = "rive-renderer-perf-v1";
pub const SAMPLE_COUNT: usize = 7;

const REQUIRED_SCENES: [(&str, &str); 8] = [
    (
        "gm-CubicStroke",
        "../../fixtures/renderer/streams/gm/CubicStroke.rive-stream",
    ),
    (
        "gm-OverStroke",
        "../../fixtures/renderer/streams/gm/OverStroke.rive-stream",
    ),
    (
        "gm-batchedconvexpaths",
        "../../fixtures/renderer/streams/gm/batchedconvexpaths.rive-stream",
    ),
    (
        "gm-batchedtriangulations",
        "../../fixtures/renderer/streams/gm/batchedtriangulations.rive-stream",
    ),
    (
        "gm-bevel180strokes",
        "../../fixtures/renderer/streams/gm/bevel180strokes.rive-stream",
    ),
    (
        "gm-bug339297",
        "../../fixtures/renderer/streams/gm/bug339297.rive-stream",
    ),
    (
        "gm-bug339297_as_clip",
        "../../fixtures/renderer/streams/gm/bug339297_as_clip.rive-stream",
    ),
    (
        "gm-bug5099",
        "../../fixtures/renderer/streams/gm/bug5099.rive-stream",
    ),
];

const REQUIRED_MODES: [Mode; 2] = [Mode::ClockwiseAtomic, Mode::Msaa];

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    ClockwiseAtomic,
    Msaa,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub modes: Vec<Mode>,
    pub defaults: Defaults,
    pub scene: Vec<Scene>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Defaults {
    pub width: u32,
    pub height: u32,
    pub frame: u32,
    pub adapter: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub id: String,
    pub stream: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunRequest {
    pub protocol: &'static str,
    pub release: bool,
    pub profile: &'static str,
    pub debug: bool,
    pub stream: String,
    pub frame: u32,
    pub mode: Mode,
    pub width: u32,
    pub height: u32,
    pub adapter: String,
}

impl RunRequest {
    fn for_scene(scene: &Scene, defaults: &Defaults, mode: Mode) -> Self {
        Self {
            protocol: RUNNER_PROTOCOL,
            release: true,
            profile: "release",
            debug: false,
            stream: scene.stream.clone(),
            frame: defaults.frame,
            mode,
            width: defaults.width,
            height: defaults.height,
            adapter: defaults.adapter.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunnerResponse {
    pub protocol: String,
    pub release: bool,
    pub profile: String,
    pub debug: bool,
    pub stream: String,
    pub frame: u32,
    pub mode: Mode,
    pub width: u32,
    pub height: u32,
    pub adapter: String,
    pub median_ns: u64,
    pub flushes: u64,
    pub draws: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct StructuralMetrics {
    pub flushes: u64,
    pub draws: u64,
}

impl From<&RunnerResponse> for StructuralMetrics {
    fn from(response: &RunnerResponse) -> Self {
        Self {
            flushes: response.flushes,
            draws: response.draws,
        }
    }
}

pub trait Runner {
    fn run(&mut self, request: &RunRequest) -> Result<RunnerResponse, String>;
}

pub struct SubprocessRunner {
    program: PathBuf,
    working_directory: PathBuf,
}

impl SubprocessRunner {
    pub fn new(program: PathBuf, working_directory: PathBuf) -> Self {
        Self {
            program,
            working_directory,
        }
    }
}

impl Runner for SubprocessRunner {
    fn run(&mut self, request: &RunRequest) -> Result<RunnerResponse, String> {
        let mut child = Command::new(&self.program)
            .arg("--renderer-perf-protocol")
            .arg(RUNNER_PROTOCOL)
            .current_dir(&self.working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("failed to start {}: {error}", self.program.display()))?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| format!("{} did not expose stdin", self.program.display()))?;
        serde_json::to_writer(&mut stdin, request)
            .map_err(|error| format!("failed to encode runner request: {error}"))?;
        stdin
            .write_all(b"\n")
            .map_err(|error| format!("failed to send runner request: {error}"))?;
        drop(stdin);

        let output = child
            .wait_with_output()
            .map_err(|error| format!("failed to wait for {}: {error}", self.program.display()))?;
        if !output.status.success() {
            return Err(format!(
                "runner {} exited with {}: {}",
                self.program.display(),
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        parse_runner_response(&String::from_utf8_lossy(&output.stdout))
    }
}

pub fn load_manifest(path: &Path) -> Result<Manifest, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let manifest = toml::from_str::<Manifest>(&contents)
        .map_err(|error| format!("invalid renderer manifest {}: {error}", path.display()))?;
    let directory = path.parent().unwrap_or_else(|| Path::new("."));
    validate_manifest(&manifest, directory)?;
    Ok(manifest)
}

pub fn validate_manifest(manifest: &Manifest, directory: &Path) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!(
            "manifest schema must be {MANIFEST_SCHEMA}, got {}",
            manifest.schema
        ));
    }
    if manifest.defaults.width == 0 || manifest.defaults.height == 0 {
        return Err("manifest dimensions must be non-zero".to_owned());
    }
    if manifest.defaults.adapter.trim().is_empty() {
        return Err("manifest adapter must be non-empty".to_owned());
    }
    if manifest.scene.len() != REQUIRED_SCENES.len() {
        return Err(format!(
            "manifest must contain exactly {} scenes, got {}",
            REQUIRED_SCENES.len(),
            manifest.scene.len()
        ));
    }

    if manifest.modes != REQUIRED_MODES {
        return Err(format!(
            "manifest modes must be {}, {}",
            mode_name(REQUIRED_MODES[0]),
            mode_name(REQUIRED_MODES[1])
        ));
    }

    for (index, (scene, (id, stream))) in manifest.scene.iter().zip(REQUIRED_SCENES).enumerate() {
        if scene.id != id || scene.stream != stream {
            return Err(format!(
                "scene {} must be {id} ({stream}), got {} ({})",
                index + 1,
                scene.id,
                scene.stream,
            ));
        }
        let stream_path = directory.join(&scene.stream);
        if !stream_path.is_file() {
            return Err(format!(
                "scene {} stream does not exist: {}",
                scene.id,
                stream_path.display()
            ));
        }
    }
    Ok(())
}

pub fn parse_runner_response(stdout: &str) -> Result<RunnerResponse, String> {
    serde_json::from_str(stdout.trim()).map_err(|error| format!("malformed runner JSON: {error}"))
}

pub fn run_benchmark(
    manifest: &Manifest,
    baseline: &mut dyn Runner,
    candidate: &mut dyn Runner,
) -> Result<Report, String> {
    let mut scenes = Vec::with_capacity(manifest.scene.len());
    for scene in &manifest.scene {
        for &mode in &manifest.modes {
            let id = format!("{}-{}", scene.id, mode_name(mode));
            let request = RunRequest::for_scene(scene, &manifest.defaults, mode);
            let mut baseline_medians = Vec::with_capacity(SAMPLE_COUNT);
            let mut candidate_medians = Vec::with_capacity(SAMPLE_COUNT);
            let mut structural = None;

            for sample in 0..SAMPLE_COUNT {
                let baseline_response = baseline.run(&request)?;
                validate_response("baseline", &id, sample, &request, &baseline_response)?;
                let baseline_structural = StructuralMetrics::from(&baseline_response);
                validate_structural(&id, "baseline", sample, structural, baseline_structural)?;
                structural = Some(baseline_structural);
                baseline_medians.push(baseline_response.median_ns);

                let candidate_response = candidate.run(&request)?;
                validate_response("candidate", &id, sample, &request, &candidate_response)?;
                let candidate_structural = StructuralMetrics::from(&candidate_response);
                validate_structural(&id, "candidate", sample, structural, candidate_structural)?;
                candidate_medians.push(candidate_response.median_ns);
            }

            let baseline = TimingSummary::from_samples(baseline_medians)?;
            let candidate = TimingSummary::from_samples(candidate_medians)?;
            let ratio = ratio(candidate.min_of_medians_ns, baseline.min_of_medians_ns)?;
            scenes.push(SceneReport {
                id: id.clone(),
                stream: scene.stream.clone(),
                frame: manifest.defaults.frame,
                mode,
                width: manifest.defaults.width,
                height: manifest.defaults.height,
                adapter: manifest.defaults.adapter.clone(),
                baseline,
                candidate,
                structural: structural
                    .ok_or_else(|| format!("scene {id} did not produce structural metrics"))?,
                ratio,
            });
        }
    }

    let aggregate = Aggregate::from_scenes(&scenes)?;
    Ok(Report {
        schema: REPORT_SCHEMA,
        runner_protocol: RUNNER_PROTOCOL,
        release: true,
        profile: "release",
        debug: false,
        samples_per_runner: SAMPLE_COUNT,
        manifest_schema: MANIFEST_SCHEMA,
        scenes,
        aggregate,
    })
}

fn validate_response(
    runner: &str,
    scene_id: &str,
    sample: usize,
    request: &RunRequest,
    response: &RunnerResponse,
) -> Result<(), String> {
    let mismatch = response.protocol != request.protocol
        || response.release != request.release
        || response.profile != request.profile
        || response.debug != request.debug
        || response.stream != request.stream
        || response.frame != request.frame
        || response.mode != request.mode
        || response.width != request.width
        || response.height != request.height
        || response.adapter != request.adapter;
    if mismatch {
        return Err(format!(
            "scene {} sample {} {runner} response did not echo the fenced request",
            scene_id,
            sample + 1
        ));
    }
    if response.median_ns == 0 {
        return Err(format!(
            "scene {} sample {} {runner} reported zero median_ns",
            scene_id,
            sample + 1
        ));
    }
    Ok(())
}

fn validate_structural(
    scene_id: &str,
    runner: &str,
    sample: usize,
    expected: Option<StructuralMetrics>,
    actual: StructuralMetrics,
) -> Result<(), String> {
    if let Some(expected) = expected {
        if actual != expected {
            return Err(format!(
                "scene {} sample {} {runner} structural mismatch: expected flushes={} draws={}, got flushes={} draws={}",
                scene_id,
                sample + 1,
                expected.flushes,
                expected.draws,
                actual.flushes,
                actual.draws
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize)]
pub struct TimingSummary {
    pub sample_medians_ns: Vec<u64>,
    pub min_of_medians_ns: u64,
    pub p50_ns: u64,
    pub p95_ns: u64,
    pub spread_ns: u64,
}

impl TimingSummary {
    fn from_samples(sample_medians_ns: Vec<u64>) -> Result<Self, String> {
        if sample_medians_ns.len() != SAMPLE_COUNT {
            return Err(format!(
                "expected {SAMPLE_COUNT} sample medians, got {}",
                sample_medians_ns.len()
            ));
        }
        let mut sorted = sample_medians_ns.clone();
        sorted.sort_unstable();
        let min_of_medians_ns = sorted[0];
        let p50_ns = sorted[(sorted.len() - 1) / 2];
        let p95_index = ((sorted.len() * 95).div_ceil(100)).saturating_sub(1);
        let p95_ns = sorted[p95_index];
        let spread_ns = sorted[sorted.len() - 1] - min_of_medians_ns;
        Ok(Self {
            sample_medians_ns,
            min_of_medians_ns,
            p50_ns,
            p95_ns,
            spread_ns,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct SceneReport {
    pub id: String,
    pub stream: String,
    pub frame: u32,
    pub mode: Mode,
    pub width: u32,
    pub height: u32,
    pub adapter: String,
    pub baseline: TimingSummary,
    pub candidate: TimingSummary,
    pub structural: StructuralMetrics,
    pub ratio: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Aggregate {
    pub baseline_min_of_medians_ns_sum: u64,
    pub candidate_min_of_medians_ns_sum: u64,
    pub ratio: f64,
    pub worst_scene: String,
    pub worst_ratio: f64,
}

impl Aggregate {
    fn from_scenes(scenes: &[SceneReport]) -> Result<Self, String> {
        let baseline_min_of_medians_ns_sum = scenes
            .iter()
            .map(|scene| scene.baseline.min_of_medians_ns)
            .sum();
        let candidate_min_of_medians_ns_sum = scenes
            .iter()
            .map(|scene| scene.candidate.min_of_medians_ns)
            .sum();
        let ratio = ratio(
            candidate_min_of_medians_ns_sum,
            baseline_min_of_medians_ns_sum,
        )?;
        let worst = scenes
            .iter()
            .max_by(|left, right| left.ratio.total_cmp(&right.ratio))
            .ok_or_else(|| "report contains no scenes".to_owned())?;
        Ok(Self {
            baseline_min_of_medians_ns_sum,
            candidate_min_of_medians_ns_sum,
            ratio,
            worst_scene: worst.id.clone(),
            worst_ratio: worst.ratio,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub runner_protocol: &'static str,
    pub release: bool,
    pub profile: &'static str,
    pub debug: bool,
    pub samples_per_runner: usize,
    pub manifest_schema: &'static str,
    pub scenes: Vec<SceneReport>,
    pub aggregate: Aggregate,
}

pub fn render_json(report: &Report) -> Result<String, String> {
    serde_json::to_string_pretty(report)
        .map(|json| format!("{json}\n"))
        .map_err(|error| format!("failed to render JSON report: {error}"))
}

pub fn render_markdown(report: &Report) -> String {
    let mut markdown = String::from(
        "# Rive Renderer Performance\n\nSchema: `rive-renderer-perf-v1`  \nRunner protocol: `rive-renderer-perf-runner-v1`\n\n| scene | mode | baseline min ns | baseline p50 ns | baseline p95 ns | baseline spread ns | candidate min ns | candidate p50 ns | candidate p95 ns | candidate spread ns | ratio | flushes | draws |\n| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n",
    );
    for scene in &report.scenes {
        markdown.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {:.6} | {} | {} |\n",
            scene.id,
            mode_name(scene.mode),
            scene.baseline.min_of_medians_ns,
            scene.baseline.p50_ns,
            scene.baseline.p95_ns,
            scene.baseline.spread_ns,
            scene.candidate.min_of_medians_ns,
            scene.candidate.p50_ns,
            scene.candidate.p95_ns,
            scene.candidate.spread_ns,
            scene.ratio,
            scene.structural.flushes,
            scene.structural.draws,
        ));
    }
    markdown.push_str(&format!(
        "\nAggregate min-of-medians: baseline {} ns, candidate {} ns, ratio {:.6}. Worst scene: {} ({:.6}).\n",
        report.aggregate.baseline_min_of_medians_ns_sum,
        report.aggregate.candidate_min_of_medians_ns_sum,
        report.aggregate.ratio,
        report.aggregate.worst_scene,
        report.aggregate.worst_ratio,
    ));
    markdown
}

pub fn check_threshold(report: &Report, max_ratio: f64) -> Result<(), String> {
    if !max_ratio.is_finite() || max_ratio <= 0.0 {
        return Err("--max-ratio must be a finite value greater than zero".to_owned());
    }
    if report.aggregate.worst_ratio > max_ratio {
        return Err(format!(
            "renderer performance threshold failed: {} ratio {:.6} exceeds {:.6}",
            report.aggregate.worst_scene, report.aggregate.worst_ratio, max_ratio
        ));
    }
    Ok(())
}

fn ratio(numerator: u64, denominator: u64) -> Result<f64, String> {
    if denominator == 0 {
        return Err("cannot calculate ratio with zero baseline".to_owned());
    }
    Ok(numerator as f64 / denominator as f64)
}

fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::ClockwiseAtomic => "clockwise-atomic",
        Mode::Msaa => "msaa",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    struct FakeRunner {
        responses: VecDeque<RunnerResponse>,
    }

    impl Runner for FakeRunner {
        fn run(&mut self, _request: &RunRequest) -> Result<RunnerResponse, String> {
            self.responses
                .pop_front()
                .ok_or_else(|| "fake runner ran out of responses".to_owned())
        }
    }

    fn manifest() -> Manifest {
        let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("renderer-scenes.toml");
        load_manifest(&manifest_path).expect("fixture manifest must be valid")
    }

    fn response(scene: &Scene, defaults: &Defaults, mode: Mode, median_ns: u64) -> RunnerResponse {
        RunnerResponse {
            protocol: RUNNER_PROTOCOL.to_owned(),
            release: true,
            profile: "release".to_owned(),
            debug: false,
            stream: scene.stream.clone(),
            frame: defaults.frame,
            mode,
            width: defaults.width,
            height: defaults.height,
            adapter: defaults.adapter.clone(),
            median_ns,
            flushes: 3,
            draws: 11,
        }
    }

    fn responses_for(manifest: &Manifest, median_ns: u64) -> VecDeque<RunnerResponse> {
        manifest
            .scene
            .iter()
            .flat_map(|scene| {
                manifest.modes.iter().flat_map(move |&mode| {
                    std::iter::repeat_n(
                        response(scene, &manifest.defaults, mode, median_ns),
                        SAMPLE_COUNT,
                    )
                })
            })
            .collect()
    }

    #[test]
    fn malformed_runner_output_is_rejected() {
        let error = parse_runner_response("not-json").expect_err("must reject malformed output");
        assert!(error.contains("malformed runner JSON"));
    }

    #[test]
    fn fence_mismatch_is_rejected() {
        let manifest = manifest();
        let scene = &manifest.scene[0];
        let request = RunRequest::for_scene(scene, &manifest.defaults, manifest.modes[0]);
        let mut response = response(scene, &manifest.defaults, manifest.modes[0], 10);
        response.debug = true;
        let error = validate_response("candidate", &scene.id, 0, &request, &response)
            .expect_err("debug mismatch must fail");
        assert!(error.contains("fenced request"));
    }

    #[test]
    fn structural_mismatch_is_rejected() {
        let manifest = manifest();
        let mut baseline = FakeRunner {
            responses: responses_for(&manifest, 10),
        };
        let mut candidate_responses = responses_for(&manifest, 12);
        candidate_responses[0].draws = 12;
        let mut candidate = FakeRunner {
            responses: candidate_responses,
        };
        let error = run_benchmark(&manifest, &mut baseline, &mut candidate)
            .expect_err("draw parity must fail");
        assert!(error.contains("structural mismatch"));
        assert!(error.contains("draws"));
    }

    #[test]
    fn ratio_math_uses_min_of_medians() {
        let baseline =
            TimingSummary::from_samples(vec![16, 10, 15, 14, 13, 12, 11]).expect("seven samples");
        let candidate =
            TimingSummary::from_samples(vec![25, 15, 21, 19, 18, 17, 16]).expect("seven samples");
        assert_eq!(baseline.min_of_medians_ns, 10);
        assert_eq!(candidate.min_of_medians_ns, 15);
        assert_eq!(baseline.p50_ns, 13);
        assert_eq!(candidate.p95_ns, 25);
        assert_eq!(candidate.spread_ns, 10);
        assert_eq!(
            ratio(candidate.min_of_medians_ns, baseline.min_of_medians_ns).unwrap(),
            1.5
        );
    }

    #[test]
    fn report_ordering_follows_manifest_order() {
        let manifest = manifest();
        let mut baseline = FakeRunner {
            responses: responses_for(&manifest, 10),
        };
        let mut candidate = FakeRunner {
            responses: responses_for(&manifest, 12),
        };
        let report = run_benchmark(&manifest, &mut baseline, &mut candidate).expect("report");
        let json = render_json(&report).expect("json");
        let markdown = render_markdown(&report);
        assert_eq!(report.scenes.len(), 16);
        assert_eq!(report.scenes[0].id, "gm-CubicStroke-clockwise-atomic");
        assert_eq!(report.scenes[15].id, "gm-bug5099-msaa");
        assert!(
            json.find(&report.scenes[0].id).unwrap() < json.find(&report.scenes[15].id).unwrap()
        );
        assert!(
            markdown.find(&report.scenes[0].id).unwrap()
                < markdown.find(&report.scenes[15].id).unwrap()
        );
    }

    #[test]
    fn manifest_rejects_an_unapproved_scene_set() {
        let mut manifest = manifest();
        manifest.scene[0].id = "something-else".to_owned();
        let error = validate_manifest(&manifest, Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect_err("fixed scene list must not drift");
        assert!(error.contains("scene 1"));
    }
}
