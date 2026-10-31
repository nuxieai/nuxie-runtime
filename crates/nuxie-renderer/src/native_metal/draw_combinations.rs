//! Mechanical translation of the pinned upstream Metal draw-combination
//! generator, cross-checked against the shared precompiled-function namespace.
//!
//! Source: rive-runtime `74e5d3f635c3c463fe10e4327261705549ee5919`,
//! `renderer/src/shaders/metal/generate_draw_combinations.py` and
//! `renderer/src/metal/render_context_metal_impl.mm`.
//!
//! The upstream generator iterates Python sets of identity-hashed `Feature`
//! objects. Ordered arrays below preserve the exact emission captured by pool
//! job `8eb47d12-0906-4ce1-8828-f76a6d84f51f`, checked against its provenance
//! JSON. The output SHA-256 is
//! `f7ad64879b30b6738cdd6889eaa3a0a7418d456df87cf28cef5069cdd1ca7185`.
//! Separate Python processes may emit equivalent sets in different orders;
//! the exact-byte fixture records this emission, not deterministic upstream
//! generation. The earlier 3ed35ee0/d3ec0fcb fixture remains historical evidence.

use super::pipeline_names::{
    DRAW_INTERIOR_TRIANGLES, ENABLE_ADVANCED_BLEND, ENABLE_ANGULAR_GRADIENT, ENABLE_CLIP_RECT,
    ENABLE_CLIPPING, ENABLE_DITHER, ENABLE_EVEN_ODD, ENABLE_FEATHER, ENABLE_HSL_BLEND_MODES,
    ENABLE_MODULATED_IMAGE, ENABLE_NESTED_CLIPPING, FEATHER_ATLAS_BLIT,
};

const WHOLE_PROGRAM_FEATURES: u32 = ENABLE_CLIPPING
    | ENABLE_CLIP_RECT
    | ENABLE_ADVANCED_BLEND
    | ENABLE_FEATHER
    | ENABLE_MODULATED_IMAGE;
const FRAGMENT_ONLY_FEATURES: u32 = ENABLE_EVEN_ODD
    | ENABLE_NESTED_CLIPPING
    | ENABLE_HSL_BLEND_MODES
    | ENABLE_DITHER
    | ENABLE_ANGULAR_GRADIENT;
const ALL_FEATURES: u32 = WHOLE_PROGRAM_FEATURES | FRAGMENT_ONLY_FEATURES;
const NON_ATLAS_COVERAGE_FEATURES: u32 = ENABLE_FEATHER | ENABLE_EVEN_ODD | ENABLE_NESTED_CLIPPING;
const NON_IMAGE_MESH_FEATURES: u32 = ENABLE_FEATHER
    | ENABLE_EVEN_ODD
    | ENABLE_NESTED_CLIPPING
    | DRAW_INTERIOR_TRIANGLES
    | FEATHER_ATLAS_BLIT
    | ENABLE_MODULATED_IMAGE
    | ENABLE_ANGULAR_GRADIENT;

/// Upstream's `Feature` value, retained to preserve emitted `#define` order.
#[derive(Clone, Copy, Debug)]
struct Feature {
    name: &'static str,
    index: usize,
    bit: u32,
}

const FEATURE_CLIPPING: Feature = Feature {
    name: "ENABLE_CLIPPING",
    index: 0,
    bit: ENABLE_CLIPPING,
};
const FEATURE_CLIP_RECT: Feature = Feature {
    name: "ENABLE_CLIP_RECT",
    index: 1,
    bit: ENABLE_CLIP_RECT,
};
const FEATURE_ADVANCED_BLEND: Feature = Feature {
    name: "ENABLE_ADVANCED_BLEND",
    index: 2,
    bit: ENABLE_ADVANCED_BLEND,
};
const FEATURE_FEATHER: Feature = Feature {
    name: "ENABLE_FEATHER",
    index: 3,
    bit: ENABLE_FEATHER,
};
const FEATURE_EVEN_ODD: Feature = Feature {
    name: "ENABLE_EVEN_ODD",
    index: 5,
    bit: ENABLE_EVEN_ODD,
};
const FEATURE_NESTED_CLIPPING: Feature = Feature {
    name: "ENABLE_NESTED_CLIPPING",
    index: 6,
    bit: ENABLE_NESTED_CLIPPING,
};
const FEATURE_HSL_BLEND_MODES: Feature = Feature {
    name: "ENABLE_HSL_BLEND_MODES",
    index: 7,
    bit: ENABLE_HSL_BLEND_MODES,
};
const FEATURE_DITHER: Feature = Feature {
    name: "ENABLE_DITHER",
    index: 8,
    bit: ENABLE_DITHER,
};
const FEATURE_MODULATED_IMAGE: Feature = Feature {
    name: "ENABLE_MODULATED_IMAGE",
    index: 4,
    bit: ENABLE_MODULATED_IMAGE,
};
const FEATURE_ANGULAR_GRADIENT: Feature = Feature {
    name: "ENABLE_ANGULAR_GRADIENT",
    index: 9,
    bit: ENABLE_ANGULAR_GRADIENT,
};
const FEATURE_INTERIOR_TRIANGLES: Feature = Feature {
    name: "DRAW_INTERIOR_TRIANGLES",
    index: 10,
    bit: DRAW_INTERIOR_TRIANGLES,
};
const FEATURE_ATLAS_BLIT: Feature = Feature {
    name: "FEATHER_ATLAS_BLIT",
    index: 11,
    bit: FEATHER_ATLAS_BLIT,
};

