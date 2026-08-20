//! Background Metal shader compilation corresponding to pinned upstream
//! `renderer/src/metal/background_shader_compiler.h:17-61` and
//! `renderer/src/metal/background_shader_compiler.mm:28-93,277-344` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
//!
//! Queue ordering and shutdown intentionally follow Objective-C++: the worker
//! starts on the first push, pending jobs are FIFO, and completed jobs are
//! popped LIFO. Once the worker observes `should_quit`, it exits before taking
//! another pending job; no stronger ordering is implied while Drop races to
//! acquire the queue mutex. Rust exposes compilation and planning failures as
//! typed results instead of storing a nil library and asserting.

use super::shader_compile_plan::{
    build_shader_compile_plan, ApplePlatform, BackgroundCompileJob, BackgroundCompilePlanError,
    MacroDefinition, MetalFeatures, SynthesizedFailureType,
};
use std::any::Any;
use std::collections::VecDeque;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};

/// Metal language version selected by the pinned platform conditionals.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MetalLanguageVersion {
    Version2_2,
    Version2_3,
}

/// The observable compile-option values passed to `newLibraryWithSource`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MetalShaderCompileOptions {
    pub(crate) language_version: MetalLanguageVersion,
    pub(crate) fast_math_enabled: bool,
    /// Request invariance preservation when the selector is available on the
    /// running OS. The native adapter performs the availability check.
    pub(crate) preserve_invariance_when_available: bool,
}

/// One exact key/value pair for `MTLCompileOptions.preprocessorMacros`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MetalPreprocessorMacro {
    pub(crate) name: &'static str,
    pub(crate) value: &'static str,
}

impl From<MacroDefinition> for MetalPreprocessorMacro {
    fn from(definition: MacroDefinition) -> Self {
        Self {
            name: definition.name.metal_token(),
            value: definition.value.metal_value(),
        }
    }
}

/// Fully materialized input to the injected or native Metal compiler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MetalShaderCompileRequest {
    pub(crate) source: String,
    pub(crate) preprocessor_macros: Vec<MetalPreprocessorMacro>,
    pub(crate) options: MetalShaderCompileOptions,
}

/// Backend error before it is paired with the submitted source for diagnosis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MetalLibraryCompileFailure {
    pub(crate) localized_description: Option<String>,
}

impl MetalLibraryCompileFailure {
    pub(crate) fn new(localized_description: impl Into<String>) -> Self {
        Self {
            localized_description: Some(localized_description.into()),
        }
    }
}

/// Typed replacement for the upstream nil-library/assert failure path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BackgroundShaderCompileError {
    Plan(BackgroundCompilePlanError),
    SynthesizedShaderCompilation,
    /// The injected compiler unwound. This is terminal for the current job;
    /// the serial worker catches it and continues with later queued jobs.
    CompilerPanicked {
        message: Option<String>,
    },
    MetalCompilation {
        localized_description: Option<String>,
        source: String,
    },
}

impl BackgroundShaderCompileError {
    /// Format the same one-based, four-column source listing emitted upstream.
    pub(crate) fn numbered_source(&self) -> Option<String> {
        let Self::MetalCompilation { source, .. } = self else {
            return None;
        };
        let mut numbered = String::new();
        for (index, line) in source.lines().enumerate() {
            numbered.push_str(&format!("{:4}| {line}\n", index + 1));
        }
        Some(numbered)
    }
}

/// A completed job and either its retained library or typed failure.
pub(crate) struct FinishedBackgroundCompileJob<Library> {
    pub(crate) job: BackgroundCompileJob,
    pub(crate) result: Result<Library, BackgroundShaderCompileError>,
}

type Compiler<Library> = dyn Fn(&MetalShaderCompileRequest) -> Result<Library, MetalLibraryCompileFailure>
    + Send
    + 'static;

struct Shared<Library> {
    state: Mutex<State<Library>>,
    work_added: Condvar,
    work_finished: Condvar,
}

struct State<Library> {
    pending_jobs: VecDeque<BackgroundCompileJob>,
    finished_jobs: Vec<FinishedBackgroundCompileJob<Library>>,
    should_quit: bool,
}

