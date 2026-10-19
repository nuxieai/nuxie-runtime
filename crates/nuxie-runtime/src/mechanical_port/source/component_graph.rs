//! Data-oriented scheduling state for one initialized Artboard.
//!
//! Upstream keeps Component::m_Dirt, the dependents list and the onDirty hook
//! on each heap object and walks them through pointers. Here the graph members'
//! dirt, their dependents (by graph order) and their onDirty/update dispatch
//! live in dense arrays indexed by graph order. The edges are fixed once
//! Artboard::initialize has sorted them, so the arrays are built once. The
//! addDirt cascade and the updateComponents walk run over these arrays and only
//! touch a component's payload when its type has real onDirty or update work.
//!
//! Members are the dependency order's entries except the Artboard root, whose
//! dirt shares a cell with its scheduling bits, and occurrences outside the
//! order (DrawTargets, cycle remnants); those keep the occurrence-handle path.

use std::cell::Cell;
use std::rc::Rc;

use crate::mechanical_port::source::{
    artboard::RuntimeArtboardDirtyHandle,
    bones::{bone::Bone, root_bone::RootBone, skin::Skin},
    component::ComponentOccurrenceHandle,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core::CoreObject,
    node::Node as NodeObject,
    shapes::{path_cells::PathCells, shape::Shape, shape_cells::ShapeCells},
    world_transform_component::{PlainTransform, TransformCells, Translation},
};
use std::any::Any;

type UpdateHandler = fn(&CoreHandle, ComponentDirt) -> bool;
type DirtyHandler = fn(&CoreHandle, ComponentDirt);

/// Where a member's dirt lives and what its onDirty runs. Types whose
/// dirt-phase state lives outside the component carry the cells it runs on.
enum OnDirty {
    /// The root or another occurrence whose dirt lives outside the graph.
    Outside,
    /// An authored Component whose onDirty is Component's empty default.
    Noop,
    /// An authored Component whose onDirty runs after the payload borrow.
    Handler(DirtyHandler),
    /// An authored Component whose onDirty runs inside the payload borrow.
    Receiver,
    /// Path::onDirty, on the path's cells.
    Path(Rc<PathCells>),
    /// A Skin of a PointsPath: Skin::onDirty marks that path dirty.
    SkinnedPath(CoreHandle, Rc<PathCells>),
    /// A PathComposer, through the cells it shares with its Shape.
    Composer(Rc<ShapeCells>),
    /// Another runtime helper (a TextVariationHelper, or a composer whose
    /// Shape no longer holds it).
    Helper,
}

impl OnDirty {
    #[inline]
    fn is_outside(&self) -> bool {
        matches!(self, Self::Outside)
    }

    /// A runtime helper, whose liveness is its owner's.
    #[inline]
    fn is_helper(&self) -> bool {
        matches!(self, Self::Composer(_) | Self::Helper)
    }
}

/// How an authored member updates, so the update walk reads only the column
/// that member needs.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Update {
    // The transform cells kernel (`transforms`).
    Transform,
    // A registered handler (`update`).
    Handler,
    // The receiver's own component_update.
    Receiver,
}

/// Everything a cascade step and its onDirty read, in one record.
struct Node {
    dirt: Cell<ComponentDirt>,
    // Cleared when an authored member's occurrence retires.
    alive: Cell<bool>,
    update: Update,
    // Component::m_collapsedRunSkip: the length of the collapsed run starting
    // here, as markCollapsedRuns last measured it.
    collapsed_run_skip: Cell<u16>,
    dependents_start: u32,
    dependents_end: u32,
    on_dirty: OnDirty,
}

/// Dependents are graph orders; this bit marks an index into `outside`
/// (occurrences that are not graph members, such as DrawTargets).
const OUTSIDE_DEPENDENT: u32 = 1 << 31;

/// Cascade levels walked on the stack before the cascade recurses.
const CASCADE_DEPTH: usize = 16;

/// Artboard::onComponentDirty notifications from members without onDirty
/// work, held while no other code runs: nothing can observe the Artboard
/// between them, so they apply as one batch before the next call out.
struct PendingNotifications {
    count: u32,
    lowest_order: u32,
}

impl PendingNotifications {
    const NONE: Self = Self {
        count: 0,
        lowest_order: u32::MAX,
    };

    #[inline]
    fn add(&mut self, order: u32) {
        self.count += 1;
        self.lowest_order = self.lowest_order.min(order);
    }

    #[inline]
    fn flush(&mut self, artboard: &RuntimeArtboardDirtyHandle) {
        if self.count != 0 {
            artboard.on_components_dirty(self.count, self.lowest_order);
            *self = Self::NONE;
        }
    }
}

