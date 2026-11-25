//! The geometry of a zoomed pair: a step unit's two versions flying as one box
//! (docs/live-moves.md). The pair itself is `in_place::InPlacePair`.
//!
//! The box's ends are read once, when the step starts. Each frame the box is
//! made by its rule at the flight's progress, and each version is solved onto
//! it: hung from its top center at its turn, at its own scale, clipped to it,
//! and at the hand-over's opacity. No end refuses the step: an end that isn't
//! a plain rectangle, or a matrix that can't be inverted, has its own rule.
use crate::{
    css_layout::paint::Radii,
    source::{
        artboard::FlightClip,
        math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
    },
};
use std::f32::consts::PI;

/// The hand-over: the arriving version reaches 1, and the leaving one leaves
/// it, over this much of the flight's progress.
pub(in crate::live_move) const HAND_OVER: f32 = 0.65;

/// What a pair reads once when its step starts, in the page's drawing space.
#[derive(Clone, Copy, Debug)]
pub struct Ends {
    /// The leaving frame as drawn when the step starts.
    pub leaving: Mat2D,
    /// The arriving frame at its resting place.
    pub arriving: Mat2D,
    pub leaving_size: Vec2D,
    pub arriving_size: Vec2D,
    pub leaving_corners: Radii,
    pub arriving_corners: Radii,
}

/// The rule the box follows, from the step unit's `zoom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::live_move) enum Rule {
    /// 1, leaving the source: the arriving shape drawn at a scale that blends
    /// evenly from k, L's share of A, to 1.
    Leaving,
    /// 2, returning: the sizes blend evenly, at the arriving side's live
    /// scale, moved by its tracker.
    Returning,
}
impl Rule {
    /// The rule a step unit's `zoom` names, or None for no zoom.
    pub(in crate::live_move) fn of(zoom: u32) -> Option<Self> {
        match zoom {
            1 => Some(Self::Leaving),
            2 => Some(Self::Returning),
            _ => None,
        }
    }
}

/// One end of the box: a version's frame content box under its end's matrix.
#[derive(Clone, Copy, Debug)]
pub(in crate::live_move) struct End {
    center: Vec2D,
    turn: f32,
    size: Vec2D,
    /// Each axis's scale: the lengths of the matrix's columns.
    scales: Vec2D,
    content: Vec2D,
    radii: Radii,
}
impl End {
    /// A plain end is the rectangle its matrix makes of its content box. A
    /// skewed or mirrored end's rectangle is the axis-aligned box around it,
    /// unturned, with both axis scales that box's width over the content's
    /// width. Each corner's radii take the axis scales.
    pub(in crate::live_move) fn of(matrix: Mat2D, content: Vec2D, corners: Radii) -> Self {
        let (a, b, c, d) = (matrix[0], matrix[1], matrix[2], matrix[3]);
        let (sx, sy) = (a.hypot(b), c.hypot(d));
        let skewed = (a * c + b * d).abs() > 1e-4 * (sx * sy).max(1e-6);
        let mirrored = a * d - b * c < 0.0;
        let radii = |scales: Vec2D| corners.map(|r| Vec2D::new(r.x * scales.x, r.y * scales.y));
        if skewed || mirrored {
            let around = matrix.map_bounding_box(Aabb::new(0.0, 0.0, content.x, content.y));
            let scale = if content.x > 0.0 {
                around.width() / content.x
            } else {
                0.0
            };
            let scales = Vec2D::new(scale, scale);
            Self {
                center: around.center(),
                turn: 0.0,
                size: Vec2D::new(around.width(), around.height()),
                scales,
                content,
                radii: radii(scales),
            }
        } else {
            let scales = Vec2D::new(sx, sy);
            Self {
                center: matrix * (content * 0.5),
                turn: b.atan2(a),
                size: Vec2D::new(sx * content.x, sy * content.y),
                scales,
                content,
                radii: radii(scales),
            }
        }
    }
}

