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

/// The one reading of a flying element's box (docs/live-moves.md, Flying in
/// place). Every reader of where a flying element is, its size, how it sits
/// in its place and where its own content draws reads it here, whatever the
/// element is: the ends, the hold, the lift's pose and stretch, a zoom's
/// ends, tracker and clip, its corners and shadow, and its cover anchor.
///
/// The box is the element's layout box: the border box its layout node gets
/// in its parent, at its offset there (for a placement, where its placed
/// root's frame starts). Its place is its parent as drawn, then that
/// offset. Its own transform O takes the box into its place: a CSS box's
/// own transform; a component placement's own matrix without its
/// translation; nothing for a text, an image or a video. What draws inside
/// the box is its content: a text's wrapping, an image's fit and origin, a
/// placed root's own turn and scale, as a flight slot carries a unit holding
/// the element. The box matrix is place · O, and C takes the box to the
/// element's own drawing space, where its world draws: a box's is
/// identity. Only a box paints a box of its own, so only a box draws a
/// flight's corners and shadow.
#[derive(Clone)]
pub(super) struct Flying {
    element: CoreHandle,
    shape: Shape,
}

#[derive(Clone)]
enum Shape {
    /// A CSS box, its own layout node.
    Css,
    /// A text, an image or a video: the layout participant it hosts.
    Hosted(CoreHandle),
    /// A component placement: its placed artboard's root, its layout node.
    Placement(CoreHandle),
}

impl Flying {
    /// `element`'s box, or None for an element that has no layout box.
    pub(super) fn read(element: &CoreHandle) -> Option<Self> {
        use crate::source::layout::layout_participant::LayoutParticipant;
        let shape = element
            .with(|object| {
                if let Some(nested) = object.as_nested_artboard() {
                    return nested
                        .artboard_instance_default()
                        .map(|root| Shape::Placement(root.core_handle()));
                }
                if let Some(layout) = object.as_layout_component() {
                    return (layout.is_css_mode() && !layout.is_artboard()).then_some(Shape::Css);
                }
                object
                    .as_container_component()?
                    .children()
                    .iter()
                    .find(|child| child.is_type_of(LayoutParticipant::TYPE_KEY))
                    .cloned()
                    .map(Shape::Hosted)
            })
            .flatten()?;
        Some(Self {
            element: element.clone(),
            shape,
        })
    }

    /// What a flight key may name in place, decided here, by what the
    /// element is, at take-off, before anything is marked: a box, a text, an
    /// image, a video or a component placement of a CSS artboard, each
    /// admitted once an in-place page with it as the keyed element ran
    /// through the equivalence test. A key on anything else is refused by
    /// name.
    pub(super) fn of(element: &CoreHandle) -> Result<Self> {
        use crate::source::generated::{shapes::image_base::ImageBase, text::text_base::TextBase};
        // A video is an image too.
        let admitted = element
            .with(|object| {
                object.as_nested_artboard().is_some()
                    || object
                        .as_layout_component()
                        .is_some_and(|layout| layout.is_css_mode() && !layout.is_artboard())
            })
            .unwrap_or(false)
            || element.is_type_of(ImageBase::TYPE_KEY)
            || element.is_type_of(TextBase::TYPE_KEY);
        ensure!(
            admitted,
            "a flight key on an element that is not a box, a text, an image, a video or a \
             component placement does not fly in place"
        );
        let flying = Self::read(element)
            .ok_or_else(|| anyhow::anyhow!("a flight key names no layout box"))?;
        // A placed root without its own CSS record is laid out by its own
        // layout, and no in-place page with one draws as its slot page even
        // at rest, so it is not admitted.
        if let Shape::Placement(root) = &flying.shape {
            ensure!(
                root.with(|object| object.as_layout_component().map(|l| l.is_css_mode()))
                    .flatten()
                    .unwrap_or(false),
                "a flight key on a component placement of an artboard without its own CSS \
                 record does not fly in place"
            );
        }
        Ok(flying)
    }

    /// Whether `element` still reads as this box: a placement still shows
    /// the root it took off with, a host still hosts its participant. A
    /// bound component that swaps its artboard while its placement flies
    /// changes its box's layout node, and the step cancels as when a copy
    /// is gone.
    pub(super) fn still_reads(&self) -> bool {
        Self::read(&self.element).is_some_and(|now| match (&self.shape, &now.shape) {
            (Shape::Css, Shape::Css) => true,
            (Shape::Hosted(was), Shape::Hosted(is))
            | (Shape::Placement(was), Shape::Placement(is)) => was == is,
            _ => false,
        })
    }

