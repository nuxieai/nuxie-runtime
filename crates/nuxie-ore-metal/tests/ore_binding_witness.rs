#![cfg(target_os = "macos")]

use std::collections::BTreeMap;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::time::{Duration, Instant};

use nuxie_ore_metal::mechanical_port::source::renderer::include::rive::renderer::ore::ore_binding_map_hpp::{BindingMap, ResourceKind};
use nuxie_ore_metal::mechanical_port::source::renderer::include::rive::renderer::ore::ore_context_hpp::FrameDescriptor;
use nuxie_ore_metal::mechanical_port::source::renderer::include::rive::renderer::ore::ore_context_metal_hpp::ContextMetal;
use nuxie_ore_metal::mechanical_port::source::renderer::src::ore::metal::ore_shader_module_metal_hpp::ShaderModuleMetal;
use nuxie_ore_metal::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::{
    BindGroupDesc, BindGroupLayoutDesc, BindGroupLayoutEntry, BindingKind, BufferDesc, BufferUsage,
    ClearColor, ColorAttachment, ColorTargetState, PipelineDesc, RenderPassDesc, ShaderModuleDesc,
    StageVisibility, TextureFormat, UBOEntry,
};
use objc2_metal::{
    MTLCreateSystemDefaultDevice, MTLDevice, MTLPixelFormat, MTLRegion, MTLStorageMode, MTLTexture,
    MTLTextureDescriptor, MTLTextureUsage,
};
use sha2::{Digest, Sha256};

// Mechanical fixture extraction from Rive
// tests/gm/ore_gm_shaders.rstb.hpp::kBindingWitness at pinned revision
// 4ac7b32798da0482e441ef09304dc3b480ed3ee5.
// MSL SHA-256: e13a2df60b11b7522c37725046e19e09620f78a514d783aa907a2efc646869a5
const BINDING_WITNESS_MSL: &[u8] = br#"// language: metal1.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct Uniforms {
    metal::float4 color;
};
struct type_3 {
    metal::float2 inner[3];
};

struct vs_mainInput {
};
struct vs_mainOutput {
    metal::float4 member [[position]];
};
vertex vs_mainOutput vs_main(
  uint vid [[vertex_id]]
) {
    type_3 positions = type_3 {metal::float2(-1.0, -1.0), metal::float2(3.0, -1.0), metal::float2(-1.0, 3.0)};
    metal::float2 _e13 = positions.inner[vid];
    return vs_mainOutput { metal::float4(_e13, 0.0, 1.0) };
}


struct fs_mainOutput {
    metal::float4 member_1 [[color(0)]];
};
fragment fs_mainOutput fs_main(
  constant Uniforms& u_low [[buffer(0)]]
, constant Uniforms& u_high [[buffer(1)]]
) {
    metal::float4 _e2 = u_low.color;
    metal::float4 _e6 = u_high.color;
    return fs_mainOutput { metal::float4(_e2.xyz + _e6.xyz, 1.0) };
}
"#;

// BindingMap v2, two 14-byte rows. Authored bindings 0 and 7 resolve to
// Metal buffer slots 0 and 1 for vertex, fragment, and compute stages.
// SHA-256: 8a2aa27a73c79b03ee5868aa07b0a25294e281454326df657b0a7c6cf3f5ba22
const BINDING_WITNESS_MAP: [u8; 36] = [
    2, 1, 14, 0, 2, 0, 0, 0, // header
    0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // group 0, binding 0 -> slot 0
    0, 7, 0, 7, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, // group 0, binding 7 -> slot 1
];

const WIDTH: usize = 128;
const HEIGHT: usize = 128;

fn uniform_bytes(values: [f32; 4]) -> [u8; 16] {
    let mut bytes = [0; 16];
    for (index, value) in values.into_iter().enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_ne_bytes());
    }
    bytes
}

