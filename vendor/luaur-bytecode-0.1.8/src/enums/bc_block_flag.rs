#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BcBlockFlag {
    Dead = 1 << 0,
}
