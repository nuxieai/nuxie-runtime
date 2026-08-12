#![cfg(target_os = "macos")]

use std::borrow::Cow;

use nuxie_render_api::{
    BlendMode, Factory as _, GpuCanvasPassState, GpuCanvasPipelineState, GpuCanvasPlan,
    GpuCanvasShader, GpuCanvasShaderArtifact, GpuCanvasShaderBinding,
    GpuCanvasShaderBindingReflection, GpuCanvasShaderBuiltin, GpuCanvasShaderEntry,
    GpuCanvasShaderEntryReflection, GpuCanvasShaderInterfaceBinding, GpuCanvasShaderInterfaceType,
    GpuCanvasShaderInterfaceVariable, GpuCanvasShaderProvenance, GpuCanvasShaderResourceKind,
    GpuCanvasShaderStage, GpuCanvasShaderTextureSampleType, GpuCanvasShaderTextureViewDimension,
    GpuCanvasUniformBuffer, ImageSampler, Renderer as _,
};
use nuxie_renderer::{RenderMode, WgpuFactory};

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

fragment float4 fragment_main(
    constant float4& first [[buffer(0)]],
    constant float4& second [[buffer(1)]]) {
    return first + second;
}
"#;

const PROBE_WGSL: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

struct Tint { color: vec4<f32>, }
@group(0) @binding(0) var<uniform> first: Tint;
@group(2) @binding(3) var<uniform> second: Tint;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return first.color + second.color;
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

