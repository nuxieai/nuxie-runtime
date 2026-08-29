//! Direct ports of pinned `tests/unit_tests/runtime/solo_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{
    CoreHandle, File, FileImportLimits, PersistentFactory, RuntimeArtboardInstanceHandle,
    RuntimeFactoryHandle, RuntimeFileHandle, RuntimeStateMachineInstanceHandle,
    ScriptExecutionLimits, ScriptedFile, ViewModelInstanceRuntime, import_unsigned_scripted,
};
use nuxie_render_api::{NullFactory, SerializingFactory};
use nuxie_runtime::source::{
    artboard::Artboard,
    core::CoreType,
    generated::{core_registry::CoreRegistry, solo_base::SoloBase},
    math::vec2d::Vec2D,
    nested_artboard::NestedArtboard,
    shapes::{
        paint::{fill::Fill, solid_color::SolidColor},
        shape::Shape,
    },
    solo::Solo,
};
use silver_corpus::{compare_sriv, parse_sriv};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

fn pinned_silver(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let silver = PathBuf::from(root)
        .join("tests/unit_tests/silvers")
        .join(format!("{name}.sriv"));
    std::fs::read(&silver)
        .unwrap_or_else(|error| panic!("read pinned silver {}: {error}", silver.display()))
}

fn compare_silver(name: &str, actual: &[u8]) {
    let actual = parse_sriv(actual).expect("valid Rust SRIV stream");
    let expected = parse_sriv(&pinned_silver(name)).expect("valid pinned SRIV stream");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("{name} differs: {difference}"));
}

struct NativeFixture {
    _factory: PersistentFactory<NullFactory>,
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
}

fn native_fixture(name: &str) -> NativeFixture {
    let mut factory = PersistentFactory::new(NullFactory::new());
    let file = File::import(
        &pinned_fixture(name),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained null factory"),
        None,
        None,
        None,
    )
    .unwrap_or_else(|| panic!("{name} imports"));
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard instance");
    NativeFixture {
        _factory: factory,
        _file: file,
        artboard,
    }
}

fn named<T: CoreType>(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    artboard
        .with_artboard(|artboard| artboard.base.find_handle::<T>(name))
        .unwrap_or_else(|| panic!("component {name}"))
}

fn collapsed(component: &CoreHandle) -> bool {
    component
        .with(|component| {
            component
                .as_component()
                .expect("Solo child is a Component")
                .is_collapsed()
        })
        .expect("live Solo child")
}

fn advance_pointer(
    machine: &RuntimeStateMachineInstanceHandle,
    artboard: &RuntimeArtboardInstanceHandle,
    seconds: f32,
) {
    machine.with_instance_mut(|machine| machine.advance(seconds, true));
    artboard.advance_default(seconds);
}

fn bool_input(machine: &RuntimeStateMachineInstanceHandle, name: &str) -> bool {
    machine.with_instance(|machine| machine.get_bool(name).expect("bool input").value())
}

fn clickable_rectangle_color(nested: &CoreHandle) -> (CoreHandle, u32) {
    let instance = nested
        .with(|nested| {
            nested
                .as_nested_artboard()
                .expect("NestedArtboard")
                .artboard_instance_handle(0)
        })
        .flatten()
        .expect("mounted nested artboard");
    let rectangle = named::<Shape>(&instance, "Clickable-Rectangle");
    let fill = rectangle
        .with(|rectangle| {
            rectangle
                .as_container_component()
                .expect("Shape container")
                .children()
                .get(1)
                .cloned()
        })
        .flatten()
        .expect("Clickable-Rectangle fill");
    assert!(fill.is_type_of(Fill::TYPE_KEY));
    let color = fill
        .with_downcast::<Fill, _>(|fill| fill.base.paint())
        .flatten()
        .expect("Fill paint");
    assert!(color.is_type_of(SolidColor::TYPE_KEY));
    let value = color
        .with_downcast::<SolidColor, _>(|color| color.base.color_value() as u32)
        .expect("live SolidColor");
    (color, value)
}

fn color_value(color: &CoreHandle) -> u32 {
    color
        .with_downcast::<SolidColor, _>(|color| color.base.color_value() as u32)
        .expect("live SolidColor")
}

#[test]
fn file_with_skins_in_solos_loads_correctly() {
    let fixture = native_fixture("death_knight.riv");
    fixture.artboard.advance_default(0.0);
    assert_eq!(
        fixture
            .artboard
            .with_artboard(|artboard| artboard.base.count::<Solo>()),
        2
    );
}