/// Serial background compiler with the queue semantics of the pinned C++
/// implementation. The callback seam is a Rust observability adaptation used
/// by tests and by the native `newLibraryWithSource` adapter below.
pub(crate) struct BackgroundShaderCompiler<Library: Send + 'static> {
    metal_features: MetalFeatures,
    platform: ApplePlatform,
    shared: Arc<Shared<Library>>,
    compiler: Mutex<Option<Box<Compiler<Library>>>>,
    compiler_thread: Mutex<Option<JoinHandle<()>>>,
}

impl<Library: Send + 'static> BackgroundShaderCompiler<Library> {
    pub(crate) fn new(
        metal_features: MetalFeatures,
        platform: ApplePlatform,
        compiler: impl Fn(&MetalShaderCompileRequest) -> Result<Library, MetalLibraryCompileFailure>
            + Send
            + 'static,
    ) -> Self {
        Self {
            metal_features,
            platform,
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    pending_jobs: VecDeque::new(),
                    finished_jobs: Vec::new(),
                    should_quit: false,
                }),
                work_added: Condvar::new(),
                work_finished: Condvar::new(),
            }),
            compiler: Mutex::new(Some(Box::new(compiler))),
            compiler_thread: Mutex::new(None),
        }
    }

    /// Start the worker lazily and append to the FIFO pending queue.
    pub(crate) fn push_job(&self, job: BackgroundCompileJob) {
        let mut state = lock_recovering_poison(&self.shared.state);
        let mut compiler_thread = lock_recovering_poison(&self.compiler_thread);
        if compiler_thread.is_none() {
            let compiler = lock_recovering_poison(&self.compiler)
                .take()
                .expect("background compiler callback is present before thread start");
            let shared = Arc::clone(&self.shared);
            let metal_features = self.metal_features;
            let platform = self.platform;
            *compiler_thread = Some(thread::spawn(move || {
                thread_main(shared, metal_features, platform, compiler);
            }));
        }
        state.pending_jobs.push_back(job);
        drop(compiler_thread);
        drop(state);
        self.shared.work_added.notify_all();
    }

    /// Pop the newest finished job. With `wait == false`, an empty completion
    /// stack returns immediately; with `wait == true`, it blocks until a worker
    /// publishes a result.
    pub(crate) fn pop_finished_job(
        &self,
        wait: bool,
    ) -> Option<FinishedBackgroundCompileJob<Library>> {
        let mut state = lock_recovering_poison(&self.shared.state);
        while state.finished_jobs.is_empty() {
            if !wait {
                return None;
            }
            state = wait_recovering_poison(&self.shared.work_finished, state);
        }
        state.finished_jobs.pop()
    }

    /// Observable form of upstream `std::thread::joinable`, primarily for
    /// verifying that construction itself does not start the worker.
    pub(crate) fn is_started(&self) -> bool {
        lock_recovering_poison(&self.compiler_thread).is_some()
    }
}

impl<Library: Send + 'static> Drop for BackgroundShaderCompiler<Library> {
    fn drop(&mut self) {
        let thread = lock_recovering_poison(&self.compiler_thread).take();
        if let Some(thread) = thread {
            {
                let mut state = lock_recovering_poison(&self.shared.state);
                state.should_quit = true;
            }
            self.shared.work_added.notify_all();
            // Upstream always joins. Callback panics are converted to job
            // failures; joining still reclaims the worker after any unrelated
            // implementation panic.
            let _ = thread.join();
        }
    }
}

pub(crate) fn build_metal_shader_compile_request(
    job: BackgroundCompileJob,
    metal_features: MetalFeatures,
    platform: ApplePlatform,
) -> Result<MetalShaderCompileRequest, BackgroundCompilePlanError> {
    let plan = build_shader_compile_plan(job, metal_features, platform)?;
    Ok(MetalShaderCompileRequest {
        source: plan.materialize_source(),
        preprocessor_macros: plan.defines.into_iter().map(Into::into).collect(),
        options: MetalShaderCompileOptions {
            language_version: match platform {
                ApplePlatform::IosDevice { .. } | ApplePlatform::IosSimulator { .. } => {
                    MetalLanguageVersion::Version2_2
                }
                ApplePlatform::MacOs => MetalLanguageVersion::Version2_3,
            },
            fast_math_enabled: true,
            preserve_invariance_when_available: true,
        },
    })
}

