//! Imported GPU-canvas execution shared by the exact browser renderers.
//!
//! Shader translation remains a trusted-editor concern. This module consumes
//! the exact RSTB target selected by the active renderer and executes the
//! backend-neutral plan through Rive's ORE API. It never selects a device,
//! translates shader source, or depends on `wgpu`.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::sync::{Arc, Weak};

use nuxie_ore_metal::context::{ContextApi, FrameDescriptor, ShaderTarget};
use nuxie_ore_metal::gpu_resource::AnyResourceHandle;
use nuxie_ore_metal::types::{
    kMaxBindGroups, BindGroupDesc, BindGroupLayoutDesc, BindGroupLayoutEntry, BindingKind,
    BlendFactor, BlendOp, BlendState, BufferDesc, BufferUsage, ClearColor, ColorAttachment,
    ColorTargetState, ColorWriteMask, CompareFunction, CullMode, DepthStencilAttachment,
    DepthStencilState, Filter, IndexFormat, LoadOp, PipelineDesc, PrimitiveTopology,
    RenderPassDesc, SampEntry, SampleType, SamplerDesc, StageVisibility, StencilFaceState,
    StencilOp, StoreOp, TexEntry, TextureAspect, TextureDataDesc, TextureDesc, TextureFormat,
    TextureType, TextureViewDesc, TextureViewDimension, UBOEntry, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexStepMode, WrapMode,
};
use nuxie_render_api::{
    GpuCanvasAttachmentView, GpuCanvasBlendState, GpuCanvasColorAttachment,
    GpuCanvasDepthStencilAttachment, GpuCanvasDepthStencilState, GpuCanvasDrawCommand,
    GpuCanvasError, GpuCanvasPipelinePlan, GpuCanvasPipelineShaders, GpuCanvasPlan,
    GpuCanvasRenderPass, GpuCanvasShaderArtifact, GpuCanvasShaderBinding, GpuCanvasShaderEntry,
    GpuCanvasShaderEntrySelection, GpuCanvasShaderProfile, GpuCanvasShaderResourceKind,
    GpuCanvasShaderStage, GpuCanvasShaderTextureSampleType, GpuCanvasShaderTextureViewDimension,
    GpuCanvasStencilFace, GpuCanvasTextureBinding, GpuCanvasTextureUpload, RenderGpuCanvasShader,
};

use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImageHandle;

const MAX_GPU_CANVAS_DIMENSION: u32 = 2_048;
const MAX_DRAW_INVOCATIONS: u64 = 1_000_000;
const MAX_VERTEX_BUFFERS: usize = 8;
const MAX_VERTEX_ATTRIBUTES: usize = 16;
const MAX_BINDING_INDEX: u32 = 7;
const MAX_UNIFORM_BUFFER_BYTES: usize = 64 * 1024;
const MAX_VERTEX_BUFFER_BYTES: usize = 16 * 1024 * 1024;
const MAX_RETAINED_TEXTURES: usize = 16;

fn rejected(message: impl Into<String>) -> GpuCanvasError {
    GpuCanvasError::new(format!("exact GPU-canvas: {}", message.into()))
}

/// One exact ORE context and the authored texture resources that belong to it.
struct ExactGpuCanvasContext<C>(Rc<std::cell::RefCell<C>>);

impl<C> ExactGpuCanvasContext<C> {
    fn as_ref(&self) -> std::cell::Ref<'_, C> {
        self.0.borrow()
    }

    fn as_mut(&mut self) -> std::cell::RefMut<'_, C> {
        self.0.borrow_mut()
    }
}

pub(crate) struct ExactGpuCanvas<C: ContextApi> {
    context: ExactGpuCanvasContext<C>,
    profile: GpuCanvasShaderProfile,
    retained_textures: Vec<RetainedTexture>,
    frame_number: u64,
}

impl<C: ContextApi + 'static> ExactGpuCanvas<C> {
    pub(crate) fn new(
        context: Box<C>,
        profile: GpuCanvasShaderProfile,
    ) -> Result<Self, GpuCanvasError> {
        Self::new_shared(Rc::new(std::cell::RefCell::new(*context)), profile)
    }

    /// Share the one ORE context already owned by the source RenderContext.
    pub(crate) fn new_shared(
        context: Rc<std::cell::RefCell<C>>,
        profile: GpuCanvasShaderProfile,
    ) -> Result<Self, GpuCanvasError> {
        Self::from_context(ExactGpuCanvasContext(context), profile)
    }

    fn from_context(
        context: ExactGpuCanvasContext<C>,
        profile: GpuCanvasShaderProfile,
    ) -> Result<Self, GpuCanvasError> {
        let expected = match profile {
            GpuCanvasShaderProfile::WebGpu => ShaderTarget::wgsl,
            GpuCanvasShaderProfile::WebGl2 => ShaderTarget::glsl,
            GpuCanvasShaderProfile::TrustedVulkanSpirV => ShaderTarget::spirv,
            GpuCanvasShaderProfile::TrustedAppleMetal => {
                return Err(rejected("trusted Metal uses its concrete ORE adapter"));
            }
        };
        if context.as_ref().shaderTarget() != expected {
            return Err(rejected(format!(
                "ORE target {:?} does not match profile {profile:?}",
                context.as_ref().shaderTarget()
            )));
        }
        Ok(Self {
            context,
            profile,
            retained_textures: Vec::new(),
            frame_number: 0,
        })
    }

    pub(crate) fn context_mut(&mut self) -> std::cell::RefMut<'_, C> {
        self.context.as_mut()
    }

    pub(crate) fn context_handle(&self) -> nuxie_render_api::OreContextHandle {
        self.context.0.clone()
    }

    pub(crate) fn next_frame_number(&mut self) -> u64 {
        self.frame_number = self.frame_number.wrapping_add(1);
        self.frame_number
    }

    pub(crate) fn begin_frame(&mut self, frame_number: u64) {
        self.context.as_mut().beginFrame(&FrameDescriptor::new(
            frame_number.saturating_sub(1),
            frame_number,
        ));
    }

    /// Begin an ORE frame on the exact externally owned command buffer that
    /// the concrete backend supplies, as required by Vulkan upstream.
    ///
    /// # Safety
    ///
    /// `external_command_buffer` must belong to this context's device and
    /// remain recording-valid until [`Self::end_frame`] returns.
    pub(crate) unsafe fn begin_frame_external(
        &mut self,
        safe_frame_number: u64,
        current_frame_number: u64,
        external_command_buffer: std::ptr::NonNull<std::ffi::c_void>,
    ) {
        self.context.as_mut().beginFrame(&unsafe {
            FrameDescriptor::withExternalCommandBuffer(
                external_command_buffer,
                safe_frame_number,
                current_frame_number,
            )
        });
    }

    pub(crate) fn end_frame(&mut self) {
        // Inline deferred pass finish replays through this same context. End
        // the borrow before that callback, as with the source raw back-pointer.
        let pass = self
            .context
            .as_ref()
            .activeRenderPass()
            .and_then(|pass| pass.upgrade());
        if let Some(pass) = pass {
            if !pass.isFinished() {
                pass.finish();
            }
        }
        self.context.as_mut().endFrame();
    }

    pub(crate) fn make_shader_artifact(
        &mut self,
        artifact: &GpuCanvasShaderArtifact,
        execution_anchor: Rc<dyn Any>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        let occurrence = ExactGpuCanvasShaderOccurrence::compile(
            &mut *self.context.as_mut(),
            self.profile,
            artifact,
            execution_anchor,
        )?;
        #[allow(clippy::arc_with_non_send_sync)]
        Ok(Arc::new(occurrence))
    }

    pub(crate) fn make_shader_occurrence(
        &mut self,
        prepared: &Arc<dyn RenderGpuCanvasShader>,
        execution_anchor: Rc<dyn Any>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        let prepared = prepared
            .as_any()
            .downcast_ref::<ExactGpuCanvasShaderOccurrence>()
            .ok_or_else(|| rejected("prepared shader belongs to another renderer"))?;
        if prepared.profile != self.profile
            || !Rc::ptr_eq(&prepared.execution_anchor, &execution_anchor)
        {
            return Err(rejected(
                "prepared shader belongs to another renderer/device domain",
            ));
        }
        self.make_shader_artifact(&prepared.artifact, execution_anchor)
    }

    /// Execute a submission after the concrete backend has begun its ORE
    /// frame. The caller always pairs this with [`Self::end_frame`], including
    /// on error, because WebGPU owns the external encoder lifecycle.
    pub(crate) fn execute_current_frame(
        &mut self,
        canvas: &rcp<RenderCanvas>,
        pipelines: &[GpuCanvasPipelineShaders],
        plan: &GpuCanvasPlan,
        execution_anchor: &Rc<dyn Any>,
    ) -> Result<RiveRenderImageHandle, GpuCanvasError> {
        validate_submission(plan)?;
        let pipeline_plans = materialize_pipeline_plans(plan);
        let render_passes = materialize_render_passes(plan);
        if pipelines.len() != pipeline_plans.len() {
            return Err(rejected(format!(
                "{} shader pairs do not match {} pipeline snapshots",
                pipelines.len(),
                pipeline_plans.len()
            )));
        }

        let canvas_ptr = canvas.get().cast::<std::ffi::c_void>();
        let canvas_view = unsafe { self.context.as_mut().wrapCanvasTexture(canvas_ptr) };
        let canvas_view = canvas_view.ok_or_else(|| {
            rejected(context_error(
                &*self.context.as_ref(),
                "wrap canvas texture",
            ))
        })?;
        let texture_specs = collect_texture_specs(&pipeline_plans, &render_passes)?;
        let retained_textures = self.retain_textures(&texture_specs, execution_anchor)?;

        let mut built = Vec::with_capacity(pipeline_plans.len());
        for (shaders, pipeline_plan) in pipelines.iter().zip(&pipeline_plans) {
            built.push(build_pipeline(
                &mut *self.context.as_mut(),
                self.profile,
                shaders,
                pipeline_plan,
                &retained_textures,
                execution_anchor,
            )?);
        }

        let mut attachment_views = BTreeMap::new();
        for pass in &render_passes {
            for attachment in &pass.color_attachments {
                retain_attachment_view(
                    &mut *self.context.as_mut(),
                    &retained_textures,
                    &attachment.view,
                    &mut attachment_views,
                )?;
                if let Some(resolve) = &attachment.resolve_target {
                    retain_attachment_view(
                        &mut *self.context.as_mut(),
                        &retained_textures,
                        resolve,
                        &mut attachment_views,
                    )?;
                }
            }
            if let Some(depth) = &pass.depth_stencil_attachment {
                retain_attachment_view(
                    &mut *self.context.as_mut(),
                    &retained_textures,
                    &depth.view,
                    &mut attachment_views,
                )?;
            }
        }

        for pass in &render_passes {
            execute_pass(
                &mut *self.context.as_mut(),
                pass,
                &built,
                &canvas_view,
                &attachment_views,
                plan.width,
                plan.height,
            )?;
        }

        RiveRenderImageHandle::from_exact(unsafe { &*canvas.get() }.ref_render_image())
            .ok_or_else(|| rejected("render canvas did not publish an image"))
    }

    fn retain_textures(
        &mut self,
        specs: &BTreeMap<u64, GpuCanvasTextureBinding>,
        execution_anchor: &Rc<dyn Any>,
    ) -> Result<BTreeMap<u64, RetainedTextureResource>, GpuCanvasError> {
        self.retained_textures
            .retain(|retained| retained.lifetime.upgrade().is_some());
        for spec in specs.values() {
            if let Some(retained) = self
                .retained_textures
                .iter_mut()
                .find(|retained| retained.resource_id == spec.resource_id)
            {
                retained.update(spec)?;
                continue;
            }
            if self.retained_textures.len() >= MAX_RETAINED_TEXTURES {
                return Err(rejected(format!(
                    "more than {MAX_RETAINED_TEXTURES} live authored textures"
                )));
            }
            self.retained_textures.push(RetainedTexture::create(
                &mut *self.context.as_mut(),
                spec,
                execution_anchor,
            )?);
        }
        specs
            .keys()
            .map(|resource_id| {
                let texture = self
                    .retained_textures
                    .iter()
                    .find(|retained| retained.resource_id == *resource_id)
                    .ok_or_else(|| rejected("authored texture disappeared during submission"))?;
                Ok((*resource_id, texture.resource.clone()))
            })
            .collect()
    }
}

