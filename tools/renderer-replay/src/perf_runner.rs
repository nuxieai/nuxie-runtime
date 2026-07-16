use nuxie_render_stream::RenderStream;
use perf_compare::renderer_perf::{
    validate_runner_request, AdapterIdentity, BackendWorkMetrics, Measurement, Mode, RunRequest,
    RunnerResponse, StructuralMetrics, RUNNER_PROTOCOL,
};
use std::fs;
use std::io;
use std::time::Instant;

#[derive(Clone, Copy)]
pub enum BackendKind {
    RustWgpu,
    #[cfg(feature = "perf-dawn")]
    CppDawn,
}

pub fn run(kind: BackendKind) -> Result<(), String> {
    validate_arguments()?;
    if cfg!(debug_assertions) {
        return Err("renderer perf runners must be built with --release".to_owned());
    }

    let request: RunRequest = serde_json::from_reader(io::stdin().lock())
        .map_err(|error| format!("invalid runner request JSON: {error}"))?;
    validate_request(&request)?;
    if request.measurement == Measurement::Counters && !cfg!(feature = "perf-counters") {
        return Err("counter requests require the perf-counters feature".to_owned());
    }
    let stream = RenderStream::parse(
        &fs::read_to_string(&request.stream)
            .map_err(|error| format!("failed to read {}: {error}", request.stream))?,
    )
    .map_err(|error| format!("failed to parse {}: {error}", request.stream))?;
    if request.frame as usize >= stream.frames.len() {
        return Err(format!(
            "{} has no requested frame {}",
            request.stream, request.frame
        ));
    }
    let clear = stream.clear_color.unwrap_or(0);
    let native_adapter = nuxie_renderer_ffi::metal_adapter_identity()
        .map_err(|error| format!("Metal adapter identity failed: {error}"))?;
    let selected_adapter = AdapterIdentity {
        backend: "metal".to_owned(),
        name: native_adapter.name,
        vendor: native_adapter.vendor,
        device: native_adapter.device,
        driver: native_adapter.driver,
    };
    let mut backend: Box<dyn LiveBackend> = match kind {
        BackendKind::RustWgpu => Box::new(RustBackend::new(&request, &selected_adapter)?),
        #[cfg(feature = "perf-dawn")]
        BackendKind::CppDawn => Box::new(CppBackend::new(&request, &selected_adapter)?),
    };

    let total_frames = request
        .timing
        .warmup_frames
        .checked_add(request.timing.measured_frames)
        .ok_or_else(|| "runner frame count overflow".to_owned())?;
    let mut measured = Vec::with_capacity(request.timing.measured_frames as usize);
    let mut expected_structural = None;
    let mut expected_work = None;
    for frame_number in 0..total_frames {
        let start = Instant::now();
        let metrics = backend.render(
            &stream,
            request.frame as usize,
            clear,
            request.measurement == Measurement::Counters,
        )?;
        let elapsed_ns = u64::try_from(start.elapsed().as_nanos())
            .unwrap_or(u64::MAX)
            .max(1);
        if let Some(expected) = expected_structural {
            if metrics.structural != expected {
                return Err(format!(
                    "frame {} structural metrics changed from {:?} to {:?}",
                    frame_number + 1,
                    expected,
                    metrics.structural
                ));
            }
        } else {
            if metrics.structural.logical_flushes == 0 {
                return Err("renderer reported zero logical flushes".to_owned());
            }
            expected_structural = Some(metrics.structural);
        }
        if frame_number >= request.timing.warmup_frames {
            if request.measurement == Measurement::Counters {
                if let Some(expected) = expected_work {
                    if metrics.backend_work != expected {
                        return Err(format!(
                            "frame {} backend work changed from {:?} to {:?}",
                            frame_number + 1,
                            expected,
                            metrics.backend_work
                        ));
                    }
                } else {
                    expected_work = Some(metrics.backend_work);
                }
            }
            measured.push(elapsed_ns);
        }
    }
    let measured_frame_median_ns = lower_median(&mut measured)?;
    let metrics = expected_structural.ok_or_else(|| "runner rendered no frames".to_owned())?;
    let response = RunnerResponse {
        protocol: request.protocol,
        release: request.release,
        profile: request.profile,
        debug: request.debug,
        stream: request.stream,
        frame: request.frame,
        mode: request.mode,
        width: request.width,
        height: request.height,
        adapter_selection: request.adapter_selection,
        measurement: request.measurement,
        selected_adapter,
        timing: request.timing,
        measured_frame_median_ns,
        logical_flushes: metrics.logical_flushes,
        draws: metrics.draws,
        atomic_strategy_partitions: metrics.atomic_strategy_partitions,
        backend_work: expected_work.unwrap_or_default(),
    };
    serde_json::to_writer(io::stdout().lock(), &response)
        .map_err(|error| format!("failed to encode runner response: {error}"))?;
    println!();
    Ok(())
}

