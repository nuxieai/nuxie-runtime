//! Metal branch of tests/gm/ore_gm_helper.hpp, plus the owned Rust GM host.
use crate::deferred::cmd::{deferred_replayer::DeferredFrameSink, render_replay::RendererOwner};
use crate::{
    native_metal::{
        NativeMetalContextOptions, NativeMetalFactory, NativeMetalFrame, ShaderCompilationMode,
    },
    RenderMode,
};
use nuxie_ore_metal::{
    binding_map::{ResourceKind, TextureSampleType, TextureViewDim},
    context::FrameDescriptor,
};
pub(super) use nuxie_ore_metal::{
    context::ContextApi, gpu_resource::AnyResourceHandle, render_pass::RenderPassApi, types::*,
};
pub(super) use nuxie_render_api::*;
use sha2::{Digest, Sha256};
pub(super) use std::{cell::RefCell, rc::Rc};

pub(super) const SIZE: u32 = 256;
pub(super) fn fixture(relative: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(relative);
    std::fs::read(&path)
        .unwrap_or_else(|e| panic!("{}: {e}; run tools/fetch-test-assets.sh", path.display()))
}
pub(super) fn assert_pixels_equal(a: &[u8], b: &[u8]) {
    assert_eq!(a.len(), SIZE as usize * SIZE as usize * 4);
    assert_eq!(b.len(), a.len());
    // Upstream gmmain.cpp: parityMaxChannelDiff = atomic ? 8 : 0.
    // This host explicitly selects RasterOrdering, never atomic or fallback.
    assert_eq!(
        a.iter().zip(b).position(|(a, b)| a != b),
        None,
        "non-atomic GM parity differs"
    );
}

enum Frame {
    Screen(NativeMetalFrame),
    Canvas(Box<dyn RenderCanvasFrame>),
}
impl Frame {
    fn renderer(&mut self) -> &mut dyn Renderer {
        match self {
            Self::Screen(f) => f,
            Self::Canvas(f) => f.renderer(),
        }
    }
}
struct FrameRenderer(Rc<RefCell<Option<Frame>>>);
impl Renderer for FrameRenderer {
    fn save(&mut self) {
        self.0.borrow_mut().as_mut().unwrap().renderer().save();
    }
    fn restore(&mut self) {
        self.0.borrow_mut().as_mut().unwrap().renderer().restore();
    }
    fn transform(&mut self, m: Mat2D) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .renderer()
            .transform(m);
    }
    fn draw_path(&mut self, p: &dyn RenderPath, paint: &dyn RenderPaint) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .renderer()
            .draw_path(p, paint);
    }
    fn clip_path(&mut self, p: &dyn RenderPath) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .renderer()
            .clip_path(p);
    }
    fn draw_image(&mut self, i: Option<&dyn RenderImage>, s: ImageSampler, b: BlendMode, o: f32) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .renderer()
            .draw_image(i, s, b, o);
    }
    fn draw_image_mesh(
        &mut self,
        i: Option<&dyn RenderImage>,
        s: ImageSampler,
        v: Option<&dyn RenderBuffer>,
        u: Option<&dyn RenderBuffer>,
        idx: Option<&dyn RenderBuffer>,
        vc: u32,
        ic: u32,
        b: BlendMode,
        o: f32,
    ) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .renderer()
            .draw_image_mesh(i, s, v, u, idx, vc, ic, b, o);
    }
    fn modulate_opacity(&mut self, o: f32) {
        self.0
            .borrow_mut()
            .as_mut()
            .unwrap()
            .renderer()
            .modulate_opacity(o);
    }
}

