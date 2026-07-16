use crate::renderer_perf::{
    AdapterIdentity, BackendWorkMetrics, MANIFEST_SCHEMA, Manifest, Measurement, Mode,
    RUNNER_PROTOCOL, RunRequest, Runner, StructuralMetrics, mode_name, validate_adapter,
    validate_response, validate_structural,
};
use serde::Serialize;
use std::cmp::Ordering;

pub const COUNTER_REPORT_SCHEMA: &str = "rive-renderer-perf-counters-v1";

#[derive(Clone, Debug, Serialize)]
pub struct CounterSceneReport {
    pub id: String,
    pub stream: String,
    pub mode: Mode,
    pub selected_adapter: AdapterIdentity,
    pub structural: StructuralMetrics,
    pub baseline: BackendWorkMetrics,
    pub candidate: BackendWorkMetrics,
    pub baseline_directional_ns: u64,
    pub candidate_directional_ns: u64,
    pub directional_ratio: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CounterExcess {
    pub scene: String,
    pub counter: &'static str,
    pub baseline: u64,
    pub candidate: u64,
    pub absolute_excess: u64,
    pub ratio: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CounterReport {
    pub schema: &'static str,
    pub runner_protocol: &'static str,
    pub manifest_schema: &'static str,
    pub measurement: Measurement,
    pub warmup_frames: u32,
    pub measured_frames: u32,
    pub scenes: Vec<CounterSceneReport>,
    pub ranked_excesses: Vec<CounterExcess>,
}

pub fn run_counter_compare(
    manifest: &Manifest,
    baseline: &mut dyn Runner,
    candidate: &mut dyn Runner,
) -> Result<CounterReport, String> {
    let mut scenes = Vec::with_capacity(manifest.scene.len() * manifest.modes.len());
    for scene in &manifest.scene {
        for &mode in &manifest.modes {
            let id = format!("{}-{}", scene.id, mode_name(mode));
            let request =
                RunRequest::for_scene(scene, &manifest.defaults, mode, Measurement::Counters);
            let baseline_response = baseline.run(&request)?;
            validate_response("baseline", &id, 0, &request, &baseline_response)?;
            validate_adapter(
                &id,
                "baseline",
                0,
                None,
                &baseline_response.selected_adapter,
            )?;
            validate_work(&id, "baseline", baseline_response.backend_work)?;

            let candidate_response = candidate.run(&request)?;
            validate_response("candidate", &id, 0, &request, &candidate_response)?;
            validate_adapter(
                &id,
                "candidate",
                0,
                Some(&baseline_response.selected_adapter),
                &candidate_response.selected_adapter,
            )?;
            validate_work(&id, "candidate", candidate_response.backend_work)?;

            let structural = StructuralMetrics::from(&baseline_response);
            validate_structural(
                &id,
                "candidate",
                0,
                Some(structural),
                StructuralMetrics::from(&candidate_response),
            )?;
            let directional_ratio = candidate_response.measured_frame_median_ns as f64
                / baseline_response.measured_frame_median_ns as f64;
            scenes.push(CounterSceneReport {
                id,
                stream: scene.stream.clone(),
                mode,
                selected_adapter: baseline_response.selected_adapter,
                structural,
                baseline: baseline_response.backend_work,
                candidate: candidate_response.backend_work,
                baseline_directional_ns: baseline_response.measured_frame_median_ns,
                candidate_directional_ns: candidate_response.measured_frame_median_ns,
                directional_ratio,
            });
        }
    }
    let ranked_excesses = rank_excesses(&scenes);
    Ok(CounterReport {
        schema: COUNTER_REPORT_SCHEMA,
        runner_protocol: RUNNER_PROTOCOL,
        manifest_schema: MANIFEST_SCHEMA,
        measurement: Measurement::Counters,
        warmup_frames: crate::renderer_perf::COUNTER_WARMUP_FRAMES,
        measured_frames: crate::renderer_perf::COUNTER_MEASURED_FRAMES,
        scenes,
        ranked_excesses,
    })
}

fn validate_work(scene: &str, runner: &str, work: BackendWorkMetrics) -> Result<(), String> {
    if work.command_encoders == 0 || work.render_passes == 0 || work.queue_submissions == 0 {
        return Err(format!(
            "scene {scene} {runner} did not report live backend work: command_encoders={} render_passes={} queue_submissions={}",
            work.command_encoders, work.render_passes, work.queue_submissions
        ));
    }
    Ok(())
}

fn counters(
    baseline: BackendWorkMetrics,
    candidate: BackendWorkMetrics,
) -> [(&'static str, u64, u64); 14] {
    [
        (
            "command_encoders",
            baseline.command_encoders,
            candidate.command_encoders,
        ),
        (
            "render_passes",
            baseline.render_passes,
            candidate.render_passes,
        ),
        (
            "bind_groups_created",
            baseline.bind_groups_created,
            candidate.bind_groups_created,
        ),
        (
            "bind_group_sets",
            baseline.bind_group_sets,
            candidate.bind_group_sets,
        ),
        (
            "texture_bindings",
            baseline.texture_bindings,
            candidate.texture_bindings,
        ),
        (
            "buffer_upload_calls",
            baseline.buffer_upload_calls,
            candidate.buffer_upload_calls,
        ),
        (
            "buffer_upload_bytes",
            baseline.buffer_upload_bytes,
            candidate.buffer_upload_bytes,
        ),
        (
            "texture_upload_calls",
            baseline.texture_upload_calls,
            candidate.texture_upload_calls,
        ),
        (
            "texture_upload_bytes",
            baseline.texture_upload_bytes,
            candidate.texture_upload_bytes,
        ),
        (
            "queue_submissions",
            baseline.queue_submissions,
            candidate.queue_submissions,
        ),
        (
            "gpu_draw_calls",
            baseline.gpu_draw_calls,
            candidate.gpu_draw_calls,
        ),
        (
            "gpu_draw_instances",
            baseline.gpu_draw_instances,
            candidate.gpu_draw_instances,
        ),
        (
            "tessellation_spans",
            baseline.tessellation_spans,
            candidate.tessellation_spans,
        ),
        (
            "path_patches",
            baseline.path_patches,
            candidate.path_patches,
        ),
    ]
}

fn rank_excesses(scenes: &[CounterSceneReport]) -> Vec<CounterExcess> {
    let mut excesses = scenes
        .iter()
        .flat_map(|scene| {
            counters(scene.baseline, scene.candidate)
                .into_iter()
                .filter(|(_, baseline, candidate)| candidate > baseline)
                .map(|(counter, baseline, candidate)| CounterExcess {
                    scene: scene.id.clone(),
                    counter,
                    baseline,
                    candidate,
                    absolute_excess: candidate - baseline,
                    ratio: (baseline != 0).then(|| candidate as f64 / baseline as f64),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    excesses.sort_by(|left, right| match (left.ratio, right.ratio) {
        (None, None) => right.absolute_excess.cmp(&left.absolute_excess),
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(left_ratio), Some(right_ratio)) => right_ratio
            .total_cmp(&left_ratio)
            .then_with(|| right.absolute_excess.cmp(&left.absolute_excess)),
    });
    excesses
}

pub fn render_json(report: &CounterReport) -> Result<String, String> {
    serde_json::to_string_pretty(report)
        .map(|json| format!("{json}\n"))
        .map_err(|error| format!("failed to render counter JSON: {error}"))
}

pub fn render_markdown(report: &CounterReport) -> String {
    let mut markdown = format!(
        "# Rive Renderer Work Counters\n\nSchema: `{}`  \nProtocol: `{}`  \nCapture: {} warmup + {} measured frame; timing is directional only.\n\n## Ranked Candidate Excess\n\n| rank | scene | counter | C++ Dawn | Rust wgpu | excess | ratio |\n| ---: | --- | --- | ---: | ---: | ---: | ---: |\n",
        report.schema, report.runner_protocol, report.warmup_frames, report.measured_frames,
    );
    if report.ranked_excesses.is_empty() {
        markdown.push_str("| 1 | none | none | 0 | 0 | 0 | 1.000 |\n");
    } else {
        for (index, excess) in report.ranked_excesses.iter().enumerate() {
            let ratio = excess
                .ratio
                .map(|value| format!("{value:.3}"))
                .unwrap_or_else(|| "new".to_owned());
            markdown.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} | {} |\n",
                index + 1,
                excess.scene,
                excess.counter,
                excess.baseline,
                excess.candidate,
                excess.absolute_excess,
                ratio,
            ));
        }
    }
    markdown.push_str(
        "\n## Directional Snapshot\n\n| scene | C++ Dawn ns | Rust wgpu ns | ratio |\n| --- | ---: | ---: | ---: |\n",
    );
    for scene in &report.scenes {
        markdown.push_str(&format!(
            "| {} | {} | {} | {:.3} |\n",
            scene.id,
            scene.baseline_directional_ns,
            scene.candidate_directional_ns,
            scene.directional_ratio,
        ));
    }
    markdown
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scene(baseline: BackendWorkMetrics, candidate: BackendWorkMetrics) -> CounterSceneReport {
        CounterSceneReport {
            id: "scene-msaa".to_owned(),
            stream: "scene.rive-stream".to_owned(),
            mode: Mode::Msaa,
            selected_adapter: AdapterIdentity {
                backend: "metal".to_owned(),
                name: "GPU".to_owned(),
                vendor: "vendor".to_owned(),
                device: "device".to_owned(),
                driver: "driver".to_owned(),
            },
            structural: StructuralMetrics {
                logical_flushes: 1,
                draws: 1,
                atomic_strategy_partitions: 0,
            },
            baseline,
            candidate,
            baseline_directional_ns: 10,
            candidate_directional_ns: 20,
            directional_ratio: 2.0,
        }
    }

    #[test]
    fn ranks_new_work_before_finite_ratios() {
        let baseline = BackendWorkMetrics {
            render_passes: 2,
            ..BackendWorkMetrics::default()
        };
        let candidate = BackendWorkMetrics {
            render_passes: 4,
            bind_groups_created: 1,
            ..BackendWorkMetrics::default()
        };
        let ranked = rank_excesses(&[scene(baseline, candidate)]);
        assert_eq!(ranked[0].counter, "bind_groups_created");
        assert_eq!(ranked[0].ratio, None);
        assert_eq!(ranked[1].counter, "render_passes");
        assert_eq!(ranked[1].ratio, Some(2.0));
    }
}
