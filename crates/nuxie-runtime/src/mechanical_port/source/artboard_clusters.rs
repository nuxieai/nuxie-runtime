//! Source-only Artboard state, translated from artboard_clusters.hpp.
use super::core::CoreHandle;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

#[derive(Default)]
pub(crate) struct ArtboardSourceState {
    pub animations: Vec<CoreHandle>,
    pub state_machines: Vec<CoreHandle>,
    pub invalid_objects: Vec<Option<CoreHandle>>,
    // Shared-keyframe identity -> first serialized source bind. Built once;
    // source artboard bindings do not change during playback.
    pub key_frame_binds: RefCell<HashMap<CoreHandle, CoreHandle>>,
    pub key_frame_binds_built: Cell<bool>,
}
