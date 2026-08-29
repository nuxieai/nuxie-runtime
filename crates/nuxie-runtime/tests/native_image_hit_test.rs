//! Preserved image hit-testing regressions, using the actual imported owners.
//! Authority: pinned Artboard::hitTest and Image::hitTest (including clip TODO).

use nuxie_binary::{
    FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, encode_runtime_file,
};
use nuxie_render_api::{Factory, PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    assets::image_asset::ImageAsset,
    generated::{
        core_registry::CoreRegistry, transform_component_base::TransformComponentBase,
        world_transform_component_base::WorldTransformComponentBase,
    },
    hit_info::HitInfo,
    math::{aabb::IAabb, mat2d::Mat2D},
    shapes::image::Image,
};
use nuxie_runtime::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};

fn property(owner: &str, name: &str, value: FixtureValue) -> FixtureProperty {
    let definition = nuxie_schema::definition_by_name(owner).unwrap();
    let property = std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == name)
        .unwrap();
    FixtureProperty {
        key: property.key.int,
        value,
    }
}

fn record(owner: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
    FixtureRecord {
        type_key: nuxie_schema::definition_by_name(owner)
            .unwrap()
            .type_key
            .int,
        properties,
    }
}

fn fixture(
    image_count: usize,
    clip: bool,
) -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    Vec<CoreHandle>,
) {
    let mut records = vec![record("Backboard", vec![]), record("ImageAsset", vec![])];
    let artboard_properties = if clip {
        vec![
            property("Artboard", "clip", FixtureValue::Bool(true)),
            property("LayoutComponent", "width", FixtureValue::Double(1.0)),
            property("LayoutComponent", "height", FixtureValue::Double(1.0)),
        ]
    } else {
        vec![]
    };
    records.push(record("Artboard", artboard_properties));
    for _ in 0..image_count {
        records.push(record(
            "Image",
            vec![
                property("Image", "parentId", FixtureValue::Uint(0)),
                property("Image", "assetId", FixtureValue::Uint(0)),
            ],
        ));
    }
    // The binary descriptor is only a fixture writer, never an execution graph.
    let descriptor = RuntimeFile::from_fixture_records(records).unwrap();
    let bytes = encode_runtime_file(&descriptor).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(|file| file.artboard_default()).unwrap();
    artboard.update_pass(true);

    // Replace the old facade's register_image_dimensions with the native
    // ImageAsset resource setter, supplying an actual factory-decoded image.
    let mut png_bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, 100, 50);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&vec![0; 100 * 50 * 4]).unwrap();
    }
    let image = factory.decode_image(&png_bytes).unwrap();
    assert_eq!((image.width(), image.height()), (100, 50));
    let asset = file.with_file(|file| file.asset(0)).unwrap();
    ImageAsset::set_render_image_occurrence(&asset, Some(std::rc::Rc::from(image)));
    let images = artboard
        .with_artboard(|artboard| artboard.objects_typed::<Image>().iter().collect::<Vec<_>>());
    assert_eq!(images.len(), image_count);
    (file, artboard, images)
}

fn hit(artboard: &RuntimeArtboardInstanceHandle, x: i32, y: i32) -> Option<CoreHandle> {
    // Preserve the original caller's four-pixel area in caller space.
    let mut info = HitInfo {
        area: IAabb {
            left: x - 2,
            top: y - 2,
            right: x + 2,
            bottom: y + 2,
        },
        mounts: Vec::new(),
    };
    artboard.with_artboard_mut(|artboard| artboard.hit_test(&mut info, &Mat2D::identity()))
}

#[test]
fn artboard_hit_test_reaches_the_pinned_image_hittester_rectangle_path() {
    let (_file, artboard, images) = fixture(1, false);
    assert_eq!(hit(&artboard, 0, 0), Some(images[0].clone()));
    assert!(
        hit(&artboard, 80, 0).is_none(),
        "the Image rectangle must not become an unbounded geometry fallback"
    );
}

#[test]
fn image_hit_test_keeps_the_pinned_caller_space_area_under_artboard_scale() {
    let (_file, artboard, _images) = fixture(1, false);
    assert!(CoreRegistry::set_double_handle(
        &artboard.core_handle(),
        i32::from(TransformComponentBase::SCALE_X_PROPERTY_KEY),
        2.0
    ));
    artboard.update_pass(true);
    assert!(
        hit(&artboard, -103, 0).is_none(),
        "the four-pixel caller-space area ends left of the scaled Image"
    );
}

#[test]
fn image_hit_test_preserves_pinned_opacity_and_unimplemented_clip_behavior() {
    let (_file, artboard, images) = fixture(1, true);
    assert!(CoreRegistry::set_double_handle(
        &images[0],
        i32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY),
        0.0
    ));
    artboard.update_pass(true);
    assert_eq!(
        hit(&artboard, 40, 0),
        Some(images[0].clone()),
        "pinned Image hit testing ignores render opacity and the Artboard clip TODO"
    );
}

#[test]
fn artboard_hit_test_returns_only_the_first_pinned_overlapping_image() {
    let (_file, artboard, images) = fixture(2, false);
    assert_eq!(hit(&artboard, 0, 0), Some(images[0].clone()));
}
