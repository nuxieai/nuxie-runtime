//! Literal ports of upstream `scripting_drawable_scroll_test.cpp` (de3e8609).
#![cfg(all(
    feature = "luau",
    feature = "compiler",
    feature = "upstream-test-seams"
))]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    NoopScriptHost, RuntimeScriptingVmHandle,
    source::{
        advance_flags::AdvanceFlags,
        animation::state_machine_instance::HitComponent,
        assets::script_asset::ScriptAsset,
        core::{CoreArena, CoreHandle},
        hit_result::HitResult,
        math::{mat2d::Mat2D, vec2d::Vec2D},
        scripted::scripted_drawable::{HitScriptedDrawable, ScriptedDrawable},
        scroll_event::{ScrollEvent, ScrollPhase},
    },
};
use nuxie_scripting::vm::{ScriptProgram, ScriptVm};
mod support;
use support::compile_source;

const WANTS_POINTER_SCROLL: u32 = 1 << 21;
const SCROLL_SCRIPT: &str = r#"type MyMap = {}
local claim = true
local last = ''

function init(self: MyMap, context: Context): boolean
  return true
end

function pointerScroll(self: MyMap, event: ScrollEvent)
  last = string.format('%d %g,%g %g,%g %s %s %g',
    event.id,
    event.position.x, event.position.y,
    event.delta.x, event.delta.y,
    event.phase,
    tostring(event.precise),
    event.timeStamp)
  if claim then
    event:hit()
  end
end

function getLast(): string
  return last
end

function setClaim(value: boolean)
  claim = value
end

return function(): Node<MyMap>
  return {
    init = init,
    pointerScroll = pointerScroll,
  }
end
"#;

struct Fixture {
    _vm: RuntimeScriptingVmHandle,
    _arena: CoreArena,
    program: ScriptProgram,
    drawable: CoreHandle,
    hit: HitScriptedDrawable,
}
impl Fixture {
    fn new(methods: u32) -> Self {
        let mut payload = vec![0];
        payload.extend(compile_source(SCROLL_SCRIPT).unwrap());
        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let program = vm
            .register_protocol_script_with_factory("scroll", &payload, &mut factory)
            .unwrap();
        let mut instance = vm
            .instantiate_registered_script_with_context(&program, None, Vec::new())
            .unwrap();
        assert!(instance.call_init(&mut NoopScriptHost).unwrap());
        let vm = RuntimeScriptingVmHandle::new(Box::new(vm));
        // Like upstream's dirt-stubbing drawable, retain a standalone owner;
        // no artboard graph participates in these dispatch/latching assertions.
        let arena = CoreArena::default();
        let drawable = arena.insert(ScriptedDrawable::default());
        let asset = drawable.insert_sibling(ScriptAsset::default()).unwrap();
        drawable
            .with_mut(|owner| {
                let object = owner.as_scripted_drawable_mut().unwrap();
                object.scripted.set_asset(drawable.clone(), Some(asset));
                object
                    .scripted
                    .install_script_instance(instance, vm.clone());
                object.scripted.set_implemented_methods(methods);
            })
            .unwrap();
        let hit = HitScriptedDrawable::new(drawable.clone());
        Self {
            _vm: vm,
            _arena: arena,
            program,
            drawable,
            hit,
        }
    }
    fn last(&self) -> String {
        self.program
            .upstream_test_module_string_getter("getLast")
            .unwrap()
    }
    fn claim(&self, value: bool) {
        self.program
            .upstream_test_module_bool_setter("setClaim", value)
            .unwrap();
    }
    fn advance(&self, elapsed: f32) {
        ScriptedDrawable::advance_occurrence(
            &self.drawable,
            elapsed,
            AdvanceFlags(
                AdvanceFlags::ANIMATE.0
                    | AdvanceFlags::NEW_FRAME.0
                    | AdvanceFlags::ADVANCE_NESTED.0,
            ),
        );
    }
}
fn trackpad(phase: ScrollPhase, dx: f32, dy: f32) -> ScrollEvent {
    ScrollEvent {
        delta: Vec2D::new(dx, dy),
        phase,
        precise: true,
    }
}
fn wheel(dy: f32) -> ScrollEvent {
    ScrollEvent {
        delta: Vec2D::new(0.0, dy),
        ..ScrollEvent::default()
    }
}

#[test]
fn pointer_scroll_receives_the_event_in_the_drawables_space() {
    let fixture = Fixture::new(WANTS_POINTER_SCROLL);
    fixture
        .drawable
        .with_mut(|owner| {
            owner
                .as_world_transform_component_mut()
                .unwrap()
                .set_world_transform(Mat2D::new(2.0, 0.0, 0.0, 2.0, 10.0, 20.0));
        })
        .unwrap();
    let result = fixture.hit.process_scroll(
        Vec2D::new(30.0, 40.0),
        &trackpad(ScrollPhase::Update, 4.0, -6.0),
        1.5,
        3,
    );
    assert_eq!(result, HitResult::HitOpaque);
    assert_eq!(fixture.last(), "3 10,10 2,-3 update true 1.5");
    fixture
        .hit
        .process_scroll(Vec2D::new(30.0, 40.0), &wheel(-10.0), 2.0, 0);
    assert_eq!(fixture.last(), "0 10,10 0,-5 update false 2");
}

