//! 5ab9af03 ore_scratch_pass_objects_test.cpp's two live GL scenarios.
//! The .mm file only includes that C++ test for Apple's Objective-C headers.
//! Browser adaptation: read the render textures directly, without the unrelated
//! window/canvas srcOver composite. Pixel tests use the real WebGL2 provider.

use super::{
    browser_provider::BrowserWebGl2Provider,
    gles3_decl::*,
    ore_context_gl_decl::ContextGL,
    ore_context_gl_impl as ore,
    ore_texture_gl_decl::{TextureGL, TextureViewGL},
};
use nuxie_ore_metal::{
    binding_map::BindingMap, gpu_resource::AnyResourceHandle, render_pass::RenderPassApi, types::*,
};
use web_sys::HtmlCanvasElement;

fn texture(ctx: &mut ContextGL, size: u32, format: TextureFormat) -> AnyResourceHandle {
    let texture = ore::makeTexture(
        ctx,
        &TextureDesc {
            width: size,
            height: size,
            format,
            renderTarget: true,
            ..TextureDesc::default()
        },
    )
    .expect("render texture");
    ore::makeTextureView(
        ctx,
        &TextureViewDesc {
            texture: Some(&texture),
            ..TextureViewDesc::default()
        },
    )
    .expect("render texture view")
}

fn desc(view: &AnyResourceHandle, color: [f32; 4]) -> RenderPassDesc<'_> {
    let mut desc = RenderPassDesc::default();
    desc.colorAttachments[0].view = Some(view);
    desc.colorAttachments[0].clearColor = ClearColor {
        r: color[0],
        g: color[1],
        b: color[2],
        a: color[3],
    };
    desc
}

fn pipeline(ctx: &mut ContextGL) -> AnyResourceHandle {
    // Equivalent to ore_gm::kTriangle: position + interpolated RGBA, no bindings.
    let vertex = b"#version 300 es\nlayout(location=0) in vec2 position; layout(location=1) in vec4 color; out vec4 v_color; void main(){gl_Position=vec4(position,0.0,1.0);v_color=color;}";
    let fragment = b"#version 300 es\nprecision highp float; in vec4 v_color; layout(location=0) out vec4 color; void main(){color=v_color;}";
    let map = BindingMap::default().toBlob();
    let mut module = |code: &[u8], stage| {
        ore::makeShaderModule(
            ctx,
            &ShaderModuleDesc {
                code: Some(code),
                codeSize: code.len() as u32,
                stage,
                bindingMapBytes: Some(&map),
                bindingMapSize: map.len() as u32,
                ..ShaderModuleDesc::default()
            },
        )
        .expect("solid triangle shader compiles")
    };
    let vs = module(vertex, ShaderStage::vertex);
    let fs = module(fragment, ShaderStage::fragment);
    let attrs = [
        VertexAttribute {
            offset: 0,
            shaderSlot: 0,
            format: VertexFormat::float2,
            pad: [0; 3],
        },
        VertexAttribute {
            offset: 8,
            shaderSlot: 1,
            format: VertexFormat::float4,
            pad: [0; 3],
        },
    ];
    let buffers = [VertexBufferLayout {
        stride: 24,
        attributes: Some(&attrs),
        attributeCount: 2,
        ..VertexBufferLayout::default()
    }];
    ore::makePipeline(
        ctx,
        &PipelineDesc {
            vertexModule: Some(&vs),
            fragmentModule: Some(&fs),
            vertexBuffers: Some(&buffers),
            vertexBufferCount: 1,
            colorTargets: [ColorTargetState {
                format: TextureFormat::rgba8unorm,
                ..ColorTargetState::default()
            }; 4],
            ..PipelineDesc::default()
        },
        None,
    )
    .expect("solid triangle pipeline links")
}

fn triangle(ctx: &mut ContextGL, color: [f32; 3]) -> AnyResourceHandle {
    let mut bytes = Vec::new();
    for [x, y] in [[-1.0f32, -1.0], [3.0, -1.0], [-1.0, 3.0]] {
        for value in [x, y, color[0], color[1], color[2], 1.0] {
            bytes.extend_from_slice(&value.to_ne_bytes());
        }
    }
    ore::makeBuffer(
        ctx,
        &BufferDesc {
            usage: BufferUsage::vertex,
            size: bytes.len() as u32,
            data: Some(&bytes),
            immutable: false,
            label: None,
        },
    )
    .expect("triangle vertex buffer")
}

fn draw(pass: &mut dyn RenderPassApi, pipeline: &AnyResourceHandle, vbo: &AnyResourceHandle) {
    pass.setPipeline(Some(pipeline));
    pass.setVertexBuffer(0, Some(vbo), 0);
    pass.setViewport(0.0, 0.0, 64.0, 64.0, 0.0, 1.0);
    pass.draw(3, 1, 0, 0);
}

