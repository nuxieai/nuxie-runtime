#![cfg(all(
    feature = "ore-metal-authored-msl",
    any(target_os = "ios", target_os = "macos")
))]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use nuxie::ore_metal_gpu_canvas::OreMetalGpuCanvas;
use nuxie::render_api::authored_ore_shader::ExactGpuCanvasShaderOccurrence;
use nuxie::render_api::{
    OreContextHandle, RawPath, RenderCanvas, RenderCanvasError, RenderCanvasFrame,
};
use nuxie::{
    ColorInt, Factory, File, FileImportLimits, FillRule, GpuCanvasError, GpuCanvasPipelineShaders,
    GpuCanvasPlan, GpuCanvasShaderArtifact, GpuCanvasShaderProfile, GpuCanvasShaderStage,
    ImageDecodeError, NoopScriptHostExtension, PersistentFactory, RecordingFactory, RenderBuffer,
    RenderBufferFlags, RenderBufferType, RenderGpuCanvasShader, RenderImage, RenderPaint,
    RenderPath, RenderShader, ScriptExecutionCapability, ScriptExecutionLimits, ScriptedFile,
    import_scripted,
};
use nuxie_ore_metal::context::{CanvasTextureInfo, FrameDescriptor, ShaderTarget};
use nuxie_ore_metal::metal::context::{ContextMetal, MetalRenderCanvasBridge};
use nuxie_ore_metal::metal::shader_module::ShaderModuleMetal;
use nuxie_ore_metal::ore_cmd::ore_make_replay::OreResident;
use nuxie_renderer::deferred::ore::ore_deferred_context::DeferredOreContext;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice, MTLLibrary, MTLPixelFormat,
    MTLRegion, MTLStorageMode, MTLTexture, MTLTextureDescriptor, MTLTextureUsage,
};

#[path = "support/authored_msl_gpu_canvas.rs"]
mod fixture;
use fixture::{BINDING_MAP, EXPECTED_PIXEL, HEIGHT, PROBE_MSL, WIDTH, imported_file};

struct OreProbeFactory {
    inner: RecordingFactory,
    recorder: Rc<RefCell<DeferredOreContext>>,
    real: Rc<RefCell<ContextMetal>>,
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    artifacts: Vec<GpuCanvasShaderArtifact>,
    canvases: Vec<MetalCanvasImage>,
}

impl OreProbeFactory {
    fn new() -> Result<Self, GpuCanvasError> {
        let device = MTLCreateSystemDefaultDevice()
            .ok_or_else(|| GpuCanvasError::new("Metal device is unavailable"))?;
        let queue = device
            .newCommandQueue()
            .ok_or_else(|| GpuCanvasError::new("Metal command queue creation failed"))?;
        let real = Rc::new(RefCell::new(
            *ContextMetal::MakeChecked(Some(device.clone()), Some(queue.clone()))
                .expect("paired Metal device and queue"),
        ));
        let recorder = Rc::new(RefCell::new(DeferredOreContext::new(Some(real.clone()))));
        Ok(Self {
            inner: RecordingFactory::new(),
            recorder,
            real,
            device,
            queue,
            artifacts: Vec::new(),
            canvases: Vec::new(),
        })
    }

    fn replay(&self) -> OreResident {
        let mut residents = OreResident::default();
        let completion = {
            let mut real = self.real.borrow_mut();
            real.beginFrame(&FrameDescriptor::new(0, 0));
            // Sessionless DeferredOreContext wraps the real canvas directly;
            // its resource table retains that view until this replay completes.
            self.recorder
                .borrow()
                .replayFrame(&mut *real, &mut residents, &mut |_| {
                    panic!("sessionless replay must not request a canvas ID")
                });
            assert!(real.lastError().is_empty(), "{}", real.lastError());
            real.end_frame_with_completion()
                .expect("submitted ORE frame")
        };
        let deadline = Instant::now() + Duration::from_secs(30);
        while completion.result().is_none() && Instant::now() < deadline {
            std::thread::yield_now();
        }
        completion
            .result()
            .expect("Metal completion before readback deadline")
            .expect("Metal command buffer completes successfully");
        residents
    }
}

