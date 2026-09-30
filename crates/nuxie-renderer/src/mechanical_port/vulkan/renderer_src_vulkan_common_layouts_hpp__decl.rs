//! Complete mechanical declaration translation of
//! `renderer/src/vulkan/common_layouts.hpp`.
//! Updated through upstream `c14cb2510071bd4cfa08d52ba5cd44d98c362237`.

#![allow(non_snake_case, non_upper_case_globals)]

use super::vkutil_decl::kColorWriteMaskRGBA;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    ImageMeshInstance, ImageRectInstance, ImageRectVertex, PatchVertex, TriangleVertex,
    VertexAttribute, VertexElementFormat,
};
use ash::vk;
use std::sync::LazyLock;

// Typed expansion of the pinned `shaders/constants.glsl` dependency.
const PLS_PLANE_COUNT: u32 = 4;

pub(crate) const MAX_RENDER_PASS_ATTACHMENTS: u32 = PLS_PLANE_COUNT + 1;

pub(crate) static EMPTY_VERTEX_INPUT_STATE: LazyLock<
    vk::PipelineVertexInputStateCreateInfo<'static>,
> = LazyLock::new(vk::PipelineVertexInputStateCreateInfo::default);

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

pub(crate) const fn getVkFormat(format: VertexElementFormat) -> vk::Format {
    match format {
        VertexElementFormat::float1 => vk::Format::R32_SFLOAT,
        VertexElementFormat::float2 => vk::Format::R32G32_SFLOAT,
        VertexElementFormat::float3 => vk::Format::R32G32B32_SFLOAT,
        VertexElementFormat::float4 => vk::Format::R32G32B32A32_SFLOAT,
        VertexElementFormat::uint8x4 => vk::Format::R8G8B8A8_UINT,
        VertexElementFormat::sint8x4 => vk::Format::R8G8B8A8_SINT,
        VertexElementFormat::unorm8x4 => vk::Format::R8G8B8A8_UNORM,
        VertexElementFormat::snorm8x4 => vk::Format::R8G8B8A8_SNORM,
        VertexElementFormat::uint16x2 => vk::Format::R16G16_UINT,
        VertexElementFormat::sint16x2 => vk::Format::R16G16_SINT,
        VertexElementFormat::unorm16x2 => vk::Format::R16G16_UNORM,
        VertexElementFormat::snorm16x2 => vk::Format::R16G16_SNORM,
        VertexElementFormat::uint16x4 => vk::Format::R16G16B16A16_UINT,
        VertexElementFormat::sint16x4 => vk::Format::R16G16B16A16_SINT,
        VertexElementFormat::float16x2 => vk::Format::R16G16_SFLOAT,
        VertexElementFormat::float16x4 => vk::Format::R16G16B16A16_SFLOAT,
        VertexElementFormat::uint32 => vk::Format::R32_UINT,
    }
}

// The explicit attribute slice selects the source ImageRect/Mesh template.
pub(crate) const fn appendImageDrawInstanceAttribs<const N: usize, const OUT: usize>(
    binding: u32,
    geometryAttribs: [vk::VertexInputAttributeDescription; N],
    attributes: &[VertexAttribute],
) -> [vk::VertexInputAttributeDescription; OUT] {
    assert!(OUT == N + attributes.len());
    let mut result = [vk::VertexInputAttributeDescription {
        location: 0,
        binding: 0,
        format: vk::Format::UNDEFINED,
        offset: 0,
    }; OUT];
    let mut i = 0;
    while i < N {
        result[i] = geometryAttribs[i];
        i += 1;
    }
    let mut i = 0;
    while i < attributes.len() {
        let src = &attributes[i];
        result[N + i] = vk::VertexInputAttributeDescription {
            location: src.attributeIndex,
            binding,
            format: getVkFormat(src.format),
            offset: src.byteOffset,
        };
        i += 1;
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
        stride: core::mem::size_of::<ImageRectInstance>() as u32,
        input_rate: vk::VertexInputRate::INSTANCE,
    },
];
pub(crate) static ImageRectVertexAttribs: [vk::VertexInputAttributeDescription; 12] =
    appendImageDrawInstanceAttribs::<1, 12>(
        ImageRectImageAttribBufferBinding,
        [vk::VertexInputAttributeDescription {
            location: 0,
            binding: ImageRectGeometryBufferBinding,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 0,
        }],
        ImageRectInstance::getAttributes(),
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
        stride: core::mem::size_of::<ImageMeshInstance>() as u32,
        input_rate: vk::VertexInputRate::INSTANCE,
    },
];
pub(crate) static ImageMeshVertexAttribs: [vk::VertexInputAttributeDescription;
    2 + ImageMeshInstance::AttributeCount] =
    appendImageDrawInstanceAttribs::<2, { 2 + ImageMeshInstance::AttributeCount }>(
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
        ImageMeshInstance::getAttributes(),
    );
pub(crate) static IMAGE_MESH_VERTEX_INPUT_STATE: LazyLock<
    vk::PipelineVertexInputStateCreateInfo<'static>,
> = LazyLock::new(|| {
    vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&ImageMeshInputBindings)
        .vertex_attribute_descriptions(&ImageMeshVertexAttribs)
});