    /// The node that holds the box's place in layout.
    pub(super) fn layout_node(&self) -> &CoreHandle {
        match &self.shape {
            Shape::Css => &self.element,
            Shape::Hosted(node) | Shape::Placement(node) => node,
        }
    }

    fn parent(&self) -> Result<CoreHandle> {
        self.element
            .with(|object| object.component_parent_handle())
            .flatten()
            .ok_or_else(|| anyhow::anyhow!("a flying element has no parent"))
    }

    /// The box's offset in its parent, without its own transform: a CSS
    /// box's layout position; a participant's laid-out box, where its host's
    /// world is placed from; for a placement, its world origin in its
    /// parent's, which holds its layout position, since layout moves a
    /// placement's world and not its local matrix, then where its root's
    /// frame starts from there. Under a parent scaled to 0 the world origin
    /// has no answer, and a placement falls back to its local translation:
    /// the lift, which can't be solved through a scale of 0, draws nothing
    /// until it can.
    fn offset(&self, parent: &CoreHandle) -> Result<Vec2D> {
        use crate::source::layout::layout_participant::LayoutParticipant;
        let offset = match &self.shape {
            Shape::Css => self
                .element
                .with(|object| object.as_layout_component().map(|l| l.layout_translation()))
                .flatten(),
            Shape::Hosted(participant) => {
                participant.with_downcast::<LayoutParticipant, _>(|participant| {
                    let bounds = participant.layout_bounds();
                    Vec2D::new(bounds.left(), bounds.top())
                })
            }
            Shape::Placement(root) => self
                .element
                .with(|object| {
                    let local = object.as_transform_component()?.transform();
                    Some(Vec2D::new(local[4], local[5]))
                })
                .flatten()
                .zip(self.placement_linear().zip(frame_start(root)))
                .map(|(local, (linear, start))| {
                    let origin = world_of(&self.element)
                        .and_then(|world| offset_in(world_of(parent).unwrap_or_default(), world))
                        .unwrap_or(local);
                    origin + linear * start
                }),
        };
        offset.ok_or_else(|| anyhow::anyhow!("a flying element has no layout place"))
    }

    /// Where the box is placed, without its own transform, read at `when`:
    /// its parent as `drawn` reads it, then its offset. It is what the slot
    /// path reads from a home slot, which a key on the element itself does
    /// not move.
    pub(super) fn place(&self, when: When) -> Result<Mat2D> {
        let parent = self.parent()?;
        Ok(drawn(&parent, when)? * Mat2D::from_translation(self.offset(&parent)?))
    }

    /// Where the box rests in its parent, its size from its hold while
    /// held. Zero when it has no layout node.
    pub(super) fn resting_size(&self) -> Vec2D {
        flight_hold::resting_box(self.layout_node()).map_or_else(Vec2D::default, |rest| {
            Vec2D::new(rest.width(), rest.height())
        })
    }

    /// The size the box draws at now, which layout interpolation moves
    /// toward its rest over time.
    fn drawn_size(&self) -> Vec2D {
        use crate::source::layout::layout_participant::LayoutParticipant;
        let size = match &self.shape {
            Shape::Css => self
                .element
                .with(|o| {
                    o.as_layout_component()
                        .map(|l| Vec2D::new(l.layout_width(), l.layout_height()))
                })
                .flatten(),
            Shape::Hosted(participant) => {
                participant.with_downcast::<LayoutParticipant, _>(|participant| {
                    let bounds = participant.layout_bounds();
                    Vec2D::new(bounds.width(), bounds.height())
                })
            }
            Shape::Placement(root) => root.with_downcast::<Artboard, _>(|root| {
                Vec2D::new(root.layout_width(), root.layout_height())
            }),
        };
        size.unwrap_or_else(|| self.resting_size())
    }

    /// A placement's own matrix without its translation, which its place
    /// holds.
    fn placement_linear(&self) -> Option<Mat2D> {
        self.element
            .with(|object| {
                let local = object.as_transform_component()?.transform();
                Some(Mat2D::new(local[0], local[1], local[2], local[3], 0.0, 0.0))
            })
            .flatten()
    }