/// The box at one progress, in the page's drawing space.
pub(in crate::live_move) struct ZoomBox {
    center: Vec2D,
    turn: f32,
    size: Vec2D,
    radii: Radii,
}

/// The box by `rule` at clamped progress `p`. The center blends evenly
/// and the turn the shorter way. Leaving the source, k is L's share of A
/// on the axis where they differ least, width on a tie, and each size
/// and corner radius is lerp(L / k, A, p) times lerp(k, 1, p). Returning,
/// or where k is 0 or not finite, each is lerp(L, A, p) times r, and the
/// center moves by t.
pub(in crate::live_move) fn zoom_box(
    rule: Rule,
    [l, a]: &[End; 2],
    p: f32,
    (r, t): (f32, Vec2D),
) -> ZoomBox {
    let lerp = |l: f32, a: f32, p: f32| l + (a - l) * p;
    let k = {
        let (width, height) = (l.size.x / a.size.x, l.size.y / a.size.y);
        if (width - 1.0).abs() <= (height - 1.0).abs() {
            width
        } else {
            height
        }
    };
    let leaves_the_source = rule == Rule::Leaving && k != 0.0 && k.is_finite();
    let blend = |l: f32, a: f32| {
        if leaves_the_source {
            lerp(l / k, a, p) * lerp(k, 1.0, p)
        } else {
            lerp(l, a, p) * r
        }
    };
    let apart = (a.turn - l.turn + PI).rem_euclid(2.0 * PI) - PI;
    let mut radii = Radii::default();
    for (corner, (from, to)) in radii.iter_mut().zip(l.radii.iter().zip(a.radii)) {
        *corner = Vec2D::new(blend(from.x, to.x), blend(from.y, to.y));
    }
    ZoomBox {
        center: Vec2D::lerp(l.center, a.center, p) + t,
        turn: l.turn + apart * p,
        size: Vec2D::new(blend(l.size.x, a.size.x), blend(l.size.y, a.size.y)),
        radii,
    }
}

/// Hangs a version's content from the box's top center at the box's turn,
/// at λ times its end's axis scales: λ, those scales, and the content-to-page
/// matrix, None when λ is not a positive finite number.
pub(in crate::live_move) fn hang(end: &End, zoom_box: &ZoomBox) -> (f32, Vec2D, Option<Mat2D>) {
    let lambda = if end.size.x > 0.0 {
        zoom_box.size.x / end.size.x
    } else if end.size.y > 0.0 {
        zoom_box.size.y / end.size.y
    } else {
        0.0
    };
    let scales = end.scales * lambda;
    let solvable = lambda > 0.0 && lambda.is_finite() && scales.x > 0.0 && scales.y > 0.0;
    let target = solvable.then(|| {
        let (sin, cos) = zoom_box.turn.sin_cos();
        let half = zoom_box.size.y / 2.0;
        let top = zoom_box.center + Vec2D::new(half * sin, -half * cos);
        let from_center = end.content.x / 2.0 * scales.x;
        Mat2D::new(
            cos * scales.x,
            sin * scales.x,
            -sin * scales.y,
            cos * scales.y,
            top.x - from_center * cos,
            top.y - from_center * sin,
        )
    });
    (lambda, scales, target)
}

/// The box in a version's content space, the clip it draws in; None when
/// it is not finite.
pub(in crate::live_move) fn content_clip(
    end: &End,
    zoom_box: &ZoomBox,
    scales: Vec2D,
) -> Option<FlightClip> {
    let clip = FlightClip {
        width: end.content.x,
        height: zoom_box.size.y / scales.y,
        radii: zoom_box
            .radii
            .map(|r| Vec2D::new(r.x / scales.x, r.y / scales.y)),
    };
    (clip.height.is_finite()
        && clip
            .radii
            .iter()
            .all(|r| r.x.is_finite() && r.y.is_finite()))
    .then_some(clip)
}
