//! Mechanical translation of renderer/src/image_draw_attributes.hpp at 4921ab81.
//! Semantic names come from the same ordered minifier batch as the shader owners.
#![allow(non_upper_case_globals)]
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    VertexAttribute, VertexElementFormat,
};

pub const ImageDrawInstanceBaseAttributes: [VertexAttribute; 7] = [
    VertexAttribute {
        format: VertexElementFormat::float4,
        attributeIndex: 2,
        byteOffset: 0,
        semanticName: "YB",
    },
    VertexAttribute {
        format: VertexElementFormat::float4,
        attributeIndex: 3,
        byteOffset: 16,
        semanticName: "SB",
    },
    VertexAttribute {
        format: VertexElementFormat::float4,
        attributeIndex: 4,
        byteOffset: 32,
        semanticName: "PB",
    },
    VertexAttribute {
        format: VertexElementFormat::uint32,
        attributeIndex: 5,
        byteOffset: 48,
        semanticName: "ZB",
    },
    VertexAttribute {
        format: VertexElementFormat::uint32,
        attributeIndex: 6,
        byteOffset: 52,
        semanticName: "AC",
    },
    VertexAttribute {
        format: VertexElementFormat::uint32,
        attributeIndex: 7,
        byteOffset: 56,
        semanticName: "BC",
    },
    VertexAttribute {
        format: VertexElementFormat::uint32,
        attributeIndex: 8,
        byteOffset: 60,
        semanticName: "MC",
    },
];
pub const ImageRectInstanceAttributes: [VertexAttribute; 11] = [
    ImageDrawInstanceBaseAttributes[0],
    ImageDrawInstanceBaseAttributes[1],
    ImageDrawInstanceBaseAttributes[2],
    ImageDrawInstanceBaseAttributes[3],
    ImageDrawInstanceBaseAttributes[4],
    ImageDrawInstanceBaseAttributes[5],
    ImageDrawInstanceBaseAttributes[6],
    VertexAttribute { format: VertexElementFormat::float4, attributeIndex: 9, byteOffset: 64, semanticName: "OD" },
    VertexAttribute { format: VertexElementFormat::float4, attributeIndex: 10, byteOffset: 80, semanticName: "PD" },
    VertexAttribute { format: VertexElementFormat::float4, attributeIndex: 11, byteOffset: 96, semanticName: "DD" },
    VertexAttribute { format: VertexElementFormat::float4, attributeIndex: 12, byteOffset: 112, semanticName: "PC" },
];
pub const ImageMeshInstanceAttributes: [VertexAttribute; 8] = [
    ImageDrawInstanceBaseAttributes[0], ImageDrawInstanceBaseAttributes[1],
    ImageDrawInstanceBaseAttributes[2], ImageDrawInstanceBaseAttributes[3],
    ImageDrawInstanceBaseAttributes[4], ImageDrawInstanceBaseAttributes[5],
    ImageDrawInstanceBaseAttributes[6],
    VertexAttribute { format: VertexElementFormat::float4, attributeIndex: 9, byteOffset: 64, semanticName: "HC" },
];

const _: () = {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
        ImageDrawInstanceBase, ImageMeshInstance, ImageRectInstance,
    };
    use core::mem::{offset_of, size_of};
    assert!(size_of::<ImageDrawInstanceBase>() == 64);
    assert!(size_of::<ImageRectInstance>() == 128);
    assert!(size_of::<ImageMeshInstance>() == 80);
    assert!(
        offset_of!(ImageDrawInstanceBase, m_viewMatrix)
            == ImageDrawInstanceBaseAttributes[0].byteOffset as usize
    );
    assert!(
        offset_of!(ImageDrawInstanceBase, m_clipRectInverseMatrix)
            == ImageDrawInstanceBaseAttributes[1].byteOffset as usize
    );
    assert!(
        offset_of!(ImageDrawInstanceBase, m_translate)
            == ImageDrawInstanceBaseAttributes[2].byteOffset as usize
    );
    assert!(
        offset_of!(ImageDrawInstanceBase, m_clipRectInverseTranslate)
            == offset_of!(ImageDrawInstanceBase, m_translate) + 2 * size_of::<f32>()
    );
    assert!(
        offset_of!(ImageDrawInstanceBase, m_modulatedColor)
            == ImageDrawInstanceBaseAttributes[3].byteOffset as usize
    );
    assert!(
        offset_of!(ImageDrawInstanceBase, m_clipID)
            == ImageDrawInstanceBaseAttributes[4].byteOffset as usize
    );
    assert!(
        offset_of!(ImageDrawInstanceBase, m_blendMode)
            == ImageDrawInstanceBaseAttributes[5].byteOffset as usize
    );
    assert!(
        offset_of!(ImageDrawInstanceBase, m_zIndex)
            == ImageDrawInstanceBaseAttributes[6].byteOffset as usize
    );
    assert!(offset_of!(ImageRectInstance, m_commons) == 0);
    assert!(offset_of!(ImageRectInstance, m_imageMatrix) == 64);
    assert!(offset_of!(ImageRectInstance, m_gradientMatrix) == 80);
    assert!(offset_of!(ImageRectInstance, m_imageTranslate) == 96);
    assert!(offset_of!(ImageRectInstance, m_gradientTranslate) == 104);
    assert!(offset_of!(ImageRectInstance, m_gradTextureHorizontalSpan) == 112);
    assert!(offset_of!(ImageRectInstance, m_gradTextureY) == 120);
    assert!(offset_of!(ImageRectInstance, m_gradientType) == 124);
    assert!(offset_of!(ImageMeshInstance, m_commons) == 0);
    assert!(offset_of!(ImageMeshInstance, m_uvTransform) == 64);
};
