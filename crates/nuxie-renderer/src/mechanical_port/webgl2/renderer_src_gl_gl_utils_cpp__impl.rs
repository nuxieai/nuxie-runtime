//! Complete mechanical implementation translation of
//! `renderer/src/gl/gl_utils.cpp` for the frozen WebGL2 configuration.

#![allow(non_snake_case)]

use super::gl_utils_decl::{
    Buffer, DebugPrintErrorAndAbort, Framebuffer, GLObject, GLObjectType,
    GLUtilsSourceConfiguration, Program, Renderbuffer, Shader, Texture, VAO,
};
use super::gles3_decl::*;
use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::{
    ImageFilter, ImageSampler, ImageWrap,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::IAABB;

pub(crate) const PINNED_SOURCE: &str = include_str!("source/renderer_src_gl_gl_utils.cpp");

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex, OnceLock,
    },
};
static ABANDONED_COUNT: AtomicU32 = AtomicU32::new(0);
static RECLAIMED_COUNT: AtomicU32 = AtomicU32::new(0);
fn abandoned_names() -> &'static Mutex<HashMap<(u64, u64), Vec<(GLObjectType, GLuint)>>> {
    static NAMES: OnceLock<Mutex<HashMap<(u64, u64), Vec<(GLObjectType, GLuint)>>>> =
        OnceLock::new();
    NAMES.get_or_init(|| Mutex::new(HashMap::new()))
}
pub(crate) fn delete_name(kind: GLObjectType, id: GLuint) {
    recordGLCommand(match kind {
        GLObjectType::buffer => GLCommand::DeleteBuffer(id),
        GLObjectType::texture => GLCommand::DeleteTexture(id),
        GLObjectType::framebuffer => GLCommand::DeleteFramebuffer(id),
        GLObjectType::renderbuffer => GLCommand::DeleteRenderbuffer(id),
        GLObjectType::vertexArray => GLCommand::DeleteVertexArray(id),
        GLObjectType::shader => GLCommand::DeleteShader(id),
        GLObjectType::program => GLCommand::DeleteProgram(id),
    });
}
pub(crate) fn abandon_name(kind: GLObjectType, id: GLuint, owner: (u64, u64)) {
    abandoned_names()
        .lock()
        .unwrap()
        .entry(owner)
        .or_default()
        .push((kind, id));
    ABANDONED_COUNT.fetch_add(1, Ordering::Relaxed);
}
pub(crate) fn ReclaimAbandonedNames() {
    if ABANDONED_COUNT.load(Ordering::Acquire) == RECLAIMED_COUNT.load(Ordering::Relaxed) {
        return;
    }
    let Some(owner) = currentGLExecutionIdentity() else {
        return;
    };
    let Some(mine) = abandoned_names().lock().unwrap().remove(&owner) else {
        return;
    };
    for &(kind, id) in &mine {
        delete_name(kind, id);
    }
    RECLAIMED_COUNT.fetch_add(mine.len() as u32, Ordering::Release);
}
pub(crate) fn AbandonedNameCount() -> u32 {
    ABANDONED_COUNT.load(Ordering::Relaxed)
}
pub(crate) fn ReclaimedNameCount() -> u32 {
    RECLAIMED_COUNT.load(Ordering::Relaxed)
}

const GLSL_GLSL_VERSION: &str = "NC";
const GLSL_VERTEX: &str = "DB";
const GLSL_FRAGMENT: &str = "GB";
const GLSL_BASE_INSTANCE_UNIFORM_NAME: &str = "BE";
const GLSL_TESS_TEXTURE_FLOATING_POINT: &str = "IF";
const GLSL_GL_RENDERER_MALI: &str = "JF";
const GLSL_GLSL: &str = include_str!("source/generated_glsl_embedded/glsl.minified.glsl");

fn generatedObject(kind: GLObjectKind) -> GLObject {
    GLObject::fromAdoptedID(generateGLObject(kind))
}

pub(crate) fn newBuffer() -> Buffer {
    Buffer(generatedObject(GLObjectKind::Buffer))
}

