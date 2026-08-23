//! Deterministic source-plan translation of the upstream Metal background
//! shader compiler.
//!
//! The source citations are `renderer/src/metal/background_shader_compiler.h:17-30`
//! and `renderer/src/metal/background_shader_compiler.mm:94-275` at pinned
//! upstream SHA `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
//!
//! This module owns the `BackgroundCompileJob` inputs, deterministic
//! macro/source-fragment assembly, and the exact generated source payloads.
//! The worker lifecycle and `newLibraryWithSource` adapter live in
//! `background_shader_compiler`.

pub(crate) use super::capabilities::{ApplePlatform, AtomicBarrierType};
pub(crate) use super::pipeline_names::{
    CLOCKWISE_FILL, ENABLE_ADVANCED_BLEND, ENABLE_CLIPPING, ENABLE_CLIP_RECT, ENABLE_DITHER,
    ENABLE_EVEN_ODD, ENABLE_FEATHER, ENABLE_HSL_BLEND_MODES, ENABLE_NESTED_CLIPPING,
    FIXED_FUNCTION_COLOR_OUTPUT, SHADER_FEATURE_COUNT,
};
use crate::gpu::DrawType;

/// Synchronization choices from upstream `gpu::InterlockMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InterlockMode {
    RasterOrdering,
    Atomics,
    Clockwise,
    ClockwiseAtomic,
    Msaa,
}

/// The feature-mask input to `BackgroundCompileJob`.
pub(crate) type ShaderFeatures = u32;

/// The miscellaneous-flag input to `BackgroundCompileJob`.
pub(crate) type ShaderMiscFlags = u32;

pub(crate) const CLIP_UPDATE_ONLY: ShaderMiscFlags = 1 << 2;
pub(crate) const NESTED_CLIP_UPDATE_ONLY: ShaderMiscFlags = 1 << 3;
pub(crate) const BORROWED_COVERAGE_PASS: ShaderMiscFlags = 1 << 4;
pub(crate) const STORE_COLOR_CLEAR: ShaderMiscFlags = 1 << 5;
pub(crate) const LOAD_COLOR_FROM_DST_TEXTURE: ShaderMiscFlags = 1 << 6;
pub(crate) const SWIZZLE_COLOR_BGRA_TO_RGBA: ShaderMiscFlags = 1 << 7;
pub(crate) const COALESCED_RESOLVE_AND_TRANSFER: ShaderMiscFlags = 1 << 8;

/// The `MetalFeatures` input used by the upstream compiler. The only member
/// read by `background_shader_compiler.mm:115-119` is the atomic barrier type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MetalFeatures {
    pub(crate) atomic_barrier_type: AtomicBarrierType,
}

impl Default for MetalFeatures {
    fn default() -> Self {
        Self {
            atomic_barrier_type: AtomicBarrierType::renderPassBreak,
        }
    }
}

/// Inputs copied from upstream `BackgroundCompileJob`. The compiled Metal
/// library is intentionally absent because this module only creates a plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BackgroundCompileJob {
    pub(crate) draw_type: DrawType,
    pub(crate) shader_features: ShaderFeatures,
    pub(crate) interlock_mode: InterlockMode,
    pub(crate) shader_misc_flags: ShaderMiscFlags,
    pub(crate) synthesized_failure_type: SynthesizedFailureType,
}

/// Test/tool failure injection corresponding to upstream
/// `gpu::SynthesizedFailureType` under `WITH_RIVE_TOOLS`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SynthesizedFailureType {
    #[default]
    None,
    UbershaderLoad,
    ShaderCompilation,
    PipelineCreation,
}

impl BackgroundCompileJob {
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
            synthesized_failure_type: SynthesizedFailureType::None,
        }
    }

    pub(crate) const fn with_synthesized_failure(
        mut self,
        synthesized_failure_type: SynthesizedFailureType,
    ) -> Self {
        self.synthesized_failure_type = synthesized_failure_type;
        self
    }
}

/// A generated Metal preprocessor macro and its exact upstream value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MacroDefinition {
    pub(crate) name: ShaderMacro,
    pub(crate) value: MacroValue,
}

/// Values used by the Objective-C dictionary in the pinned implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MacroValue {
    Empty,
    One,
    True,
}

impl MacroValue {
    /// The exact NSString value inserted into `preprocessorMacros` upstream.
    pub(crate) const fn metal_value(self) -> &'static str {
        match self {
            Self::Empty => "",
            Self::One => "1",
            Self::True => "true",
        }
    }
}

/// Generated shader macro names used by `background_shader_compiler.mm`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShaderMacro {
    Vertex,
    Fragment,
    EnableClipping,
    EnableClipRect,
    EnableAdvancedBlend,
    EnableFeather,
    EnableEvenOdd,
    EnableNestedClipping,
    EnableHslBlendModes,
    EnableDither,
    PlsImplDeviceBuffer,
    PlsImplDeviceBufferRasterOrdered,
    FixedFunctionColorOutput,
    ClockwiseFill,
    EnableInstanceIndex,
    DrawPath,
    DrawInteriorTriangles,
    FeatherAtlasBlit,
    DrawImage,
    DrawImageRect,
    DrawImageMesh,
    DrawRenderTargetUpdateBounds,
    InitializePls,
    StoreColorClear,
    SwizzleColorBgraToRgba,
    ResolvePls,
    CoalescedPlsResolveAndTransfer,
}

// This is the minimal exported-token artifact produced by the pinned
// upstream renderer's authoritative batch-minification rule. Keep the token
// spellings with the generated artifact instead of duplicating them in Rust:
// minify.py assigns names across the complete input batch, so a token map from
// a different or stale build directory is not interchangeable.
const METAL_MACRO_TOKEN_FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/native_metal/background_shader_macros.txt"
));

