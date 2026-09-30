use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
};

use crate::mechanical_port::source::core::CoreHandle;

/// Shared transition backend child payload; callbacks live on ScriptInstance.
pub use crate::scripting::ScriptTransitionChildRef as TransitionChildRef;

/// Shared state from rive/scripted/script_backend.hpp. Handles are non-owning
/// arena identities; each object retains its Rust VM handle independently.
pub struct ScriptBackend {
    display_scale: Cell<f32>,
    scripted_objects: RefCell<HashSet<CoreHandle>>,
}

impl Default for ScriptBackend {
    fn default() -> Self {
        Self {
            display_scale: Cell::new(1.0),
            scripted_objects: RefCell::new(HashSet::new()),
        }
    }
}

impl ScriptBackend {
    pub fn display_scale(&self) -> f32 {
        self.display_scale.get()
    }

    pub fn set_display_scale(&self, scale: f32) {
        if scale == self.display_scale.get() {
            return;
        }
        self.display_scale.set(scale);
        // Script callbacks may register or unregister objects. Do not hold the
        // registry borrow over dispatch, and recheck membership in the snapshot.
        let objects: Vec<_> = self.scripted_objects.borrow().iter().cloned().collect();
        for object in objects {
            if self.scripted_objects.borrow().contains(&object) {
                object.with_mut(|object| {
                    if let Some(layout) = object.as_scripted_layout_mut() {
                        layout.display_scale_changed();
                    } else if let Some(scripted) = object.as_scripted_object_mut() {
                        scripted.display_scale_changed();
                    }
                });
            }
        }
    }

    pub fn register_scripted_object(&self, object: CoreHandle) {
        self.scripted_objects.borrow_mut().insert(object);
    }

    pub fn unregister_scripted_object(&self, object: &CoreHandle) {
        self.scripted_objects.borrow_mut().remove(object);
    }
}
