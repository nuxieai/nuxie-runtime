//! Complete WebGL2-configuration declaration translation of
//! `renderer/include/rive/renderer/gl/gles3.hpp`.

#![allow(non_snake_case, non_upper_case_globals)]

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_gl_gles3.hpp");

pub(crate) type GLenum = u32;
pub(crate) type GLuint = u32;
pub(crate) type GLint = i32;
pub(crate) type GLsizei = i32;
pub(crate) type GLfloat = f32;
pub(crate) type GLbitfield = u32;
pub(crate) type GLboolean = u8;

/// Exact GL object namespaces whose source APIs synchronously return a real
/// name. Names from different kinds may legally have the same numeric value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GLObjectKind {
    Buffer,
    Framebuffer,
    Program,
    Renderbuffer,
    Sampler,
    Texture,
    VertexArray,
}

/// Exact observable outcomes of the source's draft WebGL PLS admission
/// helper. The browser provider performs the JavaScript object inspection and
/// retains the extension only for `Enabled`; Rust preserves the distinctions
/// so browser conformance cannot collapse the acceptance matrix to one bool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WebGLShaderPixelLocalStorageEnableResult {
    ExtensionUnavailable,
    NonCoherent,
    DeprecatedVersion,
    Enabled,
}

pub(crate) const GL_NONE: GLenum = 0;
pub(crate) const GL_FALSE: GLboolean = 0;
pub(crate) const GL_TRUE: GLboolean = 1;
pub(crate) const GL_ZERO: GLenum = 0;
pub(crate) const GL_ONE: GLenum = 1;
pub(crate) const GL_FRONT: GLenum = 0x0404;
pub(crate) const GL_BACK: GLenum = 0x0405;
pub(crate) const GL_FRONT_AND_BACK: GLenum = 0x0408;
pub(crate) const GL_CW: GLenum = 0x0900;
pub(crate) const GL_CCW: GLenum = 0x0901;
pub(crate) const GL_NEVER: GLenum = 0x0200;
pub(crate) const GL_LESS: GLenum = 0x0201;
pub(crate) const GL_EQUAL: GLenum = 0x0202;
pub(crate) const GL_LEQUAL: GLenum = 0x0203;
pub(crate) const GL_GREATER: GLenum = 0x0204;
pub(crate) const GL_NOTEQUAL: GLenum = 0x0205;
pub(crate) const GL_GEQUAL: GLenum = 0x0206;
pub(crate) const GL_ALWAYS: GLenum = 0x0207;
pub(crate) const GL_KEEP: GLenum = 0x1E00;
pub(crate) const GL_REPLACE: GLenum = 0x1E01;
pub(crate) const GL_INCR: GLenum = 0x1E02;
pub(crate) const GL_INCR_WRAP: GLenum = 0x8507;
pub(crate) const GL_DECR: GLenum = 0x1E03;
pub(crate) const GL_DECR_WRAP: GLenum = 0x8508;
pub(crate) const GL_INVERT: GLenum = 0x150A;
pub(crate) const GL_POINTS: GLenum = 0x0000;
pub(crate) const GL_LINES: GLenum = 0x0001;
pub(crate) const GL_LINE_STRIP: GLenum = 0x0003;
pub(crate) const GL_TRIANGLES: GLenum = 0x0004;
pub(crate) const GL_TRIANGLE_STRIP: GLenum = 0x0005;
pub(crate) const GL_ARRAY_BUFFER: GLenum = 0x8892;
pub(crate) const GL_ARRAY_BUFFER_BINDING: GLenum = 0x8894;
pub(crate) const GL_ELEMENT_ARRAY_BUFFER: GLenum = 0x8893;
pub(crate) const GL_ELEMENT_ARRAY_BUFFER_BINDING: GLenum = 0x8895;
pub(crate) const GL_COPY_WRITE_BUFFER: GLenum = 0x8F37;
pub(crate) const GL_STATIC_DRAW: GLenum = 0x88E4;
pub(crate) const GL_DYNAMIC_DRAW: GLenum = 0x88E8;
pub(crate) const GL_UNIFORM_BUFFER: GLenum = 0x8A11;
pub(crate) const GL_PIXEL_UNPACK_BUFFER: GLenum = 0x88EC;
pub(crate) const GL_SCISSOR_TEST: GLenum = 0x0C11;
pub(crate) const GL_DEPTH_TEST: GLenum = 0x0B71;
pub(crate) const GL_STENCIL_TEST: GLenum = 0x0B90;
pub(crate) const GL_CULL_FACE: GLenum = 0x0B44;
pub(crate) const GL_BLEND: GLenum = 0x0BE2;
pub(crate) const GL_DITHER: GLenum = 0x0BD0;
pub(crate) const GL_POLYGON_OFFSET_FILL: GLenum = 0x8037;
pub(crate) const GL_RASTERIZER_DISCARD: GLenum = 0x8C89;
pub(crate) const GL_SAMPLE_ALPHA_TO_COVERAGE: GLenum = 0x809E;
pub(crate) const GL_SAMPLE_COVERAGE: GLenum = 0x80A0;
pub(crate) const GL_UNPACK_ROW_LENGTH: GLenum = 0x0CF2;
pub(crate) const GL_UNPACK_SKIP_ROWS: GLenum = 0x0CF3;
pub(crate) const GL_UNPACK_SKIP_PIXELS: GLenum = 0x0CF4;
pub(crate) const GL_UNPACK_ALIGNMENT: GLenum = 0x0CF5;
pub(crate) const GL_PACK_ROW_LENGTH: GLenum = 0x0D02;
pub(crate) const GL_PACK_SKIP_ROWS: GLenum = 0x0D03;
pub(crate) const GL_PACK_SKIP_PIXELS: GLenum = 0x0D04;
pub(crate) const GL_PACK_ALIGNMENT: GLenum = 0x0D05;
pub(crate) const GL_FUNC_ADD: GLenum = 0x8006;
pub(crate) const GL_MIN: GLenum = 0x8007;
pub(crate) const GL_MAX: GLenum = 0x8008;
pub(crate) const GL_FUNC_SUBTRACT: GLenum = 0x800A;
pub(crate) const GL_FUNC_REVERSE_SUBTRACT: GLenum = 0x800B;
pub(crate) const GL_SRC_COLOR: GLenum = 0x0300;
pub(crate) const GL_ONE_MINUS_SRC_COLOR: GLenum = 0x0301;
pub(crate) const GL_SRC_ALPHA: GLenum = 0x0302;
pub(crate) const GL_ONE_MINUS_SRC_ALPHA: GLenum = 0x0303;
pub(crate) const GL_DST_ALPHA: GLenum = 0x0304;
pub(crate) const GL_ONE_MINUS_DST_ALPHA: GLenum = 0x0305;
pub(crate) const GL_DST_COLOR: GLenum = 0x0306;
pub(crate) const GL_ONE_MINUS_DST_COLOR: GLenum = 0x0307;
pub(crate) const GL_SRC_ALPHA_SATURATE: GLenum = 0x0308;
pub(crate) const GL_CONSTANT_COLOR: GLenum = 0x8001;
pub(crate) const GL_ONE_MINUS_CONSTANT_COLOR: GLenum = 0x8002;
pub(crate) const GL_VERTEX_SHADER: GLenum = 0x8B31;
pub(crate) const GL_FRAGMENT_SHADER: GLenum = 0x8B30;
pub(crate) const GL_TEXTURE_2D: GLenum = 0x0DE1;
pub(crate) const GL_TEXTURE_3D: GLenum = 0x806F;
pub(crate) const GL_TEXTURE_2D_ARRAY: GLenum = 0x8C1A;
pub(crate) const GL_TEXTURE_CUBE_MAP: GLenum = 0x8513;
pub(crate) const GL_TEXTURE_CUBE_MAP_POSITIVE_X: GLenum = 0x8515;
pub(crate) const GL_TEXTURE0: GLenum = 0x84C0;
pub(crate) const GL_TEXTURE_MIN_FILTER: GLenum = 0x2801;
pub(crate) const GL_TEXTURE_MAG_FILTER: GLenum = 0x2800;
pub(crate) const GL_TEXTURE_WRAP_S: GLenum = 0x2802;
pub(crate) const GL_TEXTURE_WRAP_T: GLenum = 0x2803;
pub(crate) const GL_CLAMP_TO_EDGE: GLenum = 0x812F;
pub(crate) const GL_REPEAT: GLenum = 0x2901;
pub(crate) const GL_MIRRORED_REPEAT: GLenum = 0x8370;
pub(crate) const GL_LINEAR: GLenum = 0x2601;
pub(crate) const GL_NEAREST: GLenum = 0x2600;
pub(crate) const GL_NEAREST_MIPMAP_NEAREST: GLenum = 0x2700;
pub(crate) const GL_LINEAR_MIPMAP_NEAREST: GLenum = 0x2701;
pub(crate) const GL_NEAREST_MIPMAP_LINEAR: GLenum = 0x2702;
pub(crate) const GL_LINEAR_MIPMAP_LINEAR: GLenum = 0x2703;
pub(crate) const GL_COLOR_BUFFER_BIT: GLbitfield = 0x00004000;
pub(crate) const GL_DEPTH_BUFFER_BIT: GLbitfield = 0x00000100;
pub(crate) const GL_STENCIL_BUFFER_BIT: GLbitfield = 0x00000400;
pub(crate) const GL_DONT_CARE: GLenum = 0x1100;
pub(crate) const GL_UNPACK_IMAGE_HEIGHT: GLenum = 0x806E;
pub(crate) const GL_R8: GLenum = 0x8229;
pub(crate) const GL_R32I: GLenum = 0x8235;
pub(crate) const GL_R32UI: GLenum = 0x8236;
pub(crate) const GL_RG8: GLenum = 0x822B;
pub(crate) const GL_RG32UI: GLenum = 0x823C;
pub(crate) const GL_RGBA8: GLenum = 0x8058;
pub(crate) const GL_RGBA8_SNORM: GLenum = 0x8F97;
pub(crate) const GL_RGBA16F: GLenum = 0x881A;
pub(crate) const GL_RG16F: GLenum = 0x822F;
pub(crate) const GL_R16F: GLenum = 0x822D;
pub(crate) const GL_RGBA32F: GLenum = 0x8814;
pub(crate) const GL_RGBA32UI: GLenum = 0x8D70;
pub(crate) const GL_RG32F: GLenum = 0x8230;
pub(crate) const GL_R32F: GLenum = 0x822E;
pub(crate) const GL_RGB10_A2: GLenum = 0x8059;
pub(crate) const GL_R11F_G11F_B10F: GLenum = 0x8C3A;
pub(crate) const GL_DEPTH_COMPONENT16: GLenum = 0x81A5;
pub(crate) const GL_DEPTH24_STENCIL8: GLenum = 0x88F0;
pub(crate) const GL_DEPTH_COMPONENT32F: GLenum = 0x8CAC;
pub(crate) const GL_DEPTH32F_STENCIL8: GLenum = 0x8CAD;
pub(crate) const GL_COMPRESSED_RGB_S3TC_DXT1_EXT: GLenum = 0x83F0;
pub(crate) const GL_COMPRESSED_RGBA_S3TC_DXT5_EXT: GLenum = 0x83F3;
pub(crate) const GL_COMPRESSED_RGBA_BPTC_UNORM: GLenum = 0x8E8C;
pub(crate) const GL_COMPRESSED_RGB8_ETC2: GLenum = 0x9274;
pub(crate) const GL_COMPRESSED_RGBA8_ETC2_EAC: GLenum = 0x9278;
pub(crate) const GL_RED: GLenum = 0x1903;
pub(crate) const GL_RG: GLenum = 0x8227;
pub(crate) const GL_RG_INTEGER: GLenum = 0x8228;
pub(crate) const GL_RGB: GLenum = 0x1907;
pub(crate) const GL_RGBA: GLenum = 0x1908;
pub(crate) const GL_RGBA_INTEGER: GLenum = 0x8D99;
pub(crate) const GL_DEPTH_COMPONENT: GLenum = 0x1902;
pub(crate) const GL_DEPTH_STENCIL: GLenum = 0x84F9;
pub(crate) const GL_UNSIGNED_BYTE: GLenum = 0x1401;
pub(crate) const GL_BYTE: GLenum = 0x1400;
pub(crate) const GL_SHORT: GLenum = 0x1402;
pub(crate) const GL_INT: GLenum = 0x1404;
pub(crate) const GL_UNSIGNED_INT: GLenum = 0x1405;
pub(crate) const GL_HALF_FLOAT: GLenum = 0x140B;
pub(crate) const GL_FLOAT: GLenum = 0x1406;
pub(crate) const GL_UNSIGNED_INT_2_10_10_10_REV: GLenum = 0x8368;
pub(crate) const GL_UNSIGNED_INT_10F_11F_11F_REV: GLenum = 0x8C3B;
pub(crate) const GL_UNSIGNED_SHORT: GLenum = 0x1403;
pub(crate) const GL_UNSIGNED_INT_24_8: GLenum = 0x84FA;
pub(crate) const GL_FLOAT_32_UNSIGNED_INT_24_8_REV: GLenum = 0x8DAD;
pub(crate) const GL_FRAMEBUFFER: GLenum = 0x8D40;
pub(crate) const GL_RENDERBUFFER: GLenum = 0x8D41;
pub(crate) const GL_FRAMEBUFFER_BINDING: GLenum = 0x8CA6;
pub(crate) const GL_READ_FRAMEBUFFER: GLenum = 0x8CA8;
pub(crate) const GL_DRAW_FRAMEBUFFER: GLenum = 0x8CA9;
pub(crate) const GL_FRAMEBUFFER_COMPLETE: GLenum = 0x8CD5;
pub(crate) const GL_COLOR_ATTACHMENT0: GLenum = 0x8CE0;
pub(crate) const GL_COLOR_ATTACHMENT1: GLenum = 0x8CE1;
pub(crate) const GL_DEPTH_ATTACHMENT: GLenum = 0x8D00;
pub(crate) const GL_STENCIL_ATTACHMENT: GLenum = 0x8D20;
pub(crate) const GL_DEPTH_STENCIL_ATTACHMENT: GLenum = 0x821A;
pub(crate) const GL_VERTEX_ARRAY_BINDING: GLenum = 0x85B5;
pub(crate) const GL_UNIFORM_BUFFER_BINDING: GLenum = 0x8A28;
pub(crate) const GL_CURRENT_PROGRAM: GLenum = 0x8B8D;
pub(crate) const GL_COLOR: GLenum = 0x1800;
pub(crate) const GL_DEPTH: GLenum = 0x1801;
pub(crate) const GL_STENCIL: GLenum = 0x1802;
pub(crate) const GL_TEXTURE_WRAP_R: GLenum = 0x8072;
pub(crate) const GL_TEXTURE_MIN_LOD: GLenum = 0x813A;
pub(crate) const GL_TEXTURE_MAX_LOD: GLenum = 0x813B;
pub(crate) const GL_TEXTURE_COMPARE_MODE: GLenum = 0x884C;
pub(crate) const GL_TEXTURE_COMPARE_FUNC: GLenum = 0x884D;
pub(crate) const GL_COMPARE_REF_TO_TEXTURE: GLenum = 0x884E;
pub(crate) const GL_COMPILE_STATUS: GLenum = 0x8B81;
pub(crate) const GL_LINK_STATUS: GLenum = 0x8B82;
pub(crate) const GL_INFO_LOG_LENGTH: GLenum = 0x8B84;
pub(crate) const GL_RENDERER: GLenum = 0x1F01;
pub(crate) const GL_VERSION: GLenum = 0x1F02;
pub(crate) const GL_EXTENSIONS: GLenum = 0x1F03;
pub(crate) const GL_NUM_EXTENSIONS: GLenum = 0x821D;
pub(crate) const GL_MAX_TEXTURE_SIZE: GLenum = 0x0D33;
pub(crate) const GL_MAX_3D_TEXTURE_SIZE: GLenum = 0x8073;
pub(crate) const GL_MAX_CUBE_MAP_TEXTURE_SIZE: GLenum = 0x851C;
pub(crate) const GL_MAX_DRAW_BUFFERS: GLenum = 0x8824;
pub(crate) const GL_MAX_COMBINED_TEXTURE_IMAGE_UNITS: GLenum = 0x8B4D;
pub(crate) const GL_MAX_VERTEX_ATTRIBS: GLenum = 0x8869;
pub(crate) const GL_MAX_UNIFORM_BLOCK_SIZE: GLenum = 0x8A30;
pub(crate) const GL_MAX_SAMPLES: GLenum = 0x8D57;
pub(crate) const GL_INVALID_INDEX: GLuint = 0xFFFF_FFFF;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GLCommand {
    Clear(GLbitfield),
    ClearColor(f32, f32, f32, f32),
    FrontFace(GLenum),
    DepthRange(f32, f32),
    DepthFunc(GLenum),
    ClearDepth(f32),
    ClearStencil(i32),
    Enable(GLenum),
    Disable(GLenum),
    PixelStore(GLenum, i32),
    BindAttribLocation {
        program: GLuint,
        index: GLuint,
        name: Vec<u8>,
    },
    BindBuffer(GLenum, GLuint),
    BindBufferRange {
        target: GLenum,
        index: GLuint,
        buffer: GLuint,
        offset: u32,
        size: u32,
    },
    BindFramebuffer(GLenum, GLuint),
    BindFramebufferFromQuery(GLenum, u64),
    BindRenderbuffer(GLenum, GLuint),
    BindSampler(GLuint, GLuint),
    /// Provider forwards through retained `gl.pv`; absent extension is a
    /// source-specified silent no-op.
    ProvokingVertex(GLenum),
    Scissor(u32, u32, u32, u32),
    Viewport(i32, i32, i32, i32),
    PolygonOffset(f32, f32),
    CullFace(GLenum),
    BlendEquation(GLenum),
    BlendEquationSeparate(GLenum, GLenum),
    BlendFunc(GLenum, GLenum),
    BlendFuncSeparate(GLenum, GLenum, GLenum, GLenum),
    BlendColor(f32, f32, f32, f32),
    ColorMask(bool, bool, bool, bool),
    DepthMask(bool),
    StencilMask(GLuint),
    StencilMaskSeparate(GLenum, GLuint),
    StencilFunc(GLenum, i32, GLuint),
    StencilOp(GLenum, GLenum, GLenum),
    StencilFuncSeparate(GLenum, GLenum, i32, GLuint),
    StencilOpSeparate(GLenum, GLenum, GLenum, GLenum),
    UseProgram(GLuint),
    BindVertexArray(GLuint),
    BindVertexArrayFromQuery(u64),
    ClearBufferDepthStencil {
        buffer: GLenum,
        drawbuffer: GLint,
        depth: f32,
        stencil: GLint,
    },
    ClearBufferFloat {
        buffer: GLenum,
        drawbuffer: GLint,
        values: [f32; 4],
        value_count: u8,
    },
    ClearBufferInt {
        buffer: GLenum,
        drawbuffer: GLint,
        values: [GLint; 4],
        value_count: u8,
    },
    ClearBufferUInt {
        buffer: GLenum,
        drawbuffer: GLint,
        values: [GLuint; 4],
        value_count: u8,
    },
    EnableVertexAttribArray(GLuint),
    DisableVertexAttribArray(GLuint),
    VertexAttribIPointer {
        index: GLuint,
        size: GLint,
        type_: GLenum,
        stride: GLsizei,
        offset: u32,
    },
    VertexAttribPointer {
        index: GLuint,
        size: GLint,
        type_: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        offset: u32,
    },
    VertexAttribDivisor(GLuint, GLuint),
    DrawArrays {
        mode: GLenum,
        first: u32,
        count: u32,
    },
    DrawArraysInstanced {
        mode: GLenum,
        first: u32,
        count: u32,
        instanceCount: u32,
    },
    DrawElements {
        mode: GLenum,
        count: u32,
        type_: GLenum,
        offset: u32,
    },
    DrawElementsInstanced {
        mode: GLenum,
        count: u32,
        type_: GLenum,
        offset: u32,
        instanceCount: u32,
    },
    DrawElementsInstancedBaseInstance {
        mode: GLenum,
        count: u32,
        type_: GLenum,
        offset: u32,
        instance_count: u32,
        base_instance: u32,
    },
    BlendBarrierKHR,
    ReadBuffer(GLenum),
    FramebufferTexture2D {
        target: GLenum,
        attachment: GLenum,
        texture_target: GLenum,
        texture: GLuint,
        level: GLint,
    },
    FramebufferTextureLayer {
        target: GLenum,
        attachment: GLenum,
        texture: GLuint,
        level: GLint,
        layer: GLint,
    },
    FramebufferRenderbuffer {
        target: GLenum,
        attachment: GLenum,
        renderbuffer_target: GLenum,
        renderbuffer: GLuint,
    },
    DrawBuffers(Vec<GLenum>),
    Flush,
    GenerateMipmap(GLenum),
    InvalidateFramebuffer {
        target: GLenum,
        attachments: Vec<GLenum>,
    },
    LineWidth(f32),
    /// Provider forwards through retained `gl.pls` and resolves the numeric
    /// name through the Emscripten/WebGL texture table; absent PLS is a no-op.
    FramebufferTexturePixelLocalStorageANGLE {
        plane: GLint,
        backing_texture: GLuint,
        level: GLint,
        layer: GLint,
        usage: GLenum,
    },
    /// Provider forwards a fresh four-float value through retained `gl.pls`;
    /// absent PLS is a source-specified silent no-op.
    FramebufferPixelLocalClearValuefvANGLE {
        plane: GLint,
        value: [GLfloat; 4],
    },
    /// Exact synchronous `HEAPU32` operation range after Rust ownership
    /// conversion. Providers consume every element before returning and
    /// silently no-op when no `gl.pls` object has been retained.
    BeginPixelLocalStorageANGLE {
        load_ops: Vec<GLenum>,
    },
    /// Exact synchronous `HEAPU32` operation range after Rust ownership
    /// conversion. Providers consume every element before returning and
    /// silently no-op when no `gl.pls` object has been retained.
    EndPixelLocalStorageANGLE {
        store_ops: Vec<GLenum>,
    },
    RenderbufferStorageMultisample {
        target: GLenum,
        samples: GLsizei,
        internal_format: GLenum,
        width: u32,
        height: u32,
    },
    TexStorage2D {
        target: GLenum,
        levels: u32,
        internal_format: GLenum,
        width: u32,
        height: u32,
    },
    TexStorage3D {
        target: GLenum,
        levels: u32,
        internal_format: GLenum,
        width: u32,
        height: u32,
        depth: u32,
    },
    DeleteProgram(GLuint),
    DeleteVertexArray(GLuint),
    DeleteBuffer(GLuint),
    DeleteTexture(GLuint),
    DeleteFramebuffer(GLuint),
    DeleteRenderbuffer(GLuint),
    DeleteSampler(GLuint),
    GenerateBuffer(GLuint),
    GenerateTexture(GLuint),
    GenerateFramebuffer(GLuint),
    GenerateRenderbuffer(GLuint),
    GenerateSampler(GLuint),
    GenerateVertexArray(GLuint),
    CreateProgram(GLuint),
    CreateShader(GLenum, GLuint),
    SamplerParameterFloat {
        sampler: GLuint,
        parameter: GLenum,
        value: f32,
    },
    SamplerParameterInt {
        sampler: GLuint,
        parameter: GLenum,
        value: GLint,
    },
    ShaderSource(GLuint, String),
    ShaderSourceBytes {
        shader: GLuint,
        source: Option<Vec<u8>>,
    },
    ShaderSourceBypassingEmscripten {
        shader: GLuint,
        minimal_source: String,
        raw_source: String,
    },
    CompileShader(GLuint),
    PrintShaderCompilationErrors(GLuint),
    ValidateShaderCompilationAndAbort {
        shader: GLuint,
        stderr_flush_delay_ms: u64,
    },
    DeleteShader(GLuint),
    AttachShader(GLuint, GLuint),
    LinkProgram(GLuint),
    PrintLinkProgramErrors(GLuint),
    ValidateProgramLinkAndAbort(GLuint),
    TextureParameter(GLenum, GLenum, i32),
    BlitFramebuffer([i32; 8], GLbitfield, GLenum),
    Uniform1iByName(GLuint, String, GLint),
    GetInteger(GLenum, u64),
    BindBufferFromQuery(GLenum, u64),
    BufferSubData {
        target: GLenum,
        offset: u32,
        data: Vec<u8>,
    },
    BufferData {
        target: GLenum,
        size: usize,
        data: Option<Vec<u8>>,
        usage: GLenum,
    },
    ActiveTexture(GLenum),
    BindTexture(GLenum, GLuint),
    CompressedTexSubImage2D {
        target: GLenum,
        level: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        format: GLenum,
        data: Vec<u8>,
    },
    CompressedTexSubImage3D {
        target: GLenum,
        level: u32,
        x: u32,
        y: u32,
        z: u32,
        width: u32,
        height: u32,
        depth: u32,
        format: GLenum,
        data: Vec<u8>,
    },
    TexSubImage2D {
        target: GLenum,
        level: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        format: GLenum,
        type_: GLenum,
        data: Vec<u8>,
    },
    TexSubImage3D {
        target: GLenum,
        level: u32,
        x: u32,
        y: u32,
        z: u32,
        width: u32,
        height: u32,
        depth: u32,
        format: GLenum,
        type_: GLenum,
        data: Vec<u8>,
    },
    PixelStoreFromQuery(GLenum, u64),
    Uniform1iLocation {
        location: GLint,
        value: GLint,
    },
    UniformBlockBinding {
        program: GLuint,
        block_index: GLuint,
        binding: GLuint,
    },
}

