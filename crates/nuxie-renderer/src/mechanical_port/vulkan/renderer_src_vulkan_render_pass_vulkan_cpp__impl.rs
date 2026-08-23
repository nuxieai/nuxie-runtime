//! Complete mechanical implementation translation of
//! `renderer/src/vulkan/render_pass_vulkan.cpp`.

#![allow(non_snake_case)]

use super::render_pass_vulkan_decl::{
    RenderPassOptionsVulkan, RenderPassVulkan, FORMAT_BIT_COUNT, KEY_BIT_COUNT,
    KEY_NO_INTERLOCK_MODE_BIT_COUNT, LOAD_OP_BIT_COUNT, RENDER_PASS_OPTION_COUNT,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    InterlockMode, LoadAction, INTERLOCK_MODE_BIT_COUNT,
};
use ash::vk;
use std::ffi::CString;
use std::sync::Arc;

use super::draw_pipeline_layout_vulkan_decl::DrawPipelineLayoutVulkan;
use super::vkutil_decl;
use super::vulkan_context_decl::VulkanContext;

const LAST_NON_SPARSE_VK_FORMAT: i32 = vk::Format::ASTC_12X12_SRGB_BLOCK.as_raw();

// Exact return indices from the pinned switch. Raw values are used for the
// formats newer than ash's pinned Vulkan-Headers revision.
const SPARSE_FORMAT_INDICES: &[(i32, u32)] = &[
    (vk::Format::G8B8G8R8_422_UNORM.as_raw(), 0),
    (vk::Format::B8G8R8G8_422_UNORM.as_raw(), 1),
    (vk::Format::G8_B8_R8_3PLANE_420_UNORM.as_raw(), 2),
    (vk::Format::G8_B8R8_2PLANE_420_UNORM.as_raw(), 3),
    (vk::Format::G8_B8_R8_3PLANE_422_UNORM.as_raw(), 4),
    (vk::Format::G8_B8R8_2PLANE_422_UNORM.as_raw(), 5),
    (vk::Format::G8_B8_R8_3PLANE_444_UNORM.as_raw(), 6),
    (vk::Format::R10X6_UNORM_PACK16.as_raw(), 7),
    (vk::Format::R10X6G10X6_UNORM_2PACK16.as_raw(), 8),
    (vk::Format::R10X6G10X6B10X6A10X6_UNORM_4PACK16.as_raw(), 9),
    (
        vk::Format::G10X6B10X6G10X6R10X6_422_UNORM_4PACK16.as_raw(),
        10,
    ),
    (
        vk::Format::B10X6G10X6R10X6G10X6_422_UNORM_4PACK16.as_raw(),
        11,
    ),
    (
        vk::Format::G10X6_B10X6_R10X6_3PLANE_420_UNORM_3PACK16.as_raw(),
        12,
    ),
    (
        vk::Format::G10X6_B10X6R10X6_2PLANE_420_UNORM_3PACK16.as_raw(),
        13,
    ),
    (
        vk::Format::G10X6_B10X6_R10X6_3PLANE_422_UNORM_3PACK16.as_raw(),
        14,
    ),
    (
        vk::Format::G10X6_B10X6R10X6_2PLANE_422_UNORM_3PACK16.as_raw(),
        15,
    ),
    (
        vk::Format::G10X6_B10X6_R10X6_3PLANE_444_UNORM_3PACK16.as_raw(),
        16,
    ),
    (vk::Format::R12X4_UNORM_PACK16.as_raw(), 17),
    (vk::Format::R12X4G12X4_UNORM_2PACK16.as_raw(), 18),
    (vk::Format::R12X4G12X4B12X4A12X4_UNORM_4PACK16.as_raw(), 19),
    (
        vk::Format::G12X4B12X4G12X4R12X4_422_UNORM_4PACK16.as_raw(),
        20,
    ),
    (
        vk::Format::B12X4G12X4R12X4G12X4_422_UNORM_4PACK16.as_raw(),
        21,
    ),
    (
        vk::Format::G12X4_B12X4_R12X4_3PLANE_420_UNORM_3PACK16.as_raw(),
        22,
    ),
    (
        vk::Format::G12X4_B12X4R12X4_2PLANE_420_UNORM_3PACK16.as_raw(),
        23,
    ),
    (
        vk::Format::G12X4_B12X4_R12X4_3PLANE_422_UNORM_3PACK16.as_raw(),
        24,
    ),
    (
        vk::Format::G12X4_B12X4R12X4_2PLANE_422_UNORM_3PACK16.as_raw(),
        25,
    ),
    (
        vk::Format::G12X4_B12X4_R12X4_3PLANE_444_UNORM_3PACK16.as_raw(),
        26,
    ),
    (vk::Format::G16B16G16R16_422_UNORM.as_raw(), 27),
    (vk::Format::B16G16R16G16_422_UNORM.as_raw(), 28),
    (vk::Format::G16_B16_R16_3PLANE_420_UNORM.as_raw(), 29),
    (vk::Format::G16_B16R16_2PLANE_420_UNORM.as_raw(), 30),
    (vk::Format::G16_B16_R16_3PLANE_422_UNORM.as_raw(), 31),
    (vk::Format::G16_B16R16_2PLANE_422_UNORM.as_raw(), 32),
    (vk::Format::G16_B16_R16_3PLANE_444_UNORM.as_raw(), 33),
    (vk::Format::G8_B8R8_2PLANE_444_UNORM.as_raw(), 34),
    (
        vk::Format::G10X6_B10X6R10X6_2PLANE_444_UNORM_3PACK16.as_raw(),
        35,
    ),
    (
        vk::Format::G12X4_B12X4R12X4_2PLANE_444_UNORM_3PACK16.as_raw(),
        36,
    ),
    (vk::Format::G16_B16R16_2PLANE_444_UNORM.as_raw(), 37),
    (vk::Format::A4R4G4B4_UNORM_PACK16.as_raw(), 38),
    (vk::Format::A4B4G4R4_UNORM_PACK16.as_raw(), 39),
    (vk::Format::ASTC_4X4_SFLOAT_BLOCK.as_raw(), 40),
    (vk::Format::ASTC_5X4_SFLOAT_BLOCK.as_raw(), 41),
    (vk::Format::ASTC_5X5_SFLOAT_BLOCK.as_raw(), 42),
    (vk::Format::ASTC_6X5_SFLOAT_BLOCK.as_raw(), 43),
    (vk::Format::ASTC_6X6_SFLOAT_BLOCK.as_raw(), 44),
    (vk::Format::ASTC_8X5_SFLOAT_BLOCK.as_raw(), 45),
    (vk::Format::ASTC_8X6_SFLOAT_BLOCK.as_raw(), 46),
    (vk::Format::ASTC_8X8_SFLOAT_BLOCK.as_raw(), 47),
    (vk::Format::ASTC_10X5_SFLOAT_BLOCK.as_raw(), 48),
    (vk::Format::ASTC_10X6_SFLOAT_BLOCK.as_raw(), 49),
    (vk::Format::ASTC_10X8_SFLOAT_BLOCK.as_raw(), 50),
    (vk::Format::ASTC_10X10_SFLOAT_BLOCK.as_raw(), 51),
    (vk::Format::ASTC_12X10_SFLOAT_BLOCK.as_raw(), 52),
    (vk::Format::ASTC_12X12_SFLOAT_BLOCK.as_raw(), 53),
    (1_000_470_000, 54),
    (1_000_470_001, 55),
    (vk::Format::PVRTC1_2BPP_UNORM_BLOCK_IMG.as_raw(), 56),
    (vk::Format::PVRTC1_4BPP_UNORM_BLOCK_IMG.as_raw(), 57),
    (vk::Format::PVRTC2_2BPP_UNORM_BLOCK_IMG.as_raw(), 58),
    (vk::Format::PVRTC2_4BPP_UNORM_BLOCK_IMG.as_raw(), 59),
    (vk::Format::PVRTC1_2BPP_SRGB_BLOCK_IMG.as_raw(), 60),
    (vk::Format::PVRTC1_4BPP_SRGB_BLOCK_IMG.as_raw(), 61),
    (vk::Format::PVRTC2_2BPP_SRGB_BLOCK_IMG.as_raw(), 62),
    (vk::Format::PVRTC2_4BPP_SRGB_BLOCK_IMG.as_raw(), 63),
    (1_000_460_000, 64),
    (1_000_464_000, 65),
    (1_000_609_000, 66),
    (1_000_609_001, 67),
    (1_000_609_002, 68),
    (1_000_609_003, 69),
    (1_000_609_004, 70),
    (1_000_609_005, 71),
    (1_000_609_006, 72),
    (1_000_609_007, 73),
    (1_000_609_008, 74),
    (1_000_609_009, 75),
    (1_000_609_010, 76),
    (1_000_609_011, 77),
    (1_000_609_012, 78),
    (1_000_609_013, 79),
];