fn read(domain: &GLExecutionDomain, view: &AnyResourceHandle, size: u32) -> Vec<u8> {
    let view = view.downcast_ref::<TextureViewGL>().expect("GL view");
    let texture = view
        .texture()
        .downcast_ref::<TextureGL>()
        .expect("GL texture");
    domain.withCurrent(|| {
        let previous = domain.getInteger(GL_FRAMEBUFFER_BINDING) as u32;
        let fbo = domain.generateObject(GLObjectKind::Framebuffer);
        recordGLCommand(GLCommand::BindFramebuffer(GL_FRAMEBUFFER, fbo));
        recordGLCommand(GLCommand::FramebufferTexture2D {
            target: GL_FRAMEBUFFER,
            attachment: GL_COLOR_ATTACHMENT0,
            texture_target: GL_TEXTURE_2D,
            texture: texture.m_glTexture,
            level: 0,
        });
        assert_eq!(
            domain.checkFramebufferStatus(GL_FRAMEBUFFER),
            GL_FRAMEBUFFER_COMPLETE
        );
        recordGLCommand(GLCommand::ReadBuffer(GL_COLOR_ATTACHMENT0));
        let pixels = domain.readPixelsRGBA8(0, 0, size, size);
        recordGLCommand(GLCommand::DeleteFramebuffer(fbo));
        recordGLCommand(GLCommand::BindFramebuffer(GL_FRAMEBUFFER, previous));
        pixels
    })
}

fn solid(pixels: &[u8], channels: [bool; 3]) {
    assert_eq!(pixels.len(), 64 * 64 * 4);
    for y in 1..63 {
        for x in 1..63 {
            for channel in 0..3 {
                let value = pixels[(y * 64 + x) * 4 + channel];
                assert!(
                    if channels[channel] {
                        value >= 0xf0
                    } else {
                        value <= 0x10
                    },
                    "pixel {x},{y} channel{channel}={value}"
                );
            }
        }
    }
}

pub fn run_scratch_pass_browser_tests(canvas: HtmlCanvasElement) -> Result<String, String> {
    let (provider, adapter, _) = BrowserWebGl2Provider::new(canvas, 64, 64)
        .map_err(|error| format!("WebGL2 unavailable: {error:?}"))?;
    let domain = GLExecutionDomain::new(Box::new(provider));
    let mut ctx = ContextGL::Make(domain.stamp(), std::ptr::null_mut())
        .ok_or("ORE GL context unavailable")?;
    {
        let pipeline = pipeline(&mut ctx);
        let green = triangle(&mut ctx, [0.0, 1.0, 0.0]);
        let magenta = triangle(&mut ctx, [1.0, 0.0, 1.0]);
        let b = texture(&mut ctx, 64, TextureFormat::rgba8unorm);
        let a0 = texture(&mut ctx, 32, TextureFormat::rgba8unorm);
        let a1 = texture(&mut ctx, 32, TextureFormat::rgba8unorm);
        let depth = texture(&mut ctx, 32, TextureFormat::depth32float);
        ore::beginFrame(
            &mut ctx,
            &nuxie_ore_metal::context::FrameDescriptor::new(0, 1),
        );
        ore::beginRenderPass(&mut ctx, &desc(&b, [1.0, 0.0, 0.0, 1.0]), None)
            .unwrap()
            .finish();
        let mut a = desc(&a0, [0.0, 0.0, 1.0, 1.0]);
        a.colorCount = 2;
        a.colorAttachments[1].view = Some(&a1);
        a.colorAttachments[1].clearColor = ClearColor {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        };
        a.depthStencil.view = Some(&depth);
        a.depthStencil.depthStoreOp = StoreOp::store;
        ore::beginRenderPass(&mut ctx, &a, None).unwrap().finish();
        let mut pass_b =
            ore::beginRenderPass(&mut ctx, &desc(&b, [0.0, 0.0, 1.0, 1.0]), None).unwrap();
        draw(&mut *pass_b, &pipeline, &green);
        pass_b.finish();
        ore::endFrame(&mut ctx);
        solid(&read(&domain, &b, 64), [false, true, false]);
        let pixels = read(&domain, &a1, 32);
        assert_eq!(pixels.len(), 32 * 32 * 4);
        let yellow = pixels
            .chunks_exact(4)
            .filter(|p| p[0] >= 0xf0 && p[1] >= 0xf0 && p[2] <= 0x10)
            .count();
        let green_count = pixels
            .chunks_exact(4)
            .filter(|p| p[0] <= 0x10 && p[1] >= 0xf0 && p[2] <= 0x10)
            .count();
        assert_eq!(green_count, 0);
        assert!(yellow >= 32 * 32 / 4);

        let outer = texture(&mut ctx, 64, TextureFormat::rgba8unorm);
        let inner = texture(&mut ctx, 64, TextureFormat::rgba8unorm);
        ore::beginFrame(
            &mut ctx,
            &nuxie_ore_metal::context::FrameDescriptor::new(1, 2),
        );
        let mut outer_pass =
            ore::beginRenderPass(&mut ctx, &desc(&outer, [0.0, 0.0, 1.0, 1.0]), None).unwrap();
        let mut inner_pass =
            ore::beginRenderPass(&mut ctx, &desc(&inner, [0.0, 0.0, 1.0, 1.0]), None).unwrap();
        draw(&mut *inner_pass, &pipeline, &magenta);
        inner_pass.finish();
        draw(&mut *outer_pass, &pipeline, &green);
        outer_pass.finish();
        ore::endFrame(&mut ctx);
        solid(&read(&domain, &outer, 64), [false, true, false]);
        solid(&read(&domain, &inner, 64), [true, false, true]);
    }
    domain.withCurrent(|| {});
    drop(ctx);
    domain.shutdown();
    Ok(format!(
        "5ab9 scratch scrub and overlap pixel assertions passed: {adapter}"
    ))
}