/// Required browser/native GL authority for one concrete current context.
/// Every submitted command executes before returning, and every query/name
/// method executes all preceding commands before publishing its result.
/// There is deliberately no production recorder or fabricated default.
pub(crate) trait GLExecutionProvider {
    /// Installs the weak, owner-thread lifecycle ingress that a browser
    /// adapter wires to `webglcontextlost` / `webglcontextrestored`. The
    /// callback must be invoked only after this method returns; synchronous
    /// provider reentry is rejected without changing domain state.
    fn installContextLifecycleIngress(&mut self, ingress: GLContextLifecycleIngress);

    /// Installs the weak owner-thread drain ingress and returns a thread-safe
    /// host handle whose `post()` schedules that ingress asynchronously. This
    /// is separate from GL provider callbacks: finalizer wakeups must never
    /// synchronously reenter the active provider or renderer stack.
    fn installFinalReleaseIngress(
        &mut self,
        ingress: GLFinalReleaseIngress,
    ) -> Arc<dyn nuxie_ore_metal::gpu_resource::ResourceFinalReleaseWake>;

    fn submit(&mut self, command: GLCommand);

    fn generateObject(&mut self, kind: GLObjectKind) -> GLuint;
    fn createProgram(&mut self) -> GLuint;
    fn createShader(&mut self, shaderType: GLenum) -> GLuint;