/// Returns whether a valid program exists for the given feature set.
pub(crate) const fn is_valid_feature_set(feature_set: u32) -> bool {
    if feature_set & ENABLE_NESTED_CLIPPING != 0 && feature_set & ENABLE_CLIPPING == 0 {
        return false;
    }
    if feature_set & ENABLE_HSL_BLEND_MODES != 0 && feature_set & ENABLE_ADVANCED_BLEND == 0 {
        return false;
    }
    true
}

/// Returns whether a feature set is the simplest set defining a unique vertex
/// shader. Fragment-only features have no effect on the vertex shader.
pub(crate) const fn is_unique_vertex_feature_set(feature_set: u32) -> bool {
    feature_set & FRAGMENT_ONLY_FEATURES == 0
}

/// Returns whether a feature set is compatible with an image mesh shader.
pub(crate) const fn is_image_mesh_feature_set(feature_set: u32) -> bool {
    feature_set & NON_IMAGE_MESH_FEATURES == 0
}

/// Construct the twelve-character namespace identifier used by the generator.
pub(crate) fn namespace_id(feature_set: u32) -> String {
    let mut id = ['0'; 12];
    for feature in ALL_FEATURES_ARRAY
        .iter()
        .chain([FEATURE_INTERIOR_TRIANGLES, FEATURE_ATLAS_BLIT].iter())
    {
        if feature_set & feature.bit != 0 {
            id[feature.index] = '1';
        }
    }
    id.iter().collect()
}

/// The shader kind emitted by the upstream generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShaderType {
    Vertex,
    Fragment,
}

/// The two draw kinds understood by the upstream generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CombinationDrawType {
    Path,
    ImageMesh,
}

/// The fill convention used by a generated path namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FillType {
    Clockwise,
    Legacy,
}

/// One emitted shader namespace in the pinned upstream inventory.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DrawCombination {
    pub(crate) shader_type: ShaderType,
    pub(crate) draw_type: CombinationDrawType,
    pub(crate) fill_type: FillType,
    pub(crate) feature_set: u32,
    feature_order: &'static [Feature],
}

