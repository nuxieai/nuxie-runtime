//! Nuxie live instance moves. Call under the scene occurrence's between-frame guard.
pub(crate) mod collapse;
mod drawn;
pub(crate) mod frame;
mod in_place;
pub use frame::FrameScope;
#[cfg(test)]
pub(crate) use in_place::end_of as in_place_end;
mod follow;
pub(crate) mod plan;
pub mod records;
mod transport;
mod values;
mod wide;
pub use collapse::set_row_collapsed;
pub(crate) use collapse::{
    apply_row_collapse, effectively_hidden, reconcile_row_hosts, reconcile_slot_collapse,
};
pub use drawn::When;
pub use follow::SizeMode;
pub use plan::{LiveMoves, screens, zoom::Ends};
#[cfg(feature = "test-support")]
pub use transport::refill_slot;
pub(crate) use transport::{SlotState, reconcile_slot, settle_leaving_rows};
pub use transport::{carry, validate_move};
pub(crate) use values::ValueBlend;

#[cfg(feature = "test-support")]
pub mod device_fixture;