    fn getInteger(&mut self, parameter: GLenum) -> GLint;
    fn getString(&mut self, parameter: GLenum) -> Option<Vec<u8>>;
    fn getExtension(&mut self, index: GLuint) -> Option<Vec<u8>>;
    fn enableWebGLExtension(&mut self, name: &str) -> bool;

    /// Implements `enable_WEBGL_shader_pixel_local_storage_coherent` exactly:
    /// request `WEBGL_shader_pixel_local_storage`, require `isCoherent()`,
    /// require the 2026 five-argument framebuffer-texture entry point, warn on
    /// deprecated arity using the supplied exact source warning, and retain
    /// the accepted extension on this context. The four-valued result prevents
    /// a browser adapter from hiding any source branch behind one bool.
    #[cfg(not(test))]
    fn enableWebGLShaderPixelLocalStorageCoherent(
        &mut self,
        deprecatedVersionWarning: &'static str,
    ) -> WebGLShaderPixelLocalStorageEnableResult;
    #[cfg(test)]
    fn enableWebGLShaderPixelLocalStorageCoherent(
        &mut self,
        _deprecatedVersionWarning: &'static str,
    ) -> WebGLShaderPixelLocalStorageEnableResult {
        WebGLShaderPixelLocalStorageEnableResult::ExtensionUnavailable
    }

    /// Implements `enable_WEBGL_provoking_vertex` exactly: request the
    /// extension, retain the returned object as this context's `gl.pv`, and
    /// return its JavaScript truthiness. Later `ProvokingVertex` commands must
    /// forward through that retained object and silently no-op when absent.
    #[cfg(not(test))]
    fn enableWebGLProvokingVertex(&mut self) -> bool;
    #[cfg(test)]
    fn enableWebGLProvokingVertex(&mut self) -> bool {
        false
    }

    /// Synchronous `gl.pls.getFramebufferPixelLocalStorageParameterWEBGL`
    /// bridge. Providers return zero when no retained `gl.pls` exists.
    #[cfg(not(test))]
    fn getFramebufferPixelLocalStorageParameter(
        &mut self,
        plane: GLint,
        parameter: GLenum,
    ) -> GLint;
    #[cfg(test)]
    fn getFramebufferPixelLocalStorageParameter(
        &mut self,
        _plane: GLint,
        _parameter: GLenum,
    ) -> GLint {
        0
    }
    fn isObject(&mut self, kind: GLObjectKind, name: GLuint) -> bool;
    fn checkFramebufferStatus(&mut self, target: GLenum) -> GLenum;
    fn shaderParameter(&mut self, shader: GLuint, parameter: GLenum) -> GLint;
    fn shaderInfoLog(&mut self, shader: GLuint, maxLength: usize) -> Vec<u8>;
    fn programParameter(&mut self, program: GLuint, parameter: GLenum) -> GLint;
    fn programInfoLog(&mut self, program: GLuint, maxLength: usize) -> Vec<u8>;
    fn uniformBlockIndex(&mut self, program: GLuint, name: &[u8]) -> GLuint;
    fn uniformLocation(&mut self, program: GLuint, name: &[u8]) -> GLint;
    fn readPixelsRGBA8(&mut self, x: i32, y: i32, width: u32, height: u32) -> Vec<u8>;

    fn contextLost(&mut self, nextGeneration: u64);
}

struct GLExecutionDomainInner {
    key: u64,
    generation: Cell<u64>,
    live: Cell<bool>,
    rendererRetired: Cell<bool>,
    ownerThread: std::thread::ThreadId,
    provider: RefCell<Box<dyn GLExecutionProvider>>,
    callingProvider: Cell<bool>,
    finalReleaseDrain: RefCell<Option<nuxie_ore_metal::gpu_resource::ResourceFinalReleaseDrain>>,
    finalReleaseExecutionDomain:
        RefCell<Option<nuxie_ore_metal::gpu_resource::ResourceFinalReleaseExecutionDomain>>,
    drainingFinalReleases: Cell<bool>,
}

/// Thread-affine lifetime root shared by RenderContextGLImpl, ContextGL, GL
/// state, render targets, passes, PLS, and their resource-finalization queue.
#[derive(Clone)]
pub(crate) struct GLExecutionDomain(Rc<GLExecutionDomainInner>);

/// Weak production ingress retained by the browser/provider adapter. It does
/// not keep a renderer or provider alive and is deliberately thread-affine,
/// matching WebGL's event-loop ownership.
#[derive(Clone)]
pub(crate) struct GLContextLifecycleIngress(Weak<GLExecutionDomainInner>);

/// Weak owner-thread endpoint retained by the host adapter. A thread-safe
/// wake posts to the host loop; the posted task invokes this endpoint later,
/// outside provider and renderer call stacks.
#[derive(Clone)]
pub(crate) struct GLFinalReleaseIngress(Weak<GLExecutionDomainInner>);

#[cfg(test)]
#[derive(Default)]
pub(crate) struct TestFinalReleaseWake(std::sync::atomic::AtomicUsize);

#[cfg(test)]
impl TestFinalReleaseWake {
    pub(crate) fn takePosts(&self) -> usize {
        self.0.swap(0, Ordering::AcqRel)
    }
}

#[cfg(test)]
impl nuxie_ore_metal::gpu_resource::ResourceFinalReleaseWake for TestFinalReleaseWake {
    fn post(&self) {
        self.0.fetch_add(1, Ordering::Release);
    }
}

/// Browser restoration cannot revive a graph stamped with the lost
/// generation. The adapter must discard the old renderer/provider and create
/// a new root against the restored WebGL context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GLContextRecovery {
    RecreateRenderer,
}

impl GLContextLifecycleIngress {
    pub(crate) fn contextLost(&self) -> bool {
        let Some(inner) = self.0.upgrade() else {
            return false;
        };
        GLExecutionDomain(inner).markContextLost();
        true
    }

    pub(crate) fn contextRestored(&self) -> Option<GLContextRecovery> {
        let Some(inner) = self.0.upgrade() else {
            return None;
        };
        let domain = GLExecutionDomain(inner);
        domain.assertProviderNotActive();
        assert!(
            domain.isOwnerThread(),
            "GL context restoration is observed on its owner thread"
        );
        if domain.isRendererRetired() {
            // The browser may deliver a queued restoration event after the
            // old renderer root was explicitly discarded. Its listener is
            // stale and must not request a second replacement renderer.
            return None;
        }
        assert!(
            !domain.isLive(),
            "a live GL renderer does not require context restoration"
        );
        Some(GLContextRecovery::RecreateRenderer)
    }
}

impl GLFinalReleaseIngress {
    pub(crate) fn drainFinalReleases(&self) -> bool {
        let Some(inner) = self.0.upgrade() else {
            return false;
        };
        GLExecutionDomain(inner).drainPostedFinalReleases();
        true
    }
}

impl Drop for GLExecutionDomainInner {
    fn drop(&mut self) {
        assert_eq!(
            self.ownerThread,
            std::thread::current().id(),
            "GL execution domain must retire on its creation thread"
        );
        // Every queued concrete GL payload owns a strong GLExecutionStamp, so
        // it would retain this domain and prevent `GLExecutionDomainInner`
        // from reaching Drop. This raw terminal drain can therefore contain
        // only unstamped/synthetic callbacks and must not reenter GL.
        self.live.set(false);
        self.rendererRetired.set(true);
        let drain = self.finalReleaseDrain.get_mut().take();
        let executionDomain = self.finalReleaseExecutionDomain.get_mut().take();
        if let Some(drain) = drain.as_ref() {
            let executionDomain = executionDomain
                .as_ref()
                .expect("a bound GL final-release drain retains its authority");
            drain
                .close_in_execution_domain(executionDomain)
                .expect("final GL domain close uses its retained authority");
            drain
                .drain_in_execution_domain(executionDomain)
                .expect("final GL domain drain runs on its owner thread");
        }
    }
}

/// Creation-time identity for an object that owns or operates on GL names.
/// The strong domain clone keeps the provider alive while the object is live;
/// the captured generation prevents a stale name from being submitted to a
/// replacement browser context that reused the same numeric GLuint.
#[derive(Clone)]
pub(crate) struct GLExecutionStamp {
    domain: GLExecutionDomain,
    generation: u64,
}

impl std::fmt::Debug for GLExecutionStamp {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("GLExecutionStamp")
            .field("domainKey", &self.domain.key())
            .field("generation", &self.generation)
            .finish()
    }
}

static NEXT_GL_EXECUTION_DOMAIN_KEY: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static CURRENT_GL_EXECUTION_DOMAINS: RefCell<Vec<Weak<GLExecutionDomainInner>>> =
        const { RefCell::new(Vec::new()) };
}

struct CurrentGLExecutionDomainGuard {
    key: u64,
}

impl Drop for CurrentGLExecutionDomainGuard {
    fn drop(&mut self) {
        CURRENT_GL_EXECUTION_DOMAINS.with(|domains| {
            let domain = domains
                .borrow_mut()
                .pop()
                .and_then(|domain| domain.upgrade())
                .expect("current GL execution-domain stack underflow");
            assert_eq!(
                domain.key, self.key,
                "GL execution-domain guards must unwind in stack order"
            );
        });
    }
}

