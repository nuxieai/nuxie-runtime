//! renderer/cmd/render_handle.hpp at e949498e.
pub type RenderHandle = u32;
pub const INVALID_RENDER_HANDLE: RenderHandle = u32::MAX;
pub const CANVAS_HANDLE_FLAG: RenderHandle = super::handle_flags::HANDLE_FOREIGN_FLAG;
pub const CANVAS_HANDLE_MASK: RenderHandle = super::handle_flags::HANDLE_FOREIGN_MASK;
