//! Mechanical translation of renderer/src/image_draw_attributes.hpp at 2210ed87.
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
        semanticName: "WB",
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
        semanticName: "NB",
    },
    VertexAttribute {
        format: VertexElementFormat::float1,
        attributeIndex: 5,
        byteOffset: 48,
        semanticName: "XB",
    },
    VertexAttribute {
        format: VertexElementFormat::uint32,
        attributeIndex: 6,
        byteOffset: 52,
        semanticName: "YB",
    },
    VertexAttribute {
        format: VertexElementFormat::uint32,
        attributeIndex: 7,
        byteOffset: 56,
        semanticName: "ZB",
    },
    VertexAttribute {
        format: VertexElementFormat::uint32,
        attributeIndex: 8,
        byteOffset: 60,
        semanticName: "MC",
    },
];
pub const ImageRectInstanceAttributes: [VertexAttribute; 7] = ImageDrawInstanceBaseAttributes;
pub const ImageMeshInstanceAttributes: [VertexAttribute; 7] = ImageDrawInstanceBaseAttributes;

const _: () = {
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
        ImageDrawInstanceBase, ImageMeshInstance, ImageRectInstance,
    };
    use core::mem::{offset_of, size_of};
    assert!(size_of::<ImageDrawInstanceBase>() == 64);
    assert!(size_of::<ImageRectInstance>() == 64);
    assert!(size_of::<ImageMeshInstance>() == 64);
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
        offset_of!(ImageDrawInstanceBase, m_opacity)
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
    assert!(offset_of!(ImageMeshInstance, m_commons) == 0);
};
