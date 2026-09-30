//! Complete GPU-free upload/view range suites from upstream 14bdbfaf.
use super::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::{
    context::{ContextApi, ReplayCaps},
    gpu_resource::AnyResourceHandle,
    types::*,
};

fn texture(
    ctx: &mut DeferredOreContext,
    size: u32,
    kind: TextureType,
    layers: u32,
    mips: u32,
) -> AnyResourceHandle {
    ctx.makeTexture(&TextureDesc {
        width: size,
        height: size,
        depthOrArrayLayers: layers,
        r#type: kind,
        numMipmaps: mips,
        ..TextureDesc::default()
    })
    .unwrap()
}

#[test]
fn zero_upload_fields_fill_from_the_mip_level() {
    // Each source SECTION repeats the successful initial upload.
    for short_buffer in [false, true] {
        let mut ctx = DeferredOreContext::new(ReplayCaps::default());
        let tex = texture(&mut ctx, 8, TextureType::texture2D, 1, 2);
        let pixels = vec![0; 8 * 8 * 4];
        let mut d = TextureDataDesc {
            data: Some(&pixels),
            dataSize: pixels.len() as u32,
            mipLevel: 1,
            ..TextureDataDesc::default()
        };
        // Result::Ok is the approved Rust equivalent of true with empty error.
        assert!(tex.upload(&d).is_ok());
        d.mipLevel = 0;
        if short_buffer {
            d.dataSize = 8;
            assert!(tex.upload(&d).unwrap_err().to_string().contains("needs"));
        } else {
            d.x = 6;
            d.y = 6;
            assert!(tex.upload(&d).is_ok());
        }
    }
}

#[test]
fn upload_regions_outside_the_texture_are_rejected() {
    for section in 0..6 {
        let mut ctx = DeferredOreContext::new(ReplayCaps::default());
        let tex = texture(&mut ctx, 8, TextureType::texture2D, 1, 2);
        let pixels = vec![0; 8 * 8 * 4];
        let mut d = TextureDataDesc {
            data: Some(&pixels),
            ..TextureDataDesc::default()
        };
        match section {
            0 => d.mipLevel = 2,
            1 => d.layer = 1,
            2 => d.x = 8,
            3 => {
                d.x = 4;
                d.width = 8;
            }
            4 => {
                d.width = 4;
                d.bytesPerRow = 8;
            }
            5 => {
                d.height = 4;
                d.rowsPerImage = 2;
            }
            _ => unreachable!(),
        }
        let error = tex.upload(&d).unwrap_err();
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn upload_layers_and_depth_follow_the_texture_type() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    let pixels = vec![0; 8 * 8 * 8 * 4];
    {
        let tex = texture(&mut ctx, 8, TextureType::cube, 1, 1);
        let mut d = TextureDataDesc {
            data: Some(&pixels),
            layer: 5,
            ..TextureDataDesc::default()
        };
        assert!(tex.upload(&d).is_ok());
        d.layer = 6;
        assert!(tex.upload(&d).is_err());
    }
    {
        let tex = texture(&mut ctx, 8, TextureType::texture3D, 8, 1);
        let mut d = TextureDataDesc {
            data: Some(&pixels),
            z: 2,
            depth: 0,
            ..TextureDataDesc::default()
        };
        assert!(tex.upload(&d).is_ok());
        d.depth = 8;
        assert!(tex.upload(&d).is_err());
        d.depth = 1;
        d.layer = 1;
        assert!(tex.upload(&d).is_err());
    }
}

#[test]
fn zero_view_counts_span_the_rest_of_the_texture() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    let tex = texture(&mut ctx, 16, TextureType::array2D, 3, 4);
    let d = TextureViewDesc {
        texture: Some(&tex),
        dimension: TextureViewDimension::array2D,
        baseMipLevel: 1,
        mipCount: 0,
        baseLayer: 1,
        layerCount: 0,
        ..TextureViewDesc::default()
    };
    let view = ctx.makeTextureView(&d).unwrap();
    let view = view.textureViewBase().unwrap();
    assert_eq!(view.mipCount(), 3);
    assert_eq!(view.layerCount(), 2);
}

#[test]
fn view_ranges_past_the_texture_are_rejected_with_a_message() {
    for section in 0..4 {
        let mut ctx = DeferredOreContext::new(ReplayCaps::default());
        let tex = texture(&mut ctx, 16, TextureType::array2D, 3, 4);
        let mut d = TextureViewDesc {
            texture: Some(&tex),
            dimension: TextureViewDimension::array2D,
            ..TextureViewDesc::default()
        };
        match section {
            0 => d.baseMipLevel = 4,
            1 => {
                d.baseMipLevel = 2;
                d.mipCount = 3;
            }
            2 => d.baseLayer = 3,
            3 => {
                d.baseLayer = 1;
                d.layerCount = 3;
            }
            _ => unreachable!(),
        }
        ctx.clearLastError();
        assert!(ctx.makeTextureView(&d).is_none());
        assert!(!ctx.lastError().is_empty());
    }
}

#[test]
fn view_layers_follow_the_texture_type() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    {
        let tex = texture(&mut ctx, 16, TextureType::cube, 1, 4);
        let mut d = TextureViewDesc {
            texture: Some(&tex),
            dimension: TextureViewDimension::cube,
            layerCount: 6,
            ..TextureViewDesc::default()
        };
        assert!(ctx.makeTextureView(&d).is_some());
        d.layerCount = 0;
        let whole = ctx.makeTextureView(&d).unwrap();
        assert_eq!(whole.textureViewBase().unwrap().layerCount(), 6);
    }
    {
        let tex = texture(&mut ctx, 16, TextureType::texture3D, 8, 4);
        let mut d = TextureViewDesc {
            texture: Some(&tex),
            dimension: TextureViewDimension::texture3D,
            layerCount: 0,
            ..TextureViewDesc::default()
        };
        let whole = ctx.makeTextureView(&d).unwrap();
        assert_eq!(whole.textureViewBase().unwrap().layerCount(), 1);
        d.layerCount = 8;
        assert!(ctx.makeTextureView(&d).is_none());
    }
}
