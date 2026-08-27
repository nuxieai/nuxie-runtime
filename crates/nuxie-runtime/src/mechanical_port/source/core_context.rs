use crate::mechanical_port::source::core::Core;

pub use crate::mechanical_port::source::status_code::StatusCode;

pub trait CoreContext {
    fn resolve(&self, id: u32) -> Option<&mut Core>;
}
