//! Complete mechanical declaration translation of
//! `renderer/src/vulkan/common_layouts.hpp`.

#![allow(non_snake_case, non_upper_case_globals)]

use super::vkutil_decl::kColorWriteMaskRGBA;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    ImageDrawInstance, ImageRectVertex, PatchVertex, TriangleVertex,
};
use ash::vk;
use std::sync::LazyLock;

// Typed expansion of the pinned `shaders/constants.glsl` dependency.
const PLS_PLANE_COUNT: u32 = 4;
const IMAGE_FIRST_ATTRIB_IDX: u32 = 2;
const IMAGE_VIEW_MATRIX_ATTRIB_IDX: u32 = 2;
const IMAGE_CLIP_RECT_INVERSE_MATRIX_ATTRIB_IDX: u32 = 3;
const IMAGE_TRANSLATES_ATTRIB_IDX: u32 = 4;
const IMAGE_PACKED_ATTRIBS_IDX: u32 = 5;
const IMAGE_LAST_ATTRIB_IDX: u32 = 5;
const IMAGE_ATTRIB_COUNT: usize =
    (IMAGE_LAST_ATTRIB_IDX + 1 - IMAGE_FIRST_ATTRIB_IDX) as usize;

pub(crate) const MAX_RENDER_PASS_ATTACHMENTS: u32 = PLS_PLANE_COUNT + 1;

pub(crate) static PATH_INPUT_BINDINGS: [vk::VertexInputBindingDescription; 1] =
    [vk::VertexInputBindingDescription {
        binding: 0,
        stride: core::mem::size_of::<PatchVertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    }];

pub(crate) static PATH_VERTEX_ATTRIBS: [vk::VertexInputAttributeDescription; 2] = [
    vk::VertexInputAttributeDescription {
        location: 0,
        binding: 0,
        format: vk::Format::R32G32B32A32_SFLOAT,
        offset: 0,
    },
    vk::VertexInputAttributeDescription {
        location: 1,
        binding: 0,
        format: vk::Format::R32G32B32A32_SFLOAT,
        offset: 4 * core::mem::size_of::<f32>() as u32,
    },
];

pub(crate) static PATH_VERTEX_INPUT_STATE: LazyLock<
    vk::PipelineVertexInputStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&PATH_INPUT_BINDINGS)
        .vertex_attribute_descriptions(&PATH_VERTEX_ATTRIBS)
});

pub(crate) static INTERIOR_TRI_INPUT_BINDINGS: [vk::VertexInputBindingDescription; 1] =
    [vk::VertexInputBindingDescription {
        binding: 0,
        stride: core::mem::size_of::<TriangleVertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    }];

pub(crate) static INTERIOR_TRI_VERTEX_ATTRIBS: [vk::VertexInputAttributeDescription; 1] =
    [vk::VertexInputAttributeDescription {
        location: 0,
        binding: 0,
        format: vk::Format::R32G32B32_SFLOAT,
        offset: 0,
    }];

pub(crate) static INTERIOR_TRI_VERTEX_INPUT_STATE: LazyLock<
    vk::PipelineVertexInputStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&INTERIOR_TRI_INPUT_BINDINGS)
        .vertex_attribute_descriptions(&INTERIOR_TRI_VERTEX_ATTRIBS)
});

const fn imageDrawInstanceAttribs(
    binding: u32,
) -> [vk::VertexInputAttributeDescription; IMAGE_ATTRIB_COUNT] {
    [
        vk::VertexInputAttributeDescription {
            location: IMAGE_VIEW_MATRIX_ATTRIB_IDX,
            binding,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: (IMAGE_VIEW_MATRIX_ATTRIB_IDX - IMAGE_FIRST_ATTRIB_IDX)
                * core::mem::size_of::<u32>() as u32
                * 4,
        },
        vk::VertexInputAttributeDescription {
            location: IMAGE_CLIP_RECT_INVERSE_MATRIX_ATTRIB_IDX,
            binding,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: (IMAGE_CLIP_RECT_INVERSE_MATRIX_ATTRIB_IDX - IMAGE_FIRST_ATTRIB_IDX)
                * core::mem::size_of::<u32>() as u32
                * 4,
        },
        vk::VertexInputAttributeDescription {
            location: IMAGE_TRANSLATES_ATTRIB_IDX,
            binding,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: (IMAGE_TRANSLATES_ATTRIB_IDX - IMAGE_FIRST_ATTRIB_IDX)
                * core::mem::size_of::<u32>() as u32
                * 4,
        },
        vk::VertexInputAttributeDescription {
            location: IMAGE_PACKED_ATTRIBS_IDX,
            binding,
            format: vk::Format::R32G32B32A32_UINT,
            offset: (IMAGE_PACKED_ATTRIBS_IDX - IMAGE_FIRST_ATTRIB_IDX)
                * core::mem::size_of::<u32>() as u32
                * 4,
        },
    ]
}