impl GLExecutionDomain {
    pub(crate) fn new(provider: Box<dyn GLExecutionProvider>) -> Self {
        let key = NEXT_GL_EXECUTION_DOMAIN_KEY.fetch_add(1, Ordering::Relaxed);
        assert_ne!(key, 0, "GL execution-domain key overflow");
        let domain = Self(Rc::new(GLExecutionDomainInner {
            key,
            generation: Cell::new(1),
            live: Cell::new(true),
            rendererRetired: Cell::new(false),
            ownerThread: std::thread::current().id(),
            provider: RefCell::new(provider),
            callingProvider: Cell::new(false),
            finalReleaseDrain: RefCell::new(None),
            finalReleaseExecutionDomain: RefCell::new(None),
            drainingFinalReleases: Cell::new(false),
        }));
        domain.installFinalReleaseDrain(
            nuxie_ore_metal::gpu_resource::ResourceFinalReleaseDrain::new(),
        );
        let lifecycleIngress = GLContextLifecycleIngress(Rc::downgrade(&domain.0));
        let finalReleaseIngress = GLFinalReleaseIngress(Rc::downgrade(&domain.0));
        let wake = domain.withLifecycleProvider(|provider| {
            provider.installContextLifecycleIngress(lifecycleIngress);
            provider.installFinalReleaseIngress(finalReleaseIngress)
        });
        domain.resourceFinalReleaseDrain().install_wake(wake);
        domain
    }

    pub(crate) fn key(&self) -> u64 {
        self.0.key
    }

    pub(crate) fn generation(&self) -> u64 {
        self.0.generation.get()
    }

    pub(crate) fn isLive(&self) -> bool {
        self.0.live.get()
    }

    pub(crate) fn isRendererRetired(&self) -> bool {
        self.0.rendererRetired.get()
    }

    pub(crate) fn isOwnerThread(&self) -> bool {
        self.0.ownerThread == std::thread::current().id()
    }

    pub(crate) fn stamp(&self) -> GLExecutionStamp {
        self.assertUsable();
        assert!(
            !self.isRendererRetired(),
            "a retired GL renderer cannot stamp new source objects"
        );
        GLExecutionStamp {
            domain: self.clone(),
            generation: self.generation(),
        }
    }

    pub(crate) fn installFinalReleaseDrain(
        &self,
        drain: nuxie_ore_metal::gpu_resource::ResourceFinalReleaseDrain,
    ) {
        self.assertUsable();
        assert!(
            self.0.finalReleaseDrain.borrow().is_none(),
            "GL execution domain accepts one ORE final-release drain"
        );
        let executionDomain = drain
            .bind_execution_domain(self.key())
            .expect("an ORE final-release drain belongs to exactly one GL execution domain");
        *self.0.finalReleaseDrain.borrow_mut() = Some(drain);
        *self.0.finalReleaseExecutionDomain.borrow_mut() = Some(executionDomain);
    }

    pub(crate) fn ownerThreadFinalReleaseRoute(
        &self,
    ) -> nuxie_ore_metal::gpu_resource::OwnerThreadFinalReleaseRoute {
        self.0
            .finalReleaseDrain
            .borrow()
            .as_ref()
            .expect("GL owner-thread finalization requires an installed ORE drain")
            .owner_thread_route()
    }

    pub(crate) fn resourceFinalReleaseDrain(
        &self,
    ) -> nuxie_ore_metal::gpu_resource::ResourceFinalReleaseDrain {
        self.0
            .finalReleaseDrain
            .borrow()
            .as_ref()
            .expect("GL resource final-release drain is initialized with the domain")
            .clone()
    }

    pub(crate) fn drainFinalReleases(&self) -> usize {
        self.assertProviderNotActive();
        assert_eq!(
            currentGLExecutionDomain()
                .as_ref()
                .map(GLExecutionDomain::key),
            Some(self.key()),
            "GL final releases require their own current execution-domain scope"
        );
        self.drainFinalReleasesAuthorized()
    }

    fn drainFinalReleasesAuthorized(&self) -> usize {
        assert!(
            self.isOwnerThread(),
            "GL final releases drain only on the context owner thread"
        );
        if self.0.drainingFinalReleases.replace(true) {
            return 0;
        }

        struct DrainGuard<'a>(&'a Cell<bool>);
        impl Drop for DrainGuard<'_> {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }
        let _guard = DrainGuard(&self.0.drainingFinalReleases);
        let drain = self.0.finalReleaseDrain.borrow().clone();
        let executionDomain = self.0.finalReleaseExecutionDomain.borrow();
        drain.map_or(0, |drain| {
            drain
                .drain_in_execution_domain(
                    executionDomain
                        .as_ref()
                        .expect("a bound GL final-release drain retains its authority"),
                )
                .expect("GL resource final releases drain only on the context owner thread")
        })
    }

    fn drainPostedFinalReleases(&self) {
        self.assertProviderNotActive();
        assert!(
            self.isOwnerThread(),
            "posted GL final releases run on the context owner thread"
        );
        if self.isLive() {
            self.withCurrent(|| {});
        } else {
            // A lost context has no valid ambient GL scope. Every routed
            // concrete destructor carries its creation stamp and drops Rust
            // ownership while suppressing stale numeric-name deletion.
            self.drainFinalReleasesAuthorized();
        }
    }

    fn assertUsable(&self) {
        assert!(
            self.isOwnerThread(),
            "GL execution is confined to its owning context thread"
        );
        assert!(self.isLive(), "GL execution domain is context-lost");
    }

    fn assertProviderNotActive(&self) {
        assert!(
            !self.0.callingProvider.get(),
            "GLExecutionProvider callback reentry is unsupported"
        );
    }

    pub(crate) fn withCurrent<R>(&self, callback: impl FnOnce() -> R) -> R {
        self.assertProviderNotActive();
        self.assertUsable();
        let nestedSameDomain = CURRENT_GL_EXECUTION_DOMAINS.with(|domains| {
            let nested = domains.borrow().iter().any(|domain| {
                domain
                    .upgrade()
                    .is_some_and(|domain| domain.key == self.key())
            });
            domains.borrow_mut().push(Rc::downgrade(&self.0));
            nested
        });
        let _guard = CurrentGLExecutionDomainGuard { key: self.key() };
        if !nestedSameDomain {
            self.drainFinalReleases();
        }
        let result = callback();
        if !nestedSameDomain {
            self.drainFinalReleases();
        }
        result
    }

    /// Establishes the exact current execution authority beneath a translated
    /// source `RefCell` borrow. The borrow already exists when this method is
    /// entered, so running queued concrete destructors here could reborrow the
    /// same source object and panic. Normal renderer entry points use
    /// `withCurrent`; this narrow scope deliberately leaves the FIFO for the
    /// surrounding or next independent safe boundary.
    pub(crate) fn withCurrentWhileSourceBorrowed<R>(&self, callback: impl FnOnce() -> R) -> R {
        self.assertProviderNotActive();
        self.assertUsable();
        CURRENT_GL_EXECUTION_DOMAINS.with(|domains| {
            domains.borrow_mut().push(Rc::downgrade(&self.0));
        });
        let _guard = CurrentGLExecutionDomainGuard { key: self.key() };
        callback()
    }

    fn withProvider<R>(&self, callback: impl FnOnce(&mut dyn GLExecutionProvider) -> R) -> R {
        // Finalizers drain only at the outermost same-domain `withCurrent`
        // boundaries. Provider calls and nested scopes may run while source
        // RefCell guards are live and are therefore never independent safe
        // points for concrete destruction.
        self.assertProviderNotActive();
        self.assertUsable();
        assert_eq!(
            currentGLExecutionDomain()
                .as_ref()
                .map(GLExecutionDomain::key),
            Some(self.key()),
            "GL provider access requires its current execution-domain scope"
        );
        assert!(
            !self.0.callingProvider.replace(true),
            "GLExecutionProvider callback reentry is unsupported"
        );
        struct ProviderCallGuard<'a>(&'a Cell<bool>);
        impl Drop for ProviderCallGuard<'_> {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }
        let _guard = ProviderCallGuard(&self.0.callingProvider);
        callback(&mut **self.0.provider.borrow_mut())
    }

    fn withLifecycleProvider<R>(
        &self,
        callback: impl FnOnce(&mut dyn GLExecutionProvider) -> R,
    ) -> R {
        assert!(
            self.isOwnerThread(),
            "GL provider lifecycle callbacks run on the context owner thread"
        );
        assert!(
            !self.0.callingProvider.replace(true),
            "GLExecutionProvider callback reentry is unsupported"
        );
        struct ProviderCallGuard<'a>(&'a Cell<bool>);
        impl Drop for ProviderCallGuard<'_> {
            fn drop(&mut self) {
                self.0.set(false);
            }
        }
        let _guard = ProviderCallGuard(&self.0.callingProvider);
        callback(&mut **self.0.provider.borrow_mut())
    }

    pub(crate) fn submit(&self, command: GLCommand) {
        self.withProvider(|provider| provider.submit(command));
    }

    pub(crate) fn generateObject(&self, kind: GLObjectKind) -> GLuint {
        self.withProvider(|provider| provider.generateObject(kind))
    }

    pub(crate) fn createProgram(&self) -> GLuint {
        self.withProvider(|provider| provider.createProgram())
    }

    pub(crate) fn createShader(&self, shaderType: GLenum) -> GLuint {
        self.withProvider(|provider| provider.createShader(shaderType))
    }

    pub(crate) fn getInteger(&self, parameter: GLenum) -> GLint {
        self.withProvider(|provider| provider.getInteger(parameter))
    }

    pub(crate) fn getString(&self, parameter: GLenum) -> Option<Vec<u8>> {
        self.withProvider(|provider| provider.getString(parameter))
    }

    pub(crate) fn getExtension(&self, index: GLuint) -> Option<Vec<u8>> {
        self.withProvider(|provider| provider.getExtension(index))
    }

    pub(crate) fn enableWebGLExtension(&self, name: &str) -> bool {
        self.withProvider(|provider| provider.enableWebGLExtension(name))
    }

    pub(crate) fn enableWebGLShaderPixelLocalStorageCoherent(
        &self,
        deprecatedVersionWarning: &'static str,
    ) -> WebGLShaderPixelLocalStorageEnableResult {
        self.withProvider(|provider| {
            provider.enableWebGLShaderPixelLocalStorageCoherent(deprecatedVersionWarning)
        })
    }

    pub(crate) fn enableWebGLProvokingVertex(&self) -> bool {
        self.withProvider(|provider| provider.enableWebGLProvokingVertex())
    }

    pub(crate) fn getFramebufferPixelLocalStorageParameter(
        &self,
        plane: GLint,
        parameter: GLenum,
    ) -> GLint {
        self.withProvider(|provider| {
            provider.getFramebufferPixelLocalStorageParameter(plane, parameter)
        })
    }

    pub(crate) fn isObject(&self, kind: GLObjectKind, name: GLuint) -> bool {
        self.withProvider(|provider| provider.isObject(kind, name))
    }

    pub(crate) fn checkFramebufferStatus(&self, target: GLenum) -> GLenum {
        self.withProvider(|provider| provider.checkFramebufferStatus(target))
    }

    pub(crate) fn shaderParameter(&self, shader: GLuint, parameter: GLenum) -> GLint {
        self.withProvider(|provider| provider.shaderParameter(shader, parameter))
    }

    pub(crate) fn shaderInfoLog(&self, shader: GLuint, maxLength: usize) -> Vec<u8> {
        self.withProvider(|provider| provider.shaderInfoLog(shader, maxLength))
    }

    pub(crate) fn programParameter(&self, program: GLuint, parameter: GLenum) -> GLint {
        self.withProvider(|provider| provider.programParameter(program, parameter))
    }

    pub(crate) fn programInfoLog(&self, program: GLuint, maxLength: usize) -> Vec<u8> {
        self.withProvider(|provider| provider.programInfoLog(program, maxLength))
    }

    pub(crate) fn uniformBlockIndex(&self, program: GLuint, name: &[u8]) -> GLuint {
        self.withProvider(|provider| provider.uniformBlockIndex(program, name))
    }

    pub(crate) fn uniformLocation(&self, program: GLuint, name: &[u8]) -> GLint {
        self.withProvider(|provider| provider.uniformLocation(program, name))
    }

    pub(crate) fn readPixelsRGBA8(&self, x: i32, y: i32, width: u32, height: u32) -> Vec<u8> {
        self.withProvider(|provider| provider.readPixelsRGBA8(x, y, width, height))
    }

    pub(crate) fn markContextLost(&self) {
        self.assertProviderNotActive();
        assert!(
            self.isOwnerThread(),
            "context loss must publish on its owner thread"
        );
        if !self.isLive() {
            return;
        }
        // Destroy every already-released generation-N resource while
        // generation N is still current. Later releases retain their creation
        // stamp and will quarantine their stale GL deletion after restore.
        self.withCurrent(|| {});
        self.0.live.set(false);
        let nextGeneration = self
            .generation()
            .checked_add(1)
            .expect("GL context generation overflow");
        self.0.generation.set(nextGeneration);
        self.withLifecycleProvider(|provider| provider.contextLost(nextGeneration));
    }

    /// Retire the renderer root without closing its final-release route or
    /// invalidating its still-current context generation. Source-external
    /// Canvas/image/target handles may outlive the renderer; their later zero
    /// transitions preserve exact deletion authority until actual context
    /// loss. Owner-thread releases run in source RAII order; worker releases
    /// post asynchronously to `GLFinalReleaseIngress`. Queued stamped owners
    /// retain this domain; a delivered task weak-upgrades it only for the
    /// drain. The route closes after the last stamped owner and temporary
    /// ingress upgrade disappear.
    pub(crate) fn retireRenderer(&self) {
        self.assertProviderNotActive();
        assert!(
            self.isOwnerThread(),
            "GL renderer retirement is confined to its owner thread"
        );
        assert!(
            !self.0.rendererRetired.replace(true),
            "GL renderer root retires exactly once"
        );
        if self.isLive() {
            // Complete every release that predates retirement. Later releases
            // use the retained provider and generation until real context
            // loss, matching source RAII for handles that outlive the root.
            self.withCurrent(|| {});
        }
    }

    /// Explicit terminal owner-thread teardown for construction failure and
    /// tests that prove no source-external stamped owner survives. Normal
    /// RenderContext destruction uses `retireRenderer()` instead.
    pub(crate) fn shutdown(&self) {
        self.assertProviderNotActive();
        assert!(
            self.isOwnerThread(),
            "GL execution-domain shutdown is confined to its owner thread"
        );
        self.0.rendererRetired.set(true);
        let drain = self.0.finalReleaseDrain.borrow_mut().take();
        let executionDomain = self.0.finalReleaseExecutionDomain.borrow_mut().take();
        if let Some(drain) = drain.as_ref() {
            // Close first while holding the queue's synchronization boundary;
            // every producer racing with terminal teardown is then either in
            // this final drain or rejected and quarantined.
            drain
                .close_in_execution_domain(
                    executionDomain
                        .as_ref()
                        .expect("a bound GL final-release drain retains its authority"),
                )
                .expect("terminal GL final-release close uses its retained authority");
        }
        if self.isLive() {
            self.withCurrent(|| {
                if let Some(drain) = drain.as_ref() {
                    drain
                        .drain_in_execution_domain(
                            executionDomain
                                .as_ref()
                                .expect("a bound GL final-release drain retains its authority"),
                        )
                        .expect("terminal GL final-release drain runs on its owner thread");
                }
            });
            self.0.live.set(false);
        } else if let Some(drain) = drain.as_ref() {
            // Context loss invalidates numeric GL names, not Rust ownership.
            // Run the terminal FIFO raw on the owner thread so every queued
            // payload and strong execution stamp is released. Concrete GL
            // destructors consult their creation stamp and suppress stale
            // Delete* commands without requiring a current provider scope.
            drain
                .drain_in_execution_domain(
                    executionDomain
                        .as_ref()
                        .expect("a bound GL final-release drain retains its authority"),
                )
                .expect("lost-context terminal final-release drain runs on its owner thread");
        }
        // Dropping the detached route after `live = false` makes any producer
        // that violated the stop-before-shutdown contract quarantine safely.
        drop(drain);
        drop(executionDomain);
    }
}