#[test]
fn children_load_correctly() {
    let fixture = native_fixture("solo_test.riv");
    let artboard = &fixture.artboard;
    artboard.advance_default(0.0);
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.base.count::<Solo>()),
        1
    );
    let solo = artboard
        .with_artboard(|artboard| artboard.base.object_handle_at::<Solo>(0))
        .expect("Solo");
    let children = solo
        .with(|solo| {
            solo.as_container_component()
                .expect("Solo container")
                .children()
                .to_vec()
        })
        .expect("live Solo");
    assert_eq!(children.len(), 3);
    assert_eq!(
        children
            .iter()
            .map(|child| {
                assert!(child.is_type_of(Shape::TYPE_KEY));
                child
                    .with(|child| child.as_component().unwrap().name().to_owned())
                    .unwrap()
            })
            .collect::<Vec<_>>(),
        ["Blue", "Green", "Red"]
    );
    let blue = named::<Shape>(artboard, "Blue");
    let green = named::<Shape>(artboard, "Green");
    let red = named::<Shape>(artboard, "Red");
    assert!(!collapsed(&blue));
    assert!(collapsed(&green));
    assert!(collapsed(&red));
    for parent in [&green, &red] {
        let children = parent
            .with(|parent| {
                parent
                    .as_container_component()
                    .expect("Shape container")
                    .children()
                    .to_vec()
            })
            .expect("live Shape");
        assert_eq!(children.len(), 2);
        assert!(children.iter().all(collapsed));
    }

    let machine = artboard
        .default_state_machine()
        .expect("default state machine");
    machine.advance_and_apply(0.0);
    assert!(collapsed(&blue));
    assert!(collapsed(&green));
    assert!(!collapsed(&red));
    machine.advance_and_apply(0.5);
    assert!(collapsed(&blue));
    assert!(!collapsed(&green));
    assert!(collapsed(&red));
    machine.advance_and_apply(0.5);
    assert!(!collapsed(&blue));
    assert!(collapsed(&green));
    assert!(collapsed(&red));
}

#[test]
fn nested_solos_work() {
    let fixture = native_fixture("nested_solo.riv");
    let artboard = &fixture.artboard;
    artboard.advance_default(0.0);
    let s1 = named::<Solo>(artboard, "Solo 1");
    let s2 = named::<Solo>(artboard, "Solo 2");
    let s3 = named::<Solo>(artboard, "Solo 3");
    let a = named::<Shape>(artboard, "A");
    let b = named::<Shape>(artboard, "B");
    let c = named::<Shape>(artboard, "C");
    let d = named::<Shape>(artboard, "D");
    let e = named::<Shape>(artboard, "E");
    let f = named::<Shape>(artboard, "F");
    let g = named::<Shape>(artboard, "G");
    let h = named::<Shape>(artboard, "H");
    let i = named::<Shape>(artboard, "I");
    for (solo, active) in [(&s1, &a), (&s2, &d), (&s3, &h)] {
        let id = artboard.with_artboard(|artboard| artboard.base.id_of(active));
        assert!(CoreRegistry::set_uint_handle(
            solo,
            SoloBase::ACTIVE_COMPONENT_ID_PROPERTY_KEY.into(),
            id,
        ));
    }
    artboard.advance_default(0.0);
    for (name, expected) in [
        (&a, false),
        (&b, true),
        (&c, true),
        (&d, true),
        (&e, true),
        (&f, true),
        (&g, true),
        (&h, true),
        (&i, true),
    ] {
        assert_eq!(collapsed(name), expected);
    }

    let g_id = artboard.with_artboard(|artboard| artboard.base.id_of(&g));
    assert!(CoreRegistry::set_uint_handle(
        &s3,
        SoloBase::ACTIVE_COMPONENT_ID_PROPERTY_KEY.into(),
        g_id,
    ));
    artboard.advance_default(0.0);
    for (name, expected) in [
        (&a, false),
        (&b, true),
        (&c, true),
        (&d, true),
        (&e, true),
        (&f, true),
        (&g, true),
        (&h, true),
        (&i, true),
    ] {
        assert_eq!(collapsed(name), expected);
    }

    let c_id = artboard.with_artboard(|artboard| artboard.base.id_of(&c));
    assert!(CoreRegistry::set_uint_handle(
        &s1,
        SoloBase::ACTIVE_COMPONENT_ID_PROPERTY_KEY.into(),
        c_id,
    ));
    artboard.advance_default(0.0);
    for (name, expected) in [
        (&a, true),
        (&b, true),
        (&c, false),
        (&d, false),
        (&e, true),
        (&f, true),
        (&g, false),
        (&h, true),
        (&i, true),
    ] {
        assert_eq!(collapsed(name), expected);
    }
}

