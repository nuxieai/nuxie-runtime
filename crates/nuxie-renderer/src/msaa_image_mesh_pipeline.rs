//! MSAA image meshes translated from C++ `DrawType::imageMesh`.

use crate::work_metrics::CountedDeviceExt;
use crate::{
    atomic_pipeline::image_sampler,
    gpu::{FlushUniforms, ImageDrawInstance},
};
use nuxie_render_api::ImageSampler;
use std::sync::Arc;

pub(crate) struct MsaaImageMeshPipeline {
    fixed: PipelineVariants,
    advanced: PipelineVariants,
    advanced_hsl: PipelineVariants,
    flush_layout: wgpu::BindGroupLayout,
    image_layout: wgpu::BindGroupLayout,
    _dummy_destination: wgpu::Texture,
    dummy_destination_view: wgpu::TextureView,
}

struct PipelineVariants {
    unclipped: wgpu::RenderPipeline,
    unclipped_rect: Option<wgpu::RenderPipeline>,
    path_clip: wgpu::RenderPipeline,
    path_clip_rect: Option<wgpu::RenderPipeline>,
}

pub(crate) struct PreparedImageMesh {
    pub flush_group: wgpu::BindGroup,
    pub image_group: wgpu::BindGroup,
    pub vertices: Arc<wgpu::Buffer>,
    pub uvs: Arc<wgpu::Buffer>,
    pub indices: Arc<wgpu::Buffer>,
    pub index_count: u32,
    pub instance_index: u32,
}

pub(crate) struct PreparedImageMeshResources {
    fixed_flush_group: wgpu::BindGroup,
    advanced_flush_group: wgpu::BindGroup,
}

