//! A zoomed pair: a step unit's two versions flying as one box. It is one entry
//! of the plan, so nothing that walks the plan can hold one version without the
//! other, and a flight alone carries no zoom (docs/live-moves.md).
//!
//! The box's ends are read once, when the step starts. Each frame the box is
//! made by its rule at the flight's progress, and each version is solved onto
//! it: hung from its top center at its turn, at its own scale, clipped to it,
//! and at the hand-over's opacity. Each version is laid out once, at its
//! departure size. No end refuses the step: an end that isn't a plain
//! rectangle, or a matrix that can't be inverted, has its own rule.
use crate::{
    CoreHandle,
    css_layout::paint::Radii,
    live_move::{
        drawn::{When, drawn},
        follow::{Flight, ProjectedPose, draw_pose},
        transport::child,
    },
    source::{
        artboard::FlightClip,
        math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
    },
};
use anyhow::Result;
use std::{cell::Cell, collections::HashSet, f32::consts::PI};

/// The hand-over: the arriving version reaches 1, and the leaving one leaves
/// it, over this much of the flight's progress.
const HAND_OVER: f32 = 0.65;

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
pub(super) enum Rule {
    /// 1, leaving the source: the arriving shape drawn at a scale that blends
    /// evenly from k, L's share of A, to 1.
    Leaving,
    /// 2, returning: the sizes blend evenly, at the arriving side's live
    /// scale, moved by its tracker.
    Returning,
}
impl Rule {
    /// The rule a step unit's `zoom` names, or None for no zoom.
    pub(super) fn of(zoom: u32) -> Option<Self> {
        match zoom {
            1 => Some(Self::Leaving),
            2 => Some(Self::Returning),
            _ => None,
        }
    }
}

