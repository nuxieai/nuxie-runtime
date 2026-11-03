use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::live_move_fixture as fixture;
use nuxie_runtime::{
    live_move::*,
    source::{
        animation::nested_state_machine::NestedStateMachine,
        artboard_component_list::ArtboardComponentList, core::CoreType,
        generated::core_registry::CoreRegistry, nested_artboard::NestedArtboard, node::Node,
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
        Self::load(bytes, true)
    }
    fn load(bytes: &[u8], prepare: bool) -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("fixture imports");
        let root = file
            .with_file(|f| {
                f.artboard_named("Page")
                    .or_else(|| f.artboard_named("$nuxie.page"))
            })
            .unwrap();
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
        if root.with_artboard(|b| !b.find_all_handles::<records::LiveStep>().is_empty()) {
            empty_slot(&f).unwrap();
            let af = a.with_artboard(|b| b.find_all_handles::<NestedArtboard>())[1].clone();
            empty_slot(&af).unwrap();
        }
        let mut moves = LiveMoves::default();
        if prepare {
            moves.prepare(&root).unwrap();
        }
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
        set(&s.moves.flights(&s.root)[0].controls(), 62003, progress);
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
    assert!(move_live(&s.s, &s.s).is_err());
    assert_eq!(child(&s.s).unwrap().core_handle(), id);
    assert!(move_live(&s.f, &s.d).is_err());
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
    close(val(&property(&model, "prop"), 575), 10.0);
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
    set(&property(&vm(&s.a), "offset"), 575, 999.0);
    s.advance(0.0);
    close(val(&property(&model, "prop"), 575), 321.0);
    close(val(&count, 575), 7.0);
}
#[test]
fn a_target_first_two_way_prop_keeps_its_order_through_a_move_and_home() {
    use source::data_bind::{
        data_bind::{DataBind, SOURCE_TO_TARGET_FIRST, TWO_WAY},
        data_bind_context::DataBindContext,
    };

    let mut s = Scene::from_bytes(&fixture::live_page_fixture());
    // Author the prop binds in the Screen containers, as for slot replacement.
    // Both endpoints matter: arrival discards the destination's original unit
    // and retains its authored bind before deriving the moved unit's bind.
    for (row, slot, path) in [(&s.a, &s.s, 1), (&s.b, &s.d, 2)] {
        let prop = property(&vm(&child(slot).unwrap()), "prop");
        let container = row.core_handle().data_bind_container().unwrap();
        // Drop the list's Ref before removal; a for-scrutinee temporary would
        // otherwise keep the container borrowed throughout the loop.
        let binds = container.data_binds().to_vec();
        for bind in binds {
            if bind
                .with(|o| o.as_data_bind().and_then(DataBind::target))
                .flatten()
                == Some(prop.clone())
            {
                container.remove_and_delete_data_bind(&bind);
            }
        }
        let mut bind = DataBindContext::default();
        bind.base.base = DataBind::new(TWO_WAY, 575, 0);
        bind.decode_source_path_ids(&[path, 0]);
        let bind = slot.insert_sibling(bind).unwrap();
        bind.with_mut(|o| o.as_data_bind_mut().unwrap().set_target(Some(prop)));
        container.add_data_bind(bind);
    }

    let original = child(&s.s).unwrap();
    let prop = property(&vm(&original), "prop");
    let assert_order = |slot: &CoreHandle| {
        let unit = child(slot).unwrap();
        assert_eq!(unit.core_handle(), original.core_handle());
        let binds: Vec<_> = unit
            .core_handle()
            .data_bind_container()
            .unwrap()
            .data_binds()
            .to_vec()
            .into_iter()
            .filter(|bind| {
                bind.with(|o| {
                    o.as_data_bind().is_some_and(|b| {
                        b.is_instance_value_bind() && b.target() == Some(prop.clone())
                    })
                }) == Some(true)
            })
            .collect();
        assert_eq!(binds.len(), 1, "one derived prop binding on the moved unit");
        binds[0].with(|o| {
            let bind = o.as_data_bind().unwrap();
            assert_ne!(bind.base.flags() & TWO_WAY, 0, "prop remains two-way");
            assert_eq!(
                bind.base.flags() & SOURCE_TO_TARGET_FIRST,
                0,
                "prop remains target-first"
            );
        });
    };

    s.begin();
    s.advance(0.1);
    s.end();
    s.advance(0.0);
    assert_order(&s.d);
    set(&prop, 575, 321.0);
    s.advance(0.0);
    close(val(&property(&vm(&s.b), "offset"), 575), 321.0);

    s.moves.step_begin(&s.root, "A", Some("back")).unwrap();
    s.advance(0.1);
    s.moves.step_end(&s.root, "A", Some("back")).unwrap();
    s.advance(0.0);
    assert_order(&s.s);
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
        let hosts = row.with_artboard(|b| b.nested_artboards().to_vec());
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
    let inactive = rows["B"].with_artboard(|b| b.nested_artboards()[0].clone());
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

fn mount_layout_page(
    file: &RuntimeFileHandle,
) -> (
    RuntimeArtboardInstanceHandle,
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
) {
    let root = file.with_file(|f| f.artboard_named("LayoutPage")).unwrap();
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(model));
    machine.advance_and_apply(0.0);
    let screen = screens(&root).unwrap()["LayoutScreen"].clone();
    (root, screen, machine)
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
    let (_root, screen, machine) = mount_layout_page(&file);
    machine.advance_and_apply(0.0);
    let source = screen
        .with_artboard(|b| {
            b.find_handle::<source::nested_artboard_layout::NestedArtboardLayout>("source")
        })
        .unwrap();
    let destination = screen
        .with_artboard(|b| b.find_handle::<NestedArtboard>("destination"))
        .unwrap();
    let sibling = screen
        .with_artboard(|b| b.find_handle::<source::layout_component::LayoutComponent>("sibling"))
        .unwrap();
    let position = || {
        sibling
            .with_downcast::<source::layout_component::LayoutComponent, _>(|b| b.layout_x())
            .unwrap()
    };
    let before = position();
    assert!(before >= 149.9, "sibling before={before}");
    move_live(&source, &destination).unwrap();
    machine.advance_and_apply(0.0);
    close(position(), before);
    let bounds = source
        .with_downcast::<source::nested_artboard_layout::NestedArtboardLayout, _>(|s| {
            s.layout_bounds()
        })
        .unwrap();
    close(bounds.width(), 150.0);
    close(bounds.height(), 80.0);
    move_live(&destination, &source).unwrap();
    machine.advance_and_apply(0.0);
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
    set(&s.moves.flights(&s.root)[0].controls(), 62003, 0.5);
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
    // Scale mode changes only the flight transform and keeps its departure box.
    let flight_record =
        s.b.with_artboard(|b| b.find_all_handles::<live_move::records::LiveSlot>())
            .into_iter()
            .find(|h| {
                h.with_downcast::<live_move::records::LiveSlot, _>(|r| r.values.role == 1)
                    .unwrap()
            })
            .unwrap();
    flight_record.with_downcast_mut::<live_move::records::LiveSlot, _>(|r| r.values.size_mode = 0);
    s.moves.step_begin(&s.root, "A", Some("back")).unwrap();
    s.moves
        .with_flights(&s.root, |flights| match &mut flights[0] {
            Entry::One(flight) => flight.size_mode = SizeMode::Scale,
            Entry::Pair(_) => panic!("this step zooms nothing"),
        });
    set(&s.moves.flights(&s.root)[0].controls(), 62003, 0.5);
    s.advance(0.0);
    let f = slot_pose(&s.f).unwrap();
    close(f.size.x, 200.0);
    close(f.size.y, 120.0);
    // The slot keeps the blended scale; the stretch is the unit's own,
    // innermost factor.
    close(f.transform.decompose().scale_x(), 3.0);
    close(f.transform.decompose().scale_y(), 4.0);
    let stretch = child(&s.f).unwrap().with_artboard(|b| b.flight_stretch());
    close(stretch.x, 150.0 / 200.0);
    close(stretch.y, 100.0 / 120.0);
}

#[test]
fn timeline_controls_progress_and_path_offset() {
    let mut s = Scene::from_bytes(&fixture::control_fixture());
    s.begin();

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
    set(&s.moves.flights(&s.root)[0].controls(), 62003, 1.0);
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
    assert_eq!(s.moves.retained_history_len(&s.root), 0);
    assert!(
        validate_move(&s.d, &s.s).is_err(),
        "an arena lease does not make a removed row mounted"
    );
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
        assert!(!focus_manager.with_focus_manager(|m| m.has_primary_focus(&focus_node)));
        set_row_collapsed(&list, 1, false).unwrap();
        let same_node = unit.with_artboard(|b| b.find_all_handles::<FocusData>())[0]
            .with_downcast::<FocusData, _>(|f| f.existing_focus_node())
            .flatten()
            .unwrap();
        assert!(std::rc::Rc::ptr_eq(&focus_node, &same_node));
        // Native collapse blurs hidden fields. Wake retains the node, and the
        // application can focus that same field again without rebuilding it.
        if focused {
            focus_manager.with_focus_manager_mut(|m| m.set_focus(same_node));
        }
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

fn measure_advances(enabled: bool) -> u64 {
    hosted_scene::measure_walks(enabled).advance
}

#[test]
fn published_records_have_their_authored_parents() {
    use live_move::records::{LiveSlot, LiveStep, LiveStepUnit};
    let s = Scene::new();
    let steps = s.root.with_artboard(|b| b.find_all_handles::<LiveStep>());
    let units = s
        .root
        .with_artboard(|b| b.find_all_handles::<LiveStepUnit>());
    let types = s.root.with_artboard(|b| {
        b.objects()
            .iter()
            .map(|h| h.as_ref().map(|h| h.core_type()))
            .collect::<Vec<_>>()
    });
    assert_eq!(steps.len(), 2, "root object types: {types:?}");
    assert_eq!(units.len(), 2, "root object types: {types:?}");
    for unit in units {
        let parent = unit
            .with_downcast::<LiveStepUnit, _>(|u| u.parent())
            .flatten()
            .unwrap();
        assert!(steps.contains(&parent));
    }
    assert_eq!(
        s.b.with_artboard(|b| b.find_all_handles::<LiveSlot>().len()),
        2
    );
}

#[test]
fn native_keyframes_drive_live_slot_progress() {
    let mut s = Scene::from_bytes(&fixture::keyed_fixture());
    s.begin();
    s.advance(0.5);
    let control = s.moves.flights(&s.root)[0].controls();
    close(val(&control, 62003), 0.5);
    let a = slot_pose(&s.s).unwrap();
    let b = slot_pose(&s.d).unwrap();
    let f = slot_pose(&s.f).unwrap();
    close(
        f.transform[4],
        (a.transform[4] + b.transform[4]) * 0.5 + 7.0,
    );
    close(
        f.transform[5],
        (a.transform[5] + b.transform[5]) * 0.5 + 11.0,
    );
}

#[test]
fn two_artboard_versions_keep_identity_and_reverse_authored_draw_order() {
    let mut s = Scene::from_bytes(&fixture::two_version_fixture());
    let a = child(&s.s).unwrap().core_handle();
    let b = child(&s.d).unwrap().core_handle();
    assert_ne!(
        child(&s.s).unwrap().with_artboard(|b| b.name().to_owned()),
        child(&s.d).unwrap().with_artboard(|b| b.name().to_owned())
    );
    assert!(
        validate_move(&s.s, &s.d)
            .unwrap_err()
            .to_string()
            .contains("different unit artboards")
    );
    for (incoming, step, keys, colors) in [
        (
            "B",
            "t1",
            [11, 12],
            ["color=0xffff0000", "color=0xff0000ff"],
        ),
        (
            "A",
            "back",
            [13, 14],
            ["color=0xff0000ff", "color=0xffff0000"],
        ),
    ] {
        s.moves.step_begin(&s.root, incoming, Some(step)).unwrap();
        assert!(child(&s.s).is_none() && child(&s.d).is_none());
        assert_eq!(s.moves.flights(&s.root).len(), 2);
        for (flight, key) in s.moves.flights(&s.root).iter().zip(keys) {
            let values = flight
                .controls()
                .with_downcast::<live_move::records::LiveSlot, _>(|r| r.values.clone())
                .unwrap();
            assert_eq!(values.flight_key, key);
            assert_eq!(
                child(&flight.flight).unwrap().core_handle(),
                if values.unit_key == 1 {
                    a.clone()
                } else {
                    b.clone()
                }
            );
            set(&flight.controls(), 62003, 0.5);
        }
        s.advance(0.0);
        s.factory.borrow_mut().clear();
        let mut renderer = s.factory.borrow().make_renderer();
        s.root.draw(&mut renderer);
        let stream = s.factory.borrow().stream();
        let draws = stream
            .lines()
            .filter(|line| line.starts_with("drawPath "))
            .collect::<Vec<_>>();
        assert_eq!(draws.len(), 2, "exactly the two versions draw");
        for (draw, color) in draws.iter().zip(colors) {
            assert!(draw.contains(color), "{draw}");
        }
        s.moves.step_end(&s.root, incoming, Some(step)).unwrap();
        assert_eq!(child(&s.s).unwrap().core_handle(), a);
        assert_eq!(child(&s.d).unwrap().core_handle(), b);
        assert_eq!(s.moves.retained_history_len(&s.root), 0);
    }
}

#[test]
fn two_version_fade_multiplies_blended_opacity_at_each_progress() {
    let mut s = Scene::from_bytes(&fixture::two_version_fixture());
    set(&s.s, 18, 0.4);
    set(&s.d, 18, 0.8);
    s.advance(0.0);
    close(slot_pose(&s.s).unwrap().opacity, 0.4);
    close(slot_pose(&s.d).unwrap().opacity, 0.8);
    for (incoming, step, incoming_key) in [("B", "t1", 2), ("A", "back", 1)] {
        s.moves.step_begin(&s.root, incoming, Some(step)).unwrap();
        for t in [0.0_f32, 0.5, 1.0] {
            let a = slot_pose(&s.moves.flights(&s.root)[0].from).unwrap();
            let b = slot_pose(&s.moves.flights(&s.root)[0].to).unwrap();
            let expected = a.opacity + (b.opacity - a.opacity) * t;
            for flight in &s.moves.flights(&s.root) {
                let incoming = flight
                    .controls()
                    .with_downcast::<live_move::records::LiveSlot, _>(|r| {
                        r.values.unit_key == incoming_key
                    })
                    .unwrap();
                let fade = if incoming {
                    (t * 2.0).min(1.0)
                } else {
                    ((1.0 - t) * 2.0).min(1.0)
                };
                set(&flight.controls(), 62003, t);
                set(&flight.controls(), 62014, fade);
            }
            s.advance(0.0);
            for flight in &s.moves.flights(&s.root) {
                let opacity = expected * val(&flight.controls(), 62014);
                close(slot_pose(&flight.flight).unwrap().opacity, opacity);
                let paint = child(&flight.flight)
                    .unwrap()
                    .with_artboard(|b| b.find_all_handles::<source::shapes::paint::fill::Fill>())
                    [0]
                .clone();
                close(
                    paint
                        .with(|o| o.as_shape_paint().unwrap().render_opacity())
                        .unwrap(),
                    opacity,
                );
                let pose = slot_pose(&flight.flight).unwrap();
                close(
                    pose.transform[4],
                    a.transform[4] + (b.transform[4] - a.transform[4]) * t,
                );
            }
        }
        s.moves.step_end(&s.root, incoming, Some(step)).unwrap();
    }
}

#[test]
fn malformed_flight_plans_are_rejected_before_transport() {
    for (kind, error) in [
        (1, "unknown flightKey 999"),
        (2, "different unit artboards"),
        (3, "duplicate"),
    ] {
        let mut s = Scene::load(&fixture::invalid_plan_fixture(kind), false);
        let a = child(&s.s).unwrap().core_handle();
        let b = child(&s.d).unwrap().core_handle();
        assert!(
            s.moves
                .prepare(&s.root)
                .unwrap_err()
                .to_string()
                .contains(error)
        );
        assert_eq!(child(&s.s).unwrap().core_handle(), a);
        assert_eq!(child(&s.d).unwrap().core_handle(), b);
        assert!(s.moves.flights(&s.root).is_empty());
    }
}

#[test]
fn native_keyframes_drive_live_slot_fade() {
    let mut s = Scene::from_bytes(&fixture::keyed_fixture());
    s.begin();
    s.advance(0.5);
    let flight = &s.moves.flights(&s.root)[0];
    close(val(&flight.controls(), 62014), 0.5);
    let a = slot_pose(&s.s).unwrap();
    let b = slot_pose(&s.d).unwrap();
    close(
        slot_pose(&flight.flight).unwrap().opacity,
        (a.opacity + b.opacity) * 0.25,
    );
}

#[test]
fn removing_the_holder_brings_its_borrowed_instance_home_first() {
    let mut s = Scene::new();
    let unit = child(&s.s).unwrap().core_handle();
    s.begin();
    s.end();
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let a = list
        .with_downcast::<ArtboardComponentList, _>(|l| l.list_item(0))
        .flatten()
        .unwrap();
    ArtboardComponentList::update_list_occurrence(&list, &[a]);
    s.advance(0.0);
    assert_eq!(child(&s.s).unwrap().core_handle(), unit);
    assert_eq!(s.moves.retained_history_len(&s.root), 0);
}

#[test]
fn lua_private_state_survives_flight_landing_and_a_back_step() {
    use nuxie_scripting::vm::ScriptVm;
    use source::{
        assets::script_asset::ScriptAsset, scripted::scripted_drawable::ScriptedDrawable,
    };
    let mut s = Scene::from_bytes(&fixture::scripted_fixture());
    let source = r#"
local elapsed=0
local inits=0
function init(self, context) inits+=1 return true end
function advance(self, seconds) elapsed+=seconds return true end
function getElapsed() return math.floor(elapsed*1000+0.5) end
function getInits() return inits end
return function() return {init=init,advance=advance} end
"#;
    luaur_common::set_all_flags(true);
    let mut len = 0;
    let pointer = luaur_compiler::functions::luau_compile::luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut len,
    );
    assert!(!pointer.is_null() && len > 0);
    let bytecode = unsafe { std::slice::from_raw_parts(pointer.cast::<u8>(), len) }.to_vec();
    unsafe extern "C" {
        fn free(p: *mut std::ffi::c_void);
    }
    unsafe { free(pointer.cast()) };
    assert_ne!(
        bytecode[0],
        0,
        "{}",
        String::from_utf8_lossy(&bytecode[1..])
    );
    let payload = [vec![0], bytecode].concat();
    let vm = ScriptVm::new();
    let program = vm
        .register_protocol_script_with_factory("live-unit", &payload, &mut s.factory)
        .unwrap();
    let mut instance = vm
        .instantiate_registered_script_with_context(&program, None, Vec::new())
        .unwrap();
    assert!(instance.call_init(&mut NoopScriptHost).unwrap());
    let vm = RuntimeScriptingVmHandle::new(Box::new(vm));
    let unit = child(&s.s).unwrap();
    let drawable = unit
        .with_artboard(|b| b.find_handle::<ScriptedDrawable>("script"))
        .unwrap();
    let asset = drawable.insert_sibling(ScriptAsset::default()).unwrap();
    drawable
        .with_downcast_mut::<ScriptedDrawable, _>(|d| {
            d.scripted.set_asset(drawable.clone(), Some(asset));
            d.scripted.install_script_instance(instance, vm);
            d.scripted
                .set_implemented_methods(source::scripted::scripted_object::ADVANCES_BIT);
        })
        .unwrap();
    s.advance(0.1);
    assert_eq!(
        program
            .upstream_test_module_i32_getter("getElapsed")
            .unwrap(),
        100
    );
    s.begin();
    s.advance(0.2);
    s.end();
    s.advance(0.1);
    s.moves.step_begin(&s.root, "A", Some("back")).unwrap();
    s.advance(0.1);
    s.moves.step_end(&s.root, "A", Some("back")).unwrap();
    assert_eq!(
        program
            .upstream_test_module_i32_getter("getElapsed")
            .unwrap(),
        500
    );
    assert_eq!(
        program.upstream_test_module_i32_getter("getInits").unwrap(),
        1
    );
    assert_eq!(
        child(&s.s)
            .unwrap()
            .with_artboard(|b| b.find_handle::<ScriptedDrawable>("script"))
            .unwrap(),
        drawable
    );
}

#[test]
fn invalid_slot_parent_is_dropped_during_import() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::invalid_parent_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = file.with_file(|f| f.artboard_default()).unwrap();
    assert!(
        root.with_artboard(|b| b.find_all_handles::<live_move::records::LiveSlot>())
            .is_empty()
    );
}
#[test]
fn a_file_without_records_keeps_the_upstream_playback_and_mounted_slots() {
    let mut s = Scene::from_bytes(&fixture::ordinary_fixture(2));
    // Ordinary scenes keep all four mounted instances and running wrappers.
    // The stateless dispatcher must leave the native scene untouched.
    let a = child(&s.s).unwrap().core_handle();
    let b = child(&s.d).unwrap().core_handle();
    s.advance(0.3);
    assert_eq!(child(&s.s).unwrap().core_handle(), a);
    assert_eq!(child(&s.d).unwrap().core_handle(), b);
    assert!(s.moves.flights(&s.root).is_empty());
    for slot in [&s.s, &s.d] {
        let keyed = child(slot)
            .unwrap()
            .with_artboard(|b| b.find_handle::<Node>("keyed"))
            .unwrap();
        close(val(&keyed, 13), 30.0);
        close(val(&keyed, 14), 30.0);
    }
}
#[test]
fn scale_mode_is_read_from_the_published_flight_record() {
    let mut s = Scene::from_bytes(&fixture::scale_fixture());
    child(&s.d).unwrap().set_size(200.0, 120.0);
    s.advance(0.0);
    s.begin();
    assert_eq!(s.moves.flights(&s.root)[0].size_mode, SizeMode::Scale);
    set(&s.moves.flights(&s.root)[0].controls(), 62003, 0.5);
    s.advance(0.0);
    let f = slot_pose(&s.f).unwrap();
    close(f.size.x, 100.0);
    close(f.size.y, 80.0);
    close(f.transform.decompose().scale_x(), 1.0);
    close(f.transform.decompose().scale_y(), 1.0);
    let stretch = child(&s.f).unwrap().with_artboard(|b| b.flight_stretch());
    close(stretch.x, 1.5);
    close(stretch.y, 1.25);
}
#[test]
fn incompatible_wrappers_and_cross_player_moves_are_refused_without_taking_the_source() {
    let s = Scene::new();
    let other = Scene::new();
    let source = child(&s.s).unwrap().core_handle();
    assert!(move_live(&s.s, &other.d).is_err());
    let wrapper =
        s.d.with(|o| o.as_nested_artboard().unwrap().nested_animations()[0].clone())
            .unwrap();
    CoreRegistry::set_uint_handle(&wrapper, 198, 1234);
    assert!(move_live(&s.s, &s.d).is_err());
    assert_eq!(child(&s.s).unwrap().core_handle(), source);
}

#[test]
fn navigating_to_a_kept_row_comes_home_from_a_third_screen() {
    let mut s = Scene::from_bytes(&fixture::fixture(3));
    let original = child(&s.s).unwrap().core_handle();
    s.begin();
    s.end();
    s.moves.step_begin(&s.root, "R2", Some("third")).unwrap();
    s.advance(0.1);
    s.moves.step_end(&s.root, "R2", Some("third")).unwrap();
    assert!(child(&s.s).is_none());
    assert!(child(&s.d).is_none());
    s.moves.step_begin(&s.root, "B", None).unwrap();
    s.moves.step_end(&s.root, "B", None).unwrap();
    assert_eq!(child(&s.d).unwrap().core_handle(), original);
    s.moves.step_begin(&s.root, "A", None).unwrap();
    s.moves.step_end(&s.root, "A", None).unwrap();
    assert_eq!(child(&s.s).unwrap().core_handle(), original);
}
#[test]
fn another_players_membership_edits_do_not_rebuild_this_players_index() {
    let mut s = Scene::new();
    let other = Scene::new();
    s.advance(0.0);
    let list = other
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    set_row_collapsed(&list, 1, true).unwrap();
    hosted_scene::measure_walks(true);
    s.advance(0.01);
    assert_eq!(hosted_scene::measure_walks(false).index_build, 0);
}

#[test]
fn removing_a_third_holder_preserves_the_other_kept_rows_home() {
    let mut s = Scene::from_bytes(&fixture::fixture(3));
    let original = child(&s.s).unwrap().core_handle();
    s.begin();
    s.end();
    s.moves.step_begin(&s.root, "R2", Some("third")).unwrap();
    s.moves.step_end(&s.root, "R2", Some("third")).unwrap();
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let keep = list
        .with_downcast::<ArtboardComponentList, _>(|l| {
            vec![l.list_item(0).unwrap(), l.list_item(1).unwrap()]
        })
        .unwrap();
    ArtboardComponentList::update_list_occurrence(&list, &keep);
    assert_eq!(child(&s.s).unwrap().core_handle(), original);
    s.moves.step_begin(&s.root, "B", None).unwrap();
    s.moves.step_end(&s.root, "B", None).unwrap();
    assert_eq!(child(&s.d).unwrap().core_handle(), original);
}

#[path = "../src/test_support/paint_artboard.rs"]
mod paint_support;

#[test]
fn hosted_index_includes_existing_paint_artboard_occurrences_without_live_records() {
    let mut factory = PersistentFactory::new(nuxie_render_api::NullFactory);
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let source = paint_support::Scene::source(factory.clone(), 100.0, 50.0);
    let host = paint_support::Scene::host(factory, Some(source.artboard.clone()), 10.0, 10.0, 0);
    let root = host.instance();
    let nested = paint_support::inner(&paint_support::paint_of(&root));
    let mut index = nuxie_runtime::hosted_scene::HostedIndex::default();
    let active = index.active(&root);
    assert_eq!(active.len(), 2);
    assert!(active.contains(&nested.core_handle()));
}

#[test]
fn takeoff_discards_the_fresh_single_version_destination() {
    let mut s = Scene::new();
    let fresh = child(&s.d).unwrap().downgrade();
    s.begin();
    assert!(child(&s.s).is_none());
    assert!(
        child(&s.d).is_none(),
        "incoming fresh unit must leave at take-off"
    );
    assert!(fresh.upgrade().is_none(), "fresh playback must be released");
    let mut renderer = s.factory.borrow().make_renderer();
    s.factory.borrow_mut().clear();
    s.root.draw(&mut renderer);
    assert_eq!(
        s.factory
            .borrow()
            .stream()
            .lines()
            .filter(|line| line.starts_with("drawPath "))
            .count(),
        1
    );
}

#[test]
fn removing_a_flight_holder_returns_home_and_prunes_the_active_step() {
    let mut s = Scene::new();
    let original = child(&s.s).unwrap().core_handle();
    s.begin();
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let a = list
        .with_downcast::<ArtboardComponentList, _>(|l| l.list_item(0))
        .flatten()
        .unwrap();
    ArtboardComponentList::update_list_occurrence(&list, &[a]);
    s.advance(0.0);
    assert_eq!(child(&s.s).unwrap().core_handle(), original);
    assert!(
        s.moves.flights(&s.root).is_empty(),
        "removed flight slots must leave the controller"
    );
    assert_eq!(s.moves.retained_history_len(&s.root), 0);
    s.moves.step_begin(&s.root, "A", None).unwrap();
    s.moves.step_end(&s.root, "A", None).unwrap();
}

#[test]
fn following_two_flights_settles_their_shared_row_once_without_playback() {
    let mut s = Scene::from_bytes(&fixture::two_version_fixture());
    s.begin();
    s.advance(0.0);
    nuxie_runtime::hosted_scene::measure_walks(true);
    s.moves.after_advance(&s.root).unwrap();
    let visits = nuxie_runtime::hosted_scene::measure_walks(false);
    assert_eq!(visits.advance, 0);
    assert_eq!(
        visits.layout, 3,
        "one row pass updates its two moving units"
    );
}

#[test]
fn removing_the_incoming_screen_during_a_back_flight_returns_to_a_surviving_home() {
    let mut s = Scene::new();
    let original = child(&s.s).unwrap().core_handle();
    s.begin();
    s.end();
    s.moves.step_begin(&s.root, "A", Some("back")).unwrap();
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let b = list
        .with_downcast::<ArtboardComponentList, _>(|l| l.list_item(1))
        .flatten()
        .unwrap();
    ArtboardComponentList::update_list_occurrence(&list, &[b]);
    s.advance(0.0);
    assert_eq!(child(&s.d).unwrap().core_handle(), original);
    assert!(child(&s.f).is_none());
    assert!(s.moves.flights(&s.root).is_empty());
    assert_eq!(s.moves.retained_history_len(&s.root), 0);
    s.moves.step_begin(&s.root, "B", None).unwrap();
    s.moves.step_end(&s.root, "B", None).unwrap();
}

#[test]
fn two_flights_read_their_own_published_sizing_modes() {
    let mut s = Scene::from_bytes(&fixture::two_version_mixed_size_fixture());
    child(&s.d).unwrap().set_size(200.0, 120.0);
    s.advance(0.0);
    s.begin();
    for flight in &s.moves.flights(&s.root) {
        let unit_key = flight
            .controls()
            .with_downcast::<live_move::records::LiveSlot, _>(|r| r.values.unit_key)
            .unwrap();
        assert_eq!(
            flight.size_mode,
            if unit_key == 2 {
                SizeMode::Scale
            } else {
                SizeMode::Layout
            }
        );
        set(&flight.controls(), 62003, 0.5);
    }
    s.advance(0.0);
    for flight in &s.moves.flights(&s.root) {
        let pose = slot_pose(&flight.flight).unwrap();
        let stretch = child(&flight.flight)
            .unwrap()
            .with_artboard(|b| b.flight_stretch());
        if flight.size_mode == SizeMode::Scale {
            close(pose.size.x, 200.0);
            close(pose.size.y, 120.0);
            close(pose.transform.decompose().scale_x(), 1.0);
            close(pose.transform.decompose().scale_y(), 1.0);
            close(stretch.x, 0.75);
            close(stretch.y, 100.0 / 120.0);
        } else {
            close(pose.size.x, 150.0);
            close(pose.size.y, 100.0);
            close(stretch.x, 1.0);
            close(stretch.y, 1.0);
        }
    }
}

#[test]
fn a_loaned_incoming_version_returns_before_its_two_version_takeoff() {
    let mut s = Scene::from_bytes(&fixture::two_version_incoming_away_fixture());
    let incoming = child(&s.d).unwrap().core_handle();
    let third = screens(&s.root).unwrap()["R2"]
        .with_artboard(|b| b.find_all_handles::<NestedArtboard>())[0]
        .clone();
    move_live(&s.d, &third).unwrap();
    assert!(child(&s.d).is_none());
    assert_eq!(child(&third).unwrap().core_handle(), incoming);
    s.begin();
    assert!(child(&s.d).is_none() && child(&third).is_none());
    let flight_list = s.moves.flights(&s.root);
    let flight = flight_list
        .iter()
        .find(|f| {
            f.controls()
                .with_downcast::<live_move::records::LiveSlot, _>(|r| r.values.flight_key == 12)
                .unwrap()
        })
        .unwrap();
    assert_eq!(child(&flight.flight).unwrap().core_handle(), incoming);
    s.end();
    assert_eq!(child(&s.d).unwrap().core_handle(), incoming);
    assert!(child(&third).is_none());
}

#[test]
fn removing_a_two_version_landing_screen_settles_survivors_and_allows_the_next_step() {
    let mut s = Scene::from_bytes(&fixture::two_version_fixture());
    let incoming = child(&s.d).unwrap().core_handle();
    s.begin();
    assert_eq!(s.moves.flights(&s.root).len(), 2);
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let b = list
        .with_downcast::<ArtboardComponentList, _>(|l| l.list_item(1))
        .flatten()
        .unwrap();
    // A has no borrowed instance when removed. Both flight holders are on B,
    // but the outgoing version's landing slot was on A.
    ArtboardComponentList::update_list_occurrence(&list, &[b]);
    s.advance(0.0);
    assert!(
        s.moves.flights(&s.root).is_empty(),
        "a missing landing must cancel the whole active step"
    );
    // No slot keeps a flight's placement or a link to the flight that
    // carried its frame (UNIV-5635).
    for slot in
        s.b.with_artboard(|b| b.find_all_handles::<NestedArtboard>())
    {
        assert_eq!(slot_placement(&slot).unwrap(), (false, false));
    }
    assert_eq!(child(&s.d).unwrap().core_handle(), incoming);
    assert_eq!(s.moves.retained_history_len(&s.root), 0);
    s.moves.step_begin(&s.root, "B", None).unwrap();
    s.moves.step_end(&s.root, "B", None).unwrap();
}

#[test]
fn flight_drawn_opacity_matches_ends_under_translucent_ancestors_and_hosts() {
    for (ancestor, hosted) in [(0.5, 1.0), (1.0, 0.5), (0.5, 0.5)] {
        let mut s = Scene::new();
        let list = s
            .root
            .with_artboard(|a| a.find_handle::<ArtboardComponentList>("screens"))
            .unwrap();
        let card =
            s.a.with_artboard(|a| a.find_handle::<Node>("card"))
                .unwrap();
        let header =
            s.b.with_artboard(|a| a.find_handle::<Node>("header"))
                .unwrap();
        let layer =
            s.b.with_artboard(|a| a.find_handle::<Node>("flight"))
                .unwrap();
        set(&list, 18, hosted);
        set(&s.b.core_handle(), 18, ancestor);
        set(&card, 18, 0.4);
        set(&header, 18, 0.6);
        set(&layer, 18, 0.8);
        s.advance(0.0);
        s.advance(0.0);
        let paint_opacity = |host: &CoreHandle| {
            let paint = child(host)
                .unwrap()
                .with_artboard(|a| a.find_all_handles::<source::shapes::paint::fill::Fill>())[0]
                .clone();
            paint
                .with(|o| o.as_shape_paint().unwrap().render_opacity())
                .unwrap()
        };
        // The ordinary slots' native paints are the oracle. Artboard host
        // opacity is supplied by the host, so do not invent a second authored
        // artboard-opacity product on top of its rendering behavior.
        let expected = [paint_opacity(&s.s), paint_opacity(&s.d)];
        if ancestor == 0.5 && hosted == 1.0 {
            close(expected[0], 0.4);
            close(expected[1], 0.3);
        }
        s.begin();
        for (progress, opacity) in [(0.0, expected[0]), (1.0, expected[1])] {
            set(&s.moves.flights(&s.root)[0].controls(), 62003, progress);
            s.advance(0.0);
            close(slot_pose(&s.f).unwrap().opacity, opacity);
            assert!(
                (paint_opacity(&s.f) - opacity).abs() < 0.002,
                "flight at {progress}: ancestor={ancestor}, host={hosted}, paint={}, expected={opacity}",
                paint_opacity(&s.f)
            );
        }
        s.end();
        assert!(
            (paint_opacity(&s.d) - expected[1]).abs() < 0.002,
            "landing: ancestor={ancestor}, host={hosted}, paint={}, expected={}",
            paint_opacity(&s.d),
            expected[1]
        );
    }
}

#[test]
fn collapsed_rows_are_skipped_by_next_and_previous_focus_and_wake_with_the_same_nodes() {
    use source::focus_data::FocusData;
    // The existing device fixture has real fonts, so native focus callbacks
    // exercise text shaping as well as both traversal directions.
    let mut s = Scene::from_bytes(&live_move::device_fixture::device_fixture(2));
    let focus = |host: &CoreHandle| {
        let data = child(host)
            .unwrap()
            .with_artboard(|a| a.find_all_handles::<FocusData>())[0]
            .clone();
        data.with_downcast::<FocusData, _>(FocusData::existing_focus_node)
            .flatten()
            .unwrap()
    };
    let a = focus(&s.s);
    let b = focus(&s.d);
    let manager = child(&s.s)
        .unwrap()
        .with_artboard(|a| a.focus_manager())
        .unwrap();
    let list = s
        .root
        .with_artboard(|a| a.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let stops = |forward: bool| {
        manager.with_focus_manager_mut(|m| {
            m.clear_focus();
            let mut nodes = Vec::new();
            for _ in 0..10 {
                let advanced = if forward {
                    m.focus_next()
                } else {
                    m.focus_previous()
                };
                if !advanced {
                    return nodes;
                }
                nodes.push(m.primary_focus().unwrap());
            }
            panic!("focus traversal did not reach the root edge");
        })
    };
    assert!(
        stops(true).iter().any(|n| std::rc::Rc::ptr_eq(n, &b)),
        "expanded next reaches B"
    );
    for _ in 0..3 {
        set_row_collapsed(&list, 1, true).unwrap();
        for forward in [true, false] {
            let nodes = stops(forward);
            assert!(nodes.iter().any(|n| std::rc::Rc::ptr_eq(n, &a)));
            assert!(
                !nodes.iter().any(|n| std::rc::Rc::ptr_eq(n, &b)),
                "traversal forward={forward} must not enter a collapsed row"
            );
        }
        set_row_collapsed(&list, 1, false).unwrap();
        s.advance(0.0);
        assert!(
            std::rc::Rc::ptr_eq(&b, &focus(&s.d)),
            "wake preserves the field's identity"
        );
        for forward in [true, false] {
            assert!(stops(forward).iter().any(|n| std::rc::Rc::ptr_eq(n, &b)));
        }
    }
}

#[test]
fn projected_vm_lists_refresh_before_hosted_membership_changes() {
    for inside_frame in [false, true] {
        let mut s = Scene::new();
        let page = RuntimeOwnedViewModelHandle::from_native(s._file.clone(), vm(&s.root)).unwrap();
        let list = s
            .root
            .with_artboard(|a| a.find_handle::<ArtboardComponentList>("screens"))
            .unwrap();
        set_row_collapsed(&list, 1, true).unwrap();
        let sleeping = page.list_item_by_property_name_path("screens", 1).unwrap();
        let roots = [page.clone()];
        let mut frame = s.moves.frame_scope(&s.root, &roots).unwrap();
        assert!(
            !page
                .reachable_change_owner_snapshot()
                .unwrap()
                .iter()
                .any(|v| v.ptr_eq(&sleeping))
        );
        if !inside_frame {
            drop(frame.take());
        }
        let added = RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(s._file.clone(), 1).unwrap(),
        );
        let revision = hosted_scene::membership_revision(&s.root.core_handle());
        assert!(page.push_list_item_by_property_name_path("screens", &added));
        assert_eq!(
            hosted_scene::membership_revision(&s.root.core_handle()),
            revision,
            "list notification precedes hosted row reconciliation"
        );
        if !inside_frame {
            frame = s.moves.frame_scope(&s.root, &roots).unwrap();
        }
        let owners = page.reachable_change_owner_snapshot().unwrap();
        assert!(
            owners.iter().any(|v| v.ptr_eq(&added)),
            "new models must remain reachable to value-change notification filtering"
        );
        assert!(!owners.iter().any(|v| v.ptr_eq(&sleeping)));
        let capture = RuntimeViewModelChangeCapture::begin().unwrap();
        assert!(
            added
                .borrow_mut()
                .set_number_by_property_name("offset", 321.0)
        );
        let changes =
            RuntimeOwnedViewModelHandle::resolve_change_capture_across_with_owners(&roots, capture)
                .unwrap();
        assert!(changes.iter().any(|(owner, change)| owner.ptr_eq(&added)
            && change.value == RuntimeViewModelChangeValue::Number(321.0)));
        drop(frame);
        assert!(page.remove_list_item_by_property_name_path("screens", 2));
        let frame = s.moves.frame_scope(&s.root, &roots).unwrap();
        assert!(
            !page
                .reachable_change_owner_snapshot()
                .unwrap()
                .iter()
                .any(|v| v.ptr_eq(&added))
        );
        drop(frame);
        // A host can rebuild the projection while a list edit is provisional.
        // Rollback must invalidate that rebuilt cache as well as the first edit.
        let transaction = RuntimeOwnedViewModelGraphTransaction::begin(&roots, 4096).unwrap();
        assert!(page.push_list_item_by_property_name_path("screens", &added));
        let frame = s.moves.frame_scope(&s.root, &roots).unwrap();
        assert!(
            page.reachable_change_owner_snapshot()
                .unwrap()
                .iter()
                .any(|v| v.ptr_eq(&added))
        );
        drop(frame);
        drop(transaction);
        let frame = s.moves.frame_scope(&s.root, &roots).unwrap();
        assert!(
            !page
                .reachable_change_owner_snapshot()
                .unwrap()
                .iter()
                .any(|v| v.ptr_eq(&added))
        );
        drop(frame);
    }
}

#[test]
fn a_layout_flight_with_nonzero_placement_matches_both_endpoints() {
    use source::nested_artboard_layout::NestedArtboardLayout;
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::layout_flight_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let (root, screen, machine) = mount_layout_page(&file);
    machine.advance_and_apply(0.0);
    let source = screen
        .with_artboard(|a| a.find_handle::<NestedArtboardLayout>("source"))
        .unwrap();
    let destination = screen
        .with_artboard(|a| a.find_handle::<NestedArtboard>("destination"))
        .unwrap();
    let flight = screen
        .with_artboard(|a| a.find_handle::<NestedArtboardLayout>("flight"))
        .unwrap();
    set(&destination, 13, 260.0);
    set(&destination, 14, 25.0);
    machine.advance_and_apply(0.0);
    close(slot_pose(&flight).unwrap().transform[4], 170.0);
    let original = child(&source).unwrap().core_handle();
    empty_slot(&flight).unwrap();
    move_live(&source, &flight).unwrap();
    let follower = Flight::new(
        flight.clone(),
        source.clone(),
        destination.clone(),
        SizeMode::Layout,
    )
    .unwrap();
    let mut moves = LiveMoves::default();
    moves.with_flights(&root, |flights| flights.push(Entry::One(follower)));
    for (progress, endpoint) in [(0.0, &source), (1.0, &destination)] {
        set(&moves.flights(&root)[0].controls(), 62003, progress);
        machine.advance_and_apply(0.0);
        moves.after_advance(&root).unwrap();
        let actual = slot_pose(&flight).unwrap();
        let expected = slot_pose(endpoint).unwrap();
        for i in 0..6 {
            close(actual.transform[i], expected.transform[i]);
        }
        assert_eq!(child(&flight).unwrap().core_handle(), original);
    }
    moves.with_flights(&root, |flights| flights.clear());
    move_live(&flight, &destination).unwrap();
    machine.advance_and_apply(0.0);
    close(slot_pose(&destination).unwrap().transform[4], 260.0);
    close(slot_pose(&destination).unwrap().transform[5], 25.0);
    assert_eq!(child(&destination).unwrap().core_handle(), original);
}

#[test]
fn overlapping_step_units_are_rejected_before_any_slot_changes() {
    use live_move::records::{LiveStep, LiveStepUnit};
    let mut valid = Scene::load(&fixture::overlapping_plan_fixture(0), false);
    valid.begin();
    assert_eq!(
        valid.moves.flights(&valid.root).len(),
        4,
        "two independent version pairs are valid"
    );
    valid.end();
    for kind in [4, 5, 6] {
        let mut s = Scene::from_bytes(&fixture::overlapping_plan_fixture(0));
        let step = s
            .root
            .with_artboard(|a| a.find_all_handles::<LiveStep>())
            .into_iter()
            .find(|r| {
                r.with_downcast::<LiveStep, _>(|r| r.values.name == "t1")
                    .unwrap()
            })
            .unwrap();
        let member = s
            .root
            .with_artboard(|a| a.find_all_handles::<LiveStepUnit>())
            .into_iter()
            .find(|r| {
                r.with_downcast::<LiveStepUnit, _>(|r| {
                    r.parent().as_ref() == Some(&step) && r.values.outgoing_unit_key == 2
                })
                .unwrap()
            })
            .unwrap();
        member.with_downcast_mut::<LiveStepUnit, _>(|r| match kind {
            4 => {
                r.values.incoming_unit_key = 2;
                r.values.incoming_flight_key = 13;
            }
            5 => {
                r.values.outgoing_unit_key = 1;
                r.values.incoming_flight_key = 11;
                r.values.outgoing_flight_key = 11;
            }
            6 => {
                r.values.incoming_flight_key = 11;
            }
            _ => unreachable!(),
        });
        s.moves.reset_for_test(&s.root);
        let slots = [&s.a, &s.b]
            .into_iter()
            .flat_map(|row| row.with_artboard(|a| a.find_all_handles::<NestedArtboard>()))
            .collect::<Vec<_>>();
        let before = slots
            .iter()
            .map(|slot| child(slot).map(|v| v.core_handle()))
            .collect::<Vec<_>>();
        assert!(s.moves.step_begin(&s.root, "B", Some("t1")).is_err());
        assert_eq!(
            slots
                .iter()
                .map(|slot| child(slot).map(|v| v.core_handle()))
                .collect::<Vec<_>>(),
            before,
            "invalid plan {kind} must leave every mounted instance unchanged"
        );
        assert!(s.moves.flights(&s.root).is_empty());
        assert_eq!(s.moves.retained_history_len(&s.root), 0);
    }
}

#[test]
fn import_rejects_overlapping_step_unit_records() {
    for kind in [4, 5, 6] {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        assert!(
            File::import(
                &fixture::overlapping_plan_fixture(kind),
                RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
                None,
                None,
                None
            )
            .is_none(),
            "invalid plan {kind} must be rejected at import"
        );
    }
}

#[test]
fn waking_a_row_refreshes_inherited_opacity_and_world_transform() {
    fn draw(s: &mut Scene) -> String {
        s.factory.borrow_mut().clear();
        let mut renderer = s.factory.borrow().make_renderer();
        s.root.draw(&mut renderer);
        s.factory.borrow().canonical_recording().stream().to_owned()
    }
    for (opacity, x, y) in [(1.0, 70.0, 30.0), (0.5, 70.0, 30.0), (0.25, -12.0, 18.0)] {
        let (mut sleeping, mut control) = (Scene::new(), Scene::new());
        let list = |s: &Scene| {
            s.root
                .with_artboard(|a| a.find_handle::<ArtboardComponentList>("screens"))
                .unwrap()
        };
        let asleep_list = list(&sleeping);
        let unit = child(&sleeping.d).unwrap().core_handle();
        let before = draw(&mut control);
        set_row_collapsed(&asleep_list, 1, true).unwrap();
        for s in [&mut sleeping, &mut control] {
            let list = list(s);
            set(&list, 18, opacity);
            set(&list, 13, x);
            set(&list, 14, y);
            s.advance(0.0);
        }
        set_row_collapsed(&asleep_list, 1, false).unwrap();
        sleeping.advance(0.0);
        control.advance(0.0);
        assert_eq!(child(&sleeping.d).unwrap().core_handle(), unit);
        let paint = child(&sleeping.d)
            .unwrap()
            .with_artboard(|a| a.find_all_handles::<source::shapes::paint::fill::Fill>())[0]
            .clone();
        close(
            paint
                .with(|o| o.as_shape_paint().unwrap().render_opacity())
                .unwrap(),
            opacity,
        );
        let expected = draw(&mut control);
        assert_ne!(
            expected, before,
            "the never-collapsed control must reflect the host change"
        );
        assert_eq!(
            draw(&mut sleeping),
            expected,
            "wake must catch up to opacity {opacity} and host translation ({x}, {y})"
        );
    }
}

fn semantic_field_id(unit: &RuntimeArtboardInstanceHandle) -> u32 {
    use source::semantic::semantic_data::SemanticData;
    unit.with_artboard(|b| b.find_all_handles::<SemanticData>())
        .iter()
        .find_map(|h| {
            h.with_downcast::<SemanticData, _>(|d| d.existing_semantic_node())
                .flatten()
                .filter(|n| n.borrow().role == 6)
                .map(|n| n.borrow().id())
        })
        .unwrap()
}
#[test]
fn sleeping_row_cut_restores_semantics_round4() {
    let mut s = Scene::from_bytes(&fixture::semantic_fixture());
    s.machine.with_instance_mut(|m| m.enable_semantics());
    let manager = s.machine.with_instance(|m| m.semantic_manager()).unwrap();
    let unit = child(&s.s).unwrap();
    let id = semantic_field_id(&unit);
    let visible = || manager.with_semantic_manager_mut(|m| m.snapshot().iter().any(|n| n.id == id));
    assert!(visible());
    s.begin();
    s.end();
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    set_row_collapsed(&list, 1, true).unwrap();
    assert!(!visible());
    s.moves.step_begin(&s.root, "A", None).unwrap();
    s.moves.step_end(&s.root, "A", None).unwrap();
    assert_eq!(child(&s.s).unwrap().core_handle(), unit.core_handle());
    assert_eq!(semantic_field_id(&unit), id);
    assert!(
        visible(),
        "a cut out of a sleeping row must restore the field"
    );
}
#[test]
fn wake_under_a_collapsed_list_keeps_semantics_hidden_round4() {
    use source::component::ComponentOccurrenceHandle;
    let mut s = Scene::from_bytes(&fixture::semantic_fixture());
    s.machine.with_instance_mut(|m| m.enable_semantics());
    let manager = s.machine.with_instance(|m| m.semantic_manager()).unwrap();
    let unit = child(&s.d).unwrap();
    let id = semantic_field_id(&unit);
    let visible = || manager.with_semantic_manager_mut(|m| m.snapshot().iter().any(|n| n.id == id));
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    set_row_collapsed(&list, 1, true).unwrap();
    ComponentOccurrenceHandle::Authored(list.clone()).collapse(true);
    set_row_collapsed(&list, 1, false).unwrap();
    assert!(
        !visible(),
        "waking under a collapsed list must expose no semantic field"
    );
    ComponentOccurrenceHandle::Authored(list.clone()).collapse(false);
    s.advance(0.0);
    assert!(visible());
    assert_eq!(semantic_field_id(&unit), id);
    // The opposite order must also compose: list wake cannot wake a sleeping row.
    set_row_collapsed(&list, 1, true).unwrap();
    ComponentOccurrenceHandle::Authored(list.clone()).collapse(true);
    ComponentOccurrenceHandle::Authored(list.clone()).collapse(false);
    assert!(!visible());
    set_row_collapsed(&list, 1, false).unwrap();
    assert!(visible());
}

// Read actual renderer geometry and its transform stack, independently of the
// follower's pose helper. These fixtures only paint straight-edged rectangles.
fn drawn_bounds(s: &mut Scene) -> Vec<[f32; 4]> {
    use source::math::{mat2d::Mat2D, vec2d::Vec2D};
    s.factory.borrow_mut().clear();
    let mut renderer = s.factory.borrow().make_renderer();
    s.root.draw(&mut renderer);
    let recording = s.factory.borrow().canonical_recording();
    let mut matrix = Mat2D::identity();
    let mut stack = Vec::new();
    let mut bounds = Vec::new();
    for line in recording.stream().lines() {
        if line == "save" {
            stack.push(matrix);
        } else if line == "restore" {
            matrix = stack.pop().unwrap();
        } else if let Some(values) = line.strip_prefix("transform matrix=[") {
            let v: Vec<f32> = values
                .trim_end_matches(']')
                .split(',')
                .map(|v| v.parse().unwrap())
                .collect();
            matrix = matrix * Mat2D::new(v[0], v[1], v[2], v[3], v[4], v[5]);
        } else if line.starts_with("drawPath ") {
            let points = line
                .split("points=[")
                .nth(1)
                .unwrap()
                .split(']')
                .next()
                .unwrap();
            let mut b = [
                f32::INFINITY,
                f32::INFINITY,
                f32::NEG_INFINITY,
                f32::NEG_INFINITY,
            ];
            for point in points.split("),(") {
                let v: Vec<f32> = point
                    .trim_matches(['(', ')'])
                    .split(',')
                    .map(|v| v.parse().unwrap())
                    .collect();
                let p = matrix * Vec2D::new(v[0], v[1]);
                b[0] = b[0].min(p.x);
                b[1] = b[1].min(p.y);
                b[2] = b[2].max(p.x);
                b[3] = b[3].max(p.y);
            }
            bounds.push(b);
        }
    }
    assert!(stack.is_empty());
    bounds
}

#[test]
fn follow_path_rows_draw_the_same_bounds_at_flight_endpoints_round4() {
    let mut s = Scene::from_bytes(&fixture::follow_path_fixture());
    set(&s.a.core_handle(), 16, 1.5);
    set(&s.a.core_handle(), 17, 0.75);
    set(&s.b.core_handle(), 16, 0.65);
    set(&s.b.core_handle(), 17, 1.3);
    s.advance(0.0);
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let transforms: Vec<_> = (0..2)
        .map(|i| {
            list.with_downcast::<ArtboardComponentList, _>(|l| {
                l.draw_transform_for_index(i).unwrap()
            })
            .unwrap()
        })
        .collect();
    assert!(
        (transforms[0].decompose().rotation() - transforms[1].decompose().rotation()).abs() > 1.0,
        "the actual ListFollowPathConstraint must orient the rows differently"
    );
    set_row_collapsed(&list, 1, true).unwrap();
    let source = drawn_bounds(&mut s);
    assert_eq!(source.len(), 1);
    set_row_collapsed(&list, 1, false).unwrap();
    set_row_collapsed(&list, 0, true).unwrap();
    let destination = drawn_bounds(&mut s);
    assert_eq!(destination.len(), 1);
    set_row_collapsed(&list, 0, false).unwrap();
    s.advance(0.0);
    s.begin();
    for (progress, expected) in [(0.0, source[0]), (1.0, destination[0])] {
        let control =
            s.b.with_artboard(|b| b.find_all_handles::<records::LiveSlot>())
                .into_iter()
                .find(|h| {
                    h.with_downcast::<records::LiveSlot, _>(|r| r.parent().as_ref() == Some(&s.f))
                        .unwrap()
                })
                .unwrap();
        control.with_downcast_mut::<records::LiveSlot, _>(|r| r.values.progress = progress);
        s.moves.after_advance(&s.root).unwrap();
        let flight = drawn_bounds(&mut s);
        assert_eq!(flight.len(), 1);
        for (got, want) in flight[0].into_iter().zip(expected) {
            close(got, want);
        }
    }
    s.end();
    for (got, want) in drawn_bounds(&mut s)[0].into_iter().zip(destination[0]) {
        close(got, want);
    }
}

#[test]
fn one_wake_in_forty_rows_reports_native_dirt() {
    use std::{collections::HashMap, time::Instant};
    let mut s = Scene::from_bytes(&device_fixture::device_fixture(40));
    s.machine.with_instance_mut(|m| m.enable_semantics());
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    for i in 1..40 {
        set_row_collapsed(&list, i, true).unwrap();
    }
    s.advance(0.0);
    let mut index = hosted_scene::HostedIndex::default();
    let boards = index.retained(&s.root).to_vec();
    let snapshot = || -> HashMap<CoreHandle, u16> {
        boards
            .iter()
            .flat_map(|b| {
                b.with_downcast::<Artboard, _>(|b| {
                    b.objects().iter().flatten().cloned().collect::<Vec<_>>()
                })
                .unwrap()
            })
            .filter_map(|h| {
                h.with(|o| o.as_component().map(|c| c.dirt().0))
                    .flatten()
                    .map(|d| (h, d))
            })
            .collect()
    };
    let before = snapshot();
    let start = Instant::now();
    set_row_collapsed(&list, 1, false).unwrap();
    let wake = start.elapsed();
    let after = snapshot();
    let mut changed = 0;
    let mut added = 0;
    let mut removed = 0;
    let mut per_board = Vec::new();
    for board in &boards {
        let (name, objects, root_collapsed) = board
            .with_downcast::<Artboard, _>(|b| {
                (
                    b.name().to_owned(),
                    b.objects().iter().flatten().cloned().collect::<Vec<_>>(),
                    b.is_collapsed(),
                )
            })
            .unwrap();
        assert!(
            !root_collapsed,
            "upstream never collapses the artboard root"
        );
        let count = objects
            .iter()
            .filter(|h| before.get(h) != after.get(h))
            .count();
        if count > 0 {
            per_board.push((name, count));
        }
    }
    for (handle, old) in &before {
        let new = after[handle];
        if new != *old {
            changed += 1;
            added |= new & !old;
            removed |= old & !new;
        }
    }
    // Every other sleeping row, including its hosted instances, is untouched.
    let rows = screens(&s.root).unwrap();
    for i in 2..40 {
        let row = &rows[&format!("R{i}")];
        let descendants = hosted_scene::HostedIndex::default().retained(row).to_vec();
        for board in descendants {
            for object in board
                .with_downcast::<Artboard, _>(|b| {
                    b.objects().iter().flatten().cloned().collect::<Vec<_>>()
                })
                .unwrap()
            {
                assert_eq!(
                    before.get(&object),
                    after.get(&object),
                    "wake touched another sleeping row {i}"
                );
            }
        }
    }
    assert!(changed > 0);
    assert_ne!(
        removed & source::component_dirt::ComponentDirt::COLLAPSED.0,
        0
    );
    let start = Instant::now();
    s.advance(0.0);
    let settle = start.elapsed();
    println!(
        "WAKE40 wake_ms={:.6} settle_ms={:.6} changed_components={changed} added_dirt=0x{added:04x} removed_dirt=0x{removed:04x} per_board={per_board:?}",
        wake.as_secs_f64() * 1000.0,
        settle.as_secs_f64() * 1000.0
    );
}

#[test]
fn nested_list_descendants_follow_the_same_native_collapse_path() {
    let s = Scene::from_bytes(&fixture::semantic_nested_list_fixture());
    let unit = child(&s.s).unwrap();
    let inner = unit
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("inner"))
        .unwrap();
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let item = list
        .with_downcast::<ArtboardComponentList, _>(|l| l.list_item(1))
        .flatten()
        .unwrap();
    ArtboardComponentList::update_list_occurrence(&inner, &[item]);
    let row = inner
        .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(0))
        .flatten()
        .unwrap();
    let hosts = row.with_artboard(|b| b.nested_artboards().to_vec());
    let nested_unit = child(&hosts[0]).unwrap();
    s.machine.with_instance_mut(|m| m.enable_semantics());
    let manager = s.machine.with_instance(|m| m.semantic_manager()).unwrap();
    let id = semantic_field_id(&nested_unit);
    let visible = || manager.with_semantic_manager_mut(|m| m.snapshot().iter().any(|n| n.id == id));
    assert!(visible());
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    set_row_collapsed(&list, 0, true).unwrap();
    assert!(!visible());
    set_row_collapsed(&list, 0, false).unwrap();
    assert!(
        visible(),
        "wake must restore a unit's nested list descendants"
    );
    // Ordinary nested content participates in its enclosing Screen's native
    // collapse. Only Page.screens supports live row sleep or unit transport.
    assert!(set_row_collapsed(&inner, 0, true).is_err());
    assert!(empty_slot(&hosts[1]).is_err());
    set_row_collapsed(&list, 0, true).unwrap();
    set_row_collapsed(&list, 0, false).unwrap();
    assert!(visible());
    empty_slot(&s.d).unwrap();
    move_live(&s.s, &s.d).unwrap();
    set_row_collapsed(&list, 1, true).unwrap();
    assert!(!visible());
    move_live(&s.d, &s.s).unwrap();
    assert!(visible());
    assert_eq!(semantic_field_id(&nested_unit), id);
}

#[test]
fn ordinary_nested_row_cannot_wake_a_sleeping_screen_round9() {
    let s = Scene::from_bytes(&fixture::semantic_nested_list_fixture());
    let unit = child(&s.s).unwrap();
    let inner = unit
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("inner"))
        .unwrap();
    let outer = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let item = outer
        .with_downcast::<ArtboardComponentList, _>(|l| l.list_item(1))
        .flatten()
        .unwrap();
    ArtboardComponentList::update_list_occurrence(&inner, &[item]);
    let row = inner
        .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(0))
        .flatten()
        .unwrap();
    let slots = row.with_artboard(|b| b.nested_artboards().to_vec());
    let nested = child(&slots[0]).unwrap();
    s.machine.advance_and_apply(0.0);
    s.machine.with_instance_mut(|m| m.enable_semantics());
    let manager = s.machine.with_instance(|m| m.semantic_manager()).unwrap();
    let id = semantic_field_id(&nested);
    let visible = || manager.with_semantic_manager_mut(|m| m.snapshot().iter().any(|n| n.id == id));
    let prop = property(&vm(&nested), "prop");
    let original = val(&prop, 575);
    assert!(visible());
    assert!(set_row_collapsed(&inner, 0, true).is_err());
    set_row_collapsed(&outer, 0, true).unwrap();
    assert!(
        !inner.with(|o| o.component_is_collapsed()).unwrap(),
        "the local bit does not carry hosted ancestry"
    );
    set(&property(&vm(&row), "offset"), 575, 777.0);
    assert!(set_row_collapsed(&inner, 0, false).is_err());
    assert!(
        !visible(),
        "an ordinary nested row cannot expose an asleep Screen's field"
    );
    close(val(&prop, 575), original);
    set_row_collapsed(&outer, 0, false).unwrap();
    s.machine.advance_and_apply(0.0);
    assert!(visible());
    assert_eq!(semantic_field_id(&nested), id);
    close(val(&prop, 575), 777.0);
}

#[test]
fn ordinary_ten_thousand_row_append_does_no_live_move_work_round5() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::ordinary_list_fixture(10_000),
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
    machine.with_instance_mut(|m| m.bind_view_model_instance(model.clone()));
    machine.advance_and_apply(0.0);
    let page = RuntimeOwnedViewModelHandle::from_native(file, model).unwrap();
    let row = page.list_item_by_property_name_path("screens", 0).unwrap();
    let list = root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    assert_eq!(
        list.with_downcast::<ArtboardComponentList, _>(|l| l.artboard_count()),
        Some(10_000)
    );
    hosted_scene::measure_walks(true);
    assert!(page.push_list_item_by_property_name_path("screens", &row));
    machine.advance_and_apply(0.0);
    let visits = hosted_scene::measure_walks(false);
    assert_eq!(
        list.with_downcast::<ArtboardComponentList, _>(|l| l.artboard_count()),
        Some(10_001)
    );
    assert_eq!(
        visits.loan_row, 0,
        "an ordinary append must skip the live-move unmount hook's row work"
    );
}

#[test]
fn record_free_frames_scan_capability_once_per_occurrence() {
    let before = hosted_scene::bookkeeping_counts();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::ordinary_fixture(8),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(file.with_file(|f| f.artboard_count()), 10);
    hosted_scene::measure_walks(true);
    // Play only the small unit; the Page and eight Screens remain unused.
    let root = file.with_file(|f| f.artboard_named("mascot")).unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    for _ in 0..128 {
        // Fresh adapters must reuse the decision stored on the occurrence.
        let mut moves = LiveMoves::default();
        assert!(moves.frame_scope(&root, &[]).unwrap().is_none());
        machine.advance_and_apply(1.0 / 60.0);
        assert!(!moves.after_advance(&root).unwrap());
        // Media queries use this same capability entry point.
        assert!(!hosted_scene::enabled(&root));
    }
    let walks = hosted_scene::measure_walks(false);
    assert_eq!(
        walks.capability_scan, 1,
        "one scan across all frames and queries"
    );
    assert_eq!(hosted_scene::bookkeeping_counts(), before);
}

#[test]
fn record_free_create_and_free_has_no_bookkeeping_round6() {
    let before = hosted_scene::bookkeeping_counts();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::ordinary_list_fixture(2),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    for iteration in 0..128 {
        let (page, row) = if iteration % 2 == 0 {
            (
                file.with_file(|f| f.artboard_named("Page")).unwrap(),
                file.with_file(|f| f.artboard_named("Row")).unwrap(),
            )
        } else {
            let page = file.with_file(|f| f.artboard_named_source("Page")).unwrap();
            let row = file.with_file(|f| f.artboard_named_source("Row")).unwrap();
            (
                Artboard::instance_from_handle(&page).unwrap(),
                Artboard::instance_from_handle(&row).unwrap(),
            )
        };
        let model = file
            .with_file(|f| f.create_default_view_model_instance_for_artboard(page.core_handle()))
            .unwrap();
        let machine = page.default_state_machine_handle().unwrap();
        machine.with_instance_mut(|m| m.bind_view_model_instance(model));
        machine.advance_and_apply(0.0);
        assert_eq!(hosted_scene::bookkeeping_counts(), before);
        let mut moves = LiveMoves::default();
        moves.prepare(&page).unwrap();
        assert!(moves.frame_scope(&page, &[]).unwrap().is_none());
        let mut index = hosted_scene::HostedIndex::default();
        assert_eq!(index.active(&page).len(), 3);
        assert_eq!(index.retained(&page).len(), 3);
        let list = page
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
            .unwrap();
        assert!(set_row_collapsed(&list, 0, true).is_err());
        assert_eq!(hosted_scene::bookkeeping_counts(), before);
        drop((machine, row, page));
        assert_eq!(hosted_scene::bookkeeping_counts(), before);
    }
    drop(file);
    assert_eq!(hosted_scene::bookkeeping_counts(), before);
}
#[test]
fn live_occurrence_builds_and_reclaims_bookkeeping_round6() {
    let before = hosted_scene::bookkeeping_counts();
    for _ in 0..8 {
        let s = Scene::new();
        assert!(hosted_scene::bookkeeping_counts().revisions > before.revisions);
        assert!(hosted_scene::bookkeeping_counts().indexed_handles > before.indexed_handles);
        drop(s);
        assert_eq!(hosted_scene::bookkeeping_counts(), before);
    }
}
#[test]
fn dropping_an_in_flight_occurrence_reclaims_all_bookkeeping_round6() {
    let before = hosted_scene::bookkeeping_counts();
    let mut s = Scene::new();
    s.begin();
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    set_row_collapsed(&list, 0, true).unwrap();
    let model = RuntimeOwnedViewModelHandle::from_native(s._file.clone(), vm(&s.root)).unwrap();
    let scope = s
        .moves
        .frame_scope(&s.root, std::slice::from_ref(&model))
        .unwrap()
        .unwrap();
    let during = hosted_scene::bookkeeping_counts();
    assert!(during.loan_lists > before.loan_lists);
    assert!(during.projections > before.projections);
    assert!(during.row_indices > before.row_indices);
    assert!(during.slot_states > before.slot_states);
    assert_eq!(during.active_frames, 1);
    let weak = s.root.downgrade();
    drop(s);
    assert!(weak.upgrade().is_none());
    assert_eq!(hosted_scene::bookkeeping_counts(), before);
    drop((scope, model));
}

#[test]
fn child_dispatch_cannot_retarget_its_owning_scene_plan_round6() {
    let mut s = Scene::new();
    let before = hosted_scene::bookkeeping_counts();
    let mut adapter = LiveMoves::default();
    assert!(adapter.frame_scope(&s.a, &[]).unwrap().is_none());
    assert_eq!(hosted_scene::bookkeeping_counts(), before);
    s.begin();
    s.end();
}

#[test]
fn crossed_two_version_flights_release_rows_and_units_round7() {
    let before = hosted_scene::bookkeeping_counts();
    let mut s = Scene::load(&fixture::two_version_fixture(), false);
    let mapping = s
        .root
        .with_artboard(|b| b.find_all_handles::<records::LiveStepUnit>())[0]
        .clone();
    mapping.with_downcast_mut::<records::LiveStepUnit, _>(|r| r.values.incoming_flight_key = 2);
    s.moves.prepare(&s.root).unwrap();
    let rows = [s.a.downgrade(), s.b.downgrade()];
    let units = [
        child(&s.s).unwrap().downgrade(),
        child(&s.d).unwrap().downgrade(),
    ];
    s.begin();
    let flights = s.moves.flights(&s.root);
    assert_eq!(flights.len(), 2);
    // A flies in B, and B flies in A. Both units have explicit row-owned VMs.
    for (flight, expected) in flights.iter().zip([s.b.core_handle(), s.a.core_handle()]) {
        assert_eq!(
            flight
                .flight
                .with(|o| o.as_component().unwrap().artboard_handle())
                .flatten()
                .unwrap(),
            expected
        );
    }
    drop(flights);
    drop(s);
    for row in rows {
        assert!(row.upgrade().is_none(), "crossed flight retained a row");
    }
    for unit in units {
        assert!(unit.upgrade().is_none(), "crossed flight retained a unit");
    }
    assert_eq!(hosted_scene::bookkeeping_counts(), before);
}

#[test]
fn checkpoint_rollback_retains_and_restores_replaced_rows_round7() {
    for begin in [
        RuntimeOwnedViewModelGraphTransaction::begin,
        RuntimeOwnedViewModelGraphTransaction::begin_live,
    ] {
        let s = Scene::new();
        let root_model =
            RuntimeOwnedViewModelHandle::from_native(s._file.clone(), vm(&s.root)).unwrap();
        let unit = child(&s.s).unwrap();
        let unit_model =
            RuntimeOwnedViewModelHandle::from_native(s._file.clone(), vm(&unit)).unwrap();
        let roots = [root_model, unit_model];
        let old_property = property(&vm(&unit), "count");
        let previous = val(&old_property, 575);
        let weak = s.a.downgrade();
        let transaction = begin(&roots, 4096).unwrap();
        set(&old_property, 575, previous + 20.0);
        assert!(roots[0].remove_list_item_by_property_name_path("screens", 0));
        let replacement = s
            ._file
            .with_file(|f| f.create_default_view_model_instance_for_artboard(s.root.core_handle()))
            .unwrap();
        s.machine
            .with_instance_mut(|m| m.bind_view_model_instance(replacement));
        s.machine.advance_and_apply(0.0);
        drop(unit);
        let Scene { a, b, .. } = s;
        drop((a, b));
        let retained = weak.upgrade();
        let rollback = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(transaction)));
        assert!(rollback.is_ok(), "rollback lost captured property storage");
        let retained = retained.expect("checkpoint must retain the replaced row");
        close(val(&old_property, 575), previous);
        assert_eq!(
            roots[0].list_item_count_by_property_name_path("screens"),
            Some(2)
        );
        drop(retained);
        assert!(weak.upgrade().is_none());
    }
}

#[test]
fn import_validation_misplaced_page_slot_is_preserved_and_ignored() {
    let before = hosted_scene::bookkeeping_counts();
    let s = Scene::from_bytes(&fixture::root_owned_vm_fixture());
    let source = s
        .root
        .with_artboard(|b| b.find_handle::<NestedArtboard>("root unit"))
        .unwrap();
    assert!(
        source
            .with(|o| o
                .as_nested_artboard()
                .unwrap()
                .children()
                .iter()
                .any(|h| h.is_type_of(records::LiveSlot::TYPE_KEY) && h.is_alive()))
            .unwrap()
    );
    let unit = child(&source).unwrap();
    let weak = s.root.downgrade();
    assert!(move_live(&source, &s.f).is_err());
    assert_eq!(child(&source).unwrap().core_handle(), unit.core_handle());
    assert!(child(&s.f).is_none());
    drop(unit);
    drop(s);
    assert!(weak.upgrade().is_none());
    assert_eq!(hosted_scene::bookkeeping_counts(), before);
}

#[test]
fn fresh_page_rebind_releases_retired_flight_rows_round8() {
    let mut s = Scene::new();
    for _ in 0..4 {
        let rows = [s.a.downgrade(), s.b.downgrade()];
        let units = [
            child(&s.s).unwrap().downgrade(),
            child(&s.d).unwrap().downgrade(),
        ];
        s.begin();
        let replacement = s
            ._file
            .with_file(|f| f.create_default_view_model_instance_for_artboard(s.root.core_handle()))
            .unwrap();
        s.machine
            .with_instance_mut(|m| m.bind_view_model_instance(replacement));
        s.machine.advance_and_apply(0.0);
        let mounted = screens(&s.root).unwrap();
        s.a = mounted["A"].clone();
        s.b = mounted["B"].clone();
        s.s =
            s.a.with_artboard(|b| b.find_all_handles::<NestedArtboard>())[0].clone();
        let slots =
            s.b.with_artboard(|b| b.find_all_handles::<NestedArtboard>());
        s.d = slots[0].clone();
        s.f = slots[1].clone();
        s.moves.prepare(&s.root).unwrap();
        assert!(
            rows.iter().all(|r| r.upgrade().is_none()),
            "retired row retained by a scene lease"
        );
        assert!(
            units.iter().all(|u| u.upgrade().is_none()),
            "retired unit retained after rebind"
        );
        assert_eq!(hosted_scene::bookkeeping_counts().unit_vm_storage, 0);
    }
}

fn bound_replace(s: &Scene, slot: &CoreHandle, name: &str) {
    use nuxie_runtime::source::viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard;
    let value = slot
        .insert_sibling(ViewModelInstanceArtboard::default())
        .unwrap();
    let asset = s
        ._file
        .with_file(|f| f.bindable_artboard_named(name))
        .unwrap();
    value.with_downcast_mut::<ViewModelInstanceArtboard, _>(|v| {
        v.set_asset(Some(asset));
    });
    NestedArtboard::update_artboard_occurrence(slot, Some(value));
}

#[test]
fn bound_slot_replacement_rejects_current_incompatible_definition_round8() {
    let s = Scene::new();
    move_live(&s.s, &s.d).unwrap();
    move_live(&s.d, &s.s).unwrap();
    bound_replace(&s, &s.s, "mascot");
    bound_replace(&s, &s.d, "A");
    let original = child(&s.s).unwrap();
    let destination = child(&s.d).unwrap();
    let error = validate_move(&s.s, &s.d)
        .expect_err("replaced artboard must invalidate the saved definition");
    assert!(
        error.to_string().contains("different unit artboards"),
        "{error}"
    );
    assert_eq!(child(&s.s).unwrap().core_handle(), original.core_handle());
    assert_eq!(
        child(&s.d).unwrap().core_handle(),
        destination.core_handle()
    );
}

#[test]
fn bound_slot_replacement_adopts_fresh_prop_templates_round8() {
    let s = Scene::new();
    move_live(&s.s, &s.d).unwrap();
    move_live(&s.d, &s.s).unwrap();
    bound_replace(&s, &s.s, "mascot");
    bound_replace(&s, &s.d, "mascot");
    let model = vm(&child(&s.d).unwrap());
    use nuxie_runtime::source::data_bind::{
        data_bind::DataBind, data_bind_context::DataBindContext,
    };
    let mut fresh = DataBindContext::default();
    fresh.base.base = DataBind::new(0, 575, 0);
    fresh.decode_source_path_ids(&[2, 0]);
    let fresh = s.d.insert_sibling(fresh).unwrap();
    fresh.with_mut(|o| {
        o.as_data_bind_mut()
            .unwrap()
            .set_target(Some(property(&model, "count")))
    });
    s.b.with_artboard_mut(|b| b.add_data_bind(fresh));
    let original = child(&s.s).unwrap();
    move_live(&s.s, &s.d).unwrap();
    original.update_data_binds(true);
    close(val(&property(&vm(&original), "count"), 575), 200.0);
}

#[cfg(feature = "tools")]
#[test]
fn tools_replaced_live_page_uses_its_definition_records_round8() {
    use nuxie_runtime::source::{core::binary_reader::BinaryReader, runtime_header::RuntimeHeader};
    let bytes = fixture::fixture(2);
    let s = Scene::from_bytes(&bytes);
    let mut reader = BinaryReader::new(&bytes);
    let mut header = RuntimeHeader::default();
    assert!(RuntimeHeader::read(&mut reader, &mut header));
    let offset = bytes.len() - reader.position().len();
    let range = s._file.with_file(|f| f.artboard_byte_range(0));
    assert_eq!(
        s._file.with_file_mut(
            |f| f.replace_artboard(0, &bytes[offset + range.start..offset + range.end])
        ),
        ImportResult::Success
    );
    let root = s._file.with_file(|f| f.artboard_named("Page")).unwrap();
    let model = s
        ._file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(model));
    machine.advance_and_apply(0.0);
    let mut moves = LiveMoves::default();
    moves.step_begin(&root, "B", Some("t1")).unwrap();
    moves.step_end(&root, "B", Some("t1")).unwrap();
}

#[test]
fn retained_row_binding_count_stays_flat_after_replacement_and_transport_round9() {
    use source::{
        data_bind::{
            converters::data_converter_to_number::DataConverterToNumber, data_bind::DataBind,
            data_bind_context::DataBindContext,
        },
        generated::data_bind::{
            converters::data_converter_base::DataConverterBase, data_bind_base::DataBindBase,
        },
        viewmodel::viewmodel_instance::ViewModelInstance,
    };
    let s = Scene::new();
    let cycle = || {
        for (row, slot, path) in [(&s.a, &s.s, 1), (&s.b, &s.d, 2)] {
            bound_replace(&s, slot, "mascot");
            let model = vm(&child(slot).unwrap());
            let prop = property(&model, "prop");
            let container = row.core_handle().data_bind_container().unwrap();
            let binds = container.data_binds().to_vec();
            for bind in binds {
                if bind
                    .with(|o| o.as_data_bind().and_then(DataBind::target))
                    .flatten()
                    == Some(prop.clone())
                {
                    container.remove_and_delete_data_bind(&bind);
                }
            }
            // Native property replacement retires the preceding authored binds
            // through their real owner, before supplying this replacement's bind.
            model.with_downcast_mut::<ViewModelInstance, _>(|m| {
                assert!(m.remove_value(1));
                m.add_value(prop.clone());
            });
            let mut fresh = DataBindContext::default();
            fresh.base.base = DataBind::new(0, 575, 0);
            fresh.decode_source_path_ids(&[path, 0]);
            let fresh = slot.insert_sibling(fresh).unwrap();
            let converter = slot
                .insert_sibling(DataConverterToNumber::default())
                .unwrap();
            fresh.with_mut(|o| {
                let bind = o.as_data_bind_mut().unwrap();
                bind.set_target(Some(prop));
                bind.set_converter(Some(converter));
            });
            row.with_artboard_mut(|b| b.add_data_bind(fresh));
        }
        move_live(&s.s, &s.d).unwrap();
        move_live(&s.d, &s.s).unwrap();
        let unit = child(&s.s).unwrap();
        unit.update_data_binds(true);
        close(val(&property(&vm(&unit), "prop"), 575), 10.0);
    };
    let counts = || {
        [&s.a, &s.b].map(|row| {
            let arena = row.core_handle().retain_arena().unwrap();
            (
                arena.count_instances(DataBindBase::TYPE_KEY),
                arena.count_instances(DataConverterBase::TYPE_KEY),
            )
        })
    };
    cycle();
    let before = counts();
    eprintln!("retained row binding/converter counts: {before:?}");
    assert!(
        before
            .iter()
            .all(|(binds, converters)| *binds > 0 && *converters > 0)
    );
    for _ in 0..8 {
        cycle();
        assert_eq!(
            counts(),
            before,
            "retained row leaked prop bindings or converters"
        );
    }
}

#[test]
fn mutable_list_collapse_reconciles_after_its_borrow_round9() {
    let s = Scene::from_bytes(&fixture::semantic_fixture());
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    set_row_collapsed(&list, 1, true).unwrap();
    list.with_downcast_mut::<ArtboardComponentList, _>(|l| l.collapse(true));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        list.with_downcast_mut::<ArtboardComponentList, _>(|l| l.collapse(false))
    }));
    assert!(
        result.is_ok(),
        "mutable list collapse borrowed its own host recursively"
    );
    for (slot, expected) in [(&s.s, false), (&s.d, true)] {
        assert_eq!(slot.with(|o| o.component_is_collapsed()), Some(expected));
    }
}

#[test]
fn import_validation_preserves_slots_regardless_of_default_rows() {
    for empty in [true, false] {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &fixture::endpoint_shape_fixture(empty),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        for name in ["A", "B", "R2", "R3", "R4"] {
            let board = file.with_file(|f| f.artboard_named(name)).unwrap();
            let has_slots =
                board.with_artboard(|b| !b.find_all_handles::<records::LiveSlot>().is_empty());
            assert!(has_slots, "{name}, empty defaults: {empty}");
        }
        if !empty {
            let root = fresh_page(&file);
            let rows = screens(&root).unwrap();
            let slots = rows["R4"].with_artboard(|b| b.find_all_handles::<NestedArtboard>());
            let unit = child(&slots[0]).unwrap().core_handle();
            let flight = child(&slots[1]).unwrap().core_handle();
            LiveMoves::default().prepare(&root).unwrap();
            assert!(move_live(&slots[0], &slots[1]).is_err());
            assert!(empty_slot(&slots[1]).is_err());
            assert_eq!(child(&slots[0]).unwrap().core_handle(), unit);
            assert_eq!(child(&slots[1]).unwrap().core_handle(), flight);
        }
    }
}

// Patch only known encoded declaration properties in the extracted Page run.
#[cfg(feature = "tools")]
fn replacement_page(s: &Scene, rejected: bool) -> Vec<u8> {
    use source::{core::binary_reader::BinaryReader, runtime_header::RuntimeHeader};
    let bytes = fixture::fixture(2);
    let mut reader = BinaryReader::new(&bytes);
    assert!(RuntimeHeader::read(
        &mut reader,
        &mut RuntimeHeader::default()
    ));
    let offset = bytes.len() - reader.position().len();
    let range = s._file.with_file(|f| f.artboard_byte_range(0));
    let mut page = bytes[offset + range.start..offset + range.end].to_vec();
    fn key(mut n: u16) -> Vec<u8> {
        let mut bytes = Vec::new();
        while n >= 128 {
            bytes.push((n as u8 & 127) | 128);
            n >>= 7;
        }
        bytes.push(n as u8);
        bytes
    }
    for field in [records::FROM_ARTBOARD, records::TO_ARTBOARD] {
        let mut prefix = key(field);
        prefix.push(1); // one-byte artboard name
        let mut count = 0;
        for i in 0..page.len() - prefix.len() {
            if page[i..].starts_with(&prefix) {
                let value = &mut page[i + prefix.len()];
                assert!(matches!(*value, b'A' | b'B'));
                *value = if rejected {
                    if *value == b'A' { b'X' } else { b'Y' }
                } else {
                    b'B'
                };
                count += 1;
            }
        }
        assert_eq!(count, 2);
    }
    if rejected {
        // Duplicate a valid mapping at the end of the Page run. Individual
        // lifecycle validation succeeds, but validate_step_units rejects it.
        let prefix = key(records::LiveStepUnit::TYPE_KEY);
        let start = page
            .windows(prefix.len())
            .position(|w| w == prefix)
            .unwrap();
        let end = start + page[start..].iter().position(|v| *v == 0).unwrap() + 1;
        let duplicate = page[start..end].to_vec();
        page.extend(duplicate);
    }
    page
}

fn fresh_page(file: &RuntimeFileHandle) -> RuntimeArtboardInstanceHandle {
    let root = file.with_file(|f| f.artboard_named("Page")).unwrap();
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(model));
    machine.advance_and_apply(0.0);
    root
}

#[cfg(feature = "tools")]
#[test]
fn import_validation_rejected_replacement_preserves_existing_definitions() {
    let s = Scene::new();
    let page = s
        ._file
        .with_file(|f| f.artboard_named_source("Page"))
        .unwrap();
    let records = ["A", "B"].map(|name| {
        s._file
            .with_file(|f| f.artboard_named_source(name))
            .unwrap()
            .with_downcast::<Artboard, _>(|b| b.find_all_handles::<records::LiveSlot>())
            .unwrap()
    });
    let bytes = replacement_page(&s, true);
    assert_eq!(
        s._file.with_file_mut(|f| f.replace_artboard(0, &bytes)),
        ImportResult::Malformed
    );
    assert_eq!(
        s._file
            .with_file(|f| f.artboard_named_source("Page"))
            .unwrap(),
        page
    );
    for (name, before) in ["A", "B"].into_iter().zip(records) {
        let after = s
            ._file
            .with_file(|f| f.artboard_named_source(name))
            .unwrap()
            .with_downcast::<Artboard, _>(|b| b.find_all_handles::<records::LiveSlot>())
            .unwrap();
        assert!(!before.is_empty());
        assert_eq!(before, after, "{name} definition records changed");
        assert!(before.iter().all(CoreHandle::is_alive));
    }
    let root = fresh_page(&s._file);
    let rows = screens(&root).unwrap();
    let source = rows["A"].with_artboard(|b| b.find_all_handles::<NestedArtboard>())[0].clone();
    let destination =
        rows["B"].with_artboard(|b| b.find_all_handles::<NestedArtboard>())[0].clone();
    let unit = child(&source).unwrap().core_handle();
    let mut moves = LiveMoves::default();
    moves.step_begin(&root, "B", Some("t1")).unwrap();
    moves.step_end(&root, "B", Some("t1")).unwrap();
    assert_eq!(child(&destination).unwrap().core_handle(), unit);
}

#[cfg(feature = "tools")]
#[test]
fn import_validation_successful_replacement_ignores_former_screen_slots() {
    let mut s = Scene::new();
    let bytes = replacement_page(&s, false);
    assert_eq!(
        s._file.with_file_mut(|f| f.replace_artboard(0, &bytes)),
        ImportResult::Success
    );
    let root = fresh_page(&s._file);
    let rows = screens(&root).unwrap();
    let a = &rows["A"];
    assert!(
        !a.with_artboard(|b| b.find_all_handles::<records::LiveSlot>())
            .is_empty()
    );
    let slots = a.with_artboard(|b| b.find_all_handles::<NestedArtboard>());
    let unit = child(&slots[0]).unwrap().core_handle();
    let flight = child(&slots[1]).unwrap().core_handle();
    let mut moves = LiveMoves::default();
    moves.prepare(&root).unwrap();
    assert_eq!(
        child(&slots[1]).unwrap().core_handle(),
        flight,
        "ignored flight was emptied"
    );
    assert!(move_live(&slots[0], &slots[1]).is_err());
    assert!(empty_slot(&slots[1]).is_err());
    assert_eq!(child(&slots[0]).unwrap().core_handle(), unit);
    // The retained old occurrence still has its original A/B classification.
    s.begin();
    s.end();
}

#[test]
fn import_validation_preserves_and_ignores_steps_without_screens() {
    let mut bytes = fixture::fixture(2);
    for i in 0..bytes.len() - 7 {
        if &bytes[i..i + 7] == b"screens" {
            bytes[i..i + 7].copy_from_slice(b"content");
        }
    }
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = fresh_page(&file);
    assert_eq!(
        root.with_artboard(|b| b.find_all_handles::<records::LiveStep>().len()),
        2
    );
    assert_eq!(
        root.with_artboard(|b| b.find_all_handles::<records::LiveStepUnit>().len()),
        2
    );
    assert!(!hosted_scene::enabled(&root));
    let mut moves = LiveMoves::default();
    moves.prepare(&root).unwrap();
    assert!(moves.frame_scope(&root, &[]).unwrap().is_none());
    assert!(moves.step_begin(&root, "B", Some("t1")).is_err());
}

#[test]
fn record_free_page_sleeps_wakes_and_cuts() {
    let mut s = Scene::load(&fixture::record_free_page_fixture(), false);
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let original = child(&s.s).unwrap().core_handle();
    set_row_collapsed(&list, 0, true).expect("record-free page can sleep");
    assert!(s.moves.step_begin(&s.root, "A", None).is_err());
    set_row_collapsed(&list, 0, false).expect("record-free page can wake");
    s.moves
        .step_begin(&s.root, "A", None)
        .expect("record-free cut begins");
    s.moves
        .step_end(&s.root, "A", None)
        .expect("record-free cut ends");
    assert_eq!(child(&s.s).unwrap().core_handle(), original);
}

#[test]
fn record_free_nonpage_refuses_sleep_and_cut_without_allocating() {
    let mut s = Scene::load(&fixture::ordinary_fixture(2), false);
    let before = hosted_scene::bookkeeping_counts();
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    assert!(set_row_collapsed(&list, 0, true).is_err());
    assert!(s.moves.step_begin(&s.root, "A", None).is_err());
    assert_eq!(hosted_scene::bookkeeping_counts(), before);
}

fn page_model(s: &Scene) -> RuntimeOwnedViewModelHandle {
    let model = s
        .root
        .data_context()
        .unwrap()
        .with_context(|c| c.main_view_model_instance())
        .unwrap();
    RuntimeOwnedViewModelHandle::from_native(s._file.clone(), model).unwrap()
}

#[test]
fn pending_insert_is_applied_before_takeoff() {
    let mut s = Scene::new();
    let page = page_model(&s);
    let incoming = page.list_item_by_property_name_path("screens", 1).unwrap();
    assert!(page.remove_list_item_by_property_name_path("screens", 1));
    s.advance(0.0);
    assert!(!screens(&s.root).unwrap().contains_key("B"));
    assert!(page.insert_list_item_by_property_name_path("screens", 1, &incoming));
    // No advance or draw is allowed between insertion and take-off.
    s.moves.step_begin(&s.root, "B", Some("t1")).unwrap();
    let b = screens(&s.root).unwrap()["B"].clone();
    let slots = b.with_artboard(|b| b.find_all_handles::<NestedArtboard>());
    assert!(
        child(&slots[0]).is_none(),
        "fresh incoming unit must not draw before take-off"
    );
    assert_eq!(drawn_bounds(&mut s).len(), 1);
}

#[test]
fn pending_move_preserves_awake_and_sleeping_rows() {
    for asleep in [false, true] {
        let mut s = Scene::new();
        let page = page_model(&s);
        let list = s
            .root
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
            .unwrap();
        let board = s.b.core_handle();
        let unit = child(&s.d).unwrap().core_handle();
        let playback = machine(&s.d);
        let model =
            s.b.data_context()
                .unwrap()
                .with_context(|c| c.main_view_model_instance())
                .unwrap();
        if asleep {
            set_row_collapsed(&list, 1, true).unwrap();
        }
        assert!(page.move_list_item_by_property_name_path("screens", 1, 0));
        // A status operation must settle membership without a playback advance.
        s.moves.step_begin(&s.root, "A", None).unwrap();
        s.moves.step_end(&s.root, "A", None).unwrap();
        let first = list
            .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(0))
            .flatten()
            .unwrap();
        assert_eq!(first.core_handle(), board);
        assert_eq!(child(&s.d).unwrap().core_handle(), unit);
        assert!(playback.downgrade().ptr_eq(&machine(&s.d).downgrade()));
        assert_eq!(
            first
                .data_context()
                .unwrap()
                .with_context(|c| c.main_view_model_instance())
                .unwrap(),
            model
        );
        if asleep {
            assert!(s.moves.step_begin(&s.root, "B", Some("t1")).is_err());
            set_row_collapsed(&list, 0, false).unwrap();
        }
        assert!(page.move_list_item_by_property_name_path("screens", 0, 1));
        let outgoing = child(&s.s).unwrap().core_handle();
        s.moves.step_begin(&s.root, "B", Some("t1")).unwrap();
        assert_eq!(child(&s.f).unwrap().core_handle(), outgoing);
        s.moves.step_end(&s.root, "B", Some("t1")).unwrap();
    }
}

#[test]
fn page_content_height_tracks_top_row_and_bound_text_without_drawing_changes() {
    use page_player::{PagePlayer, Playback, Step};
    use source::layout_component::LayoutComponent;
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::fit_content_page_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let mut board = ArtboardInstance::from_native(file.clone(), 0).unwrap();
    let root = board.native_handle();
    let vm = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let vm = RuntimeOwnedViewModelHandle::from_native(file, vm).unwrap();
    board.bind_owned_view_model_handle(vm.clone());
    let mut machine = board.default_state_machine_instance().unwrap();
    machine.bind_owned_view_model_handle(vm.clone());
    let mut playback = Playback::StateMachine(Box::new(machine));
    let player = PagePlayer::default();
    let roots = std::slice::from_ref(&vm);
    player
        .step(&mut board, &mut playback, Step::default(), roots, None)
        .unwrap();
    let measure = |board: &ArtboardInstance, factory: &PersistentFactory<RecordingFactory>| {
        factory.borrow_mut().clear();
        let mut renderer = factory.borrow().make_renderer();
        board.draw(&mut renderer);
        let before = factory.borrow().canonical_recording().stream().to_owned();
        let serial = root.with_artboard(|b| b.change_serial());
        hosted_scene::measure_walks(true);
        let height = player.content_height(board).unwrap();
        assert_eq!(root.with_artboard(|b| b.change_serial()), serial);
        assert_eq!(height, player.content_height(board).unwrap());
        let walks = hosted_scene::measure_walks(false);
        assert_eq!(
            walks.intrinsic_measure, 1,
            "second measurement must use the cache"
        );
        assert_eq!(walks.advance, 0);
        factory.borrow_mut().clear();
        let mut renderer = factory.borrow().make_renderer();
        board.draw(&mut renderer);
        assert_eq!(factory.borrow().canonical_recording().stream(), before);
        height
    };
    let label_height = |name: &str| {
        screens(&root).unwrap()[name]
            .with_artboard(|b| b.find_handle::<LayoutComponent>("labelBox"))
            .unwrap()
            .with_downcast::<LayoutComponent, _>(|l| l.layout_height())
            .unwrap()
    };
    let b_height = measure(&board, &factory);
    assert!(label_height("B") > 0.0, "fixture must lay out real text");
    close(b_height, 160.0 + label_height("B"));
    assert!(vm.move_list_item_by_property_name_path("screens", 0, 1));
    player
        .step(&mut board, &mut playback, Step::default(), roots, None)
        .unwrap();
    let a_height = measure(&board, &factory);
    close(a_height, 80.0 + label_height("A"));
    close(b_height - a_height, 80.0);
    let row = vm.list_item_by_property_name_path("screens", 1).unwrap();
    assert!(row.borrow_mut().set_string_by_property_name(
        "label",
        "More wrapping text for the sheet. ".repeat(30).as_bytes()
    ));
    player
        .step(&mut board, &mut playback, Step::default(), roots, None)
        .unwrap();
    let grown = measure(&board, &factory);
    assert!(
        grown > a_height + 24.0,
        "bound text must grow intrinsic height"
    );
    let output = player
        .step(&mut board, &mut playback, Step::default(), roots, None)
        .unwrap();
    assert!(output.events.is_empty());
    assert!(output.commands.is_empty());
    assert!(output.changes.is_empty());
}

#[test]
fn pending_remove_is_applied_before_row_sleep() {
    let mut s = Scene::new();
    let page = page_model(&s);
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let incoming = s.b.core_handle();
    assert!(page.remove_list_item_by_property_name_path("screens", 0));
    set_row_collapsed(&list, 0, true).unwrap();
    assert_eq!(
        list.with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(0))
            .flatten()
            .unwrap()
            .core_handle(),
        incoming
    );
    assert!(s.moves.step_begin(&s.root, "B", None).is_err());
    set_row_collapsed(&list, 0, false).unwrap();
    s.moves.step_begin(&s.root, "B", None).unwrap();
    s.moves.step_end(&s.root, "B", None).unwrap();
}

fn string_value(model: &CoreHandle, name: &str) -> String {
    CoreRegistry::get_string_handle(&property(model, name), 561).unwrap()
}
fn color_value(model: &CoreHandle, name: &str) -> u32 {
    CoreRegistry::get_color_handle(&property(model, name), 555).unwrap() as u32
}
fn flight_progress(s: &mut Scene, progress: f32) {
    for flight in s.moves.flights(&s.root) {
        set(&flight.controls(), 62003, progress);
    }
    s.advance(0.0);
}

#[test]
fn incoming_number_and_color_blend_then_bindings_drive_after_landing() {
    let mut s = Scene::from_bytes(&fixture::values_fixture(false));
    let unit = child(&s.s).unwrap();
    let model = vm(&unit);
    // The start is the current instance value, rather than its old source.
    set(&property(&model, "prop"), 575, 30.0);
    s.begin();
    for (progress, number, color) in [
        (0.0, 30.0, 0x20406080),
        (0.5, 115.0, 0x80b09880),
        (1.0, 200.0, 0xe0c0a080),
    ] {
        flight_progress(&mut s, progress);
        close(val(&property(&model, "prop"), 575), number);
        assert_eq!(color_value(&model, "accent"), color);
    }
    s.end();
    assert_eq!(child(&s.d).unwrap().core_handle(), unit.core_handle());
    set(&property(&vm(&s.b), "offset"), 575, 321.0);
    assert!(CoreRegistry::set_color_handle(
        &property(&vm(&s.b), "accent"),
        555,
        0x12345678
    ));
    s.advance(0.0);
    close(val(&property(&model, "prop"), 575), 321.0);
    assert_eq!(color_value(&model, "accent"), 0x12345678);
}

/// Flies the unit's `accent` from `from`, its current value, to `to`, the
/// landing Screen's value, and reads it at each progress. The reads are hex so
/// a failed comparison shows every step.
fn accent_blend(from: u32, to: u32, progresses: &[f32]) -> Vec<String> {
    let mut s = Scene::from_bytes(&fixture::values_fixture(false));
    let model = vm(&child(&s.s).unwrap());
    assert!(CoreRegistry::set_color_handle(
        &property(&model, "accent"),
        555,
        from as i32
    ));
    assert!(CoreRegistry::set_color_handle(
        &property(&vm(&s.b), "accent"),
        555,
        to as i32
    ));
    s.begin();
    let reads = progresses
        .iter()
        .map(|&progress| {
            flight_progress(&mut s, progress);
            format!("{:#010x}", color_value(&model, "accent"))
        })
        .collect();
    s.end();
    reads
}

#[test]
fn a_clear_end_keeps_the_other_ends_color() {
    let (purple, clear) = (0xffa67dc1, 0x00000000);
    let at = [0.25, 0.5, 0.75];
    assert_eq!(
        [
            accent_blend(purple, clear, &at),
            accent_blend(clear, purple, &at)
        ],
        [
            ["0xbfa67dc1", "0x80a67dc1", "0x40a67dc1"],
            ["0x40a67dc1", "0x80a67dc1", "0xbfa67dc1"]
        ]
    );
}

#[test]
fn a_clear_end_keeps_the_other_ends_color_on_the_last_steps() {
    // So near the end that alpha rounds to 0 and nothing draws, but the
    // channels stay the color's own.
    assert_eq!(
        accent_blend(0x82ffffff, 0x00000000, &[1.0 - f32::EPSILON]),
        ["0x00ffffff"]
    );
}

#[test]
fn a_faint_end_counts_by_its_strength() {
    assert_eq!(accent_blend(0x0f000000, 0xffffffff, &[0.5]), ["0x87f1f1f1"]);
}

#[test]
fn clear_to_clear_stays_clear() {
    let halfway = accent_blend(0x00a67dc1, 0x00000000, &[0.5]);
    assert!(halfway[0].starts_with("0x00"), "{halfway:?}");
    for (from, to) in [
        (0xffa67dc1u32, 0x00000000u32),
        (0x00000000, 0xffa67dc1),
        (0x0f000000, 0xffffffff),
    ] {
        assert_eq!(
            accent_blend(from, to, &[0.0, 1.0]),
            [format!("{from:#010x}"), format!("{to:#010x}")]
        );
    }
}

#[test]
fn equal_strength_colors_blend_as_before() {
    assert_eq!(accent_blend(0xff204060, 0xff60a0c0, &[0.5]), ["0xff407090"]);
}

/// Removes row B and inserts its Screen model again, as a host adds the
/// incoming row for a step. The page has not advanced since the insert.
fn add_row_b_for_the_step(s: &mut Scene) {
    let page = page_model(s);
    let incoming = page.list_item_by_property_name_path("screens", 1).unwrap();
    assert!(page.remove_list_item_by_property_name_path("screens", 1));
    s.advance(0.0);
    assert!(!screens(&s.root).unwrap().contains_key("B"));
    assert!(page.insert_list_item_by_property_name_path("screens", 1, &incoming));
}
fn row_b_and_its_home_slot(s: &Scene) -> (RuntimeArtboardInstanceHandle, CoreHandle) {
    let row = screens(&s.root).unwrap()["B"].clone();
    let home = row.with_artboard(|b| b.find_all_handles::<NestedArtboard>())[0].clone();
    (row, home)
}

#[test]
fn a_unit_flying_into_a_row_added_for_the_step_blends_its_number_and_color() {
    for settle in [false, true] {
        let mut s = Scene::from_bytes(&fixture::values_fixture(false));
        let unit = child(&s.s).unwrap();
        let model = vm(&unit);
        let context = unit.data_context().unwrap();
        add_row_b_for_the_step(&mut s);
        let list = s
            .root
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
            .unwrap();
        // Waking the row flushes the insert and empties its fresh flight slot
        // before any advance, as the SDK does; it may then settle one frame.
        set_row_collapsed(&list, 1, false).unwrap();
        if settle {
            s.advance(0.0);
        }
        s.begin();
        flight_progress(&mut s, 0.5);
        let at_half = val(&property(&model, "prop"), 575);
        assert!(
            (at_half - 105.0).abs() < 0.002,
            "settle {settle}: expected 105 halfway, got {at_half}"
        );
        assert_eq!(color_value(&model, "accent"), 0x80b09880, "settle {settle}");
        s.end();
        s.advance(0.0);
        let (row, home) = row_b_and_its_home_slot(&s);
        assert_eq!(child(&home).unwrap().core_handle(), unit.core_handle());
        assert!(context.ptr_eq(&unit.data_context().unwrap()));
        assert!(
            context
                .with_context(|c| c.parent())
                .unwrap()
                .ptr_eq(&row.data_context().unwrap())
        );
        close(val(&property(&model, "prop"), 575), 200.0);
        assert_eq!(color_value(&model, "accent"), 0xe0c0a080);
        set(&property(&vm(&row), "offset"), 575, 321.0);
        assert!(CoreRegistry::set_color_handle(
            &property(&vm(&row), "accent"),
            555,
            0x12345678
        ));
        s.advance(0.0);
        let followed = val(&property(&model, "prop"), 575);
        assert!(
            (followed - 321.0).abs() < 0.002,
            "settle {settle}: expected 321 after landing, got {followed}"
        );
        assert_eq!(color_value(&model, "accent"), 0x12345678, "settle {settle}");
    }
}

#[test]
fn a_never_advanced_incoming_version_departs_bound_and_follows_its_screen() {
    let mut s = Scene::from_bytes(&fixture::two_version_values_fixture());
    add_row_b_for_the_step(&mut s);
    // No frame and no wake between the insert and take-off.
    s.begin();
    flight_progress(&mut s, 0.5);
    s.end();
    s.advance(0.0);
    let (row, home) = row_b_and_its_home_slot(&s);
    let incoming = vm(&child(&home).unwrap());
    set(&property(&vm(&row), "offset"), 575, 400.0);
    s.advance(0.0);
    close(val(&property(&incoming, "prop"), 575), 400.0);
}

#[test]
fn incoming_values_follow_live_destination_changes_during_flight() {
    let mut s = Scene::from_bytes(&fixture::values_fixture(false));
    let model = vm(&child(&s.s).unwrap());
    s.begin();
    flight_progress(&mut s, 0.5);
    close(val(&property(&model, "prop"), 575), 105.0);
    set(&property(&vm(&s.b), "offset"), 575, 310.0);
    assert!(CoreRegistry::set_color_handle(
        &property(&vm(&s.b), "accent"),
        555,
        0x6080a0c0
    ));
    s.advance(0.0);
    close(val(&property(&model, "prop"), 575), 160.0);
    assert_eq!(color_value(&model, "accent"), 0x407090b0);
    flight_progress(&mut s, 1.0);
    close(val(&property(&model, "prop"), 575), 310.0);
    assert_eq!(color_value(&model, "accent"), 0x6080a0c0);
    // Landing samples the latest destination even without another follow.
    set(&property(&vm(&s.b), "offset"), 575, 410.0);
    s.end();
    close(val(&property(&model, "prop"), 575), 410.0);
}

#[test]
fn incoming_string_switches_at_takeoff_and_private_values_stay() {
    let mut s = Scene::from_bytes(&fixture::values_fixture(false));
    let model = vm(&child(&s.s).unwrap());
    assert_eq!(string_value(&model, "label"), "outgoing");
    set(&property(&model, "count"), 575, 7.0);
    s.begin();
    assert_eq!(string_value(&model, "label"), "incoming");
    close(val(&property(&model, "prop"), 575), 10.0);
    close(val(&property(&model, "count"), 575), 7.0);
    flight_progress(&mut s, 0.5);
    s.end();
    close(val(&property(&model, "count"), 575), 7.0);
}

#[test]
fn unbound_typed_text_writes_the_incoming_prop_source_in_both_directions() {
    for (two_way, settle) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut s = Scene::from_bytes(&fixture::values_fixture(two_way));
        let unit = child(&s.s).unwrap();
        let model = vm(&unit);
        let field = unit
            .with_artboard(|b| b.find_handle::<source::text::text_input::TextInput>("edit"))
            .unwrap();
        assert!(
            field
                .with_downcast_mut::<source::text::text_input::TextInput, _>(
                    |f| f.text_input("typed locally")
                )
                .unwrap()
        );
        if settle {
            s.advance(0.0);
            assert_eq!(string_value(&model, "text"), "typed locally");
        }
        s.begin();
        assert_eq!(string_value(&vm(&s.b), "text"), "typed locally");
        s.end();
        assert_eq!(string_value(&vm(&s.b), "text"), "typed locally");
        assert_eq!(string_value(&model, "text"), "typed locally");
        assert_eq!(
            CoreRegistry::get_string_handle(&field, 817).as_deref(),
            Some("typed locally")
        );
        // From landing on, the incoming prop drives the same input normally.
        assert!(CoreRegistry::set_string_handle(
            &property(&vm(&s.b), "text"),
            561,
            "later".into()
        ));
        s.advance(0.0);
        assert_eq!(
            CoreRegistry::get_string_handle(&field, 817).as_deref(),
            Some("later")
        );
    }
}

#[test]
fn authored_unbound_text_does_not_override_the_incoming_prop() {
    let mut s = Scene::from_bytes(&fixture::values_fixture(false));
    let unit = child(&s.s).unwrap();
    let model = vm(&unit);
    assert!(CoreRegistry::set_string_handle(
        &property(&model, "text"),
        561,
        "authored".into()
    ));
    s.advance(0.0);
    s.begin();
    s.end();
    assert_eq!(string_value(&vm(&s.b), "text"), "destination text");
    assert_eq!(string_value(&model, "text"), "destination text");
}

#[test]
fn takeoff_and_landing_without_an_advance_adopt_destination_values_at_once() {
    let mut s = Scene::from_bytes(&fixture::values_fixture(false));
    let model = vm(&child(&s.s).unwrap());
    s.begin();
    s.end();
    close(val(&property(&model, "prop"), 575), 200.0);
    assert_eq!(color_value(&model, "accent"), 0xe0c0a080);
    assert!(s.moves.flights(&s.root).is_empty());
    s.moves.step_begin(&s.root, "A", None).unwrap();
    close(val(&property(&model, "prop"), 575), 10.0);
    assert_eq!(color_value(&model, "accent"), 0x20406080);
    s.moves.step_end(&s.root, "A", None).unwrap();
}

#[test]
fn two_versions_blend_against_their_own_landing_slot_values() {
    let mut s = Scene::from_bytes(&fixture::two_version_values_fixture());
    let outgoing = vm(&child(&s.s).unwrap());
    let incoming = vm(&child(&s.d).unwrap());
    s.begin();
    close(val(&property(&outgoing, "prop"), 575), 10.0);
    close(val(&property(&incoming, "prop"), 575), 200.0);
    set(&property(&vm(&s.a), "offset"), 575, 110.0);
    set(&property(&vm(&s.b), "offset"), 575, 400.0);
    flight_progress(&mut s, 0.5);
    close(val(&property(&outgoing, "prop"), 575), 60.0);
    close(val(&property(&incoming, "prop"), 575), 300.0);
    s.end();
    close(val(&property(&outgoing, "prop"), 575), 110.0);
    close(val(&property(&incoming, "prop"), 575), 400.0);
}

#[test]
fn values_follow_the_same_native_keyed_progress_as_the_pose() {
    let mut s = Scene::from_bytes(&fixture::keyed_fixture());
    let model = vm(&child(&s.s).unwrap());
    s.begin();
    s.advance(0.5);
    let progress = val(&s.moves.flights(&s.root)[0].controls(), 62003);
    close(progress, 0.5);
    close(val(&property(&model, "prop"), 575), 105.0);
}

/// A key that runs past 1 leaves the flight's progress at 1.25. The
/// follower clamps it once and hands that one number to the values and to
/// the pose, so both land: raw progress would give `prop` 247.5 and
/// elapsed time 105, where the clamped 1 gives 200.
#[test]
fn values_follow_the_poses_clamped_progress_when_a_key_overshoots() {
    let mut s = Scene::from_bytes(&fixture::keyed_overshoot_fixture());
    let model = vm(&child(&s.s).unwrap());
    s.begin();
    let start = val(&property(&model, "prop"), 575);
    assert!(
        (start - 200.0).abs() > 1.0,
        "the source's prop starts away from 200: {start}"
    );
    s.advance(0.5);
    close(val(&s.moves.flights(&s.root)[0].controls(), 62003), 1.25);
    close(val(&property(&model, "prop"), 575), 200.0);
    let a = slot_pose(&s.s).unwrap();
    let b = slot_pose(&s.d).unwrap();
    let f = slot_pose(&s.f).unwrap();
    assert!(
        (a.transform[4] - b.transform[4]).abs() > 1.0,
        "the slots sit apart in x: {} and {}",
        a.transform[4],
        b.transform[4]
    );
    close(f.transform[4], b.transform[4] + 7.0);
    close(f.transform[5], b.transform[5] + 11.0);
}

#[test]
fn converted_two_way_numbers_blend_without_writing_back_presentation_values() {
    use source::data_bind::converters::{
        data_converter_operation::ArithmeticOperation,
        data_converter_operation_value::DataConverterOperationValue,
    };
    use source::viewmodel::viewmodel_instance::ViewModelInstance;
    let mut s = Scene::new();
    let model = vm(&child(&s.s).unwrap());
    let fresh = child(&s.d).unwrap();
    let fresh_model = vm(&fresh);
    let target = property(&fresh_model, "prop");
    let converter =
        s.d.insert_sibling(DataConverterOperationValue::new(
            ArithmeticOperation::Multiply,
            2.0,
        ))
        .unwrap();
    let mut binds = s.b.with_artboard(|b| b.data_bind_handles());
    binds.extend(fresh.with_artboard(|b| b.data_bind_handles()));
    binds.extend(
        fresh_model
            .with_downcast::<ViewModelInstance, _>(|v| v.value_data_binds().to_vec())
            .unwrap(),
    );
    let mut configured = 0;
    for bind in binds {
        if bind
            .with(|o| o.as_data_bind().and_then(|b| b.target()))
            .flatten()
            .as_ref()
            == Some(&target)
        {
            assert!(CoreRegistry::set_uint_handle(&bind, 587, 2 | 8));
            bind.with_mut(|o| {
                o.as_data_bind_mut()
                    .unwrap()
                    .set_converter(Some(converter.clone()))
            });
            configured += 1;
        }
    }
    assert!(configured > 0);
    s.begin();
    flight_progress(&mut s, 0.5);
    close(val(&property(&model, "prop"), 575), 205.0);
    close(val(&property(&vm(&s.b), "offset"), 575), 200.0);
    set(&property(&vm(&s.b), "offset"), 575, 310.0);
    s.advance(0.0);
    close(val(&property(&model, "prop"), 575), 315.0);
    close(val(&property(&vm(&s.b), "offset"), 575), 310.0);
    s.end();
    close(val(&property(&model, "prop"), 575), 620.0);
    close(val(&property(&vm(&s.b), "offset"), 575), 310.0);
}

#[test]
fn outgoing_bound_typed_text_does_not_write_the_incoming_source() {
    use source::data_bind::{data_bind::DataBind, data_bind_context::DataBindContext};
    let mut s = Scene::from_bytes(&fixture::values_fixture(false));
    let unit = child(&s.s).unwrap();
    let model = vm(&unit);
    let mut bind = DataBindContext::default();
    bind.base.base = DataBind::new(0, 561, 0);
    bind.decode_source_path_ids(&[1, 3]);
    let bind = s.s.insert_sibling(bind).unwrap();
    bind.with_mut(|o| {
        o.as_data_bind_mut()
            .unwrap()
            .set_target(Some(property(&model, "text")))
    });
    s.a.with_artboard_mut(|b| b.add_data_bind(bind));
    s.advance(0.0);
    let field = unit
        .with_artboard(|b| b.find_handle::<source::text::text_input::TextInput>("edit"))
        .unwrap();
    assert!(
        field
            .with_downcast_mut::<source::text::text_input::TextInput, _>(
                |f| f.text_input("new edit")
            )
            .unwrap()
    );
    s.begin();
    s.end();
    assert_eq!(string_value(&vm(&s.b), "text"), "destination text");
    assert_eq!(string_value(&model, "text"), "destination text");
    assert_eq!(
        CoreRegistry::get_string_handle(&field, 817).as_deref(),
        Some("destination text")
    );
}

// Inline VM import normally makes two-way props source-first. Explicitly
// restore authored target-first ordering in the native and mounted bindings.
fn target_first_props(row: &RuntimeArtboardInstanceHandle, slot: &CoreHandle, names: &[&str]) {
    use source::viewmodel::viewmodel_instance::ViewModelInstance;
    let unit = child(slot).unwrap();
    let model = vm(&unit);
    let mut binds = row.with_artboard(|b| b.data_bind_handles());
    binds.extend(unit.with_artboard(|b| b.data_bind_handles()));
    binds.extend(
        model
            .with_downcast::<ViewModelInstance, _>(|v| v.value_data_binds().to_vec())
            .unwrap(),
    );
    for name in names {
        let target = property(&model, name);
        let mut configured = 0;
        for bind in &binds {
            if bind
                .with(|o| o.as_data_bind().and_then(|b| b.target()))
                .flatten()
                .as_ref()
                == Some(&target)
            {
                assert!(CoreRegistry::set_uint_handle(bind, 587, 2));
                configured += 1;
            }
        }
        assert!(configured > 0, "no prop binding for {name}");
    }
}

fn assert_target_first_prop(unit: &RuntimeArtboardInstanceHandle, target: &CoreHandle) {
    let binds = unit.with_artboard(|b| b.data_bind_handles());
    assert!(
        binds.iter().any(|bind| bind.with(|o| {
            let b = o.as_data_bind().unwrap();
            b.target().as_ref() == Some(target) && b.to_source() && !b.source_to_target_runs_first()
        }) == Some(true)),
        "normal authored two-way ordering must resume"
    );
}

#[test]
fn target_first_two_way_midpoint_landing_adopts_source_before_writeback() {
    for latest in [200.0, 310.0] {
        let mut s = Scene::from_bytes(&fixture::values_fixture(false));
        target_first_props(&s.b, &s.d, &["prop", "accent"]);
        let unit = child(&s.s).unwrap();
        let model = vm(&unit);
        let destination = vm(&s.b);
        s.begin();
        flight_progress(&mut s, 0.5);
        close(val(&property(&model, "prop"), 575), 105.0);
        assert_eq!(color_value(&model, "accent"), 0x80b09880);
        // No further follow: landing must sample the latest source itself.
        set(&property(&destination, "offset"), 575, latest);
        assert!(CoreRegistry::set_color_handle(
            &property(&destination, "accent"),
            555,
            0x6080a0c0
        ));
        s.end();
        close(val(&property(&destination, "offset"), 575), latest);
        close(val(&property(&model, "prop"), 575), latest);
        assert_eq!(color_value(&destination, "accent"), 0x6080a0c0);
        assert_eq!(color_value(&model, "accent"), 0x6080a0c0);
        assert_target_first_prop(&unit, &property(&model, "prop"));
        // The handoff is one-time: real edits after landing still write back.
        set(&property(&model, "prop"), 575, 444.0);
        assert!(CoreRegistry::set_color_handle(
            &property(&model, "accent"),
            555,
            0x12345678
        ));
        s.advance(0.0);
        close(val(&property(&destination, "offset"), 575), 444.0);
        assert_eq!(color_value(&destination, "accent"), 0x12345678);
    }
}

#[test]
fn target_first_two_way_return_after_midflight_row_removal_adopts_home_source() {
    let mut s = Scene::from_bytes(&fixture::values_fixture(false));
    target_first_props(&s.a, &s.s, &["prop", "accent"]);
    target_first_props(&s.b, &s.d, &["prop", "accent"]);
    let unit = child(&s.s).unwrap();
    let model = vm(&unit);
    let home = vm(&s.a);
    let destination = vm(&s.b);
    s.begin();
    flight_progress(&mut s, 0.5);
    close(val(&property(&model, "prop"), 575), 105.0);
    let list = s
        .root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let a = list
        .with_downcast::<ArtboardComponentList, _>(|l| l.list_item(0))
        .flatten()
        .unwrap();
    ArtboardComponentList::update_list_occurrence(&list, &[a]);
    s.advance(0.0);
    assert_eq!(child(&s.s).unwrap().core_handle(), unit.core_handle());
    assert!(s.moves.flights(&s.root).is_empty());
    close(val(&property(&home, "offset"), 575), 10.0);
    close(val(&property(&destination, "offset"), 575), 200.0);
    close(val(&property(&model, "prop"), 575), 10.0);
    assert_eq!(color_value(&home, "accent"), 0x20406080);
    assert_eq!(color_value(&destination, "accent"), 0xe0c0a080);
    assert_eq!(color_value(&model, "accent"), 0x20406080);
    assert_target_first_prop(&unit, &property(&model, "prop"));
    set(&property(&model, "prop"), 575, 88.0);
    s.advance(0.0);
    close(val(&property(&home, "offset"), 575), 88.0);
    close(val(&property(&destination, "offset"), 575), 200.0);
}

#[test]
fn target_first_two_way_expired_projection_adopts_live_source_before_writeback() {
    for queued_target in [false, true] {
        let mut s = Scene::new();
        target_first_props(&s.b, &s.d, &["prop"]);
        let unit = child(&s.s).unwrap();
        let model = vm(&unit);
        let target = property(&model, "prop");
        let source = property(&vm(&s.b), "offset");
        s.begin();
        flight_progress(&mut s, 0.5);
        close(val(&target, 575), 105.0);
        if queued_target {
            // Leave target dirt queued at the synthesized value, as well as
            // a newer source; neither ordering may write the blend back.
            set(&target, 575, 104.0);
            set(&target, 575, 105.0);
            set(&source, 575, 310.0);
        }
        // Cancellation can drop only the projection and retain the mounted
        // binding. Even a clean binding must settle its source-only handoff.
        s.moves.with_flights(&s.root, |flights| flights.clear());
        unit.update_data_binds(true);
        let expected = if queued_target { 310.0 } else { 200.0 };
        close(val(&source, 575), expected);
        close(val(&target, 575), expected);
        assert_target_first_prop(&unit, &target);
        set(&target, 575, 444.0);
        unit.update_data_binds(true);
        close(val(&source, 575), 444.0);
    }
}

// A landed unit keeps its home model, the model its home slot lends it, and
// copies the landing slot's bindings onto it. The SDK retires the home Screen
// right after End, so these tests sleep and wake Screens around a landing.
fn screens_list(s: &Scene) -> CoreHandle {
    s.root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap()
}
fn land_in_b(s: &mut Scene) {
    s.begin();
    flight_progress(s, 1.0);
    s.end();
}
fn close_in(what: &str, a: f32, b: f32) {
    assert!((a - b).abs() < 0.002, "{what}: expected {b}, got {a}");
}

#[test]
fn a_landed_unit_follows_its_new_screen_while_its_home_sleeps() {
    let mut s = Scene::from_bytes(&fixture::values_fixture(false));
    let list = screens_list(&s);
    let unit = child(&s.s).unwrap();
    let model = vm(&unit);
    // The start is the current instance value, rather than its old source.
    set(&property(&model, "prop"), 575, 30.0);
    s.begin();
    for (progress, number, color) in [
        (0.0, 30.0, 0x20406080),
        (0.5, 115.0, 0x80b09880),
        (1.0, 200.0, 0xe0c0a080),
    ] {
        flight_progress(&mut s, progress);
        close(val(&property(&model, "prop"), 575), number);
        assert_eq!(color_value(&model, "accent"), color);
    }
    s.end();
    // The SDK retires the Screen the unit came from right after End.
    set_row_collapsed(&list, 0, true).unwrap();
    assert_eq!(child(&s.d).unwrap().core_handle(), unit.core_handle());
    set(&property(&vm(&s.b), "offset"), 575, 321.0);
    assert!(CoreRegistry::set_color_handle(
        &property(&vm(&s.b), "accent"),
        555,
        0x12345678
    ));
    s.advance(0.0);
    close(val(&property(&model, "prop"), 575), 321.0);
    assert_eq!(color_value(&model, "accent"), 0x12345678);
}

#[test]
fn a_landed_units_own_writes_reach_its_new_screen_while_its_home_sleeps() {
    for latest in [200.0, 310.0] {
        let mut s = Scene::from_bytes(&fixture::values_fixture(false));
        let list = screens_list(&s);
        target_first_props(&s.b, &s.d, &["prop", "accent"]);
        let unit = child(&s.s).unwrap();
        let model = vm(&unit);
        let destination = vm(&s.b);
        s.begin();
        flight_progress(&mut s, 0.5);
        close(val(&property(&model, "prop"), 575), 105.0);
        assert_eq!(color_value(&model, "accent"), 0x80b09880);
        // No further follow: landing must sample the latest source itself.
        set(&property(&destination, "offset"), 575, latest);
        assert!(CoreRegistry::set_color_handle(
            &property(&destination, "accent"),
            555,
            0x6080a0c0
        ));
        s.end();
        // The SDK retires the Screen the unit came from right after End.
        set_row_collapsed(&list, 0, true).unwrap();
        close(val(&property(&destination, "offset"), 575), latest);
        close(val(&property(&model, "prop"), 575), latest);
        assert_eq!(color_value(&destination, "accent"), 0x6080a0c0);
        assert_eq!(color_value(&model, "accent"), 0x6080a0c0);
        assert_target_first_prop(&unit, &property(&model, "prop"));
        // The unit's own writes travel back to the Screen it landed on, the
        // way a tap's trigger does.
        set(&property(&model, "prop"), 575, 444.0);
        assert!(CoreRegistry::set_color_handle(
            &property(&model, "accent"),
            555,
            0x12345678
        ));
        s.advance(0.0);
        close(val(&property(&destination, "offset"), 575), 444.0);
        assert_eq!(color_value(&destination, "accent"), 0x12345678);
    }
}

#[test]
fn a_landed_unit_sleeps_and_wakes_with_the_screen_that_shows_it() {
    // The home awake is the guard; the home asleep is the case.
    for home_awake in [true, false] {
        let what = if home_awake {
            "home awake"
        } else {
            "home asleep"
        };
        let mut s = Scene::from_bytes(&fixture::values_fixture(false));
        let list = screens_list(&s);
        let model = vm(&child(&s.s).unwrap());
        land_in_b(&mut s);
        close_in(what, val(&property(&model, "prop"), 575), 200.0);
        if !home_awake {
            set_row_collapsed(&list, 0, true).unwrap();
        }
        set_row_collapsed(&list, 1, true).unwrap();
        set(&property(&vm(&s.b), "offset"), 575, 555.0);
        s.advance(0.0);
        // A change written while the showing Screen sleeps waits for its wake.
        close_in(
            &format!("{what}, showing Screen asleep"),
            val(&property(&model, "prop"), 575),
            200.0,
        );
        set_row_collapsed(&list, 1, false).unwrap();
        s.advance(0.0);
        close_in(
            &format!("{what}, showing Screen woken"),
            val(&property(&model, "prop"), 575),
            555.0,
        );
    }
}

#[test]
fn waking_the_home_later_changes_nothing_for_a_landed_unit() {
    let mut s = Scene::from_bytes(&fixture::values_fixture(false));
    let list = screens_list(&s);
    let model = vm(&child(&s.s).unwrap());
    set(&property(&model, "prop"), 575, 30.0);
    land_in_b(&mut s);
    set_row_collapsed(&list, 0, true).unwrap();
    set(&property(&vm(&s.b), "offset"), 575, 321.0);
    assert!(CoreRegistry::set_color_handle(
        &property(&vm(&s.b), "accent"),
        555,
        0x12345678
    ));
    s.advance(0.0);
    set_row_collapsed(&list, 0, false).unwrap();
    s.advance(0.0);
    close(val(&property(&model, "prop"), 575), 321.0);
    assert_eq!(color_value(&model, "accent"), 0x12345678);
    // The home's own bindings no longer drive the unit it lent out.
    set(&property(&vm(&s.a), "offset"), 575, 999.0);
    s.advance(0.0);
    close(val(&property(&model, "prop"), 575), 321.0);
}

#[test]
fn back_after_the_home_slept_returns_the_unit_with_its_own_bindings() {
    use source::data_bind::data_bind::DataBind;
    let mut s = Scene::from_bytes(&fixture::values_fixture(false));
    let list = screens_list(&s);
    let unit = child(&s.s).unwrap();
    let model = vm(&unit);
    land_in_b(&mut s);
    set_row_collapsed(&list, 0, true).unwrap();
    // Back in the SDK's order: wake the home, Begin, fly, End, sleep B.
    set_row_collapsed(&list, 0, false).unwrap();
    s.moves.step_begin(&s.root, "A", Some("back")).unwrap();
    flight_progress(&mut s, 1.0);
    s.moves.step_end(&s.root, "A", Some("back")).unwrap();
    set_row_collapsed(&list, 1, true).unwrap();
    assert_eq!(child(&s.s).unwrap().core_handle(), unit.core_handle());
    set(&property(&vm(&s.a), "offset"), 575, 77.0);
    s.advance(0.0);
    close(val(&property(&model, "prop"), 575), 77.0);
    set(&property(&vm(&s.b), "offset"), 575, 88.0);
    s.advance(0.0);
    close(val(&property(&model, "prop"), 575), 77.0);
    set_row_collapsed(&list, 1, false).unwrap();
    s.advance(0.0);
    close(val(&property(&model, "prop"), 575), 77.0);
    // One instance-value binding per home prop: prop, accent and label. A's
    // text has no Screen binding.
    let mut ids: Vec<u32> = unit
        .core_handle()
        .data_bind_container()
        .unwrap()
        .data_binds()
        .to_vec()
        .into_iter()
        .filter_map(|bind| {
            bind.with(|o| {
                o.as_data_bind()
                    .filter(|b| b.is_instance_value_bind())
                    .and_then(DataBind::target)
            })
            .flatten()
        })
        .map(|target| {
            target
                .with(|o| {
                    o.as_view_model_instance_value()
                        .map(|v| v.view_model_property_id())
                })
                .flatten()
                .unwrap()
        })
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, [1, 2, 3]);
}

#[test]
fn a_landed_units_model_reference_writes_back_while_its_home_sleeps() {
    use source::viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    };
    fn referenced(model: &CoreHandle) -> Option<CoreHandle> {
        property(model, "ref")
            .with_downcast::<ViewModelInstanceViewModel, _>(|v| v.reference_view_model_instance())
            .flatten()
    }
    // Lands the unit in B, then points the unit's reference at Ref's second
    // instance. The copied reference binding can't push, so it is polled.
    let land_and_repoint = |sleep: &[i32]| {
        let mut s = Scene::from_bytes(&fixture::values_reference_fixture());
        let list = screens_list(&s);
        let model = vm(&child(&s.s).unwrap());
        let second = s
            ._file
            .with_file(|f| f.create_view_model_instance_named("Ref", "second"))
            .unwrap();
        land_in_b(&mut s);
        let landing = referenced(&vm(&s.b));
        assert!(landing.is_some());
        assert_ne!(landing, Some(second.clone()));
        for &row in sleep {
            set_row_collapsed(&list, row, true).unwrap();
        }
        assert!(ViewModelInstance::replace_view_model_by_name(
            &model,
            "ref",
            second.clone()
        ));
        s.advance(0.0);
        (s, list, landing, second)
    };
    // Control: with the home awake, B hears the unit's reference.
    let (s, _, _, second) = land_and_repoint(&[]);
    assert_eq!(
        referenced(&vm(&s.b)),
        Some(second),
        "home awake: B takes the unit's reference"
    );
    // The SDK retires the home right after End.
    let (s, _, _, second) = land_and_repoint(&[0]);
    assert_eq!(
        referenced(&vm(&s.b)),
        Some(second),
        "home asleep: B takes the unit's reference"
    );
    // Guard: while B sleeps it keeps its reference, and takes the unit's
    // once it wakes, whether or not the home sleeps.
    let (mut s, list, landing, second) = land_and_repoint(&[0, 1]);
    assert_eq!(
        referenced(&vm(&s.b)),
        landing,
        "B asleep keeps its reference"
    );
    set_row_collapsed(&list, 1, false).unwrap();
    s.advance(0.0);
    assert_eq!(
        referenced(&vm(&s.b)),
        Some(second),
        "B woken with the home asleep: B takes the unit's reference"
    );
}

mod page_player_event_context {
    use super::*;
    use page_player::{ErrorKind, OwnedPlayerEvent, Playback, Step, own_reported_events};

    /// A page player over a fixture's first artboard, bound to its default
    /// view model, after one settling step.
    struct EventScene {
        _factory: PersistentFactory<RecordingFactory>,
        board: ArtboardInstance,
        playback: Playback,
        vm: RuntimeOwnedViewModelHandle,
        player: page_player::PagePlayer,
    }
    impl EventScene {
        fn new(bytes: &[u8]) -> Self {
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            let file = File::import(
                bytes,
                RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
                None,
                None,
                None,
            )
            .unwrap();
            let mut board = ArtboardInstance::from_native(file.clone(), 0).unwrap();
            let root = board.native_handle();
            let model = file
                .with_file(|f| {
                    f.create_default_view_model_instance_for_artboard(root.core_handle())
                })
                .unwrap();
            let vm = RuntimeOwnedViewModelHandle::from_native(file, model).unwrap();
            board.bind_owned_view_model_handle(vm.clone());
            let mut machine = board.default_state_machine_instance().unwrap();
            machine.bind_owned_view_model_handle(vm.clone());
            let mut scene = Self {
                _factory: factory,
                board,
                playback: Playback::StateMachine(Box::new(machine)),
                vm,
                player: page_player::PagePlayer::default(),
            };
            scene.step(0.0);
            scene
        }
        fn step(&mut self, elapsed_seconds: f32) -> page_player::Output {
            self.player
                .step(
                    &mut self.board,
                    &mut self.playback,
                    Step {
                        elapsed_seconds,
                        ..Default::default()
                    },
                    std::slice::from_ref(&self.vm),
                    None,
                )
                .unwrap()
        }
    }

    fn queue(scene: &EventScene, indices: &[i32]) {
        let screens = scene
            .board
            .native_handle()
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens").unwrap());
        let screen = screens
            .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(0))
            .flatten()
            .unwrap();
        let list =
            screen.with_artboard(|b| b.find_handle::<ArtboardComponentList>("items").unwrap());
        for &index in indices {
            let row = list
                .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(index))
                .flatten()
                .unwrap();
            let host = row.with_artboard(|b| b.find_handle::<NestedArtboard>("unit").unwrap());
            machine(&host).with_instance_mut(|m| {
                let event = m.resolve_event(1).expect("authored unit event");
                m.report_event(event, 0.0);
            });
        }
    }
    fn row(scene: &EventScene, index: usize) -> RuntimeOwnedViewModelHandle {
        scene
            .vm
            .list_item_by_property_name_path("screens", 0)
            .unwrap()
            .list_item_by_property_name_path("items", index)
            .unwrap()
    }
    fn native(scene: &mut EventScene) -> Vec<StateMachineReportedEvent> {
        let Playback::StateMachine(m) = &mut scene.playback else {
            unreachable!()
        };
        m.take_reported_events()
    }
    fn context(event: &OwnedPlayerEvent) -> &StateMachineEventContext {
        event
            .context
            .as_ref()
            .expect("owned result must retain the ordinary event context")
    }
    #[test]
    fn repeated_shared_vm_occurrences_keep_emission_order() {
        for page in [false, true] {
            let mut s = EventScene::new(&fixture::event_context_fixture(page));
            let a = row(&s, 0);
            let b = row(&s, 1);
            assert_eq!(a.instance_identity(), b.instance_identity());
            queue(&s, &[1, 0]);
            let expected = native(&mut s);
            assert_eq!(expected.len(), 2);
            assert_eq!(
                expected[0].context().unwrap().view_model_instance_id(),
                Some(a.instance_identity())
            );
            assert_eq!(expected[0].context().unwrap().occurrence()[1].item_index, 1);
            assert_eq!(expected[1].context().unwrap().occurrence()[1].item_index, 0);
            assert_ne!(
                expected[0].context().unwrap().occurrence(),
                expected[1].context().unwrap().occurrence()
            );
            assert_eq!(
                expected[0].context().unwrap().view_model_instance_id(),
                expected[1].context().unwrap().view_model_instance_id()
            );
            assert!(s.step(0.0).events.is_empty());
            queue(&s, &[1, 0]);
            let output = s.step(0.0);
            assert_eq!(output.events.len(), 2);
            for (owned, native) in output.events.iter().zip(&expected) {
                assert_eq!(context(owned), native.context().unwrap());
                assert_eq!(owned.event_local_index, native.event_local_index().unwrap());
                assert_eq!(owned.name.as_deref(), Some(b"item-event".as_slice()));
            }
            assert!(s.step(0.0).events.is_empty());
        }
    }
    #[test]
    fn nested_host_path_and_source_address_survive_scene_drop() {
        let mut s = EventScene::new(&fixture::event_context_fixture(true));
        queue(&s, &[0]);
        let reports = native(&mut s);
        let expected = reports[0].context().unwrap().clone();
        let source = reports[0].source_artboard_global_id().unwrap();
        drop(reports);
        assert_eq!(
            expected.path().len(),
            3,
            "Screen list, item list, then nested unit host"
        );
        s.step(0.0);
        queue(&s, &[0]);
        let output = s.step(0.0);
        drop(s);
        assert_eq!(context(&output.events[0]), &expected);
        assert_eq!(output.events[0].source_artboard_global_id, Some(source));
        assert_ne!(
            Some(expected.path().last().unwrap().artboard_global_id),
            Some(source)
        );
    }
    struct WriteEffects(RuntimeOwnedViewModelHandle, bool);
    impl page_player::Effects for WriteEffects {
        fn begin(&self) -> Box<dyn page_player::EffectTransaction> {
            self.0
                .borrow_mut()
                .set_number_by_property_name("value", 42.0);
            Box::new(WriteTransaction(self.1))
        }
    }
    struct WriteTransaction(bool);
    impl page_player::EffectTransaction for WriteTransaction {
        fn finish(self: Box<Self>) -> Result<Vec<page_player::HostCommand>, ScriptError> {
            if self.0 {
                Err(ScriptError::new("context projection abort"))
            } else {
                Ok(Vec::new())
            }
        }
    }
    #[test]
    fn aborted_publication_drops_context_and_rolls_back_values() {
        let mut s = EventScene::new(&fixture::event_context_fixture(true));
        let row = row(&s, 0);
        queue(&s, &[0]);
        let effects = WriteEffects(row.clone(), false);
        let pending = s
            .player
            .operation(
                &mut s.board,
                &mut s.playback,
                Step::default(),
                None,
                std::slice::from_ref(&s.vm),
                None,
                None,
                false,
                Some(&effects),
                |_| (),
            )
            .unwrap();
        assert_eq!(
            context(&pending.output.as_ref().unwrap().events[0])
                .occurrence()
                .len(),
            2
        );
        assert_eq!(
            row.borrow().number_value_by_property_name("value"),
            Some(42.0)
        );
        drop(pending); // Host publication fails: no Output is published.
        assert_eq!(
            row.borrow().number_value_by_property_name("value"),
            Some(0.0)
        );
        assert!(page_player::is_poisoned(&s.board.native_handle()));
        assert!(
            s.player
                .step(
                    &mut s.board,
                    &mut s.playback,
                    Step::default(),
                    std::slice::from_ref(&s.vm),
                    None
                )
                .is_err()
        );
        // Also fail the actual step after event projection, before any Output
        // can escape. Its graph writes must roll back with the copied contexts.
        let mut s = EventScene::new(&fixture::event_context_fixture(true));
        let answer = self::row(&s, 0);
        queue(&s, &[1, 0]);
        let effects = WriteEffects(answer.clone(), true);
        let error = s
            .player
            .step(
                &mut s.board,
                &mut s.playback,
                Step::default(),
                std::slice::from_ref(&s.vm),
                Some(&effects),
            )
            .err()
            .expect("failed step must publish no Output");
        assert_eq!(error.kind, ErrorKind::RuntimeError);
        assert!(error.message.contains("context projection abort"));
        assert_eq!(
            answer.borrow().number_value_by_property_name("value"),
            Some(0.0)
        );
    }
    #[test]
    fn context_path_and_occurrence_bytes_share_event_budget() {
        // Use the ordinary pointer-context API to supply a large, already owned
        // context without constructing hundreds of thousands of nested hosts.
        let root = std::env::var_os("RIVE_RUNTIME_DIR").unwrap();
        let bytes = std::fs::read(
            std::path::Path::new(&root).join("tests/unit_tests/assets/event_on_listener.riv"),
        )
        .unwrap();
        for occurrence in [false, true] {
            let mut factory = PersistentFactory::new(RecordingFactory::default());
            let file = File::import(
                &bytes,
                RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
                None,
                None,
                None,
            )
            .unwrap();
            let mut board = ArtboardInstance::from_native(file, 0).unwrap();
            let mut m = board.state_machine_instance(0).unwrap();
            m.advance_and_apply(0.0);
            let count = 8 * 1024 * 1024;
            let ctx = if occurrence {
                StateMachineEventContext::new(
                    Vec::new(),
                    vec![
                        RuntimeGeometryHitOccurrence {
                            artboard_global_id: 1,
                            host_local_id: 2,
                            item_index: 0,
                            occurrence_identity: 7
                        };
                        count / std::mem::size_of::<RuntimeGeometryHitOccurrence>() + 1
                    ],
                )
            } else {
                StateMachineEventContext::new(
                    vec![
                        RuntimeGeometryHitPathSegment {
                            artboard_global_id: 1,
                            local_id: 2
                        };
                        count / std::mem::size_of::<RuntimeGeometryHitPathSegment>() + 1
                    ],
                    Vec::new(),
                )
            };
            m.pointer_down_with_event_context(343.0, 116.0, 17, &ctx);
            let reports = m.reported_events();
            assert_eq!(reports.len(), 2);
            assert!(reports.iter().all(|r| r.context() == Some(&ctx)));
            let error = own_reported_events(reports, 0, 0, 0)
                .expect_err("context storage must count toward the existing event byte budget");
            assert_eq!(error.0, ErrorKind::LimitExceeded);
            let player = page_player::PagePlayer::default();
            let mut playback = Playback::StateMachine(Box::new(m));
            let error = player
                .step(&mut board, &mut playback, Step::default(), &[], None)
                .err()
                .expect("oversized context must reject the step without an Output");
            assert_eq!(error.kind, ErrorKind::LimitExceeded);
            assert!(page_player::is_poisoned(&board.native_handle()));
        }
    }
}

#[test]
fn hosted_css_item_rehomes_into_flight_destination_and_home_in_one_advance() {
    for (destination_css, plain_flight) in [(true, false), (false, false), (true, true)] {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &if plain_flight {
                fixture::hosted_css_plain_flight_fixture()
            } else {
                fixture::hosted_css_layout_fixture(destination_css)
            },
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let (_root, screen, machine) = mount_layout_page(&file);
        let slot = |name| {
            screen
                .with_artboard(|b| b.find_handle::<NestedArtboard>(name))
                .unwrap()
        };
        let source = slot("source");
        let flight = slot("flight");
        let destination = slot("destination");
        let original = child(&source).unwrap();
        let geometry = |unit: &RuntimeArtboardInstanceHandle| {
            unit.with_artboard(|b| (b.layout_x(), b.layout_width(), b.layout_height()))
        };
        assert_eq!(geometry(&original), (36.0, 260.0, 108.0));
        let destination_box = geometry(&child(&destination).unwrap());
        let flight_box = geometry(&child(&flight).unwrap());
        if destination_css {
            assert_eq!(destination_box, (56.0, 300.0, 108.0));
        }
        for (from, to, expected) in [
            (&source, &flight, flight_box),
            (&flight, &destination, destination_box),
            (&destination, &source, (36.0, 260.0, 108.0)),
        ] {
            move_live(from, to).unwrap();
            machine.advance_and_apply(0.0);
            assert_eq!(child(to).unwrap().core_handle(), original.core_handle());
            assert_eq!(
                geometry(&original),
                expected,
                "destination CSS {destination_css}"
            );
        }
    }
}

#[test]
fn a_plain_slot_to_native_layout_flight_reflows_at_interpolated_sizes() {
    use source::{
        generated::nested_artboard_layout_base::NestedArtboardLayoutBase,
        layout_component::LayoutComponent,
    };
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::native_layout_flight_reflow_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let (root, screen, machine) = mount_layout_page(&file);
    let slot = |name| {
        screen
            .with_artboard(|b| b.find_handle::<NestedArtboard>(name))
            .unwrap()
    };
    // This fixture's destination is a plain host; its source is a native layout
    // host. Fly from the plain 100 x 80 unit towards the 200 x 120 layout box.
    let from = slot("destination");
    let to = slot("source");
    let flight = slot("flight");
    for (slot, width, height) in [(&to, 200.0, 120.0), (&flight, 60.0, 40.0)] {
        set(
            slot,
            NestedArtboardLayoutBase::INSTANCE_WIDTH_PROPERTY_KEY.into(),
            width,
        );
        set(
            slot,
            NestedArtboardLayoutBase::INSTANCE_HEIGHT_PROPERTY_KEY.into(),
            height,
        );
    }
    machine.advance_and_apply(0.0);
    let start = slot_pose(&from).unwrap().size;
    let end = slot_pose(&to).unwrap().size;
    close(start.x, 100.0);
    close(start.y, 80.0);
    close(end.x, 200.0);
    close(end.y, 120.0);
    let original = child(&from).unwrap();
    let reflow = original
        .with_artboard(|b| b.find_handle::<LayoutComponent>("reflow"))
        .unwrap();
    empty_slot(&flight).unwrap();
    move_live(&from, &flight).unwrap();
    let follower = Flight::new(flight.clone(), from.clone(), to.clone(), SizeMode::Layout).unwrap();
    let mut moves = LiveMoves::default();
    moves.with_flights(&root, |flights| flights.push(Entry::One(follower)));
    for progress in [0.5, 0.25, 0.75] {
        set(&moves.flights(&root)[0].controls(), 62003, progress);
        machine.advance_and_apply(0.0);
        moves.after_advance(&root).unwrap();
        let width = 100.0 + 100.0 * progress;
        let height = 80.0 + 40.0 * progress;
        assert_eq!(
            child(&flight).unwrap().core_handle(),
            original.core_handle()
        );
        // The native route sizes the unit's layout, as upstream's
        // StyleOverrider does; upstream draws and clips from that layout size.
        let (layout_width, layout_height) =
            original.with_artboard(|b| (b.layout_width(), b.layout_height()));
        let size = reflow
            .with_downcast::<LayoutComponent, _>(|c| (c.layout_width(), c.layout_height()))
            .unwrap();
        for (label, actual, expected) in [
            ("layout width", layout_width, width),
            ("layout height", layout_height, height),
            ("child width", size.0, width),
            ("child height", size.1, height),
        ] {
            assert!(
                (actual - expected).abs() < 0.002,
                "progress {progress}, {label}: expected {expected}, got {actual}; layout=({layout_width}, {layout_height}), child={size:?}"
            );
        }
    }
    moves.with_flights(&root, |flights| flights.clear());
    move_live(&flight, &to).unwrap();
    machine.advance_and_apply(0.0);
    assert_eq!(child(&to).unwrap().core_handle(), original.core_handle());
    let landed = original.with_artboard(|b| (b.layout_width(), b.layout_height()));
    close(landed.0, 200.0);
    close(landed.1, 120.0);
    let landed_child = reflow
        .with_downcast::<LayoutComponent, _>(|c| (c.layout_width(), c.layout_height()))
        .unwrap();
    close(landed_child.0, 200.0);
    close(landed_child.1, 120.0);
}

/// A CSS flight slot authored in flow still never pushes: the runtime takes
/// each Screen occurrence's flight slot out of flow before its first layout.
fn in_flow_css_flight_frame(hit_test: bool) {
    use css_layout::{keys::POSITION, record};
    use source::{
        layout_component::LayoutComponent,
        math::{mat2d::Mat2D, vec2d::Vec2D},
        pointer_button::PointerButton,
    };
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::shared_layout_flight_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = file.with_file(|f| f.artboard_named("SharedPage")).unwrap();
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let player = root.default_state_machine_handle().unwrap();
    player.with_instance_mut(|m| m.bind_view_model_instance(model));
    player.advance_and_apply(0.0);
    let list = root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let row = |index| {
        list.with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(index))
            .flatten()
            .unwrap()
    };
    let screen = row(0);
    let sibling = row(1);
    let sibling_machine = list
        .with_downcast::<ArtboardComponentList, _>(|l| l.state_machine_instance(1))
        .flatten()
        .unwrap();
    let slot = |name| {
        screen
            .with_artboard(|b| b.find_handle::<NestedArtboard>(name))
            .unwrap()
    };
    let from = slot("source");
    let to = slot("destination");
    let flight = slot("flight");
    let position = |host: &CoreHandle| {
        let item = host
            .with(|o| record::find_in(o.as_container_component().unwrap().children()))
            .flatten()
            .unwrap();
        record::with_values(&item, |values| values.choice(POSITION)).unwrap()
    };
    assert_eq!(
        position(&flight),
        2,
        "the occurrence's flight slot is absolute"
    );
    let definition = file
        .with_file(|f| f.artboard_named_source("FlightScreen"))
        .unwrap()
        .with_downcast::<Artboard, _>(|b| b.find_handle::<NestedArtboard>("flight"))
        .flatten()
        .unwrap();
    assert_eq!(position(&definition), 0, "the definition stays as imported");
    let spacer = screen
        .with_artboard(|b| b.find_handle::<LayoutComponent>("spacer"))
        .unwrap();
    // The flight is first in the Screen's column; in flow it would push the
    // spacer down by its height, and the sibling Screen with it.
    let rows = || {
        (
            spacer
                .with_downcast::<LayoutComponent, _>(|c| c.layout_y())
                .unwrap(),
            screen.with_artboard(|b| b.layout_height()),
            sibling.with_artboard(|b| b.layout_y()),
        )
    };
    let at_rest = rows();
    close(at_rest.0, 0.0);
    close(at_rest.1, 30.0);
    close(at_rest.2, 30.0);
    child(&to).unwrap().set_size(200.0, 120.0);
    player.advance_and_apply(0.0);
    let original = child(&from).unwrap();
    let unit_machine = machine(&from);
    empty_slot(&flight).unwrap();
    move_live(&from, &flight).unwrap();
    let follower = Flight::new(flight.clone(), from, to, SizeMode::Layout).unwrap();
    let mut moves = LiveMoves::default();
    moves.with_flights(&root, |flights| flights.push(Entry::One(follower)));
    let time = |machine: &RuntimeStateMachineInstanceHandle| {
        machine
            .with_instance_mut(|m| m.current_animation_by_index(0))
            .unwrap()
            .first_animation(|a| a.time())
            .unwrap()
    };
    let sibling_size = sibling.with_artboard(|b| (b.layout_width(), b.layout_height()));
    for (frame, progress) in [0.5, 0.25, 0.75].into_iter().enumerate() {
        set(&moves.flights(&root)[0].controls(), 62003, progress);
        player.advance_and_apply(0.1);
        let before = (time(&unit_machine), time(&sibling_machine));
        moves.after_advance(&root).unwrap();
        assert_eq!(
            before,
            (time(&unit_machine), time(&sibling_machine)),
            "settlement must not advance either machine"
        );
        close(before.0, 0.1 * (frame + 1) as f32);
        close(before.1, 0.1 * (frame + 1) as f32);
        let height = 80.0 + 40.0 * progress;
        close(
            original.with_artboard(|b| b.layout_width()),
            100.0 + 100.0 * progress,
        );
        close(original.with_artboard(|b| b.layout_height()), height);
        let (spacer_y, screen_height, y) = rows();
        close(spacer_y, at_rest.0);
        close(screen_height, at_rest.1);
        close(y, at_rest.2);
        assert_eq!(
            sibling.with_artboard(|b| (b.layout_width(), b.layout_height())),
            sibling_size,
            "the sibling keeps its size"
        );
        if hit_test {
            // In flow, the flight would have pushed the sibling to here.
            let pushed = y + height;
            for (pointer, (point, hits)) in [(y + 1.0, 9.0), (pushed + 1.0, 0.0)]
                .into_iter()
                .enumerate()
            {
                sibling_machine.set_number("hits", 0.0);
                player.with_instance_mut(|m| {
                    m.pointer_down(
                        Vec2D::new(10.0, point),
                        (frame * 2 + pointer) as i32,
                        PointerButton::Primary,
                    )
                });
                close(
                    sibling_machine.with_instance(|m| m.get_number("hits").unwrap().value()),
                    hits,
                );
            }
        } else {
            // This is the exact cached transform consumed by list drawing.
            let transform = list
                .with_downcast::<ArtboardComponentList, _>(|l| l.draw_transform_for_index(1))
                .flatten()
                .unwrap();
            close(transform[5], y);
            factory.borrow_mut().clear();
            let mut renderer = factory.borrow().make_renderer();
            root.draw(&mut renderer);
            let stream = factory.borrow().stream();
            let mut matrix = Mat2D::identity();
            let mut stack = Vec::new();
            let mut drawn_y = None;
            for line in stream.lines() {
                if line == "save" {
                    stack.push(matrix);
                } else if line == "restore" {
                    matrix = stack.pop().unwrap();
                } else if let Some(values) = line
                    .strip_prefix("transform matrix=[")
                    .and_then(|s| s.strip_suffix(']'))
                {
                    let mut local = Mat2D::identity();
                    for (i, value) in values.split(',').enumerate() {
                        local[i] = value.parse().unwrap();
                    }
                    matrix *= local;
                } else if line.starts_with("drawPath ") {
                    assert!(drawn_y.is_none(), "only the sibling paints");
                    drawn_y = Some((matrix * Vec2D::new(0.0, 0.0)).y);
                }
            }
            close(drawn_y.unwrap(), y);
        }
    }
}

#[test]
fn an_in_flow_css_flight_slot_resizes_without_moving_the_next_rows_drawing() {
    in_flow_css_flight_frame(false);
}

#[test]
fn an_in_flow_css_flight_slot_resizes_without_moving_the_next_rows_hit_testing() {
    in_flow_css_flight_frame(true);
}

/// An item record written after the first layout through the ordinary
/// setter, the route a binding or animation takes, moves the hosted unit's
/// siblings in a Screen that is a row of a list, in and out of flow
/// (UNIV-5601).
#[test]
fn an_item_record_write_moves_a_list_row_screens_siblings() {
    use css_layout::{keys::POSITION, record};
    use source::layout_component::LayoutComponent;
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::shared_layout_flight_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = file.with_file(|f| f.artboard_named("SharedPage")).unwrap();
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let player = root.default_state_machine_handle().unwrap();
    player.with_instance_mut(|m| m.bind_view_model_instance(model));
    player.advance_and_apply(0.0);
    let list = root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap();
    let row = |index| {
        list.with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(index))
            .flatten()
            .unwrap()
    };
    let (screen, sibling) = (row(0), row(1));
    let item = screen
        .with_artboard(|b| b.find_handle::<NestedArtboard>("flight"))
        .unwrap()
        .with(|o| record::find_in(o.as_container_component().unwrap().children()))
        .flatten()
        .unwrap();
    let spacer = screen
        .with_artboard(|b| b.find_handle::<LayoutComponent>("spacer"))
        .unwrap();
    let rows = || {
        (
            spacer
                .with_downcast::<LayoutComponent, _>(|c| c.layout_y())
                .unwrap(),
            screen.with_artboard(|b| b.layout_height()),
            sibling.with_artboard(|b| b.layout_y()),
        )
    };
    // The runtime took the flight slot out of flow before the first layout.
    let (spacer_y, height, y) = rows();
    close(spacer_y, 0.0);
    close(height, 30.0);
    close(y, 30.0);
    assert!(CoreRegistry::set_uint_handle(&item, POSITION as i32, 0));
    player.advance_and_apply(0.0);
    // Back in flow, the 40-high slot pushes the spacer down, its Screen grows,
    // and the next row follows.
    let (spacer_y, height, y) = rows();
    close(spacer_y, 40.0);
    close(height, 70.0);
    close(y, 70.0);
    // And out of flow again, everything returns.
    assert!(CoreRegistry::set_uint_handle(&item, POSITION as i32, 2));
    player.advance_and_apply(0.0);
    let (spacer_y, height, y) = rows();
    close(spacer_y, 0.0);
    close(height, 30.0);
    close(y, 30.0);
}

fn a_flight_aims_at_the_settled_destination(progresses: &[f32], two_flights: bool) {
    use source::{
        layout_component::LayoutComponent,
        math::{mat2d::Mat2D, vec2d::Vec2D},
    };
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &if two_flights {
            fixture::shared_layout_two_flight_endpoint_fixture()
        } else {
            fixture::shared_layout_endpoint_fixture()
        },
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = file.with_file(|f| f.artboard_named("SharedPage")).unwrap();
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let player = root.default_state_machine_handle().unwrap();
    player.with_instance_mut(|m| m.bind_view_model_instance(model));
    player.advance_and_apply(0.0);
    let rows = screens(&root).unwrap();
    let screen = &rows["FlightScreen"];
    let destination = &rows["Sibling"];
    let slot = |row: &RuntimeArtboardInstanceHandle, name| {
        row.with_artboard(|b| b.find_handle::<NestedArtboard>(name))
            .unwrap()
    };
    let from = slot(screen, "source");
    let to = slot(destination, "landing");
    let flight = slot(screen, "flight");
    empty_slot(&slot(screen, "destination")).unwrap();
    set(&from, 13, 30.0);
    set(&to, 13, 200.0);
    set(&to, 14, 20.0);
    child(&to).unwrap().set_size(200.0, 120.0);
    if two_flights {
        let from2 = slot(screen, "source2");
        let to2 = slot(destination, "landing2");
        child(&from2).unwrap().set_size(80.0, 40.0);
        child(&to2).unwrap().set_size(300.0, 160.0);
        let color = child(&from2)
            .unwrap()
            .with_artboard(|b| {
                b.find_handle::<source::shapes::paint::solid_color::SolidColor>("markerColor")
            })
            .unwrap();
        assert!(CoreRegistry::set_color_handle(
            &color,
            37,
            0xffabcdef_u32 as i32
        ));
    }
    player.advance_and_apply(0.0);
    let rows_at_rest = (
        screen.with_artboard(|b| b.layout_height()),
        destination.with_artboard(|b| b.layout_y()),
    );
    let original = child(&from).unwrap();
    let playback = machine(&from);
    let reflow = original
        .with_artboard(|b| b.find_handle::<LayoutComponent>("reflow"))
        .unwrap();
    let time = || {
        playback
            .with_instance_mut(|m| m.current_animation_by_index(0))
            .unwrap()
            .first_animation(|a| a.time())
            .unwrap()
    };
    let draw = || {
        factory.borrow_mut().clear();
        let mut renderer = factory.borrow().make_renderer();
        root.draw(&mut renderer);
        let stream = factory.borrow().stream();
        let mut matrix = Mat2D::identity();
        let mut stack = Vec::new();
        let mut points = Vec::new();
        for line in stream.lines() {
            if line == "save" {
                stack.push(matrix);
            } else if line == "restore" {
                matrix = stack.pop().unwrap();
            } else if let Some(values) = line
                .strip_prefix("transform matrix=[")
                .and_then(|s| s.strip_suffix(']'))
            {
                let mut local = Mat2D::identity();
                for (i, value) in values.split(',').enumerate() {
                    local[i] = value.parse().unwrap();
                }
                matrix *= local;
            } else if line.starts_with("drawPath ") && line.contains("color=0xffedcba9") {
                assert!(points.is_empty(), "only the live unit's marker paints");
                for point in line
                    .split("points=[")
                    .nth(1)
                    .unwrap()
                    .split(']')
                    .next()
                    .unwrap()
                    .split("),(")
                {
                    let (x, y) = point.trim_matches(['(', ')']).split_once(',').unwrap();
                    points.push(matrix * Vec2D::new(x.parse().unwrap(), y.parse().unwrap()));
                }
            }
        }
        assert!(!points.is_empty());
        Vec2D::new(
            points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min),
            points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min),
        )
    };
    let mut moves = LiveMoves::default();
    moves.step_begin(&root, "Sibling", Some("shared")).unwrap();
    for (frame, progress) in progresses.iter().copied().enumerate() {
        for follower in moves.flights(&root) {
            set(&follower.controls(), 62003, progress);
        }
        player.advance_and_apply(0.1);
        let before = time();
        let solves = Artboard::layout_pass_count();
        hosted_scene::measure_walks(true);
        moves.after_advance(&root).unwrap();
        assert_eq!(
            Artboard::layout_pass_count() - solves,
            1,
            "resize solves the shared tree once; pose settlement must not solve again"
        );
        let visits = hosted_scene::measure_walks(false);
        assert_eq!(visits.advance, 0);
        close(time(), before);
        close(time(), 0.1 * (frame + 1) as f32);
        assert_eq!(
            child(&flight).unwrap().core_handle(),
            original.core_handle()
        );
        let width = 100.0 + 100.0 * progress;
        let height = 80.0 + 40.0 * progress;
        close(original.with_artboard(|b| b.layout_width()), width);
        close(
            reflow
                .with_downcast::<LayoutComponent, _>(|b| b.layout_width())
                .unwrap(),
            width,
        );
        close(
            reflow
                .with_downcast::<LayoutComponent, _>(|b| b.layout_height())
                .unwrap(),
            height,
        );
        // Flight slots are out of flow, so neither row moves.
        close(screen.with_artboard(|b| b.layout_height()), rows_at_rest.0);
        close(destination.with_artboard(|b| b.layout_y()), rows_at_rest.1);
        let a = slot_pose(&from).unwrap().transform;
        let b = slot_pose(&to).unwrap().transform;
        let actual = draw();
        close(actual.x, a[4] + (b[4] - a[4]) * progress);
        close(actual.y, a[5] + (b[5] - a[5]) * progress);
        let solves = Artboard::layout_pass_count();
        moves.after_advance(&root).unwrap();
        assert_eq!(
            Artboard::layout_pass_count(),
            solves,
            "reprojecting a pose must leave layout clean"
        );
        close(draw().y, actual.y);
        close(time(), before);
    }
    assert_eq!(*progresses.last().unwrap(), 1.0);
    let before_landing = draw();
    let before_time = time();
    moves.step_end(&root, "Sibling", Some("shared")).unwrap();
    let after_landing = draw();
    close(after_landing.x, before_landing.x);
    close(after_landing.y, before_landing.y);
    close(time(), before_time);
    assert_eq!(child(&to).unwrap().core_handle(), original.core_handle());
    close(original.with_artboard(|b| b.layout_width()), 200.0);
    close(original.with_artboard(|b| b.layout_height()), 120.0);
}

#[test]
fn a_resizing_flight_reaches_its_settled_destination_without_a_landing_jump() {
    a_flight_aims_at_the_settled_destination(&[1.0], false);
}

#[test]
fn a_resizing_flight_interpolates_between_settled_endpoints_of_the_same_frame() {
    a_flight_aims_at_the_settled_destination(&[0.25, 0.5, 0.75, 1.0], false);
}

#[test]
fn two_resizing_flights_follow_their_own_endpoints_in_the_same_frame() {
    a_flight_aims_at_the_settled_destination(&[0.25, 0.5, 0.75, 1.0], true);
}

fn css_follower_owns_box(source_kind: u8, mode: SizeMode) {
    use css_layout::{keys::*, record};
    use source::{
        layout_component::LayoutComponent,
        math::{mat2d::Mat2D, vec2d::Vec2D},
        shapes::paint::solid_color::SolidColor,
    };
    // The follower writes only the flight slot's size; authored placement on
    // the flight record is replaced by the pose override.
    for case in ["size", "placement"] {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &fixture::css_follower_fixture(source_kind),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let (root, screen, player) = mount_layout_page(&file);
        let slot = |name| {
            screen
                .with_artboard(|b| b.find_handle::<NestedArtboard>(name))
                .unwrap()
        };
        let from = slot("source");
        let to = slot("destination");
        let flight = slot("flight");
        let css = |slot: &CoreHandle| {
            slot.with(|o| record::find_in(o.as_container_component().unwrap().children()))
                .flatten()
                .unwrap()
        };
        let item = css(&flight);
        let enum_value =
            |key, value| assert!(CoreRegistry::set_uint_handle(&item, i32::from(key), value));
        let number = |key, value| set(&item, i32::from(key), value);
        if case == "placement" {
            enum_value(WIDTH_KIND, 0);
            enum_value(HEIGHT_KIND, 0);
            enum_value(INSET_TOP_KIND, 1);
            number(INSET_TOP_KIND + 1, 500.0);
            number(MARGIN_TOP_KIND + 1, 70.0);
        }
        player.advance_and_apply(0.0);
        let original = child(&from).unwrap();
        let reflow = original
            .with_artboard(|b| b.find_handle::<LayoutComponent>("reflow"))
            .unwrap();
        let ink = original
            .with_artboard(|b| b.find_handle::<SolidColor>("ink"))
            .unwrap();
        assert!(CoreRegistry::set_color_handle(
            &ink,
            37,
            0xff007766_u32 as i32
        ));
        player.advance_and_apply(0.0);
        let style = |unit: &RuntimeArtboardInstanceHandle| {
            unit.with_artboard(|b| b.base.base.layout_style().clone())
        };
        let home_style = style(&original);
        let destination_style = style(&child(&to).unwrap());
        close(slot_pose(&from).unwrap().size.x, 100.0);
        close(slot_pose(&to).unwrap().size.x, 200.0);
        let follower = Flight::new(flight.clone(), from.clone(), to.clone(), mode).unwrap();
        empty_slot(&flight).unwrap();
        move_live(&from, &flight).unwrap();
        let mut moves = LiveMoves::default();
        moves.with_flights(&root, |flights| flights.push(Entry::One(follower)));
        for progress in [0.5, 0.25, 0.75, 1.0] {
            set(&moves.flights(&root)[0].controls(), 62003, progress);
            player.advance_and_apply(0.0);
            moves.after_advance(&root).unwrap();
            let interpolated = (100.0 + 100.0 * progress, 80.0 + 40.0 * progress);
            let expected = if mode == SizeMode::Layout {
                interpolated
            } else {
                (100.0, 80.0)
            };
            assert_eq!(
                child(&flight).unwrap().core_handle(),
                original.core_handle()
            );
            let flight_box = record::with_values(&item, |values| {
                (
                    values.choice(WIDTH_KIND),
                    values.number(WIDTH_KIND + 1),
                    values.choice(HEIGHT_KIND),
                    values.number(HEIGHT_KIND + 1),
                    values.choice(POSITION),
                )
            })
            .unwrap();
            assert_eq!(
                (flight_box.0, flight_box.2, flight_box.4),
                (1, 1, 2),
                "source {source_kind}, {mode:?}, {case}: the flight record holds a length box out of flow"
            );
            close(flight_box.1, expected.0);
            close(flight_box.3, expected.1);
            for (label, actual, expected) in [
                (
                    "width",
                    original.with_artboard(|b| b.layout_width()),
                    expected.0,
                ),
                (
                    "height",
                    original.with_artboard(|b| b.layout_height()),
                    expected.1,
                ),
                (
                    "child width",
                    reflow.with_downcast::<LayoutComponent, _>(b_width).unwrap(),
                    expected.0,
                ),
                (
                    "child height",
                    reflow
                        .with_downcast::<LayoutComponent, _>(|b| b.layout_height())
                        .unwrap(),
                    expected.1,
                ),
            ] {
                assert!(
                    (actual - expected).abs() < 0.002,
                    "source {source_kind}, {mode:?}, {case}, t={progress}: {label} expected {expected}, got {actual}"
                );
            }
            factory.borrow_mut().clear();
            let mut renderer = factory.borrow().make_renderer();
            root.draw(&mut renderer);
            let stream = factory.borrow().stream();
            let mut matrix = Mat2D::identity();
            let mut stack = Vec::new();
            let mut points = Vec::new();
            for line in stream.lines() {
                if line == "save" {
                    stack.push(matrix);
                } else if line == "restore" {
                    matrix = stack.pop().unwrap();
                } else if let Some(values) = line
                    .strip_prefix("transform matrix=[")
                    .and_then(|s| s.strip_suffix(']'))
                {
                    let mut local = Mat2D::identity();
                    for (i, v) in values.split(',').enumerate() {
                        local[i] = v.parse().unwrap();
                    }
                    matrix *= local;
                } else if line.starts_with("drawPath ") && line.contains("color=0xff007766") {
                    for point in line
                        .split("points=[")
                        .nth(1)
                        .unwrap()
                        .split(']')
                        .next()
                        .unwrap()
                        .split("),(")
                    {
                        let (x, y) = point.trim_matches(['(', ')']).split_once(',').unwrap();
                        points.push(matrix * Vec2D::new(x.parse().unwrap(), y.parse().unwrap()));
                    }
                }
            }
            assert!(!points.is_empty());
            let extent = |x: bool| {
                let values = points
                    .iter()
                    .map(|p| if x { p.x } else { p.y })
                    .collect::<Vec<_>>();
                values.iter().copied().fold(f32::NEG_INFINITY, f32::max)
                    - values.iter().copied().fold(f32::INFINITY, f32::min)
            };
            // Scale mode lays out the departure box and scales it, so both
            // modes draw the interpolated size.
            close(extent(true), interpolated.0);
            close(extent(false), interpolated.1);
        }
        moves.with_flights(&root, |flights| flights.clear());
        move_live(&flight, &to).unwrap();
        player.advance_and_apply(0.0);
        assert_eq!(child(&to).unwrap().core_handle(), original.core_handle());
        assert_eq!(
            style(&original),
            destination_style,
            "destination restores its whole style, {case}"
        );
        close(original.with_artboard(|b| b.layout_width()), 200.0);
        close(original.with_artboard(|b| b.layout_height()), 120.0);
        move_live(&to, &from).unwrap();
        player.advance_and_apply(0.0);
        assert_eq!(child(&from).unwrap().core_handle(), original.core_handle());
        assert_eq!(
            style(&original),
            home_style,
            "home restores its whole style, {case}"
        );
        close(original.with_artboard(|b| b.layout_width()), 100.0);
        close(original.with_artboard(|b| b.layout_height()), 80.0);

        // Cancellation returns directly from a flight, without passing through
        // the destination's record to accidentally erase transient item fields.
        let follower = Flight::new(flight.clone(), from.clone(), to.clone(), mode).unwrap();
        move_live(&from, &flight).unwrap();
        moves.with_flights(&root, |flights| flights.push(Entry::One(follower)));
        set(&moves.flights(&root)[0].controls(), 62003, 0.5);
        player.advance_and_apply(0.0);
        moves.after_advance(&root).unwrap();
        moves.with_flights(&root, |flights| flights.clear());
        move_live(&flight, &from).unwrap();
        player.advance_and_apply(0.0);
        assert_eq!(child(&from).unwrap().core_handle(), original.core_handle());
        assert_eq!(
            style(&original),
            home_style,
            "direct return restores the whole home style, {case}"
        );
        close(original.with_artboard(|b| b.layout_width()), 100.0);
        close(original.with_artboard(|b| b.layout_height()), 80.0);
    }
    fn b_width(b: &source::layout_component::LayoutComponent) -> f32 {
        b.layout_width()
    }
}
#[test]
fn hosted_css_follower_record_free_source_layout() {
    css_follower_owns_box(0, SizeMode::Layout);
}
#[test]
fn hosted_css_follower_record_free_source_scale() {
    css_follower_owns_box(0, SizeMode::Scale);
}
#[test]
fn hosted_css_follower_record_source_layout() {
    css_follower_owns_box(1, SizeMode::Layout);
}
#[test]
fn hosted_css_follower_record_source_scale() {
    css_follower_owns_box(1, SizeMode::Scale);
}
#[test]
fn hosted_css_follower_native_overrides_source() {
    for mode in [SizeMode::Layout, SizeMode::Scale] {
        css_follower_owns_box(2, mode);
    }
}

/// One path `board` drew: its stream line, its final points without a closing
/// repeat of the first vertex, and the final points of the clip set before it.
struct DrawnPath {
    line: String,
    points: Vec<source::math::vec2d::Vec2D>,
    clip: Vec<source::math::vec2d::Vec2D>,
}

/// The paths drawn by `board` whose stream lines are `wanted`.
fn drawn_paths(
    factory: &PersistentFactory<RecordingFactory>,
    board: &RuntimeArtboardInstanceHandle,
    wanted: impl Fn(&str) -> bool,
) -> Vec<DrawnPath> {
    use source::math::{mat2d::Mat2D, vec2d::Vec2D};
    let final_points = |line: &str, matrix: Mat2D| {
        let mut points = Vec::new();
        for point in line
            .split("points=[")
            .nth(1)
            .unwrap()
            .split(']')
            .next()
            .unwrap()
            .split("),(")
            .filter(|point| !point.is_empty())
        {
            let (x, y) = point.trim_matches(['(', ')']).split_once(',').unwrap();
            points.push(matrix * Vec2D::new(x.parse().unwrap(), y.parse().unwrap()));
        }
        points
    };
    factory.borrow_mut().clear();
    let mut renderer = factory.borrow().make_renderer();
    board.draw(&mut renderer);
    let stream = factory.borrow().stream();
    let mut matrix = Mat2D::identity();
    let mut stack = Vec::new();
    let mut clip = None;
    let mut draws = Vec::new();
    for line in stream.lines() {
        if line == "save" {
            stack.push(matrix);
        } else if line == "restore" {
            matrix = stack.pop().unwrap();
        } else if let Some(values) = line
            .strip_prefix("transform matrix=[")
            .and_then(|s| s.strip_suffix(']'))
        {
            let mut local = Mat2D::identity();
            for (i, v) in values.split(',').enumerate() {
                local[i] = v.parse().unwrap();
            }
            matrix *= local;
        } else if line.starts_with("clipPath ") {
            clip = Some((line, matrix));
        } else if line.starts_with("drawPath ") && wanted(line) {
            let mut points = final_points(line, matrix);
            if points.len() > 1 && points.first() == points.last() {
                points.pop();
            }
            draws.push(DrawnPath {
                line: line.to_owned(),
                points,
                clip: clip.map_or_else(Vec::new, |(line, matrix)| final_points(line, matrix)),
            });
        }
    }
    draws
}

/// The ink of `color` drawn by `board`, as final points, without a closing
/// repeat of the first vertex.
fn ink_points(
    factory: &PersistentFactory<RecordingFactory>,
    board: &RuntimeArtboardInstanceHandle,
    color: &str,
) -> Vec<source::math::vec2d::Vec2D> {
    let mut draws = drawn_paths(factory, board, |line| line.contains(color));
    assert!(draws.len() <= 1, "one draw of {color}");
    draws.pop().map_or_else(Vec::new, |draw| draw.points)
}

fn assert_corners(
    drawn: &[source::math::vec2d::Vec2D],
    expected: &[source::math::vec2d::Vec2D],
    label: &str,
) {
    assert_eq!(drawn.len(), expected.len(), "{label}: {drawn:?}");
    for corner in expected {
        assert!(
            drawn
                .iter()
                .any(|p| (p.x - corner.x).abs() < 0.01 && (p.y - corner.y).abs() < 0.01),
            "{label}: no drawn corner at {corner:?} in {drawn:?}"
        );
    }
}

/// CSS draws a transformed box as translate · rotate · scale about its
/// origin, and a flight's size change belongs to the box, not to that
/// transform: the oracle is each frame's blended rectangle (a mix of the two
/// rest rectangles, measured by drawing the unposed unit in each slot),
/// slid by its offset and turned about its center.
fn turned_unit_flies_as_a_css_box(mode: SizeMode) {
    use source::{math::vec2d::Vec2D, shapes::paint::solid_color::SolidColor};
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::css_follower_turned_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let (root, screen, player) = mount_layout_page(&file);
    let slot = |name| {
        screen
            .with_artboard(|b| b.find_handle::<NestedArtboard>(name))
            .unwrap()
    };
    let (from, to, flight) = (slot("source"), slot("destination"), slot("flight"));
    player.advance_and_apply(0.0);
    let unit = child(&from).unwrap();
    let ink = unit
        .with_artboard(|b| b.find_handle::<SolidColor>("ink"))
        .unwrap();
    assert!(CoreRegistry::set_color_handle(
        &ink,
        37,
        0xff007766_u32 as i32
    ));
    let draw = || ink_points(&factory, &root, "color=0xff007766");
    let rect = |points: Vec<Vec2D>| {
        let x = points.iter().map(|p| p.x);
        let y = points.iter().map(|p| p.y);
        (
            Vec2D::new(
                x.clone().fold(f32::INFINITY, f32::min),
                y.clone().fold(f32::INFINITY, f32::min),
            ),
            Vec2D::new(
                x.fold(f32::NEG_INFINITY, f32::max),
                y.fold(f32::NEG_INFINITY, f32::max),
            ),
        )
    };
    // The rest rectangles, unposed.
    set(&unit.core_handle(), 15, 0.0);
    player.advance_and_apply(0.0);
    let a = rect(draw());
    move_live(&from, &to).unwrap();
    player.advance_and_apply(0.0);
    let b = rect(draw());
    move_live(&to, &from).unwrap();
    player.advance_and_apply(0.0);
    close(b.1.x - b.0.x, 200.0);
    close(b.1.y - b.0.y, 120.0);
    close(a.1.x - a.0.x, 100.0);
    close(a.1.y - a.0.y, 80.0);
    let turn = source::math::mat2d::Mat2D::from_rotation(0.35);
    // Turned by the file's own turn, then also slid by a written offset.
    set(&unit.core_handle(), 15, 0.35);
    for slide in [0.0, 10.0] {
        let label = format!("{mode:?}, slid {slide}");
        set(&unit.core_handle(), 13, slide);
        player.advance_and_apply(0.0);
        let expected = |t: f32| {
            let lo = a.0 + (b.0 - a.0) * t;
            let hi = a.1 + (b.1 - a.1) * t;
            let center = Vec2D::new((lo.x + hi.x) * 0.5, (lo.y + hi.y) * 0.5);
            [lo, Vec2D::new(hi.x, lo.y), hi, Vec2D::new(lo.x, hi.y)]
                .map(|corner| center + Vec2D::new(slide, 0.0) + turn * (corner - center))
        };
        assert_corners(&draw(), &expected(0.0), &format!("{label}: at rest"));
        let follower = Flight::new(flight.clone(), from.clone(), to.clone(), mode).unwrap();
        empty_slot(&flight).unwrap();
        move_live(&from, &flight).unwrap();
        let mut moves = LiveMoves::default();
        moves.with_flights(&root, |flights| flights.push(Entry::One(follower)));
        let last = [0.25, 0.5, 0.75, 1.0]
            .map(|progress| {
                set(&moves.flights(&root)[0].controls(), 62003, progress);
                player.advance_and_apply(0.0);
                moves.after_advance(&root).unwrap();
                let drawn = draw();
                assert_corners(
                    &drawn,
                    &expected(progress),
                    &format!("{label}, t={progress}"),
                );
                drawn
            })
            .into_iter()
            .last()
            .unwrap();
        // Land through a step's own landing, with no advance between the last
        // frame and the draw.
        moves.land_flights(&root).unwrap();
        assert_eq!(child(&to).unwrap().core_handle(), unit.core_handle());
        let landed = draw();
        assert_corners(
            &landed,
            &last,
            &format!("{label}: landing keeps the last frame"),
        );
        assert_corners(
            &landed,
            &expected(1.0),
            &format!("{label}: landed in the destination"),
        );
        player.advance_and_apply(0.0);
        assert_corners(
            &draw(),
            &expected(1.0),
            &format!("{label}: at rest in the destination"),
        );
        move_live(&to, &from).unwrap();
        player.advance_and_apply(0.0);
    }
}
#[test]
fn a_flight_stretches_under_the_frames_own_slide_and_turn_layout() {
    turned_unit_flies_as_a_css_box(SizeMode::Layout);
}
#[test]
fn a_flight_stretches_under_the_frames_own_slide_and_turn_scale() {
    turned_unit_flies_as_a_css_box(SizeMode::Scale);
}

/// The stand-in that holds a unit's place while it is away places like the
/// unit: the slot it leaves keeps its pose at rest, at takeoff and whenever
/// it is placed again.
#[test]
fn a_vacated_slot_keeps_its_pose() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::css_follower_turned_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let (_root, screen, player) = mount_layout_page(&file);
    let slot = |name| {
        screen
            .with_artboard(|b| b.find_handle::<NestedArtboard>(name))
            .unwrap()
    };
    let (from, to) = (slot("source"), slot("destination"));
    player.advance_and_apply(0.0);
    let pose = |label: &str| {
        let pose = slot_pose(&from).unwrap();
        (label.to_owned(), pose.transform, pose.size, pose.opacity)
    };
    let at_rest = pose("at rest");
    move_live(&from, &to).unwrap();
    player.advance_and_apply(0.0);
    assert!(child(&from).is_none(), "the unit left");
    let after = pose("just after it leaves");
    from.with_mut(|o| {
        o.as_nested_artboard_mut()
            .unwrap()
            .mark_host_transform_dirty()
    })
    .unwrap();
    player.advance_and_apply(0.0);
    let placed_again = pose("placed again");
    for vacated in [after, placed_again] {
        assert_eq!(
            (vacated.1, vacated.2, vacated.3),
            (at_rest.1, at_rest.2, at_rest.3),
            "{}",
            vacated.0
        );
    }
}

/// CSS 2.1 Appendix E, as `css_stacking_slot_takes_its_frames_layer`: a
/// turned box is a stacking context and paints over later flow boxes. A
/// unit transported between Screens takes that layer in the Screen it
/// reaches, and both Screens re-sort.
#[test]
fn css_stacking_slot_takes_its_frames_layer_across_screens() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::css_two_screen_stacking_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = file
        .with_file(|f| f.artboard_named("StackingPage"))
        .unwrap();
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let player = root.default_state_machine_handle().unwrap();
    player.with_instance_mut(|m| m.bind_view_model_instance(model));
    player.advance_and_apply(0.0);
    let rows = screens(&root).unwrap();
    let (a, b) = (rows["A"].clone(), rows["B"].clone());
    let slot = |row: &RuntimeArtboardInstanceHandle| {
        row.with_artboard(|r| r.find_handle::<NestedArtboard>("slot"))
            .unwrap()
    };
    let (from, to) = (slot(&a), slot(&b));
    const UNIT: &str = "color=0xffcc3333";
    const LATER: &str = "color=0xff3333cc";
    // The colors a Screen paints, bottom first.
    let order = |row: &RuntimeArtboardInstanceHandle| {
        factory.borrow_mut().clear();
        let mut renderer = factory.borrow().make_renderer();
        row.draw(&mut renderer);
        let stream = factory.borrow().stream();
        let mut found: Vec<_> = [UNIT, LATER]
            .into_iter()
            .filter_map(|color| {
                stream
                    .lines()
                    .position(|line| line.starts_with("drawPath ") && line.contains(color))
                    .map(|at| (at, color))
            })
            .collect();
        found.sort();
        found
            .into_iter()
            .map(|(_, color)| color)
            .collect::<Vec<_>>()
    };
    let sorts =
        |row: &RuntimeArtboardInstanceHandle| row.with_artboard(|r| r.draw_order_change_counter());
    empty_slot(&to).unwrap();
    player.advance_and_apply(0.0);
    assert_eq!(order(&a), [LATER, UNIT], "A at rest");
    assert_eq!(order(&b), [LATER], "B empty");
    let (left, reached) = (sorts(&a), sorts(&b));
    move_live(&from, &to).unwrap();
    player.advance_and_apply(0.0);
    assert_eq!(order(&a), [LATER], "A after the move");
    assert_eq!(order(&b), [LATER, UNIT], "B after the move");
    assert_ne!(sorts(&a), left, "the Screen the unit left re-sorts");
    assert_ne!(sorts(&b), reached, "the Screen the unit reached re-sorts");
    move_live(&to, &from).unwrap();
    player.advance_and_apply(0.0);
    assert_eq!(order(&a), [LATER, UNIT], "A after the return");
    assert_eq!(order(&b), [LATER], "B after the return");
}

/// The colors of `css_two_version_corner_fixture`'s three units.
const UNIT_A: &str = "color=0xffcc3333";
const UNIT_B: &str = "color=0xff3333cc";
const UNIT_C: &str = "color=0xff33cc33";

/// A drawn rectangle, as its least and greatest corners.
type Rect = (source::math::vec2d::Vec2D, source::math::vec2d::Vec2D);

/// The page of `css_two_version_corner_fixture` and its variants, mounted
/// as `Scene::load` mounts a published page. `s` is unit a's home on A,
/// `d` unit b's home on B and `f` unit a's flight slot on B. Preparing
/// empties the fresh flight slots.
fn corner_scene(bytes: &[u8]) -> Scene {
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
    let model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(model));
    machine.advance_and_apply(0.0);
    let rows = screens(&root).unwrap();
    let (a, b) = (rows["A"].clone(), rows["B"].clone());
    let mut moves = LiveMoves::default();
    moves.prepare(&root).unwrap();
    Scene {
        _file: file,
        factory,
        root,
        machine,
        s: corner_slot(&a, "a"),
        d: corner_slot(&b, "b"),
        f: corner_slot(&b, "a flight"),
        a,
        b,
        moves,
    }
}
fn corner_slot(row: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    row.with_artboard(|r| r.find_handle::<NestedArtboard>(name))
        .unwrap()
}
fn drawn_rect(points: &[source::math::vec2d::Vec2D]) -> Rect {
    use source::math::vec2d::Vec2D;
    assert!(!points.is_empty(), "nothing drawn");
    points.iter().fold((points[0], points[0]), |(lo, hi), p| {
        (
            Vec2D::new(lo.x.min(p.x), lo.y.min(p.y)),
            Vec2D::new(hi.x.max(p.x), hi.y.max(p.y)),
        )
    })
}
fn mix_rect(a: Rect, b: Rect, t: f32) -> Rect {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
}
/// The eight points where a CSS rounded rectangle's straight edges meet its
/// corners: `radius` in from each corner, along both edges.
fn tangent_points((lo, hi): Rect, radius: f32) -> [source::math::vec2d::Vec2D; 8] {
    use source::math::vec2d::Vec2D;
    [
        Vec2D::new(lo.x + radius, lo.y),
        Vec2D::new(hi.x - radius, lo.y),
        Vec2D::new(hi.x, lo.y + radius),
        Vec2D::new(hi.x, hi.y - radius),
        Vec2D::new(hi.x - radius, hi.y),
        Vec2D::new(lo.x + radius, hi.y),
        Vec2D::new(lo.x, hi.y - radius),
        Vec2D::new(lo.x, lo.y + radius),
    ]
}
/// `drawn` outlines `rect` with `radius` point corners: it spans the
/// rectangle and passes through all eight of its tangent points, and
/// through none of any `wrong` radius's.
fn assert_outline(
    drawn: &[source::math::vec2d::Vec2D],
    rect: Rect,
    radius: f32,
    wrong: &[f32],
    label: &str,
) {
    use source::math::vec2d::Vec2D;
    let near = |p: &Vec2D, q: &Vec2D| (p.x - q.x).abs() < 0.01 && (p.y - q.y).abs() < 0.01;
    let spans = drawn_rect(drawn);
    assert!(
        near(&spans.0, &rect.0) && near(&spans.1, &rect.1),
        "{label}: drawn over {spans:?}, expected {rect:?}"
    );
    for point in tangent_points(rect, radius) {
        assert!(
            drawn.iter().any(|p| near(p, &point)),
            "{label}: no {radius} pt corner point at {point:?} in {drawn:?}"
        );
    }
    for &other in wrong.iter().filter(|&&other| other != radius) {
        for point in tangent_points(rect, other) {
            assert!(
                !drawn.iter().any(|p| near(p, &point)),
                "{label}: a {other} pt corner point at {point:?} in {drawn:?}"
            );
        }
    }
}
/// Units a and b at rest, each drawn at home with its own corners.
fn corner_rest_rects(s: &Scene) -> (Rect, Rect) {
    let (a, b) = (
        ink_points(&s.factory, &s.root, UNIT_A),
        ink_points(&s.factory, &s.root, UNIT_B),
    );
    let rects = (drawn_rect(&a), drawn_rect(&b));
    for (rect, size) in [(rects.0, (80.0, 60.0)), (rects.1, (160.0, 120.0))] {
        close(rect.1.x - rect.0.x, size.0);
        close(rect.1.y - rect.0.y, size.1);
    }
    assert_outline(&a, rects.0, 4.0, &[8.0], "a at rest");
    assert_outline(&b, rects.1, 8.0, &[4.0], "b at rest");
    rects
}

/// A flight's corner controls replace a unit's own corners only when it
/// carries all four (docs/live-moves.md). Otherwise each version draws its
/// own corners in flight, as before.
#[test]
fn corner_controls_guard_authored_corners() {
    // (a) No corner keys, all -1: each version draws its own corners, 4
    // and 8, on the blended rectangle.
    let mut s = corner_scene(&fixture::css_two_version_corner_fixture(false));
    let (a, b) = corner_rest_rects(&s);
    s.begin();
    for t in [0.25, 0.5, 1.0] {
        flight_progress(&mut s, t);
        let rect = mix_rect(a, b, t);
        assert_outline(
            &ink_points(&s.factory, &s.root, UNIT_A),
            rect,
            4.0,
            &[8.0],
            &format!("no corner keys, a at t={t}"),
        );
        assert_outline(
            &ink_points(&s.factory, &s.root, UNIT_B),
            rect,
            8.0,
            &[4.0],
            &format!("no corner keys, b at t={t}"),
        );
    }

    // (b) Three corners keyed, the bottom left left at -1: no override, the
    // same as (a).
    let mut s = corner_scene(&fixture::css_three_corner_keys_fixture());
    let (a, b) = corner_rest_rects(&s);
    s.begin();
    let mut elapsed = 0.0;
    for t in [0.25, 0.5, 1.0] {
        s.advance(t - elapsed);
        elapsed = t;
        let keyed = 4.0 + (8.0 - 4.0) * t;
        for flight in s.moves.flights(&s.root) {
            close(val(&flight.controls(), 62003), t);
            for corner in 62015..=62017 {
                close(val(&flight.controls(), corner), keyed);
            }
            close(val(&flight.controls(), 62018), -1.0);
        }
        let rect = mix_rect(a, b, t);
        assert_outline(
            &ink_points(&s.factory, &s.root, UNIT_A),
            rect,
            4.0,
            &[8.0, keyed],
            &format!("three corner keys, a at t={t}"),
        );
        assert_outline(
            &ink_points(&s.factory, &s.root, UNIT_B),
            rect,
            8.0,
            &[4.0, keyed],
            &format!("three corner keys, b at t={t}"),
        );
    }

    // (c) Corner keys on translated units: they draw as without the keys.
    let draws = [false, true].map(|keyed| {
        let mut s = Scene::from_bytes(&if keyed {
            fixture::two_version_corner_keys_fixture()
        } else {
            fixture::two_version_fixture()
        });
        let draw = |s: &Scene| {
            s.factory.borrow_mut().clear();
            let mut renderer = s.factory.borrow().make_renderer();
            s.root.draw(&mut renderer);
            s.factory.borrow().canonical_recording().stream().to_owned()
        };
        s.begin();
        for flight in s.moves.flights(&s.root) {
            set(&flight.controls(), 62003, 0.5);
        }
        s.advance(0.5);
        if keyed {
            for flight in s.moves.flights(&s.root) {
                for corner in 62015..=62018 {
                    close(val(&flight.controls(), corner), 6.0);
                }
            }
        }
        let flying = draw(&s);
        s.end();
        (flying, draw(&s))
    });
    assert_eq!(
        draws[0].0, draws[1].0,
        "translated units draw as without corner keys, in flight"
    );
    assert_eq!(
        draws[0].1, draws[1].1,
        "translated units draw as without corner keys, landed"
    );

    // (d) Bad records: a resting slot with a corner other than -1, and a
    // flight slot with a NaN corner, are each dropped at import.
    for (kind, screen, host) in [(1, "A", "a"), (2, "B", "a flight")] {
        let good = slot_records_on(
            &fixture::css_two_version_corner_fixture(false),
            screen,
            host,
        );
        let bad = slot_records_on(&fixture::css_bad_corner_record_fixture(kind), screen, host);
        assert_eq!(good.iter().filter(|on| **on).count(), 1, "kind {kind}");
        assert_eq!(
            bad.iter().filter(|on| **on).count(),
            0,
            "kind {kind}: the malformed record on {screen}'s slot {host} is dropped at import"
        );
        assert_eq!(
            bad.len(),
            good.len() - 1,
            "kind {kind}: the Screen's other records stay"
        );
    }
}

/// Whether each of `screen`'s LiveSlot records, as the file imports them, is
/// on the slot named `host`.
fn slot_records_on(bytes: &[u8], screen: &str, host: &str) -> Vec<bool> {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("fixture imports");
    let row = file.with_file(|f| f.artboard_named(screen)).unwrap();
    row.with_artboard(|board| {
        let host = board.find_handle::<NestedArtboard>(host).unwrap();
        board
            .find_all_handles::<live_move::records::LiveSlot>()
            .iter()
            .map(|record| {
                record
                    .with_downcast::<live_move::records::LiveSlot, _>(|r| r.parent())
                    .flatten()
                    == Some(host.clone())
            })
            .collect::<Vec<_>>()
    })
}

/// Two versions of an element share one outline (docs/live-moves.md): in
/// flight, each draws its flight slot's corner controls as its own corners,
/// so at progress t both draw 4 + (8 - 4) t point corners on the blended
/// rectangle. A one-version flight draws them over its own corners, which
/// it draws again from landing on.
#[test]
fn two_versions_draw_their_flight_slots_corners() {
    use source::{
        layout::layout_component_style::LayoutComponentStyle,
        shapes::paint::solid_color::SolidColor,
    };
    let mut s = corner_scene(&fixture::css_two_version_corner_fixture(true));
    let (a, b) = corner_rest_rects(&s);
    s.begin();
    let mut elapsed = 0.0;
    for t in [0.25, 0.5, 1.0] {
        s.advance(t - elapsed);
        elapsed = t;
        let radius = 4.0 + (8.0 - 4.0) * t;
        for flight in s.moves.flights(&s.root) {
            close(val(&flight.controls(), 62003), t);
            for corner in 62015..=62018 {
                close(val(&flight.controls(), corner), radius);
            }
        }
        let rect = mix_rect(a, b, t);
        for (color, version) in [(UNIT_A, "a"), (UNIT_B, "b")] {
            assert_outline(
                &ink_points(&s.factory, &s.root, color),
                rect,
                radius,
                &[4.0, 8.0],
                &format!("version {version} at t={t}"),
            );
        }
    }

    // One version: unit c, its own corners written to 2 after load as a
    // binding writes them, flies with the same keys and lands with its own.
    const MOVED: &str = "color=0xff007766";
    let mut s = corner_scene(&fixture::css_two_version_corner_fixture(true));
    let (home, landing) = (corner_slot(&s.a, "c"), corner_slot(&s.b, "c"));
    let unit = child(&home).unwrap();
    let (ink, style) = unit.with_artboard(|board| {
        (
            board.find_handle::<SolidColor>("ink").unwrap(),
            board
                .objects()
                .iter()
                .flatten()
                .find(|object| object.is_type_of(LayoutComponentStyle::TYPE_KEY))
                .cloned()
                .unwrap(),
        )
    });
    // Its own color tells it from the fresh unit c in its landing slot.
    assert!(CoreRegistry::set_color_handle(
        &ink,
        37,
        0xff007766_u32 as i32
    ));
    set(&style, 640, 2.0);
    s.advance(0.0);
    let from = drawn_rect(&ink_points(&s.factory, &s.root, MOVED));
    assert_outline(
        &ink_points(&s.factory, &s.root, MOVED),
        from,
        2.0,
        &[],
        "one version at rest",
    );
    let to = drawn_rect(&ink_points(&s.factory, &s.root, UNIT_C));
    s.moves.step_begin(&s.root, "B", Some("t2")).unwrap();
    let mut elapsed = 0.0;
    for t in [0.25, 0.5, 1.0] {
        s.advance(t - elapsed);
        elapsed = t;
        assert_outline(
            &ink_points(&s.factory, &s.root, MOVED),
            mix_rect(from, to, t),
            4.0 + (8.0 - 4.0) * t,
            &[2.0],
            &format!("one version at t={t}"),
        );
    }
    s.moves.step_end(&s.root, "B", Some("t2")).unwrap();
    assert_eq!(child(&landing).unwrap().core_handle(), unit.core_handle());
    assert_outline(
        &ink_points(&s.factory, &s.root, MOVED),
        to,
        2.0,
        &[8.0],
        "one version landed",
    );
    s.advance(0.0);
    assert_outline(
        &ink_points(&s.factory, &s.root, MOVED),
        to,
        2.0,
        &[8.0],
        "one version at rest on B",
    );
}

/// The Page's Screens list in `corner_scene`'s fixtures: A is row 0, B row 1.
fn corner_screens(s: &Scene) -> CoreHandle {
    s.root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .unwrap()
}

/// At rest every Screen draws as authored (docs/live-moves.md). Landing
/// clears a flight's corners: with no advance after the last flight frame,
/// the arriving b draws its own 8 and the leaving a, home on A, its own 4.
/// A's row then sleeps through an advance and wakes, and a's first frame
/// draws 4, whether A fell asleep after the landing or before it.
#[test]
fn landing_draws_each_versions_own_corners_at_once() {
    for asleep_at_landing in [false, true] {
        let when = if asleep_at_landing {
            "A asleep at landing"
        } else {
            "A awake at landing"
        };
        let mut s = corner_scene(&fixture::css_two_version_corner_fixture(true));
        let (a, b) = corner_rest_rects(&s);
        let list = corner_screens(&s);
        s.begin();
        s.advance(1.0);
        for flight in s.moves.flights(&s.root) {
            close(val(&flight.controls(), 62003), 1.0);
            for corner in 62015..=62018 {
                close(val(&flight.controls(), corner), 8.0);
            }
        }
        if asleep_at_landing {
            set_row_collapsed(&list, 0, true).unwrap();
        }
        s.end();
        // A sleeping row leaves the list's layout, so B sits higher while A
        // sleeps.
        let landed = ink_points(&s.factory, &s.root, UNIT_B);
        let at = if asleep_at_landing {
            drawn_rect(&landed)
        } else {
            b
        };
        close(at.1.x - at.0.x, 160.0);
        close(at.1.y - at.0.y, 120.0);
        assert_outline(&landed, at, 8.0, &[4.0], &format!("{when}: b landed"));
        if !asleep_at_landing {
            assert_outline(
                &ink_points(&s.factory, &s.root, UNIT_A),
                a,
                4.0,
                &[8.0],
                &format!("{when}: a landed home on A"),
            );
            set_row_collapsed(&list, 0, true).unwrap();
        }
        s.advance(0.5);
        set_row_collapsed(&list, 0, false).unwrap();
        s.advance(0.0);
        assert_outline(
            &ink_points(&s.factory, &s.root, UNIT_A),
            a,
            4.0,
            &[8.0],
            &format!("{when}: a's first frame after waking"),
        );
        assert_outline(
            &ink_points(&s.factory, &s.root, UNIT_B),
            b,
            8.0,
            &[4.0],
            &format!("{when}: b after A wakes"),
        );
    }
}

/// `color`'s unit draws a `size` box with `radius` point corners, and with
/// none of any `wrong` radius's, wherever its Screen now places it.
fn assert_drawn_box(
    s: &Scene,
    color: &str,
    size: (f32, f32),
    radius: f32,
    wrong: &[f32],
    label: &str,
) {
    let drawn = ink_points(&s.factory, &s.root, color);
    let rect = drawn_rect(&drawn);
    close(rect.1.x - rect.0.x, size.0);
    close(rect.1.y - rect.0.y, size.1);
    assert_outline(&drawn, rect, radius, wrong, label);
}

/// A cancelled step leaves each surviving version at rest with its own
/// corners, as authored (docs/live-moves.md). At progress 0.5 both draw 6.
/// Removing B brings a home to A through the unmount's loan return, where it
/// draws 4; removing A brings b home on B through the unused-flight pass,
/// where it draws 8.
#[test]
fn a_cancelled_step_leaves_both_versions_at_their_own_corners() {
    for removed in [1, 0] {
        let mut s = corner_scene(&fixture::css_two_version_corner_fixture(true));
        let (a, b) = corner_rest_rects(&s);
        let (unit_a, unit_b) = (
            child(&s.s).unwrap().core_handle(),
            child(&s.d).unwrap().core_handle(),
        );
        s.begin();
        s.advance(0.5);
        let rect = mix_rect(a, b, 0.5);
        for (color, version) in [(UNIT_A, "a"), (UNIT_B, "b")] {
            assert_outline(
                &ink_points(&s.factory, &s.root, color),
                rect,
                6.0,
                &[4.0, 8.0],
                &format!("version {version} at the half"),
            );
        }
        let list = corner_screens(&s);
        let kept = list
            .with_downcast::<ArtboardComponentList, _>(|l| l.list_item(1 - removed))
            .flatten()
            .unwrap();
        ArtboardComponentList::update_list_occurrence(&list, &[kept]);
        s.advance(0.0);
        assert!(
            s.moves.flights(&s.root).is_empty(),
            "removing a Screen cancels the step"
        );
        if removed == 1 {
            assert_eq!(child(&s.s).unwrap().core_handle(), unit_a);
            assert_drawn_box(
                &s,
                UNIT_A,
                (80.0, 60.0),
                4.0,
                &[6.0, 8.0],
                "B removed: a home on A",
            );
        } else {
            assert_eq!(child(&s.d).unwrap().core_handle(), unit_b);
            assert_drawn_box(
                &s,
                UNIT_B,
                (160.0, 120.0),
                8.0,
                &[4.0, 6.0],
                "A removed: b home on B",
            );
        }
    }
}

/// B is added as a row only for the step, the path a host takes going
/// forward (`add_row_b_for_the_step`). Both versions fly in its fresh flight
/// slots and draw their corner controls, 6 at the half, then their own
/// corners from the landing on (docs/live-moves.md).
#[test]
fn versions_flying_into_a_row_added_for_the_step_share_corners() {
    for settle in [false, true] {
        let mut s = corner_scene(&fixture::css_two_version_corner_fixture(false));
        let (a, b) = corner_rest_rects(&s);
        add_row_b_for_the_step(&mut s);
        // Waking the row flushes the insert and empties its fresh flight
        // slots before any advance, as the SDK does; it may then settle one
        // frame.
        set_row_collapsed(&corner_screens(&s), 1, false).unwrap();
        if settle {
            s.advance(0.0);
        }
        s.begin();
        for flight in s.moves.flights(&s.root) {
            set(&flight.controls(), 62003, 0.5);
            for corner in 62015..=62018 {
                set(&flight.controls(), corner, 6.0);
            }
        }
        s.advance(0.0);
        let rect = mix_rect(a, b, 0.5);
        for (color, version) in [(UNIT_A, "a"), (UNIT_B, "b")] {
            assert_outline(
                &ink_points(&s.factory, &s.root, color),
                rect,
                6.0,
                &[4.0, 8.0],
                &format!("settle {settle}: version {version} at the half"),
            );
        }
        s.end();
        for _ in 0..2 {
            assert_outline(
                &ink_points(&s.factory, &s.root, UNIT_A),
                a,
                4.0,
                &[6.0, 8.0],
                &format!("settle {settle}: a home on A"),
            );
            assert_outline(
                &ink_points(&s.factory, &s.root, UNIT_B),
                b,
                8.0,
                &[4.0, 6.0],
                &format!("settle {settle}: b home on B"),
            );
            s.advance(0.0);
        }
    }
}

/// A second step through the same flight slots takes off with its own
/// corners. The first step's keys ended at 8; the second step's frame-0
/// keys (4) are applied before it begins, as a host's step begin applies
/// them, and its take-off frame draws 4 on both versions.
#[test]
fn a_reused_flight_slot_takes_off_with_this_steps_corners() {
    let mut s = corner_scene(&fixture::css_two_version_corner_fixture(true));
    let (a, _) = corner_rest_rects(&s);
    s.begin();
    let controls = s
        .moves
        .flights(&s.root)
        .iter()
        .map(|flight| flight.controls())
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 2);
    s.advance(1.0);
    s.end();
    s.advance(0.0);
    for control in &controls {
        close(val(control, 62003), 1.0);
        for corner in 62015..=62018 {
            close(val(control, corner), 8.0);
        }
    }
    let mut keys = s.b.animation_named("t1").unwrap();
    keys.set_time(0.0);
    keys.apply(1.0);
    for control in &controls {
        close(val(control, 62003), 0.0);
        for corner in 62015..=62018 {
            close(val(control, corner), 4.0);
        }
    }
    s.begin();
    let flights = s.moves.flights(&s.root);
    assert_eq!(flights.len(), 2);
    assert!(
        flights
            .iter()
            .all(|flight| controls.contains(&flight.controls()))
    );
    for (color, version) in [(UNIT_A, "a"), (UNIT_B, "b")] {
        assert_outline(
            &ink_points(&s.factory, &s.root, color),
            a,
            4.0,
            &[8.0],
            &format!("version {version} at take-off"),
        );
    }
}

/// A shadow as `drawn_shadows` reads it: `rect` is its path's, final, so its
/// box moved by its offset, and `clip` the region it may draw in.
#[derive(Debug)]
struct DrawnShadow {
    color: u32,
    blur: f32,
    rect: Rect,
    clip: Option<Rect>,
}

/// Every shadow the Page draws, at any blur or alpha. These Pages draw only
/// their units' inks and shadows, so a shadow is each path without an ink's
/// channels. A fade changes an ink's alpha alone.
fn drawn_shadows(s: &Scene) -> Vec<DrawnShadow> {
    let field = |line: &str, name: &str| {
        line.split(name)
            .nth(1)
            .unwrap()
            .split(',')
            .next()
            .unwrap()
            .to_owned()
    };
    let is_ink = |line: &str| {
        let color = field(line, "color=0x");
        [UNIT_A, UNIT_B, UNIT_C]
            .iter()
            .any(|ink| ink.ends_with(&color[2..]))
    };
    drawn_paths(&s.factory, &s.root, |line| !is_ink(line))
        .into_iter()
        .map(|draw| DrawnShadow {
            color: u32::from_str_radix(&field(&draw.line, "color=0x"), 16).unwrap(),
            blur: field(&draw.line, ",feather=").parse().unwrap(),
            rect: drawn_rect(&draw.points),
            clip: (!draw.clip.is_empty()).then(|| drawn_rect(&draw.clip)),
        })
        .collect()
}

fn near_rect(a: Rect, b: Rect) -> bool {
    [a.0.x - b.0.x, a.0.y - b.0.y, a.1.x - b.1.x, a.1.y - b.1.y]
        .iter()
        .all(|gap| gap.abs() < 0.01)
}

/// The Page draws exactly these shadows, each at its offset from its box.
fn assert_shadows(s: &Scene, expected: &[(fixture::Shadow, Rect)], label: &str) {
    use source::math::vec2d::Vec2D;
    let drawn = drawn_shadows(s);
    assert_eq!(drawn.len(), expected.len(), "{label}: {drawn:x?}");
    for (shadow, rect) in expected {
        let by = Vec2D::new(shadow.offset.0, shadow.offset.1);
        let at = (rect.0 + by, rect.1 + by);
        assert!(
            drawn.iter().any(|d| d.color == shadow.color
                && (d.blur - shadow.blur).abs() < 0.01
                && near_rect(d.rect, at)),
            "{label}: no {shadow:x?} at {at:?} in {drawn:x?}"
        );
    }
}

/// The seven shadow controls as `css_two_version_shadow_fixture(true)` keys
/// them at progress `t`.
fn keyed_shadow(t: f32) -> [f32; 7] {
    fixture::SHADOW_KEYS.map(|(from, to)| from + (to - from) * t)
}

/// What seven control values draw: docs/live-moves.md's alpha and channels,
/// rounded to 8 bits.
fn drawn_as([alpha, red, green, blue, blur, x, y]: [f32; 7]) -> fixture::Shadow {
    let byte = |value: f32| value.round() as u32;
    fixture::Shadow {
        color: byte(alpha * 255.0) << 24 | byte(red) << 16 | byte(green) << 8 | byte(blue),
        blur,
        offset: (x, y),
    }
}

fn assert_shadow_controls(controls: &CoreHandle, values: [f32; 7]) {
    for (control, value) in (62019..).zip(values) {
        close(val(controls, control), value);
    }
}

fn write_shadow_controls(controls: &CoreHandle, values: [f32; 7]) {
    for (control, value) in (62019..).zip(values) {
        set(controls, control, value);
    }
}

/// The controls of the flight slots "a flight" and "b flight".
fn pair_controls(s: &Scene) -> (CoreHandle, CoreHandle) {
    let controls = |name| {
        let slot = corner_slot(&s.b, name);
        s.moves
            .flights(&s.root)
            .iter()
            .find(|flight| flight.flight == slot)
            .unwrap()
            .controls()
    };
    (controls("a flight"), controls("b flight"))
}

/// Unit b's own shadow, which the fixture authors clear, made visible as a
/// binding writes it.
fn show_b_shadow(s: &mut Scene) {
    use source::shapes::paint::solid_color::SolidColor;
    let shadow = child(&s.d)
        .unwrap()
        .with_artboard(|b| b.find_handle::<SolidColor>("shadow"))
        .unwrap();
    assert!(CoreRegistry::set_color_handle(
        &shadow,
        37,
        fixture::UNIT_B_SHADOW.color as i32
    ));
    s.advance(0.0);
}

/// Clears a unit's ink, as a binding writes it, so its shadow is the only paint
/// that can draw.
fn clear_ink(unit: &RuntimeArtboardInstanceHandle) {
    use source::shapes::paint::solid_color::SolidColor;
    let ink = unit
        .with_artboard(|u| u.find_handle::<SolidColor>("ink"))
        .unwrap();
    assert!(CoreRegistry::set_color_handle(&ink, 37, 0));
}

/// Host geometry, queried on a unit, lists its root at `size` or, for None,
/// not at all. A unit's root is the only geometry it has.
fn assert_catalogued(
    s: &Scene,
    unit: &RuntimeArtboardInstanceHandle,
    size: Option<(f32, f32)>,
    label: &str,
) {
    let mut host = ArtboardInstance::from_native_handle(s._file.clone(), 0, unit.clone());
    let listed: Vec<_> = host
        .visible_geometry_with_bounds()
        .iter()
        .map(|hit| {
            (
                hit.bounds.max_x - hit.bounds.min_x,
                hit.bounds.max_y - hit.bounds.min_y,
            )
        })
        .collect();
    assert_eq!(
        listed.len(),
        usize::from(size.is_some()),
        "{label}: {listed:?}"
    );
    if let Some((width, height)) = size {
        close(listed[0].0, width);
        close(listed[0].1, height);
    }
}

/// A unit's painted bounds, in its own space.
fn painted_bounds(unit: &RuntimeArtboardInstanceHandle) -> source::math::aabb::Aabb {
    let mut bounds = source::math::aabb::Aabb::default();
    unit.with_artboard_mut(|u| u.base.base.base.painted_world_bounds(&mut bounds));
    bounds
}

/// One shadow for a held pair (docs/live-moves.md). In flight, a flight slot's
/// seven shadow controls replace the color, blur and offset of its unit's outer
/// shadow, and alpha 0 draws none: a's slot keys all seven, b's alpha 0.
#[test]
fn two_versions_draw_their_flight_slots_shadow() {
    let mut s = corner_scene(&fixture::css_two_version_shadow_fixture(true));
    let (a, b) = corner_rest_rects(&s);
    show_b_shadow(&mut s);
    s.begin();
    let (controls_a, controls_b) = pair_controls(&s);
    let mut elapsed = 0.0;
    for t in [0.25, 0.5, 1.0] {
        s.advance(t - elapsed);
        elapsed = t;
        assert_shadow_controls(&controls_a, keyed_shadow(t));
        assert_shadow_controls(&controls_b, [0.0; 7]);
        assert_shadows(
            &s,
            &[(drawn_as(keyed_shadow(t)), mix_rect(a, b, t))],
            &format!("at t={t}"),
        );
    }
}

/// At rest every Screen draws as authored (docs/live-moves.md). Landing clears
/// a flight's shadow: with no advance after the last flight frame, the arriving
/// b draws its own shadow and the leaving a, home on A, its own. A's row then
/// sleeps through an advance and wakes, and a's first frame draws its own,
/// whether A fell asleep after the landing or before it.
#[test]
fn landing_draws_each_versions_own_shadow_at_once() {
    use fixture::{UNIT_A_SHADOW, UNIT_B_SHADOW};
    for asleep_at_landing in [false, true] {
        let when = if asleep_at_landing {
            "A asleep at landing"
        } else {
            "A awake at landing"
        };
        let mut s = corner_scene(&fixture::css_two_version_shadow_fixture(true));
        let (a, b) = corner_rest_rects(&s);
        show_b_shadow(&mut s);
        let list = corner_screens(&s);
        s.begin();
        let (controls_a, _) = pair_controls(&s);
        s.advance(1.0);
        assert_shadow_controls(&controls_a, keyed_shadow(1.0));
        if asleep_at_landing {
            set_row_collapsed(&list, 0, true).unwrap();
        }
        s.end();
        // A sleeping row leaves the list's layout, so B sits higher while A
        // sleeps.
        if asleep_at_landing {
            let at = drawn_rect(&ink_points(&s.factory, &s.root, UNIT_B));
            assert_shadows(&s, &[(UNIT_B_SHADOW, at)], &format!("{when}: landed"));
        } else {
            assert_shadows(
                &s,
                &[(UNIT_A_SHADOW, a), (UNIT_B_SHADOW, b)],
                &format!("{when}: landed"),
            );
            set_row_collapsed(&list, 0, true).unwrap();
        }
        s.advance(0.5);
        set_row_collapsed(&list, 0, false).unwrap();
        s.advance(0.0);
        assert_shadows(
            &s,
            &[(UNIT_A_SHADOW, a), (UNIT_B_SHADOW, b)],
            &format!("{when}: a's first frame after waking"),
        );
    }
}

/// A cancelled step leaves each surviving version at rest with its own shadow,
/// as authored (docs/live-moves.md). Removing B brings a home to A through the
/// unmount's loan return; removing A brings b home on B through the
/// unused-flight pass. Each draws its own in the first frame.
#[test]
fn a_cancelled_step_leaves_both_versions_at_their_own_shadow() {
    for removed in [1, 0] {
        let mut s = corner_scene(&fixture::css_two_version_shadow_fixture(true));
        let (a, b) = corner_rest_rects(&s);
        show_b_shadow(&mut s);
        s.begin();
        s.advance(0.5);
        assert_shadows(
            &s,
            &[(drawn_as(keyed_shadow(0.5)), mix_rect(a, b, 0.5))],
            "at the half",
        );
        let list = corner_screens(&s);
        let kept = list
            .with_downcast::<ArtboardComponentList, _>(|l| l.list_item(1 - removed))
            .flatten()
            .unwrap();
        ArtboardComponentList::update_list_occurrence(&list, &[kept]);
        s.advance(0.0);
        assert!(
            s.moves.flights(&s.root).is_empty(),
            "removing a Screen cancels the step"
        );
        // The Screen left moves up the list, so each box is read where it is.
        let (own, ink, label) = if removed == 1 {
            (fixture::UNIT_A_SHADOW, UNIT_A, "B removed: a home on A")
        } else {
            (fixture::UNIT_B_SHADOW, UNIT_B, "A removed: b home on B")
        };
        let at = drawn_rect(&ink_points(&s.factory, &s.root, ink));
        assert_shadows(&s, &[(own, at)], label);
    }
}

/// A second step through the same flight slots takes off with its own shadow.
/// The first step's keys ended at the last frame's values; the second step's
/// frame-0 keys are applied before it begins, as a host's step begin applies
/// them, and its take-off frame draws them.
#[test]
fn a_reused_flight_slot_takes_off_with_this_steps_shadow() {
    let mut s = corner_scene(&fixture::css_two_version_shadow_fixture(true));
    let (a, _) = corner_rest_rects(&s);
    show_b_shadow(&mut s);
    s.begin();
    let (controls_a, controls_b) = pair_controls(&s);
    s.advance(1.0);
    s.end();
    s.advance(0.0);
    assert_shadow_controls(&controls_a, keyed_shadow(1.0));
    let mut keys = s.b.animation_named("t1").unwrap();
    keys.set_time(0.0);
    keys.apply(1.0);
    assert_shadow_controls(&controls_a, keyed_shadow(0.0));
    assert_shadow_controls(&controls_b, [0.0; 7]);
    s.begin();
    let flights = s.moves.flights(&s.root);
    assert_eq!(flights.len(), 2);
    assert!(
        flights
            .iter()
            .all(|flight| [&controls_a, &controls_b].contains(&&flight.controls()))
    );
    assert_shadows(&s, &[(drawn_as(keyed_shadow(0.0)), a)], "at take-off");
}

/// A flight's shadow controls replace a unit's own outer shadow only when
/// alpha is 0 or more, and only that shadow (docs/live-moves.md). Each case
/// before the bad records draws as if the controls were not there.
#[test]
fn shadow_controls_guard_authored_shadows() {
    use fixture::{UNIT_A_SHADOW, UNIT_B_SHADOW};
    // The blur each keyed case's flight slot read, and its keys' value there.
    // Checked last: the keys of a runtime without these controls are pruned,
    // and every case drawing its own says nothing unless they arrived.
    let mut arrived = Vec::new();
    let half = keyed_shadow(0.5)[4];

    // (a) No shadow keys: each version draws its own, on the blended rectangle.
    let mut s = corner_scene(&fixture::css_two_version_shadow_fixture(false));
    let (a, b) = corner_rest_rects(&s);
    show_b_shadow(&mut s);
    s.begin();
    for t in [0.25, 0.5, 1.0] {
        flight_progress(&mut s, t);
        let rect = mix_rect(a, b, t);
        assert_shadows(
            &s,
            &[(UNIT_A_SHADOW, rect), (UNIT_B_SHADOW, rect)],
            &format!("no shadow keys at t={t}"),
        );
    }

    // (b) Alpha -1 with the other six keyed: the same as (a).
    let mut s = corner_scene(&fixture::css_shadow_alpha_off_fixture());
    let (a, b) = corner_rest_rects(&s);
    show_b_shadow(&mut s);
    s.begin();
    let mut elapsed = 0.0;
    for t in [0.25, 0.5, 1.0] {
        s.advance(t - elapsed);
        elapsed = t;
        let rect = mix_rect(a, b, t);
        assert_shadows(
            &s,
            &[(UNIT_A_SHADOW, rect), (UNIT_B_SHADOW, rect)],
            &format!("alpha -1 at t={t}"),
        );
    }
    arrived.push((val(&pair_controls(&s).0, 62023), keyed_shadow(1.0)[4]));

    // (c) Shadow keys on translated units, each with Rive's own soft shadow:
    // they draw as without the keys.
    let draws = [false, true].map(|keyed| {
        let mut s = Scene::from_bytes(&fixture::two_version_shadow_keys_fixture(keyed));
        let draw = |s: &Scene| {
            s.factory.borrow_mut().clear();
            let mut renderer = s.factory.borrow().make_renderer();
            s.root.draw(&mut renderer);
            s.factory.borrow().canonical_recording().stream().to_owned()
        };
        s.begin();
        let flights = s.moves.flights(&s.root);
        for flight in &flights {
            set(&flight.controls(), 62003, 0.5);
        }
        s.advance(0.5);
        if keyed {
            for flight in &flights {
                arrived.push((val(&flight.controls(), 62023), half));
            }
        }
        let flying = draw(&s);
        s.end();
        (flying, draw(&s))
    });
    assert!(
        draws[0].0.contains(",feather=12,"),
        "translated units draw their Feather"
    );
    assert_eq!(
        draws[0].0, draws[1].0,
        "translated units draw as without shadow keys, in flight"
    );
    assert_eq!(
        draws[0].1, draws[1].1,
        "translated units draw as without shadow keys, landed"
    );

    // (d) An inset shadow is left alone, and a unit whose only shadow is inset
    // draws no outer one: a's slot keys all seven at their half.
    let inset = |keyed: bool| {
        let mut s = corner_scene(&fixture::css_inset_shadow_fixture(keyed));
        s.begin();
        if keyed {
            s.advance(0.5);
        } else {
            flight_progress(&mut s, 0.5);
        }
        (val(&pair_controls(&s).0, 62023), drawn_shadows(&s))
    };
    let ((blur, keyed), (_, own)) = (inset(true), inset(false));
    arrived.push((blur, half));
    assert_eq!(keyed.len(), 1, "one inset shadow: {keyed:x?}");
    assert_eq!(own.len(), 1, "one inset shadow: {own:x?}");
    assert_eq!(keyed[0].color, UNIT_A_SHADOW.color, "{keyed:x?}");
    close(keyed[0].blur, UNIT_A_SHADOW.blur);
    assert!(
        near_rect(keyed[0].rect, own[0].rect),
        "{keyed:x?} against {own:x?}"
    );

    // (e) A box with no outer shadow draws none: unit c flies alone under the
    // same keys, past a at home and the clear b.
    let mut s = corner_scene(&fixture::css_two_version_shadow_fixture(true));
    let (a, _) = corner_rest_rects(&s);
    s.moves.step_begin(&s.root, "B", Some("t2")).unwrap();
    s.advance(0.5);
    arrived.push((val(&s.moves.flights(&s.root)[0].controls(), 62023), half));
    assert_shadows(&s, &[(UNIT_A_SHADOW, a)], "unit c flies with shadow keys");

    // (f) Bad records: a resting slot with a shadow value, and a flight slot
    // with a NaN, are each dropped at import, on any of the seven controls.
    for (kind, screen, host) in [(1, "A", "a"), (2, "B", "a flight")] {
        let good = slot_records_on(
            &fixture::css_two_version_shadow_fixture(false),
            screen,
            host,
        );
        assert_eq!(good.iter().filter(|on| **on).count(), 1, "kind {kind}");
        for control in 0..fixture::SHADOW_CONTROLS.len() {
            let bad = slot_records_on(
                &fixture::css_bad_shadow_record_fixture(kind, control),
                screen,
                host,
            );
            assert_eq!(
                bad.iter().filter(|on| **on).count(),
                0,
                "kind {kind}, control {control}: dropped from {screen}'s slot {host} at import"
            );
            assert_eq!(
                bad.len(),
                good.len() - 1,
                "kind {kind}, control {control}: the Screen's other records stay"
            );
        }
    }

    for (read, keyed) in arrived {
        close(read, keyed);
    }

    // (g) A non-finite control fails the frame, as a non-finite fade does.
    for control in 62019..=62025 {
        let mut s = corner_scene(&fixture::css_two_version_shadow_fixture(false));
        s.begin();
        set(&pair_controls(&s).0, control, f32::NAN);
        s.machine.advance_and_apply(0.0);
        assert!(s.moves.after_advance(&s.root).is_err(), "control {control}");
    }
}

/// A shadow's clip reaches 1.5 times its blur, its offset's two sides and a
/// margin of 2 past its box, and its painted bounds 1.5 times its blur and its
/// offset's length: `css_shadow_feather_written_at_run_time_widens_its_reach`.
/// A flight's shadow measures with its own numbers, and landing measures with
/// the unit's own again, with no advance between.
#[test]
fn a_flight_shadow_wider_than_its_own_reaches_its_region() {
    use source::math::vec2d::Vec2D;
    const OVERRIDE: [f32; 7] = [0.5, 0.0, 0.0, 0.0, 40.0, 3.0, -4.0];
    let mut s = corner_scene(&fixture::css_two_version_shadow_fixture(false));
    let (a, b) = corner_rest_rects(&s);
    let unit = child(&s.s).unwrap();
    let measure = |s: &Scene, rect: Rect, blur: f32, (x, y): (f32, f32), label: &str| {
        let drawn = drawn_shadows(s);
        assert_eq!(drawn.len(), 1, "{label}: {drawn:x?}");
        let region = 1.5 * blur + x.abs() + y.abs() + 2.0;
        let by = Vec2D::new(region, region);
        let clip = drawn[0].clip.expect("a shadow draws inside its clip");
        assert!(
            near_rect(clip, (rect.0 - by, rect.1 + by)),
            "{label}: the clip is {clip:?} around {rect:?}, expected {region} past it"
        );
        let reach = 1.5 * blur + x.hypot(y);
        let bounds = painted_bounds(&unit);
        close(bounds.width(), rect.1.x - rect.0.x + 2.0 * reach);
        close(bounds.height(), rect.1.y - rect.0.y + 2.0 * reach);
    };
    let own = fixture::UNIT_A_SHADOW;
    measure(&s, a, own.blur, own.offset, "at rest");
    s.begin();
    write_shadow_controls(&pair_controls(&s).0, OVERRIDE);
    flight_progress(&mut s, 0.5);
    measure(
        &s,
        mix_rect(a, b, 0.5),
        OVERRIDE[4],
        (OVERRIDE[5], OVERRIDE[6]),
        "in flight",
    );
    flight_progress(&mut s, 1.0);
    s.end();
    measure(&s, a, own.blur, own.offset, "landed");
}

/// The controls replace a shadow bound or keyed (docs/live-moves.md). Unit a's
/// timeline writes its Feather's strength before, through and after the
/// flight. In flight the flight's shadow draws; at landing the timeline's value
/// does, with nothing put back.
#[test]
fn a_bound_shadow_is_replaced_in_flight_and_back_at_landing() {
    let mut s = corner_scene(&fixture::css_two_version_shadow_fixture(true));
    let (a, b) = corner_rest_rects(&s);
    let mut pulse = child(&s.s).unwrap().animation_named("pulse").unwrap();
    // "pulse" keys the strength 10 to 30 over a second.
    let own = |at: f32| fixture::Shadow {
        blur: 10.0 + 20.0 * at,
        ..fixture::UNIT_A_SHADOW
    };
    pulse.set_time(0.25);
    pulse.apply(1.0);
    s.advance(0.0);
    assert_shadows(&s, &[(own(0.25), a)], "the timeline's shadow at rest");
    s.begin();
    let mut elapsed = 0.0;
    for (t, at) in [(0.25, 0.5), (0.5, 0.75)] {
        pulse.set_time(at);
        pulse.apply(1.0);
        s.advance(t - elapsed);
        elapsed = t;
        assert_shadows(
            &s,
            &[(drawn_as(keyed_shadow(t)), mix_rect(a, b, t))],
            &format!("in flight at t={t}"),
        );
    }
    pulse.set_time(1.0);
    pulse.apply(1.0);
    s.advance(1.0 - elapsed);
    s.end();
    assert_shadows(&s, &[(own(1.0), a)], "landed");
    s.advance(0.0);
    assert_shadows(&s, &[(own(1.0), a)], "at rest on A");
}

/// Alpha alone says whether a flight's shadow draws (docs/live-moves.md): a
/// clear own shadow draws it, alpha 0 hides a visible own shadow, and one with
/// no blur still draws. Drawing and painted bounds agree.
#[test]
fn a_clear_own_shadow_draws_the_flight_shadow() {
    const SHOWN: [f32; 7] = [0.3, 10.0, 20.0, 30.0, 8.0, 0.0, 4.0];
    const NO_BLUR: [f32; 7] = [0.3, 10.0, 20.0, 30.0, 0.0, 0.0, 4.0];
    let mut s = corner_scene(&fixture::css_two_version_shadow_fixture(false));
    let (a, b) = corner_rest_rects(&s);
    let (unit_a, unit_b) = (child(&s.s).unwrap(), child(&s.d).unwrap());
    s.begin();
    let (controls_a, controls_b) = pair_controls(&s);
    write_shadow_controls(&controls_a, [0.0; 7]);
    write_shadow_controls(&controls_b, SHOWN);
    flight_progress(&mut s, 0.5);
    let rect = mix_rect(a, b, 0.5);
    assert_shadows(&s, &[(drawn_as(SHOWN), rect)], "b's clear own, a's .4 own");
    let (width, height) = (rect.1.x - rect.0.x, rect.1.y - rect.0.y);
    let hidden = painted_bounds(&unit_a);
    close(hidden.width(), width);
    close(hidden.height(), height);
    let reach = 1.5 * SHOWN[4] + SHOWN[6];
    let shown = painted_bounds(&unit_b);
    close(shown.width(), width + 2.0 * reach);
    close(shown.height(), height + 2.0 * reach);
    write_shadow_controls(&controls_a, NO_BLUR);
    write_shadow_controls(&controls_b, [0.0; 7]);
    flight_progress(&mut s, 0.75);
    assert_shadows(
        &s,
        &[(drawn_as(NO_BLUR), mix_rect(a, b, 0.75))],
        "a's with no blur, b's alpha 0",
    );
}

/// Host geometry lists a CSS root by what it draws (docs/CSS_LAYOUT.md, 4.5),
/// so a flight's shadow decides for the paint it replaces. With a shadow as the
/// only paint that can draw, a clear own shadow is listed at the flight's reach
/// and alpha 0 leaves a visible own shadow out, each by its own at landing.
#[test]
fn host_geometry_lists_a_root_by_its_flight_shadow() {
    const SHOWN: [f32; 7] = [0.3, 10.0, 20.0, 30.0, 8.0, 0.0, 4.0];
    let mut s = corner_scene(&fixture::css_two_version_shadow_fixture(false));
    let (a, b) = corner_rest_rects(&s);
    let (unit_a, unit_b) = (child(&s.s).unwrap(), child(&s.d).unwrap());
    clear_ink(&unit_a);
    clear_ink(&unit_b);
    s.advance(0.0);
    let own = fixture::UNIT_A_SHADOW;
    let own_reach = 1.5 * own.blur + own.offset.0.hypot(own.offset.1);
    let at_rest = Some((
        a.1.x - a.0.x + 2.0 * own_reach,
        a.1.y - a.0.y + 2.0 * own_reach,
    ));
    assert_catalogued(&s, &unit_a, at_rest, "a's own shadow at rest");
    assert_catalogued(&s, &unit_b, None, "b's clear own shadow at rest");
    s.begin();
    let (controls_a, controls_b) = pair_controls(&s);
    write_shadow_controls(&controls_a, [0.0; 7]);
    write_shadow_controls(&controls_b, SHOWN);
    flight_progress(&mut s, 0.5);
    let rect = mix_rect(a, b, 0.5);
    let reach = 1.5 * SHOWN[4] + SHOWN[6];
    assert_catalogued(&s, &unit_a, None, "a's own shadow under alpha 0");
    assert_catalogued(
        &s,
        &unit_b,
        Some((
            rect.1.x - rect.0.x + 2.0 * reach,
            rect.1.y - rect.0.y + 2.0 * reach,
        )),
        "b's clear own shadow under alpha .3",
    );
    flight_progress(&mut s, 1.0);
    s.end();
    assert_catalogued(&s, &unit_a, at_rest, "a landed");
    assert_catalogued(&s, &unit_b, None, "b landed");
}

/// A flight's shadow takes its unit's opacity as the unit's other paints do:
/// its own alpha .4 under the fade's scope at .5 draws .2. A CSS unit holds
/// the fade out of its paints, and this factory has no canvas to take a picture.
#[test]
fn the_flight_shadow_takes_the_units_fade() {
    let mut s = corner_scene(&fixture::css_two_version_shadow_fixture(false));
    let (a, b) = corner_rest_rects(&s);
    s.begin();
    let (controls, _) = pair_controls(&s);
    set(&controls, 62014, 0.5);
    write_shadow_controls(&controls, [0.4, 200.0, 100.0, 50.0, 12.0, 0.0, 3.0]);
    flight_progress(&mut s, 0.5);
    assert_shadows(
        &s,
        &[(
            fixture::Shadow {
                color: 0x66c8_6432,
                blur: 12.0,
                offset: (0.0, 3.0),
            },
            mix_rect(a, b, 0.5),
        )],
        "at fade .5",
    );
    let stream = s.factory.borrow().stream();
    let scope = stream.find("modulateOpacity opacity=0.5");
    let shadow = stream.find("feather=12");
    assert!(scope.is_some() && scope < shadow, "{stream}");
}

#[test]
fn returning_a_live_unit_does_not_restore_destroyed_listener_snapshots() {
    for expired_first in [false, true] {
        // Prepare must not remember the slot before the extra host listener
        // exists: the first move captures the registrations below.
        let s = Scene::load(&fixture::fixture(2), false);
        let extra = s.a.state_machine_at(0).unwrap();
        let expired = extra.downgrade();
        let moved_machine = machine(&s.s);
        let original = moved_machine.with_instance(|m| m.nested_event_listeners());
        assert!(original.iter().any(|listener| listener.ptr_eq(&expired)));
        let survivors: Vec<_> = original
            .iter()
            .filter(|listener| !listener.ptr_eq(&expired))
            .cloned()
            .collect();
        assert!(!survivors.is_empty());
        if expired_first {
            // An expired first entry must not become the restored parent.
            moved_machine.with_instance_mut(|m| {
                for listener in original {
                    m.remove_nested_event_listener(listener);
                }
                m.add_nested_event_listener(expired.clone());
                for listener in &survivors {
                    m.add_nested_event_listener(listener.clone());
                }
            });
        }
        // Transport must also preserve supported duplicate live registrations.
        moved_machine.with_instance_mut(|m| {
            m.add_nested_event_listener(survivors[0].clone());
        });
        let expected: Vec<_> = moved_machine
            .with_instance(|m| m.nested_event_listeners())
            .into_iter()
            .filter(|listener| !listener.ptr_eq(&expired))
            .collect();

        move_live(&s.s, &s.d).unwrap();
        drop(extra);
        assert!(expired.upgrade().is_none());
        move_live(&s.d, &s.s).unwrap();

        let restored = machine(&s.s);
        assert!(restored.downgrade().ptr_eq(&moved_machine.downgrade()));
        let actual = restored.with_instance(|m| m.nested_event_listeners());
        assert_eq!(actual.len(), expected.len());
        assert!(actual.iter().zip(&expected).all(|(a, b)| a.ptr_eq(b)));
        assert!(actual.iter().all(|listener| listener.upgrade().is_some()));
        let parent = restored
            .with_instance(|m| m.parent_state_machine_instance())
            .unwrap();
        assert!(parent.downgrade().ptr_eq(&expected[0]));
    }
}

/// Paint order (UNIV-5635, design `runs/univ-5635/runtime/design.md`, its
/// Tests section). On the upper Screen, B, each flight paints where its
/// matched element paints at rest: directly under the first thing painted
/// after the element that draws something and covers it, on top when
/// nothing does, and never inside a clipping parent. The pages come from
/// `css_paint_fixture`: Screens A and B, 400 by 300, both rows at the
/// page's origin, every box and unit filled with its own color. The oracle
/// is B's recorded draw stream: stream order is "paints under", and a test
/// asserts two drawn boxes overlap, as the fixture's numbers say they do,
/// before it reads their order as visible. Frames are progress 0 (just
/// after take-off), 0.25, 0.5, 0.75 and 1 (just before landing), "go" (A to
/// B) and "back" (B to A). B's entrance timeline advances with the
/// progress going forward and rests on the way back.
mod paint_order {
    use super::*;
    use fixture::PaintCase;
    use nuxie_runtime::source::{
        drawable::RuntimeDrawableOccurrence,
        layout_component::LayoutComponent,
        math::{mat2d::Mat2D, vec2d::Vec2D},
        pointer_button::PointerButton,
    };

    const PROGRESS: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Way {
        Go,
        Back,
    }

    /// One drawPath: its line in the stream, its drawn box, and the drawn
    /// boxes of the clipPaths open around it.
    #[derive(Clone, Debug)]
    struct Draw {
        at: usize,
        rect: Rect,
        clips: Vec<Rect>,
    }

    /// A Screen's draws, by fill color.
    struct Painted {
        label: String,
        draws: Vec<(u32, Draw)>,
    }

    fn rgb(name: &str) -> u32 {
        fixture::paint_color(name) & 0x00ff_ffff
    }
    fn rect(left: f32, top: f32, right: f32, bottom: f32) -> Rect {
        (Vec2D::new(left, top), Vec2D::new(right, bottom))
    }
    /// The two boxes share an area, not only an edge.
    fn overlaps(a: Rect, b: Rect) -> bool {
        a.0.x.max(b.0.x) + 0.01 < a.1.x.min(b.1.x) && a.0.y.max(b.0.y) + 0.01 < a.1.y.min(b.1.y)
    }
    fn assert_rect(label: &str, got: Rect, want: Rect) {
        let near = |a: f32, b: f32| (a - b).abs() < 0.05;
        assert!(
            near(got.0.x, want.0.x)
                && near(got.0.y, want.0.y)
                && near(got.1.x, want.1.x)
                && near(got.1.y, want.1.y),
            "{label}: drawn over {got:?}, expected {want:?}"
        );
    }
    fn path_points(line: &str, matrix: Mat2D) -> Vec<Vec2D> {
        let Some(list) = line
            .split("points=[")
            .nth(1)
            .and_then(|rest| rest.split(']').next())
        else {
            return Vec::new();
        };
        if list.is_empty() {
            return Vec::new();
        }
        list.split("),(")
            .map(|point| {
                let (x, y) = point.trim_matches(['(', ')']).split_once(',').unwrap();
                matrix * Vec2D::new(x.parse().unwrap(), y.parse().unwrap())
            })
            .collect()
    }
    fn painted(
        factory: &PersistentFactory<RecordingFactory>,
        board: &RuntimeArtboardInstanceHandle,
        label: String,
    ) -> Painted {
        factory.borrow_mut().clear();
        let mut renderer = factory.borrow().make_renderer();
        board.draw(&mut renderer);
        let stream = factory.borrow().stream();
        let mut matrix = Mat2D::identity();
        let mut stack: Vec<(Mat2D, usize)> = Vec::new();
        let mut clips: Vec<Rect> = Vec::new();
        let mut draws = Vec::new();
        for (at, line) in stream.lines().enumerate() {
            if line == "save" {
                stack.push((matrix, clips.len()));
            } else if line == "restore" {
                let (saved, open) = stack.pop().expect("a save for each restore");
                matrix = saved;
                clips.truncate(open);
            } else if let Some(values) = line
                .strip_prefix("transform matrix=[")
                .and_then(|s| s.strip_suffix(']'))
            {
                let mut local = Mat2D::identity();
                for (i, v) in values.split(',').enumerate() {
                    local[i] = v.parse().unwrap();
                }
                matrix *= local;
            } else if line.starts_with("clipPath ") {
                let points = path_points(line, matrix);
                if !points.is_empty() {
                    clips.push(drawn_rect(&points));
                }
            } else if line.starts_with("drawPath ") {
                let color = line
                    .split(" paint=")
                    .nth(1)
                    .and_then(|paint| paint.split("color=0x").nth(1))
                    .and_then(|hex| u32::from_str_radix(hex.get(..8)?, 16).ok());
                let points = path_points(line, matrix);
                if let (Some(color), false) = (color, points.is_empty()) {
                    draws.push((
                        color & 0x00ff_ffff,
                        Draw {
                            at,
                            rect: drawn_rect(&points),
                            clips: clips.clone(),
                        },
                    ));
                }
            }
        }
        Painted { label, draws }
    }
    impl Painted {
        fn find(&self, name: &str) -> Option<&Draw> {
            let mut found = self
                .draws
                .iter()
                .filter(|(color, _)| *color == rgb(name))
                .map(|(_, draw)| draw);
            let first = found.next();
            assert!(
                found.next().is_none(),
                "{}: {name} is drawn more than once",
                self.label
            );
            first
        }
        fn get(&self, name: &str) -> &Draw {
            self.find(name)
                .unwrap_or_else(|| panic!("{}: {name} is not drawn", self.label))
        }
        /// `below` is drawn before `above`.
        fn under(&self, below: &str, above: &str) {
            assert!(
                self.get(below).at < self.get(above).at,
                "{}: {below} must be drawn before {above}; drawn {:?}",
                self.label,
                self.names()
            );
        }
        /// The two drawn boxes overlap, so their order shows.
        fn meets(&self, a: &str, b: &str) {
            let (ra, rb) = (self.get(a).rect, self.get(b).rect);
            assert!(
                overlaps(ra, rb),
                "{}: {a} {ra:?} and {b} {rb:?} must overlap",
                self.label
            );
        }
        /// Every name is drawn, in this order.
        fn order(&self, names: &[&str]) {
            let ats: Vec<_> = names.iter().map(|name| self.get(name).at).collect();
            assert!(
                ats.windows(2).all(|pair| pair[0] < pair[1]),
                "{}: expected {names:?} in this order; drawn {:?}",
                self.label,
                self.names()
            );
        }
        /// The last draws are these, in this order.
        fn tail(&self, names: &[&str]) {
            let last: Vec<u32> = self.draws[self.draws.len().saturating_sub(names.len())..]
                .iter()
                .map(|(color, _)| *color)
                .collect();
            let want: Vec<u32> = names.iter().map(|name| rgb(name)).collect();
            assert_eq!(
                last,
                want,
                "{}: expected the last draws {names:?}; drawn {:?}",
                self.label,
                self.names()
            );
        }
        /// The draws as names, where the stream's colors are known.
        fn names(&self) -> Vec<String> {
            self.draws
                .iter()
                .map(|(color, _)| format!("{color:06x}"))
                .collect()
        }
    }

    /// A slot's or box's CSS record.
    fn record_of(owner: &CoreHandle) -> CoreHandle {
        owner
            .with(|o| o.as_container_component().map(|c| c.children().to_vec()))
            .flatten()
            .unwrap_or_default()
            .into_iter()
            .find(|child| child.is_type_of(nuxie_runtime::css_layout::keys::LAYOUT_CSS_STYLE))
            .expect("a CSS record")
    }

    /// Keeps the draw commands of a stream, with each path and paint id
    /// replaced and each number to two decimals.
    fn normalized(stream: &str) -> Vec<String> {
        const DRAWING: [&str; 7] = [
            "save",
            "restore",
            "transform",
            "clipPath",
            "drawPath",
            "drawImage",
            "modulate",
        ];
        let mut lines = Vec::new();
        for line in stream.lines() {
            if !DRAWING.iter().any(|command| line.starts_with(command)) {
                continue;
            }
            let mut out = String::new();
            let mut token = String::new();
            let flush = |token: &mut String, out: &mut String| {
                if token.is_empty() {
                    return;
                }
                if out.ends_with("id=") {
                    out.push('#');
                } else if let Ok(value) = token.parse::<f32>() {
                    let value = if value.abs() < 0.005 { 0.0 } else { value };
                    out.push_str(&format!("{value:.2}"));
                } else {
                    out.push_str(token);
                }
                token.clear();
            };
            for c in line.chars() {
                if ",()[]{}= ".contains(c) {
                    flush(&mut token, &mut out);
                    out.push(c);
                } else {
                    token.push(c);
                }
            }
            flush(&mut token, &mut out);
            lines.push(out);
        }
        lines
    }

    struct PaintScene {
        _file: RuntimeFileHandle,
        factory: PersistentFactory<RecordingFactory>,
        root: RuntimeArtboardInstanceHandle,
        machine: RuntimeStateMachineInstanceHandle,
        moves: LiveMoves,
        case: PaintCase,
        progress: f32,
        pointer: i32,
    }
    impl PaintScene {
        /// The page mounted as a host mounts a published page. Preparing
        /// empties the fresh flight slots.
        fn new(case: PaintCase) -> Self {
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            let file = File::import(
                &fixture::css_paint_fixture(case),
                RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
                None,
                None,
                None,
            )
            .expect("fixture imports");
            let root = file.with_file(|f| f.artboard_named("Page")).unwrap();
            let model = file
                .with_file(|f| {
                    f.create_default_view_model_instance_for_artboard(root.core_handle())
                })
                .unwrap();
            let machine = root.default_state_machine_handle().unwrap();
            machine.with_instance_mut(|m| m.bind_view_model_instance(model));
            machine.advance_and_apply(0.0);
            let mut moves = LiveMoves::default();
            moves.prepare(&root).unwrap();
            let mut scene = Self {
                _file: file,
                factory,
                root,
                machine,
                moves,
                case,
                progress: 0.0,
                pointer: 0,
            };
            scene.advance(0.0);
            scene
        }
        fn row(&self, name: &str) -> RuntimeArtboardInstanceHandle {
            screens(&self.root).unwrap()[name].clone()
        }
        fn slot(&self, row: &str, name: &str) -> CoreHandle {
            self.row(row)
                .with_artboard(|r| r.find_handle::<NestedArtboard>(name))
                .unwrap_or_else(|| panic!("no slot {name} on {row}"))
        }
        fn layout(&self, row: &str, name: &str) -> CoreHandle {
            self.row(row)
                .with_artboard(|r| r.find_handle::<LayoutComponent>(name))
                .unwrap_or_else(|| panic!("no box {name} on {row}"))
        }
        fn painted(&self, row: &str, label: &str) -> Painted {
            painted(
                &self.factory,
                &self.row(row),
                format!("{:?} {label}", self.case),
            )
        }
        /// The upper Screen in a flight frame.
        fn frame(&self, way: Way, p: f32) -> Painted {
            self.painted("B", &format!("{way:?} at {p}"))
        }
        fn stream(&self, row: &str) -> Vec<String> {
            self.factory.borrow_mut().clear();
            let mut renderer = self.factory.borrow().make_renderer();
            self.row(row).draw(&mut renderer);
            normalized(&self.factory.borrow().stream())
        }
        fn sorts(&self, row: &str) -> u8 {
            self.row(row)
                .with_artboard(|r| r.draw_order_change_counter())
        }
        fn advance(&mut self, dt: f32) {
            self.machine.advance_and_apply(dt);
            self.moves.after_advance(&self.root).unwrap();
        }
        fn begin(&mut self, to: &str, step: &str) {
            self.moves.step_begin(&self.root, to, Some(step)).unwrap();
            self.progress = 0.0;
        }
        /// One frame at `p`: every flight's progress, and B's timeline
        /// advanced by the progress made.
        fn to(&mut self, p: f32) {
            for flight in self.moves.flights(&self.root) {
                set(&flight.controls(), 62003, p);
            }
            let dt = p - self.progress;
            self.progress = p;
            self.advance(dt);
        }
        /// Lands, then draws a frame at rest.
        fn end(&mut self, to: &str, step: &str) {
            self.moves.step_end(&self.root, to, Some(step)).unwrap();
            self.advance(0.0);
        }
        fn fly_step(&mut self, to: &str, step: &str, mut each: impl FnMut(&mut Self, f32)) {
            self.begin(to, step);
            for p in PROGRESS {
                self.to(p);
                each(self, p);
            }
            self.end(to, step);
        }
        fn fly(&mut self, way: Way, mut each: impl FnMut(&mut Self, f32)) {
            let (to, step) = match way {
                Way::Go => ("B", "go"),
                Way::Back => ("A", "back"),
            };
            self.fly_step(to, step, |s, p| each(s, p));
        }
        fn both_ways(&mut self, mut each: impl FnMut(&mut Self, Way, f32)) {
            for way in [Way::Go, Way::Back] {
                self.fly(way, |s, p| each(s, way, p));
            }
        }
        /// Taps the page at (x, y) and reads B's "hits", which a pointer
        /// down on B's listened box sets to 9.
        fn tap(&mut self, x: f32, y: f32) -> f32 {
            let list = self
                .root
                .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
                .unwrap();
            let upper = list
                .with_downcast::<ArtboardComponentList, _>(|l| l.state_machine_instance(1))
                .flatten()
                .unwrap();
            upper.set_number("hits", 0.0);
            self.pointer += 1;
            let pointer = self.pointer;
            self.machine.with_instance_mut(|m| {
                m.pointer_down(Vec2D::new(x, y), pointer, PointerButton::Primary);
            });
            upper.with_instance(|m| m.get_number("hits").unwrap().value())
        }
    }

    /// Test 1, AppStoreCard: Back in flow (10, 10, 40 by 40), then `#blur`
    /// (absolute, full screen, opacity keyed 0.1 to 1), then `#page`
    /// (absolute, its insets and size keyed) holding `#card0`, then a
    /// paintless full screen `.catch`. Back's flight is drawn before the
    /// blur's fill, and card0's flight is drawn last.
    #[test]
    fn paint_order_01_app_store_card_back_flies_under_the_blur() {
        let mut s = PaintScene::new(PaintCase::AppStoreCard);
        let rest = s.painted("B", "at rest");
        rest.meets("back", "blur");
        rest.under("back", "blur");
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            f.meets("back", "blur");
            f.under("back", "blur");
            f.tail(&["card0"]);
        });
    }

    /// Test 2, Match: `.catch`, the red picture (0, 0, 400 by 200), Back
    /// (10, 10, 40 by 40), the bar (0, 0, 400 by 50) and `#panel` (0, 150,
    /// 400 by 150), which slides up from 150 below as B enters. On A, the
    /// red is at (100, 100, 200 by 120), Back at (10, 250) and the bar at
    /// (0, 250). By those numbers, going forward the red's flight meets the
    /// panel from progress 0.75 (its bottom 220 - 20p passes the panel's
    /// top, 300 - 150p, after 0.62), and Back's and the bar's never do. On
    /// the way back the red meets it at every frame, and Back's and the
    /// bar's from 0.5 (their bottoms, 50 + 240p and 50 + 250p, pass 150
    /// after 0.42 and 0.4). Where they meet, the red's flight is drawn
    /// before the panel's fill, and the panel's fill before the bar's and
    /// Back's flights.
    #[test]
    fn paint_order_02_match_red_flies_under_the_panel() {
        let mut s = PaintScene::new(PaintCase::Match { red_above: false });
        s.fly(Way::Go, |s, p| {
            let f = s.frame(Way::Go, p);
            if p >= 0.75 {
                f.meets("red", "panel");
                f.under("red", "panel");
            }
        });
        s.fly(Way::Back, |s, p| {
            let f = s.frame(Way::Back, p);
            f.meets("red", "panel");
            f.under("red", "panel");
            if p >= 0.5 {
                for flight in ["bar", "back"] {
                    f.meets(flight, "panel");
                    f.under("panel", flight);
                }
            }
        });
    }

    /// Taps Match half way Back at (200, 180), which is on both the red's
    /// flight (50 to 350, 50 to 210) and the panel, in a frame after an
    /// advance, since a machine takes a changed order into its hits at its
    /// next advance. The red and the panel are both opaque, so only the top
    /// one hears it. Returns the panel's "hits" and the red's.
    fn tap_half_way_back(red_above: bool) -> (f32, f32) {
        let mut s = PaintScene::new(PaintCase::Match { red_above });
        s.fly(Way::Go, |_, _| {});
        let mut hits = None;
        s.fly(Way::Back, |s, p| {
            if p == 0.5 {
                s.advance(0.0);
                let red = machine(&s.slot("B", "red flight"));
                red.set_number("hits", 0.0);
                let panel = s.tap(200.0, 180.0);
                hits = Some((
                    panel,
                    red.with_instance(|m| m.get_number("hits").unwrap().value()),
                ));
            }
        });
        hits.unwrap()
    }

    /// The hit test's controls: at rest, with B entered, the panel alone is
    /// under (200, 250) and takes the tap; with the red after the panel, its
    /// flight paints on top, takes the tap and leaves the panel at 0.
    #[test]
    fn paint_order_02_hit_control_the_red_above_the_panel_takes_the_tap() {
        let mut s = PaintScene::new(PaintCase::Match { red_above: false });
        s.advance(1.0);
        close(s.tap(200.0, 250.0), 9.0);

        let (panel, red) = tap_half_way_back(true);
        close(red, 9.0);
        close(panel, 0.0);
    }

    /// The red's flight is under the panel, so the panel takes a tap on the
    /// two and the red's target does not fire.
    #[test]
    fn paint_order_02_hit_the_panel_over_the_flight_takes_the_tap() {
        let (panel, red) = tap_half_way_back(false);
        close(panel, 9.0);
        close(red, 0.0);
    }

    /// Test 3, Calmly welcome: `#lake` and `#scrim` absolute and full
    /// screen (the scrim's opacity keyed 0.2 to 0.6), a relative flex header
    /// holding the chevron and the title, then the orb and the card, both
    /// absolute. The lake's flight is drawn before the scrim's fill, and the
    /// chevron after it. The title's, orb's and card's flights are last, in
    /// that order.
    #[test]
    fn paint_order_03_calmly_lake_flies_under_the_scrim() {
        let mut s = PaintScene::new(PaintCase::Calmly);
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            f.meets("lake", "scrim");
            f.order(&["lake", "scrim", "chevron"]);
            f.tail(&["title", "orb", "card"]);
        });
    }

    /// Test 4, ListToGrid, both toggles: three flex rows (picture 80 by 100,
    /// title, detail) or a grid of three cells (picture 100 by 80 over a
    /// title). On B the rows or cells slide in from 100 to the left and fade
    /// in from 0.5, both held until the half. Each picture touches the next
    /// row or cell and never overlaps it. Every picture's flight is drawn
    /// after every row, cell, title and detail on B.
    #[test]
    fn paint_order_04_list_to_grid_pictures_fly_on_top() {
        for grid_above in [false, true] {
            let mut s = PaintScene::new(PaintCase::ListToGrid { grid_above });
            let boxes: Vec<String> = (1..=3)
                .flat_map(|n| {
                    if grid_above {
                        vec![format!("cell{n}"), format!("ctitle{n}")]
                    } else {
                        vec![format!("row{n}"), format!("title{n}"), format!("detail{n}")]
                    }
                })
                .collect();
            let holder = |n: usize| {
                if grid_above {
                    format!("cell{n}")
                } else {
                    format!("row{n}")
                }
            };
            s.both_ways(|s, way, p| {
                let f = s.frame(way, p);
                for n in 1..=3 {
                    let picture = format!("pic{n}");
                    // Each picture is over its own row or cell where it rests
                    // on B.
                    if (way == Way::Go && p == 1.0) || (way == Way::Back && p == 0.0) {
                        f.meets(&picture, &holder(n));
                    }
                    for other in &boxes {
                        f.under(other, &picture);
                    }
                }
            });
        }
    }

    /// Test 5, Catzy staged: an absolute full screen dim first, the in-flow
    /// title (20, 0) and cards (0, 30) and (0, 90), a relative plan slot
    /// (0, 150) holding Yearly and the absolute ribbon (270, 140, 80 by 24,
    /// its opacity keyed 0.3 to 1 and its scale 0.5 to 1), Continue in the
    /// flow (0, 210), then the absolute deal (20, 260). The cards', title's
    /// and Continue's flights come before the dim's flight; then come the
    /// dim's flight, Yearly's flight, the ribbon, and the deal's flight
    /// last.
    #[test]
    fn paint_order_05_catzy_flights_keep_the_dims_place() {
        let mut s = PaintScene::new(PaintCase::Catzy);
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            for flight in ["title", "card1", "card2", "cont", "yearly"] {
                f.meets(flight, "dim");
            }
            f.meets("yearly", "ribbon");
            f.order(&[
                "title", "card1", "card2", "cont", "dim", "yearly", "ribbon", "deal",
            ]);
        });
    }

    /// Test 6, clipping holder: E (20, 20, 100 by 60) inside H (0, 0, 300
    /// by 150, `overflow: hidden`), a badge after E in H covering it (60,
    /// 40, 40 by 40), and a box C after H covering E (0, 50, 200 by 100).
    /// The flight is drawn outside any open clipPath of H, after the badge
    /// and before C.
    #[test]
    fn paint_order_06_clipping_holder_flight_paints_outside_the_clip() {
        let mut s = PaintScene::new(PaintCase::ClippingHolder);
        let rest = s.painted("B", "at rest");
        let holder = rest.get("H").rect;
        let inside = |draw: &Draw| {
            draw.clips.iter().any(|clip| {
                overlaps(*clip, holder)
                    && (clip.0 - holder.0).x.abs() < 0.05
                    && (clip.0 - holder.0).y.abs() < 0.05
                    && (clip.1 - holder.1).x.abs() < 0.05
                    && (clip.1 - holder.1).y.abs() < 0.05
            })
        };
        // The reading's positive control: at rest E is inside H's clip.
        assert!(inside(rest.get("E")), "at rest E is drawn in H's clip");
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            f.meets("E", "badge");
            f.meets("E", "C");
            assert!(
                !inside(f.get("E")),
                "{}: the flight is drawn inside H's clip",
                f.label
            );
            f.under("badge", "E");
            f.under("E", "C");
        });
    }

    /// Test 7, badge in a plain card: an absolute badge (90, 30, 40 by 40)
    /// in a static, non-clipping card covers E (20, 20, 100 by 60) there.
    /// The flight is drawn before the badge.
    #[test]
    fn paint_order_07_badge_in_a_plain_card_covers_the_flight() {
        let mut s = PaintScene::new(PaintCase::BadgeInCard);
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            f.meets("E", "badge");
            f.under("E", "badge");
        });
    }

    /// Test 8, round 1 F1: a z-index 2 overlay O (60, 30, 100 by 60) inside
    /// an auto-z absolute wrapper that comes before E (20, 20, relative,
    /// z-index 1) in tree order. The flight is drawn before the overlay.
    #[test]
    fn paint_order_08_overlay_in_an_auto_z_wrapper_covers_the_flight() {
        let mut s = PaintScene::new(PaintCase::OverlayInWrapper);
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            f.meets("E", "O");
            f.under("E", "O");
        });
    }

    /// Test 9, round 2 F1: E relative at (36, 20), then a static block
    /// wrapper W holding a static z-index 2 child K (0, 40, 200 by 100).
    /// Mid-flight the test writes W's record display to flex, which lifts K
    /// over E. Before the write the flight is drawn above K; in the frame
    /// after it, under K. The order is sorted again in that frame and in no
    /// other mid-flight frame.
    #[test]
    fn paint_order_09_a_display_write_lifts_the_child_over_the_flight() {
        let mut s = PaintScene::new(PaintCase::DisplayWrite);
        let wrapper = record_of(&s.layout("B", "W"));
        s.begin("B", "go");
        let mut sorts = None;
        for p in PROGRESS {
            if p == 0.5 {
                assert!(CoreRegistry::set_uint_handle(&wrapper, 61000, 1));
                close(
                    CoreRegistry::get_double_handle(&wrapper, 61000).unwrap(),
                    1.0,
                );
            }
            s.to(p);
            let f = s.frame(Way::Go, p);
            f.meets("E", "K");
            if p < 0.5 {
                f.under("K", "E");
            } else {
                f.under("E", "K");
            }
            let now = s.sorts("B");
            if let Some(before) = sorts {
                if p == 0.5 {
                    assert_ne!(now, before, "the write sorts B again");
                } else {
                    assert_eq!(now, before, "no sort at {p}");
                }
            }
            sorts = Some(now);
        }
        s.end("B", "go");
    }

    /// Test 10, round 3 F1: E1 in flow (20, 20), E2 relative (60, 40) and
    /// E3 relative with z-index 2 (100, 60), their flight slots in reverse
    /// order in the file. The flights' stream order follows the anchors'
    /// paint order, E1, E2, E3, not file order.
    #[test]
    fn paint_order_10_flights_follow_their_anchors_paint_order() {
        let mut s = PaintScene::new(PaintCase::ThreeAnchors);
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            f.meets("E1", "E2");
            f.meets("E2", "E3");
            f.order(&["E1", "E2", "E3"]);
        });
    }

    /// Test 11, resting geometry. E's resting box against a positioned C
    /// (or, in the eighth case, a filled slot D) after it, case by case:
    /// 1. C (150, 150) over E (100, 100, 100 square) through its insets;
    /// 2. C placed at (150, 260) and translated -110 by an unkeyed y;
    /// 3. C over E, its x keyed from 250 to 0, so it starts away and meets
    ///    E's flight only at the end of "go" (250 (1 - p) < 50);
    /// 4. C beside E at (200, 100), turned a quarter turn by a binding;
    /// 5. C beside E, its rotation keyed 0, π/4, 0: over E only mid-step;
    /// 6. E's frame scaled 2 about its center by a binding (50 to 250), C
    ///    (220, 140, 10 by 20) outside E's layout box;
    /// 7. E (100, 100, 100 by 60) turned 0.35 at rest, so its corner
    ///    reaches (207.3, 119), and C (202, 116, 4 by 6) there;
    /// 8. a slot D (220, 100) whose unit is scaled 2 (170 to 370), no C;
    /// 9. to 11. E's frame scaled 2 by a binding, the anchor 100 wide from
    ///    x 0, so at rest E draws from x -50 to 150, and C spans x 120 to
    ///    130. In the ninth and tenth the slot on A is 200 wide; the ninth
    ///    flies forward in Scale mode, the tenth both ways in Layout mode.
    ///    In the eleventh both are 100 wide, and the origins differ: the
    ///    anchor's 0.5, the flight slot's 1, A's 0.25; Scale mode.
    /// The flight is drawn under C (or D) at every frame, except in the
    /// fifth, motion at rest, where it is drawn above C.
    fn resting(n: u8) {
        let mut s = PaintScene::new(PaintCase::Resting(n));
        let coverer = if n == 8 { "D" } else { "C" };
        let rest = s.painted("B", "at rest");
        let (sin, cos) = 0.35f32.sin_cos();
        let e = match n {
            6 => rect(50.0, 50.0, 250.0, 250.0),
            7 => {
                let (w, h) = (50.0 * cos + 30.0 * sin, 50.0 * sin + 30.0 * cos);
                rect(150.0 - w, 130.0 - h, 150.0 + w, 130.0 + h)
            }
            9..=11 => rect(-50.0, 70.0, 150.0, 190.0),
            _ => rect(100.0, 100.0, 200.0, 200.0),
        };
        assert_rect("E at rest", rest.get("E").rect, e);
        match n {
            1 | 2 => assert_rect(
                "C at rest",
                rest.get("C").rect,
                rect(150.0, 150.0, 250.0, 250.0),
            ),
            // B's timeline is at its start: C is 250 to the right.
            3 => assert_rect(
                "C at rest",
                rest.get("C").rect,
                rect(400.0, 150.0, 500.0, 250.0),
            ),
            5 => assert_rect(
                "C at rest",
                rest.get("C").rect,
                rect(200.0, 100.0, 300.0, 200.0),
            ),
            6 => assert_rect(
                "C at rest",
                rest.get("C").rect,
                rect(220.0, 140.0, 230.0, 160.0),
            ),
            7 => assert_rect(
                "C at rest",
                rest.get("C").rect,
                rect(202.0, 116.0, 206.0, 122.0),
            ),
            8 => assert_rect(
                "D at rest",
                rest.get("D").rect,
                rect(170.0, 50.0, 370.0, 250.0),
            ),
            9..=11 => assert_rect(
                "C at rest",
                rest.get("C").rect,
                rect(120.0, 110.0, 130.0, 130.0),
            ),
            _ => {}
        }
        assert_eq!(
            overlaps(rest.get("E").rect, rest.get(coverer).rect),
            !matches!(n, 3 | 5),
            "case {n}: E and {coverer} at rest"
        );
        rest.under("E", coverer);
        let ways: &[Way] = if n == 9 {
            &[Way::Go]
        } else {
            &[Way::Go, Way::Back]
        };
        for &way in ways {
            s.fly(way, |s, p| {
                let f = s.frame(way, p);
                let visible = match n {
                    3 => way == Way::Back || p == 1.0,
                    5 => way == Way::Go && p == 0.5,
                    // Measured from E's resting box, above.
                    9..=11 => false,
                    _ => true,
                };
                if visible {
                    f.meets("E", coverer);
                }
                if n == 5 {
                    f.under(coverer, "E");
                } else {
                    f.under("E", coverer);
                }
            });
        }
    }
    #[test]
    fn paint_order_11_resting_01_insets() {
        resting(1);
    }
    #[test]
    fn paint_order_11_resting_02_unkeyed_translate() {
        resting(2);
    }
    #[test]
    fn paint_order_11_resting_03_keyed_translate_that_starts_away() {
        resting(3);
    }
    #[test]
    fn paint_order_11_resting_04_bound_rotation() {
        resting(4);
    }
    #[test]
    fn paint_order_11_resting_05_keyed_rotation_is_motion_at_rest() {
        resting(5);
    }
    #[test]
    fn paint_order_11_resting_06_frame_scaled_by_a_binding() {
        resting(6);
    }
    #[test]
    fn paint_order_11_resting_07_frame_turned_at_rest() {
        resting(7);
    }
    #[test]
    fn paint_order_11_resting_08_scaled_slot_after_e() {
        resting(8);
    }
    #[test]
    fn paint_order_11_resting_09_unequal_ends_scale_mode() {
        resting(9);
    }
    #[test]
    fn paint_order_11_resting_10_unequal_ends_layout_mode() {
        resting(10);
    }
    #[test]
    fn paint_order_11_resting_11_slot_origins_differ() {
        resting(11);
    }

    /// Test 12, keyed layout, E at (100, 100, 100 by 60): C, absolute at
    /// (0, 100), its width keyed 0 to 200; D, relative, pushed down by a
    /// sibling S whose height is keyed 0 to 100; G, a painted item 100%
    /// wide in a relative CSS grid whose column track is keyed 0 to 200;
    /// P, a painted item in a relative grid of two 100 point columns, its
    /// column keyed from 1 to 2 at the half; R, relative and 100 down, a
    /// painted item 100% wide in B's root, itself a CSS grid whose column
    /// track is keyed 0 to 200.
    ///
    /// The next five are moved through an auto-high box, whose own record
    /// has no key, by a paintless S 300 wide with its height keyed from 0.
    /// W, relative, 300 by 60, follows a wrapper holding S, keyed to 100.
    /// F, relative and painted, holds S, keyed to 200, and is its height.
    /// K, 300 by 60, sits at the bottom edge of a relative wrapper holding
    /// S, keyed to 200. V, relative, 300 by 60, follows an auto-high grid of
    /// two 100 point columns holding I and J, each 30 high; J's column is
    /// keyed from 2 to 1 at the half, which puts it in a second row under I.
    /// T is absolute with a left and no top or bottom, 300 by 60, after a
    /// wrapper holding S, keyed to 100, so its static position follows S.
    ///
    /// The next four are relative, 300 by 60, each pushed down from 0 by 100
    /// through something layout reads that the object tree does not hold.
    /// H follows an auto-high slot whose frame holds S, on the frame's own
    /// timeline. N follows an auto-high slot whose frame has no children and
    /// keys its own record's top padding. L follows a list whose one row
    /// keys its own record's height. O follows a plain node holding S.
    ///
    /// X is the same box after a paintless translated wrapper, with no
    /// record, that hugs S.
    ///
    /// By those numbers each meets E going forward only once its keyed
    /// value has moved: C, G and R from 0.75 (past 100 wide after 0.5), D
    /// from 0.5 (D's top 100p passes 40 after 0.4) and P from 0.5; W, T, H,
    /// N, L, O and X from 0.5, as D; F from 0.75 (its bottom 200p passes 100
    /// after 0.5); K from 0.75 (its bottom 200p, as F's); V from 0.5 (its
    /// top goes from 30 to 60, so its bottom goes from 90 to 120). The
    /// flight is drawn before each at every frame, both ways.
    fn keyed_layout(which: char) {
        let mut s = PaintScene::new(PaintCase::KeyedLayout(which));
        let name = which.to_string();
        let from = if matches!(which, 'C' | 'F' | 'G' | 'K' | 'R') {
            0.75
        } else {
            0.5
        };
        let rest = s.painted("B", "at rest");
        if let Some(keyed) = rest.find(&name) {
            assert!(
                !overlaps(rest.get("E").rect, keyed.rect),
                "{name} starts away from E"
            );
        }
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            if f.find(&name).is_none() {
                // Zero wide at the start: nothing to order.
                assert!(way == Way::Go && p == 0.0, "{}: {name} not drawn", f.label);
                return;
            }
            if way == Way::Back || p >= from {
                f.meets("E", &name);
            }
            f.under("E", &name);
        });
    }
    #[test]
    fn paint_order_12_keyed_layout_c_width() {
        keyed_layout('C');
    }
    #[test]
    fn paint_order_12_keyed_layout_d_pushed_by_a_sibling() {
        keyed_layout('D');
    }
    #[test]
    fn paint_order_12_keyed_layout_g_grid_track() {
        keyed_layout('G');
    }
    #[test]
    fn paint_order_12_keyed_layout_p_grid_placement() {
        keyed_layout('P');
    }
    #[test]
    fn paint_order_12_keyed_layout_r_root_grid_track() {
        keyed_layout('R');
    }
    #[test]
    fn paint_order_12_keyed_layout_w_pushed_through_a_wrapper() {
        keyed_layout('W');
    }
    #[test]
    fn paint_order_12_keyed_layout_f_wrapper_fill() {
        keyed_layout('F');
    }
    #[test]
    fn paint_order_12_keyed_layout_k_absolute_at_a_wrapper_edge() {
        keyed_layout('K');
    }
    #[test]
    fn paint_order_12_keyed_layout_v_placement_in_an_auto_high_grid() {
        keyed_layout('V');
    }
    #[test]
    fn paint_order_12_keyed_layout_t_static_position() {
        keyed_layout('T');
    }
    #[test]
    fn paint_order_12_keyed_layout_h_pushed_through_a_hosted_frame() {
        keyed_layout('H');
    }
    #[test]
    fn paint_order_12_keyed_layout_n_padding_on_a_frames_own_record() {
        keyed_layout('N');
    }
    #[test]
    fn paint_order_12_keyed_layout_l_pushed_by_a_list_row() {
        keyed_layout('L');
    }
    #[test]
    fn paint_order_12_keyed_layout_o_pushed_through_a_plain_node() {
        keyed_layout('O');
    }
    #[test]
    fn paint_order_12_keyed_layout_x_pushed_through_a_translated_box() {
        keyed_layout('X');
    }

    /// Test 12's guards, boxes keyed layout cannot move, all clear of E. M
    /// is absolute at (0, 0, 60 by 60), with insets on both axes, beside S,
    /// whose height is keyed 0 to 100. Q, relative, 300 by 60, holds only a
    /// paintless absolute child whose width is keyed 0 to 100. U, relative,
    /// 300 by 60 at y 100, follows an in-flow E at (100, 0) whose frame
    /// keys an S inside it: while E flies its anchor holds the placeholder
    /// that `remember` kept, so nothing the frame keys moves U. Y, relative,
    /// 300 by 60, follows a translated wrapper that holds S, keyed to 100,
    /// and is absolute, so it pushes nothing. Each is measured, so nothing
    /// covers E and the flight is drawn last at every frame, both ways.
    fn keyed_layout_measured(which: char) {
        let mut s = PaintScene::new(PaintCase::KeyedLayout(which));
        let name = which.to_string();
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            assert!(
                !overlaps(f.get("E").rect, f.get(&name).rect),
                "{}: {name} stays clear of E",
                f.label
            );
            f.tail(&["E"]);
        });
    }
    #[test]
    fn paint_order_12_keyed_layout_m_insets_on_both_axes_stay_measured() {
        keyed_layout_measured('M');
    }
    #[test]
    fn paint_order_12_keyed_layout_q_absolute_keyed_child_leaves_its_parent_measured() {
        keyed_layout_measured('Q');
    }
    #[test]
    fn paint_order_12_keyed_layout_u_a_flown_frames_key_leaves_what_follows_measured() {
        keyed_layout_measured('U');
    }
    #[test]
    fn paint_order_12_keyed_layout_y_an_absolute_translated_box_pushes_nothing() {
        keyed_layout_measured('Y');
    }

    /// Test 13, bound z-index: E in flow (20, 20), then a relative C (0,
    /// 40, 200 by 100), its z-index bound to B's "z", 2. Mid-flight "z"
    /// drops to -1. Before the drop the flight is drawn under C; in the
    /// frame after it, above C, with nothing covering. The order is sorted
    /// again in that frame and in no other mid-flight frame.
    #[test]
    fn paint_order_13_a_bound_z_index_drop_uncovers_the_flight() {
        let mut s = PaintScene::new(PaintCase::BoundZIndex);
        let z = property(&vm(&s.row("B")), "z");
        s.begin("B", "go");
        let mut sorts = None;
        for p in PROGRESS {
            if p == 0.5 {
                set(&z, 575, -1.0);
            }
            s.to(p);
            let f = s.frame(Way::Go, p);
            f.meets("E", "C");
            if p < 0.5 {
                f.under("E", "C");
            } else {
                f.under("C", "E");
                f.tail(&["E"]);
            }
            let now = s.sorts("B");
            if let Some(before) = sorts {
                if p == 0.5 {
                    assert_ne!(now, before, "the drop sorts B again");
                } else {
                    assert_eq!(now, before, "no sort at {p}");
                }
            }
            sorts = Some(now);
        }
        s.end("B", "go");
    }

    /// Test 14, hinge (round 4 F1): container R's record display is block
    /// in one state and a flex column in the other. It holds E (20, 20) and
    /// a static z-index 2 child K (0, 40, 200 by 100) after E. The flight is
    /// drawn under K in both states.
    fn hinge(flex: bool) {
        let mut s = PaintScene::new(PaintCase::Hinge { flex });
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            f.meets("E", "K");
            f.under("E", "K");
        });
    }
    #[test]
    fn paint_order_14_hinge_block() {
        hinge(false);
    }
    #[test]
    fn paint_order_14_hinge_flex() {
        hinge(true);
    }

    /// Test 15, sizes (round 4 F3): the page resized as a host resizes it,
    /// which lays both Screens out at its size, E's slot written centered
    /// (w / 2 - 50, h / 2 - 30, 100 by 60), and C, 80 by 60 at the bottom
    /// right corner (w - 80, h - 60). By those numbers they overlap only
    /// while w < 260 and h < 180. The flight is drawn under C where they
    /// overlap, and on top elsewhere.
    fn sizes(w: f32, h: f32) {
        let mut s = PaintScene::new(PaintCase::Sizes);
        s.root.set_size(w, h);
        for row in ["A", "B"] {
            let record = record_of(&s.slot(row, "E"));
            set(&record, 61061, w / 2.0 - 50.0);
            set(&record, 61052, h / 2.0 - 30.0);
        }
        s.advance(0.0);
        let covers = w < 260.0 && h < 180.0;
        let rest = s.painted("B", &format!("{w} by {h} at rest"));
        let e = rect(
            w / 2.0 - 50.0,
            h / 2.0 - 30.0,
            w / 2.0 + 50.0,
            h / 2.0 + 30.0,
        );
        assert_rect("E at rest", rest.get("E").rect, e);
        assert_rect(
            "C at rest",
            rest.get("C").rect,
            rect(w - 80.0, h - 60.0, w, h),
        );
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            assert_eq!(
                overlaps(f.get("E").rect, f.get("C").rect),
                covers,
                "{}: E and C",
                f.label
            );
            if covers {
                f.under("E", "C");
            } else {
                f.under("C", "E");
            }
        });
    }
    #[test]
    fn paint_order_15_sizes_200_by_150() {
        sizes(200.0, 150.0);
    }
    #[test]
    fn paint_order_15_sizes_200_by_300() {
        sizes(200.0, 300.0);
    }
    #[test]
    fn paint_order_15_sizes_400_by_150() {
        sizes(400.0, 150.0);
    }
    #[test]
    fn paint_order_15_sizes_400_by_300() {
        sizes(400.0, 300.0);
    }

    /// Test 16, two versions under a coverer: Ea on A and Eb on B, both at
    /// (100, 100, 100 by 60), and C (150, 120, 100 by 60) on B. Both flights
    /// are drawn before C, and the upper Screen's version, Eb, after Ea,
    /// both ways.
    #[test]
    fn paint_order_16_two_versions_fly_under_the_coverer() {
        let mut s = PaintScene::new(PaintCase::TwoVersions);
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            f.meets("Ea", "C");
            f.meets("Eb", "C");
            f.order(&["Ea", "Eb", "C"]);
        });
    }

    /// Test 17 (a), files without flights: a CSS file with no live-move
    /// records, where a static plain `NestedArtboard` with a record holds a
    /// CSS frame turned at rest and a later in-flow box C overlaps it. The
    /// box is drawn after the host, as on main, and nothing sorts the order
    /// again over later frames.
    #[test]
    fn paint_order_17a_a_file_without_flights_keeps_its_order() {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &fixture::css_paint_plain_host_fixture(),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("fixture imports");
        let board = file.with_file(|f| f.artboard_named("Plain")).unwrap();
        let player = board.default_state_machine_handle().unwrap();
        player.advance_and_apply(0.0);
        let sorts = board.with_artboard(|b| b.draw_order_change_counter());
        for frame in 0..3 {
            let f = painted(&factory, &board, format!("plain host, frame {frame}"));
            f.meets("E", "C");
            f.order(&["E", "C"]);
            assert_eq!(
                board.with_artboard(|b| b.draw_order_change_counter()),
                sorts
            );
            player.advance_and_apply(0.1);
        }
    }

    /// Test 17 (a), plain-host anchor: in a live-move page, a static plain
    /// host at (50, 40) holds E's CSS frame, turned at rest, and a later
    /// in-flow box C (0, 0, 200 by 100) overlaps E. C's fill is drawn after
    /// the flight in every frame, both ways, as C is drawn after E at rest.
    #[test]
    fn paint_order_17a_plain_host_anchor_flies_under_the_later_box() {
        let mut s = PaintScene::new(PaintCase::PlainAnchor);
        let rest = s.painted("B", "at rest");
        rest.meets("E", "C");
        rest.under("E", "C");
        s.both_ways(|s, way, p| {
            let f = s.frame(way, p);
            f.meets("E", "C");
            f.under("E", "C");
        });
    }

    /// Test 17 (b): after landing, B's stream equals its stream at rest
    /// before the step.
    #[test]
    fn paint_order_17b_the_stream_after_landing_matches_rest() {
        let mut s = PaintScene::new(PaintCase::TurnedFrame);
        let before = s.stream("B");
        assert!(before.iter().any(|line| line.starts_with("drawPath")));
        s.fly(Way::Go, |_, _| {});
        assert_eq!(s.stream("B"), before);
    }

    /// Test 17 (c): with E's frame turned at rest (a stacking context),
    /// `draw_order_change_counter` changes at take-off and at landing, and
    /// at no frame between, both ways.
    #[test]
    fn paint_order_17c_the_order_sorts_only_at_take_off_and_landing() {
        let mut s = PaintScene::new(PaintCase::TurnedFrame);
        for (way, to, step) in [(Way::Go, "B", "go"), (Way::Back, "A", "back")] {
            let start = s.sorts("B");
            s.begin(to, step);
            s.to(0.0);
            let flying = s.sorts("B");
            assert_ne!(flying, start, "{way:?}: take-off sorts B");
            for p in &PROGRESS[1..] {
                s.to(*p);
                assert_eq!(s.sorts("B"), flying, "{way:?}: no sort at {p}");
            }
            s.end(to, step);
            assert_ne!(s.sorts("B"), flying, "{way:?}: landing sorts B");
        }
    }

    /// Test 17 (d): E and C inside a group M masked by S. The flight is
    /// outside the mask's run of members in B's draw list.
    #[test]
    fn paint_order_17d_the_flight_stays_out_of_the_mask_run() {
        let mut s = PaintScene::new(PaintCase::MaskedGroup);
        let run = |s: &PaintScene, slot: &str| {
            let item = RuntimeDrawableOccurrence::authored(s.slot("B", slot));
            let (mask, items) = s
                .row("B")
                .with_artboard(|b| (b.all_layer_masks()[0].clone(), b.drawables().to_vec()));
            let members: Vec<usize> = items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.layer_masks().contains(&mask))
                .map(|(at, _)| at)
                .collect();
            assert!(!members.is_empty(), "the mask has members");
            let at = items.iter().position(|i| i == &item).expect("listed");
            (members[0], at, *members.last().unwrap())
        };
        // The reading's positive control: at rest E's slot is in the run.
        let (first, at, last) = run(&s, "E");
        assert!(first <= at && at <= last, "E is masked at rest");
        s.both_ways(|s, way, p| {
            let (first, at, last) = run(s, "E flight");
            assert!(
                at < first || at > last,
                "{way:?} at {p}: the flight ({at}) is in the mask's run {first} to {last}"
            );
        });
    }

    /// Test 18 (a), frame context: E's frame is turned at rest, so its slot
    /// is a stacking context, and a later in-flow C (0, 50, 200 by 100)
    /// overlaps E (36, 20, 100 by 60). C's fill is drawn before the flight
    /// in every frame, as it is drawn before E at rest; just before
    /// take-off and just after landing, C is drawn before E.
    #[test]
    fn paint_order_18a_a_turned_frame_keeps_its_layer_in_flight() {
        let mut s = PaintScene::new(PaintCase::TurnedFrame);
        for way in [Way::Go, Way::Back] {
            let before = s.painted("B", &format!("before {way:?}"));
            before.meets("E", "C");
            before.under("C", "E");
            s.fly(way, |s, p| {
                let f = s.frame(way, p);
                f.meets("E", "C");
                f.under("C", "E");
            });
            if way == Way::Go {
                let after = s.painted("B", "after landing");
                after.under("C", "E");
            }
        }
    }

    /// Test 18 (b): E's frame is not turned. Mid-flight the test writes the
    /// flying frame's opacity to 0.5, which makes E's slot a stacking
    /// context. Before the write the flight is drawn before C; in the frame
    /// after it, after C. The order is sorted again in that frame and in no
    /// other mid-flight frame.
    #[test]
    fn paint_order_18b_an_opacity_write_lifts_the_flight_over_c() {
        let mut s = PaintScene::new(PaintCase::FadeWrite);
        s.begin("B", "go");
        let mut sorts = None;
        for p in PROGRESS {
            if p == 0.5 {
                let frame = child(&s.slot("B", "E flight")).unwrap();
                set(&frame.core_handle(), 18, 0.5);
            }
            s.to(p);
            let f = s.frame(Way::Go, p);
            f.meets("E", "C");
            if p < 0.5 {
                f.under("E", "C");
            } else {
                f.under("C", "E");
            }
            let now = s.sorts("B");
            if let Some(before) = sorts {
                if p == 0.5 {
                    assert_ne!(now, before, "the write sorts B again");
                } else {
                    assert_eq!(now, before, "no sort at {p}");
                }
            }
            sorts = Some(now);
        }
        s.end("B", "go");
    }

    /// Test 18 (c), round 5 F1: E's frame rests at opacity 0.5, so its slot
    /// is a stacking context and paints after a later in-flow C. E flies in
    /// a native layout flight slot, and again in a CSS one. Mid-flight the
    /// test writes the flying frame's opacity to 1. Before the write the
    /// flight is drawn after C, as E is at rest; in the first frame drawn
    /// after the write, before C. In the CSS flight slot the order is
    /// sorted again in that frame and in no other mid-flight frame.
    fn fade_at_rest(native: bool) {
        let mut s = PaintScene::new(PaintCase::FadeAtRest { native });
        let rest = s.painted("B", "at rest");
        rest.meets("E", "C");
        rest.under("C", "E");
        s.begin("B", "go");
        let mut sorts = None;
        for p in PROGRESS {
            if p == 0.5 {
                let frame = child(&s.slot("B", "E flight")).unwrap();
                set(&frame.core_handle(), 18, 1.0);
            }
            s.to(p);
            let f = s.frame(Way::Go, p);
            f.meets("E", "C");
            if p < 0.5 {
                f.under("C", "E");
            } else {
                f.under("E", "C");
            }
            if !native {
                let now = s.sorts("B");
                if let Some(before) = sorts {
                    if p == 0.5 {
                        assert_ne!(now, before, "the write sorts B again");
                    } else {
                        assert_eq!(now, before, "no sort at {p}");
                    }
                }
                sorts = Some(now);
            }
        }
        s.end("B", "go");
    }
    #[test]
    fn paint_order_18c_opacity_at_rest_native_flight_slot() {
        fade_at_rest(true);
    }
    #[test]
    fn paint_order_18c_opacity_at_rest_css_flight_slot() {
        fade_at_rest(false);
    }

    /// Test 19 (b), cancel with no home: two versions of E, turned at rest,
    /// fly "go"; C covers B's E, and E2 rests earlier on B (20, 20, 60 by
    /// 40), where C does not cover it. Removing A mid-flight cancels the
    /// step, which sorts B once. With A mounted again, "back2" flies only
    /// E2 from B to A: its flight is drawn last, after C.
    #[test]
    fn paint_order_19b_a_cancelled_step_leaves_no_placement() {
        let mut s = PaintScene::new(PaintCase::Cancel);
        let page = RuntimeOwnedViewModelHandle::from_native(
            s._file.clone(),
            s.root
                .data_context()
                .unwrap()
                .with_context(|c| c.main_view_model_instance())
                .unwrap(),
        )
        .unwrap();
        let outgoing = page.list_item_by_property_name_path("screens", 0).unwrap();
        s.begin("B", "go");
        s.to(0.0);
        let flying = s.sorts("B");
        for p in [0.25, 0.5] {
            s.to(p);
            assert_eq!(s.sorts("B"), flying, "no sort at {p}");
        }
        assert!(page.remove_list_item_by_property_name_path("screens", 0));
        s.advance(0.0);
        assert!(s.moves.flights(&s.root).is_empty(), "removing A cancels");
        assert_ne!(s.sorts("B"), flying, "the cancel sorts B");
        assert!(page.insert_list_item_by_property_name_path("screens", 0, &outgoing));
        s.advance(0.0);
        let mut sorts = None;
        s.fly_step("A", "back2", |s, p| {
            let f = s.frame(Way::Back, p);
            f.under("C", "E2");
            f.tail(&["E2"]);
            let now = s.sorts("B");
            if let Some(before) = sorts {
                assert_eq!(now, before, "no sort at {p}");
            }
            sorts = Some(now);
        });
    }
}

/// What the one reader of an element as drawn rests on (UNIV-5653). Every
/// geometric oracle here reads what the renderer got, through `frame_draws`.
mod drawn {
    use super::*;
    use fixture::PaintCase;
    use nuxie_render_api as render;
    use nuxie_render_stream::{DrawnClip, FrameDraw, RoundedRect, frame_draws, stream_ops};

    /// A page mounted as a host mounts a published page.
    struct DrawnScene {
        _file: RuntimeFileHandle,
        factory: PersistentFactory<RecordingFactory>,
        root: RuntimeArtboardInstanceHandle,
        machine: RuntimeStateMachineInstanceHandle,
        moves: LiveMoves,
    }
    impl DrawnScene {
        fn new(case: PaintCase) -> Self {
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            let file = File::import(
                &fixture::css_paint_fixture(case),
                RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
                None,
                None,
                None,
            )
            .expect("fixture imports");
            let root = file.with_file(|f| f.artboard_named("Page")).unwrap();
            let model = file
                .with_file(|f| {
                    f.create_default_view_model_instance_for_artboard(root.core_handle())
                })
                .unwrap();
            let machine = root.default_state_machine_handle().unwrap();
            machine.with_instance_mut(|m| m.bind_view_model_instance(model));
            machine.advance_and_apply(0.0);
            let mut moves = LiveMoves::default();
            moves.prepare(&root).unwrap();
            let mut scene = Self {
                _file: file,
                factory,
                root,
                machine,
                moves,
            };
            scene.advance(0.0);
            scene
        }
        fn row(&self, name: &str) -> RuntimeArtboardInstanceHandle {
            screens(&self.root).unwrap()[name].clone()
        }
        fn slot(&self, row: &str, name: &str) -> CoreHandle {
            self.row(row)
                .with_artboard(|r| r.find_handle::<NestedArtboard>(name))
                .unwrap_or_else(|| panic!("no slot {name} on {row}"))
        }
        fn advance(&mut self, dt: f32) {
            self.machine.advance_and_apply(dt);
            self.moves.after_advance(&self.root).unwrap();
        }
        fn draws(&self) -> Vec<FrameDraw> {
            page_draws(&self.factory, &self.root)
        }
    }

    fn recorded(
        factory: &PersistentFactory<RecordingFactory>,
        board: &RuntimeArtboardInstanceHandle,
    ) -> String {
        factory.borrow_mut().clear();
        let mut renderer = factory.borrow().make_renderer();
        board.draw(&mut renderer);
        factory.borrow().stream()
    }
    fn page_draws(
        factory: &PersistentFactory<RecordingFactory>,
        board: &RuntimeArtboardInstanceHandle,
    ) -> Vec<FrameDraw> {
        frame_draws(&stream_ops(&recorded(factory, board)).unwrap(), &[])
    }
    fn of_rgb(draws: &[FrameDraw], rgb: u32) -> Vec<&FrameDraw> {
        draws
            .iter()
            .filter(|draw| draw.rgb() == Some(rgb & 0x00ff_ffff))
            .collect()
    }
    /// The one draw of a color: the frame's own background, whose matrix is the
    /// frame's content matrix.
    fn one_of<'a>(draws: &'a [FrameDraw], rgb: u32, label: &str) -> &'a FrameDraw {
        let found = of_rgb(draws, rgb);
        assert_eq!(found.len(), 1, "{label}: one draw of {rgb:08x}");
        found[0]
    }
    fn drawn_size(draw: &FrameDraw) -> (f32, f32) {
        let (mut left, mut top) = (f32::INFINITY, f32::INFINITY);
        let (mut right, mut bottom) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        for point in &draw.points {
            left = left.min(point.x);
            right = right.max(point.x);
            top = top.min(point.y);
            bottom = bottom.max(point.y);
        }
        (right - left, bottom - top)
    }

    /// R0. Calmly's curve takes y above 1, and Rive's cubic easing doesn't clamp
    /// it, so a key on that curve passes its end value.
    #[test]
    fn a_cubic_easing_keys_past_its_end() {
        let mut s = DrawnScene::new(PaintCase::EasedScale);
        s.advance(0.873);
        let draws = s.draws();
        let (x, y) = one_of(&draws, fixture::paint_color("E"), "at 0.873").axis_scales();
        assert!(x > 2.0 && y > 2.0, "drawn scale {x}, {y} is not above 2");
        assert!(
            x < 2.05 && y < 2.05,
            "drawn scale {x}, {y} is past the curve"
        );
    }

    /// R0b. `step_begin` binds and lays out the arriving Screen before the plan
    /// reads any slot, so a slot's place read straight after it is where the
    /// slot rests after landing.
    #[test]
    fn the_arriving_screen_is_laid_out_when_a_step_begins() {
        let mut s = DrawnScene::new(PaintCase::BoundSubtitle);
        let arriving = s.slot("B", "E");
        let one_line = slot_pose(&arriving).unwrap();
        assert!(CoreRegistry::set_string_handle(
            &property(&vm(&s.row("B")), "subtitle"),
            561,
            "A subtitle long enough to wrap onto a second line once it is bound. ".repeat(3),
        ));
        s.moves.step_begin(&s.root, "B", Some("go")).unwrap();
        let at_takeoff = slot_pose(&arriving).unwrap();
        assert!(
            at_takeoff.transform[5] > one_line.transform[5] + 20.0,
            "the bound text must move the slot: {} from {}",
            at_takeoff.transform[5],
            one_line.transform[5]
        );
        for flight in s.moves.flights(&s.root) {
            set(&flight.controls(), 62003, 1.0);
        }
        s.advance(0.1);
        s.moves.step_end(&s.root, "B", Some("go")).unwrap();
        s.advance(0.0);
        let landed = slot_pose(&arriving).unwrap();
        for i in 0..6 {
            assert!(
                (at_takeoff.transform[i] - landed.transform[i]).abs() < 1e-3,
                "component {i}: {} at takeoff, {} after landing",
                at_takeoff.transform[i],
                landed.transform[i]
            );
        }
    }

    /// R0c. A flight slot whose record has size mode 1 is laid out at the blend
    /// of its two slots' sizes: the frame's background is that size with no
    /// scale on it.
    #[test]
    fn a_layout_flight_lays_out_at_the_blend() {
        let mut s = corner_scene(&fixture::css_two_version_corner_fixture(false));
        s.begin();
        for flight in s.moves.flights(&s.root) {
            set(&flight.controls(), 62003, 0.5);
        }
        s.advance(0.0);
        let draws = page_draws(&s.factory, &s.root);
        for (version, rgb) in [("a", 0xcc3333), ("b", 0x3333cc)] {
            let draw = one_of(&draws, rgb, version);
            let (width, height) = drawn_size(draw);
            close(width, (80.0 + 160.0) / 2.0);
            close(height, (60.0 + 120.0) / 2.0);
            let (x, y) = draw.axis_scales();
            close(x, 1.0);
            close(y, 1.0);
        }
    }

    /// R0d. A frame's own opacity multiplies its flight slot's fade, which is
    /// why a zoomed pair leaves that value out.
    #[test]
    fn a_frames_own_opacity_multiplies_its_slots_fade() {
        let rgb = fixture::paint_color("E");
        let halfway = |own: f32| {
            let mut s = DrawnScene::new(PaintCase::OwnOpacity(own));
            s.moves.step_begin(&s.root, "B", Some("go")).unwrap();
            let flights = s.moves.flights(&s.root);
            assert_eq!(flights.len(), 1, "one version flies");
            set(&flights[0].controls(), 62003, 0.5);
            s.advance(0.1);
            close(val(&flights[0].controls(), 62014), 0.5);
            s.draws()
        };
        let draws = halfway(0.5);
        let draw = one_of(&draws, rgb, "own opacity 0.5");
        assert!(
            (draw.opacity() - 0.25).abs() < 0.003,
            "drawn at {}, under {:?}, in {:08x}",
            draw.opacity(),
            draw.modulations,
            draw.color.unwrap()
        );
        let draws = halfway(0.0);
        assert!(
            of_rgb(&draws, rgb).is_empty(),
            "a frame at its own 0 draws nothing"
        );
    }

    /// R1l. The runtime's harness has no canvas host, so a frame fading as one
    /// picture takes the scope fallback here, not the canvas path a device
    /// takes. Read from the raw stream. It is why the reader's test of a frame
    /// fading as one picture runs in the renderer's harness.
    #[test]
    fn the_runtime_harness_fades_a_picture_by_scope() {
        use render::Factory;
        let mut s = corner_scene(&fixture::css_box_root_versions_fixture());
        assert!(s.factory.borrow_mut().canvas_content_host().is_none());
        s.begin();
        for flight in s.moves.flights(&s.root) {
            set(&flight.controls(), 62003, 0.5);
            set(&flight.controls(), 62014, 0.5);
        }
        s.advance(0.0);
        let stream = recorded(&s.factory, &s.root);
        let child = format!("color=0x{:08x}", fixture::CSS_BOX_ROOT_CHILD);
        // Every unit's child has one color, so a child is told by the
        // background it was drawn over. Step "t1" flies a and b; c rests at
        // both its homes, one on each Screen, faded by nothing.
        let (mut scopes, mut saved) = (Vec::new(), Vec::new());
        let mut over: Option<(usize, Vec<f32>)> = None;
        let (mut backgrounds, mut children) = ([0; 3], [0; 3]);
        for line in stream.lines() {
            if line == "save" {
                saved.push(scopes.len());
            } else if line == "restore" {
                scopes.truncate(saved.pop().unwrap());
            } else if let Some(opacity) = line.strip_prefix("modulateOpacity opacity=") {
                scopes.push(opacity.parse::<f32>().unwrap());
            } else if line.starts_with("drawPath ") {
                if let Some(unit) = [UNIT_A, UNIT_B, UNIT_C]
                    .iter()
                    .position(|color| line.contains(color))
                {
                    let fade = if unit == 2 { vec![] } else { vec![0.5] };
                    assert_eq!(scopes, fade, "the scopes around {line}");
                    backgrounds[unit] += 1;
                    over = Some((unit, fade));
                } else if line.contains(&child) {
                    let (unit, fade) = over.take().expect("a child over its background");
                    assert_eq!(scopes, fade, "the scopes around {line}");
                    children[unit] += 1;
                }
            }
        }
        assert_eq!(backgrounds, [1, 1, 2], "each unit's background is drawn");
        assert_eq!(children, [1, 1, 2], "each unit's child is drawn");
        assert!(!stream.contains("drawImage"), "no picture is drawn");
    }

    /// A 100 by 60 box's clip, as a CSS box's rounded rectangle winds, turned
    /// `turn` about its top left, with every corner's two radii.
    fn turned_clip(
        factory: &mut RecordingFactory,
        renderer: &mut dyn render::Renderer,
        turn: f32,
        (rx, ry): (f32, f32),
    ) {
        use render::{Factory, FillRule, Mat2D, RawPath};
        const KAPPA: f32 = 0.552_284_8;
        let (w, h) = (100.0, 60.0);
        let mut path = RawPath::new();
        path.move_to(rx, 0.0);
        path.line_to(w - rx, 0.0);
        path.cubic_to(w - rx + KAPPA * rx, 0.0, w, ry - KAPPA * ry, w, ry);
        path.line_to(w, h - ry);
        path.cubic_to(w, h - ry + KAPPA * ry, w - rx + KAPPA * rx, h, w - rx, h);
        path.line_to(rx, h);
        path.cubic_to(rx - KAPPA * rx, h, 0.0, h - ry + KAPPA * ry, 0.0, h - ry);
        path.line_to(0.0, ry);
        path.cubic_to(0.0, ry - KAPPA * ry, rx - KAPPA * rx, 0.0, rx, 0.0);
        path.close();
        let (sin, cos) = turn.sin_cos();
        renderer.transform(Mat2D([cos, sin, -sin, cos, 30.0, 40.0]));
        let clip = factory.make_render_path(path, FillRule::NonZero);
        renderer.clip_path(clip.as_ref());
    }

    /// A small square drawn in `color`, under what is in scope.
    fn dab(factory: &mut RecordingFactory, renderer: &mut dyn render::Renderer, color: u32) {
        use render::{Factory, FillRule, RawPath};
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(1.0, 0.0);
        path.line_to(1.0, 1.0);
        path.close();
        let path = factory.make_render_path(path, FillRule::NonZero);
        let mut paint = factory.make_render_paint();
        paint.color(color);
        renderer.draw_path(path.as_ref(), paint.as_ref());
    }

    /// R1k. The oracle sees a turn and a corner a bounding box would hide, and
    /// reads an opacity from both of the places it can be.
    #[test]
    fn frame_draws_sees_a_wrong_turn_and_a_wrong_corner() {
        use render::Renderer;
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        for (turn, radii) in [(0.2, (6.0, 4.0)), (0.19, (6.0, 4.0)), (0.2, (6.0, 3.5))] {
            renderer.save();
            turned_clip(&mut factory, &mut renderer, turn, radii);
            dab(&mut factory, &mut renderer, 0xff00_0000);
            renderer.restore();
        }
        renderer.save();
        renderer.modulate_opacity(0.5);
        dab(&mut factory, &mut renderer, 0xff10_2030);
        renderer.restore();
        dab(&mut factory, &mut renderer, 0x8010_2030);
        renderer.save();
        renderer.modulate_opacity(0.5);
        dab(&mut factory, &mut renderer, 0x8010_2030);
        renderer.restore();
        renderer.save();
        renderer.modulate_opacity(-0.01);
        dab(&mut factory, &mut renderer, 0xff10_2030);
        renderer.restore();
        let draws = frame_draws(&stream_ops(&factory.stream()).unwrap(), &[]);
        assert_eq!(draws.len(), 7);

        let read = |draw: &FrameDraw| {
            let clip: &DrawnClip = &draw.clips[0];
            RoundedRect::read(clip).expect("a rounded rectangle")
        };
        let (right, wrong_turn, wrong_corner) = (read(&draws[0]), read(&draws[1]), read(&draws[2]));
        assert!((right.width - 100.0).abs() < 1e-3, "{right:?}");
        assert!((right.height - 60.0).abs() < 1e-3, "{right:?}");
        assert!((right.turn - 0.2).abs() < 1e-5, "{right:?}");
        for (x, y) in right.radii {
            assert!(
                (x - 6.0).abs() < 1e-3 && (y - 4.0).abs() < 1e-3,
                "{right:?}"
            );
        }
        assert_eq!(right.difference(&right), None);
        assert!(right.difference(&wrong_turn).is_some(), "a wrong turn");
        assert!(right.difference(&wrong_corner).is_some(), "a wrong corner");

        let near = |a: f32, b: f32| assert!((a - b).abs() < 0.003, "expected {b}, got {a}");
        near(draws[3].opacity(), 0.5);
        assert_eq!(draws[3].modulations, [0.5]);
        near(draws[4].opacity(), 0.5);
        assert!(draws[4].modulations.is_empty());
        assert_eq!(draws[4].color, Some(0x8010_2030));
        near(draws[5].opacity(), 0.25);
        assert_eq!(draws[6].opacity(), 0.0);
    }

    /// R2m. Placement keeps a level's bound scale beside a keyed turn: with the
    /// keys running, the matrix it gives each case's slot is the frame's content
    /// matrix in the draw recorded after the keys end. The claim it pins is the
    /// defect PR 1 fixes: today a marked level is taken whole at identity.
    #[test]
    fn resting_boxes_keep_a_bound_value_beside_a_key() {
        let mut s = DrawnScene::new(PaintCase::BoundBesideKey);
        let cases = [("held", s.slot("B", "held")), ("own", s.slot("B", "own"))];
        s.advance(0.5);
        let draws = s.draws();
        for (name, _) in &cases {
            let [xx, xy, ..] = one_of(&draws, fixture::paint_color(name), name).matrix.0;
            close(xy.atan2(xx), 0.15);
        }
        let resting: Vec<_> = cases
            .iter()
            .map(|(name, slot)| {
                s.moves
                    .resting_slot(slot)
                    .unwrap_or_else(|| panic!("{name} has a resting place"))
            })
            .collect();
        s.advance(1.0);
        let draws = s.draws();
        let mut wrong = Vec::new();
        for ((name, _), rest) in cases.iter().zip(resting) {
            let draw = one_of(&draws, fixture::paint_color(name), name);
            let [xx, xy, ..] = draw.matrix.0;
            close(xy.atan2(xx), 0.0);
            let (x, y) = draw.axis_scales();
            close(x, 1.5);
            close(y, 1.5);
            if (0..6).any(|i| (rest[i] - draw.matrix.0[i]).abs() > 1e-3) {
                wrong.push(format!(
                    "{name}: rests at {:?}, drawn at {:?}",
                    (0..6).map(|i| rest[i]).collect::<Vec<_>>(),
                    draw.matrix.0
                ));
            }
        }
        assert!(wrong.is_empty(), "{}", wrong.join("; "));
    }

    type Matrix = source::math::mat2d::Mat2D;

    fn runtime_matrix(matrix: render::Mat2D) -> Matrix {
        let [xx, xy, yx, yy, tx, ty] = matrix.0;
        Matrix::new(xx, xy, yx, yy, tx, ty)
    }
    fn differs(a: Matrix, b: Matrix) -> bool {
        (0..6).any(|i| (a[i] - b[i]).abs() > 1e-3)
    }
    /// The reader's matrix is the draw's, to 1e-3.
    fn assert_reads(read: Matrix, draw: &FrameDraw, label: &str) {
        assert!(
            !differs(read, runtime_matrix(draw.matrix)),
            "{label}: reads {:?}, the draw has {:?}",
            read.values(),
            draw.matrix.0
        );
    }
    fn turn(matrix: render::Mat2D) -> f32 {
        matrix.0[1].atan2(matrix.0[0])
    }

    /// R1a to R1h's shared check, with B's keys running: `drawn(content, Live)`
    /// is the draw of `content`'s color halfway through E's flight, and again at
    /// rest in E's home after it lands. It gives the two matrices.
    fn reader_is_the_draw_of(
        kind: fixture::ReaderKind,
        name: &str,
        content: impl Fn(&RuntimeArtboardInstanceHandle) -> CoreHandle,
    ) -> (render::Mat2D, render::Mat2D) {
        let rgb = fixture::paint_color(name);
        let mut s = DrawnScene::new(PaintCase::Reader { kind, flies: true });
        s.advance(0.25);
        s.moves.step_begin(&s.root, "B", Some("go")).unwrap();
        let flights = s.moves.flights(&s.root);
        assert_eq!(flights.len(), 1, "E flies alone");
        set(&flights[0].controls(), 62003, 0.5);
        s.advance(0.0);
        let frame = child(&flights[0].flight).expect("E is in its flight slot");
        let read = s.moves.drawn(&content(&frame), When::Live).unwrap();
        let draws = s.draws();
        let draw = one_of(&draws, rgb, "in flight");
        assert_reads(read, draw, "in flight");
        let in_flight = draw.matrix;

        set(&flights[0].controls(), 62003, 1.0);
        s.advance(0.0);
        s.moves.step_end(&s.root, "B", Some("go")).unwrap();
        s.advance(0.0);
        let frame = child(&s.slot("B", "E")).expect("E is home");
        let read = s.moves.drawn(&content(&frame), When::Live).unwrap();
        let draws = s.draws();
        let draw = one_of(&draws, rgb, "at rest");
        assert_reads(read, draw, "at rest");
        (in_flight, draw.matrix)
    }
    fn reader_is_the_draw(kind: fixture::ReaderKind) -> (render::Mat2D, render::Mat2D) {
        reader_is_the_draw_of(kind, "E", RuntimeArtboardInstanceHandle::core_handle)
    }

    /// R1a. Layout's placement alone.
    #[test]
    fn the_reader_is_the_draw_placement() {
        let (in_flight, rest) = reader_is_the_draw(fixture::ReaderKind::Placement);
        close(in_flight.0[4], 78.0);
        close(in_flight.0[5], 55.0);
        close(rest.0[4], 120.0);
        close(rest.0[5], 90.0);
    }

    /// R1b. A CSS ancestor's keyed scale.
    #[test]
    fn the_reader_is_the_draw_ancestor_scale() {
        let (_, rest) = reader_is_the_draw(fixture::ReaderKind::CardScale);
        close(rest.0[0], 0.625);
        close(rest.0[3], 0.625);
    }

    /// R1c. The slot's own turn.
    #[test]
    fn the_reader_is_the_draw_slot_turn() {
        let (_, rest) = reader_is_the_draw(fixture::ReaderKind::SlotTurn);
        close(turn(rest), 0.4);
    }

    /// R1d. The frame's own keyed scale.
    #[test]
    fn the_reader_is_the_draw_frame_keyed_scale() {
        let (_, rest) = reader_is_the_draw(fixture::ReaderKind::FrameKeyedScale);
        let scale = rest.0[0];
        assert!(scale > 0.5 && scale < 1.0, "the key is running: {scale}");
    }

    /// R1e. The frame's own bound scale.
    #[test]
    fn the_reader_is_the_draw_frame_bound_scale() {
        let (_, rest) = reader_is_the_draw(fixture::ReaderKind::FrameBoundScale);
        close(rest.0[0], 2.0);
        close(rest.0[3], 2.0);
    }

    /// R1f. The frame's own turn.
    #[test]
    fn the_reader_is_the_draw_frame_turn() {
        let (_, rest) = reader_is_the_draw(fixture::ReaderKind::FrameTurn);
        close(turn(rest), 0.35);
    }

    /// R1g. A CSS-slot offset on a turned, scaled frame, which turns and scales
    /// about its pivot and not about the offset.
    #[test]
    fn the_reader_is_the_draw_css_slot_offset() {
        let (_, rest) = reader_is_the_draw(fixture::ReaderKind::SlotFrameOffset);
        close(turn(rest), 0.35);
        close(rest.0[0].hypot(rest.0[1]), 1.5);
        let (sin, cos) = 0.35f32.sin_cos();
        let (pivot_x, pivot_y) = (50.0, 30.0);
        close(
            rest.0[4],
            120.0 + pivot_x - 1.5 * (cos * pivot_x - sin * pivot_y),
        );
        let offset = rest.0[5] - (90.0 + pivot_y - 1.5 * (sin * pivot_x + cos * pivot_y));
        assert!(
            offset > 1.0 && offset < 12.0,
            "the y key is running: {offset}"
        );
    }

    /// R1h. A hosted frame with a fit inside the unit, read on its content.
    #[test]
    fn the_reader_is_the_draw_hosted_fit() {
        use source::nested_artboard_leaf::NestedArtboardLeaf;
        let (_, rest) = reader_is_the_draw_of(fixture::ReaderKind::HostedFit, "inner", |frame| {
            let leaf = frame.with_artboard(|unit| unit.find_all_handles::<NestedArtboardLeaf>());
            child(&leaf[0]).expect("the leaf hosts inner").core_handle()
        });
        close(rest.0[0], 2.0);
        close(rest.0[3], 2.0);
    }

    /// R1i. A Screen turned and scaled unequally, with its list-row matrix.
    #[test]
    fn the_reader_is_the_draw_turned_screen_on_a_row() {
        let mut s = Scene::from_bytes(&fixture::follow_path_fixture());
        set(&s.a.core_handle(), 16, 1.5);
        set(&s.a.core_handle(), 17, 0.75);
        set(&s.b.core_handle(), 16, 0.65);
        set(&s.b.core_handle(), 17, 1.3);
        s.advance(0.0);
        let list = s
            .root
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
            .unwrap();
        set_row_collapsed(&list, 1, true).unwrap();
        let read = s
            .moves
            .drawn(&child(&s.s).unwrap().core_handle(), When::Live)
            .unwrap();
        let draws = page_draws(&s.factory, &s.root);
        assert_eq!(draws.len(), 1, "one Screen draws");
        assert_reads(read, &draws[0], "at rest");
        let (x, y) = draws[0].axis_scales();
        assert!((x - y).abs() > 0.1, "scaled unequally: {x}, {y}");
        set_row_collapsed(&list, 1, false).unwrap();
        s.advance(0.0);
        s.begin();
        for flight in s.moves.flights(&s.root) {
            set(&flight.controls(), 62003, 0.5);
        }
        s.moves.after_advance(&s.root).unwrap();
        let read = s
            .moves
            .drawn(&child(&s.f).unwrap().core_handle(), When::Live)
            .unwrap();
        let draws = page_draws(&s.factory, &s.root);
        assert_eq!(draws.len(), 1, "the flight draws");
        assert_reads(read, &draws[0], "in flight");
        for flight in s.moves.flights(&s.root) {
            set(&flight.controls(), 62003, 1.0);
        }
        s.moves.after_advance(&s.root).unwrap();
        s.end();
        s.moves.after_advance(&s.root).unwrap();
        let read = s
            .moves
            .drawn(&child(&s.d).unwrap().core_handle(), When::Live)
            .unwrap();
        let draws = page_draws(&s.factory, &s.root);
        assert_eq!(draws.len(), 1, "one Screen draws");
        assert_reads(read, &draws[0], "at rest on the turned row");
        assert!(turn(draws[0].matrix).abs() > 1.0, "turned by its row");
        let (x, y) = draws[0].axis_scales();
        assert!((x - y).abs() > 0.1, "scaled unequally: {x}, {y}");
    }

    fn rest_scene(kind: fixture::ReaderKind) -> (DrawnScene, CoreHandle) {
        let s = DrawnScene::new(PaintCase::Reader { kind, flies: false });
        let frame = child(&s.slot("B", "E")).expect("E is home").core_handle();
        (s, frame)
    }

    /// R2a to R2d's shared check: halfway through a key that ends at identity,
    /// Rest isn't what draws now, and is what draws once the key has ended.
    fn rest_is_the_draw_once_the_keys_end(kind: fixture::ReaderKind) {
        let (mut s, frame) = rest_scene(kind);
        s.advance(0.5);
        let rest = s.moves.drawn(&frame, When::Rest).unwrap();
        assert!(
            differs(rest, s.moves.drawn(&frame, When::Live).unwrap()),
            "the key is running"
        );
        s.advance(1.0);
        let draws = s.draws();
        assert_reads(rest, one_of(&draws, fixture::paint_color("E"), "E"), "rest");
    }

    /// R2a. An ancestor's keyed scale.
    #[test]
    fn rest_is_the_draw_once_the_keys_end_ancestor_scale() {
        rest_is_the_draw_once_the_keys_end(fixture::ReaderKind::CardScale);
    }

    /// R2b. An ancestor's keyed turn.
    #[test]
    fn rest_is_the_draw_once_the_keys_end_ancestor_turn() {
        rest_is_the_draw_once_the_keys_end(fixture::ReaderKind::CardTurn);
    }

    /// R2c. The frame's own keyed scale.
    #[test]
    fn rest_is_the_draw_once_the_keys_end_frame_scale() {
        rest_is_the_draw_once_the_keys_end(fixture::ReaderKind::FrameKeyedScale);
    }

    /// R2d. An ancestor's keyed translate.
    #[test]
    fn rest_is_the_draw_once_the_keys_end_ancestor_translate() {
        rest_is_the_draw_once_the_keys_end(fixture::ReaderKind::CardTranslate);
    }

    /// R2e. A keyed layout width above: Rest reads layout as laid out when read,
    /// so it differs from the end halfway (a stated limit), and equals it there.
    #[test]
    fn rest_is_the_draw_once_the_keys_end_layout_width() {
        let (mut s, frame) = rest_scene(fixture::ReaderKind::CardWidth);
        s.advance(0.5);
        let halfway = s.moves.drawn(&frame, When::Rest).unwrap();
        s.advance(1.0);
        let draws = s.draws();
        let draw = one_of(&draws, fixture::paint_color("E"), "E");
        assert!(
            differs(halfway, runtime_matrix(draw.matrix)),
            "the width moves E: halfway {:?}, at the end {:?}",
            halfway.values(),
            draw.matrix.0
        );
        close(draw.matrix.0[4], 40.0 + 300.0 - 10.0 - 100.0);
        assert_reads(
            s.moves.drawn(&frame, When::Rest).unwrap(),
            draw,
            "at the end",
        );
    }

    /// R2i. A key that ends off identity is taken at identity, a stated limit:
    /// Rest is the recorded draw with that level's held factor taken out.
    #[test]
    fn rest_takes_a_key_that_ends_off_identity_at_identity() {
        let (mut s, frame) = rest_scene(fixture::ReaderKind::CardHeldScale);
        s.advance(1.5);
        let rest = s.moves.drawn(&frame, When::Rest).unwrap();
        let draws = s.draws();
        let card = runtime_matrix(one_of(&draws, fixture::paint_color("card"), "card").matrix);
        let draw = runtime_matrix(one_of(&draws, fixture::paint_color("E"), "E").matrix);
        // The card's own held transform: 0.5 about its center.
        let held = Matrix::new(0.5, 0.0, 0.0, 0.5, 50.0, 25.0);
        let without = card * held.invert_or_identity() * card.invert_or_identity() * draw;
        assert!(
            !differs(rest, without),
            "{:?}, {:?}",
            rest.values(),
            without.values()
        );
        assert!(differs(rest, draw), "it isn't what draws");
        close(rest[0], 1.0);
        close(rest[3], 1.0);
        close(rest[4], 50.0);
        close(rest[5], 40.0);
    }

    /// R2k. A level with a bound scale and a running keyed turn, on an ancestor
    /// and on the frame itself: Rest keeps the bound scale and takes the turn at
    /// identity.
    #[test]
    fn rest_keeps_a_bound_value_beside_a_key() {
        let mut s = DrawnScene::new(PaintCase::BoundBesideKey);
        let frames =
            ["held", "own"].map(|name| (name, child(&s.slot("B", name)).unwrap().core_handle()));
        s.advance(0.5);
        let rests = frames.clone().map(|(_, frame)| {
            let rest = s.moves.drawn(&frame, When::Rest).unwrap();
            assert!(
                differs(rest, s.moves.drawn(&frame, When::Live).unwrap()),
                "the turn is running"
            );
            rest
        });
        s.advance(1.0);
        let draws = s.draws();
        for ((name, _), rest) in frames.iter().zip(rests) {
            let draw = one_of(&draws, fixture::paint_color(name), name);
            close(turn(draw.matrix), 0.0);
            close(draw.matrix.0[0], 1.5);
            close(draw.matrix.0[3], 1.5);
            assert_reads(rest, draw, name);
        }
    }

    /// R2n. A CSS-slot frame's own keyed offset and turn and its bound x, about
    /// its own origin: the artboard level's transform, not a CSS box's.
    #[test]
    fn rest_is_the_draw_of_a_css_slot_frames_own_keys() {
        let (mut s, frame) = rest_scene(fixture::ReaderKind::SlotFrameKeys);
        s.advance(0.5);
        let rest = s.moves.drawn(&frame, When::Rest).unwrap();
        assert!(
            differs(rest, s.moves.drawn(&frame, When::Live).unwrap()),
            "the keys are running"
        );
        s.advance(1.0);
        let draws = s.draws();
        let draw = one_of(&draws, fixture::paint_color("E"), "E");
        close(draw.matrix.0[4], 128.0);
        close(draw.matrix.0[5], 90.0);
        assert_reads(rest, draw, "rest");
    }

    /// Smaller than E's 100 by 60 frame, with each corner's two radii unequal.
    const CLIP_SIZE: (f32, f32) = (80.0, 50.0);
    const CLIP_RADII: [(f32, f32); 4] = [(6.0, 4.0), (10.0, 3.0), (2.0, 8.0), (5.0, 1.0)];

    /// E, halfway through its flight, with its flight clip set.
    fn clipped_flight() -> DrawnScene {
        use source::{artboard::FlightClip, math::vec2d::Vec2D};
        let mut s = DrawnScene::new(PaintCase::TurnedInClippingHolder);
        s.moves.step_begin(&s.root, "B", Some("go")).unwrap();
        let flights = s.moves.flights(&s.root);
        assert_eq!(flights.len(), 1, "E flies alone");
        set(&flights[0].controls(), 62003, 0.5);
        s.advance(0.0);
        let frame = child(&flights[0].flight).expect("E is in its flight slot");
        let clip = FlightClip {
            width: CLIP_SIZE.0,
            height: CLIP_SIZE.1,
            radii: CLIP_RADII.map(|(x, y)| Vec2D::new(x, y)),
        };
        s.moves.set_flight_clip(&frame, Some(clip));
        s.advance(0.0);
        s
    }
    fn land_flight(s: &mut DrawnScene) {
        for flight in s.moves.flights(&s.root) {
            set(&flight.controls(), 62003, 1.0);
        }
        s.advance(0.0);
        s.moves.step_end(&s.root, "B", Some("go")).unwrap();
        s.advance(0.0);
    }
    /// The flight clip mapped by a frame's recorded content matrix: where its
    /// background is drawn is where its clip must be.
    fn clip_on(frame: &FrameDraw) -> RoundedRect {
        let [xx, xy, yx, yy, tx, ty] = frame.matrix.0;
        let at = |x: f32, y: f32| render::Vec2D::new(xx * x + yx * y + tx, xy * x + yy * y + ty);
        let (scale_x, scale_y) = frame.axis_scales();
        let (width, height) = CLIP_SIZE;
        RoundedRect {
            center: at(width / 2.0, height / 2.0),
            width: width * scale_x,
            height: height * scale_y,
            turn: xy.atan2(xx),
            corners: [
                at(0.0, 0.0),
                at(width, 0.0),
                at(width, height),
                at(0.0, height),
            ],
            radii: CLIP_RADII.map(|(x, y)| (x * scale_x, y * scale_y)),
        }
    }
    fn has_clip(draw: &FrameDraw, clip: &RoundedRect) -> bool {
        draw.clips
            .iter()
            .filter_map(RoundedRect::read)
            .any(|drawn| drawn.difference(clip).is_none())
    }
    /// Whether a clip in scope at `draw` covers the same box as `holder`'s draw.
    fn inside(draw: &FrameDraw, holder: &FrameDraw) -> bool {
        let box_of = |points: &[render::Vec2D]| {
            points.iter().fold(
                (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
                |(left, top, right, bottom), point| {
                    (
                        left.min(point.x),
                        top.min(point.y),
                        right.max(point.x),
                        bottom.max(point.y),
                    )
                },
            )
        };
        let (left, top, right, bottom) = box_of(&holder.points);
        draw.clips.iter().any(|clip| {
            let (l, t, r, b) = box_of(&clip.points);
            (l - left).abs() < 0.05
                && (t - top).abs() < 0.05
                && (r - right).abs() < 0.05
                && (b - bottom).abs() < 0.05
        })
    }

    /// R3a. A flight clip travels with its frame: it is in scope at every draw
    /// of the frame's content, as the frame's recorded content matrix maps it,
    /// and the anchor's ancestors' clips aren't.
    #[test]
    fn a_flight_clip_bounds_the_frames_ink() {
        let (e, inner, holder) = (
            fixture::paint_color("E"),
            fixture::paint_color("inner"),
            fixture::paint_color("H"),
        );
        // The control: E drawn at home is inside H's clip.
        let mut home = clipped_flight();
        land_flight(&mut home);
        let draws = home.draws();
        let holder_draw = one_of(&draws, holder, "H at rest");
        assert!(
            inside(one_of(&draws, e, "E at rest"), holder_draw),
            "at rest E is drawn in H's clip"
        );

        let s = clipped_flight();
        let draws = s.draws();
        let frame = one_of(&draws, e, "E in flight");
        close(turn(frame.matrix), 0.35);
        close(frame.axis_scales().0, 1.5);
        close(frame.axis_scales().1, 1.5);
        let content = [("E", frame), ("inner", one_of(&draws, inner, "inner"))];
        for (name, draw) in content {
            assert!(!inside(draw, holder_draw), "{name} is drawn in H's clip");
        }
        let clip = clip_on(frame);
        for (name, draw) in content {
            assert!(
                has_clip(draw, &clip),
                "{name}: {clip:?} isn't in scope, only {:?}",
                draw.clips
            );
        }
    }

    /// R3c. Landing clears the clip: it was in scope in flight, and the frame at
    /// home draws without it.
    #[test]
    fn landing_clears_a_flight_clip() {
        let (e, inner) = (fixture::paint_color("E"), fixture::paint_color("inner"));
        let mut s = clipped_flight();
        let draws = s.draws();
        let frame = one_of(&draws, e, "E in flight");
        assert!(
            has_clip(frame, &clip_on(frame)),
            "the clip isn't in scope in flight, only {:?}",
            frame.clips
        );

        land_flight(&mut s);
        let draws = s.draws();
        let frame = one_of(&draws, e, "E at home");
        let clip = clip_on(frame);
        for (name, draw) in [("E", frame), ("inner", one_of(&draws, inner, "inner"))] {
            assert!(!has_clip(draw, &clip), "{name} keeps the clip at home");
        }
    }

    /// R4 to R9: the zoom, read from the draw. Every expected box is made here
    /// from recorded draws by the rule: L from the leaving frame's draw before
    /// the step, A from the arriving frame's draw after the step lands (the step
    /// runs once, resets, and runs again), and on Back r and t from the keyed
    /// ancestor's recorded draw against its draw at rest.
    mod zoom {
        use super::*;
        use fixture::ZoomCase as Case;
        use render::{Mat2D, Renderer, Vec2D};

        /// The tracked point: the center of the 400 by 300 page.
        const PAGE_CENTER: (f32, f32) = (200.0, 150.0);
        const IDENTITY: Mat2D = Mat2D([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
        /// R4's keyed progress, which each frame after the first takes 0.1 s to
        /// reach.
        const PROGRESS: [f32; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];
        /// R6's keyed progress.
        const HAND_OVER: [f32; 5] = [0.2, 0.35, 0.5, 0.65, 0.8];
        const BOTH: [Way; 2] = [Way::Forward, Way::Back];

        #[derive(Clone, Copy, Debug, PartialEq)]
        enum Way {
            /// From A to B: "a" leaves and "b" arrives, with zoom 1.
            Forward,
            /// From B to A: "b" leaves and "a" arrives, with zoom 2.
            Back,
        }
        impl Way {
            fn to(self) -> &'static str {
                match self {
                    Way::Forward => "B",
                    Way::Back => "A",
                }
            }
            fn step(self) -> &'static str {
                match self {
                    Way::Forward => "go",
                    Way::Back => "back",
                }
            }
            fn leaving(self) -> &'static str {
                match self {
                    Way::Forward => "a",
                    Way::Back => "b",
                }
            }
            fn arriving(self) -> &'static str {
                match self {
                    Way::Forward => "b",
                    Way::Back => "a",
                }
            }
        }

        fn color(version: &str) -> u32 {
            fixture::paint_color(version)
        }
        fn child_color(version: &str) -> u32 {
            fixture::paint_color(&format!("{version} child"))
        }
        /// The frame's own radius, which only the writer of a case knows: the
        /// draw of a background doesn't carry it as a number. A frame with no
        /// height has none, since a radius is held to the frame's size. R5
        /// holds the box at each end to the outline the frame draws there, so
        /// a number here that the draw doesn't show fails it.
        fn radius(case: Case, version: &str) -> f32 {
            match (case, version) {
                (Case::ZeroHeight { .. } | Case::ZeroBox { .. }, "a") => 0.0,
                (Case::UnequalScale, _) => 12.0,
                (Case::Page { .. }, "a") => 14.0,
                (Case::Page { .. }, _) => 0.0,
                (_, "a") => 4.0,
                _ => 8.0,
            }
        }

        fn mul(a: Mat2D, b: Mat2D) -> Mat2D {
            let ([a0, a1, a2, a3, a4, a5], [b0, b1, b2, b3, b4, b5]) = (a.0, b.0);
            Mat2D([
                a0 * b0 + a2 * b1,
                a1 * b0 + a3 * b1,
                a0 * b2 + a2 * b3,
                a1 * b2 + a3 * b3,
                a0 * b4 + a2 * b5 + a4,
                a1 * b4 + a3 * b5 + a5,
            ])
        }
        fn invert(m: Mat2D) -> Option<Mat2D> {
            let [a, b, c, d, x, y] = m.0;
            let det = a * d - b * c;
            (det.abs() > 1e-9).then(|| {
                Mat2D([
                    d / det,
                    -b / det,
                    -c / det,
                    a / det,
                    (c * y - d * x) / det,
                    (b * x - a * y) / det,
                ])
            })
        }
        fn at(m: Mat2D, x: f32, y: f32) -> Vec2D {
            let [a, b, c, d, tx, ty] = m.0;
            Vec2D::new(a * x + c * y + tx, b * x + d * y + ty)
        }
        fn translate(x: f32, y: f32) -> Mat2D {
            Mat2D([1.0, 0.0, 0.0, 1.0, x, y])
        }
        fn bounds(points: impl IntoIterator<Item = Vec2D>) -> (f32, f32, f32, f32) {
            points.into_iter().fold(
                (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
                |(left, top, right, bottom), p| {
                    (left.min(p.x), top.min(p.y), right.max(p.x), bottom.max(p.y))
                },
            )
        }
        fn lerp(a: f32, b: f32, t: f32) -> f32 {
            a + (b - a) * t
        }
        fn near(read: f32, want: f32, label: &str) {
            assert!(
                (read - want).abs() < 0.003,
                "{label}: read {read}, expected {want}"
            );
        }
        /// Whether two matrices put a content box of this size in the same place.
        fn same_place(a: Mat2D, b: Mat2D, (width, height): (f32, f32), tolerance: f32) -> bool {
            [(0.0, 0.0), (width, 0.0), (0.0, height), (width, height)]
                .into_iter()
                .all(|(x, y)| {
                    let (p, q) = (at(a, x, y), at(b, x, y));
                    (p.x - q.x).abs() <= tolerance && (p.y - q.y).abs() <= tolerance
                })
        }

        /// One end of the box: a version's frame content box under its matrix.
        #[derive(Clone, Copy, Debug)]
        struct End {
            center: Vec2D,
            turn: f32,
            width: f32,
            height: f32,
            scales: (f32, f32),
            content: (f32, f32),
            radii: [(f32, f32); 4],
        }
        impl End {
            /// From a frame's own background: its matrix is the frame's content
            /// matrix, and its points give the content's size.
            fn read(draw: &FrameDraw, radius: f32) -> Self {
                let inverse = invert(draw.matrix).expect("a frame that is drawn can be inverted");
                let (left, top, right, bottom) =
                    bounds(draw.points.iter().map(|p| at(inverse, p.x, p.y)));
                assert!(
                    left.abs() < 1e-2 && top.abs() < 1e-2,
                    "the background starts at the content's origin: {left}, {top}"
                );
                Self::of(draw.matrix, (right, bottom), radius)
            }
            /// The rule: a skewed or mirrored end's rectangle is the axis-aligned
            /// box around its content box, unturned, with both axis scales that
            /// rectangle's width over the content's width.
            fn of(matrix: Mat2D, content: (f32, f32), radius: f32) -> Self {
                let [a, b, c, d, ..] = matrix.0;
                let (width, height) = content;
                let (sx, sy) = (a.hypot(b), c.hypot(d));
                let skewed = (a * c + b * d).abs() > 1e-4 * (sx * sy).max(1e-6);
                let mirrored = a * d - b * c < 0.0;
                if skewed || mirrored {
                    let (left, top, right, bottom) = bounds([
                        at(matrix, 0.0, 0.0),
                        at(matrix, width, 0.0),
                        at(matrix, 0.0, height),
                        at(matrix, width, height),
                    ]);
                    let scale = (right - left) / width;
                    Self {
                        center: Vec2D::new((left + right) / 2.0, (top + bottom) / 2.0),
                        turn: 0.0,
                        width: right - left,
                        height: bottom - top,
                        scales: (scale, scale),
                        content,
                        radii: [(radius * scale, radius * scale); 4],
                    }
                } else {
                    Self {
                        center: at(matrix, width / 2.0, height / 2.0),
                        turn: b.atan2(a),
                        width: sx * width,
                        height: sy * height,
                        scales: (sx, sy),
                        content,
                        radii: [(radius * sx, radius * sy); 4],
                    }
                }
            }
        }

        /// What the box follows on Back, read from the draw.
        #[derive(Clone, Copy, Debug)]
        struct Live {
            r: f32,
            t: Vec2D,
        }
        impl Default for Live {
            fn default() -> Self {
                Self {
                    r: 1.0,
                    t: Vec2D::new(0.0, 0.0),
                }
            }
        }
        impl Live {
            /// K = the keyed ancestor's draw now, over its draw at rest.
            fn between(now: Mat2D, rest: Mat2D, fit: Mat2D) -> Self {
                let k = mul(now, invert(rest).expect("the ancestor at rest inverts"));
                let [a, b, c, d, ..] = k.0;
                let tracked = at(fit, PAGE_CENTER.0, PAGE_CENTER.1);
                let moved = at(k, tracked.x, tracked.y);
                Self {
                    r: (a * d - b * c).abs().sqrt(),
                    t: Vec2D::new(moved.x - tracked.x, moved.y - tracked.y),
                }
            }
        }

        /// The box at clamped progress `p`.
        #[derive(Clone, Copy, Debug)]
        struct ZoomBox {
            center: Vec2D,
            turn: f32,
            width: f32,
            height: f32,
            radii: [(f32, f32); 4],
        }
        impl ZoomBox {
            fn rounded(&self) -> RoundedRect {
                let (sin, cos) = self.turn.sin_cos();
                let (x, y) = (self.width / 2.0, self.height / 2.0);
                let point = |x: f32, y: f32| {
                    Vec2D::new(
                        self.center.x + x * cos - y * sin,
                        self.center.y + x * sin + y * cos,
                    )
                };
                RoundedRect {
                    center: self.center,
                    width: self.width,
                    height: self.height,
                    turn: self.turn,
                    corners: [point(-x, -y), point(x, -y), point(x, y), point(-x, y)],
                    radii: self.radii,
                }
            }
        }
        /// L's share of A on the axis where the two sizes differ least, width on
        /// a tie.
        fn share(l: &End, a: &End) -> f32 {
            let (width, height) = (l.width / a.width, l.height / a.height);
            if (width - 1.0).abs() <= (height - 1.0).abs() {
                width
            } else {
                height
            }
        }
        fn box_at(way: Way, l: &End, a: &End, p: f32, live: Live) -> ZoomBox {
            let p = p.clamp(0.0, 1.0);
            let half_turn = std::f32::consts::PI;
            let apart = (a.turn - l.turn + half_turn).rem_euclid(std::f32::consts::TAU) - half_turn;
            let k = share(l, a);
            let leaves_the_source = way == Way::Forward && k != 0.0 && k.is_finite();
            let blend = |l: f32, a: f32| {
                if leaves_the_source {
                    lerp(l / k, a, p) * lerp(k, 1.0, p)
                } else {
                    lerp(l, a, p) * live.r
                }
            };
            let mut radii = [(0.0, 0.0); 4];
            for (i, radius) in radii.iter_mut().enumerate() {
                *radius = (
                    blend(l.radii[i].0, a.radii[i].0),
                    blend(l.radii[i].1, a.radii[i].1),
                );
            }
            ZoomBox {
                center: Vec2D::new(
                    lerp(l.center.x, a.center.x, p) + live.t.x,
                    lerp(l.center.y, a.center.y, p) + live.t.y,
                ),
                turn: l.turn + apart * p,
                width: blend(l.width, a.width),
                height: blend(l.height, a.height),
                radii,
            }
        }
        /// G: the version hung from the box's top center at the box's turn, at
        /// its own scale. It is its end's matrix when the box is its end.
        fn target(zoom_box: &ZoomBox, end: &End) -> Mat2D {
            let lambda = if end.width > 0.0 {
                zoom_box.width / end.width
            } else {
                zoom_box.height / end.height
            };
            let (sin, cos) = zoom_box.turn.sin_cos();
            let (sx, sy) = (lambda * end.scales.0, lambda * end.scales.1);
            let half = zoom_box.height / 2.0;
            let top = Vec2D::new(
                zoom_box.center.x + half * sin,
                zoom_box.center.y - half * cos,
            );
            let from_center = end.content.0 / 2.0 * sx;
            Mat2D([
                cos * sx,
                sin * sx,
                -sin * sy,
                cos * sy,
                top.x - from_center * cos,
                top.y - from_center * sin,
            ])
        }
        /// The box is the clip in scope at the version's draws, and the version's
        /// content hangs from the box's top edge at the box's width. The clip is
        /// read first: a flight with no pair has none.
        fn on_box(draws: &[FrameDraw], version: &str, end: &End, zoom_box: &ZoomBox, label: &str) {
            let frame = one_of(draws, color(version), label);
            let clip = zoom_box.rounded();
            for draw in [frame, one_of(draws, child_color(version), label)] {
                assert!(
                    has_clip(draw, &clip),
                    "{label}: {version}: the box {clip:?} isn't in scope, only {:?}",
                    draw.clips
                );
            }
            let want = target(zoom_box, end);
            let tolerance = zoom_box.width * 1e-3;
            assert!(
                same_place(frame.matrix, want, end.content, tolerance),
                "{label}: {version} draws at {:?}, the box hangs it at {:?}",
                frame.matrix.0,
                want.0
            );
        }
        /// Where a version's child sits in its frame: at rest and in flight
        /// alike, since neither version is laid out again.
        fn placed_child(draws: &[FrameDraw], version: &str, label: &str) -> Mat2D {
            let frame = one_of(draws, color(version), label).matrix;
            mul(
                invert(frame).expect("a frame that is drawn can be inverted"),
                one_of(draws, child_color(version), label).matrix,
            )
        }
        /// Whether two drawn paths have the same points in the same order. A
        /// frame's background and its flight clip are both CSS rounded
        /// rectangles (`add_rounded_rect`), so the same box with the same
        /// corners gives the same points.
        fn same_outline(a: &[Vec2D], b: &[Vec2D], tolerance: f32) -> bool {
            let same_point = |p: &Vec2D, q: &Vec2D| {
                (p.x - q.x).abs() <= tolerance && (p.y - q.y).abs() <= tolerance
            };
            a.len() == b.len() && a.iter().zip(b).all(|(p, q)| same_point(p, q))
        }
        /// A version at its own end: its frame draws the outline `own` draws,
        /// and a clip in scope at it is that outline, corners and all. Both
        /// sides are read from the draw, not from the case's radius.
        fn meets_own_outline(draw: &FrameDraw, own: &FrameDraw, tolerance: f32, label: &str) {
            assert!(
                same_outline(&draw.points, &own.points, tolerance),
                "{label}: the frame draws {:?}, at its own end {:?}",
                draw.points,
                own.points
            );
            assert!(
                draw.clips
                    .iter()
                    .any(|clip| same_outline(&clip.points, &own.points, tolerance)),
                "{label}: no clip in scope is the frame's own outline {:?}, only {:?}",
                own.points,
                draw.clips
            );
        }
        fn no_ink(draws: &[FrameDraw], rgb: u32, label: &str) {
            for draw in of_rgb(draws, rgb) {
                assert!(
                    draw.opacity() == 0.0,
                    "{label}: {rgb:08x} draws at {}",
                    draw.opacity()
                );
            }
        }
        fn leaving_rule(p: f32) -> f32 {
            ((1.0 - p) / 0.65).min(1.0)
        }
        fn arriving_rule(p: f32) -> f32 {
            (p / 0.65).min(1.0)
        }
        fn set_progress(s: &DrawnScene, p: f32) {
            for flight in s.moves.flights(&s.root) {
                set(&flight.controls(), 62003, p);
            }
        }
        fn draws_under(s: &DrawnScene, fit: Mat2D) -> Vec<FrameDraw> {
            s.factory.borrow_mut().clear();
            let mut renderer = s.factory.borrow().make_renderer();
            renderer.save();
            renderer.transform(fit);
            s.root.draw(&mut renderer);
            renderer.restore();
            let stream = s.factory.borrow().stream();
            frame_draws(&stream_ops(&stream).unwrap(), &[])
        }

        /// One way through one case.
        #[derive(Clone, Copy)]
        struct Run {
            case: Case,
            way: Way,
            /// How long the keys run before the step starts.
            before: f32,
            fit: Mat2D,
            /// The box whose keyed scale above the arriving element r follows on
            /// Back.
            keyed: Option<&'static str>,
            /// Whether that scale moves the tracked point.
            moves_the_center: Option<bool>,
            setup: Option<fn(&DrawnScene)>,
        }
        impl Run {
            fn new(case: Case, way: Way) -> Self {
                Self {
                    case,
                    way,
                    before: 0.0,
                    fit: IDENTITY,
                    keyed: None,
                    moves_the_center: None,
                    setup: None,
                }
            }
            fn label(&self) -> String {
                format!("{:?} {:?}", self.case, self.way)
            }
            fn scene(&self) -> DrawnScene {
                let mut s = DrawnScene::new(PaintCase::Zoom(self.case));
                if let Some(setup) = self.setup {
                    setup(&s);
                    s.advance(0.0);
                }
                if self.before > 0.0 {
                    s.advance(self.before);
                }
                s
            }
            fn begin(&self, s: &mut DrawnScene) {
                s.moves
                    .step_begin(&s.root, self.way.to(), Some(self.way.step()))
                    .unwrap();
            }
            fn end(&self, s: &mut DrawnScene) {
                s.moves
                    .step_end(&s.root, self.way.to(), Some(self.way.step()))
                    .unwrap();
            }
            /// The flight at progress `p`, `dt` later.
            fn at(&self, s: &mut DrawnScene, p: f32, dt: f32) -> Vec<FrameDraw> {
                set_progress(s, p);
                s.advance(dt);
                draws_under(s, self.fit)
            }
            /// What the step comes to: it runs once, and its keys end.
            fn landed(&self) -> Vec<FrameDraw> {
                let mut s = self.scene();
                self.begin(&mut s);
                set_progress(&s, 1.0);
                s.advance(0.1);
                self.end(&mut s);
                s.advance(3.0);
                draws_under(&s, self.fit)
            }
            fn ends(&self, before: &[FrameDraw], rest: &[FrameDraw]) -> (End, End) {
                let (leaving, arriving) = (self.way.leaving(), self.way.arriving());
                (
                    End::read(
                        one_of(before, color(leaving), "the leaving frame before the step"),
                        radius(self.case, leaving),
                    ),
                    End::read(
                        one_of(rest, color(arriving), "the arriving frame after landing"),
                        radius(self.case, arriving),
                    ),
                )
            }
            fn live(&self, draws: &[FrameDraw], rest: &[FrameDraw]) -> Live {
                match (self.way, self.keyed) {
                    (Way::Back, Some(name)) => Live::between(
                        one_of(draws, color(name), name).matrix,
                        one_of(rest, color(name), name).matrix,
                        self.fit,
                    ),
                    _ => Live::default(),
                }
            }
        }

        /// R4's check at each progress: where a version's rule is above 0 it is
        /// on the box and its child is where it rests in it, and where its rule
        /// is 0 it draws nothing.
        fn zoom_on_one_box(run: &Run) {
            let label = run.label();
            let (leaving, arriving) = (run.way.leaving(), run.way.arriving());
            let rest = run.landed();
            let mut s = run.scene();
            let before = draws_under(&s, run.fit);
            let (l, a) = run.ends(&before, &rest);
            let at_rest = [
                placed_child(&before, leaving, &label),
                placed_child(&rest, arriving, &label),
            ];
            for (place, end) in at_rest.iter().zip([&l, &a]) {
                let [.., x, y] = place.0;
                let (width, height) = end.content;
                assert!(
                    (x - (width - 40.0)).abs() < 1e-2 && (y - (height - 30.0)).abs() < 1e-2,
                    "{label}: the child rests 10 from its frame's far edges: {x}, {y}"
                );
            }
            run.begin(&mut s);
            for (i, p) in PROGRESS.into_iter().enumerate() {
                let draws = run.at(&mut s, p, if i == 0 { 0.0 } else { 0.1 });
                let label = format!("{label} at {p}");
                let live = run.live(&draws, &rest);
                if i == 0 && run.way == Way::Back && run.keyed.is_some() {
                    assert!(
                        live.r < 0.999,
                        "{label}: the keys are running: r = {}",
                        live.r
                    );
                }
                if let (0, Way::Back, Some(moves)) = (i, run.way, run.moves_the_center) {
                    let moved = live.t.x.hypot(live.t.y);
                    assert_eq!(moved > 1.0, moves, "{label}: t = {:?}", live.t);
                    if !moves {
                        assert!(moved < 1e-3, "{label}: t = {:?}", live.t);
                    }
                }
                let zoom_box = box_at(run.way, &l, &a, p, live);
                let versions = [
                    (leaving, &l, leaving_rule(p), at_rest[0]),
                    (arriving, &a, arriving_rule(p), at_rest[1]),
                ];
                for (version, end, rule, place) in versions {
                    if rule == 0.0 {
                        no_ink(&draws, color(version), &label);
                        no_ink(&draws, child_color(version), &label);
                        continue;
                    }
                    on_box(&draws, version, end, &zoom_box, &label);
                    let now = placed_child(&draws, version, &label);
                    assert!(
                        (0..6).all(|i| (now.0[i] - place.0[i]).abs() < 1e-3),
                        "{label}: {version} was laid out again: its child is at {:?}, at rest {:?}",
                        now.0,
                        place.0
                    );
                }
            }
        }

        fn keyed_above(way: Way) -> Run {
            Run {
                before: 0.5,
                keyed: Some("card a"),
                ..Run::new(Case::CardsKeyed, way)
            }
        }
        fn slot_turn(way: Way) -> Run {
            Run::new(Case::SlotTurn, way)
        }
        fn frame_keyed_scale(way: Way) -> Run {
            Run::new(Case::FrameKeyedScale, way)
        }
        fn frame_bound_scale(way: Way) -> Run {
            Run::new(Case::FrameBoundScale, way)
        }
        fn frame_turn(way: Way) -> Run {
            Run::new(Case::FrameTurn, way)
        }
        fn frame_offset(way: Way) -> Run {
            Run {
                before: 0.25,
                ..Run::new(Case::FrameOffset, way)
            }
        }
        /// B, which holds the flight layer, turned and scaled equally.
        fn turn_the_flights_screen(s: &DrawnScene) {
            let screen = s.row("B").core_handle();
            set(&screen, 15, 0.2);
            set(&screen, 16, 1.25);
            set(&screen, 17, 1.25);
        }
        fn turned_screen(way: Way) -> Run {
            Run {
                setup: Some(turn_the_flights_screen),
                ..Run::new(Case::Plain, way)
            }
        }
        fn host_fit(way: Way) -> Run {
            Run {
                fit: Mat2D([0.5, 0.0, 0.0, 0.5, 10.0, 20.0]),
                ..Run::new(Case::Plain, way)
            }
        }
        fn root_keyed(way: Way) -> Run {
            Run {
                keyed: Some("page a"),
                moves_the_center: Some(false),
                ..Run::new(Case::RootKeyed, way)
            }
        }
        fn taller_on_one_side(way: Way) -> Run {
            Run::new(
                Case::Sizes {
                    a: (60.0, 120.0),
                    b: (120.0, 160.0),
                },
                way,
            )
        }
        fn taller_on_both_sides(way: Way) -> Run {
            Run::new(
                Case::Sizes {
                    a: (150.0, 120.0),
                    b: (100.0, 200.0),
                },
                way,
            )
        }
        fn unequal_scale(way: Way) -> Run {
            Run::new(Case::UnequalScale, way)
        }
        fn off_center(way: Way) -> Run {
            Run {
                keyed: Some("card a"),
                moves_the_center: Some(true),
                ..Run::new(Case::OffCenterCard, way)
            }
        }
        fn inner_card(way: Way) -> Run {
            Run {
                keyed: Some("card a"),
                moves_the_center: Some(true),
                ..Run::new(Case::InnerCard, way)
            }
        }
        fn both_ways(kind: fn(Way) -> Run) {
            for way in BOTH {
                zoom_on_one_box(&kind(way));
            }
        }

        /// R4a. A keyed scale above the leaving element before takeoff.
        #[test]
        fn zoom_draws_both_versions_on_one_box_a_keyed_scale_above() {
            both_ways(keyed_above);
        }
        /// R4b. The slot's own turn.
        #[test]
        fn zoom_draws_both_versions_on_one_box_the_slots_turn() {
            both_ways(slot_turn);
        }
        /// R4c. The frame's own keyed scale, keyed to end at 1, on both versions.
        #[test]
        fn zoom_draws_both_versions_on_one_box_the_frames_keyed_scale() {
            both_ways(frame_keyed_scale);
        }
        /// R4d. Its bound scale.
        #[test]
        fn zoom_draws_both_versions_on_one_box_the_frames_bound_scale() {
            both_ways(frame_bound_scale);
        }
        /// R4e. Its own turn.
        #[test]
        fn zoom_draws_both_versions_on_one_box_the_frames_turn() {
            both_ways(frame_turn);
        }
        /// R4f. A CSS-slot offset with a pivot, its y keyed to end at 0 and still
        /// running at takeoff.
        #[test]
        fn zoom_draws_both_versions_on_one_box_a_css_slot_offset() {
            both_ways(frame_offset);
        }
        /// R4g. The Screen that holds the flight layer turned and scaled equally,
        /// unkeyed, so the solve inverts a flight-slot parent that isn't identity.
        #[test]
        fn zoom_draws_both_versions_on_one_box_a_turned_screen() {
            let run = turned_screen(Way::Forward);
            let draws = draws_under(&run.scene(), IDENTITY);
            let draw = one_of(&draws, color("b"), "b at rest");
            close(turn(draw.matrix), 0.2);
            close(draw.axis_scales().0, 1.25);
            both_ways(turned_screen);
        }
        /// R4h. The host's fit, drawn under a fit transform.
        #[test]
        fn zoom_draws_both_versions_on_one_box_the_hosts_fit() {
            both_ways(host_fit);
        }
        /// R4i. On Back, a keyed scale above the arriving element about the
        /// screen's center, so r follows it and t is 0.
        #[test]
        fn zoom_draws_both_versions_on_one_box_a_scale_about_the_screens_center() {
            zoom_on_one_box(&root_keyed(Way::Back));
        }
        /// R4j. A leaving element proportionally taller than the arriving one, so
        /// k is taken on the height: both shares on one side of 1, then one above
        /// and one below.
        #[test]
        fn zoom_draws_both_versions_on_one_box_shares_on_the_height() {
            let end = |w, h| End::of(IDENTITY, (w, h), 0.0);
            assert_eq!(share(&end(60.0, 120.0), &end(120.0, 160.0)), 0.75);
            assert_eq!(share(&end(150.0, 120.0), &end(100.0, 200.0)), 0.6);
            both_ways(taller_on_one_side);
            both_ways(taller_on_both_sides);
        }
        /// R4k. An end with unequal axis scales, so each corner's two radii differ.
        #[test]
        fn zoom_draws_both_versions_on_one_box_unequal_axis_scales() {
            let run = unequal_scale(Way::Forward);
            let draws = draws_under(&run.scene(), IDENTITY);
            let end = End::read(one_of(&draws, color("a"), "a at rest"), 12.0);
            assert!(
                (end.radii[0].0 - end.radii[0].1).abs() > 1.0,
                "{:?}",
                end.radii
            );
            both_ways(unequal_scale);
        }
        /// R4l. On Back, a keyed scale above the arriving element about a pivot
        /// away from the screen's center, so the tracker moves the box.
        #[test]
        fn zoom_draws_both_versions_on_one_box_a_scale_off_center() {
            zoom_on_one_box(&off_center(Way::Back));
        }
        /// R4n. On Back, a keyed scale on a box that isn't the root, about its
        /// own center: r follows it and t is where it puts the screen's center.
        #[test]
        fn zoom_draws_both_versions_on_one_box_a_card_in_the_screen() {
            zoom_on_one_box(&inner_card(Way::Back));
        }

        /// R5. When the flight lands the arriving version draws as it does after
        /// landing, with no change across the landing: its matrix, its outline,
        /// and a clip in scope that is the outline it draws after landing.
        /// Forward, at 0 the leaving version draws as it did before the step,
        /// its clip the outline it drew then. On Back it draws at r times its
        /// takeoff about the box's center, with r read from the draw.
        #[test]
        fn zoom_meets_each_version_at_its_own_end() {
            let kinds: [fn(Way) -> Run; 15] = [
                keyed_above,
                slot_turn,
                frame_keyed_scale,
                frame_bound_scale,
                frame_turn,
                frame_offset,
                turned_screen,
                host_fit,
                root_keyed,
                taller_on_one_side,
                taller_on_both_sides,
                unequal_scale,
                off_center,
                inner_card,
                |way| Run::new(Case::Plain, way),
            ];
            for kind in kinds {
                for way in BOTH {
                    let run = kind(way);
                    let label = run.label();
                    let (leaving, arriving) = (way.leaving(), way.arriving());
                    let rest = run.landed();
                    let mut s = run.scene();
                    let before = draws_under(&s, run.fit);
                    let (l, a) = run.ends(&before, &rest);
                    run.begin(&mut s);

                    let draws = run.at(&mut s, 0.0, 0.0);
                    let live = run.live(&draws, &rest);
                    let zoom_box = box_at(way, &l, &a, 0.0, live);
                    on_box(&draws, leaving, &l, &zoom_box, &format!("{label} at 0"));
                    no_ink(&draws, color(arriving), &format!("{label} at 0"));
                    // On Back the box at 0 is the takeoff times r about its
                    // center, moved by t, which `on_box` has checked.
                    if way == Way::Forward {
                        assert!(
                            same_place(
                                one_of(&draws, color(leaving), &label).matrix,
                                one_of(&before, color(leaving), &label).matrix,
                                l.content,
                                l.width * 1e-3
                            ),
                            "{label}: at 0 the leaving version draws as before the step"
                        );
                        meets_own_outline(
                            one_of(&draws, color(leaving), &label),
                            one_of(&before, color(leaving), &label),
                            l.width * 1e-3,
                            &format!("{label} at 0"),
                        );
                    }

                    let draws = run.at(&mut s, 1.0, 0.1);
                    let flown = one_of(&draws, color(arriving), &label).matrix;
                    let tolerance = a.width * 1e-3;
                    if way == Way::Forward {
                        assert!(
                            same_place(
                                flown,
                                one_of(&rest, color(arriving), &label).matrix,
                                a.content,
                                tolerance
                            ),
                            "{label}: at 1 the arriving version draws as after landing"
                        );
                    }
                    let draws = run.at(&mut s, 1.0, 3.0);
                    let live = run.live(&draws, &rest);
                    assert!((live.r - 1.0).abs() < 1e-3, "{label}: r = {}", live.r);
                    let zoom_box = box_at(way, &l, &a, 1.0, live);
                    on_box(&draws, arriving, &a, &zoom_box, &format!("{label} landing"));
                    no_ink(&draws, color(leaving), &format!("{label} landing"));
                    let landing = one_of(&draws, color(arriving), &label).clone();

                    run.end(&mut s);
                    s.advance(0.0);
                    let draws = draws_under(&s, run.fit);
                    let home = one_of(&draws, color(arriving), &label);
                    assert!(
                        same_place(landing.matrix, home.matrix, a.content, tolerance),
                        "{label}: the landing moves it from {:?} to {:?}",
                        landing.matrix.0,
                        home.matrix.0
                    );
                    meets_own_outline(
                        &landing,
                        home,
                        tolerance,
                        &format!("{label} across the landing"),
                    );
                }
            }
        }

        /// R5b. On Back, under an ancestor keyed from 0.8 to 0.5 and held: when
        /// the flight lands the arriving version draws on the box, A taken at
        /// identity times r = 0.5 about the box's center and moved by t, and
        /// after landing where the held key puts it. The stated limit.
        #[test]
        fn zoom_lands_with_a_jump_under_a_key_that_ends_off_identity() {
            let run = Run::new(Case::CardHeld, Way::Back);
            let mut s = run.scene();
            let before = s.draws();
            let l = End::read(one_of(&before, color("b"), "b at rest"), 8.0);
            let a = End::of(translate(36.0, 20.0), (80.0, 60.0), 4.0);
            run.begin(&mut s);
            run.at(&mut s, 1.0, 0.1);
            let draws = run.at(&mut s, 1.0, 3.0);
            let card = one_of(&draws, color("card a"), "the card held").matrix;
            let live = Live::between(card, IDENTITY, IDENTITY);
            close(live.r, 0.5);
            on_box(
                &draws,
                "a",
                &a,
                &box_at(Way::Back, &l, &a, 1.0, live),
                "when the flight lands",
            );
            let flown = one_of(&draws, color("a"), "a in flight").matrix;
            run.end(&mut s);
            s.advance(0.0);
            let draws = s.draws();
            let home = one_of(&draws, color("a"), "a at home").matrix;
            let card = one_of(&draws, color("card a"), "the card held").matrix;
            let held = mul(card, translate(36.0, 20.0));
            assert!(
                same_place(home, held, a.content, 1e-2),
                "after landing it draws where the held key puts it: {:?}, not {:?}",
                home.0,
                held.0
            );
            assert!(
                !same_place(home, flown, a.content, 1.0),
                "the jump is there: {:?}",
                home.0
            );
        }

        /// R5c. The arriving frame's own scale keyed from 0.6 to 1 over twice the
        /// step's length: when the flight lands the box is A, the frame at scale
        /// 1, and after landing the frame draws at its running key's value.
        #[test]
        fn zoom_lands_on_the_resting_place_while_the_element_animates() {
            for way in BOTH {
                let run = Run::new(Case::FrameRunningScale, way);
                let label = run.label();
                let (leaving, arriving) = (way.leaving(), way.arriving());
                let rest = run.landed();
                let mut s = run.scene();
                let before = s.draws();
                let (l, a) = run.ends(&before, &rest);
                run.begin(&mut s);
                for (i, p) in PROGRESS.into_iter().enumerate() {
                    let draws = run.at(&mut s, p, if i == 0 { 0.0 } else { 0.125 });
                    let zoom_box = box_at(way, &l, &a, p, Live::default());
                    let label = format!("{label} at {p}");
                    if p < 1.0 {
                        on_box(&draws, leaving, &l, &zoom_box, &label);
                    }
                    if p > 0.0 {
                        on_box(&draws, arriving, &a, &zoom_box, &label);
                    }
                }
                let draws = s.draws();
                let flown = one_of(&draws, color(arriving), &label).matrix;
                let at_rest = one_of(&rest, color(arriving), &label).matrix;
                assert!(
                    same_place(flown, at_rest, a.content, a.width * 1e-3),
                    "{label}: the box is A, not its animated size"
                );
                run.end(&mut s);
                s.advance(0.0);
                let draws = s.draws();
                let home = one_of(&draws, color(arriving), &label);
                near(home.axis_scales().0, 0.8, &label);
                near(home.axis_scales().1, 0.8, &label);
                assert!(
                    !same_place(home.matrix, flown, a.content, 1.0),
                    "{label}: the frame draws at its running key's value"
                );
            }
        }

        /// R5d. A, read at takeoff, equals the arriving frame's recorded draw
        /// after landing, when a bound text changes the step's layout.
        #[test]
        fn zoom_reads_the_arriving_screen_laid_out_at_takeoff() {
            let mut s = DrawnScene::new(PaintCase::Zoom(Case::BoundSubtitle));
            let home = s.slot("B", "b");
            let one_line = slot_pose(&home).unwrap();
            assert!(CoreRegistry::set_string_handle(
                &property(&vm(&s.row("B")), "subtitle"),
                561,
                "A subtitle long enough to wrap onto a second line once it is bound. ".repeat(3),
            ));
            s.moves.step_begin(&s.root, "B", Some("go")).unwrap();
            let at_takeoff = slot_pose(&home).unwrap();
            assert!(
                at_takeoff.transform[5] > one_line.transform[5] + 20.0,
                "the bound text must move the slot: {} from {}",
                at_takeoff.transform[5],
                one_line.transform[5]
            );
            let ends = s.moves.pair_ends(&s.root);
            assert_eq!(ends.len(), 1, "a zoomed step holds one pair");
            let arriving = ends[0].arriving;
            set_progress(&s, 1.0);
            s.advance(0.1);
            s.moves.step_end(&s.root, "B", Some("go")).unwrap();
            s.advance(3.0);
            let draws = s.draws();
            assert_reads(arriving, one_of(&draws, color("b"), "b landed"), "A");
        }

        fn rule_where_it_draws(draws: &[FrameDraw], way: Way, p: f32, label: &str) {
            for (version, rule) in [
                (way.leaving(), leaving_rule(p)),
                (way.arriving(), arriving_rule(p)),
            ] {
                for draw in of_rgb(draws, color(version)) {
                    near(draw.opacity(), rule, &format!("{label}: {version} at {p}"));
                }
            }
        }
        /// Each version's one background draw is at the rule, and the arriving
        /// one comes later.
        fn hands_over(draws: &[FrameDraw], way: Way, p: f32, label: &str) {
            let label = format!("{label} at {p}");
            let (leaving, arriving) = (way.leaving(), way.arriving());
            near(
                one_of(draws, color(leaving), &label).opacity(),
                leaving_rule(p),
                &format!("{label}: {leaving}"),
            );
            near(
                one_of(draws, color(arriving), &label).opacity(),
                arriving_rule(p),
                &format!("{label}: {arriving}"),
            );
            let place = |version: &str| {
                let rgb = color(version) & 0x00ff_ffff;
                draws.iter().position(|draw| draw.rgb() == Some(rgb))
            };
            assert!(
                place(arriving) > place(leaving),
                "{label}: the arriving version draws later"
            );
        }

        /// R6. Each version's recorded opacity is the rule, with the record's
        /// fade held at 0.5 to show it isn't read, the arriving version later
        /// both ways, and progress keyed to 1.0076 draws the box at 1.
        #[test]
        fn zoom_hands_over_by_the_rule() {
            for way in BOTH {
                let run = Run::new(Case::FadeHeld, way);
                let label = run.label();
                let rest = run.landed();
                let mut s = run.scene();
                let before = s.draws();
                let (l, a) = run.ends(&before, &rest);
                run.begin(&mut s);
                for p in HAND_OVER {
                    hands_over(&run.at(&mut s, p, 0.1), way, p, &label);
                }
                let draws = run.at(&mut s, 1.0076, 0.1);
                let zoom_box = box_at(way, &l, &a, 1.0, Live::default());
                on_box(&draws, way.arriving(), &a, &zoom_box, &label);
                no_ink(&draws, color(way.leaving()), &label);
            }
        }
        /// R6, under a faded ancestor: an ancestor of the leaving element keyed
        /// from 1 to 0.5 and one of the arriving element at a static 0.5. The
        /// arriving version changes to its real opacity after landing.
        #[test]
        fn zoom_hands_over_by_the_rule_under_a_faded_ancestor() {
            for (way, a_keyed) in [(Way::Forward, true), (Way::Back, false)] {
                let run = Run::new(Case::AncestorFades { a_keyed }, way);
                let label = run.label();
                let mut s = run.scene();
                run.begin(&mut s);
                for p in HAND_OVER {
                    hands_over(&run.at(&mut s, p, 0.1), way, p, &label);
                }
                run.at(&mut s, 1.0, 0.0);
                run.end(&mut s);
                s.advance(0.0);
                let draws = s.draws();
                let home = one_of(&draws, color(way.arriving()), &label);
                near(home.opacity(), 0.5, &format!("{label} after landing"));
            }
        }
        /// R6, on the frame itself: each version is at the rule, not the rule
        /// times its own. After landing the frame draws at its own opacity, so
        /// the property stayed live underneath.
        #[test]
        fn zoom_hands_over_by_the_rule_on_a_frame_with_its_own_opacity() {
            let cases: [(Case, Option<f32>); 3] = [
                (Case::OwnOpacity(0.5), Some(0.5)),
                (Case::OwnOpacity(0.0), None),
                // Half a second into a key from 1 to 0.3.
                (Case::OwnOpacityKeyed, Some(0.65)),
            ];
            for (case, after) in cases {
                for way in BOTH {
                    let run = Run::new(case, way);
                    let label = run.label();
                    let mut s = run.scene();
                    run.begin(&mut s);
                    for p in HAND_OVER {
                        hands_over(&run.at(&mut s, p, 0.1), way, p, &label);
                    }
                    run.at(&mut s, 1.0, 0.0);
                    run.end(&mut s);
                    s.advance(0.0);
                    let draws = s.draws();
                    let label = format!("{label} after landing");
                    match after {
                        Some(own) => near(
                            one_of(&draws, color(way.arriving()), &label).opacity(),
                            own,
                            &label,
                        ),
                        None => no_ink(&draws, color(way.arriving()), &label),
                    }
                }
            }
        }

        /// How many of each entry the plan holds.
        fn entries(s: &mut DrawnScene) -> (usize, usize) {
            let (mut ones, mut pairs) = (0, 0);
            s.moves.with_flights(&s.root, |flights| {
                for entry in flights.iter() {
                    match entry {
                        Entry::One(_) => ones += 1,
                        Entry::Pair(_) => pairs += 1,
                    }
                }
            });
            (ones, pairs)
        }
        fn no_clip(s: &mut DrawnScene, version: &str) {
            let draws = Run::new(Case::Plain, Way::Forward).at(s, 0.5, 0.0);
            let draw = one_of(&draws, color(version), version);
            assert!(draw.clips.is_empty(), "{version}: {:?}", draw.clips);
        }

        /// R7. (a) One unit at both ends with zoom 1 flies one ordinary flight
        /// with no clip. (c) Two versions with zoom 3 aren't dropped: both fly as
        /// today. (b) During a zoom step `flights()` gives two plain flights, and
        /// dropping the entry through `with_flights` removes both.
        #[test]
        fn the_zoom_lives_only_in_a_pair() {
            let mut s = DrawnScene::new(PaintCase::Zoom(Case::OneUnit));
            s.moves.step_begin(&s.root, "B", Some("go")).unwrap();
            assert_eq!(entries(&mut s), (1, 0), "one unit flies one flight");
            assert_eq!(s.moves.flights(&s.root).len(), 1);
            no_clip(&mut s, "a");

            let mut s = DrawnScene::new(PaintCase::Zoom(Case::ZoomOutOfRange));
            s.moves.step_begin(&s.root, "B", Some("go")).unwrap();
            assert_eq!(entries(&mut s), (2, 0), "zoom 3 reads 0");
            no_clip(&mut s, "a");
            no_clip(&mut s, "b");

            let mut s = DrawnScene::new(PaintCase::Zoom(Case::Plain));
            s.moves.step_begin(&s.root, "B", Some("go")).unwrap();
            assert_eq!(entries(&mut s), (0, 1), "a zoomed step holds one pair");
            assert_eq!(s.moves.flights(&s.root).len(), 2, "two plain flights");
            s.moves.with_flights(&s.root, |flights| flights.clear());
            assert!(s.moves.flights(&s.root).is_empty(), "the pair goes whole");
        }

        /// R8's first check on every end: each version is at the rule where it
        /// draws, and neither the takeoff nor any frame returns an error.
        fn rule_holds(run: &Run) {
            let mut s = run.scene();
            run.begin(&mut s);
            for p in [0.2, 0.5, 0.8] {
                let draws = run.at(&mut s, p, 0.1);
                rule_where_it_draws(&draws, run.way, p, &run.label());
            }
        }
        /// An end whose matrix isn't a plain rectangle's, in its real draw: it
        /// draws unskewed, unmirrored and unturned on the box, and changes to
        /// its real draw at its own end.
        fn unskews(case: Case) {
            for way in BOTH {
                let run = Run::new(case, way);
                rule_holds(&run);
                zoom_on_one_box(&run);
                let real = match way {
                    Way::Forward => draws_under(&run.scene(), IDENTITY),
                    Way::Back => run.landed(),
                };
                let real = one_of(&real, color("a"), "a's real draw");
                let [xx, xy, yx, yy, ..] = real.matrix.0;
                assert!(
                    (xx * yx + xy * yy).abs() > 1e-2 || xx * yy - xy * yx < 0.0,
                    "{:?}: a really draws skewed or mirrored: {:?}",
                    case,
                    real.matrix.0
                );
            }
        }
        /// R8, skewed: a turned frame under an ancestor scaled unequally.
        #[test]
        fn zoom_runs_on_ends_that_arent_plain_rectangles_skewed() {
            unskews(Case::Skewed);
        }
        /// R8, a mirrored end.
        #[test]
        fn zoom_runs_on_ends_that_arent_plain_rectangles_mirrored() {
            unskews(Case::Mirrored);
        }
        /// An end "a" at (36, 20) with no area, which has no background to read
        /// from: "b" is on the box the rule makes with it.
        fn empty_end(case: Case, content: (f32, f32), draws_nothing: bool) {
            let constant = End::of(translate(36.0, 20.0), content, 0.0);
            for way in BOTH {
                let run = Run::new(case, way);
                let label = run.label();
                rule_holds(&run);
                let rest = run.landed();
                let mut s = run.scene();
                let before = s.draws();
                let (l, a, b) = match way {
                    Way::Forward => {
                        let b = End::read(one_of(&rest, color("b"), &label), 8.0);
                        (constant, b, b)
                    }
                    Way::Back => {
                        let b = End::read(one_of(&before, color("b"), &label), 8.0);
                        (b, constant, b)
                    }
                };
                run.begin(&mut s);
                for p in [0.25, 0.5, 0.75] {
                    let draws = run.at(&mut s, p, 0.1);
                    let zoom_box = box_at(way, &l, &a, p, Live::default());
                    let label = format!("{label} at {p}");
                    on_box(&draws, "b", &b, &zoom_box, &label);
                    if draws_nothing {
                        no_ink(&draws, color("a"), &label);
                        no_ink(&draws, child_color("a"), &label);
                    }
                }
            }
        }
        /// R8, zero height in size mode 1: k comes from the width.
        #[test]
        fn zoom_runs_on_ends_that_arent_plain_rectangles_zero_height_laid_out() {
            let end = |w, h| End::of(IDENTITY, (w, h), 0.0);
            assert_eq!(share(&end(80.0, 0.0), &end(160.0, 120.0)), 0.5);
            empty_end(Case::ZeroHeight { mode: 1 }, (80.0, 0.0), false);
        }
        /// R8, zero height in size mode 0, as the compiler writes text and icons.
        #[test]
        fn zoom_runs_on_ends_that_arent_plain_rectangles_zero_height_scaled() {
            empty_end(Case::ZeroHeight { mode: 0 }, (80.0, 0.0), false);
        }
        /// R8, 0 by 0 in size mode 1: k is 0, or not finite, so the step takes the
        /// returning blend, and that version draws nothing.
        #[test]
        fn zoom_runs_on_ends_that_arent_plain_rectangles_empty_laid_out() {
            let end = |w, h| End::of(IDENTITY, (w, h), 0.0);
            assert_eq!(share(&end(0.0, 0.0), &end(160.0, 120.0)), 0.0);
            empty_end(Case::ZeroBox { mode: 1 }, (0.0, 0.0), true);
        }
        /// R8, 0 by 0 in size mode 0.
        #[test]
        fn zoom_runs_on_ends_that_arent_plain_rectangles_empty_scaled() {
            empty_end(Case::ZeroBox { mode: 0 }, (0.0, 0.0), true);
        }
        /// R8, a scale keyed to 0 and back above the arriving frame: under the
        /// zero scale nothing of that version draws on Back, and it draws on the
        /// box again once the scale returns.
        #[test]
        fn zoom_runs_on_ends_that_arent_plain_rectangles_scale_through_zero() {
            for way in BOTH {
                let run = Run {
                    keyed: Some("card a"),
                    ..Run::new(Case::CardsThroughZero, way)
                };
                let label = run.label();
                rule_holds(&run);
                let rest = run.landed();
                let mut s = run.scene();
                let before = s.draws();
                let (l, a) = run.ends(&before, &rest);
                run.begin(&mut s);
                for p in [0.25, 0.5, 0.75] {
                    // Each p is also how far the one-second keys have run.
                    let draws = run.at(&mut s, p, 0.25);
                    let label = format!("{label} at {p}");
                    if way == Way::Back && p == 0.5 {
                        for version in ["a", "b"] {
                            no_ink(&draws, color(version), &label);
                            no_ink(&draws, child_color(version), &label);
                        }
                        continue;
                    }
                    let zoom_box = box_at(way, &l, &a, p, run.live(&draws, &rest));
                    on_box(&draws, way.leaving(), &l, &zoom_box, &label);
                    on_box(&draws, way.arriving(), &a, &zoom_box, &label);
                }
            }
        }
        /// R8, a flight parent scaled to 0 at takeoff: nothing draws until it can
        /// be inverted, and then both versions are on the box.
        #[test]
        fn zoom_runs_on_ends_that_arent_plain_rectangles_parent_from_zero() {
            for way in BOTH {
                let run = Run::new(Case::LayerFromZero, way);
                let label = run.label();
                rule_holds(&run);
                let rest = run.landed();
                let mut s = run.scene();
                let before = s.draws();
                let (l, a) = run.ends(&before, &rest);
                run.begin(&mut s);
                let draws = run.at(&mut s, 0.2, 0.0);
                for version in ["a", "b"] {
                    no_ink(&draws, color(version), &label);
                    no_ink(&draws, child_color(version), &label);
                }
                for p in [0.4, 0.6] {
                    let draws = run.at(&mut s, p, 0.25);
                    let zoom_box = box_at(way, &l, &a, p, Live::default());
                    let label = format!("{label} at {p}");
                    on_box(&draws, way.leaving(), &l, &zoom_box, &label);
                    on_box(&draws, way.arriving(), &a, &zoom_box, &label);
                }
            }
        }
        /// R8, a scale above the arriving frame keyed from 0 at takeoff: the
        /// arriving end is empty, at the card's center, since Rest can't correct
        /// that level. On Back H₀ can't be inverted, so r is 1 and t is 0. After
        /// landing the frame draws where its keys put it.
        #[test]
        fn zoom_runs_on_ends_that_arent_plain_rectangles_scale_from_zero() {
            let cases = [
                (
                    Way::Forward,
                    false,
                    "card b",
                    (60.0, 20.0),
                    (220.0, 140.0),
                    (160.0, 120.0),
                ),
                (
                    Way::Back,
                    true,
                    "card a",
                    (16.0, 10.0),
                    (170.0, 110.0),
                    (80.0, 60.0),
                ),
            ];
            for (way, on_a, card, place, (x, y), content) in cases {
                let run = Run::new(Case::CardFromZero { on_a }, way);
                let label = run.label();
                let (leaving, arriving) = (way.leaving(), way.arriving());
                rule_holds(&run);
                let mut s = run.scene();
                let before = s.draws();
                let l = End::read(
                    one_of(&before, color(leaving), &label),
                    radius(run.case, leaving),
                );
                let a = End::of(Mat2D([0.0, 0.0, 0.0, 0.0, x, y]), content, 0.0);
                run.begin(&mut s);
                for p in [0.25, 0.5, 0.75] {
                    let draws = run.at(&mut s, p, 0.125);
                    let zoom_box = box_at(way, &l, &a, p, Live::default());
                    let label = format!("{label} at {p}");
                    on_box(&draws, leaving, &l, &zoom_box, &label);
                    no_ink(&draws, color(arriving), &label);
                    no_ink(&draws, child_color(arriving), &label);
                }
                run.at(&mut s, 1.0, 0.0);
                run.end(&mut s);
                s.advance(3.0);
                let draws = s.draws();
                let home = one_of(&draws, color(arriving), &label).matrix;
                let card = one_of(&draws, color(card), card).matrix;
                let keyed = mul(card, translate(place.0, place.1));
                assert!(
                    same_place(home, keyed, content, 1e-2),
                    "{label}: after landing it draws where its keys put it: {:?}, not {:?}",
                    home.0,
                    keyed.0
                );
            }
        }

        /// R9. The upper Screen's own opacity held at 0.5, in this harness's scope
        /// fallback: both versions' recorded opacity is the rule, not times 0.5,
        /// and they lie on the box. The zoom leaves the Screen's bar as it draws
        /// with no step running: at 0.25 its paint's alpha and its scopes are
        /// those it has at rest, and it draws before both versions, as the
        /// flight placement puts a flight whose anchor nothing after it covers.
        #[test]
        fn zoom_versions_keep_their_strength_under_a_fading_screen() {
            let alpha = |draw: &FrameDraw| (draw.color.unwrap() >> 24) as f32 / 255.0;
            for way in BOTH {
                let run = Run::new(Case::FadingScreen, way);
                let label = run.label();
                let rest = run.landed();
                let mut s = run.scene();
                let before = s.draws();
                let (l, a) = run.ends(&before, &rest);
                let bar_at_rest = one_of(&before, color("bar"), "the bar at rest").clone();
                run.begin(&mut s);
                for p in [0.25, 0.5, 0.75] {
                    let draws = run.at(&mut s, p, 0.1);
                    hands_over(&draws, way, p, &label);
                    let zoom_box = box_at(way, &l, &a, p, Live::default());
                    let label = format!("{label} at {p}");
                    on_box(&draws, way.leaving(), &l, &zoom_box, &label);
                    on_box(&draws, way.arriving(), &a, &zoom_box, &label);
                    if p != 0.25 {
                        continue;
                    }
                    let bar = one_of(&draws, color("bar"), &label);
                    assert!(
                        (alpha(bar) - alpha(&bar_at_rest)).abs() <= 1.0 / 255.0,
                        "{label}: the bar's paint alpha is {}, at rest {}",
                        alpha(bar),
                        alpha(&bar_at_rest)
                    );
                    assert_eq!(
                        bar.modulations, bar_at_rest.modulations,
                        "{label}: the bar's scopes are those it has at rest"
                    );
                    let place = |rgb: u32| {
                        draws
                            .iter()
                            .position(|draw| draw.rgb() == Some(rgb & 0x00ff_ffff))
                            .unwrap_or_else(|| panic!("{label}: no draw of {rgb:08x}"))
                    };
                    for version in [way.leaving(), way.arriving()] {
                        assert!(
                            place(color("bar")) < place(color(version)),
                            "{label}: the bar draws before {version}"
                        );
                    }
                }
            }
        }
    }
}
