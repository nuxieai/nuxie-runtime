/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/pls_load_store_ext.glsl.
 *
 * This Phase-1 owner retains the shader bytes exactly and exposes the
 * authority-ledger branches, exports, functions, and dependencies as literal
 * source-shaped data. It does not compile, evaluate, simplify, or generate
 * shader artifacts.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/pls_load_store_ext.glsl";
pub const PINNED_SOURCE_SHA256: &str =
    "39d167247268280cac6bbf5d9febdd30fea9fcf1bce5016eca1170e4544feb82";
pub const PINNED_SOURCE_LINE_COUNT: usize = 105;
pub const PINNED_SOURCE_BYTE_COUNT: usize = 2218;
pub const TRANSLATION_UNIT: &str = "metal-shader-source-batch";
pub const TRANSLATION_TARGET: &str = "crates/nuxie-renderer/src/mechanical_port/source/renderer/src/shaders/pls_load_store_ext_glsl.rs";
pub const TRANSLATION_DISPOSITION: &str = "required";
pub const TRANSLATION_BEHAVIOR: &str = "preserve-full-source-non-metal-rule";

/// Exact pinned GLSL source, retained for provenance and line-for-line audit.
pub const PINNED_PLS_LOAD_STORE_EXT_GLSL_SOURCE: &str = r###"/*
 * Copyright 2022 Rive
 */

// The EXT_shader_pixel_local_storage extension does not provide a mechanism to
// load, store, or clear pixel local storage contents. This shader performs
// custom load, store, and clear operations via fullscreen draws.

#ifdef @VERTEX
void main()
{
    // [-1, -1] .. [+1, +1]
    gl_Position = vec4(mix(vec2(-1, 1),
                           vec2(1, -1),
                           equal(gl_VertexID & ivec2(1, 2), ivec2(0))),
                       0,
                       1);
#ifdef @POST_INVERT_Y
    gl_Position.y = -gl_Position.y;
#endif
}
#endif

#ifdef @FRAGMENT

#extension GL_EXT_shader_pixel_local_storage : require
#ifdef GL_ARM_shader_framebuffer_fetch
#extension GL_ARM_shader_framebuffer_fetch : require
#else
#extension GL_EXT_shader_framebuffer_fetch : require
#endif

#ifdef @CLEAR_COLOR
#if __VERSION__ >= 310
layout(binding = 0, std140) uniform ClearColor { uniform highp vec4 value; }
clearColor;
#else
uniform mediump vec4 @clearColor;
#endif
#endif

#ifdef GL_EXT_shader_pixel_local_storage

#ifdef @STORE_COLOR
__pixel_local_inEXT PLS
#else
__pixel_local_outEXT PLS
#endif
{
    layout(rgba8) mediump vec4 colorBuffer;
    layout(r32ui) highp uint clipBuffer;
    layout(rgba8) mediump vec4 scratchColorBuffer;
    layout(r32ui) highp uint coverageCountBuffer;
};

#ifndef GL_ARM_shader_framebuffer_fetch
#ifdef @LOAD_COLOR
layout(location = 0) inout mediump vec4 fragColor;
#endif
#endif

#ifdef @STORE_COLOR
layout(location = 0) out mediump vec4 fragColor;
#endif

void main()
{
#ifdef @CLEAR_COLOR
#if __VERSION__ >= 310
    colorBuffer = clearColor.value;
#else
    colorBuffer = @clearColor;
#endif
#endif

#ifdef @LOAD_COLOR
#ifdef GL_ARM_shader_framebuffer_fetch
    colorBuffer = gl_LastFragColorARM;
#else
    colorBuffer = fragColor;
#endif
#endif

#ifdef @CLEAR_COVERAGE
    coverageCountBuffer = 0u;
#endif

#ifdef @CLEAR_CLIP
    clipBuffer = 0u;
#endif

#ifdef @STORE_COLOR
    fragColor = colorBuffer;
#endif
}