#[test]
fn a_claimed_trackpad_gesture_latches_until_it_ends_or_goes_quiet() {
    let fixture = Fixture::new(WANTS_POINTER_SCROLL);
    let hit = &fixture.hit;
    let at = Vec2D::new(5.0, 5.0);
    hit.process_scroll(at, &trackpad(ScrollPhase::Begin, 0.0, 0.0), 0.0, 0);
    assert!(hit.scroll_gesture_active());
    hit.process_scroll(at, &trackpad(ScrollPhase::Update, 0.0, -8.0), 0.0, 0);
    assert!(hit.scroll_gesture_active());
    hit.process_scroll(at, &trackpad(ScrollPhase::End, 0.0, 0.0), 0.0, 0);
    assert!(!hit.scroll_gesture_active());
    hit.process_scroll(at, &trackpad(ScrollPhase::Momentum, 0.0, -4.0), 0.0, 0);
    assert!(hit.scroll_gesture_active());
    fixture.advance(0.05);
    assert!(hit.scroll_gesture_active());
    hit.process_scroll(at, &trackpad(ScrollPhase::Momentum, 0.0, -2.0), 0.0, 0);
    fixture.advance(0.05);
    assert!(hit.scroll_gesture_active());
    fixture.advance(0.06);
    assert!(!hit.scroll_gesture_active());
    hit.process_scroll(at, &trackpad(ScrollPhase::Momentum, 0.0, -1.0), 0.0, 0);
    hit.process_scroll(at, &trackpad(ScrollPhase::InertiaCancel, 0.0, 0.0), 0.0, 0);
    assert!(!hit.scroll_gesture_active());
    hit.process_scroll(at, &trackpad(ScrollPhase::Begin, 0.0, 0.0), 0.0, 0);
    hit.cancel_scroll();
    assert!(!hit.scroll_gesture_active());
    hit.process_scroll(at, &wheel(-10.0), 0.0, 0);
    assert!(!hit.scroll_gesture_active());
}

#[test]
fn a_script_that_does_not_claim_the_scroll_lets_it_pass() {
    let fixture = Fixture::new(WANTS_POINTER_SCROLL);
    fixture.claim(false);
    let hit = &fixture.hit;
    let at = Vec2D::new(1.0, 2.0);
    assert_eq!(
        hit.process_scroll(at, &trackpad(ScrollPhase::Begin, 0.0, 0.0), 0.0, 0),
        HitResult::None
    );
    assert_eq!(fixture.last(), "0 1,2 0,0 begin true 0");
    assert!(!hit.scroll_gesture_active());
    fixture.claim(true);
    hit.process_scroll(at, &trackpad(ScrollPhase::Begin, 0.0, 0.0), 0.0, 0);
    fixture.claim(false);
    assert_eq!(
        hit.process_scroll(at, &trackpad(ScrollPhase::Update, 0.0, -4.0), 0.0, 0),
        HitResult::None
    );
    assert!(hit.scroll_gesture_active());
}

#[test]
fn a_script_without_pointer_scroll_never_wants_scroll() {
    let fixture = Fixture::new(1 << 3);
    assert!(
        !fixture
            .hit
            .wants_scroll(Vec2D::new(1.0, 1.0), &wheel(-10.0))
    );
    assert!(!fixture.hit.has_scroll_target(Vec2D::new(1.0, 1.0)));
}

#[test]
fn scroll_event_new_defaults_to_a_wheel_update() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    let code = compile_source("local ev = ScrollEvent.new(3, Vector.xy(1, 2), Vector.xy(0, -4))\nreturn string.format('%d %g,%g %g,%g %s %s %g', ev.id,\n  ev.position.x, ev.position.y, ev.delta.x, ev.delta.y, ev.phase,\n  tostring(ev.precise), ev.timeStamp)").unwrap();
    assert_eq!(
        vm.eval_bytecode::<String>("scroll-default", &code).unwrap(),
        "3 1,2 0,-4 update false 0"
    );
}

#[test]
fn scroll_event_new_keeps_every_field_it_is_given() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    let code = compile_source("local ev = ScrollEvent.new(1, Vector.xy(5, 6),\n  Vector.xy(7, 8), 'momentum', true, 2.5)\nreturn string.format('%s %s %g', ev.phase,\n  tostring(ev.precise), ev.timeStamp)").unwrap();
    assert_eq!(
        vm.eval_bytecode::<String>("scroll-full", &code).unwrap(),
        "momentum true 2.5"
    );
}

#[test]
fn scroll_phase_options_follow_upstream_c_string_boundaries() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    let code = compile_source(
        r#"
local position = Vector.xy(0, 0)
local suffix = string.char(0, 255)
assert(ScrollEvent.new(0, position, position, 'update' .. suffix).phase == 'update')
assert(ScrollEvent.new(0, position, position, 'momentum' .. suffix).phase == 'momentum')
return true
"#,
    )
    .unwrap();
    assert!(
        vm.eval_bytecode::<bool>("scroll-c-string-phase", &code)
            .unwrap()
    );
}
