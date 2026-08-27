use std::collections::HashMap;

use crate::mechanical_port::source::{
    component::Component,
    layout::{
        axis::Axis, n_sliced_node::NSlicedNode, n_slicer::NSlicer,
        n_slicer_tile_mode::NSlicerTileModeType,
    },
};

#[derive(Default)]
pub struct NSlicerDetailsState {
    pub xs: Vec<*mut Axis>,
    pub ys: Vec<*mut Axis>,
    pub tile_modes: HashMap<i32, NSlicerTileModeType>,
}

pub trait NSlicerDetails {
    fn details_state(&mut self) -> &mut NSlicerDetailsState;
    fn axis_changed(&mut self);
    fn patch_index(&mut self, patch_x: i32, patch_y: i32) -> i32 {
        patch_y * (self.details_state().xs.len() as i32 + 1) + patch_x
    }
    fn add_axis_x(&mut self, axis: *mut Axis) {
        self.details_state().xs.push(axis);
    }
    fn add_axis_y(&mut self, axis: *mut Axis) {
        self.details_state().ys.push(axis);
    }
    fn add_tile_mode(&mut self, patch_index: i32, style: NSlicerTileModeType) {
        self.details_state().tile_modes.insert(patch_index, style);
    }
}

pub fn from(component: &mut Component) -> Option<&mut dyn NSlicerDetails> {
    match component.core_type() {
        NSlicer::TYPE_KEY => component
            .as_mut::<NSlicer>()
            .map(|value| value as &mut dyn NSlicerDetails),
        NSlicedNode::TYPE_KEY => component
            .as_mut::<NSlicedNode>()
            .map(|value| value as &mut dyn NSlicerDetails),
        _ => None,
    }
}
