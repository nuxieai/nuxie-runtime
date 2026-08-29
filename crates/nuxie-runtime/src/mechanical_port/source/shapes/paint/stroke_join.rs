#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StrokeJoin {
    Miter = 0,
    Round = 1,
    Bevel = 2,
}

impl From<u32> for StrokeJoin {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Miter,
            1 => Self::Round,
            2 => Self::Bevel,
            _ => panic!("invalid StrokeJoin value: {value}"),
        }
    }
}

impl From<StrokeJoin> for nuxie_render_api::StrokeJoin {
    fn from(value: StrokeJoin) -> Self {
        match value {
            StrokeJoin::Miter => Self::Miter,
            StrokeJoin::Round => Self::Round,
            StrokeJoin::Bevel => Self::Bevel,
        }
    }
}
