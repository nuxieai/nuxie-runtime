//! The lift of an element flying in place (docs/live-moves.md, Flying in
//! place).
//!
//! While a matched element flies in place, each Screen's copy stays in its
//! own Screen's tree. Its own Screen's draw leaves its entries out, and the
//! upper Screen draws it from one runtime-only entry of its own sorted draw
//! list, the lift entry, which `paint_order::place` puts where the element
//! paints at rest. The lift entry draws the copy's own entries of its own
//! Screen's list and nothing else, so none of its ancestors' clips opens,
//! under the flight's transform and opacity: save, the transform, the opacity
//! as one picture or a scope, inside it the zoom's clip when there is one and
//! the entries, restore.
//!
//! Which entries draw for each lifted copy is worked out where its Screen's
//! draw order is sorted (`Artboard::sort_draw_order`), so it is never older
//! than the list. Nothing here is read for a file without flight records.
use super::group_opacity::GroupPicture;
use crate::source::{
    artboard::{Artboard, FlightClip},
    core::CoreHandle,
    drawable::{DrawableProxy, ProxyDrawing, RuntimeDrawableOccurrence},
    math::mat2d::Mat2D,
    renderer::Renderer,
};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

/// One lifted copy: what its lift entry draws and where.
pub(crate) struct LiftState {
    /// The Screen that holds the copy.
    pub(crate) row: CoreHandle,
    /// The copy: the element whose entries the lift draws.
    pub(crate) element: CoreHandle,
    /// The upper Screen's own copy, where the lift paints.
    pub(crate) anchor: CoreHandle,
    /// A transferring unit's lift: the component placement it carries inside
    /// the anchor. The lift sorts as that placement sorts, while its cover
    /// still reads the anchor.
    pub(crate) placed_as: Option<CoreHandle>,
    /// What covers the anchor on the upper Screen, read each frame; None
    /// paints the lift on top.
    pub(crate) coverer: Option<RuntimeDrawableOccurrence>,
    /// From the copy's Screen's drawing space to the upper Screen's, with the
    /// flight's pose.
    pub(crate) matrix: Mat2D,
    /// The flight's opacity, applied to the copy's entries as one picture or
    /// a scope.
    pub(crate) opacity: f32,
    /// A zoom's box, in the element's box (live_move/in_place.rs, `Flying`).
    pub(crate) clip: Option<FlightClip>,
    /// From the element's box to its own drawing space, where its world
    /// draws: identity for a CSS box.
    pub(crate) clip_space: Mat2D,
    /// The copy's entries in its Screen's draw order, set by its sort.
    pub(crate) entries: Vec<RuntimeDrawableOccurrence>,
    pub(crate) picture: GroupPicture,
    /// This lift's entry in the upper Screen's list.
    entry: Option<RuntimeDrawableOccurrence>,
}

#[derive(Clone)]
pub(crate) struct Lift(pub(crate) Rc<RefCell<LiftState>>);

