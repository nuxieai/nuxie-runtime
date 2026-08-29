//! One-for-one native owner ports of pinned
//! `tests/unit_tests/runtime/scroll_velocity_test.cpp`.

use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard},
};

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::{RuntimeStateMachineInstanceHandle, StateMachineInstance},
    constraints::scrolling::{scroll_constraint::ScrollConstraint, scroll_physics},
    math::vec2d::Vec2D,
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

static CLOCK_LOCK: Mutex<()> = Mutex::new(());

struct DeterministicClock {
    previous: bool,
    _lock: MutexGuard<'static, ()>,
}

impl DeterministicClock {
    fn new() -> Self {
        let lock = CLOCK_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let previous = File::deterministic_mode();
        File::set_deterministic_mode(true);
        Self {
            previous,
            _lock: lock,
        }
    }
}

impl Drop for DeterministicClock {
    fn drop(&mut self) {
        File::set_deterministic_mode(self.previous);
    }
}

struct Fixture {
    // Drop the machine before its artboard and defining File.
    state_machine: Option<RuntimeStateMachineInstanceHandle>,
    _artboard: RuntimeArtboardInstanceHandle,
    scroll: CoreHandle,
    _file: RuntimeFileHandle,
    _clock: DeterministicClock,
}

fn fixture(name: &str, with_state_machine: bool) -> Fixture {
    // Preserve the previous Rust test's explicit pointer timestamps through
    // the actual source File::deterministicMode/ScrollPhysics clock branch.
    let clock = DeterministicClock::new();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("explicit retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(
        &pinned_fixture(name),
        retained,
        Some(&mut result),
        None,
        None,
    )
    .unwrap_or_else(|| panic!("{name} imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    let source = file.with_file(File::artboard).expect("source artboard");
    let artboard = Artboard::instance_from_handle(&source).expect("artboard instance");
    let state_machine = with_state_machine.then(|| {
        let definition = source
            .with_downcast::<Artboard, _>(|artboard| {
                artboard.state_machine_named("State Machine 1")
            })
            .flatten()
            .expect("State Machine 1 definition");
        StateMachineInstance::new(definition, artboard.downgrade())
    });
    let scroll = artboard
        .with_artboard(|artboard| {
            artboard
                .find_all_handles::<ScrollConstraint>()
                .first()
                .cloned()
        })
        .expect("fixture has a ScrollConstraint");
    artboard.advance_default(0.0);
    Fixture {
        state_machine,
        _artboard: artboard,
        scroll,
        _file: file,
        _clock: clock,
    }
}

fn velocity_x(scroll: &CoreHandle) -> f32 {
    scroll
        .with_downcast::<ScrollConstraint, _>(ScrollConstraint::velocity_x)
        .expect("live ScrollConstraint")
}

fn velocity_y(scroll: &CoreHandle) -> f32 {
    scroll
        .with_downcast::<ScrollConstraint, _>(ScrollConstraint::velocity_y)
        .expect("live ScrollConstraint")
}

fn scroll_active(scroll: &CoreHandle) -> bool {
    scroll
        .with_downcast::<ScrollConstraint, _>(ScrollConstraint::scroll_active)
        .expect("live ScrollConstraint")
}

fn physics_running(scroll: &CoreHandle) -> bool {
    let physics = scroll
        .with_downcast::<ScrollConstraint, _>(ScrollConstraint::physics)
        .flatten()
        .expect("ScrollConstraint has physics");
    physics
        .with(|physics| {
            scroll_physics::from_core(physics)
                .expect("retained native ScrollPhysics")
                .is_running()
        })
        .expect("live ScrollPhysics")
}

#[test]
fn scroll_constraint_velocity_and_scroll_active_during_drag() {
    let fixture = fixture("layout/layout_scroll_vertical.riv", true);
    let state_machine = fixture.state_machine.as_ref().expect("state machine");

    assert_eq!(velocity_x(&fixture.scroll), 0.0);
    assert_eq!(velocity_y(&fixture.scroll), 0.0);
    assert!(!scroll_active(&fixture.scroll));

    state_machine.with_instance_mut(|machine| {
        machine.pointer_move(Vec2D::new(50.0, 250.0), 0.0, 0);
        machine.pointer_down(Vec2D::new(50.0, 250.0), 0);
    });
    state_machine.advance_and_apply(0.1);

    assert!(scroll_active(&fixture.scroll));
    assert_eq!(velocity_y(&fixture.scroll), 0.0);

    state_machine.with_instance_mut(|machine| {
        machine.pointer_move(Vec2D::new(50.0, 50.0), 1.0, 0);
    });
    state_machine.advance_and_apply(0.0);
    assert_ne!(velocity_y(&fixture.scroll), 0.0);
    assert!(scroll_active(&fixture.scroll));

    state_machine.advance_and_apply(0.1);
    assert!(scroll_active(&fixture.scroll));

    state_machine.with_instance_mut(|machine| {
        machine.pointer_up(Vec2D::new(50.0, 50.0), 0);
    });
    assert!(physics_running(&fixture.scroll));
    assert!(scroll_active(&fixture.scroll));
}

#[test]
fn scroll_constraint_velocity_resets_after_physics_settles() {
    let fixture = fixture("layout/layout_scroll_vertical.riv", true);
    let state_machine = fixture.state_machine.as_ref().expect("state machine");

    state_machine.with_instance_mut(|machine| {
        machine.pointer_move(Vec2D::new(50.0, 250.0), 0.0, 0);
        machine.pointer_down(Vec2D::new(50.0, 250.0), 0);
    });
    state_machine.advance_and_apply(0.1);
    state_machine.with_instance_mut(|machine| {
        machine.pointer_move(Vec2D::new(50.0, 50.0), 1.0, 0);
    });
    state_machine.advance_and_apply(0.0);
    state_machine.with_instance_mut(|machine| {
        machine.pointer_up(Vec2D::new(50.0, 50.0), 0);
    });

    assert!(physics_running(&fixture.scroll));
    assert!(scroll_active(&fixture.scroll));

    for _ in 0..600 {
        state_machine.advance_and_apply(0.016);
        if !physics_running(&fixture.scroll) {
            break;
        }
    }

    assert!(!physics_running(&fixture.scroll));
    assert!(!scroll_active(&fixture.scroll));
    assert_eq!(velocity_x(&fixture.scroll), 0.0);
    assert_eq!(velocity_y(&fixture.scroll), 0.0);
}

#[test]
fn scroll_constraint_horizontal_velocity() {
    let fixture = fixture("layout/layout_scroll_horizontal.riv", true);
    let state_machine = fixture.state_machine.as_ref().expect("state machine");

    state_machine.with_instance_mut(|machine| {
        machine.pointer_move(Vec2D::new(250.0, 50.0), 0.0, 0);
        machine.pointer_down(Vec2D::new(250.0, 50.0), 0);
    });
    state_machine.advance_and_apply(0.1);
    state_machine.with_instance_mut(|machine| {
        machine.pointer_move(Vec2D::new(50.0, 50.0), 1.0, 0);
    });
    state_machine.advance_and_apply(0.0);

    assert_ne!(velocity_x(&fixture.scroll), 0.0);
    assert_eq!(velocity_y(&fixture.scroll), 0.0);
    assert!(scroll_active(&fixture.scroll));
    state_machine.with_instance_mut(|machine| {
        machine.pointer_up(Vec2D::new(50.0, 50.0), 0);
    });
}

#[test]
fn scroll_constraint_scroll_active_false_when_idle() {
    let fixture = fixture("layout/layout_scroll_vertical.riv", false);
    assert!(
        fixture
            .scroll
            .with_downcast_mut::<ScrollConstraint, _>(|scroll| {
                scroll.set_scroll_percent_y(0.5);
            })
            .is_some()
    );

    assert!(!scroll_active(&fixture.scroll));
    assert_eq!(velocity_x(&fixture.scroll), 0.0);
    assert_eq!(velocity_y(&fixture.scroll), 0.0);
}
