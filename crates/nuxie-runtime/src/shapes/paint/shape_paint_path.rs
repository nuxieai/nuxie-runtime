//! ShapePaintPath owns RawPath plus one lazily materialized RenderPath. Initial
//! construction performs `addRawPath` before `fillRule`; later dirt performs
//! `rewind` then `addRawPath` without replaying construction-only fill rules.

use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};

use crate::{
    draw::{RuntimePathBackendSlot, RuntimePathCommand, RuntimeShapePathState},
    math::raw_path::{runtime_raw_path_from_commands, runtime_rebuild_raw_path_from_commands},
};

/// Clone-owned counterpart of C++ `ShapePaintPath`. RawPath is the sole CPU
/// geometry source; the backend RenderPath remains in its one-to-one sidecar.
#[derive(Debug)]
pub(crate) struct RuntimeShapePaintPathOwner {
    pub(crate) dirty: Cell<bool>,
    pub(crate) retained: RefCell<Option<RuntimeShapePathState>>,
    pub(crate) backend: RuntimePathBackendSlot,
}

impl Clone for RuntimeShapePaintPathOwner {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl Default for RuntimeShapePaintPathOwner {
    fn default() -> Self {
        Self {
            dirty: Cell::new(true),
            retained: RefCell::new(None),
            backend: RuntimePathBackendSlot::default(),
        }
    }
}

impl RuntimeShapePaintPathOwner {
    pub(crate) fn mark_dirty(&self) {
        self.dirty.set(true);
    }

    pub(crate) fn replace_retained(&self, retained: RuntimeShapePathState) {
        *self.retained.borrow_mut() = Some(retained);
        self.dirty.set(false);
    }

    pub(crate) fn rebuild_retained_from_commands(&self, commands: &[RuntimePathCommand]) {
        let mut retained = self.retained.borrow_mut();
        match retained.as_mut() {
            Some(retained) => runtime_rebuild_raw_path_from_commands(
                Arc::make_mut(&mut retained.raw_path),
                commands,
            ),
            None => {
                *retained = Some(RuntimeShapePathState {
                    raw_path: Arc::new(runtime_raw_path_from_commands(commands)),
                });
            }
        }
        self.dirty.set(false);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Materialization {
    Create,
    Refresh,
    Reuse,
}

pub(crate) fn materialization(
    has_render_path: bool,
    raw_mutation_changed: bool,
) -> Materialization {
    if !has_render_path {
        Materialization::Create
    } else if raw_mutation_changed {
        Materialization::Refresh
    } else {
        Materialization::Reuse
    }
}
