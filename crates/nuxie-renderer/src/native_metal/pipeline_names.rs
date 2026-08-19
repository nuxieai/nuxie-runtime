//! Names for the precompiled Metal draw functions.
//!
//! This is a direct Rust translation of `DrawPipeline::GetPrecompiledFunctionName`
//! in upstream `renderer/src/metal/render_context_metal_impl.mm` pinned at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
//! The bit positions are part of the generated-shader namespace contract and
//! must stay in sync with upstream `generate_draw_combinations.py`.

use super::super::gpu::DrawType;

/// Number of feature bits in an upstream Metal draw-function namespace.
pub(crate) const SHADER_FEATURE_COUNT: usize = 8;

// `ShaderFeatures` is not yet represented as a Rust type. Keep the upstream
// bit positions explicit until the common platform-neutral shader-key type is
// introduced; callers pass the same mask values used by the C++ backend.
pub(crate) const ENABLE_CLIPPING: u32 = 1 << 0;
pub(crate) const ENABLE_CLIP_RECT: u32 = 1 << 1;
pub(crate) const ENABLE_ADVANCED_BLEND: u32 = 1 << 2;
pub(crate) const ENABLE_FEATHER: u32 = 1 << 3;
pub(crate) const ENABLE_EVEN_ODD: u32 = 1 << 4;
pub(crate) const ENABLE_NESTED_CLIPPING: u32 = 1 << 5;
pub(crate) const ENABLE_HSL_BLEND_MODES: u32 = 1 << 6;
pub(crate) const ENABLE_DITHER: u32 = 1 << 7;

/// Upstream `ShaderMiscFlags::clockwiseFill`.
pub(crate) const CLOCKWISE_FILL: u32 = 1 << 1;

/// Return the fully qualified name of a precompiled Metal draw function.
///
/// Upstream uses `RIVE_UNREACHABLE` for draw types that do not have a
/// precompiled path/image function. Rust callers get `None` for those values,
/// so the unsupported case remains explicit while preserving the same names
/// for every supported draw type.
pub(crate) fn precompiled_function_name(
    draw_type: DrawType,
    shader_features: u32,
    shader_misc_flags: u32,
    function_base_name: &str,
) -> Option<String> {
    // Each feature corresponds to a specific index in namespaceID. These must
    // stay in sync with upstream `generate_draw_combinations.py`.
    let mut namespace_id = ['0'; SHADER_FEATURE_COUNT + 2];
    for (index, character) in namespace_id[..SHADER_FEATURE_COUNT].iter_mut().enumerate() {
        if shader_features & (1 << index) != 0 {
            *character = '1';
        }
    }

    let namespace_prefix = match draw_type {
        DrawType::MidpointFanPatches
        | DrawType::MidpointFanCenterAaPatches
        | DrawType::OuterCurvePatches
        | DrawType::InteriorTriangulation
        // Rust's `AtlasBlit` is upstream's `featherAtlasBlit`.
        | DrawType::AtlasBlit => {
            if shader_misc_flags & CLOCKWISE_FILL != 0 {
                'c'
            } else {
                'p'
            }
        }
        DrawType::ImageMesh => 'm',
        DrawType::ImageRect
        | DrawType::MsaaStrokes
        | DrawType::MsaaMidpointFanBorrowedCoverage
        | DrawType::MsaaMidpointFans
        | DrawType::MsaaMidpointFanStencilReset
        | DrawType::MsaaMidpointFanPathsStencil
        | DrawType::MsaaMidpointFanPathsCover
        | DrawType::MsaaOuterCubics
        | DrawType::ClipReset
        | DrawType::RenderPassInitialize
        | DrawType::RenderPassResolve => return None,
    };

    if draw_type == DrawType::InteriorTriangulation {
        namespace_id[SHADER_FEATURE_COUNT] = '1';
    } else if draw_type == DrawType::AtlasBlit {
        namespace_id[SHADER_FEATURE_COUNT] = '1';
        namespace_id[SHADER_FEATURE_COUNT + 1] = '1';
    }

    let namespace_id: String = namespace_id.into_iter().collect();
    Some(format!(
        "{namespace_prefix}{namespace_id}::{function_base_name}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precompiled_names_match_upstream_namespace_table() {
        let cases = [
            (
                "path",
                DrawType::MidpointFanPatches,
                0,
                0,
                "p0000000000::drawPath",
            ),
            (
                "clockwise path with all shader features",
                DrawType::OuterCurvePatches,
                ENABLE_CLIPPING
                    | ENABLE_CLIP_RECT
                    | ENABLE_ADVANCED_BLEND
                    | ENABLE_FEATHER
                    | ENABLE_EVEN_ODD
                    | ENABLE_NESTED_CLIPPING
                    | ENABLE_HSL_BLEND_MODES
                    | ENABLE_DITHER,
                CLOCKWISE_FILL,
                "c1111111100::drawOuter",
            ),
            (
                "interior triangulation",
                DrawType::InteriorTriangulation,
                0,
                0,
                "p0000000010::drawInterior",
            ),
            (
                "feather atlas blit",
                DrawType::AtlasBlit,
                0,
                0,
                "p0000000011::drawFeatherAtlas",
            ),
            (
                "clockwise feather atlas blit",
                DrawType::AtlasBlit,
                ENABLE_FEATHER,
                CLOCKWISE_FILL,
                "c0001000011::drawFeatherAtlas",
            ),
            (
                "image mesh",
                DrawType::ImageMesh,
                ENABLE_CLIP_RECT,
                0,
                "m0100000000::drawImageMesh",
            ),
        ];

        for (label, draw_type, features, misc, expected) in cases {
            assert_eq!(
                precompiled_function_name(
                    draw_type,
                    features,
                    misc,
                    expected.rsplit("::").next().unwrap()
                ),
                Some(expected.to_owned()),
                "{label}"
            );
        }
    }

    #[test]
    fn unsupported_draw_types_do_not_claim_precompiled_functions() {
        assert_eq!(
            precompiled_function_name(DrawType::RenderPassResolve, 0, 0, "resolve"),
            None
        );
    }
}
