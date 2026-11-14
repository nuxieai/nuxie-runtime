//! The plan's in-place branch (docs/live-moves.md, Flying in place).
//!
//! A step of a file that holds flight records flies each matched element in
//! its own Screen. Nothing moves between Screens: at take-off each copy's
//! place in layout is held (Layout mode), its ancestors' opacity is left out
//! of it, and its Screen's draw leaves it out, while the upper Screen, the
//! one holding the step's flight records, draws it from a lift entry placed
//! where today's flight slot would paint (css_layout/lift.rs). Each frame
//! runs the slot path's four barriers: Size, Layout, Pose and Settle.
//! Landing and cancel clear the marks; nothing was moved, so nothing is put
//! back.
use super::{
    drawn::{When, drawn},
    follow::{FlightFrame, SizeMode, draw_pose, lift_cover},
    plan::zoom::{self, Rule},
    records::{LiveFlight, Values},
    wide::Wide,
};
use crate::{
    Artboard, CoreHandle,
    css_layout::paint::Radii,
    css_layout::{
        flight_hold,
        lift::{Lift, OpacityRoot, set_opacity_root},
        record::LayoutCssStyle,
    },
    source::{
        artboard::FlightClip,
        component_dirt::ComponentDirt,
        layout_component::FlightShadow,
        math::{mat2d::Mat2D, vec2d::Vec2D},
    },
};
use anyhow::{Result, ensure};
use std::{cell::Cell, collections::HashSet};

/// One end of a flight: where an element rests, without its own transform,
/// in the page's drawing space; its resting size; and the opacity above it.
#[derive(Clone, Copy, Debug)]
pub(super) struct End {
    pub(super) place: Mat2D,
    pub(super) size: Vec2D,
    pub(super) opacity: f32,
}

/// The element a flight key names on a Screen, read from the Screen's own
/// records: the element whose CSS record carries the key. Structural work,
/// at a step or a membership change.
pub(super) fn keyed_elements(objects: &[CoreHandle]) -> Vec<(u32, CoreHandle)> {
    objects
        .iter()
        .filter_map(|object| {
            object
                .with_downcast::<LayoutCssStyle, _>(|record| {
                    let key = record.flight_key();
                    (key != 0)
                        .then(|| record.component().parent_handle())
                        .flatten()
                        .map(|element| (key, element))
                })
                .flatten()
        })
        .collect()
}

/// A Screen's flight records by key, in file order.
pub(super) fn flight_records(objects: &[CoreHandle]) -> Vec<(u32, CoreHandle)> {
    objects
        .iter()
        .filter_map(|object| {
            object
                .with_downcast::<LiveFlight, _>(|record| record.values.key)
                .map(|key| (key, object.clone()))
        })
        .collect()
}

/// The layout node a Layout-mode flight holds: a box's or a participant's
/// own, or for a component placement its placed artboard's root.
fn layout_node(element: &CoreHandle) -> CoreHandle {
    element
        .with(|object| {
            object
                .as_nested_artboard()
                .and_then(|nested| nested.artboard_instance_default())
                .map(|instance| instance.core_handle())
        })
        .flatten()
        .unwrap_or_else(|| element.clone())
}

/// Where `element` rests in its layout parent, its size from its hold while
/// held. Zero when it has no layout node.
fn resting_size(element: &CoreHandle) -> Vec2D {
    flight_hold::resting_box(&layout_node(element)).map_or_else(Vec2D::default, |rest| {
        Vec2D::new(rest.width(), rest.height())
    })
}

/// The size `element` draws at now: a box's laid-out size, which layout
/// interpolation moves toward its rest over time, else its rest.
fn drawn_size(element: &CoreHandle) -> Vec2D {
    layout_node(element)
        .with(|object| {
            object
                .as_layout_component()
                .map(|layout| Vec2D::new(layout.layout_width(), layout.layout_height()))
        })
        .flatten()
        .unwrap_or_else(|| resting_size(element))
}

/// The CSS box whose own paints and clip a flight's corners and shadow
/// replace: the element, or a placement's placed artboard's root.
fn box_of(element: &CoreHandle) -> CoreHandle {
    layout_node(element)
}