fn vk_sparse_format_index(format: vk::Format) -> u32 {
    assert!(format.as_raw() > LAST_NON_SPARSE_VK_FORMAT);
    SPARSE_FORMAT_INDICES
        .iter()
        .find_map(|&(raw, index)| (raw == format.as_raw()).then_some(index))
        .unwrap_or_else(|| panic!("Given sparse VkFormat is not supported"))
}

fn vk_format_key(format: vk::Format) -> u32 {
    if format.as_raw() <= LAST_NON_SPARSE_VK_FORMAT {
        format.as_raw() as u32
    } else {
        vk_sparse_format_index(format) + LAST_NON_SPARSE_VK_FORMAT as u32 + 1
    }
}

pub(crate) fn KeyNoInterlockMode(
    renderPassOptions: RenderPassOptionsVulkan,
    renderTargetFormat: vk::Format,
    loadAction: LoadAction,
) -> u32 {
    assert!((loadAction as u32) < 1 << LOAD_OP_BIT_COUNT);
    let mut key = loadAction as u32;
    let renderFormatKey = vk_format_key(renderTargetFormat);
    assert!(renderFormatKey < 1 << FORMAT_BIT_COUNT);
    assert_eq!(key << FORMAT_BIT_COUNT >> FORMAT_BIT_COUNT, key);
    key = (key << FORMAT_BIT_COUNT) | renderFormatKey;
    assert!(renderPassOptions.0 < 1 << RENDER_PASS_OPTION_COUNT);
    assert_eq!(
        key << RENDER_PASS_OPTION_COUNT >> RENDER_PASS_OPTION_COUNT,
        key
    );
    key = (key << RENDER_PASS_OPTION_COUNT) | renderPassOptions.0;
    assert!(key < 1 << KEY_NO_INTERLOCK_MODE_BIT_COUNT);
    key
}