pub(super) struct GmHost {
    pub factory: PersistentFactory<NativeMetalFactory>,
    pub ore: OreContextHandle,
    screen: Rc<RefCell<Option<Frame>>>,
    canvas: Rc<RefCell<Option<Frame>>>,
    clear: u32,
    screen_initialized: bool,
}
impl GmHost {
    pub fn new(clear: u32) -> Self {
        Self::with_screen(clear, true)
    }
    pub fn with_screen(clear: u32, open: bool) -> Self {
        let mut native_factory = NativeMetalFactory::new_with_mode_and_context_options(
            SIZE,
            SIZE,
            RenderMode::RasterOrdering,
            NativeMetalContextOptions {
                // gmmain.cpp and goldens.cpp choose this before the Metal
                // window is created, so paired captures use the same shaders.
                shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
                ..NativeMetalContextOptions::default()
            },
        )
        .expect("live Metal GM factory");
        // Upstream gmmain.cpp and goldens.cpp disable time-budgeted
        // triangulation for every comparison frame.
        native_factory.use_deterministic_validation_thresholds();
        let mut factory = PersistentFactory::new(native_factory);
        let ore = factory.ore().expect("live Metal ORE context");
        let frame = open.then(|| {
            Frame::Screen(
                factory
                    .borrow()
                    .begin_frame(clear)
                    .expect("GM screen frame"),
            )
        });
        Self {
            factory,
            ore,
            screen: Rc::new(RefCell::new(frame)),
            canvas: Rc::new(RefCell::new(None)),
            clear,
            screen_initialized: open,
        }
    }
    pub fn screen(&self) -> RendererOwner {
        Rc::new(RefCell::new(Box::new(FrameRenderer(self.screen.clone()))))
    }
    pub fn canvas(&mut self, w: u32, h: u32) -> RenderCanvasHandle {
        Rc::new(RefCell::new(
            self.factory.make_render_canvas(w, h).expect("GM canvas"),
        ))
    }
    // testing_window_metal_texture.mm::beginOreFrame uses beginFrame({});
    // Metal allocates its own command buffer on the shared renderer queue.
    pub fn begin_ore(&self) {
        self.ore
            .borrow_mut()
            .beginFrame(&FrameDescriptor::new(0, 0));
    }
    pub fn end_ore(&self) {
        self.ore.borrow_mut().endFrame();
    }
    pub fn finish(mut self) -> Vec<u8> {
        self.finish_frame()
    }
    pub fn finish_frame(&mut self) -> Vec<u8> {
        let frame = self
            .screen
            .borrow_mut()
            .take()
            .expect("GM screen was drawn");
        self.screen_initialized = false;
        match frame {
            Frame::Screen(frame) => frame.finish().expect("GM Metal readback"),
            Frame::Canvas(_) => unreachable!(),
        }
    }

    fn flush_screen(&mut self) {
        let frame = self.screen.borrow_mut().take();
        if let Some(Frame::Screen(frame)) = frame {
            frame.finish_without_readback().expect("GM screen flush");
        }
    }
}
impl DeferredFrameSink for GmHost {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.factory.persistent_context().unwrap()
    }
    fn begin_screen_frame(&mut self, target: u64) -> Option<RendererOwner> {
        assert_eq!(target, 0);
        // DagGMSink flushes any open main bracket and resumes that same
        // target with preserve. Only the first bracket performs its clear.
        self.flush_screen();
        let frame = if self.screen_initialized {
            self.factory.borrow().begin_frame_preserving()
        } else {
            self.factory.borrow().begin_frame(self.clear)
        };
        *self.screen.borrow_mut() = Some(Frame::Screen(frame.expect("GM screen frame")));
        self.screen_initialized = true;
        Some(self.screen())
    }
    fn begin_ore_frame(&mut self) {
        self.begin_ore();
    }
    fn end_ore_frame(&mut self) {
        self.end_ore();
    }
    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        clear: u32,
    ) -> Option<RendererOwner> {
        self.flush_screen();
        assert!(self.canvas.borrow().is_none());
        *self.canvas.borrow_mut() = Some(Frame::Canvas(
            canvas
                .borrow_mut()
                .begin_frame(clear)
                .expect("GM canvas frame"),
        ));
        Some(Rc::new(RefCell::new(Box::new(FrameRenderer(
            self.canvas.clone(),
        )))))
    }
    fn end_canvas_content(&mut self) {
        if let Some(Frame::Canvas(frame)) = self.canvas.borrow_mut().take() {
            frame.finish().expect("GM canvas flush");
        }
    }
}

#[test]
fn screen_canvas_screen_brackets_preserve_main_pixels() {
    let render = |interrupt: bool| {
        let mut host = GmHost::new(0xff203040);
        let mut raw = RawPath::new();
        raw.add_rect(Aabb::new(16.0, 24.0, 64.0, 80.0));
        let path = host.factory.make_render_path(raw, FillRule::NonZero);
        let mut paint = host.factory.make_render_paint();
        paint.color(0xffc04020);
        host.screen()
            .borrow_mut()
            .draw_path(path.as_ref(), paint.as_ref());
        if interrupt {
            let canvas = host.canvas(32, 32);
            host.begin_canvas_content(canvas, 0xff00ff00).unwrap();
            host.end_canvas_content();
            host.begin_screen_frame(0).unwrap();
        }
        host.finish()
    };
    assert_pixels_equal(&render(false), &render(true));
}

