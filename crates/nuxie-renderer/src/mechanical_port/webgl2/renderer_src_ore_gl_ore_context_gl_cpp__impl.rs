//! Complete mechanical implementation translation of
//! `renderer/src/ore/gl/ore_context_gl.cpp` for the frozen WebGL2 branch.

#![allow(non_snake_case)]

use super::gles3_decl::*;
use super::ore_bind_group_gl_decl::{
    BindGroupGL, BindGroupLayoutGL, GLSamplerBinding, GLTexBinding, GLUBOBinding,
};
use super::ore_buffer_gl_decl::BufferGL;
use super::ore_context_gl_decl::ContextGL;
use super::ore_pipeline_gl_decl::PipelineGL;
use super::ore_render_pass_gl_decl::{GLResolveEntry, RenderPassGL};
use super::ore_sampler_gl_decl::SamplerGL;
use super::ore_shader_module_gl_decl::ShaderModuleGL;
use super::ore_texture_gl_decl::{TextureGL, TextureViewGL};
use super::render_target_gl_decl::{
    RenderTargetGL, TextureRenderTargetGL, TEXTURE_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID,
};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::LiteRttiBase;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture as RiveTexture;
use nuxie_ore_metal::bind_group_layout::BindGroupLayout;
use nuxie_ore_metal::binding_map::BindingMap;
use nuxie_ore_metal::buffer::BufferApi;
use nuxie_ore_metal::context::{ActiveRenderPass, ContextApi, FrameDescriptor, ShaderTarget};
use nuxie_ore_metal::gpu_resource::{AnyResourceHandle, ResourceHandle};
use nuxie_ore_metal::render_pass::RenderPassApi;
use nuxie_ore_metal::shader_module::GLFixupKind;
use nuxie_ore_metal::texture::TextureApi;
use nuxie_ore_metal::types::{
    kMaxBindGroups, BindGroupDesc, BindGroupLayoutDesc, BindGroupLayoutEntry, BindingKind,
    BufferDesc, BufferUsage, CompareFunction, Features, Filter, LoadOp, PipelineDesc,
    RenderPassDesc, SamplerDesc, ShaderModuleDesc, ShaderStage, TextureAspect, TextureDesc,
    TextureFormat, TextureType, TextureViewDesc, TextureViewDimension, WrapMode,
};
use std::ffi::c_void;
use std::rc::Weak as RcWeak;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_gl_ore_context_gl.cpp");

fn withCurrentContext<R>(context: &mut ContextGL, callback: impl FnOnce(&mut ContextGL) -> R) -> R {
    let execution = context.executionStamp().clone();
    execution.withCurrent(|| callback(context))
}

fn withCurrentContextRef<R>(context: &ContextGL, callback: impl FnOnce(&ContextGL) -> R) -> R {
    let execution = context.executionStamp().clone();
    execution.withCurrent(|| callback(context))
}

fn compatibleExecution(context: &ContextGL, resource: &GLExecutionStamp) -> bool {
    let execution = context.executionStamp();
    execution.sameDomain(resource) && execution.generation() == resource.generation()
}

fn submit(_context: &mut ContextGL, command: GLCommand) {
    recordGLCommand(command);
}

fn generateName(_context: &mut ContextGL, kind: GLObjectKind) -> GLuint {
    generateGLObject(kind)
}

fn createProgram(_context: &mut ContextGL) -> GLuint {
    createGLProgram()
}

fn createShader(_context: &mut ContextGL, shaderType: GLenum) -> GLuint {
    createGLShader(shaderType)
}

fn sourceCString(bytes: &[u8]) -> &[u8] {
    bytes.split(|byte| *byte == 0).next().unwrap_or_default()
}

fn sourceLog(bytes: &[u8]) -> String {
    String::from_utf8_lossy(sourceCString(bytes)).into_owned()
}

fn reject(context: &ContextGL, message: impl Into<String>) {
    context.base.setLastError(message.into());
}

pub(crate) fn oreFormatToGLInternal(format: TextureFormat) -> GLenum {
    match format {
        TextureFormat::r8unorm => GL_R8,
        TextureFormat::rg8unorm => GL_RG8,
        TextureFormat::rgba8unorm | TextureFormat::bgra8unorm => GL_RGBA8,
        TextureFormat::rgba8snorm => GL_RGBA8_SNORM,
        TextureFormat::rgba16float => GL_RGBA16F,
        TextureFormat::rg16float => GL_RG16F,
        TextureFormat::r16float => GL_R16F,
        TextureFormat::rgba32float => GL_RGBA32F,
        TextureFormat::rg32float => GL_RG32F,
        TextureFormat::r32float => GL_R32F,
        TextureFormat::rgb10a2unorm => GL_RGB10_A2,
        TextureFormat::r11g11b10float => GL_R11F_G11F_B10F,
        TextureFormat::depth16unorm => GL_DEPTH_COMPONENT16,
        TextureFormat::depth24plusStencil8 => GL_DEPTH24_STENCIL8,
        TextureFormat::depth32float => GL_DEPTH_COMPONENT32F,
        TextureFormat::depth32floatStencil8 => GL_DEPTH32F_STENCIL8,
        // Frozen Emscripten gles3.hpp preprocessing leaves both desktop BC
        // macros undefined, regardless of numeric constants in the Rust GL
        // vocabulary.
        TextureFormat::bc1unorm | TextureFormat::bc3unorm | TextureFormat::bc7unorm => {
            unreachable!("BC texture formats are unreachable in the pinned WebGL2 branch")
        }
        TextureFormat::etc2rgb8 => GL_COMPRESSED_RGB8_ETC2,
        TextureFormat::etc2rgba8 => GL_COMPRESSED_RGBA8_ETC2_EAC,
        TextureFormat::astc4x4 => GL_COMPRESSED_RGBA_ASTC_4x4_KHR,
        TextureFormat::astc6x6 => GL_COMPRESSED_RGBA_ASTC_6x6_KHR,
        TextureFormat::astc8x8 => GL_COMPRESSED_RGBA_ASTC_8x8_KHR,
    }
}

pub(crate) fn oreTextureTypeToGLTarget(textureType: TextureType) -> GLenum {
    match textureType {
        TextureType::texture2D => GL_TEXTURE_2D,
        TextureType::cube => GL_TEXTURE_CUBE_MAP,
        TextureType::texture3D => GL_TEXTURE_3D,
        TextureType::array2D => GL_TEXTURE_2D_ARRAY,
    }
}

pub(crate) fn oreFilterToGL(filter: Filter) -> GLenum {
    match filter {
        Filter::linear => GL_LINEAR,
        Filter::nearest => GL_NEAREST,
    }
}

pub(crate) fn oreMipmapFilterToGL(min: Filter, mip: Filter) -> GLenum {
    match (min, mip) {
        (Filter::linear, Filter::linear) => GL_LINEAR_MIPMAP_LINEAR,
        (Filter::linear, Filter::nearest) => GL_LINEAR_MIPMAP_NEAREST,
        (Filter::nearest, Filter::linear) => GL_NEAREST_MIPMAP_LINEAR,
        (Filter::nearest, Filter::nearest) => GL_NEAREST_MIPMAP_NEAREST,
    }
}

pub(crate) fn oreWrapToGL(mode: WrapMode) -> GLenum {
    match mode {
        WrapMode::repeat => GL_REPEAT,
        WrapMode::mirrorRepeat => GL_MIRRORED_REPEAT,
        WrapMode::clampToEdge => GL_CLAMP_TO_EDGE,
    }
}

pub(crate) fn oreCompareFunctionToGL(function: CompareFunction) -> GLenum {
    match function {
        CompareFunction::none | CompareFunction::never => GL_NEVER,
        CompareFunction::less => GL_LESS,
        CompareFunction::equal => GL_EQUAL,
        CompareFunction::lessEqual => GL_LEQUAL,
        CompareFunction::greater => GL_GREATER,
        CompareFunction::notEqual => GL_NOTEQUAL,
        CompareFunction::greaterEqual => GL_GEQUAL,
        CompareFunction::always => GL_ALWAYS,
    }
}

fn queriedFeatures(executionDomain: &GLExecutionDomain) -> Features {
    let mut features = Features::default();

    features.colorBufferFloat = false;
    features.perTargetBlend = false;
    features.perTargetWriteMask = false;
    features.textureViewSampling = false;
    features.drawBaseInstance = false;
    features.depthBiasClamp = false;
    features.anisotropicFiltering = false;
    features.texture3D = true;
    features.textureArrays = true;
    features.computeShaders = false;
    features.storageBuffers = false;
    features.bc = false;
    features.etc2 = true;
    features.astc = false;

    let maxTextureSize = executionDomain.getInteger(GL_MAX_TEXTURE_SIZE);
    let maxCubeSize = executionDomain.getInteger(GL_MAX_CUBE_MAP_TEXTURE_SIZE);
    let max3DSize = executionDomain.getInteger(GL_MAX_3D_TEXTURE_SIZE);
    let maxUniformBufferSize = executionDomain.getInteger(GL_MAX_UNIFORM_BLOCK_SIZE);
    let maxDrawBuffers = executionDomain.getInteger(GL_MAX_DRAW_BUFFERS);
    let maxVertexAttributes = executionDomain.getInteger(GL_MAX_VERTEX_ATTRIBS);
    let maxTextureUnits = executionDomain.getInteger(GL_MAX_COMBINED_TEXTURE_IMAGE_UNITS);
    let maxSamples = executionDomain.getInteger(GL_MAX_SAMPLES);

    features.maxTextureSize2D = maxTextureSize as u32;
    features.maxTextureSizeCube = maxCubeSize as u32;
    features.maxTextureSize3D = max3DSize as u32;
    features.maxUniformBufferSize = maxUniformBufferSize as u32;
    features.maxColorAttachments = (maxDrawBuffers as u32).min(4);
    features.maxVertexAttributes = maxVertexAttributes as u32;
    features.maxSamplers = maxTextureUnits as u32;
    features.maxSamples = maxSamples.max(1) as u32;

    let extensionCount = executionDomain.getInteger(GL_NUM_EXTENSIONS);
    for index in 0..extensionCount.max(0) as u32 {
        let Some(extension) = executionDomain.getExtension(index) else {
            continue;
        };
        match sourceCString(&extension) {
            b"GL_EXT_color_buffer_float" => {
                features.colorBufferFloat = true;
                features.colorBufferHalfFloat = true;
            }
            b"GL_EXT_texture_filter_anisotropic" => {
                features.anisotropicFiltering = true;
            }
            b"GL_KHR_texture_compression_astc_ldr" => {
                features.astc = true;
            }
            _ => {}
        }
    }

    // Frozen RIVE_WEBGL branch. These two capabilities are independent.
    if executionDomain.enableWebGLExtension("EXT_color_buffer_float") {
        features.colorBufferFloat = true;
        features.colorBufferHalfFloat = true;
    }
    if executionDomain.enableWebGLExtension("EXT_color_buffer_half_float") {
        features.colorBufferHalfFloat = true;
    }

    features
}