pub(crate) fn Key(
    interlockMode: InterlockMode,
    renderPassOptions: RenderPassOptionsVulkan,
    renderTargetFormat: vk::Format,
    loadAction: LoadAction,
) -> u32 {
    let mut key = KeyNoInterlockMode(renderPassOptions, renderTargetFormat, loadAction);
    assert_eq!(
        key << INTERLOCK_MODE_BIT_COUNT >> INTERLOCK_MODE_BIT_COUNT,
        key
    );
    assert!((interlockMode as u32) < 1 << INTERLOCK_MODE_BIT_COUNT);
    key = (key << INTERLOCK_MODE_BIT_COUNT) | interlockMode as u32;
    assert!(key < 1 << KEY_BIT_COUNT);
    key
}

const COLOR_PLANE_IDX: usize = 0;
const CLIP_PLANE_IDX: usize = 1;
const SCRATCH_COLOR_PLANE_IDX: usize = 2;
const COVERAGE_PLANE_IDX: usize = 3;
const PLS_PLANE_COUNT: usize = 4;
const COALESCED_ATOMIC_RESOLVE_IDX: usize = SCRATCH_COLOR_PLANE_IDX;
const MSAA_DEPTH_STENCIL_IDX: usize = 1;
const MSAA_RESOLVE_IDX: usize = 2;
const MSAA_COLOR_SEED_IDX: usize = 3;
const MAX_RENDER_PASS_ATTACHMENTS: usize = PLS_PLANE_COUNT + 1;
const MAX_SUBPASSES: usize = 3;
const MAX_SUBPASS_DEPS: usize = 9;

const fn vk_color_load_op(
    loadAction: LoadAction,
    interlockMode: InterlockMode,
) -> vk::AttachmentLoadOp {
    match loadAction {
        LoadAction::preserveRenderTarget => {
            if matches!(interlockMode, InterlockMode::msaa) {
                vk::AttachmentLoadOp::DONT_CARE
            } else {
                vk::AttachmentLoadOp::LOAD
            }
        }
        LoadAction::clear => vk::AttachmentLoadOp::CLEAR,
        LoadAction::dontCare => vk::AttachmentLoadOp::DONT_CARE,
    }
}

fn attachment_ref(attachment: usize, image_layout: vk::ImageLayout) -> vk::AttachmentReference {
    vk::AttachmentReference::default()
        .attachment(attachment as u32)
        .layout(image_layout)
}

