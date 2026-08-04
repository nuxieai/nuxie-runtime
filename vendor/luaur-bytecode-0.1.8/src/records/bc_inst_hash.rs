#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct BcInstHash;

impl BcInstHash {
    pub(crate) const M: u32 = 0x5bd1e995;
    pub(crate) const R: i32 = 24;
}
