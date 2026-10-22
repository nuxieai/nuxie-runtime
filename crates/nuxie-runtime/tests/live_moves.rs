use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::live_move_fixture as fixture;
use nuxie_runtime::{
    live_move::*,
    source::{
        animation::nested_state_machine::NestedStateMachine,
        artboard_component_list::ArtboardComponentList, generated::core_registry::CoreRegistry,
        nested_artboard::NestedArtboard, node::Node,
    },
    *,
};
struct Scene {
    _file: RuntimeFileHandle,
    factory: PersistentFactory<RecordingFactory>,
    root: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    a: RuntimeArtboardInstanceHandle,
    b: RuntimeArtboardInstanceHandle,
    s: CoreHandle,
    d: CoreHandle,
    f: CoreHandle,
    moves: LiveMoves,
}
fn child(h: &CoreHandle) -> Option<RuntimeArtboardInstanceHandle> {
    h.with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
        .flatten()
}
fn machine(h: &CoreHandle) -> RuntimeStateMachineInstanceHandle {
    h.with(|o| o.as_nested_artboard().unwrap().nested_animations().to_vec())
        .unwrap()
        .iter()
        .find_map(|a| {
            a.with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
                .flatten()
        })
        .unwrap()
}
fn val(h: &CoreHandle, key: i32) -> f32 {
    CoreRegistry::get_double_handle(h, key).unwrap()
}
fn set(h: &CoreHandle, key: i32, v: f32) {
    assert!(CoreRegistry::set_double_handle(h, key, v));
}
fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.002, "expected {b}, got {a}");
}
impl Scene {
    fn new() -> Self {
        Self::from_bytes(&fixture::fixture(2))
    }
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("fixture imports");
        let root = file.with_file(|f| f.artboard_named("Page")).unwrap();
        let vm = file
            .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
            .unwrap();
        let machine = root.default_state_machine_handle().unwrap();
        machine.with_instance_mut(|m| m.bind_view_model_instance(vm));
        machine.advance_and_apply(0.0);
        let rows = screens(&root).unwrap();
        let a = rows["A"].clone();
        let b = rows["B"].clone();
        let s = a.with_artboard(|b| b.find_all_handles::<NestedArtboard>())[0].clone();
        let hs = b.with_artboard(|b| b.find_all_handles::<NestedArtboard>());
        let d = hs[0].clone();
        let f = hs[1].clone();
        empty_slot(&f).unwrap();
        let af = a.with_artboard(|b| b.find_all_handles::<NestedArtboard>())[1].clone();
        empty_slot(&af).unwrap();
        let mut moves = LiveMoves::default();
        moves.set_moves(fixture::MOVES).unwrap();
        Self {
            _file: file,
            factory,
            root,
            machine,
            a,
            b,
            s,
            d,
            f,
            moves,
        }
    }
    fn advance(&mut self, dt: f32) {
        self.machine.advance_and_apply(dt);
        self.moves.after_advance(&self.root).unwrap();
    }
    fn begin(&mut self) {
        self.moves.step_begin(&self.root, "B", Some("t1")).unwrap();
    }
    fn end(&mut self) {
        self.moves.step_end(&self.root, "B", Some("t1")).unwrap();
    }
}
#[test]
fn continuity_identity_and_once_only() {
    let mut s = Scene::new();
    let original = child(&s.s).unwrap();
    let m = machine(&s.s);
    let objects = original.with_artboard(|b| b.objects().to_vec());
    assert_eq!(objects.iter().flatten().count(), 12);
    let keyed = original
        .with_artboard(|b| b.find_handle::<Node>("keyed"))
        .unwrap();
    s.advance(0.3);
    close(val(&keyed, 13), 30.0);
    close(val(&keyed, 14), 30.0);
    s.begin();
    assert!(child(&s.s).is_none());
    assert!(child(&s.f).unwrap().core_handle() == original.core_handle());
    for _ in 0..10 {
        s.advance(0.02);
    }
    close(val(&keyed, 13), 50.0);
    close(val(&keyed, 14), 50.0);
    s.end();
    assert_eq!(child(&s.d).unwrap().core_handle(), original.core_handle());
    assert!(m.downgrade().ptr_eq(&machine(&s.d).downgrade()));
    assert_eq!(
        m.with_instance(|m| m.parent_nested_artboard()),
        Some(s.d.clone())
    );
    assert_eq!(objects, original.with_artboard(|b| b.objects().to_vec()));
    s.moves.step_begin(&s.root, "A", Some("back")).unwrap();
    s.advance(0.1);
    s.moves.step_end(&s.root, "A", Some("back")).unwrap();
    close(val(&keyed, 13), 60.0);
    close(val(&keyed, 14), 60.0);
}
#[test]
fn follows_current_frame_and_comes_home() {
    let mut s = Scene::new();
    let id = child(&s.s).unwrap().core_handle();
    s.begin();
    for (frame, progress) in [0.0, 0.5, 1.0].into_iter().enumerate() {
        s.moves.flights[0].progress = progress;
        s.advance(0.1);
        let a = slot_pose(&s.s).unwrap();
        let b = slot_pose(&s.d).unwrap();
        let f = slot_pose(&s.f).unwrap();
        close(b.transform[4], 205.0 + 10.0 * (frame + 1) as f32);
        close(
            f.transform[4],
            a.transform[4] + (b.transform[4] - a.transform[4]) * progress,
        );
        close(
            f.transform[5],
            a.transform[5] + (b.transform[5] - a.transform[5]) * progress,
        );
    }
    s.end();
    s.moves.step_begin(&s.root, "A", None).unwrap();
    assert_eq!(child(&s.s).unwrap().core_handle(), id);
    assert!(child(&s.d).is_none());
    s.moves.step_end(&s.root, "A", None).unwrap();
    s.moves.step_begin(&s.root, "B", None).unwrap();
    assert_eq!(child(&s.d).unwrap().core_handle(), id);
    s.moves.step_end(&s.root, "B", None).unwrap();
}
#[test]
fn rejected_moves_do_not_mutate_slots() {
    let s = Scene::new();
    let id = child(&s.s).unwrap().core_handle();
    assert!(move_instance(&s.s, &s.s).is_err());
    assert_eq!(child(&s.s).unwrap().core_handle(), id);
    assert!(move_instance(&s.f, &s.d).is_err());
    assert!(child(&s.f).is_none());
}