use crate::authored_ore_shader::ExactGpuCanvasShaderOccurrence;
struct RetainedTexture {
    resource_id: u64,
    lifetime: Weak<()>,
    descriptor: TextureIdentity,
    uploads: Vec<GpuCanvasTextureUpload>,
    applied_uploads: usize,
    resource: RetainedTextureResource,
    external_image: Option<Rc<dyn nuxie_render_api::RenderImage>>,
}

#[derive(Clone)]
enum RetainedTextureResource {
    AuthoredTexture(AnyResourceHandle),
    ExternalImageView(AnyResourceHandle),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TextureIdentity {
    width: u32,
    height: u32,
    depth_or_array_layers: u32,
    format: String,
    texture_type: String,
    render_target: bool,
    sample_count: u32,
    mip_level_count: u32,
}

impl TextureIdentity {
    fn from_binding(binding: &GpuCanvasTextureBinding) -> Self {
        Self {
            width: binding.width,
            height: binding.height,
            depth_or_array_layers: binding.depth_or_array_layers,
            format: binding.format.clone(),
            texture_type: binding.texture_type.clone(),
            render_target: binding.render_target,
            sample_count: binding.sample_count,
            mip_level_count: binding.mip_level_count,
        }
    }
}

impl RetainedTexture {
    fn create(
        context: &mut dyn ContextApi,
        binding: &GpuCanvasTextureBinding,
        execution_anchor: &Rc<dyn Any>,
    ) -> Result<Self, GpuCanvasError> {
        let descriptor = TextureIdentity::from_binding(binding);
        let (resource, external_image) = if let Some(image) = &binding.external_image {
            validate_external_image_binding(binding, image.as_ref())?;
            let source = image
                .as_any()
                .downcast_ref::<RiveRenderImageHandle>()
                .ok_or_else(|| {
                    rejected("Image:view() image is not a GPU-backed RiveRenderImage")
                })?;
            let source_texture = source
                .source_texture_for_execution_anchor(execution_anchor)
                .ok_or_else(|| {
                    rejected("Image:view() image belongs to another renderer/device domain")
                })?;
            // Pinned lua_gpu.cpp:3632-3693 passes the retained
            // RiveRenderImage's exact gpu::Texture to Context::wrapRiveTexture.
            // In Vulkan this also records the required shader-read transition
            // in the active host command buffer (ore_context_vulkan.cpp:
            // 1697-1705). No authored replacement texture is allocated.
            let view = unsafe {
                context.wrapRiveTexture(
                    source_texture.as_ptr().cast(),
                    binding.width,
                    binding.height,
                )
            }
            .ok_or_else(|| rejected(context_error(context, "wrap Image:view() texture")))?;
            (
                RetainedTextureResource::ExternalImageView(view),
                Some(Rc::clone(image)),
            )
        } else {
            let texture = context
                .makeTexture(&TextureDesc {
                    width: binding.width,
                    height: binding.height,
                    depthOrArrayLayers: binding.depth_or_array_layers,
                    format: texture_format(&binding.format)?,
                    r#type: texture_type(&binding.texture_type)?,
                    renderTarget: binding.render_target,
                    numMipmaps: binding.mip_level_count,
                    sampleCount: binding.sample_count,
                    label: Some("authored GPU-canvas texture"),
                })
                .ok_or_else(|| rejected(context_error(context, "allocate authored texture")))?;
            (RetainedTextureResource::AuthoredTexture(texture), None)
        };
        let mut retained = Self {
            resource_id: binding.resource_id,
            lifetime: binding.lifetime.downgrade(),
            descriptor,
            uploads: Vec::new(),
            applied_uploads: 0,
            resource,
            external_image,
        };
        retained.apply_uploads(binding)?;
        Ok(retained)
    }

    fn update(&mut self, binding: &GpuCanvasTextureBinding) -> Result<(), GpuCanvasError> {
        if self.descriptor != TextureIdentity::from_binding(binding)
            || !same_external_image(
                self.external_image.as_ref(),
                binding.external_image.as_ref(),
            )
            || !binding.uploads.starts_with(&self.uploads)
        {
            return Err(rejected(format!(
                "authored texture {} changed after allocation",
                binding.resource_id
            )));
        }
        self.apply_uploads(binding)
    }

    fn apply_uploads(&mut self, binding: &GpuCanvasTextureBinding) -> Result<(), GpuCanvasError> {
        if matches!(self.resource, RetainedTextureResource::ExternalImageView(_)) {
            if !binding.uploads.is_empty() {
                return Err(rejected(
                    "Image:view() cannot receive authored texture uploads",
                ));
            }
            return Ok(());
        }
        let RetainedTextureResource::AuthoredTexture(texture) = &self.resource else {
            unreachable!("external image view returned above")
        };
        for upload in binding.uploads.iter().skip(self.applied_uploads) {
            texture
                .upload(&TextureDataDesc {
                    data: Some(&upload.bytes),
                    bytesPerRow: upload.bytes_per_row,
                    rowsPerImage: upload.rows_per_image,
                    mipLevel: upload.mip_level,
                    layer: upload.array_layer,
                    x: upload.x,
                    y: upload.y,
                    z: upload.z,
                    width: upload.width,
                    height: upload.height,
                    depth: upload.depth,
                })
                .map_err(|error| rejected(format!("texture upload failed: {error:?}")))?;
        }
        self.uploads = binding.uploads.clone();
        self.applied_uploads = binding.uploads.len();
        Ok(())
    }
}

fn same_external_image(
    left: Option<&Rc<dyn nuxie_render_api::RenderImage>>,
    right: Option<&Rc<dyn nuxie_render_api::RenderImage>>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => Rc::ptr_eq(left, right),
        _ => false,
    }
}