impl Lift {
    pub(crate) fn new(row: CoreHandle, element: CoreHandle, anchor: CoreHandle) -> Self {
        Self(Rc::new(RefCell::new(LiftState {
            row,
            element,
            anchor,
            placed_as: None,
            coverer: None,
            matrix: Mat2D::identity(),
            opacity: 1.0,
            clip: None,
            clip_space: Mat2D::identity(),
            entries: Vec::new(),
            picture: GroupPicture::default(),
            entry: None,
        })))
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    /// The lift's entry in the upper Screen's list: one runtime-only
    /// drawable, made once per lift.
    pub(crate) fn entry(&self) -> RuntimeDrawableOccurrence {
        if let Some(entry) = self.0.borrow().entry.clone() {
            return entry;
        }
        let entry = RuntimeDrawableOccurrence::RuntimeProxy(Rc::new(RefCell::new(
            DrawableProxy::new(Box::new(LiftEntry {
                lift: Rc::downgrade(&self.0),
                owner: self.0.borrow().anchor.clone(),
            })),
        )));
        self.0.borrow_mut().entry = Some(entry.clone());
        entry
    }
}

/// The lift entry's drawing. A lift is drawn by the range walk, which knows
/// the copy's Screen (`Artboard::draw_lift`); drawn any other way it draws
/// nothing. It answers no hit: the copy's own hit components stay on its
/// own Screen.
struct LiftEntry {
    lift: std::rc::Weak<RefCell<LiftState>>,
    owner: CoreHandle,
}

impl ProxyDrawing for LiftEntry {
    fn draw_proxy(&mut self, _renderer: &mut Renderer, _needs_save_operation: bool) {}
    fn is_proxy_hidden(&self) -> bool {
        false
    }
    fn owner_handle(&self) -> CoreHandle {
        self.owner.clone()
    }
    fn hittable_component(&self) -> Option<CoreHandle> {
        None
    }
    fn lift(&self) -> Option<Lift> {
        self.lift.upgrade().map(Lift)
    }
}

/// The lift an entry is, when it is a lift entry.
pub(crate) fn lift_of(entry: &RuntimeDrawableOccurrence) -> Option<Lift> {
    entry
        .with_proxy(|proxy| proxy.proxy_drawing().lift())
        .flatten()
}

/// Whether `entry` draws for `element`, an element of the artboard `root`:
/// it is the element or a proxy it owns, or it sits under it. A lift entry
/// draws for nothing. It reads no object but `entry`'s ancestors below
/// `root`, so a sort of `root` can ask.
pub(crate) fn draws_for(
    entry: &RuntimeDrawableOccurrence,
    element: &CoreHandle,
    root: &CoreHandle,
) -> bool {
    if lift_of(entry).is_some() {
        return false;
    }
    let mut node = match entry {
        RuntimeDrawableOccurrence::Authored(handle) => Some(handle.clone()),
        RuntimeDrawableOccurrence::RuntimeProxy(proxy) => Some(proxy.borrow().owner_handle()),
    };
    while let Some(current) = node {
        if current == *element {
            return true;
        }
        if current == *root {
            return false;
        }
        node = current
            .with(|object| object.component_parent_handle())
            .flatten();
    }
    false
}

/// The lifts a Screen takes part in during a step: those it draws, whose
/// anchor is on it, and those of its own copies, which its draw leaves out.
/// Set at a step's take-off and cleared at its end; each change sorts the
/// Screen again.
#[derive(Default)]
pub(crate) struct ScreenLifts {
    /// The lifts this Screen draws, in placing order: a later flight record
    /// first, so it draws on top.
    pub(crate) drawn: Vec<Lift>,
    /// The lifts of this Screen's own copies.
    pub(crate) away: Vec<Lift>,
    /// Every entry of this Screen's list that draws for one of `away`, set
    /// by the sort.
    pub(crate) skip: Rc<HashSet<RuntimeDrawableOccurrence>>,
    /// The last sort placed lift entries or marked skipped ones.
    marked: bool,
}

impl ScreenLifts {
    pub(crate) fn is_empty(&self) -> bool {
        self.drawn.is_empty() && self.away.is_empty()
    }

    /// Whether the next sort has lifts to place or mark, or marks of the
    /// last one to clear.
    pub(crate) fn touched(&self) -> bool {
        self.marked || !self.is_empty()
    }
}

/// Marks a sorted list's lifted entries: for each lift of this Screen's own
/// copies, its entries in draw order, and the set the Screen's draw leaves
/// out. `first` is the sorted list of `root`'s first entry in draw order.
pub(crate) fn mark_entries(
    lifts: &mut ScreenLifts,
    first: Option<RuntimeDrawableOccurrence>,
    root: &CoreHandle,
) {
    lifts.marked = !lifts.is_empty();
    if lifts.away.is_empty() {
        if !lifts.skip.is_empty() {
            lifts.skip = Rc::default();
        }
        return;
    }
    #[cfg(feature = "test-support")]
    MARKS.with(|count| count.set(count.get() + 1));
    let mut entries = vec![Vec::new(); lifts.away.len()];
    let mut skip = HashSet::new();
    let mut current = first;
    while let Some(entry) = current {
        current = entry
            .with(crate::source::drawable::Drawable::prev_drawable)
            .flatten();
        for (index, lift) in lifts.away.iter().enumerate() {
            if draws_for(&entry, &lift.0.borrow().element, root) {
                entries[index].push(entry.clone());
                skip.insert(entry.clone());
                break;
            }
        }
    }
    for (lift, entries) in lifts.away.iter().zip(entries) {
        lift.0.borrow_mut().entries = entries;
    }
    lifts.skip = Rc::new(skip);
}

#[cfg(feature = "test-support")]
thread_local! {
    static MARKS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// How many times a sort marked lifted entries on this thread.
#[cfg(feature = "test-support")]
pub fn lift_marks() -> usize {
    MARKS.with(std::cell::Cell::get)
}

/// Starts or ends a lifted copy's opacity root: while set, `element` renders
/// without its ancestors' opacity, which the lift applies instead, and with
/// `own` set it leaves its own out too, as a zoomed pair's version does.
pub(crate) fn set_opacity_root(element: &CoreHandle, root: OpacityRoot) {
    use crate::source::component_dirt::ComponentDirt;
    let changed = element
        .with_mut(|object| {
            let transform = object.as_transform_component_mut()?;
            let changed = transform.opacity_root != root;
            transform.opacity_root = root;
            Some(changed)
        })
        .flatten()
        .unwrap_or(false);
    if changed {
        crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
            element.clone(),
        )
        .add_dirt(ComponentDirt::RENDER_OPACITY, true);
    }
}

/// Where a lifted copy's render opacity starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum OpacityRoot {
    /// Not lifted: from its parent, as authored.
    #[default]
    None,
    /// From itself: its ancestors' opacity is left out.
    Own,
    /// From 1: its ancestors' and its own opacity are left out.
    Nothing,
}

