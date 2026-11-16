//! The lift of an element flying in place (docs/live-moves.md, Flying in
//! place).
//!
//! While a matched element flies in place, each Screen's copy stays in its
//! own Screen's tree. Its own Screen's draw leaves its entries out, and the
//! upper Screen draws it from one runtime-only entry of its own sorted draw
//! list, the lift entry, which `paint_order::place` puts where a flight slot
//! would go. The lift entry draws the copy's own entries of its own Screen's
//! list and nothing else, so none of its ancestors' clips opens, under the
//! flight's transform and opacity: save, the transform, the opacity as one
//! picture or a scope, inside it the zoom's clip when there is one and the
//! entries, restore.
//!
//! Which entries draw for each lifted copy is worked out where its Screen's
//! draw order is sorted (`Artboard::sort_draw_order`), so it is never older
//! than the list. Nothing here is read for a file without flight records.
use super::group_opacity::GroupPicture;
use crate::source::{
    artboard::{Artboard, FlightClip},
    core::CoreHandle,
    drawable::{DrawableProxy, ProxyDrawing, RuntimeDrawableOccurrence},
    math::{mat2d::Mat2D, vec2d::Vec2D},
    renderer::Renderer,
};
use std::{cell::RefCell, collections::HashSet, rc::Rc};

/// One lifted copy: what its lift entry draws and where.
pub(crate) struct LiftState {
    /// The Screen that holds the copy.
    pub(crate) row: CoreHandle,
    /// The upper Screen, which draws the lift: `row` for the upper copy.
    /// Read through `Lift::drawing_screen`.
    board: CoreHandle,
    /// The copy: the element whose entries the lift draws.
    pub(crate) element: CoreHandle,
    /// The upper Screen's own copy, where the lift paints.
    pub(crate) anchor: CoreHandle,
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
    /// Where the lift draws the copy, in the copy's own Screen's drawing
    /// space: from a point of the copy at rest to where it is drawn now.
    /// None while the lift can't be solved and draws nothing.
    drawn: Option<Mat2D>,
    /// `drawn` in the page's space: from where the page shows a point of the
    /// copy at rest to where it shows it drawn, which a semantic bound or a
    /// field's geometry, read through its Screen's root, takes last.
    page: Option<Mat2D>,
    /// This lift's entry in the upper Screen's list.
    entry: Option<RuntimeDrawableOccurrence>,
}

#[derive(Clone)]
pub(crate) struct Lift(pub(crate) Rc<RefCell<LiftState>>);

impl LiftState {
    /// The Pose barrier's map of where the lift draws its copy
    /// (live_move/in_place.rs, `drawn_map`).
    /// Returns whether where the lift draws the copy changed.
    pub(crate) fn set_drawn(&mut self, maps: Option<(Mat2D, Mat2D)>) -> bool {
        // A map that can't be undone draws nothing.
        let maps = maps.filter(|(drawn, _)| drawn.invert(&mut Mat2D::default()));
        let before = self.drawn;
        self.drawn = maps.map(|(drawn, _)| drawn);
        self.page = maps.map(|(_, page)| page);
        self.drawn != before
    }
}

/// Asks the semantic tree to read again where `element`'s nodes are, as a
/// lift starts drawing it elsewhere, moves it or stops: the nearest node at
/// or above the element and every node under that. Nothing else tells it,
/// since the copy's own world never moves.
pub(crate) fn semantics_moved(element: &CoreHandle) {
    use crate::mechanical_port::source::semantic::semantic_data::SemanticData;
    let Some(node) = SemanticData::find_closest_semantic_node_handle(Some(element.clone())) else {
        return;
    };
    let (manager, id) = {
        let node = node.borrow();
        (node.manager(), node.id())
    };
    if let Some(manager) = manager {
        manager.with_semantic_manager_mut(|manager| manager.mark_boundary_dirty(id));
    }
}

/// The Screen that draws a lifted copy: the upper Screen, which draws every
/// lift as it holds every flight slot. Above the copy, every reader starts
/// here and not on the copy's own Screen (`DrawnAncestors`): covers,
/// drawing and the pose, visibility, clips and semantic bounds. It answers
/// where its space sits in the copy's own Screen's.
pub(crate) struct DrawingScreen {
    pub(crate) screen: CoreHandle,
    /// The lift's matrix: from the copy's own Screen's space to this one's,
    /// with the flight's pose.
    pub(crate) matrix: Mat2D,
    /// The lift's drawn map: None while it draws nothing.
    drawn: Option<Mat2D>,
}