// Rust's stable const generics cannot spell `[T; N + IMAGE_ATTRIB_COUNT]` in
// the return type. `OUT` is checked during constant evaluation and preserves
// the source template's exact concatenation operation.
pub(crate) const fn appendImageDrawInstanceAttribs<const N: usize, const OUT: usize>(
    binding: u32,
    geometryAttribs: [vk::VertexInputAttributeDescription; N],
) -> [vk::VertexInputAttributeDescription; OUT] {
    assert!(OUT == N + IMAGE_ATTRIB_COUNT);
    let mut result = [vk::VertexInputAttributeDescription {
        location: 0,
        binding: 0,
        format: vk::Format::UNDEFINED,
        offset: 0,
    }; OUT];
    let mut index = 0;
    while index < N {
        result[index] = geometryAttribs[index];
        index += 1;
    }
    let image = imageDrawInstanceAttribs(binding);
    let mut image_index = 0;
    while image_index < IMAGE_ATTRIB_COUNT {
        result[N + image_index] = image[image_index];
        image_index += 1;
    }
    result
}

pub(crate) const ImageRectGeometryBufferBinding: u32 = 0;
pub(crate) const ImageRectImageAttribBufferBinding: u32 = 1;
pub(crate) static ImageRectInputBindings: [vk::VertexInputBindingDescription; 2] = [
    vk::VertexInputBindingDescription {
        binding: ImageRectGeometryBufferBinding,
        stride: core::mem::size_of::<ImageRectVertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    },
    vk::VertexInputBindingDescription {
        binding: ImageRectImageAttribBufferBinding,
        stride: core::mem::size_of::<ImageDrawInstance>() as u32,
        input_rate: vk::VertexInputRate::INSTANCE,
    },
];
pub(crate) static ImageRectVertexAttribs: [vk::VertexInputAttributeDescription; 5] =
    appendImageDrawInstanceAttribs::<1, 5>(
        ImageRectImageAttribBufferBinding,
        [vk::VertexInputAttributeDescription {
            location: 0,
            binding: ImageRectGeometryBufferBinding,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 0,
        }],
    );
pub(crate) static IMAGE_RECT_VERTEX_INPUT_STATE: LazyLock<
    vk::PipelineVertexInputStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&ImageRectInputBindings)
        .vertex_attribute_descriptions(&ImageRectVertexAttribs)
});

pub(crate) const ImageMeshVertexBufferBinding: u32 = 0;
pub(crate) const ImageMeshUVBufferBinding: u32 = 1;
pub(crate) const ImageMeshImageAttribBufferBinding: u32 = 2;
pub(crate) static ImageMeshInputBindings: [vk::VertexInputBindingDescription; 3] = [
    vk::VertexInputBindingDescription {
        binding: ImageMeshVertexBufferBinding,
        stride: core::mem::size_of::<f32>() as u32 * 2,
        input_rate: vk::VertexInputRate::VERTEX,
    },
    vk::VertexInputBindingDescription {
        binding: ImageMeshUVBufferBinding,
        stride: core::mem::size_of::<f32>() as u32 * 2,
        input_rate: vk::VertexInputRate::VERTEX,
    },
    vk::VertexInputBindingDescription {
        binding: ImageMeshImageAttribBufferBinding,
        stride: core::mem::size_of::<ImageDrawInstance>() as u32,
        input_rate: vk::VertexInputRate::INSTANCE,
    },
];
pub(crate) static ImageMeshVertexAttribs: [vk::VertexInputAttributeDescription; 6] =
    appendImageDrawInstanceAttribs::<2, 6>(
        ImageMeshImageAttribBufferBinding,
        [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: ImageMeshVertexBufferBinding,
                format: vk::Format::R32G32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: ImageMeshUVBufferBinding,
                format: vk::Format::R32G32_SFLOAT,
                offset: 0,
            },
        ],
    );
pub(crate) static IMAGE_MESH_VERTEX_INPUT_STATE: LazyLock<
    vk::PipelineVertexInputStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&ImageMeshInputBindings)
        .vertex_attribute_descriptions(&ImageMeshVertexAttribs)
});

pub(crate) static EMPTY_VERTEX_INPUT_STATE: LazyLock<
    vk::PipelineVertexInputStateCreateInfo<'static>,
> = LazyLock::new(vk::PipelineVertexInputStateCreateInfo::default);

pub(crate) static INPUT_ASSEMBLY_TRIANGLE_STRIP: LazyLock<
    vk::PipelineInputAssemblyStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_STRIP)
});