#[derive(Clone)]
struct MetalCanvasImage(Rc<MetalRenderCanvasBridge>);

impl MetalCanvasImage {
    fn pixels(&self) -> Vec<u8> {
        let mut pixels = vec![0; (self.width() * self.height() * 4) as usize];
        // SAFETY: the caller waits for the ORE submission before reading this
        // shared RGBA8 texture; the vector covers the complete region and pitch.
        unsafe {
            self.0
                .texture
                .as_ref()
                .expect("retained canvas texture")
                .getBytes_bytesPerRow_fromRegion_mipmapLevel(
                    std::ptr::NonNull::new(pixels.as_mut_ptr().cast()).unwrap(),
                    self.width() as usize * 4,
                    MTLRegion {
                        origin: objc2_metal::MTLOrigin { x: 0, y: 0, z: 0 },
                        size: objc2_metal::MTLSize {
                            width: self.width() as usize,
                            height: self.height() as usize,
                            depth: 1,
                        },
                    },
                    0,
                );
        }
        pixels
    }
}

impl RenderImage for MetalCanvasImage {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn retain_image(&self) -> Rc<dyn RenderImage> {
        Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
    fn width(&self) -> u32 {
        self.0.width
    }
    fn height(&self) -> u32 {
        self.0.height
    }
    fn ore_texture_info(&self) -> Option<CanvasTextureInfo> {
        Some(CanvasTextureInfo {
            canvas: Rc::as_ptr(&self.0).cast_mut().cast(),
            // Only the canvas color-view projection is used by this fixture.
            texture: std::ptr::null_mut(),
            width: self.width(),
            height: self.height(),
            owner: Some(self.0.clone()),
        })
    }
}

struct MetalCanvas(MetalCanvasImage);
impl RenderCanvas for MetalCanvas {
    fn width(&self) -> u32 {
        self.0.width()
    }
    fn height(&self) -> u32 {
        self.0.height()
    }
    fn render_image(&self) -> Rc<dyn RenderImage> {
        self.0.retain_image()
    }
    fn begin_frame(
        &mut self,
        _: ColorInt,
    ) -> Result<Box<dyn RenderCanvasFrame>, RenderCanvasError> {
        panic!("this GPUCanvas fixture never begins a 2D canvas frame")
    }
}

impl Factory for OreProbeFactory {
    fn is_render_context(&self) -> bool {
        true
    }

    fn ore(&mut self) -> Option<OreContextHandle> {
        Some(self.recorder.clone())
    }

    fn make_render_canvas(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Box<dyn RenderCanvas>, RenderCanvasError> {
        let descriptor = MTLTextureDescriptor::new();
        descriptor.setPixelFormat(MTLPixelFormat::RGBA8Unorm);
        descriptor.setUsage(MTLTextureUsage::RenderTarget | MTLTextureUsage::ShaderRead);
        descriptor.setStorageMode(MTLStorageMode::Shared);
        unsafe {
            descriptor.setWidth(width as usize);
            descriptor.setHeight(height as usize);
            descriptor.setMipmapLevelCount(1);
        }
        let texture = self
            .device
            .newTextureWithDescriptor(&descriptor)
            .ok_or_else(|| RenderCanvasError::new("Metal canvas allocation failed"))?;
        let image = MetalCanvasImage(Rc::new(MetalRenderCanvasBridge {
            width,
            height,
            texture: Some(texture),
        }));
        self.canvases.push(image.clone());
        Ok(Box::new(MetalCanvas(image)))
    }

    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        self.inner
            .make_render_buffer(buffer_type, flags, size_in_bytes)
    }

    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.inner
            .make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.inner
            .make_radial_gradient(cx, cy, radius, colors, stops)
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        self.inner.make_render_path(raw_path, fill_rule)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.inner.make_empty_render_path()
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.inner.make_render_paint()
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.inner.decode_image(data)
    }

