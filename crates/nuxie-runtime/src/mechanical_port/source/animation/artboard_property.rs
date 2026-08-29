#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtboardProperty {
    Width = 0,
    Height = 1,
    Ratio = 2,
}

impl ArtboardProperty {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Width),
            1 => Some(Self::Height),
            2 => Some(Self::Ratio),
            _ => None,
        }
    }
}
