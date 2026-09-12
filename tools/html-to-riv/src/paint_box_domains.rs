//! Conditional bounds for ordinary rounded box paint. This establishes finite
//! native arithmetic and the thin-size predicate, not Chrome layout equality.
use super::{wrapping_coordinates, wrapping_domains::Slot, wrapping_position,
    wrapping_sizes::MachineInterval};

const LIMIT: f32 = 16384.;
const THIN: f64 = 1. / 16.;
const U: f64 = 1. / 16777216.;
const TINY: f64 = f32::from_bits(1) as f64;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Unresolved {
    Count,
    Viewport { axis: usize },
    Arithmetic { item: usize, axis: usize },
    ThinThreshold { item: usize, axis: usize },
}
#[derive(Debug, Clone, Copy)]
pub(super) struct Axis {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub measured_size: [f32; 2],
    pub subtraction_error: f64,
    pub thin_flag: bool,
}
#[derive(Debug)]
pub(super) struct Proof {
    pub axes: Vec<[Axis; 2]>,
    pub viewport: [MachineInterval; 2],
}
fn up(a: f64, b: f64) -> f64 { (a + b).next_up() }
fn product_up(a: f64, b: f64) -> f64 { (a * b).next_up() }
fn error(magnitude: f64) -> f64 { up(product_up(U, magnitude), TINY) }

fn axis(start: [f32; 2], size: MachineInterval, item: usize, axis: usize)
    -> Result<Axis, Unresolved>
{
    let arithmetic = || Unresolved::Arithmetic { item, axis };
    if start.iter().any(|v| !v.is_finite()) || start[0] > start[1] {
        return Err(arithmetic());
    }
    // Identity linear transforms: target.world * origin(1) adds the native
    // extent once. origin(0) copies its translation. Monotone f32 endpoints
    // enclose the machine results, including all source-domain viewports.
    let end = [start[0] + size.lower(), start[1] + size.upper()];
    // The local-space copy reconstructs end - start. Independent endpoints
    // conservatively check finiteness even though they lose the correlation.
    let measured_size = [end[0] - start[1], end[1] - start[0]];
    if end.iter().chain(&measured_size).any(|v| !v.is_finite()) {
        return Err(arithmetic());
    }
    let magnitude = (start[0] as f64).abs().max((start[1] as f64).abs());
    let addition_error = error(up(magnitude, size.upper() as f64));
    // q=fl(fl(p+s)-p). Preserve shared p when bounding q-s; independent
    // subtraction bounds above alone would reject every responsive interval.
    let subtraction_error = up(addition_error,
        error(up(size.upper() as f64, addition_error)));
    if !subtraction_error.is_finite() { return Err(arithmetic()); }
    let thin_flag = if size.upper() == 0. {
        // fl(p+0)-p is exactly numerical zero for every finite p.
        false
    } else if start[0] == start[1] && size.lower() == size.upper() {
        // A point domain allows checking the exact machine operation rather
        // than losing an exact threshold to a conservative error bound.
        let measured = (start[0] + size.lower()) - start[0];
        if (measured > THIN as f32) != (size.lower() > THIN as f32) {
            return Err(Unresolved::ThinThreshold { item, axis });
        }
        measured > THIN as f32
    } else if (size.lower() as f64 - subtraction_error).next_down() > THIN {
        true
    } else if up(size.upper() as f64, subtraction_error) <= THIN {
        false
    } else {
        return Err(Unresolved::ThinThreshold { item, axis });
    };
    Ok(Axis { start, end, measured_size, subtraction_error, thin_flag })
}