const PATH_VERTEX_FEATURES: [Feature; 5] = [
    FEATURE_MODULATED_IMAGE,
    FEATURE_CLIP_RECT,
    FEATURE_ADVANCED_BLEND,
    FEATURE_FEATHER,
    FEATURE_CLIPPING,
];
const PATH_FRAGMENT_FEATURES: [Feature; 10] = [
    FEATURE_MODULATED_IMAGE,
    FEATURE_EVEN_ODD,
    FEATURE_CLIP_RECT,
    FEATURE_DITHER,
    FEATURE_ADVANCED_BLEND,
    FEATURE_NESTED_CLIPPING,
    FEATURE_FEATHER,
    FEATURE_HSL_BLEND_MODES,
    FEATURE_ANGULAR_GRADIENT,
    FEATURE_CLIPPING,
];
const INTERIOR_VERTEX_FEATURES: [Feature; 6] = [
    FEATURE_INTERIOR_TRIANGLES,
    FEATURE_MODULATED_IMAGE,
    FEATURE_CLIPPING,
    FEATURE_FEATHER,
    FEATURE_CLIP_RECT,
    FEATURE_ADVANCED_BLEND,
];
const INTERIOR_FRAGMENT_FEATURES: [Feature; 11] = [
    FEATURE_MODULATED_IMAGE,
    FEATURE_EVEN_ODD,
    FEATURE_CLIP_RECT,
    FEATURE_DITHER,
    FEATURE_ADVANCED_BLEND,
    FEATURE_INTERIOR_TRIANGLES,
    FEATURE_NESTED_CLIPPING,
    FEATURE_FEATHER,
    FEATURE_HSL_BLEND_MODES,
    FEATURE_ANGULAR_GRADIENT,
    FEATURE_CLIPPING,
];
const ATLAS_VERTEX_FEATURES: [Feature; 6] = [
    FEATURE_MODULATED_IMAGE,
    FEATURE_CLIP_RECT,
    FEATURE_ADVANCED_BLEND,
    FEATURE_INTERIOR_TRIANGLES,
    FEATURE_ATLAS_BLIT,
    FEATURE_CLIPPING,
];
const ATLAS_FRAGMENT_FEATURES: [Feature; 9] = [
    FEATURE_MODULATED_IMAGE,
    FEATURE_CLIP_RECT,
    FEATURE_ADVANCED_BLEND,
    FEATURE_INTERIOR_TRIANGLES,
    FEATURE_HSL_BLEND_MODES,
    FEATURE_DITHER,
    FEATURE_ATLAS_BLIT,
    FEATURE_ANGULAR_GRADIENT,
    FEATURE_CLIPPING,
];
const IMAGE_VERTEX_FEATURES: [Feature; 3] =
    [FEATURE_CLIPPING, FEATURE_ADVANCED_BLEND, FEATURE_CLIP_RECT];
const IMAGE_FRAGMENT_FEATURES: [Feature; 5] = [
    FEATURE_CLIP_RECT,
    FEATURE_ADVANCED_BLEND,
    FEATURE_HSL_BLEND_MODES,
    FEATURE_DITHER,
    FEATURE_CLIPPING,
];

/// The ten namespaces emitted by the pinned Python generator.
pub(crate) const DRAW_COMBINATIONS: [DrawCombination; 10] = [
    DrawCombination {
        shader_type: ShaderType::Vertex,
        draw_type: CombinationDrawType::Path,
        fill_type: FillType::Legacy,
        feature_set: WHOLE_PROGRAM_FEATURES,
        feature_order: &PATH_VERTEX_FEATURES,
    },
    DrawCombination {
        shader_type: ShaderType::Fragment,
        draw_type: CombinationDrawType::Path,
        fill_type: FillType::Legacy,
        feature_set: ALL_FEATURES,
        feature_order: &PATH_FRAGMENT_FEATURES,
    },
    DrawCombination {
        shader_type: ShaderType::Fragment,
        draw_type: CombinationDrawType::Path,
        fill_type: FillType::Clockwise,
        feature_set: ALL_FEATURES,
        feature_order: &PATH_FRAGMENT_FEATURES,
    },
    DrawCombination {
        shader_type: ShaderType::Vertex,
        draw_type: CombinationDrawType::Path,
        fill_type: FillType::Legacy,
        feature_set: WHOLE_PROGRAM_FEATURES | DRAW_INTERIOR_TRIANGLES,
        feature_order: &INTERIOR_VERTEX_FEATURES,
    },
    DrawCombination {
        shader_type: ShaderType::Fragment,
        draw_type: CombinationDrawType::Path,
        fill_type: FillType::Legacy,
        feature_set: ALL_FEATURES | DRAW_INTERIOR_TRIANGLES,
        feature_order: &INTERIOR_FRAGMENT_FEATURES,
    },
    DrawCombination {
        shader_type: ShaderType::Fragment,
        draw_type: CombinationDrawType::Path,
        fill_type: FillType::Clockwise,
        feature_set: ALL_FEATURES | DRAW_INTERIOR_TRIANGLES,
        feature_order: &INTERIOR_FRAGMENT_FEATURES,
    },
    DrawCombination {
        shader_type: ShaderType::Vertex,
        draw_type: CombinationDrawType::Path,
        fill_type: FillType::Legacy,
        feature_set: (WHOLE_PROGRAM_FEATURES | DRAW_INTERIOR_TRIANGLES | FEATHER_ATLAS_BLIT)
            & !NON_ATLAS_COVERAGE_FEATURES,
        feature_order: &ATLAS_VERTEX_FEATURES,
    },
    DrawCombination {
        shader_type: ShaderType::Fragment,
        draw_type: CombinationDrawType::Path,
        fill_type: FillType::Legacy,
        feature_set: (ALL_FEATURES | DRAW_INTERIOR_TRIANGLES | FEATHER_ATLAS_BLIT)
            & !NON_ATLAS_COVERAGE_FEATURES,
        feature_order: &ATLAS_FRAGMENT_FEATURES,
    },
    DrawCombination {
        shader_type: ShaderType::Vertex,
        draw_type: CombinationDrawType::ImageMesh,
        fill_type: FillType::Legacy,
        feature_set: WHOLE_PROGRAM_FEATURES & !NON_IMAGE_MESH_FEATURES,
        feature_order: &IMAGE_VERTEX_FEATURES,
    },
    DrawCombination {
        shader_type: ShaderType::Fragment,
        draw_type: CombinationDrawType::ImageMesh,
        fill_type: FillType::Legacy,
        feature_set: ALL_FEATURES & !NON_IMAGE_MESH_FEATURES,
        feature_order: &IMAGE_FRAGMENT_FEATURES,
    },
];

