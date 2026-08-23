//! Compatible Metal draw-pipeline selection and cache policy.
//!
//! This module is the bounded translation of pinned upstream
//! `renderer/include/rive/renderer/metal/render_context_metal_impl.h:93-109,222-229,235-266`
//! and `renderer/src/metal/render_context_metal_impl.mm:651-704,1117-1224` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
//!
//! The cache owns its backend by value. The production backend can therefore
//! own the one long-lived `NativeBackgroundShaderCompiler` immediately, while
//! tests use a deterministic scripted backend through the same interface. No
//! compiler queue operation escapes this module: otherwise two consumers could
//! steal each other's LIFO completions.

use super::shader_compile_plan::{
    InterlockMode, ShaderFeatures, ShaderMiscFlags, BORROWED_COVERAGE_PASS,
    COALESCED_RESOLVE_AND_TRANSFER, ENABLE_ADVANCED_BLEND, ENABLE_CLIPPING, ENABLE_CLIP_RECT,
    ENABLE_DITHER, ENABLE_EVEN_ODD, ENABLE_HSL_BLEND_MODES, ENABLE_NESTED_CLIPPING,
    FIXED_FUNCTION_COLOR_OUTPUT, STORE_COLOR_CLEAR, SWIZZLE_COLOR_BGRA_TO_RGBA,
};
pub(crate) use super::context_options::{
    NativeMetalContextOptions, ShaderCompilationMode,
};
#[cfg(test)]
use super::context_options::NativeMetalSynthesizedFailureType;
use crate::gpu::DrawType;
use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

const ALL_SHADER_FEATURES: ShaderFeatures = (1 << 8) - 1;
const EXCLUSIVE_ATOMIC_UBERSHADER_FEATURES: ShaderFeatures = ENABLE_ADVANCED_BLEND;
const INTERLOCK_MODE_BIT_COUNT: u32 = 3;
const SHADER_FEATURE_COUNT: u32 = 8;
const DRAW_TYPE_KEY_BIT_COUNT: u32 = 3;
const CLOCKWISE_FILL: ShaderMiscFlags = 1 << 1;

/// Platform facts read by upstream `UbershaderFeaturesMaskFor`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PipelinePlatformFeatures {
    pub(crate) supports_raster_ordering: bool,
    pub(crate) supports_clip_planes: bool,
}

/// Test/tool failures from pinned `SynthesizedFailureType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PipelineFailureInjection {
    #[default]
    None,
    UbershaderLoad,
    ShaderCompilation,
    PipelineCreation,
}

/// A source-shaped lookup request. The cache derives all keys and fallback
/// features; callers cannot supply a mismatched fallback key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PipelineRequest {
    pub(crate) draw_type: DrawType,
    pub(crate) shader_features: ShaderFeatures,
    pub(crate) interlock_mode: InterlockMode,
    pub(crate) shader_misc_flags: ShaderMiscFlags,
    pub(crate) failure_injection: PipelineFailureInjection,
}

impl PipelineRequest {
    pub(crate) const fn new(
        draw_type: DrawType,
        shader_features: ShaderFeatures,
        interlock_mode: InterlockMode,
        shader_misc_flags: ShaderMiscFlags,
    ) -> Self {
        Self {
            draw_type,
            shader_features,
            interlock_mode,
            shader_misc_flags,
            failure_injection: PipelineFailureInjection::None,
        }
    }

    pub(crate) const fn with_failure(mut self, failure: PipelineFailureInjection) -> Self {
        self.failure_injection = failure;
        self
    }
}

/// Exact packed identity returned by pinned `gpu::ShaderUniqueKey`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct PipelineKey(u32);

impl PipelineKey {
    pub(crate) const fn get(self) -> u32 {
        self.0
    }
}

/// A validated job submitted to the concrete compiler backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PipelineJob {
    pub(crate) draw_type: DrawType,
    pub(crate) shader_features: ShaderFeatures,
    pub(crate) interlock_mode: InterlockMode,
    pub(crate) shader_misc_flags: ShaderMiscFlags,
    pub(crate) failure_injection: PipelineFailureInjection,
}

impl PipelineJob {
    fn from_request(request: PipelineRequest, shader_features: ShaderFeatures) -> Self {
        Self {
            draw_type: request.draw_type,
            shader_features,
            interlock_mode: request.interlock_mode,
            shader_misc_flags: request.shader_misc_flags,
            failure_injection: request.failure_injection,
        }
    }