fn vm(board: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    board
        .data_context()
        .unwrap()
        .with_context(|c| c.main_view_model_instance())
        .unwrap()
}
fn property(vm: &CoreHandle, name: &str) -> CoreHandle {
    vm.with_downcast::<source::viewmodel::viewmodel_instance::ViewModelInstance, _>(|vm| {
        vm.property_value_named(name)
    })
    .flatten()
    .unwrap_or_else(|| {
        panic!(
            "missing {name} on VM {:?}",
            CoreRegistry::get_uint_handle(vm, 566)
        )
    })
}
#[test]
fn private_values_text_and_destination_prop_survive() {
    let mut s = Scene::new();
    let original = child(&s.s).unwrap();
    let model = vm(&original);
    let count = property(&model, "count");
    set(&count, 575, 7.0);
    let text = original
        .with_artboard(|b| b.find_handle::<source::text::text_input::TextInput>("edit"))
        .unwrap();
    assert!(
        text.with_downcast_mut::<source::text::text_input::TextInput, _>(
            |t| t.text_input("hello mascot")
        )
        .unwrap()
    );
    let context = original.data_context().unwrap();
    s.begin();
    s.advance(0.2);
    s.end();
    s.advance(0.0);
    assert_eq!(vm(&child(&s.d).unwrap()), model);
    close(val(&count, 575), 7.0);
    assert_eq!(
        text.with_downcast::<source::text::text_input::TextInput, _>(|t| t.text().to_owned())
            .unwrap(),
        "hello mascot"
    );
    assert!(context.ptr_eq(&original.data_context().unwrap()));
    assert!(
        context
            .with_context(|c| c.parent())
            .unwrap()
            .ptr_eq(&s.b.data_context().unwrap())
    );
    close(val(&property(&model, "prop"), 575), 200.0);
    let offset = property(&vm(&s.b), "offset");
    set(&offset, 575, 321.0);
    s.advance(0.0);
    close(val(&property(&model, "prop"), 575), 321.0);
    close(val(&count, 575), 7.0);
}
#[test]
fn hits_follow_the_new_host() {
    let mut s = Scene::new();
    let moved = machine(&s.s);
    s.begin();
    s.advance(0.0);
    s.end();
    s.advance(0.0);
    let p = slot_pose(&s.d).unwrap().transform * source::math::vec2d::Vec2D::new(20.0, 20.0);
    s.machine.with_instance_mut(|m| {
        m.pointer_down(p, 0, source::pointer_button::PointerButton::Primary)
    });
    close(
        moved.with_instance(|m| m.get_number("hits").unwrap().value()),
        9.0,
    );
    moved.set_number("hits", 0.0);
    let p = slot_pose(&s.s).unwrap().transform * source::math::vec2d::Vec2D::new(20.0, 20.0);
    s.machine.with_instance_mut(|m| {
        m.pointer_down(p, 1, source::pointer_button::PointerButton::Primary)
    });
    close(
        moved.with_instance(|m| m.get_number("hits").unwrap().value()),
        0.0,
    );
}
#[test]
fn empty_source_draws_no_geometry() {
    let mut s = Scene::new();
    s.factory.borrow_mut().clear();
    let mut renderer = s.factory.borrow().make_renderer();
    s.a.draw(&mut renderer);
    let before = s
        .factory
        .borrow()
        .stream()
        .lines()
        .filter(|l| l.starts_with("drawPath "))
        .count();
    assert_eq!(before, 1);
    s.begin();
    s.factory.borrow_mut().clear();
    s.a.draw(&mut renderer);
    let after = s
        .factory
        .borrow()
        .stream()
        .lines()
        .filter(|l| l.starts_with("drawPath "))
        .count();
    assert_eq!(after, 0);
}

