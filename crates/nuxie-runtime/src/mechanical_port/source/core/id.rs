//! Runtime object references remain flat indices into an artboard's objects.
//! The editor-only `(client, object)` identity is not part of this build.
pub type Id = u32;

pub const EMPTY_ID: Id = u32::MAX;