    fn key(self) -> Result<PipelineKey, PipelineCacheError> {
        pipeline_key(
            self.draw_type,
            self.shader_features,
            self.interlock_mode,
            self.shader_misc_flags,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PipelineCacheError {
    InvalidRequest(&'static str),
    InvalidCompletion(&'static str),
    Poisoned,
}

impl fmt::Display for PipelineCacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => {
                write!(formatter, "invalid pipeline request: {message}")
            }
            Self::InvalidCompletion(message) => {
                write!(formatter, "invalid pipeline completion: {message}")
            }
            Self::Poisoned => formatter.write_str("compatible pipeline cache is poisoned"),
        }
    }
}

impl std::error::Error for PipelineCacheError {}

/// Exact feature mask from pinned `gpu::ShaderFeaturesMaskFor`.
pub(crate) fn shader_features_mask_for(
    draw_type: DrawType,
    interlock_mode: InterlockMode,
) -> Result<ShaderFeatures, PipelineCacheError> {
    let interlock_mask = match interlock_mode {
        InterlockMode::RasterOrdering => ALL_SHADER_FEATURES,
        InterlockMode::Atomics => ALL_SHADER_FEATURES & !ENABLE_NESTED_CLIPPING,
        InterlockMode::Clockwise => ALL_SHADER_FEATURES & !ENABLE_EVEN_ODD,
        InterlockMode::ClockwiseAtomic => {
            ALL_SHADER_FEATURES & !ENABLE_EVEN_ODD & !ENABLE_NESTED_CLIPPING
        }
        InterlockMode::Msaa => {
            ENABLE_CLIP_RECT | ENABLE_ADVANCED_BLEND | ENABLE_HSL_BLEND_MODES | ENABLE_DITHER
        }
    };

    let draw_mask = match draw_type {
        DrawType::ImageRect | DrawType::ImageMesh | DrawType::AtlasBlit
            if interlock_mode != InterlockMode::Atomics =>
        {
            ENABLE_CLIPPING
                | ENABLE_CLIP_RECT
                | ENABLE_ADVANCED_BLEND
                | ENABLE_HSL_BLEND_MODES
                | ENABLE_DITHER
        }
        DrawType::MidpointFanPatches
        | DrawType::MidpointFanCenterAaPatches
        | DrawType::OuterCurvePatches
        | DrawType::InteriorTriangulation
        | DrawType::ImageRect
        | DrawType::ImageMesh
        | DrawType::AtlasBlit
        | DrawType::MsaaStrokes
        | DrawType::MsaaMidpointFanBorrowedCoverage
        | DrawType::MsaaDynamicMidpointFans
        | DrawType::MsaaMidpointFans
        | DrawType::MsaaMidpointFanStencilReset
        | DrawType::MsaaMidpointFanPathsStencil
        | DrawType::MsaaMidpointFanPathsCover
        | DrawType::MsaaOuterCubics => ALL_SHADER_FEATURES,
        DrawType::ClipReset => ENABLE_DITHER,
        DrawType::RenderPassInitialize => match interlock_mode {
            InterlockMode::Atomics => ENABLE_CLIPPING | ENABLE_ADVANCED_BLEND | ENABLE_DITHER,
            InterlockMode::Msaa => ENABLE_DITHER,
            InterlockMode::ClockwiseAtomic => 0,
            _ => {
                return Err(PipelineCacheError::InvalidRequest(
                    "render-pass initialize requires atomics, MSAA, or clockwise-atomic",
                ));
            }
        },
        DrawType::RenderPassResolve => match interlock_mode {
            InterlockMode::Atomics => ALL_SHADER_FEATURES,
            InterlockMode::RasterOrdering | InterlockMode::Msaa => ENABLE_DITHER,
            _ => {
                return Err(PipelineCacheError::InvalidRequest(
                    "render-pass resolve requires raster ordering, atomics, or MSAA",
                ));
            }
        },
    };
    Ok(draw_mask & interlock_mask)
}

/// Exact compatible fallback feature calculation from pinned
/// `UbershaderFeaturesMaskFor`.
pub(crate) fn ubershader_features_mask_for(
    request: PipelineRequest,
    platform: PipelinePlatformFeatures,
) -> Result<ShaderFeatures, PipelineCacheError> {
    let mut output = shader_features_mask_for(request.draw_type, request.interlock_mode)?;
    if request.interlock_mode == InterlockMode::Atomics {
        output &= request.shader_features | !EXCLUSIVE_ATOMIC_UBERSHADER_FEATURES;
    }
    if request.shader_features & output != request.shader_features {
        return Err(PipelineCacheError::InvalidRequest(
            "requested shader features are not supported by this draw/interlock pair",
        ));
    }
    if request.interlock_mode == InterlockMode::Msaa && !platform.supports_clip_planes {
        output &= !ENABLE_CLIP_RECT;
    }
    if request.shader_misc_flags & (BORROWED_COVERAGE_PASS | FIXED_FUNCTION_COLOR_OUTPUT) != 0 {
        output &= !ENABLE_ADVANCED_BLEND;
    }
    if request.interlock_mode == InterlockMode::Atomics
        && request.shader_misc_flags & COALESCED_RESOLVE_AND_TRANSFER != 0
    {
        output |= ENABLE_ADVANCED_BLEND;
    }
    Ok(output)
}

/// Exact packed identity from pinned `gpu.cpp:305-381`.
pub(crate) fn pipeline_key(
    draw_type: DrawType,
    shader_features: ShaderFeatures,
    interlock_mode: InterlockMode,
    shader_misc_flags: ShaderMiscFlags,
) -> Result<PipelineKey, PipelineCacheError> {
    if shader_misc_flags & COALESCED_RESOLVE_AND_TRANSFER != 0
        && (draw_type != DrawType::RenderPassResolve
            || shader_features & ENABLE_ADVANCED_BLEND == 0
            || interlock_mode != InterlockMode::Atomics)
    {
        return Err(PipelineCacheError::InvalidRequest(
            "coalesced resolve requires advanced-blend atomic resolve",
        ));
    }
    if shader_misc_flags & (STORE_COLOR_CLEAR | SWIZZLE_COLOR_BGRA_TO_RGBA) != 0
        && (draw_type != DrawType::RenderPassInitialize || interlock_mode != InterlockMode::Atomics)
    {
        return Err(PipelineCacheError::InvalidRequest(
            "store-clear and BGRA swizzle require atomic initialize",
        ));
    }

    let draw_type_key = match draw_type {
        DrawType::MidpointFanPatches
        | DrawType::MidpointFanCenterAaPatches
        | DrawType::OuterCurvePatches
        | DrawType::MsaaStrokes
        | DrawType::MsaaMidpointFanBorrowedCoverage
        | DrawType::MsaaDynamicMidpointFans
        | DrawType::MsaaMidpointFans
        | DrawType::MsaaMidpointFanStencilReset
        | DrawType::MsaaMidpointFanPathsStencil
        | DrawType::MsaaMidpointFanPathsCover
        | DrawType::MsaaOuterCubics => 0,
        DrawType::InteriorTriangulation => 1,
        DrawType::AtlasBlit => 2,
        DrawType::ImageRect => 3,
        DrawType::ImageMesh => 4,
        DrawType::RenderPassInitialize => {
            if !matches!(
                interlock_mode,
                InterlockMode::Atomics | InterlockMode::Msaa | InterlockMode::ClockwiseAtomic
            ) {
                return Err(PipelineCacheError::InvalidRequest(
                    "render-pass initialize has an invalid interlock",
                ));
            }
            5
        }
        DrawType::RenderPassResolve => {
            if !matches!(
                interlock_mode,
                InterlockMode::RasterOrdering | InterlockMode::Atomics | InterlockMode::Msaa
            ) {
                return Err(PipelineCacheError::InvalidRequest(
                    "render-pass resolve has an invalid interlock",
                ));
            }
            6
        }
        DrawType::ClipReset => {
            if !matches!(
                interlock_mode,
                InterlockMode::ClockwiseAtomic | InterlockMode::Msaa
            ) {
                return Err(PipelineCacheError::InvalidRequest(
                    "clip reset requires clockwise-atomic or MSAA",
                ));
            }
            7
        }
    };
    let interlock_key = match interlock_mode {
        InterlockMode::RasterOrdering => 0,
        InterlockMode::Atomics => 1,
        InterlockMode::Clockwise => 2,
        InterlockMode::ClockwiseAtomic => 3,
        InterlockMode::Msaa => 4,
    };
    let masked_features = shader_features & shader_features_mask_for(draw_type, interlock_mode)?;
    let key = ((((shader_misc_flags << INTERLOCK_MODE_BIT_COUNT) | interlock_key)
        << SHADER_FEATURE_COUNT)
        | masked_features)
        << DRAW_TYPE_KEY_BIT_COUNT
        | draw_type_key;
    Ok(PipelineKey(key))
}

/// One constructor-time raster-ordering ubershader from upstream lines
/// 651-704. Order is observable to the injected backend and matches the two
/// nested source loops, including the skipped clockwise AtlasBlit case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RasterPreloadSpec {
    pub(crate) job: PipelineJob,
    pub(crate) key: PipelineKey,
    pub(crate) vertex_function: &'static str,
    pub(crate) fragment_function: &'static str,
}

/// The exact seven constructor-time raster ubershaders and their metallib
/// names. The short `GC`/`JB` exports are pinned by `DrawShaderLibrary`.
pub(crate) fn raster_preload_specs() -> Result<[RasterPreloadSpec; 7], PipelineCacheError> {
    let make =
        |draw_type, shader_features, shader_misc_flags, vertex_function, fragment_function| {
            let job = PipelineJob {
                draw_type,
                shader_features,
                interlock_mode: InterlockMode::RasterOrdering,
                shader_misc_flags,
                failure_injection: PipelineFailureInjection::None,
            };
            Ok(RasterPreloadSpec {
                job,
                key: job.key()?,
                vertex_function,
                fragment_function,
            })
        };

    Ok([
        make(
            DrawType::MidpointFanPatches,
            ALL_SHADER_FEATURES,
            0,
            "p1111000000::GC",
            "p1111111100::JB",
        )?,
        make(
            DrawType::MidpointFanPatches,
            ALL_SHADER_FEATURES,
            CLOCKWISE_FILL,
            "p1111000000::GC",
            "c1111111100::JB",
        )?,
        make(
            DrawType::InteriorTriangulation,
            ALL_SHADER_FEATURES,
            0,
            "p1111000010::GC",
            "p1111111110::JB",
        )?,
        make(
            DrawType::InteriorTriangulation,
            ALL_SHADER_FEATURES,
            CLOCKWISE_FILL,
            "p1111000010::GC",
            "c1111111110::JB",
        )?,
        make(
            DrawType::AtlasBlit,
            shader_features_mask_for(DrawType::AtlasBlit, InterlockMode::RasterOrdering)?,
            0,
            "p1110000011::GC",
            "p1110001111::JB",
        )?,
        make(
            DrawType::ImageMesh,
            shader_features_mask_for(DrawType::ImageMesh, InterlockMode::RasterOrdering)?,
            0,
            "m1110000000::GC",
            "m1110001100::JB",
        )?,
        make(
            DrawType::ImageMesh,
            shader_features_mask_for(DrawType::ImageMesh, InterlockMode::RasterOrdering)?,
            CLOCKWISE_FILL,
            "m1110000000::GC",
            "m1110001100::JB",
        )?,
    ])
}

/// A compiler completion before main-thread pipeline realization.
pub(crate) struct FinishedPipelineJob<Artifact, Error> {
    pub(crate) job: PipelineJob,
    pub(crate) result: Result<Artifact, Error>,
}

/// The sole seam between cache policy and native Metal work.
///
/// A production adapter owns the retained device and the one background
/// compiler. The cache owns that adapter by value, so scheduling and completion
/// routing cannot be split across consumers. Tests use a scripted adapter.
pub(crate) trait PipelineCacheBackend {
    type Pipeline: Clone;
    type Artifact;
    type Error: Clone;