#[test]
fn collapsed_rows_cost_one_row_and_wake_with_data() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::fixture(40),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = file.with_file(|f| f.artboard_named("Page")).unwrap();
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(model));
    machine.advance_and_apply(0.0);
    let list = root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let rows = screens(&root).unwrap();
    for row in rows.values() {
        let hosts = row.with_artboard(|b| b.nested_artboards());
        empty_slot(&hosts[1]).unwrap();
    }
    machine.advance_and_apply(0.0);
    measure_advances(true);
    machine.advance_and_apply(0.01);
    let all = measure_advances(false);
    let mut renderer = factory.borrow().make_renderer();
    factory.borrow_mut().clear();
    root.draw(&mut renderer);
    let all_draws = factory
        .borrow()
        .stream()
        .lines()
        .filter(|l| l.starts_with("drawPath "))
        .count();
    for i in 1..40 {
        set_row_collapsed(&list, i, true).unwrap();
    }
    let inactive = rows["B"].with_artboard(|b| b.nested_artboards())[0].clone();
    let unit = child(&inactive).unwrap();
    let keyed = unit
        .with_artboard(|b| b.find_handle::<Node>("keyed"))
        .unwrap();
    let before = val(&keyed, 13);
    set(&property(&vm(&rows["B"]), "offset"), 575, 777.0);
    measure_advances(true);
    machine.advance_and_apply(0.01);
    let one = measure_advances(false);
    close(val(&keyed, 13), before);
    factory.borrow_mut().clear();
    root.draw(&mut renderer);
    let one_draws = factory
        .borrow()
        .stream()
        .lines()
        .filter(|l| l.starts_with("drawPath "))
        .count();
    assert_eq!(all, 81);
    assert_eq!(one, 3);
    assert_eq!(all_draws, 40);
    assert_eq!(one_draws, 1);
    fn timed(m: &RuntimeStateMachineInstanceHandle) -> std::time::Duration {
        let t = std::time::Instant::now();
        for _ in 0..100 {
            m.advance_and_apply(0.001);
        }
        t.elapsed()
    }
    let collapsed = timed(&machine);
    let baseline = Scene::new();
    let baseline_list = baseline
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    set_row_collapsed(&baseline_list, 1, true).unwrap();
    baseline.machine.advance_and_apply(0.0);
    let baseline_time = timed(&baseline.machine);
    set_row_collapsed(&list, 1, false).unwrap();
    machine.advance_and_apply(0.0);
    close(val(&property(&vm(&unit), "prop"), 575), 777.0);
    for i in 2..40 {
        set_row_collapsed(&list, i, false).unwrap();
    }
    let expanded = timed(&machine);
    println!(
        "rows: advances={all}/{one}; draws={all_draws}/{one_draws}; 100 advances expanded={expanded:?}, collapsed={collapsed:?}, one-row baseline={baseline_time:?}"
    );
    assert!(collapsed.as_secs_f64() < expanded.as_secs_f64() * 0.5);
}