pub(crate) fn Make(executionStamp: GLExecutionStamp) -> Option<Box<ContextGL>> {
    let features = executionStamp.withCurrent(|| queriedFeatures(executionStamp.domain()));
    Some(Box::new(ContextGL::newBase(features, executionStamp)))
}

/// The authored destructor body is empty.
pub(crate) fn destroy(_context: &mut ContextGL) {}

fn beginFrameCurrent(context: &mut ContextGL, _descriptor: &FrameDescriptor) {
    let program = context.executionDomain().getInteger(GL_CURRENT_PROGRAM);
    let arrayBuffer = context
        .executionDomain()
        .getInteger(GL_ARRAY_BUFFER_BINDING);
    let uniformBuffer = context
        .executionDomain()
        .getInteger(GL_UNIFORM_BUFFER_BINDING);
    let framebuffer = context.executionDomain().getInteger(GL_FRAMEBUFFER_BINDING);
    let vertexArray = context
        .executionDomain()
        .getInteger(GL_VERTEX_ARRAY_BINDING);
    context.m_savedState.program = program;
    context.m_savedState.arrayBuffer = arrayBuffer;
    context.m_savedState.uniformBuffer = uniformBuffer;
    context.m_savedState.framebuffer = framebuffer;
    context.m_savedState.vertexArray = vertexArray;
}

/// Exact authored no-op. Do not add finish, flush, or waiting here.
fn waitForGPUCurrent(_context: &mut ContextGL) {}

fn endFrameCurrent(context: &mut ContextGL) {
    let program = context.m_savedState.program;
    if program == 0
        || context
            .executionDomain()
            .isObject(GLObjectKind::Program, program as GLuint)
    {
        submit(context, GLCommand::UseProgram(program as GLuint));
    }

    let vertexArray = context.m_savedState.vertexArray;
    if vertexArray == 0
        || context
            .executionDomain()
            .isObject(GLObjectKind::VertexArray, vertexArray as GLuint)
    {
        submit(context, GLCommand::BindVertexArray(vertexArray as GLuint));
    }

    let arrayBuffer = context.m_savedState.arrayBuffer;
    if arrayBuffer == 0
        || context
            .executionDomain()
            .isObject(GLObjectKind::Buffer, arrayBuffer as GLuint)
    {
        submit(
            context,
            GLCommand::BindBuffer(GL_ARRAY_BUFFER, arrayBuffer as GLuint),
        );
    }

    let uniformBuffer = context.m_savedState.uniformBuffer;
    if uniformBuffer == 0
        || context
            .executionDomain()
            .isObject(GLObjectKind::Buffer, uniformBuffer as GLuint)
    {
        submit(
            context,
            GLCommand::BindBuffer(GL_UNIFORM_BUFFER, uniformBuffer as GLuint),
        );
    }

    let framebuffer = context.m_savedState.framebuffer;
    if framebuffer == 0
        || context
            .executionDomain()
            .isObject(GLObjectKind::Framebuffer, framebuffer as GLuint)
    {
        submit(
            context,
            GLCommand::BindFramebuffer(GL_FRAMEBUFFER, framebuffer as GLuint),
        );
    }
}

fn makeBufferCurrent(context: &mut ContextGL, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
    let data = match desc.data_prefix() {
        Ok(data) => data.map(<[u8]>::to_vec),
        Err(_) => {
            reject(
                context,
                "makeBuffer: BufferDesc::size exceeds the supplied data span",
            );
            return None;
        }
    };

    let mut buffer = BufferGL::new(desc.size, desc.usage, context.executionStamp().clone());
    buffer.m_glBuffer = generateName(context, GLObjectKind::Buffer);
    buffer.m_glTarget = match desc.usage {
        BufferUsage::vertex => GL_ARRAY_BUFFER,
        BufferUsage::index => GL_ELEMENT_ARRAY_BUFFER,
        BufferUsage::uniform => GL_UNIFORM_BUFFER,
        BufferUsage::upload => unreachable!("BufferUsage::upload is unreachable for ContextGL"),
    };

    let usage = if desc.data.is_some() {
        GL_STATIC_DRAW
    } else {
        GL_DYNAMIC_DRAW
    };
    if desc.usage == BufferUsage::index {
        let previousEBO = context
            .executionDomain()
            .getInteger(GL_ELEMENT_ARRAY_BUFFER_BINDING);
        submit(
            context,
            GLCommand::BindBuffer(GL_ELEMENT_ARRAY_BUFFER, buffer.m_glBuffer),
        );
        submit(
            context,
            GLCommand::BufferData {
                target: GL_ELEMENT_ARRAY_BUFFER,
                size: desc.size as usize,
                data,
                usage,
            },
        );
        submit(
            context,
            GLCommand::BindBuffer(GL_ELEMENT_ARRAY_BUFFER, previousEBO as GLuint),
        );
    } else {
        submit(
            context,
            GLCommand::BindBuffer(GL_COPY_WRITE_BUFFER, buffer.m_glBuffer),
        );
        submit(
            context,
            GLCommand::BufferData {
                target: GL_COPY_WRITE_BUFFER,
                size: desc.size as usize,
                data,
                usage,
            },
        );
        submit(context, GLCommand::BindBuffer(GL_COPY_WRITE_BUFFER, 0));
    }

    let domain = nuxie_ore_metal::context_backend_domain(&context.base);
    Some(ResourceHandle::new_buffer_in_domain(None, domain, buffer).erase())
}

fn makeTextureCurrent(
    context: &mut ContextGL,
    desc: &TextureDesc<'_>,
) -> Option<AnyResourceHandle> {
    let mut texture = TextureGL::new(desc, context.executionStamp().clone());
    texture.m_glTexture = generateName(context, GLObjectKind::Texture);
    texture.m_glTarget = oreTextureTypeToGLTarget(desc.r#type);
    let internalFormat = oreFormatToGLInternal(desc.format);

    submit(context, GLCommand::ActiveTexture(GL_TEXTURE0));
    submit(
        context,
        GLCommand::BindTexture(texture.m_glTarget, texture.m_glTexture),
    );

    match desc.r#type {
        TextureType::texture2D if desc.sampleCount > 1 => {
            submit(context, GLCommand::DeleteTexture(texture.m_glTexture));
            texture.m_glTexture = 0;
            texture.m_glTarget = 0;

            texture.m_glRenderbuffer = generateName(context, GLObjectKind::Renderbuffer);
            submit(
                context,
                GLCommand::BindRenderbuffer(GL_RENDERBUFFER, texture.m_glRenderbuffer),
            );
            submit(
                context,
                GLCommand::RenderbufferStorageMultisample {
                    target: GL_RENDERBUFFER,
                    samples: desc.sampleCount as GLsizei,
                    internal_format: internalFormat,
                    width: desc.width,
                    height: desc.height,
                },
            );
            submit(context, GLCommand::BindRenderbuffer(GL_RENDERBUFFER, 0));
        }
        TextureType::texture2D => submit(
            context,
            GLCommand::TexStorage2D {
                target: GL_TEXTURE_2D,
                levels: desc.numMipmaps,
                internal_format: internalFormat,
                width: desc.width,
                height: desc.height,
            },
        ),
        TextureType::cube => submit(
            context,
            GLCommand::TexStorage2D {
                target: GL_TEXTURE_CUBE_MAP,
                levels: desc.numMipmaps,
                internal_format: internalFormat,
                width: desc.width,
                height: desc.height,
            },
        ),
        TextureType::texture3D => submit(
            context,
            GLCommand::TexStorage3D {
                target: GL_TEXTURE_3D,
                levels: desc.numMipmaps,
                internal_format: internalFormat,
                width: desc.width,
                height: desc.height,
                depth: desc.depthOrArrayLayers,
            },
        ),
        TextureType::array2D => submit(
            context,
            GLCommand::TexStorage3D {
                target: GL_TEXTURE_2D_ARRAY,
                levels: desc.numMipmaps,
                internal_format: internalFormat,
                width: desc.width,
                height: desc.height,
                depth: desc.depthOrArrayLayers,
            },
        ),
    }

    if texture.m_glRenderbuffer == 0 {
        submit(
            context,
            GLCommand::TextureParameter(
                texture.m_glTarget,
                GL_TEXTURE_MIN_FILTER,
                GL_NEAREST as GLint,
            ),
        );
        submit(
            context,
            GLCommand::TextureParameter(
                texture.m_glTarget,
                GL_TEXTURE_MAG_FILTER,
                GL_NEAREST as GLint,
            ),
        );
        submit(
            context,
            GLCommand::TextureParameter(
                texture.m_glTarget,
                GL_TEXTURE_WRAP_S,
                GL_CLAMP_TO_EDGE as GLint,
            ),
        );
        submit(
            context,
            GLCommand::TextureParameter(
                texture.m_glTarget,
                GL_TEXTURE_WRAP_T,
                GL_CLAMP_TO_EDGE as GLint,
            ),
        );
        submit(context, GLCommand::BindTexture(texture.m_glTarget, 0));
    }

    // Preserve the pinned defect: makeTexture never sets m_glOwnsTexture.
    debug_assert!(!texture.m_glOwnsTexture);
    let domain = nuxie_ore_metal::context_backend_domain(&context.base);
    Some(ResourceHandle::new_texture_in_domain(None, domain, texture).erase())
}

fn makeTextureViewCurrent(
    context: &mut ContextGL,
    desc: &TextureViewDesc<'_>,
) -> Option<AnyResourceHandle> {
    let textureOwner = desc.texture?.clone();
    let Some(texture) = textureOwner.downcast_ref::<TextureGL>() else {
        reject(
            context,
            "makeTextureView: texture does not belong to the GL backend",
        );
        return None;
    };
    if !compatibleExecution(context, texture.executionStamp()) {
        reject(
            context,
            "makeTextureView: texture belongs to a different GL execution generation",
        );
        return None;
    }
    let domain = nuxie_ore_metal::context_backend_domain(&context.base);

    let mut view = TextureViewGL::new(textureOwner, desc, context.executionStamp().clone());
    view.m_glTextureView = 0;
    Some(ResourceHandle::new_in_domain(None, domain, view).erase())
}

