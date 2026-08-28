#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StrokeJoin {
    Miter = 0,
    Round = 1,
    Bevel = 2,
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
