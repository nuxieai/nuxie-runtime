use super::transport::{child, read, size};
use crate::{
    Artboard, CoreHandle, RuntimeArtboardInstanceHandle,
    css_layout::{group_opacity, hosted_frame, record},
    source::{
        artboard_component_list::ArtboardComponentList,
        drawable::RuntimeDrawableOccurrence,
        generated::nested_artboard_layout_base::NestedArtboardLayoutBase,
        layout_component::{FlightShadow, LayoutComponent},
        math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
        nested_artboard::NestedArtboard,
        shapes::paint::color::color_argb,
    },
};
use anyhow::{Result, ensure};
use std::collections::HashMap;
use taffy::style::Position;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SizeMode {
    #[default]
    Scale,
    Layout,
}
/// A component's pose in root drawing coordinates. Artboard's existing draw
/// projection includes frame origins, self transforms, full list-row matrices
/// and nested layout placement.
pub(super) fn draw_pose(component: &CoreHandle) -> Result<(Mat2D, f32, f32)> {
    let (board, world, opacity, child_opacity) = component
        .with(|o| {
            let transform = o.as_world_transform_component()?;
            let board = if o.as_artboard().is_some() {
                Some(component.clone())
            } else {
                o.as_component().and_then(|c| c.artboard_handle())
            };
            Some((
                board,
                if o.as_artboard().is_some() {
                    Mat2D::identity()
                } else {
                    transform.world_transform()
                },
                o.as_transform_component()?.render_opacity(),
                o.world_transform_child_opacity().unwrap_or(1.0),
            ))
        })
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("pose has no transform"))?;
    // A box that fades as one picture keeps its fade out of the number its
    // children render at, so the pose takes those fades back.
    let opacity = opacity * group_opacity::held_above(component);
    let matrix = if let Some(board) = board {
        board
            .with_downcast_mut::<Artboard, _>(|board| {
                if let Some(root) = board.semantic_root_matrix() {
                    return root * world;
                }
                // A host callback maps the root: only points can be read.
                let origin = board.semantic_root_transform(world * Vec2D::new(0.0, 0.0));
                let x = board.semantic_root_transform(world * Vec2D::new(1.0, 0.0)) - origin;
                let y = board.semantic_root_transform(world * Vec2D::new(0.0, 1.0)) - origin;
                Mat2D::new(x.x, x.y, y.x, y.y, origin.x, origin.y)
            })
            .ok_or_else(|| anyhow::anyhow!("pose artboard retired"))?
    } else {
        world
    };
    Ok((matrix, opacity, child_opacity))
}
/// A placement's size: its instance's, or while it is empty the one
/// `remember` kept.
fn slot_size(host: &CoreHandle) -> Result<Vec2D> {
    let frozen_size = read(host, |s| {
        s.live_move
            .as_ref()
            .and_then(|state| state.size)
            .or_else(|| s.natural_size_for_layout())
    })?;
    Ok(child(host)?
        .map(|b| size(host, &b))
        .or(frozen_size)
        .unwrap_or_default())
}
/// One read of the keyed controls, shared by size, value and pose projection.
pub(super) struct FlightFrame {
    pub(super) progress: f32,
    path: Vec2D,
    fade: f32,
    blended_size: Vec2D,
    /// The corners the flying element draws in place of its own, when the
    /// controls carry all four (docs/live-moves.md).
    pub(super) corners: Option<[f32; 4]>,
    /// The shadow the flying element draws in place of its own outer shadows,
    /// when the controls carry one (docs/live-moves.md).
    pub(super) shadow: Option<FlightShadow>,
}
impl FlightFrame {
    /// One read of a flight record's controls, with its two ends' sizes `a`
    /// and `b` blended at its progress.
    pub(super) fn read(values: &super::records::Values, a: Vec2D, b: Vec2D) -> Result<Self> {
        let (progress, path_x, path_y, fade, corners, shadow) = (
            values.progress,
            values.path_x,
            values.path_y,
            values.fade,
            values.corners,
            values.shadow,
        );
        ensure!(
            progress.is_finite()
                && path_x.is_finite()
                && path_y.is_finite()
                && fade.is_finite()
                && corners.iter().all(|corner| corner.is_finite())
                && shadow.iter().all(|value| value.is_finite()),
            "non-finite flight controls"
        );
        let t = progress.clamp(0.0, 1.0);
        let mix = |a: f32, b: f32| a + (b - a) * t;
        let blended_size = Vec2D::new(mix(a.x, b.x), mix(a.y, b.y));
        ensure!(
            blended_size.x.is_finite() && blended_size.y.is_finite(),
            "non-finite flight size"
        );
        Ok(Self {
            progress: t,
            path: Vec2D::new(path_x, path_y),
            fade,
            blended_size,
            // A corner below 0 (-1 by default) carries no corners: the unit
            // draws its own.
            corners: corners
                .iter()
                .all(|corner| *corner >= 0.0)
                .then_some(corners),
            shadow: flight_shadow(shadow),
        })
    }