impl MsaaImageMeshPipeline {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let no_clip_vertex = shader(
            device,
            "nuxie-msaa-image-mesh-vertex",
            include_str!("generated/draw_msaa_image_mesh.webgpu_noclipdistance_vert.wgsl"),
        );
        let clip_rect_vertex = device
            .features()
            .contains(wgpu::Features::CLIP_DISTANCES)
            .then(|| {
                shader(
                    device,
                    "nuxie-msaa-image-mesh-clip-rect-vertex",
                    include_str!("generated/draw_msaa_image_mesh.webgpu_vert.wgsl"),
                )
            });
        let fixed_fragment = shader(
            device,
            "nuxie-msaa-image-mesh-fragment",
            include_str!("generated/draw_msaa_image_mesh.webgpu_fixedcolor_frag.wgsl"),
        );
        let advanced_fragment = shader(
            device,
            "nuxie-msaa-image-mesh-advanced-fragment",
            include_str!("generated/draw_msaa_image_mesh.webgpu_frag.wgsl"),
        );
        let flush_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-msaa-image-mesh-flush-layout"),
            entries: &[
                uniform_entry(0),
                texture_entry(12, wgpu::TextureSampleType::Float { filterable: false }),
            ],
        });
        let image_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-msaa-image-mesh-image-layout"),
            entries: &[
                texture_entry(11, wgpu::TextureSampleType::Float { filterable: true }),
                sampler_entry(13),
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nuxie-msaa-image-mesh-pipeline-layout"),
            bind_group_layouts: &[Some(&flush_layout), Some(&image_layout)],
            immediate_size: 0,
        });
        let create_pipeline = |label,
                               vertex: &wgpu::ShaderModule,
                               fragment: &wgpu::ShaderModule,
                               clip_rect,
                               path_clip,
                               advanced,
                               hsl| {
            let vertex_constants: &[(&str, f64)] = if clip_rect {
                &[("0", path_clip as u8 as f64), ("1", 1.0)]
            } else {
                &[("0", path_clip as u8 as f64)]
            };
            let fixed_constants = [("7", 1.0)];
            let advanced_constants = [("6", hsl as u8 as f64), ("7", 1.0)];
            let fragment_constants = if advanced {
                &advanced_constants[..]
            } else {
                &fixed_constants[..]
            };
            let keep = wgpu::StencilOperation::Keep;
            let stencil_face = if path_clip {
                wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Equal,
                    fail_op: keep,
                    depth_fail_op: keep,
                    pass_op: keep,
                }
            } else {
                wgpu::StencilFaceState {
                    compare: wgpu::CompareFunction::Always,
                    fail_op: keep,
                    depth_fail_op: keep,
                    pass_op: keep,
                }
            };
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: vertex,
                    entry_point: Some("main"),
                    compilation_options: options(vertex_constants),
                    buffers: &[
                        Some(image_mesh_vertex_layout(0)),
                        Some(image_mesh_vertex_layout(1)),
                        Some(ImageDrawInstance::layout()),
                    ],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: wgpu::StencilState {
                        front: stencil_face,
                        back: stencil_face,
                        read_mask: if path_clip { 0xff } else { 0 },
                        write_mask: if path_clip { 0xff } else { 0 },
                    },
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 4,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: fragment,
                    entry_point: Some("main"),
                    compilation_options: options(fragment_constants),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let make_variants =
            |label: &'static str, fragment: &wgpu::ShaderModule, advanced, hsl| PipelineVariants {
                unclipped: create_pipeline(
                    label,
                    &no_clip_vertex,
                    fragment,
                    false,
                    false,
                    advanced,
                    hsl,
                ),
                unclipped_rect: clip_rect_vertex.as_ref().map(|vertex| {
                    create_pipeline(label, vertex, fragment, true, false, advanced, hsl)
                }),
                path_clip: create_pipeline(
                    label,
                    &no_clip_vertex,
                    fragment,
                    false,
                    true,
                    advanced,
                    hsl,
                ),
                path_clip_rect: clip_rect_vertex.as_ref().map(|vertex| {
                    create_pipeline(label, vertex, fragment, true, true, advanced, hsl)
                }),
            };
        let dummy_destination = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-msaa-image-mesh-dummy-destination"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let dummy_destination_view = dummy_destination.create_view(&Default::default());
        Self {
            fixed: make_variants(
                "nuxie-msaa-image-mesh-fixed-pipeline",
                &fixed_fragment,
                false,
                false,
            ),
            advanced: make_variants(
                "nuxie-msaa-image-mesh-advanced-pipeline",
                &advanced_fragment,
                true,
                false,
            ),
            advanced_hsl: make_variants(
                "nuxie-msaa-image-mesh-advanced-hsl-pipeline",
                &advanced_fragment,
                true,
                true,
            ),
            flush_layout,
            image_layout,
            _dummy_destination: dummy_destination,
            dummy_destination_view,
        }
    }

    pub(crate) fn supports_clip_rect(&self) -> bool {
        self.fixed.unclipped_rect.is_some()
    }

    pub(crate) fn pipeline(
        &self,
        path_clip: bool,
        clip_rect: bool,
        advanced: bool,
        hsl: bool,
    ) -> &wgpu::RenderPipeline {
        let variants = if advanced {
            if hsl {
                &self.advanced_hsl
            } else {
                &self.advanced
            }
        } else {
            &self.fixed
        };
        match (path_clip, clip_rect) {
            (false, false) => &variants.unclipped,
            (false, true) => variants
                .unclipped_rect
                .as_ref()
                .expect("image mesh clip rect prepared without clip distances"),
            (true, false) => &variants.path_clip,
            (true, true) => variants
                .path_clip_rect
                .as_ref()
                .expect("clipped image mesh clip rect prepared without clip distances"),
        }
    }

    pub(crate) fn prepare_resources(
        &self,
        device: &wgpu::Device,
        uniforms: &FlushUniforms,
        destination: Option<&wgpu::TextureView>,
    ) -> PreparedImageMeshResources {
        let uniform_buffer = upload(device, "nuxie-msaa-image-mesh-uniforms", uniforms);
        let make_flush_group = |destination: &wgpu::TextureView| {
            device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("nuxie-msaa-image-mesh-flush-group"),
                layout: &self.flush_layout,
                entries: &[
                    binding(0, uniform_buffer.as_entire_binding()),
                    binding(12, wgpu::BindingResource::TextureView(destination)),
                ],
            })
        };
        let fixed_flush_group = make_flush_group(&self.dummy_destination_view);
        let advanced_flush_group = destination
            .map(make_flush_group)
            .unwrap_or_else(|| fixed_flush_group.clone());
        PreparedImageMeshResources {
            fixed_flush_group,
            advanced_flush_group,
        }
    }

    pub(crate) fn prepare_image_group(
        &self,
        device: &wgpu::Device,
        image: &wgpu::TextureView,
        sampler: ImageSampler,
    ) -> wgpu::BindGroup {
        let sampler = device.create_sampler(&image_sampler(sampler));
        device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-msaa-image-mesh-image-group"),
            layout: &self.image_layout,
            entries: &[
                binding(11, wgpu::BindingResource::TextureView(image)),
                binding(13, wgpu::BindingResource::Sampler(&sampler)),
            ],
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &self,
        resources: &PreparedImageMeshResources,
        image_group: &wgpu::BindGroup,
        advanced_blend: bool,
        vertices: &Arc<wgpu::Buffer>,
        uvs: &Arc<wgpu::Buffer>,
        indices: &Arc<wgpu::Buffer>,
        index_count: u32,
        instance_index: u32,
    ) -> PreparedImageMesh {
        PreparedImageMesh {
            flush_group: if advanced_blend {
                resources.advanced_flush_group.clone()
            } else {
                resources.fixed_flush_group.clone()
            },
            image_group: image_group.clone(),
            vertices: Arc::clone(vertices),
            uvs: Arc::clone(uvs),
            indices: Arc::clone(indices),
            index_count,
            instance_index,
        }
    }
}

fn shader(device: &wgpu::Device, label: &'static str, source: &'static str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

fn upload<T: bytemuck::Pod>(device: &wgpu::Device, label: &'static str, value: &T) -> wgpu::Buffer {
    device.create_counted_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::bytes_of(value),
        usage: wgpu::BufferUsages::UNIFORM,
    })
}

const IMAGE_MESH_POSITION_ATTRIBUTE: [wgpu::VertexAttribute; 1] = [wgpu::VertexAttribute {
    format: wgpu::VertexFormat::Float32x2,
    offset: 0,
    shader_location: 0,
}];
const IMAGE_MESH_UV_ATTRIBUTE: [wgpu::VertexAttribute; 1] = [wgpu::VertexAttribute {
    format: wgpu::VertexFormat::Float32x2,
    offset: 0,
    shader_location: 1,
}];

fn image_mesh_vertex_layout(shader_location: u32) -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<[f32; 2]>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: match shader_location {
            0 => &IMAGE_MESH_POSITION_ATTRIBUTE,
            1 => &IMAGE_MESH_UV_ATTRIBUTE,
            _ => unreachable!("image mesh only has position and UV streams"),
        },
    }
}

fn options<'a>(constants: &'a [(&'a str, f64)]) -> wgpu::PipelineCompilationOptions<'a> {
    wgpu::PipelineCompilationOptions {
        constants,
        ..Default::default()
    }
}

fn binding(binding: u32, resource: wgpu::BindingResource<'_>) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry { binding, resource }
}

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn texture_entry(binding: u32, sample_type: wgpu::TextureSampleType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    }
}