#[test]
#[ignore = "requires a real macOS Metal device; run make renderer-apple-passthrough-probe"]
fn trusted_factory_profile_materializes_authenticated_msl() {
    let digest = [0x42; 32];
    // SAFETY: The synthetic artifact below is wholly defined in this probe and
    // tied to the same test-only digest. Production callers cannot mint this
    // authority without crossing the documented unsafe provenance boundary.
    let provenance =
        unsafe { GpuCanvasShaderProvenance::for_verified_artifact_digest_unchecked(321, digest) };
    let entries = vec![
        GpuCanvasShaderEntry {
            stage: GpuCanvasShaderStage::Vertex,
            logical_entry_point: "vertex_main".into(),
            physical_entry_point: "vertex_main".into(),
        },
        GpuCanvasShaderEntry {
            stage: GpuCanvasShaderStage::Fragment,
            logical_entry_point: "fragment_main".into(),
            physical_entry_point: "fragment_main".into(),
        },
    ];
    let variable = |binding, interface_type| GpuCanvasShaderInterfaceVariable {
        binding,
        interface_type,
    };
    let reflection = vec![
        GpuCanvasShaderEntryReflection {
            stage: GpuCanvasShaderStage::Vertex,
            logical_entry_point: "vertex_main".into(),
            physical_entry_point: "vertex_main".into(),
            workgroup_size: [1, 1, 1],
            inputs: vec![variable(
                GpuCanvasShaderInterfaceBinding::Builtin(GpuCanvasShaderBuiltin::VertexIndex),
                GpuCanvasShaderInterfaceType::Uint,
            )],
            outputs: vec![variable(
                GpuCanvasShaderInterfaceBinding::Builtin(GpuCanvasShaderBuiltin::Position),
                GpuCanvasShaderInterfaceType::Float4,
            )],
        },
        GpuCanvasShaderEntryReflection {
            stage: GpuCanvasShaderStage::Fragment,
            logical_entry_point: "fragment_main".into(),
            physical_entry_point: "fragment_main".into(),
            workgroup_size: [1, 1, 1],
            inputs: Vec::new(),
            outputs: vec![variable(
                GpuCanvasShaderInterfaceBinding::Location {
                    location: 0,
                    interpolation: None,
                    sampling: None,
                },
                GpuCanvasShaderInterfaceType::Float4,
            )],
        },
    ];
    let fragment = 1 << GpuCanvasShaderStage::Fragment as u8;
    let uniform = |group, binding, slot| GpuCanvasShaderBinding {
        group,
        binding,
        kind: GpuCanvasShaderResourceKind::UniformBuffer,
        stage_mask: fragment,
        backend_space: group,
        backend_slots: [None, Some(slot), None],
        texture_view_dimension: GpuCanvasShaderTextureViewDimension::D2,
        texture_sample_type: GpuCanvasShaderTextureSampleType::Float,
        texture_multisampled: false,
    };
    let bindings = vec![uniform(0, 0, 0), uniform(2, 3, 1)];
    let binding_reflection = vec![
        GpuCanvasShaderBindingReflection {
            group: 0,
            binding: 0,
            array_count: 1,
            min_buffer_size: 16,
        },
        GpuCanvasShaderBindingReflection {
            group: 2,
            binding: 3,
            array_count: 1,
            min_buffer_size: 16,
        },
    ];
    // SAFETY: source, entries, bindings, and reflection are the exact synthetic parts
    // covered by the test-only authority above.
    let shader = unsafe {
        nuxie_render_api::GpuCanvasAppleMetalShader::from_verified_parts(
            provenance,
            321,
            digest,
            PROBE_MSL.into(),
            entries,
            bindings.clone(),
            reflection,
            binding_reflection,
        )
    }
    .unwrap();
    let mut native_factory =
        WgpuFactory::new_with_trusted_apple_metal_shaders(16, 16, RenderMode::Msaa)
            .expect("opt-in trusted-MSL factory starts on Metal");
    let native_module = native_factory
        .make_gpu_canvas_shader_artifact(&GpuCanvasShaderArtifact::TrustedAppleMetal(shader))
        .expect("authenticated MSL passes shared reflection checks and compiles");

    let web_bindings = bindings
        .into_iter()
        .map(|mut binding| {
            binding.backend_slots = [None, Some(binding.binding.into()), None];
            binding
        })
        .collect();
    let web_shader = GpuCanvasShader {
        source: PROBE_WGSL.into(),
        entries: vec![
            GpuCanvasShaderEntry {
                stage: GpuCanvasShaderStage::Vertex,
                logical_entry_point: "vs_main".into(),
                physical_entry_point: "vs_main".into(),
            },
            GpuCanvasShaderEntry {
                stage: GpuCanvasShaderStage::Fragment,
                logical_entry_point: "fs_main".into(),
                physical_entry_point: "fs_main".into(),
            },
        ],
        bindings: web_bindings,
    };
    let mut web_factory = WgpuFactory::new_with_mode(16, 16, RenderMode::Msaa)
        .expect("reference WGSL factory starts on Metal");
    let web_module = web_factory
        .make_gpu_canvas_shader(&web_shader)
        .expect("reference WGSL validates");
    let plan = GpuCanvasPlan {
        vertex_entry: None,
        fragment_entry: None,
        width: 16,
        height: 16,
        clear_color: [0.0, 0.0, 0.0, 1.0],
        vertex_count: 3,
        instance_count: 1,
        first_vertex: 0,
        first_instance: 0,
        uniform_buffers: vec![
            GpuCanvasUniformBuffer {
                group: 0,
                binding: 0,
                bytes: [0.125_f32, 0.25, 0.375, 0.5]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            },
            GpuCanvasUniformBuffer {
                group: 2,
                binding: 3,
                bytes: [0.125_f32, 0.25, 0.375, 0.5]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            },
        ],
        vertex_layouts: Vec::new(),
        vertex_buffers: Vec::new(),
        index_buffer: None,
        indexed_draw: None,
        texture_bindings: Vec::new(),
        sampler_bindings: Vec::new(),
        pipeline_state: GpuCanvasPipelineState::default(),
        pass_state: GpuCanvasPassState::default(),
        pipelines: Vec::new(),
        render_passes: Vec::new(),
    };
    let render =
        |factory: &mut WgpuFactory,
         module: &std::sync::Arc<dyn nuxie_render_api::RenderGpuCanvasShader>| {
            let image = factory
                .make_gpu_canvas_image(module, module, &plan)
                .expect("authored shader draws into a retained image");
            let mut frame = factory.begin_frame(0xff00_0000);
            frame.draw_image(
                Some(image.as_ref()),
                ImageSampler::default(),
                BlendMode::SrcOver,
                1.0,
            );
            frame.finish().expect("authored image composites")
        };
    let native_pixels = render(&mut native_factory, &native_module);
    let web_pixels = render(&mut web_factory, &web_module);
    assert_eq!(
        native_pixels, web_pixels,
        "trusted target-2 MSL must match the target-0 WGSL renderer output byte-for-byte"
    );
}
