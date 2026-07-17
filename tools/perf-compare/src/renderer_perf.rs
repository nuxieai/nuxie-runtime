use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const MANIFEST_SCHEMA: &str = "rive-renderer-perf-scenes-v1";
pub const RUNNER_PROTOCOL: &str = "rive-renderer-perf-runner-v1";
pub const REPORT_SCHEMA: &str = "rive-renderer-perf-v2";
pub const ESTIMATOR: &str = "cpp-control-min-paired-v1";
pub const PAIR_ORDER: &str = "counterbalanced-scene-sample-v1";
pub const SAMPLE_COUNT: usize = 7;
pub const WARMUP_FRAMES: u32 = 10;
pub const MEASURED_FRAMES: u32 = 100;
pub const COUNTER_WARMUP_FRAMES: u32 = 10;
pub const COUNTER_MEASURED_FRAMES: u32 = 1;

const REQUIRED_WIDTH: u32 = 1024;
const REQUIRED_HEIGHT: u32 = 1024;
const REQUIRED_FRAME: u32 = 0;
const REQUIRED_ADAPTER_SELECTION: &str = "high-performance";

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SampleOrder {
    CppThenCandidate,
    CandidateThenCpp,
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
    pub adapter_selection: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub id: String,
    pub stream: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunRequest {
    pub protocol: String,
    pub release: bool,
    pub profile: String,
    pub debug: bool,
    pub stream: String,
    pub frame: u32,
    pub mode: Mode,
    pub width: u32,
    pub height: u32,
    pub adapter_selection: String,
    #[serde(default)]
    pub measurement: Measurement,
    pub timing: TimingMethod,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Measurement {
    #[default]
    Timing,
    Counters,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TimingScope {
    SubmitToGpuComplete,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GpuCompletion {
    WaitForCompletionEachFrame,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TimingMethod {
    pub warmup_frames: u32,
    pub measured_frames: u32,
    pub scope: TimingScope,
    pub gpu_completion: GpuCompletion,
}

impl TimingMethod {
    pub fn required() -> Self {
        Self {
            warmup_frames: WARMUP_FRAMES,
            measured_frames: MEASURED_FRAMES,
            scope: TimingScope::SubmitToGpuComplete,
            gpu_completion: GpuCompletion::WaitForCompletionEachFrame,
        }
    }

    pub fn counters() -> Self {
        Self {
            warmup_frames: COUNTER_WARMUP_FRAMES,
            measured_frames: COUNTER_MEASURED_FRAMES,
            scope: TimingScope::SubmitToGpuComplete,
            gpu_completion: GpuCompletion::WaitForCompletionEachFrame,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterIdentity {
    pub backend: String,
    pub name: String,
    pub vendor: String,
    pub device: String,
    pub driver: String,
}

impl RunRequest {
    pub fn for_scene(
        scene: &Scene,
        defaults: &Defaults,
        mode: Mode,
        measurement: Measurement,
    ) -> Self {
        Self {
            protocol: RUNNER_PROTOCOL.to_owned(),
            release: true,
            profile: "release".to_owned(),
            debug: false,
            stream: scene.stream.clone(),
            frame: defaults.frame,
            mode,
            width: defaults.width,
            height: defaults.height,
            adapter_selection: defaults.adapter_selection.clone(),
            measurement,
            timing: match measurement {
                Measurement::Timing => TimingMethod::required(),
                Measurement::Counters => TimingMethod::counters(),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BackendWorkMetrics {
    pub command_encoders: u64,
    pub render_passes: u64,
    pub bind_groups_created: u64,
    pub bind_group_sets: u64,
    pub texture_bindings: u64,
    #[serde(default)]
    pub buffer_clear_calls: u64,
    #[serde(default)]
    pub buffer_clear_bytes: u64,
    pub buffer_upload_calls: u64,
    pub buffer_upload_bytes: u64,
    pub texture_upload_calls: u64,
    pub texture_upload_bytes: u64,
    pub queue_submissions: u64,
    pub gpu_draw_calls: u64,
    pub gpu_draw_instances: u64,
    pub tessellation_spans: u64,
    pub path_patches: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    pub adapter_selection: String,
    #[serde(default)]
    pub measurement: Measurement,
    pub selected_adapter: AdapterIdentity,
    pub timing: TimingMethod,
    pub measured_frame_median_ns: u64,
    pub logical_flushes: u64,
    pub draws: u64,
    pub atomic_strategy_partitions: u64,
    #[serde(default)]
    pub backend_work: BackendWorkMetrics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct StructuralMetrics {
    pub logical_flushes: u64,
    pub draws: u64,
    pub atomic_strategy_partitions: u64,
}

impl From<&RunnerResponse> for StructuralMetrics {
    fn from(response: &RunnerResponse) -> Self {
        Self {
            logical_flushes: response.logical_flushes,
            draws: response.draws,
            atomic_strategy_partitions: response.atomic_strategy_partitions,
        }
    }
}

pub fn validate_runner_request(request: &RunRequest) -> Result<(), String> {
    if request.protocol != RUNNER_PROTOCOL
        || !request.release
        || request.profile != "release"
        || request.debug
        || request.frame != REQUIRED_FRAME
        || request.width != REQUIRED_WIDTH
        || request.height != REQUIRED_HEIGHT
        || request.adapter_selection != REQUIRED_ADAPTER_SELECTION
        || request.timing
            != match request.measurement {
                Measurement::Timing => TimingMethod::required(),
                Measurement::Counters => TimingMethod::counters(),
            }
    {
        return Err(format!(
            "runner request must use protocol={RUNNER_PROTOCOL}, release profile, {REQUIRED_WIDTH}x{REQUIRED_HEIGHT} frame {REQUIRED_FRAME}, adapter_selection={REQUIRED_ADAPTER_SELECTION}, and the fenced measurement method"
        ));
    }
    Ok(())
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
    if manifest.defaults.width != REQUIRED_WIDTH
        || manifest.defaults.height != REQUIRED_HEIGHT
        || manifest.defaults.frame != REQUIRED_FRAME
        || manifest.defaults.adapter_selection != REQUIRED_ADAPTER_SELECTION
    {
        return Err(format!(
            "manifest defaults must be {REQUIRED_WIDTH}x{REQUIRED_HEIGHT}, frame {REQUIRED_FRAME}, adapter_selection={REQUIRED_ADAPTER_SELECTION}"
        ));
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
            let request =
                RunRequest::for_scene(scene, &manifest.defaults, mode, Measurement::Timing);
            let mut baseline_medians = Vec::with_capacity(SAMPLE_COUNT);
            let mut candidate_medians = Vec::with_capacity(SAMPLE_COUNT);
            let mut sample_orders = Vec::with_capacity(SAMPLE_COUNT);
            let mut structural = None;
            let mut selected_adapter = None;
            let scene_index = scenes.len();

            for sample in 0..SAMPLE_COUNT {
                let baseline_first = (scene_index + sample) % 2 == 0;
                let (baseline_response, candidate_response) = if baseline_first {
                    (baseline.run(&request)?, candidate.run(&request)?)
                } else {
                    let candidate_response = candidate.run(&request)?;
                    let baseline_response = baseline.run(&request)?;
                    (baseline_response, candidate_response)
                };
                sample_orders.push(if baseline_first {
                    SampleOrder::CppThenCandidate
                } else {
                    SampleOrder::CandidateThenCpp
                });

                validate_response("baseline", &id, sample, &request, &baseline_response)?;
                validate_adapter(
                    &id,
                    "baseline",
                    sample,
                    selected_adapter.as_ref(),
                    &baseline_response.selected_adapter,
                )?;
                selected_adapter = Some(baseline_response.selected_adapter.clone());
                let baseline_structural = StructuralMetrics::from(&baseline_response);
                validate_structural(&id, "baseline", sample, structural, baseline_structural)?;
                structural = Some(baseline_structural);
                baseline_medians.push(baseline_response.measured_frame_median_ns);

                validate_response("candidate", &id, sample, &request, &candidate_response)?;
                validate_adapter(
                    &id,
                    "candidate",
                    sample,
                    selected_adapter.as_ref(),
                    &candidate_response.selected_adapter,
                )?;
                let candidate_structural = StructuralMetrics::from(&candidate_response);
                validate_structural(&id, "candidate", sample, structural, candidate_structural)?;
                candidate_medians.push(candidate_response.measured_frame_median_ns);
            }

            let baseline = TimingSummary::from_samples(baseline_medians)?;
            let candidate = TimingSummary::from_samples(candidate_medians)?;
            let control_selected_pair =
                ControlSelectedPair::from_samples(&baseline, &candidate, &sample_orders)?;
            scenes.push(SceneReport {
                id: id.clone(),
                stream: scene.stream.clone(),
                frame: manifest.defaults.frame,
                mode,
                width: manifest.defaults.width,
                height: manifest.defaults.height,
                adapter_selection: manifest.defaults.adapter_selection.clone(),
                selected_adapter: selected_adapter
                    .ok_or_else(|| format!("scene {id} did not report an adapter"))?,
                timing: TimingMethod::required(),
                baseline,
                candidate,
                sample_orders,
                control_selected_pair,
                structural: structural
                    .ok_or_else(|| format!("scene {id} did not produce structural metrics"))?,
            });
        }
    }

    let aggregate = Aggregate::from_scenes(&scenes)?;
    Ok(Report {
        schema: REPORT_SCHEMA,
        runner_protocol: RUNNER_PROTOCOL,
        estimator: ESTIMATOR,
        pair_order: PAIR_ORDER,
        release: true,
        profile: "release",
        debug: false,
        samples_per_runner: SAMPLE_COUNT,
        manifest_schema: MANIFEST_SCHEMA,
        scenes,
        aggregate,
    })
}

#[derive(Clone, Debug, Serialize)]
pub struct ControlSelectedPair {
    pub sample_index: usize,
    pub cpp_control_ns: u64,
    pub candidate_ns: u64,
    pub candidate_over_cpp: f64,
}

impl ControlSelectedPair {
    fn from_samples(
        baseline: &TimingSummary,
        candidate: &TimingSummary,
        sample_orders: &[SampleOrder],
    ) -> Result<Self, String> {
        if baseline.sample_medians_ns.len() != candidate.sample_medians_ns.len()
            || baseline.sample_medians_ns.len() != sample_orders.len()
        {
            return Err("paired timing vectors have different lengths".to_owned());
        }
        let (index, &cpp_control_ns) = baseline
            .sample_medians_ns
            .iter()
            .enumerate()
            .min_by_key(|(_, timing)| *timing)
            .ok_or_else(|| "paired timing vectors are empty".to_owned())?;
        let candidate_ns = candidate.sample_medians_ns[index];
        Ok(Self {
            sample_index: index + 1,
            cpp_control_ns,
            candidate_ns,
            candidate_over_cpp: ratio(candidate_ns, cpp_control_ns)?,
        })
    }
}

pub(crate) fn validate_response(
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
        || response.adapter_selection != request.adapter_selection
        || response.measurement != request.measurement
        || response.timing != request.timing;
    if mismatch {
        return Err(format!(
            "scene {} sample {} {runner} response did not echo the fenced request",
            scene_id,
            sample + 1
        ));
    }
    if response.measured_frame_median_ns == 0 {
        return Err(format!(
            "scene {} sample {} {runner} reported zero measured_frame_median_ns",
            scene_id,
            sample + 1
        ));
    }
    Ok(())
}

pub(crate) fn validate_adapter(
    scene_id: &str,
    runner: &str,
    sample: usize,
    expected: Option<&AdapterIdentity>,
    actual: &AdapterIdentity,
) -> Result<(), String> {
    if [
        &actual.backend,
        &actual.name,
        &actual.vendor,
        &actual.device,
        &actual.driver,
    ]
    .iter()
    .any(|field| field.trim().is_empty())
    {
        return Err(format!(
            "scene {} sample {} {runner} reported an incomplete selected_adapter",
            scene_id,
            sample + 1
        ));
    }
    if let Some(expected) = expected {
        if actual != expected {
            return Err(format!(
                "scene {} sample {} {runner} selected a different physical adapter: expected {:?}, got {:?}",
                scene_id,
                sample + 1,
                expected,
                actual
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_structural(
    scene_id: &str,
    runner: &str,
    sample: usize,
    expected: Option<StructuralMetrics>,
    actual: StructuralMetrics,
) -> Result<(), String> {
    if let Some(expected) = expected {
        if actual != expected {
            return Err(format!(
                "scene {} sample {} {runner} structural mismatch: expected logical_flushes={} draws={} atomic_strategy_partitions={}, got logical_flushes={} draws={} atomic_strategy_partitions={}",
                scene_id,
                sample + 1,
                expected.logical_flushes,
                expected.draws,
                expected.atomic_strategy_partitions,
                actual.logical_flushes,
                actual.draws,
                actual.atomic_strategy_partitions,
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
    pub adapter_selection: String,
    pub selected_adapter: AdapterIdentity,
    pub timing: TimingMethod,
    pub baseline: TimingSummary,
    pub candidate: TimingSummary,
    pub sample_orders: Vec<SampleOrder>,
    pub control_selected_pair: ControlSelectedPair,
    pub structural: StructuralMetrics,
}

#[derive(Clone, Debug, Serialize)]
pub struct Aggregate {
    pub cpp_control_selected_ns_sum: u64,
    pub candidate_paired_ns_sum: u64,
    pub candidate_over_cpp: f64,
    pub worst_scene: WorstScene,
}

#[derive(Clone, Debug, Serialize)]
pub struct WorstScene {
    pub id: String,
    pub sample_index: usize,
    pub cpp_control_ns: u64,
    pub candidate_ns: u64,
    pub candidate_over_cpp: f64,
}

impl Aggregate {
    fn from_scenes(scenes: &[SceneReport]) -> Result<Self, String> {
        let cpp_control_selected_ns_sum = scenes.iter().try_fold(0_u64, |sum, scene| {
            sum.checked_add(scene.control_selected_pair.cpp_control_ns)
                .ok_or_else(|| "report C++ control timing overflow".to_owned())
        })?;
        let candidate_paired_ns_sum = scenes.iter().try_fold(0_u64, |sum, scene| {
            sum.checked_add(scene.control_selected_pair.candidate_ns)
                .ok_or_else(|| "report candidate timing overflow".to_owned())
        })?;
        let candidate_over_cpp = ratio(candidate_paired_ns_sum, cpp_control_selected_ns_sum)?;
        let mut scenes = scenes.iter();
        let mut worst = scenes
            .next()
            .ok_or_else(|| "report contains no scenes".to_owned())?;
        for scene in scenes {
            if scene.control_selected_pair.candidate_over_cpp
                > worst.control_selected_pair.candidate_over_cpp
            {
                worst = scene;
            }
        }
        Ok(Self {
            cpp_control_selected_ns_sum,
            candidate_paired_ns_sum,
            candidate_over_cpp,
            worst_scene: WorstScene {
                id: worst.id.clone(),
                sample_index: worst.control_selected_pair.sample_index,
                cpp_control_ns: worst.control_selected_pair.cpp_control_ns,
                candidate_ns: worst.control_selected_pair.candidate_ns,
                candidate_over_cpp: worst.control_selected_pair.candidate_over_cpp,
            },
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub schema: &'static str,
    pub runner_protocol: &'static str,
    pub estimator: &'static str,
    pub pair_order: &'static str,
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
        "# Rive Renderer Performance\n\nSchema: `rive-renderer-perf-v2`  \nEstimator: `cpp-control-min-paired-v1`  \nPair order: `counterbalanced-scene-sample-v1`  \nRunner protocol: `rive-renderer-perf-runner-v1`\n\n| scene | mode | C++ selected sample | C++ selected ns | paired candidate ns | paired ratio | baseline p50 ns | baseline p95 ns | baseline spread ns | candidate p50 ns | candidate p95 ns | candidate spread ns | logical flushes | draws | atomic strategy partitions |\n| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n",
    );
    for scene in &report.scenes {
        markdown.push_str(&format!(
            "| {} | {} | {} | {} | {} | {:.6} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            scene.id,
            mode_name(scene.mode),
            scene.control_selected_pair.sample_index,
            scene.control_selected_pair.cpp_control_ns,
            scene.control_selected_pair.candidate_ns,
            scene.control_selected_pair.candidate_over_cpp,
            scene.baseline.p50_ns,
            scene.baseline.p95_ns,
            scene.baseline.spread_ns,
            scene.candidate.p50_ns,
            scene.candidate.p95_ns,
            scene.candidate.spread_ns,
            scene.structural.logical_flushes,
            scene.structural.draws,
            scene.structural.atomic_strategy_partitions,
        ));
    }
    markdown.push_str(&format!(
        "\nAggregate control-selected pairs: C++ {} ns, candidate {} ns, ratio {:.6}. Worst scene: {} sample {} ({:.6}).\n",
        report.aggregate.cpp_control_selected_ns_sum,
        report.aggregate.candidate_paired_ns_sum,
        report.aggregate.candidate_over_cpp,
        report.aggregate.worst_scene.id,
        report.aggregate.worst_scene.sample_index,
        report.aggregate.worst_scene.candidate_over_cpp,
    ));
    markdown
}

pub fn check_threshold(report: &Report, max_ratio: f64) -> Result<(), String> {
    if !max_ratio.is_finite() || max_ratio <= 0.0 {
        return Err("--max-ratio must be a finite value greater than zero".to_owned());
    }
    if report.aggregate.worst_scene.candidate_over_cpp > max_ratio {
        return Err(format!(
            "renderer performance threshold failed: {} ratio {:.6} exceeds {:.6}",
            report.aggregate.worst_scene.id,
            report.aggregate.worst_scene.candidate_over_cpp,
            max_ratio
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

pub fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::ClockwiseAtomic => "clockwise-atomic",
        Mode::Msaa => "msaa",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::rc::Rc;

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

    struct RecordingRunner {
        label: &'static str,
        responses: VecDeque<RunnerResponse>,
        calls: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Runner for RecordingRunner {
        fn run(&mut self, _request: &RunRequest) -> Result<RunnerResponse, String> {
            self.calls.borrow_mut().push(self.label);
            self.responses
                .pop_front()
                .ok_or_else(|| "recording runner ran out of responses".to_owned())
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
            adapter_selection: defaults.adapter_selection.clone(),
            measurement: Measurement::Timing,
            selected_adapter: adapter_identity(),
            timing: TimingMethod::required(),
            measured_frame_median_ns: median_ns,
            logical_flushes: 3,
            draws: 11,
            atomic_strategy_partitions: 2,
            backend_work: BackendWorkMetrics::default(),
        }
    }

    fn adapter_identity() -> AdapterIdentity {
        AdapterIdentity {
            backend: "metal".to_owned(),
            name: "Test GPU".to_owned(),
            vendor: "Test Vendor".to_owned(),
            device: "Test Device".to_owned(),
            driver: "1.0".to_owned(),
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
    fn legacy_runner_output_defaults_new_clear_counters_only() {
        let manifest = manifest();
        let scene = &manifest.scene[0];
        let mut json =
            serde_json::to_value(response(scene, &manifest.defaults, manifest.modes[0], 10))
                .expect("response must serialize");
        let backend_work = json["backend_work"]
            .as_object_mut()
            .expect("backend work must be an object");
        backend_work.remove("buffer_clear_calls");
        backend_work.remove("buffer_clear_bytes");

        let parsed = parse_runner_response(&json.to_string()).expect("legacy response must parse");
        assert_eq!(parsed.backend_work.buffer_clear_calls, 0);
        assert_eq!(parsed.backend_work.buffer_clear_bytes, 0);
    }

    #[test]
    fn runner_output_missing_an_established_counter_is_rejected() {
        let manifest = manifest();
        let scene = &manifest.scene[0];
        let mut json =
            serde_json::to_value(response(scene, &manifest.defaults, manifest.modes[0], 10))
                .expect("response must serialize");
        json["backend_work"]
            .as_object_mut()
            .expect("backend work must be an object")
            .remove("gpu_draw_calls");

        let error = parse_runner_response(&json.to_string())
            .expect_err("missing established counter must fail");
        assert!(error.contains("gpu_draw_calls"));
    }

    #[test]
    fn fence_mismatch_is_rejected() {
        let manifest = manifest();
        let scene = &manifest.scene[0];
        let request = RunRequest::for_scene(
            scene,
            &manifest.defaults,
            manifest.modes[0],
            Measurement::Timing,
        );
        let mut response = response(scene, &manifest.defaults, manifest.modes[0], 10);
        response.debug = true;
        let error = validate_response("candidate", &scene.id, 0, &request, &response)
            .expect_err("debug mismatch must fail");
        assert!(error.contains("fenced request"));
    }

    #[test]
    fn timing_method_mismatch_is_rejected() {
        let manifest = manifest();
        let scene = &manifest.scene[0];
        let request = RunRequest::for_scene(
            scene,
            &manifest.defaults,
            manifest.modes[0],
            Measurement::Timing,
        );
        let mut response = response(scene, &manifest.defaults, manifest.modes[0], 10);
        response.timing.measured_frames -= 1;
        let error = validate_response("candidate", &scene.id, 0, &request, &response)
            .expect_err("timing method mismatch must fail");
        assert!(error.contains("fenced request"));
    }

    #[test]
    fn live_runner_request_rejects_a_relaxed_measurement_count() {
        let manifest = manifest();
        let mut request = RunRequest::for_scene(
            &manifest.scene[0],
            &manifest.defaults,
            manifest.modes[0],
            Measurement::Timing,
        );
        request.timing.measured_frames -= 1;
        let error = validate_runner_request(&request).expect_err("relaxed fence must fail");
        assert!(error.contains("fenced measurement method"));
    }

    #[test]
    fn physical_adapter_mismatch_is_rejected() {
        let manifest = manifest();
        let mut baseline = FakeRunner {
            responses: responses_for(&manifest, 10),
        };
        let mut candidate_responses = responses_for(&manifest, 12);
        candidate_responses[0].selected_adapter.device = "Other Device".to_owned();
        let mut candidate = FakeRunner {
            responses: candidate_responses,
        };
        let error = run_benchmark(&manifest, &mut baseline, &mut candidate)
            .expect_err("physical adapter mismatch must fail");
        assert!(error.contains("different physical adapter"));
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
    fn control_minimum_selects_the_candidate_from_the_same_sample() {
        let baseline =
            TimingSummary::from_samples(vec![16, 10, 15, 14, 13, 12, 11]).expect("seven samples");
        let candidate =
            TimingSummary::from_samples(vec![5, 30, 21, 19, 18, 17, 16]).expect("seven samples");
        let orders = vec![SampleOrder::CppThenCandidate; SAMPLE_COUNT];
        let selected =
            ControlSelectedPair::from_samples(&baseline, &candidate, &orders).expect("pair");
        assert_eq!(baseline.min_of_medians_ns, 10);
        assert_eq!(candidate.min_of_medians_ns, 5);
        assert_eq!(baseline.p50_ns, 13);
        assert_eq!(selected.sample_index, 2);
        assert_eq!(selected.cpp_control_ns, 10);
        assert_eq!(selected.candidate_ns, 30);
        assert_eq!(selected.candidate_over_cpp, 3.0);
    }

    #[test]
    fn control_selection_uses_the_first_minimum_on_ties() {
        let baseline =
            TimingSummary::from_samples(vec![10, 20, 10, 30, 40, 50, 60]).expect("seven samples");
        let candidate =
            TimingSummary::from_samples(vec![15, 1, 25, 1, 1, 1, 1]).expect("seven samples");
        let orders = vec![SampleOrder::CppThenCandidate; SAMPLE_COUNT];
        let selected =
            ControlSelectedPair::from_samples(&baseline, &candidate, &orders).expect("pair");
        assert_eq!(selected.sample_index, 1);
        assert_eq!(selected.candidate_ns, 15);
        assert_eq!(selected.candidate_over_cpp, 1.5);
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
        assert_eq!(report.schema, REPORT_SCHEMA);
        assert_eq!(report.estimator, ESTIMATOR);
        assert_eq!(report.pair_order, PAIR_ORDER);
        assert_eq!(report.scenes[0].id, "gm-CubicStroke-clockwise-atomic");
        assert_eq!(report.scenes[15].id, "gm-bug5099-msaa");
        let cpp_first = report
            .scenes
            .iter()
            .flat_map(|scene| &scene.sample_orders)
            .filter(|order| **order == SampleOrder::CppThenCandidate)
            .count();
        let candidate_first = report.scenes.len() * SAMPLE_COUNT - cpp_first;
        assert_eq!(cpp_first, candidate_first);
        assert!(
            json.find(&report.scenes[0].id).unwrap() < json.find(&report.scenes[15].id).unwrap()
        );
        assert!(
            markdown.find(&report.scenes[0].id).unwrap()
                < markdown.find(&report.scenes[15].id).unwrap()
        );
    }

    #[test]
    fn runner_execution_order_is_counterbalanced_and_matches_the_report() {
        let manifest = manifest();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut baseline = RecordingRunner {
            label: "cpp",
            responses: responses_for(&manifest, 10),
            calls: Rc::clone(&calls),
        };
        let mut candidate = RecordingRunner {
            label: "candidate",
            responses: responses_for(&manifest, 12),
            calls: Rc::clone(&calls),
        };
        let report = run_benchmark(&manifest, &mut baseline, &mut candidate).expect("report");
        let calls = calls.borrow();
        assert_eq!(calls.len(), report.scenes.len() * SAMPLE_COUNT * 2);
        for (pair, expected) in calls
            .chunks_exact(2)
            .zip(report.scenes.iter().flat_map(|scene| &scene.sample_orders))
        {
            match expected {
                SampleOrder::CppThenCandidate => assert_eq!(pair, ["cpp", "candidate"]),
                SampleOrder::CandidateThenCpp => assert_eq!(pair, ["candidate", "cpp"]),
            }
        }
    }

    #[test]
    fn manifest_rejects_an_unapproved_scene_set() {
        let mut manifest = manifest();
        manifest.scene[0].id = "something-else".to_owned();
        let error = validate_manifest(&manifest, Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect_err("fixed scene list must not drift");
        assert!(error.contains("scene 1"));
    }

    #[test]
    fn manifest_rejects_unpinned_defaults() {
        let mut manifest = manifest();
        manifest.defaults.width = 1;
        let error = validate_manifest(&manifest, Path::new(env!("CARGO_MANIFEST_DIR")))
            .expect_err("benchmark defaults must not drift");
        assert!(error.contains("manifest defaults must be"));
    }

    #[cfg(unix)]
    #[test]
    fn subprocess_runner_exercises_the_wire_contract() {
        let manifest = manifest();
        let scene = &manifest.scene[0];
        let mode = manifest.modes[0];
        let request = RunRequest::for_scene(scene, &manifest.defaults, mode, Measurement::Timing);
        let expected_response = response(scene, &manifest.defaults, mode, 1234);
        let unique = format!(
            "rive-renderer-perf-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let directory = std::env::temp_dir().join(unique);
        std::fs::create_dir(&directory).unwrap();
        let script = directory.join("runner.sh");
        let request_capture = directory.join("request.json");
        let response_json = serde_json::to_string(&expected_response).unwrap();
        std::fs::write(
            &script,
            format!(
                "#!/bin/sh\nset -eu\ntest \"$1\" = \"--renderer-perf-protocol\"\ntest \"$2\" = \"{RUNNER_PROTOCOL}\"\nIFS= read -r request\nprintf '%s\\n' \"$request\" > '{}'\nprintf '%s\\n' '{}'\n",
                request_capture.display(),
                response_json
            ),
        )
        .unwrap();
        let mut permissions = std::fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).unwrap();

        let mut runner = SubprocessRunner::new(script, directory.clone());
        let actual_response = runner.run(&request).expect("subprocess wire round trip");
        assert_eq!(actual_response.timing, TimingMethod::required());
        assert_eq!(actual_response.selected_adapter, adapter_identity());
        let captured: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(request_capture).unwrap()).unwrap();
        assert_eq!(captured["protocol"], RUNNER_PROTOCOL);
        assert_eq!(captured["timing"]["warmup_frames"], WARMUP_FRAMES);
        assert_eq!(captured["timing"]["measured_frames"], MEASURED_FRAMES);
        assert_eq!(
            captured["timing"]["gpu_completion"],
            "wait-for-completion-each-frame"
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn subprocess_runner_reports_failure_stderr() {
        let manifest = manifest();
        let request = RunRequest::for_scene(
            &manifest.scene[0],
            &manifest.defaults,
            manifest.modes[0],
            Measurement::Timing,
        );
        let unique = format!("rive-renderer-perf-fail-{}", std::process::id());
        let directory = std::env::temp_dir().join(unique);
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir(&directory).unwrap();
        let script = directory.join("runner.sh");
        std::fs::write(&script, "#!/bin/sh\necho deliberate-failure >&2\nexit 7\n").unwrap();
        let mut permissions = std::fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).unwrap();

        let mut runner = SubprocessRunner::new(script, directory.clone());
        let error = runner
            .run(&request)
            .expect_err("runner failure must propagate");
        assert!(error.contains("deliberate-failure"));
        std::fs::remove_dir_all(directory).unwrap();
    }
}