/// One end of the box: a version's frame content box under its end's matrix.
#[derive(Clone, Copy, Debug)]
struct End {
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
    fn of(matrix: Mat2D, content: Vec2D, corners: Radii) -> Self {
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
struct ZoomBox {
    center: Vec2D,
    turn: f32,
    size: Vec2D,
    radii: Radii,
}

/// What the box follows on Back: the arriving element's home slot, empty
/// while its frame flies and still laid out where it rests.
struct Tracker {
    home: CoreHandle,
    /// H₀⁻¹: the home slot at rest, inverted.
    rest_inverse: Mat2D,
    /// c: the center of the page's drawing space.
    center: Vec2D,
    /// q = H₀⁻¹ c: the tracked point in the home slot's own space.
    point: Vec2D,
}

pub struct Pair {
    /// The leaving version, then the arriving one.
    flights: [Flight; 2],
    rule: Rule,
    ends: Ends,
    /// None forward, and on Back when H₀ can't be inverted: r is 1, t is 0.
    tracker: Option<Tracker>,
    /// The last progress read, kept when the record's isn't finite.
    progress: Cell<f32>,
    /// The last r and t, kept while the arriving Screen sleeps.
    live: Cell<(f32, Vec2D)>,
    /// Each version's last matrix and stretch, kept while it can't be solved.
    last: [Cell<(Mat2D, Vec2D)>; 2],
}

impl Pair {
    /// Reads the box's ends with both frames in their holders, after the
    /// returns have settled and before either version moves: L, the leaving
    /// frame as drawn; A, the arriving frame at its resting place; each
    /// content size, the flight's departure size; each frame's own corners;
    /// and on Back the home slot at rest and the tracked point, `center`. It
    /// refuses no end's shape.
    pub(super) fn take_off(
        rule: Rule,
        holders: [&CoreHandle; 2],
        flights: [Flight; 2],
        center: Vec2D,
    ) -> Result<Self> {
        let frame = |holder: &CoreHandle| -> Result<CoreHandle> {
            child(holder)?
                .map(|instance| instance.core_handle())
                .ok_or_else(|| anyhow::anyhow!("a zoomed version has no holder"))
        };
        let corners = |frame: &CoreHandle| {
            frame
                .with(|o| o.as_artboard()?.base.base.css_box_geometry())
                .flatten()
                .map_or_else(Radii::default, |geometry| geometry.radii)
        };
        let (leaving, arriving) = (frame(holders[0])?, frame(holders[1])?);
        let ends = Ends {
            leaving: drawn(&leaving, When::Live)?,
            arriving: drawn(&arriving, When::Rest)?,
            leaving_size: flights[0].departure_size(),
            arriving_size: flights[1].departure_size(),
            leaving_corners: corners(&leaving),
            arriving_corners: corners(&arriving),
        };
        let tracker = match rule {
            Rule::Leaving => None,
            Rule::Returning => {
                let home = holders[1].clone();
                let mut rest_inverse = Mat2D::identity();
                let inverts =
                    drawn(&home, When::Rest)?.invert(&mut rest_inverse) && finite(rest_inverse);
                inverts.then(|| Tracker {
                    point: rest_inverse * center,
                    home,
                    rest_inverse,
                    center,
                })
            }
        };
        Ok(Self {
            flights,
            rule,
            ends,
            tracker,
            progress: Cell::new(0.0),
            live: Cell::new((1.0, Vec2D::default())),
            last: [
                Cell::new((Mat2D::identity(), Vec2D::new(1.0, 1.0))),
                Cell::new((Mat2D::identity(), Vec2D::new(1.0, 1.0))),
            ],
        })
    }
    pub(super) fn flights(&self) -> &[Flight] {
        &self.flights
    }
    /// Each version's flight, for the plan to move it to its flight slot.
    pub(super) fn flights_mut(&mut self) -> &mut [Flight; 2] {
        &mut self.flights
    }
    #[cfg_attr(not(feature = "test-support"), allow(dead_code))]
    pub(super) fn ends(&self) -> Ends {
        self.ends
    }
    /// The versions whose flight slot's Screen is awake this frame.
    pub(super) fn awake<'a>(
        &'a self,
        active: &'a HashSet<CoreHandle>,
    ) -> impl Iterator<Item = &'a Flight> + 'a {
        self.flights.iter().filter(|f| active.contains(&f.board))
    }

    /// The size phase: reads the flight's progress, clamped, from the
    /// leaving version's record, keeping the last when it isn't finite, and
    /// lays each version out at its departure size, with no flight corners
    /// or shadow. Its values blend as any flight's do.
    pub(super) fn size(&self, active: &HashSet<CoreHandle>) -> Result<()> {
        if let Some(progress) = self.flights[0].progress() {
            self.progress.set(progress.clamp(0.0, 1.0));
        }
        let progress = self.progress.get();
        for flight in self.awake(active) {
            flight.apply_size(progress, flight.departure_size(), None, None)?;
        }
        Ok(())
    }

    /// The pose phase: makes the box at the progress the size phase read and
    /// solves each awake version onto it.
    pub(super) fn pose(&self, active: &HashSet<CoreHandle>) -> Result<()> {
        let p = self.progress.get();
        let ends = [
            End::of(
                self.ends.leaving,
                self.ends.leaving_size,
                self.ends.leaving_corners,
            ),
            End::of(
                self.ends.arriving,
                self.ends.arriving_size,
                self.ends.arriving_corners,
            ),
        ];
        let zoom_box = self.zoom_box(&ends, p, self.follow(active)?);
        let rules = [((1.0 - p) / HAND_OVER).min(1.0), (p / HAND_OVER).min(1.0)];
        for version in 0..2 {
            if active.contains(&self.flights[version].board) {
                self.solve(version, &ends[version], &zoom_box, rules[version])?;
            }
        }
        Ok(())
    }

    /// r and t on Back: K is the home slot as drawn now over its rest, r the
    /// square root of K's area scale, and t how far the tracked point has
    /// moved. While the arriving Screen sleeps, the last r and t stay.
    fn follow(&self, active: &HashSet<CoreHandle>) -> Result<(f32, Vec2D)> {
        let Some(tracker) = &self.tracker else {
            return Ok((1.0, Vec2D::default()));
        };
        if active.contains(&self.flights[1].to_board) {
            let now = drawn(&tracker.home, When::Live)?;
            let r = (now * tracker.rest_inverse).determinant().abs().sqrt();
            let t = now * tracker.point - tracker.center;
            if r.is_finite() && t.x.is_finite() && t.y.is_finite() {
                self.live.set((r, t));
            }
        }
        Ok(self.live.get())
    }

    /// The box by the rule at clamped progress `p`. The center blends evenly
    /// and the turn the shorter way. Leaving the source, k is L's share of A
    /// on the axis where they differ least, width on a tie, and each size
    /// and corner radius is lerp(L / k, A, p) times lerp(k, 1, p). Returning,
    /// or where k is 0 or not finite, each is lerp(L, A, p) times r, and the
    /// center moves by t.
    fn zoom_box(&self, [l, a]: &[End; 2], p: f32, (r, t): (f32, Vec2D)) -> ZoomBox {
        let lerp = |l: f32, a: f32, p: f32| l + (a - l) * p;
        let k = {
            let (width, height) = (l.size.x / a.size.x, l.size.y / a.size.y);
            if (width - 1.0).abs() <= (height - 1.0).abs() {
                width
            } else {
                height
            }
        };
        let leaves_the_source = self.rule == Rule::Leaving && k != 0.0 && k.is_finite();
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

    /// Solves one version onto the box. Its scale λ is the box's width over
    /// its end's width, else the height over the height; its target G hangs
    /// its content from the box's top center at the box's turn, at λ times
    /// its end's axis scales. The frame's stretch is (λ, λ) and its own
    /// opacity is left out; its slot's matrix is the flight parent's inverse
    /// times G times the frame's content transform's inverse, and its slot's
    /// opacity is the rule over the parent's. Its clip is the box in its
    /// content space. A version with no λ, or whose matrices can't be
    /// inverted, draws nothing and keeps its last matrix.
    fn solve(&self, version: usize, end: &End, zoom_box: &ZoomBox, rule: f32) -> Result<()> {
        let flight = &self.flights[version];
        let Some(unit) = child(&flight.flight)? else {
            return Ok(());
        };
        let lambda = if end.size.x > 0.0 {
            zoom_box.size.x / end.size.x
        } else if end.size.y > 0.0 {
            zoom_box.size.y / end.size.y
        } else {
            0.0
        };
        let scales = end.scales * lambda;
        let parent = flight
            .flight
            .with(|o| o.component_parent_handle())
            .flatten();
        let (parent_world, _, parent_opacity) = parent
            .as_ref()
            .map(draw_pose)
            .transpose()?
            .unwrap_or((Mat2D::identity(), 1.0, 1.0));
        let opacity = if parent_opacity == 0.0 {
            0.0
        } else {
            rule / parent_opacity
        };
        let solvable = lambda > 0.0 && lambda.is_finite() && scales.x > 0.0 && scales.y > 0.0;
        let stretch = if solvable {
            Vec2D::new(lambda, lambda)
        } else {
            self.last[version].get().1
        };
        let content = unit.with_artboard_mut(|b| {
            b.set_flight_skips_own_opacity(true);
            b.set_flight_stretch(stretch);
            b.content_transform()
        });
        let matrix = solvable
            .then(|| {
                let (sin, cos) = zoom_box.turn.sin_cos();
                let half = zoom_box.size.y / 2.0;
                let top = zoom_box.center + Vec2D::new(half * sin, -half * cos);
                let from_center = end.content.x / 2.0 * scales.x;
                let target = Mat2D::new(
                    cos * scales.x,
                    sin * scales.x,
                    -sin * scales.y,
                    cos * scales.y,
                    top.x - from_center * cos,
                    top.y - from_center * sin,
                );
                let (mut parent_inverse, mut content_inverse) =
                    (Mat2D::identity(), Mat2D::identity());
                (parent_world.invert(&mut parent_inverse) && content.invert(&mut content_inverse))
                    .then(|| parent_inverse * target * content_inverse)
            })
            .flatten()
            .filter(|matrix| finite(*matrix));
        let Some(matrix) = matrix.filter(|_| opacity.is_finite()) else {
            // Nothing under it draws: it keeps its last matrix and solves
            // again once it can.
            let (matrix, stretch) = self.last[version].get();
            return flight.apply_pose(ProjectedPose {
                matrix,
                opacity: 0.0,
                stretch,
            });
        };
        self.last[version].set((matrix, stretch));
        let clip = FlightClip {
            width: end.content.x,
            height: zoom_box.size.y / scales.y,
            radii: zoom_box
                .radii
                .map(|r| Vec2D::new(r.x / scales.x, r.y / scales.y)),
        };
        let clip = (clip.height.is_finite()
            && clip
                .radii
                .iter()
                .all(|r| r.x.is_finite() && r.y.is_finite()))
        .then_some(clip);
        unit.with_artboard_mut(|b| b.set_flight_clip(clip));
        flight.apply_pose(ProjectedPose {
            matrix,
            opacity,
            stretch,
        })
    }
}

fn finite(matrix: Mat2D) -> bool {
    matrix.values().iter().all(|value| value.is_finite())
}