impl GLExecutionStamp {
    pub(crate) fn domain(&self) -> &GLExecutionDomain {
        &self.domain
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    pub(crate) fn isCurrentGeneration(&self) -> bool {
        self.domain.isLive() && self.domain.generation() == self.generation
    }

    pub(crate) fn sameDomain(&self, other: &Self) -> bool {
        self.domain.key() == other.domain.key() && self.generation == other.generation
    }

    pub(crate) fn withCurrent<R>(&self, callback: impl FnOnce() -> R) -> R {
        assert_eq!(
            self.domain.generation(),
            self.generation,
            "GL object belongs to a stale context generation"
        );
        self.domain.withCurrent(callback)
    }

    /// Runs a GL destructor only while the object's creation generation is
    /// still current. Returning `None` deliberately suppresses stale GLuint
    /// deletion after loss/restore; the Rust allocation still tears down.
    pub(crate) fn withDeleteCurrent<R>(&self, callback: impl FnOnce() -> R) -> Option<R> {
        assert!(
            self.domain.isOwnerThread(),
            "GL destruction must be routed back to the context owner thread"
        );
        self.isCurrentGeneration()
            .then(|| self.domain.withCurrent(callback))
    }
}

fn currentGLExecutionDomain() -> Option<GLExecutionDomain> {
    CURRENT_GL_EXECUTION_DOMAINS.with(|domains| {
        domains
            .borrow()
            .last()
            .and_then(Weak::upgrade)
            .map(GLExecutionDomain)
    })
}

pub(crate) fn recordGLCommand(command: GLCommand) {
    if let Some(domain) = currentGLExecutionDomain() {
        domain.submit(command);
        return;
    }

    #[cfg(test)]
    TEST_GL_COMMAND_STREAM.with(|stream| stream.borrow_mut().commands.push(command));

    #[cfg(not(test))]
    panic!("GL command submitted without a current GLExecutionDomain");
}

pub(crate) fn generateGLObject(kind: GLObjectKind) -> GLuint {
    if let Some(domain) = currentGLExecutionDomain() {
        return domain.generateObject(kind);
    }

    #[cfg(test)]
    return TEST_GL_COMMAND_STREAM.with(|stream| {
        let mut stream = stream.borrow_mut();
        let name = stream.allocateName();
        stream.commands.push(match kind {
            GLObjectKind::Buffer => GLCommand::GenerateBuffer(name),
            GLObjectKind::Framebuffer => GLCommand::GenerateFramebuffer(name),
            GLObjectKind::Program => {
                panic!("program identities must come from createGLProgram")
            }
            GLObjectKind::Renderbuffer => GLCommand::GenerateRenderbuffer(name),
            GLObjectKind::Sampler => GLCommand::GenerateSampler(name),
            GLObjectKind::Texture => GLCommand::GenerateTexture(name),
            GLObjectKind::VertexArray => GLCommand::GenerateVertexArray(name),
        });
        name
    });

    #[cfg(not(test))]
    panic!("GL object generated without a current GLExecutionDomain");
}

pub(crate) fn createGLProgram() -> GLuint {
    if let Some(domain) = currentGLExecutionDomain() {
        return domain.createProgram();
    }

    #[cfg(test)]
    return TEST_GL_COMMAND_STREAM.with(|stream| {
        let mut stream = stream.borrow_mut();
        let name = stream.allocateName();
        stream.commands.push(GLCommand::CreateProgram(name));
        name
    });

    #[cfg(not(test))]
    panic!("GL program created without a current GLExecutionDomain");
}

pub(crate) fn createGLShader(shaderType: GLenum) -> GLuint {
    if let Some(domain) = currentGLExecutionDomain() {
        return domain.createShader(shaderType);
    }

    #[cfg(test)]
    return TEST_GL_COMMAND_STREAM.with(|stream| {
        let mut stream = stream.borrow_mut();
        let name = stream.allocateName();
        stream
            .commands
            .push(GLCommand::CreateShader(shaderType, name));
        name
    });

    #[cfg(not(test))]
    panic!("GL shader created without a current GLExecutionDomain");
}

#[cfg(test)]
#[derive(Debug)]
struct TestGLCommandStream {
    nextName: GLuint,
    nextQuerySlot: u64,
    commands: Vec<GLCommand>,
}

#[cfg(test)]
impl Default for TestGLCommandStream {
    fn default() -> Self {
        Self {
            nextName: 1,
            nextQuerySlot: 1,
            commands: Vec::new(),
        }
    }
}

#[cfg(test)]
impl TestGLCommandStream {
    fn allocateName(&mut self) -> GLuint {
        let name = self.nextName;
        self.nextName = self.nextName.checked_add(1).expect("GL name overflow");
        name
    }
}

#[cfg(test)]
thread_local! {
    static TEST_GL_COMMAND_STREAM: RefCell<TestGLCommandStream> =
        RefCell::new(TestGLCommandStream::default());
}

/// Transitional test-only spelling retained while all completed owners move
/// from virtual IDs to synchronous typed generation.
pub(crate) fn allocateGLName() -> GLuint {
    assert!(
        currentGLExecutionDomain().is_none(),
        "allocateGLName is forbidden with a production GLExecutionDomain; use typed generation"
    );

    #[cfg(test)]
    return TEST_GL_COMMAND_STREAM.with(|stream| stream.borrow_mut().allocateName());

    #[cfg(not(test))]
    panic!("allocateGLName is test-only; use generateGLObject/createGLProgram/createGLShader");
}

/// Transitional test-only query slots. Production domains query actual host
/// state synchronously and store the returned GLuint exactly.
pub(crate) fn allocateGLQuerySlot() -> u64 {
    assert!(
        currentGLExecutionDomain().is_none(),
        "query slots are forbidden with a production GLExecutionDomain"
    );

    #[cfg(test)]
    return TEST_GL_COMMAND_STREAM.with(|stream| {
        let mut stream = stream.borrow_mut();
        let slot = stream.nextQuerySlot;
        stream.nextQuerySlot = stream
            .nextQuerySlot
            .checked_add(1)
            .expect("GL query slot overflow");
        slot
    });

    #[cfg(not(test))]
    panic!("GL query slots are test-only; query the current execution domain synchronously");
}

pub(crate) fn takeGLCommands() -> Vec<GLCommand> {
    assert!(
        currentGLExecutionDomain().is_none(),
        "production GLExecutionDomain commands execute synchronously and cannot be taken"
    );

    #[cfg(test)]
    return TEST_GL_COMMAND_STREAM.with(|stream| std::mem::take(&mut stream.borrow_mut().commands));

    #[cfg(not(test))]
    panic!("there is no production GL command recorder");
}

#[cfg(test)]
pub(crate) fn resetGLCommandStream() {
    TEST_GL_COMMAND_STREAM.with(|stream| *stream.borrow_mut() = TestGLCommandStream::default());
}

pub(crate) const WEBGL_debug_renderer_info: u32 = 1;
pub(crate) const GL_UNMASKED_VENDOR_WEBGL: GLenum = 0x9245;
pub(crate) const GL_UNMASKED_RENDERER_WEBGL: GLenum = 0x9246;

pub(crate) const GL_ANGLE_shader_pixel_local_storage: u32 = 1;
pub(crate) const GL_MAX_PIXEL_LOCAL_STORAGE_PLANES_ANGLE: GLenum = 0x96E0;
pub(crate) const GL_MAX_COMBINED_DRAW_BUFFERS_AND_PIXEL_LOCAL_STORAGE_PLANES_ANGLE: GLenum = 0x96E1;
pub(crate) const GL_PIXEL_LOCAL_STORAGE_ACTIVE_PLANES_ANGLE: GLenum = 0x96E2;
pub(crate) const GL_LOAD_OP_ZERO_ANGLE: GLenum = 0x96E3;
pub(crate) const GL_LOAD_OP_CLEAR_ANGLE: GLenum = 0x96E4;
pub(crate) const GL_LOAD_OP_LOAD_ANGLE: GLenum = 0x96E5;
pub(crate) const GL_STORE_OP_STORE_ANGLE: GLenum = 0x96E6;
pub(crate) const GL_PIXEL_LOCAL_FORMAT_ANGLE: GLenum = 0x96E7;
pub(crate) const GL_PIXEL_LOCAL_TEXTURE_NAME_ANGLE: GLenum = 0x96E8;
pub(crate) const GL_PIXEL_LOCAL_TEXTURE_LEVEL_ANGLE: GLenum = 0x96E9;
pub(crate) const GL_PIXEL_LOCAL_TEXTURE_LAYER_ANGLE: GLenum = 0x96EA;
pub(crate) const GL_PIXEL_LOCAL_CLEAR_VALUE_FLOAT_ANGLE: GLenum = 0x96EB;
pub(crate) const GL_PIXEL_LOCAL_CLEAR_VALUE_INT_ANGLE: GLenum = 0x96EC;
pub(crate) const GL_PIXEL_LOCAL_CLEAR_VALUE_UNSIGNED_INT_ANGLE: GLenum = 0x96ED;

pub(crate) const GL_ANGLE_provoking_vertex: u32 = 1;
pub(crate) const GL_FIRST_VERTEX_CONVENTION_ANGLE: GLenum = 0x8E4D;
pub(crate) const GL_LAST_VERTEX_CONVENTION_ANGLE: GLenum = 0x8E4E;
pub(crate) const GL_PROVOKING_VERTEX_ANGLE: GLenum = 0x8E4F;

pub(crate) const GL_KHR_blend_equation_advanced: u32 = 1;
pub(crate) const GL_MULTIPLY_KHR: GLenum = 0x9294;
pub(crate) const GL_SCREEN_KHR: GLenum = 0x9295;
pub(crate) const GL_OVERLAY_KHR: GLenum = 0x9296;
pub(crate) const GL_DARKEN_KHR: GLenum = 0x9297;
pub(crate) const GL_LIGHTEN_KHR: GLenum = 0x9298;
pub(crate) const GL_COLORDODGE_KHR: GLenum = 0x9299;
pub(crate) const GL_COLORBURN_KHR: GLenum = 0x929A;
pub(crate) const GL_HARDLIGHT_KHR: GLenum = 0x929B;
pub(crate) const GL_SOFTLIGHT_KHR: GLenum = 0x929C;
pub(crate) const GL_DIFFERENCE_KHR: GLenum = 0x929E;
pub(crate) const GL_EXCLUSION_KHR: GLenum = 0x92A0;
pub(crate) const GL_HSL_HUE_KHR: GLenum = 0x92AD;
pub(crate) const GL_HSL_SATURATION_KHR: GLenum = 0x92AE;
pub(crate) const GL_HSL_COLOR_KHR: GLenum = 0x92AF;
pub(crate) const GL_HSL_LUMINOSITY_KHR: GLenum = 0x92B0;
pub(crate) const GL_BLEND_ADVANCED_COHERENT_KHR: GLenum = 0x9285;

pub(crate) const GL_EXT_clip_cull_distance: u32 = 1;
pub(crate) const GL_CLIP_DISTANCE0_EXT: GLenum = 0x3000;
pub(crate) const GL_CLIP_DISTANCE1_EXT: GLenum = 0x3001;
pub(crate) const GL_CLIP_DISTANCE2_EXT: GLenum = 0x3002;
pub(crate) const GL_CLIP_DISTANCE3_EXT: GLenum = 0x3003;
pub(crate) const GL_KHR_parallel_shader_compile: u32 = 1;
pub(crate) const GL_MAX_SHADER_COMPILER_THREADS_KHR: GLenum = 0x91B0;
pub(crate) const GL_COMPLETION_STATUS_KHR: GLenum = 0x91B1;

pub(crate) const GL_COMPRESSED_RGBA_ASTC_4x4_KHR: GLenum = 0x93B0;
pub(crate) const GL_COMPRESSED_RGBA_ASTC_6x6_KHR: GLenum = 0x93B4;
pub(crate) const GL_COMPRESSED_RGBA_ASTC_8x8_KHR: GLenum = 0x93B7;
pub(crate) const GL_COMPRESSED_RGBA_ASTC_12x12_KHR: GLenum = 0x93BD;
pub(crate) const GL_SHADER_STORAGE_BUFFER: GLenum = 0x90D2;
pub(crate) const GL_MAX_VERTEX_SHADER_STORAGE_BLOCKS: GLenum = 0x90D6;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GLCapabilities {
    pub(crate) isGLES: bool,
    pub(crate) isANGLESystemDriver: bool,
    pub(crate) isAdreno: bool,
    pub(crate) isMali: bool,
    pub(crate) isPowerVR: bool,
    pub(crate) contextVersionMajor: u32,
    pub(crate) contextVersionMinor: u32,
    pub(crate) vendorDriverVersionMajor: u32,
    pub(crate) vendorDriverVersionMinor: u32,
    pub(crate) adrenoSeries: u32,
    pub(crate) maxSupportedInstancesPerFlush: u32,
    pub(crate) needsFloatingPointTessellationTexture: bool,
    pub(crate) usePixelLocalStorage2AsWorkaround: bool,
    pub(crate) avoidTexture2DArrayWithWebGLPLS: bool,
    pub(crate) avoidPartialFramebufferBlits: bool,
    pub(crate) ANGLE_base_vertex_base_instance_shader_builtin: bool,
    pub(crate) ANGLE_shader_pixel_local_storage: bool,
    pub(crate) ANGLE_shader_pixel_local_storage_coherent: bool,
    pub(crate) ANGLE_polygon_mode: bool,
    pub(crate) ANGLE_provoking_vertex: bool,
    pub(crate) ARM_shader_framebuffer_fetch: bool,
    pub(crate) ARB_fragment_shader_interlock: bool,
    pub(crate) ARB_shader_image_load_store: bool,
    pub(crate) ARB_shader_storage_buffer_object: bool,
    pub(crate) OES_shader_image_atomic: bool,
    pub(crate) KHR_blend_equation_advanced: bool,
    pub(crate) KHR_blend_equation_advanced_coherent: bool,
    pub(crate) KHR_parallel_shader_compile: bool,
    pub(crate) EXT_base_instance: bool,
    pub(crate) EXT_clip_cull_distance: bool,
    pub(crate) EXT_color_buffer_half_float: bool,
    pub(crate) OES_texture_half_float_linear: bool,
    pub(crate) EXT_color_buffer_float: bool,
    pub(crate) EXT_float_blend: bool,
    pub(crate) EXT_multisampled_render_to_texture: bool,
    pub(crate) EXT_shader_framebuffer_fetch: bool,
    pub(crate) EXT_shader_pixel_local_storage: bool,
    pub(crate) EXT_shader_pixel_local_storage2: bool,
    pub(crate) INTEL_fragment_shader_ordering: bool,
    pub(crate) QCOM_shader_framebuffer_fetch_noncoherent: bool,
    pub(crate) EXT_texture_compression_s3tc: bool,
    pub(crate) EXT_texture_compression_bptc: bool,
    pub(crate) KHR_texture_compression_astc_ldr: bool,
    pub(crate) supportsETC2: bool,
}

impl GLCapabilities {
    pub(crate) const fn IsVersionAtLeast(
        aMajor: u32,
        aMinor: u32,
        bMajor: u32,
        bMinor: u32,
    ) -> bool {
        ((aMajor as u64) << 32 | aMinor as u64) >= ((bMajor as u64) << 32 | bMinor as u64)
    }