fn emit_shader(out: &mut String, combination: DrawCombination) {
    debug_assert!(is_valid_feature_set(combination.feature_set));
    if combination.shader_type == ShaderType::Vertex {
        debug_assert!(is_unique_vertex_feature_set(combination.feature_set));
        out.push_str("#define VERTEX\n");
    } else {
        out.push_str("#define FRAGMENT\n");
    }
    if combination.draw_type == CombinationDrawType::ImageMesh {
        debug_assert!(is_image_mesh_feature_set(combination.feature_set));
    }

    for feature in combination.feature_order {
        debug_assert!(combination.feature_set & feature.bit != 0);
        out.push_str("#define ");
        out.push_str(feature.name);
        out.push_str(" 1\n");
    }
    if combination.fill_type == FillType::Clockwise {
        out.push_str("#define CLOCKWISE_FILL 1\n");
    }
    match combination.draw_type {
        CombinationDrawType::Path => {
            out.push_str("#define DRAW_PATH 1\nnamespace ");
            out.push(if combination.fill_type == FillType::Clockwise {
                'c'
            } else {
                'p'
            });
            out.push_str(&namespace_id(combination.feature_set));
            out.push_str("\n{\n#include \"gradient_packing_common.minified.glsl\"\n#include \"draw_path.minified.vert\"\n#include \"");
            if combination.feature_set & FEATHER_ATLAS_BLIT != 0 {
                out.push_str("draw_mesh.minified.frag");
            } else {
                out.push_str("draw_raster_order_path.minified.frag");
            }
            out.push_str("\"\n}\n#undef DRAW_PATH\n");
        }
        CombinationDrawType::ImageMesh => {
            out.push_str("#define DRAW_IMAGE 1\n#define DRAW_IMAGE_MESH 1\nnamespace m");
            out.push_str(&namespace_id(combination.feature_set));
            out.push_str(
                "\n{\n#include \"draw_image_mesh.minified.vert\"\n#include \"draw_mesh.minified.frag\"\n}\n#undef DRAW_IMAGE_MESH\n#undef DRAW_IMAGE\n",
            );
        }
    }
    for feature in combination.feature_order {
        out.push_str("#undef ");
        out.push_str(feature.name);
        out.push('\n');
    }
    if combination.shader_type == ShaderType::Vertex {
        out.push_str("#undef VERTEX\n");
    } else {
        out.push_str("#undef FRAGMENT\n");
    }
    if combination.fill_type == FillType::Clockwise {
        out.push_str("#undef CLOCKWISE_FILL\n");
    }
    out.push('\n');
}

/// Emit the complete precompiled Metal shader-combination source.
pub(crate) fn generate_draw_combinations() -> String {
    let mut out = String::new();
    for combination in DRAW_COMBINATIONS {
        emit_shader(&mut out, combination);
    }
    out
}

/// Alias emphasizing that this is the source consumed by the Metal compiler.
pub(crate) fn generated_shader_source() -> String {
    generate_draw_combinations()
}