    /// The blend of two ends' sizes at this frame's progress.
    pub(super) fn blended_size(&self) -> Vec2D {
        self.blended_size
    }

    /// The flight's own fade control.
    pub(super) fn fade(&self) -> f32 {
        self.fade
    }

    /// The pose between two ends' page matrices at this frame's progress:
    /// each decomposed, turned the short way round, scale, skew and place
    /// blended, the path's offset added.
    pub(super) fn blend(&self, a: Mat2D, b: Mat2D) -> Mat2D {
        let t = self.progress;
        let mix = |a: f32, b: f32| a + (b - a) * t;
        let (ac, bc) = (a.decompose(), b.decompose());
        let mut blended = ac;
        let angle = (bc.rotation() - ac.rotation() + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        blended.set_rotation(ac.rotation() + angle * t);
        blended.set_scale_x(mix(ac.scale_x(), bc.scale_x()));
        blended.set_scale_y(mix(ac.scale_y(), bc.scale_y()));
        blended.set_skew(mix(ac.skew(), bc.skew()));
        blended.set_x(mix(ac.x(), bc.x()) + self.path.x);
        blended.set_y(mix(ac.y(), bc.y()) + self.path.y);
        Mat2D::compose(&blended)
    }
}
/// The shadow seven controls carry. An alpha below 0 (-1 by default) carries
/// none: the unit draws its own. Alpha and channels round to 8 bits.
fn flight_shadow([alpha, red, green, blue, blur, x, y]: [f32; 7]) -> Option<FlightShadow> {
    let byte = |value: f32| value.clamp(0.0, 255.0).round() as i32;
    (alpha >= 0.0).then(|| FlightShadow {
        color: color_argb(byte(alpha * 255.0), byte(red), byte(green), byte(blue)),
        blur: blur.max(0.0),
        offset: Vec2D::new(x, y),
    })
}

/// Whether a slot is a stacking context in its Screen's paint order, as the
/// sort reads it: it has a CSS record, and its own opacity or the frame at
/// rest in it makes one (css_layout/hosted_frame.rs).
pub(super) fn anchor_context(anchor: &CoreHandle) -> bool {
    anchor
        .with(|o| {
            o.as_container_component()
                .and_then(|slot| record::find_in(slot.children()))
                .is_some()
                && hosted_frame::slot_makes_context(o)
        })
        .unwrap_or(false)
}

/// The two boxes overlap by more than 0.01 on both axes: touching edges do
/// not cover.
fn overlaps(a: Aabb, b: Aabb) -> bool {
    a.left().max(b.left()) + 0.01 < a.right().min(b.right())
        && a.top().max(b.top()) + 0.01 < a.bottom().min(b.bottom())
}

/// Whether a drawable can cover a flight: it will draw, and it is not a
/// box's own layout drawable (whose draw only restores) or a box's proxy
/// with no shape paint. A placement a step emptied has no artboard, so it
/// will not draw.
fn draws_something(item: &RuntimeDrawableOccurrence) -> bool {
    if !item.will_draw() {
        return false;
    }
    match item {
        RuntimeDrawableOccurrence::RuntimeProxy(proxy) => {
            let owner = proxy.borrow().owner_handle();
            owner
                .with(|o| {
                    o.as_layout_component()
                        .is_none_or(|layout| layout.has_shape_paints())
                })
                .unwrap_or(false)
        }
        RuntimeDrawableOccurrence::Authored(handle) => handle
            .with(|o| o.as_layout_component().is_none())
            .unwrap_or(false),
    }
}

/// The bounds of the box (0, 0, `size`) under `transform`, from its corners.
fn transformed_box(transform: Mat2D, size: Vec2D) -> Aabb {
    Aabb::from_points(&[
        transform * Vec2D::new(0.0, 0.0),
        transform * Vec2D::new(size.x, 0.0),
        transform * Vec2D::new(0.0, size.y),
        transform * Vec2D::new(size.x, size.y),
    ])
}

/// Resting boxes on one Screen, in its root's frame. A box is its laid-out
/// box composed up to the Screen root, each level's own transform applied
/// as it is now, or at identity (its rest) when a timeline keys it. A slot's
/// box is its layout box under the transform of the frame at rest in it.
/// None means the drawable counts as covering whatever its geometry: a level
/// on its chain holds keyed layout or follows the flow beside a sibling that
/// does, the Screen root's own layout is keyed, or a level is not CSS (a
/// translated layout or a plain node).
struct Resting {
    board: CoreHandle,
    /// The Screen root's own resting transform: identity, or None when a
    /// timeline keys its layout (its record, or a grid track among its
    /// children), so that every box on the Screen counts as covering. Only
    /// its own mark counts. The root's box is the Screen's, so keyed layout
    /// inside it moves nothing through the root.
    root: Option<Mat2D>,
    worlds: HashMap<CoreHandle, Option<Mat2D>>,
    /// `holds_keyed_layout`'s answer for each box it has read.
    held: HashMap<CoreHandle, bool>,
}
impl Resting {
    fn new(board: &CoreHandle) -> Self {
        let keyed = css_record(board).is_some_and(|record| record::is_layout_keyed(&record));
        Self {
            board: board.clone(),
            root: (!keyed).then(Mat2D::identity),
            worlds: HashMap::new(),
            held: HashMap::new(),
        }
    }