#[test]
fn focus_scope_moves_without_blurring_the_field() {
    let mut s = Scene::new();
    let unit = child(&s.s).unwrap();
    let data =
        unit.with_artboard(|b| b.find_all_handles::<source::focus_data::FocusData>())[0].clone();
    let node = data
        .with_downcast::<source::focus_data::FocusData, _>(|f| f.existing_focus_node())
        .flatten()
        .unwrap();
    let manager = unit.with_artboard(|b| b.focus_manager()).unwrap();
    manager.with_focus_manager_mut(|m| m.set_focus(node.clone()));
    assert!(manager.with_focus_manager(|m| m.has_primary_focus(&node)));
    s.begin();
    assert!(manager.with_focus_manager(|m| m.has_primary_focus(&node)));
    s.advance(0.1);
    s.end();
    assert!(manager.with_focus_manager(|m| m.has_primary_focus(&node)));
    let text = unit
        .with_artboard(|b| b.find_handle::<source::text::text_input::TextInput>("edit"))
        .unwrap();
    assert!(
        text.with_downcast::<source::text::text_input::TextInput, _>(|t| t.is_focused())
            .unwrap()
    );
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    set_row_collapsed(&list, 0, true).unwrap();
    s.advance(0.1);
    assert!(manager.with_focus_manager(|m| m.has_primary_focus(&node)));
}

#[test]
fn empty_layout_slot_preserves_the_siblings_position() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::layout_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = file.with_file(|f| f.artboard_named("LayoutPage")).unwrap();
    root.advance_default(0.0);
    let source = root
        .with_artboard(|b| {
            b.find_handle::<source::nested_artboard_layout::NestedArtboardLayout>("source")
        })
        .unwrap();
    let destination = root
        .with_artboard(|b| b.find_handle::<NestedArtboard>("destination"))
        .unwrap();
    let sibling = root
        .with_artboard(|b| b.find_handle::<source::layout_component::LayoutComponent>("sibling"))
        .unwrap();
    let position = || {
        sibling
            .with_downcast::<source::layout_component::LayoutComponent, _>(|b| b.layout_x())
            .unwrap()
    };
    let before = position();
    assert!(before >= 149.9, "sibling before={before}");
    move_instance(&source, &destination).unwrap();
    root.advance_default(0.0);
    close(position(), before);
    let bounds = source
        .with_downcast::<source::nested_artboard_layout::NestedArtboardLayout, _>(|s| {
            s.layout_bounds()
        })
        .unwrap();
    close(bounds.width(), 150.0);
    close(bounds.height(), 80.0);
    move_instance(&destination, &source).unwrap();
    root.advance_default(0.0);
    close(position(), before);
}