fn validate_external_image_binding(
    binding: &GpuCanvasTextureBinding,
    image: &dyn nuxie_render_api::RenderImage,
) -> Result<(), GpuCanvasError> {
    if binding.width != image.width()
        || binding.height != image.height()
        || binding.depth_or_array_layers != 1
        || binding.format != "rgba8unorm"
        || binding.texture_type != "2d"
        || binding.render_target
        || binding.sample_count != 1
        || binding.mip_level_count != 1
        || binding.view_dimension != "2d"
        || binding.base_mip_level != 0
        || binding.mip_level_count_in_view != 1
        || binding.base_array_layer != 0
        || binding.array_layer_count != 1
        || !binding.uploads.is_empty()
    {
        return Err(rejected(
            "Image:view() binding does not describe its exact retained 2D image view",
        ));
    }
    Ok(())
}

struct BuiltPipeline {
    pipeline: AnyResourceHandle,
    groups: Vec<Option<AnyResourceHandle>>,
    vertex_buffers: Vec<(u32, AnyResourceHandle)>,
    index_buffer: Option<(IndexFormat, AnyResourceHandle)>,
    _layouts: Vec<Option<AnyResourceHandle>>,
    _buffers: Vec<AnyResourceHandle>,
    _textures: Vec<AnyResourceHandle>,
    _views: Vec<AnyResourceHandle>,
    _samplers: Vec<AnyResourceHandle>,
}

