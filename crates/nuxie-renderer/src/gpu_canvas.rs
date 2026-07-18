//! Arbitrary WGSL draw execution for editor GPU-canvas critique frames.
//!
//! Luau execution lives in `nuxie-scripting`; this module accepts only its
//! typed draw plan and owns shader modules, buffers, submission, and readback.

use std::collections::BTreeSet;
use std::num::NonZeroU64;

use wgpu::util::DeviceExt;

use super::{align_to, map_buffer, RendererError, WgpuFactory};

const MAX_GPU_CANVAS_DIMENSION: u32 = 2_048;
const MAX_UNIFORM_BUFFER_BYTES: usize = 64 * 1024;
const MAX_VERTEX_BUFFER_BYTES: usize = 16 * 1024 * 1024;
const MAX_DRAW_INVOCATIONS: u64 = 1_000_000;
const MAX_VERTEX_BUFFERS: usize = 8;
const MAX_VERTEX_ATTRIBUTES: usize = 16;
const MAX_BIND_GROUPS: u32 = 4;
const MAX_UNIFORM_BINDINGS_PER_GROUP: usize = 12;
const MAX_BINDING_INDEX: u32 = 255;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasUniformBuffer {
    pub group: u32,
    pub binding: u32,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasVertexAttribute {
    pub shader_location: u32,
    pub offset: u64,
    pub format: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasVertexLayout {
    pub stride: u64,
    pub attributes: Vec<GpuCanvasVertexAttribute>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuCanvasVertexBuffer {
    pub slot: u32,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpuCanvasRenderPlan {
    pub shader_wgsl: String,
    pub width: u32,
    pub height: u32,
    pub clear_color: [f64; 4],
    pub vertex_count: u32,
    pub instance_count: u32,
    pub first_vertex: u32,
    pub first_instance: u32,
    pub uniform_buffers: Vec<GpuCanvasUniformBuffer>,
    pub vertex_layouts: Vec<GpuCanvasVertexLayout>,
    pub vertex_buffers: Vec<GpuCanvasVertexBuffer>,
}

impl WgpuFactory {
    /// Execute one validated script-authored WGSL pass and return tightly
    /// packed RGBA pixels. The caller retains the factory across temporal
    /// samples so device selection and shader behavior stay fixed.
    pub async fn render_gpu_canvas(
        &self,
        plan: &GpuCanvasRenderPlan,
    ) -> Result<Vec<u8>, RendererError> {
        validate_gpu_canvas_plan(plan)?;
        let device = &self.context.device;
        let queue = &self.context.queue;
        let validation_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("nuxie-gpu-canvas-shader"),
            source: wgpu::ShaderSource::Wgsl(plan.shader_wgsl.clone().into()),
        });

        let max_group = plan.uniform_buffers.iter().map(|buffer| buffer.group).max();
        let mut bind_group_layouts = Vec::new();
        let mut bind_groups = Vec::new();
        let mut uniform_gpu_buffers = Vec::new();
        if let Some(max_group) = max_group {
            for group in 0..=max_group {
                let group_buffers = plan
                    .uniform_buffers
                    .iter()
                    .filter(|buffer| buffer.group == group)
                    .collect::<Vec<_>>();
                let entries = group_buffers
                    .iter()
                    .map(|buffer| wgpu::BindGroupLayoutEntry {
                        binding: buffer.binding,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZeroU64::new(buffer.bytes.len() as u64),
                        },
                        count: None,
                    })
                    .collect::<Vec<_>>();
                let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("nuxie-gpu-canvas-bind-group-layout"),
                    entries: &entries,
                });
                let first_buffer = uniform_gpu_buffers.len();
                for buffer in &group_buffers {
                    uniform_gpu_buffers.push(device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("nuxie-gpu-canvas-uniform"),
                            contents: &buffer.bytes,
                            usage: wgpu::BufferUsages::UNIFORM,
                        },
                    ));
                }
                let binding_entries = group_buffers
                    .iter()
                    .enumerate()
                    .map(|(index, buffer)| wgpu::BindGroupEntry {
                        binding: buffer.binding,
                        resource: uniform_gpu_buffers[first_buffer + index].as_entire_binding(),
                    })
                    .collect::<Vec<_>>();
                bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("nuxie-gpu-canvas-bind-group"),
                    layout: &layout,
                    entries: &binding_entries,
                }));
                bind_group_layouts.push(layout);
            }
        }
        let layout_refs = bind_group_layouts.iter().map(Some).collect::<Vec<_>>();
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("nuxie-gpu-canvas-pipeline-layout"),
            bind_group_layouts: &layout_refs,
            immediate_size: 0,
        });

        let vertex_attributes = plan
            .vertex_layouts
            .iter()
            .map(|layout| {
                layout
                    .attributes
                    .iter()
                    .map(|attribute| {
                        Ok(wgpu::VertexAttribute {
                            format: vertex_format(&attribute.format)?,
                            offset: attribute.offset,
                            shader_location: attribute.shader_location,
                        })
                    })
                    .collect::<Result<Vec<_>, RendererError>>()
            })
            .collect::<Result<Vec<_>, RendererError>>()?;
        let vertex_layouts = plan
            .vertex_layouts
            .iter()
            .zip(&vertex_attributes)
            .map(|(layout, attributes)| {
                Some(wgpu::VertexBufferLayout {
                    array_stride: layout.stride,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes,
                })
            })
            .collect::<Vec<_>>();
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("nuxie-gpu-canvas-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &vertex_layouts,
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        let vertex_gpu_buffers = plan
            .vertex_buffers
            .iter()
            .map(|buffer| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("nuxie-gpu-canvas-vertex-buffer"),
                    contents: &buffer.bytes,
                    usage: wgpu::BufferUsages::VERTEX,
                })
            })
            .collect::<Vec<_>>();
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("nuxie-gpu-canvas-target"),
            size: wgpu::Extent3d {
                width: plan.width,
                height: plan.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
        let unpadded_bytes_per_row = plan.width.saturating_mul(4);
        let padded_bytes_per_row =
            align_to(unpadded_bytes_per_row, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("nuxie-gpu-canvas-readback"),
            size: u64::from(padded_bytes_per_row).saturating_mul(u64::from(plan.height)),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("nuxie-gpu-canvas-encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("nuxie-gpu-canvas-pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: plan.clear_color[0],
                            g: plan.clear_color[1],
                            b: plan.clear_color[2],
                            a: plan.clear_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&pipeline);
            for (index, bind_group) in bind_groups.iter().enumerate() {
                pass.set_bind_group(index as u32, bind_group, &[]);
            }
            for (buffer, gpu_buffer) in plan.vertex_buffers.iter().zip(&vertex_gpu_buffers) {
                pass.set_vertex_buffer(buffer.slot, gpu_buffer.slice(..));
            }
            pass.draw(
                plan.first_vertex..plan.first_vertex.saturating_add(plan.vertex_count),
                plan.first_instance..plan.first_instance.saturating_add(plan.instance_count),
            );
        }
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(plan.height),
                },
            },
            target.size(),
        );
        queue.submit(Some(encoder.finish()));
        if let Some(error) = validation_scope.pop().await {
            return Err(RendererError::InvalidGpuCanvas(format!(
                "wgpu rejected the validated plan: {error}"
            )));
        }

        let slice = readback.slice(..);
        map_buffer(&self.context, &slice).await?;
        let mapped = slice
            .get_mapped_range()
            .map_err(|error| RendererError::Map(error.to_string()))?;
        let mut pixels = Vec::with_capacity(unpadded_bytes_per_row as usize * plan.height as usize);
        for row in mapped.chunks_exact(padded_bytes_per_row as usize) {
            pixels.extend_from_slice(&row[..unpadded_bytes_per_row as usize]);
        }
        drop(mapped);
        readback.unmap();
        Ok(pixels)
    }
}

