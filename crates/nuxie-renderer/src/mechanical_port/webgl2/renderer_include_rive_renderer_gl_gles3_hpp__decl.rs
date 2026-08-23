//! Complete WebGL2-configuration declaration translation of
//! `renderer/include/rive/renderer/gl/gles3.hpp`.

#![allow(non_snake_case, non_upper_case_globals)]

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_gl_gles3.hpp");

pub(crate) type GLenum = u32;
pub(crate) type GLuint = u32;
pub(crate) type GLint = i32;
pub(crate) type GLsizei = i32;
pub(crate) type GLfloat = f32;
pub(crate) type GLbitfield = u32;

pub(crate) const GL_NONE: GLenum = 0;
pub(crate) const GL_FALSE: GLenum = 0;
pub(crate) const GL_ZERO: GLenum = 0;
pub(crate) const GL_ONE: GLenum = 1;
pub(crate) const GL_FRONT: GLenum = 0x0404;
pub(crate) const GL_BACK: GLenum = 0x0405;
pub(crate) const GL_CW: GLenum = 0x0900;
pub(crate) const GL_LESS: GLenum = 0x0201;
pub(crate) const GL_EQUAL: GLenum = 0x0202;
pub(crate) const GL_LEQUAL: GLenum = 0x0203;
pub(crate) const GL_NOTEQUAL: GLenum = 0x0205;
pub(crate) const GL_ALWAYS: GLenum = 0x0207;
pub(crate) const GL_KEEP: GLenum = 0x1E00;
pub(crate) const GL_REPLACE: GLenum = 0x1E01;
pub(crate) const GL_INCR_WRAP: GLenum = 0x8507;
pub(crate) const GL_DECR: GLenum = 0x1E03;
pub(crate) const GL_DECR_WRAP: GLenum = 0x8508;
pub(crate) const GL_ARRAY_BUFFER: GLenum = 0x8892;
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
pub(crate) const GL_ONE_MINUS_SRC_ALPHA: GLenum = 0x0303;
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
pub(crate) const GL_LINEAR_MIPMAP_NEAREST: GLenum = 0x2701;
pub(crate) const GL_COLOR_BUFFER_BIT: GLbitfield = 0x00004000;
pub(crate) const GL_UNPACK_IMAGE_HEIGHT: GLenum = 0x806E;
pub(crate) const GL_R8: GLenum = 0x8229;
pub(crate) const GL_RG8: GLenum = 0x822B;
pub(crate) const GL_RGBA8: GLenum = 0x8058;
pub(crate) const GL_RGBA8_SNORM: GLenum = 0x8F97;
pub(crate) const GL_RGBA16F: GLenum = 0x881A;
pub(crate) const GL_RG16F: GLenum = 0x822F;
pub(crate) const GL_R16F: GLenum = 0x822D;
pub(crate) const GL_RGBA32F: GLenum = 0x8814;
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
pub(crate) const GL_RGB: GLenum = 0x1907;
pub(crate) const GL_RGBA: GLenum = 0x1908;
pub(crate) const GL_DEPTH_COMPONENT: GLenum = 0x1902;
pub(crate) const GL_DEPTH_STENCIL: GLenum = 0x84F9;
pub(crate) const GL_UNSIGNED_BYTE: GLenum = 0x1401;
pub(crate) const GL_BYTE: GLenum = 0x1400;
pub(crate) const GL_HALF_FLOAT: GLenum = 0x140B;
pub(crate) const GL_FLOAT: GLenum = 0x1406;
pub(crate) const GL_UNSIGNED_INT_2_10_10_10_REV: GLenum = 0x8368;
pub(crate) const GL_UNSIGNED_INT_10F_11F_11F_REV: GLenum = 0x8C3B;
pub(crate) const GL_UNSIGNED_SHORT: GLenum = 0x1403;
pub(crate) const GL_UNSIGNED_INT_24_8: GLenum = 0x84FA;
pub(crate) const GL_FLOAT_32_UNSIGNED_INT_24_8_REV: GLenum = 0x8DAD;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GLCommand {
    FrontFace(GLenum),
    DepthRange(f32, f32),
    DepthFunc(GLenum),
    ClearDepth(f32),
    ClearStencil(i32),
    Enable(GLenum),
    Disable(GLenum),
    PixelStore(GLenum, i32),
    BindBuffer(GLenum, GLuint),
    ProvokingVertex(GLenum),
    Scissor(u32, u32, u32, u32),
    CullFace(GLenum),
    BlendEquation(GLenum),
    BlendFunc(GLenum, GLenum),
    ColorMask(bool, bool, bool, bool),
    DepthMask(bool),
    StencilMask(GLuint),
    StencilFunc(GLenum, i32, GLuint),
    StencilOp(GLenum, GLenum, GLenum),
    StencilFuncSeparate(GLenum, GLenum, i32, GLuint),
    StencilOpSeparate(GLenum, GLenum, GLenum, GLenum),
    UseProgram(GLuint),
    BindVertexArray(GLuint),
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
    GenerateVertexArray(GLuint),
    CreateProgram(GLuint),
    CreateShader(GLenum, GLuint),
    ShaderSource(GLuint, String),
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
}

#[derive(Debug)]
struct GLCommandStream {
    nextName: GLuint,
    nextQuerySlot: u64,
    commands: Vec<GLCommand>,
}

impl Default for GLCommandStream {
    fn default() -> Self {
        Self {
            nextName: 1,
            nextQuerySlot: 1,
            commands: Vec::new(),
        }
    }
}

pub(crate) fn allocateGLQuerySlot() -> u64 {
    GL_COMMAND_STREAM.with(|stream| {
        let mut stream = stream.borrow_mut();
        let slot = stream.nextQuerySlot;
        stream.nextQuerySlot = stream
            .nextQuerySlot
            .checked_add(1)
            .expect("GL query slot overflow");
        slot
    })
}

thread_local! {
    static GL_COMMAND_STREAM: std::cell::RefCell<GLCommandStream> =
        std::cell::RefCell::new(GLCommandStream::default());
}

pub(crate) fn recordGLCommand(command: GLCommand) {
    GL_COMMAND_STREAM.with(|stream| stream.borrow_mut().commands.push(command));
}

pub(crate) fn allocateGLName() -> GLuint {
    GL_COMMAND_STREAM.with(|stream| {
        let mut stream = stream.borrow_mut();
        let name = stream.nextName;
        stream.nextName = stream.nextName.checked_add(1).expect("GL name overflow");
        name
    })
}

pub(crate) fn takeGLCommands() -> Vec<GLCommand> {
    GL_COMMAND_STREAM.with(|stream| std::mem::take(&mut stream.borrow_mut().commands))
}

#[cfg(test)]
pub(crate) fn resetGLCommandStream() {
    GL_COMMAND_STREAM.with(|stream| *stream.borrow_mut() = GLCommandStream::default());
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
        Self::IsVersionAtLeast(self.contextVersionMajor, self.contextVersionMinor, major, minor)
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
}