/// A transform input that a plain transform member's generated setter writes
/// to its TransformCells.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TransformInput {
    NodeX,
    NodeY,
    RootBoneX,
    RootBoneY,
    Rotation,
    ScaleX,
    ScaleY,
    Opacity,
}

impl TransformInput {
    #[inline]
    fn from_key(property_key: i32) -> Option<Self> {
        use crate::mechanical_port::source::generated::{
            bones::root_bone_base::RootBoneBase, node_base::NodeBase,
            transform_component_base::TransformComponentBase,
            world_transform_component_base::WorldTransformComponentBase,
        };
        let key = u16::try_from(property_key).ok()?;
        Some(match key {
            NodeBase::X_PROPERTY_KEY => Self::NodeX,
            NodeBase::Y_PROPERTY_KEY => Self::NodeY,
            RootBoneBase::X_PROPERTY_KEY => Self::RootBoneX,
            RootBoneBase::Y_PROPERTY_KEY => Self::RootBoneY,
            TransformComponentBase::ROTATION_PROPERTY_KEY => Self::Rotation,
            TransformComponentBase::SCALE_X_PROPERTY_KEY => Self::ScaleX,
            TransformComponentBase::SCALE_Y_PROPERTY_KEY => Self::ScaleY,
            WorldTransformComponentBase::OPACITY_PROPERTY_KEY => Self::Opacity,
            _ => return None,
        })
    }

    /// This input's cell accessors on a member updated by `kernel`, if the
    /// member's type has the property. Node (and Shape) own x/y under
    /// NodeBase's keys, a RootBone under its own; a Bone has no position.
    #[inline]
    fn accessors(
        self,
        kernel: &PlainTransform,
    ) -> Option<(fn(&TransformCells) -> f32, fn(&TransformCells, f32))> {
        let owns_position =
            |node: bool| kernel.translation == Translation::Own && kernel.node == node;
        Some(match self {
            Self::NodeX if owns_position(true) => (TransformCells::x, TransformCells::set_x),
            Self::NodeY if owns_position(true) => (TransformCells::y, TransformCells::set_y),
            Self::RootBoneX if owns_position(false) => (TransformCells::x, TransformCells::set_x),
            Self::RootBoneY if owns_position(false) => (TransformCells::y, TransformCells::set_y),
            Self::Rotation => (TransformCells::rotation, TransformCells::set_rotation),
            Self::ScaleX => (TransformCells::scale_x, TransformCells::set_scale_x),
            Self::ScaleY => (TransformCells::scale_y, TransformCells::set_scale_y),
            Self::Opacity => (TransformCells::opacity, TransformCells::set_opacity),
            _ => return None,
        })
    }
}

pub(crate) struct ComponentGraph {
    // Shared with each member's DirtBinding so Components read their own
    // dirt directly.
    nodes: Rc<[Node]>,
    dependents: Box<[u32]>,
    // Cold, per member: only real update work and fallbacks read these.
    update: Box<[Option<UpdateHandler>]>,
    // Members whose update runs entirely on transform cells.
    transforms: Box<[Option<PlainTransform>]>,
    occurrences: Box<[ComponentOccurrenceHandle]>,
    outside: Box<[ComponentOccurrenceHandle]>,
    artboard: RuntimeArtboardDirtyHandle,
}

