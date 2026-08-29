//! Direct ports of all six cases in pinned
//! `tests/unit_tests/runtime/artboard_transform_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{Factory, PersistentFactory, RecordingFactory, SerializingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::StateMachineInstance,
    generated::{
        core_registry::CoreRegistry, layout_component_base::LayoutComponentBase,
        transform_component_base::TransformComponentBase,
    },
    math::{mat2d::Mat2D, vec2d::Vec2D},
    nested_artboard::NestedArtboard,
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_number::ViewModelInstanceNumber,
    },
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};

use nuxie_sriv as sriv;

fn pinned_path(relative: &str) -> PathBuf {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    PathBuf::from(root).join("tests/unit_tests").join(relative)
}

fn import<F: Factory + 'static>(
    asset: &str,
    factory: &mut PersistentFactory<F>,
) -> RuntimeFileHandle {
    let path = pinned_path(&format!("assets/{asset}"));
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let retained = RuntimeFactoryHandle::from_factory(factory).expect("explicit retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes, retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("{asset} imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    file
}

struct Fixture {
    // Keep the defining File alive while its instance and shared definitions are used.
    file: RuntimeFileHandle,
    factory: PersistentFactory<RecordingFactory>,
    artboard: RuntimeArtboardInstanceHandle,
}

fn fixture(asset: &str, artboard_name: Option<&str>) -> Fixture {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = import(asset, &mut factory);
    let source = file
        .with_file(|file| match artboard_name {
            Some(name) => file.artboard_named_source(name),
            None => file.artboard(),
        })
        .expect("source artboard");
    let artboard = Artboard::instance_from_handle(&source).expect("artboard instance");
    Fixture {
        file,
        factory,
        artboard,
    }
}

fn set_double(owner: &CoreHandle, property: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(
        owner,
        i32::from(property),
        value
    ));
}

fn draw_recording(fixture: &Fixture) -> String {
    // Each upstream renderer starts with an empty transform log. The factory
    // itself is the same retained factory that allocated resources at import.
    fixture.factory.borrow_mut().clear();
    let mut renderer = fixture.factory.borrow().make_renderer();
    fixture.artboard.draw(&mut renderer);
    fixture.factory.borrow().stream()
}