pub(super) fn wrap_canvas(
    ctx: &mut dyn ContextApi,
    canvas: &RenderCanvasHandle,
) -> AnyResourceHandle {
    // The retained typed canvas owns both native pointers throughout wrapping.
    unsafe { ctx.wrapCanvasTextureInfo(nuxie_render_api::canvas_texture_info(canvas)) }
        .expect("GM target view")
}
pub(super) fn target_format(view: &AnyResourceHandle) -> TextureFormat {
    view.textureViewBase().unwrap().texture().format().unwrap()
}
pub(super) fn triangle_bytes() -> Vec<u8> {
    [
        [0.0f32, 0.6, 1.0, 0.2, 0.2, 1.0],
        [-0.6, -0.6, 0.2, 1.0, 0.2, 1.0],
        [0.6, -0.6, 0.2, 0.2, 1.0, 1.0],
    ]
    .into_iter()
    .flatten()
    .flat_map(f32::to_ne_bytes)
    .collect()
}
pub(super) fn vertex_buffer(ctx: &mut dyn ContextApi, label: &str) -> AnyResourceHandle {
    let bytes = triangle_bytes();
    ctx.makeBuffer(&BufferDesc {
        usage: BufferUsage::vertex,
        size: bytes.len() as u32,
        data: Some(&bytes),
        immutable: false,
        label: Some(label),
    })
    .expect("GM vertex buffer")
}
pub(super) fn shader(ctx: &mut dyn ContextApi, id: u32) -> AnyResourceHandle {
    use nuxie_runtime::source::{assets::shader_asset::ShaderAsset, factory::RuntimeFactoryHandle};
    let header = fixture("gm/ore_gm_shaders.rstb.hpp");
    assert_eq!(
        format!("{:x}", Sha256::digest(&header)),
        "dda092b3d96973c4d924064e774bcb3428dcf31bfb96d5238aad19c771e7d9da"
    );
    let header = String::from_utf8(header).unwrap();
    let data = header
        .split("kShaderData")
        .nth(1)
        .unwrap()
        .split_once('{')
        .unwrap()
        .1
        .split_once('}')
        .unwrap()
        .0;
    let bytes: Vec<u8> = data
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| u8::from_str_radix(s.trim_start_matches("0x"), 16).unwrap())
        .collect();
    const OFFSETS: [usize; 14] = [
        0, 5034, 10027, 17395, 29294, 37099, 43680, 49294, 55222, 60791, 68716, 77034, 82520, 90955,
    ];
    let mut envelope = vec![0];
    envelope.extend_from_slice(&bytes[OFFSETS[id as usize]..OFFSETS[id as usize + 1]]);
    let mut null = PersistentFactory::new(NullFactory);
    let mut asset = ShaderAsset::default();
    assert!(asset.decode(
        &envelope,
        &RuntimeFactoryHandle::from_factory(&mut null).unwrap()
    ));
    let blob = asset.find_shader(2);
    let binding_map = asset.find_shader(10);
    assert!(!blob.is_empty() && !binding_map.is_empty());
    let mut views = Vec::new();
    let mut code = None;
    let mut size = 0;
    assert!(
        nuxie_ore_metal::rstb_entry_container::parseWholeModuleContainer(
            Some(blob),
            blob.len() as u32,
            &mut views,
            Some(&mut code),
            Some(&mut size)
        )
    );
    let pair_bytes = asset
        .texture_sampler_pairs()
        .iter()
        .flat_map(|pair| {
            [
                pair.tex_group,
                pair.tex_binding,
                pair.samp_group,
                pair.samp_binding,
            ]
        })
        .collect::<Vec<_>>();
    ctx.makeShaderModule(&ShaderModuleDesc {
        code,
        codeSize: size,
        bindingMapBytes: Some(binding_map),
        bindingMapSize: binding_map.len() as u32,
        texSamplerPairBytes: (!pair_bytes.is_empty()).then_some(pair_bytes.as_slice()),
        texSamplerPairSize: pair_bytes.len() as u32,
        shaderAssetId: 0x80000000 + id,
        ..Default::default()
    })
    .unwrap_or_else(|| panic!("GM shader: {}", ctx.lastError()))
}
pub(super) fn triangle_pipeline(
    ctx: &mut dyn ContextApi,
    module: &AnyResourceHandle,
    format: TextureFormat,
    label: &str,
) -> AnyResourceHandle {
    let attrs = [
        VertexAttribute {
            offset: 0,
            shaderSlot: 0,
            format: VertexFormat::float2,
            ..Default::default()
        },
        VertexAttribute {
            offset: 8,
            shaderSlot: 1,
            format: VertexFormat::float4,
            ..Default::default()
        },
    ];
    let layouts = [VertexBufferLayout {
        stride: 24,
        stepMode: VertexStepMode::vertex,
        attributes: Some(&attrs),
        attributeCount: 2,
    }];
    let mut desc = PipelineDesc {
        vertexModule: Some(module),
        fragmentModule: Some(module),
        vertexBuffers: Some(&layouts),
        vertexBufferCount: 1,
        label: Some(label),
        ..Default::default()
    };
    desc.colorTargets[0].format = format;
    ctx.makePipeline(&desc, None)
        .unwrap_or_else(|| panic!("GM pipeline: {}", ctx.lastError()))
}
pub(super) fn pass_desc<'a>(
    view: &'a AnyResourceHandle,
    label: Option<&'a str>,
    color: [f32; 4],
) -> RenderPassDesc<'a> {
    let mut desc = RenderPassDesc {
        label,
        ..Default::default()
    };
    desc.colorAttachments[0] = ColorAttachment {
        view: Some(view),
        loadOp: LoadOp::clear,
        storeOp: StoreOp::store,
        clearColor: ClearColor {
            r: color[0],
            g: color[1],
            b: color[2],
            a: color[3],
        },
        ..Default::default()
    };
    desc
}
pub(super) fn triangle_pass(
    pass: &mut dyn RenderPassApi,
    pipeline: &AnyResourceHandle,
    vb: &AnyResourceHandle,
) {
    pass.setPipeline(Some(pipeline));
    pass.setVertexBuffer(0, Some(vb), 0);
    pass.setViewport(0.0, 0.0, 256.0, 256.0, 0.0, 1.0);
    pass.draw(3, 1, 0, 0);
    pass.finish();
}
pub(super) fn draw_canvas(
    renderer: &mut dyn Renderer,
    canvas: &RenderCanvasHandle,
    x: f32,
    y: f32,
    flip: bool,
) {
    renderer.save();
    renderer.translate(x, y);
    if flip {
        renderer.translate(0.0, canvas.borrow().height() as f32);
        renderer.scale(1.0, -1.0);
    }
    renderer.draw_image(
        Some(canvas.borrow().render_image().as_ref()),
        ImageSampler {
            filter: ImageFilter::Nearest,
            ..Default::default()
        },
        BlendMode::SrcOver,
        1.0,
    );
    renderer.restore();
}
// The triangle GMs do not apply even an identity transform around this draw.
pub(super) fn draw_canvas_at_origin(renderer: &mut dyn Renderer, canvas: &RenderCanvasHandle) {
    renderer.save();
    renderer.draw_image(
        Some(canvas.borrow().render_image().as_ref()),
        ImageSampler {
            filter: ImageFilter::Nearest,
            ..Default::default()
        },
        BlendMode::SrcOver,
        1.0,
    );
    renderer.restore();
}
pub(super) fn layout_from_shader(
    ctx: &mut dyn ContextApi,
    shader: &AnyResourceHandle,
    group: u32,
) -> AnyResourceHandle {
    let bm = &shader.shaderModuleBase().unwrap().m_bindingMap;
    let mut entries = Vec::new();
    for i in 0..bm.size() {
        if entries.len() == 16 {
            break;
        }
        let e = bm.at(i);
        if u32::from(e.group) != group {
            continue;
        }
        let kind = match e.kind {
            ResourceKind::StorageBufferRO => BindingKind::storageBufferRO,
            ResourceKind::StorageBufferRW => BindingKind::storageBufferRW,
            ResourceKind::SampledTexture => BindingKind::sampledTexture,
            ResourceKind::StorageTexture => BindingKind::storageTexture,
            ResourceKind::Sampler => BindingKind::sampler,
            ResourceKind::ComparisonSampler => BindingKind::comparisonSampler,
            _ => BindingKind::uniformBuffer,
        };
        entries.push(BindGroupLayoutEntry {
            binding: u32::from(e.binding),
            kind,
            visibility: StageVisibility {
                mask: e.stageMask & 7,
            },
            hasDynamicOffset: false,
            textureViewDim: match e.textureViewDim {
                TextureViewDim::Cube => TextureViewDimension::cube,
                TextureViewDim::CubeArray => TextureViewDimension::cubeArray,
                TextureViewDim::D3 => TextureViewDimension::texture3D,
                TextureViewDim::D2Array => TextureViewDimension::array2D,
                _ => TextureViewDimension::texture2D,
            },
            textureSampleType: match e.textureSampleType {
                TextureSampleType::UnfilterableFloat => SampleType::floatUnfilterable,
                TextureSampleType::Depth => SampleType::depth,
                TextureSampleType::Sint => SampleType::sint,
                TextureSampleType::Uint => SampleType::uint,
                _ => SampleType::floatFilterable,
            },
            textureMultisampled: e.textureMultisampled,
            nativeSlotVS: if e.backendSlot[0] == u16::MAX {
                u32::MAX
            } else {
                u32::from(e.backendSlot[0])
            },
            nativeSlotFS: if e.backendSlot[1] == u16::MAX {
                u32::MAX
            } else {
                u32::from(e.backendSlot[1])
            },
            ..Default::default()
        });
    }
    ctx.makeBindGroupLayout(&BindGroupLayoutDesc {
        groupIndex: group,
        entries: Some(&entries),
        entryCount: entries.len() as u32,
        ..Default::default()
    })
    .expect("GM reflected layout")
}