#[test]
fn rotation_scale_opacity_layout_size_and_path_are_blended() {
    let mut s = Scene::new();
    let pa =
        s.a.with_artboard(|b| b.find_handle::<Node>("card"))
            .unwrap();
    let pb =
        s.b.with_artboard(|b| b.find_handle::<Node>("header"))
            .unwrap();
    let pf =
        s.b.with_artboard(|b| b.find_handle::<Node>("flight"))
            .unwrap();
    for (h, r, x, y, o) in [
        (&pa, 170.0_f32.to_radians(), 2.0, 3.0, 0.4),
        (&pb, (-170.0_f32).to_radians(), 4.0, 5.0, 0.8),
        (&pf, 0.7, 1.2, 0.8, 0.9),
    ] {
        set(h, 15, r);
        set(h, 16, x);
        set(h, 17, y);
        set(h, 18, o);
    }
    child(&s.d).unwrap().set_size(200.0, 120.0);
    s.advance(0.0);
    s.begin();
    s.moves.flights[0].progress = 0.5;
    s.advance(0.0);
    let f = slot_pose(&s.f).unwrap();
    let a = slot_pose(&s.s).unwrap();
    let b = slot_pose(&s.d).unwrap();
    let c = f.transform.decompose();
    close(c.x(), (a.transform[4] + b.transform[4]) * 0.5);
    close(c.y(), (a.transform[5] + b.transform[5]) * 0.5);
    close(c.rotation().abs(), std::f32::consts::PI);
    close(c.scale_x(), 3.0);
    close(c.scale_y(), 4.0);
    close(f.opacity, 0.6);
    let paint = child(&s.f)
        .unwrap()
        .with_artboard(|b| b.find_all_handles::<source::shapes::paint::fill::Fill>())[0]
        .clone();
    close(
        paint
            .with(|o| o.as_shape_paint().unwrap().render_opacity())
            .unwrap(),
        0.6,
    );

    close(f.size.x, 150.0);
    close(f.size.y, 100.0);
    let unit = child(&s.f).unwrap();
    close(unit.with_artboard(|b| b.layout_width()), 150.0);
    let reflow = unit
        .with_artboard(|b| b.find_handle::<source::layout_component::LayoutComponent>("reflow"))
        .unwrap();
    close(
        reflow
            .with_downcast::<source::layout_component::LayoutComponent, _>(|c| c.layout_width())
            .unwrap(),
        150.0,
    );

    s.end();
    let d = slot_pose(&s.d).unwrap();
    close(d.size.x, 200.0);
    close(d.size.y, 120.0);
    // Scale mode changes the flight's transform while retaining its unit box.
    s.moves.step_begin(&s.root, "A", Some("back")).unwrap();
    s.moves.flights[0].progress = 0.5;
    s.advance(0.0);
    let f = slot_pose(&s.f).unwrap();
    close(f.size.x, 200.0);
    close(f.size.y, 120.0);
    close(f.transform.decompose().scale_x(), 3.0 * 150.0 / 200.0);
    close(f.transform.decompose().scale_y(), 4.0 * 100.0 / 120.0);
}

#[test]
fn timeline_controls_progress_and_path_offset() {
    let mut s = Scene::from_bytes(&fixture::control_fixture());
    s.begin();
    s.moves.flights[0].progress = 0.9;
    s.advance(0.1);
    let a = slot_pose(&s.s).unwrap();
    let b = slot_pose(&s.d).unwrap();
    let f = slot_pose(&s.f).unwrap();
    close(
        f.transform[4],
        a.transform[4] + (b.transform[4] - a.transform[4]) * 0.25 + 7.0,
    );
    close(
        f.transform[5],
        a.transform[5] + (b.transform[5] - a.transform[5]) * 0.25 + 11.0,
    );
    let progress =
        s.b.with_artboard(|b| b.find_handle::<Node>("progress"))
            .unwrap();
    set(&progress, 13, 1.0);
    s.advance(0.1);
    let b = slot_pose(&s.d).unwrap();
    let f = slot_pose(&s.f).unwrap();
    close(f.transform[4], b.transform[4] + 7.0);
    close(f.transform[5], b.transform[5] + 11.0);
}

#[test]
fn moved_vm_survives_removing_its_original_row() {
    let mut s = Scene::new();
    let unit = child(&s.s).unwrap();
    let model = vm(&unit);
    let count = property(&model, "count");
    set(&count, 575, 7.0);
    s.begin();
    s.end();
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let item_b = list
        .with_downcast::<ArtboardComponentList, _>(|l| l.list_item(1))
        .flatten()
        .unwrap();
    s.a = s.b.clone();
    ArtboardComponentList::update_list_occurrence(&list, &[item_b]);
    set(&count, 575, 8.0);
    s.advance(0.1);
    assert_eq!(vm(&child(&s.d).unwrap()), model);
    close(val(&count, 575), 8.0);
}