impl ShaderMacro {
    /// The generated token used as the dictionary key by the Metal compiler.
    pub(crate) fn metal_token(self) -> &'static str {
        let identifier = self.identifier();
        for line in METAL_MACRO_TOKEN_FIXTURE.lines() {
            let Some((fixture_identifier, token)) = line.split_once('=') else {
                continue;
            };
            if fixture_identifier == identifier {
                return token;
            }
        }
        panic!("missing generated Metal token for {identifier}");
    }

    /// The unexpanded macro identifier used in the pinned C++ source.
    pub(crate) const fn identifier(self) -> &'static str {
        match self {
            Self::Vertex => "GLSL_VERTEX",
            Self::Fragment => "GLSL_FRAGMENT",
            Self::EnableClipping => "GLSL_ENABLE_CLIPPING",
            Self::EnableClipRect => "GLSL_ENABLE_CLIP_RECT",
            Self::EnableAdvancedBlend => "GLSL_ENABLE_ADVANCED_BLEND",
            Self::EnableFeather => "GLSL_ENABLE_FEATHER",
            Self::EnableEvenOdd => "GLSL_ENABLE_EVEN_ODD",
            Self::EnableNestedClipping => "GLSL_ENABLE_NESTED_CLIPPING",
            Self::EnableHslBlendModes => "GLSL_ENABLE_HSL_BLEND_MODES",
            Self::EnableDither => "GLSL_ENABLE_DITHER",
            Self::PlsImplDeviceBuffer => "GLSL_PLS_IMPL_DEVICE_BUFFER",
            Self::PlsImplDeviceBufferRasterOrdered => "GLSL_PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED",
            Self::FixedFunctionColorOutput => "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
            Self::ClockwiseFill => "GLSL_CLOCKWISE_FILL",
            Self::EnableInstanceIndex => "GLSL_ENABLE_INSTANCE_INDEX",
            Self::DrawPath => "GLSL_DRAW_PATH",
            Self::DrawInteriorTriangles => "GLSL_DRAW_INTERIOR_TRIANGLES",
            Self::FeatherAtlasBlit => "GLSL_FEATHER_ATLAS_BLIT",
            Self::DrawImage => "GLSL_DRAW_IMAGE",
            Self::DrawImageRect => "GLSL_DRAW_IMAGE_RECT",
            Self::DrawImageMesh => "GLSL_DRAW_IMAGE_MESH",
            Self::DrawRenderTargetUpdateBounds => "GLSL_DRAW_RENDER_TARGET_UPDATE_BOUNDS",
            Self::InitializePls => "GLSL_INITIALIZE_PLS",
            Self::StoreColorClear => "GLSL_STORE_COLOR_CLEAR",
            Self::SwizzleColorBgraToRgba => "GLSL_SWIZZLE_COLOR_BGRA_TO_RGBA",
            Self::ResolvePls => "GLSL_RESOLVE_PLS",
            Self::CoalescedPlsResolveAndTransfer => "GLSL_COALESCED_PLS_RESOLVE_AND_TRANSFER",
        }
    }
}

/// One of the generated GLSL/Metal source objects appended by the pinned
/// compiler. The order in `BackgroundCompilePlan::source_fragments` is part of
/// the translation contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SourceFragment {
    Metal,
    Constants,
    FlushUniforms,
    Common,
    AdvancedBlend,
    DrawPathCommon,
    DrawPathVertex,
    DrawRasterOrderPathFragment,
    AtomicDraw,
    DrawImageMeshVertex,
    DrawMeshFragment,
}

impl SourceFragment {
    /// The exact generated C++ symbol appended by the upstream implementation.
    pub(crate) const fn symbol(self) -> &'static str {
        match self {
            Self::Metal => "gpu::glsl::metal",
            Self::Constants => "gpu::glsl::constants",
            Self::FlushUniforms => "gpu::glsl::flush_uniforms",
            Self::Common => "gpu::glsl::common",
            Self::AdvancedBlend => "gpu::glsl::advanced_blend",
            Self::DrawPathCommon => "gpu::glsl::draw_path_common",
            Self::DrawPathVertex => "gpu::glsl::draw_path_vert",
            Self::DrawRasterOrderPathFragment => "gpu::glsl::draw_raster_order_path_frag",
            Self::AtomicDraw => "gpu::glsl::atomic_draw",
            Self::DrawImageMeshVertex => "gpu::glsl::draw_image_mesh_vert",
            Self::DrawMeshFragment => "gpu::glsl::draw_mesh_frag",
        }
    }

    /// The exact minified UTF-8 payload of the generated C++ source object.
    pub(crate) const fn source(self) -> &'static str {
        match self {
            Self::Metal => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/metal.glsl"
            )),
            Self::Constants => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/constants.glsl"
            )),
            Self::FlushUniforms => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/flush_uniforms.glsl"
            )),
            Self::Common => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/common.glsl"
            )),
            Self::AdvancedBlend => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/advanced_blend.glsl"
            )),
            Self::DrawPathCommon => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/draw_path_common.glsl"
            )),
            Self::DrawPathVertex => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/draw_path.vert"
            )),
            Self::DrawRasterOrderPathFragment => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/draw_raster_order_path.frag"
            )),
            Self::AtomicDraw => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/atomic_draw.glsl"
            )),
            Self::DrawImageMeshVertex => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/draw_image_mesh.vert"
            )),
            Self::DrawMeshFragment => include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/native_metal/background_shader_sources/draw_mesh.frag"
            )),
        }
    }
}

/// The complete deterministic result of the source-assembly portion of the
/// upstream background compiler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BackgroundCompilePlan {
    pub(crate) defines: Vec<MacroDefinition>,
    pub(crate) source_fragments: Vec<SourceFragment>,
}

impl BackgroundCompilePlan {
    /// Reproduce the `NSMutableString` construction in the pinned Objective-C++
    /// implementation. `metal` initializes the string; every later selected
    /// fragment is appended with exactly one additional line feed.
    pub(crate) fn materialize_source(&self) -> String {
        let byte_len = self
            .source_fragments
            .iter()
            .map(|fragment| fragment.source().len())
            .sum::<usize>()
            + self.source_fragments.len().saturating_sub(1);
        let mut source = String::with_capacity(byte_len);
        for (index, fragment) in self.source_fragments.iter().enumerate() {
            source.push_str(fragment.source());
            if index != 0 {
                source.push('\n');
            }
        }
        source
    }
}