fn makeSamplerCurrent(
    context: &mut ContextGL,
    desc: &SamplerDesc<'_>,
) -> Option<AnyResourceHandle> {
    let mut sampler = SamplerGL::new(context.executionStamp().clone());
    sampler.m_glSampler = generateName(context, GLObjectKind::Sampler);
    let name = sampler.m_glSampler;

    let minFilter = if desc.maxLod > 0.0 {
        oreMipmapFilterToGL(desc.minFilter, desc.mipmapFilter)
    } else {
        oreFilterToGL(desc.minFilter)
    };
    submit(
        context,
        GLCommand::SamplerParameterInt {
            sampler: name,
            parameter: GL_TEXTURE_MIN_FILTER,
            value: minFilter as GLint,
        },
    );
    submit(
        context,
        GLCommand::SamplerParameterInt {
            sampler: name,
            parameter: GL_TEXTURE_MAG_FILTER,
            value: oreFilterToGL(desc.magFilter) as GLint,
        },
    );
    for (parameter, value) in [
        (GL_TEXTURE_WRAP_S, oreWrapToGL(desc.wrapU)),
        (GL_TEXTURE_WRAP_T, oreWrapToGL(desc.wrapV)),
        (GL_TEXTURE_WRAP_R, oreWrapToGL(desc.wrapW)),
    ] {
        submit(
            context,
            GLCommand::SamplerParameterInt {
                sampler: name,
                parameter,
                value: value as GLint,
            },
        );
    }
    submit(
        context,
        GLCommand::SamplerParameterFloat {
            sampler: name,
            parameter: GL_TEXTURE_MIN_LOD,
            value: desc.minLod,
        },
    );
    submit(
        context,
        GLCommand::SamplerParameterFloat {
            sampler: name,
            parameter: GL_TEXTURE_MAX_LOD,
            value: desc.maxLod,
        },
    );
    if desc.compare != CompareFunction::none {
        submit(
            context,
            GLCommand::SamplerParameterInt {
                sampler: name,
                parameter: GL_TEXTURE_COMPARE_MODE,
                value: GL_COMPARE_REF_TO_TEXTURE as GLint,
            },
        );
        submit(
            context,
            GLCommand::SamplerParameterInt {
                sampler: name,
                parameter: GL_TEXTURE_COMPARE_FUNC,
                value: oreCompareFunctionToGL(desc.compare) as GLint,
            },
        );
    }

    let domain = nuxie_ore_metal::context_backend_domain(&context.base);
    Some(ResourceHandle::new_in_domain(None, domain, sampler).erase())
}

fn validatedShaderSource(
    context: &ContextGL,
    desc: &ShaderModuleDesc<'_>,
) -> Option<Option<Vec<u8>>> {
    let codeSize = match desc.codeSize() {
        Ok(size) => size as usize,
        Err(_) => {
            reject(
                context,
                "makeShaderModule: codeSize exceeds the supplied code span",
            );
            return None;
        }
    };
    let source = match desc.code {
        Some(code) => Some(code.get(..codeSize)?.to_vec()),
        None if codeSize == 0 => None,
        None => return None,
    };

    let bindingMapSize = match desc.bindingMapSize() {
        Ok(size) => size as usize,
        Err(_) => {
            reject(
                context,
                "makeShaderModule: bindingMapSize exceeds its backing span",
            );
            return None;
        }
    };
    if bindingMapSize == 0 || desc.bindingMapBytes.is_none() {
        reject(context, "makeShaderModule: binding-map sidecar is required");
        return None;
    }
    let mut parsedMap = BindingMap::default();
    if !BindingMap::fromBlob(desc.bindingMapBytes, bindingMapSize, Some(&mut parsedMap)) {
        reject(
            context,
            "makeShaderModule: binding-map sidecar failed to parse",
        );
        return None;
    }
    if desc.glFixupSize().is_err() {
        reject(
            context,
            "makeShaderModule: glFixupSize exceeds its backing span",
        );
        return None;
    }
    Some(source)
}

fn makeShaderModuleCurrent(
    context: &mut ContextGL,
    desc: &ShaderModuleDesc<'_>,
) -> Option<AnyResourceHandle> {
    let source = validatedShaderSource(context, desc)?;
    let sourceBytes = source.as_deref().unwrap_or_default();
    let mut module = ShaderModuleGL::new(context.executionStamp().clone());

    let isVertex = if desc.stage != ShaderStage::autoDetect {
        desc.stage == ShaderStage::vertex
    } else {
        sourceBytes
            .windows(b"gl_Position".len())
            .any(|window| window == b"gl_Position")
    };
    module.m_glShaderType = if isVertex {
        GL_VERTEX_SHADER
    } else {
        GL_FRAGMENT_SHADER
    };
    module.m_glShader = createShader(context, module.m_glShaderType);

    submit(
        context,
        GLCommand::ShaderSourceBytes {
            shader: module.m_glShader,
            source,
        },
    );
    submit(context, GLCommand::CompileShader(module.m_glShader));

    let status = context
        .executionDomain()
        .shaderParameter(module.m_glShader, GL_COMPILE_STATUS);
    if status == 0 {
        let logLength = context
            .executionDomain()
            .shaderParameter(module.m_glShader, GL_INFO_LOG_LENGTH);
        let log = if logLength > 0 {
            let maxLength = (logLength as usize).min(1024);
            context
                .executionDomain()
                .shaderInfoLog(module.m_glShader, maxLength)
        } else {
            Vec::new()
        };
        context
            .base
            .setLastError(format!("Ore GL shader compile error: {}", sourceLog(&log)));
        submit(context, GLCommand::DeleteShader(module.m_glShader));
        // Preserve the source defect: m_glShader remains nonzero, so the
        // local ShaderModuleGL destructor issues a second deletion.
        return None;
    }

    module.applyBindingMapFromDesc(desc);
    let domain = nuxie_ore_metal::context_backend_domain(&context.base);
    Some(ResourceHandle::new_in_domain(None, domain, module).erase())
}

pub(crate) fn attachNoOpGLFragmentShader(context: &mut ContextGL, program: GLuint) {
    const SOURCE: &[u8] = b"#version 300 es\nvoid main() {}\n";
    let shader = createShader(context, GL_FRAGMENT_SHADER);
    submit(
        context,
        GLCommand::ShaderSourceBytes {
            shader,
            source: Some(SOURCE.to_vec()),
        },
    );
    submit(context, GLCommand::CompileShader(shader));
    submit(context, GLCommand::AttachShader(program, shader));
    submit(context, GLCommand::DeleteShader(shader));
}

pub(crate) fn oreGLFixupProgramBindings(
    context: &mut ContextGL,
    program: GLuint,
    vertexModule: Option<&ShaderModuleGL>,
    fragmentModule: Option<&ShaderModuleGL>,
) {
    submit(context, GLCommand::UseProgram(program));

    let apply = |context: &mut ContextGL, entry: &nuxie_ore_metal::shader_module::GLFixupEntry| {
        if entry.kind == GLFixupKind::UBOBlock {
            let index = context
                .executionDomain()
                .uniformBlockIndex(program, &entry.name);
            if index != GL_INVALID_INDEX {
                submit(
                    context,
                    GLCommand::UniformBlockBinding {
                        program,
                        block_index: index,
                        binding: entry.slot.into(),
                    },
                );
            }
        } else {
            // Preserve the source forward-compatible branch: every unknown
            // raw kind is treated as SamplerUniform.
            let location = context
                .executionDomain()
                .uniformLocation(program, &entry.name);
            if location >= 0 {
                submit(
                    context,
                    GLCommand::Uniform1iLocation {
                        location,
                        value: entry.slot.into(),
                    },
                );
            }
        }
    };

    if let Some(module) = vertexModule {
        for entry in &module.m_glFixup {
            apply(context, entry);
        }
    }
    if let Some(module) = fragmentModule {
        if vertexModule.is_none_or(|vertex| !std::ptr::eq(vertex, module)) {
            for entry in &module.m_glFixup {
                apply(context, entry);
            }
        }
    }

    submit(context, GLCommand::UseProgram(0));
}

fn validatePipelineSpans(context: &ContextGL, desc: &PipelineDesc<'_>) -> bool {
    if desc.colorCount > 4 {
        reject(
            context,
            "makePipeline: colorCount exceeds the four source color targets",
        );
        return false;
    }
    let Ok(vertexBufferCount) = desc.vertexBufferCount() else {
        reject(
            context,
            "makePipeline: vertexBufferCount exceeds its backing span",
        );
        return false;
    };
    if desc.bindGroupLayoutCount().is_err() {
        reject(
            context,
            "makePipeline: bindGroupLayoutCount exceeds its backing span",
        );
        return false;
    }
    for layout in desc
        .vertexBuffers
        .unwrap_or_default()
        .iter()
        .take(vertexBufferCount as usize)
    {
        if layout.attributeCount().is_err() {
            reject(
                context,
                "makePipeline: vertex attributeCount exceeds its backing span",
            );
            return false;
        }
    }
    true
}

fn publishPipelineError(context: &ContextGL, outError: &mut Option<&mut String>, message: String) {
    if let Some(destination) = outError.as_deref_mut() {
        *destination = message;
    } else {
        context
            .base
            .setLastError(format!("makePipeline: {message}"));
    }
}