/// One end: the element's place in its parent without its own transform, as
/// a slot's pose is today, so its own turn, scale and offset stay inside its
/// draw. A CSS box's place is its parent's under its layout position; any
/// other element's is its parent's under the translation of its local
/// matrix. Its opacity is its parent's child opacity times the fades the
/// boxes above it hold out of it.
pub(super) fn end(element: &CoreHandle) -> Result<End> {
    let (parent, translation) = layout_place(element)?;
    let (parent_page, _, child_opacity) = draw_pose(&parent)?;
    Ok(End {
        place: parent_page * Mat2D::from_translation(translation),
        size: resting_size(element),
        opacity: child_opacity * crate::css_layout::group_opacity::held_above(element),
    })
}

/// `element`'s parent and its place in it without its own transform: a CSS
/// box's layout position, or the translation of any other element's local
/// matrix.
fn layout_place(element: &CoreHandle) -> Result<(CoreHandle, Vec2D)> {
    let parent = element
        .with(|object| object.component_parent_handle())
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("a flying element has no parent"))?;
    let css_box = element
        .with(|object| {
            object
                .as_layout_component()
                .filter(|layout| layout.is_css_mode() && !layout.is_artboard())
                .map(|layout| layout.layout_translation())
        })
        .flatten();
    let translation = match css_box {
        Some(translation) => translation,
        None => element
            .with(|object| {
                object.as_transform_component().map(|transform| {
                    let local = transform.transform();
                    Vec2D::new(local[4], local[5])
                })
            })
            .flatten()
            .ok_or_else(|| anyhow::anyhow!("a flying element has no transform"))?,
    };
    Ok((parent, translation))
}

/// Where `element` is placed without its own transform, read at `when`:
/// its parent as `drawn` reads it, then its layout place. It is what the
/// slot path reads from a home slot, which a key on the element itself
/// does not move.
fn place(element: &CoreHandle, when: When) -> Result<Mat2D> {
    let (parent, translation) = layout_place(element)?;
    Ok(drawn(&parent, when)? * Mat2D::from_translation(translation))
}

/// One flying copy of an in-place step.
#[derive(Clone)]
pub struct InPlaceFlight {
    /// Its flight record, whose controls its step's timeline keys.
    pub record: CoreHandle,
    /// The copy.
    pub element: CoreHandle,
    /// The Screen that holds the copy.
    pub row: CoreHandle,
    /// The step's two elements: the leaving Screen's, then the arriving
    /// Screen's. Both copies fly from one to the other.
    pub from: CoreHandle,
    pub to: CoreHandle,
    from_row: CoreHandle,
    to_row: CoreHandle,
    /// The upper Screen: the one holding the step's flight records, which
    /// draws every lift.
    pub board: CoreHandle,
    pub size_mode: SizeMode,
    /// A version of a zoomed pair (`InPlacePair`): laid out where it rests,
    /// with no corners or shadow, and solved onto the pair's box.
    pub zoomed: bool,
    /// The copy's own resting size when the step started.
    departure: Vec2D,
    /// The node a Layout-mode copy holds; None in Scale mode and in a zoom.
    held: Option<CoreHandle>,
    pub(crate) lift: Lift,
    /// Each end as last read, kept for a Screen asleep mid-step.
    from_end: Cell<Option<End>>,
    to_end: Cell<Option<End>>,
    /// The lift's last transform, kept while it can't be solved.
    last: Cell<Mat2D>,
}

impl InPlaceFlight {
    /// A copy of a step unit, read before anything is marked. `anchor` is
    /// the upper Screen's own copy.
    pub(super) fn new(
        record: CoreHandle,
        element: CoreHandle,
        row: CoreHandle,
        (from, from_row): (CoreHandle, CoreHandle),
        (to, to_row): (CoreHandle, CoreHandle),
        board: CoreHandle,
        zoomed: bool,
    ) -> Result<Self> {
        let size_mode = match Self::values_of(&record)?.size_mode {
            1 => SizeMode::Layout,
            _ => SizeMode::Scale,
        };
        let anchor = if to_row == board {
            to.clone()
        } else {
            from.clone()
        };
        let departure = resting_size(&element);
        let held = (size_mode == SizeMode::Layout && !zoomed).then(|| layout_node(&element));
        if let Some(node) = &held {
            ensure!(
                flight_hold::can_begin(node),
                "a Layout-mode flight needs an element laid out by a parent"
            );
        }
        Ok(Self {
            lift: Lift::new(row.clone(), element.clone(), anchor),
            record,
            element,
            row,
            from,
            to,
            from_row,
            to_row,
            board,
            size_mode,
            zoomed,
            departure,
            held,
            from_end: Cell::new(None),
            to_end: Cell::new(None),
            last: Cell::new(Mat2D::identity()),
        })
    }