pub(crate) static INPUT_ASSEMBLY_TRIANGLE_LIST: LazyLock<
    vk::PipelineInputAssemblyStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
});

pub(crate) static SINGLE_VIEWPORT: LazyLock<vk::PipelineViewportStateCreateInfo<'static>> =
    LazyLock::new(|| {
        vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1)
    });

pub(crate) static RASTER_STATE_CULL_BACK_CCW: LazyLock<
    vk::PipelineRasterizationStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0)
});

pub(crate) static RASTER_STATE_CULL_BACK_CW: LazyLock<
    vk::PipelineRasterizationStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .line_width(1.0)
});

pub(crate) static RASTER_STATE_CULL_NONE_CW: LazyLock<
    vk::PipelineRasterizationStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::CLOCKWISE)
        .line_width(1.0)
});

pub(crate) static MSAA_DISABLED: LazyLock<vk::PipelineMultisampleStateCreateInfo<'static>> =
    LazyLock::new(|| {
        vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
    });

pub(crate) static BLEND_DISABLED_VALUES: LazyLock<vk::PipelineColorBlendAttachmentState> =
    LazyLock::new(|| {
        vk::PipelineColorBlendAttachmentState::default().color_write_mask(kColorWriteMaskRGBA)
    });

pub(crate) static SINGLE_ATTACHMENT_BLEND_DISABLED: LazyLock<
    vk::PipelineColorBlendStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(core::slice::from_ref(&*BLEND_DISABLED_VALUES))
});

pub(crate) static DYNAMIC_VIEWPORT_SCISSOR_VALUES: [vk::DynamicState; 2] =
    [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
pub(crate) static DYNAMIC_VIEWPORT_SCISSOR: LazyLock<
    vk::PipelineDynamicStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineDynamicStateCreateInfo::default()
        .dynamic_states(&DYNAMIC_VIEWPORT_SCISSOR_VALUES)
});

pub(crate) static DYNAMIC_PIPELINE_STATE_VALUES: [vk::DynamicState; 8] = [
    vk::DynamicState::VIEWPORT,
    vk::DynamicState::SCISSOR,
    vk::DynamicState::DEPTH_WRITE_ENABLE,
    vk::DynamicState::STENCIL_COMPARE_MASK,
    vk::DynamicState::STENCIL_WRITE_MASK,
    vk::DynamicState::STENCIL_OP,
    vk::DynamicState::CULL_MODE,
    vk::DynamicState::COLOR_WRITE_ENABLE_EXT,
];
pub(crate) static DYNAMIC_PIPELINE_STATE: LazyLock<
    vk::PipelineDynamicStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineDynamicStateCreateInfo::default()
        .dynamic_states(&DYNAMIC_PIPELINE_STATE_VALUES)
});

pub(crate) static SINGLE_ATTACHMENT_SUBPASS_REFERENCE: LazyLock<vk::AttachmentReference> =
    LazyLock::new(|| {
        vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
    });
