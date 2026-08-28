use std::collections::HashMap;

use crate::mechanical_port::source::{
    core::CoreHandle,
    layout::{
        axis::Axis, n_sliced_node::NSlicedNode, n_slicer::NSlicer,
        n_slicer_tile_mode::NSlicerTileModeType,
    },
};

#[derive(Default)]
pub struct NSlicerDetailsState {
    pub xs: Vec<CoreHandle>,
    pub ys: Vec<CoreHandle>,
    pub tile_modes: HashMap<i32, NSlicerTileModeType>,
}

pub trait NSlicerDetails {
    fn details_state(&mut self) -> &mut NSlicerDetailsState;
    fn axis_changed(&mut self);
    fn patch_index(&mut self, patch_x: i32, patch_y: i32) -> i32 {
        patch_y * (self.details_state().xs.len() as i32 + 1) + patch_x
    }
    fn add_axis_x(&mut self, axis: CoreHandle) {
        self.details_state().xs.push(axis);
    }
    fn add_axis_y(&mut self, axis: CoreHandle) {
        self.details_state().ys.push(axis);
    }
    fn add_tile_mode(&mut self, patch_index: i32, style: NSlicerTileModeType) {
        self.details_state().tile_modes.insert(patch_index, style);
    }
}

pub fn is_details(component: &CoreHandle) -> bool {
    component.with_downcast::<NSlicer, _>(|_| ()).is_some()
        || component.with_downcast::<NSlicedNode, _>(|_| ()).is_some()
}

pub fn axis_changed(component: &CoreHandle) -> bool {
    if component
        .with_downcast_mut::<NSlicer, _>(NSlicerDetails::axis_changed)
        .is_some()
    {
        true
    } else {
        component
            .with_downcast_mut::<NSlicedNode, _>(NSlicerDetails::axis_changed)
            .is_some()
    }
}

pub fn add_axis_x(component: &CoreHandle, axis: CoreHandle) -> bool {
    if component
        .with_downcast_mut::<NSlicer, _>(|details| details.add_axis_x(axis.clone()))
        .is_some()
    {
        true
    } else {
        component
            .with_downcast_mut::<NSlicedNode, _>(|details| details.add_axis_x(axis))
            .is_some()
    }
}

pub fn add_axis_y(component: &CoreHandle, axis: CoreHandle) -> bool {
    if component
        .with_downcast_mut::<NSlicer, _>(|details| details.add_axis_y(axis.clone()))
        .is_some()
    {
        true
    } else {
        component
            .with_downcast_mut::<NSlicedNode, _>(|details| details.add_axis_y(axis))
            .is_some()
    }
}

pub fn add_tile_mode(component: &CoreHandle, patch_index: i32, style: NSlicerTileModeType) -> bool {
    if component
        .with_downcast_mut::<NSlicer, _>(|details| details.add_tile_mode(patch_index, style))
        .is_some()
    {
        true
    } else {
        component
            .with_downcast_mut::<NSlicedNode, _>(|details| {
                details.add_tile_mode(patch_index, style)
            })
            .is_some()
    }
}
