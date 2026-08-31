//! renderer/ore/cmd/ore_handle.hpp at e949498e.
pub type ResourceHandle = u32;
pub const INVALID_HANDLE: ResourceHandle = u32::MAX;
pub const REAL_RESOURCE_FLAG: ResourceHandle = crate::cmd::handle_flags::HANDLE_FOREIGN_FLAG;
pub const REAL_RESOURCE_MASK: ResourceHandle = crate::cmd::handle_flags::HANDLE_FOREIGN_MASK;