fn makePipelineCurrent(
    context: &mut ContextGL,
    desc: &PipelineDesc<'_>,
    mut outError: Option<&mut String>,
) -> Option<AnyResourceHandle> {
    if !validatePipelineSpans(context, desc) {
        if let Some(destination) = outError.as_deref_mut() {
            *destination = context.base.lastError();
        }
        return None;
    }

    let Some(mut pipeline) = PipelineGL::new(desc, context.executionStamp().clone()) else {
        publishPipelineError(
            context,
            &mut outError,
            "descriptor spans are invalid".to_owned(),
        );
        return None;
    };

    let Some(vertexOwner) = desc.vertexModule else {
        publishPipelineError(context, &mut outError, "vertex module is null".to_owned());
        return None;
    };
    let Some(vertexModule) = vertexOwner.downcast_ref::<ShaderModuleGL>() else {
        debug_assert!(false, "ContextGL requires ShaderModuleGL vertex modules");
        publishPipelineError(
            context,
            &mut outError,
            "vertex module does not belong to the GL backend".to_owned(),
        );
        return None;
    };
    if !compatibleExecution(context, vertexModule.executionStamp()) {
        publishPipelineError(
            context,
            &mut outError,
            "vertex module belongs to a different GL execution generation".to_owned(),
        );
        return None;
    }
    let fragmentModule = match desc.fragmentModule {
        Some(owner) => match owner.downcast_ref::<ShaderModuleGL>() {
            Some(module) if compatibleExecution(context, module.executionStamp()) => Some(module),
            Some(_) => {
                publishPipelineError(
                    context,
                    &mut outError,
                    "fragment module belongs to a different GL execution generation".to_owned(),
                );
                return None;
            }
            None => {
                debug_assert!(false, "ContextGL requires ShaderModuleGL fragment modules");
                publishPipelineError(
                    context,
                    &mut outError,
                    "fragment module does not belong to the GL backend".to_owned(),
                );
                return None;
            }
        },
        None => None,
    };

    // Restore the C++ derived-to-base copy made by Pipeline(desc). Rust's
    // erased base constructor cannot discover ShaderModuleGL on its own.
    *pipeline.m_bindingMap = vertexModule.m_bindingMap.clone();

    use nuxie_ore_metal::bind_group_layout::{
        validateColorRequiresFragment, validateLayoutBasesAgainstBindingMap,
    };
    let layoutCount = desc.bindGroupLayoutCount().ok()? as usize;
    let layoutHandles = desc
        .bindGroupLayouts
        .unwrap_or_default()
        .get(..layoutCount)?;
    let mut layoutBases = Vec::with_capacity(layoutCount);
    for layout in layoutHandles {
        let Some(layoutOwner) = layout else {
            layoutBases.push(None);
            continue;
        };
        let Some(layout) = layoutOwner.downcast_ref::<BindGroupLayoutGL>() else {
            publishPipelineError(
                context,
                &mut outError,
                "bind-group layout does not belong to the GL backend".to_owned(),
            );
            return None;
        };
        if !compatibleExecution(context, layout.executionStamp()) {
            publishPipelineError(
                context,
                &mut outError,
                "bind-group layout belongs to a different GL execution generation".to_owned(),
            );
            return None;
        }
        layoutBases.push(Some(&**layout));
    }
    let mut validationError = String::new();
    if !validateLayoutBasesAgainstBindingMap(
        &pipeline.m_bindingMap,
        desc.bindGroupLayouts.map(|_| layoutBases.as_slice()),
        desc.bindGroupLayoutCount,
        Some(&mut validationError),
    ) || !validateColorRequiresFragment(
        desc.colorCount,
        desc.fragmentModule.is_some(),
        Some(&mut validationError),
    ) {
        publishPipelineError(context, &mut outError, validationError);
        return None;
    }

    pipeline.m_glProgram = createProgram(context);
    submit(
        context,
        GLCommand::AttachShader(pipeline.m_glProgram, vertexModule.m_glShader),
    );
    if let Some(fragment) = fragmentModule {
        submit(
            context,
            GLCommand::AttachShader(pipeline.m_glProgram, fragment.m_glShader),
        );
    } else {
        attachNoOpGLFragmentShader(context, pipeline.m_glProgram);
    }

    let vertexBufferCount = desc.vertexBufferCount().ok()? as usize;
    for layout in desc
        .vertexBuffers
        .unwrap_or_default()
        .iter()
        .take(vertexBufferCount)
    {
        let attributeCount = layout.attributeCount().ok()? as usize;
        for attribute in layout.attributes.iter().take(attributeCount) {
            submit(
                context,
                GLCommand::BindAttribLocation {
                    program: pipeline.m_glProgram,
                    index: attribute.shaderSlot,
                    name: format!("a_attr{}", attribute.shaderSlot).into_bytes(),
                },
            );
        }
    }

    submit(context, GLCommand::LinkProgram(pipeline.m_glProgram));
    let status = context
        .executionDomain()
        .programParameter(pipeline.m_glProgram, GL_LINK_STATUS);
    if status == 0 {
        let mut linkLog = "GL program link failed".to_owned();
        let logLength = context
            .executionDomain()
            .programParameter(pipeline.m_glProgram, GL_INFO_LOG_LENGTH);
        if logLength > 0 {
            let bytes = context
                .executionDomain()
                .programInfoLog(pipeline.m_glProgram, (logLength as usize).min(1024));
            linkLog = sourceLog(&bytes);
        }
        context
            .base
            .setLastError(format!("Ore GL program link error: {linkLog}"));
        if let Some(destination) = outError.as_deref_mut() {
            *destination = linkLog;
        }
        return None;
    }

    oreGLFixupProgramBindings(
        context,
        pipeline.m_glProgram,
        Some(vertexModule),
        fragmentModule,
    );

    let domain = nuxie_ore_metal::context_backend_domain(&context.base);
    Some(ResourceHandle::new_in_domain(None, domain, pipeline).erase())
}

fn nativeSlot(
    context: &ContextGL,
    layout: &BindGroupLayout,
    groupIndex: u32,
    binding: u32,
    expected: BindingKind,
) -> Option<u32> {
    let Some(entry) = layout.findEntry(binding) else {
        reject(
            context,
            format!(
                "makeBindGroup: (group={groupIndex}, binding={binding}) not declared in BindGroupLayout"
            ),
        );
        return None;
    };
    let samplerKind =
        |kind: BindingKind| matches!(kind, BindingKind::sampler | BindingKind::comparisonSampler);
    let kindMatches = entry.kind == expected || (samplerKind(entry.kind) && samplerKind(expected));
    if !kindMatches {
        reject(
            context,
            format!("makeBindGroup: (group={groupIndex}, binding={binding}) layout kind mismatch"),
        );
        return None;
    }
    let slot = if entry.nativeSlotVS != BindGroupLayoutEntry::kNativeSlotAbsent {
        entry.nativeSlotVS
    } else {
        entry.nativeSlotFS
    };
    if slot == BindGroupLayoutEntry::kNativeSlotAbsent {
        reject(
            context,
            format!(
                "makeBindGroup: (group={groupIndex}, binding={binding}) layout has no resolved native slot — call makeLayoutFromShader"
            ),
        );
        return None;
    }
    Some(slot)
}

fn makeBindGroupCurrent(
    context: &mut ContextGL,
    desc: &BindGroupDesc<'_>,
) -> Option<AnyResourceHandle> {
    let Some(layoutOwner) = desc.layout else {
        reject(context, "makeBindGroup: BindGroupDesc::layout is null");
        return None;
    };
    let domain = nuxie_ore_metal::context_backend_domain(&context.base);
    let Some(layout) = layoutOwner.downcast_ref::<BindGroupLayoutGL>() else {
        reject(context, "makeBindGroup: layout has the wrong resource type");
        return None;
    };
    if !compatibleExecution(context, layout.executionStamp()) {
        reject(
            context,
            "makeBindGroup: layout belongs to a different GL execution generation",
        );
        return None;
    }
    let groupIndex = layout.groupIndex();
    if groupIndex >= kMaxBindGroups {
        reject(
            context,
            format!("makeBindGroup: layout->groupIndex {groupIndex} out of range"),
        );
        return None;
    }
    if desc.uboCount().is_err() || desc.textureCount().is_err() || desc.samplerCount().is_err() {
        reject(
            context,
            "makeBindGroup: a descriptor count exceeds its backing span",
        );
        return None;
    }

    let mut group = BindGroupGL::new(context.executionStamp().clone());
    nuxie_ore_metal::install_bind_group_backend_context(&mut group.base, &context.base);
    let mut dynamicOffsetCount = 0;
    let mut retainedBuffers = Vec::new();
    let mut retainedViews = Vec::new();
    let mut retainedSamplers = Vec::new();

    for entry in desc.ubos.iter().take(desc.uboCount.min(8) as usize) {
        let Some(bufferOwner) = entry.buffer else {
            debug_assert!(false, "ContextGL UBO entry requires a buffer");
            reject(context, "makeBindGroup: UBO entry buffer is null");
            return None;
        };
        let Some(buffer) = bufferOwner.downcast_ref::<BufferGL>() else {
            debug_assert!(false, "ContextGL requires BufferGL UBO entries");
            reject(context, "makeBindGroup: UBO entry is not BufferGL");
            return None;
        };
        if !compatibleExecution(context, buffer.executionStamp()) {
            reject(
                context,
                "makeBindGroup: UBO buffer belongs to a different GL execution generation",
            );
            return None;
        }
        let Some(slot) = nativeSlot(
            context,
            layout,
            groupIndex,
            entry.slot,
            BindingKind::uniformBuffer,
        ) else {
            continue;
        };
        let hasDynamicOffset = layout.hasDynamicOffset(entry.slot);
        if hasDynamicOffset {
            dynamicOffsetCount += 1;
        }
        group.m_glUBOs.push(GLUBOBinding {
            buffer: buffer.m_glBuffer,
            offset: entry.offset,
            size: if entry.size != 0 {
                entry.size
            } else {
                buffer.size()
            },
            binding: entry.slot,
            slot,
            hasDynamicOffset,
        });
        retainedBuffers.push(bufferOwner.clone());
    }
    group.m_glUBOs.sort_by_key(|binding| binding.binding);

    for entry in desc.textures.iter().take(desc.textureCount.min(8) as usize) {
        let Some(viewOwner) = entry.view else {
            debug_assert!(false, "ContextGL texture entry requires a view");
            reject(context, "makeBindGroup: texture entry view is null");
            return None;
        };
        let Some(view) = viewOwner.downcast_ref::<TextureViewGL>() else {
            debug_assert!(false, "ContextGL requires TextureViewGL entries");
            reject(context, "makeBindGroup: texture entry is not TextureViewGL");
            return None;
        };
        if !compatibleExecution(context, view.executionStamp()) {
            reject(
                context,
                "makeBindGroup: texture view belongs to a different GL execution generation",
            );
            return None;
        }
        let Some(texture) = view.texture().downcast_ref::<TextureGL>() else {
            debug_assert!(false, "TextureViewGL must retain TextureGL");
            reject(
                context,
                "makeBindGroup: texture view does not retain TextureGL",
            );
            return None;
        };
        if !compatibleExecution(context, texture.executionStamp()) {
            reject(
                context,
                "makeBindGroup: retained texture belongs to a different GL execution generation",
            );
            return None;
        }
        let Some(slot) = nativeSlot(
            context,
            layout,
            groupIndex,
            entry.slot,
            BindingKind::sampledTexture,
        ) else {
            continue;
        };
        group.m_glTextures.push(GLTexBinding {
            texture: if view.m_glTextureView != 0 {
                view.m_glTextureView
            } else {
                texture.m_glTexture
            },
            target: texture.m_glTarget,
            slot,
        });
        retainedViews.push(viewOwner.clone());
    }

    for entry in desc.samplers.iter().take(desc.samplerCount.min(8) as usize) {
        let Some(samplerOwner) = entry.sampler else {
            debug_assert!(false, "ContextGL sampler entry requires a sampler");
            reject(context, "makeBindGroup: sampler entry is null");
            return None;
        };
        let Some(sampler) = samplerOwner.downcast_ref::<SamplerGL>() else {
            debug_assert!(false, "ContextGL requires SamplerGL entries");
            reject(context, "makeBindGroup: sampler entry is not SamplerGL");
            return None;
        };
        if !compatibleExecution(context, sampler.executionStamp()) {
            reject(
                context,
                "makeBindGroup: sampler belongs to a different GL execution generation",
            );
            return None;
        }
        let Some(slot) = nativeSlot(
            context,
            layout,
            groupIndex,
            entry.slot,
            BindingKind::sampler,
        ) else {
            continue;
        };
        group.m_glSamplers.push(GLSamplerBinding {
            sampler: sampler.m_glSampler,
            slot,
        });
        retainedSamplers.push(samplerOwner.clone());
    }

    nuxie_ore_metal::install_bind_group_backend_parts(
        &mut group.base,
        dynamicOffsetCount,
        Some(layoutOwner.clone()),
        retainedBuffers,
        retainedViews,
        retainedSamplers,
    );
    Some(ResourceHandle::new_in_domain(None, domain, group).erase())
}