    fn preload(&mut self, spec: RasterPreloadSpec) -> Result<Self::Pipeline, Self::Error>;
    fn schedule(&mut self, job: PipelineJob) -> Result<(), Self::Error>;
    fn pop_finished(
        &mut self,
        wait: bool,
    ) -> Option<FinishedPipelineJob<Self::Artifact, Self::Error>>;
    fn realize(
        &mut self,
        job: PipelineJob,
        artifact: Self::Artifact,
    ) -> Result<Self::Pipeline, Self::Error>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PipelineFailure<Error> {
    Preload(Error),
    Schedule(Error),
    Compilation(Error),
    Realization(Error),
    InjectedPipelineCreation,
}

/// Caller-visible non-ready state. `Absent` is useful when reporting an early
/// injected ubershader failure, which must not mutate the cache.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PipelineAvailability<Error> {
    Pending,
    Failed(PipelineFailure<Error>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PipelineSelection<Pipeline, Error> {
    Ready {
        pipeline: Pipeline,
        requested_key: PipelineKey,
        selected_key: PipelineKey,
    },
    InjectedUbershaderLoad,
    Unavailable {
        requested_key: PipelineKey,
        fallback_key: PipelineKey,
        requested: PipelineAvailability<Error>,
        fallback: PipelineAvailability<Error>,
    },
}

enum PipelineEntry<Pipeline, Error> {
    Pending { submitted_job: PipelineJob },
    Ready(Pipeline),
    Failed(PipelineFailure<Error>),
}

struct PipelineCacheInner<Backend: PipelineCacheBackend> {
    backend: Backend,
    entries: HashMap<PipelineKey, PipelineEntry<Backend::Pipeline, Backend::Error>>,
}

/// Long-lived compatible-pipeline cache. A single mutex deliberately
/// serializes lookup, scheduling, completion routing, and realization, matching
/// upstream's render-thread mutation while remaining safe behind the context's
/// retained `Arc`.
pub(crate) struct CompatibleDrawPipelineCache<Backend: PipelineCacheBackend> {
    options: NativeMetalContextOptions,
    platform: PipelinePlatformFeatures,
    inner: Mutex<PipelineCacheInner<Backend>>,
}

impl<Backend: PipelineCacheBackend> CompatibleDrawPipelineCache<Backend> {
    pub(crate) fn new(
        options: NativeMetalContextOptions,
        platform: PipelinePlatformFeatures,
        mut backend: Backend,
    ) -> Result<Self, PipelineCacheError> {
        let mut entries = HashMap::new();
        if platform.supports_raster_ordering {
            for spec in raster_preload_specs()? {
                let entry = match backend.preload(spec) {
                    Ok(pipeline) => PipelineEntry::Ready(pipeline),
                    Err(error) => PipelineEntry::Failed(PipelineFailure::Preload(error)),
                };
                if entries.insert(spec.key, entry).is_some() {
                    return Err(PipelineCacheError::InvalidCompletion(
                        "duplicate raster preload key",
                    ));
                }
            }
        }
        Ok(Self {
            options,
            platform,
            inner: Mutex::new(PipelineCacheInner { backend, entries }),
        })
    }

    /// Select the requested pipeline or its compatible ubershader fallback.
    /// Pending and failed entries never escape as usable pipeline owners.
    pub(crate) fn select(
        &self,
        request: PipelineRequest,
    ) -> Result<PipelineSelection<Backend::Pipeline, Backend::Error>, PipelineCacheError> {
        let ubershader_features = ubershader_features_mask_for(request, self.platform)?;
        let requested_features = match self.options.shader_compilation_mode {
            ShaderCompilationMode::OnlyUbershaders => ubershader_features,
            ShaderCompilationMode::AllowAsynchronous | ShaderCompilationMode::AlwaysSynchronous => {
                request.shader_features
            }
        };

        // Pinned lines 1140-1147 fail before key lookup or scheduling, including
        // when a usable precompiled pipeline already exists.
        if request.failure_injection == PipelineFailureInjection::UbershaderLoad {
            return Ok(PipelineSelection::InjectedUbershaderLoad);
        }

        let requested_job = PipelineJob::from_request(request, requested_features);
        let fallback_job = PipelineJob::from_request(request, ubershader_features);
        let requested_key = requested_job.key()?;
        let fallback_key = fallback_job.key()?;
        let mut inner = self
            .inner
            .lock()
            .map_err(|_| PipelineCacheError::Poisoned)?;

        let wait_for_requested = requested_features == ubershader_features
            || self.options.shader_compilation_mode != ShaderCompilationMode::AllowAsynchronous;
        let requested = resolve_key(
            &mut inner,
            requested_job,
            wait_for_requested,
            request.failure_injection,
        )?;
        if let ResolveOutcome::Ready(pipeline) = requested {
            return Ok(PipelineSelection::Ready {
                pipeline,
                requested_key,
                selected_key: requested_key,
            });
        }

        if requested_key != fallback_key {
            let fallback = resolve_key(&mut inner, fallback_job, true, request.failure_injection)?;
            if let ResolveOutcome::Ready(pipeline) = fallback {
                return Ok(PipelineSelection::Ready {
                    pipeline,
                    requested_key,
                    selected_key: fallback_key,
                });
            }
            return Ok(PipelineSelection::Unavailable {
                requested_key,
                fallback_key,
                requested: requested.availability(),
                fallback: fallback.availability(),
            });
        }

        let availability = requested.availability();
        Ok(PipelineSelection::Unavailable {
            requested_key,
            fallback_key,
            requested: availability.clone(),
            fallback: availability,
        })
    }
}

enum ResolveOutcome<Pipeline, Error> {
    Ready(Pipeline),
    Pending,
    Failed(PipelineFailure<Error>),
}

impl<Pipeline, Error: Clone> ResolveOutcome<Pipeline, Error> {
    fn availability(&self) -> PipelineAvailability<Error> {
        match self {
            Self::Ready(_) => unreachable!("ready outcomes return before availability reporting"),
            Self::Pending => PipelineAvailability::Pending,
            Self::Failed(failure) => PipelineAvailability::Failed(failure.clone()),
        }
    }
}

fn resolve_key<Backend: PipelineCacheBackend>(
    inner: &mut PipelineCacheInner<Backend>,
    job: PipelineJob,
    wait: bool,
    current_failure_injection: PipelineFailureInjection,
) -> Result<ResolveOutcome<Backend::Pipeline, Backend::Error>, PipelineCacheError> {
    let key = job.key()?;
    if !inner.entries.contains_key(&key) {
        // Preserve pinned order: submit first, then publish the placeholder.
        let entry = match inner.backend.schedule(job) {
            Ok(()) => PipelineEntry::Pending { submitted_job: job },
            Err(error) => PipelineEntry::Failed(PipelineFailure::Schedule(error)),
        };
        inner.entries.insert(key, entry);
    }

    if matches!(inner.entries.get(&key), Some(PipelineEntry::Pending { .. })) {
        while let Some(completed) = inner.backend.pop_finished(wait) {
            let completed_key = completed.job.key().map_err(|_| {
                PipelineCacheError::InvalidCompletion("completed job has an invalid key")
            })?;
            let Some(PipelineEntry::Pending { submitted_job }) = inner.entries.get(&completed_key)
            else {
                return Err(PipelineCacheError::InvalidCompletion(
                    "completed job does not have a pending placeholder",
                ));
            };
            // A key intentionally groups compatible patch draw types, so key
            // identity—not full struct equality—is the source routing rule.
            debug_assert_eq!(submitted_job.key(), Ok(completed_key));

            let entry = match completed.result {
                Err(error) => PipelineEntry::Failed(PipelineFailure::Compilation(error)),
                Ok(_artifact)
                    if current_failure_injection == PipelineFailureInjection::PipelineCreation =>
                {
                    // Pinned line 1202 applies the descriptor currently doing
                    // the polling, even to an unrelated completion.
                    PipelineEntry::Failed(PipelineFailure::InjectedPipelineCreation)
                }
                Ok(artifact) => match inner.backend.realize(completed.job, artifact) {
                    Ok(pipeline) => PipelineEntry::Ready(pipeline),
                    Err(error) => PipelineEntry::Failed(PipelineFailure::Realization(error)),
                },
            };
            inner.entries.insert(completed_key, entry);
            if completed_key == key {
                break;
            }
        }
    }

    match inner.entries.get(&key) {
        Some(PipelineEntry::Ready(pipeline)) => Ok(ResolveOutcome::Ready(pipeline.clone())),
        Some(PipelineEntry::Pending { .. }) => Ok(ResolveOutcome::Pending),
        Some(PipelineEntry::Failed(failure)) => Ok(ResolveOutcome::Failed(failure.clone())),
        None => Ok(ResolveOutcome::Pending),
    }
}

#[cfg(all(
    feature = "native-metal-experimental",
    any(
        target_os = "ios",
        target_os = "macos",
        target_os = "tvos",
        target_os = "visionos"
    )
))]
mod metal_backend {
    use super::*;
    use crate::native_metal::background_shader_compiler::{
        BackgroundShaderCompileError, NativeBackgroundShaderCompiler, NativeMetalLibraryCreation,
    };
    use crate::native_metal::draw_pipeline::{DrawPipeline, DrawPipelineError, MetalInterlockMode};
    use crate::native_metal::draw_shader::DrawShaderLibrary;
    use crate::native_metal::shader_compile_plan::{
        BackgroundCompileJob, MetalFeatures, SynthesizedFailureType,
    };
    use objc2::rc::Retained;
    use objc2::runtime::ProtocolObject;
    use objc2_foundation::NSString;
    use objc2_metal::{MTLDevice, MTLLibrary};