#[test]
fn semantic_subtree_has_one_stable_occurrence_at_every_move_boundary() {
    use source::{
        focus_data::FocusData, semantic::semantic_data::SemanticData, text::text_input::TextInput,
    };
    use std::collections::HashSet;
    for focused in [false, true] {
        let mut s = Scene::from_bytes(&fixture::semantic_fixture());
        s.machine.with_instance_mut(|m| m.enable_semantics());
        let manager = s.machine.with_instance(|m| m.semantic_manager()).unwrap();
        let unit = child(&s.s).unwrap();
        let boundary = unit.with_artboard(|b| b.semantic_boundary_node()).unwrap();
        let nodes = unit.with_artboard(|b| b.find_all_handles::<SemanticData>());
        let ids: HashSet<_> = nodes
            .iter()
            .map(|h| {
                h.with_downcast::<SemanticData, _>(|d| {
                    d.existing_semantic_node().unwrap().borrow().id()
                })
                .unwrap()
            })
            .collect();
        assert_eq!(ids.len(), 3);
        let field = nodes
            .iter()
            .find_map(|h| {
                h.with_downcast::<SemanticData, _>(|d| d.existing_semantic_node())
                    .flatten()
                    .filter(|n| n.borrow().role == 6)
            })
            .unwrap();
        let group_id = nodes
            .iter()
            .find_map(|h| {
                h.with_downcast::<SemanticData, _>(|d| d.existing_semantic_node())
                    .flatten()
                    .filter(|n| n.borrow().role == 9)
            })
            .unwrap()
            .borrow()
            .id();
        let field_id = field.borrow().id();
        let text = unit
            .with_artboard(|b| b.find_handle::<TextInput>("edit"))
            .unwrap();
        text.with_downcast_mut::<TextInput, _>(|t| t.text_input("hello mascot"));
        let focus_node = unit.with_artboard(|b| b.find_all_handles::<FocusData>())[0]
            .with_downcast::<FocusData, _>(|f| f.existing_focus_node())
            .flatten()
            .unwrap();
        let focus_manager = unit.with_artboard(|b| b.focus_manager()).unwrap();
        if focused {
            focus_manager.with_focus_manager_mut(|m| m.set_focus(focus_node.clone()));
        }
        let check = |host: &CoreHandle| {
            let snapshot = manager.with_semantic_manager_mut(|m| m.snapshot().to_vec());
            let unique: HashSet<_> = snapshot.iter().map(|n| n.id).collect();
            assert_eq!(
                unique.len(),
                snapshot.len(),
                "duplicate semantic ids: {snapshot:?}"
            );
            let parent =
                SemanticData::find_closest_semantic_node_handle(Some(host.clone())).unwrap();
            assert_eq!(
                boundary.borrow().parent().unwrap().borrow().id(),
                parent.borrow().id()
            );
            for id in &ids {
                assert_eq!(snapshot.iter().filter(|n| n.id == *id).count(), 1);
            }
            assert_eq!(
                snapshot
                    .iter()
                    .find(|n| n.id == group_id)
                    .unwrap()
                    .parent_id,
                parent.borrow().id() as i32
            );
            let mut descendants = HashSet::from([group_id]);
            loop {
                let old_len = descendants.len();
                for node in &snapshot {
                    if descendants.contains(&(node.parent_id as u32)) {
                        descendants.insert(node.id);
                    }
                }
                if descendants.len() == old_len {
                    break;
                }
            }
            assert_eq!(descendants, ids);
            assert_eq!(field.borrow().id(), field_id);
            assert_eq!(
                snapshot
                    .iter()
                    .find(|n| n.id == field_id)
                    .unwrap()
                    .state_flags
                    & (1 << 7)
                    != 0,
                focused
            );
            assert_eq!(
                snapshot.iter().find(|n| n.id == field_id).unwrap().label,
                "mascot field"
            );
            assert_eq!(
                text.with_downcast::<TextInput, _>(|t| t.text().to_owned())
                    .unwrap(),
                "hello mascot"
            );
            assert_eq!(
                text.with_downcast::<TextInput, _>(|t| t.is_focused())
                    .unwrap(),
                focused
            );
            assert_eq!(
                focus_manager.with_focus_manager(|m| m.has_primary_focus(&focus_node)),
                focused
            );
        };
        check(&s.s);
        s.begin();
        check(&s.f);
        let list = s
            .root
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
            .unwrap();
        let keyed = unit
            .with_artboard(|b| b.find_handle::<Node>("keyed"))
            .unwrap();
        let before_sleep = val(&keyed, 13);
        set_row_collapsed(&list, 1, true).unwrap();
        s.advance(0.1);
        close(val(&keyed, 13), before_sleep);
        let asleep = manager.with_semantic_manager_mut(|m| m.snapshot().to_vec());
        assert!(asleep.iter().all(|node| !ids.contains(&node.id)));
        set_row_collapsed(&list, 1, false).unwrap();
        check(&s.f);
        s.advance(0.1);
        check(&s.f);
        s.end();
        check(&s.d);
        s.moves.step_begin(&s.root, "A", None).unwrap();
        check(&s.s);
        s.moves.step_end(&s.root, "A", None).unwrap();
        check(&s.s);
        // Repeat with a return flight to catch links retained across several moves.
        s.begin();
        check(&s.f);
        s.end();
        check(&s.d);
        s.moves.step_begin(&s.root, "A", Some("back")).unwrap();
        check(&s.f);
        s.moves.step_end(&s.root, "A", Some("back")).unwrap();
        check(&s.s);
        let list = s
            .root
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
            .unwrap();
        set_row_collapsed(&list, 0, true).unwrap();
        let asleep = manager.with_semantic_manager_mut(|m| m.snapshot().to_vec());
        assert!(asleep.iter().all(|node| !ids.contains(&node.id)));
        set_row_collapsed(&list, 0, false).unwrap();
        check(&s.s);
    }
}

