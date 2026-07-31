//! ShapePaintMutator retains inherited render opacity and visibility flags on
//! the concrete SolidColor/Gradient occurrence; identical opacity writes are
//! no-ops at the generated setter boundary.

pub(crate) fn render_opacity_changed(previous: f32, next: f32) -> bool {
    previous != next
}
