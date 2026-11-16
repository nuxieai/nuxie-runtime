//! Nuxie: `HitGate`, one row a path from a state machine into a hit. Each
//! row hands the gate's hit the same event open and shut: open, the hit
//! hears it as asked; shut, as inside a copy flying in place, the hit hears
//! nothing, or only a miss or an exit, so what it is already in ends. The
//! rows are every `HitComponent` method a machine calls on a hit
//! (`hit_components`, the capture hits and the scroll latch all hold gates);
//! housekeeping that delivers no input (preparing an event, a pointer's
//! enabling, letting a drag or scroll go) passes through either way.
use super::*;
use crate::mechanical_port::source::{
    animation::entry_state::EntryState,
    artboard::{ArtboardInstance, RuntimeArtboardInstanceHandle},
    core::CoreArena,
    scroll_event::ScrollPhase,
};
use std::cell::RefCell;

/// A hit that says yes to everything and writes down what it is handed.
struct Spy {
    component: CoreHandle,
    heard: Rc<RefCell<Vec<String>>>,
}

impl Spy {
    fn note(&self, what: impl Into<String>) {
        self.heard.borrow_mut().push(what.into());
    }
}

impl HitComponent for Spy {
    fn has_capture_listeners(&self) -> bool {
        true
    }
    fn blocks_capture(&self, _: Vec2D) -> bool {
        self.note("blocks_capture");
        true
    }
    fn capture_pointer(
        &self,
        _: Vec2D,
        hit_type: ListenerType,
        _: i32,
        _: f32,
        _: PointerButton,
    ) -> HitResult {
        self.note(format!("capture {hit_type:?}"));
        HitResult::Hit
    }
    fn component(&self) -> RuntimeDrawableOccurrence {
        RuntimeDrawableOccurrence::Authored(self.component.clone())
    }
    fn process_event(
        &self,
        _: &mut StateMachineInstance,
        _: Vec2D,
        hit_type: ListenerType,
        can_hit: bool,
        _: f32,
        _: i32,
        _: PointerButton,
    ) -> HitResult {
        self.note(format!("event {hit_type:?} can_hit {can_hit}"));
        if can_hit {
            HitResult::Hit
        } else {
            HitResult::None
        }
    }
    fn process_gamepad_invocation(
        &self,
        _: &ListenerInvocation,
        _: Option<&CoreHandle>,
    ) -> HitResult {
        self.note("gamepad");
        HitResult::Hit
    }
    fn prepare_event(&self, _: Vec2D, _: ListenerType, _: i32) {
        self.note("prepare");
    }
    fn hit_test(&self, _: Vec2D) -> bool {
        self.note("hit_test");
        true
    }
    fn hit_test_bounded(&self, _: Vec2D) -> bool {
        self.note("hit_test_bounded");
        true
    }
    fn enable_pointer_events(&self, _: i32) {
        self.note("enable");
    }
    fn disable_pointer_events(&self, _: i32) {
        self.note("disable");
    }
    fn wants_scroll(&self, _: Vec2D, _: &ScrollEvent) -> bool {
        self.note("wants_scroll");
        true
    }
    fn scroll_gesture_active(&self) -> bool {
        self.note("scroll_gesture_active");
        true
    }
    fn cancel_scroll(&self) {
        self.note("cancel_scroll");
    }
    fn has_scroll_target(&self, _: Vec2D) -> bool {
        self.note("has_scroll_target");
        true
    }
    fn occludes_scroll(&self, _: Vec2D) -> bool {
        self.note("occludes_scroll");
        true
    }
    fn listens_to_button_at(&self, _: Vec2D, _: PointerButton) -> bool {
        self.note("listens_to_button_at");
        true
    }
    fn occludes_pointer(&self, _: Vec2D) -> bool {
        self.note("occludes_pointer");
        true
    }
    fn holds_drag(&self, _: i32) -> bool {
        self.note("holds_drag");
        true
    }
    fn yield_drag(&self, _: i32, _: Vec2D, _: f32) {
        self.note("yield_drag");
    }
    fn process_scroll(&self, _: Vec2D, _: &ScrollEvent, _: f32, _: i32) -> HitResult {
        self.note("scroll");
        HitResult::Hit
    }
}