#[test]
fn collapsed_semantic_geometry_is_validated_again_before_publishing_on_wake() {
    use source::{
        generated::layout_component_base::LayoutComponentBase,
        math::path_types::PathDirection,
        semantic::semantic_provider::{SemanticGeometryError, validate_semantic_geometry},
    };
    let mut s = Scene::from_bytes(&fixture::semantic_fixture());
    s.machine.with_instance_mut(|m| m.enable_semantics());
    assert!(CoreRegistry::set_bool_handle(
        &s.b.core_handle(),
        LayoutComponentBase::CLIP_PROPERTY_KEY.into(),
        true
    ));
    s.advance(0.0);
    assert_eq!(validate_semantic_geometry(&s.root.core_handle()), Ok(()));
    s.b.with_artboard_mut(|b| {
        let bounds = b.bounds();
        let mut path = b.local_path().unwrap();
        path.rewind();
        for _ in 0..5000 {
            path.add_rect(bounds, PathDirection::Clockwise);
        }
    });
    assert_eq!(
        validate_semantic_geometry(&s.root.core_handle()),
        Err(SemanticGeometryError::LimitExceeded)
    );
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    set_row_collapsed(&list, 1, true).unwrap();
    assert_eq!(validate_semantic_geometry(&s.root.core_handle()), Ok(()));
    set_row_collapsed(&list, 1, false).unwrap();
    assert_eq!(
        validate_semantic_geometry(&s.root.core_handle()),
        Err(SemanticGeometryError::LimitExceeded)
    );
    s.b.with_artboard_mut(|b| {
        let bounds = b.bounds();
        let mut path = b.local_path().unwrap();
        path.rewind();
        path.add_rect(bounds, PathDirection::Clockwise);
    });
    assert_eq!(validate_semantic_geometry(&s.root.core_handle()), Ok(()));
}

#[test]
fn semantics_enabled_after_collapse_publish_only_active_rows_and_wake_with_stable_ids() {
    let s = Scene::from_bytes(&fixture::semantic_fixture());
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    set_row_collapsed(&list, 1, true).unwrap();
    s.machine.with_instance_mut(|m| m.enable_semantics());
    let manager = s.machine.with_instance(|m| m.semantic_manager()).unwrap();
    let fields = || {
        let snapshot = manager.with_semantic_manager_mut(|m| m.snapshot().to_vec());
        assert_eq!(
            snapshot
                .iter()
                .map(|n| n.id)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            snapshot.len()
        );
        let mut ids: Vec<_> = snapshot
            .iter()
            .filter(|n| n.role == 6)
            .map(|n| n.id)
            .collect();
        ids.sort_unstable();
        ids
    };
    let active = fields();
    assert_eq!(active.len(), 1);
    set_row_collapsed(&list, 1, false).unwrap();
    let both = fields();
    assert_eq!(both.len(), 2);
    assert!(both.contains(&active[0]));
    set_row_collapsed(&list, 1, true).unwrap();
    assert_eq!(fields(), active);
    set_row_collapsed(&list, 1, false).unwrap();
    assert_eq!(fields(), both);
}
