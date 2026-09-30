//! Supplemental production-owner regressions for upstream d9747935.
//! Expectations follow PaintImage::imageSampler and PaintImage::clone.
use nuxie_render_api::{ImageFilter, ImageSampler, ImageWrap};
use nuxie_runtime::source::{
    assets::image_asset::ImageAsset,
    core::{CoreArena, CoreHandle},
    generated::{
        assets::image_asset_base::ImageAssetBase, core_registry::CoreRegistry,
        shapes::paint::paint_image_base::PaintImageBase,
    },
    shapes::paint::paint_image::PaintImage,
};

fn set(owner: &CoreHandle, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(owner, i32::from(key), value));
}

fn sampler(paint: &CoreHandle) -> ImageSampler {
    paint
        .with_downcast::<PaintImage, _>(|paint| paint.image_sampler().into())
        .expect("live PaintImage")
}

fn attach(paint: &CoreHandle, asset: &CoreHandle) {
    paint
        .with_downcast_mut::<PaintImage, _>(|paint| paint.set_asset(Some(asset.clone())))
        .expect("live PaintImage");
}

#[test]
fn image_paint_inherits_and_independently_overrides_sampler_axes() {
    let arena = CoreArena::default();
    let paint = arena.insert(PaintImage::default());
    assert_eq!(sampler(&paint), ImageSampler::LINEAR_CLAMP);
    let asset = arena.insert(ImageAsset::default());
    set(&asset, ImageAssetBase::SAMPLER_FILTER_PROPERTY_KEY, 1);
    set(&asset, ImageAssetBase::SAMPLER_WRAP_X_PROPERTY_KEY, 1);
    set(&asset, ImageAssetBase::SAMPLER_WRAP_Y_PROPERTY_KEY, 2);
    attach(&paint, &asset);
    let inherited = ImageSampler {
        filter: ImageFilter::Nearest,
        wrap_x: ImageWrap::Repeat,
        wrap_y: ImageWrap::Mirror,
    };
    assert_eq!(sampler(&paint), inherited);

    // Paint values are one-based; zero inherits each axis independently.
    set(&paint, PaintImageBase::IMAGE_SAMPLER_FILTER_PROPERTY_KEY, 1);
    assert_eq!(
        sampler(&paint),
        ImageSampler {
            filter: ImageFilter::Bilinear,
            ..inherited
        }
    );
    set(&paint, PaintImageBase::IMAGE_SAMPLER_WRAP_X_PROPERTY_KEY, 3);
    set(&paint, PaintImageBase::IMAGE_SAMPLER_WRAP_Y_PROPERTY_KEY, 1);
    assert_eq!(
        sampler(&paint),
        ImageSampler {
            filter: ImageFilter::Bilinear,
            wrap_x: ImageWrap::Mirror,
            wrap_y: ImageWrap::Clamp,
        }
    );
    set(&paint, PaintImageBase::IMAGE_SAMPLER_FILTER_PROPERTY_KEY, 0);
    set(&paint, PaintImageBase::IMAGE_SAMPLER_WRAP_X_PROPERTY_KEY, 0);
    set(&paint, PaintImageBase::IMAGE_SAMPLER_WRAP_Y_PROPERTY_KEY, 0);
    assert_eq!(sampler(&paint), inherited);
}

#[test]
fn image_paint_clamps_invalid_asset_and_override_sampler_values() {
    let arena = CoreArena::default();
    let paint = arena.insert(PaintImage::default());
    let asset = arena.insert(ImageAsset::default());
    attach(&paint, &asset);
    for (filter, wrap) in [(2, 3), (255, 255)] {
        set(&asset, ImageAssetBase::SAMPLER_FILTER_PROPERTY_KEY, filter);
        set(&asset, ImageAssetBase::SAMPLER_WRAP_X_PROPERTY_KEY, wrap);
        set(&asset, ImageAssetBase::SAMPLER_WRAP_Y_PROPERTY_KEY, wrap);
        assert_eq!(sampler(&paint), ImageSampler::LINEAR_CLAMP);
    }
    set(&asset, ImageAssetBase::SAMPLER_FILTER_PROPERTY_KEY, 1);
    set(&asset, ImageAssetBase::SAMPLER_WRAP_X_PROPERTY_KEY, 2);
    set(&asset, ImageAssetBase::SAMPLER_WRAP_Y_PROPERTY_KEY, 1);
    for (filter, wrap) in [(3, 4), (255, 255)] {
        set(
            &paint,
            PaintImageBase::IMAGE_SAMPLER_FILTER_PROPERTY_KEY,
            filter,
        );
        set(
            &paint,
            PaintImageBase::IMAGE_SAMPLER_WRAP_X_PROPERTY_KEY,
            wrap,
        );
        set(
            &paint,
            PaintImageBase::IMAGE_SAMPLER_WRAP_Y_PROPERTY_KEY,
            wrap,
        );
        assert_eq!(sampler(&paint), ImageSampler::LINEAR_CLAMP);
    }
}

#[test]
fn image_paint_clone_preserves_resolved_asset_identity_and_authored_sampler() {
    let arena = CoreArena::default();
    let asset = arena.insert(ImageAsset::default());
    let paint = arena.insert(PaintImage::default());
    attach(&paint, &asset);
    set(&paint, PaintImageBase::IMAGE_ASSET_ID_PROPERTY_KEY, 17);
    set(&paint, PaintImageBase::IMAGE_SAMPLER_FILTER_PROPERTY_KEY, 2);
    set(&paint, PaintImageBase::IMAGE_SAMPLER_WRAP_X_PROPERTY_KEY, 3);
    let clone = paint.clone_occurrence().expect("PaintImage clone");
    assert!(clone != paint);
    assert!(
        clone
            .with_downcast::<PaintImage, _>(|paint| paint.image_asset())
            .flatten()
            == Some(asset.clone())
    );
    assert_eq!(
        CoreRegistry::get_id_handle(
            &clone,
            i32::from(PaintImageBase::IMAGE_ASSET_ID_PROPERTY_KEY)
        ),
        Some(17)
    );
    assert_eq!(sampler(&clone), sampler(&paint));

    // Removing the original does not clear the clone's resolved asset, and
    // its inherited axis continues to observe the same live asset occurrence.
    drop(arena.remove(&paint).expect("remove original PaintImage"));
    set(&asset, ImageAssetBase::SAMPLER_WRAP_Y_PROPERTY_KEY, 1);
    assert_eq!(
        sampler(&clone),
        ImageSampler {
            filter: ImageFilter::Nearest,
            wrap_x: ImageWrap::Mirror,
            wrap_y: ImageWrap::Repeat,
        }
    );
}