impl ComponentGraph {
    /// Build from a sorted dependency order and bind every member's
    /// Component dirt to the graph, keeping its current bits.
    pub(crate) fn build(
        root: &CoreHandle,
        order: &[ComponentOccurrenceHandle],
        artboard: RuntimeArtboardDirtyHandle,
    ) -> Rc<Self> {
        let mut on_dirty = Vec::with_capacity(order.len());
        let mut transforms = Vec::with_capacity(order.len());
        let mut update = Vec::with_capacity(order.len());
        let mut dirt = Vec::with_capacity(order.len());
        for occurrence in order {
            transforms.push(match occurrence {
                ComponentOccurrenceHandle::Authored(handle) if handle != root => {
                    handle.with(plain_transform).flatten()
                }
                _ => None,
            });
            let (member_on_dirty, update_handler) = match occurrence {
                ComponentOccurrenceHandle::Authored(handle) if handle == root => {
                    (OnDirty::Outside, None)
                }
                ComponentOccurrenceHandle::Authored(handle) => handle
                    .with(|object| {
                        let update_handler = object.component_update_handler();
                        let actual: &dyn Any = object;
                        let member_on_dirty = if let Some(path) = object.as_path() {
                            // Every Path type runs Path::onDirty.
                            OnDirty::Path(Rc::clone(path.cells()))
                        } else if let Some((path, cells)) = actual
                            .downcast_ref::<Skin>()
                            .and_then(Skin::skinned_path_cells)
                        {
                            OnDirty::SkinnedPath(path, cells)
                        } else if let Some(handler) = object.component_on_dirty_handler() {
                            OnDirty::Handler(handler)
                        } else if object.component_on_dirty_is_noop() {
                            OnDirty::Noop
                        } else {
                            OnDirty::Receiver
                        };
                        (member_on_dirty, update_handler)
                    })
                    .unwrap_or((OnDirty::Outside, None)),
                ComponentOccurrenceHandle::PathComposer(composer) => (
                    composer.upgrade().map_or(OnDirty::Helper, |composer| {
                        OnDirty::Composer(Rc::clone(composer.borrow().cells_ref()))
                    }),
                    None,
                ),
                ComponentOccurrenceHandle::TextVariationHelper(_) => (OnDirty::Helper, None),
            };
            on_dirty.push(member_on_dirty);
            update.push(update_handler);
            dirt.push(
                occurrence
                    .with_component(|component| component.dirt())
                    .unwrap_or(ComponentDirt::NONE),
            );
        }
        let is_outside: Vec<bool> = on_dirty.iter().map(OnDirty::is_outside).collect();
        // Graph order is the dependency-order index for every member.
        let member_index = |occurrence: &ComponentOccurrenceHandle| -> Option<u32> {
            let index = occurrence
                .with_component(|component| component.graph_order())
                .filter(|&value| value != u32::MAX)?;
            (order.get(index as usize) == Some(occurrence) && !is_outside[index as usize])
                .then_some(index)
        };
        let mut nodes = Vec::with_capacity(order.len());
        let mut dependents = Vec::new();
        let mut outside = Vec::new();
        let mut on_dirty = on_dirty.into_iter();
        for (index, occurrence) in order.iter().enumerate() {
            let start = dependents.len() as u32;
            for dependent in occurrence
                .with_component(|component| component.dependents_snapshot())
                .unwrap_or_default()
                .iter()
            {
                dependents.push(match member_index(dependent) {
                    Some(member) => member,
                    None => {
                        outside.push(dependent.clone());
                        (outside.len() as u32 - 1) | OUTSIDE_DEPENDENT
                    }
                });
            }
            nodes.push(Node {
                dirt: Cell::new(dirt[index]),
                alive: Cell::new(true),
                on_dirty: on_dirty.next().expect("one onDirty per member"),
                update: if transforms[index].is_some() {
                    Update::Transform
                } else if update[index].is_some() {
                    Update::Handler
                } else {
                    Update::Receiver
                },
                collapsed_run_skip: Cell::new(0),
                dependents_start: start,
                dependents_end: dependents.len() as u32,
            });
        }
        let graph = Rc::new(Self {
            nodes: Rc::from(nodes),
            dependents: dependents.into_boxed_slice(),
            update: update.into_boxed_slice(),
            transforms: transforms.into_boxed_slice(),
            occurrences: order.to_vec().into_boxed_slice(),
            outside: outside.into_boxed_slice(),
            artboard,
        });
        for (index, occurrence) in order.iter().enumerate() {
            if graph.nodes[index].on_dirty.is_outside() {
                continue;
            }
            occurrence.with_component_mut(|component| {
                component.bind_graph_dirt(Rc::clone(&graph), index as u32)
            });
            if let ComponentOccurrenceHandle::Authored(handle) = occurrence {
                handle.bind_component_graph(Rc::clone(&graph));
            }
        }
        // The nodes start with no collapsed-run skips: the first pass marks
        // them, whatever an earlier walk measured.
        graph.artboard.collapse_changed();
        graph
    }

    /// Member `i`'s Component dirt, wherever it lives.
    #[inline(always)]
    fn member_dirt(&self, i: usize) -> ComponentDirt {
        let node = &self.nodes[i];
        if node.on_dirty.is_outside() {
            return self.outside_member_dirt(i);
        }
        node.dirt.get()
    }

    /// The dirt of an outside member (the Artboard root), which lives with its
    /// occurrence.
    #[cold]
    #[inline(never)]
    fn outside_member_dirt(&self, i: usize) -> ComponentDirt {
        self.occurrences[i]
            .with_component(|component| component.dirt())
            .unwrap_or(ComponentDirt::NONE)
    }

    /// The skip from a collapsed member at `index` to the end of its run of
    /// collapsed members, `run_end`.
    fn collapsed_run_skip(run_end: usize, index: usize) -> u16 {
        (run_end - index).min(u16::MAX as usize) as u16
    }