fn validate_gpu_canvas_plan(plan: &GpuCanvasRenderPlan) -> Result<(), RendererError> {
    let invalid = |message: String| RendererError::InvalidGpuCanvas(message);
    if plan.width == 0
        || plan.height == 0
        || plan.width > MAX_GPU_CANVAS_DIMENSION
        || plan.height > MAX_GPU_CANVAS_DIMENSION
    {
        return Err(invalid(format!(
            "dimensions must be between 1 and {MAX_GPU_CANVAS_DIMENSION}"
        )));
    }
    if plan
        .clear_color
        .iter()
        .any(|component| !component.is_finite() || !(0.0..=1.0).contains(component))
    {
        return Err(invalid(
            "clear color components must be finite values from 0 through 1".into(),
        ));
    }
    if plan.vertex_count == 0 || plan.instance_count == 0 {
        return Err(invalid(
            "vertex and instance counts must be positive".into(),
        ));
    }
    let vertex_end = plan
        .first_vertex
        .checked_add(plan.vertex_count)
        .ok_or_else(|| invalid("vertex range overflow".into()))?;
    let instance_end = plan
        .first_instance
        .checked_add(plan.instance_count)
        .ok_or_else(|| invalid("instance range overflow".into()))?;
    let invocations = u64::from(plan.vertex_count)
        .checked_mul(u64::from(plan.instance_count))
        .ok_or_else(|| invalid("draw invocation count overflow".into()))?;
    if invocations > MAX_DRAW_INVOCATIONS
        || u64::from(vertex_end) > MAX_DRAW_INVOCATIONS
        || u64::from(instance_end) > MAX_DRAW_INVOCATIONS
    {
        return Err(invalid(format!(
            "draw ranges may cover at most {MAX_DRAW_INVOCATIONS} invocations"
        )));
    }

    let mut bindings = BTreeSet::new();
    let mut group_counts = [0_usize; MAX_BIND_GROUPS as usize];
    for buffer in &plan.uniform_buffers {
        if buffer.group >= MAX_BIND_GROUPS {
            return Err(invalid(format!(
                "bind group must be less than {MAX_BIND_GROUPS}"
            )));
        }
        if buffer.binding > MAX_BINDING_INDEX {
            return Err(invalid(format!(
                "uniform binding must be at most {MAX_BINDING_INDEX}"
            )));
        }
        if buffer.bytes.is_empty() || buffer.bytes.len() > MAX_UNIFORM_BUFFER_BYTES {
            return Err(invalid(format!(
                "uniform buffers must contain between 1 and {MAX_UNIFORM_BUFFER_BYTES} bytes"
            )));
        }
        if buffer.bytes.len() % 4 != 0 {
            return Err(invalid(
                "uniform buffer byte lengths must be four-byte aligned".into(),
            ));
        }
        if !bindings.insert((buffer.group, buffer.binding)) {
            return Err(invalid(format!(
                "uniform binding {} in group {} is duplicated",
                buffer.binding, buffer.group
            )));
        }
        group_counts[buffer.group as usize] += 1;
        if group_counts[buffer.group as usize] > MAX_UNIFORM_BINDINGS_PER_GROUP {
            return Err(invalid(format!(
                "bind group {} exceeds {MAX_UNIFORM_BINDINGS_PER_GROUP} uniform bindings",
                buffer.group
            )));
        }
    }
    if plan.vertex_layouts.len() > MAX_VERTEX_BUFFERS
        || plan.vertex_buffers.len() > MAX_VERTEX_BUFFERS
        || plan.vertex_layouts.len() != plan.vertex_buffers.len()
    {
        return Err(invalid(format!(
            "vertex layout and buffer counts must match and be at most {MAX_VERTEX_BUFFERS}"
        )));
    }
    let mut locations = BTreeSet::new();
    let mut buffer_slots = BTreeSet::new();
    let mut attribute_count = 0;
    for buffer in &plan.vertex_buffers {
        if buffer.slot as usize >= MAX_VERTEX_BUFFERS {
            return Err(invalid(format!(
                "vertex buffer slot must be less than {MAX_VERTEX_BUFFERS}"
            )));
        }
        if !buffer_slots.insert(buffer.slot) {
            return Err(invalid(format!(
                "vertex buffer slot {} is duplicated",
                buffer.slot
            )));
        }
        if buffer.bytes.is_empty() || buffer.bytes.len() > MAX_VERTEX_BUFFER_BYTES {
            return Err(invalid(format!(
                "vertex buffers must contain between 1 and {MAX_VERTEX_BUFFER_BYTES} bytes"
            )));
        }
    }
    for (slot, layout) in plan.vertex_layouts.iter().enumerate() {
        if layout.stride == 0 || layout.stride > 2_048 {
            return Err(invalid(
                "vertex layout stride must be between 1 and 2048 bytes".into(),
            ));
        }
        if layout.attributes.is_empty() {
            return Err(invalid(
                "vertex layouts must contain at least one attribute".into(),
            ));
        }
        let slot = u32::try_from(slot).map_err(|_| invalid("vertex slot overflow".into()))?;
        let buffer = plan
            .vertex_buffers
            .iter()
            .find(|buffer| buffer.slot == slot)
            .ok_or_else(|| invalid(format!("vertex buffer slot {slot} is not bound")))?;
        let required_bytes = u64::from(vertex_end)
            .checked_mul(layout.stride)
            .ok_or_else(|| invalid("vertex buffer byte range overflow".into()))?;
        if required_bytes > buffer.bytes.len() as u64 {
            return Err(invalid(format!(
                "vertex buffer slot {slot} requires {required_bytes} bytes"
            )));
        }
        for attribute in &layout.attributes {
            attribute_count += 1;
            if attribute_count > MAX_VERTEX_ATTRIBUTES {
                return Err(invalid(format!(
                    "pipelines support at most {MAX_VERTEX_ATTRIBUTES} vertex attributes"
                )));
            }
            if attribute.shader_location >= MAX_VERTEX_ATTRIBUTES as u32
                || !locations.insert(attribute.shader_location)
            {
                return Err(invalid(format!(
                    "vertex attribute location {} is out of range or duplicated",
                    attribute.shader_location
                )));
            }
            let size = vertex_format_size(&attribute.format)?;
            if attribute
                .offset
                .checked_add(size)
                .is_none_or(|end| end > layout.stride)
            {
                return Err(invalid(format!(
                    "vertex attribute at offset {} exceeds stride {}",
                    attribute.offset, layout.stride
                )));
            }
        }
    }
    Ok(())
}

