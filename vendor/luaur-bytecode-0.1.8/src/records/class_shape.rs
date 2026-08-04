extern crate alloc;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ClassShape {
    pub className: i32,
    pub propertyNames: Vec<i32>,
    pub methodNames: Vec<i32>,
}
