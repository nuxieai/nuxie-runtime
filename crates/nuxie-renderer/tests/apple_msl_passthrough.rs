#![cfg(target_os = "macos")]

use std::borrow::Cow;

const PROBE_MSL: &str = r#"
#include <metal_stdlib>
using namespace metal;

struct VertexOutput {
    float4 position [[position]];
};

vertex VertexOutput vertex_main(uint vertex_index [[vertex_id]]) {
    const float2 positions[3] = {
        float2(-1.0, -1.0),
        float2( 3.0, -1.0),
        float2(-1.0,  3.0),
    };
    VertexOutput output;
    output.position = float4(positions[vertex_index], 0.0, 1.0);
    return output;
}

fragment float4 fragment_main() {
    return float4(0.25, 0.5, 0.75, 1.0);
}
"#;

#[test]
#[ignore = "requires a real macOS Metal device; run make renderer-apple-passthrough-probe"]
fn pinned_wgpu_creates_an_explicit_layout_pipeline_from_msl() {
    pollster::block_on(async {
        let mut instance_descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
        instance_descriptor.backends = wgpu::Backends::METAL;
        // The future Apple path must not compile wgpu's internal indirect-
        // validation or timestamp-normalization WGSL during device creation.
        // nuxie-renderer currently records only direct draws; the production
        // cutover still needs a ratchet that keeps that invariant true.
        instance_descriptor.flags = wgpu::InstanceFlags::empty();
        let instance = wgpu::Instance::new(instance_descriptor);
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("macOS Metal adapter is required for the passthrough probe");
        assert_eq!(adapter.get_info().backend, wgpu::Backend::Metal);
        assert!(
            adapter
                .features()
                .contains(wgpu::Features::PASSTHROUGH_SHADERS),
            "the selected Metal adapter must advertise PASSTHROUGH_SHADERS"
        );

        let (device, _queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("UNIV-1642 MSL passthrough probe"),
                required_features: wgpu::Features::PASSTHROUGH_SHADERS,
                ..Default::default()
            })
            .await
            .expect("Metal device request with PASSTHROUGH_SHADERS must succeed");

        let entry_points = [
            wgpu::PassthroughShaderEntryPoint {
                name: Cow::Borrowed("vertex_main"),
                workgroup_size: (0, 0, 0),
            },
            wgpu::PassthroughShaderEntryPoint {
                name: Cow::Borrowed("fragment_main"),
                workgroup_size: (0, 0, 0),
            },
        ];
        // SAFETY: This test owns the complete MSL source and matching pipeline
        // descriptor. It deliberately exercises wgpu's unsafe passthrough
        // boundary; production use remains blocked on the reflection, binding,
        // provenance, and real-device gates in the UNIV-1642 decision.
        let module = unsafe {
            device.create_shader_module_passthrough(wgpu::ShaderModuleDescriptorPassthrough {
                label: Some("UNIV-1642 MSL module"),
                entry_points: Cow::Borrowed(&entry_points),
                msl: Some(Cow::Borrowed(PROBE_MSL)),
                ..Default::default()
            })
        };
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UNIV-1642 empty pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let _pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UNIV-1642 MSL passthrough pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vertex_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fragment_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
    });
}