pub(crate) fn newTexture() -> Texture {
    Texture(generatedObject(GLObjectKind::Texture))
}

pub(crate) fn newFramebuffer() -> Framebuffer {
    Framebuffer(generatedObject(GLObjectKind::Framebuffer))
}

pub(crate) fn newRenderbuffer() -> Renderbuffer {
    Renderbuffer(generatedObject(GLObjectKind::Renderbuffer))
}

pub(crate) fn newVAO() -> VAO {
    VAO(generatedObject(GLObjectKind::VertexArray))
}

pub(crate) fn newProgram() -> Program {
    let id = createGLProgram();
    Program {
        m_object: GLObject::fromAdoptedID(id),
        m_vertexShader: Shader::default(),
        m_fragmentShader: Shader::default(),
    }
}

pub(crate) fn resetTexture(texture: &mut Texture, adoptedID: GLuint) {
    texture.0.adoptName(GLObjectType::texture, adoptedID);
}

pub(crate) fn moveAssignTexture(texture: &mut Texture, mut rhs: Texture) {
    texture.0.adopt(GLObjectType::texture, &mut rhs.0);
}

pub(crate) fn resetFramebuffer(framebuffer: &mut Framebuffer, adoptedID: GLuint) {
    framebuffer
        .0
        .adoptName(GLObjectType::framebuffer, adoptedID);
}

pub(crate) fn moveAssignFramebuffer(framebuffer: &mut Framebuffer, mut rhs: Framebuffer) {
    framebuffer.0.adopt(GLObjectType::framebuffer, &mut rhs.0);
}

pub(crate) fn resetRenderbuffer(renderbuffer: &mut Renderbuffer, adoptedID: GLuint) {
    renderbuffer
        .0
        .adoptName(GLObjectType::renderbuffer, adoptedID);
}

pub(crate) fn moveAssignRenderbuffer(renderbuffer: &mut Renderbuffer, mut rhs: Renderbuffer) {
    renderbuffer.0.adopt(GLObjectType::renderbuffer, &mut rhs.0);
}

pub(crate) fn resetShader(shader: &mut Shader, adoptedID: GLuint) {
    shader.0.adoptName(GLObjectType::shader, adoptedID);
}

pub(crate) fn resetProgram(program: &mut Program, adoptedProgramID: GLuint) {
    program
        .m_object
        .adoptName(GLObjectType::program, adoptedProgramID);
}

pub(crate) fn moveAssignProgram(program: &mut Program, mut rhs: Program) {
    program
        .m_object
        .adopt(GLObjectType::program, &mut rhs.m_object);
    program
        .m_vertexShader
        .0
        .adopt(GLObjectType::shader, &mut rhs.m_vertexShader.0);
    program
        .m_fragmentShader
        .0
        .adopt(GLObjectType::shader, &mut rhs.m_fragmentShader.0);
}

impl Drop for Buffer {
    fn drop(&mut self) {
        self.0.destroy(GLObjectType::buffer);
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.reset(0);
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        self.reset(0);
    }
}

impl Drop for Renderbuffer {
    fn drop(&mut self) {
        self.reset(0);
    }
}

impl Drop for VAO {
    fn drop(&mut self) {
        self.0.destroy(GLObjectType::vertexArray);
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        self.reset(0);
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        self.m_object.destroy(GLObjectType::program);
        // C++ members then destruct in reverse declaration order.
        self.m_fragmentShader.reset(0);
        self.m_vertexShader.reset(0);
    }
}

pub(crate) fn CompileAndAttachShader(
    program: GLuint,
    shaderType: GLenum,
    source: &str,
    capabilities: &GLCapabilities,
    debugPrintErrorAndAbort: DebugPrintErrorAndAbort,
) {
    CompileAndAttachShaderParts(
        program,
        shaderType,
        &[],
        &[source],
        capabilities,
        debugPrintErrorAndAbort,
    )
}