fn makeBindGroupLayoutCurrent(
    context: &mut ContextGL,
    desc: &BindGroupLayoutDesc<'_>,
) -> Option<AnyResourceHandle> {
    if desc.groupIndex >= kMaxBindGroups {
        reject(
            context,
            format!(
                "makeBindGroupLayout: groupIndex {} out of range [0, {})",
                desc.groupIndex, kMaxBindGroups
            ),
        );
        return None;
    }
    let entryCount = match desc.entryCount() {
        Ok(count) => count as usize,
        Err(_) => {
            reject(
                context,
                "makeBindGroupLayout: entryCount exceeds its backing span",
            );
            return None;
        }
    };
    let mut layout = BindGroupLayoutGL::new(context.executionStamp().clone());
    nuxie_ore_metal::install_bind_group_layout_backend_parts(
        &mut layout.base,
        &context.base,
        desc.groupIndex,
        desc.entries.iter().take(entryCount).copied().collect(),
    );
    let domain = nuxie_ore_metal::context_backend_domain(&context.base);
    Some(ResourceHandle::new_in_domain(None, domain, layout).erase())
}

type GLAttachment<'a> = (&'a TextureViewGL, &'a TextureGL);

fn glAttachment<'a>(
    context: &ContextGL,
    owner: &'a AnyResourceHandle,
    role: &str,
) -> Option<GLAttachment<'a>> {
    let Some(view) = owner.downcast_ref::<TextureViewGL>() else {
        debug_assert!(false, "ContextGL attachments require TextureViewGL");
        reject(
            context,
            format!("beginRenderPass: {role} is not TextureViewGL"),
        );
        return None;
    };
    if !compatibleExecution(context, view.executionStamp()) {
        reject(
            context,
            format!("beginRenderPass: {role} belongs to a different GL execution generation"),
        );
        return None;
    }
    let Some(texture) = view.texture().downcast_ref::<TextureGL>() else {
        debug_assert!(false, "TextureViewGL must retain TextureGL");
        reject(
            context,
            format!("beginRenderPass: {role} does not retain TextureGL"),
        );
        return None;
    };
    if !compatibleExecution(context, texture.executionStamp()) {
        reject(
            context,
            format!("beginRenderPass: {role} retains a stale or foreign GL texture"),
        );
        return None;
    }
    Some((view, texture))
}