pub(crate) const MaxVertexBinding: u32 = ImageMeshImageAttribBufferBinding;

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
pub(crate) static DYNAMIC_VIEWPORT_SCISSOR: LazyLock<vk::PipelineDynamicStateCreateInfo<'static>> =
    LazyLock::new(|| {
        vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&DYNAMIC_VIEWPORT_SCISSOR_VALUES)
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
            .color_attachments(core::slice::from_ref(&*SINGLE_ATTACHMENT_SUBPASS_REFERENCE))
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
        assert_eq!(ImageRectInputBindings[1].stride, 128);
        assert_eq!(ImageRectVertexAttribs.len(), 12);
        assert_eq!(
            ImageRectVertexAttribs.map(|attribute| attribute.location),
            [0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
        );
        assert_eq!(
            ImageRectVertexAttribs.map(|attribute| attribute.offset),
            [0, 0, 16, 32, 48, 52, 56, 60, 64, 80, 96, 112]
        );
        assert_eq!(ImageRectVertexAttribs[4].format, vk::Format::R32_UINT);
        assert!(
            ImageRectVertexAttribs[8..]
                .iter()
                .all(|attribute| { attribute.format == vk::Format::R32G32B32A32_SFLOAT })
        );
        assert_eq!(ImageMeshInputBindings.len(), 3);
        assert_eq!(ImageMeshVertexAttribs.len(), 10);
        assert_eq!(
            ImageMeshVertexAttribs.map(|attribute| attribute.location),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
        assert_eq!(
            ImageMeshVertexAttribs.map(|attribute| attribute.offset),
            [0, 0, 0, 16, 32, 48, 52, 56, 60, 64]
        );
        assert_eq!(PATH_VERTEX_INPUT_STATE.vertex_binding_description_count, 1);
        assert_eq!(
            PATH_VERTEX_INPUT_STATE.vertex_attribute_description_count,
            2
        );
        assert_eq!(
            PATH_VERTEX_INPUT_STATE.p_vertex_binding_descriptions,
            PATH_INPUT_BINDINGS.as_ptr()
        );
        assert_eq!(
            PATH_VERTEX_INPUT_STATE.p_vertex_attribute_descriptions,
            PATH_VERTEX_ATTRIBS.as_ptr()
        );
        assert_eq!(
            IMAGE_RECT_VERTEX_INPUT_STATE.vertex_binding_description_count,
            2
        );
        assert_eq!(
            IMAGE_RECT_VERTEX_INPUT_STATE.vertex_attribute_description_count,
            12
        );
        assert_eq!(
            IMAGE_MESH_VERTEX_INPUT_STATE.vertex_binding_description_count,
            3
        );
        assert_eq!(
            IMAGE_MESH_VERTEX_INPUT_STATE.vertex_attribute_description_count,
            10
        );
        assert_eq!(
            IMAGE_MESH_VERTEX_INPUT_STATE.p_vertex_binding_descriptions,
            ImageMeshInputBindings.as_ptr()
        );
        assert_eq!(
            IMAGE_MESH_VERTEX_INPUT_STATE.p_vertex_attribute_descriptions,
            ImageMeshVertexAttribs.as_ptr()
        );
        assert_eq!(EMPTY_VERTEX_INPUT_STATE.vertex_binding_description_count, 0);
        assert_eq!(
            EMPTY_VERTEX_INPUT_STATE.vertex_attribute_description_count,
            0
        );
    }

    #[test]
    fn complete_fixed_and_dynamic_pipeline_state_denominator_matches_source() {
        assert_eq!(
            INPUT_ASSEMBLY_TRIANGLE_STRIP.topology,
            vk::PrimitiveTopology::TRIANGLE_STRIP
        );
        assert_eq!(
            INPUT_ASSEMBLY_TRIANGLE_LIST.topology,
            vk::PrimitiveTopology::TRIANGLE_LIST
        );
        assert_eq!(SINGLE_VIEWPORT.viewport_count, 1);
        assert_eq!(SINGLE_VIEWPORT.scissor_count, 1);
        assert_eq!(
            RASTER_STATE_CULL_BACK_CCW.cull_mode,
            vk::CullModeFlags::BACK
        );
        assert_eq!(
            RASTER_STATE_CULL_BACK_CCW.front_face,
            vk::FrontFace::COUNTER_CLOCKWISE
        );
        assert_eq!(
            RASTER_STATE_CULL_BACK_CW.front_face,
            vk::FrontFace::CLOCKWISE
        );
        assert_eq!(RASTER_STATE_CULL_NONE_CW.cull_mode, vk::CullModeFlags::NONE);
        assert_eq!(
            MSAA_DISABLED.rasterization_samples,
            vk::SampleCountFlags::TYPE_1
        );
        assert_eq!(BLEND_DISABLED_VALUES.color_write_mask, kColorWriteMaskRGBA);
        assert_eq!(SINGLE_ATTACHMENT_BLEND_DISABLED.attachment_count, 1);
        assert_eq!(
            SINGLE_ATTACHMENT_BLEND_DISABLED.p_attachments,
            core::ptr::from_ref(&*BLEND_DISABLED_VALUES)
        );
        assert_eq!(
            DYNAMIC_VIEWPORT_SCISSOR_VALUES,
            [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]
        );
        assert_eq!(SINGLE_ATTACHMENT_SUBPASS_REFERENCE.attachment, 0);
        assert_eq!(
            SINGLE_ATTACHMENT_SUBPASS_REFERENCE.layout,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        );
        assert_eq!(
            SINGLE_ATTACHMENT_SUBPASS.pipeline_bind_point,
            vk::PipelineBindPoint::GRAPHICS
        );
        assert_eq!(SINGLE_ATTACHMENT_SUBPASS.color_attachment_count, 1);
        assert_eq!(
            SINGLE_ATTACHMENT_SUBPASS.p_color_attachments,
            core::ptr::from_ref(&*SINGLE_ATTACHMENT_SUBPASS_REFERENCE)
        );
    }
}
