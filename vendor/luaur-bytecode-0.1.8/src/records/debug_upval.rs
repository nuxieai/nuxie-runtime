#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DebugUpval {
    pub(crate) name: u32,
}