    const SPECIALIZED_VERTEX_MAIN: &str = "GC";
    const SPECIALIZED_FRAGMENT_MAIN: &str = "JB";

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(crate) enum MetalPipelineCacheError {
        Background(BackgroundShaderCompileError),
        DrawPipeline(DrawPipelineError),
        MissingPrecompiledLibrary,
        UnsupportedInterlock(InterlockMode),
    }

    pub(crate) struct MetalPipelineCacheBackend {
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        precompiled_library: Option<DrawShaderLibrary>,
        compiler: NativeBackgroundShaderCompiler,
    }

    impl MetalPipelineCacheBackend {
        pub(crate) fn new(
            device: Retained<ProtocolObject<dyn MTLDevice>>,
            precompiled_library: Option<DrawShaderLibrary>,
            metal_features: MetalFeatures,
            platform: super::super::capabilities::ApplePlatform,
        ) -> Self {
            Self {
                compiler: NativeBackgroundShaderCompiler::new_metal(
                    device.clone(),
                    metal_features,
                    platform,
                ),
                device,
                precompiled_library,
            }
        }

        #[cfg(test)]
        pub(crate) fn compiler_is_started(&self) -> bool {
            self.compiler.is_started()
        }
    }

    fn metal_interlock_mode(
        interlock_mode: InterlockMode,
    ) -> Result<MetalInterlockMode, MetalPipelineCacheError> {
        match interlock_mode {
            InterlockMode::RasterOrdering => Ok(MetalInterlockMode::RasterOrdering),
            InterlockMode::Atomics => Ok(MetalInterlockMode::Atomics),
            other => Err(MetalPipelineCacheError::UnsupportedInterlock(other)),
        }
    }