    pub(crate) const fn isContextVersionAtLeast(&self, major: u32, minor: u32) -> bool {
        Self::IsVersionAtLeast(
            self.contextVersionMajor,
            self.contextVersionMinor,
            major,
            minor,
        )
    }

    pub(crate) const fn isVendorDriverVersionAtLeast(&self, major: u32, minor: u32) -> bool {
        Self::IsVersionAtLeast(
            self.vendorDriverVersionMajor,
            self.vendorDriverVersionMinor,
            major,
            minor,
        )
    }
}

unsafe extern "C" {
    pub(crate) fn webgl_enable_WEBGL_shader_pixel_local_storage_coherent() -> bool;
    pub(crate) fn webgl_shader_pixel_local_storage_is_coherent() -> bool;
    pub(crate) fn glFramebufferTexturePixelLocalStorageANGLE(
        plane: GLint,
        backingtexture: GLuint,
        level: GLint,
        layer: GLint,
        usage: GLenum,
    );
    pub(crate) fn glFramebufferPixelLocalClearValuefvANGLE(plane: GLint, value: *const GLfloat);
    pub(crate) fn glBeginPixelLocalStorageANGLE(n: GLsizei, loadops: *const GLenum);
    pub(crate) fn glEndPixelLocalStorageANGLE(n: GLsizei, storeops: *const GLenum);
    pub(crate) fn glGetFramebufferPixelLocalStorageParameterivANGLE(
        plane: GLint,
        pname: GLenum,
        param: *mut GLint,
    );
    pub(crate) fn webgl_enable_WEBGL_provoking_vertex() -> bool;
    pub(crate) fn glProvokingVertexANGLE(provokeMode: GLenum);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, VecDeque};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[derive(Clone, Debug, PartialEq)]
    enum ProviderEvent {
        Submit(GLCommand),
        Generate(GLObjectKind, GLuint),
        CreateProgram(GLuint),
        CreateShader(GLenum, GLuint),
        GetInteger(GLenum, GLint),
        ContextLost(u64),
        FinalRelease,
        ProviderDrop,
    }

    struct FinalizerProbe(Rc<RefCell<Vec<ProviderEvent>>>);

    unsafe fn releaseFinalizerProbe(payload: usize) {
        let probe = unsafe { Box::from_raw(payload as *mut FinalizerProbe) };
        probe.0.borrow_mut().push(ProviderEvent::FinalRelease);
    }

    struct BorrowFinalizerProbe {
        borrowed: Rc<RefCell<usize>>,
        events: Rc<RefCell<Vec<ProviderEvent>>>,
    }

    unsafe fn releaseBorrowFinalizerProbe(payload: usize) {
        let probe = unsafe { Box::from_raw(payload as *mut BorrowFinalizerProbe) };
        *probe.borrowed.borrow_mut() += 1;
        probe.events.borrow_mut().push(ProviderEvent::FinalRelease);
    }

    struct StaleDeleteFinalizerProbe {
        events: Rc<RefCell<Vec<ProviderEvent>>>,
        stamp: GLExecutionStamp,
        name: GLuint,
    }

    unsafe fn releaseStaleDeleteFinalizerProbe(payload: usize) {
        let probe = unsafe { Box::from_raw(payload as *mut StaleDeleteFinalizerProbe) };
        let _ = probe
            .stamp
            .withDeleteCurrent(|| recordGLCommand(GLCommand::DeleteBuffer(probe.name)));
        probe.events.borrow_mut().push(ProviderEvent::FinalRelease);
    }

    #[derive(Clone, Copy, Debug)]
    enum ProviderReentryAction {
        Command,
        ContextLoss,
        Shutdown,
    }

    struct ProviderReentry {
        action: ProviderReentryAction,
        domain: Weak<GLExecutionDomainInner>,
        finalReleaseRoute: nuxie_ore_metal::gpu_resource::OwnerThreadFinalReleaseRoute,
    }

    struct RecordingProvider {
        events: Rc<RefCell<Vec<ProviderEvent>>>,
        names: VecDeque<GLuint>,
        integers: HashMap<GLenum, GLint>,
        lifecycleIngress: Rc<RefCell<Option<GLContextLifecycleIngress>>>,
        finalReleaseIngress: Rc<RefCell<Option<GLFinalReleaseIngress>>>,
        finalReleaseWake: Arc<TestFinalReleaseWake>,
        reentry: Rc<RefCell<Option<ProviderReentry>>>,
    }

    impl RecordingProvider {
        fn new(
            events: Rc<RefCell<Vec<ProviderEvent>>>,
            names: impl IntoIterator<Item = GLuint>,
        ) -> Self {
            Self {
                events,
                names: names.into_iter().collect(),
                integers: HashMap::new(),
                lifecycleIngress: Rc::new(RefCell::new(None)),
                finalReleaseIngress: Rc::new(RefCell::new(None)),
                finalReleaseWake: Arc::new(TestFinalReleaseWake::default()),
                reentry: Rc::new(RefCell::new(None)),
            }
        }

        fn withInteger(mut self, parameter: GLenum, value: GLint) -> Self {
            self.integers.insert(parameter, value);
            self
        }

        fn nextName(&mut self) -> GLuint {
            self.names
                .pop_front()
                .expect("recording provider has a frozen real-name result")
        }
    }

    impl Drop for RecordingProvider {
        fn drop(&mut self) {
            self.events.borrow_mut().push(ProviderEvent::ProviderDrop);
        }
    }

    impl GLExecutionProvider for RecordingProvider {
        fn installContextLifecycleIngress(&mut self, ingress: GLContextLifecycleIngress) {
            let previous = self.lifecycleIngress.borrow_mut().replace(ingress);
            assert!(previous.is_none(), "provider accepts one lifecycle ingress");
        }