fn build_pipeline(
    context: &mut dyn ContextApi,
    profile: GpuCanvasShaderProfile,
    shaders: &GpuCanvasPipelineShaders,
    plan: &GpuCanvasPipelinePlan,
    retained_textures: &BTreeMap<u64, RetainedTextureResource>,
    execution_anchor: &Rc<dyn Any>,
) -> Result<BuiltPipeline, GpuCanvasError> {
    let vertex = shader_occurrence(&shaders.vertex, profile, execution_anchor, "vertex")?;
    let (vertex_module, vertex_entry) =
        vertex.module_for(GpuCanvasShaderStage::Vertex, plan.vertex_entry.as_ref())?;
    // Upstream resolves an explicit fragment before parsing color targets.
    // The combined-file fallback remains deferred until the parsed target
    // count is known.
    let explicit_fragment = shaders
        .fragment
        .as_ref()
        .map(|fragment| shader_occurrence(fragment, profile, execution_anchor, "fragment"))
        .transpose()?;
    let explicit_fragment_module_and_entry = explicit_fragment
        .map(|fragment| {
            fragment.module_for(GpuCanvasShaderStage::Fragment, plan.fragment_entry.as_ref())
        })
        .transpose()?;

    let vertex_attributes = plan
        .vertex_layouts
        .iter()
        .map(|layout| {
            layout
                .attributes
                .iter()
                .map(|attribute| {
                    Ok(VertexAttribute {
                        format: vertex_format(&attribute.format)?,
                        offset: u32::try_from(attribute.offset)
                            .map_err(|_| rejected("vertex attribute offset exceeds u32"))?,
                        shaderSlot: attribute.shader_location,
                    })
                })
                .collect::<Result<Vec<_>, GpuCanvasError>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let vertex_layouts = plan
        .vertex_layouts
        .iter()
        .zip(&vertex_attributes)
        .map(|(layout, attributes)| {
            Ok(VertexBufferLayout {
                stride: u32::try_from(layout.stride)
                    .map_err(|_| rejected("vertex stride exceeds u32"))?,
                stepMode: vertex_step_mode(&layout.step_mode)?,
                attributes: Some(attributes),
                attributeCount: u32::try_from(attributes.len())
                    .map_err(|_| rejected("vertex attribute count exceeds u32"))?,
            })
        })
        .collect::<Result<Vec<_>, GpuCanvasError>>()?;

    let (color_targets, color_count) = color_targets(&plan.pipeline_state.color_targets)?;
    let fragment_module_and_entry =
        match fragment_selection(explicit_fragment.is_some(), color_count as usize) {
            FragmentSelection::Explicit => explicit_fragment_module_and_entry,
            FragmentSelection::CombinedVertex => Some(
                vertex.module_for(GpuCanvasShaderStage::Fragment, plan.fragment_entry.as_ref())?,
            ),
            FragmentSelection::None => None,
        };
    // Mirrors pinned lua_gpu.cpp:2181-2217 exactly. Automatic layouts always
    // come from the vertex module's complete binding map. They are not stage
    // filtered and an explicitly separate fragment module is not merged in.
    let bindings = vertex.bindings();
    let layouts = make_layouts(context, bindings)?;
    let layout_refs = layouts.iter().map(Option::as_ref).collect::<Vec<_>>();
    let (depth_stencil, stencil_front, stencil_back, stencil_read_mask, stencil_write_mask) =
        depth_stencil(plan.pipeline_state.depth_stencil.as_ref())?;
    let index_format = plan
        .index_buffer
        .as_ref()
        .map(|buffer| index_format(&buffer.format))
        .transpose()?
        .unwrap_or(IndexFormat::none);
    let mut pipeline_error = String::new();
    let pipeline = context
        .makePipeline(
            &PipelineDesc {
                vertexModule: Some(vertex_module),
                vertexEntryPoint: Some(&vertex_entry.physical_entry_point),
                fragmentModule: fragment_module_and_entry.map(|(module, _)| module),
                fragmentEntryPoint: fragment_module_and_entry
                    .map(|(_, entry)| entry.physical_entry_point.as_str()),
                vertexBuffers: (!vertex_layouts.is_empty()).then_some(vertex_layouts.as_slice()),
                vertexBufferCount: u32::try_from(vertex_layouts.len())
                    .map_err(|_| rejected("vertex layout count exceeds u32"))?,
                topology: primitive_topology(&plan.pipeline_state.topology)?,
                indexFormat: index_format,
                cullMode: cull_mode(&plan.pipeline_state.cull_mode)?,
                winding: winding(&plan.pipeline_state.winding)?,
                colorTargets: color_targets,
                colorCount: color_count,
                depthStencil: depth_stencil,
                stencilFront: stencil_front,
                stencilBack: stencil_back,
                stencilReadMask: stencil_read_mask,
                stencilWriteMask: stencil_write_mask,
                sampleCount: plan.pipeline_state.sample_count,
                bindGroupLayouts: (!layout_refs.is_empty()).then_some(layout_refs.as_slice()),
                bindGroupLayoutCount: u32::try_from(layout_refs.len())
                    .map_err(|_| rejected("binding-group layout count exceeds u32"))?,
                label: Some("authored GPU-canvas pipeline"),
            },
            Some(&mut pipeline_error),
        )
        .ok_or_else(|| {
            rejected(if pipeline_error.is_empty() {
                context_error(context, "create authored pipeline")
            } else {
                pipeline_error
            })
        })?;

    let resources = make_pipeline_resources(context, bindings, plan, &layouts, retained_textures)?;
    Ok(BuiltPipeline {
        pipeline,
        groups: resources.groups,
        vertex_buffers: resources.vertex_buffers,
        index_buffer: resources.index_buffer,
        _layouts: layouts,
        _buffers: resources.buffers,
        _textures: resources.textures,
        _views: resources.views,
        _samplers: resources.samplers,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FragmentSelection {
    Explicit,
    CombinedVertex,
    None,
}

/// Mirrors pinned lua_gpu.cpp:1943-1947 and 2096-2111 exactly: an authored
/// fragment always wins, combined-file fallback exists only for color output,
/// and a vertex-only pipeline with no color targets has no fragment stage.
fn fragment_selection(has_explicit_fragment: bool, color_target_count: usize) -> FragmentSelection {
    if has_explicit_fragment {
        FragmentSelection::Explicit
    } else if color_target_count > 0 {
        FragmentSelection::CombinedVertex
    } else {
        FragmentSelection::None
    }
}

fn shader_occurrence<'a>(
    shader: &'a Arc<dyn RenderGpuCanvasShader>,
    profile: GpuCanvasShaderProfile,
    execution_anchor: &Rc<dyn Any>,
    stage: &str,
) -> Result<&'a ExactGpuCanvasShaderOccurrence, GpuCanvasError> {
    let shader = shader
        .as_any()
        .downcast_ref::<ExactGpuCanvasShaderOccurrence>()
        .ok_or_else(|| rejected(format!("{stage} shader belongs to another renderer")))?;
    if shader.profile != profile || !Rc::ptr_eq(&shader.execution_anchor, execution_anchor) {
        return Err(rejected(format!(
            "{stage} shader belongs to another renderer/device domain"
        )));
    }
    Ok(shader)
}

fn make_layouts(
    context: &mut dyn ContextApi,
    bindings: &[GpuCanvasShaderBinding],
) -> Result<Vec<Option<AnyResourceHandle>>, GpuCanvasError> {
    let Some(max_group) = bindings.iter().map(|binding| binding.group).max() else {
        return Ok(Vec::new());
    };
    if u32::from(max_group) >= kMaxBindGroups {
        return Err(rejected("binding group exceeds ORE kMaxBindGroups"));
    }
    let mut layouts = (0..=max_group).map(|_| None).collect::<Vec<_>>();
    for group in 0..=max_group {
        let entries = bindings
            .iter()
            .filter(|binding| binding.group == group)
            .map(layout_entry)
            .collect::<Result<Vec<_>, _>>()?;
        if entries.is_empty() {
            continue;
        }
        let layout = context
            .makeBindGroupLayout(&BindGroupLayoutDesc {
                groupIndex: u32::from(group),
                entries: Some(&entries),
                entryCount: u32::try_from(entries.len())
                    .map_err(|_| rejected("binding layout count exceeds u32"))?,
                label: Some("authored GPU-canvas bind-group layout"),
            })
            .ok_or_else(|| rejected(context_error(context, "create bind-group layout")))?;
        layouts[usize::from(group)] = Some(layout);
    }
    Ok(layouts)
}

fn layout_entry(binding: &GpuCanvasShaderBinding) -> Result<BindGroupLayoutEntry, GpuCanvasError> {
    let [vs, fs, cs] = binding.backend_slots;
    let mut entry = BindGroupLayoutEntry {
        binding: u32::from(binding.binding),
        kind: binding_kind(binding.kind)?,
        visibility: StageVisibility {
            mask: binding.stage_mask,
        },
        textureMultisampled: binding.texture_multisampled,
        nativeSlotVS: native_slot(vs),
        nativeSlotFS: native_slot(fs),
        nativeSlotCS: native_slot(cs),
        ..BindGroupLayoutEntry::default()
    };
    match binding.kind {
        GpuCanvasShaderResourceKind::SampledTexture => {
            entry.textureViewDim = binding_texture_dimension(binding.texture_view_dimension)?;
            entry.textureSampleType = binding_sample_type(binding.texture_sample_type)?;
        }
        GpuCanvasShaderResourceKind::StorageTexture => {
            entry.textureViewDim = binding_texture_dimension(binding.texture_view_dimension)?;
        }
        GpuCanvasShaderResourceKind::UniformBuffer
        | GpuCanvasShaderResourceKind::StorageBufferReadOnly
        | GpuCanvasShaderResourceKind::StorageBufferReadWrite
        | GpuCanvasShaderResourceKind::Sampler
        | GpuCanvasShaderResourceKind::ComparisonSampler => {}
    }
    Ok(entry)
}

fn native_slot(slot: Option<u16>) -> u32 {
    slot.map_or(BindGroupLayoutEntry::kNativeSlotAbsent, u32::from)
}

struct PipelineResources {
    groups: Vec<Option<AnyResourceHandle>>,
    buffers: Vec<AnyResourceHandle>,
    textures: Vec<AnyResourceHandle>,
    views: Vec<AnyResourceHandle>,
    samplers: Vec<AnyResourceHandle>,
    vertex_buffers: Vec<(u32, AnyResourceHandle)>,
    index_buffer: Option<(IndexFormat, AnyResourceHandle)>,
}

fn make_pipeline_resources(
    context: &mut dyn ContextApi,
    bindings: &[GpuCanvasShaderBinding],
    plan: &GpuCanvasPipelinePlan,
    layouts: &[Option<AnyResourceHandle>],
    retained_textures: &BTreeMap<u64, RetainedTextureResource>,
) -> Result<PipelineResources, GpuCanvasError> {
    let mut uniform_buffers = BTreeMap::new();
    let mut buffers = Vec::new();
    for uniform in &plan.uniform_buffers {
        let buffer = context
            .makeBuffer(
                &BufferDesc::initialized(BufferUsage::uniform, &uniform.bytes, true)
                    .map_err(|_| rejected("uniform buffer exceeds u32"))?,
            )
            .ok_or_else(|| rejected(context_error(context, "allocate uniform buffer")))?;
        if uniform_buffers
            .insert((uniform.group, uniform.binding), buffer.clone())
            .is_some()
        {
            return Err(rejected("duplicate uniform binding"));
        }
        buffers.push(buffer);
    }

    let mut texture_views = BTreeMap::new();
    let mut textures = Vec::new();
    let mut views = Vec::new();
    for binding in &plan.texture_bindings {
        let resource = retained_textures
            .get(&binding.resource_id)
            .ok_or_else(|| rejected("sampled texture is absent from retained resources"))?;
        let view = match resource {
            RetainedTextureResource::AuthoredTexture(texture) => {
                textures.push(texture.clone());
                make_texture_view(context, texture, binding)?
            }
            RetainedTextureResource::ExternalImageView(view) => view.clone(),
        };
        if texture_views
            .insert((binding.group, binding.binding), view.clone())
            .is_some()
        {
            return Err(rejected("duplicate sampled-texture binding"));
        }
        views.push(view);
    }

    let mut sampler_resources = BTreeMap::new();
    let mut samplers = Vec::new();
    for binding in &plan.sampler_bindings {
        let sampler = context
            .makeSampler(&SamplerDesc {
                minFilter: filter(&binding.min_filter)?,
                magFilter: filter(&binding.mag_filter)?,
                mipmapFilter: filter(&binding.mipmap_filter)?,
                wrapU: wrap_mode(&binding.address_mode_u)?,
                wrapV: wrap_mode(&binding.address_mode_v)?,
                wrapW: wrap_mode(&binding.address_mode_w)?,
                compare: binding
                    .compare
                    .as_deref()
                    .map(compare_function)
                    .transpose()?
                    .unwrap_or(CompareFunction::none),
                minLod: binding.lod_min_clamp,
                maxLod: binding.lod_max_clamp,
                maxAnisotropy: u32::from(binding.max_anisotropy),
                label: Some("authored GPU-canvas sampler"),
            })
            .ok_or_else(|| rejected(context_error(context, "allocate sampler")))?;
        if sampler_resources
            .insert((binding.group, binding.binding), sampler.clone())
            .is_some()
        {
            return Err(rejected("duplicate sampler binding"));
        }
        samplers.push(sampler);
    }

    let mut groups = (0..layouts.len()).map(|_| None).collect::<Vec<_>>();
    for (group_index, layout) in layouts.iter().enumerate() {
        let Some(layout) = layout else {
            continue;
        };
        let group_index_u32 =
            u32::try_from(group_index).map_err(|_| rejected("group index exceeds u32"))?;
        let mut ubos = Vec::new();
        let mut tex = Vec::new();
        let mut samp = Vec::new();
        for binding in bindings
            .iter()
            .filter(|binding| usize::from(binding.group) == group_index)
        {
            let identity = (group_index_u32, u32::from(binding.binding));
            match binding.kind {
                GpuCanvasShaderResourceKind::UniformBuffer => {
                    let buffer = uniform_buffers.get(&identity).ok_or_else(|| {
                        rejected(format!(
                            "uniform group {} binding {} is missing",
                            identity.0, identity.1
                        ))
                    })?;
                    let size = u32::try_from(
                        plan.uniform_buffers
                            .iter()
                            .find(|uniform| (uniform.group, uniform.binding) == identity)
                            .expect("uniform map and plan agree")
                            .bytes
                            .len(),
                    )
                    .map_err(|_| rejected("uniform size exceeds u32"))?;
                    ubos.push(UBOEntry {
                        slot: identity.1,
                        buffer: Some(buffer),
                        offset: 0,
                        size,
                    });
                }
                GpuCanvasShaderResourceKind::SampledTexture => {
                    let view = texture_views.get(&identity).ok_or_else(|| {
                        rejected(format!(
                            "texture group {} binding {} is missing",
                            identity.0, identity.1
                        ))
                    })?;
                    tex.push(TexEntry {
                        slot: identity.1,
                        view: Some(view),
                    });
                }
                GpuCanvasShaderResourceKind::Sampler
                | GpuCanvasShaderResourceKind::ComparisonSampler => {
                    let sampler = sampler_resources.get(&identity).ok_or_else(|| {
                        rejected(format!(
                            "sampler group {} binding {} is missing",
                            identity.0, identity.1
                        ))
                    })?;
                    samp.push(SampEntry {
                        slot: identity.1,
                        sampler: Some(sampler),
                    });
                }
                _ => {
                    return Err(rejected(
                        "storage resources are not exposed by GPUCanvas plans",
                    ));
                }
            }
        }
        let group = context
            .makeBindGroup(&BindGroupDesc {
                layout: Some(layout),
                ubos: &ubos,
                uboCount: u32::try_from(ubos.len())
                    .map_err(|_| rejected("uniform binding count exceeds u32"))?,
                textures: &tex,
                textureCount: u32::try_from(tex.len())
                    .map_err(|_| rejected("texture binding count exceeds u32"))?,
                samplers: &samp,
                samplerCount: u32::try_from(samp.len())
                    .map_err(|_| rejected("sampler binding count exceeds u32"))?,
                label: Some("authored GPU-canvas bind group"),
            })
            .ok_or_else(|| rejected(context_error(context, "create bind group")))?;
        groups[group_index] = Some(group);
    }

    let mut vertex_buffers = Vec::new();
    for authored in &plan.vertex_buffers {
        let buffer = context
            .makeBuffer(
                &BufferDesc::initialized(BufferUsage::vertex, &authored.bytes, true)
                    .map_err(|_| rejected("vertex buffer exceeds u32"))?,
            )
            .ok_or_else(|| rejected(context_error(context, "allocate vertex buffer")))?;
        buffers.push(buffer.clone());
        vertex_buffers.push((authored.slot, buffer));
    }
    let index_buffer = plan
        .index_buffer
        .as_ref()
        .map(|authored| {
            let format = index_format(&authored.format)?;
            let buffer = context
                .makeBuffer(
                    &BufferDesc::initialized(BufferUsage::index, &authored.bytes, true)
                        .map_err(|_| rejected("index buffer exceeds u32"))?,
                )
                .ok_or_else(|| rejected(context_error(context, "allocate index buffer")))?;
            buffers.push(buffer.clone());
            Ok((format, buffer))
        })
        .transpose()?;

    Ok(PipelineResources {
        groups,
        buffers,
        textures,
        views,
        samplers,
        vertex_buffers,
        index_buffer,
    })
}

fn execute_pass(
    context: &mut dyn ContextApi,
    authored: &GpuCanvasRenderPass,
    pipelines: &[BuiltPipeline],
    canvas_view: &AnyResourceHandle,
    attachment_views: &BTreeMap<AttachmentViewKey, AnyResourceHandle>,
    width: u32,
    height: u32,
) -> Result<(), GpuCanvasError> {
    if authored.color_attachments.len() > 4 {
        return Err(rejected("render pass has more than four color attachments"));
    }
    let mut colors = [ColorAttachment::default(); 4];
    for (destination, attachment) in colors.iter_mut().zip(&authored.color_attachments) {
        *destination = ColorAttachment {
            view: Some(resolve_attachment(
                &attachment.view,
                canvas_view,
                attachment_views,
            )?),
            resolveTarget: attachment
                .resolve_target
                .as_ref()
                .map(|view| resolve_attachment(view, canvas_view, attachment_views))
                .transpose()?,
            loadOp: load_op(&attachment.load_op)?,
            storeOp: store_op(&attachment.store_op)?,
            clearColor: clear_color(attachment.clear_color)?,
        };
    }
    let depth = authored
        .depth_stencil_attachment
        .as_ref()
        .map(|attachment| depth_attachment(attachment, canvas_view, attachment_views))
        .transpose()?
        .unwrap_or_default();
    let mut pass_error = String::new();
    let mut pass = context
        .beginRenderPass(
            &RenderPassDesc {
                colorAttachments: colors,
                colorCount: u32::try_from(authored.color_attachments.len())
                    .map_err(|_| rejected("color attachment count exceeds u32"))?,
                depthStencil: depth,
                label: Some("authored GPU-canvas render pass"),
            },
            Some(&mut pass_error),
        )
        .ok_or_else(|| {
            rejected(if pass_error.is_empty() {
                context_error(context, "begin render pass")
            } else {
                pass_error
            })
        })?;

    for draw in &authored.draws {
        let pipeline = pipelines
            .get(draw.pipeline_index as usize)
            .ok_or_else(|| rejected("draw references an absent pipeline snapshot"))?;
        context.clearLastError();
        pass.setPipeline(Some(&pipeline.pipeline));
        let pipeline_error = context.lastError();
        if !pipeline_error.is_empty() {
            return Err(rejected(format!("setPipeline: {pipeline_error}")));
        }
        for (slot, buffer) in &pipeline.vertex_buffers {
            pass.setVertexBuffer(*slot, Some(buffer), 0);
        }
        if let Some((format, buffer)) = &pipeline.index_buffer {
            pass.setIndexBuffer(Some(buffer), *format, 0);
        }
        for (group_index, group) in pipeline.groups.iter().enumerate() {
            if let Some(group) = group {
                pass.setBindGroup(
                    u32::try_from(group_index).map_err(|_| rejected("group index exceeds u32"))?,
                    Some(group),
                    None,
                    0,
                );
            }
        }
        apply_pass_state(pass.as_mut(), draw, width, height)?;
        if let Some(indexed) = &draw.indexed_draw {
            if pipeline.index_buffer.is_none() {
                return Err(rejected("indexed draw has no index buffer"));
            }
            pass.drawIndexed(
                indexed.index_count,
                indexed.instance_count,
                indexed.first_index,
                indexed.base_vertex,
                indexed.first_instance,
            );
        } else {
            pass.draw(
                draw.vertex_count,
                draw.instance_count,
                draw.first_vertex,
                draw.first_instance,
            );
        }
    }
    pass.finish();
    Ok(())
}

fn apply_pass_state(
    pass: &mut dyn nuxie_ore_metal::render_pass::RenderPassApi,
    draw: &GpuCanvasDrawCommand,
    width: u32,
    height: u32,
) -> Result<(), GpuCanvasError> {
    let viewport = draw
        .pass_state
        .viewport
        .unwrap_or([0.0, 0.0, width as f32, height as f32]);
    if !viewport.into_iter().all(f32::is_finite) || viewport[2] <= 0.0 || viewport[3] <= 0.0 {
        return Err(rejected("viewport is non-finite or empty"));
    }
    pass.setViewport(viewport[0], viewport[1], viewport[2], viewport[3], 0.0, 1.0);
    if let Some([x, y, width, height]) = draw.pass_state.scissor_rect {
        pass.setScissorRect(x, y, width, height);
    }
    pass.setStencilReference(draw.pass_state.stencil_reference);
    let [r, g, b, a] = draw.pass_state.blend_color;
    if ![r, g, b, a].into_iter().all(f64::is_finite) {
        return Err(rejected("blend color is non-finite"));
    }
    pass.setBlendColor(r as f32, g as f32, b as f32, a as f32);
    Ok(())
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AttachmentViewKey {
    resource_id: u64,
    view_dimension: String,
    base_mip_level: u32,
    mip_count: u32,
    base_array_layer: u32,
    layer_count: u32,
}

impl AttachmentViewKey {
    fn from_binding(binding: &GpuCanvasTextureBinding) -> Self {
        Self {
            resource_id: binding.resource_id,
            view_dimension: binding.view_dimension.clone(),
            base_mip_level: binding.base_mip_level,
            mip_count: binding.mip_level_count_in_view,
            base_array_layer: binding.base_array_layer,
            layer_count: binding.array_layer_count,
        }
    }
}

fn retain_attachment_view(
    context: &mut dyn ContextApi,
    retained_textures: &BTreeMap<u64, RetainedTextureResource>,
    view: &GpuCanvasAttachmentView,
    attachment_views: &mut BTreeMap<AttachmentViewKey, AnyResourceHandle>,
) -> Result<(), GpuCanvasError> {
    let GpuCanvasAttachmentView::Texture(binding) = view else {
        return Ok(());
    };
    let key = AttachmentViewKey::from_binding(binding);
    if attachment_views.contains_key(&key) {
        return Ok(());
    }
    let resource = retained_textures
        .get(&binding.resource_id)
        .ok_or_else(|| rejected("attachment texture is not retained"))?;
    let RetainedTextureResource::AuthoredTexture(texture) = resource else {
        return Err(rejected(
            "Image:view() cannot be used as a render attachment",
        ));
    };
    let view = make_texture_view(context, texture, binding)?;
    attachment_views.insert(key, view);
    Ok(())
}

fn resolve_attachment<'a>(
    view: &GpuCanvasAttachmentView,
    canvas_view: &'a AnyResourceHandle,
    attachment_views: &'a BTreeMap<AttachmentViewKey, AnyResourceHandle>,
) -> Result<&'a AnyResourceHandle, GpuCanvasError> {
    match view {
        GpuCanvasAttachmentView::Canvas => Ok(canvas_view),
        GpuCanvasAttachmentView::Texture(binding) => attachment_views
            .get(&AttachmentViewKey::from_binding(binding))
            .ok_or_else(|| rejected("attachment view is absent")),
    }
}

fn depth_attachment<'a>(
    attachment: &'a GpuCanvasDepthStencilAttachment,
    canvas_view: &'a AnyResourceHandle,
    attachment_views: &'a BTreeMap<AttachmentViewKey, AnyResourceHandle>,
) -> Result<DepthStencilAttachment<'a>, GpuCanvasError> {
    if !attachment.depth_clear_value.is_finite() {
        return Err(rejected("depth clear value is non-finite"));
    }
    Ok(DepthStencilAttachment {
        view: Some(resolve_attachment(
            &attachment.view,
            canvas_view,
            attachment_views,
        )?),
        depthLoadOp: load_op(&attachment.depth_load_op)?,
        depthStoreOp: store_op(&attachment.depth_store_op)?,
        depthClearValue: attachment.depth_clear_value,
        ..DepthStencilAttachment::default()
    })
}