pub(crate) static SINGLE_ATTACHMENT_SUBPASS: LazyLock<vk::SubpassDescription<'static>> =
    LazyLock::new(|| {
        vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(core::slice::from_ref(
                &*SINGLE_ATTACHMENT_SUBPASS_REFERENCE,
            ))
    });

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_vertex_layout_denominator_matches_source() {
        assert_eq!(MAX_RENDER_PASS_ATTACHMENTS, 5);
        assert_eq!(PATH_INPUT_BINDINGS[0].stride, 32);
        assert_eq!(PATH_VERTEX_ATTRIBS.len(), 2);
        assert_eq!(PATH_VERTEX_ATTRIBS[1].offset, 16);
        assert_eq!(INTERIOR_TRI_INPUT_BINDINGS[0].stride, 12);
        assert_eq!(ImageRectInputBindings[0].stride, 16);
        assert_eq!(ImageRectInputBindings[1].stride, 64);
        assert_eq!(ImageRectVertexAttribs.len(), 5);
        assert_eq!(ImageMeshInputBindings.len(), 3);
        assert_eq!(ImageMeshVertexAttribs.len(), 6);
        assert_eq!(
            ImageMeshVertexAttribs.map(|attribute| attribute.location),
            [0, 1, 2, 3, 4, 5]
        );
        assert_eq!(
            ImageMeshVertexAttribs.map(|attribute| attribute.offset),
            [0, 0, 0, 16, 32, 48]
        );
        assert_eq!(PATH_VERTEX_INPUT_STATE.vertex_binding_description_count, 1);
        assert_eq!(PATH_VERTEX_INPUT_STATE.vertex_attribute_description_count, 2);
        assert_eq!(
            PATH_VERTEX_INPUT_STATE.p_vertex_binding_descriptions,
            PATH_INPUT_BINDINGS.as_ptr()
        );
        assert_eq!(
            PATH_VERTEX_INPUT_STATE.p_vertex_attribute_descriptions,
            PATH_VERTEX_ATTRIBS.as_ptr()
        );
        assert_eq!(IMAGE_RECT_VERTEX_INPUT_STATE.vertex_binding_description_count, 2);
        assert_eq!(IMAGE_RECT_VERTEX_INPUT_STATE.vertex_attribute_description_count, 5);
        assert_eq!(IMAGE_MESH_VERTEX_INPUT_STATE.vertex_binding_description_count, 3);
        assert_eq!(IMAGE_MESH_VERTEX_INPUT_STATE.vertex_attribute_description_count, 6);
        assert_eq!(
            IMAGE_MESH_VERTEX_INPUT_STATE.p_vertex_binding_descriptions,
            ImageMeshInputBindings.as_ptr()
        );
        assert_eq!(
            IMAGE_MESH_VERTEX_INPUT_STATE.p_vertex_attribute_descriptions,
            ImageMeshVertexAttribs.as_ptr()
        );
        assert_eq!(EMPTY_VERTEX_INPUT_STATE.vertex_binding_description_count, 0);
        assert_eq!(EMPTY_VERTEX_INPUT_STATE.vertex_attribute_description_count, 0);
    }

    #[test]
    fn complete_fixed_and_dynamic_pipeline_state_denominator_matches_source() {
        assert_eq!(INPUT_ASSEMBLY_TRIANGLE_STRIP.topology, vk::PrimitiveTopology::TRIANGLE_STRIP);
        assert_eq!(INPUT_ASSEMBLY_TRIANGLE_LIST.topology, vk::PrimitiveTopology::TRIANGLE_LIST);
        assert_eq!(SINGLE_VIEWPORT.viewport_count, 1);
        assert_eq!(SINGLE_VIEWPORT.scissor_count, 1);
        assert_eq!(RASTER_STATE_CULL_BACK_CCW.cull_mode, vk::CullModeFlags::BACK);
        assert_eq!(RASTER_STATE_CULL_BACK_CCW.front_face, vk::FrontFace::COUNTER_CLOCKWISE);
        assert_eq!(RASTER_STATE_CULL_BACK_CW.front_face, vk::FrontFace::CLOCKWISE);
        assert_eq!(RASTER_STATE_CULL_NONE_CW.cull_mode, vk::CullModeFlags::NONE);
        assert_eq!(MSAA_DISABLED.rasterization_samples, vk::SampleCountFlags::TYPE_1);
        assert_eq!(BLEND_DISABLED_VALUES.color_write_mask, kColorWriteMaskRGBA);
        assert_eq!(SINGLE_ATTACHMENT_BLEND_DISABLED.attachment_count, 1);
        assert_eq!(
            SINGLE_ATTACHMENT_BLEND_DISABLED.p_attachments,
            core::ptr::from_ref(&*BLEND_DISABLED_VALUES)
        );
        assert_eq!(DYNAMIC_VIEWPORT_SCISSOR_VALUES, [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);
        assert_eq!(DYNAMIC_PIPELINE_STATE_VALUES, [
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::SCISSOR,
            vk::DynamicState::DEPTH_WRITE_ENABLE,
            vk::DynamicState::STENCIL_COMPARE_MASK,
            vk::DynamicState::STENCIL_WRITE_MASK,
            vk::DynamicState::STENCIL_OP,
            vk::DynamicState::CULL_MODE,
            vk::DynamicState::COLOR_WRITE_ENABLE_EXT,
        ]);
        assert_eq!(DYNAMIC_PIPELINE_STATE.dynamic_state_count, 8);
        assert_eq!(
            DYNAMIC_PIPELINE_STATE.p_dynamic_states,
            DYNAMIC_PIPELINE_STATE_VALUES.as_ptr()
        );
        assert_eq!(SINGLE_ATTACHMENT_SUBPASS_REFERENCE.attachment, 0);
        assert_eq!(SINGLE_ATTACHMENT_SUBPASS_REFERENCE.layout, vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        assert_eq!(SINGLE_ATTACHMENT_SUBPASS.pipeline_bind_point, vk::PipelineBindPoint::GRAPHICS);
        assert_eq!(SINGLE_ATTACHMENT_SUBPASS.color_attachment_count, 1);
        assert_eq!(
            SINGLE_ATTACHMENT_SUBPASS.p_color_attachments,
            core::ptr::from_ref(&*SINGLE_ATTACHMENT_SUBPASS_REFERENCE)
        );
    }
}