    /// The box's own transform O, and O' for the copy stretched by
    /// `stretch` to `size`, the stretch innermost, as a frame's own
    /// transform is built (`hosted_frame::self_transform`): a box turning
    /// and scaling about its pivot taken from `size`, its CSS translate
    /// unscaled; any other element's box stretched from its top left, its
    /// content inside it, as a flying unit holding it stretches.
    fn own_transforms(&self, size: Vec2D, stretch: Vec2D) -> Option<(Mat2D, Mat2D)> {
        let own = match &self.shape {
            Shape::Css => {
                return self
                    .element
                    .with(|object| {
                        let layout = object.as_layout_component()?;
                        let mut sized = layout.own_transform_sized(size);
                        sized.scale_by_values(stretch.x, stretch.y);
                        Some((layout.build_own_transform(), sized))
                    })
                    .flatten();
            }
            Shape::Hosted(_) => Mat2D::identity(),
            Shape::Placement(_) => self.placement_linear()?,
        };
        Some((own, own * Mat2D::from_scale(stretch.x, stretch.y)))
    }

    /// The box matrix at `when`: the box, from (0, 0) to its size, in the
    /// page's drawing space.
    pub(super) fn box_matrix(&self, when: When) -> Result<Mat2D> {
        match &self.shape {
            Shape::Css => drawn(&self.element, when),
            Shape::Hosted(_) => self.place(when),
            Shape::Placement(root) => {
                let start =
                    frame_start(root).ok_or_else(|| anyhow::anyhow!("a placed root retired"))?;
                Ok(drawn(&self.element, when)? * Mat2D::from_translation(start))
            }
        }
    }

    /// C: from the box to the element's own drawing space, the space its
    /// world draws in. Identity for a box; for a placement, to where its
    /// root's frame starts; for a text, an image or a video, its world taken
    /// back from its place. None when its world can't be inverted.
    pub(super) fn content_from_box(&self) -> Option<Mat2D> {
        match &self.shape {
            Shape::Css => Some(Mat2D::identity()),
            Shape::Placement(root) => frame_start(root).map(Mat2D::from_translation),
            Shape::Hosted(_) => {
                let parent = self.parent().ok()?;
                let place = Wide::of(world_of(&parent)?)
                    * Wide::of(Mat2D::from_translation(self.offset(&parent).ok()?));
                Some((Wide::of(world_of(&self.element)?).inverse()? * place).narrow())
            }
        }
    }

    /// The corners its box draws with: a box's own; none for anything else,
    /// which paints no box.
    fn corners(&self) -> Radii {
        let geometry = match &self.shape {
            Shape::Css => self
                .element
                .with(|object| object.as_layout_component()?.css_box_geometry())
                .flatten(),
            Shape::Hosted(_) | Shape::Placement(_) => None,
        };
        geometry.map_or_else(Radii::default, |geometry| geometry.radii)
    }

    /// Writes the corners and shadow its box draws in place of its own;
    /// None for its own. Only a box paints one: a change rebuilds its paths.
    fn set_paint(&self, corners: Option<[f32; 4]>, shadow: Option<FlightShadow>) {
        if !matches!(self.shape, Shape::Css) {
            return;
        }
        let changed = self
            .element
            .with_mut(|object| {
                let layout = object.as_layout_component_mut()?;
                let corners = layout.set_flight_corners(corners);
                let shadow = layout.set_flight_shadow(shadow);
                Some(corners || shadow)
            })
            .flatten()
            .unwrap_or(false);
        if changed {
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                self.element.clone(),
            )
            .add_dirt(ComponentDirt::PATH, false);
        }
    }

    /// Where the box rests, for its cover: the level whose resting world it
    /// sits under, its matrix there, and its resting size. A text's, an
    /// image's or a video's sits in its parent at its participant's resting
    /// box; a placement's is its own resting world, from where its root's
    /// frame starts at its resting size. None for a box, which the cover
    /// reads as any box.
    pub(super) fn rest_anchor(&self) -> Option<(CoreHandle, Mat2D, Vec2D)> {
        match &self.shape {
            Shape::Css => None,
            Shape::Hosted(participant) => {
                let rest = flight_hold::resting_box(participant)?;
                Some((
                    self.parent().ok()?,
                    Mat2D::from_translation(Vec2D::new(rest.left(), rest.top())),
                    Vec2D::new(rest.width(), rest.height()),
                ))
            }
            Shape::Placement(root) => Some((
                self.element.clone(),
                Mat2D::from_translation(resting_frame_start(root)?),
                self.resting_size(),
            )),
        }
    }
}

