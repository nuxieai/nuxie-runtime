//! Every case and SECTION in upstream 2dbbfe18 pointer_button_test.cpp.
#![cfg(feature = "testing")]
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        listener_types::{
            listener_input_type::ListenerInputType,
            listener_input_type_pointer_button::ListenerInputTypePointerButton,
        },
        nested_state_machine::NestedStateMachine,
        state_machine::StateMachine,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    core::{CoreArena, CoreObject, CoreType},
    generated::core_registry::CoreRegistry,
    hit_result::HitResult,
    listener_type::ListenerType as L,
    math::vec2d::Vec2D,
    nested_artboard::NestedArtboard,
    pointer_button::PointerButton as B,
    scene::SceneBehavior,
    text::text_input::TextInput,
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_number::ViewModelInstanceNumber,
    },
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};
fn v(x: f32, y: f32) -> Vec2D {
    Vec2D::new(x, y)
}
fn file(asset: &str) -> RuntimeFileHandle {
    let root =
        std::path::PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream"));
    let bytes = std::fs::read(root.join("tests/unit_tests/assets").join(asset)).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap()
}
fn source(file: &RuntimeFileHandle, name: Option<&str>) -> CoreHandle {
    file.with_file(|f| match name {
        Some(name) => f.artboard_named_source(name),
        None => f.artboard_at_source(0),
    })
    .unwrap()
}
fn first_listener(source: &CoreHandle, name: Option<&str>) -> CoreHandle {
    let machine = source
        .with_downcast::<Artboard, _>(|a| match name {
            Some(n) => a.state_machine_named(n),
            None => a.state_machine_handle_at(0),
        })
        .flatten()
        .unwrap();
    machine
        .with_downcast::<StateMachine, _>(|m| {
            assert!(m.listener_count() > 0);
            m.listener(0).unwrap()
        })
        .unwrap()
}
fn down(m: &RuntimeStateMachineInstanceHandle, b: B, p: Vec2D) {
    m.with_instance_mut(|m| m.pointer_down(p, 0, b));
}
fn up(m: &RuntimeStateMachineInstanceHandle, b: B, p: Vec2D) {
    m.with_instance_mut(|m| m.pointer_up(p, 0, b));
}
fn moved(m: &RuntimeStateMachineInstanceHandle, p: Vec2D) {
    m.with_instance_mut(|m| m.pointer_move(p, 0.0, 0));
}
fn events(m: &RuntimeStateMachineInstanceHandle) -> usize {
    m.with_instance(|m| m.reported_event_count())
}
struct ClickScene {
    _file: RuntimeFileHandle,
    _artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
}
impl ClickScene {
    fn new() -> Self {
        let file = file("click_event.riv");
        let artboard = Artboard::instance_from_handle(&source(&file, Some("art-1"))).unwrap();
        let machine = artboard.state_machine_named("sm-1").unwrap();
        machine.with_instance_mut(|m| m.advance_seconds(0.0));
        artboard.advance_default(0.0);
        machine.with_instance_mut(|m| m.advance_seconds(0.0));
        assert_eq!(machine.with_instance(|m| m.hit_components_count()), 2);
        assert_eq!(events(&machine), 0);
        Self {
            _file: file,
            _artboard: artboard,
            machine,
        }
    }
    fn down(&self, b: B) {
        down(&self.machine, b, v(75.0, 75.0));
    }
    fn up(&self, b: B) {
        up(&self.machine, b, v(75.0, 75.0));
    }
    fn events(&self) -> usize {
        events(&self.machine)
    }
}
struct DownScene {
    _file: RuntimeFileHandle,
    _artboard: RuntimeArtboardInstanceHandle,
    _arena: CoreArena,
    machine: RuntimeStateMachineInstanceHandle,
    vm: CoreHandle,
}
impl DownScene {
    fn new(inputs: &[(L, B)]) -> Self {
        let file = file("listener_input_values.riv");
        let listener = first_listener(&source(&file, None), None);
        let arena = CoreArena::default();
        listener
            .with(|l| {
                let l = l.as_state_machine_listener().unwrap();
                assert_eq!(l.listener_input_type_count(), 1);
                let input = l.listener_input_type(0).unwrap();
                input
                    .with(|i| {
                        assert_eq!(i.listener_input_type_value(), Some(L::Down as u32));
                        assert_eq!(i.listener_input_type_pointer_button(), Some(B::Primary));
                    })
                    .unwrap();
            })
            .unwrap();
        for (i, (kind, button)) in inputs.iter().enumerate() {
            let mut input = ListenerInputTypePointerButton::default();
            CoreRegistry::set_uint(&mut input, 965, *kind as u32);
            CoreRegistry::set_uint(&mut input, 468, button.0 as u32);
            let handle = arena.insert(input);
            listener
                .with_mut(|l| {
                    let l = l.as_state_machine_listener_mut().unwrap();
                    if i == 0 {
                        l.replace_listener_input_type_for_testing(0, handle);
                    } else {
                        l.add_listener_input_type_for_testing(handle);
                    }
                    assert_eq!(
                        l.listener_input_type(i)
                            .unwrap()
                            .with(|i| i.listener_input_type_pointer_button())
                            .flatten(),
                        Some(*button)
                    );
                })
                .unwrap();
        }
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        let machine = artboard.state_machine_at(0).unwrap();
        let vm = file
            .with_file(|f| {
                f.create_default_view_model_instance_for_artboard(artboard.core_handle())
            })
            .unwrap();
        machine.with_instance_mut(|m| m.bind_view_model_instance(vm.clone()));
        machine.advance_and_apply(0.016);
        Self {
            _file: file,
            _artboard: artboard,
            _arena: arena,
            machine,
            vm,
        }
    }
    fn down(&self, b: B, p: Vec2D) {
        down(&self.machine, b, p);
        self.machine.advance_and_apply(0.016);
    }
    fn up(&self, b: B, p: Vec2D) {
        up(&self.machine, b, p);
        self.machine.advance_and_apply(0.016);
    }
    fn moved(&self, p: Vec2D) {
        moved(&self.machine, p);
        self.machine.advance_and_apply(0.016);
    }
    fn px(&self) -> f32 {
        self.vm
            .with_downcast::<ViewModelInstance, _>(|m| m.property_value_named("px"))
            .flatten()
            .unwrap()
            .with_downcast::<ViewModelInstanceNumber, _>(|n| n.base.property_value())
            .unwrap()
    }
}
#[test]
fn plain_listener_input_types_are_primary_button_listeners() {
    let f = file("listener_input_values.riv");
    let l = first_listener(&source(&f, None), None);
    l.with(|l| {
        let base = l.as_state_machine_listener().unwrap();
        assert_eq!(base.listener_input_type_count(), 1);
        assert_eq!(
            base.listener_input_type(0)
                .unwrap()
                .with(|i| i.listener_input_type_pointer_button())
                .flatten(),
            Some(B::Primary)
        );
        assert_eq!(l.state_machine_listener_has(L::Down), Some(true));
        assert_eq!(
            l.state_machine_listener_has_button(L::Down, B::Primary),
            Some(true)
        );
        assert_eq!(
            l.state_machine_listener_has_button(L::Down, B::Secondary),
            Some(false)
        );
        assert_eq!(
            l.state_machine_listener_has_button(L::Down, B::Middle),
            Some(false)
        );
    })
    .unwrap();
}
#[test]
fn pointer_button_input_types_listen_to_their_button_only() {
    let mut input = ListenerInputTypePointerButton::default();
    CoreRegistry::set_uint(&mut input, 965, L::Click as u32);
    assert!(input.is_type_of(ListenerInputType::TYPE_KEY));
    assert_eq!(input.pointer_button(), B::Primary);
    CoreRegistry::set_uint(&mut input, 468, B::Secondary.0 as u32);
    assert_eq!(input.pointer_button(), B::Secondary);
    CoreRegistry::set_uint(&mut input, 468, B::Middle.0 as u32);
    assert_eq!(input.pointer_button(), B::Middle);
}
#[test]
fn legacy_single_listeners_only_respond_to_primary() {
    let f = file("click_event.riv");
    let l = first_listener(&source(&f, Some("art-1")), Some("sm-1"));
    l.with(|l| {
        assert_eq!(
            l.as_state_machine_listener()
                .unwrap()
                .listener_input_type_count(),
            0
        );
        assert_eq!(l.state_machine_listener_has(L::Click), Some(true));
        assert_eq!(
            l.state_machine_listener_has_button(L::Click, B::Primary),
            Some(true)
        );
        assert_eq!(
            l.state_machine_listener_has_button(L::Click, B::Secondary),
            Some(false)
        );
    })
    .unwrap();
}
#[test]
fn primary_down_listener_ignores_other_buttons() {
    let s = DownScene::new(&[]);
    assert_eq!(s.px(), 0.0);
    s.down(B::Secondary, v(120.0, 310.0));
    assert_eq!(s.px(), 0.0);
    s.down(B::Middle, v(120.0, 310.0));
    assert_eq!(s.px(), 0.0);
    s.down(B::Primary, v(120.0, 310.0));
    assert_eq!(s.px(), 120.0);
}
#[test]
fn secondary_down_listener_fires_only_secondary() {
    let s = DownScene::new(&[(L::Down, B::Secondary)]);
    assert_eq!(s.px(), 0.0);
    s.down(B::Primary, v(120.0, 310.0));
    assert_eq!(s.px(), 0.0);
    s.down(B::Middle, v(120.0, 310.0));
    assert_eq!(s.px(), 0.0);
    s.down(B::Secondary, v(120.0, 310.0));
    assert_eq!(s.px(), 120.0);
    s.down(B::Secondary, v(200.0, 310.0));
    assert_eq!(s.px(), 200.0);
}
#[test]
fn listens_to_button_at_reports_buttons_under_point() {
    let inside = v(120.0, 310.0);
    let outside = v(-50.0, -50.0);
    let primary = DownScene::new(&[]);
    assert!(primary
        .machine
        .with_instance_mut(|m| m.listens_to_button_at(inside, B::Primary)));
    assert!(!primary
        .machine
        .with_instance_mut(|m| m.listens_to_button_at(inside, B::Secondary)));
    assert!(!primary
        .machine
        .with_instance_mut(|m| m.listens_to_button_at(outside, B::Primary)));
    let s = DownScene::new(&[(L::Down, B::Secondary)]);
    assert!(s
        .machine
        .with_instance_mut(|m| m.listens_to_button_at(inside, B::Secondary)));
    for b in [B::Primary, B::Middle] {
        assert!(!s
            .machine
            .with_instance_mut(|m| m.listens_to_button_at(inside, b)));
    }
    assert!(!s
        .machine
        .with_instance_mut(|m| m.listens_to_button_at(outside, B::Secondary)));
    let legacy = ClickScene::new();
    assert!(legacy
        .machine
        .with_instance_mut(|m| m.listens_to_button_at(v(75.0, 75.0), B::Primary)));
    assert!(!legacy
        .machine
        .with_instance_mut(|m| m.listens_to_button_at(v(75.0, 75.0), B::Secondary)));
    assert!(!legacy
        .machine
        .with_instance_mut(|m| m.listens_to_button_at(v(300.0, 75.0), B::Primary)));
}
#[test]
fn synthetic_drag_events_carry_dragging_button() {
    // Each tuple is one independently constructed upstream SECTION.
    for (press, drag, end_kind, end_button, after_move, after_up) in [
        (
            B::Secondary,
            B::Secondary,
            L::DragEnd,
            B::Secondary,
            150.0,
            200.0,
        ),
        (
            B::Secondary,
            B::Secondary,
            L::DragEnd,
            B::Primary,
            150.0,
            150.0,
        ),
        (
            B::Secondary,
            B::Secondary,
            L::DragStart,
            B::Secondary,
            150.0,
            150.0,
        ),
        (B::Primary, B::Primary, L::DragEnd, B::Primary, 150.0, 200.0),
        (B::Primary, B::Secondary, L::DragEnd, B::Secondary, 0.0, 0.0),
    ] {
        let s = DownScene::new(&[(L::Drag, drag), (end_kind, end_button)]);
        s.down(press, v(100.0, 100.0));
        if end_button == B::Secondary && end_kind == L::DragEnd && press == B::Secondary {
            assert_eq!(s.px(), 0.0);
        }
        s.moved(v(150.0, 150.0));
        assert_eq!(s.px(), after_move);
        s.up(press, v(200.0, 200.0));
        assert_eq!(s.px(), after_up);
    }
}
#[test]
fn primary_click_ignores_other_buttons() {
    let s = ClickScene::new();
    for (b, n) in [
        (B::Primary, 1),
        (B::Secondary, 1),
        (B::Middle, 1),
        (B::Primary, 2),
    ] {
        s.down(b);
        s.up(b);
        assert_eq!(s.events(), n);
    }
}
#[test]
fn click_needs_same_button_down_and_up() {
    let s = ClickScene::new();
    s.down(B::Primary);
    s.up(B::Secondary);
    assert_eq!(s.events(), 0);
    s.up(B::Primary);
    assert_eq!(s.events(), 1);
    s.down(B::Secondary);
    s.up(B::Primary);
    assert_eq!(s.events(), 1);
}
#[test]
fn held_press_survives_another_button() {
    let s = ClickScene::new();
    s.down(B::Primary);
    s.down(B::Secondary);
    s.up(B::Secondary);
    assert_eq!(s.events(), 0);
    s.up(B::Primary);
    assert_eq!(s.events(), 1);
}
#[test]
fn releasing_other_button_outside_keeps_held_press() {
    let s = ClickScene::new();
    s.down(B::Primary);
    up(&s.machine, B::Secondary, v(300.0, 75.0));
    s.up(B::Primary);
    assert_eq!(s.events(), 1);
    s.down(B::Primary);
    up(&s.machine, B::Primary, v(300.0, 75.0));
    s.up(B::Primary);
    assert_eq!(s.events(), 1);
}
#[test]
fn exported_secondary_click_fixture_loads_pointer_button_input() {
    let f = file("pointer_button_secondary.riv");
    let source = source(&f, None);
    let definition = source
        .with_downcast::<Artboard, _>(|a| a.state_machine_handle_at(0))
        .flatten()
        .unwrap();
    assert_eq!(
        definition.with_downcast::<StateMachine, _>(|m| m.listener_count()),
        Some(1)
    );
    let l = first_listener(&source, None);
    let input = l
        .with(|l| {
            let l = l.as_state_machine_listener().unwrap();
            assert_eq!(l.listener_input_type_count(), 1);
            l.listener_input_type(0).unwrap()
        })
        .unwrap();
    input
        .with(|i| {
            assert!(CoreObject::is_type_of(i, ListenerInputTypePointerButton::TYPE_KEY));
            assert_eq!(i.listener_input_type_pointer_button(), Some(B::Secondary));
            assert_eq!(i.listener_input_type_value(), Some(L::Click as u32));
        })
        .unwrap();
    let a = f.with_file(|f| f.artboard_default()).unwrap();
    let m = a.state_machine_at(0).unwrap();
    m.advance_and_apply(0.0);
    assert!(!m.with_instance(|m| m.get_bool("clicked").unwrap().value()));
    down(&m, B::Primary, v(250.0, 250.0));
    up(&m, B::Primary, v(250.0, 250.0));
    assert_eq!(events(&m), 0);
    assert!(!m.with_instance(|m| m.get_bool("clicked").unwrap().value()));
    down(&m, B::Secondary, v(250.0, 250.0));
    up(&m, B::Secondary, v(250.0, 250.0));
    assert_eq!(events(&m), 1);
    assert!(m.with_instance(|m| m.get_bool("clicked").unwrap().value()));
}
#[test]
fn nested_pointer_buttons_dispatch_through_host() {
    for drag_button in [B::Secondary, B::Primary] {
        let f = file("pointer_button_nested.riv");
        let a = f.with_file(|f| f.artboard_default()).unwrap();
        let m = a.state_machine_at(0).unwrap();
        m.advance_and_apply(0.0);
        let nested = a.with_artboard(|a| {
            a.objects_typed::<NestedArtboard>()
                .iter()
                .collect::<Vec<_>>()
        });
        assert_eq!(nested.len(), 1);
        let animation = nested[0]
            .with_downcast::<NestedArtboard, _>(|n| {
                assert_eq!(n.nested_animations().len(), 1);
                n.nested_animations()[0].clone()
            })
            .unwrap();
        let inner = animation
            .with_downcast::<NestedStateMachine, _>(|n| n.state_machine_instance())
            .flatten()
            .unwrap();
        assert!(m.with_instance_mut(|m| m.listens_to_button_at(v(200.0, 200.0), B::Secondary)));
        assert!(!m.with_instance_mut(|m| m.listens_to_button_at(v(200.0, 200.0), B::Primary)));
        assert!(!m.with_instance_mut(|m| m.listens_to_button_at(v(20.0, 20.0), B::Secondary)));
        down(&m, B::Primary, v(200.0, 200.0));
        up(&m, B::Primary, v(200.0, 200.0));
        assert_eq!(events(&inner), 0);
        down(&m, B::Secondary, v(200.0, 200.0));
        up(&m, B::Secondary, v(200.0, 200.0));
        assert_eq!(events(&inner), 1);
        m.advance_and_apply(0.016);
        down(&m, drag_button, v(150.0, 150.0));
        moved(&m, v(170.0, 170.0));
        let after_move = events(&inner);
        if drag_button == B::Secondary {
            assert!(after_move >= 1);
        }
        up(&m, drag_button, v(180.0, 180.0));
        if drag_button == B::Secondary {
            assert!(events(&inner) > after_move);
        } else {
            assert_eq!(events(&inner), 0);
        }
    }
}
#[test]
fn scroll_views_only_drag_primary() {
    let f = file("layout/layout_scroll_vertical.riv");
    let a = Artboard::instance_from_handle(&source(&f, None)).unwrap();
    let m = a.state_machine_named("State Machine 1").unwrap();
    let scroll = a.with_artboard(|a| a.objects_typed::<ScrollConstraint>().first().unwrap());
    m.advance_and_apply(0.0);
    let over = a.with_artboard(|a| v(a.width() / 2.0, a.height() / 2.0));
    let dragged = v(over.x, over.y - 50.0);
    down(&m, B::Secondary, over);
    moved(&m, dragged);
    up(&m, B::Secondary, dragged);
    m.advance_and_apply(0.0);
    assert_eq!(
        scroll.with_downcast::<ScrollConstraint, _>(|s| s.offset_y()),
        Some(0.0)
    );
    down(&m, B::Primary, over);
    moved(&m, dragged);
    m.advance_and_apply(0.0);
    assert_ne!(
        scroll.with_downcast::<ScrollConstraint, _>(|s| s.offset_y()),
        Some(0.0)
    );
    up(&m, B::Primary, dragged);
}
#[test]
fn text_inputs_only_select_primary() {
    let f = file("text_input.riv");
    let a = Artboard::instance_from_handle(&source(&f, Some("Text Input - Multiline"))).unwrap();
    let m = a.state_machine_at(0).unwrap();
    m.advance_and_apply(0.0);
    let input = a.with_artboard(|a| a.objects_typed::<TextInput>().first().unwrap());
    input
        .with_downcast_mut::<TextInput, _>(|i| {
            i.raw_text_input().set_text("hello world".to_owned())
        })
        .unwrap();
    m.advance_and_apply(0.0);
    down(&m, B::Secondary, v(8.0, 8.0));
    assert_eq!(
        input.with_downcast::<TextInput, _>(|i| i.is_dragging()),
        Some(false)
    );
    up(&m, B::Secondary, v(8.0, 8.0));
    down(&m, B::Primary, v(8.0, 8.0));
    assert_eq!(
        input.with_downcast::<TextInput, _>(|i| i.is_dragging()),
        Some(true)
    );
    up(&m, B::Primary, v(8.0, 8.0));
}
#[test]
fn scenes_without_listeners_ignore_buttons() {
    let f = file("listener_input_values.riv");
    let a = f.with_file(|f| f.artboard_default()).unwrap();
    assert!(a.with_artboard(|a| a.animation_count()) > 0);
    let mut animation = a.animation_at(0).unwrap();
    let scene: &mut dyn SceneBehavior = animation.as_mut();
    assert_eq!(
        scene.pointer_down(v(1.0, 1.0), 0, B::Secondary),
        HitResult::None
    );
    assert_eq!(
        scene.pointer_up(v(1.0, 1.0), 0, B::Secondary),
        HitResult::None
    );
}