/// A typed representation of every `RIVE_UNREACHABLE` or asserted combination
/// reached by the translated source-assembly code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BackgroundCompilePlanError {
    UnsupportedDrawType {
        draw_type: DrawType,
    },
    UnsupportedInterlockMode {
        interlock_mode: InterlockMode,
    },
    DrawRequiresAtomicInterlock {
        draw_type: DrawType,
        interlock_mode: InterlockMode,
    },
    DrawUnavailableOnIos {
        draw_type: DrawType,
    },
    AtomicInterlockUnavailableOnIos,
    LoadColorFromDstTextureUnsupported,
}

const FEATURE_MACROS: [(ShaderFeatures, ShaderMacro); SHADER_FEATURE_COUNT] = [
    (ENABLE_CLIPPING, ShaderMacro::EnableClipping),
    (ENABLE_CLIP_RECT, ShaderMacro::EnableClipRect),
    (ENABLE_ADVANCED_BLEND, ShaderMacro::EnableAdvancedBlend),
    (ENABLE_FEATHER, ShaderMacro::EnableFeather),
    (ENABLE_EVEN_ODD, ShaderMacro::EnableEvenOdd),
    (ENABLE_NESTED_CLIPPING, ShaderMacro::EnableNestedClipping),
    (ENABLE_HSL_BLEND_MODES, ShaderMacro::EnableHslBlendModes),
    (ENABLE_DITHER, ShaderMacro::EnableDither),
];

/// Assemble the plan corresponding to `background_shader_compiler.mm:99-275`.
pub(crate) fn build_shader_compile_plan(
    job: BackgroundCompileJob,
    metal_features: MetalFeatures,
    platform: ApplePlatform,
) -> Result<BackgroundCompilePlan, BackgroundCompilePlanError> {
    let mut defines = vec![
        MacroDefinition {
            name: ShaderMacro::Vertex,
            value: MacroValue::Empty,
        },
        MacroDefinition {
            name: ShaderMacro::Fragment,
            value: MacroValue::Empty,
        },
    ];

    for (feature, macro_name) in FEATURE_MACROS {
        if job.shader_features & feature != 0 {
            defines.push(MacroDefinition {
                name: macro_name,
                value: MacroValue::One,
            });
        }
    }

    if job.interlock_mode == InterlockMode::Atomics {
        // Atomic mode uses device buffers instead of framebuffer fetches.
        defines.push(MacroDefinition {
            name: ShaderMacro::PlsImplDeviceBuffer,
            value: MacroValue::Empty,
        });
        if metal_features.atomic_barrier_type == AtomicBarrierType::rasterOrderGroup {
            defines.push(MacroDefinition {
                name: ShaderMacro::PlsImplDeviceBufferRasterOrdered,
                value: MacroValue::Empty,
            });
        }
        if job.shader_misc_flags & FIXED_FUNCTION_COLOR_OUTPUT != 0 {
            defines.push(MacroDefinition {
                name: ShaderMacro::FixedFunctionColorOutput,
                value: MacroValue::Empty,
            });
        }
    }

    if job.shader_misc_flags & CLOCKWISE_FILL != 0 {
        defines.push(MacroDefinition {
            name: ShaderMacro::ClockwiseFill,
            value: MacroValue::One,
        });
    }

    append_draw_defines(&mut defines, job, platform)?;

    let mut source_fragments = vec![
        SourceFragment::Metal,
        SourceFragment::Constants,
        SourceFragment::FlushUniforms,
        SourceFragment::Common,
    ];
    if job.shader_features & ENABLE_ADVANCED_BLEND != 0 {
        source_fragments.push(SourceFragment::AdvancedBlend);
    }

    append_interlock_sources(&mut source_fragments, job, platform)?;

    Ok(BackgroundCompilePlan {
        defines,
        source_fragments,
    })
}