    /// Artboard::markCollapsedRuns: measure every collapsed run, matching the
    /// update walk's Collapsed dirt bit rather than virtual isCollapsed.
    fn mark_collapsed_runs(&self) {
        self.artboard.clear_collapsed_runs_dirty();
        let mut run_end = self.nodes.len();
        for (i, node) in self.nodes.iter().enumerate().rev() {
            if !self.member_dirt(i).contains(ComponentDirt::COLLAPSED) {
                run_end = i;
                continue;
            }
            node.collapsed_run_skip
                .set(Self::collapsed_run_skip(run_end, i));
        }
    }

    /// Artboard::verifyCollapsedRuns: a collapse change that skipped
    /// collapseChanged() leaves a stale skip, which aborts testing builds.
    #[cfg(any(test, feature = "testing"))]
    fn verify_collapsed_runs(&self) {
        let mut run_end = self.nodes.len();
        for (i, node) in self.nodes.iter().enumerate().rev() {
            if !self.member_dirt(i).contains(ComponentDirt::COLLAPSED) {
                run_end = i;
                continue;
            }
            let expected = Self::collapsed_run_skip(run_end, i);
            if node.collapsed_run_skip.get() != expected {
                let name = self.occurrences[i]
                    .with_component(|component| component.base.name().to_owned())
                    .unwrap_or_default();
                eprintln!(
                    "Collapsed '{name}' at {i} skips {}, not {expected}: a collapse change didn't reach collapseChanged()",
                    node.collapsed_run_skip.get()
                );
                std::process::abort();
            }
        }
    }

    /// Whether member `index` is still live (not retired).
    #[inline]
    pub(crate) fn is_live(&self, index: u32) -> bool {
        self.nodes[index as usize].alive.get()
    }

    pub(crate) fn mark_retired(&self, index: u32) {
        if let Some(node) = self.nodes.get(index as usize) {
            node.alive.set(false);
        }
    }

    /// A member Component's handle on its own dirt in the shared node array.
    pub(crate) fn dirt_binding(&self, index: u32) -> DirtBinding {
        DirtBinding {
            nodes: Rc::clone(&self.nodes),
            index,
        }
    }

    /// Member `index`'s Component dirt.
    #[inline]
    pub(crate) fn node_dirt(&self, index: u32) -> ComponentDirt {
        self.nodes[index as usize].dirt.get()
    }

    #[inline]
    pub(crate) fn set_dirt(&self, index: u32, value: ComponentDirt) {
        self.nodes[index as usize].dirt.set(value);
    }

    /// Artboard::onComponentDirty for a member: its graph order is its index.
    #[inline]
    pub(crate) fn notify_artboard(&self, index: u32) {
        self.artboard.on_component_dirty_at(index);
    }

    /// Component::addDirt for a member reached without a held payload borrow.
    /// The duplicate-bit return is inline; most adds from onDirty handlers
    /// find their bits already set.
    #[inline]
    pub(crate) fn add_dirt(&self, index: u32, value: ComponentDirt, recurse: bool) -> bool {
        #[cfg(feature = "tools")]
        trace::event(self, 'A', index, value, recurse);
        let node = &self.nodes[index as usize];
        // A retired member returns false too, and neither return has side
        // effects, so this check may go first. An outside member's dirt lives
        // with its occurrence, not here.
        if !node.on_dirty.is_outside() && node.dirt.get().contains(value) {
            return false;
        }
        self.add_new_dirt(index, value, recurse)
    }

    #[inline(never)]
    fn add_new_dirt(&self, index: u32, value: ComponentDirt, recurse: bool) -> bool {
        let i = index as usize;
        let node = &self.nodes[i];
        if node.on_dirty.is_outside() {
            return self.occurrences[i].add_dirt(value, recurse);
        }
        if !self.mark(i, value) {
            return false;
        }
        if recurse && node.dependents_start != node.dependents_end {
            self.cascade(index, value);
        }
        true
    }