#[test]
fn hit_test_on_solos() {
    let fixture = native_fixture("hit_test_solos.riv");
    let artboard = &fixture.artboard;
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.base.state_machine_count()),
        1
    );
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    advance_pointer(&machine, artboard, 0.0);
    assert!(machine.with_instance(|machine| machine.get_bool("hovered").is_some()));

    for (x, y, expected) in [
        (200.0, 100.0, true),
        (200.0, 300.0, false),
        (200.0, 400.0, false),
    ] {
        machine.with_instance_mut(|machine| machine.pointer_move(Vec2D::new(x, y), 0.0, 0));
        assert_eq!(bool_input(&machine, "hovered"), expected);
    }
    advance_pointer(&machine, artboard, 1.5);
    for (x, y, expected) in [
        (200.0, 100.0, false),
        (200.0, 300.0, true),
        (200.0, 400.0, false),
    ] {
        machine.with_instance_mut(|machine| machine.pointer_move(Vec2D::new(x, y), 0.0, 0));
        assert_eq!(bool_input(&machine, "hovered"), expected);
    }
    advance_pointer(&machine, artboard, 1.0);
    for (x, y, expected) in [
        (200.0, 100.0, false),
        (200.0, 300.0, false),
        (200.0, 400.0, true),
    ] {
        machine.with_instance_mut(|machine| machine.pointer_move(Vec2D::new(x, y), 0.0, 0));
        assert_eq!(bool_input(&machine, "hovered"), expected);
    }
}

#[test]
fn hit_test_on_nested_artboards_in_solos() {
    let fixture = native_fixture("pointer_events_nested_artboards_in_solos.riv");
    let main_artboard = &fixture.artboard;
    let parent_handle = named::<Artboard>(main_artboard, "Parent-Artboard");
    let parent = parent_handle
        .with_downcast::<Artboard, _>(|artboard| artboard.runtime_weak_handle().upgrade())
        .flatten()
        .expect("Parent-Artboard runtime occurrence");
    Artboard::update_components_handle(&parent.core_handle());
    let active = named::<NestedArtboard>(&parent, "Nested-Artboard-Active");
    let inactive = named::<NestedArtboard>(&parent, "Nested-Artboard-Inactive");
    let (active_color, _) = clickable_rectangle_color(&active);
    let (inactive_color, _) = clickable_rectangle_color(&inactive);

    assert_eq!(
        main_artboard.with_artboard(|artboard| artboard.base.state_machine_count()),
        1
    );
    let machine = main_artboard.state_machine_at(0).expect("state machine 0");
    advance_pointer(&machine, main_artboard, 0.0);
    assert!(!collapsed(&active));
    assert!(collapsed(&inactive));
    assert_eq!(color_value(&active_color), 0xFF00_B511);
    assert_eq!(color_value(&inactive_color), 0xFF74_7474);

    advance_pointer(&machine, main_artboard, 0.1);
    assert!(collapsed(&active));
    assert!(!collapsed(&inactive));
    assert_eq!(color_value(&inactive_color), 0xFF00_B511);

    advance_pointer(&machine, main_artboard, 0.1);
    assert!(!collapsed(&active));
    assert!(collapsed(&inactive));
    machine.with_instance_mut(|machine| machine.pointer_up(Vec2D::new(200.0, 200.0), 0));
    machine.with_instance_mut(|machine| machine.advance(0.0, true));
    parent.advance_default(0.0);
    machine.with_instance_mut(|machine| machine.advance(0.1, true));
    parent.advance_default(0.1);
    assert!(collapsed(&active));
    assert!(!collapsed(&inactive));
    assert_eq!(color_value(&active_color), 0xFFC8_0000);
    assert_eq!(color_value(&inactive_color), 0xFF00_B511);
}

// The exact synthetic construction and every assertion for
// "solo index/name selection skips property-like children" live beside the
// private Solo owner in `nuxie-runtime/src/artboard/tests.rs`.

