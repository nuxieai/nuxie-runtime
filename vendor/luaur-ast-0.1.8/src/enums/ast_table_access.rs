#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AstTableAccess {
    Read = 0b01,
    Write = 0b10,
    ReadWrite = 0b11,
}