pub(crate) fn CompileAndAttachShaderParts(
    program: GLuint,
    shaderType: GLenum,
    defines: &[&str],
    inputSources: &[&str],
    capabilities: &GLCapabilities,
    debugPrintErrorAndAbort: DebugPrintErrorAndAbort,
) {
    let shader = CompileShaderParts(
        shaderType,
        defines,
        inputSources,
        capabilities,
        debugPrintErrorAndAbort,
    );
    recordGLCommand(GLCommand::AttachShader(program, shader));
    recordGLCommand(GLCommand::DeleteShader(shader));
}

pub(crate) fn CompileShader(
    shaderType: GLenum,
    source: &str,
    capabilities: &GLCapabilities,
    debugPrintErrorAndAbort: DebugPrintErrorAndAbort,
) -> GLuint {
    CompileShaderParts(
        shaderType,
        &[],
        &[source],
        capabilities,
        debugPrintErrorAndAbort,
    )
}

pub(crate) fn CompileShaderParts(
    shaderType: GLenum,
    defines: &[&str],
    inputSources: &[&str],
    capabilities: &GLCapabilities,
    debugPrintErrorAndAbort: DebugPrintErrorAndAbort,
) -> GLuint {
    let mut shaderSource = format!(
        "#version {}{}0{}\n#define {GLSL_GLSL_VERSION} {}{}0\n",
        capabilities.contextVersionMajor,
        capabilities.contextVersionMinor,
        if capabilities.isGLES() { " es" } else { "" },
        capabilities.contextVersionMajor,
        capabilities.contextVersionMinor,
    );

    if shaderType == GL_VERTEX_SHADER {
        shaderSource.push_str(&format!("#define {GLSL_VERTEX}\n"));
    } else {
        // Preserve the authoritative source condition exactly. The C++ says
        // `else if (GL_FRAGMENT_SHADER)`, whose nonzero enum value makes this
        // branch true for every non-vertex shader type.
        shaderSource.push_str(&format!("#define {GLSL_FRAGMENT}\n"));
    }
    for define in defines {
        shaderSource.push_str(&format!("#define {define} true\n"));
    }
    if !capabilities.ANGLE_base_vertex_base_instance_shader_builtin() {
        shaderSource.push_str(&format!(
            "#define {GLSL_BASE_INSTANCE_UNIFORM_NAME} {}\n",
            super::gl_utils_decl::BASE_INSTANCE_UNIFORM_NAME
        ));
    }
    if capabilities.needsFloatingPointTessellationTexture {
        shaderSource.push_str(&format!("#define {GLSL_TESS_TEXTURE_FLOATING_POINT}\n"));
    }
    if capabilities.isMali() {
        shaderSource.push_str(&format!("#define {GLSL_GL_RENDERER_MALI}\n"));
    }
    shaderSource.push_str(GLSL_GLSL);
    shaderSource.push('\n');
    for source in inputSources {
        shaderSource.push_str(source);
        shaderSource.push('\n');
    }

    CompileRawGLSL(shaderType, &shaderSource, debugPrintErrorAndAbort)
}

pub(crate) fn CompileRawGLSL(
    shaderType: GLenum,
    rawGLSL: &str,
    debugPrintErrorAndAbort: DebugPrintErrorAndAbort,
) -> GLuint {
    CompileRawGLSLWithConfiguration(
        shaderType,
        rawGLSL,
        debugPrintErrorAndAbort,
        GLUtilsSourceConfiguration::FROZEN_WEBGL2,
    )
}

pub(crate) fn CompileRawGLSLWithConfiguration(
    shaderType: GLenum,
    rawGLSL: &str,
    debugPrintErrorAndAbort: DebugPrintErrorAndAbort,
    configuration: GLUtilsSourceConfiguration,
) -> GLuint {
    let shader = createGLShader(shaderType);
    if configuration.bypassEmscriptenShaderParser {
        let minimalSource = if shaderType == GL_VERTEX_SHADER {
            "#version 300 es\nvoid main() { gl_Position = vec4(0); }"
        } else {
            "#version 300 es\nvoid main() {}"
        };
        recordGLCommand(GLCommand::ShaderSourceBypassingEmscripten {
            shader,
            minimal_source: minimalSource.to_owned(),
            raw_source: rawGLSL.to_owned(),
        });
    } else {
        recordGLCommand(GLCommand::ShaderSource(shader, rawGLSL.to_owned()));
    }
    recordGLCommand(GLCommand::CompileShader(shader));
    if configuration.debug && debugPrintErrorAndAbort == DebugPrintErrorAndAbort::yes {
        recordGLCommand(GLCommand::ValidateShaderCompilationAndAbort {
            shader,
            stderr_flush_delay_ms: 1000,
        });
    }
    shader
}

