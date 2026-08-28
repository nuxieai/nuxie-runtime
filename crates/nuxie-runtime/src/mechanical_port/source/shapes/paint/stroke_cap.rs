#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StrokeCap {
    Butt = 0,
    Round = 1,
    Square = 2,
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
