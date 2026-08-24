//! Narrow authenticated authored-MSL adapter over the concrete ORE Metal port.
//!
//! This first executable seam intentionally supports one graphics module, one
//! pipeline, one pass, one non-indexed draw, and uniform buffers only. Every
//! unsupported family fails before command-buffer submission.

use std::any::Any;
use std::collections::BTreeMap;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::time::{Duration, Instant};

use nuxie_ore_metal::context::FrameDescriptor;
use nuxie_ore_metal::gpu_resource::AnyResourceHandle;
use nuxie_ore_metal::metal::context::ContextMetal;
use nuxie_ore_metal::metal::shader_module::ShaderModuleMetal;
use nuxie_ore_metal::types::{
    BindGroupDesc, BindGroupLayoutDesc, BindGroupLayoutEntry, BindingKind, BufferDesc, BufferUsage,
    ClearColor, ColorAttachment, ColorTargetState, CullMode, FaceWinding, PipelineDesc,
    PrimitiveTopology, RenderPassDesc, ShaderModuleDesc, StageVisibility, TextureFormat, UBOEntry,
};
use nuxie_render_api::{
    GpuCanvasAppleMetalShader, GpuCanvasAttachmentView, GpuCanvasError, GpuCanvasPipelinePlan,
    GpuCanvasPipelineShaders, GpuCanvasPlan, GpuCanvasShaderArtifact, GpuCanvasShaderProfile,
    GpuCanvasShaderResourceKind, RenderGpuCanvasShader, RenderImage,
};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLCommandQueue, MTLDevice, MTLPixelFormat, MTLRegion, MTLStorageMode, MTLTexture,
    MTLTextureDescriptor, MTLTextureUsage,
};

fn unsupported(message: impl Into<String>) -> GpuCanvasError {
    GpuCanvasError::new(format!("ORE Metal authored-MSL tracer: {}", message.into()))
}

/// Recording-thread-confined ORE service used by an explicit native Factory.
pub struct OreMetalGpuCanvas {
    context: ContextMetal,
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    domain: Rc<()>,
}

impl OreMetalGpuCanvas {
    /// Construct the adapter from the concrete product factory's retained
    /// Metal service. The adapter never selects a second device or queue.
    pub fn from_device_queue(
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    ) -> Self {
        Self {
            context: *ContextMetal::MakeChecked(Some(device.clone()), Some(queue))
                .expect("retained Metal device and queue construct ORE"),
            device,
            domain: Rc::new(()),
        }
    }

    pub fn shader_profile(&self) -> GpuCanvasShaderProfile {
        GpuCanvasShaderProfile::TrustedAppleMetal
    }

    pub fn make_shader_artifact(
        &mut self,
        artifact: &GpuCanvasShaderArtifact,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        let GpuCanvasShaderArtifact::TrustedAppleMetal(shader) = artifact else {
            return Err(unsupported("target-0/WebGPU artifacts are not accepted"));
        };
        self.compile_occurrence(shader)
    }

    pub fn make_shader_occurrence(
        &mut self,
        prepared: &Arc<dyn RenderGpuCanvasShader>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        let prepared = prepared
            .as_any()
            .downcast_ref::<OreMetalShaderOccurrence>()
            .ok_or_else(|| unsupported("shader occurrence belongs to another backend"))?;
        let domain = prepared
            .domain
            .upgrade()
            .ok_or_else(|| unsupported("shader occurrence domain has expired"))?;
        if !Rc::ptr_eq(&domain, &self.domain) {
            return Err(unsupported(
                "shader occurrence belongs to another Metal device domain",
            ));
        }
        self.compile_occurrence(&prepared.artifact)
    }

    /// Native library identity used by the product-root occurrence oracle.
    #[doc(hidden)]
    pub fn shader_module_identity(
        &self,
        shader: &Arc<dyn RenderGpuCanvasShader>,
    ) -> Result<usize, GpuCanvasError> {
        let occurrence = self.occurrence(shader)?;
        let module = occurrence
            .module
            .downcast_ref::<ShaderModuleMetal>()
            .ok_or_else(|| unsupported("shader occurrence has no Metal module"))?;
        let library = module
            .mtlLibrary()
            .ok_or_else(|| unsupported("shader occurrence has no Metal library"))?;
        Ok(std::ptr::from_ref(library).cast::<()>() as usize)
    }

