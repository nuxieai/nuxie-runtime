#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TransformSpace {
    World = 0,
    Local = 1,
}