#[test]
fn data_bound_solos_with_enums_work_in_both_directions() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let file = File::import(
        &pinned_fixture("databind_solo_to_enum.riv"),
        RuntimeFactoryHandle::from_factory(&mut silver).expect("retained silver factory"),
        None,
        None,
        None,
    )
    .expect("databind_solo_to_enum imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let (width, height) =
        artboard.with_artboard(|artboard| (artboard.base.width(), artboard.base.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model_id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
    let view_model = file
        .with_file_mut(|file| {
            if view_model_id == u32::MAX {
                file.create_view_model_instance_for_artboard(artboard.core_handle())
            } else {
                file.create_view_model_instance_at(view_model_id as usize, 0)
            }
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("view-model instance");
    let enum_property = view_model
        .property_enum("enuToSource")
        .expect("enuToSource enum");
    assert_eq!(enum_property.value_index(), 3);
    machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(view_model.instance());
    });
    machine.advance_and_apply(0.0);
    let mut renderer = silver.borrow().make_renderer();
    artboard.draw(&mut renderer);
    silver.borrow_mut().add_frame();
    machine.with_instance_mut(|machine| {
        machine.pointer_down(Vec2D::new(425.0, 70.0), 0);
        machine.pointer_up(Vec2D::new(425.0, 70.0), 0);
    });
    machine.advance_and_apply(0.016);
    artboard.draw(&mut renderer);
    assert_eq!(enum_property.value_index(), 5);
    compare_silver("databind_solo_to_enum", &silver.borrow().bytes());
}

#[test]
fn do_not_advance_collapsed_scripts() {
    let mut factory = PersistentFactory::new(NullFactory::new());
    let file: ScriptedFile = import_unsigned_scripted(
        &pinned_fixture("script_advance_test.riv"),
        &mut factory,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("script_advance_test imports with trusted scripts");
    let artboard = file
        .native_file()
        .with_file(File::artboard_default)
        .expect("default artboard");
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = file
        .native_file()
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("default view-model instance");
    machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(view_model.instance());
    });
    let solo_index = view_model
        .property_number("soloIndex")
        .expect("soloIndex number");
    let advance_count = view_model
        .property_number("advanceCount")
        .expect("advanceCount number");
    assert_eq!(solo_index.value(), 0.0);
    assert_eq!(advance_count.value(), 0.0);
    for expected in [1.0, 2.0] {
        machine.advance_and_apply(0.016);
        assert_eq!(advance_count.value(), expected);
    }
    for (index, expected) in [(1.0, 3.0), (2.0, 4.0), (3.0, 5.0)] {
        solo_index.set_value(index);
        assert_eq!(solo_index.value(), index);
        machine.advance_and_apply(0.016);
        assert_eq!(advance_count.value(), expected);
    }
    machine.advance_and_apply(0.016);
    assert_eq!(advance_count.value(), 5.0);
    solo_index.set_value(0.0);
    assert_eq!(solo_index.value(), 0.0);
    machine.advance_and_apply(0.016);
    assert_eq!(advance_count.value(), 5.0);
    machine.advance_and_apply(0.016);
    assert_eq!(advance_count.value(), 6.0);
}

#[test]
fn data_bind_by_index_skipping_non_hierarchical_children() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let file = File::import(
        &pinned_fixture("solo_index_test.riv"),
        RuntimeFactoryHandle::from_factory(&mut silver).expect("retained silver factory"),
        None,
        None,
        None,
    )
    .expect("solo_index_test imports");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let (width, height) =
        artboard.with_artboard(|artboard| (artboard.base.width(), artboard.base.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut renderer = silver.borrow().make_renderer();
    let machine = artboard.state_machine_at(0).expect("state machine 0");
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("default view-model instance");
    machine.with_instance_mut(|machine| {
        machine.bind_view_model_instance(view_model.instance());
    });
    let index_property = view_model.property_number("index").expect("index number");
    for index in [0.0, 1.0, 2.0, 3.0] {
        if index != 0.0 {
            index_property.set_value(index);
            assert_eq!(index_property.value(), index);
        }
        machine.advance_and_apply(0.1);
        artboard.draw(&mut renderer);
        if index != 3.0 {
            silver.borrow_mut().add_frame();
        }
    }
    compare_silver("solo_index_test", &silver.borrow().bytes());
}