fn beginRenderPassCurrent(
    context: &mut ContextGL,
    desc: &RenderPassDesc<'_>,
    _outError: Option<&mut String>,
) -> Option<Box<dyn RenderPassApi>> {
    context.base.finishActiveRenderPass();

    if desc.colorCount > 4 {
        reject(
            context,
            "beginRenderPass: colorCount exceeds the four source color attachments",
        );
        return None;
    }

    // Validate the borrowed descriptor graph before issuing GL mutations.
    // These are safe Rust boundary rejections for source assertions/casts.
    let mut colors: [Option<GLAttachment<'_>>; 4] = [None; 4];
    let mut resolves: [Option<GLAttachment<'_>>; 4] = [None; 4];
    for index in 0..desc.colorCount as usize {
        let attachment = &desc.colorAttachments[index];
        if let Some(owner) = attachment.view {
            colors[index] = Some(glAttachment(
                context,
                owner,
                &format!("color attachment {index}"),
            )?);
            if let Some(resolveOwner) = attachment.resolveTarget {
                resolves[index] = Some(glAttachment(
                    context,
                    resolveOwner,
                    &format!("resolve attachment {index}"),
                )?);
            }
        }
    }
    let depth = match desc.depthStencil.view {
        Some(owner) => Some(glAttachment(context, owner, "depth/stencil attachment")?),
        None => None,
    };

    let pass = RenderPassGL::new(&context.base, context.executionStamp().clone());
    let mut state = pass.inner.borrowState();

    let mut colorFormats = [TextureFormat::r8unorm; 4];
    let mut sampleCount = 1;
    for index in 0..desc.colorCount as usize {
        if let Some((_, texture)) = colors[index] {
            colorFormats[index] = texture.format();
            sampleCount = texture.sampleCount();
        }
    }
    let (depthFormat, hasDepth) = if let Some((_, texture)) = depth {
        if desc.colorCount == 0 {
            sampleCount = texture.sampleCount();
        }
        (texture.format(), true)
    } else {
        (TextureFormat::r8unorm, false)
    };
    nuxie_ore_metal::render_pass_install_attachment_metadata(
        &mut state.base,
        colorFormats,
        desc.colorCount,
        depthFormat,
        hasDepth,
        sampleCount,
    );

    // Source glGetIntegerv calls are synchronous in this execution domain;
    // store the actual host GLuints for RenderPassGL::finish restoration.
    state.m_prevVAO = context
        .executionDomain()
        .getInteger(GL_VERTEX_ARRAY_BINDING) as GLuint;
    state.m_prevFBO = context.executionDomain().getInteger(GL_FRAMEBUFFER_BINDING) as GLuint;

    state.m_glFBO = generateName(context, GLObjectKind::Framebuffer);
    state.m_ownsFBO = true;
    submit(
        context,
        GLCommand::BindFramebuffer(GL_FRAMEBUFFER, state.m_glFBO),
    );

    let mut drawBuffers = vec![GL_NONE; desc.colorCount as usize];
    for index in 0..desc.colorCount as usize {
        let Some((view, texture)) = colors[index] else {
            continue;
        };
        let attachment = GL_COLOR_ATTACHMENT0.wrapping_add(index as GLenum);
        if texture.m_glRenderbuffer != 0 {
            submit(
                context,
                GLCommand::FramebufferRenderbuffer {
                    target: GL_FRAMEBUFFER,
                    attachment,
                    renderbuffer_target: GL_RENDERBUFFER,
                    renderbuffer: texture.m_glRenderbuffer,
                },
            );
        } else if texture.r#type() == TextureType::cube {
            submit(
                context,
                GLCommand::FramebufferTexture2D {
                    target: GL_FRAMEBUFFER,
                    attachment,
                    texture_target: GL_TEXTURE_CUBE_MAP_POSITIVE_X.wrapping_add(view.baseLayer()),
                    texture: texture.m_glTexture,
                    level: view.baseMipLevel() as GLint,
                },
            );
        } else if matches!(
            texture.r#type(),
            TextureType::array2D | TextureType::texture3D
        ) {
            submit(
                context,
                GLCommand::FramebufferTextureLayer {
                    target: GL_FRAMEBUFFER,
                    attachment,
                    texture: texture.m_glTexture,
                    level: view.baseMipLevel() as GLint,
                    layer: view.baseLayer() as GLint,
                },
            );
        } else {
            submit(
                context,
                GLCommand::FramebufferTexture2D {
                    target: GL_FRAMEBUFFER,
                    attachment,
                    texture_target: GL_TEXTURE_2D,
                    texture: texture.m_glTexture,
                    level: view.baseMipLevel() as GLint,
                },
            );
        }

        if let Some((_, resolveTexture)) = resolves[index] {
            let resolveIndex = state.m_glResolveCount as usize;
            state.m_glResolves[resolveIndex] = GLResolveEntry {
                colorIndex: index as u32,
                resolveTarget: resolveTexture.m_glTarget,
                resolveTex: resolveTexture.m_glTexture,
                width: texture.width(),
                height: texture.height(),
            };
            state.m_glResolveCount += 1;
        }
        drawBuffers[index] = attachment;
    }
    if desc.colorCount > 0 {
        submit(context, GLCommand::DrawBuffers(drawBuffers));
    }

    if let Some((view, texture)) = depth {
        let hasStencil = matches!(
            texture.format(),
            TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
        );
        let attachment = if hasStencil {
            GL_DEPTH_STENCIL_ATTACHMENT
        } else {
            GL_DEPTH_ATTACHMENT
        };
        if texture.m_glRenderbuffer != 0 {
            submit(
                context,
                GLCommand::FramebufferRenderbuffer {
                    target: GL_FRAMEBUFFER,
                    attachment,
                    renderbuffer_target: GL_RENDERBUFFER,
                    renderbuffer: texture.m_glRenderbuffer,
                },
            );
        } else {
            // Preserve the source's unconditional GL_TEXTURE_2D depth target.
            submit(
                context,
                GLCommand::FramebufferTexture2D {
                    target: GL_FRAMEBUFFER,
                    attachment,
                    texture_target: GL_TEXTURE_2D,
                    texture: texture.m_glTexture,
                    level: view.baseMipLevel() as GLint,
                },
            );
        }
    }

    #[cfg(debug_assertions)]
    {
        let status = context
            .executionDomain()
            .checkFramebufferStatus(GL_FRAMEBUFFER);
        debug_assert_eq!(status, GL_FRAMEBUFFER_COMPLETE, "Ore GL FBO incomplete");
    }

    for index in 0..desc.colorCount as usize {
        let attachment = &desc.colorAttachments[index];
        if attachment.loadOp == LoadOp::clear {
            submit(
                context,
                GLCommand::ClearBufferFloat {
                    buffer: GL_COLOR,
                    drawbuffer: index as GLint,
                    values: [
                        attachment.clearColor.r,
                        attachment.clearColor.g,
                        attachment.clearColor.b,
                        attachment.clearColor.a,
                    ],
                    value_count: 4,
                },
            );
        }
    }

    if let Some((_, texture)) = depth {
        let hasStencil = matches!(
            texture.format(),
            TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
        );
        if desc.depthStencil.depthLoadOp == LoadOp::clear
            && hasStencil
            && desc.depthStencil.stencilLoadOp == LoadOp::clear
        {
            submit(context, GLCommand::DepthMask(true));
            submit(context, GLCommand::StencilMask(0xff));
            submit(
                context,
                GLCommand::ClearBufferDepthStencil {
                    buffer: GL_DEPTH_STENCIL,
                    drawbuffer: 0,
                    depth: desc.depthStencil.depthClearValue,
                    stencil: desc.depthStencil.stencilClearValue as GLint,
                },
            );
        } else {
            if desc.depthStencil.depthLoadOp == LoadOp::clear {
                submit(context, GLCommand::DepthMask(true));
                submit(
                    context,
                    GLCommand::ClearBufferFloat {
                        buffer: GL_DEPTH,
                        drawbuffer: 0,
                        values: [desc.depthStencil.depthClearValue, 0.0, 0.0, 0.0],
                        value_count: 1,
                    },
                );
            }
            if hasStencil && desc.depthStencil.stencilLoadOp == LoadOp::clear {
                submit(context, GLCommand::StencilMask(0xff));
                submit(
                    context,
                    GLCommand::ClearBufferInt {
                        buffer: GL_STENCIL,
                        drawbuffer: 0,
                        values: [desc.depthStencil.stencilClearValue as GLint, 0, 0, 0],
                        value_count: 1,
                    },
                );
            }
        }
    }

    state.m_glVAO = generateName(context, GLObjectKind::VertexArray);
    state.m_ownsVAO = true;
    submit(context, GLCommand::BindVertexArray(state.m_glVAO));

    let (defaultWidth, defaultHeight) = colors[0]
        .map(|(_, texture)| (texture.width(), texture.height()))
        .or_else(|| depth.map(|(_, texture)| (texture.width(), texture.height())))
        .unwrap_or((0, 0));
    if defaultWidth > 0 && defaultHeight > 0 {
        submit(
            context,
            GLCommand::Viewport(0, 0, defaultWidth as GLsizei, defaultHeight as GLsizei),
        );
        state.m_viewportWidth = defaultWidth;
        state.m_viewportHeight = defaultHeight;
    }

    drop(state);
    context.base.setActiveRenderPass(Some(&pass));
    Some(Box::new(pass))
}

unsafe fn wrapCanvasTextureCurrent(
    context: &mut ContextGL,
    canvas: *mut c_void,
) -> Option<AnyResourceHandle> {
    debug_assert!(!canvas.is_null());
    let canvas = unsafe { canvas.cast::<RenderCanvas>().as_mut() }?;
    let target = unsafe { canvas.renderTarget().as_ref() }?;
    let execution = context.executionStamp();
    if !target.belongs_to_owner_thread_execution(execution.domain().key(), execution.generation()) {
        reject(
            context,
            "wrapCanvasTexture: render target is non-GL, stale, or foreign",
        );
        return None;
    }
    // The scalar base identity above proves this is a RenderTargetGL owner
    // before any derived field is read. Its source LiteRTTI tag then proves
    // the narrower texture-target kind before the final concrete cast.
    let targetGL = unsafe { &*(target as *const _ as *const RenderTargetGL) };
    if targetGL.liteTypeID() != TEXTURE_RENDER_TARGET_GL_LITE_RTTI_TYPE_ID {
        reject(
            context,
            "wrapCanvasTexture: GL render target is not texture-backed",
        );
        return None;
    }
    let target = unsafe { &*(targetGL as *const _ as *const TextureRenderTargetGL) };
    target.assertSameExecution(execution);
    let textureID = target.externalTextureID();
    debug_assert_ne!(textureID, 0);
    if textureID == 0 {
        return None;
    }

    let desc = TextureDesc {
        width: canvas.width(),
        height: canvas.height(),
        format: TextureFormat::rgba8unorm,
        r#type: TextureType::texture2D,
        renderTarget: true,
        numMipmaps: 1,
        sampleCount: 1,
        ..TextureDesc::default()
    };
    let mut texture = TextureGL::new(&desc, context.executionStamp().clone());
    texture.m_glTexture = textureID;
    texture.m_glTarget = GL_TEXTURE_2D;
    texture.m_glOwnsTexture = false;
    let domain = nuxie_ore_metal::context_backend_domain(&context.base);
    let textureOwner = ResourceHandle::new_texture_in_domain(None, domain.clone(), texture).erase();
    let viewDesc = TextureViewDesc {
        texture: Some(&textureOwner),
        dimension: TextureViewDimension::texture2D,
        aspect: TextureAspect::all,
        baseMipLevel: 0,
        mipCount: 1,
        baseLayer: 0,
        layerCount: 1,
    };
    let view = TextureViewGL::new(
        textureOwner.clone(),
        &viewDesc,
        context.executionStamp().clone(),
    );
    Some(ResourceHandle::new_in_domain(None, domain, view).erase())
}

unsafe fn wrapRiveTextureCurrent(
    context: &mut ContextGL,
    gpuTexture: *mut c_void,
    width: u32,
    height: u32,
) -> Option<AnyResourceHandle> {
    let gpuTexture = unsafe { gpuTexture.cast::<RiveTexture>().as_ref() }?;
    let execution = context.executionStamp();
    if !gpuTexture
        .belongs_to_owner_thread_execution(execution.domain().key(), execution.generation())
    {
        reject(
            context,
            "wrapRiveTexture: texture belongs to a stale or foreign GL execution",
        );
        return None;
    }
    let textureID = gpuTexture.nativeHandle() as usize as GLuint;
    if textureID == 0 {
        return None;
    }

    let desc = TextureDesc {
        width,
        height,
        format: TextureFormat::rgba8unorm,
        r#type: TextureType::texture2D,
        renderTarget: false,
        numMipmaps: 1,
        sampleCount: 1,
        ..TextureDesc::default()
    };
    let mut texture = TextureGL::new(&desc, context.executionStamp().clone());
    texture.m_glTexture = textureID;
    texture.m_glTarget = GL_TEXTURE_2D;
    texture.m_glOwnsTexture = false;
    let domain = nuxie_ore_metal::context_backend_domain(&context.base);
    let textureOwner = ResourceHandle::new_texture_in_domain(None, domain.clone(), texture).erase();
    let viewDesc = TextureViewDesc {
        texture: Some(&textureOwner),
        dimension: TextureViewDimension::texture2D,
        aspect: TextureAspect::all,
        baseMipLevel: 0,
        mipCount: 1,
        baseLayer: 0,
        layerCount: 1,
    };
    let view = TextureViewGL::new(
        textureOwner.clone(),
        &viewDesc,
        context.executionStamp().clone(),
    );
    Some(ResourceHandle::new_in_domain(None, domain, view).erase())
}

pub(crate) fn makeBuffer(
    context: &mut ContextGL,
    desc: &BufferDesc<'_>,
) -> Option<AnyResourceHandle> {
    withCurrentContext(context, |context| makeBufferCurrent(context, desc))
}

pub(crate) fn makeTexture(
    context: &mut ContextGL,
    desc: &TextureDesc<'_>,
) -> Option<AnyResourceHandle> {
    withCurrentContext(context, |context| makeTextureCurrent(context, desc))
}

pub(crate) fn makeTextureView(
    context: &mut ContextGL,
    desc: &TextureViewDesc<'_>,
) -> Option<AnyResourceHandle> {
    withCurrentContext(context, |context| makeTextureViewCurrent(context, desc))
}

pub(crate) fn makeSampler(
    context: &mut ContextGL,
    desc: &SamplerDesc<'_>,
) -> Option<AnyResourceHandle> {
    withCurrentContext(context, |context| makeSamplerCurrent(context, desc))
}

pub(crate) fn makeShaderModule(
    context: &mut ContextGL,
    desc: &ShaderModuleDesc<'_>,
) -> Option<AnyResourceHandle> {
    withCurrentContext(context, |context| makeShaderModuleCurrent(context, desc))
}

pub(crate) fn makeBindGroupLayout(
    context: &mut ContextGL,
    desc: &BindGroupLayoutDesc<'_>,
) -> Option<AnyResourceHandle> {
    withCurrentContext(context, |context| makeBindGroupLayoutCurrent(context, desc))
}

pub(crate) fn makePipeline(
    context: &mut ContextGL,
    desc: &PipelineDesc<'_>,
    outError: Option<&mut String>,
) -> Option<AnyResourceHandle> {
    withCurrentContext(context, |context| {
        makePipelineCurrent(context, desc, outError)
    })
}

pub(crate) fn makeBindGroup(
    context: &mut ContextGL,
    desc: &BindGroupDesc<'_>,
) -> Option<AnyResourceHandle> {
    withCurrentContext(context, |context| makeBindGroupCurrent(context, desc))
}

pub(crate) fn beginRenderPass(
    context: &mut ContextGL,
    desc: &RenderPassDesc<'_>,
    outError: Option<&mut String>,
) -> Option<Box<dyn RenderPassApi>> {
    withCurrentContext(context, |context| {
        beginRenderPassCurrent(context, desc, outError)
    })
}

pub(crate) fn beginFrame(context: &mut ContextGL, descriptor: &FrameDescriptor) {
    withCurrentContext(context, |context| beginFrameCurrent(context, descriptor));
}

pub(crate) fn endFrame(context: &mut ContextGL) {
    withCurrentContext(context, endFrameCurrent);
}

pub(crate) fn waitForGPU(context: &mut ContextGL) {
    withCurrentContext(context, waitForGPUCurrent);
}

pub(crate) unsafe fn wrapCanvasTexture(
    context: &mut ContextGL,
    canvas: *mut c_void,
) -> Option<AnyResourceHandle> {
    withCurrentContext(context, |context| unsafe {
        wrapCanvasTextureCurrent(context, canvas)
    })
}

pub(crate) unsafe fn wrapRiveTexture(
    context: &mut ContextGL,
    texture: *mut c_void,
    width: u32,
    height: u32,
) -> Option<AnyResourceHandle> {
    withCurrentContext(context, |context| unsafe {
        wrapRiveTextureCurrent(context, texture, width, height)
    })
}

impl ContextApi for ContextGL {
    fn features(&self) -> Features {
        withCurrentContextRef(self, |context| context.base.features())
    }

    fn lastError(&self) -> String {
        withCurrentContextRef(self, |context| context.base.lastError())
    }

    fn activeRenderPass(&self) -> Option<RcWeak<dyn ActiveRenderPass>> {
        withCurrentContextRef(self, |context| context.base.activeRenderPass())
    }

    fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>) {
        withCurrentContextRef(self, |context| context.base.setActiveRenderPass(pass));
    }

    fn finishActiveRenderPass(&self) {
        withCurrentContextRef(self, |context| context.base.finishActiveRenderPass());
    }

    fn clearLastError(&self) {
        withCurrentContextRef(self, |context| context.base.clearLastError());
    }

    fn setLastError(&self, message: &str) {
        withCurrentContextRef(self, |context| context.base.setLastError(message));
    }

    fn makeBuffer(&mut self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        makeBuffer(self, desc)
    }

    fn makeTexture(&mut self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        makeTexture(self, desc)
    }

    fn makeTextureView(&mut self, desc: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        makeTextureView(self, desc)
    }

    fn makeSampler(&mut self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        makeSampler(self, desc)
    }

    fn makeShaderModule(&mut self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        makeShaderModule(self, desc)
    }

    fn makeBindGroupLayout(&mut self, desc: &BindGroupLayoutDesc<'_>) -> Option<AnyResourceHandle> {
        makeBindGroupLayout(self, desc)
    }

    fn makePipeline(
        &mut self,
        desc: &PipelineDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        makePipeline(self, desc, outError)
    }

    fn makeBindGroup(&mut self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        makeBindGroup(self, desc)
    }

    fn beginRenderPass(
        &mut self,
        desc: &RenderPassDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        beginRenderPass(self, desc, outError)
    }

    fn beginFrame(&mut self, descriptor: &FrameDescriptor) {
        beginFrame(self, descriptor);
    }

    fn endFrame(&mut self) {
        endFrame(self);
    }

    fn waitForGPU(&mut self) {
        waitForGPU(self);
    }

    unsafe fn wrapCanvasTexture(&mut self, canvas: *mut c_void) -> Option<AnyResourceHandle> {
        unsafe { wrapCanvasTexture(self, canvas) }
    }

    unsafe fn wrapRiveTexture(
        &mut self,
        texture: *mut c_void,
        width: u32,
        height: u32,
    ) -> Option<AnyResourceHandle> {
        unsafe { wrapRiveTexture(self, texture, width, height) }
    }

    fn shaderTarget(&self) -> ShaderTarget {
        ContextGL::shaderTarget(self)
    }
}