    fn compile_occurrence(
        &mut self,
        shader: &GpuCanvasAppleMetalShader,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        let code_size = u32::try_from(shader.source().len())
            .map_err(|_| unsupported("shader source exceeds u32"))?;
        let binding_map_size = u32::try_from(shader.binding_map_bytes().len())
            .map_err(|_| unsupported("binding map exceeds u32"))?;
        let module = self
            .context
            .makeShaderModule(&ShaderModuleDesc {
                code: Some(shader.source().as_bytes()),
                codeSize: code_size,
                bindingMapBytes: Some(shader.binding_map_bytes()),
                bindingMapSize: binding_map_size,
                label: Some("authenticated GPUCanvas MSL"),
                ..ShaderModuleDesc::default()
            })
            .ok_or_else(|| unsupported("authenticated target-2 MSL failed to compile"))?;
        // The public Factory contract uses Arc for backend occurrences. This
        // concrete occurrence remains recording-thread confined by its Rc
        // domain witness and is never presented as Send or Sync.
        #[allow(clippy::arc_with_non_send_sync)]
        let occurrence: Arc<dyn RenderGpuCanvasShader> = Arc::new(OreMetalShaderOccurrence {
            artifact: shader.clone(),
            module,
            domain: Rc::downgrade(&self.domain),
        });
        Ok(occurrence)
    }

