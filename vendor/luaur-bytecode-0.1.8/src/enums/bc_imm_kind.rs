#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BcImmKind {
    Boolean,
    Int,
    Import,
}