/// Where a placed root's frame starts in its placement's space, before the
/// root's own transform: the top left of its bounds (`Artboard::bounds`).
/// A mounted root draws its content's origin at its placement's, so its
/// frame starts its content origin before it; one that frames its origin
/// starts at 0.
fn frame_start(root: &CoreHandle) -> Option<Vec2D> {
    frame_start_at(root, |root| {
        Vec2D::new(root.layout_width(), root.layout_height())
    })
}

/// Where a placed root's frame starts where it rests: at its resting size
/// while its hold lays it out at another.
fn resting_frame_start(root: &CoreHandle) -> Option<Vec2D> {
    frame_start_at(root, |root| root.resting_layout().1)
}

fn frame_start_at(root: &CoreHandle, size: impl FnOnce(&Artboard) -> Vec2D) -> Option<Vec2D> {
    root.with_downcast::<Artboard, _>(|root| {
        if root.frame_origin() {
            Vec2D::default()
        } else {
            -root.content_origin_at(size(root))
        }
    })
}

/// A component's world; an artboard's is its own space.
fn world_of(component: &CoreHandle) -> Option<Mat2D> {
    component
        .with(|object| {
            if object.as_artboard().is_some() {
                return Some(Mat2D::identity());
            }
            object
                .as_world_transform_component()
                .map(|transform| transform.world_transform())
        })
        .flatten()
}

/// Where `world`'s origin sits in `parent`'s space, in f64: `parent`'s
/// linear part inverted over the difference of the translations.
fn offset_in(parent: Mat2D, world: Mat2D) -> Option<Vec2D> {
    let p = parent.values().map(f64::from);
    let w = world.values().map(f64::from);
    let det = p[0] * p[3] - p[1] * p[2];
    if det == 0.0 || !det.is_finite() {
        return None;
    }
    let (dx, dy) = (w[4] - p[4], w[5] - p[5]);
    #[allow(clippy::cast_possible_truncation)]
    Some(Vec2D::new(
        ((p[3] * dx - p[2] * dy) / det) as f32,
        ((p[0] * dy - p[1] * dx) / det) as f32,
    ))
}

/// One end: the element's box placed in its parent without its own
/// transform, as a slot's pose is today, so its own turn, scale and offset
/// stay inside its draw. Its opacity is its parent's child opacity times the
/// fades the boxes above it hold out of it.
pub(super) fn end(flying: &Flying) -> Result<End> {
    let parent = flying.parent()?;
    let (_, _, child_opacity) = draw_pose(&parent)?;
    Ok(End {
        place: flying.place(When::Live)?,
        size: flying.resting_size(),
        opacity: child_opacity * crate::css_layout::group_opacity::held_above(&flying.element),
    })
}

/// An element's end, its place and resting size, for tests outside
/// live_move.
#[cfg(test)]
pub(crate) fn end_of(element: &CoreHandle) -> Result<(Mat2D, Vec2D)> {
    let end = end(&Flying::of(element)?)?;
    Ok((end.place, end.size))
}