fn vertex_format_size(name: &str) -> Result<u64, RendererError> {
    match name {
        "float32" => Ok(4),
        "float32x2" => Ok(8),
        "float32x3" => Ok(12),
        "float32x4" => Ok(16),
        _ => Err(RendererError::InvalidGpuCanvas(format!(
            "unsupported vertex format '{name}'"
        ))),
    }
}

fn vertex_format(name: &str) -> Result<wgpu::VertexFormat, RendererError> {
    match name {
        "float32" => Ok(wgpu::VertexFormat::Float32),
        "float32x2" => Ok(wgpu::VertexFormat::Float32x2),
        "float32x3" => Ok(wgpu::VertexFormat::Float32x3),
        "float32x4" => Ok(wgpu::VertexFormat::Float32x4),
        _ => Err(RendererError::Unsupported(
            "GPU-canvas vertex format is not implemented",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_plan() -> GpuCanvasRenderPlan {
        GpuCanvasRenderPlan {
            shader_wgsl: String::new(),
            width: 8,
            height: 8,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            vertex_count: 3,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
            uniform_buffers: Vec::new(),
            vertex_layouts: Vec::new(),
            vertex_buffers: Vec::new(),
        }
    }

    #[test]
    fn product_vertex_formats_are_explicit_and_fail_closed() {
        assert_eq!(
            vertex_format("float32x3").unwrap(),
            wgpu::VertexFormat::Float32x3
        );
        assert!(vertex_format("snorm10x3").is_err());
    }

    #[test]
    fn product_plan_limits_fail_before_backend_allocation() {
        let mut plan = valid_plan();
        plan.width = MAX_GPU_CANVAS_DIMENSION + 1;
        assert!(validate_gpu_canvas_plan(&plan).is_err());

        let mut plan = valid_plan();
        plan.vertex_count = MAX_DRAW_INVOCATIONS as u32;
        plan.instance_count = 2;
        assert!(validate_gpu_canvas_plan(&plan).is_err());

        let mut plan = valid_plan();
        plan.uniform_buffers = vec![GpuCanvasUniformBuffer {
            group: 0,
            binding: 0,
            bytes: vec![0; MAX_UNIFORM_BUFFER_BYTES + 4],
        }];
        assert!(validate_gpu_canvas_plan(&plan).is_err());
    }

    #[test]
    fn product_plan_rejects_duplicate_bindings_and_vertex_slots() {
        let mut plan = valid_plan();
        plan.uniform_buffers = vec![
            GpuCanvasUniformBuffer {
                group: 0,
                binding: 0,
                bytes: vec![0; 16],
            },
            GpuCanvasUniformBuffer {
                group: 0,
                binding: 0,
                bytes: vec![0; 16],
            },
        ];
        assert!(validate_gpu_canvas_plan(&plan).is_err());

        let mut plan = valid_plan();
        plan.vertex_layouts = vec![
            GpuCanvasVertexLayout {
                stride: 4,
                attributes: vec![GpuCanvasVertexAttribute {
                    shader_location: 0,
                    offset: 0,
                    format: "float32".into(),
                }],
            },
            GpuCanvasVertexLayout {
                stride: 4,
                attributes: vec![GpuCanvasVertexAttribute {
                    shader_location: 1,
                    offset: 0,
                    format: "float32".into(),
                }],
            },
        ];
        plan.vertex_buffers = vec![
            GpuCanvasVertexBuffer {
                slot: 0,
                bytes: vec![0; 12],
            },
            GpuCanvasVertexBuffer {
                slot: 0,
                bytes: vec![0; 12],
            },
        ];
        assert!(validate_gpu_canvas_plan(&plan).is_err());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn executes_validated_wgsl_and_reads_real_pixels() {
        let Ok(factory) = WgpuFactory::new(8, 8) else {
            eprintln!("GPU adapter unavailable; browser execution remains a separate proof");
            return;
        };
        let plan = GpuCanvasRenderPlan {
            shader_wgsl: r#"
                @vertex
                fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
                    let x = f32(i32(index) - 1);
                    let y = f32(i32(index & 1u) * 2 - 1);
                    return vec4<f32>(x, y, 0.0, 1.0);
                }

                @fragment
                fn fs_main() -> @location(0) vec4<f32> {
                    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
                }
            "#
            .into(),
            width: 8,
            height: 8,
            clear_color: [0.0, 0.0, 0.0, 1.0],
            vertex_count: 3,
            instance_count: 1,
            first_vertex: 0,
            first_instance: 0,
            uniform_buffers: Vec::new(),
            vertex_layouts: Vec::new(),
            vertex_buffers: Vec::new(),
        };
        let pixels =
            pollster::block_on(factory.render_gpu_canvas(&plan)).expect("WGSL draw completes");
        assert_eq!(pixels.len(), 8 * 8 * 4);
        assert!(
            pixels
                .chunks_exact(4)
                .any(|pixel| pixel[0] > 240 && pixel[1] < 10 && pixel[2] < 10),
            "fullscreen triangle produces red pixels"
        );
    }
}
