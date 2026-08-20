//! Mechanical translation of the pinned upstream Metal draw-combination
//! generator, cross-checked against the shared precompiled-function namespace
//! lookup owned by `pipeline_names`.
//!
//! Sources, pinned to rive-runtime commit
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`:
//!
//! - `renderer/src/shaders/metal/generate_draw_combinations.py:1-161`
//! - `renderer/src/metal/render_context_metal_impl.mm:183-261`
//!
//! The upstream generator iterates Python sets of identity-hashed `Feature`
//! objects. The inventory below records the ordering of a captured upstream
//! generator emission whose full SHA-256 is
//! `9f33bdcd7b8831c0654848677d5270698069f98ceebf698c28790d4b32ffed7c`.
//! It was captured by running
//! `python3 renderer/src/shaders/metal/generate_draw_combinations.py <output>`
//! from the pinned checkout. Because `Feature` keeps identity hashing, separate
//! Python processes can emit the same feature sets in different textual orders;
//! the fixture preserves this one exact upstream emission rather than claiming
//! that the upstream script has deterministic bytes.

use super::pipeline_names::{
    DRAW_INTERIOR_TRIANGLES, ENABLE_ADVANCED_BLEND, ENABLE_CLIPPING, ENABLE_CLIP_RECT,
    ENABLE_DITHER, ENABLE_EVEN_ODD, ENABLE_FEATHER, ENABLE_HSL_BLEND_MODES, ENABLE_NESTED_CLIPPING,
    FEATHER_ATLAS_BLIT,
};

const WHOLE_PROGRAM_FEATURES: u32 =
    ENABLE_CLIPPING | ENABLE_CLIP_RECT | ENABLE_ADVANCED_BLEND | ENABLE_FEATHER;
const FRAGMENT_ONLY_FEATURES: u32 =
    ENABLE_EVEN_ODD | ENABLE_NESTED_CLIPPING | ENABLE_HSL_BLEND_MODES | ENABLE_DITHER;
const ALL_FEATURES: u32 = WHOLE_PROGRAM_FEATURES | FRAGMENT_ONLY_FEATURES;
const NON_ATLAS_COVERAGE_FEATURES: u32 = ENABLE_FEATHER | ENABLE_EVEN_ODD | ENABLE_NESTED_CLIPPING;
const NON_IMAGE_MESH_FEATURES: u32 = ENABLE_FEATHER
    | ENABLE_EVEN_ODD
    | ENABLE_NESTED_CLIPPING
    | DRAW_INTERIOR_TRIANGLES
    | FEATHER_ATLAS_BLIT;

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
    index: 4,
    bit: ENABLE_EVEN_ODD,
};
const FEATURE_NESTED_CLIPPING: Feature = Feature {
    name: "ENABLE_NESTED_CLIPPING",
    index: 5,
    bit: ENABLE_NESTED_CLIPPING,
};
const FEATURE_HSL_BLEND_MODES: Feature = Feature {
    name: "ENABLE_HSL_BLEND_MODES",
    index: 6,
    bit: ENABLE_HSL_BLEND_MODES,
};
const FEATURE_DITHER: Feature = Feature {
    name: "ENABLE_DITHER",
    index: 7,
    bit: ENABLE_DITHER,
};
const FEATURE_INTERIOR_TRIANGLES: Feature = Feature {
    name: "DRAW_INTERIOR_TRIANGLES",
    index: 8,
    bit: DRAW_INTERIOR_TRIANGLES,
};
const FEATURE_ATLAS_BLIT: Feature = Feature {
    name: "FEATHER_ATLAS_BLIT",
    index: 9,
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

/// Construct the ten-character namespace identifier used by the generator.
pub(crate) fn namespace_id(feature_set: u32) -> String {
    let mut id = ['0'; 10];
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

const PATH_VERTEX_FEATURES: [Feature; 4] = [
    FEATURE_CLIP_RECT,
    FEATURE_ADVANCED_BLEND,
    FEATURE_FEATHER,
    FEATURE_CLIPPING,
];
const PATH_FRAGMENT_FEATURES: [Feature; 8] = [
    FEATURE_CLIPPING,
    FEATURE_CLIP_RECT,
    FEATURE_NESTED_CLIPPING,
    FEATURE_DITHER,
    FEATURE_FEATHER,
    FEATURE_HSL_BLEND_MODES,
    FEATURE_EVEN_ODD,
    FEATURE_ADVANCED_BLEND,
];
const INTERIOR_VERTEX_FEATURES: [Feature; 5] = [
    FEATURE_CLIPPING,
    FEATURE_FEATHER,
    FEATURE_INTERIOR_TRIANGLES,
    FEATURE_CLIP_RECT,
    FEATURE_ADVANCED_BLEND,
];
const INTERIOR_FRAGMENT_FEATURES: [Feature; 9] = [
    FEATURE_CLIPPING,
    FEATURE_INTERIOR_TRIANGLES,
    FEATURE_CLIP_RECT,
    FEATURE_NESTED_CLIPPING,
    FEATURE_DITHER,
    FEATURE_FEATHER,
    FEATURE_HSL_BLEND_MODES,
    FEATURE_EVEN_ODD,
    FEATURE_ADVANCED_BLEND,
];
const ATLAS_VERTEX_FEATURES: [Feature; 5] = [
    FEATURE_CLIPPING,
    FEATURE_INTERIOR_TRIANGLES,
    FEATURE_CLIP_RECT,
    FEATURE_ATLAS_BLIT,
    FEATURE_ADVANCED_BLEND,
];
const ATLAS_FRAGMENT_FEATURES: [Feature; 7] = [
    FEATURE_INTERIOR_TRIANGLES,
    FEATURE_CLIP_RECT,
    FEATURE_DITHER,
    FEATURE_ATLAS_BLIT,
    FEATURE_ADVANCED_BLEND,
    FEATURE_HSL_BLEND_MODES,
    FEATURE_CLIPPING,
];
const IMAGE_VERTEX_FEATURES: [Feature; 3] =
    [FEATURE_CLIP_RECT, FEATURE_ADVANCED_BLEND, FEATURE_CLIPPING];
const IMAGE_FRAGMENT_FEATURES: [Feature; 5] = [
    FEATURE_CLIPPING,
    FEATURE_CLIP_RECT,
    FEATURE_DITHER,
    FEATURE_HSL_BLEND_MODES,
    FEATURE_ADVANCED_BLEND,
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
            out.push_str("\n{\n#include \"draw_path.minified.vert\"\n#include \"");
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

const ALL_FEATURES_ARRAY: [Feature; 8] = [
    FEATURE_CLIPPING,
    FEATURE_CLIP_RECT,
    FEATURE_ADVANCED_BLEND,
    FEATURE_FEATHER,
    FEATURE_EVEN_ODD,
    FEATURE_NESTED_CLIPPING,
    FEATURE_HSL_BLEND_MODES,
    FEATURE_DITHER,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::DrawType;
    use crate::native_metal::pipeline_names::{precompiled_function_name, CLOCKWISE_FILL};

    // The raw fixture deliberately has no provenance header because the test
    // compares every captured upstream byte. Its provenance and the reason a
    // single capture is necessary are recorded in this module's top-level docs.
    const CAPTURED_UPSTREAM_SOURCE: &str =
        include_str!("../../tests/fixtures/native_metal/draw_combinations-4ac7b327-9f33bdcd.metal");

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
            "p1111000000",
            "p1111111100",
            "c1111111100",
            "p1111000010",
            "p1111111110",
            "c1111111110",
            "p1110000011",
            "p1110001111",
            "m1110000000",
            "m1110001100",
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
            Some("p0000000000::drawPath".to_owned())
        );
        assert_eq!(
            precompiled_function_name(
                DrawType::OuterCurvePatches,
                ALL_FEATURES,
                CLOCKWISE_FILL,
                "drawOuter"
            ),
            Some("c1111111100::drawOuter".to_owned())
        );
        assert_eq!(
            precompiled_function_name(DrawType::InteriorTriangulation, 0, 0, "drawInterior"),
            Some("p0000000010::drawInterior".to_owned())
        );
        assert_eq!(
            precompiled_function_name(DrawType::AtlasBlit, 0, 0, "drawFeatherAtlas"),
            Some("p0000000011::drawFeatherAtlas".to_owned())
        );
        assert_eq!(
            precompiled_function_name(DrawType::ImageMesh, ENABLE_CLIP_RECT, 0, "drawImageMesh"),
            Some("m0100000000::drawImageMesh".to_owned())
        );
        assert_eq!(
            precompiled_function_name(DrawType::RenderPassResolve, 0, 0, "resolve"),
            None
        );
        assert_eq!(
            precompiled_function_name(DrawType::MsaaDynamicMidpointFans, 0, 0, "drawPath"),
            None
        );
    }
}
