//! Native executable ports of the formerly partial Wave A bounds, iterator,
//! and stateful component cases.
use nuxie_render_api::{PersistentFactory, RecordingFactory, RecordingRenderer};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    bindable_artboard::RuntimeBindableArtboardHandle,
    core::{CoreHandle, CoreType},
    factory::RuntimeFactoryHandle,
    file::{File, RuntimeFileHandle},
    generated::{core_registry::CoreRegistry, text::text_value_run_base::TextValueRunBase},
    layout::n_sliced_node::NSlicedNode,
    layout_component::LayoutComponent,
    math::aabb::Aabb,
    math::vec2d::Vec2D,
    nested_artboard::NestedArtboard,
    node::Node,
    shapes::{image::Image, paint::shape_paint::ShapePaint, path::Path, shape::Shape},
    text::{text::Text, text_value_run::TextValueRun},
    viewmodel::{
        viewmodel::ViewModel, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_artboard::ViewModelInstanceArtboard,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_color::ViewModelInstanceColor,
        viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_string::ViewModelInstanceString,
    },
};
use std::path::PathBuf;
fn import(name: &str) -> (RuntimeFileHandle, RecordingRenderer) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let bytes =
        std::fs::read(root.join("tests/unit_tests/assets").join(name)).expect("pinned fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let renderer = factory.borrow().make_renderer();
    let handle = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    (
        File::import(&bytes, handle, None, None, None).expect("native File import"),
        renderer,
    )
}
fn read<T: std::any::Any, R>(handle: &CoreHandle, f: impl FnOnce(&T) -> R) -> R {
    handle.with_downcast(f).expect("native owner")
}
fn write<T: std::any::Any, R>(handle: &CoreHandle, f: impl FnOnce(&mut T) -> R) -> R {
    handle.with_downcast_mut(f).expect("native owner")
}
fn find<T: CoreType>(root: &CoreHandle, name: &str) -> CoreHandle {
    read::<Artboard, _>(root, |artboard| artboard.find_handle::<T>(name)).expect(name)
}
fn property(instance: &CoreHandle, name: &str) -> CoreHandle {
    read::<ViewModelInstance, _>(instance, |instance| instance.property_value_named(name))
        .expect(name)
}
fn close(actual: f32, expected: f32, label: &str) {
    // Catch Approx's default epsilon is 100 * float epsilon, evaluated in double.
    let difference = (f64::from(actual) - f64::from(expected)).abs();
    assert!(
        difference <= f64::from(100.0 * f32::EPSILON) * f64::from(expected).abs(),
        "{label}: expected {expected}, got {actual}"
    );
}
fn advance(root: &CoreHandle) {
    Artboard::advance_handle(
        root,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}
#[test]
fn upstream_background_shape_bounds_call_the_world_and_local_owners() {
    let (file, _renderer) = import("background_measure.riv");
    let artboard = file.with_file(File::artboard).expect("default artboard");
    let background = find::<Shape>(&artboard, "background");
    let name = find::<TextValueRun>(&artboard, "nameRun");
    advance(&artboard);
    let initial = read::<Shape, _>(&background, |shape| shape.compute_world_bounds(None));
    close(initial.width(), 42.010925, "initial width");
    close(initial.height(), 29.995453, "initial height");
    assert!(CoreRegistry::set_string_handle(
        &name,
        TextValueRunBase::TEXT_PROPERTY_KEY as i32,
        "much much longer".into()
    ));
    advance(&artboard);
    let extended = read::<Shape, _>(&background, |shape| shape.compute_world_bounds(None));
    close(extended.width(), 138.01093, "extended width");
    close(extended.height(), 29.995453, "extended height");
    write::<Artboard, _>(&artboard, |artboard| {
        artboard.mutable_world_transform().scale_by_values(0.5, 0.5);
        artboard.mark_world_transform_dirty();
    });
    advance(&artboard);
    let scaled = read::<Shape, _>(&background, |shape| shape.compute_world_bounds(None));
    close(scaled.width(), 138.01093 / 2.0, "scaled width");
    close(scaled.height(), 29.995453 / 2.0, "scaled height");
    let local = read::<Shape, _>(&background, Shape::compute_local_bounds);
    close(local.width(), 138.01093, "local width");
    close(local.height(), 29.995453, "local height");
}
#[test]
fn upstream_local_bounds_executes_the_complete_object_matrix() {
    let (file, _renderer) = import("local_bounds.riv");
    let artboard = file.with_file(File::artboard).expect("default artboard");
    let shape1 = find::<Shape>(&artboard, "Shape1");
    let shape2 = find::<Shape>(&artboard, "Shape2");
    let shape3 = find::<Shape>(&artboard, "Shape3");
    let text1 = find::<Text>(&artboard, "Text1");
    let text2 = find::<Text>(&artboard, "Text2");
    let group = find::<Node>(&artboard, "Group1");
    let image = find::<Image>(&artboard, "Image1");
    assert!(read::<Image, _>(&image, Image::image_asset).is_some());
    let nslice = find::<NSlicedNode>(&artboard, "NSlice2");
    let custom_shape = find::<Shape>(&artboard, "CustomShape1");
    let custom_path = find::<Path>(&artboard, "CustomPath1");
    let container = find::<LayoutComponent>(&artboard, "LayoutContainer");
    let cell = find::<LayoutComponent>(&artboard, "LayoutCellLeft");
    advance(&artboard);
    let cases = [
        (
            read::<Shape, _>(&shape1, Shape::local_bounds),
            [-35.0, -35.0, 35.0, 35.0],
            [false; 4],
        ),
        (
            read::<Shape, _>(&shape2, Shape::local_bounds),
            [-80.0, -80.0, 0.0, 0.0],
            [false; 4],
        ),
        (
            read::<Shape, _>(&shape3, Shape::local_bounds),
            [0.0, 0.0, 60.0, 60.0],
            [false; 4],
        ),
        (
            read::<Text, _>(&text1, Text::local_bounds),
            [0.0, 0.0, 159.55078, 24.19921],
            [false, false, true, true],
        ),
        (
            read::<Text, _>(&text2, Text::local_bounds),
            [-79.77539, -12.099609, 79.77539, 12.099609],
            [true; 4],
        ),
        (
            read::<Node, _>(&group, |node| node.local_bounds()),
            [0.0; 4],
            [false; 4],
        ),
        (
            read::<Image, _>(&image, Image::local_bounds),
            [-64.0, -64.0, 64.0, 64.0],
            [false; 4],
        ),
        (
            read::<NSlicedNode, _>(&nslice, NSlicedNode::local_bounds),
            [0.0, 0.0, 112.1891, 77.7086],
            [false, false, true, true],
        ),
        (
            read::<Shape, _>(&custom_shape, Shape::local_bounds),
            [-27.82596, -32.0276, 105.36988, 52.38258],
            [true; 4],
        ),
        (
            custom_path
                .with(|path| path.as_path().expect("Path-derived owner").local_bounds())
                .expect("live Path-derived owner"),
            [-11.52589, -25.32601, 100.66321, 52.38258],
            [true; 4],
        ),
        (
            read::<LayoutComponent, _>(&container, LayoutComponent::local_bounds),
            [0.0, 0.0, 200.0, 100.0],
            [false; 4],
        ),
        (
            read::<LayoutComponent, _>(&cell, LayoutComponent::local_bounds),
            [0.0, 0.0, 88.0, 84.0],
            [false; 4],
        ),
    ];
    for (bounds, expected, approximate) in cases {
        let actual = [bounds.left(), bounds.top(), bounds.right(), bounds.bottom()];
        for i in 0..4 {
            if approximate[i] {
                close(actual[i], expected[i], "local edge");
            } else {
                assert_eq!(actual[i], expected[i]);
            }
        }
    }
}
#[test]
fn upstream_child_typed_iterators_execute_the_iterator_owners() {
    let (file, _renderer) = import("juice.riv");
    let artboard = file
        .with_file(|file| file.artboard_default())
        .expect("juice");
    artboard.with_artboard(|artboard| {
        let mut count = 0;
        for child in artboard.children_typed::<Node>().iter() {
            assert_eq!(
                child
                    .with(|child| child.as_component().unwrap().name().to_owned())
                    .unwrap(),
                "root"
            );
            count += 1;
        }
        assert_eq!(count, 1);
        let mut paint_count = 0;
        for paint in artboard.children_typed::<ShapePaint>().iter() {
            assert!(
                !paint
                    .with(|paint| paint.as_shape_paint_behavior().unwrap().is_translucent())
                    .unwrap()
            );
            paint_count += 1;
        }
        assert_eq!(paint_count, 1);
        let paints = artboard.objects_typed::<ShapePaint>();
        let count = paints.iter().count();
        assert_eq!(paints.size(), 20);
        assert_eq!(paints.size(), count);
    });
}
struct StatefulFixture {
    file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    root: CoreHandle,
    renderer: RecordingRenderer,
}
impl StatefulFixture {
    fn load(asset: &str, name: &str) -> Self {
        let (file, renderer) = import(asset);
        let artboard = file
            .with_file(|file| file.artboard_named(name))
            .expect("named artboard");
        let machine = artboard
            .state_machine_instance_handle(0)
            .expect("state machine");
        let id = artboard.with_artboard(|artboard| artboard.base.base.view_model_id());
        let root = if id == u32::MAX {
            file.with_file_mut(|file| {
                file.create_view_model_instance_for_artboard(artboard.core_handle())
            })
        } else {
            file.with_file(|file| file.create_view_model_instance_at(id as usize, 0))
        }
        .expect("main VMI");
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(root.clone()));
        let mut fixture = Self {
            file,
            artboard,
            machine,
            root,
            renderer,
        };
        fixture.frames(1, 0.0);
        fixture
    }
    fn frames(&mut self, count: usize, elapsed: f32) {
        for _ in 0..count {
            self.machine.advance_and_apply(elapsed);
            self.artboard.draw(&mut self.renderer);
        }
    }
    fn source(&self, name: &str) -> RuntimeBindableArtboardHandle {
        self.file
            .with_file(|file| file.bindable_artboard_named(name))
            .expect("source artboard")
    }
    fn set_source(&self, name: &str, source: Option<RuntimeBindableArtboardHandle>) {
        write::<ViewModelInstanceArtboard, _>(&property(&self.root, name), |value| {
            value.set_asset(source)
        });
    }
    fn swapped_nested(&self, names: &[&str]) -> Option<CoreHandle> {
        let nested = self
            .artboard
            .with_artboard(|artboard| artboard.nested_artboards());
        nested.into_iter().find(|nested| {
            nested
                .with(|nested| nested.as_nested_artboard().unwrap().source_artboard())
                .flatten()
                .and_then(|source| {
                    source.with(|source| source.as_component().unwrap().name().to_owned())
                })
                .is_some_and(|name| names.contains(&name.as_str()))
        })
    }
    fn click(&mut self, y: f32) {
        self.machine.with_instance_mut(|machine| {
            machine.pointer_down(Vec2D::new(50.0, y), 1);
            machine.pointer_up(Vec2D::new(50.0, y), 1);
        });
        self.frames(1, 0.016);
    }
}
fn nested_instance(nested: &CoreHandle) -> RuntimeArtboardInstanceHandle {
    nested
        .with(|nested| {
            nested
                .as_nested_artboard()
                .unwrap()
                .artboard_instance_default()
        })
        .flatten()
        .expect("nested instance")
}
fn main_vmi(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    artboard
        .data_context()
        .expect("nested data context")
        .with_context(|context| context.main_view_model_instance())
        .expect("nested main VMI")
}
#[test]
fn upstream_stateful_component_dynamic_artboard_swap_replays_the_complete_fixture() {
    let mut f = StatefulFixture::load("stateful_artboard_swap.riv", "Main");
    let button = f.source("Button");
    let stroked = f.source("StrokedButton");
    f.frames(2, 0.016);
    assert!(f.swapped_nested(&["Button", "StrokedButton"]).is_none());
    f.set_source("buttonArtboard", Some(button.clone()));
    f.frames(5, 0.016);
    f.swapped_nested(&["Button"]).expect("Button nested");
    f.set_source("buttonArtboard", Some(stroked));
    f.frames(5, 0.016);
    let nested = f
        .swapped_nested(&["StrokedButton"])
        .expect("StrokedButton nested");
    let stroked_vmi = main_vmi(&nested_instance(&nested));
    write::<ViewModelInstanceNumber, _>(&property(&stroked_vmi, "strokeWidth"), |value| {
        value.set_value(8.0)
    });
    f.frames(5, 0.016);
    f.set_source("buttonArtboard", Some(button.clone()));
    f.frames(5, 0.016);
    let nested = f.swapped_nested(&["Button"]).expect("Button restored");
    let vmi = main_vmi(&nested_instance(&nested));
    property(&vmi, "count");
    assert!(
        read::<ViewModelInstance, _>(&vmi, |vmi| vmi.property_value_named("strokeWidth")).is_none()
    );
    f.set_source("buttonArtboard", None);
    f.frames(5, 0.016);
    assert!(f.swapped_nested(&["Button", "StrokedButton"]).is_none());
    f.set_source("buttonArtboard", Some(button));
    f.frames(5, 0.016);
    let nested = f.swapped_nested(&["Button"]).expect("Button after clear");
    let vmi = main_vmi(&nested_instance(&nested));
    property(&vmi, "count");
    assert!(
        read::<ViewModelInstance, _>(&vmi, |vmi| vmi.property_value_named("strokeWidth")).is_none()
    );
}
fn stateful_child(nested: &CoreHandle) -> Option<CoreHandle> {
    nested
        .with(|nested| {
            nested
                .as_container_component()
                .unwrap()
                .children_typed::<ViewModelInstance>()
                .first()
        })
        .flatten()
}
#[test]
fn upstream_stateful_nested_source_switch_replays_matching_and_different_vm_lifetimes() {
    let mut f = StatefulFixture::load("stateful_source_switch.riv", "ParentArtboard");
    let matching = f.source("MatchingArtboardA");
    let different = f.source("DifferentArtboardB");
    let nested = f
        .artboard
        .with_artboard(|artboard| artboard.nested_artboards())
        .into_iter()
        .find(|nested| {
            nested
                .with(|nested| nested.as_nested_artboard().unwrap().base.is_stateful())
                .unwrap()
        })
        .expect("stateful nested artboard");
    let child = stateful_child(&nested).expect("stateful child");
    let initial_id = read::<ViewModelInstance, _>(&child, |child| child.base.view_model_id());
    let label = property(&f.root, "labelInput");
    f.frames(5, 0.016);
    f.set_source("sourceArtboard", Some(matching.clone()));
    f.frames(5, 0.016);
    f.swapped_nested(&["MatchingArtboardA"])
        .expect("matching source");
    assert_eq!(main_vmi(&nested_instance(&nested)), child);
    write::<ViewModelInstanceString, _>(&label, |label| label.set_value("Matching A"));
    f.frames(10, 0.016);
    f.set_source("sourceArtboard", Some(different));
    f.frames(5, 0.016);
    f.swapped_nested(&["DifferentArtboardB"])
        .expect("different source");
    let bound = main_vmi(&nested_instance(&nested));
    assert_ne!(bound, child);
    assert_ne!(
        read::<ViewModelInstance, _>(&bound, |vmi| vmi.base.view_model_id()),
        read::<ViewModelInstance, _>(&child, |vmi| vmi.base.view_model_id())
    );
    assert_eq!(stateful_child(&nested), Some(child.clone()));
    assert_eq!(
        read::<ViewModelInstance, _>(&child, |child| child.base.view_model_id()),
        initial_id
    );
    write::<ViewModelInstanceString, _>(&label, |label| label.set_value("Different B"));
    f.frames(10, 0.016);
    f.set_source("sourceArtboard", Some(matching));
    f.frames(5, 0.016);
    f.swapped_nested(&["MatchingArtboardA"])
        .expect("matching source restored");
    assert_eq!(main_vmi(&nested_instance(&nested)), child);
    write::<ViewModelInstanceString, _>(&label, |label| label.set_value("Matching A Again"));
    f.frames(10, 0.016);
}
fn add_item(instance: &CoreHandle, list: &CoreHandle) {
    let item = instance
        .insert_sibling(ViewModelInstanceListItem::default())
        .expect("list item allocation");
    write::<ViewModelInstanceListItem, _>(&item, |item| {
        item.set_view_model_instance(Some(instance.clone()))
    });
    write::<ViewModelInstanceList, _>(list, |list| list.add_item(item));
}
#[test]
fn upstream_stateful_component_list_bridge_replays_add_remove_click_readd_and_clear() {
    let mut f = StatefulFixture::load("stateful_list_props.riv", "Main");
    let list = property(&f.root, "buttons");
    let model = f
        .file
        .with_file(|file| file.view_model_named("ButtonVM"))
        .expect("ButtonVM");
    let make_button = |label: &str, tint: u32| {
        let button = f
            .file
            .with_file_mut(|file| file.create_view_model_instance(model.clone()))
            .expect("button instance");
        write::<ViewModelInstanceString, _>(&property(&button, "label"), |value| {
            value.set_value(label)
        });
        write::<ViewModelInstanceColor, _>(&property(&button, "tint"), |value| {
            value.set_value(tint as i32)
        });
        add_item(&button, &list);
        button
    };
    let _alpha = make_button("Alpha", 0xffff3344);
    let beta = make_button("Beta", 0xff33aaff);
    let gamma = make_button("Gamma", 0xff44cc55);
    f.frames(3, 0.016);
    assert_eq!(
        read::<ViewModelInstanceList, _>(&list, |list| list.list_items().len()),
        3
    );
    write::<ViewModelInstanceList, _>(&list, |list| list.remove_item_at(1));
    f.frames(5, 0.016);
    assert_eq!(
        read::<ViewModelInstanceList, _>(&list, |list| list.list_items().len()),
        2
    );
    let gamma_clicked = property(&gamma, "clicked");
    let beta_clicked = property(&beta, "clicked");
    assert!(!read::<ViewModelInstanceBoolean, _>(
        &gamma_clicked,
        ViewModelInstanceBoolean::value
    ));
    f.click(73.0);
    let gamma_after =
        read::<ViewModelInstanceBoolean, _>(&gamma_clicked, ViewModelInstanceBoolean::value);
    let beta_removed =
        read::<ViewModelInstanceBoolean, _>(&beta_clicked, ViewModelInstanceBoolean::value);
    f.frames(3, 0.016);
    add_item(&beta, &list);
    f.frames(5, 0.016);
    assert_eq!(
        read::<ViewModelInstanceList, _>(&list, |list| list.list_items().len()),
        3
    );
    f.click(118.0);
    let beta_after =
        read::<ViewModelInstanceBoolean, _>(&beta_clicked, ViewModelInstanceBoolean::value);
    f.frames(3, 0.016);
    while read::<ViewModelInstanceList, _>(&list, |list| !list.list_items().is_empty()) {
        write::<ViewModelInstanceList, _>(&list, |list| list.remove_item_at(0));
    }
    f.frames(5, 0.016);
    assert_eq!(
        read::<ViewModelInstanceList, _>(&list, |list| list.list_items().len()),
        0
    );
    assert!(gamma_after);
    assert!(!beta_removed);
    assert!(beta_after);
}