    /// Component::addDirt's own work for member `i` once its duplicate-bit
    /// check has passed: the liveness check, the dirt write, onDirty and the
    /// Artboard notification. Returns whether it added dirt (so the caller
    /// continues into its dependents). Only members that gain dirt get here.
    #[inline(always)]
    fn mark(&self, i: usize, value: ComponentDirt) -> bool {
        let node = &self.nodes[i];
        let live = if node.on_dirty.is_helper() {
            self.occurrences[i].is_live_helper()
        } else {
            // Retired authored members behave like a stale handle.
            node.alive.get()
        };
        if !live {
            return false;
        }
        let dirt = node.dirt.get() | value;
        node.dirt.set(dirt);
        match &node.on_dirty {
            OnDirty::Noop => {}
            OnDirty::Handler(handler) => handler(self.authored(i), dirt),
            OnDirty::Receiver => {
                self.authored(i).with_mut(|object| {
                    object.component_on_dirty(dirt);
                });
            }
            // The owner is read only when the path's deferred rebuild needs it.
            OnDirty::Path(path) => path.on_dirty_with(|| self.authored(i), dirt),
            OnDirty::SkinnedPath(path, cells) => {
                // Skin::onDirty: opacity cannot move bones; otherwise
                // PointsPath::markSkinDirty, i.e. Path::markPathDirty.
                if (dirt & !ComponentDirt::COLLAPSED) != ComponentDirt::RENDER_OPACITY {
                    cells.mark_path_dirty(path);
                }
            }
            OnDirty::Composer(shape) => shape.composer_on_dirty(),
            OnDirty::Helper => self.occurrences[i].on_dirty_helper(dirt),
            OnDirty::Outside => unreachable!("outside occurrences keep their own path"),
        }
        self.notify_artboard(i as u32);
        true
    }

    #[inline]
    fn authored(&self, i: usize) -> &CoreHandle {
        match &self.occurrences[i] {
            ComponentOccurrenceHandle::Authored(handle) => handle,
            _ => unreachable!("authored entries hold authored occurrences"),
        }
    }

    /// DependencyHelper::addDirtToDependents over the static edges, recursing
    /// like upstream: each dependent's whole cascade finishes before its next
    /// sibling.
    pub(crate) fn add_dirt_to_dependents(&self, index: u32, value: ComponentDirt) {
        #[cfg(feature = "tools")]
        trace::event(self, 'P', index, value, true);
        self.cascade(index, value);
    }

    /// Upstream's recursion, depth first: each dependent's whole cascade
    /// finishes before its next sibling. The walk keeps the remaining
    /// dependents of each open level on a fixed stack and recurses only past
    /// CASCADE_DEPTH levels.
    fn cascade(&self, index: u32, value: ComponentDirt) {
        let node = &self.nodes[index as usize];
        let mut levels = [(0u32, 0u32); CASCADE_DEPTH];
        let mut depth = 0;
        let (mut next, mut end) = (node.dependents_start, node.dependents_end);
        let mut pending = PendingNotifications::NONE;
        loop {
            if next == end {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                (next, end) = levels[depth];
                continue;
            }
            let dependent = self.dependents[next as usize];
            next += 1;
            // An outside dependent's index has OUTSIDE_DEPENDENT set, so it is
            // never a member index. Members here are never outside:
            // build routes those through `outside`.
            let Some(child) = self.nodes.get(dependent as usize) else {
                pending.flush(&self.artboard);
                self.add_dirt_outside(dependent, value);
                continue;
            };
            debug_assert!(!child.on_dirty.is_outside());
            let dirt = child.dirt.get();
            if dirt.contains(value) {
                continue;
            }
            // A live member with no onDirty work: Component::addDirt's write,
            // with its Artboard notification held until other code can run.
            if matches!(child.on_dirty, OnDirty::Noop) && child.alive.get() {
                child.dirt.set(dirt | value);
                pending.add(dependent);
            } else {
                pending.flush(&self.artboard);
                if !self.mark(dependent as usize, value) {
                    continue;
                }
            }
            if child.dependents_start == child.dependents_end {
                continue;
            }
            if depth == CASCADE_DEPTH {
                pending.flush(&self.artboard);
                self.cascade(dependent, value);
                continue;
            }
            levels[depth] = (next, end);
            depth += 1;
            (next, end) = (child.dependents_start, child.dependents_end);
        }
        pending.flush(&self.artboard);
    }

    /// Component::addDirt(value, true) on a dependent outside the graph.
    #[cold]
    #[inline(never)]
    fn add_dirt_outside(&self, dependent: u32, value: ComponentDirt) {
        self.outside[(dependent & !OUTSIDE_DEPENDENT) as usize].add_dirt(value, true);
    }