#else

// This shader is being parsed by WebGPU for introspection purposes.
layout(location = 0) out mediump vec4 unused;
void main() { unused = vec4(0, 1, 0, 1); }

#endif // GL_EXT_shader_pixel_local_storage

#endif // FRAGMENT
"###;

/// Stable aliases used by later source-audit queues.
pub const PINNED_PLS_LOAD_STORE_EXT_SOURCE: &str = PINNED_PLS_LOAD_STORE_EXT_GLSL_SOURCE;
pub const PLS_LOAD_STORE_EXT_GLSL_SOURCE: &str = PINNED_PLS_LOAD_STORE_EXT_GLSL_SOURCE;

pub const SOURCE_SHA256: &str = PINNED_SOURCE_SHA256;
pub const SOURCE_LINE_COUNT: usize = PINNED_SOURCE_LINE_COUNT;
pub const SOURCE_BYTE_COUNT: usize = PINNED_SOURCE_BYTE_COUNT;

pub const fn pinned_source() -> &'static str {
    PINNED_PLS_LOAD_STORE_EXT_GLSL_SOURCE
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMetadata {
    pub upstream_commit: &'static str,
    pub upstream_path: &'static str,
    pub source_sha256: &'static str,
    pub source_line_count: usize,
    pub source_byte_count: usize,
    pub target_path: &'static str,
    pub translation_unit: &'static str,
    pub translation_disposition: &'static str,
    pub translation_behavior: &'static str,
}

pub const SOURCE_METADATA: SourceMetadata = SourceMetadata {
    upstream_commit: PINNED_UPSTREAM_COMMIT,
    upstream_path: PINNED_SOURCE_PATH,
    source_sha256: PINNED_SOURCE_SHA256,
    source_line_count: PINNED_SOURCE_LINE_COUNT,
    source_byte_count: PINNED_SOURCE_BYTE_COUNT,
    target_path: TRANSLATION_TARGET,
    translation_unit: TRANSLATION_UNIT,
    translation_disposition: TRANSLATION_DISPOSITION,
    translation_behavior: TRANSLATION_BEHAVIOR,
};

/// Every semantic preprocessor block in the pinned source, in source order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBlock {
    pub block_id: &'static str,
    pub block_start: u16,
    pub block_end: u16,
    pub block_depth: u8,
    pub branch_count: u8,
}

pub const CONDITIONAL_BLOCKS: &[ConditionalBlock] = &[
    ConditionalBlock {
        block_id: "pp-0558",
        block_start: 9,
        block_end: 22,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0559",
        block_start: 18,
        block_end: 20,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0560",
        block_start: 24,
        block_end: 105,
        block_depth: 0,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0561",
        block_start: 27,
        block_end: 31,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0562",
        block_start: 33,
        block_end: 40,
        block_depth: 1,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0563",
        block_start: 34,
        block_end: 39,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0564",
        block_start: 42,
        block_end: 103,
        block_depth: 1,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0565",
        block_start: 44,
        block_end: 48,
        block_depth: 2,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0566",
        block_start: 56,
        block_end: 60,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0567",
        block_start: 57,
        block_end: 59,
        block_depth: 3,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0568",
        block_start: 62,
        block_end: 64,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0569",
        block_start: 68,
        block_end: 74,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0570",
        block_start: 69,
        block_end: 73,
        block_depth: 3,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0571",
        block_start: 76,
        block_end: 82,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0572",
        block_start: 77,
        block_end: 81,
        block_depth: 3,
        branch_count: 2,
    },
    ConditionalBlock {
        block_id: "pp-0573",
        block_start: 84,
        block_end: 86,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0574",
        block_start: 88,
        block_end: 90,
        block_depth: 2,
        branch_count: 1,
    },
    ConditionalBlock {
        block_id: "pp-0575",
        block_start: 92,
        block_end: 94,
        block_depth: 2,
        branch_count: 1,
    },
];