    /// A drawable's resting box: its own slot's, or that of the CSS box it
    /// paints for (its owner, or the nearest box above its component).
    fn drawable_box(&mut self, item: &RuntimeDrawableOccurrence) -> Option<Aabb> {
        let mut component = match item {
            RuntimeDrawableOccurrence::Authored(handle) => handle.clone(),
            RuntimeDrawableOccurrence::RuntimeProxy(proxy) => proxy.borrow().owner_handle(),
        };
        if component
            .with(|o| o.as_nested_artboard().is_some())
            .unwrap_or(false)
        {
            return self.slot_box(&component);
        }
        loop {
            if component == self.board {
                let root = self.root?;
                let size = self.board.with_downcast::<Artboard, _>(|b| {
                    Vec2D::new(b.layout_width(), b.layout_height())
                })?;
                return Some(transformed_box(root, size));
            }
            match component.with(|o| o.as_layout_component().map(|l| l.is_css_mode())) {
                Some(Some(true)) => break,
                Some(Some(false)) => return None,
                _ => {}
            }
            component = component.with(|o| o.component_parent_handle()).flatten()?;
        }
        let world = self.world(&component)?;
        // A box flying in place rests where its hold keeps it.
        let size = match crate::css_layout::flight_hold::held_rest(&component) {
            Some(rest) => Vec2D::new(rest.width(), rest.height()),
            None => component
                .with(|o| {
                    o.as_layout_component().map(|l| {
                        let bounds = l.layout_bounds();
                        Vec2D::new(bounds.width(), bounds.height())
                    })
                })
                .flatten()?,
        };
        Some(transformed_box(world, size))
    }

    /// A CSS slot's resting box: its layout box and size under the
    /// transform of the frame at rest in it.
    fn slot_box(&mut self, slot: &CoreHandle) -> Option<Aabb> {
        let world = self.world(slot)?;
        let rest = read(slot, hosted_frame::frame_at_rest).ok()?;
        let size = slot_size(slot).ok()?;
        Some(transformed_box(
            world * hosted_frame::rest_transform(&rest),
            size,
        ))
    }

