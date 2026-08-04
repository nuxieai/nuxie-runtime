#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DebugLocal {
    pub(crate) name: u32,
    pub(crate) reg: u8,
    pub(crate) startpc: u32,
    pub(crate) endpc: u32,
}
