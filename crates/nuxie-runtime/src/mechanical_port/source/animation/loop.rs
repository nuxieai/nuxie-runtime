/// C++'s fixed-underlying enum also represents unnamed serialized values.
#[repr(transparent)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Loop(u32);

#[allow(non_upper_case_globals)]
impl Loop {
    pub const OneShot: Self = Self(0);
    pub const Loop: Self = Self(1);
    pub const PingPong: Self = Self(2);

    pub const fn value(self) -> u32 {
        self.0
    }
}

impl From<u32> for Loop {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl std::fmt::Debug for Loop {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::OneShot => formatter.write_str("OneShot"),
            Self::Loop => formatter.write_str("Loop"),
            Self::PingPong => formatter.write_str("PingPong"),
            _ => formatter.debug_tuple("Loop").field(&self.0).finish(),
        }
    }
}