/// Every branch entry remains literal, in authority/source order. The active
/// paths are ledger spellings; they are not evaluated as Rust cfg expressions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionalBranch {
    pub block_id: &'static str,
    pub branch_ordinal: u8,
    pub branch_line: u16,
    pub directive: &'static str,
    pub active_branch_path: &'static str,
}

pub const CONDITIONAL_BRANCHES: &[ConditionalBranch] = &[
    ConditionalBranch {
        block_id: "pp-0558",
        branch_ordinal: 1,
        branch_line: 9,
        directive: "#ifdef @VERTEX",
        active_branch_path: "(defined(@VERTEX))",
    },
    ConditionalBranch {
        block_id: "pp-0559",
        branch_ordinal: 1,
        branch_line: 18,
        directive: "#ifdef @POST_INVERT_Y",
        active_branch_path: "(defined(@VERTEX)) && (defined(@POST_INVERT_Y))",
    },
    ConditionalBranch {
        block_id: "pp-0560",
        branch_ordinal: 1,
        branch_line: 24,
        directive: "#ifdef @FRAGMENT",
        active_branch_path: "(defined(@FRAGMENT))",
    },
    ConditionalBranch {
        block_id: "pp-0561",
        branch_ordinal: 1,
        branch_line: 27,
        directive: "#ifdef GL_ARM_shader_framebuffer_fetch",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_ARM_shader_framebuffer_fetch))",
    },
    ConditionalBranch {
        block_id: "pp-0561",
        branch_ordinal: 2,
        branch_line: 29,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(GL_ARM_shader_framebuffer_fetch))))",
    },
    ConditionalBranch {
        block_id: "pp-0562",
        branch_ordinal: 1,
        branch_line: 33,
        directive: "#ifdef @CLEAR_COLOR",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@CLEAR_COLOR))",
    },
    ConditionalBranch {
        block_id: "pp-0563",
        branch_ordinal: 1,
        branch_line: 34,
        directive: "#if __VERSION__ >= 310",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@CLEAR_COLOR)) && (__VERSION__ >= 310)",
    },
    ConditionalBranch {
        block_id: "pp-0563",
        branch_ordinal: 2,
        branch_line: 37,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(@CLEAR_COLOR)) && (!((__VERSION__ >= 310)))",
    },
    ConditionalBranch {
        block_id: "pp-0564",
        branch_ordinal: 1,
        branch_line: 42,
        directive: "#ifdef GL_EXT_shader_pixel_local_storage",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage))",
    },
    ConditionalBranch {
        block_id: "pp-0564",
        branch_ordinal: 2,
        branch_line: 97,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (!((defined(GL_EXT_shader_pixel_local_storage))))",
    },
    ConditionalBranch {
        block_id: "pp-0565",
        branch_ordinal: 1,
        branch_line: 44,
        directive: "#ifdef @STORE_COLOR",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (defined(@STORE_COLOR))",
    },
    ConditionalBranch {
        block_id: "pp-0565",
        branch_ordinal: 2,
        branch_line: 46,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (!((defined(@STORE_COLOR))))",
    },
    ConditionalBranch {
        block_id: "pp-0566",
        branch_ordinal: 1,
        branch_line: 56,
        directive: "#ifndef GL_ARM_shader_framebuffer_fetch",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (!defined(GL_ARM_shader_framebuffer_fetch))",
    },
    ConditionalBranch {
        block_id: "pp-0567",
        branch_ordinal: 1,
        branch_line: 57,
        directive: "#ifdef @LOAD_COLOR",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (!defined(GL_ARM_shader_framebuffer_fetch)) && (defined(@LOAD_COLOR))",
    },
    ConditionalBranch {
        block_id: "pp-0568",
        branch_ordinal: 1,
        branch_line: 62,
        directive: "#ifdef @STORE_COLOR",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (defined(@STORE_COLOR))",
    },
    ConditionalBranch {
        block_id: "pp-0569",
        branch_ordinal: 1,
        branch_line: 68,
        directive: "#ifdef @CLEAR_COLOR",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (defined(@CLEAR_COLOR))",
    },
    ConditionalBranch {
        block_id: "pp-0570",
        branch_ordinal: 1,
        branch_line: 69,
        directive: "#if __VERSION__ >= 310",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (defined(@CLEAR_COLOR)) && (__VERSION__ >= 310)",
    },
    ConditionalBranch {
        block_id: "pp-0570",
        branch_ordinal: 2,
        branch_line: 71,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (defined(@CLEAR_COLOR)) && (!((__VERSION__ >= 310)))",
    },
    ConditionalBranch {
        block_id: "pp-0571",
        branch_ordinal: 1,
        branch_line: 76,
        directive: "#ifdef @LOAD_COLOR",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (defined(@LOAD_COLOR))",
    },
    ConditionalBranch {
        block_id: "pp-0572",
        branch_ordinal: 1,
        branch_line: 77,
        directive: "#ifdef GL_ARM_shader_framebuffer_fetch",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (defined(@LOAD_COLOR)) && (defined(GL_ARM_shader_framebuffer_fetch))",
    },
    ConditionalBranch {
        block_id: "pp-0572",
        branch_ordinal: 2,
        branch_line: 79,
        directive: "#else",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (defined(@LOAD_COLOR)) && (!((defined(GL_ARM_shader_framebuffer_fetch))))",
    },
    ConditionalBranch {
        block_id: "pp-0573",
        branch_ordinal: 1,
        branch_line: 84,
        directive: "#ifdef @CLEAR_COVERAGE",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (defined(@CLEAR_COVERAGE))",
    },
    ConditionalBranch {
        block_id: "pp-0574",
        branch_ordinal: 1,
        branch_line: 88,
        directive: "#ifdef @CLEAR_CLIP",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (defined(@CLEAR_CLIP))",
    },
    ConditionalBranch {
        block_id: "pp-0575",
        branch_ordinal: 1,
        branch_line: 92,
        directive: "#ifdef @STORE_COLOR",
        active_branch_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage)) && (defined(@STORE_COLOR))",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExportedSymbol {
    pub source_line: u16,
    pub source_name: &'static str,
    pub generated_name: &'static str,
    pub generated_header_name: &'static str,
}

