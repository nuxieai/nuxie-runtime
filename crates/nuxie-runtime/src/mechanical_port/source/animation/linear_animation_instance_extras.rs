//! Cold cluster from upstream animation/linear_animation_instance_extras.hpp.
//! Allocated only for scripted interpolators or data-bound keyframe values.

use crate::mechanical_port::source::{
    core::CoreHandle, data_bind::data_bind_container::DataBindContainerWeak,
    scripted::scripted_interpolator::StatefulScriptedInterpolator,
};
use std::collections::HashMap;

/// Teardown is explicit in LinearAnimationInstance. Each scripted holder
/// removes its own binds before its clone; keyframe binds precede their targets.
#[derive(Default)]
pub(super) struct LAIBindingExtras {
    pub scripted_interpolators: HashMap<CoreHandle, StatefulScriptedInterpolator>,
    pub keyframe_value_holders: HashMap<CoreHandle, CoreHandle>,
    pub keyframe_value_binds: HashMap<CoreHandle, CoreHandle>,
    // Safe Rust field projection: cleanup can run while Artboard is borrowed.
    pub bind_container: DataBindContainerWeak,
}