#[test]
fn pinned_binding_witness_draws_through_ore_metal_without_a_backend_facade() {
    assert_eq!(
        format!("{:x}", Sha256::digest(BINDING_WITNESS_MSL)),
        "e13a2df60b11b7522c37725046e19e09620f78a514d783aa907a2efc646869a5"
    );
    assert_eq!(
        format!("{:x}", Sha256::digest(BINDING_WITNESS_MAP)),
        "8a2aa27a73c79b03ee5868aa07b0a25294e281454326df657b0a7c6cf3f5ba22"
    );

    let device = MTLCreateSystemDefaultDevice().expect("the macOS ORE witness requires Metal");
    let queue = device
        .newCommandQueue()
        .expect("create the pinned witness command queue");
    let mut context = *ContextMetal::MakeChecked(Some(device.clone()), Some(queue))
        .expect("retained Metal device and queue construct ORE");

    let module = context
        .makeShaderModule(&ShaderModuleDesc {
            code: Some(BINDING_WITNESS_MSL),
            codeSize: u32::try_from(BINDING_WITNESS_MSL.len()).expect("small witness MSL"),
            bindingMapBytes: Some(&BINDING_WITNESS_MAP),
            bindingMapSize: u32::try_from(BINDING_WITNESS_MAP.len())
                .expect("small witness binding map"),
            label: Some("pinned ore_binding_witness"),
            ..ShaderModuleDesc::default()
        })
        .expect("compile the pinned target-2 MSL and target-10 BindingMap");

    let module_metal = module
        .downcast_ref::<ShaderModuleMetal>()
        .expect("ContextMetal publishes ShaderModuleMetal");
    let layout_entries = (0..module_metal.bindingMap().size())
        .filter_map(|index| {
            let reflected = module_metal.bindingMap().at(index);
            (reflected.group == 0).then(|| {
                assert_eq!(reflected.kind, ResourceKind::UniformBuffer);
                BindGroupLayoutEntry {
                    binding: u32::from(reflected.binding),
                    kind: BindingKind::uniformBuffer,
                    visibility: StageVisibility {
                        mask: reflected.stageMask,
                    },
                    nativeSlotVS: if reflected.backendSlot[0] == BindingMap::kAbsent {
                        BindGroupLayoutEntry::kNativeSlotAbsent
                    } else {
                        u32::from(reflected.backendSlot[0])
                    },
                    nativeSlotFS: if reflected.backendSlot[1] == BindingMap::kAbsent {
                        BindGroupLayoutEntry::kNativeSlotAbsent
                    } else {
                        u32::from(reflected.backendSlot[1])
                    },
                    // The pinned helper derives layouts for render pipelines
                    // and leaves the compute-native slot absent.
                    nativeSlotCS: BindGroupLayoutEntry::kNativeSlotAbsent,
                    ..BindGroupLayoutEntry::default()
                }
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        layout_entries
            .iter()
            .map(|entry| (entry.binding, entry.nativeSlotFS))
            .collect::<Vec<_>>(),
        [(0, 0), (7, 1)],
        "the runtime layout must be derived from the exact target-10 map"
    );
    let incomplete_layout = context
        .makeBindGroupLayout(&BindGroupLayoutDesc {
            groupIndex: 0,
            entries: &layout_entries[..1],
            entryCount: 1,
            label: Some("intentionally incomplete witness group 0"),
        })
        .expect("materialize the incomplete negative-case layout");
    let incomplete_pipeline_layouts = [Some(&incomplete_layout)];
    let mut incomplete_error = String::new();
    assert!(
        context
            .makePipeline(
                &PipelineDesc {
                    vertexModule: Some(&module),
                    fragmentModule: Some(&module),
                    bindGroupLayouts: Some(&incomplete_pipeline_layouts),
                    bindGroupLayoutCount: 1,
                    ..PipelineDesc::default()
                },
                Some(&mut incomplete_error),
            )
            .is_none(),
        "the target-10 map must reject a layout that omits authored binding 7"
    );
    assert_eq!(
        incomplete_error,
        "@group(0) @binding(7): layout has no entry for this binding (shader expects uniformBuffer)"
    );

    let layout = context
        .makeBindGroupLayout(&BindGroupLayoutDesc {
            groupIndex: 0,
            entries: &layout_entries,
            entryCount: u32::try_from(layout_entries.len()).expect("small witness layout"),
            label: Some("pinned witness group 0"),
        })
        .expect("materialize the exact authored binding layout");
    let pipeline_layouts = [Some(&layout)];
    let pipeline = context
        .makePipeline(
            &PipelineDesc {
                vertexModule: Some(&module),
                vertexEntryPoint: Some("vs_main"),
                fragmentModule: Some(&module),
                fragmentEntryPoint: Some("fs_main"),
                colorTargets: [
                    ColorTargetState {
                        format: TextureFormat::rgba8unorm,
                        ..ColorTargetState::default()
                    },
                    ColorTargetState::default(),
                    ColorTargetState::default(),
                    ColorTargetState::default(),
                ],
                bindGroupLayouts: Some(&pipeline_layouts),
                bindGroupLayoutCount: 1,
                label: Some("pinned witness pipeline"),
                ..PipelineDesc::default()
            },
            None,
        )
        .expect("build the exact witness pipeline");

    let low_bytes = uniform_bytes([0.3, 0.0, 0.0, 0.0]);
    let high_bytes = uniform_bytes([0.0, 0.6, 0.0, 0.0]);
    let low = context
        .makeBuffer(
            &BufferDesc::initialized(BufferUsage::uniform, &low_bytes, true)
                .expect("small low uniform descriptor"),
        )
        .expect("create authored binding 0");
    let high = context
        .makeBuffer(
            &BufferDesc::initialized(BufferUsage::uniform, &high_bytes, true)
                .expect("small high uniform descriptor"),
        )
        .expect("create authored binding 7");
    let ubos = [
        UBOEntry {
            slot: 0,
            buffer: Some(&low),
            offset: 0,
            size: 16,
        },
        UBOEntry {
            slot: 7,
            buffer: Some(&high),
            offset: 0,
            size: 16,
        },
    ];
    let group = context
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&layout),
            ubos: &ubos,
            uboCount: u32::try_from(ubos.len()).expect("small witness UBO set"),
            label: Some("pinned witness group"),
            ..BindGroupDesc::default()
        })
        .expect("bind both sparse authored uniform slots");
    let repeat_high_bytes = uniform_bytes([0.0, 0.4, 0.0, 0.0]);
    let repeat_high = context
        .makeBuffer(
            &BufferDesc::initialized(BufferUsage::uniform, &repeat_high_bytes, true)
                .expect("small repeated high uniform descriptor"),
        )
        .expect("create the repeated authored binding 7");
    let repeat_ubos = [
        UBOEntry {
            slot: 0,
            buffer: Some(&low),
            offset: 0,
            size: 16,
        },
        UBOEntry {
            slot: 7,
            buffer: Some(&repeat_high),
            offset: 0,
            size: 16,
        },
    ];
    let repeat_group = context
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&layout),
            ubos: &repeat_ubos,
            uboCount: u32::try_from(repeat_ubos.len()).expect("small witness UBO set"),
            label: Some("repeated witness group"),
            ..BindGroupDesc::default()
        })
        .expect("bind the observably different repeated uniform");

    let descriptor = MTLTextureDescriptor::new();
    descriptor.setPixelFormat(MTLPixelFormat::RGBA8Unorm);
    descriptor.setStorageMode(MTLStorageMode::Shared);
    descriptor.setUsage(MTLTextureUsage::RenderTarget);
    // SAFETY: both non-zero extents fit NSUInteger and describe one ordinary
    // two-dimensional texture mip level.
    unsafe {
        descriptor.setWidth(WIDTH);
        descriptor.setHeight(HEIGHT);
        descriptor.setMipmapLevelCount(1);
    }
    let texture = device
        .newTextureWithDescriptor(&descriptor)
        .expect("allocate shared witness render target");
    let view = context
        .wrap_native_texture(texture.clone(), WIDTH as u32, HEIGHT as u32, true)
        .expect("witness texture belongs to the context device");

    let frames = [(&group, 153_u8), (&repeat_group, 102_u8)];
    for (frame_index, (frame_group, expected_green)) in frames.into_iter().enumerate() {
        let expected_serial = (frame_index + 1) as u64;
        context.beginFrame(&FrameDescriptor {
            externalCommandBuffer: None,
            safeFrameNumber: 0,
            currentFrameNumber: expected_serial,
        });
        let mut pass = context
            .beginRenderPass(
                &RenderPassDesc {
                    colorAttachments: [
                        ColorAttachment {
                            view: Some(&view),
                            clearColor: ClearColor {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            },
                            ..ColorAttachment::default()
                        },
                        ColorAttachment::default(),
                        ColorAttachment::default(),
                        ColorAttachment::default(),
                    ],
                    label: Some("pinned witness pass"),
                    ..RenderPassDesc::default()
                },
                None,
            )
            .expect("begin witness render pass");
        pass.setPipeline(Some(&pipeline));
        pass.setBindGroup(0, Some(frame_group), None, 0);
        pass.setViewport(0.0, 0.0, WIDTH as f32, HEIGHT as f32, 0.0, 1.0);
        pass.draw(3, 1, 0, 0);
        pass.finish();
        assert_eq!(context.currentSerial(), expected_serial);
        context.endFrame();

        let deadline = Instant::now() + Duration::from_secs(5);
        while context.completedSerial() < expected_serial && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            context.completedSerial(),
            expected_serial,
            "Metal completion timed out"
        );

        let mut pixels = vec![0_u8; WIDTH * HEIGHT * 4];
        let pointer = NonNull::new(pixels.as_mut_ptr().cast::<c_void>())
            .expect("non-empty readback allocation");
        // SAFETY: `pixels` owns WIDTH*HEIGHT*4 writable bytes, the row pitch
        // is exact for RGBA8, and the region is the complete mip-0 extent.
        unsafe {
            texture.getBytes_bytesPerRow_fromRegion_mipmapLevel(
                pointer,
                WIDTH * 4,
                MTLRegion {
                    origin: objc2_metal::MTLOrigin { x: 0, y: 0, z: 0 },
                    size: objc2_metal::MTLSize {
                        width: WIDTH,
                        height: HEIGHT,
                        depth: 1,
                    },
                },
                0,
            );
        }

        let histogram =
            pixels
                .chunks_exact(4)
                .fold(BTreeMap::<[u8; 4], usize>::new(), |mut counts, pixel| {
                    *counts
                        .entry(pixel.try_into().expect("RGBA pixel"))
                        .or_default() += 1;
                    counts
                });
        assert_eq!(histogram.values().sum::<usize>(), WIDTH * HEIGHT);
        assert!(
            histogram.keys().all(|pixel| {
                (76..=78).contains(&pixel[0])
                    && (expected_green.saturating_sub(1)..=expected_green.saturating_add(1))
                        .contains(&pixel[1])
                    && pixel[2] == 0
                    && pixel[3] == 255
            }),
            "frame {expected_serial} must contain both authored uniforms; histogram={histogram:?}"
        );
        assert!(
            pixels.chunks_exact(4).all(|pixel| pixel[1] > pixel[0]),
            "binding 7 must contribute green; the historical broken route rendered red"
        );
    }
}