/// The nine @-prefixed identifiers occurring directly in this shader source.
pub const EXPORTED_SYMBOLS: &[ExportedSymbol] = &[
    ExportedSymbol {
        source_line: 9,
        source_name: "@VERTEX",
        generated_name: "CB",
        generated_header_name: "GLSL_VERTEX",
    },
    ExportedSymbol {
        source_line: 18,
        source_name: "@POST_INVERT_Y",
        generated_name: "JC",
        generated_header_name: "GLSL_POST_INVERT_Y",
    },
    ExportedSymbol {
        source_line: 24,
        source_name: "@FRAGMENT",
        generated_name: "FB",
        generated_header_name: "GLSL_FRAGMENT",
    },
    ExportedSymbol {
        source_line: 33,
        source_name: "@CLEAR_COLOR",
        generated_name: "JE",
        generated_header_name: "GLSL_CLEAR_COLOR",
    },
    ExportedSymbol {
        source_line: 38,
        source_name: "@clearColor",
        generated_name: "KE",
        generated_header_name: "GLSL_clearColor",
    },
    ExportedSymbol {
        source_line: 44,
        source_name: "@STORE_COLOR",
        generated_name: "SD",
        generated_header_name: "GLSL_STORE_COLOR",
    },
    ExportedSymbol {
        source_line: 57,
        source_name: "@LOAD_COLOR",
        generated_name: "LE",
        generated_header_name: "GLSL_LOAD_COLOR",
    },
    ExportedSymbol {
        source_line: 84,
        source_name: "@CLEAR_COVERAGE",
        generated_name: "TD",
        generated_header_name: "GLSL_CLEAR_COVERAGE",
    },
    ExportedSymbol {
        source_line: 88,
        source_name: "@CLEAR_CLIP",
        generated_name: "JF",
        generated_header_name: "GLSL_CLEAR_CLIP",
    },
];