pub(crate) fn PrintShaderCompilationErrors(shader: GLuint) {
    recordGLCommand(GLCommand::PrintShaderCompilationErrors(shader));
}

pub(crate) fn LinkProgram(program: GLuint, debugPrintErrorAndAbort: DebugPrintErrorAndAbort) {
    LinkProgramWithConfiguration(
        program,
        debugPrintErrorAndAbort,
        GLUtilsSourceConfiguration::FROZEN_WEBGL2,
    )
}

pub(crate) fn LinkProgramWithConfiguration(
    program: GLuint,
    debugPrintErrorAndAbort: DebugPrintErrorAndAbort,
    configuration: GLUtilsSourceConfiguration,
) {
    recordGLCommand(GLCommand::LinkProgram(program));
    if configuration.debug && debugPrintErrorAndAbort == DebugPrintErrorAndAbort::yes {
        recordGLCommand(GLCommand::ValidateProgramLinkAndAbort(program));
    }
}

pub(crate) fn PrintLinkProgramErrors(program: GLuint) {
    recordGLCommand(GLCommand::PrintLinkProgramErrors(program));
}

pub(crate) fn compileOwnedShader(
    shader: &mut Shader,
    shaderType: GLenum,
    defines: &[&str],
    sources: &[&str],
    capabilities: &GLCapabilities,
) {
    let id = CompileShaderParts(
        shaderType,
        defines,
        sources,
        capabilities,
        DebugPrintErrorAndAbort::yes,
    );
    shader.reset(id);
}

pub(crate) fn compileAndAttachOwnedShader(
    program: &mut Program,
    shaderType: GLenum,
    defines: &[&str],
    sources: &[&str],
    capabilities: &GLCapabilities,
) {
    assert!(shaderType == GL_VERTEX_SHADER || shaderType == GL_FRAGMENT_SHADER);
    let internalShader = if shaderType == GL_VERTEX_SHADER {
        &mut program.m_vertexShader
    } else {
        &mut program.m_fragmentShader
    };
    internalShader.compileParts(shaderType, defines, sources, capabilities);
    recordGLCommand(GLCommand::AttachShader(
        program.m_object.id(),
        internalShader.id(),
    ));
}

pub(crate) fn SetTexture2DSamplingParams(minFilter: GLenum, magFilter: GLenum) {
    for (parameter, value) in [
        (GL_TEXTURE_MIN_FILTER, minFilter),
        (GL_TEXTURE_MAG_FILTER, magFilter),
        (GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE),
        (GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE),
    ] {
        recordGLCommand(GLCommand::TextureParameter(
            GL_TEXTURE_2D,
            parameter,
            value as GLint,
        ));
    }
}

fn glWrapFromImageWrap(wrap: ImageWrap) -> GLint {
    if wrap == ImageWrap::clamp {
        GL_CLAMP_TO_EDGE as GLint
    } else if wrap == ImageWrap::repeat {
        GL_REPEAT as GLint
    } else if wrap == ImageWrap::mirror {
        GL_MIRRORED_REPEAT as GLint
    } else {
        panic!("unreachable ImageWrap value {}", wrap.0)
    }
}

fn glMinFilterForImageFilter(filter: ImageFilter) -> GLint {
    if filter == ImageFilter::bilinear {
        GL_LINEAR_MIPMAP_NEAREST as GLint
    } else if filter == ImageFilter::nearest {
        GL_NEAREST as GLint
    } else {
        panic!("unreachable ImageFilter value {}", filter.0)
    }
}