    pub fn make_image_with_pipelines(
        &mut self,
        shaders: &[GpuCanvasPipelineShaders],
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        let [shader_pair] = shaders else {
            return Err(unsupported("exactly one pipeline is required"));
        };
        let fragment = shader_pair.fragment.as_ref().unwrap_or(&shader_pair.vertex);
        if !Arc::ptr_eq(&shader_pair.vertex, fragment) {
            return Err(unsupported(
                "separate vertex and fragment modules are deferred",
            ));
        }
        let shader = self.occurrence(&shader_pair.vertex)?;

        let [pipeline_plan] = plan.pipelines.as_slice() else {
            return Err(unsupported(
                "exactly one explicit pipeline snapshot is required",
            ));
        };
        let [render_pass] = plan.render_passes.as_slice() else {
            return Err(unsupported("exactly one explicit render pass is required"));
        };
        let [draw] = render_pass.draws.as_slice() else {
            return Err(unsupported("exactly one draw is required"));
        };
        if draw.pipeline_index != 0 || draw.indexed_draw.is_some() {
            return Err(unsupported(
                "indexed or nonzero-pipeline draws are deferred",
            ));
        }
        let [attachment] = render_pass.color_attachments.as_slice() else {
            return Err(unsupported("exactly one color attachment is required"));
        };
        if !matches!(attachment.view, GpuCanvasAttachmentView::Canvas)
            || attachment.resolve_target.is_some()
            || attachment.load_op != "clear"
            || attachment.store_op != "store"
            || render_pass.depth_stencil_attachment.is_some()
        {
            return Err(unsupported(
                "only clear/store canvas attachments are supported",
            ));
        }
        if plan.width == 0 || plan.height == 0 {
            return Err(unsupported("canvas dimensions must be nonzero"));
        }
        if draw.vertex_count != 3
            || draw.instance_count != 1
            || draw.first_vertex != 0
            || draw.first_instance != 0
        {
            return Err(unsupported(
                "only one fullscreen non-indexed triangle is supported",
            ));
        }
        if draw.pass_state.scissor_rect.is_some()
            || draw.pass_state.stencil_reference != 0
            || draw.pass_state.blend_color != [0.0; 4]
        {
            return Err(unsupported(
                "dynamic scissor/stencil/blend state is deferred",
            ));
        }
        let [clear_r, clear_g, clear_b, clear_a] = attachment.clear_color;
        if ![clear_r, clear_g, clear_b, clear_a]
            .into_iter()
            .all(|channel| channel.is_finite() && (0.0..=1.0).contains(&channel))
        {
            return Err(unsupported(
                "clear color must contain finite normalized channels",
            ));
        }

        validate_pipeline_shape(pipeline_plan)?;
        let layouts = self.make_layouts(&shader.artifact)?;
        let layout_refs = layouts.iter().map(Option::as_ref).collect::<Vec<_>>();
        let vertex_entry = pipeline_plan
            .vertex_entry
            .as_ref()
            .ok_or_else(|| unsupported("vertex entry selection is absent"))?;
        let fragment_entry = pipeline_plan
            .fragment_entry
            .as_ref()
            .ok_or_else(|| unsupported("fragment entry selection is absent"))?;
        require_entry(&shader.artifact, vertex_entry, true)?;
        require_entry(&shader.artifact, fragment_entry, false)?;
        let pipeline = self
            .context
            .makePipeline(
                &PipelineDesc {
                    vertexModule: Some(&shader.module),
                    vertexEntryPoint: Some(&vertex_entry.physical_entry_point),
                    fragmentModule: Some(&shader.module),
                    fragmentEntryPoint: Some(&fragment_entry.physical_entry_point),
                    topology: PrimitiveTopology::triangleList,
                    cullMode: CullMode::none,
                    winding: FaceWinding::counterClockwise,
                    colorTargets: [
                        ColorTargetState {
                            format: TextureFormat::rgba8unorm,
                            ..ColorTargetState::default()
                        },
                        ColorTargetState::default(),
                        ColorTargetState::default(),
                        ColorTargetState::default(),
                    ],
                    bindGroupLayouts: Some(&layout_refs),
                    bindGroupLayoutCount: u32::try_from(layout_refs.len())
                        .map_err(|_| unsupported("binding-group layout count exceeds u32"))?,
                    label: Some("authenticated GPUCanvas pipeline"),
                    ..PipelineDesc::default()
                },
                None,
            )
            .ok_or_else(|| unsupported("ORE pipeline creation failed"))?;
        let groups = self.make_groups(&shader.artifact, pipeline_plan, &layouts)?;

        let descriptor = MTLTextureDescriptor::new();
        descriptor.setPixelFormat(MTLPixelFormat::RGBA8Unorm);
        descriptor.setStorageMode(MTLStorageMode::Shared);
        descriptor.setUsage(MTLTextureUsage::RenderTarget);
        let width = usize::try_from(plan.width)
            .map_err(|_| unsupported("canvas width exceeds NSUInteger"))?;
        let height = usize::try_from(plan.height)
            .map_err(|_| unsupported("canvas height exceeds NSUInteger"))?;
        // SAFETY: validated nonzero u32 extents widen losslessly to NSUInteger.
        unsafe {
            descriptor.setWidth(width);
            descriptor.setHeight(height);
            descriptor.setMipmapLevelCount(1);
        }
        let texture = self
            .device
            .newTextureWithDescriptor(&descriptor)
            .ok_or_else(|| unsupported("render-target allocation failed"))?;
        let view = self
            .context
            .wrap_native_texture(texture.clone(), plan.width, plan.height, true)
            .ok_or_else(|| unsupported("render target belongs to another Metal device"))?;

        let viewport =
            draw.pass_state
                .viewport
                .unwrap_or([0.0, 0.0, plan.width as f32, plan.height as f32]);
        let [viewport_x, viewport_y, viewport_width, viewport_height] = viewport;
        if !viewport.into_iter().all(f32::is_finite)
            || viewport_width <= 0.0
            || viewport_height <= 0.0
        {
            return Err(unsupported(
                "viewport must have finite coordinates and positive dimensions",
            ));
        }

        self.context.beginFrame(&FrameDescriptor::new(0, 0));
        let mut pass = self
            .context
            .beginRenderPass(
                &RenderPassDesc {
                    colorAttachments: [
                        ColorAttachment {
                            view: Some(&view),
                            clearColor: ClearColor {
                                r: clear_r as f32,
                                g: clear_g as f32,
                                b: clear_b as f32,
                                a: clear_a as f32,
                            },
                            ..ColorAttachment::default()
                        },
                        ColorAttachment::default(),
                        ColorAttachment::default(),
                        ColorAttachment::default(),
                    ],
                    label: Some("authenticated GPUCanvas pass"),
                    ..RenderPassDesc::default()
                },
                None,
            )
            .ok_or_else(|| unsupported(self.context.lastError()))?;
        pass.setPipeline(Some(&pipeline));
        for (group_index, group) in groups.iter().enumerate() {
            if let Some(group) = group {
                let group_index = u32::try_from(group_index)
                    .map_err(|_| unsupported("binding-group index exceeds u32"))?;
                pass.setBindGroup(group_index, Some(group), None, 0);
            }
        }
        pass.setViewport(
            viewport_x,
            viewport_y,
            viewport_width,
            viewport_height,
            0.0,
            1.0,
        );
        pass.draw(
            draw.vertex_count,
            draw.instance_count,
            draw.first_vertex,
            draw.first_instance,
        );
        pass.finish();
        let completion = self
            .context
            .end_frame_with_completion()
            .ok_or_else(|| unsupported("Metal frame had no command buffer to submit"))?;

        let deadline = Instant::now()
            .checked_add(Duration::from_secs(5))
            .ok_or_else(|| unsupported("Metal completion deadline overflow"))?;
        while completion.result().is_none() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        completion
            .result()
            .ok_or_else(|| unsupported("Metal completion timed out"))?
            .map_err(unsupported)?;
        let row_bytes = usize::try_from(plan.width)
            .ok()
            .and_then(|width| width.checked_mul(4))
            .ok_or_else(|| unsupported("readback row size overflow"))?;
        let byte_len = row_bytes
            .checked_mul(height)
            .ok_or_else(|| unsupported("readback allocation overflow"))?;
        let mut pixels = vec![0_u8; byte_len];
        let pointer = NonNull::new(pixels.as_mut_ptr().cast::<c_void>())
            .ok_or_else(|| unsupported("readback allocation is empty"))?;
        // SAFETY: the owned vector spans the exact RGBA8 region and row pitch.
        unsafe {
            texture.getBytes_bytesPerRow_fromRegion_mipmapLevel(
                pointer,
                row_bytes,
                MTLRegion {
                    origin: objc2_metal::MTLOrigin { x: 0, y: 0, z: 0 },
                    size: objc2_metal::MTLSize {
                        width,
                        height,
                        depth: 1,
                    },
                },
                0,
            );
        }
        Ok(Box::new(OreMetalGpuCanvasImage {
            width: plan.width,
            height: plan.height,
            pixels: pixels.into(),
            _texture: texture,
            _view: view,
        }))
    }