/// A gate around a spy, open and shut, and what each spy heard.
struct Row {
    _arena: CoreArena,
    open: Rc<HitGate>,
    shut: Rc<HitGate>,
    open_heard: Rc<RefCell<Vec<String>>>,
    shut_heard: Rc<RefCell<Vec<String>>>,
}

fn row() -> Row {
    let arena = CoreArena::default();
    let gate = |lifted: bool| {
        let heard = Rc::new(RefCell::new(Vec::new()));
        let gate = HitGate::wrap(Spy {
            component: arena.insert(EntryState::default()),
            heard: heard.clone(),
        });
        gate.lifted.set(lifted);
        (gate, heard)
    };
    let (open, open_heard) = gate(false);
    let (shut, shut_heard) = gate(true);
    Row {
        _arena: arena,
        open,
        shut,
        open_heard,
        shut_heard,
    }
}

impl Row {
    /// Runs `call` on both gates; returns what each answered and what each
    /// spy heard.
    fn run<T>(&self, call: impl Fn(&HitGate) -> T) -> ((T, Vec<String>), (T, Vec<String>)) {
        let open = call(&self.open);
        let shut = call(&self.shut);
        (
            (open, self.open_heard.borrow().clone()),
            (shut, self.shut_heard.borrow().clone()),
        )
    }
}

fn at() -> Vec2D {
    Vec2D::new(1.0, 1.0)
}

fn wheel() -> ScrollEvent {
    ScrollEvent {
        delta: Vec2D::new(0.0, -30.0),
        phase: ScrollPhase::Update,
        precise: false,
    }
}

fn heard(what: &[&str]) -> Vec<String> {
    what.iter().map(|w| w.to_string()).collect()
}

/// The main pointer loop: a press reaches the hit open; shut, the hit hears
/// it as a miss, so a hover or press it is in ends.
#[test]
fn hit_gate_main_loop_hands_a_lifted_hit_a_miss() {
    let row = row();
    let arena = CoreArena::default();
    let definition = arena.insert(StateMachine::default());
    let artboard = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
    let machine = StateMachineInstance::new(definition, artboard.downgrade());
    let (open, shut) = row.run(|gate| {
        machine.with_instance_mut(|machine| {
            gate.process_event(
                machine,
                at(),
                ListenerType::Down,
                true,
                0.0,
                1,
                PointerButton::Primary,
            )
        })
    });
    assert_eq!(open, (HitResult::Hit, heard(&["event Down can_hit true"])));
    assert_eq!(
        shut,
        (HitResult::None, heard(&["event Down can_hit false"]))
    );
}

/// The main loop reaches a drawable's listeners directly
/// (`as_hit_drawable`), with `takes_input` in its can-hit.
#[test]
fn hit_gate_drawable_listeners_read_takes_input() {
    let row = row();
    let (open, shut) = row.run(|gate| gate.takes_input());
    assert_eq!(open, (true, vec![]));
    assert_eq!(shut, (false, vec![]));
}

/// Capture: a lifted hit's capture listeners hear only an exit, as an
/// occluded one does, a nested artboard's or list row's machine included.
#[test]
fn hit_gate_capture_hands_a_lifted_hit_an_exit() {
    let row = row();
    let (open, shut) = row
        .run(|gate| gate.capture_pointer(at(), ListenerType::Down, 1, 0.0, PointerButton::Primary));
    assert_eq!(open.1, heard(&["capture Down"]));
    assert_eq!(shut.1, heard(&["capture Exit"]));
}

/// Capture's occluders: a lifted hit blocks no capture behind it.
#[test]
fn hit_gate_capture_occluders_skip_a_lifted_hit() {
    let row = row();
    let (open, shut) = row.run(|gate| gate.blocks_capture(at()));
    assert_eq!(open, (true, heard(&["blocks_capture"])));
    assert_eq!(shut, (false, vec![]));
}

