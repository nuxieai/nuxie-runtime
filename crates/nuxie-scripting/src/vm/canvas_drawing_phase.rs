//! Direct Rust owner for pinned C++ `ScopedCanvasDrawingPhase` and
//! `ScriptingContext::canvasDrawingPhase`.

use std::cell::Cell;
use std::rc::Rc;

/// VM-owned state checked by canvas-level GPU entry points.
#[derive(Debug, Clone, Default)]
pub struct CanvasDrawingPhase {
    active: Rc<Cell<bool>>,
}

impl CanvasDrawingPhase {
    /// Whether this VM is currently invoking authored `drawCanvas` callbacks.
    pub fn is_active(&self) -> bool {
        self.active.get()
    }

    /// Enter the drawing phase and restore the exact previous value on drop.
    pub fn scoped(&self) -> ScopedCanvasDrawingPhase {
        ScopedCanvasDrawingPhase::new(Some(self))
    }
}

/// Rust RAII translation of pinned C++ `ScopedCanvasDrawingPhase`.
///
/// The nullable constructor preserves the upstream early-init/teardown
/// behavior, while cloning the state handle lets safe Rust restore nested
/// scopes without retaining a borrowed context.
#[derive(Debug)]
pub struct ScopedCanvasDrawingPhase {
    active: Option<Rc<Cell<bool>>>,
    previous: bool,
}

impl ScopedCanvasDrawingPhase {
    pub fn new(context: Option<&CanvasDrawingPhase>) -> Self {
        let active = context.map(|context| Rc::clone(&context.active));
        let previous = active.as_ref().is_some_and(|active| active.get());
        if let Some(active) = active.as_ref() {
            active.set(true);
        }
        Self { active, previous }
    }
}

impl Drop for ScopedCanvasDrawingPhase {
    fn drop(&mut self) {
        if let Some(active) = self.active.as_ref() {
            active.set(self.previous);
        }
    }
}