    fn occurrence<'a>(
        &self,
        shader: &'a Arc<dyn RenderGpuCanvasShader>,
    ) -> Result<&'a OreMetalShaderOccurrence, GpuCanvasError> {
        let shader = shader
            .as_any()
            .downcast_ref::<OreMetalShaderOccurrence>()
            .ok_or_else(|| unsupported("pipeline shader belongs to another backend"))?;
        let domain = shader
            .domain
            .upgrade()
            .ok_or_else(|| unsupported("pipeline shader domain has expired"))?;
        if !Rc::ptr_eq(&domain, &self.domain) {
            return Err(unsupported(
                "pipeline shader belongs to another Metal device domain",
            ));
        }
        Ok(shader)
    }

    fn make_layouts(
        &mut self,
        shader: &GpuCanvasAppleMetalShader,
    ) -> Result<Vec<Option<AnyResourceHandle>>, GpuCanvasError> {
        let max_group = shader.bindings().iter().map(|binding| binding.group).max();
        let Some(max_group) = max_group else {
            return Ok(Vec::new());
        };
        if max_group >= 4 {
            return Err(unsupported("binding group exceeds ORE kMaxBindGroups"));
        }
        let mut by_group = BTreeMap::<u8, Vec<BindGroupLayoutEntry>>::new();
        for binding in shader.bindings() {
            if binding.kind != GpuCanvasShaderResourceKind::UniformBuffer {
                return Err(unsupported("only uniform-buffer bindings are supported"));
            }
            let reflection = shader
                .binding_reflection()
                .iter()
                .find(|reflection| {
                    (reflection.group, reflection.binding) == (binding.group, binding.binding)
                })
                .ok_or_else(|| unsupported("binding reflection is missing"))?;
            if reflection.array_count != 1 {
                return Err(unsupported("binding arrays are deferred"));
            }
            let min_size = u32::try_from(reflection.min_buffer_size)
                .map_err(|_| unsupported("minimum buffer size exceeds u32"))?;
            let [native_slot_vs, native_slot_fs, native_slot_cs] = binding.backend_slots;
            by_group
                .entry(binding.group)
                .or_default()
                .push(BindGroupLayoutEntry {
                    binding: u32::from(binding.binding),
                    kind: BindingKind::uniformBuffer,
                    visibility: StageVisibility {
                        mask: binding.stage_mask,
                    },
                    minBindingSize: min_size,
                    nativeSlotVS: slot(native_slot_vs),
                    nativeSlotFS: slot(native_slot_fs),
                    nativeSlotCS: slot(native_slot_cs),
                    ..BindGroupLayoutEntry::default()
                });
        }
        let mut layouts = (0..=max_group).map(|_| None).collect::<Vec<_>>();
        for (group, entries) in by_group {
            let layout = self
                .context
                .makeBindGroupLayout(&BindGroupLayoutDesc {
                    groupIndex: u32::from(group),
                    entries: &entries,
                    entryCount: u32::try_from(entries.len())
                        .map_err(|_| unsupported("binding-group entry count exceeds u32"))?,
                    label: Some("authenticated GPUCanvas group layout"),
                })
                .ok_or_else(|| unsupported(self.context.lastError()))?;
            let slot = layouts
                .get_mut(usize::from(group))
                .ok_or_else(|| unsupported("binding-group layout index is out of range"))?;
            *slot = Some(layout);
        }
        Ok(layouts)
    }

    fn make_groups(
        &mut self,
        shader: &GpuCanvasAppleMetalShader,
        plan: &GpuCanvasPipelinePlan,
        layouts: &[Option<AnyResourceHandle>],
    ) -> Result<Vec<Option<AnyResourceHandle>>, GpuCanvasError> {
        if !plan.vertex_layouts.is_empty()
            || !plan.vertex_buffers.is_empty()
            || plan.index_buffer.is_some()
            || !plan.texture_bindings.is_empty()
            || !plan.sampler_bindings.is_empty()
        {
            return Err(unsupported(
                "vertex/index/texture/sampler resources are deferred",
            ));
        }
        let mut buffers = BTreeMap::<(u32, u32), AnyResourceHandle>::new();
        for uniform in &plan.uniform_buffers {
            let reflection = shader
                .binding_reflection()
                .iter()
                .find(|reflection| {
                    (u32::from(reflection.group), u32::from(reflection.binding))
                        == (uniform.group, uniform.binding)
                })
                .ok_or_else(|| unsupported("uniform is absent from authenticated reflection"))?;
            let minimum_size = usize::try_from(reflection.min_buffer_size)
                .map_err(|_| unsupported("minimum buffer size exceeds usize"))?;
            if uniform.bytes.len() < minimum_size {
                return Err(unsupported("uniform snapshot is smaller than reflection"));
            }
            let buffer = self
                .context
                .makeBuffer(
                    &BufferDesc::initialized(BufferUsage::uniform, &uniform.bytes, true)
                        .map_err(|_| unsupported("uniform snapshot exceeds u32"))?,
                )
                .ok_or_else(|| unsupported("uniform-buffer allocation failed"))?;
            if buffers
                .insert((uniform.group, uniform.binding), buffer)
                .is_some()
            {
                return Err(unsupported("duplicate uniform binding"));
            }
        }
        if buffers.len() != shader.bindings().len() {
            return Err(unsupported(
                "uniform snapshots do not cover every shader binding",
            ));
        }
        let mut groups = (0..layouts.len()).map(|_| None).collect::<Vec<_>>();
        for (group_index, layout) in layouts.iter().enumerate() {
            let Some(layout) = layout else {
                continue;
            };
            let mut entries = Vec::new();
            for binding in shader
                .bindings()
                .iter()
                .filter(|binding| usize::from(binding.group) == group_index)
            {
                let buffer = buffers
                    .get(&(u32::from(binding.group), u32::from(binding.binding)))
                    .ok_or_else(|| unsupported("uniform snapshot is missing"))?;
                let size = u32::try_from(
                    plan.uniform_buffers
                        .iter()
                        .find(|uniform| {
                            (uniform.group, uniform.binding)
                                == (u32::from(binding.group), u32::from(binding.binding))
                        })
                        .ok_or_else(|| unsupported("uniform snapshot is missing"))?
                        .bytes
                        .len(),
                )
                .map_err(|_| unsupported("uniform snapshot exceeds u32"))?;
                entries.push(UBOEntry {
                    slot: u32::from(binding.binding),
                    buffer: Some(buffer),
                    offset: 0,
                    size,
                });
            }
            let group = self
                .context
                .makeBindGroup(&BindGroupDesc {
                    layout: Some(layout),
                    ubos: &entries,
                    uboCount: u32::try_from(entries.len())
                        .map_err(|_| unsupported("uniform binding count exceeds u32"))?,
                    label: Some("authenticated GPUCanvas bind group"),
                    ..BindGroupDesc::default()
                })
                .ok_or_else(|| unsupported(self.context.lastError()))?;
            let slot = groups
                .get_mut(group_index)
                .ok_or_else(|| unsupported("binding-group index is out of range"))?;
            *slot = Some(group);
        }
        Ok(groups)
    }
}