    fn values_of(record: &CoreHandle) -> Result<Values> {
        record
            .with_downcast::<LiveFlight, _>(|record| record.values.clone())
            .ok_or_else(|| anyhow::anyhow!("flight record retired"))
    }

    /// Both ends, read live for an awake Screen and kept for one asleep.
    fn ends(&self, active: Option<&HashSet<CoreHandle>>) -> Result<(End, End)> {
        let read =
            |element: &CoreHandle, row: &CoreHandle, kept: &Cell<Option<End>>| match kept.get() {
                Some(end) if active.is_some_and(|active| !active.contains(row)) => Ok(end),
                _ => {
                    let end = end(element)?;
                    kept.set(Some(end));
                    Ok::<_, anyhow::Error>(end)
                }
            };
        Ok((
            read(&self.from, &self.from_row, &self.from_end)?,
            read(&self.to, &self.to_row, &self.to_end)?,
        ))
    }

    /// The Size barrier's read: the controls and both ends' resting sizes.
    pub(super) fn size_frame(&self, active: Option<&HashSet<CoreHandle>>) -> Result<FlightFrame> {
        let (a, b) = self.ends(active)?;
        let frame = FlightFrame::read(&Self::values_of(&self.record)?, a.size, b.size)?;
        if self.size_mode == SizeMode::Scale {
            ensure!(
                self.departure.x > 0.0 && self.departure.y > 0.0,
                "scale mode needs a nonzero box"
            );
        }
        Ok(frame)
    }

    /// The Size barrier's writes: the held copy's flight size, and the
    /// corners and shadow its box draws in place of its own.
    pub(super) fn apply_size(&self, frame: &FlightFrame) {
        if let Some(node) = &self.held {
            let size = frame.blended_size();
            flight_hold::set_size(node, (size.x.max(0.0), size.y.max(0.0)));
        }
        set_flight_paint(&self.element, frame.corners, frame.shadow);
    }

    /// The Pose barrier's read: the lift's transform, from the copy's own
    /// Screen's drawing space to the upper Screen's, and its opacity.
    ///
    /// With U the upper Screen's page matrix, L the copy's Screen's, S the
    /// copy's place now and P the blended pose, the lift's transform is
    /// U⁻¹ · P · S⁻¹ · L. In Scale mode P carries the stretch as its
    /// innermost scale: the blended size over the size the copy draws at
    /// now, so a copy resized mid-step still draws at the blend, while its
    /// layout interpolates toward the new size too. Where
    /// S, U or the copy's size can't be inverted, as inside a box scaled to
    /// 0, the copy draws nothing and keeps its last transform until it can
    /// be solved again, as a zoomed version does.
    pub(super) fn project(
        &self,
        frame: &FlightFrame,
        active: Option<&HashSet<CoreHandle>>,
    ) -> Result<(Mat2D, f32)> {
        let (a, b) = self.ends(active)?;
        let mut pose = frame.blend(a.place, b.place);
        let own = end(&self.element)?;
        if self.size_mode == SizeMode::Scale {
            let size = frame_size(frame, a.size, b.size);
            let drawn = drawn_size(&self.element);
            pose = pose * Mat2D::from_scale(size.x / drawn.x, size.y / drawn.y);
        }
        let (upper, _, _) = draw_pose(&self.board)?;
        let (row, _, _) = draw_pose(&self.row)?;
        let t = frame.progress;
        let opacity = (a.opacity + (b.opacity - a.opacity) * t) * frame.fade();
        ensure!(opacity.is_finite(), "non-finite flight opacity");
        let matrix = lift_matrix(upper, pose, own.place, row)
            .filter(|matrix| matrix.values().iter().all(|v| v.is_finite()));
        Ok(match matrix {
            Some(matrix) => {
                self.last.set(matrix);
                (matrix, opacity)
            }
            None => (self.last.get(), 0.0),
        })
    }