    /// A level's resting world transform, from the Screen root down.
    fn world(&mut self, level: &CoreHandle) -> Option<Mat2D> {
        if *level == self.board {
            return self.root;
        }
        if let Some(world) = self.worlds.get(level) {
            return *world;
        }
        let world = self.compose(level);
        self.worlds.insert(level.clone(), world);
        world
    }

    fn compose(&mut self, level: &CoreHandle) -> Option<Mat2D> {
        let parent = level.with(|o| o.component_parent_handle()).flatten()?;
        let (local, record) = level
            .with(|o| {
                if let Some(layout) = o.as_layout_component() {
                    if !layout.is_css_mode() {
                        return None;
                    }
                    // A box flying in place rests where its hold keeps its
                    // place, at its resting size.
                    if let Some((moved, size)) = layout.flight_rest() {
                        return Some((
                            Mat2D::from_translation(layout.layout_translation() + moved)
                                * layout.own_transform_at_rest_sized(size),
                            layout.css_record()?,
                        ));
                    }
                    return Some((
                        Mat2D::from_translation(layout.layout_translation())
                            * layout.own_transform_at_rest(),
                        layout.css_record()?,
                    ));
                }
                let slot = o.as_nested_artboard()?;
                // A plain host or a native layout host is not measured.
                if !level.is_type_of(NestedArtboardLayoutBase::TYPE_KEY) {
                    return None;
                }
                let record = record::find_in(slot.children())?;
                let node = o.as_transform_component()?.transform();
                // The slot's own placement (nested_artboard_layout.rs): its
                // layout box, and a pivot that stays put when it turns. A
                // placed root flying in place rests where its hold keeps it,
                // its origin and pivot taken from its resting size.
                let (mut base, pivot) = slot.live_move_layout_instance()?.with_artboard(|i| {
                    let (at, size) = i.resting_layout();
                    let origin = if i.frame_origin() {
                        Vec2D::default()
                    } else {
                        -i.content_origin_at(size)
                    };
                    (
                        at - origin,
                        i.origin_is_pivot_only().then(|| {
                            Vec2D::new(i.pivot_origin_x() * size.x, i.pivot_origin_y() * size.y)
                        }),
                    )
                });
                if let Some(origin) = parent
                    .with(|parent| parent.as_artboard().map(Artboard::origin))
                    .flatten()
                {
                    base += origin;
                }
                let mut local = Mat2D::from_translation(base) * node;
                if let Some(p) = pivot
                    && (node[0] != 1.0 || node[1] != 0.0 || node[2] != 0.0 || node[3] != 1.0)
                {
                    local[4] += p.x - (node[0] * p.x + node[2] * p.y);
                    local[5] += p.y - (node[1] * p.x + node[3] * p.y);
                }
                Some((local, record))
            })
            .flatten()?;
        if self.holds_keyed_layout(level) || self.pushed_by_keyed_layout(&[level], &record, &parent)
        {
            return None;
        }
        Some(self.world(&parent)? * local)
    }

    /// Whether layout can read a keyed record for this box's size or place:
    /// its own record is keyed (a grid track or placement marks it), or a box
    /// in flow inside it holds keyed layout, as layout's own children go. A
    /// translated box has no record and counts by what it holds. A slot
    /// counts by both records, the host's here and its frame's as its child.
    /// Conservative on purpose: a fixed-height wrapper counts too, as
    /// covering is the safe side.
    fn holds_keyed_layout(&mut self, item: &CoreHandle) -> bool {
        if let Some(held) = self.held.get(item) {
            return *held;
        }
        // A box flying in place counts by its own record alone, as a slot
        // whose frame flies holds only its placeholder: a flown element's
        // own keys reach no one at rest.
        let flying = item
            .with(|o| o.as_layout_component().map(|l| l.flight_rest().is_some()))
            .flatten()
            .unwrap_or(false);
        let held = css_record(item).is_some_and(|record| record::is_layout_keyed(&record))
            || (!flying
                && layout_children_in_flow(item)
                    .iter()
                    .any(|child| self.holds_keyed_layout(child)));
        self.held.insert(item.clone(), held);
        held
    }