pub const EXPORTED_SWITCHES: &[ExportedSymbol] = EXPORTED_SYMBOLS;

pub const DIRECT_SOURCE_EXPORT_IDENTIFIERS: &[&str] = &[
    "VERTEX",
    "POST_INVERT_Y",
    "FRAGMENT",
    "CLEAR_COLOR",
    "clearColor",
    "STORE_COLOR",
    "LOAD_COLOR",
    "CLEAR_COVERAGE",
    "CLEAR_CLIP",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderFunction {
    pub source_line: u16,
    pub end_line: u16,
    pub name: &'static str,
    pub signature: &'static str,
    pub guard_path: &'static str,
    pub inline_qualifier: &'static str,
}

/// Function declarations are retained as source spellings and ranges. Their
/// bodies remain in the pinned GLSL source rather than becoming executable
/// Rust functions.
pub const EXPORTED_FUNCTIONS: &[ShaderFunction] = &[
    ShaderFunction {
        source_line: 10,
        end_line: 21,
        name: "main",
        signature: "void main()",
        guard_path: "(defined(@VERTEX))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 66,
        end_line: 95,
        name: "main",
        signature: "void main()",
        guard_path: "(defined(@FRAGMENT)) && (defined(GL_EXT_shader_pixel_local_storage))",
        inline_qualifier: "",
    },
    ShaderFunction {
        source_line: 101,
        end_line: 101,
        name: "main",
        signature: "void main() { unused = vec4(0, 1, 0, 1); }",
        guard_path: "(defined(@FRAGMENT)) && (!((defined(GL_EXT_shader_pixel_local_storage))))",
        inline_qualifier: "",
    },
];

pub const FUNCTION_DECLARATIONS: &[ShaderFunction] = EXPORTED_FUNCTIONS;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShaderInclude {
    pub upstream_file: &'static str,
    pub include_line: u16,
    pub directive: &'static str,
    pub include_token: &'static str,
    pub include_syntax: &'static str,
    pub active_branch_path: &'static str,
    pub resolution_kind: &'static str,
    pub resolved_source: &'static str,
    pub source_unit: &'static str,
    pub dependency_unit: &'static str,
    pub correspondence_owner: &'static str,
    pub mapping_status: &'static str,
    pub translation_status: &'static str,
    pub translation_disposition: &'static str,
}

/// The pinned GLSL source has no direct #include/#import directive.
pub const INCLUDE_INVENTORY: &[ShaderInclude] = &[];

/// No incoming include/source edge is present in the pinned Metal-port
/// include authority for this GL-only shader source.
pub const CONSUMER_INCLUDE_AUTHORITY: &[ShaderInclude] = &[];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IncludeDependency {
    pub including_source: &'static str,
    pub include_line: u16,
    pub include_token: &'static str,
    pub include_syntax: &'static str,
    pub active_branch_path: &'static str,
    pub resolution_kind: &'static str,
    pub resolved_source: &'static str,
    pub source_unit: &'static str,
    pub dependency_unit: &'static str,
    pub translation_disposition: &'static str,
}

/// This shader source has no direct #include/#import directive or incoming
/// include/source dependency authority entry.
pub const INCLUDE_DEPENDENCIES: &[IncludeDependency] = &[];
pub const DIRECT_SOURCE_INCLUDES: &[&str] = &[];
pub const SOURCE_DEPENDENCY_EDGES: &[IncludeDependency] = INCLUDE_DEPENDENCIES;