    /// The Pose barrier's write: only the lift's, which no layout reads.
    pub(super) fn apply_pose(&self, (matrix, opacity): (Mat2D, f32)) {
        let mut lift = self.lift.0.borrow_mut();
        lift.matrix = matrix;
        lift.opacity = opacity;
    }

    /// A zoomed version's clip: the pair's box in the copy's content space.
    fn apply_clip(&self, clip: Option<FlightClip>) {
        self.lift.0.borrow_mut().clip = clip;
    }

    /// What covers the copy's anchor on the upper Screen, stored in the lift.
    /// True when it changed, so the Screen sorts again.
    pub(super) fn read_cover(&self) -> bool {
        let (anchor, skip) = {
            let lift = self.lift.0.borrow();
            let skip = self
                .board
                .with_downcast::<Artboard, _>(|board| {
                    std::rc::Rc::clone(&board.lifts.borrow().skip)
                })
                .unwrap_or_default();
            (lift.anchor.clone(), skip)
        };
        let coverer = lift_cover(&self.board, &anchor, &skip);
        let mut lift = self.lift.0.borrow_mut();
        let same = match (&lift.coverer, &coverer) {
            (Some(a), Some(b)) => crate::css_layout::paint_order::same(a, b),
            (None, None) => true,
            _ => false,
        };
        lift.coverer = coverer;
        !same
    }

    /// Starts the copy's marks: its hold, its opacity root and its lift on
    /// both Screens.
    pub(super) fn mark(&self, opacity_root: OpacityRoot) {
        if let Some(node) = &self.held {
            flight_hold::begin(node, (self.departure.x, self.departure.y));
        }
        set_opacity_root(&self.element, opacity_root);
        self.row.with_downcast::<Artboard, _>(|row| {
            row.lifts.borrow_mut().away.push(self.lift.clone());
        });
        self.board.with_downcast::<Artboard, _>(|board| {
            board.lifts.borrow_mut().drawn.push(self.lift.clone());
        });
        changed(&self.row);
        changed(&self.board);
    }

    /// Ends every mark of the copy, whatever is still there of it.
    pub(super) fn clear(&self) {
        if let Some(node) = &self.held {
            flight_hold::end(node);
        }
        set_opacity_root(&self.element, OpacityRoot::None);
        set_flight_paint(&self.element, None, None);
        for row in [&self.row, &self.board] {
            row.with_downcast::<Artboard, _>(|row| {
                let mut lifts = row.lifts.borrow_mut();
                lifts.drawn.retain(|lift| !lift.ptr_eq(&self.lift));
                lifts.away.retain(|lift| !lift.ptr_eq(&self.lift));
                if lifts.away.is_empty() {
                    lifts.skip = std::rc::Rc::default();
                }
            });
            changed(row);
        }
        self.lift.0.borrow_mut().picture.release();
    }

    /// Whether the copy, its Screen and its record are all still there: a
    /// structural change that drops one cancels the step.
    pub(super) fn is_whole(&self, rows: &HashSet<CoreHandle>) -> bool {
        rows.contains(&self.row)
            && rows.contains(&self.board)
            && rows.contains(&self.from_row)
            && rows.contains(&self.to_row)
            && [&self.element, &self.from, &self.to].iter().all(|element| {
                element
                    .with(|object| object.as_component().and_then(|c| c.artboard_handle()))
                    .flatten()
                    .is_some()
            })
            && self.record.with(|_| ()).is_some()
    }

    /// The Screens the copy lays out in: its two ends' and its own.
    pub(super) fn rows(&self) -> [&CoreHandle; 3] {
        [&self.from_row, &self.to_row, &self.row]
    }
}

