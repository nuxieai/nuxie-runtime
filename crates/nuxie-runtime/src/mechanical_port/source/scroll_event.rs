use super::math::vec2d::Vec2D;

/// Position within an indirect scroll gesture. Phaseless wheels only update.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollPhase {
    Begin = 0,
    #[default]
    Update = 1,
    End = 2,
    Momentum = 3,
    InertiaCancel = 4,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ScrollEvent {
    /// Content movement in artboard pixels; later content is negative.
    pub delta: Vec2D,
    pub phase: ScrollPhase,
    pub precise: bool,
}
