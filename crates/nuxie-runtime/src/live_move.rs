//! Nuxie live instance moves. Call under the scene occurrence's between-frame guard.
pub(crate) mod collapse;
pub(crate) mod frame;
pub use frame::FrameScope;
mod affine;
mod follow;
pub(crate) mod plan;
pub mod records;
mod transport;
mod values;
pub use collapse::set_row_collapsed;
pub(crate) use collapse::{
    apply_row_collapse, effectively_hidden, reconcile_row_hosts, reconcile_slot_collapse,
};
pub use follow::{Flight, Pose, SizeMode, slot_pose};
pub use plan::{LiveMoves, screens};
pub(crate) use transport::{SlotState, reconcile_slot, return_row_loans};
pub use transport::{empty_slot, move_live, validate_move};
pub(crate) use values::ValueBlend;

// Nuxie slots can carry the full local matrix produced by flight following.
// An ordinary host falls through to the original composition function.
pub(crate) fn compose_slot_transform<T: crate::source::transform_component::TransformUpdate>(
    object: &mut T,
) {
    if let Some(matrix) = object
        .as_nested_artboard()
        .and_then(|s| s.live_move.as_ref().and_then(|state| state.local_override))
    {
        object
            .as_transform_component_mut()
            .expect("slot transform")
            .cells()
            .set_local(matrix);
    }
    crate::source::transform_component::compose_world_transform(object);
}

#[cfg(feature = "test-support")]
pub mod device_fixture;