fn append_draw_defines(
    defines: &mut Vec<MacroDefinition>,
    job: BackgroundCompileJob,
    platform: ApplePlatform,
) -> Result<(), BackgroundCompilePlanError> {
    let empty = |name| MacroDefinition {
        name,
        value: MacroValue::Empty,
    };
    match job.draw_type {
        DrawType::MidpointFanPatches
        | DrawType::MidpointFanCenterAaPatches
        | DrawType::OuterCurvePatches => {
            // Add baseInstance to the instanceID for path draws.
            defines.push(empty(ShaderMacro::EnableInstanceIndex));
            defines.push(empty(ShaderMacro::DrawPath));
        }
        DrawType::InteriorTriangulation => defines.push(empty(ShaderMacro::DrawInteriorTriangles)),
        DrawType::AtlasBlit => defines.push(MacroDefinition {
            name: ShaderMacro::FeatherAtlasBlit,
            value: MacroValue::One,
        }),
        DrawType::ImageRect => {
            if is_rive_ios(platform) {
                return Err(BackgroundCompilePlanError::DrawUnavailableOnIos {
                    draw_type: job.draw_type,
                });
            }
            if job.interlock_mode != InterlockMode::Atomics {
                return Err(BackgroundCompilePlanError::DrawRequiresAtomicInterlock {
                    draw_type: job.draw_type,
                    interlock_mode: job.interlock_mode,
                });
            }
            defines.push(empty(ShaderMacro::DrawImage));
            defines.push(empty(ShaderMacro::DrawImageRect));
        }
        DrawType::ImageMesh => {
            defines.push(empty(ShaderMacro::DrawImage));
            defines.push(empty(ShaderMacro::DrawImageMesh));
        }
        DrawType::RenderPassInitialize => {
            if is_rive_ios(platform) {
                return Err(BackgroundCompilePlanError::DrawUnavailableOnIos {
                    draw_type: job.draw_type,
                });
            }
            if job.interlock_mode != InterlockMode::Atomics {
                return Err(BackgroundCompilePlanError::DrawRequiresAtomicInterlock {
                    draw_type: job.draw_type,
                    interlock_mode: job.interlock_mode,
                });
            }
            defines.push(empty(ShaderMacro::DrawRenderTargetUpdateBounds));
            defines.push(empty(ShaderMacro::InitializePls));
            if job.shader_misc_flags & STORE_COLOR_CLEAR != 0 {
                // The upstream value is "true", not an empty specialization
                // define, because some backends branch on this at runtime.
                defines.push(MacroDefinition {
                    name: ShaderMacro::StoreColorClear,
                    value: MacroValue::True,
                });
            }
            // Metal copies the render target directly to the storage buffer
            // instead of seeding it with the shader.
            if job.shader_misc_flags & LOAD_COLOR_FROM_DST_TEXTURE != 0 {
                return Err(BackgroundCompilePlanError::LoadColorFromDstTextureUnsupported);
            }
            if job.shader_misc_flags & SWIZZLE_COLOR_BGRA_TO_RGBA != 0 {
                defines.push(empty(ShaderMacro::SwizzleColorBgraToRgba));
            }
        }
        DrawType::RenderPassResolve => {
            if is_rive_ios(platform) {
                return Err(BackgroundCompilePlanError::DrawUnavailableOnIos {
                    draw_type: job.draw_type,
                });
            }
            if job.interlock_mode != InterlockMode::Atomics {
                return Err(BackgroundCompilePlanError::DrawRequiresAtomicInterlock {
                    draw_type: job.draw_type,
                    interlock_mode: job.interlock_mode,
                });
            }
            defines.push(empty(ShaderMacro::DrawRenderTargetUpdateBounds));
            defines.push(empty(ShaderMacro::ResolvePls));
            if job.shader_misc_flags & COALESCED_RESOLVE_AND_TRANSFER != 0 {
                defines.push(empty(ShaderMacro::CoalescedPlsResolveAndTransfer));
            }
        }
        DrawType::MsaaStrokes
        | DrawType::MsaaMidpointFanBorrowedCoverage
        | DrawType::MsaaDynamicMidpointFans
        | DrawType::MsaaMidpointFans
        | DrawType::MsaaMidpointFanStencilReset
        | DrawType::MsaaMidpointFanPathsStencil
        | DrawType::MsaaMidpointFanPathsCover
        | DrawType::MsaaOuterCubics
        | DrawType::ClipReset => {
            return Err(BackgroundCompilePlanError::UnsupportedDrawType {
                draw_type: job.draw_type,
            });
        }
    }
    Ok(())
}

fn append_interlock_sources(
    source_fragments: &mut Vec<SourceFragment>,
    job: BackgroundCompileJob,
    platform: ApplePlatform,
) -> Result<(), BackgroundCompilePlanError> {
    match job.interlock_mode {
        InterlockMode::Atomics => {
            // The pinned `#ifndef RIVE_IOS` removes this branch on iOS
            // devices; the remaining branch asserts raster ordering.
            if is_rive_ios(platform) {
                return Err(BackgroundCompilePlanError::AtomicInterlockUnavailableOnIos);
            }
            source_fragments.push(SourceFragment::DrawPathCommon);
            source_fragments.push(SourceFragment::AtomicDraw);
        }
        InterlockMode::RasterOrdering => match job.draw_type {
            DrawType::MidpointFanPatches
            | DrawType::MidpointFanCenterAaPatches
            | DrawType::OuterCurvePatches
            | DrawType::InteriorTriangulation => {
                source_fragments.push(SourceFragment::DrawPathCommon);
                source_fragments.push(SourceFragment::DrawPathVertex);
                source_fragments.push(SourceFragment::DrawRasterOrderPathFragment);
            }
            DrawType::AtlasBlit => {
                source_fragments.push(SourceFragment::DrawPathCommon);
                source_fragments.push(SourceFragment::DrawPathVertex);
                source_fragments.push(SourceFragment::DrawMeshFragment);
            }
            DrawType::ImageMesh => {
                source_fragments.push(SourceFragment::DrawImageMeshVertex);
                source_fragments.push(SourceFragment::DrawMeshFragment);
            }
            DrawType::ImageRect | DrawType::RenderPassInitialize | DrawType::RenderPassResolve => {
                return Err(BackgroundCompilePlanError::DrawRequiresAtomicInterlock {
                    draw_type: job.draw_type,
                    interlock_mode: job.interlock_mode,
                });
            }
            DrawType::MsaaStrokes
            | DrawType::MsaaMidpointFanBorrowedCoverage
            | DrawType::MsaaDynamicMidpointFans
            | DrawType::MsaaMidpointFans
            | DrawType::MsaaMidpointFanStencilReset
            | DrawType::MsaaMidpointFanPathsStencil
            | DrawType::MsaaMidpointFanPathsCover
            | DrawType::MsaaOuterCubics
            | DrawType::ClipReset => {
                return Err(BackgroundCompilePlanError::UnsupportedDrawType {
                    draw_type: job.draw_type,
                });
            }
        },
        InterlockMode::Clockwise | InterlockMode::ClockwiseAtomic | InterlockMode::Msaa => {
            return Err(BackgroundCompilePlanError::UnsupportedInterlockMode {
                interlock_mode: job.interlock_mode,
            });
        }
    }
    Ok(())
}

