use super::*;
use nuxie_graph::NSlicerTileModeNode;
use std::collections::BTreeMap;

/// Clone-owned `NSlicerDetails` callback state. The binary graph is only the
/// import seed; Axis and tile-mode children register here in authored order,
/// matching their C++ `onAddedDirty` callbacks.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeNSlicerDetailsOwner {
    pub(crate) local_id: usize,
    pub(crate) global_id: u32,
    pub(crate) type_name: &'static str,
    pub(crate) x_axes: Vec<NSlicerAxisNode>,
    pub(crate) y_axes: Vec<NSlicerAxisNode>,
    pub(crate) tile_modes: BTreeMap<u64, NSlicerTileModeNode>,
}

impl RuntimeNSlicerDetailsOwner {
    pub(crate) fn from_graph(details: &NSlicerDetailsNode) -> Option<Self> {
        if !is_details(details.type_name) {
            return None;
        }
        let mut owner = Self {
            local_id: details.local_id,
            global_id: details.global_id,
            type_name: details.type_name,
            x_axes: Vec::new(),
            y_axes: Vec::new(),
            tile_modes: BTreeMap::new(),
        };
        for axis in &details.x_axes {
            super::axis_x::on_added_dirty(&mut owner, axis, Some(details.local_id))?;
        }
        for axis in &details.y_axes {
            super::axis_y::on_added_dirty(&mut owner, axis, Some(details.local_id))?;
        }
        for mode in &details.tile_modes {
            super::n_slicer_tile_mode::on_added_dirty(&mut owner, mode, Some(details.local_id))?;
        }
        Some(owner)
    }

    pub(crate) fn add_axis_x(&mut self, axis: &NSlicerAxisNode) {
        self.x_axes.push(axis.clone());
    }

    pub(crate) fn add_axis_y(&mut self, axis: &NSlicerAxisNode) {
        self.y_axes.push(axis.clone());
    }

    pub(crate) fn add_tile_mode(&mut self, mode: &NSlicerTileModeNode) {
        self.tile_modes.insert(mode.patch_index, mode.clone());
    }

    pub(crate) fn patch_index(&self, patch_x: usize, patch_y: usize) -> Option<u64> {
        patch_y
            .checked_mul(self.x_axes.len().checked_add(1)?)
            .and_then(|index| index.checked_add(patch_x))
            .and_then(|index| u64::try_from(index).ok())
    }
}

pub(crate) fn is_details(type_name: &str) -> bool {
    matches!(type_name, "NSlicer" | "NSlicedNode")
}

pub(crate) fn axis_bucket(type_name: &str) -> Option<bool> {
    super::axis_x::is_axis(type_name)
        .then_some(true)
        .or_else(|| super::axis_y::is_axis(type_name).then_some(false))
}
