//! Literal capture_listener_test.cpp at upstream 542aa31c.
//! Fixture/RML are read from the exact upstream source checkout.
#[path = "support/text_style_global.rs"]
mod support;
use nuxie_runtime::source::{
    animation::{
        nested_state_machine::NestedStateMachine, state_machine::StateMachine,
        state_machine_instance::StateMachineInstance, state_machine_listener::StateMachineListener,
    },
    artboard_component_list::ArtboardComponentList,
    focus_data::FocusData,
    generated::animation::state_machine_listener_base::StateMachineListenerBase,
    hit_result::HitResult,
    input::focusable::{Key, KeyModifiers},
    listener_type::ListenerType,
    nested_artboard::NestedArtboard,
    text::{cursor::Cursor, text_input::TextInput},
};
use std::sync::{Mutex, MutexGuard};
use support::*;
static TEST_LOCK: Mutex<()> = Mutex::new(());
fn isolated() -> MutexGuard<'static, ()> {
    TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}
const ON_FIELD_A_TEXT: Vec2D = Vec2D::new(40.0, 44.0);
const ON_FIELD_A_VIEWPORT: Vec2D = Vec2D::new(100.0, 44.0);
const ON_FIELD_B_TEXT: Vec2D = Vec2D::new(40.0, 112.0);
const ON_FIELD_B_VIEWPORT: Vec2D = Vec2D::new(100.0, 112.0);
const UNDER_OVERLAY: Vec2D = Vec2D::new(200.0, 112.0);
const OUTSIDE_SLOTS: Vec2D = Vec2D::new(270.0, 112.0);
const ON_LIST_ROW: Vec2D = Vec2D::new(100.0, 44.0);
const UNDER_LIST_OVERLAY: Vec2D = Vec2D::new(200.0, 44.0);
const ON_SECOND_LIST_ROW: Vec2D = Vec2D::new(100.0, 80.0);
const ON_LEAF: Vec2D = Vec2D::new(40.0, 30.0);
fn capture(file: &RuntimeFileHandle, artboard_name: &str, listener_name: &str) {
    let artboard = source(file, artboard_name);
    let machines = artboard
        .with_downcast::<Artboard, _>(|a| {
            (0..a.state_machine_count())
                .map(|i| a.state_machine_handle_at(i).unwrap())
                .collect::<Vec<_>>()
        })
        .unwrap();
    for machine in machines {
        let listeners = machine
            .with_downcast::<StateMachine, _>(|m| {
                (0..m.listener_count())
                    .map(|i| m.listener(i).unwrap())
                    .collect::<Vec<_>>()
            })
            .unwrap();
        for listener in listeners {
            if listener
                .with_downcast::<StateMachineListener, _>(|l| l.name() == listener_name)
                .unwrap()
            {
                assert!(CoreRegistry::set_uint_handle(
                    &listener,
                    StateMachineListenerBase::PHASE_VALUE_PROPERTY_KEY as i32,
                    1
                ));
                return;
            }
        }
    }
    panic!("no listener named {listener_name}");
}
#[derive(Clone)]
struct Machine(RuntimeStateMachineInstanceHandle);
impl Machine {
    fn advance(&self) {
        self.0.advance_and_apply(0.0);
    }
    fn flag(&self, n: &str) -> bool {
        self.0.with_instance(|m| m.get_bool(n).unwrap().value())
    }
    fn set_flag(&self, n: &str, v: bool) {
        assert!(self.0.with_instance(|m| m.get_bool(n).is_some()));
        self.0.set_bool(n, v);
    }
    fn down(&self, p: Vec2D) -> HitResult {
        self.down_button(p, 0, PointerButton::Primary)
    }
    fn down_button(&self, p: Vec2D, id: i32, b: PointerButton) -> HitResult {
        self.0.with_instance_mut(|m| m.pointer_down(p, id, b))
    }
    fn up(&self, p: Vec2D) -> HitResult {
        self.up_button(p, 0, PointerButton::Primary)
    }
    fn up_button(&self, p: Vec2D, id: i32, b: PointerButton) -> HitResult {
        self.0.with_instance_mut(|m| m.pointer_up(p, id, b))
    }
    fn move_to(&self, p: Vec2D) {
        self.move_id(p, 0);
    }
    fn move_id(&self, p: Vec2D, id: i32) {
        self.0.with_instance_mut(|m| m.pointer_move(p, 0.0, id));
    }
    fn exit(&self, p: Vec2D) {
        self.0.with_instance_mut(|m| m.pointer_exit(p, 0));
    }
}
struct Field {
    machine: Machine,
    input: CoreHandle,
    focus_data: CoreHandle,
}
impl Field {
    fn flag(&self, n: &str) -> bool {
        self.machine.flag(n)
    }
    fn set_flag(&self, n: &str, v: bool) {
        self.machine.set_flag(n, v)
    }
    fn caret(&self) -> u32 {
        self.input
            .with_downcast_mut::<TextInput, _>(|i| {
                i.raw_text_input().cursor().end().code_point_index()
            })
            .unwrap()
    }
    fn set_caret(&self, i: u32) {
        self.input
            .with_downcast_mut::<TextInput, _>(|t| {
                t.raw_text_input()
                    .set_cursor(Cursor::collapsed(CursorPosition::unresolved(i)))
            })
            .unwrap();
    }
    fn focused(&self) -> bool {
        self.input
            .with_downcast::<TextInput, _>(TextInput::is_focused)
            .unwrap()
    }
    fn text(&self) -> String {
        self.input
            .with_downcast::<TextInput, _>(|i| i.text().to_string())
            .unwrap()
    }
    fn set_text(&self, s: &str) {
        assert!(CoreRegistry::set_string_handle(&self.input, 817, s.into()));
    }
}
struct HostScene {
    machine: Machine,
    a: Field,
    b: Field,
    _artboard: RuntimeArtboardInstanceHandle,
}
impl HostScene {
    fn new(file: &RuntimeFileHandle) -> Self {
        let artboard = file
            .with_file(|f| f.artboard_named("Host"))
            .expect("Host instance");
        let machine = Machine(artboard.state_machine_at(0).unwrap());
        let field = |name: &str| {
            let nested = artboard
                .with_artboard(|a| a.find_handle::<NestedArtboard>(name))
                .unwrap();
            let (animations, instance) = nested
                .with_downcast::<NestedArtboard, _>(|n| {
                    (
                        n.nested_animations().to_vec(),
                        n.artboard_instance_default().unwrap(),
                    )
                })
                .unwrap();
            assert_eq!(animations.len(), 1);
            let machine = Machine(
                animations[0]
                    .with_downcast::<NestedStateMachine, _>(
                        NestedStateMachine::state_machine_instance,
                    )
                    .flatten()
                    .unwrap(),
            );
            let input = instance.with_artboard(|a| a.find_all_handles::<TextInput>()[0].clone());
            let focus_data = input
                .with_downcast::<TextInput, _>(|i| {
                    i.children()
                        .iter()
                        .find(|c| c.is_type_of(FocusData::TYPE_KEY))
                        .cloned()
                })
                .flatten()
                .unwrap();
            Field {
                machine,
                input,
                focus_data,
            }
        };
        let a = field("Field A");
        let b = field("Field B");
        machine.advance();
        machine.advance();
        Self {
            machine,
            a,
            b,
            _artboard: artboard,
        }
    }
    fn flag(&self, n: &str) -> bool {
        self.machine.flag(n)
    }
    fn focus(&self, f: &Field) {
        let node = f
            .focus_data
            .with_downcast_mut::<FocusData, _>(FocusData::focus_node)
            .unwrap();
        let manager = self.machine.0.with_instance(|m| m.focus_manager()).unwrap();
        manager.with_focus_manager_mut(|m| m.set_focus(node));
        assert!(f.focused());
    }
    fn key(&self, key: Key, pressed: bool) -> bool {
        // Match the production host entry: release the SMI borrow before
        // dispatch, since listener actions may re-enter it or change focus.
        let manager = self.machine.0.with_instance(|m| m.focus_manager());
        let claimed = manager.is_some_and(|manager| {
            manager.key_input_dispatch(key, KeyModifiers::NONE, pressed, false)
        });
        self.machine.advance();
        claimed
    }
    fn press(&self, key: Key) -> bool {
        self.key(key, true)
    }
    fn release(&self, key: Key) -> bool {
        self.key(key, false)
    }
    fn click(&self, p: Vec2D) {
        self.click_id(p, 0)
    }
    fn click_id(&self, p: Vec2D, id: i32) {
        self.machine.down_button(p, id, PointerButton::Primary);
        self.machine.up_button(p, id, PointerButton::Primary);
        self.machine.advance();
    }
}
struct ListScene {
    machine: Machine,
    list: CoreHandle,
    view_model: CoreHandle,
    _artboard: RuntimeArtboardInstanceHandle,
}
impl ListScene {
    fn new(file: &RuntimeFileHandle) -> Self {
        let artboard = file
            .with_file(|f| f.artboard_named("List Host"))
            .expect("List Host instance");
        let machine = Machine(artboard.state_machine_at(0).unwrap());
        let view_model = vm(file, &artboard);
        machine
            .0
            .with_instance_mut(|m| m.bind_view_model_instance(view_model.clone()));
        machine.advance();
        machine.advance();
        let list = artboard
            .with_artboard(|a| a.find_handle::<ArtboardComponentList>("Rows"))
            .unwrap();
        assert_eq!(
            list.with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::item_count),
            Some(1)
        );
        Self {
            machine,
            list,
            view_model,
            _artboard: artboard,
        }
    }
    fn add_row(&self, file: &RuntimeFileHandle) {
        let rows = property(&self.view_model, "rows");
        let item = rows
            .insert_sibling(ViewModelInstanceListItem::default())
            .unwrap();
        let row = file
            .with_file(|f| f.create_view_model_instance_for_name("Row"))
            .unwrap();
        item.with_downcast_mut::<ViewModelInstanceListItem, _>(|i| {
            i.set_view_model_instance(Some(row))
        })
        .unwrap();
        rows.with_downcast_mut::<ViewModelInstanceList, _>(|l| l.add_item(item))
            .unwrap();
        self.machine.advance();
        self.machine.advance();
        assert_eq!(
            self.list
                .with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::item_count),
            Some(2)
        );
    }
    fn row_hovered(&self, index: i32) -> bool {
        let row = self
            .list
            .with_downcast::<ArtboardComponentList, _>(|l| l.state_machine_instance(index))
            .flatten()
            .unwrap();
        Machine(row).flag("rowHovered")
    }
}
#[test]
fn fields_take_keys_and_presses_without_capture() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    let scene = HostScene::new(&file);
    assert_eq!(StateMachineInstance::live_capture_machines(), 0);
    scene.click(ON_FIELD_B_TEXT);
    assert!(scene.b.focused());
    assert!(!scene.flag("slotBPressed"));
    scene.b.set_caret(2);
    assert!(scene.press(Key::UP));
    assert!(scene.b.focused());
    assert_eq!(scene.b.caret(), 0);
}
#[test]
fn host_capture_key_runs_ahead_of_nested_input() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    capture(&file, "Host", "Slot B Up");
    let scene = HostScene::new(&file);
    scene.focus(&scene.b);
    scene.b.set_caret(2);
    assert!(scene.press(Key::UP));
    assert!(scene.a.focused());
    assert!(!scene.b.focused());
    assert_eq!(scene.b.caret(), 2);
    assert_eq!(scene.b.text(), "AB");
    assert!(!scene.b.flag("fieldUp"));
    scene.focus(&scene.b);
    scene.b.set_caret(0);
    assert!(scene.press(Key::DOWN));
    assert!(scene.b.focused());
    assert_eq!(scene.b.caret(), 2);
}
#[test]
fn outer_capture_key_beats_nested_one() {
    let _guard = isolated();
    for host_captures in [true, false] {
        let file = load("capture_listeners.riv");
        if host_captures {
            capture(&file, "Host", "Slot B Up");
        }
        capture(&file, "Field", "Field Up");
        let scene = HostScene::new(&file);
        scene.focus(&scene.b);
        if !host_captures {
            scene.b.set_caret(2);
        }
        assert!(scene.press(Key::UP));
        if host_captures {
            assert!(scene.a.focused());
            assert!(!scene.b.flag("fieldUp"));
        } else {
            assert!(scene.b.flag("fieldUp"));
            assert!(scene.b.focused());
            assert_eq!(scene.b.caret(), 2);
        }
    }
}
#[test]
fn capture_key_release_listener_claims_press_too() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    capture(&file, "Host", "Slot B Enter Release");
    let scene = HostScene::new(&file);
    scene.focus(&scene.b);
    assert!(scene.press(Key::ENTER));
    assert!(!scene.flag("slotBEnterReleased"));
    assert!(!scene.b.flag("fieldEnter"));
    assert!(scene.release(Key::ENTER));
    assert!(scene.flag("slotBEnterReleased"));
}
#[test]
fn text_input_editing_runs_once_per_key() {
    let _guard = isolated();
    let backspaced = |times| {
        let file = load("capture_listeners.riv");
        let scene = HostScene::new(&file);
        scene.b.set_text("ABCDEF");
        scene.machine.advance();
        scene.b.set_caret(6);
        for _ in 0..times {
            scene
                .b
                .input
                .with_downcast_mut::<TextInput, _>(|i| {
                    i.key_input(Key::BACKSPACE, KeyModifiers::NONE, true, false)
                })
                .unwrap();
        }
        scene.b.text()
    };
    let once = backspaced(1);
    assert_ne!(once, backspaced(2));
    let file = load("capture_listeners.riv");
    let scene = HostScene::new(&file);
    scene.b.set_text("ABCDEF");
    scene.machine.advance();
    scene.focus(&scene.b);
    scene.b.set_caret(6);
    assert!(scene.press(Key::BACKSPACE));
    assert_eq!(scene.b.text(), once);
}
#[test]
fn filtered_text_input_key_listener_runs_actions() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    let scene = HostScene::new(&file);
    scene.focus(&scene.b);
    assert!(scene.press(Key::ENTER));
    assert!(scene.b.flag("fieldEnter"));
    assert!(scene.release(Key::ENTER));
    assert!(scene.flag("slotBEnterReleased"));
}
#[test]
fn unhandled_text_input_keys_bubble_past_match_any_listeners() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    let scene = HostScene::new(&file);
    scene.focus(&scene.b);
    assert!(!scene.press(Key::TAB));
    assert!(!scene.press(Key::ESCAPE));
    assert!(scene.b.focused());
}
#[test]
fn host_capture_press_keeps_gesture_from_field() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    capture(&file, "Host", "Slot B Press");
    let scene = HostScene::new(&file);
    assert_eq!(StateMachineInstance::live_capture_machines(), 1);
    scene.b.set_caret(2);
    scene.machine.move_to(ON_FIELD_B_VIEWPORT);
    assert!(scene.b.flag("fieldHovered"));
    assert_ne!(scene.machine.down(ON_FIELD_B_TEXT), HitResult::None);
    scene.machine.advance();
    assert!(scene.flag("slotBPressed"));
    assert!(!scene.b.focused());
    assert_eq!(scene.b.caret(), 2);
    scene.machine.move_to(ON_FIELD_B_VIEWPORT);
    scene.machine.move_to(OUTSIDE_SLOTS);
    scene.machine.advance();
    assert_eq!(scene.b.caret(), 2);
    assert!(scene.b.flag("fieldHovered"));
    scene.machine.up(OUTSIDE_SLOTS);
    scene.machine.advance();
    assert!(!scene.b.flag("fieldHovered"));
    assert!(!scene.b.focused());
    scene.click(ON_FIELD_A_VIEWPORT);
    assert!(scene.a.focused());
    assert!(scene.flag("slotAPressed"));
}
#[test]
fn nested_capture_listener_runs_under_host_without_one() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    capture(&file, "Field", "Viewport Down");
    let scene = HostScene::new(&file);
    scene.b.set_caret(0);
    scene.click(ON_FIELD_B_TEXT);
    assert!(scene.b.focused());
    assert_eq!(scene.b.caret(), 0);
}
#[test]
fn capture_press_that_moves_focus_keeps_it() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    capture(&file, "Field", "Viewport Down");
    let scene = HostScene::new(&file);
    scene.click(ON_FIELD_B_VIEWPORT);
    assert!(scene.b.focused());
    scene.click(OUTSIDE_SLOTS);
    assert!(!scene.b.focused());
}
#[test]
fn capture_move_leaves_nested_hover_alone() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    capture(&file, "Host", "Slot B Move");
    let scene = HostScene::new(&file);
    scene.machine.move_to(ON_FIELD_B_VIEWPORT);
    assert!(scene.flag("slotBMoved"));
    assert!(scene.b.flag("fieldHovered"));
    scene.machine.move_to(OUTSIDE_SLOTS);
    assert!(!scene.b.flag("fieldHovered"));
    scene.click(ON_FIELD_B_TEXT);
    assert!(scene.b.focused());
}
#[test]
fn opaque_overlay_shields_capture_target() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    capture(&file, "Host", "Slot B Press");
    let scene = HostScene::new(&file);
    scene.click(UNDER_OVERLAY);
    assert!(scene.flag("overlayPressed"));
    assert!(!scene.flag("slotBPressed"));
    scene.click(ON_FIELD_B_VIEWPORT);
    assert!(scene.flag("slotBPressed"));
}
#[test]
fn capture_claim_holds_one_pointer_only() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    capture(&file, "Host", "Slot B Press");
    let scene = HostScene::new(&file);
    scene.b.set_caret(2);
    scene
        .machine
        .down_button(ON_FIELD_B_TEXT, 1, PointerButton::Primary);
    scene.machine.advance();
    assert!(scene.flag("slotBPressed"));
    assert!(!scene.b.focused());
    scene.b.set_flag("fieldHovered", false);
    scene.machine.move_id(ON_FIELD_B_VIEWPORT, 2);
    assert!(scene.b.flag("fieldHovered"));
    scene.click_id(ON_FIELD_A_TEXT, 2);
    assert!(scene.a.focused());
    scene.machine.move_id(ON_FIELD_B_VIEWPORT, 1);
    scene
        .machine
        .up_button(ON_FIELD_B_VIEWPORT, 1, PointerButton::Primary);
    scene.machine.advance();
    assert!(!scene.b.focused());
    assert_eq!(scene.b.caret(), 2);
}
#[test]
fn another_button_keeps_first_buttons_claim() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    capture(&file, "Host", "Slot B Press");
    let scene = HostScene::new(&file);
    scene
        .machine
        .down_button(ON_FIELD_B_TEXT, 0, PointerButton::Primary);
    scene
        .machine
        .down_button(ON_FIELD_B_TEXT, 0, PointerButton::Secondary);
    scene
        .machine
        .up_button(ON_FIELD_B_TEXT, 0, PointerButton::Secondary);
    scene.machine.advance();
    assert!(scene.flag("slotBPressed"));
    scene.machine.move_to(ON_FIELD_B_VIEWPORT);
    scene.machine.advance();
    assert!(!scene.b.flag("fieldHovered"));
    assert!(!scene.b.focused());
    scene
        .machine
        .up_button(ON_FIELD_B_VIEWPORT, 0, PointerButton::Primary);
    scene.machine.advance();
    assert!(scene.b.flag("fieldHovered"));
}
#[test]
fn capture_drag_release_stays_off_claimed_field() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    {
        let scene = HostScene::new(&file);
        scene.click(ON_FIELD_A_VIEWPORT);
        assert!(scene.a.flag("fieldReleased"));
    }
    capture(&file, "Host", "Slot A Drag");
    let scene = HostScene::new(&file);
    let dragged = ON_FIELD_A_VIEWPORT + Vec2D::new(5.0, 0.0);
    scene.machine.down(ON_FIELD_A_VIEWPORT);
    scene.machine.move_to(dragged);
    scene.machine.advance();
    assert!(scene.flag("slotADragged"));
    scene.machine.up(dragged);
    scene.machine.advance();
    assert!(!scene.a.flag("fieldReleased"));
    assert!(scene.a.flag("fieldHovered"));
}
#[test]
fn host_claim_holds_while_overlay_covers_field() {
    let _guard = isolated();
    for nested_capture in [false, true] {
        let file = load("capture_listeners.riv");
        capture(&file, "Host", "Slot B Press");
        if nested_capture {
            capture(&file, "Field", "Viewport Enter");
        }
        let scene = HostScene::new(&file);
        scene.machine.down(ON_FIELD_B_VIEWPORT);
        scene.machine.advance();
        assert!(scene.flag("slotBPressed"));
        scene.machine.move_to(UNDER_OVERLAY);
        scene.machine.move_to(ON_FIELD_B_VIEWPORT);
        scene.machine.advance();
        assert!(!scene.b.flag("fieldHovered"));
        assert!(!scene.b.focused());
        scene.machine.up(ON_FIELD_B_VIEWPORT);
        scene.machine.advance();
        assert!(scene.b.flag("fieldHovered"));
    }
}
#[test]
fn host_claim_holds_while_overlay_covers_list_row() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    {
        let scene = ListScene::new(&file);
        scene.machine.move_to(ON_LIST_ROW);
        assert!(scene.row_hovered(0));
        scene.machine.move_to(UNDER_LIST_OVERLAY);
        assert!(!scene.row_hovered(0));
    }
    capture(&file, "List Host", "List Slot Press");
    let scene = ListScene::new(&file);
    scene.machine.down(ON_LIST_ROW);
    scene.machine.advance();
    assert!(scene.machine.flag("listSlotPressed"));
    scene.machine.move_to(UNDER_LIST_OVERLAY);
    scene.machine.move_to(ON_LIST_ROW);
    scene.machine.advance();
    assert!(!scene.row_hovered(0));
    scene.machine.up(ON_LIST_ROW);
    scene.machine.advance();
    assert!(scene.row_hovered(0));
}
#[test]
fn outer_claim_for_another_button_keeps_inner_claimed_scope() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    capture(&file, "Claims", "Outer Secondary Press");
    capture(&file, "Claims", "Inner Press");
    capture(&file, "Claims", "Leaf Move");
    let artboard = file
        .with_file(|f| f.artboard_named("Claims"))
        .expect("Claims instance");
    let machine = Machine(artboard.state_machine_at(0).unwrap());
    machine.advance();
    machine.advance();
    machine.down_button(ON_LEAF, 0, PointerButton::Primary);
    machine.down_button(ON_LEAF, 0, PointerButton::Secondary);
    machine.advance();
    assert!(machine.flag("innerPressed"));
    assert!(machine.flag("outerPressed"));
    machine.up_button(ON_LEAF, 0, PointerButton::Primary);
    machine.move_to(ON_LEAF + Vec2D::new(5.0, 0.0));
    machine.advance();
    assert!(!machine.flag("leafMoved"));
    machine.up_button(ON_LEAF, 0, PointerButton::Secondary);
    machine.move_to(ON_LEAF);
    machine.advance();
    assert!(machine.flag("leafMoved"));
}
#[test]
fn nested_own_release_leaves_host_claim() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    capture(&file, "Field", "Viewport Down");
    capture(&file, "Host", "Slot B Secondary Press");
    let scene = HostScene::new(&file);
    scene
        .machine
        .down_button(ON_FIELD_B_VIEWPORT, 0, PointerButton::Primary);
    scene
        .machine
        .down_button(ON_FIELD_B_VIEWPORT, 0, PointerButton::Secondary);
    scene.machine.advance();
    assert!(scene.flag("slotBSecondaryPressed"));
    scene
        .machine
        .up_button(ON_FIELD_B_VIEWPORT, 0, PointerButton::Primary);
    scene
        .machine
        .move_to(ON_FIELD_B_VIEWPORT + Vec2D::new(5.0, 0.0));
    scene.machine.advance();
    assert!(!scene.b.flag("fieldHovered"));
    scene
        .machine
        .up_button(ON_FIELD_B_VIEWPORT, 0, PointerButton::Secondary);
    scene.machine.advance();
    assert!(scene.b.flag("fieldHovered"));
}
#[test]
fn list_row_added_during_claim_is_under_it_too() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    {
        let scene = ListScene::new(&file);
        scene.add_row(&file);
        scene.machine.move_to(ON_SECOND_LIST_ROW);
        assert!(scene.row_hovered(1));
    }
    capture(&file, "List Host", "List Slot Press");
    let scene = ListScene::new(&file);
    scene.machine.down(ON_LIST_ROW);
    scene.machine.advance();
    assert!(scene.machine.flag("listSlotPressed"));
    scene.add_row(&file);
    scene.machine.move_to(ON_SECOND_LIST_ROW);
    scene.machine.advance();
    assert!(!scene.row_hovered(1));
    scene.machine.up(ON_SECOND_LIST_ROW);
    scene.machine.advance();
    assert!(scene.row_hovered(1));
}
#[test]
fn pointer_dispatch_skips_capture_while_tree_has_none() {
    let _guard = isolated();
    let file = load("capture_listeners.riv");
    let scene = HostScene::new(&file);
    assert_eq!(StateMachineInstance::live_capture_machines(), 0);
    let visits = StateMachineInstance::capture_pass_visits();
    scene.machine.move_to(ON_FIELD_B_VIEWPORT);
    scene.click(ON_FIELD_B_TEXT);
    scene.machine.exit(OUTSIDE_SLOTS);
    assert_eq!(StateMachineInstance::capture_pass_visits(), visits);
    let capture_file = load("capture_listeners.riv");
    capture(&capture_file, "Host", "Slot B Move");
    let capture_scene = HostScene::new(&capture_file);
    assert_eq!(StateMachineInstance::live_capture_machines(), 1);
    let visits = StateMachineInstance::capture_pass_visits();
    scene.machine.move_to(ON_FIELD_B_VIEWPORT);
    scene.click(ON_FIELD_B_TEXT);
    assert_eq!(StateMachineInstance::capture_pass_visits(), visits);
    let visits = StateMachineInstance::capture_pass_visits();
    capture_scene.machine.move_to(ON_FIELD_B_VIEWPORT);
    assert_eq!(StateMachineInstance::capture_pass_visits(), visits + 1);
}

