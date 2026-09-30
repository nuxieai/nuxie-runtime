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

/// A grid min-content probe's at-most zero asks for intrinsic content, whereas
/// an ordinary at-most zero remains a real zero-sized box.
pub fn measure_mode_for_content(
    mode: LayoutMeasureMode,
    available: f32,
    probing: bool,
) -> LayoutMeasureMode {
    if probing && mode == LayoutMeasureMode::AtMost && !(available > 0.0) {
        LayoutMeasureMode::Undefined
    } else {
        mode
    }
}
