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
        .with_flights(&s.root, |flights| flights[0].size_mode = SizeMode::Scale);
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
    moves.with_flights(&root, |flights| flights.push(follower));
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
        (0.5, 115.0, 0x80808080),
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
        assert_eq!(color_value(&model, "accent"), 0x80808080, "settle {settle}");
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
    assert_eq!(color_value(&model, "accent"), 0x406080a0);
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
        assert_eq!(color_value(&model, "accent"), 0x80808080);
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
    moves.with_flights(&root, |flights| flights.push(follower));
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
    moves.with_flights(&root, |flights| flights.push(follower));
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
        moves.with_flights(&root, |flights| flights.push(follower));
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
        moves.with_flights(&root, |flights| flights.push(follower));
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

/// The ink of `color` drawn by `board`, as final points, without a closing
/// repeat of the first vertex.
fn ink_points(
    factory: &PersistentFactory<RecordingFactory>,
    board: &RuntimeArtboardInstanceHandle,
    color: &str,
) -> Vec<source::math::vec2d::Vec2D> {
    use source::math::{mat2d::Mat2D, vec2d::Vec2D};
    factory.borrow_mut().clear();
    let mut renderer = factory.borrow().make_renderer();
    board.draw(&mut renderer);
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
        } else if line.starts_with("drawPath ") && line.contains(color) {
            assert!(points.is_empty(), "one draw of {color}");
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
    if points.len() > 1 && points.first() == points.last() {
        points.pop();
    }
    points
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
        moves.with_flights(&root, |flights| flights.push(follower));
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