    fn gpu_canvas_shader_profile(&self) -> GpuCanvasShaderProfile {
        GpuCanvasShaderProfile::TrustedAppleMetal
    }

    fn make_gpu_canvas_shader_artifact(
        &mut self,
        shader: &GpuCanvasShaderArtifact,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        let GpuCanvasShaderArtifact::TrustedAppleMetal(shader_bytes) = shader else {
            panic!("the ORE product probe must never request a WebGPU artifact");
        };
        assert_eq!(shader_bytes.source(), PROBE_MSL);
        assert_eq!(shader_bytes.binding_map_bytes(), BINDING_MAP);
        assert_eq!(shader_bytes.entries().len(), 2);
        assert_eq!(shader_bytes.bindings().len(), 2);
        self.artifacts.push(shader.clone());
        let module = ExactGpuCanvasShaderOccurrence::compile(
            &mut *self.recorder.borrow_mut(),
            self.gpu_canvas_shader_profile(),
            shader,
            self.recorder.clone(),
        )?;
        // lua_gpu.cpp::buildShaderEntries uses one whole MSL module for both
        // entries, created on the selected recording context, not the driver.
        assert_eq!(module.modules.len(), 1);
        assert!(
            module
                .ore_shader_entry(GpuCanvasShaderStage::Vertex, "vertex_main")
                .unwrap()
                .ptr_eq(
                    &module
                        .ore_shader_entry(GpuCanvasShaderStage::Fragment, "fragment_main")
                        .unwrap()
                )
        );
        assert!(
            module.modules[0]
                .downcast_ref::<ShaderModuleMetal>()
                .is_none()
        );
        Ok(Arc::new(module))
    }

    fn make_gpu_canvas_shader_occurrence(
        &mut self,
        _: &Arc<dyn RenderGpuCanvasShader>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        panic!("synchronous native import does not use browser async preparation")
    }

    fn make_gpu_canvas_image_with_pipelines(
        &mut self,
        _: &[GpuCanvasPipelineShaders],
        _: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        panic!("imported GPUCanvas records ORE commands, never a host plan")
    }
}

type ScriptLogs = Arc<Mutex<Vec<String>>>;

fn import_fixture(
    native_shaders_are_authorized: bool,
) -> (ScriptedFile, PersistentFactory<OreProbeFactory>, ScriptLogs) {
    let bytes = imported_file();
    let mut factory = PersistentFactory::new(OreProbeFactory::new().expect("live Metal adapter"));
    let logs = Arc::new(Mutex::new(Vec::new()));
    let captured_logs = logs.clone();
    // SAFETY: this test module is the trusted exporter boundary for these
    // fixed, reviewed MSL bytes. It binds the exact target-2 source, target-10
    // map, supplemental reflection, script, and enclosing Rive bytes. The
    // comparison capability deliberately authorizes only script execution.
    let capability = unsafe {
        if native_shaders_are_authorized {
            ScriptExecutionCapability::for_verified_native_shader_artifact_unchecked(
                &bytes,
                Arc::new(NoopScriptHostExtension),
            )
        } else {
            ScriptExecutionCapability::for_verified_artifact_unchecked(
                &bytes,
                Arc::new(NoopScriptHostExtension),
            )
        }
    }
    .expect("exact-artifact capability");
    let file = import_scripted(
        &bytes,
        &mut factory,
        None,
        FileImportLimits::new(),
        capability,
        ScriptExecutionLimits::new(),
        Some(Arc::new(move |_, line| {
            captured_logs
                .lock()
                .unwrap()
                .push(String::from_utf8_lossy(line).into_owned());
        })),
    )
    .expect("authenticated fixture imports through the native File lifecycle");
    let selected = file.vm().ore_context().expect("selected ORE context");
    assert!(Rc::ptr_eq(&selected, &factory.ore().unwrap()));
    assert!(selected.borrow().isRecording());
    assert_eq!(selected.borrow().shaderTarget(), ShaderTarget::msl);
    (file, factory, logs)
}