    fn background_job(job: PipelineJob) -> BackgroundCompileJob {
        BackgroundCompileJob::new(
            job.draw_type,
            job.shader_features,
            job.interlock_mode,
            job.shader_misc_flags,
        )
        .with_synthesized_failure(match job.failure_injection {
            PipelineFailureInjection::UbershaderLoad => SynthesizedFailureType::UbershaderLoad,
            PipelineFailureInjection::ShaderCompilation => {
                SynthesizedFailureType::ShaderCompilation
            }
            PipelineFailureInjection::PipelineCreation => SynthesizedFailureType::PipelineCreation,
            PipelineFailureInjection::None => SynthesizedFailureType::None,
        })
    }

    fn pipeline_job(job: BackgroundCompileJob) -> PipelineJob {
        PipelineJob {
            draw_type: job.draw_type,
            shader_features: job.shader_features,
            interlock_mode: job.interlock_mode,
            shader_misc_flags: job.shader_misc_flags,
            failure_injection: match job.synthesized_failure_type {
                SynthesizedFailureType::None => PipelineFailureInjection::None,
                SynthesizedFailureType::UbershaderLoad => PipelineFailureInjection::UbershaderLoad,
                SynthesizedFailureType::ShaderCompilation => {
                    PipelineFailureInjection::ShaderCompilation
                }
                SynthesizedFailureType::PipelineCreation => {
                    PipelineFailureInjection::PipelineCreation
                }
            },
        }
    }

    impl PipelineCacheBackend for MetalPipelineCacheBackend {
        type Pipeline = DrawPipeline;
        type Artifact = Retained<ProtocolObject<dyn MTLLibrary>>;
        type Error = MetalPipelineCacheError;

