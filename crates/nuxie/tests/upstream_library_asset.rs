//! Direct ports of the ten active pinned `runtime/library_asset_test.cpp` cases.

use std::path::PathBuf;

use nuxie::runtime::{
    animation::{
        linear_animation::LinearAnimation, nested_simple_animation::NestedSimpleAnimation,
        nested_state_machine::NestedStateMachine, state_machine::StateMachine,
    },
    assets::{image_asset::ImageAsset, script_asset::ScriptAsset},
    custom_property_string::CustomPropertyString,
    nested_artboard::NestedArtboard,
    shapes::{image::Image, paint::solid_color::SolidColor, rectangle::Rectangle},
};
use nuxie::{
    Artboard, File, ImportResult, PersistentFactory, RecordingFactory,
    RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};

fn bytes(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

struct Fixture {
    file: RuntimeFileHandle,
    _factory: PersistentFactory<RecordingFactory>,
}

fn load(name: &str) -> Fixture {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes(name), retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("import {name}: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    Fixture {
        file,
        _factory: factory,
    }
}

fn source(file: &RuntimeFileHandle, index: usize) -> nuxie::CoreHandle {
    file.with_file(|file| file.artboard_handle(index))
        .unwrap_or_else(|| panic!("missing artboard {index}"))
}

fn instance(file: &RuntimeFileHandle, index: usize) -> RuntimeArtboardInstanceHandle {
    let result = Artboard::instance_from_handle(&source(file, index)).expect("artboard instance");
    result.with_artboard_mut(|artboard| artboard.set_file(Some(file.downgrade())));
    result
}

fn nested_named(artboard: &nuxie::CoreHandle, name: &str) -> nuxie::CoreHandle {
    artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<NestedArtboard>(name))
        .flatten()
        .unwrap_or_else(|| panic!("missing nested {name:?}"))
}

fn assert_nested_header(nested: &nuxie::CoreHandle) {
    nested.with_downcast::<NestedArtboard, _>(|nested| {
        assert_eq!(nested.base.base.name(), "The nested artboard");
        assert_eq!((nested.base.x(), nested.base.y()), (1.0, 2.0));
        assert_eq!(nested.base.artboard_id(), 1);
    });
}

fn bind_default(file: &RuntimeFileHandle, artboard: &RuntimeArtboardInstanceHandle) {
    let view_model = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .expect("default view model");
    artboard.bind_view_model_instance(Some(view_model));
}

#[test]
fn file_with_library_artboard_loads() {
    let f = load("library_export_test.riv");
    assert_nested_header(&nested_named(&source(&f.file, 0), "The nested artboard"));
    source(&f.file, 1).with_downcast::<Artboard, _>(|a| {
        assert_eq!(a.base.name(), "Rocket");
        assert_eq!((a.width(), a.height()), (512.0, 513.0));
    });
    assert_eq!(f.file.with_file(|file| file.assets().len()), 0);
}

#[test]
fn file_with_library_animation_loads() {
    let f = load("library_export_animation_test.riv");
    let nested = nested_named(&source(&f.file, 0), "The nested artboard");
    assert_nested_header(&nested);
    source(&f.file, 1).with_downcast::<Artboard, _>(|a| {
        assert_eq!(a.animation_count(), 1);
        a.first_animation()
            .expect("animation")
            .with_downcast::<LinearAnimation, _>(|v| assert_eq!(v.base.name(), "LA Rocket"));
    });
    nested.with_downcast::<NestedArtboard, _>(|n| {
        assert_eq!(n.nested_animations().len(), 1);
        n.nested_animations()[0].with_downcast::<NestedSimpleAnimation, _>(|v| {
            assert_eq!(v.base.base.name(), "");
            assert_eq!(v.base.animation_id(), 0);
        });
    });
    assert_eq!(f.file.with_file(|file| file.assets().len()), 0);
}

#[test]
fn file_with_library_state_machine_loads() {
    let f = load("library_export_state_machine_test.riv");
    let nested = nested_named(&source(&f.file, 0), "The nested artboard");
    assert_nested_header(&nested);
    source(&f.file, 1).with_downcast::<Artboard, _>(|a| {
        assert_eq!(a.state_machine_count(), 1);
        a.first_state_machine()
            .expect("machine")
            .with_downcast::<StateMachine, _>(|v| assert_eq!(v.base.name(), "SM Rocket"));
    });
    nested.with_downcast::<NestedArtboard, _>(|n| {
        assert_eq!(n.nested_animations().len(), 1);
        n.nested_animations()[0].with_downcast::<NestedStateMachine, _>(|v| {
            assert_eq!(v.base.base.name(), "");
            assert_eq!(v.base.animation_id(), 0);
        });
    });
    assert_eq!(f.file.with_file(|file| file.assets().len()), 0);
}

#[test]
fn library_script_exports_flat_under_its_mangle_prefix() {
    let f = load("library_scope_edge_test.riv");
    let assets = f.file.with_file(|file| file.assets().to_vec());
    let script = assets
        .iter()
        .find(|a| a.with_downcast::<ScriptAsset, _>(|_| ()).is_some())
        .expect("script");
    assert_eq!(
        script
            .with_downcast::<ScriptAsset, _>(ScriptAsset::module_name)
            .as_deref(),
        Some("FruitsLib@4/FruitModule")
    );
}

#[test]
fn nested_library_scripts_export_flat_under_distinct_prefixes() {
    let f = load("nested_library_scope_test.riv");
    let assets = f.file.with_file(|file| file.assets().to_vec());
    for (name, module) in [("useb", "OuterLib@6/useb"), ("mesh", "InnerLib@4/mesh")] {
        let script = assets
            .iter()
            .find(|a| a.with_downcast::<ScriptAsset, _>(|s| s.base.name() == name) == Some(true))
            .unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(
            script
                .with_downcast::<ScriptAsset, _>(ScriptAsset::module_name)
                .as_deref(),
            Some(module)
        );
    }
}