fn thread_main<Library: Send + 'static>(
    shared: Arc<Shared<Library>>,
    metal_features: MetalFeatures,
    platform: ApplePlatform,
    compiler: Box<Compiler<Library>>,
) {
    loop {
        let job = {
            let mut state = lock_recovering_poison(&shared.state);
            while state.pending_jobs.is_empty() && !state.should_quit {
                state = wait_recovering_poison(&shared.work_added, state);
            }
            if state.should_quit {
                return;
            }
            state
                .pending_jobs
                .pop_front()
                .expect("worker only pops after observing a pending job")
        };

        let result = match build_metal_shader_compile_request(job, metal_features, platform) {
            Err(error) => Err(BackgroundShaderCompileError::Plan(error)),
            Ok(_request)
                if job.synthesized_failure_type == SynthesizedFailureType::ShaderCompilation =>
            {
                Err(BackgroundShaderCompileError::SynthesizedShaderCompilation)
            }
            Ok(request) => match catch_unwind(AssertUnwindSafe(|| compiler(&request))) {
                Ok(result) => {
                    result.map_err(|error| BackgroundShaderCompileError::MetalCompilation {
                        localized_description: error.localized_description,
                        source: request.source,
                    })
                }
                Err(payload) => Err(BackgroundShaderCompileError::CompilerPanicked {
                    message: panic_payload_message(payload.as_ref()),
                }),
            },
        };

        let mut state = lock_recovering_poison(&shared.state);
        state
            .finished_jobs
            .push(FinishedBackgroundCompileJob { job, result });
        drop(state);
        shared.work_finished.notify_all();
    }
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> Option<String> {
    payload
        .downcast_ref::<&'static str>()
        .map(|message| (*message).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
}

fn lock_recovering_poison<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn wait_recovering_poison<'a, T>(
    condition: &Condvar,
    guard: MutexGuard<'a, T>,
) -> MutexGuard<'a, T> {
    condition
        .wait(guard)
        .unwrap_or_else(|poison| poison.into_inner())
}

#[cfg(all(
    feature = "native-metal-experimental",
    any(target_os = "ios", target_os = "macos")
))]
mod native {
    use super::*;
    use objc2::rc::{autoreleasepool, Retained};
    use objc2::runtime::{NSObject, ProtocolObject};
    use objc2::{available, msg_send};
    use objc2_foundation::{NSMutableDictionary, NSString};
    use objc2_metal::{MTLCompileOptions, MTLDevice, MTLLanguageVersion, MTLLibrary};

    pub(crate) type NativeBackgroundShaderCompiler =
        BackgroundShaderCompiler<Retained<ProtocolObject<dyn MTLLibrary>>>;

    impl NativeBackgroundShaderCompiler {
        pub(crate) fn new_metal(
            device: Retained<ProtocolObject<dyn MTLDevice>>,
            metal_features: MetalFeatures,
            platform: ApplePlatform,
        ) -> Self {
            Self::new(metal_features, platform, move |request| {
                autoreleasepool(|_| compile_library(&device, request))
            })
        }
    }

    fn compile_library(
        device: &ProtocolObject<dyn MTLDevice>,
        request: &MetalShaderCompileRequest,
    ) -> Result<Retained<ProtocolObject<dyn MTLLibrary>>, MetalLibraryCompileFailure> {
        let options = MTLCompileOptions::new();
        options.setLanguageVersion(match request.options.language_version {
            MetalLanguageVersion::Version2_2 => MTLLanguageVersion::Version2_2,
            MetalLanguageVersion::Version2_3 => MTLLanguageVersion::Version2_3,
        });
        #[allow(deprecated)]
        options.setFastMathEnabled(request.options.fast_math_enabled);
        if request.options.preserve_invariance_when_available
            && available!(macos = 11.0, ios = 14.0, ..)
        {
            options.setPreserveInvariance(true);
        }

        let macros = NSMutableDictionary::<NSString, NSObject>::new();
        for definition in &request.preprocessor_macros {
            let key = NSString::from_str(definition.name);
            let value = NSString::from_str(definition.value);
            // SAFETY: NSString is an NSObject and implements NSCopying, both
            // retained locals outlive this synchronous message, and `macros`
            // is the mutable dictionary corresponding to upstream's compile
            // options. The selector returns void and copies the key.
            unsafe {
                let _: () = msg_send![&macros, setObject: &*value, forKey: &*key];
            }
        }
        // SAFETY: Every dictionary key and value is an NSString retained by
        // `macros`; Metal compile options accept that Objective-C dictionary
        // and retain/copy it for the synchronous compilation call below. This
        // is the same ownership interval as upstream's local macro dictionary.
        unsafe {
            options.setPreprocessorMacros(Some(&macros));
        }

        device
            .newLibraryWithSource_options_error(
                &NSString::from_str(&request.source),
                Some(&options),
            )
            .map_err(|error| MetalLibraryCompileFailure {
                localized_description: Some(error.localizedDescription().to_string()),
            })
    }
}