        fn preload(&mut self, spec: RasterPreloadSpec) -> Result<Self::Pipeline, Self::Error> {
            let precompiled_library = self
                .precompiled_library
                .as_ref()
                .ok_or(MetalPipelineCacheError::MissingPrecompiledLibrary)?;
            DrawPipeline::new(
                &self.device,
                Some(precompiled_library.library()),
                &NSString::from_str(spec.vertex_function),
                &NSString::from_str(spec.fragment_function),
                spec.job.draw_type,
                metal_interlock_mode(spec.job.interlock_mode)?,
                spec.job.shader_misc_flags,
            )
            .map_err(MetalPipelineCacheError::DrawPipeline)
        }

        fn schedule(&mut self, job: PipelineJob) -> Result<(), Self::Error> {
            self.compiler.push_job(background_job(job));
            Ok(())
        }

        fn pop_finished(
            &mut self,
            wait: bool,
        ) -> Option<FinishedPipelineJob<Self::Artifact, Self::Error>> {
            self.compiler.pop_finished_job(wait).map(|finished| {
                let result = finished
                    .result
                    .map_err(MetalPipelineCacheError::Background)
                    .and_then(|creation: NativeMetalLibraryCreation| {
                        if creation.error.is_some() || creation.library.is_none() {
                            Err(MetalPipelineCacheError::Background(
                                BackgroundShaderCompileError::MetalCompilation {
                                    localized_description: creation.error,
                                    source: creation.source,
                                },
                            ))
                        } else {
                            Ok(creation.library.expect("library checked nonnil"))
                        }
                    });
                FinishedPipelineJob {
                    job: pipeline_job(finished.job),
                    result,
                }
            })
        }

        fn realize(
            &mut self,
            job: PipelineJob,
            artifact: Self::Artifact,
        ) -> Result<Self::Pipeline, Self::Error> {
            DrawPipeline::new(
                &self.device,
                Some(&artifact),
                &NSString::from_str(SPECIALIZED_VERTEX_MAIN),
                &NSString::from_str(SPECIALIZED_FRAGMENT_MAIN),
                job.draw_type,
                metal_interlock_mode(job.interlock_mode)?,
                job.shader_misc_flags,
            )
            .map_err(MetalPipelineCacheError::DrawPipeline)
        }
    }

    pub(crate) type NativeCompatibleDrawPipelineCache =
        CompatibleDrawPipelineCache<MetalPipelineCacheBackend>;

    impl NativeCompatibleDrawPipelineCache {
        pub(crate) fn schedule_library_job(
            &self,
            job: PipelineJob,
        ) -> Result<(), MetalPipelineCacheError> {
            self.inner
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .backend
                .schedule(job)
        }

        pub(crate) fn pop_finished_library_job(
            &self,
            wait: bool,
        ) -> Option<
            FinishedPipelineJob<Retained<ProtocolObject<dyn MTLLibrary>>, MetalPipelineCacheError>,
        > {
            self.inner
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .backend
                .pop_finished(wait)
        }

        #[cfg(test)]
        pub(crate) fn compiler_is_started(&self) -> bool {
            self.inner
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .backend
                .compiler_is_started()
        }
    }
}