fn validate_request(request: &RunRequest) -> Result<(), String> {
    #[cfg(feature = "perf-diagnostics")]
    if std::env::var_os("RIVE_RENDERER_PERF_DIAGNOSTIC_SIZE").is_some() {
        let mut fenced = request.clone();
        fenced.width = 1024;
        fenced.height = 1024;
        return validate_runner_request(&fenced);
    }
    validate_runner_request(request)
}

fn validate_arguments() -> Result<(), String> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments
        != [
            "--renderer-perf-protocol".to_owned(),
            RUNNER_PROTOCOL.to_owned(),
        ]
    {
        return Err(format!(
            "usage: <runner> --renderer-perf-protocol {RUNNER_PROTOCOL}"
        ));
    }
    Ok(())
}

fn lower_median(values: &mut [u64]) -> Result<u64, String> {
    if values.is_empty() {
        return Err("cannot calculate a median without measured frames".to_owned());
    }
    values.sort_unstable();
    Ok(values[(values.len() - 1) / 2])
}

trait LiveBackend {
    fn render(
        &mut self,
        stream: &RenderStream,
        frame: usize,
        clear: u32,
        collect_work_metrics: bool,
    ) -> Result<LiveMetrics, String>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LiveMetrics {
    structural: StructuralMetrics,
    backend_work: BackendWorkMetrics,
}

struct RustBackend {
    factory: nuxie_renderer::WgpuFactory,
}

impl RustBackend {
    fn new(request: &RunRequest, expected: &AdapterIdentity) -> Result<Self, String> {
        let mode = match request.mode {
            Mode::ClockwiseAtomic => nuxie_renderer::RenderMode::ClockwiseAtomic,
            Mode::Msaa => nuxie_renderer::RenderMode::Msaa,
        };
        let factory =
            nuxie_renderer::WgpuFactory::new_with_mode(request.width, request.height, mode)
                .map_err(|error| format!("failed to create Rust wgpu renderer: {error}"))?;
        let actual = factory.adapter_info();
        if actual.backend != expected.backend || actual.name != expected.name {
            return Err(format!(
                "wgpu selected {} adapter {:?}, native Metal selected {:?}",
                actual.backend, actual.name, expected.name
            ));
        }
        Ok(Self { factory })
    }
}

impl LiveBackend for RustBackend {
    fn render(
        &mut self,
        stream: &RenderStream,
        frame_index: usize,
        clear: u32,
        collect_work_metrics: bool,
    ) -> Result<LiveMetrics, String> {
        let mut frame = self
            .factory
            .begin_frame_for_benchmark(clear, collect_work_metrics);
        stream
            .replay_frame(frame_index, &mut self.factory, &mut frame)
            .map_err(|error| format!("Rust stream replay failed: {error}"))?;
        let metrics = frame
            .finish_for_benchmark()
            .map_err(|error| format!("Rust renderer failed: {error}"))?;
        Ok(LiveMetrics {
            structural: StructuralMetrics {
                logical_flushes: metrics.logical_flushes,
                draws: metrics.draw_calls,
                atomic_strategy_partitions: metrics.atomic_strategy_partitions,
            },
            backend_work: BackendWorkMetrics {
                command_encoders: metrics.backend_work.command_encoders,
                render_passes: metrics.backend_work.render_passes,
                bind_groups_created: metrics.backend_work.bind_groups_created,
                bind_group_sets: metrics.backend_work.bind_group_sets,
                texture_bindings: metrics.backend_work.texture_bindings,
                buffer_upload_calls: metrics.backend_work.buffer_upload_calls,
                buffer_upload_bytes: metrics.backend_work.buffer_upload_bytes,
                texture_upload_calls: metrics.backend_work.texture_upload_calls,
                texture_upload_bytes: metrics.backend_work.texture_upload_bytes,
                queue_submissions: metrics.backend_work.queue_submissions,
                gpu_draw_calls: metrics.backend_work.gpu_draw_calls,
                gpu_draw_instances: metrics.backend_work.gpu_draw_instances,
                tessellation_spans: metrics.backend_work.tessellation_spans,
                path_patches: metrics.backend_work.path_patches,
            },
        })
    }
}

#[cfg(feature = "perf-dawn")]
struct CppBackend {
    factory: nuxie_renderer_ffi::FfiFactory,
    mode: nuxie_renderer_ffi::FfiRenderMode,
}

#[cfg(feature = "perf-dawn")]
impl CppBackend {
    fn new(request: &RunRequest, expected: &AdapterIdentity) -> Result<Self, String> {
        let mode = match request.mode {
            Mode::ClockwiseAtomic => nuxie_renderer_ffi::FfiRenderMode::ClockwiseAtomic,
            Mode::Msaa => nuxie_renderer_ffi::FfiRenderMode::Msaa,
        };
        let factory = nuxie_renderer_ffi::FfiFactory::new_dawn(request.width, request.height)
            .map_err(|error| format!("failed to create C++ Dawn renderer: {error}"))?;
        let actual_name = factory
            .adapter_name()
            .map_err(|error| format!("failed to query C++ Dawn adapter: {error}"))?;
        if actual_name != expected.name {
            return Err(format!(
                "C++ Dawn selected adapter {:?}, native Metal selected {:?}",
                actual_name, expected.name
            ));
        }
        Ok(Self { factory, mode })
    }
}

#[cfg(feature = "perf-dawn")]
impl LiveBackend for CppBackend {
    fn render(
        &mut self,
        stream: &RenderStream,
        frame_index: usize,
        clear: u32,
        collect_work_metrics: bool,
    ) -> Result<LiveMetrics, String> {
        let mut frame = self
            .factory
            .begin_frame_with_mode_and_metrics(clear, self.mode, collect_work_metrics)
            .map_err(|error| format!("failed to begin C++ Dawn frame: {error}"))?;
        stream
            .replay_frame(frame_index, &mut self.factory, &mut frame)
            .map_err(|error| format!("C++ stream replay failed: {error}"))?;
        let metrics = frame
            .end_with_metrics()
            .map_err(|error| format!("C++ Dawn renderer failed: {error}"))?;
        Ok(LiveMetrics {
            structural: StructuralMetrics {
                logical_flushes: metrics.logical_flushes,
                draws: metrics.draw_calls,
                atomic_strategy_partitions: metrics.atomic_strategy_partitions,
            },
            backend_work: BackendWorkMetrics {
                command_encoders: metrics.backend_work.command_encoders,
                render_passes: metrics.backend_work.render_passes,
                bind_groups_created: metrics.backend_work.bind_groups_created,
                bind_group_sets: metrics.backend_work.bind_group_sets,
                texture_bindings: metrics.backend_work.texture_bindings,
                buffer_upload_calls: metrics.backend_work.buffer_upload_calls,
                buffer_upload_bytes: metrics.backend_work.buffer_upload_bytes,
                texture_upload_calls: metrics.backend_work.texture_upload_calls,
                texture_upload_bytes: metrics.backend_work.texture_upload_bytes,
                queue_submissions: metrics.backend_work.queue_submissions,
                gpu_draw_calls: metrics.backend_work.gpu_draw_calls,
                gpu_draw_instances: metrics.backend_work.gpu_draw_instances,
                tessellation_spans: metrics.backend_work.tessellation_spans,
                path_patches: metrics.backend_work.path_patches,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::lower_median;

    #[test]
    fn uses_the_lower_median_for_an_even_sample_count() {
        let mut values = [9, 2, 7, 4];
        assert_eq!(lower_median(&mut values).unwrap(), 4);
    }

    #[test]
    fn rejects_an_empty_measurement_set() {
        assert!(lower_median(&mut []).is_err());
    }
}
