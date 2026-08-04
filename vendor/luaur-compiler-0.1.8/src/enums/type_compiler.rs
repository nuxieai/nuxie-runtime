#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Break,
    Continue,
}

impl Type {
    pub const Break: Self = Self::Break;
    pub const Continue: Self = Self::Continue;
}