fn dependency(
    srcSubpass: u32,
    dstSubpass: u32,
    srcStageMask: vk::PipelineStageFlags,
    dstStageMask: vk::PipelineStageFlags,
    srcAccessMask: vk::AccessFlags,
    dstAccessMask: vk::AccessFlags,
    dependencyFlags: vk::DependencyFlags,
) -> vk::SubpassDependency {
    vk::SubpassDependency::default()
        .src_subpass(srcSubpass)
        .dst_subpass(dstSubpass)
        .src_stage_mask(srcStageMask)
        .dst_stage_mask(dstStageMask)
        .src_access_mask(srcAccessMask)
        .dst_access_mask(dstAccessMask)
        .dependency_flags(dependencyFlags)
}

impl RenderPassVulkan {
    /// `storageTexturePLS` is the exact manager-owned
    /// `plsBackingType(interlockMode)` decision. The pipeline layout pointer is
    /// borrowed from the manager cache and must outlive this render pass.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        vk: &Arc<VulkanContext>,
        drawPipelineLayout: &DrawPipelineLayoutVulkan,
        interlockMode: InterlockMode,
        renderPassOptions: RenderPassOptionsVulkan,
        renderTargetFormat: vk::Format,
        loadAction: LoadAction,
        storageTexturePLS: bool,
    ) -> Self {
        let colorAttachmentLayout =
            if renderPassOptions.has(RenderPassOptionsVulkan::fixedFunctionColorOutput) {
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            } else {
                vk::ImageLayout::GENERAL
            };
        let msaaSampleCount = if interlockMode == InterlockMode::msaa {
            vk::SampleCountFlags::TYPE_4
        } else {
            vk::SampleCountFlags::TYPE_1
        };
        let mut attachments = Vec::with_capacity(MAX_RENDER_PASS_ATTACHMENTS);
        let mut colorAttachmentRefs = Vec::with_capacity(PLS_PLANE_COUNT);
        let mut depthStencilAttachmentRef = None;
        let mut resolveAttachmentRef = None;

        if !storageTexturePLS
            || renderPassOptions.has(RenderPassOptionsVulkan::fixedFunctionColorOutput)
        {
            assert_eq!(attachments.len(), COLOR_PLANE_IDX);
            assert_eq!(colorAttachmentRefs.len(), COLOR_PLANE_IDX);
            attachments.push(
                vk::AttachmentDescription::default()
                    .format(renderTargetFormat)
                    .samples(msaaSampleCount)
                    .load_op(vk_color_load_op(loadAction, interlockMode))
                    .store_op(
                        if renderPassOptions.has(RenderPassOptionsVulkan::manuallyResolved)
                            || renderPassOptions
                                .has(RenderPassOptionsVulkan::atomicCoalescedResolveAndTransfer)
                            || interlockMode == InterlockMode::msaa
                        {
                            vk::AttachmentStoreOp::DONT_CARE
                        } else {
                            vk::AttachmentStoreOp::STORE
                        },
                    )
                    .initial_layout(
                        if (renderPassOptions
                            .has(RenderPassOptionsVulkan::atomicCoalescedResolveAndTransfer)
                            && loadAction != LoadAction::preserveRenderTarget)
                            || interlockMode == InterlockMode::msaa
                        {
                            vk::ImageLayout::UNDEFINED
                        } else {
                            colorAttachmentLayout
                        },
                    )
                    .final_layout(colorAttachmentLayout),
            );
            colorAttachmentRefs.push(attachment_ref(COLOR_PLANE_IDX, colorAttachmentLayout));
        }

        if matches!(
            interlockMode,
            InterlockMode::rasterOrdering | InterlockMode::atomics | InterlockMode::clockwiseAtomic
        ) {
            assert_eq!(attachments.len(), CLIP_PLANE_IDX);
            assert_eq!(colorAttachmentRefs.len(), CLIP_PLANE_IDX);
            let resume = renderPassOptions.has(RenderPassOptionsVulkan::rasterOrderingResume);
            attachments.push(
                vk::AttachmentDescription::default()
                    .format(if interlockMode == InterlockMode::atomics {
                        vk::Format::R8G8B8A8_UNORM
                    } else if interlockMode == InterlockMode::clockwiseAtomic {
                        vk::Format::R16_SFLOAT
                    } else {
                        vk::Format::R32_UINT
                    })
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(if resume {
                        vk::AttachmentLoadOp::LOAD
                    } else {
                        vk::AttachmentLoadOp::CLEAR
                    })
                    .store_op(
                        if renderPassOptions
                            .has(RenderPassOptionsVulkan::rasterOrderingInterruptible)
                        {
                            vk::AttachmentStoreOp::STORE
                        } else {
                            vk::AttachmentStoreOp::DONT_CARE
                        },
                    )
                    .initial_layout(if resume {
                        vk::ImageLayout::GENERAL
                    } else {
                        vk::ImageLayout::UNDEFINED
                    })
                    .final_layout(vk::ImageLayout::GENERAL),
            );
            colorAttachmentRefs.push(attachment_ref(CLIP_PLANE_IDX, vk::ImageLayout::GENERAL));
        }

        if interlockMode == InterlockMode::rasterOrdering {
            let resume = renderPassOptions.has(RenderPassOptionsVulkan::rasterOrderingResume);
            let interruptible =
                renderPassOptions.has(RenderPassOptionsVulkan::rasterOrderingInterruptible);
            assert_eq!(attachments.len(), SCRATCH_COLOR_PLANE_IDX);
            assert_eq!(colorAttachmentRefs.len(), SCRATCH_COLOR_PLANE_IDX);
            attachments.push(
                vk::AttachmentDescription::default()
                    .format(vk::Format::R8G8B8A8_UNORM)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(if resume {
                        vk::AttachmentLoadOp::LOAD
                    } else {
                        vk::AttachmentLoadOp::DONT_CARE
                    })
                    .store_op(if interruptible {
                        vk::AttachmentStoreOp::STORE
                    } else {
                        vk::AttachmentStoreOp::DONT_CARE
                    })
                    .initial_layout(if resume {
                        vk::ImageLayout::GENERAL
                    } else {
                        vk::ImageLayout::UNDEFINED
                    })
                    .final_layout(vk::ImageLayout::GENERAL),
            );
            colorAttachmentRefs.push(attachment_ref(
                SCRATCH_COLOR_PLANE_IDX,
                vk::ImageLayout::GENERAL,
            ));

            assert_eq!(attachments.len(), COVERAGE_PLANE_IDX);
            assert_eq!(colorAttachmentRefs.len(), COVERAGE_PLANE_IDX);
            attachments.push(
                vk::AttachmentDescription::default()
                    .format(vk::Format::R32_UINT)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(if resume {
                        vk::AttachmentLoadOp::LOAD
                    } else {
                        vk::AttachmentLoadOp::CLEAR
                    })
                    .store_op(if interruptible {
                        vk::AttachmentStoreOp::STORE
                    } else {
                        vk::AttachmentStoreOp::DONT_CARE
                    })
                    .initial_layout(if resume {
                        vk::ImageLayout::GENERAL
                    } else {
                        vk::ImageLayout::UNDEFINED
                    })
                    .final_layout(vk::ImageLayout::GENERAL),
            );
            colorAttachmentRefs.push(attachment_ref(COVERAGE_PLANE_IDX, vk::ImageLayout::GENERAL));
            if renderPassOptions.has(RenderPassOptionsVulkan::manuallyResolved) {
                assert_eq!(attachments.len(), PLS_PLANE_COUNT);
                attachments.push(
                    vk::AttachmentDescription::default()
                        .format(renderTargetFormat)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .load_op(vk::AttachmentLoadOp::DONT_CARE)
                        .store_op(vk::AttachmentStoreOp::STORE)
                        .initial_layout(vk::ImageLayout::UNDEFINED)
                        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
                );
                resolveAttachmentRef = Some(attachment_ref(
                    PLS_PLANE_COUNT,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                ));
            }
        } else if interlockMode == InterlockMode::atomics {
            if renderPassOptions.has(RenderPassOptionsVulkan::atomicCoalescedResolveAndTransfer) {
                assert_eq!(attachments.len(), COALESCED_ATOMIC_RESOLVE_IDX);
                attachments.push(
                    vk::AttachmentDescription::default()
                        .format(renderTargetFormat)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .load_op(vk::AttachmentLoadOp::DONT_CARE)
                        .store_op(vk::AttachmentStoreOp::STORE)
                        .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
                );
                assert!(resolveAttachmentRef.is_none());
                resolveAttachmentRef = Some(attachment_ref(
                    COALESCED_ATOMIC_RESOLVE_IDX,
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                ));
            } else {
                const _: () = assert!(COLOR_PLANE_IDX == 0);
                assert!(resolveAttachmentRef.is_none());
                resolveAttachmentRef = Some(colorAttachmentRefs[0]);
            }
        } else if interlockMode == InterlockMode::msaa {
            assert_eq!(attachments.len(), MSAA_DEPTH_STENCIL_IDX);
            attachments.push(
                vk::AttachmentDescription::default()
                    .format(vkutil_decl::get_preferred_depth_stencil_format(
                        vk.supportsD24S8(),
                    ))
                    .samples(msaaSampleCount)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .stencil_load_op(vk::AttachmentLoadOp::CLEAR)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
            );
            depthStencilAttachmentRef = Some(attachment_ref(
                MSAA_DEPTH_STENCIL_IDX,
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ));
            let readsMSAAResolveAttachment = loadAction == LoadAction::preserveRenderTarget
                && !renderPassOptions.has(RenderPassOptionsVulkan::msaaSeedFromOffscreenTexture);
            let msaaResolveLayout = if readsMSAAResolveAttachment {
                vk::ImageLayout::GENERAL
            } else {
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            };
            assert_eq!(attachments.len(), MSAA_RESOLVE_IDX);
            attachments.push(
                vk::AttachmentDescription::default()
                    .format(renderTargetFormat)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(if readsMSAAResolveAttachment {
                        vk::AttachmentLoadOp::LOAD
                    } else {
                        vk::AttachmentLoadOp::DONT_CARE
                    })
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .initial_layout(
                        if readsMSAAResolveAttachment
                            || renderPassOptions.has(RenderPassOptionsVulkan::manuallyResolved)
                        {
                            msaaResolveLayout
                        } else {
                            vk::ImageLayout::UNDEFINED
                        },
                    )
                    .final_layout(msaaResolveLayout),
            );
            resolveAttachmentRef = Some(attachment_ref(MSAA_RESOLVE_IDX, msaaResolveLayout));
            assert_eq!(colorAttachmentRefs.len(), 1);
            if renderPassOptions.has(RenderPassOptionsVulkan::msaaSeedFromOffscreenTexture) {
                assert_eq!(loadAction, LoadAction::preserveRenderTarget);
                assert_eq!(attachments.len(), MSAA_COLOR_SEED_IDX);
                attachments.push(
                    vk::AttachmentDescription::default()
                        .format(renderTargetFormat)
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .load_op(vk::AttachmentLoadOp::LOAD)
                        .store_op(vk::AttachmentStoreOp::DONT_CARE)
                        .initial_layout(vk::ImageLayout::GENERAL)
                        .final_layout(vk::ImageLayout::GENERAL),
                );
            }
        }

        assert!(attachments.len() <= MAX_RENDER_PASS_ATTACHMENTS);
        let mut inputAttachmentRefs = colorAttachmentRefs.clone();
        let mut msaaColorSeedInputAttachmentRef = Vec::with_capacity(1);
        if renderPassOptions.has(RenderPassOptionsVulkan::fixedFunctionColorOutput) {
            if inputAttachmentRefs.len() > 1 {
                inputAttachmentRefs[0].attachment = vk::ATTACHMENT_UNUSED;
            } else {
                inputAttachmentRefs.clear();
            }
        }
        if interlockMode == InterlockMode::msaa && loadAction == LoadAction::preserveRenderTarget {
            msaaColorSeedInputAttachmentRef.push(attachment_ref(
                if renderPassOptions.has(RenderPassOptionsVulkan::msaaSeedFromOffscreenTexture) {
                    MSAA_COLOR_SEED_IDX
                } else {
                    MSAA_RESOLVE_IDX
                },
                vk::ImageLayout::GENERAL,
            ));
        }
        let rasterOrderedAttachmentAccess = interlockMode == InterlockMode::rasterOrdering
            && vk.features.rasterizationOrderColorAttachmentAccess;
        let mut subpassDescs = Vec::with_capacity(MAX_SUBPASSES);
        let mut subpassDeps = Vec::with_capacity(MAX_SUBPASS_DEPS);
        let externalColorInputDependency = dependency(
            vk::SUBPASS_EXTERNAL,
            0,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags::empty(),
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::COLOR_ATTACHMENT_READ,
            vk::DependencyFlags::empty(),
        );
        let addStandardColorDependencyToNextSubpass =
            |deps: &mut Vec<vk::SubpassDependency>, dstSubpassIndex: u32| {
                deps.push(dependency(
                    dstSubpassIndex - 1,
                    dstSubpassIndex,
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    vk::AccessFlags::INPUT_ATTACHMENT_READ,
                    vk::DependencyFlags::BY_REGION,
                ));
            };

        if interlockMode == InterlockMode::msaa && loadAction == LoadAction::preserveRenderTarget {
            assert_eq!(
                msaaColorSeedInputAttachmentRef.len(),
                colorAttachmentRefs.len()
            );
            assert!(subpassDescs.is_empty());
            subpassDescs.push(vk::SubpassDescription {
                pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
                input_attachment_count: msaaColorSeedInputAttachmentRef.len() as u32,
                p_input_attachments: msaaColorSeedInputAttachmentRef.as_ptr(),
                color_attachment_count: colorAttachmentRefs.len() as u32,
                p_color_attachments: colorAttachmentRefs.as_ptr(),
                ..Default::default()
            });
            subpassDeps.push(dependency(
                0,
                0,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                vk::AccessFlags::INPUT_ATTACHMENT_READ,
                vk::DependencyFlags::BY_REGION,
            ));
            subpassDeps.push(externalColorInputDependency);
            if renderPassOptions.has(RenderPassOptionsVulkan::msaaSeedFromOffscreenTexture) {
                subpassDeps.push(dependency(
                    0,
                    vk::SUBPASS_EXTERNAL,
                    vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                    vk::AccessFlags::empty(),
                    vk::DependencyFlags::empty(),
                ));
            }
            let mut externalInputDeps = dependency(
                vk::SUBPASS_EXTERNAL,
                1,
                vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags::empty(),
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
                    | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                vk::DependencyFlags::empty(),
            );
            if !renderPassOptions.has(RenderPassOptionsVulkan::manuallyResolved) {
                externalInputDeps.dst_stage_mask |= vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
                externalInputDeps.dst_access_mask |= vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
            }
            subpassDeps.push(externalInputDeps);
            addStandardColorDependencyToNextSubpass(&mut subpassDeps, subpassDescs.len() as u32);
        } else {
            let mut externalInDep = externalColorInputDependency;
            if interlockMode == InterlockMode::msaa {
                externalInDep.src_stage_mask |= vk::PipelineStageFlags::LATE_FRAGMENT_TESTS;
                externalInDep.dst_stage_mask |= vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
                externalInDep.dst_access_mask |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
            }
            subpassDeps.push(externalInDep);
        }

        if interlockMode == InterlockMode::clockwiseAtomic {
            subpassDescs.push(vk::SubpassDescription {
                pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
                color_attachment_count: colorAttachmentRefs.len() as u32,
                p_color_attachments: colorAttachmentRefs.as_ptr(),
                ..Default::default()
            });
            addStandardColorDependencyToNextSubpass(&mut subpassDeps, subpassDescs.len() as u32);
        }

        let mainSubpassIdx = subpassDescs.len() as u32;
        assert_eq!(
            colorAttachmentRefs.len() as u32,
            drawPipelineLayout.colorAttachmentCount(mainSubpassIdx, renderPassOptions)
        );
        subpassDescs.push(vk::SubpassDescription {
            flags: if rasterOrderedAttachmentAccess {
                vk::SubpassDescriptionFlags::RASTERIZATION_ORDER_ATTACHMENT_COLOR_ACCESS_EXT
            } else {
                vk::SubpassDescriptionFlags::empty()
            },
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: inputAttachmentRefs.len() as u32,
            p_input_attachments: inputAttachmentRefs.as_ptr(),
            color_attachment_count: colorAttachmentRefs.len() as u32,
            p_color_attachments: colorAttachmentRefs.as_ptr(),
            p_resolve_attachments: if interlockMode == InterlockMode::msaa
                && !renderPassOptions.has(RenderPassOptionsVulkan::manuallyResolved)
            {
                resolveAttachmentRef.as_ref().unwrap()
            } else {
                std::ptr::null()
            },
            p_depth_stencil_attachment: depthStencilAttachmentRef
                .as_ref()
                .map_or(std::ptr::null(), std::ptr::from_ref),
            ..Default::default()
        });
        if (interlockMode == InterlockMode::rasterOrdering && !rasterOrderedAttachmentAccess)
            || interlockMode == InterlockMode::atomics
            || interlockMode == InterlockMode::clockwiseAtomic
            || (interlockMode == InterlockMode::msaa
                && !renderPassOptions.has(RenderPassOptionsVulkan::fixedFunctionColorOutput))
        {
            subpassDeps.push(dependency(
                mainSubpassIdx,
                mainSubpassIdx,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                vk::AccessFlags::INPUT_ATTACHMENT_READ,
                vk::DependencyFlags::BY_REGION,
            ));
        }
        if interlockMode == InterlockMode::msaa {
            subpassDeps.push(dependency(
                subpassDescs.len() as u32 - 1,
                vk::SUBPASS_EXTERNAL,
                vk::PipelineStageFlags::LATE_FRAGMENT_TESTS
                    | vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
                    | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                vk::AccessFlags::empty(),
                vk::DependencyFlags::empty(),
            ));
        }
        if interlockMode == InterlockMode::atomics {
            addStandardColorDependencyToNextSubpass(&mut subpassDeps, subpassDescs.len() as u32);
            assert_eq!(subpassDescs.len(), 1);
            assert_eq!(
                drawPipelineLayout.colorAttachmentCount(1, renderPassOptions),
                1
            );
            let resolve = resolveAttachmentRef.as_ref().unwrap();
            subpassDescs.push(vk::SubpassDescription {
                pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
                input_attachment_count: inputAttachmentRefs.len() as u32,
                p_input_attachments: inputAttachmentRefs.as_ptr(),
                color_attachment_count: 1,
                p_color_attachments: resolve,
                ..Default::default()
            });
        } else if renderPassOptions.has(RenderPassOptionsVulkan::manuallyResolved) {
            assert!(!renderPassOptions.has(RenderPassOptionsVulkan::fixedFunctionColorOutput));
            assert!(!renderPassOptions.has(RenderPassOptionsVulkan::rasterOrderingInterruptible));
            assert_eq!(inputAttachmentRefs[0].attachment, COLOR_PLANE_IDX as u32);
            addStandardColorDependencyToNextSubpass(&mut subpassDeps, subpassDescs.len() as u32);
            let resolve = resolveAttachmentRef.as_ref().unwrap();
            subpassDescs.push(vk::SubpassDescription {
                flags: if rasterOrderedAttachmentAccess {
                    vk::SubpassDescriptionFlags::RASTERIZATION_ORDER_ATTACHMENT_COLOR_ACCESS_EXT
                } else {
                    vk::SubpassDescriptionFlags::empty()
                },
                pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
                input_attachment_count: 1,
                p_input_attachments: inputAttachmentRefs.as_ptr(),
                color_attachment_count: 1,
                p_color_attachments: resolve,
                ..Default::default()
            });
        }
        subpassDeps.push(dependency(
            subpassDescs.len() as u32 - 1,
            vk::SUBPASS_EXTERNAL,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::TRANSFER,
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            vk::AccessFlags::empty(),
            vk::DependencyFlags::empty(),
        ));
        assert!(subpassDescs.len() <= MAX_SUBPASSES);
        assert!(subpassDeps.len() <= MAX_SUBPASS_DEPS);
        let info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(&subpassDescs)
            .dependencies(&subpassDeps);
        let renderPass = match unsafe { vk.m_ashDevice.create_render_pass(&info, None) } {
            Ok(renderPass) => renderPass,
            Err(result) => super::vkutil_impl::vk_abort(result, file!(), line!()),
        };
        let label = CString::new(format!(
            "RIVE_Draw{{interlockMode={}, renderPassOptions={}, renderTargetFormat={}, loadAction={}}}",
            interlockMode as i32,
            renderPassOptions.0,
            renderTargetFormat.as_raw(),
            loadAction as i32,
        ))
        .unwrap();
        vk.setDebugNameIfEnabled(
            renderPass,
            vk::ObjectType::RENDER_PASS,
            Some(label.as_c_str()),
        );
        Self {
            m_vk: Arc::clone(vk),
            m_drawPipelineLayout: std::ptr::from_ref(drawPipelineLayout),
            m_renderPass: renderPass,
        }
    }
}