fn make_texture_view(
    context: &mut dyn ContextApi,
    texture: &AnyResourceHandle,
    binding: &GpuCanvasTextureBinding,
) -> Result<AnyResourceHandle, GpuCanvasError> {
    context
        .makeTextureView(&TextureViewDesc {
            texture: Some(texture),
            dimension: texture_view_dimension(&binding.view_dimension)?,
            aspect: TextureAspect::all,
            baseMipLevel: binding.base_mip_level,
            mipCount: binding.mip_level_count_in_view,
            baseLayer: binding.base_array_layer,
            layerCount: binding.array_layer_count,
        })
        .ok_or_else(|| rejected(context_error(context, "create texture view")))
}

fn collect_texture_specs(
    pipelines: &[GpuCanvasPipelinePlan],
    passes: &[GpuCanvasRenderPass],
) -> Result<BTreeMap<u64, GpuCanvasTextureBinding>, GpuCanvasError> {
    let attachment_bindings = passes.iter().flat_map(|pass| {
        pass.color_attachments
            .iter()
            .flat_map(|attachment| {
                std::iter::once(&attachment.view).chain(attachment.resolve_target.iter())
            })
            .chain(
                pass.depth_stencil_attachment
                    .iter()
                    .map(|attachment| &attachment.view),
            )
    });
    let bindings = pipelines
        .iter()
        .flat_map(|pipeline| pipeline.texture_bindings.iter())
        .chain(attachment_bindings.filter_map(|view| match view {
            GpuCanvasAttachmentView::Canvas => None,
            GpuCanvasAttachmentView::Texture(binding) => Some(binding),
        }));
    let mut specs = BTreeMap::<u64, GpuCanvasTextureBinding>::new();
    for binding in bindings {
        if binding.resource_id == 0 {
            return Err(rejected("authored texture resource id must be nonzero"));
        }
        match specs.get_mut(&binding.resource_id) {
            None => {
                specs.insert(binding.resource_id, binding.clone());
            }
            Some(existing) => {
                if existing.lifetime != binding.lifetime
                    || TextureIdentity::from_binding(existing)
                        != TextureIdentity::from_binding(binding)
                    || !same_external_image(
                        existing.external_image.as_ref(),
                        binding.external_image.as_ref(),
                    )
                {
                    return Err(rejected(format!(
                        "texture {} has inconsistent descriptors",
                        binding.resource_id
                    )));
                }
                if binding.uploads.starts_with(&existing.uploads) {
                    existing.uploads = binding.uploads.clone();
                } else if !existing.uploads.starts_with(&binding.uploads) {
                    return Err(rejected(format!(
                        "texture {} has divergent upload histories",
                        binding.resource_id
                    )));
                }
            }
        }
    }
    Ok(specs)
}