#[cfg(all(
    feature = "native-metal-experimental",
    any(target_os = "ios", target_os = "macos")
))]
pub(crate) use native::NativeBackgroundShaderCompiler;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::DrawType;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;

    const MAC: ApplePlatform = ApplePlatform::MacOs;
    const IOS_SIMULATOR: ApplePlatform = ApplePlatform::IosSimulator {
        host_is_arm64: true,
    };

    fn job(draw_type: DrawType) -> BackgroundCompileJob {
        BackgroundCompileJob::new(
            draw_type,
            0,
            super::super::shader_compile_plan::InterlockMode::RasterOrdering,
            0,
        )
    }

    #[test]
    fn materializes_exact_source_macros_and_platform_options() {
        let request = build_metal_shader_compile_request(
            job(DrawType::ImageMesh),
            MetalFeatures::default(),
            MAC,
        )
        .expect("image mesh request");
        assert_eq!(
            request.source,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/image_mesh_raster.metal"
            ))
        );
        assert_eq!(
            request.preprocessor_macros,
            [
                MetalPreprocessorMacro {
                    name: super::super::shader_compile_plan::ShaderMacro::Vertex.metal_token(),
                    value: "",
                },
                MetalPreprocessorMacro {
                    name: super::super::shader_compile_plan::ShaderMacro::Fragment.metal_token(),
                    value: "",
                },
                MetalPreprocessorMacro {
                    name: super::super::shader_compile_plan::ShaderMacro::DrawImage.metal_token(),
                    value: "",
                },
                MetalPreprocessorMacro {
                    name: super::super::shader_compile_plan::ShaderMacro::DrawImageMesh
                        .metal_token(),
                    value: "",
                },
            ]
        );
        assert_eq!(
            request.options,
            MetalShaderCompileOptions {
                language_version: MetalLanguageVersion::Version2_3,
                fast_math_enabled: true,
                preserve_invariance_when_available: true,
            }
        );

        let ios_request = build_metal_shader_compile_request(
            job(DrawType::ImageMesh),
            MetalFeatures::default(),
            IOS_SIMULATOR,
        )
        .expect("simulator image mesh request");
        assert_eq!(
            ios_request.options.language_version,
            MetalLanguageVersion::Version2_2
        );

        let atomic_request = build_metal_shader_compile_request(
            BackgroundCompileJob::new(
                DrawType::MidpointFanPatches,
                0,
                super::super::shader_compile_plan::InterlockMode::Atomics,
                0,
            ),
            MetalFeatures::default(),
            MAC,
        )
        .expect("atomic path request");
        assert_eq!(
            atomic_request.source,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/path_atomics.metal"
            ))
        );
    }

    #[test]
    fn starts_lazily_compiles_pending_fifo_and_pops_finished_lifo() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let order_for_compiler = Arc::clone(&order);
        let (completed_tx, completed_rx) = mpsc::channel();
        let compiler =
            BackgroundShaderCompiler::new(MetalFeatures::default(), MAC, move |request| {
                let marker = if request.preprocessor_macros.iter().any(|definition| {
                    definition.name
                        == super::super::shader_compile_plan::ShaderMacro::DrawImageMesh
                            .metal_token()
                }) {
                    3
                } else if request.preprocessor_macros.iter().any(|definition| {
                    definition.name
                        == super::super::shader_compile_plan::ShaderMacro::FeatherAtlasBlit
                            .metal_token()
                }) {
                    2
                } else {
                    1
                };
                lock_recovering_poison(&order_for_compiler).push(marker);
                completed_tx.send(()).expect("completion observer");
                Ok(marker)
            });
        assert!(!compiler.is_started());
        compiler.push_job(job(DrawType::MidpointFanPatches));
        assert!(compiler.is_started());
        compiler.push_job(job(DrawType::AtlasBlit));
        compiler.push_job(job(DrawType::ImageMesh));
        for _ in 0..3 {
            completed_rx
                .recv_timeout(Duration::from_secs(2))
                .expect("worker completion");
        }
        let mut state = lock_recovering_poison(&compiler.shared.state);
        while state.finished_jobs.len() != 3 {
            state = wait_recovering_poison(&compiler.shared.work_finished, state);
        }
        drop(state);
        assert_eq!(*lock_recovering_poison(&order), [1, 2, 3]);
        let popped: Vec<_> = (0..3)
            .map(|_| {
                compiler
                    .pop_finished_job(false)
                    .expect("finished job")
                    .result
                    .expect("compiled marker")
            })
            .collect();
        assert_eq!(popped, [3, 2, 1]);
        assert!(compiler.pop_finished_job(false).is_none());
    }

    #[test]
    fn nonblocking_pop_returns_immediately_and_blocking_pop_waits() {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let compiler = Arc::new(BackgroundShaderCompiler::new(
            MetalFeatures::default(),
            MAC,
            move |_| {
                entered_tx.send(()).expect("entered observer");
                release_rx.recv().expect("release compile");
                Ok(7)
            },
        ));
        assert!(compiler.pop_finished_job(false).is_none());
        compiler.push_job(job(DrawType::ImageMesh));
        entered_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("compiler entered");

        let waiting_compiler = Arc::clone(&compiler);
        let (popped_tx, popped_rx) = mpsc::channel();
        let pop_thread = thread::spawn(move || {
            popped_tx
                .send(waiting_compiler.pop_finished_job(true))
                .expect("pop observer");
        });
        assert!(popped_rx.recv_timeout(Duration::from_millis(50)).is_err());
        release_tx.send(()).expect("release worker");
        let finished = popped_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("blocking pop returned")
            .expect("finished job");
        assert_eq!(finished.result.expect("compile result"), 7);
        pop_thread.join().expect("pop thread");
    }

    #[test]
    fn compiler_panic_finishes_the_job_and_worker_continues() {
        let invocation_count = Arc::new(AtomicUsize::new(0));
        let invocation_count_for_compiler = Arc::clone(&invocation_count);
        let compiler = Arc::new(BackgroundShaderCompiler::new(
            MetalFeatures::default(),
            MAC,
            move |_| {
                if invocation_count_for_compiler.fetch_add(1, Ordering::SeqCst) == 0 {
                    panic!("injected compiler panic");
                }
                Ok(7)
            },
        ));

        compiler.push_job(job(DrawType::ImageMesh));
        let waiting_compiler = Arc::clone(&compiler);
        let (first_tx, first_rx) = mpsc::channel();
        let first_pop_thread = thread::spawn(move || {
            first_tx
                .send(waiting_compiler.pop_finished_job(true))
                .expect("first panic observer");
        });
        let first = first_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("panicking compiler still publishes a bounded completion")
            .expect("panicking compile job");
        assert_eq!(first.job.draw_type, DrawType::ImageMesh);
        assert_eq!(
            first.result,
            Err(BackgroundShaderCompileError::CompilerPanicked {
                message: Some("injected compiler panic".to_owned()),
            })
        );
        first_pop_thread.join().expect("first pop thread");

        compiler.push_job(job(DrawType::AtlasBlit));
        let waiting_compiler = Arc::clone(&compiler);
        let (second_tx, second_rx) = mpsc::channel();
        let second_pop_thread = thread::spawn(move || {
            second_tx
                .send(waiting_compiler.pop_finished_job(true))
                .expect("second completion observer");
        });
        let second = second_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("worker continues after compiler panic")
            .expect("subsequent compile job");
        assert_eq!(second.job.draw_type, DrawType::AtlasBlit);
        assert_eq!(second.result, Ok(7));
        second_pop_thread.join().expect("second pop thread");
        assert_eq!(invocation_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn drop_during_active_compile_sets_quit_before_next_pending_job() {
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let compile_count = Arc::new(Mutex::new(0));
        let count_for_compiler = Arc::clone(&compile_count);
        let compiler = BackgroundShaderCompiler::new(MetalFeatures::default(), MAC, move |_| {
            *lock_recovering_poison(&count_for_compiler) += 1;
            entered_tx.send(()).expect("entered observer");
            release_rx.recv().expect("release active compile");
            Ok(())
        });
        compiler.push_job(job(DrawType::MidpointFanPatches));
        entered_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("first compile entered");
        compiler.push_job(job(DrawType::ImageMesh));

        let (dropped_tx, dropped_rx) = mpsc::channel();
        let drop_thread = thread::spawn(move || {
            drop(compiler);
            dropped_tx.send(()).expect("drop observer");
        });
        assert!(dropped_rx.recv_timeout(Duration::from_millis(50)).is_err());
        release_tx.send(()).expect("release active compile");
        dropped_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("drop joined worker");
        drop_thread.join().expect("drop thread");
        assert_eq!(*lock_recovering_poison(&compile_count), 1);
    }

    #[test]
    fn reports_backend_plan_and_synthesized_failures_without_stopping_worker() {
        let callback_count = Arc::new(Mutex::new(0));
        let callback_count_for_compiler = Arc::clone(&callback_count);
        let compiler =
            BackgroundShaderCompiler::<()>::new(MetalFeatures::default(), MAC, move |_| {
                *lock_recovering_poison(&callback_count_for_compiler) += 1;
                Err(MetalLibraryCompileFailure::new("compiler rejected source"))
            });
        compiler.push_job(job(DrawType::ImageMesh));
        let backend = compiler
            .pop_finished_job(true)
            .expect("backend completion")
            .result
            .expect_err("backend failure");
        let BackgroundShaderCompileError::MetalCompilation {
            localized_description,
            source,
        } = backend
        else {
            panic!("wrong backend failure type");
        };
        assert_eq!(
            localized_description.as_deref(),
            Some("compiler rejected source")
        );
        assert_eq!(
            source,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/image_mesh_raster.metal"
            ))
        );

        compiler.push_job(
            job(DrawType::ImageMesh)
                .with_synthesized_failure(SynthesizedFailureType::ShaderCompilation),
        );
        assert_eq!(
            compiler
                .pop_finished_job(true)
                .expect("synthesized completion")
                .result,
            Err(BackgroundShaderCompileError::SynthesizedShaderCompilation)
        );

        compiler.push_job(job(DrawType::MsaaStrokes));
        assert!(matches!(
            compiler
                .pop_finished_job(true)
                .expect("plan completion")
                .result,
            Err(BackgroundShaderCompileError::Plan(
                BackgroundCompilePlanError::UnsupportedDrawType {
                    draw_type: DrawType::MsaaStrokes
                }
            ))
        ));
        assert_eq!(*lock_recovering_poison(&callback_count), 1);
    }

    #[test]
    fn failure_source_listing_matches_upstream_line_numbers() {
        let error = BackgroundShaderCompileError::MetalCompilation {
            localized_description: None,
            source: "first\nsecond\n".to_owned(),
        };
        assert_eq!(
            error.numbered_source().as_deref(),
            Some("   1| first\n   2| second\n")
        );
    }

    #[cfg(all(feature = "native-metal-experimental", target_os = "macos"))]
    #[test]
    fn live_metal_compiles_the_materialized_image_mesh_source() {
        let Some(device) = objc2_metal::MTLCreateSystemDefaultDevice() else {
            return;
        };
        let compiler =
            NativeBackgroundShaderCompiler::new_metal(device, MetalFeatures::default(), MAC);
        compiler.push_job(job(DrawType::ImageMesh));
        compiler
            .pop_finished_job(true)
            .expect("live Metal completion")
            .result
            .unwrap_or_else(|error| {
                panic!(
                    "live Metal shader compilation failed: {error:?}\n{}",
                    error.numbered_source().unwrap_or_default()
                )
            });
    }
}