#[cfg(test)]
pub(crate) use metal_backend::NativeCompatibleDrawPipelineCache;
#[cfg(test)]
pub(crate) use metal_backend::MetalPipelineCacheBackend;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashSet, VecDeque};
    use std::sync::Arc;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestPipeline(PipelineKey);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TestError(&'static str);

    #[derive(Default)]
    struct ScriptState {
        preloads: Vec<RasterPreloadSpec>,
        scheduled: Vec<PipelineJob>,
        pending: VecDeque<PipelineJob>,
        finished: Vec<FinishedPipelineJob<(), TestError>>,
        pops: Vec<bool>,
        realized: Vec<PipelineJob>,
        compilation_failures: HashSet<PipelineKey>,
        realization_failures: HashSet<PipelineKey>,
        complete_on_schedule: bool,
    }

    #[derive(Clone, Default)]
    struct ScriptBackend {
        state: Arc<Mutex<ScriptState>>,
    }

    impl ScriptBackend {
        fn state(&self) -> Arc<Mutex<ScriptState>> {
            Arc::clone(&self.state)
        }
    }

    impl PipelineCacheBackend for ScriptBackend {
        type Pipeline = TestPipeline;
        type Artifact = ();
        type Error = TestError;

        fn preload(&mut self, spec: RasterPreloadSpec) -> Result<Self::Pipeline, Self::Error> {
            self.state.lock().unwrap().preloads.push(spec);
            Ok(TestPipeline(spec.key))
        }

        fn schedule(&mut self, job: PipelineJob) -> Result<(), Self::Error> {
            let mut state = self.state.lock().unwrap();
            state.scheduled.push(job);
            if state.complete_on_schedule {
                let result = if state.compilation_failures.contains(&job.key().unwrap()) {
                    Err(TestError("compile"))
                } else {
                    Ok(())
                };
                state.finished.push(FinishedPipelineJob { job, result });
            } else {
                state.pending.push_back(job);
            }
            Ok(())
        }

        fn pop_finished(
            &mut self,
            wait: bool,
        ) -> Option<FinishedPipelineJob<Self::Artifact, Self::Error>> {
            let mut state = self.state.lock().unwrap();
            state.pops.push(wait);
            if let Some(finished) = state.finished.pop() {
                return Some(finished);
            }
            if !wait {
                return None;
            }
            let job = state.pending.pop_front()?;
            let result = if state.compilation_failures.contains(&job.key().unwrap()) {
                Err(TestError("compile"))
            } else {
                Ok(())
            };
            Some(FinishedPipelineJob { job, result })
        }

        fn realize(
            &mut self,
            job: PipelineJob,
            _artifact: Self::Artifact,
        ) -> Result<Self::Pipeline, Self::Error> {
            let mut state = self.state.lock().unwrap();
            state.realized.push(job);
            let key = job.key().unwrap();
            if state.realization_failures.contains(&key) {
                Err(TestError("realize"))
            } else {
                Ok(TestPipeline(key))
            }
        }
    }

    fn platform(raster: bool) -> PipelinePlatformFeatures {
        PipelinePlatformFeatures {
            supports_raster_ordering: raster,
            supports_clip_planes: false,
        }
    }

    fn atomic_midpoint(features: ShaderFeatures) -> PipelineRequest {
        PipelineRequest::new(
            DrawType::MidpointFanPatches,
            features,
            InterlockMode::Atomics,
            FIXED_FUNCTION_COLOR_OUTPUT,
        )
    }

    fn cache(
        mode: ShaderCompilationMode,
        raster: bool,
        backend: ScriptBackend,
    ) -> CompatibleDrawPipelineCache<ScriptBackend> {
        CompatibleDrawPipelineCache::new(
            NativeMetalContextOptions {
                shader_compilation_mode: mode,
                disable_framebuffer_reads: false,
                ..Default::default()
            },
            platform(raster),
            backend,
        )
        .unwrap()
    }

    #[test]
    fn key_and_ubershader_masks_match_pinned_oracles() {
        assert_eq!(
            NativeMetalContextOptions::default(),
            NativeMetalContextOptions {
                shader_compilation_mode: ShaderCompilationMode::AllowAsynchronous,
                disable_framebuffer_reads: false,
                synthesized_failure_type: NativeMetalSynthesizedFailureType::None,
            }
        );
        let platform = PipelinePlatformFeatures {
            supports_raster_ordering: true,
            supports_clip_planes: false,
        };
        let midpoint = PipelineRequest::new(
            DrawType::MidpointFanPatches,
            ENABLE_CLIPPING,
            InterlockMode::RasterOrdering,
            0,
        );
        assert_eq!(ubershader_features_mask_for(midpoint, platform), Ok(0xff));
        assert_eq!(
            pipeline_key(midpoint.draw_type, 0xff, midpoint.interlock_mode, 0),
            Ok(PipelineKey(0x07f8))
        );

        let atomic_resolve = PipelineRequest::new(
            DrawType::RenderPassResolve,
            ENABLE_DITHER,
            InterlockMode::Atomics,
            COALESCED_RESOLVE_AND_TRANSFER,
        );
        assert_eq!(
            ubershader_features_mask_for(atomic_resolve, platform),
            Ok(ALL_SHADER_FEATURES & !ENABLE_NESTED_CLIPPING)
        );
        assert_eq!(
            pipeline_key(
                atomic_resolve.draw_type,
                ENABLE_ADVANCED_BLEND,
                atomic_resolve.interlock_mode,
                atomic_resolve.shader_misc_flags,
            )
            .map(PipelineKey::get),
            Ok(4_196_390)
        );

        let msaa_midpoint = PipelineRequest::new(
            DrawType::MidpointFanPatches,
            ENABLE_DITHER,
            InterlockMode::Msaa,
            0,
        );
        assert_eq!(
            ubershader_features_mask_for(msaa_midpoint, platform),
            Ok(ENABLE_ADVANCED_BLEND | ENABLE_HSL_BLEND_MODES | ENABLE_DITHER),
            "pinned Metal supports clip scissors but leaves clip-distance planes disabled"
        );
        assert_eq!(
            ubershader_features_mask_for(
                msaa_midpoint,
                PipelinePlatformFeatures {
                    supports_clip_planes: true,
                    ..platform
                },
            ),
            Ok(ENABLE_CLIP_RECT | ENABLE_ADVANCED_BLEND | ENABLE_HSL_BLEND_MODES | ENABLE_DITHER),
            "the pure key policy must still distinguish a backend that really has clip planes"
        );
    }

    #[test]
    fn context_level_synthesized_failure_is_stored_but_inert() {
        let backend = ScriptBackend::default();
        let state = backend.state();
        let cache = CompatibleDrawPipelineCache::new(
            NativeMetalContextOptions {
                synthesized_failure_type: NativeMetalSynthesizedFailureType::PipelineCreation,
                ..Default::default()
            },
            platform(false),
            backend,
        )
        .unwrap();

        let _ = cache.select(atomic_midpoint(0)).unwrap();
        let state = state.lock().unwrap();
        assert!(!state.scheduled.is_empty());
        assert!(state
            .scheduled
            .iter()
            .all(|job| job.failure_injection == PipelineFailureInjection::None));
    }

    #[test]
    fn raster_preloads_are_exact_and_do_not_schedule_the_compiler() {
        let backend = ScriptBackend::default();
        let state = backend.state();
        let _cache = cache(ShaderCompilationMode::AllowAsynchronous, true, backend);
        let state = state.lock().unwrap();
        assert_eq!(
            state
                .preloads
                .iter()
                .map(|spec| spec.key.get())
                .collect::<Vec<_>>(),
            vec![0x07f8, 0x87f8, 0x07f9, 0x87f9, 0x063a, 0x063c, 0x863c]
        );
        assert_eq!(
            state
                .preloads
                .iter()
                .map(|spec| (spec.vertex_function, spec.fragment_function))
                .collect::<Vec<_>>(),
            vec![
                ("p1111000000::GC", "p1111111100::JB"),
                ("p1111000000::GC", "c1111111100::JB"),
                ("p1111000010::GC", "p1111111110::JB"),
                ("p1111000010::GC", "c1111111110::JB"),
                ("p1110000011::GC", "p1110001111::JB"),
                ("m1110000000::GC", "m1110001100::JB"),
                ("m1110000000::GC", "m1110001100::JB"),
            ]
        );
        assert!(state.scheduled.is_empty());
    }

    #[test]
    fn non_raster_context_preloads_nothing() {
        let backend = ScriptBackend::default();
        let state = backend.state();
        let _cache = cache(ShaderCompilationMode::AllowAsynchronous, false, backend);
        assert!(state.lock().unwrap().preloads.is_empty());
    }

    #[test]
    fn asynchronous_miss_schedules_once_then_waits_for_full_fallback() {
        let backend = ScriptBackend::default();
        let state = backend.state();
        let cache = cache(ShaderCompilationMode::AllowAsynchronous, false, backend);
        let request = atomic_midpoint(ENABLE_DITHER);
        let selection = cache.select(request).unwrap();
        let PipelineSelection::Ready {
            requested_key,
            selected_key,
            ..
        } = selection
        else {
            panic!("full fallback must resolve")
        };
        assert_ne!(requested_key, selected_key);
        {
            let observed = state.lock().unwrap();
            assert_eq!(observed.scheduled.len(), 2);
            assert_eq!(observed.pops, vec![false, true, true]);
        }

        let _ = cache.select(request).unwrap();
        assert_eq!(state.lock().unwrap().scheduled.len(), 2);
    }

    #[test]
    fn synchronous_and_only_uber_modes_select_the_expected_key() {
        for (mode, expect_only_full) in [
            (ShaderCompilationMode::AlwaysSynchronous, false),
            (ShaderCompilationMode::OnlyUbershaders, true),
        ] {
            let backend = ScriptBackend::default();
            let state = backend.state();
            let cache = cache(mode, false, backend);
            let selection = cache.select(atomic_midpoint(ENABLE_DITHER)).unwrap();
            let PipelineSelection::Ready {
                requested_key,
                selected_key,
                ..
            } = selection
            else {
                panic!("requested pipeline must resolve")
            };
            assert_eq!(requested_key, selected_key);
            let state = state.lock().unwrap();
            assert_eq!(state.scheduled.len(), 1);
            assert_eq!(
                state.scheduled[0].shader_features
                    == ubershader_features_mask_for(
                        atomic_midpoint(ENABLE_DITHER),
                        platform(false)
                    )
                    .unwrap(),
                expect_only_full
            );
            assert_eq!(state.pops, vec![true]);
        }
    }

    #[test]
    fn already_finished_async_specialization_wins_without_fallback() {
        let backend = ScriptBackend::default();
        backend.state.lock().unwrap().complete_on_schedule = true;
        let state = backend.state();
        let cache = cache(ShaderCompilationMode::AllowAsynchronous, false, backend);
        let selection = cache.select(atomic_midpoint(ENABLE_DITHER)).unwrap();
        let PipelineSelection::Ready {
            requested_key,
            selected_key,
            ..
        } = selection
        else {
            panic!("specialization must resolve")
        };
        assert_eq!(requested_key, selected_key);
        let state = state.lock().unwrap();
        assert_eq!(state.scheduled.len(), 1);
        assert_eq!(state.pops, vec![false]);
    }

    #[test]
    fn failed_specialization_falls_back_and_never_retries() {
        let backend = ScriptBackend::default();
        let request = atomic_midpoint(ENABLE_DITHER);
        let specialized = PipelineJob::from_request(request, request.shader_features)
            .key()
            .unwrap();
        backend
            .state
            .lock()
            .unwrap()
            .compilation_failures
            .insert(specialized);
        let state = backend.state();
        let cache = cache(ShaderCompilationMode::AlwaysSynchronous, false, backend);
        let first = cache.select(request).unwrap();
        assert!(matches!(
            first,
            PipelineSelection::Ready {
                requested_key,
                selected_key,
                ..
            } if requested_key != selected_key
        ));
        let _ = cache.select(request).unwrap();
        assert_eq!(state.lock().unwrap().scheduled.len(), 2);
    }

    #[test]
    fn failed_full_pipeline_is_cached_as_unavailable() {
        let backend = ScriptBackend::default();
        let request = atomic_midpoint(ENABLE_DITHER);
        let full_features = ubershader_features_mask_for(request, platform(false)).unwrap();
        let full = PipelineJob::from_request(request, full_features)
            .key()
            .unwrap();
        backend
            .state
            .lock()
            .unwrap()
            .realization_failures
            .insert(full);
        let state = backend.state();
        let cache = cache(ShaderCompilationMode::OnlyUbershaders, false, backend);
        assert!(matches!(
            cache.select(request).unwrap(),
            PipelineSelection::Unavailable { .. }
        ));
        assert!(matches!(
            cache.select(request).unwrap(),
            PipelineSelection::Unavailable { .. }
        ));
        assert_eq!(state.lock().unwrap().scheduled.len(), 1);
    }

    #[test]
    fn ubershader_load_injection_does_not_touch_the_cache() {
        let backend = ScriptBackend::default();
        let state = backend.state();
        let cache = cache(ShaderCompilationMode::AllowAsynchronous, true, backend);
        let preloads = state.lock().unwrap().preloads.len();
        let result = cache
            .select(
                PipelineRequest::new(
                    DrawType::MidpointFanPatches,
                    ALL_SHADER_FEATURES,
                    InterlockMode::RasterOrdering,
                    0,
                )
                .with_failure(PipelineFailureInjection::UbershaderLoad),
            )
            .unwrap();
        assert_eq!(result, PipelineSelection::InjectedUbershaderLoad);
        let state = state.lock().unwrap();
        assert_eq!(state.preloads.len(), preloads);
        assert!(state.scheduled.is_empty());
        assert!(state.pops.is_empty());
    }

    #[test]
    fn current_lookup_pipeline_failure_applies_to_unrelated_completion() {
        let backend = ScriptBackend::default();
        let state = backend.state();
        let cache = cache(ShaderCompilationMode::AllowAsynchronous, false, backend);
        let unrelated =
            PipelineJob::from_request(atomic_midpoint(ENABLE_CLIPPING), ENABLE_CLIPPING);
        {
            let mut inner = cache.inner.lock().unwrap();
            inner.entries.insert(
                unrelated.key().unwrap(),
                PipelineEntry::Pending {
                    submitted_job: unrelated,
                },
            );
            inner
                .backend
                .state
                .lock()
                .unwrap()
                .finished
                .push(FinishedPipelineJob {
                    job: unrelated,
                    result: Ok(()),
                });
        }
        let request =
            atomic_midpoint(ENABLE_DITHER).with_failure(PipelineFailureInjection::PipelineCreation);
        let _ = cache.select(request).unwrap();
        let state = state.lock().unwrap();
        assert!(!state
            .realized
            .iter()
            .any(|job| job.key() == unrelated.key()));
    }
}