/// Consume bounds from the same closed composition whose actual fields are
/// bound separately. The active cross coordinate is the final landed position;
/// the other coordinate retains its native initial world translation. All
/// initial transforms were already checked by wrapping_coordinates, so the
/// strength-one old*0 terms cannot retain infinity from initialization.
///
/// The emitted scalar roots clamp each finite raw corner to [-LIMIT,LIMIT].
/// For ordered edges, this preserves the rounded interval inside [0,viewport]:
/// negative saturation ends at -LIMIT or -LIMIT+1, still outside; positive
/// saturation starts at LIMIT, outside every allowed viewport. Edges that can
/// bound visible coverage are unchanged. In particular clamping to zero would
/// incorrectly expose wholly negative thin boxes. Thin flags use RAW extents.
/// Four 32768-square masks cover the viewport with these bounded edges, even
/// when minimum extent moves a final trailing edge to LIMIT+1. The inherited
/// origin-zero artboard clip is required. This is not GPU/pixel qualification.
pub(super) fn prove(coordinates: &wrapping_coordinates::Proof,
    position: &wrapping_position::Proof, slots: &[Slot],
    viewport: [MachineInterval; 2], row: bool) -> Result<Proof, Unresolved>
{
    if slots.is_empty() || slots.len() != coordinates.visible_world().len()
        || slots.len() != position.items.len() { return Err(Unresolved::Count); }
    for (axis, v) in viewport.iter().enumerate() {
        if v.upper() > LIMIT { return Err(Unresolved::Viewport { axis }); }
    }
    let cross = usize::from(row);
    let mut axes = Vec::with_capacity(slots.len());
    for (i, slot) in slots.iter().enumerate() {
        let mut item = Vec::with_capacity(2);
        for physical in 0..2 {
            let start = if physical == cross {
                let v = position.items[i].landed; [v.lower, v.upper]
            } else {
                let v = coordinates.visible_world()[i][physical]; [v.lower(), v.upper()]
            };
            item.push(axis(start, slot.visible_axes[physical], i, physical)?);
        }
        axes.push([item[0], item[1]]);
    }
    Ok(Proof { axes, viewport })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn interval(lo: f32, hi: f32) -> MachineInterval {
        MachineInterval::new(lo, hi).unwrap()
    }
    #[test]
    fn correlated_size_bound_contains_native_subtraction_across_large_origins() {
        let proof = axis([-32000., 32000.], interval(12., 90.), 0, 0).unwrap();
        assert!(proof.thin_flag);
        for p in [-32000., -16384., -0.5, 0., 0.5, 16384., 32000.] {
            for s in [12., 12.25, 30.5, 60., 90.] {
                let end = p + s; let measured = end - p;
                assert!((measured as f64 - s as f64).abs() <= proof.subtraction_error);
                assert!(end >= proof.end[0] && end <= proof.end[1]);
                assert!(measured >= proof.measured_size[0] && measured <= proof.measured_size[1]);
            }
        }
    }
    #[test]
    fn thin_threshold_loss_rejects_instead_of_inheriting_a_large_box_result() {
        let above = (1_f32 / 16.).next_up();
        assert!(axis([0.; 2], interval(above, above), 0, 0).unwrap().thin_flag);
        assert!(!axis([0.; 2], interval(1./16., 1./16.), 0, 0).unwrap().thin_flag);
        assert_eq!(axis([16384.; 2], interval(above, above), 2, 1).unwrap_err(),
            Unresolved::ThinThreshold { item: 2, axis: 1 });
        assert!(matches!(axis([-16384., 16384.], interval(0., 1.), 0, 0),
            Err(Unresolved::ThinThreshold { .. })));
        assert!(!axis([-f32::MAX / 4., f32::MAX / 4.], interval(0., 0.), 0, 0).unwrap().thin_flag);
    }
    #[test]
    fn nonfinite_corner_or_local_difference_rejects_before_emission() {
        assert!(matches!(axis([f32::MAX; 2], interval(f32::MAX, f32::MAX), 0, 0),
            Err(Unresolved::Arithmetic { .. })));
        assert!(matches!(axis([-f32::MAX, f32::MAX], interval(1., 1.), 0, 0),
            Err(Unresolved::Arithmetic { .. })));
    }
    #[test]
    fn saturation_preserves_artboard_intersection_including_thin_boundaries() {
        let edges = [-32768., -16384.001, -16384., -16383.999, -0.75, -0.5,
            -0.25, 0., 0.25, 0.5, 16383.5, 16384., 16384.002, 32768.];
        let round = |v: f32| (v as f64 + 0.5).floor();
        let paint = |lo: f32, hi: f32, flag: bool| {
            let l = round(lo); [l, round(hi).max(l + f64::from(u8::from(flag)))]
        };
        let clip = |r: [f64; 2], v: f64| {
            let l = r[0].max(0.); let h = r[1].min(v);
            if l >= h { None } else { Some([l, h]) }
        };
        for &lo in &edges { for &hi in &edges { if hi < lo { continue; }
            for flag in [false, true] { for viewport in [0., 0.5, 96., 16383.5, 16384.] {
                assert_eq!(clip(paint(lo, hi, flag), viewport),
                    clip(paint(lo.clamp(-LIMIT, LIMIT), hi.clamp(-LIMIT, LIMIT), flag), viewport));
            }}
        }}
    }
}