        fn installFinalReleaseIngress(
            &mut self,
            ingress: GLFinalReleaseIngress,
        ) -> Arc<dyn nuxie_ore_metal::gpu_resource::ResourceFinalReleaseWake> {
            let previous = self.finalReleaseIngress.borrow_mut().replace(ingress);
            assert!(
                previous.is_none(),
                "provider accepts one final-release ingress"
            );
            self.finalReleaseWake.clone()
        }

        fn submit(&mut self, command: GLCommand) {
            self.events
                .borrow_mut()
                .push(ProviderEvent::Submit(command));

            let Some(reentry) = self.reentry.borrow_mut().take() else {
                return;
            };
            let pointer = Box::into_raw(Box::new(FinalizerProbe(Rc::clone(&self.events))));
            let finalRelease = unsafe {
                nuxie_ore_metal::gpu_resource::OwnerThreadFinalRelease::new(
                    pointer as usize,
                    releaseFinalizerProbe,
                )
            };
            assert!(
                reentry.finalReleaseRoute.defer(finalRelease).is_ok(),
                "live provider callback enqueues its final release"
            );

            let domain = GLExecutionDomain(
                reentry
                    .domain
                    .upgrade()
                    .expect("provider reentry test retains its execution domain"),
            );
            match reentry.action {
                ProviderReentryAction::Command => {
                    recordGLCommand(GLCommand::ClearStencil(99));
                }
                ProviderReentryAction::ContextLoss => domain.markContextLost(),
                ProviderReentryAction::Shutdown => domain.shutdown(),
            }
        }

        fn generateObject(&mut self, kind: GLObjectKind) -> GLuint {
            let name = self.nextName();
            self.events
                .borrow_mut()
                .push(ProviderEvent::Generate(kind, name));
            name
        }

        fn createProgram(&mut self) -> GLuint {
            let name = self.nextName();
            self.events
                .borrow_mut()
                .push(ProviderEvent::CreateProgram(name));
            name
        }

        fn createShader(&mut self, shaderType: GLenum) -> GLuint {
            let name = self.nextName();
            self.events
                .borrow_mut()
                .push(ProviderEvent::CreateShader(shaderType, name));
            name
        }

        fn getInteger(&mut self, parameter: GLenum) -> GLint {
            let value = self.integers.get(&parameter).copied().unwrap_or_default();
            self.events
                .borrow_mut()
                .push(ProviderEvent::GetInteger(parameter, value));
            value
        }

        fn getString(&mut self, _parameter: GLenum) -> Option<Vec<u8>> {
            None
        }

        fn getExtension(&mut self, _index: GLuint) -> Option<Vec<u8>> {
            None
        }

        fn enableWebGLExtension(&mut self, _name: &str) -> bool {
            false
        }

        fn isObject(&mut self, _kind: GLObjectKind, name: GLuint) -> bool {
            name != 0
        }

        fn checkFramebufferStatus(&mut self, _target: GLenum) -> GLenum {
            GL_FRAMEBUFFER_COMPLETE
        }

        fn shaderParameter(&mut self, _shader: GLuint, _parameter: GLenum) -> GLint {
            GL_TRUE.into()
        }

        fn shaderInfoLog(&mut self, _shader: GLuint, _maxLength: usize) -> Vec<u8> {
            Vec::new()
        }

        fn programParameter(&mut self, _program: GLuint, _parameter: GLenum) -> GLint {
            GL_TRUE.into()
        }

        fn programInfoLog(&mut self, _program: GLuint, _maxLength: usize) -> Vec<u8> {
            Vec::new()
        }

        fn uniformBlockIndex(&mut self, _program: GLuint, _name: &[u8]) -> GLuint {
            0
        }

        fn uniformLocation(&mut self, _program: GLuint, _name: &[u8]) -> GLint {
            0
        }

        fn readPixelsRGBA8(&mut self, _x: i32, _y: i32, width: u32, height: u32) -> Vec<u8> {
            vec![0; width as usize * height as usize * 4]
        }

        fn contextLost(&mut self, nextGeneration: u64) {
            self.events
                .borrow_mut()
                .push(ProviderEvent::ContextLost(nextGeneration));
        }
    }

    #[test]
    fn webgl2_constants_and_version_comparison_match_the_header() {
        assert_eq!(PINNED_SOURCE.lines().count(), 277);
        assert_eq!(GL_MAX_PIXEL_LOCAL_STORAGE_PLANES_ANGLE, 0x96E0);
        assert_eq!(GL_PIXEL_LOCAL_CLEAR_VALUE_UNSIGNED_INT_ANGLE, 0x96ED);
        assert_eq!(GL_BLEND_ADVANCED_COHERENT_KHR, 0x9285);
        assert_eq!(GL_COMPRESSED_RGBA_ASTC_12x12_KHR, 0x93BD);
        assert!(GLCapabilities::IsVersionAtLeast(3, 1, 3, 0));
        assert!(!GLCapabilities::IsVersionAtLeast(3, 0, 3, 1));
        assert_eq!(GLCapabilities::default(), GLCapabilities::default());
    }

    #[test]
    fn production_names_and_queries_are_exact_provider_values_and_barriers() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let provider = RecordingProvider::new(Rc::clone(&events), [101, 7, 900])
            .withInteger(GL_ELEMENT_ARRAY_BUFFER_BINDING, 313);
        let domain = GLExecutionDomain::new(Box::new(provider));

        let (buffer, binding, program, shader) = domain.withCurrent(|| {
            recordGLCommand(GLCommand::Clear(GL_COLOR_BUFFER_BIT));
            let buffer = generateGLObject(GLObjectKind::Buffer);
            let binding = domain.getInteger(GL_ELEMENT_ARRAY_BUFFER_BINDING);
            let program = createGLProgram();
            let shader = createGLShader(GL_VERTEX_SHADER);
            recordGLCommand(GLCommand::Flush);
            (buffer, binding, program, shader)
        });