/// Runs `body` with `row`'s draw marked as a lift's, so its ranges keep the
/// entries its own draw leaves out, and with `visitor` as its draw visitor.
pub(crate) fn drawing_lift(
    row: &CoreHandle,
    visitor: Option<crate::source::artboard::RuntimeDrawVisitor>,
    body: impl FnOnce(),
) {
    let previous = row
        .with_downcast_mut::<Artboard, _>(|artboard| {
            artboard.drawing_lift += 1;
            std::mem::replace(&mut artboard.draw_visitor, visitor)
        })
        .flatten();
    struct Restore<'a>(
        &'a CoreHandle,
        Option<crate::source::artboard::RuntimeDrawVisitor>,
    );
    impl Drop for Restore<'_> {
        fn drop(&mut self) {
            let previous = self.1.take();
            self.0.with_downcast_mut::<Artboard, _>(|artboard| {
                artboard.drawing_lift -= 1;
                artboard.draw_visitor = previous;
            });
        }
    }
    let _restore = Restore(row, previous);
    body();
}

/// The world of `element`, the space its zoom clip is in.
fn element_world(element: &CoreHandle) -> Option<Mat2D> {
    element
        .with(|object| {
            object
                .as_world_transform_component()
                .map(|transform| transform.world_transform())
        })
        .flatten()
}

/// The bounds in `row`'s drawing space of a zoom's box, `clip`, in
/// `element`'s box, which `space` takes to its own drawing space: what
/// bounds the lift's picture.
pub(crate) fn clip_bounds(
    element: &CoreHandle,
    clip: FlightClip,
    space: Mat2D,
) -> Option<crate::source::math::aabb::Aabb> {
    Some((element_world(element)? * space).map_bounding_box(clip.area()))
}

/// Clips the renderer to a zoom's box, `clip`, in `element`'s box, which
/// `space` takes to its own drawing space, under the element's world, so it
/// turns and stretches with the copy.
/// The box reaches the renderer as its own outline under the element's world,
/// the input the element's paint draws with, so the clip covers the samples
/// the fill covers and a rounded edge is smoothed once. The world is taken
/// back after the clip, so the copy's entries draw under the lift's transform
/// alone. An element whose world collapses draws nothing, and takes no clip.
pub(crate) fn apply_clip(
    renderer: &mut Renderer,
    element: &CoreHandle,
    row: &CoreHandle,
    clip: FlightClip,
    space: Mat2D,
) {
    use crate::source::{math::raw_path::RawPath, shapes::paint::shape_paint_path::ShapePaintPath};
    let (Some(world), Some(factory)) = (
        element_world(element).map(|world| world * space),
        row.with_downcast::<Artboard, _>(Artboard::factory)
            .flatten(),
    ) else {
        return;
    };
    let mut raw = RawPath::default();
    super::paint::add_rounded_rect(&mut raw, clip.area(), clip.radii);
    let mut inverse = Mat2D::default();
    if !world.invert(&mut inverse) {
        return;
    }
    let mut path = ShapePaintPath::new(false);
    path.add_path(&raw, None);
    renderer.transform(nuxie_render_api::Mat2D(*world.values()));
    renderer.clip_path(path.render_path(&factory));
    renderer.transform(nuxie_render_api::Mat2D(*inverse.values()));
}
