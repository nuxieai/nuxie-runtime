//! Direct ports of the focus-bound cases added to
//! `tests/unit_tests/runtime/focus_test.cpp` at upstream b36aa3d0.

use std::{cell::RefCell, rc::Rc};

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    input::{
        focus_manager::{FocusManager, RuntimeFocusManagerHandle},
        focus_node::FocusNode,
        focusable::{Focusable, Key, KeyModifiers},
    },
    semantic::semantic_snapshot::Bounds,
};
use nuxie_runtime::{File, ImportResult, RuntimeFactoryHandle};

struct MockBoundedFocusable {
    has_bounds: bool,
    live_bounds: Bounds,
}

impl Default for MockBoundedFocusable {
    fn default() -> Self {
        Self {
            has_bounds: true,
            live_bounds: Bounds {
                min_x: 10.0,
                min_y: 20.0,
                max_x: 110.0,
                max_y: 220.0,
            },
        }
    }
}

impl Focusable for MockBoundedFocusable {
    fn key_input(&mut self, _: Key, _: KeyModifiers, _: bool, _: bool) -> bool {
        false
    }

    fn text_input(&mut self, _: &str) -> bool {
        false
    }

    fn focused(&mut self) {}

    fn blurred(&mut self) {}

    fn world_bounds(&self) -> Option<Bounds> {
        self.has_bounds.then_some(self.live_bounds)
    }
}

#[test]
fn primary_focus_bounds_prefers_live_focusable_bounds_over_cached() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let focusable = Rc::new(RefCell::new(MockBoundedFocusable::default()));
    let node = FocusNode::new(Some(focusable.clone()));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, node.clone(), None);
        manager.set_focus(node.clone());
    });

    // Stale bounds cached on the node by a previous update pass.
    node.borrow_mut().world_bounds = Bounds {
        min_x: 1.0,
        min_y: 2.0,
        max_x: 3.0,
        max_y: 4.0,
    };

    let bounds = manager
        .with_focus_manager(FocusManager::primary_focus_bounds)
        .expect("focused node has bounds");
    assert_eq!(bounds.min_x, 10.0);
    assert_eq!(bounds.min_y, 20.0);
    assert_eq!(bounds.max_x, 110.0);
    assert_eq!(bounds.max_y, 220.0);

    // When the focusable cannot compute, the cached bounds remain the
    // fallback.
    focusable.borrow_mut().has_bounds = false;
    let bounds = manager
        .with_focus_manager(FocusManager::primary_focus_bounds)
        .expect("cached bounds remain available");
    assert_eq!(bounds.min_x, 1.0);
    assert_eq!(bounds.max_y, 4.0);

    manager.with_focus_manager_mut(FocusManager::clear_focus);
    assert!(
        manager
            .with_focus_manager(FocusManager::primary_focus_bounds)
            .is_none()
    );
}

#[test]
fn primary_focus_bounds_uses_cached_bounds_without_a_focusable() {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = FocusNode::new(None);
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(None, node.clone(), None);
        manager.set_focus(node.clone());
    });

    assert!(
        manager
            .with_focus_manager(FocusManager::primary_focus_bounds)
            .is_none()
    );

    node.borrow_mut().world_bounds = Bounds {
        min_x: 5.0,
        min_y: 6.0,
        max_x: 7.0,
        max_y: 8.0,
    };
    let bounds = manager
        .with_focus_manager(FocusManager::primary_focus_bounds)
        .expect("host-pushed cached bounds");
    assert_eq!(bounds.min_x, 5.0);
    assert_eq!(bounds.max_y, 8.0);
}

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.5,
        "expected {actual} to be within 0.5 of {expected}",
    );
}

#[test]
fn focus_bounds_track_a_nested_artboard_host_that_moves() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/sync/focus_bounds_moving_host.riv");
    let bytes = std::fs::read(&fixture).unwrap_or_else(|error| {
        panic!(
            "read pinned fixture {} (run make fixtures): {error}",
            fixture.display()
        )
    });
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes, retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("focus fixture imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    let artboard = file
        .with_file(|file| file.artboard_default())
        .expect("default artboard");
    let state_machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine 0");

    // Settle without consuming time so the host is still at its authored
    // position when the first bounds are read.
    state_machine.advance_and_apply(0.0);
    state_machine.advance_and_apply(0.0);

    let focus_manager = state_machine.with_instance(|machine| machine.focus_manager());
    assert!(focus_manager.with_focus_manager(|manager| manager.primary_focus().is_some()));
    let at_rest = focus_manager
        .with_focus_manager(FocusManager::primary_focus_bounds)
        .expect("focused target bounds at rest");

    assert_near(at_rest.min_x, 100.0);
    assert_near(at_rest.min_y, 100.0);
    assert_near(at_rest.max_x - at_rest.min_x, 120.0);
    assert_near(at_rest.max_y - at_rest.min_y, 80.0);

    // Run the 200pt-right/60pt-down, 60-frame slide halfway.
    for _ in 0..30 {
        state_machine.advance_and_apply(1.0 / 60.0);
    }

    let moved = focus_manager
        .with_focus_manager(FocusManager::primary_focus_bounds)
        .expect("focused target bounds after host motion");
    assert_near(moved.min_x, at_rest.min_x + 100.0);
    assert_near(moved.min_y, at_rest.min_y + 30.0);
    assert_near(moved.max_x - moved.min_x, at_rest.max_x - at_rest.min_x);
    assert_near(moved.max_y - moved.min_y, at_rest.max_y - at_rest.min_y);
}