fn glMagFilterForImageFilter(filter: ImageFilter) -> GLint {
    if filter == ImageFilter::nearest {
        GL_NEAREST as GLint
    } else if filter == ImageFilter::bilinear {
        GL_LINEAR as GLint
    } else {
        panic!("unreachable ImageFilter value {}", filter.0)
    }
}

pub(crate) fn SetTexture2DSamplingParamsFromSampler(samplingParams: ImageSampler) {
    for (parameter, value) in [
        (
            GL_TEXTURE_MIN_FILTER,
            glMinFilterForImageFilter(samplingParams.filter),
        ),
        (
            GL_TEXTURE_MAG_FILTER,
            glMagFilterForImageFilter(samplingParams.filter),
        ),
        (GL_TEXTURE_WRAP_S, glWrapFromImageWrap(samplingParams.wrapX)),
        (GL_TEXTURE_WRAP_T, glWrapFromImageWrap(samplingParams.wrapY)),
    ] {
        recordGLCommand(GLCommand::TextureParameter(GL_TEXTURE_2D, parameter, value));
    }
}

pub(crate) fn BlitFramebuffer(bounds: IAABB, renderTargetHeight: u32, mask: GLbitfield) {
    assert!(bounds.left >= 0 && bounds.top >= 0);
    assert!(bounds.right >= bounds.left && bounds.bottom >= bounds.top);
    let l = bounds.left;
    let b = i32::try_from(
        renderTargetHeight
            .checked_sub(bounds.bottom as u32)
            .expect("blit bounds are inside render target"),
    )
    .expect("render target height fits GL coordinate");
    let r = bounds.right;
    let t = i32::try_from(
        renderTargetHeight
            .checked_sub(bounds.top as u32)
            .expect("blit bounds are inside render target"),
    )
    .expect("render target height fits GL coordinate");
    recordGLCommand(GLCommand::BlitFramebuffer(
        [l, b, r, t, l, b, r, t],
        mask,
        GL_NEAREST,
    ));
}

