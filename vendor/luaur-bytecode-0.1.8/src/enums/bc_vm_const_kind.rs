#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BcVmConstKind {
    Nil,
    Boolean,
    Number,
    Vectorf,
    Vectord,
    String,
    Import,
    Table,
    Closure,
    Integer,
}