#[test]
fn file_with_library_including_image() {
    let f = load("library_with_image.riv");
    assert_eq!(f.file.with_file(|file| file.assets().len()), 1);
    let nested = nested_named(&source(&f.file, 0), "The instance");
    let library = nested
        .with_downcast::<NestedArtboard, _>(NestedArtboard::source_artboard)
        .flatten()
        .expect("source artboard");
    let images = library
        .with_downcast::<Artboard, _>(Artboard::find_all_handles::<Image>)
        .expect("artboard");
    assert_eq!(images.len(), 1);
    images[0].with_downcast::<Image, _>(|image| {
        assert_eq!(image.asset_id(), 0);
        assert!(image.image_asset().is_some());
    });
}

#[test]
fn file_with_multiple_libraries_including_image() {
    let f = load("double_library_with_image.riv");
    assert_eq!(f.file.with_file(|file| file.assets().len()), 2);
    let host = source(&f.file, 0);
    for (nested_name, asset_name) in [
        ("The nested artboard", "MyFirstImageAsset"),
        ("Another nested artboard", "MyOtherImageAsset"),
    ] {
        let nested = nested_named(&host, nested_name);
        let library = nested
            .with_downcast::<NestedArtboard, _>(NestedArtboard::source_artboard)
            .flatten()
            .expect("source artboard");
        let images = library
            .with_downcast::<Artboard, _>(Artboard::find_all_handles::<Image>)
            .expect("artboard");
        assert_eq!(images.len(), 1);
        let asset = images[0]
            .with_downcast::<Image, _>(Image::image_asset)
            .flatten()
            .expect("image asset");
        assert_eq!(
            asset
                .with_downcast::<ImageAsset, _>(|a| a.base.base.name().to_owned())
                .as_deref(),
            Some(asset_name)
        );
    }
}

#[test]
fn file_with_data_enum() {
    let f = load("library_data_enum_test.riv");
    let artboard = instance(&f.file, 0);
    bind_default(&f.file, &artboard);
    assert!(artboard.with_artboard(|a| {
        a.find_handle::<nuxie::runtime::event::Event>("my_event")
            .is_some()
    }));
    artboard.advance_default(0.0);
    let property = artboard
        .with_artboard(|a| a.find_handle::<CustomPropertyString>("my_event_property"))
        .expect("property");
    assert_eq!(
        property
            .with_downcast::<CustomPropertyString, _>(|p| p.base.property_value().to_owned())
            .as_deref(),
        Some("red3")
    );
}

#[test]
fn file_with_view_model() {
    let f = load("library_view_model_test.riv");
    let root = instance(&f.file, 0);
    bind_default(&f.file, &root);
    let middle_host = root
        .with_artboard(|a| a.find_handle::<NestedArtboard>(""))
        .expect("nested 2");
    let middle = middle_host
        .with_downcast::<NestedArtboard, _>(NestedArtboard::artboard_instance_default)
        .flatten()
        .expect("instance 2");
    assert_eq!(middle.with_artboard(|a| a.base.name().to_owned()), "2");
    let leaf_host = middle
        .with_artboard(|a| a.find_handle::<NestedArtboard>(""))
        .expect("nested 1");
    let leaf = leaf_host
        .with_downcast::<NestedArtboard, _>(NestedArtboard::artboard_instance_default)
        .flatten()
        .expect("instance 1");
    assert_eq!(leaf.with_artboard(|a| a.base.name().to_owned()), "1");
    root.advance_default(0.0);
    for (name, value) in [("for_string", "hello"), ("for_enum", "uk")] {
        let p = leaf
            .with_artboard(|a| a.find_handle::<CustomPropertyString>(name))
            .unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(
            p.with_downcast::<CustomPropertyString, _>(|p| p.base.property_value().to_owned())
                .as_deref(),
            Some(value)
        );
    }
    let rectangle = leaf
        .with_artboard(|a| a.find_handle::<Rectangle>(""))
        .expect("rectangle");
    rectangle.with_downcast::<Rectangle, _>(|r| {
        assert_eq!((r.base.width(), r.base.height()), (123.0, 123.0))
    });
    let color = leaf
        .with_artboard(|a| a.find_handle::<SolidColor>(""))
        .expect("solid color");
    assert_eq!(
        color.with_downcast::<SolidColor, _>(|c| c.base.color_value()),
        Some(0xff0a0f42u32 as i32)
    );
}

#[test]
fn library_vmtest_1_host() {
    let f = load("library_vmtest_1_host.riv");
    let root = instance(&f.file, 0);
    bind_default(&f.file, &root);
    let nested = root
        .with_artboard(|a| a.find_handle::<NestedArtboard>(""))
        .expect("nested");
    let child = nested
        .with_downcast::<NestedArtboard, _>(NestedArtboard::artboard_instance_default)
        .flatten()
        .expect("lib2 instance");
    assert_eq!(
        child.with_artboard(|a| a.base.name().to_owned()),
        "lib2artboard"
    );
    root.advance_default(0.0);
    assert_eq!(child.with_artboard(|a| a.count::<SolidColor>()), 1);
    let color = child
        .with_artboard(|a| a.find_handle::<SolidColor>(""))
        .expect("solid color");
    assert_eq!(
        color.with_downcast::<SolidColor, _>(|c| c.base.color_value()),
        Some(0xff101566u32 as i32)
    );
}
