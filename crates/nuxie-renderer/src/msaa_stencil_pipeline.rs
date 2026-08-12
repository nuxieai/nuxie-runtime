//! MSAA stencil reset translated from Rive's WebGPU renderer.

use crate::gpu::{FlushUniforms, TriangleVertex};
use crate::shader_catalog::{self, BuiltinShaderKey};
use crate::tessellator::{TessellationUploadFrame, UploadSlice};
use crate::work_metrics::CountedDeviceExt;

pub(crate) struct MsaaStencilPipeline {
    pub clip_reset_pipeline: wgpu::RenderPipeline,
    pub nested_clip_reset_pipeline: wgpu::RenderPipeline,
    pub nested_clockwise_clip_reset_pipeline: wgpu::RenderPipeline,
    flush_layout: wgpu::BindGroupLayout,
}

pub(crate) struct PreparedStencilDraw {
    pub flush_group: wgpu::BindGroup,
    #[cfg(test)]
    pub uniforms: UploadSlice,
    pub vertices: UploadSlice,
    pub vertex_count: u32,
}

impl MsaaStencilPipeline {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let vertex = shader_catalog::create(device, BuiltinShaderKey::MsaaStencilVertex);
        let fragment = shader_catalog::create(device, BuiltinShaderKey::MsaaStencilFragment);
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
        encoder: &mut wgpu::CommandEncoder,
        uploads: &mut TessellationUploadFrame<'_>,
        uniforms: &FlushUniforms,
        bounds: [f32; 4],
        z_index: u16,
    ) -> PreparedStencilDraw {
        // Clip-heavy frames can prepare dozens of resets. Keep their small
        // payloads on the frame upload page instead of creating one
        // mapped-at-creation GPU buffer for every uniform and vertex block.
        let uniform_buffer = uploads.upload_uniforms(device, encoder, bytemuck::bytes_of(uniforms));
        let flush_group = device.create_counted_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("nuxie-msaa-stencil-flush-group"),
            layout: &self.flush_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.binding(),
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
        let vertex_buffer =
            uploads.upload_vertices(device, encoder, bytemuck::cast_slice(&vertices));
        PreparedStencilDraw {
            flush_group,
            #[cfg(test)]
            uniforms: uniform_buffer,
            vertices: vertex_buffer,
            vertex_count: vertices.len() as u32,
        }
    }
}
