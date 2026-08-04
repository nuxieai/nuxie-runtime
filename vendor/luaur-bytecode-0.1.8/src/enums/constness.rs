#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constness {
    Undetermined,
    NotAConstant,
    VmConstant,
    ImmConstant,
}