    /// Reads `slot` as a slot whose frame flies, which holds only its
    /// place: its frame's own keyed layout, and its descendants', count for
    /// nothing at rest.
    fn frame_flies(&mut self, slot: &CoreHandle) {
        if let Some(frame) = layout_instance(slot) {
            self.held.insert(frame.core_handle(), false);
        }
    }

    /// Whether a sibling in flow that holds keyed layout can move `level`,
    /// because `level` follows the flow. `level` names the level as its
    /// parent's layout lists it: itself, and the node it holds its place
    /// with when that is another (a participant, a placed root).
    fn pushed_by_keyed_layout(
        &mut self,
        level: &[&CoreHandle],
        own: &CoreHandle,
        parent: &CoreHandle,
    ) -> bool {
        record::with_values(own, |css| css.follows_the_flow()).unwrap_or(false)
            && layout_children_in_flow(parent)
                .iter()
                .any(|sibling| !level.contains(&sibling) && self.holds_keyed_layout(sibling))
    }

    /// Whether keyed layout can move or size `element`, which holds its
    /// place with `node`, as `compose` asks of every level it reads: its own
    /// record is keyed, or a box in flow inside it holds keyed layout, or a
    /// sibling in flow that does can push it. Then it has no resting place,
    /// and covering is the safe side. So it is with no record or parent.
    fn moved_by_keyed_layout(&mut self, element: &CoreHandle, node: &CoreHandle) -> bool {
        let parent = element.with(|o| o.component_parent_handle()).flatten();
        let (Some(record), Some(parent)) = (css_record(element), parent) else {
            return true;
        };
        self.holds_keyed_layout(element)
            || self.pushed_by_keyed_layout(&[element, node], &record, &parent)
    }
}

/// What covers each of `anchors`, lifted copies' anchors on the upper Screen
/// `board`, each with the element its lift sorts as (the anchor, or the
/// placement a transferring unit carries in it) and what take-off recorded
/// of where that placement rests (`TakeOff`): walking its sorted list from
/// where that element paints (`paint_order::anchor_positions`, as the sort
/// places its lift) toward the top, the first that draws something and covers what the
/// anchor draws at rest (`cover_box`). A lift entry, an entry of a lifted
/// copy (`lifted`, which draws in its lift) and an entry of the anchor
/// itself never cover it. One frame reads every anchor on a Screen at once:
/// the list, each entry's walk up to the Screen (`draws_for` for every
/// anchor in one walk) and the resting boxes are read once for all of them.
/// Both copies of a pair share their upper Screen's anchor, so they share
/// its cover.
pub(super) fn lift_covers(
    board: &CoreHandle,
    anchors: &[(CoreHandle, CoreHandle, TakeOff)],
    lifted: &std::collections::HashSet<RuntimeDrawableOccurrence>,
) -> Vec<Option<RuntimeDrawableOccurrence>> {
    use crate::css_layout::lift::lift_of;
    let list = board
        .with_downcast::<Artboard, _>(|board| board.drawables().to_vec())
        .unwrap_or_default();
    let mut index = HashMap::new();
    let mut unique: Vec<&(CoreHandle, CoreHandle, TakeOff)> = Vec::new();
    let of = anchors
        .iter()
        .map(|pair| {
            *index.entry(&pair.0).or_insert_with(|| {
                unique.push(pair);
                unique.len() - 1
            })
        })
        .collect::<Vec<_>>();
    // For each entry, the anchors it draws for; None for a lift entry.
    let draws = list
        .iter()
        .map(|item| {
            if lift_of(item).is_some() {
                return None;
            }
            let mut found = Vec::new();
            let mut node = Some(match item {
                RuntimeDrawableOccurrence::Authored(handle) => handle.clone(),
                RuntimeDrawableOccurrence::RuntimeProxy(proxy) => proxy.borrow().owner_handle(),
            });
            while let Some(current) = node {
                if let Some(&n) = index.get(&current) {
                    found.push(n);
                }
                if current == *board {
                    break;
                }
                node = current
                    .with(|object| object.component_parent_handle())
                    .flatten();
            }
            Some(found)
        })
        .collect::<Vec<_>>();
    let mut rest = Resting::new(board);
    let elements: Vec<CoreHandle> = unique
        .iter()
        .map(|(_, sorts_as, _)| sorts_as.clone())
        .collect();
    let positions = crate::css_layout::paint_order::anchor_positions(
        board,
        crate::css_layout::paint_order::container_of(board),
        &list,
        &elements,
    );
    let covers = (0..unique.len())
        .map(|n| {
            let at = positions[n];
            let (anchor, _, take_off) = unique[n];
            let anchor_box = cover_box(&mut rest, anchor, take_off);
            list[..at]
                .iter()
                .zip(&draws[..at])
                .rev()
                .find(|(item, found)| {
                    found.as_ref().is_some_and(|found| !found.contains(&n))
                        && !lifted.contains(item)
                        && draws_something(item)
                        && match (anchor_box, rest.drawable_box(item)) {
                            (Some(a), Some(b)) => overlaps(a, b),
                            _ => true,
                        }
                })
                .map(|(item, _)| item.clone())
        })
        .collect::<Vec<_>>();
    of.into_iter().map(|n| covers[n].clone()).collect()
}

/// What a lifted copy's anchor draws at rest on its Screen: its resting
/// box, joined for a transfer with the box the placement it carries draws
/// at rest (`PlacedAtRest`). None, covered by whatever it meets, when keyed
/// layout moves the anchor, or when where a transfer's placement rests is
/// unknown.
fn cover_box(rest: &mut Resting, anchor: &CoreHandle, take_off: &TakeOff) -> Option<Aabb> {
    let resting = anchor_box(rest, anchor)?;
    let drawn = match take_off {
        TakeOff::NoTransfer => return Some(resting),
        TakeOff::Unknown => return None,
        TakeOff::Recorded(placed) => match placed.drawn(rest, anchor) {
            Drawn::Box(drawn) => drawn,
            Drawn::Nothing => return Some(resting),
            Drawn::Unknown => return None,
        },
    };
    Some(Aabb::new(
        resting.left().min(drawn.left()),
        resting.top().min(drawn.top()),
        resting.right().max(drawn.right()),
        resting.bottom().max(drawn.bottom()),
    ))
}

/// What a lift's cover knows, from take-off, of where the placement it
/// sorts as rests inside its anchor. Every reader matches all three, so a
/// record that failed is never read as no transfer.
#[derive(Clone)]
pub(super) enum TakeOff {
    /// Not a transfer: the lift carries no placement, and its anchor's
    /// resting box is what it draws at rest.
    NoTransfer,
    /// A transfer, and where its placement rests, recorded at take-off.
    Recorded(PlacedAtRest),
    /// A transfer whose placement had no resting place at take-off, read
    /// as a slot whose frame flies is read (`PlacedAtRest::record`): its own
    /// host record is keyed, a keyed sibling pushes it, or a level it rests on
    /// is not measured. Unknown, so it is covered by whatever it meets, the
    /// safe side `Resting` takes.
    Unknown,
}

/// What a recorded placement draws at rest now.
enum Drawn {
    Box(Aabb),
    /// The anchor's clip leaves nothing of it.
    Nothing,
    /// The anchor has no resting place now, or the instance is gone.
    Unknown,
}

/// Where the placement a transfer carries rests inside its lift's anchor,
/// recorded at take-off, before anything moves, as `remember` keeps a slot's
/// size and pivot: its place in the anchor's resting space, its size, the
/// pivot and gate its own frame placed by, and the anchor's padding box,
/// which its clip cuts to. While the anchor flies, what is inside it lays
/// out for the flight, so the cover reads none of it: it puts the carried
/// instance's own turn and scale on these facts, in the anchor's resting
/// space, as `slot_box` reads a slot's.
#[derive(Clone)]
pub(super) struct PlacedAtRest {
    /// The instance the transfer carries, whose frame the lift draws,
    /// whatever fills the placement later.
    instance: crate::RuntimeArtboardInstanceWeakHandle,
    inset: Mat2D,
    size: Vec2D,
    pivot: Option<Vec2D>,
    gate: bool,
    padding: Aabb,
}

impl PlacedAtRest {
    /// `placement`, a child of `anchor` (`hosted_scene::boxed_placement`),
    /// at rest inside it, read now, for the carried `instance`. It is read
    /// as a slot whose frame flies: the frame in it flies, so its own keyed
    /// layout and its descendants' reach no one at rest
    /// (`Resting::frame_flies`). Unknown only when the placement or the anchor
    /// has no resting place even so: the placement's own host record is keyed,
    /// a keyed sibling pushes it, or a level is not measured.
    pub(super) fn record(
        anchor: &CoreHandle,
        placement: &CoreHandle,
        instance: crate::RuntimeArtboardInstanceWeakHandle,
    ) -> TakeOff {
        Self::read(anchor, placement, instance).map_or(TakeOff::Unknown, TakeOff::Recorded)
    }