/// A host's hit test (whether a press is this scene's), bounded or not.
#[test]
fn hit_gate_hit_tests_miss_a_lifted_hit() {
    let row = row();
    let (open, shut) = row.run(|gate| (gate.hit_test(at()), gate.hit_test_bounded(at())));
    assert_eq!(
        open,
        ((true, true), heard(&["hit_test", "hit_test_bounded"]))
    );
    assert_eq!(shut, ((false, false), vec![]));
}

/// Pointer occlusion: a lifted hit hides nothing behind it.
#[test]
fn hit_gate_pointer_occlusion_skips_a_lifted_hit() {
    let row = row();
    let (open, shut) = row.run(|gate| gate.occludes_pointer(at()));
    assert_eq!(open, (true, heard(&["occludes_pointer"])));
    assert_eq!(shut, (false, vec![]));
}

/// Which button a press there would be heard on.
#[test]
fn hit_gate_button_listening_skips_a_lifted_hit() {
    let row = row();
    let (open, shut) = row.run(|gate| gate.listens_to_button_at(at(), PointerButton::Primary));
    assert_eq!(open, (true, heard(&["listens_to_button_at"])));
    assert_eq!(shut, (false, vec![]));
}

/// Scroll: wanting a wheel, a target there, occluding one, and the scroll
/// itself.
#[test]
fn hit_gate_scroll_reaches_no_lifted_hit() {
    let row = row();
    let event = wheel();
    let (open, shut) = row.run(|gate| {
        (
            gate.wants_scroll(at(), &event),
            gate.has_scroll_target(at()),
            gate.occludes_scroll(at()),
            gate.process_scroll(at(), &event, 0.0, 1),
        )
    });
    assert_eq!(
        open,
        (
            (true, true, true, HitResult::Hit),
            heard(&[
                "wants_scroll",
                "has_scroll_target",
                "occludes_scroll",
                "scroll"
            ])
        )
    );
    assert_eq!(shut, ((false, false, false, HitResult::None), vec![]));
}

/// The scroll latch: a gesture latched on a hit whose copy then lifts lets
/// go of it, as its gesture reads inactive; cancelling still reaches it.
#[test]
fn hit_gate_scroll_latch_lets_go_of_a_lifted_hit() {
    let row = row();
    let (open, shut) = row.run(|gate| {
        let active = gate.scroll_gesture_active();
        gate.cancel_scroll();
        active
    });
    assert_eq!(
        open,
        (true, heard(&["scroll_gesture_active", "cancel_scroll"]))
    );
    assert_eq!(shut, (false, heard(&["cancel_scroll"])));
}

/// Gamepad listeners.
#[test]
fn hit_gate_gamepad_reaches_no_lifted_hit() {
    let row = row();
    let invocation = ListenerInvocation::none();
    let (open, shut) = row.run(|gate| gate.process_gamepad_invocation(&invocation, None));
    assert_eq!(open, (HitResult::Hit, heard(&["gamepad"])));
    assert_eq!(shut, (HitResult::None, vec![]));
}

/// Drags: a lifted hit holds no drag a host might take; letting one go
/// still reaches it.
#[test]
fn hit_gate_drags_held_in_a_lifted_hit_are_not_held() {
    let row = row();
    let (open, shut) = row.run(|gate| {
        let holds = gate.holds_drag(1);
        gate.yield_drag(1, at(), 0.0);
        holds
    });
    assert_eq!(open, (true, heard(&["holds_drag", "yield_drag"])));
    assert_eq!(shut, (false, heard(&["yield_drag"])));
}

/// Housekeeping that hands no input passes through either way: preparing
/// an event (so its miss is heard), a pointer's enabling, capture's census.
#[test]
fn hit_gate_housekeeping_passes_through() {
    let row = row();
    let (open, shut) = row.run(|gate| {
        gate.prepare_event(at(), ListenerType::Move, 1);
        gate.enable_pointer_events(1);
        gate.disable_pointer_events(1);
        gate.has_capture_listeners()
    });
    let all = heard(&["prepare", "enable", "disable"]);
    assert_eq!(open, (true, all.clone()));
    assert_eq!(shut, (true, all));
}
