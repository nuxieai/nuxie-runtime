//! MSAA stencil reset translated from Rive's WebGPU renderer.

use crate::gpu::{FlushUniforms, TriangleVertex};
#[cfg(target_arch = "wasm32")]
use crate::work_metrics::record_buffer_upload;
use crate::work_metrics::CountedDeviceExt;

pub(crate) struct MsaaStencilPipeline {
    pub clip_reset_pipeline: wgpu::RenderPipeline,
    pub nested_clip_reset_pipeline: wgpu::RenderPipeline,
    pub nested_clockwise_clip_reset_pipeline: wgpu::RenderPipeline,
    flush_layout: wgpu::BindGroupLayout,
}

pub(crate) struct PreparedStencilDraw {
    pub flush_group: wgpu::BindGroup,
    pub vertices: wgpu::Buffer,
    pub vertex_count: u32,
}

impl MsaaStencilPipeline {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let vertex = shader(
            device,
            "nuxie-msaa-stencil-vertex",
            include_str!("generated/draw_msaa_stencil.webgpu_noclipdistance_vert.wgsl"),
        );
        let fragment = shader(
            device,
            "nuxie-msaa-stencil-fragment",
            include_str!("generated/draw_msaa_stencil.webgpu_fixedcolor_frag.wgsl"),
        );
        let flush_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("nuxie-msaa-stencil-flush-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nuxie-msaa-stencil-pipeline-layout"),
            bind_group_layouts: &[Some(&flush_layout)],
            immediate_size: 0,
        });
        let reset_face = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::NotEqual,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Zero,
        };
        let clip_reset_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-msaa-clip-reset-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vertex,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(TriangleVertex::layout())],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24PlusStencil8,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState {
                    front: reset_face,
                    back: reset_face,
                    read_mask: 0xff,
                    write_mask: 0xff,
                },
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::empty(),
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let nested_face = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Less,
            fail_op: wgpu::StencilOperation::Zero,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Replace,
        };
        let create_nested_clip_reset_pipeline = |label, read_mask| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &vertex,
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    buffers: &[Some(TriangleVertex::layout())],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: wgpu::StencilState {
                        front: nested_face,
                        back: nested_face,
                        read_mask,
                        write_mask: 0xff,
                    },
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 4,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &fragment,
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::empty(),
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        let nested_clip_reset_pipeline =
            create_nested_clip_reset_pipeline("nuxie-msaa-nested-clip-reset-pipeline", 0xff);
        let nested_clockwise_clip_reset_pipeline = create_nested_clip_reset_pipeline(
            "nuxie-msaa-nested-clockwise-clip-reset-pipeline",
            0xc0,
        );
        Self {
            clip_reset_pipeline,
            nested_clip_reset_pipeline,
            nested_clockwise_clip_reset_pipeline,
            flush_layout,
        }
    }

    pub(crate) fn prepare_clip_reset(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        uniforms: &FlushUniforms,
        bounds: [f32; 4],
        z_index: u16,
    ) -> PreparedStencilDraw {
        let uniform_buffer = create_uploaded_buffer(
            device,
            queue,
            "nuxie-msaa-stencil-uniforms",
            bytemuck::bytes_of(uniforms),
            wgpu::BufferUsages::UNIFORM,
        );
        let flush_group = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-msaa-stencil-flush-group"),
            layout: &self.flush_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        let [left, top, right, bottom] = bounds;
        let vertices = [
            TriangleVertex::new([left, bottom], 0, z_index),
            TriangleVertex::new([left, top], 0, z_index),
            TriangleVertex::new([right, bottom], 0, z_index),
            TriangleVertex::new([right, bottom], 0, z_index),
            TriangleVertex::new([left, top], 0, z_index),
            TriangleVertex::new([right, top], 0, z_index),
        ];
        let vertex_buffer = create_uploaded_buffer(
            device,
            queue,
            "nuxie-msaa-stencil-vertices",
            bytemuck::cast_slice(&vertices),
            wgpu::BufferUsages::VERTEX,
        );
        PreparedStencilDraw {
            flush_group,
            vertices: vertex_buffer,
            vertex_count: vertices.len() as u32,
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn create_uploaded_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &'static str,
    contents: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: contents.len() as u64,
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    record_buffer_upload(contents.len() as u64);
    queue.write_buffer(&buffer, 0, contents);
    buffer
}

#[cfg(not(target_arch = "wasm32"))]
fn create_uploaded_buffer(
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    label: &'static str,
    contents: &[u8],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_counted_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents,
        usage,
    })
}

fn shader(device: &wgpu::Device, label: &str, source: &'static str) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}