        assert_eq!((buffer, binding, program, shader), (101, 313, 7, 900));
        assert_eq!(
            *events.borrow(),
            vec![
                ProviderEvent::Submit(GLCommand::Clear(GL_COLOR_BUFFER_BIT)),
                ProviderEvent::Generate(GLObjectKind::Buffer, 101),
                ProviderEvent::GetInteger(GL_ELEMENT_ARRAY_BUFFER_BINDING, 313),
                ProviderEvent::CreateProgram(7),
                ProviderEvent::CreateShader(GL_VERTEX_SHADER, 900),
                ProviderEvent::Submit(GLCommand::Flush),
            ]
        );
    }

    #[test]
    fn nested_domains_restore_outer_provider_and_unwind_after_panic() {
        let outerEvents = Rc::new(RefCell::new(Vec::new()));
        let innerEvents = Rc::new(RefCell::new(Vec::new()));
        let outer = GLExecutionDomain::new(Box::new(RecordingProvider::new(
            Rc::clone(&outerEvents),
            [],
        )));
        let inner = GLExecutionDomain::new(Box::new(RecordingProvider::new(
            Rc::clone(&innerEvents),
            [],
        )));

        outer.withCurrent(|| {
            recordGLCommand(GLCommand::ClearStencil(1));
            inner.withCurrent(|| recordGLCommand(GLCommand::ClearStencil(2)));
            recordGLCommand(GLCommand::ClearStencil(3));
        });
        assert_eq!(
            *outerEvents.borrow(),
            vec![
                ProviderEvent::Submit(GLCommand::ClearStencil(1)),
                ProviderEvent::Submit(GLCommand::ClearStencil(3)),
            ]
        );
        assert_eq!(
            *innerEvents.borrow(),
            vec![ProviderEvent::Submit(GLCommand::ClearStencil(2))]
        );

        assert!(catch_unwind(AssertUnwindSafe(|| outer.withCurrent(|| panic!("probe")))).is_err());
        assert!(currentGLExecutionDomain().is_none());
    }

    #[test]
    fn nested_same_domain_calls_defer_finalizers_until_the_outer_safe_point() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let borrowed = Rc::new(RefCell::new(0));
        let domain =
            GLExecutionDomain::new(Box::new(RecordingProvider::new(Rc::clone(&events), [])));
        let route = domain.ownerThreadFinalReleaseRoute();

        domain.withCurrent(|| {
            let borrowedGuard = borrowed.borrow_mut();
            let pointer = Box::into_raw(Box::new(BorrowFinalizerProbe {
                borrowed: Rc::clone(&borrowed),
                events: Rc::clone(&events),
            }));
            let release = unsafe {
                nuxie_ore_metal::gpu_resource::OwnerThreadFinalRelease::new(
                    pointer as usize,
                    releaseBorrowFinalizerProbe,
                )
            };
            assert!(route.defer(release).is_ok());

            domain.withCurrent(|| recordGLCommand(GLCommand::ClearStencil(17)));
            assert_eq!(
                *events.borrow(),
                vec![ProviderEvent::Submit(GLCommand::ClearStencil(17))],
                "neither a nested same-domain scope nor provider access is a drain point"
            );
            assert_eq!(*borrowedGuard, 0);
            drop(borrowedGuard);
        });

        assert_eq!(*borrowed.borrow(), 1);
        assert_eq!(
            *events.borrow(),
            vec![
                ProviderEvent::Submit(GLCommand::ClearStencil(17)),
                ProviderEvent::FinalRelease,
            ]
        );
    }

    #[test]
    fn final_release_drain_is_exclusive_and_double_install_preserves_original() {
        let domainA = GLExecutionDomain::new(Box::new(RecordingProvider::new(
            Rc::new(RefCell::new(Vec::new())),
            [],
        )));
        let originalDrain = domainA.resourceFinalReleaseDrain();

        assert!(catch_unwind(AssertUnwindSafe(|| {
            domainA.installFinalReleaseDrain(originalDrain)
        }))
        .is_err());
        assert!(domainA
            .ownerThreadFinalReleaseRoute()
            .defer(unsafe {
                nuxie_ore_metal::gpu_resource::OwnerThreadFinalRelease::new(0, |_| {})
            })
            .is_ok());
        domainA.withCurrent(|| {});
    }

    #[test]
    fn shutdown_drains_owner_finalizers_before_provider_drop_and_closes_route() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let domain =
            GLExecutionDomain::new(Box::new(RecordingProvider::new(Rc::clone(&events), [])));
        let route = domain.ownerThreadFinalReleaseRoute();
        let pointer = Box::into_raw(Box::new(FinalizerProbe(Rc::clone(&events))));
        let release = unsafe {
            nuxie_ore_metal::gpu_resource::OwnerThreadFinalRelease::new(
                pointer as usize,
                releaseFinalizerProbe,
            )
        };
        let enqueueResult = std::thread::spawn({
            let route = route.clone();
            move || route.defer(release)
        })
        .join()
        .expect("worker enqueue");
        assert!(enqueueResult.is_ok(), "live route accepts finalizer");

        domain.shutdown();
        assert_eq!(*events.borrow(), vec![ProviderEvent::FinalRelease]);

        let late =
            unsafe { nuxie_ore_metal::gpu_resource::OwnerThreadFinalRelease::new(0, |_| {}) };
        assert!(route.defer(late).is_err());
        drop(domain);
        assert_eq!(
            *events.borrow(),
            vec![ProviderEvent::FinalRelease, ProviderEvent::ProviderDrop]
        );
    }

    #[test]
    fn lifecycle_loss_is_terminal_and_lost_shutdown_suppresses_stale_deletes() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let provider = RecordingProvider::new(Rc::clone(&events), []);
        let lifecycleIngress = Rc::clone(&provider.lifecycleIngress);
        let domain = GLExecutionDomain::new(Box::new(provider));
        let route = domain.ownerThreadFinalReleaseRoute();
        let stamp = domain.stamp();
        let ingress = lifecycleIngress
            .borrow()
            .clone()
            .expect("provider retains the required lifecycle ingress");

        assert!(ingress.contextLost());
        assert!(!domain.isLive());
        assert_eq!(domain.generation(), 2);
        assert_eq!(*events.borrow(), vec![ProviderEvent::ContextLost(2)]);
        assert_eq!(
            ingress.contextRestored(),
            Some(GLContextRecovery::RecreateRenderer)
        );
        assert!(
            !domain.isLive(),
            "restoration never revives the old renderer"
        );
        assert_eq!(domain.generation(), 2);

        let pointer = Box::into_raw(Box::new(StaleDeleteFinalizerProbe {
            events: Rc::clone(&events),
            stamp,
            name: 404,
        }));
        let finalRelease = unsafe {
            nuxie_ore_metal::gpu_resource::OwnerThreadFinalRelease::new(
                pointer as usize,
                releaseStaleDeleteFinalizerProbe,
            )
        };
        assert!(
            route.defer(finalRelease).is_ok(),
            "lost domains retain ownership until terminal shutdown"
        );

        domain.shutdown();
        assert_eq!(
            *events.borrow(),
            vec![ProviderEvent::ContextLost(2), ProviderEvent::FinalRelease]
        );
        drop(domain);
        assert_eq!(
            *events.borrow(),
            vec![
                ProviderEvent::ContextLost(2),
                ProviderEvent::FinalRelease,
                ProviderEvent::ProviderDrop,
            ]
        );
    }

    #[test]
    fn retired_renderer_retains_valid_deletion_authority_for_late_releases() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let provider = RecordingProvider::new(Rc::clone(&events), []);
        let finalReleaseIngress = Rc::clone(&provider.finalReleaseIngress);
        let finalReleaseWake = Arc::clone(&provider.finalReleaseWake);
        let domain = GLExecutionDomain::new(Box::new(provider));
        let route = domain.ownerThreadFinalReleaseRoute();
        let ownerStamp = domain.stamp();
        let workerStamp = domain.stamp();

        domain.retireRenderer();
        assert!(domain.isRendererRetired());
        assert!(
            domain.isLive(),
            "normal retirement does not manufacture context loss"
        );
        assert!(catch_unwind(AssertUnwindSafe(|| domain.stamp())).is_err());

        let ownerPointer = Box::into_raw(Box::new(StaleDeleteFinalizerProbe {
            events: Rc::clone(&events),
            stamp: ownerStamp,
            name: 701,
        }));
        let ownerRelease = unsafe {
            nuxie_ore_metal::gpu_resource::OwnerThreadFinalRelease::new(
                ownerPointer as usize,
                releaseStaleDeleteFinalizerProbe,
            )
        };
        assert!(route.defer(ownerRelease).is_ok());

        let workerPointer = Box::into_raw(Box::new(StaleDeleteFinalizerProbe {
            events: Rc::clone(&events),
            stamp: workerStamp,
            name: 702,
        }));
        let workerRelease = unsafe {
            nuxie_ore_metal::gpu_resource::OwnerThreadFinalRelease::new(
                workerPointer as usize,
                releaseStaleDeleteFinalizerProbe,
            )
        };
        let workerResult = std::thread::spawn({
            let route = route.clone();
            move || route.defer(workerRelease)
        })
        .join()
        .expect("worker enqueue");
        assert!(workerResult.is_ok());

        assert_eq!(finalReleaseWake.takePosts(), 2);
        assert!(
            events.borrow().is_empty(),
            "wake posting is asynchronous and never drains inline"
        );
        let ingress = finalReleaseIngress
            .borrow()
            .clone()
            .expect("provider retains the required final-release ingress");

        drop(domain);
        assert!(
            events.borrow().is_empty(),
            "queued stamped owners retain the provider until the posted task"
        );
        assert!(ingress.drainFinalReleases());
        assert_eq!(
            *events.borrow(),
            vec![
                ProviderEvent::Submit(GLCommand::DeleteBuffer(701)),
                ProviderEvent::FinalRelease,
                ProviderEvent::Submit(GLCommand::DeleteBuffer(702)),
                ProviderEvent::FinalRelease,
                ProviderEvent::ProviderDrop,
            ],
            "retired destruction preserves source RAII while the retained generation is valid"
        );
        assert!(!ingress.drainFinalReleases());
        assert!(route
            .defer(unsafe {
                nuxie_ore_metal::gpu_resource::OwnerThreadFinalRelease::new(0, |_| {})
            })
            .is_err());
    }

    #[test]
    fn context_loss_then_retirement_uses_the_same_posted_stale_destruction_path() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let provider = RecordingProvider::new(Rc::clone(&events), []);
        let lifecycleIngress = Rc::clone(&provider.lifecycleIngress);
        let finalReleaseIngress = Rc::clone(&provider.finalReleaseIngress);
        let finalReleaseWake = Arc::clone(&provider.finalReleaseWake);
        let domain = GLExecutionDomain::new(Box::new(provider));
        let route = domain.ownerThreadFinalReleaseRoute();
        let stamp = domain.stamp();

        let lifecycle = lifecycleIngress
            .borrow()
            .clone()
            .expect("provider retains the required lifecycle ingress");
        assert!(lifecycle.contextLost());
        domain.retireRenderer();
        assert_eq!(
            lifecycle.contextRestored(),
            None,
            "a retired root ignores its stale browser restoration listener"
        );

        let pointer = Box::into_raw(Box::new(StaleDeleteFinalizerProbe {
            events: Rc::clone(&events),
            stamp,
            name: 703,
        }));
        let release = unsafe {
            nuxie_ore_metal::gpu_resource::OwnerThreadFinalRelease::new(
                pointer as usize,
                releaseStaleDeleteFinalizerProbe,
            )
        };
        assert!(route.defer(release).is_ok());
        assert_eq!(finalReleaseWake.takePosts(), 1);
        let ingress = finalReleaseIngress
            .borrow()
            .clone()
            .expect("provider retains the required final-release ingress");

        drop(domain);
        assert_eq!(*events.borrow(), vec![ProviderEvent::ContextLost(2)]);
        assert!(ingress.drainFinalReleases());
        assert_eq!(
            *events.borrow(),
            vec![
                ProviderEvent::ContextLost(2),
                ProviderEvent::FinalRelease,
                ProviderEvent::ProviderDrop,
            ]
        );
    }

    #[test]
    fn retired_renderer_then_context_loss_invalidates_remaining_stamped_deletes() {
        let events = Rc::new(RefCell::new(Vec::new()));
        let provider = RecordingProvider::new(Rc::clone(&events), []);
        let lifecycleIngress = Rc::clone(&provider.lifecycleIngress);
        let finalReleaseIngress = Rc::clone(&provider.finalReleaseIngress);
        let finalReleaseWake = Arc::clone(&provider.finalReleaseWake);
        let domain = GLExecutionDomain::new(Box::new(provider));
        let route = domain.ownerThreadFinalReleaseRoute();
        let stamp = domain.stamp();
        let lifecycle = lifecycleIngress
            .borrow()
            .clone()
            .expect("provider retains the required lifecycle ingress");

        domain.retireRenderer();
        assert!(domain.isRendererRetired());
        assert!(domain.isLive());
        assert_eq!(domain.generation(), 1);

        assert!(lifecycle.contextLost());
        assert!(!domain.isLive());
        assert_eq!(domain.generation(), 2);
        assert_eq!(
            lifecycle.contextRestored(),
            None,
            "a retired root never requests a replacement renderer"
        );

        let pointer = Box::into_raw(Box::new(StaleDeleteFinalizerProbe {
            events: Rc::clone(&events),
            stamp,
            name: 704,
        }));
        let release = unsafe {
            nuxie_ore_metal::gpu_resource::OwnerThreadFinalRelease::new(
                pointer as usize,
                releaseStaleDeleteFinalizerProbe,
            )
        };
        assert!(route.defer(release).is_ok());
        assert_eq!(finalReleaseWake.takePosts(), 1);
        let ingress = finalReleaseIngress
            .borrow()
            .clone()
            .expect("provider retains the required final-release ingress");

        drop(domain);
        assert_eq!(*events.borrow(), vec![ProviderEvent::ContextLost(2)]);
        assert!(ingress.drainFinalReleases());
        assert_eq!(
            *events.borrow(),
            vec![
                ProviderEvent::ContextLost(2),
                ProviderEvent::FinalRelease,
                ProviderEvent::ProviderDrop,
            ],
            "actual context loss suppresses the stale name while releasing ownership"
        );
    }

    #[test]
    fn provider_callback_reentry_never_drains_or_mutates_the_domain() {
        for action in [
            ProviderReentryAction::Command,
            ProviderReentryAction::ContextLoss,
            ProviderReentryAction::Shutdown,
        ] {
            let events = Rc::new(RefCell::new(Vec::new()));
            let provider = RecordingProvider::new(Rc::clone(&events), []);
            let reentry = Rc::clone(&provider.reentry);
            let domain = GLExecutionDomain::new(Box::new(provider));
            reentry.borrow_mut().replace(ProviderReentry {
                action,
                domain: Rc::downgrade(&domain.0),
                finalReleaseRoute: domain.ownerThreadFinalReleaseRoute(),
            });

            let panic = catch_unwind(AssertUnwindSafe(|| {
                domain.withCurrent(|| recordGLCommand(GLCommand::ClearStencil(7)))
            }))
            .expect_err("provider callback reentry is rejected");
            let panicMessage = panic
                .downcast_ref::<&str>()
                .copied()
                .map(str::to_owned)
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .expect("reentry panic carries its invariant message");
            assert!(
                panicMessage.contains("GLExecutionProvider callback reentry is unsupported"),
                "unexpected reentry panic for {action:?}: {panicMessage}"
            );
            assert!(domain.isLive(), "{action:?} cannot mark the domain lost");
            assert_eq!(
                domain.generation(),
                1,
                "{action:?} cannot advance generation"
            );
            assert_eq!(
                *events.borrow(),
                vec![ProviderEvent::Submit(GLCommand::ClearStencil(7))],
                "{action:?} cannot drain the callback's queued finalizer"
            );

            domain.withCurrent(|| {});
            assert_eq!(
                *events.borrow(),
                vec![
                    ProviderEvent::Submit(GLCommand::ClearStencil(7)),
                    ProviderEvent::FinalRelease,
                ],
                "a later authorized scope drains {action:?}'s queued finalizer"
            );
            domain.shutdown();
            drop(domain);
            assert_eq!(
                events.borrow().last(),
                Some(&ProviderEvent::ProviderDrop),
                "the provider remains alive through {action:?}'s authorized drain"
            );
        }
    }

    #[test]
    fn bound_final_release_authority_rejects_raw_and_foreign_drains() {
        use nuxie_ore_metal::gpu_resource::ResourceFinalReleaseDrainError;

        let eventsA = Rc::new(RefCell::new(Vec::new()));
        let eventsB = Rc::new(RefCell::new(Vec::new()));
        let domainA =
            GLExecutionDomain::new(Box::new(RecordingProvider::new(Rc::clone(&eventsA), [])));
        let domainB =
            GLExecutionDomain::new(Box::new(RecordingProvider::new(Rc::clone(&eventsB), [])));
        let rawDrainA = domainA.resourceFinalReleaseDrain();
        let pointer = Box::into_raw(Box::new(FinalizerProbe(Rc::clone(&eventsA))));
        let finalRelease = unsafe {
            nuxie_ore_metal::gpu_resource::OwnerThreadFinalRelease::new(
                pointer as usize,
                releaseFinalizerProbe,
            )
        };
        assert!(domainA
            .ownerThreadFinalReleaseRoute()
            .defer(finalRelease)
            .is_ok());

        let rawError = ResourceFinalReleaseDrainError::WrongExecutionDomain {
            expected_domain: domainA.key(),
            actual_domain: 0,
        };
        assert_eq!(rawDrainA.close(), Err(rawError));
        assert_eq!(rawDrainA.drain(), Err(rawError));
        {
            let executionDomainB = domainB.0.finalReleaseExecutionDomain.borrow();
            assert_eq!(
                rawDrainA.drain_in_execution_domain(
                    executionDomainB
                        .as_ref()
                        .expect("domain B retains its bound authority"),
                ),
                Err(ResourceFinalReleaseDrainError::WrongExecutionDomain {
                    expected_domain: domainA.key(),
                    actual_domain: domainB.key(),
                })
            );
        }
        assert!(
            eventsA.borrow().is_empty(),
            "rejected raw and foreign drains preserve A's FIFO"
        );

        domainA.withCurrent(|| {});
        assert_eq!(*eventsA.borrow(), vec![ProviderEvent::FinalRelease]);
        domainA.shutdown();
        domainB.shutdown();
    }
}