impl DrawingScreen {
    /// From its space to the copy's own Screen's, as the pose places the two
    /// (`drawn · matrix⁻¹`): the identity for the upper copy. None while the
    /// lift draws nothing.
    pub(crate) fn to_copy_screen(&self) -> Option<Mat2D> {
        let drawn = self.drawn?;
        let mut inverse = Mat2D::default();
        self.matrix.invert(&mut inverse).then(|| drawn * inverse)
    }
}

impl Lift {
    /// The Screen that draws the copy (`DrawingScreen`).
    pub(crate) fn drawing_screen(&self) -> DrawingScreen {
        let state = self.0.borrow();
        DrawingScreen {
            screen: state.board.clone(),
            matrix: state.matrix,
            drawn: state.drawn,
        }
    }

    /// Whether the lift draws its copy now: it is solved, at an opacity
    /// above 0.
    pub(crate) fn shows(&self) -> bool {
        let state = self.0.borrow();
        state.drawn.is_some() && state.opacity > 0.0
    }

    pub(crate) fn new(
        row: CoreHandle,
        board: CoreHandle,
        element: CoreHandle,
        anchor: CoreHandle,
    ) -> Self {
        Self(Rc::new(RefCell::new(LiftState {
            row,
            board,
            element,
            anchor,
            coverer: None,
            matrix: Mat2D::identity(),
            opacity: 1.0,
            clip: None,
            clip_space: Mat2D::identity(),
            entries: Vec::new(),
            picture: GroupPicture::default(),
            drawn: None,
            page: None,
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

/// The lift that draws `component`, a copy flying in place or a part of
/// one, read from its Screen's lifts and, inside a placement, its hosts';
/// None outside a step's lifts, at the cost of one look at each artboard
/// above it.
pub(crate) fn lift_around(component: &CoreHandle) -> Option<Lift> {
    let mut node = Some(component.clone());
    let mut visited = 0;
    while let Some(current) = node {
        visited += 1;
        if visited > 64 {
            return None;
        }
        let artboard = current
            .with(|object| match object.as_artboard() {
                Some(_) => Some(current.clone()),
                None => object.as_component()?.artboard_handle(),
            })
            .flatten()?;
        let away = artboard
            .with_downcast::<Artboard, _>(|artboard| {
                let lifts = artboard.lifts.try_borrow().ok()?;
                (!lifts.away.is_empty()).then(|| lifts.away.clone())
            })
            .flatten();
        if let Some(away) = away {
            let mut inner = Some(current.clone());
            while let Some(here) = inner {
                if here == artboard {
                    break;
                }
                if let Some(lift) = away.iter().find(|lift| lift.0.borrow().element == here) {
                    return Some(lift.clone());
                }
                inner = here
                    .with(|object| object.component_parent_handle())
                    .flatten();
            }
        }
        // Past this artboard: its host, on the artboard that places it.
        node = artboard
            .with_downcast::<Artboard, _>(Artboard::host)
            .flatten();
    }
    None
}

/// Whether a pointer may enter `component`: not inside a copy flying in
/// place, which takes no input for its step, as the devices turn input off
/// for one (iOS `ExperiencePageStep.swift`, Android `ExperiencePageStep.kt`).
/// The one rule a state machine's hit gate
/// (animation/state_machine_instance.rs, `HitGate`) and the host's point
/// tests (host_semantics.rs) read.
pub(crate) fn takes_input(component: &CoreHandle) -> bool {
    lift_around(component).is_none()
}

/// Where a lift draws its copy: the one map that every reader of where a
/// lifted copy draws reads, in whatever space it works in, instead of the
/// copy's resting world, which never moves (`Lifted::of`).
#[derive(Clone)]
pub(crate) struct DrawnMap {
    /// From a point of the copy at rest to where it draws, in its own
    /// Screen's drawing space.
    drawn: Mat2D,
    /// The same in the page's space, which the pose writes beside it.
    page: Mat2D,
    /// The lift's matrix, from the copy's own Screen's drawing space to the
    /// upper Screen's: `drawn` is `row⁻¹ · upper · lift`
    /// (live_move/in_place.rs, `drawn_map`).
    lift: Mat2D,
    /// The copy's own Screen, whose drawing space `drawn` is in.
    pub(crate) screen: CoreHandle,
    /// The upper Screen, which draws the copy.
    upper: CoreHandle,
}

impl DrawnMap {
    /// The map in the page's space.
    pub(crate) fn page(&self) -> Mat2D {
        self.page
    }

    /// The map in a reader's own space, the one way every reader takes it:
    /// `to_upper · lift · to_row⁻¹`, where `place` says where the reader
    /// puts each Screen's drawing space, the copy's own (`to_row`) and the
    /// upper one (`to_upper`). A reader whose space places the two Screens
    /// differently from drawing, a frame origin or a row's scale apart,
    /// still sees the copy where its upper twin is at the end. A reader
    /// that does not place the upper Screen reads it where drawing does,
    /// from the copy's own Screen: `to_row · drawn · lift⁻¹`. None when a
    /// Screen's place or the lift can't be undone.
    pub(crate) fn in_space(&self, place: impl Fn(&CoreHandle) -> Option<Mat2D>) -> Option<Mat2D> {
        let to_row = place(&self.screen)?;
        let to_upper = match place(&self.upper) {
            Some(to_upper) => to_upper,
            None => {
                let mut lift = Mat2D::default();
                self.lift.invert(&mut lift).then_some(())?;
                to_row * self.drawn * lift
            }
        };
        let mut row = Mat2D::default();
        to_row.invert(&mut row).then(|| to_upper * self.lift * row)
    }
}

/// Where a lift puts `component` for a reader that places Screens by
/// `place` (`DrawnMap::in_space`). None outside a lift and while it draws
/// nothing, which keeps the component where it rests.
pub(crate) fn drawn_in(
    component: &CoreHandle,
    place: impl Fn(&CoreHandle) -> Option<Mat2D>,
) -> Option<Mat2D> {
    match Lifted::of(component) {
        Lifted::Drawn(map) => map.in_space(place),
        Lifted::No | Lifted::Hidden => None,
    }
}

/// Where a query rooted at the first of `spaces`, each artboard's drawing
/// space in the query's root, reads `component`: one answer for its
/// geometry and the copy's own clips together, so both take the lift's map
/// or neither does.
pub(crate) enum QueryPlace {
    /// Outside any lift: where it rests.
    Plain,
    /// The query is rooted inside the copy, its own Screen out of
    /// `spaces`: the root moves with the copy, so nothing is mapped.
    Inside,
    /// The query is rooted at or above the copy's own Screen: the lift's
    /// map in the query's space.
    Above(Mat2D),
    /// Inside a lift that draws nothing.
    Hidden,
}

pub(crate) fn query_place(component: &CoreHandle, spaces: &[(CoreHandle, Mat2D)]) -> QueryPlace {
    let place = |screen: &CoreHandle| {
        spaces
            .iter()
            .find(|(owner, _)| owner == screen)
            .map(|(_, to_root)| *to_root)
    };
    match Lifted::of(component) {
        Lifted::No => QueryPlace::Plain,
        Lifted::Hidden => QueryPlace::Hidden,
        Lifted::Drawn(map) if place(&map.screen).is_none() => QueryPlace::Inside,
        Lifted::Drawn(map) => map
            .in_space(place)
            .map_or(QueryPlace::Hidden, QueryPlace::Above),
    }
}

/// What a lift does to where `component` draws: nothing, outside one; its
/// map (`DrawnMap`), inside one that draws; or nothing drawn, inside one that
/// draws nothing.
pub(crate) enum Lifted {
    No,
    Drawn(DrawnMap),
    Hidden,
}

impl Lifted {
    pub(crate) fn of(component: &CoreHandle) -> Self {
        Self::of_lift(lift_around(component).as_ref())
    }

    /// What `lift`, the lift around a component, does to where it draws.
    pub(crate) fn of_lift(lift: Option<&Lift>) -> Self {
        let Some(lift) = lift else {
            return Self::No;
        };
        let state = lift.0.borrow();
        match (state.drawn, state.page) {
            (Some(drawn), Some(page)) => Self::Drawn(DrawnMap {
                drawn,
                page,
                lift: state.matrix,
                screen: state.row.clone(),
                upper: state.board.clone(),
            }),
            _ => Self::Hidden,
        }
    }

    /// Where the page shows `point`, a point it shows of the copy at rest;
    /// None when the lift draws nothing.
    pub(crate) fn point(&self, point: Vec2D) -> Option<Vec2D> {
        match self {
            Self::No => Some(point),
            Self::Drawn(map) => Some(map.page * point),
            Self::Hidden => None,
        }
    }
}

/// Where an ancestor that `DrawnAncestors` meets sits for what draws it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Place {
    /// Outside any lift: an ordinary ancestor.
    Plain,
    /// The walk's start and what holds it up to the copy flying in place,
    /// the copy included: what the lift draws, where its map puts it.
    Copy,
    /// Past the copy up to its own Screen, that Screen included when another
    /// draws the lift: where the copy is held. A collapsed or hidden one
    /// hides it; none of it draws over it or clips it.
    Between,
    /// The Screen that draws the lift, the upper copy's own: it draws the
    /// copy at its own strength there, inside its frame, while awake.
    Screen,
    /// What is above that Screen, as for any of its content.
    Above,
}

/// The one walk up from a component for what draws it. Outside a lift it is
/// the ordinary climb, each artboard on to its host. Inside a copy flying in
/// place it climbs the copy, then past it to the copy's own Screen, then goes
/// on from the Screen that draws the copy (`Lift::drawing_screen`) and what
/// is above that, never from the copy's own Screen's hosts. Every reader
/// that climbs above a component reads this walk: visibility, rendered
/// clips and the geometry clips built on them. It stops at a loop
/// (`looped`).
pub(crate) struct DrawnAncestors {
    lift: Option<Lift>,
    /// The copy, its own Screen and the Screen that draws it.
    marks: Option<(CoreHandle, CoreHandle, CoreHandle)>,
    next: Option<(CoreHandle, Place)>,
    visited: HashSet<CoreHandle>,
    looped: bool,
}

impl DrawnAncestors {
    pub(crate) fn of(start: &CoreHandle) -> Self {
        let lift = lift_around(start);
        let marks = lift.as_ref().map(|lift| {
            let (element, row) = {
                let state = lift.0.borrow();
                (state.element.clone(), state.row.clone())
            };
            (element, row, lift.drawing_screen().screen)
        });
        let place = if marks.is_some() {
            Place::Copy
        } else {
            Place::Plain
        };
        Self {
            lift,
            marks,
            next: Some((start.clone(), place)),
            visited: HashSet::new(),
            looped: false,
        }
    }

    /// The lift around the start, inside a copy flying in place.
    pub(crate) fn lift(&self) -> Option<&Lift> {
        self.lift.as_ref()
    }

    /// The copy flying in place, inside one.
    pub(crate) fn copy(&self) -> Option<&CoreHandle> {
        self.marks.as_ref().map(|(element, _, _)| element)
    }

    /// Whether the walk met an ancestor twice and stopped.
    pub(crate) fn looped(&self) -> bool {
        self.looped
    }
}

/// The next level up: a component's parent, an artboard's host.
fn parent_or_host(component: &CoreHandle) -> Option<CoreHandle> {
    component
        .with(|object| {
            object
                .component_parent_handle()
                .or_else(|| object.as_artboard().and_then(Artboard::host))
        })
        .flatten()
}

impl Iterator for DrawnAncestors {
    type Item = (CoreHandle, Place);

    fn next(&mut self) -> Option<Self::Item> {
        let (current, mut place) = self.next.take()?;
        if !self.visited.insert(current.clone()) {
            self.looped = true;
            return None;
        }
        let up = |place: Place| parent_or_host(&current).map(|parent| (parent, place));
        self.next = match (&self.marks, place) {
            (Some((element, _, _)), Place::Copy) if current == *element => up(Place::Between),
            (Some((_, row, screen)), Place::Between) if current == *row => {
                if row == screen {
                    place = Place::Screen;
                    up(Place::Above)
                } else {
                    Some((screen.clone(), Place::Screen))
                }
            }
            (_, Place::Screen) => up(Place::Above),
            _ => up(place),
        };
        Some((current, place))
    }
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
/// The box reaches the renderer as a flying unit's does (`Artboard::draw`):
/// its own outline under the element's world, the input the element's paint
/// draws with, so the clip covers the samples the fill covers and a rounded
/// edge is smoothed once. The world is taken back after the clip, so the
/// copy's entries draw under the lift's transform alone. An element whose
/// world collapses draws nothing, and takes no clip.
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