pub(crate) const SOURCE_STATIC_HELPER_COUNT: usize = 8;
pub(crate) const SOURCE_CONTEXT_METHOD_DEFINITION_COUNT: usize = 16;
pub(crate) const SOURCE_FEATURE_BOOLEAN_ASSIGNMENT_COUNT: usize = 14;
const _: [(); 44512] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::include::rive::refcnt_hpp::{
        make_rcp, rcp, static_rcp_cast,
    };
    use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
    use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
    use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImage;
    use crate::mechanical_port::webgl2::render_context_gl_decl::TextureGLImpl;
    use std::cell::RefCell;
    use std::collections::{HashMap, VecDeque};
    use std::rc::Rc;

    #[derive(Default)]
    struct FakeProviderState {
        commands: Vec<GLCommand>,
        generated: Vec<(GLObjectKind, GLuint)>,
        queries: Vec<(GLenum, GLint)>,
        names: VecDeque<GLuint>,
        integers: HashMap<GLenum, GLint>,
        lifecycleIngress: Option<GLContextLifecycleIngress>,
        finalReleaseIngress: Option<GLFinalReleaseIngress>,
    }

    struct FakeProvider(Rc<RefCell<FakeProviderState>>);

    impl GLExecutionProvider for FakeProvider {
        fn installContextLifecycleIngress(&mut self, ingress: GLContextLifecycleIngress) {
            let previous = self.0.borrow_mut().lifecycleIngress.replace(ingress);
            assert!(previous.is_none(), "provider accepts one lifecycle ingress");
        }

        fn installFinalReleaseIngress(
            &mut self,
            ingress: GLFinalReleaseIngress,
        ) -> std::sync::Arc<dyn nuxie_ore_metal::gpu_resource::ResourceFinalReleaseWake> {
            let previous = self.0.borrow_mut().finalReleaseIngress.replace(ingress);
            assert!(
                previous.is_none(),
                "provider accepts one final-release ingress"
            );
            std::sync::Arc::new(TestFinalReleaseWake::default())
        }

        fn submit(&mut self, command: GLCommand) {
            self.0.borrow_mut().commands.push(command);
        }

        fn generateObject(&mut self, kind: GLObjectKind) -> GLuint {
            let mut state = self.0.borrow_mut();
            let name = state
                .names
                .pop_front()
                .expect("fake provider has an exact real GLuint result");
            state.generated.push((kind, name));
            name
        }

        fn createProgram(&mut self) -> GLuint {
            self.generateObject(GLObjectKind::Program)
        }

        fn createShader(&mut self, _shaderType: GLenum) -> GLuint {
            let mut state = self.0.borrow_mut();
            let name = state
                .names
                .pop_front()
                .expect("fake provider has an exact real shader result");
            state.generated.push((GLObjectKind::Program, name));
            name
        }

        fn getInteger(&mut self, parameter: GLenum) -> GLint {
            let mut state = self.0.borrow_mut();
            let value = state.integers.get(&parameter).copied().unwrap_or_default();
            state.queries.push((parameter, value));
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
            GL_TRUE as GLint
        }

        fn shaderInfoLog(&mut self, _shader: GLuint, _maxLength: usize) -> Vec<u8> {
            Vec::new()
        }

        fn programParameter(&mut self, _program: GLuint, _parameter: GLenum) -> GLint {
            GL_TRUE as GLint
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

        fn contextLost(&mut self, _nextGeneration: u64) {}
    }

    fn execution(
        names: impl IntoIterator<Item = GLuint>,
    ) -> (GLExecutionDomain, Rc<RefCell<FakeProviderState>>) {
        let state = Rc::new(RefCell::new(FakeProviderState {
            names: names.into_iter().collect(),
            ..FakeProviderState::default()
        }));
        let domain = GLExecutionDomain::new(Box::new(FakeProvider(Rc::clone(&state))));
        (domain, state)
    }

    fn context(domain: &GLExecutionDomain) -> Box<ContextGL> {
        ContextGL::Make(domain.stamp()).expect("fake WebGL2 context is constructible")
    }

    fn clearTrace(state: &Rc<RefCell<FakeProviderState>>) {
        let mut state = state.borrow_mut();
        state.commands.clear();
        state.generated.clear();
        state.queries.clear();
    }

    #[test]
    fn complete_source_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 1221);
        assert_eq!(SOURCE_STATIC_HELPER_COUNT, 8);
        assert_eq!(SOURCE_CONTEXT_METHOD_DEFINITION_COUNT, 16);
        assert_eq!(SOURCE_FEATURE_BOOLEAN_ASSIGNMENT_COUNT, 14);
    }

    #[test]
    fn frozen_webgl_format_and_state_helpers_match_source() {
        assert_eq!(oreFormatToGLInternal(TextureFormat::rgba8unorm), GL_RGBA8);
        assert_eq!(
            oreTextureTypeToGLTarget(TextureType::array2D),
            GL_TEXTURE_2D_ARRAY
        );
        assert_eq!(
            oreMipmapFilterToGL(Filter::linear, Filter::nearest),
            GL_LINEAR_MIPMAP_NEAREST
        );
        assert_eq!(oreWrapToGL(WrapMode::mirrorRepeat), GL_MIRRORED_REPEAT);
        assert_eq!(
            oreCompareFunctionToGL(CompareFunction::lessEqual),
            GL_LEQUAL
        );
    }

    #[test]
    fn exact_nonsequential_buffer_name_and_real_ebo_binding_are_restored() {
        let (domain, state) = execution([701]);
        let mut context = context(&domain);
        clearTrace(&state);
        state
            .borrow_mut()
            .integers
            .insert(GL_ELEMENT_ARRAY_BUFFER_BINDING, 911);

        let desc = BufferDesc::uninitialized(BufferUsage::index, 8);
        let bufferOwner = makeBuffer(&mut context, &desc).expect("index buffer");
        let buffer = bufferOwner
            .downcast_ref::<BufferGL>()
            .expect("GL buffer payload");
        assert_eq!(buffer.m_glBuffer, 701);
        assert_eq!(state.borrow().generated, vec![(GLObjectKind::Buffer, 701)]);
        assert_eq!(
            state.borrow().commands,
            vec![
                GLCommand::BindBuffer(GL_ELEMENT_ARRAY_BUFFER, 701),
                GLCommand::BufferData {
                    target: GL_ELEMENT_ARRAY_BUFFER,
                    size: 8,
                    data: None,
                    usage: GL_DYNAMIC_DRAW,
                },
                GLCommand::BindBuffer(GL_ELEMENT_ARRAY_BUFFER, 911),
            ]
        );

        clearTrace(&state);
        state
            .borrow_mut()
            .integers
            .insert(GL_ELEMENT_ARRAY_BUFFER_BINDING, 1903);
        bufferOwner.update(&[1, 2, 3, 4], 4, 2).unwrap();
        assert_eq!(
            state.borrow().queries,
            vec![(GL_ELEMENT_ARRAY_BUFFER_BINDING, 1903)]
        );
        assert_eq!(
            state.borrow().commands,
            vec![
                GLCommand::BindBuffer(GL_ELEMENT_ARRAY_BUFFER, 701),
                GLCommand::BufferSubData {
                    target: GL_ELEMENT_ARRAY_BUFFER,
                    offset: 2,
                    data: vec![1, 2, 3, 4],
                },
                GLCommand::BindBuffer(GL_ELEMENT_ARRAY_BUFFER, 1903),
            ]
        );

        drop(bufferOwner);
        domain.withCurrent(|| {});
        assert!(state
            .borrow()
            .commands
            .contains(&GLCommand::DeleteBuffer(701)));
        drop(context);
        domain.shutdown();
    }

    #[test]
    fn texture_upload_restores_real_unpack_row_and_image_state() {
        let (domain, state) = execution([509]);
        let mut context = context(&domain);
        let desc = TextureDesc {
            width: 2,
            height: 2,
            ..TextureDesc::default()
        };
        let textureOwner = makeTexture(&mut context, &desc).expect("texture");
        assert_eq!(
            textureOwner
                .downcast_ref::<TextureGL>()
                .expect("GL texture")
                .m_glTexture,
            509
        );

        clearTrace(&state);
        {
            let mut state = state.borrow_mut();
            state.integers.insert(GL_UNPACK_ROW_LENGTH, 37);
            state.integers.insert(GL_UNPACK_IMAGE_HEIGHT, 41);
        }
        textureOwner
            .upload(&nuxie_ore_metal::types::TextureDataDesc {
                data: Some(&[0; 16]),
                bytesPerRow: 8,
                rowsPerImage: 2,
                width: 2,
                height: 2,
                ..nuxie_ore_metal::types::TextureDataDesc::default()
            })
            .unwrap();
        assert_eq!(
            state.borrow().queries,
            vec![(GL_UNPACK_ROW_LENGTH, 37), (GL_UNPACK_IMAGE_HEIGHT, 41)]
        );
        let commands = state.borrow().commands.clone();
        assert!(commands.contains(&GLCommand::PixelStore(GL_UNPACK_ROW_LENGTH, 2)));
        assert!(commands.contains(&GLCommand::PixelStore(GL_UNPACK_IMAGE_HEIGHT, 2)));
        assert!(commands.contains(&GLCommand::PixelStore(GL_UNPACK_ROW_LENGTH, 37)));
        assert!(commands.contains(&GLCommand::PixelStore(GL_UNPACK_IMAGE_HEIGHT, 41)));
        assert_eq!(
            &commands[commands.len() - 3..],
            &[
                GLCommand::PixelStore(GL_UNPACK_ROW_LENGTH, 37),
                GLCommand::PixelStore(GL_UNPACK_IMAGE_HEIGHT, 41),
                GLCommand::BindTexture(GL_TEXTURE_2D, 0),
            ]
        );

        drop(textureOwner);
        domain.withCurrent(|| {});
        drop(context);
        domain.shutdown();
    }

    #[test]
    fn foreign_domain_resource_and_stale_context_generation_are_rejected() {
        let (firstDomain, firstState) = execution([101]);
        let (secondDomain, _) = execution([]);
        let mut first = context(&firstDomain);
        let mut second = context(&secondDomain);
        let texture = makeTexture(
            &mut first,
            &TextureDesc {
                width: 1,
                height: 1,
                ..TextureDesc::default()
            },
        )
        .expect("first-domain texture");
        let viewDesc = TextureViewDesc {
            texture: Some(&texture),
            ..TextureViewDesc::default()
        };
        assert!(makeTextureView(&mut second, &viewDesc).is_none());
        assert!(second
            .lastError()
            .contains("different GL execution generation"));

        let lifecycle = firstState
            .borrow()
            .lifecycleIngress
            .clone()
            .expect("provider retained the mandatory lifecycle ingress");
        assert!(lifecycle.contextLost());
        assert_eq!(
            lifecycle.contextRestored(),
            Some(GLContextRecovery::RecreateRenderer)
        );
        assert!(
            !firstDomain.isLive(),
            "the lost renderer remains terminal and must be recreated"
        );

        drop(texture);
        drop(first);
        firstDomain.shutdown();
        drop(second);
        secondDomain.shutdown();
    }

    #[test]
    fn stale_generation_suppresses_numeric_gl_delete_but_tears_down_payload() {
        let (domain, state) = execution([31337]);
        let mut context = context(&domain);
        let buffer = makeBuffer(
            &mut context,
            &BufferDesc::uninitialized(BufferUsage::vertex, 4),
        )
        .expect("buffer");
        clearTrace(&state);

        let lifecycle = state
            .borrow()
            .lifecycleIngress
            .clone()
            .expect("provider retained the mandatory lifecycle ingress");
        assert!(lifecycle.contextLost());
        drop(buffer);
        drop(context);
        domain.shutdown();
        assert!(!state
            .borrow()
            .commands
            .contains(&GLCommand::DeleteBuffer(31337)));
    }

    #[test]
    fn gl_bind_group_preserves_authored_group_index_through_erased_layout_owner() {
        let (domain, _) = execution([]);
        let mut context = context(&domain);
        let layoutOwner = makeBindGroupLayout(
            &mut context,
            &BindGroupLayoutDesc {
                groupIndex: 2,
                ..BindGroupLayoutDesc::default()
            },
        )
        .expect("group-two GL layout");
        assert_eq!(
            layoutOwner
                .bindGroupLayoutBase()
                .expect("erased GL layout projects its offset-zero base")
                .groupIndex(),
            2
        );

        let groupOwner = makeBindGroup(
            &mut context,
            &BindGroupDesc {
                layout: Some(&layoutOwner),
                ..BindGroupDesc::default()
            },
        )
        .expect("empty group-two GL bind group");
        let group = groupOwner
            .downcast_ref::<BindGroupGL>()
            .expect("GL bind group payload");
        assert_eq!(group.groupIndex(), 2);
        assert_eq!(
            group
                .layout()
                .expect("bind group retains its authored layout")
                .bindGroupLayoutBase()
                .expect("retained concrete GL layout projects its offset-zero base")
                .groupIndex(),
            2
        );

        drop(groupOwner);
        drop(layoutOwner);
        domain.withCurrent(|| {});
        drop(context);
        domain.shutdown();
    }

    /// The product can transfer erased source owners through worker-owned
    /// graphs whose only worker operation is the atomic final `unref`. Keep
    /// that deliberately narrower contract local to this regression instead
    /// of marking the thread-affine concrete GL texture itself `Send`.
    struct WorkerLastTextureRelease(rcp<RiveTexture>);

    unsafe impl Send for WorkerLastTextureRelease {}

    #[test]
    fn erased_rive_texture_last_release_returns_concrete_gl_delete_to_owner() {
        let (domain, state) = execution([]);
        let concrete = make_rcp(|| TextureGLImpl::new(4, 4, 4242, domain.stamp()));
        let erased: rcp<RiveTexture> = unsafe { static_rcp_cast(concrete) };
        let finalReleaseIngress = state
            .borrow()
            .finalReleaseIngress
            .clone()
            .expect("provider retained the mandatory final-release ingress");
        clearTrace(&state);

        domain.retireRenderer();
        assert!(domain.isRendererRetired());
        assert!(
            domain.isLive(),
            "normal renderer retirement preserves the texture's GL generation"
        );

        let workerRelease = WorkerLastTextureRelease(erased);
        std::thread::spawn(move || drop(workerRelease))
            .join()
            .expect("worker final release completes without touching GL");
        assert!(
            state.borrow().commands.is_empty(),
            "the worker may only enqueue the erased texture's final release"
        );

        assert!(
            finalReleaseIngress.drainFinalReleases(),
            "the posted owner-loop task reaches the retired live domain"
        );
        assert_eq!(
            state.borrow().commands,
            vec![GLCommand::DeleteTexture(4242)]
        );
        domain.shutdown();
    }

    #[test]
    fn erased_rive_texture_last_release_after_context_loss_suppresses_stale_delete() {
        let (domain, state) = execution([]);
        let concrete = make_rcp(|| TextureGLImpl::new(4, 4, 4343, domain.stamp()));
        let erased: rcp<RiveTexture> = unsafe { static_rcp_cast(concrete) };
        let lifecycleIngress = state
            .borrow()
            .lifecycleIngress
            .clone()
            .expect("provider retained the mandatory lifecycle ingress");
        let finalReleaseIngress = state
            .borrow()
            .finalReleaseIngress
            .clone()
            .expect("provider retained the mandatory final-release ingress");
        clearTrace(&state);

        assert!(lifecycleIngress.contextLost());
        domain.retireRenderer();
        assert!(domain.isRendererRetired());
        assert!(
            !domain.isLive(),
            "actual context loss invalidates the texture's GL generation"
        );

        let workerRelease = WorkerLastTextureRelease(erased);
        std::thread::spawn(move || drop(workerRelease))
            .join()
            .expect("worker stale final release completes without touching GL");
        assert!(state.borrow().commands.is_empty());

        assert!(
            finalReleaseIngress.drainFinalReleases(),
            "the posted owner-loop task still tears down the stale payload"
        );
        assert!(
            state.borrow().commands.is_empty(),
            "stale TextureGLImpl destruction must suppress DeleteTexture(4343)"
        );
        domain.shutdown();
    }

    #[test]
    fn short_compressed_upload_rejects_before_any_provider_mutation() {
        let (domain, state) = execution([603]);
        let mut context = context(&domain);
        let textureOwner = makeTexture(
            &mut context,
            &TextureDesc {
                width: 4,
                height: 4,
                format: TextureFormat::etc2rgb8,
                ..TextureDesc::default()
            },
        )
        .expect("ETC2 texture");
        clearTrace(&state);

        let result = textureOwner.upload(&nuxie_ore_metal::types::TextureDataDesc {
            data: Some(&[0; 15]),
            bytesPerRow: 8,
            rowsPerImage: 2,
            width: 4,
            height: 4,
            ..nuxie_ore_metal::types::TextureDataDesc::default()
        });
        assert_eq!(
            result,
            Err(nuxie_ore_metal::texture::TextureUploadError::DataTooShort {
                required: 16,
                actual: 15,
            })
        );
        let trace = state.borrow();
        assert!(trace.commands.is_empty());
        assert!(trace.generated.is_empty());
        assert!(trace.queries.is_empty());
        drop(trace);

        drop(textureOwner);
        domain.withCurrent(|| {});
        drop(context);
        domain.shutdown();
    }

    #[test]
    fn render_pass_outliving_context_finishes_once_and_restores_captured_names() {
        let (domain, state) = execution([5011, 6022]);
        let mut context = context(&domain);
        {
            let mut state = state.borrow_mut();
            state.integers.insert(GL_VERTEX_ARRAY_BINDING, 73);
            state.integers.insert(GL_FRAMEBUFFER_BINDING, 89);
        }
        let pass = beginRenderPass(&mut context, &RenderPassDesc::default(), None)
            .expect("attachment-free GL pass");
        clearTrace(&state);

        drop(context);
        assert!(state.borrow().commands.is_empty());
        drop(pass);
        assert_eq!(
            state.borrow().commands,
            vec![
                GLCommand::Disable(GL_SCISSOR_TEST),
                GLCommand::Disable(GL_BLEND),
                GLCommand::Disable(GL_DEPTH_TEST),
                GLCommand::Disable(GL_STENCIL_TEST),
                GLCommand::Disable(GL_CULL_FACE),
                GLCommand::Disable(GL_POLYGON_OFFSET_FILL),
                GLCommand::DepthMask(true),
                GLCommand::ColorMask(true, true, true, true),
                GLCommand::BindBuffer(GL_ARRAY_BUFFER, 0),
                GLCommand::DeleteVertexArray(6022),
                GLCommand::BindVertexArray(73),
                GLCommand::DeleteFramebuffer(5011),
                GLCommand::BindFramebuffer(GL_FRAMEBUFFER, 89),
            ]
        );

        domain.shutdown();
    }

    #[test]
    fn wrap_canvas_texture_rejects_non_gl_target_before_concrete_cast() {
        let (domain, state) = execution([]);
        let mut context = context(&domain);
        let texture = make_rcp(|| RiveTexture::new(4, 4));
        let image = make_rcp(|| unsafe { RiveRenderImage::new(texture) });
        let target = make_rcp(|| RenderTarget::new(4, 4));
        let canvas = make_rcp(|| unsafe { RenderCanvas::new(image, target) });
        clearTrace(&state);

        let wrapped =
            unsafe { wrapCanvasTexture(&mut context, canvas.get().cast::<std::ffi::c_void>()) };
        assert!(wrapped.is_none());
        assert_eq!(
            context.lastError(),
            "wrapCanvasTexture: render target is non-GL, stale, or foreign"
        );
        let trace = state.borrow();
        assert!(trace.commands.is_empty());
        assert!(trace.generated.is_empty());
        assert!(trace.queries.is_empty());
        drop(trace);

        drop(canvas);
        drop(context);
        domain.shutdown();
    }
}