    fn read(
        anchor: &CoreHandle,
        placement: &CoreHandle,
        instance: crate::RuntimeArtboardInstanceWeakHandle,
    ) -> Option<Self> {
        let board = anchor
            .with(|o| o.as_component().and_then(|c| c.artboard_handle()))
            .flatten()?;
        let mut rest = Resting::new(&board);
        rest.frame_flies(placement);
        let (anchor_world, world) = (rest.world(anchor)?, rest.world(placement)?);
        let mut inverse = Mat2D::identity();
        if !anchor_world.invert(&mut inverse) {
            return None;
        }
        let frame = read(placement, hosted_frame::frame_at_rest).ok()?;
        let padding = anchor
            .with(|o| o.as_layout_component()?.css_box_geometry())
            .flatten()?
            .padding;
        Some(Self {
            instance,
            inset: inverse * world,
            size: slot_size(placement).ok()?,
            pivot: hosted_frame::rest_pivot(&frame),
            gate: frame.gate,
            padding,
        })
    }

    /// The box it draws at rest, in the Screen's space: the anchor's
    /// resting world, its recorded place, and the carried instance's turn
    /// and scale about its recorded pivot, at its recorded size, cut to
    /// what the anchor's clip cuts to now, by paint's own rule
    /// (`paint::overflow_clip_rect`) on its recorded padding box.
    fn drawn(&self, rest: &mut Resting, anchor: &CoreHandle) -> Drawn {
        let (Some(world), Some(instance)) = (rest.world(anchor), self.instance.upgrade()) else {
            return Drawn::Unknown;
        };
        let frame = instance.core_handle();
        let turn = hosted_frame::rest_transform_about(Some(&frame), self.gate, self.pivot);
        let clips = anchor
            .with(|o| o.as_layout_component().map(|l| l.clipped_axes()))
            .flatten()
            .unwrap_or_default();
        let Some(clip) = crate::css_layout::paint::overflow_clip_rect(self.padding, clips) else {
            return Drawn::Box(transformed_box(world * self.inset * turn, self.size));
        };
        let inside = transformed_box(self.inset * turn, self.size);
        let (left, top) = (inside.left().max(clip.left()), inside.top().max(clip.top()));
        let (right, bottom) = (
            inside.right().min(clip.right()),
            inside.bottom().min(clip.bottom()),
        );
        if left < right && top < bottom {
            Drawn::Box(transformed_box(
                world * Mat2D::from_translation(Vec2D::new(left, top)),
                Vec2D::new(right - left, bottom - top),
            ))
        } else {
            Drawn::Nothing
        }
    }
}

/// A lifted copy's anchor's resting box on its Screen. An anchor that is
/// not a box rests in its own box, not its nearest box's (in_place.rs,
/// `Flying`).
fn anchor_box(rest: &mut Resting, anchor: &CoreHandle) -> Option<Aabb> {
    let flying = super::in_place::Flying::read(anchor);
    match flying.as_ref().and_then(|f| Some((f, f.rest_anchor()?))) {
        Some((flying, (level, inside, size))) => {
            // Its own place is a resting place only as a level's is.
            if rest.moved_by_keyed_layout(anchor, flying.layout_node()) {
                return None;
            }
            rest.world(&level)
                .map(|world| transformed_box(world * inside, size))
        }
        None => rest.drawable_box(&RuntimeDrawableOccurrence::Authored(anchor.clone())),
    }
}

/// `anchor`'s resting box on `board`, as its cover reads it.
#[cfg(test)]
pub(super) fn resting_anchor_box(board: &CoreHandle, anchor: &CoreHandle) -> Option<Aabb> {
    anchor_box(&mut Resting::new(board), anchor)
}

/// The matrix placement's resting boxes give a CSS box at rest, in its
/// Screen's space: its resting world, which for a box flying in place is
/// where its hold keeps it.
#[cfg(feature = "test-support")]
pub(super) fn resting_box(element: &CoreHandle) -> Option<Mat2D> {
    let board = element
        .with(|o| o.as_component().and_then(|c| c.artboard_handle()))
        .flatten()?;
    Resting::new(&board).world(element)
}

/// The CSS record layout reads `item`'s style from: a box's own, or for a
/// slot the one among its children.
fn css_record(item: &CoreHandle) -> Option<CoreHandle> {
    item.with(|o| match o.as_layout_component() {
        Some(layout) => layout.css_record(),
        None => o
            .as_container_component()
            .and_then(|c| record::find_in(c.children())),
    })
    .flatten()
}

/// What layout flows directly inside `item`. It reads layout's own list and
/// changes nothing: `layout_providers_occurrence` sees through plain nodes
/// and Solo, and the two reads `layout_node_owner_for` makes stand in for
/// `layout_node_for`, which takes a node's layout data.
/// - A list gives its rows, except a collapsed one, which has no node.
/// - A slot holds the one artboard its layout places: its frame, or while it
///   is empty the placeholder `remember` kept, never the frame in flight.
///   That placeholder holds nothing, so a flown frame's keys reach no one.
fn layout_children_in_flow(item: &CoreHandle) -> Vec<CoreHandle> {
    if let Some(instance) = layout_instance(item) {
        return vec![instance.core_handle()];
    }
    if item
        .with(|o| o.as_layout_component().is_none())
        .unwrap_or(true)
    {
        return Vec::new();
    }
    let mut children = Vec::new();
    for (child, _) in LayoutComponent::layout_providers_occurrence(item) {
        let rows = child.with_downcast::<ArtboardComponentList, _>(|list| {
            (0..list.item_count())
                .filter(|&row| !list.live_move_row_collapsed(row))
                .filter_map(|row| list.item(row))
                .map(|row| row.core_handle())
                .collect::<Vec<_>>()
        });
        match rows {
            Some(rows) => children.extend(rows),
            None => children.push(child),
        }
    }
    children.retain(in_flow);
    children
}

/// The artboard a slot's layout places, when `item` is a slot with one.
fn layout_instance(item: &CoreHandle) -> Option<RuntimeArtboardInstanceHandle> {
    item.with(|o| {
        o.as_nested_artboard()
            .and_then(NestedArtboard::live_move_layout_instance)
    })
    .flatten()
}

/// Whether layout flows `child`, by the position in the style it builds that
/// node from. Whatever sets the position writes it there: a box's or a row's
/// own record, a slot's child record for the frame it places, a translated
/// box's own style. A shape, text or picture has no such style here, so its
/// record says, and without one it holds nothing that could count.
fn in_flow(child: &CoreHandle) -> bool {
    let node = layout_instance(child).map_or_else(|| child.clone(), |i| i.core_handle());
    let absolute = node
        .with(|o| {
            o.as_layout_component()
                .map(|layout| layout.layout_style().position == Position::Absolute)
        })
        .flatten();
    match absolute {
        Some(absolute) => !absolute,
        None => css_record(child)
            .and_then(|record| record::with_values(&record, |css| css.in_flow()))
            .unwrap_or(false),
    }
}