fn is_rive_ios(platform: ApplePlatform) -> bool {
    matches!(platform, ApplePlatform::IosDevice { .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    const MAC: ApplePlatform = ApplePlatform::MacOs;
    const IOS_DEVICE: ApplePlatform = ApplePlatform::IosDevice {
        is_apple_silicon: true,
    };
    const IOS_SIMULATOR: ApplePlatform = ApplePlatform::IosSimulator {
        host_is_arm64: true,
    };

    const ALL_SHADER_MACROS: [ShaderMacro; 27] = [
        ShaderMacro::Vertex,
        ShaderMacro::Fragment,
        ShaderMacro::EnableClipping,
        ShaderMacro::EnableClipRect,
        ShaderMacro::EnableAdvancedBlend,
        ShaderMacro::EnableFeather,
        ShaderMacro::EnableEvenOdd,
        ShaderMacro::EnableNestedClipping,
        ShaderMacro::EnableHslBlendModes,
        ShaderMacro::EnableDither,
        ShaderMacro::PlsImplDeviceBuffer,
        ShaderMacro::PlsImplDeviceBufferRasterOrdered,
        ShaderMacro::FixedFunctionColorOutput,
        ShaderMacro::ClockwiseFill,
        ShaderMacro::EnableInstanceIndex,
        ShaderMacro::DrawPath,
        ShaderMacro::DrawInteriorTriangles,
        ShaderMacro::FeatherAtlasBlit,
        ShaderMacro::DrawImage,
        ShaderMacro::DrawImageRect,
        ShaderMacro::DrawImageMesh,
        ShaderMacro::DrawRenderTargetUpdateBounds,
        ShaderMacro::InitializePls,
        ShaderMacro::StoreColorClear,
        ShaderMacro::SwizzleColorBgraToRgba,
        ShaderMacro::ResolvePls,
        ShaderMacro::CoalescedPlsResolveAndTransfer,
    ];

    const BASE_SOURCES: [SourceFragment; 4] = [
        SourceFragment::Metal,
        SourceFragment::Constants,
        SourceFragment::FlushUniforms,
        SourceFragment::Common,
    ];

    fn definition(name: ShaderMacro, value: MacroValue) -> MacroDefinition {
        MacroDefinition { name, value }
    }

    fn base_defines() -> Vec<MacroDefinition> {
        vec![
            definition(ShaderMacro::Vertex, MacroValue::Empty),
            definition(ShaderMacro::Fragment, MacroValue::Empty),
        ]
    }

    fn default_job(draw_type: DrawType, interlock_mode: InterlockMode) -> BackgroundCompileJob {
        BackgroundCompileJob::new(draw_type, 0, interlock_mode, 0)
    }

    #[test]
    fn materialized_request_parts_match_the_pinned_generated_sources() {
        assert_eq!(MacroValue::Empty.metal_value(), "");
        assert_eq!(MacroValue::One.metal_value(), "1");
        assert_eq!(MacroValue::True.metal_value(), "true");

        let plan = build_shader_compile_plan(
            default_job(DrawType::ImageMesh, InterlockMode::RasterOrdering),
            MetalFeatures::default(),
            MAC,
        )
        .expect("image mesh plan");
        let expected = [
            SourceFragment::Metal.source(),
            SourceFragment::Constants.source(),
            SourceFragment::FlushUniforms.source(),
            SourceFragment::Common.source(),
            SourceFragment::DrawImageMeshVertex.source(),
            SourceFragment::DrawMeshFragment.source(),
        ]
        .into_iter()
        .enumerate()
        .fold(String::new(), |mut source, (index, fragment)| {
            source.push_str(fragment);
            if index != 0 {
                source.push('\n');
            }
            source
        });
        assert_eq!(plan.materialize_source(), expected);
    }

    fn assert_plan(
        label: &str,
        job: BackgroundCompileJob,
        features: MetalFeatures,
        platform: ApplePlatform,
        expected_defines: Vec<MacroDefinition>,
        expected_sources: Vec<SourceFragment>,
    ) {
        let plan = build_shader_compile_plan(job, features, platform)
            .unwrap_or_else(|error| panic!("{label} unexpectedly failed: {error:?}"));
        assert_eq!(plan.defines, expected_defines, "{label} defines");
        assert_eq!(plan.source_fragments, expected_sources, "{label} sources");
    }

    #[test]
    fn shader_flag_bits_match_upstream_gpu_hpp() {
        assert_eq!(SHADER_FEATURE_COUNT, 8);
        assert_eq!(
            [
                ENABLE_CLIPPING,
                ENABLE_CLIP_RECT,
                ENABLE_ADVANCED_BLEND,
                ENABLE_FEATHER,
                ENABLE_EVEN_ODD,
                ENABLE_NESTED_CLIPPING,
                ENABLE_HSL_BLEND_MODES,
                ENABLE_DITHER,
            ],
            [
                1 << 0,
                1 << 1,
                1 << 2,
                1 << 3,
                1 << 4,
                1 << 5,
                1 << 6,
                1 << 7,
            ]
        );
        assert_eq!(
            [
                FIXED_FUNCTION_COLOR_OUTPUT,
                CLOCKWISE_FILL,
                CLIP_UPDATE_ONLY,
                NESTED_CLIP_UPDATE_ONLY,
                BORROWED_COVERAGE_PASS,
                STORE_COLOR_CLEAR,
                LOAD_COLOR_FROM_DST_TEXTURE,
                SWIZZLE_COLOR_BGRA_TO_RGBA,
                COALESCED_RESOLVE_AND_TRANSFER,
            ],
            [
                1 << 0,
                1 << 1,
                1 << 2,
                1 << 3,
                1 << 4,
                1 << 5,
                1 << 6,
                1 << 7,
                1 << 8,
            ]
        );
    }

    #[test]
    fn generated_macro_fixture_is_pinned_complete_and_one_to_one() {
        for provenance in [
            "upstream_sha=4ac7b32798da0482e441ef09304dc3b480ed3ee5",
            "generator_sha256=bf4b9f529a19765c5e6f28b68ef8a73f5bd65433cd87ce723df5df923e6bc22b",
            "makefile_sha256=ec5d0d98d78051e98cda80f92cd67858cb1fb70be64cddd8ad13bcd4ad5f50fc",
            "input_set_sha256=bb1df1e11890783d83263a0e8de0509af77fbeab9ee50634da9ef028fe9270b1",
            "generated_header_sha256=b415a51a8f22f1485be7c8ec4c033609288652a4396add1129a3df4f54ebd8b7",
        ] {
            assert!(
                METAL_MACRO_TOKEN_FIXTURE.contains(provenance),
                "missing fixture provenance: {provenance}"
            );
        }

        let fixture_pairs: Vec<_> = METAL_MACRO_TOKEN_FIXTURE
            .lines()
            .filter_map(|line| line.split_once('='))
            .filter(|(name, _)| name.starts_with("GLSL_"))
            .collect();
        assert_eq!(fixture_pairs.len(), ALL_SHADER_MACROS.len());

        let fixture_identifiers: BTreeSet<_> =
            fixture_pairs.iter().map(|(name, _)| *name).collect();
        let fixture_tokens: BTreeSet<_> = fixture_pairs.iter().map(|(_, token)| *token).collect();
        assert_eq!(fixture_identifiers.len(), ALL_SHADER_MACROS.len());
        assert_eq!(fixture_tokens.len(), ALL_SHADER_MACROS.len());

        let enum_identifiers: BTreeSet<_> = ALL_SHADER_MACROS
            .iter()
            .map(|shader_macro| shader_macro.identifier())
            .collect();
        assert_eq!(enum_identifiers, fixture_identifiers);
        for shader_macro in ALL_SHADER_MACROS {
            let expected = fixture_pairs
                .iter()
                .find_map(|(identifier, token)| {
                    (*identifier == shader_macro.identifier()).then_some(*token)
                })
                .unwrap();
            assert_eq!(shader_macro.metal_token(), expected);
        }
    }

    #[test]
    fn every_feature_macro_and_source_fragment_keeps_upstream_order() {
        let all_features = ENABLE_CLIPPING
            | ENABLE_CLIP_RECT
            | ENABLE_ADVANCED_BLEND
            | ENABLE_FEATHER
            | ENABLE_EVEN_ODD
            | ENABLE_NESTED_CLIPPING
            | ENABLE_HSL_BLEND_MODES
            | ENABLE_DITHER;
        let mut expected_defines = base_defines();
        expected_defines.extend([
            definition(ShaderMacro::EnableClipping, MacroValue::One),
            definition(ShaderMacro::EnableClipRect, MacroValue::One),
            definition(ShaderMacro::EnableAdvancedBlend, MacroValue::One),
            definition(ShaderMacro::EnableFeather, MacroValue::One),
            definition(ShaderMacro::EnableEvenOdd, MacroValue::One),
            definition(ShaderMacro::EnableNestedClipping, MacroValue::One),
            definition(ShaderMacro::EnableHslBlendModes, MacroValue::One),
            definition(ShaderMacro::EnableDither, MacroValue::One),
            definition(ShaderMacro::DrawImage, MacroValue::Empty),
            definition(ShaderMacro::DrawImageMesh, MacroValue::Empty),
        ]);
        let mut expected_sources = BASE_SOURCES.to_vec();
        expected_sources.extend([
            SourceFragment::AdvancedBlend,
            SourceFragment::DrawImageMeshVertex,
            SourceFragment::DrawMeshFragment,
        ]);
        assert_plan(
            "all features",
            BackgroundCompileJob::new(
                DrawType::ImageMesh,
                all_features,
                InterlockMode::RasterOrdering,
                0,
            ),
            MetalFeatures::default(),
            MAC,
            expected_defines,
            expected_sources,
        );
    }

    #[test]
    fn every_raster_order_draw_branch_matches_upstream() {
        let path_sources = [
            SourceFragment::DrawPathCommon,
            SourceFragment::DrawPathVertex,
            SourceFragment::DrawRasterOrderPathFragment,
        ];
        for draw_type in [
            DrawType::MidpointFanPatches,
            DrawType::MidpointFanCenterAaPatches,
            DrawType::OuterCurvePatches,
        ] {
            let mut expected_defines = base_defines();
            expected_defines.extend([
                definition(ShaderMacro::EnableInstanceIndex, MacroValue::Empty),
                definition(ShaderMacro::DrawPath, MacroValue::Empty),
            ]);
            let mut expected_sources = BASE_SOURCES.to_vec();
            expected_sources.extend(path_sources);
            assert_plan(
                "raster path",
                default_job(draw_type, InterlockMode::RasterOrdering),
                MetalFeatures::default(),
                MAC,
                expected_defines,
                expected_sources,
            );
        }

        let mut interior_defines = base_defines();
        interior_defines.push(definition(
            ShaderMacro::DrawInteriorTriangles,
            MacroValue::Empty,
        ));
        let mut interior_sources = BASE_SOURCES.to_vec();
        interior_sources.extend(path_sources);
        assert_plan(
            "interior triangulation",
            default_job(
                DrawType::InteriorTriangulation,
                InterlockMode::RasterOrdering,
            ),
            MetalFeatures::default(),
            MAC,
            interior_defines,
            interior_sources,
        );

        let mut atlas_defines = base_defines();
        atlas_defines.push(definition(ShaderMacro::FeatherAtlasBlit, MacroValue::One));
        let mut atlas_sources = BASE_SOURCES.to_vec();
        atlas_sources.extend([
            SourceFragment::DrawPathCommon,
            SourceFragment::DrawPathVertex,
            SourceFragment::DrawMeshFragment,
        ]);
        assert_plan(
            "feather atlas blit",
            default_job(DrawType::AtlasBlit, InterlockMode::RasterOrdering),
            MetalFeatures::default(),
            MAC,
            atlas_defines,
            atlas_sources,
        );

        let mut image_defines = base_defines();
        image_defines.extend([
            definition(ShaderMacro::DrawImage, MacroValue::Empty),
            definition(ShaderMacro::DrawImageMesh, MacroValue::Empty),
        ]);
        let mut image_sources = BASE_SOURCES.to_vec();
        image_sources.extend([
            SourceFragment::DrawImageMeshVertex,
            SourceFragment::DrawMeshFragment,
        ]);
        assert_plan(
            "image mesh",
            default_job(DrawType::ImageMesh, InterlockMode::RasterOrdering),
            MetalFeatures::default(),
            MAC,
            image_defines,
            image_sources,
        );
    }

    #[test]
    fn every_atomic_draw_branch_matches_upstream() {
        let atomic_sources = [SourceFragment::DrawPathCommon, SourceFragment::AtomicDraw];
        let cases: &[(DrawType, &[(ShaderMacro, MacroValue)])] = &[
            (
                DrawType::MidpointFanPatches,
                &[
                    (ShaderMacro::EnableInstanceIndex, MacroValue::Empty),
                    (ShaderMacro::DrawPath, MacroValue::Empty),
                ],
            ),
            (
                DrawType::MidpointFanCenterAaPatches,
                &[
                    (ShaderMacro::EnableInstanceIndex, MacroValue::Empty),
                    (ShaderMacro::DrawPath, MacroValue::Empty),
                ],
            ),
            (
                DrawType::OuterCurvePatches,
                &[
                    (ShaderMacro::EnableInstanceIndex, MacroValue::Empty),
                    (ShaderMacro::DrawPath, MacroValue::Empty),
                ],
            ),
            (
                DrawType::InteriorTriangulation,
                &[(ShaderMacro::DrawInteriorTriangles, MacroValue::Empty)],
            ),
            (
                DrawType::AtlasBlit,
                &[(ShaderMacro::FeatherAtlasBlit, MacroValue::One)],
            ),
            (
                DrawType::ImageRect,
                &[
                    (ShaderMacro::DrawImage, MacroValue::Empty),
                    (ShaderMacro::DrawImageRect, MacroValue::Empty),
                ],
            ),
            (
                DrawType::ImageMesh,
                &[
                    (ShaderMacro::DrawImage, MacroValue::Empty),
                    (ShaderMacro::DrawImageMesh, MacroValue::Empty),
                ],
            ),
            (
                DrawType::RenderPassInitialize,
                &[
                    (ShaderMacro::DrawRenderTargetUpdateBounds, MacroValue::Empty),
                    (ShaderMacro::InitializePls, MacroValue::Empty),
                ],
            ),
            (
                DrawType::RenderPassResolve,
                &[
                    (ShaderMacro::DrawRenderTargetUpdateBounds, MacroValue::Empty),
                    (ShaderMacro::ResolvePls, MacroValue::Empty),
                ],
            ),
        ];
        for &(draw_type, draw_defines) in cases {
            let mut expected_defines = base_defines();
            expected_defines.push(definition(
                ShaderMacro::PlsImplDeviceBuffer,
                MacroValue::Empty,
            ));
            expected_defines.extend(
                draw_defines
                    .iter()
                    .map(|&(name, value)| definition(name, value)),
            );
            let mut expected_sources = BASE_SOURCES.to_vec();
            expected_sources.extend(atomic_sources);
            assert_plan(
                "atomic draw",
                default_job(draw_type, InterlockMode::Atomics),
                MetalFeatures::default(),
                MAC,
                expected_defines,
                expected_sources,
            );
        }
    }

    #[test]
    fn every_unreachable_draw_and_interlock_combination_is_typed() {
        for draw_type in [
            DrawType::MsaaStrokes,
            DrawType::MsaaMidpointFanBorrowedCoverage,
            DrawType::MsaaMidpointFans,
            DrawType::MsaaMidpointFanStencilReset,
            DrawType::MsaaDynamicMidpointFans,
            DrawType::MsaaMidpointFanPathsStencil,
            DrawType::MsaaMidpointFanPathsCover,
            DrawType::MsaaOuterCubics,
            DrawType::ClipReset,
        ] {
            assert_eq!(
                build_shader_compile_plan(
                    default_job(draw_type, InterlockMode::RasterOrdering),
                    MetalFeatures::default(),
                    MAC,
                ),
                Err(BackgroundCompilePlanError::UnsupportedDrawType { draw_type })
            );
        }

        for interlock_mode in [
            InterlockMode::Clockwise,
            InterlockMode::ClockwiseAtomic,
            InterlockMode::Msaa,
        ] {
            assert_eq!(
                build_shader_compile_plan(
                    default_job(DrawType::ImageMesh, interlock_mode),
                    MetalFeatures::default(),
                    MAC,
                ),
                Err(BackgroundCompilePlanError::UnsupportedInterlockMode { interlock_mode })
            );
        }

        for draw_type in [
            DrawType::ImageRect,
            DrawType::RenderPassInitialize,
            DrawType::RenderPassResolve,
        ] {
            for interlock_mode in [
                InterlockMode::RasterOrdering,
                InterlockMode::Clockwise,
                InterlockMode::ClockwiseAtomic,
                InterlockMode::Msaa,
            ] {
                assert_eq!(
                    build_shader_compile_plan(
                        default_job(draw_type, interlock_mode),
                        MetalFeatures::default(),
                        MAC,
                    ),
                    Err(BackgroundCompilePlanError::DrawRequiresAtomicInterlock {
                        draw_type,
                        interlock_mode,
                    })
                );
            }
        }
    }

    #[test]
    fn ios_device_and_simulator_preprocessor_boundaries_stay_distinct() {
        for draw_type in [DrawType::MidpointFanPatches, DrawType::ImageMesh] {
            assert!(build_shader_compile_plan(
                default_job(draw_type, InterlockMode::RasterOrdering),
                MetalFeatures::default(),
                IOS_DEVICE,
            )
            .is_ok());
        }

        for draw_type in [
            DrawType::ImageRect,
            DrawType::RenderPassInitialize,
            DrawType::RenderPassResolve,
        ] {
            for interlock_mode in [InterlockMode::RasterOrdering, InterlockMode::Atomics] {
                assert_eq!(
                    build_shader_compile_plan(
                        default_job(draw_type, interlock_mode),
                        MetalFeatures::default(),
                        IOS_DEVICE,
                    ),
                    Err(BackgroundCompilePlanError::DrawUnavailableOnIos { draw_type })
                );
            }
            assert!(
                build_shader_compile_plan(
                    default_job(draw_type, InterlockMode::Atomics),
                    MetalFeatures::default(),
                    IOS_SIMULATOR,
                )
                .is_ok(),
                "{draw_type:?} must remain available to the simulator"
            );
        }

        assert_eq!(
            build_shader_compile_plan(
                default_job(DrawType::ImageMesh, InterlockMode::Atomics),
                MetalFeatures::default(),
                IOS_DEVICE,
            ),
            Err(BackgroundCompilePlanError::AtomicInterlockUnavailableOnIos)
        );
        assert!(build_shader_compile_plan(
            default_job(DrawType::ImageMesh, InterlockMode::Atomics),
            MetalFeatures::default(),
            IOS_SIMULATOR,
        )
        .is_ok());
    }

    #[test]
    fn atomic_barrier_and_misc_macro_values_match_upstream() {
        for (barrier, expects_raster_ordered_define) in [
            (AtomicBarrierType::memoryBarrier, false),
            (AtomicBarrierType::rasterOrderGroup, true),
            (AtomicBarrierType::renderPassBreak, false),
        ] {
            let plan = build_shader_compile_plan(
                BackgroundCompileJob::new(
                    DrawType::ImageMesh,
                    0,
                    InterlockMode::Atomics,
                    FIXED_FUNCTION_COLOR_OUTPUT | CLOCKWISE_FILL,
                ),
                MetalFeatures {
                    atomic_barrier_type: barrier,
                },
                MAC,
            )
            .unwrap();
            assert_eq!(
                plan.defines.iter().any(|definition| {
                    definition.name == ShaderMacro::PlsImplDeviceBufferRasterOrdered
                }),
                expects_raster_ordered_define
            );
            assert!(plan.defines.contains(&definition(
                ShaderMacro::FixedFunctionColorOutput,
                MacroValue::Empty
            )));
            assert!(plan
                .defines
                .contains(&definition(ShaderMacro::ClockwiseFill, MacroValue::One)));
        }

        let raster_plan = build_shader_compile_plan(
            BackgroundCompileJob::new(
                DrawType::ImageMesh,
                0,
                InterlockMode::RasterOrdering,
                FIXED_FUNCTION_COLOR_OUTPUT,
            ),
            MetalFeatures::default(),
            MAC,
        )
        .unwrap();
        assert!(!raster_plan
            .defines
            .iter()
            .any(|definition| definition.name == ShaderMacro::FixedFunctionColorOutput));

        let ignored_misc_plan = build_shader_compile_plan(
            BackgroundCompileJob::new(
                DrawType::ImageMesh,
                0,
                InterlockMode::RasterOrdering,
                CLIP_UPDATE_ONLY | NESTED_CLIP_UPDATE_ONLY | BORROWED_COVERAGE_PASS,
            ),
            MetalFeatures::default(),
            MAC,
        )
        .unwrap();
        assert_eq!(ignored_misc_plan, raster_plan);

        let initialize_plan = build_shader_compile_plan(
            BackgroundCompileJob::new(
                DrawType::RenderPassInitialize,
                0,
                InterlockMode::Atomics,
                STORE_COLOR_CLEAR | SWIZZLE_COLOR_BGRA_TO_RGBA,
            ),
            MetalFeatures::default(),
            MAC,
        )
        .unwrap();
        assert!(initialize_plan
            .defines
            .contains(&definition(ShaderMacro::StoreColorClear, MacroValue::True)));
        assert!(initialize_plan.defines.contains(&definition(
            ShaderMacro::SwizzleColorBgraToRgba,
            MacroValue::Empty
        )));
        assert_eq!(
            build_shader_compile_plan(
                BackgroundCompileJob::new(
                    DrawType::RenderPassInitialize,
                    0,
                    InterlockMode::Atomics,
                    LOAD_COLOR_FROM_DST_TEXTURE,
                ),
                MetalFeatures::default(),
                MAC,
            ),
            Err(BackgroundCompilePlanError::LoadColorFromDstTextureUnsupported)
        );

        let resolve_plan = build_shader_compile_plan(
            BackgroundCompileJob::new(
                DrawType::RenderPassResolve,
                0,
                InterlockMode::Atomics,
                COALESCED_RESOLVE_AND_TRANSFER,
            ),
            MetalFeatures::default(),
            MAC,
        )
        .unwrap();
        assert!(resolve_plan.defines.contains(&definition(
            ShaderMacro::CoalescedPlsResolveAndTransfer,
            MacroValue::Empty
        )));
    }

    #[test]
    fn all_source_symbols_remain_exact() {
        assert_eq!(
            [
                SourceFragment::Metal.symbol(),
                SourceFragment::Constants.symbol(),
                SourceFragment::FlushUniforms.symbol(),
                SourceFragment::Common.symbol(),
                SourceFragment::AdvancedBlend.symbol(),
                SourceFragment::DrawPathCommon.symbol(),
                SourceFragment::DrawPathVertex.symbol(),
                SourceFragment::DrawRasterOrderPathFragment.symbol(),
                SourceFragment::AtomicDraw.symbol(),
                SourceFragment::DrawImageMeshVertex.symbol(),
                SourceFragment::DrawMeshFragment.symbol(),
            ],
            [
                "gpu::glsl::metal",
                "gpu::glsl::constants",
                "gpu::glsl::flush_uniforms",
                "gpu::glsl::common",
                "gpu::glsl::advanced_blend",
                "gpu::glsl::draw_path_common",
                "gpu::glsl::draw_path_vert",
                "gpu::glsl::draw_raster_order_path_frag",
                "gpu::glsl::atomic_draw",
                "gpu::glsl::draw_image_mesh_vert",
                "gpu::glsl::draw_mesh_frag",
            ]
        );
        assert_eq!(
            ShaderMacro::PlsImplDeviceBufferRasterOrdered.identifier(),
            "GLSL_PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED"
        );
    }
}