fn materialize_pipeline_plans(plan: &GpuCanvasPlan) -> Vec<GpuCanvasPipelinePlan> {
    if !plan.pipelines.is_empty() {
        return plan.pipelines.clone();
    }
    vec![GpuCanvasPipelinePlan {
        vertex_entry: plan.vertex_entry.clone(),
        fragment_entry: plan.fragment_entry.clone(),
        uniform_buffers: plan.uniform_buffers.clone(),
        vertex_layouts: plan.vertex_layouts.clone(),
        vertex_buffers: plan.vertex_buffers.clone(),
        index_buffer: plan.index_buffer.clone(),
        texture_bindings: plan.texture_bindings.clone(),
        sampler_bindings: plan.sampler_bindings.clone(),
        pipeline_state: plan.pipeline_state.clone(),
    }]
}

fn materialize_render_passes(plan: &GpuCanvasPlan) -> Vec<GpuCanvasRenderPass> {
    if !plan.render_passes.is_empty() {
        return plan.render_passes.clone();
    }
    vec![GpuCanvasRenderPass {
        color_attachments: vec![GpuCanvasColorAttachment {
            view: GpuCanvasAttachmentView::Canvas,
            resolve_target: None,
            load_op: "clear".into(),
            store_op: "store".into(),
            clear_color: plan.clear_color,
        }],
        depth_stencil_attachment: None,
        draws: vec![GpuCanvasDrawCommand {
            pipeline_index: 0,
            vertex_count: plan.vertex_count,
            instance_count: plan.instance_count,
            first_vertex: plan.first_vertex,
            first_instance: plan.first_instance,
            indexed_draw: plan.indexed_draw.clone(),
            pass_state: plan.pass_state.clone(),
        }],
    }]
}

fn validate_submission(plan: &GpuCanvasPlan) -> Result<(), GpuCanvasError> {
    if plan.width == 0
        || plan.height == 0
        || plan.width > MAX_GPU_CANVAS_DIMENSION
        || plan.height > MAX_GPU_CANVAS_DIMENSION
    {
        return Err(rejected(format!(
            "dimensions must be between 1 and {MAX_GPU_CANVAS_DIMENSION}"
        )));
    }
    let pipelines = materialize_pipeline_plans(plan);
    let passes = materialize_render_passes(plan);
    for pipeline in &pipelines {
        validate_pipeline_plan(pipeline)?;
    }
    for pass in &passes {
        if pass.color_attachments.is_empty() && pass.depth_stencil_attachment.is_none() {
            return Err(rejected("render pass has no attachment"));
        }
        for draw in &pass.draws {
            let pipeline = pipelines
                .get(draw.pipeline_index as usize)
                .ok_or_else(|| rejected("draw references an absent pipeline"))?;
            validate_draw(draw, pipeline)?;
        }
    }
    Ok(())
}

fn validate_pipeline_plan(plan: &GpuCanvasPipelinePlan) -> Result<(), GpuCanvasError> {
    let mut bindings = BTreeSet::new();
    for uniform in &plan.uniform_buffers {
        if uniform.group >= kMaxBindGroups || uniform.binding > MAX_BINDING_INDEX {
            return Err(rejected("uniform group or binding is out of range"));
        }
        if uniform.bytes.is_empty()
            || uniform.bytes.len() > MAX_UNIFORM_BUFFER_BYTES
            || uniform.bytes.len() % 4 != 0
        {
            return Err(rejected("uniform buffer size is invalid"));
        }
        if !bindings.insert((uniform.group, uniform.binding)) {
            return Err(rejected("duplicate uniform binding"));
        }
    }
    for texture in &plan.texture_bindings {
        if texture.group >= kMaxBindGroups || texture.binding > MAX_BINDING_INDEX {
            return Err(rejected("texture group or binding is out of range"));
        }
        if !bindings.insert((texture.group, texture.binding)) {
            return Err(rejected("duplicate texture binding"));
        }
    }
    for sampler in &plan.sampler_bindings {
        if sampler.group >= kMaxBindGroups || sampler.binding > MAX_BINDING_INDEX {
            return Err(rejected("sampler group or binding is out of range"));
        }
        if !bindings.insert((sampler.group, sampler.binding)) {
            return Err(rejected("duplicate sampler binding"));
        }
    }
    if plan.vertex_layouts.len() != plan.vertex_buffers.len()
        || plan.vertex_layouts.len() > MAX_VERTEX_BUFFERS
    {
        return Err(rejected("vertex layout and buffer counts do not match"));
    }
    let mut locations = BTreeSet::new();
    let mut slots = BTreeSet::new();
    let mut attribute_count = 0;
    for buffer in &plan.vertex_buffers {
        if buffer.slot as usize >= MAX_VERTEX_BUFFERS
            || buffer.bytes.is_empty()
            || buffer.bytes.len() > MAX_VERTEX_BUFFER_BYTES
            || !slots.insert(buffer.slot)
        {
            return Err(rejected("vertex buffer descriptor is invalid"));
        }
    }
    for layout in &plan.vertex_layouts {
        if layout.stride == 0 || layout.stride > 2_048 || layout.attributes.is_empty() {
            return Err(rejected("vertex layout is empty or has an invalid stride"));
        }
        for attribute in &layout.attributes {
            attribute_count += 1;
            if attribute_count > MAX_VERTEX_ATTRIBUTES
                || attribute.shader_location >= MAX_VERTEX_ATTRIBUTES as u32
                || !locations.insert(attribute.shader_location)
                || attribute
                    .offset
                    .checked_add(vertex_format_size(&attribute.format)?)
                    .is_none_or(|end| end > layout.stride)
            {
                return Err(rejected("vertex attribute is out of range or duplicated"));
            }
        }
    }
    Ok(())
}