// Literal hidden capture_bench_test.cpp. Kept opt-in as upstream; there are no
// performance assertions or default-suite timing costs.
fn bench_load(path: &str) -> Option<RuntimeFileHandle> {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR"),
    );
    let bytes = std::fs::read(root.join("tests/unit_tests/assets").join(path))
        .expect("upstream benchmark fixture");
    let mut factory = PersistentFactory::new(nuxie_render_api::NullFactory::default());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )?;
    file.with_file(File::artboard)?;
    Some(file)
}
struct BenchInstance {
    machine: Machine,
    artboard: RuntimeArtboardInstanceHandle,
}
impl BenchInstance {
    fn new(file: &RuntimeFileHandle, name: Option<&str>) -> Option<Self> {
        let artboard = file.with_file(|f| match name {
            Some(n) => f.artboard_named(n),
            None => f.artboard_default(),
        })?;
        let machine = Machine(artboard.state_machine_at(0)?);
        machine.advance();
        machine.advance();
        Some(Self { machine, artboard })
    }
}
fn first_pointer_capture(
    file: &RuntimeFileHandle,
    name: Option<&str>,
    listener_name: Option<&str>,
) -> Option<String> {
    let artboard = file.with_file(|f| match name {
        Some(n) => f.artboard_named_source(n),
        None => f.artboard(),
    })?;
    let machine = artboard
        .with_downcast::<Artboard, _>(|a| a.state_machine_handle_at(0))
        .flatten()?;
    let listeners = machine
        .with_downcast::<StateMachine, _>(|m| {
            (0..m.listener_count())
                .filter_map(|i| m.listener(i))
                .collect::<Vec<_>>()
        })
        .unwrap();
    for listener in listeners {
        let name = listener
            .with(|l| {
                let name = l.state_machine_component_name()?;
                if [
                    ListenerType::Enter,
                    ListenerType::Exit,
                    ListenerType::Down,
                    ListenerType::Up,
                    ListenerType::Move,
                    ListenerType::Click,
                    ListenerType::DragStart,
                    ListenerType::DragEnd,
                    ListenerType::Drag,
                ]
                .into_iter()
                .any(|kind| l.state_machine_listener_has(kind).unwrap_or(false))
                    && listener_name.is_none_or(|n| name == n)
                {
                    Some(name)
                } else {
                    None
                }
            })
            .flatten();
        if let Some(name) = name {
            assert!(CoreRegistry::set_uint_handle(
                &listener,
                StateMachineListenerBase::PHASE_VALUE_PROPERTY_KEY as i32,
                1
            ));
            return Some(if name.is_empty() {
                "(unnamed)".into()
            } else {
                name
            });
        }
    }
    None
}
fn count_machines(machine: &Machine) -> (usize, usize) {
    let artboard = machine.0.with_instance(|m| m.artboard()).upgrade().unwrap();
    let (nested, lists) = artboard.with_artboard(|a| {
        (
            a.nested_artboards().to_vec(),
            a.artboard_component_lists().to_vec(),
        )
    });
    let mut counts = (
        1,
        machine
            .0
            .with_instance(StateMachineInstance::hit_components_count),
    );
    for nested in nested {
        let animations = nested
            .with_downcast::<NestedArtboard, _>(|n| n.nested_animations().to_vec())
            .unwrap();
        for animation in animations {
            if let Some(child) = animation
                .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
                .flatten()
            {
                let c = count_machines(&Machine(child));
                counts.0 += c.0;
                counts.1 += c.1;
            }
        }
    }
    for list in lists {
        let indices = list
            .with_downcast_mut::<ArtboardComponentList, _>(|l| l.ordered_list_indices().to_vec())
            .unwrap();
        for index in indices {
            if let Some(child) = list
                .with_downcast::<ArtboardComponentList, _>(|l| l.state_machine_instance(index))
                .flatten()
            {
                let c = count_machines(&Machine(child));
                counts.0 += c.0;
                counts.1 += c.1;
            }
        }
    }
    counts
}
fn time_moves(scene: &BenchInstance) -> f64 {
    use std::time::{Duration, Instant};
    let (width, height) = scene.artboard.with_artboard(|a| (a.width(), a.height()));
    let mut points = Vec::new();
    for y in 0..24 {
        for x in 0..24 {
            let px = (x as f32 + 0.5) * width / 24.0;
            let py = (y as f32 + 0.5) * height / 24.0;
            points.push(Vec2D::new(if y % 2 == 0 { px } else { width - px }, py));
        }
    }
    let mut best: f64 = 1e30;
    for _ in 0..7 {
        let mut total = Duration::ZERO;
        let mut events = 0;
        for _ in 0..8 {
            for chunk in points.chunks(64) {
                let t = Instant::now();
                for p in chunk {
                    scene.machine.move_to(*p);
                }
                total += t.elapsed();
                events += chunk.len();
                scene.machine.advance();
            }
        }
        best = best.min(total.as_secs_f64() * 1e9 / events as f64);
    }
    best
}
fn time_clicks(scene: &BenchInstance) -> f64 {
    use std::time::{Duration, Instant};
    let center = scene
        .artboard
        .with_artboard(|a| Vec2D::new(a.width() * 0.5, a.height() * 0.5));
    let mut best: f64 = 1e30;
    for _ in 0..7 {
        let mut total = Duration::ZERO;
        for i in 0..512 {
            let t = Instant::now();
            scene.machine.down(center);
            scene.machine.up(center);
            total += t.elapsed();
            if i % 32 == 31 {
                scene.machine.advance();
            }
        }
        best = best.min(total.as_secs_f64() * 1e9 / 1024.0);
    }
    best
}
fn time_init(file: &RuntimeFileHandle, name: Option<&str>) -> f64 {
    let mut best: f64 = 1e30;
    for _ in 0..5 {
        let t = std::time::Instant::now();
        for _ in 0..64 {
            let artboard = file
                .with_file(|f| match name {
                    Some(n) => f.artboard_named(n),
                    None => f.artboard_default(),
                })
                .unwrap();
            let _machine = artboard.state_machine_at(0).unwrap();
        }
        best = best.min(t.elapsed().as_secs_f64() * 1e6 / 64.0);
    }
    best
}
fn time_advance(scene: &BenchInstance) -> f64 {
    let mut best: f64 = 1e30;
    for _ in 0..7 {
        let t = std::time::Instant::now();
        for _ in 0..256 {
            scene.machine.0.advance_and_apply(1.0 / 60.0);
        }
        best = best.min(t.elapsed().as_secs_f64() * 1e6 / 256.0);
    }
    best
}
#[test]
#[ignore = "upstream hidden .capture_bench: explicit release benchmark only"]
fn capture_listener_benchmark() {
    let _guard = isolated();
    println!("feature compiled in: yes");
    for (path, name, listener) in [
        ("capture_listeners.riv", Some("Host"), Some("Slot B Move")),
        ("layoutstest_8-planets-grid.riv", None, None),
        ("inventory_demo_test_v2.riv", None, None),
        ("magic_alley_db_reduced_export.riv", None, None),
    ] {
        let Some(file) = bench_load(path) else {
            println!("{path}: could not load");
            continue;
        };
        let counts = {
            let scene = BenchInstance::new(&file, name).unwrap();
            count_machines(&scene.machine)
        };
        println!(
            "{path} ({} machines, {} hit components)",
            counts.0, counts.1
        );
        println!("init {} us", time_init(&file, name));
        {
            let scene = BenchInstance::new(&file, name).unwrap();
            println!("advance {} us/frame", time_advance(&scene));
            println!(
                "A no capture anywhere move {} ns click {} ns",
                time_moves(&scene),
                time_clicks(&scene)
            );
        }
        {
            let other = bench_load("capture_listeners.riv").expect("other capture file");
            first_pointer_capture(&other, Some("Host"), Some("Slot B Move"));
            let _elsewhere = BenchInstance::new(&other, Some("Host")).unwrap();
            assert_eq!(StateMachineInstance::live_capture_machines(), 1);
            let scene = BenchInstance::new(&file, name).unwrap();
            println!(
                "B capture elsewhere move {} ns click {} ns",
                time_moves(&scene),
                time_clicks(&scene)
            );
        }
        {
            let captured = bench_load(path).expect("captured benchmark file");
            if let Some(listener) = first_pointer_capture(&captured, name, listener) {
                println!("init with capture {} us", time_init(&captured, name));
                let scene = BenchInstance::new(&captured, name).unwrap();
                println!(
                    "C capture in tree move {} ns click {} ns ({listener})",
                    time_moves(&scene),
                    time_clicks(&scene)
                );
            } else {
                println!("C capture in tree (no pointer listener)");
            }
        }
    }
}