/// A zoomed pair flying in place: plan/zoom.rs's box, read and made the same
/// way, with each version solved onto it through its lift. A version's lift
/// takes the box as its transform, at λ in place of the frame's stretch, the
/// box in its content space as its clip, and the hand-over as its opacity;
/// the copy leaves its own opacity and its ancestors' out. Neither version
/// is held: each is laid out where it rests, its departure size.
pub(super) struct InPlacePair {
    /// The leaving version's copy, then the arriving one's.
    pub(super) copies: [CoreHandle; 2],
    rule: Rule,
    ends: zoom::Ends,
    /// On Back: the arriving copy, H₀⁻¹ (it as drawn at rest, inverted), c,
    /// and q = H₀⁻¹ c. None forward, and when H₀ can't be inverted.
    tracker: Option<(CoreHandle, Mat2D, Vec2D, Vec2D)>,
    progress: Cell<f32>,
    live: Cell<(f32, Vec2D)>,
    /// Each version's last lift transform, kept while it can't be solved.
    last: [Cell<Mat2D>; 2],
}

impl InPlacePair {
    /// Reads the box's ends before anything is marked, as `zoom::Pair` does:
    /// the leaving copy as drawn, the arriving copy at its resting place, each
    /// copy's departure size and its own corners, and on Back the arriving
    /// copy at rest and the tracked point, `center`.
    pub(super) fn take_off(
        rule: Rule,
        [leaving, arriving]: [&InPlaceFlight; 2],
        center: Vec2D,
    ) -> Result<Self> {
        let corners = |element: &CoreHandle| {
            box_of(element)
                .with(|object| {
                    let layout = object.as_artboard().map_or_else(
                        || object.as_layout_component(),
                        |board| Some(&board.base.base),
                    )?;
                    layout.css_box_geometry()
                })
                .flatten()
                .map_or_else(Radii::default, |geometry| geometry.radii)
        };
        let ends = zoom::Ends {
            leaving: drawn(&leaving.element, When::Live)?,
            arriving: drawn(&arriving.element, When::Rest)?,
            leaving_size: leaving.departure,
            arriving_size: arriving.departure,
            leaving_corners: corners(&leaving.element),
            arriving_corners: corners(&arriving.element),
        };
        let tracker = match rule {
            Rule::Leaving => None,
            Rule::Returning => {
                let home = arriving.element.clone();
                let mut rest_inverse = Mat2D::identity();
                let inverts = place(&home, When::Rest)?.invert(&mut rest_inverse)
                    && rest_inverse.values().iter().all(|v| v.is_finite());
                inverts.then(|| (home, rest_inverse, center, rest_inverse * center))
            }
        };
        Ok(Self {
            copies: [leaving.element.clone(), arriving.element.clone()],
            rule,
            ends,
            tracker,
            progress: Cell::new(0.0),
            live: Cell::new((1.0, Vec2D::default())),
            last: [Cell::new(Mat2D::identity()), Cell::new(Mat2D::identity())],
        })
    }

    /// The ends the pair read when its step started.
    #[cfg_attr(not(feature = "test-support"), allow(dead_code))]
    pub(super) fn ends(&self) -> zoom::Ends {
        self.ends
    }

    /// The Pose barrier for both versions: the progress from the leaving
    /// version's record, clamped, the last kept when it isn't finite; the box
    /// by the rule; each awake version solved onto it. A version with no λ,
    /// or whose matrices can't be inverted, draws nothing and keeps its last
    /// transform.
    pub(super) fn pose(
        &self,
        versions: [&InPlaceFlight; 2],
        active: &HashSet<CoreHandle>,
    ) -> Result<()> {
        let progress = InPlaceFlight::values_of(&versions[0].record)?.progress;
        if progress.is_finite() {
            self.progress.set(progress.clamp(0.0, 1.0));
        }
        let p = self.progress.get();
        let ends = [
            zoom::End::of(
                self.ends.leaving,
                self.ends.leaving_size,
                self.ends.leaving_corners,
            ),
            zoom::End::of(
                self.ends.arriving,
                self.ends.arriving_size,
                self.ends.arriving_corners,
            ),
        ];
        let zoom_box = zoom::zoom_box(self.rule, &ends, p, self.follow(versions[1], active)?);
        let rules = [
            ((1.0 - p) / zoom::HAND_OVER).min(1.0),
            (p / zoom::HAND_OVER).min(1.0),
        ];
        for version in 0..2 {
            let copy = versions[version];
            // The upper Screen draws the lift: while it is awake the lift
            // follows, even with the copy's own Screen asleep, whose matrices
            // stay as they were.
            if !active.contains(&copy.board) {
                continue;
            }
            let (_, scales, target) = zoom::hang(&ends[version], &zoom_box);
            let (upper, _, _) = draw_pose(&copy.board)?;
            let (row, _, _) = draw_pose(&copy.row)?;
            let now = drawn(&copy.element, When::Live)?;
            let matrix = target
                .and_then(|target| lift_matrix(upper, target, now, row))
                .filter(|matrix| matrix.values().iter().all(|v| v.is_finite()));
            match matrix {
                Some(matrix) => {
                    self.last[version].set(matrix);
                    copy.apply_pose((matrix, rules[version]));
                    copy.apply_clip(zoom::content_clip(&ends[version], &zoom_box, scales));
                }
                None => copy.apply_pose((self.last[version].get(), 0.0)),
            }
        }
        Ok(())
    }