fn validate_draw(
    draw: &GpuCanvasDrawCommand,
    pipeline: &GpuCanvasPipelinePlan,
) -> Result<(), GpuCanvasError> {
    let (count, instances) = if let Some(indexed) = &draw.indexed_draw {
        let buffer = pipeline
            .index_buffer
            .as_ref()
            .ok_or_else(|| rejected("indexed draw requires an index buffer"))?;
        let bytes_per_index = match buffer.format.as_str() {
            "uint16" => 2_u64,
            "uint32" => 4_u64,
            _ => return Err(rejected("index format is invalid")),
        };
        let required = u64::from(indexed.first_index)
            .checked_add(u64::from(indexed.index_count))
            .and_then(|count| count.checked_mul(bytes_per_index))
            .ok_or_else(|| rejected("index range overflow"))?;
        if required > buffer.bytes.len() as u64 {
            return Err(rejected("indexed draw exceeds the index buffer"));
        }
        (indexed.index_count, indexed.instance_count)
    } else {
        (draw.vertex_count, draw.instance_count)
    };
    if count == 0
        || instances == 0
        || u64::from(count)
            .checked_mul(u64::from(instances))
            .is_none_or(|count| count > MAX_DRAW_INVOCATIONS)
    {
        return Err(rejected(
            "draw count is empty or exceeds the invocation limit",
        ));
    }
    Ok(())
}

fn context_error(context: &dyn ContextApi, fallback: &str) -> String {
    let error = context.lastError();
    if error.is_empty() {
        fallback.to_owned()
    } else {
        error
    }
}

fn clear_color(color: [f64; 4]) -> Result<ClearColor, GpuCanvasError> {
    if !color
        .into_iter()
        .all(|channel| channel.is_finite() && (0.0..=1.0).contains(&channel))
    {
        return Err(rejected(
            "clear color must contain normalized finite values",
        ));
    }
    Ok(ClearColor {
        r: color[0] as f32,
        g: color[1] as f32,
        b: color[2] as f32,
        a: color[3] as f32,
    })
}

fn color_targets(
    targets: &[nuxie_render_api::GpuCanvasColorTarget],
) -> Result<([ColorTargetState; 4], u32), GpuCanvasError> {
    if targets.len() > 4 {
        return Err(rejected("pipeline has more than four color targets"));
    }
    let mut output = [ColorTargetState::default(); 4];
    for (destination, target) in output.iter_mut().zip(targets) {
        *destination = ColorTargetState {
            format: texture_format(&target.format)?,
            blendEnabled: target.blend.is_some(),
            blend: target
                .blend
                .as_ref()
                .map(blend_state)
                .transpose()?
                .unwrap_or_default(),
            writeMask: color_write_mask(&target.write_mask)?,
        };
    }
    Ok((
        output,
        u32::try_from(targets.len()).map_err(|_| rejected("color target count exceeds u32"))?,
    ))
}

fn blend_state(state: &GpuCanvasBlendState) -> Result<BlendState, GpuCanvasError> {
    Ok(BlendState {
        srcColor: blend_factor(&state.src_color)?,
        dstColor: blend_factor(&state.dst_color)?,
        colorOp: blend_op(&state.color_op)?,
        srcAlpha: blend_factor(&state.src_alpha)?,
        dstAlpha: blend_factor(&state.dst_alpha)?,
        alphaOp: blend_op(&state.alpha_op)?,
    })
}

fn depth_stencil(
    state: Option<&GpuCanvasDepthStencilState>,
) -> Result<
    (
        DepthStencilState,
        StencilFaceState,
        StencilFaceState,
        u8,
        u8,
    ),
    GpuCanvasError,
> {
    let Some(state) = state else {
        return Ok((
            DepthStencilState::default(),
            StencilFaceState::default(),
            StencilFaceState::default(),
            0xff,
            0xff,
        ));
    };
    Ok((
        DepthStencilState {
            format: texture_format(&state.format)?,
            depthCompare: compare_function(&state.depth_compare)?,
            depthWriteEnabled: state.depth_write_enabled,
            depthBias: state.depth_bias,
            depthBiasSlopeScale: state.depth_bias_slope_scale,
            depthBiasClamp: state.depth_bias_clamp,
        },
        stencil_face(&state.stencil_front)?,
        stencil_face(&state.stencil_back)?,
        u8::try_from(state.stencil_read_mask)
            .map_err(|_| rejected("stencil read mask exceeds u8"))?,
        u8::try_from(state.stencil_write_mask)
            .map_err(|_| rejected("stencil write mask exceeds u8"))?,
    ))
}

fn stencil_face(face: &GpuCanvasStencilFace) -> Result<StencilFaceState, GpuCanvasError> {
    Ok(StencilFaceState {
        compare: compare_function(&face.compare)?,
        failOp: stencil_op(&face.fail_op)?,
        depthFailOp: stencil_op(&face.depth_fail_op)?,
        passOp: stencil_op(&face.pass_op)?,
    })
}

fn binding_kind(kind: GpuCanvasShaderResourceKind) -> Result<BindingKind, GpuCanvasError> {
    Ok(match kind {
        GpuCanvasShaderResourceKind::UniformBuffer => BindingKind::uniformBuffer,
        GpuCanvasShaderResourceKind::StorageBufferReadOnly => BindingKind::storageBufferRO,
        GpuCanvasShaderResourceKind::StorageBufferReadWrite => BindingKind::storageBufferRW,
        GpuCanvasShaderResourceKind::SampledTexture => BindingKind::sampledTexture,
        GpuCanvasShaderResourceKind::StorageTexture => BindingKind::storageTexture,
        GpuCanvasShaderResourceKind::Sampler => BindingKind::sampler,
        GpuCanvasShaderResourceKind::ComparisonSampler => BindingKind::comparisonSampler,
    })
}

fn binding_texture_dimension(
    dimension: GpuCanvasShaderTextureViewDimension,
) -> Result<TextureViewDimension, GpuCanvasError> {
    match dimension {
        GpuCanvasShaderTextureViewDimension::Undefined
        | GpuCanvasShaderTextureViewDimension::D1 => {
            Err(rejected("ORE does not expose 1D texture views"))
        }
        GpuCanvasShaderTextureViewDimension::D2 => Ok(TextureViewDimension::texture2D),
        GpuCanvasShaderTextureViewDimension::D2Array => Ok(TextureViewDimension::array2D),
        GpuCanvasShaderTextureViewDimension::Cube => Ok(TextureViewDimension::cube),
        GpuCanvasShaderTextureViewDimension::CubeArray => Ok(TextureViewDimension::cubeArray),
        GpuCanvasShaderTextureViewDimension::D3 => Ok(TextureViewDimension::texture3D),
    }
}

fn binding_sample_type(
    sample_type: GpuCanvasShaderTextureSampleType,
) -> Result<SampleType, GpuCanvasError> {
    match sample_type {
        GpuCanvasShaderTextureSampleType::Undefined => {
            Err(rejected("sampled texture has no sample type"))
        }
        GpuCanvasShaderTextureSampleType::Float => Ok(SampleType::floatFilterable),
        GpuCanvasShaderTextureSampleType::UnfilterableFloat => Ok(SampleType::floatUnfilterable),
        GpuCanvasShaderTextureSampleType::Depth => Ok(SampleType::depth),
        GpuCanvasShaderTextureSampleType::Sint => Ok(SampleType::sint),
        GpuCanvasShaderTextureSampleType::Uint => Ok(SampleType::uint),
    }
}

fn texture_format(value: &str) -> Result<TextureFormat, GpuCanvasError> {
    match value {
        "r8unorm" => Ok(TextureFormat::r8unorm),
        "rg8unorm" => Ok(TextureFormat::rg8unorm),
        "rgba8unorm" => Ok(TextureFormat::rgba8unorm),
        "rgba8snorm" => Ok(TextureFormat::rgba8snorm),
        "bgra8unorm" => Ok(TextureFormat::bgra8unorm),
        "rgba16float" => Ok(TextureFormat::rgba16float),
        "rg16float" => Ok(TextureFormat::rg16float),
        "r16float" => Ok(TextureFormat::r16float),
        "rgba32float" => Ok(TextureFormat::rgba32float),
        "rg32float" => Ok(TextureFormat::rg32float),
        "r32float" => Ok(TextureFormat::r32float),
        "rgb10a2unorm" => Ok(TextureFormat::rgb10a2unorm),
        "rg11b10ufloat" | "r11g11b10float" => Ok(TextureFormat::r11g11b10float),
        "depth16unorm" => Ok(TextureFormat::depth16unorm),
        "depth24plus-stencil8" => Ok(TextureFormat::depth24plusStencil8),
        "depth32float" => Ok(TextureFormat::depth32float),
        "depth32float-stencil8" => Ok(TextureFormat::depth32floatStencil8),
        "bc1-rgba-unorm" => Ok(TextureFormat::bc1unorm),
        "bc3-rgba-unorm" => Ok(TextureFormat::bc3unorm),
        "bc7-rgba-unorm" => Ok(TextureFormat::bc7unorm),
        "etc2-rgb8unorm" => Ok(TextureFormat::etc2rgb8),
        "etc2-rgba8unorm" => Ok(TextureFormat::etc2rgba8),
        "astc-4x4-unorm" => Ok(TextureFormat::astc4x4),
        "astc-6x6-unorm" => Ok(TextureFormat::astc6x6),
        "astc-8x8-unorm" => Ok(TextureFormat::astc8x8),
        _ => Err(rejected(format!("unsupported texture format '{value}'"))),
    }
}

