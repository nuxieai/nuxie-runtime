#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutMeasureMode {
    Undefined = 0,
    Exactly = 1,
    AtMost = 2,
}

/// Only an available-space bound can be removed; a decided size survives.
pub fn unbound_measure_mode(mode: LayoutMeasureMode) -> LayoutMeasureMode {
    if mode == LayoutMeasureMode::AtMost {
        LayoutMeasureMode::Undefined
    } else {
        mode
    }
}
