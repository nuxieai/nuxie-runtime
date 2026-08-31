//! Pinned source-owner module tree for the ORE translation.
pub mod source;

#[doc(hidden)]
#[cfg(all(target_vendor = "apple", feature = "metal-backend"))]
mod target_inventory;