const ALL_FEATURES_ARRAY: [Feature; 10] = [
    FEATURE_CLIPPING,
    FEATURE_CLIP_RECT,
    FEATURE_ADVANCED_BLEND,
    FEATURE_FEATHER,
    FEATURE_EVEN_ODD,
    FEATURE_NESTED_CLIPPING,
    FEATURE_HSL_BLEND_MODES,
    FEATURE_DITHER,
    FEATURE_MODULATED_IMAGE,
    FEATURE_ANGULAR_GRADIENT,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::DrawType;
    use crate::native_metal::pipeline_names::{CLOCKWISE_FILL, precompiled_function_name};

    // The raw fixture deliberately has no provenance header because the test
    // compares every captured upstream byte. Its provenance and the reason a
    // single capture is necessary are recorded in this module's top-level docs.
    const CAPTURED_UPSTREAM_SOURCE: &str =
        include_str!("../../tests/fixtures/native_metal/draw_combinations-74e5d3f6-f7ad6487.metal");

    #[test]
    fn generated_source_matches_pinned_upstream_oracle() {
        let source = generate_draw_combinations();
        let namespace_count = source.matches("namespace ").count();
        assert_eq!(namespace_count, 10);
        assert_eq!(source.as_bytes(), CAPTURED_UPSTREAM_SOURCE.as_bytes());
    }

    #[test]
    fn inventory_has_upstream_namespace_identifiers() {
        let source = generate_draw_combinations();
        for id in [
            "p111110000000",
            "p111111111100",
            "c111111111100",
            "p111110000010",
            "p111111111110",
            "c111111111110",
            "p111010000011",
            "p111010011111",
            "m111000000000",
            "m111000011000",
        ] {
            assert!(source.contains(&format!("namespace {id}")), "missing {id}");
        }
    }

    #[test]
    fn rejects_nested_clipping_without_clipping() {
        assert!(!is_valid_feature_set(ENABLE_NESTED_CLIPPING));
        assert!(!is_valid_feature_set(
            ENABLE_NESTED_CLIPPING | ENABLE_HSL_BLEND_MODES | ENABLE_ADVANCED_BLEND
        ));
        assert!(is_valid_feature_set(
            ENABLE_CLIPPING | ENABLE_NESTED_CLIPPING
        ));
    }

    #[test]
    fn rejects_hsl_without_advanced_blend() {
        assert!(!is_valid_feature_set(ENABLE_HSL_BLEND_MODES));
        assert!(!is_valid_feature_set(
            ENABLE_CLIPPING | ENABLE_HSL_BLEND_MODES
        ));
        assert!(is_valid_feature_set(
            ENABLE_ADVANCED_BLEND | ENABLE_HSL_BLEND_MODES
        ));
    }

    #[test]
    fn fragment_only_features_are_not_unique_vertex_sets() {
        assert!(is_unique_vertex_feature_set(WHOLE_PROGRAM_FEATURES));
        assert!(!is_unique_vertex_feature_set(
            WHOLE_PROGRAM_FEATURES | ENABLE_EVEN_ODD
        ));
    }

    #[test]
    fn precompiled_names_match_upstream_namespace_construction() {
        assert_eq!(
            precompiled_function_name(DrawType::MidpointFanPatches, 0, 0, "drawPath"),
            Some("p000000000000::drawPath".to_owned())
        );
        assert_eq!(
            precompiled_function_name(
                DrawType::OuterCurvePatches,
                ALL_FEATURES,
                CLOCKWISE_FILL,
                "drawOuter"
            ),
            Some("c111111111100::drawOuter".to_owned())
        );
        assert_eq!(
            precompiled_function_name(DrawType::InteriorTriangulation, 0, 0, "drawInterior"),
            Some("p000000000010::drawInterior".to_owned())
        );
        assert_eq!(
            precompiled_function_name(DrawType::AtlasBlit, 0, 0, "drawFeatherAtlas"),
            Some("p000000000011::drawFeatherAtlas".to_owned())
        );
        assert_eq!(
            precompiled_function_name(DrawType::ImageMesh, ENABLE_CLIP_RECT, 0, "drawImageMesh"),
            Some("m010000000000::drawImageMesh".to_owned())
        );
        assert_eq!(
            precompiled_function_name(DrawType::RenderPassResolve, 0, 0, "resolve"),
            None
        );
        assert_eq!(
            precompiled_function_name(DrawType::StencilDynamicMidpointFans, 0, 0, "drawPath"),
            None
        );
    }
}