#[test]
fn authenticated_rive_shader_records_and_replays_into_ore_metal_pixels() {
    let (file, factory, logs) = import_fixture(true);
    let instance = file
        .native_file()
        .with_file(File::artboard_default)
        .expect("fixture artboard");
    instance.advance_default(0.0);
    let mut renderer = factory.borrow().inner.make_renderer();
    instance.draw(&mut renderer);
    assert!(
        logs.lock().unwrap().is_empty(),
        "{:?}",
        logs.lock().unwrap()
    );
    let factory = factory.borrow();
    assert!(
        !factory.artifacts.is_empty(),
        "the imported shader was selected"
    );
    assert!(factory.recorder.borrow().streamBytes().commands > 0);
    let residents = factory.replay();
    let libraries = residents
        .objects
        .iter()
        .flatten()
        .filter_map(|resource| {
            resource.downcast_ref::<ShaderModuleMetal>().map(|module| {
                let library = module
                    .mtlLibrary()
                    .expect("replayed MSL compiled to a native library");
                assert_eq!(
                    Retained::as_ptr(&library.device()),
                    Retained::as_ptr(&factory.device),
                    "replay must use the selected real device"
                );
                library
            })
        })
        .count();
    assert!(
        libraries > 0,
        "the imported recording must materialize a native shader"
    );
    // The File's source artboard and its instance can each initialize their
    // own canvas. Only the instance's canvas is drawn; do not infer a retired
    // prepare/occurrence count from that native lifecycle.
    let pixels = factory.canvases.last().expect("instance canvas").pixels();
    assert_eq!(pixels.len(), usize::try_from(WIDTH * HEIGHT * 4).unwrap());
    assert!(
        pixels.chunks_exact(4).all(|pixel| pixel == EXPECTED_PIXEL),
        "every pixel must be the exact sum of the two authenticated UBOs"
    );
    assert_eq!(factory.inner.stream().matches("drawImage ").count(), 1);
}

#[test]
fn generic_script_trust_never_reaches_native_shader_compilation() {
    let (file, factory, logs) = import_fixture(false);
    let instance = file
        .native_file()
        .with_file(File::artboard_default)
        .expect("fixture artboard");
    instance.advance_default(0.0);
    let mut renderer = factory.borrow().inner.make_renderer();
    instance.draw(&mut renderer);
    let logs = logs.lock().unwrap();
    assert!(
        logs.iter()
            .any(|line| line.contains("scene requires native shader authority")),
        "generic trust must leave context:shader nil: {logs:?}"
    );
    let factory = factory.borrow();
    assert!(factory.artifacts.is_empty());
    assert!(factory.canvases.is_empty());
    assert_eq!(factory.recorder.borrow().streamBytes().commands, 0);
    assert!(!factory.inner.stream().contains("drawImage "));
}

#[test]
fn direct_host_utility_occurrences_still_own_distinct_native_libraries() {
    let (file, factory, logs) = import_fixture(true);
    let instance = file
        .native_file()
        .with_file(File::artboard_default)
        .expect("fixture artboard");
    instance.advance_default(0.0);
    assert!(
        logs.lock().unwrap().is_empty(),
        "{:?}",
        logs.lock().unwrap()
    );
    let factory = factory.borrow();
    let artifact = factory
        .artifacts
        .first()
        .expect("authenticated target-2 artifact");
    // This utility remains an explicit direct-host API, not the imported
    // GPUCanvas execution path. Preserve its own occurrence identity contract.
    let mut utility =
        OreMetalGpuCanvas::from_device_queue(factory.device.clone(), factory.queue.clone());
    let prepared = utility
        .make_shader_artifact(artifact)
        .expect("direct-host MSL module");
    let occurrence = utility
        .make_shader_occurrence(&prepared)
        .expect("direct-host occurrence");
    assert_ne!(
        utility.shader_module_identity(&prepared).unwrap(),
        utility.shader_module_identity(&occurrence).unwrap(),
        "explicit direct-host occurrences own distinct MTLLibrary objects"
    );
}