    /// Set a plain transform member's position, rotation, scale or opacity
    /// without borrowing it. The generated setter's steps run in order on the
    /// member's cells and node: the equality test and the cell write, then the
    /// property's changed callback, then its property observers. The callback
    /// is TransformComponent::markTransformDirty, or for opacity
    /// WorldTransformComponent::opacityChanged.
    ///
    /// Returns None, having done nothing, when `handle` is not a live member
    /// updated by the plain transform kernel, or when its type has no such
    /// property under `property_key`. The caller then takes the generated
    /// setter.
    pub(crate) fn set_plain_transform_double(
        handle: &CoreHandle,
        property_key: i32,
        value: f32,
    ) -> Option<bool> {
        let input = TransformInput::from_key(property_key)?;
        if !kernel_candidate(handle) {
            return None;
        }
        let (graph, order) = handle.component_graph()?;
        let kernel = graph.transforms.get(order as usize)?.as_ref()?;
        let (get, set) = input.accessors(kernel)?;
        let cells = &*kernel.cells;
        if get(cells) == value {
            return Some(true);
        }
        set(cells, value);
        if input == TransformInput::Opacity {
            graph.add_dirt(order, ComponentDirt::RENDER_OPACITY, true);
        } else if graph.add_dirt(order, ComponentDirt::TRANSFORM, false) {
            graph.add_dirt(order, ComponentDirt::WORLD_TRANSFORM, true);
        }
        handle.notify_property_changed(property_key as u16);
        Some(true)
    }

    /// The generated getter of a property set_plain_transform_double sets,
    /// read from the member's cells; None where that returns None.
    pub(crate) fn get_plain_transform_double(
        handle: &CoreHandle,
        property_key: i32,
    ) -> Option<f32> {
        let input = TransformInput::from_key(property_key)?;
        if !kernel_candidate(handle) {
            return None;
        }
        let (graph, order) = handle.component_graph()?;
        let kernel = graph.transforms.get(order as usize)?.as_ref()?;
        let (get, _) = input.accessors(kernel)?;
        Some(get(&kernel.cells))
    }

    /// Artboard::updateComponents over the graph. `depth` and the root's
    /// COMPONENTS bit live in the shared dirty handle, so no Artboard borrow is
    /// held while components update.
    pub(crate) fn update_components(&self) {
        #[cfg(feature = "tools")]
        trace::event(self, 'W', 0, ComponentDirt::NONE, false);
        let dirty = &self.artboard;
        let count = self.nodes.len();
        let mut step = 0;
        while step < 100 && dirty.has_component_dirt() {
            dirty.clear_components_dirt();
            if dirty.collapsed_runs_dirty() {
                self.mark_collapsed_runs();
            } else {
                #[cfg(any(test, feature = "testing"))]
                self.verify_collapsed_runs();
            }
            let mut i = 0;
            while i < count {
                // A member without dirt does nothing in upstream's walk; step
                // over it on one read. The depth only matters to a member's
                // update and its callbacks, so it is set where those can run.
                let node = &self.nodes[i];
                if node.dirt.get() == ComponentDirt::NONE && !node.on_dirty.is_outside() {
                    i += 1;
                    continue;
                }
                dirty.set_depth(i as u32);
                if self.member_dirt(i).contains(ComponentDirt::COLLAPSED) {
                    // Collapsed members aren't updated and sit in long runs (a
                    // Solo's inactive options): skip the whole run. A collapse
                    // change during this pass makes the skips stale, so step
                    // one by one until the next pass marks them again.
                    let skip = self.nodes[i].collapsed_run_skip.get();
                    if !dirty.collapsed_runs_dirty() && skip > 1 {
                        dirty.count_collapsed_run_skip();
                        i += usize::from(skip);
                    } else {
                        i += 1;
                    }
                    continue;
                }
                if !self.update_entry(i) {
                    i += 1;
                    continue;
                }
                if dirty.depth() < i as u32 {
                    break;
                }
                i += 1;
            }
            step += 1;
        }
    }

    fn update_entry(&self, i: usize) -> bool {
        let node = &self.nodes[i];
        if node.on_dirty.is_outside() {
            return self.occurrences[i].update_if_dirty();
        }
        if node.on_dirty.is_helper() {
            let dirt = node.dirt.get();
            if !needs_update(dirt) {
                return false;
            }
            node.dirt.set(ComponentDirt::NONE);
            self.occurrences[i].update_helper(dirt);
            return true;
        }
        assert!(node.alive.get(), "live component in dependency graph");
        let dirt = node.dirt.get();
        if !needs_update(dirt) {
            return false;
        }
        node.dirt.set(ComponentDirt::NONE);
        let handle = self.authored(i);
        match node.update {
            Update::Transform => {
                let kernel = self.transforms[i]
                    .as_ref()
                    .expect("transform members record their kernel");
                if kernel.takes(dirt) {
                    kernel.update(handle, dirt);
                } else {
                    self.update_owner(i, handle, dirt);
                }
            }
            Update::Handler | Update::Receiver => self.update_owner(i, handle, dirt),
        }
        true
    }