/// One flying copy of an in-place step.
#[derive(Clone)]
pub struct InPlaceFlight {
    /// Its flight record, whose controls its step's timeline keys.
    pub record: CoreHandle,
    /// The copy.
    pub element: CoreHandle,
    /// The copy's box, and each end's.
    flying: Flying,
    from_box: Flying,
    to_box: Flying,
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
    /// A version of a zoomed pair (`InPlacePair`): held at its departure
    /// size, with no corners or shadow, and solved onto the pair's box.
    pub zoomed: bool,
    /// The copy's own resting size when the step started.
    departure: Vec2D,
    /// The node a Layout-mode copy or a zoomed version holds; None in Scale
    /// mode.
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
        let flying = Flying::of(&element)?;
        let (from_box, to_box) = (Flying::of(&from)?, Flying::of(&to)?);
        let size_mode = match Self::values_of(&record)?.size_mode {
            1 => SizeMode::Layout,
            _ => SizeMode::Scale,
        };
        let anchor = if to_row == board {
            to.clone()
        } else {
            from.clone()
        };
        let departure = flying.resting_size();
        // A zoomed version is held at its departure size: its hold begins
        // there and has no Size barrier to change it.
        let held = (size_mode == SizeMode::Layout || zoomed).then(|| flying.layout_node().clone());
        if let Some(node) = &held {
            ensure!(
                flight_hold::can_begin(node),
                "a held flight needs an element laid out by a parent"
            );
        }
        Ok(Self {
            lift: Lift::new(row.clone(), element.clone(), anchor),
            record,
            element,
            flying,
            from_box,
            to_box,
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
        let read = |element: &Flying, row: &CoreHandle, kept: &Cell<Option<End>>| match kept.get() {
            Some(end) if active.is_some_and(|active| !active.contains(row)) => Ok(end),
            _ => {
                let end = end(element)?;
                kept.set(Some(end));
                Ok::<_, anyhow::Error>(end)
            }
        };
        Ok((
            read(&self.from_box, &self.from_row, &self.from_end)?,
            read(&self.to_box, &self.to_row, &self.to_end)?,
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
        self.flying.set_paint(frame.corners, frame.shadow);
    }

    /// The Pose barrier's read: the lift's transform, from the copy's own
    /// Screen's drawing space to the upper Screen's, and its opacity.
    ///
    /// With U the upper Screen's page matrix, L the copy's Screen's, S the
    /// copy's place now and P the blended pose, the lift's transform is
    /// U⁻¹ · P · S⁻¹ · L. In Scale mode the copy's own transform O is drawn
    /// as O', built as a stretched frame's is: the stretch, the blended size
    /// over the size the copy draws at now, innermost, and a box's pivot
    /// taken from the blended size; the lift is U⁻¹ · P · O' · O⁻¹ · S⁻¹ · L.
    /// So a copy resized mid-step still draws at the blend, while its layout
    /// interpolates toward the new size too, and a turned copy stretches
    /// along its own axes. Where
    /// S, U or the copy's size can't be inverted, as inside a box scaled to
    /// 0, the copy draws nothing and keeps its last transform until it can
    /// be solved again, as a zoomed version does.
    pub(super) fn project(
        &self,
        frame: &FlightFrame,
        active: Option<&HashSet<CoreHandle>>,
    ) -> Result<LiftPose> {
        let (a, b) = self.ends(active)?;
        let pose = frame.blend(a.place, b.place);
        let own = self.flying.place(When::Live)?;
        let inner = if self.size_mode == SizeMode::Scale {
            let size = frame_size(frame, a.size, b.size);
            let drawn = self.flying.drawn_size();
            let stretch = Vec2D::new(size.x / drawn.x, size.y / drawn.y);
            self.flying.own_transforms(size, stretch)
        } else {
            None
        };
        let (upper, _, _) = draw_pose(&self.board)?;
        let (row, _, _) = draw_pose(&self.row)?;
        let t = frame.progress;
        let opacity = (a.opacity + (b.opacity - a.opacity) * t) * frame.fade();
        ensure!(opacity.is_finite(), "non-finite flight opacity");
        let matrix = lift_matrix(upper, pose, inner, own, row)
            .filter(|matrix| matrix.values().iter().all(|v| v.is_finite()));
        Ok(match matrix {
            Some(matrix) => {
                self.last.set(matrix);
                (matrix, opacity, drawn_map(upper, matrix, row))
            }
            None => (self.last.get(), 0.0, None),
        })
    }

    /// The Pose barrier's write: only the lift's, which no layout reads.
    pub(super) fn apply_pose(&self, (matrix, opacity, drawn): LiftPose) {
        let moved = {
            let mut lift = self.lift.0.borrow_mut();
            lift.matrix = matrix;
            lift.opacity = opacity;
            lift.set_drawn(drawn)
        };
        if moved {
            crate::css_layout::lift::semantics_moved(&self.element);
        }
    }

    /// A zoomed version's clip: the pair's box in the copy's box, and the
    /// map from its box to its own drawing space, where the clip draws. With
    /// no such map the copy can't be drawn, and draws nothing.
    fn apply_clip(&self, clip: Option<FlightClip>) {
        let space = self.flying.content_from_box();
        let mut lift = self.lift.0.borrow_mut();
        lift.clip = clip.filter(|_| space.is_some());
        lift.clip_space = space.unwrap_or_default();
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
        self.flying.set_paint(None, None);
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
        // Its nodes are where it rests again.
        crate::css_layout::lift::semantics_moved(&self.element);
    }

    /// Whether the copy, its Screen and its record are all still there, and
    /// the copy still reads as the box it took off as: a structural change
    /// that drops one, or swaps a placement's root, cancels the step.
    pub(super) fn is_whole(&self, rows: &HashSet<CoreHandle>) -> bool {
        self.flying.still_reads()
            && rows.contains(&self.row)
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
/// the copy leaves its own opacity and its ancestors' out. Each version is
/// held at its departure size for the step, as `zoom::Pair::size` lays a
/// zoomed unit out: its place in layout stays its rest, and a size change
/// during the step, from a binding or a parent's percentage, leaves its
/// inside as it took off, the size the pair's box and clip read.
pub(super) struct InPlacePair {
    /// The leaving version's copy, then the arriving one's.
    pub(super) copies: [CoreHandle; 2],
    rule: Rule,
    ends: zoom::Ends,
    /// On Back: the arriving copy's box, H₀⁻¹ (its place at rest,
    /// inverted), c, and q = H₀⁻¹ c. None forward, and when H₀ can't be
    /// inverted.
    tracker: Option<(Flying, Mat2D, Vec2D, Vec2D)>,
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
        let ends = zoom::Ends {
            leaving: leaving.flying.box_matrix(When::Live)?,
            arriving: arriving.flying.box_matrix(When::Rest)?,
            leaving_size: leaving.departure,
            arriving_size: arriving.departure,
            leaving_corners: leaving.flying.corners(),
            arriving_corners: arriving.flying.corners(),
        };
        let tracker = match rule {
            Rule::Leaving => None,
            Rule::Returning => {
                let home = arriving.flying.clone();
                let mut rest_inverse = Mat2D::identity();
                let inverts = home.place(When::Rest)?.invert(&mut rest_inverse)
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
            let now = copy.flying.box_matrix(When::Live)?;
            let matrix = target
                .and_then(|target| lift_matrix(upper, target, None, now, row))
                .filter(|matrix| matrix.values().iter().all(|v| v.is_finite()));
            match matrix {
                Some(matrix) => {
                    self.last[version].set(matrix);
                    copy.apply_pose((matrix, rules[version], drawn_map(upper, matrix, row)));
                    copy.apply_clip(zoom::content_clip(&ends[version], &zoom_box, scales));
                }
                None => copy.apply_pose((self.last[version].get(), 0.0, None)),
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
            let now = home.place(When::Live)?;
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

/// A lift's pose: its transform, its opacity, and where it draws the copy
/// in the copy's own Screen (`drawn_map`), None while it can't be solved.
pub(super) type LiftPose = (Mat2D, f32, Option<(Mat2D, Mat2D)>);

/// Where a lift draws its copy, from a point of the copy at rest to the
/// point the lift draws it at: in the copy's own Screen's drawing space,
/// `row⁻¹ · upper · lift`, which a hit reads, and in the page's,
/// `upper · lift · row⁻¹`, which a semantic bound or a field's geometry
/// reads (css_layout/lift.rs). None when the row can't be inverted.
fn drawn_map(upper: Mat2D, lift: Mat2D, row: Mat2D) -> Option<(Mat2D, Mat2D)> {
    let row_inverse = Wide::of(row).inverse()?;
    let on = Wide::of(upper) * Wide::of(lift);
    let drawn = (row_inverse * on).narrow();
    let page = (on * row_inverse).narrow();
    [drawn, page]
        .iter()
        .all(|map| map.values().iter().all(|v| v.is_finite()))
        .then_some((drawn, page))
}

/// A lift's transform, `upper⁻¹ · pose · own⁻¹ · row`: from the copy's own
/// Screen's drawing space to the upper Screen's, where `own` is the copy's
/// place now. `inner`, the copy's own transform and the one it draws with
/// instead, `(O, O')`, puts `O' · O⁻¹` after the pose, so the copy draws
/// `pose · O'`. It cancels matrices by their inverses, so it is composed in
/// f64 (`wide.rs`). None when one of them has no inverse.
fn lift_matrix(
    upper: Mat2D,
    pose: Mat2D,
    inner: Option<(Mat2D, Mat2D)>,
    own: Mat2D,
    row: Mat2D,
) -> Option<Mat2D> {
    let mut lift = Wide::of(upper).inverse()? * Wide::of(pose);
    if let Some((own_transform, drawn)) = inner {
        lift = lift * Wide::of(drawn) * Wide::of(own_transform).inverse()?;
    }
    Some((lift * Wide::of(own).inverse()? * Wide::of(row)).narrow())
}

#[cfg(test)]
#[path = "in_place_tests.rs"]
mod tests;