fn slot(value: Option<u16>) -> u32 {
    value.map_or(BindGroupLayoutEntry::kNativeSlotAbsent, u32::from)
}

fn validate_pipeline_shape(plan: &GpuCanvasPipelinePlan) -> Result<(), GpuCanvasError> {
    let state = &plan.pipeline_state;
    let [target] = state.color_targets.as_slice() else {
        return Err(unsupported("exactly one color target is required"));
    };
    if target.format != "rgba8unorm"
        || target.write_mask != "rgba"
        || target.blend.is_some()
        || state.depth_stencil.is_some()
        || state.cull_mode != "none"
        || state.winding != "ccw"
        || state.topology != "triangle-list"
        || state.sample_count != 1
    {
        return Err(unsupported("pipeline state is outside the first tracer"));
    }
    Ok(())
}

fn require_entry(
    shader: &GpuCanvasAppleMetalShader,
    selection: &nuxie_render_api::GpuCanvasShaderEntrySelection,
    vertex: bool,
) -> Result<(), GpuCanvasError> {
    let expected_stage = if vertex {
        nuxie_render_api::GpuCanvasShaderStage::Vertex
    } else {
        nuxie_render_api::GpuCanvasShaderStage::Fragment
    };
    let found = shader.entries().iter().any(|entry| {
        entry.stage == expected_stage
            && entry.logical_entry_point == selection.logical_entry_point
            && entry.physical_entry_point == selection.physical_entry_point
    });
    if !found {
        return Err(unsupported("selected shader entry is stale or mismatched"));
    }
    Ok(())
}

struct OreMetalShaderOccurrence {
    artifact: GpuCanvasAppleMetalShader,
    module: AnyResourceHandle,
    domain: Weak<()>,
}

impl RenderGpuCanvasShader for OreMetalShaderOccurrence {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct OreMetalGpuCanvasImage {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
    _texture: Retained<ProtocolObject<dyn MTLTexture>>,
    _view: AnyResourceHandle,
}

impl OreMetalGpuCanvasImage {
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

impl RenderImage for OreMetalGpuCanvasImage {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}