    /// Member `i`'s own update: its registered handler, or its receiver's.
    fn update_owner(&self, i: usize, handle: &CoreHandle, dirt: ComponentDirt) {
        match self.update[i] {
            Some(update) => {
                update(handle, dirt);
            }
            None => {
                handle.with_mut(|object| object.component_update(dirt));
            }
        }
    }
}

/// The cells-only update of a concrete Node, Bone, RootBone, Shape (without
/// a layout participant) or Path that has no constraints and whose parent, if
/// any, is a TransformComponent (so its child opacity is
/// TransformComponent::childOpacity). A Path's kernel takes only its
/// transform-only updates (PlainTransform::takes).
fn plain_transform(object: &dyn CoreObject) -> Option<PlainTransform> {
    let actual: &dyn Any = object;
    let mut after: Option<fn(&CoreHandle, ComponentDirt)> = None;
    let mut path = None;
    let points_path =
        actual.is::<crate::mechanical_port::source::shapes::points_path::PointsPath>();
    let (translation, node) = if actual.is::<NodeObject>() {
        (Translation::Own, true)
    } else if actual.is::<RootBone>() {
        (Translation::Own, false)
    } else if actual.is::<Bone>() {
        (Translation::ParentLength, false)
    } else if let Some(shape) = actual.downcast_ref::<Shape>() {
        if shape.has_layout_participant() {
            return None;
        }
        after = Some(shape_after_transform);
        (Translation::Own, true)
    } else if is_kernel_path(actual) {
        // These paths' updates are Path::update after a prefix that acts only
        // on Path dirt, with Node's local and world transform.
        path = Some(Rc::clone(object.as_path()?.cells()));
        (Translation::Own, true)
    } else {
        return None;
    };
    let transform = object.as_transform_component()?;
    if transform.cells().has_constraints() {
        return None;
    }
    let parent = match (
        transform.parent_transform_component(),
        transform.parent_cells(),
    ) {
        (None, None) => None,
        (Some(handle), Some(cells)) => {
            let fits = handle
                .with(|parent| {
                    parent.as_transform_component().is_some()
                        && (translation == Translation::Own || parent.as_bone().is_some())
                })
                .unwrap_or(false);
            if !fits {
                return None;
            }
            Some((handle, Rc::clone(cells)))
        }
        _ => return None,
    };
    if parent.is_none() && translation == Translation::ParentLength {
        return None;
    }
    Some(PlainTransform {
        cells: Rc::clone(transform.cells()),
        parent,
        translation,
        node,
        after,
        path,
        points_path,
    })
}

/// Whether `handle` occupies its slot as one of plain_transform's concrete
/// types, read without a lease, so other types skip the graph lookup.
#[inline]
fn kernel_candidate(handle: &CoreHandle) -> bool {
    use crate::mechanical_port::source::generated::{
        bones::{bone_base::BoneBase, root_bone_base::RootBoneBase},
        node_base::NodeBase,
        shapes::{
            ellipse_base::EllipseBase, points_path_base::PointsPathBase, polygon_base::PolygonBase,
            rectangle_base::RectangleBase, shape_base::ShapeBase, star_base::StarBase,
            triangle_base::TriangleBase,
        },
    };
    matches!(
        handle.occupied_core_type(),
        Some(
            NodeBase::TYPE_KEY
                | BoneBase::TYPE_KEY
                | RootBoneBase::TYPE_KEY
                | ShapeBase::TYPE_KEY
                | PointsPathBase::TYPE_KEY
                | EllipseBase::TYPE_KEY
                | RectangleBase::TYPE_KEY
                | TriangleBase::TYPE_KEY
                | PolygonBase::TYPE_KEY
                | StarBase::TYPE_KEY
        )
    )
}

/// The concrete Path types whose update is Path::update (Node's transform
/// steps, then the rebuild) after a prefix that acts only on Path dirt.
fn is_kernel_path(actual: &dyn Any) -> bool {
    use crate::mechanical_port::source::shapes::{
        ellipse::Ellipse, points_path::PointsPath, polygon::Polygon, rectangle::Rectangle,
        star::Star, triangle::Triangle,
    };
    actual.is::<PointsPath>()
        || actual.is::<Ellipse>()
        || actual.is::<Rectangle>()
        || actual.is::<Triangle>()
        || actual.is::<Polygon>()
        || actual.is::<Star>()
}

/// Shape::update after TransformComponent::update: propagate render opacity.
fn shape_after_transform(owner: &CoreHandle, dirt: ComponentDirt) {
    if dirt.contains(ComponentDirt::RENDER_OPACITY) {
        owner.with_downcast_mut::<Shape, _>(|shape| shape.update_after_transform_super(dirt));
    }
}

