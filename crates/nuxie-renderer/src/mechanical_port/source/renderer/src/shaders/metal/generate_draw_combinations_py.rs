/*
 * Mechanical translation of the complete pinned source
 * renderer/src/shaders/metal/generate_draw_combinations.py.
 *
 * This Phase-1 owner retains the Python generator's feature-set rules,
 * emission order, output generation, command-line surface, and failure
 * behavior. It is not wired into build.rs in this phase.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::env;
use std::fs::File;
use std::io::{self, Write};

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/metal/generate_draw_combinations.py";
pub const PINNED_SOURCE_SHA256: &str =
    "826df284fa0a03d043f8d54b067ffb806e9204de3cba54faf45b80546eb69fcf";
pub const PINNED_SOURCE_LINE_COUNT: usize = 161;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 7018;
pub const TRANSLATION_UNIT: &str = "metal-shader-generator";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/metal/generate_draw_combinations_py.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-python-generator-behavior";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub source_path: &'static str,
    pub source_sha256: &'static str,
    pub source_line_count: usize,
    pub source_byte_count: usize,
    pub translation_unit: &'static str,
    pub translation_target: &'static str,
    pub translation_disposition: &'static str,
    pub translation_behavior: &'static str,
}

pub const SOURCE_METADATA: SourceMetadata = SourceMetadata {
    upstream_commit: PINNED_UPSTREAM_COMMIT,
    source_path: PINNED_SOURCE_PATH,
    source_sha256: PINNED_SOURCE_SHA256,
    source_line_count: PINNED_SOURCE_LINE_COUNT,
    source_byte_count: PINNED_SOURCE_BYTE_COUNT,
    translation_unit: TRANSLATION_UNIT,
    translation_target: TRANSLATION_TARGET,
    translation_disposition: TRANSLATION_DISPOSITION,
    translation_behavior: TRANSLATION_BEHAVIOR,
};

pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

/// Exact pinned Python source, retained for provenance and line-for-line audit.
pub const PINNED_GENERATE_DRAW_COMBINATIONS_PY_SOURCE: &str = r###"import itertools
import sys
from enum import Enum

# Organizes all combinations of valid features for draw.glsl into their own custom-named namespace.
# Generates MSL code to declare each namespace and #include draw.glsl with corresponding #defines.

class Feature():
    def __init__(self, name, index):
        self.name = name
        self.index = index

# Each feature has a specific index. These must stay in sync with render_context_metal_impl.mm.
ENABLE_CLIPPING = Feature('ENABLE_CLIPPING', 0)
ENABLE_CLIP_RECT =  Feature('ENABLE_CLIP_RECT', 1)
ENABLE_ADVANCED_BLEND = Feature('ENABLE_ADVANCED_BLEND', 2)
ENABLE_FEATHER = Feature('ENABLE_FEATHER', 3)
ENABLE_EVEN_ODD = Feature('ENABLE_EVEN_ODD', 4)
ENABLE_NESTED_CLIPPING = Feature('ENABLE_NESTED_CLIPPING', 5)
ENABLE_HSL_BLEND_MODES = Feature('ENABLE_HSL_BLEND_MODES', 6)
ENABLE_DITHER = Feature('ENABLE_DITHER', 7)
DRAW_INTERIOR_TRIANGLES = Feature('DRAW_INTERIOR_TRIANGLES', 8)
FEATHER_ATLAS_BLIT = Feature('FEATHER_ATLAS_BLIT', 9)

whole_program_features = {ENABLE_CLIPPING,
                          ENABLE_CLIP_RECT,
                          ENABLE_ADVANCED_BLEND,
                          ENABLE_FEATHER}

fragment_only_features = {ENABLE_EVEN_ODD,
                          ENABLE_NESTED_CLIPPING,
                          ENABLE_HSL_BLEND_MODES,
                          ENABLE_DITHER}

all_features = whole_program_features.union(fragment_only_features)

# Returns whether a valid program exists for the given feature set.
def is_valid_feature_set(feature_set):
    if ENABLE_NESTED_CLIPPING in feature_set and ENABLE_CLIPPING not in feature_set:
        return False
    if ENABLE_HSL_BLEND_MODES in feature_set and ENABLE_ADVANCED_BLEND not in feature_set:
        return False
    return True

# Returns whether the given feature set is the *simplest* set that defines a unique vertex shader.
# (Many feature sets produce identical vertex shaders.)
def is_unique_vertex_feature_set(feature_set):
    # Fragment-only features have no effect on the vertex shader.
    if fragment_only_features.intersection(feature_set):
        return False
    return True

non_atlas_coverage_features = {ENABLE_FEATHER,
                               ENABLE_EVEN_ODD,
                               ENABLE_NESTED_CLIPPING}

non_image_mesh_features = {ENABLE_FEATHER,
                           ENABLE_EVEN_ODD,
                           ENABLE_NESTED_CLIPPING,
                           DRAW_INTERIOR_TRIANGLES,
                           FEATHER_ATLAS_BLIT}

# Returns whether the given feature set is compatible with an image mesh shader.
def is_image_mesh_feature_set(feature_set):
    return not non_image_mesh_features.intersection(feature_set)

ShaderType = Enum('ShaderType', ['VERTEX', 'FRAGMENT'])
DrawType = Enum('DrawType', ['PATH', 'IMAGE_MESH'])
FillType = Enum('FillType', ['CLOCKWISE', 'LEGACY'])

def emit_shader(out, shader_type, draw_type, fill_type, feature_set):
    assert(is_valid_feature_set(feature_set))
    if shader_type == ShaderType.VERTEX:
        assert(is_unique_vertex_feature_set(feature_set))
        out.write('#define VERTEX\n')
    else:
        out.write('#define FRAGMENT\n')
    if draw_type == DrawType.IMAGE_MESH:
        assert(is_image_mesh_feature_set(feature_set))
    namespace_id = ['0', '0', '0', '0', '0', '0', '0', '0', '0', '0']
    for feature in feature_set:
        namespace_id[feature.index] = '1'
    for feature in feature_set:
        out.write('#define %s 1\n' % feature.name)
    if fill_type == FillType.CLOCKWISE:
        out.write('#define CLOCKWISE_FILL 1\n')
    if draw_type == DrawType.PATH:
        out.write('#define DRAW_PATH 1\n')
        out.write('namespace %s%s\n' %
                  ('c' if fill_type == FillType.CLOCKWISE else 'p',
                   ''.join(namespace_id)))
        out.write('{\n')
        out.write('#include "draw_path.minified.vert"\n')
        if FEATHER_ATLAS_BLIT in feature_set:
            out.write('#include "draw_mesh.minified.frag"\n')
        else:
            out.write('#include "draw_raster_order_path.minified.frag"\n')
        out.write('}\n')
        out.write('#undef DRAW_PATH\n')
    else:
        out.write('#define DRAW_IMAGE 1\n')
        out.write('#define DRAW_IMAGE_MESH 1\n')
        out.write('namespace m%s\n' % ''.join(namespace_id))
        out.write('{\n')
        out.write('#include "draw_image_mesh.minified.vert"\n')
        out.write('#include "draw_mesh.minified.frag"\n')
        out.write('}\n')
        out.write('#undef DRAW_IMAGE_MESH\n')
        out.write('#undef DRAW_IMAGE\n')
    for feature in feature_set:
        out.write('#undef %s\n' % feature.name)
    if shader_type == ShaderType.VERTEX:
        out.write('#undef VERTEX\n')
    else:
        out.write('#undef FRAGMENT\n')
    if fill_type == FillType.CLOCKWISE:
        out.write('#undef CLOCKWISE_FILL\n')
    out.write('\n')

# Organize all combinations of valid features into their own namespace.
out = open(sys.argv[1], 'w', newline='\n')

# Precompile the bare minimum set of shaders required to draw everything. We can compile more
# specialized shaders in the background at runtime, and use the fully-featured (slower) shaders
# while waiting for the compilations to complete.

# Path tessellation shaders.
emit_shader(out, ShaderType.VERTEX, DrawType.PATH, FillType.LEGACY,
            whole_program_features)
emit_shader(out, ShaderType.FRAGMENT, DrawType.PATH, FillType.LEGACY, all_features)
emit_shader(out, ShaderType.FRAGMENT, DrawType.PATH, FillType.CLOCKWISE, all_features)

# Interior triangulation shaders.
emit_shader(out, ShaderType.VERTEX, DrawType.PATH, FillType.LEGACY,
            whole_program_features.union({DRAW_INTERIOR_TRIANGLES}))
emit_shader(out, ShaderType.FRAGMENT, DrawType.PATH, FillType.LEGACY,
            all_features.union({DRAW_INTERIOR_TRIANGLES}))
emit_shader(out, ShaderType.FRAGMENT, DrawType.PATH, FillType.CLOCKWISE,
            all_features.union({DRAW_INTERIOR_TRIANGLES}))

# Atlas blit shaders.
emit_shader(out, ShaderType.VERTEX, DrawType.PATH, FillType.LEGACY,
            whole_program_features\
                    .union({DRAW_INTERIOR_TRIANGLES, FEATHER_ATLAS_BLIT})\
                    .difference(non_atlas_coverage_features))
emit_shader(out, ShaderType.FRAGMENT, DrawType.PATH, FillType.LEGACY,
            all_features\
                    .union({DRAW_INTERIOR_TRIANGLES, FEATHER_ATLAS_BLIT})\
                    .difference(non_atlas_coverage_features))

# Image mesh shaders.
emit_shader(out, ShaderType.VERTEX, DrawType.IMAGE_MESH, FillType.LEGACY,
            whole_program_features.difference(non_image_mesh_features))
emit_shader(out, ShaderType.FRAGMENT, DrawType.IMAGE_MESH, FillType.LEGACY,
            all_features.difference(non_image_mesh_features))

# If we wanted to emit all combos...
# for n in range(0, len(all_features) + 1):
#     for feature_set in itertools.combinations(all_features, n):
#         if not is_valid_feature_set(feature_set):
#             continue
"###;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Feature {
    pub name: &'static str,
    pub index: usize,
}

// Each feature has a specific index. These must stay in sync with
// render_context_metal_impl.mm, as in the pinned Python source.
pub const ENABLE_CLIPPING: Feature = Feature {
    name: "ENABLE_CLIPPING",
    index: 0,
};
pub const ENABLE_CLIP_RECT: Feature = Feature {
    name: "ENABLE_CLIP_RECT",
    index: 1,
};
pub const ENABLE_ADVANCED_BLEND: Feature = Feature {
    name: "ENABLE_ADVANCED_BLEND",
    index: 2,
};
pub const ENABLE_FEATHER: Feature = Feature {
    name: "ENABLE_FEATHER",
    index: 3,
};
pub const ENABLE_EVEN_ODD: Feature = Feature {
    name: "ENABLE_EVEN_ODD",
    index: 4,
};
pub const ENABLE_NESTED_CLIPPING: Feature = Feature {
    name: "ENABLE_NESTED_CLIPPING",
    index: 5,
};
pub const ENABLE_HSL_BLEND_MODES: Feature = Feature {
    name: "ENABLE_HSL_BLEND_MODES",
    index: 6,
};
pub const ENABLE_DITHER: Feature = Feature {
    name: "ENABLE_DITHER",
    index: 7,
};
pub const DRAW_INTERIOR_TRIANGLES: Feature = Feature {
    name: "DRAW_INTERIOR_TRIANGLES",
    index: 8,
};
pub const FEATHER_ATLAS_BLIT: Feature = Feature {
    name: "FEATHER_ATLAS_BLIT",
    index: 9,
};

/// A source-shaped ordered set. Python's set iteration is process-dependent;
/// retaining literal insertion order makes the mechanical port reproducible
/// while preserving the set's membership and union/difference behavior.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FeatureSet(Vec<Feature>);

impl FeatureSet {
    pub fn from_slice(features: &[Feature]) -> Self {
        let mut result = Self::default();
        for &feature in features {
            result.insert(feature);
        }
        result
    }

    pub fn insert(&mut self, feature: Feature) {
        if !self.contains(feature) {
            self.0.push(feature);
        }
    }

    pub fn contains(&self, feature: Feature) -> bool {
        self.0.contains(&feature)
    }

    pub fn intersection(&self, other: &FeatureSet) -> FeatureSet {
        FeatureSet(
            self.0
                .iter()
                .copied()
                .filter(|feature| other.contains(*feature))
                .collect(),
        )
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn union(&self, other: &FeatureSet) -> FeatureSet {
        let mut result = self.clone();
        for &feature in &other.0 {
            result.insert(feature);
        }
        result
    }

    pub fn difference(&self, other: &FeatureSet) -> FeatureSet {
        FeatureSet(
            self.0
                .iter()
                .copied()
                .filter(|feature| !other.contains(*feature))
                .collect(),
        )
    }

    pub fn iter(&self) -> impl Iterator<Item = &Feature> {
        self.0.iter()
    }
}

pub fn whole_program_features() -> FeatureSet {
    FeatureSet::from_slice(&[
        ENABLE_CLIPPING,
        ENABLE_CLIP_RECT,
        ENABLE_ADVANCED_BLEND,
        ENABLE_FEATHER,
    ])
}

pub fn fragment_only_features() -> FeatureSet {
    FeatureSet::from_slice(&[
        ENABLE_EVEN_ODD,
        ENABLE_NESTED_CLIPPING,
        ENABLE_HSL_BLEND_MODES,
        ENABLE_DITHER,
    ])
}

pub fn all_features() -> FeatureSet {
    whole_program_features().union(&fragment_only_features())
}

pub fn non_atlas_coverage_features() -> FeatureSet {
    FeatureSet::from_slice(&[ENABLE_FEATHER, ENABLE_EVEN_ODD, ENABLE_NESTED_CLIPPING])
}

pub fn non_image_mesh_features() -> FeatureSet {
    FeatureSet::from_slice(&[
        ENABLE_FEATHER,
        ENABLE_EVEN_ODD,
        ENABLE_NESTED_CLIPPING,
        DRAW_INTERIOR_TRIANGLES,
        FEATHER_ATLAS_BLIT,
    ])
}

// Returns whether a valid program exists for the given feature set.
pub fn is_valid_feature_set(feature_set: &FeatureSet) -> bool {
    if feature_set.contains(ENABLE_NESTED_CLIPPING) && !feature_set.contains(ENABLE_CLIPPING) {
        return false;
    }
    if feature_set.contains(ENABLE_HSL_BLEND_MODES) && !feature_set.contains(ENABLE_ADVANCED_BLEND)
    {
        return false;
    }
    true
}

// Returns whether the given feature set is the *simplest* set that defines a
// unique vertex shader. (Many feature sets produce identical vertex shaders.)
pub fn is_unique_vertex_feature_set(feature_set: &FeatureSet) -> bool {
    // Fragment-only features have no effect on the vertex shader.
    fragment_only_features()
        .intersection(feature_set)
        .is_empty()
}

// Returns whether the given feature set is compatible with an image mesh shader.
pub fn is_image_mesh_feature_set(feature_set: &FeatureSet) -> bool {
    non_image_mesh_features()
        .intersection(feature_set)
        .is_empty()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShaderType {
    VERTEX,
    FRAGMENT,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawType {
    PATH,
    IMAGE_MESH,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FillType {
    CLOCKWISE,
    LEGACY,
}

pub fn emit_shader<W: Write>(
    out: &mut W,
    shader_type: ShaderType,
    draw_type: DrawType,
    fill_type: FillType,
    feature_set: &FeatureSet,
) -> io::Result<()> {
    assert!(is_valid_feature_set(feature_set));
    if shader_type == ShaderType::VERTEX {
        assert!(is_unique_vertex_feature_set(feature_set));
        out.write_all(b"#define VERTEX\n")?;
    } else {
        out.write_all(b"#define FRAGMENT\n")?;
    }
    if draw_type == DrawType::IMAGE_MESH {
        assert!(is_image_mesh_feature_set(feature_set));
    }

    let mut namespace_id = ['0'; 10];
    for feature in feature_set.iter() {
        // Indexing deliberately retains Python's IndexError-like panic for an
        // invalid Feature index rather than silently accepting malformed data.
        namespace_id[feature.index] = '1';
    }
    for feature in feature_set.iter() {
        writeln!(out, "#define {} 1", feature.name)?;
    }
    if fill_type == FillType::CLOCKWISE {
        out.write_all(b"#define CLOCKWISE_FILL 1\n")?;
    }
    if draw_type == DrawType::PATH {
        out.write_all(b"#define DRAW_PATH 1\n")?;
        let namespace_prefix = if fill_type == FillType::CLOCKWISE {
            'c'
        } else {
            'p'
        };
        let namespace_id: String = namespace_id.iter().collect();
        writeln!(out, "namespace {namespace_prefix}{namespace_id}")?;
        out.write_all(b"{\n")?;
        out.write_all(b"#include \"draw_path.minified.vert\"\n")?;
        if feature_set.contains(FEATHER_ATLAS_BLIT) {
            out.write_all(b"#include \"draw_mesh.minified.frag\"\n")?;
        } else {
            out.write_all(b"#include \"draw_raster_order_path.minified.frag\"\n")?;
        }
        out.write_all(b"}\n")?;
        out.write_all(b"#undef DRAW_PATH\n")?;
    } else {
        out.write_all(b"#define DRAW_IMAGE 1\n")?;
        out.write_all(b"#define DRAW_IMAGE_MESH 1\n")?;
        let namespace_id: String = namespace_id.iter().collect();
        writeln!(out, "namespace m{namespace_id}")?;
        out.write_all(b"{\n")?;
        out.write_all(b"#include \"draw_image_mesh.minified.vert\"\n")?;
        out.write_all(b"#include \"draw_mesh.minified.frag\"\n")?;
        out.write_all(b"}\n")?;
        out.write_all(b"#undef DRAW_IMAGE_MESH\n")?;
        out.write_all(b"#undef DRAW_IMAGE\n")?;
    }
    for feature in feature_set.iter() {
        writeln!(out, "#undef {}", feature.name)?;
    }
    if shader_type == ShaderType::VERTEX {
        out.write_all(b"#undef VERTEX\n")?;
    } else {
        out.write_all(b"#undef FRAGMENT\n")?;
    }
    if fill_type == FillType::CLOCKWISE {
        out.write_all(b"#undef CLOCKWISE_FILL\n")?;
    }
    out.write_all(b"\n")?;
    Ok(())
}

/// Emit the ten precompiled shader combinations in the exact source order.
pub fn generate<W: Write>(out: &mut W) -> io::Result<()> {
    let whole_program_features = whole_program_features();
    let all_features = all_features();
    let non_atlas_coverage_features = non_atlas_coverage_features();
    let non_image_mesh_features = non_image_mesh_features();

    // Path tessellation shaders.
    emit_shader(
        out,
        ShaderType::VERTEX,
        DrawType::PATH,
        FillType::LEGACY,
        &whole_program_features,
    )?;
    emit_shader(
        out,
        ShaderType::FRAGMENT,
        DrawType::PATH,
        FillType::LEGACY,
        &all_features,
    )?;
    emit_shader(
        out,
        ShaderType::FRAGMENT,
        DrawType::PATH,
        FillType::CLOCKWISE,
        &all_features,
    )?;

    // Interior triangulation shaders.
    let whole_program_features_with_interior =
        whole_program_features.union(&FeatureSet::from_slice(&[DRAW_INTERIOR_TRIANGLES]));
    emit_shader(
        out,
        ShaderType::VERTEX,
        DrawType::PATH,
        FillType::LEGACY,
        &whole_program_features_with_interior,
    )?;
    let all_features_with_interior =
        all_features.union(&FeatureSet::from_slice(&[DRAW_INTERIOR_TRIANGLES]));
    emit_shader(
        out,
        ShaderType::FRAGMENT,
        DrawType::PATH,
        FillType::LEGACY,
        &all_features_with_interior,
    )?;
    emit_shader(
        out,
        ShaderType::FRAGMENT,
        DrawType::PATH,
        FillType::CLOCKWISE,
        &all_features_with_interior,
    )?;

    // Atlas blit shaders.
    let atlas_features = whole_program_features
        .union(&FeatureSet::from_slice(&[
            DRAW_INTERIOR_TRIANGLES,
            FEATHER_ATLAS_BLIT,
        ]))
        .difference(&non_atlas_coverage_features);
    emit_shader(
        out,
        ShaderType::VERTEX,
        DrawType::PATH,
        FillType::LEGACY,
        &atlas_features,
    )?;
    let atlas_fragment_features = all_features
        .union(&FeatureSet::from_slice(&[
            DRAW_INTERIOR_TRIANGLES,
            FEATHER_ATLAS_BLIT,
        ]))
        .difference(&non_atlas_coverage_features);
    emit_shader(
        out,
        ShaderType::FRAGMENT,
        DrawType::PATH,
        FillType::LEGACY,
        &atlas_fragment_features,
    )?;

    // Image mesh shaders.
    let image_mesh_vertex_features = whole_program_features.difference(&non_image_mesh_features);
    emit_shader(
        out,
        ShaderType::VERTEX,
        DrawType::IMAGE_MESH,
        FillType::LEGACY,
        &image_mesh_vertex_features,
    )?;
    let image_mesh_fragment_features = all_features.difference(&non_image_mesh_features);
    emit_shader(
        out,
        ShaderType::FRAGMENT,
        DrawType::IMAGE_MESH,
        FillType::LEGACY,
        &image_mesh_fragment_features,
    )?;
    Ok(())
}

/// Command-line equivalent of `open(sys.argv[1], 'w', newline='\\n')`.
///
/// The first argument is the program name, the second is the output path, and
/// additional arguments are intentionally ignored, matching the Python tool.
pub fn run<I>(arguments: I) -> io::Result<()>
where
    I: IntoIterator<Item = String>,
{
    let mut arguments = arguments.into_iter();
    let _program = arguments.next();
    let output_path = arguments
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "list index out of range"))?;
    let mut out = File::create(output_path)?;
    generate(&mut out)
}

pub fn main() -> io::Result<()> {
    run(env::args())
}