impl From<&RenderPassVulkan> for vk::RenderPass {
    fn from(value: &RenderPassVulkan) -> Self {
        value.m_renderPass
    }
}

impl Drop for RenderPassVulkan {
    fn drop(&mut self) {
        // The source intentionally does not touch m_drawPipelineLayout because
        // its destruction order relative to the manager cache is uncertain.
        unsafe {
            self.m_vk
                .m_ashDevice
                .destroy_render_pass(self.m_renderPass, None)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_key_layout_and_sparse_format_holes_are_exact() {
        assert_eq!(vk_format_key(vk::Format::R8G8B8A8_UNORM), 37);
        assert_eq!(vk_format_key(vk::Format::G8B8G8R8_422_UNORM), 185);
        assert_eq!(vk_format_key(vk::Format::PVRTC1_2BPP_UNORM_BLOCK_IMG), 241);
        assert_eq!(vk_format_key(vk::Format::from_raw(1_000_609_013)), 264);
        assert_eq!(
            KeyNoInterlockMode(
                RenderPassOptionsVulkan::manuallyResolved,
                vk::Format::R8G8B8A8_UNORM,
                LoadAction::preserveRenderTarget,
            ),
            (((LoadAction::preserveRenderTarget as u32) << FORMAT_BIT_COUNT) | 37)
                << RENDER_PASS_OPTION_COUNT
                | RenderPassOptionsVulkan::manuallyResolved.0
        );
    }
}