fn parse_matrix(line: &str) -> Option<Mat2D> {
    let values = line
        .strip_prefix("transform matrix=[")?
        .strip_suffix(']')?
        .split(',')
        .map(str::parse::<f32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let [xx, yx, xy, yy, tx, ty]: [f32; 6] = values.try_into().ok()?;
    Some(Mat2D::new(xx, yx, xy, yy, tx, ty))
}

fn contains_matrix(stream: &str, expected: Mat2D) -> bool {
    stream.lines().filter_map(parse_matrix).any(|actual| {
        actual
            .values()
            .iter()
            .zip(expected.values())
            .all(|(actual, expected)| (actual - expected).abs() <= 0.0001)
    })
}

fn clip_transform(stream: &str) -> Option<Mat2D> {
    let mut stack = vec![Mat2D::identity()];
    for line in stream.lines() {
        if line == "save" {
            stack.push(*stack.last()?);
        } else if line == "restore" {
            if stack.len() > 1 {
                stack.pop();
            }
        } else if let Some(transform) = parse_matrix(line) {
            let current = stack.last_mut()?;
            *current *= transform;
        } else if line.starts_with("clipPath path=") {
            return stack.last().copied();
        }
    }
    None
}

fn nested_instance(artboard: &RuntimeArtboardInstanceHandle) -> RuntimeArtboardInstanceHandle {
    let parent = artboard
        .with_artboard(|artboard| artboard.find_handle::<Artboard>("Parent Artboard"))
        .expect("Parent Artboard");
    Artboard::update_components_handle(&parent);
    let container = parent
        .with_downcast::<Artboard, _>(|artboard| {
            artboard.find_handle::<NestedArtboard>("Nested artboard container")
        })
        .flatten()
        .expect("nested container");
    container
        .with_downcast::<NestedArtboard, _>(|container| container.artboard_instance_handle(0))
        .flatten()
        .expect("mounted nested instance")
}

#[test]
fn artboard_bakes_its_own_rotation_and_scale_into_draw() {
    let fixture = fixture("nested_artboard_opacity.riv", None);
    fixture.artboard.advance_default(0.0);
    let root = fixture.artboard.core_handle();
    set_double(&root, TransformComponentBase::SCALE_X_PROPERTY_KEY, 2.0);
    set_double(&root, TransformComponentBase::SCALE_Y_PROPERTY_KEY, 3.0);
    fixture.artboard.advance_default(0.0);
    assert!(contains_matrix(
        &draw_recording(&fixture),
        Mat2D::new(2.0, 0.0, 0.0, 3.0, 0.0, 0.0)
    ));

    set_double(
        &root,
        TransformComponentBase::ROTATION_PROPERTY_KEY,
        1.570_796_3,
    );
    fixture.artboard.advance_default(0.0);
    let mut expected = Mat2D::from_rotation(1.570_796_3);
    expected.scale_by_values(2.0, 3.0);
    assert!(contains_matrix(&draw_recording(&fixture), expected));
}

#[test]
fn artboard_transform_is_only_pushed_when_non_default() {
    let fixture = fixture("nested_artboard_opacity.riv", None);
    fixture.artboard.advance_default(0.0);
    let plain_count = draw_recording(&fixture)
        .lines()
        .filter(|line| line.starts_with("transform matrix="))
        .count();

    // Both instances are cloned from the same source, as in the pinned test.
    let source = fixture.file.with_file(File::artboard).expect("source");
    let scaled = Artboard::instance_from_handle(&source).expect("scaled instance");
    set_double(
        &scaled.core_handle(),
        TransformComponentBase::SCALE_X_PROPERTY_KEY,
        2.0,
    );
    scaled.advance_default(0.0);
    fixture.factory.borrow_mut().clear();
    let mut renderer = fixture.factory.borrow().make_renderer();
    scaled.draw(&mut renderer);
    let scaled_count = fixture
        .factory
        .borrow()
        .stream()
        .lines()
        .filter(|line| line.starts_with("transform matrix="))
        .count();
    assert_eq!(scaled_count, plain_count + 1);
}

#[test]
fn artboard_rotation_is_honored_in_state_machine_hit_testing() {
    let fixture = fixture("opaque_hit_test.riv", Some("main"));
    set_double(
        &fixture.artboard.core_handle(),
        TransformComponentBase::ROTATION_PROPERTY_KEY,
        3.141_592_7,
    );
    fixture.artboard.advance_default(0.0);
    let source = fixture
        .file
        .with_file(|file| file.artboard_named_source("main"))
        .expect("main source");
    let definition = source
        .with_downcast::<Artboard, _>(|artboard| artboard.state_machine_named("main-state-machine"))
        .flatten()
        .expect("main-state-machine");
    let machine = StateMachineInstance::new(definition, fixture.artboard.downgrade());
    machine.with_instance_mut(|machine| machine.advance(0.0, true));
    fixture.artboard.advance_default(0.0);
    machine.with_instance_mut(|machine| machine.advance(0.0, true));
    assert!(machine.with_instance(|machine| machine.get_bool("toGreen").is_some()));

    let green_content = Vec2D::new(100.0, 250.0);
    let green_world = fixture.artboard.with_artboard(|artboard| {
        let frame = Vec2D::new(
            artboard.origin_x() * artboard.layout_width(),
            artboard.origin_y() * artboard.layout_height(),
        );
        frame + artboard.self_transform() * (green_content - frame)
    });
    assert!((green_world - green_content).length() > 1.0);
    machine.with_instance_mut(|machine| machine.pointer_down(green_content, 0));
    assert!(
        !machine.with_instance(|machine| machine.get_bool("toGreen").expect("toGreen").value())
    );
    machine.with_instance_mut(|machine| machine.pointer_down(green_world, 0));
    assert!(machine.with_instance(|machine| machine.get_bool("toGreen").expect("toGreen").value()));
}

#[test]
fn nested_artboards_own_rotation_affects_root_transform() {
    let fixture = fixture("nested_artboard_opacity.riv", None);
    let nested = nested_instance(&fixture.artboard);
    let point = Vec2D::new(10.0, 0.0);
    let before = nested.with_artboard_mut(|artboard| artboard.root_transform(point));
    set_double(
        &nested.core_handle(),
        TransformComponentBase::ROTATION_PROPERTY_KEY,
        1.570_796_4,
    );
    Artboard::update_components_handle(&nested.core_handle());
    let rotated = nested.with_artboard_mut(|artboard| artboard.root_transform(point));
    assert!((rotated - before).length() > 1.0);
}

#[test]
fn artboard_transform_and_opacity() {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let file = import("artboard_opacity_and_transform_test.riv", &mut silver);
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default instance");
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine 0");
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .expect("default view model instance");
    let property = |name| {
        view_model
            .with_downcast::<ViewModelInstance, _>(|view_model| {
                view_model.property_value_named(name)
            })
            .flatten()
            .unwrap_or_else(|| panic!("view model property {name}"))
    };
    let x_pos = property("xPos");
    let y_pos = property("yPos");
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model));
    machine.advance_and_apply(0.1);
    let mut renderer = silver.borrow().make_renderer();
    artboard.draw(&mut renderer);
    for _ in 0..11 {
        silver.borrow_mut().add_frame();
        machine.advance_and_apply(0.1);
        let position = Vec2D::new(
            x_pos
                .with_downcast::<ViewModelInstanceNumber, _>(ViewModelInstanceNumber::value)
                .expect("xPos number"),
            y_pos
                .with_downcast::<ViewModelInstanceNumber, _>(ViewModelInstanceNumber::value)
                .expect("yPos number"),
        );
        machine.with_instance_mut(|machine| machine.pointer_down(position, 0));
        machine.with_instance_mut(|machine| machine.pointer_up(position, 0));
        artboard.draw(&mut renderer);
    }
    let path = pinned_path("silvers/artboard_opacity_and_transform_test.sriv");
    let expected = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned silver {}: {error}", path.display()));
    let actual = silver.borrow().bytes().to_vec();
    // Pinned SerializingFactory::matches rejects unequal stream sizes before
    // its typed, epsilon-aware operation comparison.
    assert_eq!(actual.len(), expected.len(), "pinned SRIV byte length");
    let expected = sriv::parse_sriv(&expected).expect("valid pinned SRIV");
    let actual = sriv::parse_sriv(&actual).expect("valid native SRIV");
    sriv::compare_sriv(&expected, &actual).expect("pinned artboard transform/opacity silver");
}

#[test]
fn artboard_clip_is_transformed_by_its_own_rotation() {
    let fixture = fixture("nested_artboard_opacity.riv", None);
    let root = fixture.artboard.core_handle();
    assert!(CoreRegistry::set_bool_handle(
        &root,
        i32::from(LayoutComponentBase::CLIP_PROPERTY_KEY),
        true
    ));
    set_double(
        &root,
        TransformComponentBase::ROTATION_PROPERTY_KEY,
        1.570_796_3,
    );
    fixture.artboard.advance_default(0.0);
    let transform = clip_transform(&draw_recording(&fixture)).expect("clipPath was called");
    assert!(transform[1].abs() > 0.0001 || transform[2].abs() > 0.0001);
}
