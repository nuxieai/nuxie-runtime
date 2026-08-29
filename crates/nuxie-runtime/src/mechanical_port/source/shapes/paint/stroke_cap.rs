#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StrokeCap {
    Butt = 0,
    Round = 1,
    Square = 2,
}

impl From<u32> for StrokeCap {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Butt,
            1 => Self::Round,
            2 => Self::Square,
            _ => panic!("invalid StrokeCap value: {value}"),
        }
    }
}

impl From<StrokeCap> for nuxie_render_api::StrokeCap {
    fn from(value: StrokeCap) -> Self {
        match value {
            StrokeCap::Butt => Self::Butt,
            StrokeCap::Round => Self::Round,
            StrokeCap::Square => Self::Square,
        }
    }
}