#[inline]
fn needs_update(dirt: ComponentDirt) -> bool {
    dirt != ComponentDirt::NONE && !dirt.contains(ComponentDirt::COLLAPSED)
}

/// One member's dirt in its graph's node array. The graph order doubles as the
/// index; edges and order are fixed once the graph is built.
pub(crate) struct DirtBinding {
    nodes: Rc<[Node]>,
    index: u32,
}

impl DirtBinding {
    // Read once per listed drawable; a call costs more than the read.
    #[inline(always)]
    pub(crate) fn get(&self) -> ComponentDirt {
        self.nodes[self.index as usize].dirt.get()
    }

    #[inline]
    pub(crate) fn set(&self, value: ComponentDirt) {
        self.nodes[self.index as usize].dirt.set(value);
    }

    #[inline]
    pub(crate) fn index(&self) -> u32 {
        self.index
    }
}

/// Records the graphs and dirt traffic of a frame loop so the cascade can be
/// replayed in isolation (tools only). Text lines:
/// `G id nodes deps`, `N kind dirt start end` per node, `D deps...`,
/// `A|P|W graph index value recurse`, and `F` per frame.
#[cfg(feature = "tools")]
pub mod trace {
    use super::{ComponentDirt, ComponentGraph, OnDirty};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::fmt::Write as _;
    use std::sync::atomic::{AtomicBool, Ordering};

    // Checked before the thread-local, so an unrecorded run pays one load.
    static RECORDING: AtomicBool = AtomicBool::new(false);

    struct Recorder {
        out: String,
        graphs: HashMap<*const ComponentGraph, u32>,
    }

    thread_local! {
        static RECORDER: RefCell<Option<Recorder>> = const { RefCell::new(None) };
    }

    pub fn start() {
        RECORDING.store(true, Ordering::Relaxed);
        RECORDER.with(|recorder| {
            *recorder.borrow_mut() = Some(Recorder {
                out: String::new(),
                graphs: HashMap::new(),
            })
        });
    }

    pub fn frame() {
        RECORDER.with(|recorder| {
            if let Some(recorder) = recorder.borrow_mut().as_mut() {
                recorder.out.push_str("F\n");
            }
        });
    }

    pub fn finish(path: &std::path::Path) -> std::io::Result<()> {
        let out = RECORDER.with(|recorder| recorder.borrow_mut().take().map(|r| r.out));
        RECORDING.store(false, Ordering::Relaxed);
        std::fs::write(path, out.unwrap_or_default())
    }

    #[inline]
    pub(super) fn event(
        graph: &ComponentGraph,
        kind: char,
        index: u32,
        value: ComponentDirt,
        recurse: bool,
    ) {
        if RECORDING.load(Ordering::Relaxed) {
            record(graph, kind, index, value, recurse);
        }
    }

    #[cold]
    #[inline(never)]
    fn record(graph: &ComponentGraph, kind: char, index: u32, value: ComponentDirt, recurse: bool) {
        RECORDER.with(|recorder| {
            let mut recorder = recorder.borrow_mut();
            let Some(recorder) = recorder.as_mut() else {
                return;
            };
            let key = graph as *const ComponentGraph;
            let next = recorder.graphs.len() as u32;
            let id = *recorder.graphs.entry(key).or_insert(next);
            if id == next {
                let _ = writeln!(
                    recorder.out,
                    "G {id} {} {}",
                    graph.nodes.len(),
                    graph.dependents.len()
                );
                for node in graph.nodes.iter() {
                    let kind = match node.on_dirty {
                        OnDirty::Outside => 0,
                        OnDirty::Noop => 1,
                        // Replays treat cells dispatch as a handler call.
                        OnDirty::Handler(_) | OnDirty::Path(_) | OnDirty::SkinnedPath(..) => 2,
                        OnDirty::Receiver => 3,
                        OnDirty::Composer(_) | OnDirty::Helper => 4,
                    };
                    let _ = writeln!(
                        recorder.out,
                        "N {kind} {} {} {}",
                        node.dirt.get().0,
                        node.dependents_start,
                        node.dependents_end
                    );
                }
                recorder.out.push('D');
                for dependent in graph.dependents.iter() {
                    let _ = write!(recorder.out, " {dependent}");
                }
                recorder.out.push('\n');
            }
            let _ = writeln!(
                recorder.out,
                "{kind} {id} {index} {} {}",
                value.0, recurse as u8
            );
        });
    }
}
