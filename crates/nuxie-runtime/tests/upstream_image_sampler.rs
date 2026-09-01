//! Direct port of upstream `tests/unit_tests/runtime/image_sampler_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    generated::{
        assets::image_asset_base::ImageAssetBase, core_registry::CoreRegistry,
        shapes::image_base::ImageBase,
    },
    shapes::{
        image::Image,
        paint::image_sampler::{ImageFilter, ImageSampler, ImageWrap},
    },
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeFactoryHandle, RuntimeFileHandle,
};

fn tape_fixture() -> (RuntimeFileHandle, CoreHandle, CoreHandle) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests/assets/tape.riv");
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));

    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes, retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("tape.riv imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);

    let artboard = file.with_file(File::artboard).expect("default artboard");
    let image = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<Image>("Tape body.png"))
        .flatten()
        .expect("Tape body.png Image");
    let asset = image
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .expect("ImageAsset");
    assert!(asset.is_type_of(ImageAssetBase::TYPE_KEY));
    (file, image, asset)
}

fn image_sampler(image: &CoreHandle) -> ImageSampler {
    image
        .with_downcast::<Image, _>(Image::image_sampler)
        .expect("Image")
}

#[test]
fn image_sampler_resolves_asset_defaults_and_node_overrides() {
    let (_file, image, asset) = tape_fixture();

    assert!(image_sampler(&image) == ImageSampler::linear_clamp());

    assert!(CoreRegistry::set_uint_handle(
        &asset,
        ImageAssetBase::SAMPLER_FILTER_PROPERTY_KEY.into(),
        1,
    ));
    assert!(CoreRegistry::set_uint_handle(
        &asset,
        ImageAssetBase::SAMPLER_WRAP_X_PROPERTY_KEY.into(),
        1,
    ));
    assert!(CoreRegistry::set_uint_handle(
        &asset,
        ImageAssetBase::SAMPLER_WRAP_Y_PROPERTY_KEY.into(),
        2,
    ));
    let from_asset = image_sampler(&image);
    assert!(from_asset.filter == ImageFilter::Nearest);
    assert!(from_asset.wrap_x == ImageWrap::Repeat);
    assert!(from_asset.wrap_y == ImageWrap::Mirror);

    // Node values are offset by one, zero inherits from the asset.
    assert!(CoreRegistry::set_uint_handle(
        &image,
        ImageBase::SAMPLER_FILTER_PROPERTY_KEY.into(),
        1,
    ));
    assert!(CoreRegistry::set_uint_handle(
        &image,
        ImageBase::SAMPLER_WRAP_X_PROPERTY_KEY.into(),
        1,
    ));
    let overridden = image_sampler(&image);
    assert!(overridden.filter == ImageFilter::Bilinear);
    assert!(overridden.wrap_x == ImageWrap::Clamp);
    assert!(overridden.wrap_y == ImageWrap::Mirror);

    // Malformed file values fall back to safe defaults.
    assert!(CoreRegistry::set_uint_handle(
        &image,
        ImageBase::SAMPLER_FILTER_PROPERTY_KEY.into(),
        200,
    ));
    assert!(CoreRegistry::set_uint_handle(
        &image,
        ImageBase::SAMPLER_WRAP_X_PROPERTY_KEY.into(),
        200,
    ));
    let clamped = image_sampler(&image);
    assert!(clamped.filter == ImageFilter::Bilinear);
    assert!(clamped.wrap_x == ImageWrap::Clamp);
}