pub(crate) fn Uniform1iByName(programID: GLuint, name: &str, value: GLint) {
    // The browser executor resolves this name and must assert a non--1
    // location before issuing uniform1i, exactly as the source requires.
    assert!(
        !name.is_empty(),
        "uniform name must resolve to a real location"
    );
    recordGLCommand(GLCommand::Uniform1iByName(
        programID,
        name.to_owned(),
        value,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_source_and_generated_input_denominators_are_frozen() {
        assert_eq!(
            super::super::gl_utils_decl::PINNED_SOURCE.lines().count(),
            289
        );
        assert_eq!(PINNED_SOURCE.lines().count(), 501);
        assert_eq!(GLSL_GLSL.as_bytes().len(), 10325);
    }

    #[test]
    fn host_shader_tokens_match_the_current_generated_exports() {
        let exports = include_str!("../webgpu/source/generated_glsl/glsl.glsl.exports.h");
        for (source_name, generated_name) in [
            ("GLSL_VERSION", GLSL_GLSL_VERSION),
            ("VERTEX", GLSL_VERTEX),
            ("FRAGMENT", GLSL_FRAGMENT),
            (
                "BASE_INSTANCE_UNIFORM_NAME",
                GLSL_BASE_INSTANCE_UNIFORM_NAME,
            ),
            (
                "TESS_TEXTURE_FLOATING_POINT",
                GLSL_TESS_TEXTURE_FLOATING_POINT,
            ),
            ("GL_RENDERER_MALI", GLSL_GL_RENDERER_MALI),
        ] {
            assert!(exports.contains(&format!("#define GLSL_{source_name} \"{generated_name}\"")));
        }
    }

    #[test]
    fn shader_assembly_preserves_define_order_and_source_branch() {
        resetGLCommandStream();
        let mut capabilities = GLCapabilities::default();
        capabilities.contextVersionMajor = 3;
        capabilities.contextVersionMinor = 0;
        capabilities.needsFloatingPointTessellationTexture = true;
        capabilities.setIsGLES(true);
        capabilities.setIsMali(true);
        let shader = CompileShaderParts(
            0xFFFF,
            &["CUSTOM"],
            &["void main() {}"],
            &capabilities,
            DebugPrintErrorAndAbort::no,
        );
        let commands = takeGLCommands();
        assert_eq!(commands[0], GLCommand::CreateShader(0xFFFF, shader));
        let GLCommand::ShaderSource(_, source) = &commands[1] else {
            panic!("expected direct shader source command")
        };
        let expectedPrefix = concat!(
            "#version 300 es\n",
            "#define NC 300\n",
            "#define GB\n",
            "#define CUSTOM true\n",
            "#define BE _baseInstance\n",
            "#define IF\n",
            "#define JF\n",
        );
        assert!(source.starts_with(expectedPrefix));
        assert!(source.ends_with("void main() {}\n"));
        assert_eq!(commands[2], GLCommand::CompileShader(shader));
    }

    #[test]
    fn emscripten_bypass_and_debug_validation_are_explicit_commands() {
        resetGLCommandStream();
        let shader = CompileRawGLSLWithConfiguration(
            GL_VERTEX_SHADER,
            "real shader",
            DebugPrintErrorAndAbort::yes,
            GLUtilsSourceConfiguration {
                debug: true,
                bypassEmscriptenShaderParser: true,
            },
        );
        assert_eq!(
            takeGLCommands(),
            vec![
                GLCommand::CreateShader(GL_VERTEX_SHADER, shader),
                GLCommand::ShaderSourceBypassingEmscripten {
                    shader,
                    minimal_source: "#version 300 es\nvoid main() { gl_Position = vec4(0); }"
                        .to_owned(),
                    raw_source: "real shader".to_owned(),
                },
                GLCommand::CompileShader(shader),
                GLCommand::ValidateShaderCompilationAndAbort {
                    shader,
                    stderr_flush_delay_ms: 1000,
                },
            ]
        );
    }

    #[test]
    fn program_destroys_program_then_fragment_then_vertex() {
        resetGLCommandStream();
        {
            let mut program = Program::new();
            takeGLCommands();
            program.m_vertexShader.0.setSyntheticID(11);
            program.m_fragmentShader.0.setSyntheticID(12);
            program.m_object.setSyntheticID(13);
        }
        assert_eq!(
            takeGLCommands(),
            vec![
                GLCommand::DeleteProgram(13),
                GLCommand::DeleteShader(12),
                GLCommand::DeleteShader(11),
            ]
        );
    }

    #[test]
    fn image_sampling_and_blit_lower_exactly() {
        resetGLCommandStream();
        SetTexture2DSamplingParamsFromSampler(ImageSampler {
            wrapX: ImageWrap::repeat,
            wrapY: ImageWrap::mirror,
            filter: ImageFilter::bilinear,
        });
        BlitFramebuffer(
            IAABB {
                left: 1,
                top: 2,
                right: 5,
                bottom: 7,
            },
            20,
            GL_COLOR_BUFFER_BIT,
        );
        assert_eq!(
            takeGLCommands(),
            vec![
                GLCommand::TextureParameter(
                    GL_TEXTURE_2D,
                    GL_TEXTURE_MIN_FILTER,
                    GL_LINEAR_MIPMAP_NEAREST as GLint,
                ),
                GLCommand::TextureParameter(
                    GL_TEXTURE_2D,
                    GL_TEXTURE_MAG_FILTER,
                    GL_LINEAR as GLint,
                ),
                GLCommand::TextureParameter(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT as GLint,),
                GLCommand::TextureParameter(
                    GL_TEXTURE_2D,
                    GL_TEXTURE_WRAP_T,
                    GL_MIRRORED_REPEAT as GLint,
                ),
                GLCommand::BlitFramebuffer(
                    [1, 13, 5, 18, 1, 13, 5, 18],
                    GL_COLOR_BUFFER_BIT,
                    GL_NEAREST,
                ),
            ]
        );
    }
}