    /// r and t on Back, as `zoom::Pair` reads them from the home slot, here
    /// from the arriving copy's place now over its place at rest, without
    /// its own transform, as a slot holds its unit. While its Screen sleeps,
    /// the last r and t stay.
    fn follow(
        &self,
        arriving: &InPlaceFlight,
        active: &HashSet<CoreHandle>,
    ) -> Result<(f32, Vec2D)> {
        let Some((home, rest_inverse, center, point)) = &self.tracker else {
            return Ok((1.0, Vec2D::default()));
        };
        if active.contains(&arriving.row) {
            let now = place(home, When::Live)?;
            let r = (now * *rest_inverse).determinant().abs().sqrt();
            let t = now * *point - *center;
            if r.is_finite() && t.x.is_finite() && t.y.is_finite() {
                self.live.set((r, t));
            }
        }
        Ok(self.live.get())
    }
}

/// A Scale-mode flight's size at a frame: the blend of its ends' sizes.
fn frame_size(frame: &FlightFrame, a: Vec2D, b: Vec2D) -> Vec2D {
    let t = frame.progress;
    Vec2D::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
}

/// Writes the corners and shadow `element`'s box draws in place of its own;
/// None for its own. A change rebuilds its paths, as on a unit's root.
fn set_flight_paint(element: &CoreHandle, corners: Option<[f32; 4]>, shadow: Option<FlightShadow>) {
    let node = box_of(element);
    if node.with(|object| object.as_artboard().is_some()) == Some(true) {
        node.with_downcast_mut::<Artboard, _>(|artboard| {
            artboard.set_flight_corners(corners);
            artboard.set_flight_shadow(shadow);
        });
        return;
    }
    let changed = node
        .with_mut(|object| {
            let layout = object.as_layout_component_mut()?;
            let corners = layout.set_flight_corners(corners);
            let shadow = layout.set_flight_shadow(shadow);
            Some(corners || shadow)
        })
        .flatten()
        .unwrap_or(false);
    if changed {
        crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(node)
            .add_dirt(ComponentDirt::PATH, false);
    }
}

/// A Screen's content changed: what a lift draws or leaves out moved, so a
/// kept result is stale and its draw order sorts again.
fn changed(row: &CoreHandle) {
    if let Some(dirty) = row.artboard_dirty_handle() {
        dirty.mark_css_draw_order_dirty();
        dirty.changed();
    }
}

/// Whether `inner` sits under `outer` on their artboard.
pub(super) fn is_inside(inner: &CoreHandle, outer: &CoreHandle) -> bool {
    let mut current = inner
        .with(|object| object.component_parent_handle())
        .flatten();
    while let Some(node) = current {
        if node == *outer {
            return true;
        }
        current = node
            .with(|object| {
                if object.as_artboard().is_some() {
                    None
                } else {
                    object.component_parent_handle()
                }
            })
            .flatten();
    }
    false
}

/// A lift's transform, `upper⁻¹ · pose · own⁻¹ · row`: from the copy's own
/// Screen's drawing space to the upper Screen's, where `own` is the copy's
/// place now. It cancels `own` by its inverse, so it is composed in f64
/// (`wide.rs`). None when `upper` or `own` has no inverse.
fn lift_matrix(upper: Mat2D, pose: Mat2D, own: Mat2D, row: Mat2D) -> Option<Mat2D> {
    let lift =
        Wide::of(upper).inverse()? * Wide::of(pose) * Wide::of(own).inverse()? * Wide::of(row);
    Some(lift.narrow())
}