fn texture_type(value: &str) -> Result<TextureType, GpuCanvasError> {
    match value {
        "2d" => Ok(TextureType::texture2D),
        "cube" => Ok(TextureType::cube),
        "3d" => Ok(TextureType::texture3D),
        "2d-array" => Ok(TextureType::array2D),
        _ => Err(rejected(format!("unsupported texture type '{value}'"))),
    }
}

fn texture_view_dimension(value: &str) -> Result<TextureViewDimension, GpuCanvasError> {
    match value {
        "2d" => Ok(TextureViewDimension::texture2D),
        "cube" => Ok(TextureViewDimension::cube),
        "3d" => Ok(TextureViewDimension::texture3D),
        "2d-array" => Ok(TextureViewDimension::array2D),
        "cube-array" => Ok(TextureViewDimension::cubeArray),
        _ => Err(rejected(format!("unsupported texture view '{value}'"))),
    }
}

fn vertex_format_size(value: &str) -> Result<u64, GpuCanvasError> {
    match value {
        "float32" => Ok(4),
        "float32x2" => Ok(8),
        "float32x3" => Ok(12),
        "float32x4" => Ok(16),
        "uint8x4" | "sint8x4" | "unorm8x4" | "snorm8x4" | "float16x2" => Ok(4),
        "uint16x2" | "sint16x2" | "unorm16x2" | "snorm16x2" => Ok(4),
        "uint16x4" | "sint16x4" | "float16x4" => Ok(8),
        "uint32" => Ok(4),
        _ => Err(rejected(format!("unsupported vertex format '{value}'"))),
    }
}

fn vertex_format(value: &str) -> Result<VertexFormat, GpuCanvasError> {
    match value {
        "float32" => Ok(VertexFormat::float1),
        "float32x2" => Ok(VertexFormat::float2),
        "float32x3" => Ok(VertexFormat::float3),
        "float32x4" => Ok(VertexFormat::float4),
        "uint8x4" => Ok(VertexFormat::uint8x4),
        "sint8x4" => Ok(VertexFormat::sint8x4),
        "unorm8x4" => Ok(VertexFormat::unorm8x4),
        "snorm8x4" => Ok(VertexFormat::snorm8x4),
        "uint16x2" => Ok(VertexFormat::uint16x2),
        "sint16x2" => Ok(VertexFormat::sint16x2),
        "unorm16x2" => Ok(VertexFormat::unorm16x2),
        "snorm16x2" => Ok(VertexFormat::snorm16x2),
        "uint16x4" => Ok(VertexFormat::uint16x4),
        "sint16x4" => Ok(VertexFormat::sint16x4),
        "float16x2" => Ok(VertexFormat::float16x2),
        "float16x4" => Ok(VertexFormat::float16x4),
        "uint32" => Ok(VertexFormat::uint32),
        _ => Err(rejected(format!("unsupported vertex format '{value}'"))),
    }
}

macro_rules! string_enum {
    ($name:ident, $ty:ty, {$($wire:literal => $value:path),+ $(,)?}) => {
        fn $name(value: &str) -> Result<$ty, GpuCanvasError> {
            match value {
                $($wire => Ok($value),)+
                _ => Err(rejected(format!("unsupported {} '{value}'", stringify!($name)))),
            }
        }
    };
}

string_enum!(vertex_step_mode, VertexStepMode, {
    "vertex" => VertexStepMode::vertex,
    "instance" => VertexStepMode::instance,
});
string_enum!(index_format, IndexFormat, {
    "uint16" => IndexFormat::uint16,
    "uint32" => IndexFormat::uint32,
});
string_enum!(primitive_topology, PrimitiveTopology, {
    "point-list" => PrimitiveTopology::pointList,
    "line-list" => PrimitiveTopology::lineList,
    "line-strip" => PrimitiveTopology::lineStrip,
    "triangle-list" => PrimitiveTopology::triangleList,
    "triangle-strip" => PrimitiveTopology::triangleStrip,
});
string_enum!(cull_mode, CullMode, {
    "none" => CullMode::none,
    "front" => CullMode::front,
    "back" => CullMode::back,
});
string_enum!(winding, nuxie_ore_metal::types::FaceWinding, {
    "cw" => nuxie_ore_metal::types::FaceWinding::clockwise,
    "ccw" => nuxie_ore_metal::types::FaceWinding::counterClockwise,
});
string_enum!(filter, Filter, {
    "nearest" => Filter::nearest,
    "linear" => Filter::linear,
});
string_enum!(wrap_mode, WrapMode, {
    "repeat" => WrapMode::repeat,
    "mirror-repeat" => WrapMode::mirrorRepeat,
    "clamp-to-edge" => WrapMode::clampToEdge,
});
string_enum!(compare_function, CompareFunction, {
    "never" => CompareFunction::never,
    "less" => CompareFunction::less,
    "equal" => CompareFunction::equal,
    "less-equal" => CompareFunction::lessEqual,
    "greater" => CompareFunction::greater,
    "not-equal" => CompareFunction::notEqual,
    "greater-equal" => CompareFunction::greaterEqual,
    "always" => CompareFunction::always,
});
string_enum!(blend_factor, BlendFactor, {
    "zero" => BlendFactor::zero,
    "one" => BlendFactor::one,
    "src" => BlendFactor::srcColor,
    "one-minus-src" => BlendFactor::oneMinusSrcColor,
    "src-alpha" => BlendFactor::srcAlpha,
    "one-minus-src-alpha" => BlendFactor::oneMinusSrcAlpha,
    "dst" => BlendFactor::dstColor,
    "one-minus-dst" => BlendFactor::oneMinusDstColor,
    "dst-alpha" => BlendFactor::dstAlpha,
    "one-minus-dst-alpha" => BlendFactor::oneMinusDstAlpha,
    "src-alpha-saturated" => BlendFactor::srcAlphaSaturated,
    "constant" => BlendFactor::blendColor,
    "one-minus-constant" => BlendFactor::oneMinusBlendColor,
});
string_enum!(blend_op, BlendOp, {
    "add" => BlendOp::add,
    "subtract" => BlendOp::subtract,
    "reverse-subtract" => BlendOp::reverseSubtract,
    "min" => BlendOp::min,
    "max" => BlendOp::max,
});
string_enum!(stencil_op, StencilOp, {
    "keep" => StencilOp::keep,
    "zero" => StencilOp::zero,
    "replace" => StencilOp::replace,
    "increment-clamp" => StencilOp::incrementClamp,
    "decrement-clamp" => StencilOp::decrementClamp,
    "invert" => StencilOp::invert,
    "increment-wrap" => StencilOp::incrementWrap,
    "decrement-wrap" => StencilOp::decrementWrap,
});
string_enum!(load_op, LoadOp, {
    "clear" => LoadOp::clear,
    "load" => LoadOp::load,
    "dont-care" => LoadOp::dontCare,
});
string_enum!(store_op, StoreOp, {
    "store" => StoreOp::store,
    "discard" => StoreOp::discard,
});

fn color_write_mask(value: &str) -> Result<ColorWriteMask, GpuCanvasError> {
    let mut bits = 0_u8;
    for channel in value.chars() {
        bits |= match channel {
            'r' => ColorWriteMask::red.bits(),
            'g' => ColorWriteMask::green.bits(),
            'b' => ColorWriteMask::blue.bits(),
            'a' => ColorWriteMask::alpha.bits(),
            _ => return Err(rejected(format!("invalid color write mask '{value}'"))),
        };
    }
    Ok(ColorWriteMask::from_bits(bits))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_texture_layout_entries_accept_undefined_texture_metadata() {
        let binding = GpuCanvasShaderBinding {
            group: 0,
            binding: 7,
            kind: GpuCanvasShaderResourceKind::UniformBuffer,
            stage_mask: StageVisibility::kVertex,
            backend_space: 0,
            backend_slots: [Some(7), None, None],
            texture_view_dimension: GpuCanvasShaderTextureViewDimension::Undefined,
            texture_sample_type: GpuCanvasShaderTextureSampleType::Undefined,
            texture_multisampled: false,
        };

        let entry = layout_entry(&binding).expect("uniform layout entry");

        assert_eq!(entry.kind, BindingKind::uniformBuffer);
        assert_eq!(entry.textureViewDim, TextureViewDimension::texture2D);
        assert_eq!(entry.textureSampleType, SampleType::floatFilterable);
    }

    #[test]
    fn fragment_selection_matches_all_three_upstream_cases() {
        assert_eq!(fragment_selection(true, 0), FragmentSelection::Explicit);
        assert_eq!(fragment_selection(true, 1), FragmentSelection::Explicit);
        assert_eq!(
            fragment_selection(false, 1),
            FragmentSelection::CombinedVertex
        );
        assert_eq!(fragment_selection(false, 0), FragmentSelection::None);
    }
}
