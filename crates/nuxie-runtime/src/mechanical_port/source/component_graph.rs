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
    component::ComponentOccurrenceHandle,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    shapes::{path_cells::PathCells, shape_cells::ShapeCells},
};

type UpdateHandler = fn(&CoreHandle, ComponentDirt) -> bool;
type DirtyHandler = fn(&CoreHandle, ComponentDirt);

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Kind {
    /// The root or another occurrence whose dirt lives outside the graph.
    Outside,
    /// An authored Component whose onDirty is Component's empty default.
    Noop,
    /// An authored Component whose onDirty runs after the payload borrow.
    Handler,
    /// An authored Component whose onDirty runs inside the payload borrow.
    Receiver,
    /// An authored Component whose onDirty runs on its shared cells.
    Cells,
    /// Runtime helpers (PathComposer, TextVariationHelper).
    Helper,
}

/// The shared cells a member's onDirty runs on, for types whose dirt-phase
/// state lives outside the component.
enum NodeCells {
    Path(Rc<PathCells>),
    // A PathComposer, through the cells it shares with its Shape.
    Composer(Rc<ShapeCells>),
}

/// Everything a cascade step reads, packed so one load fetches it.
struct Node {
    dirt: Cell<ComponentDirt>,
    // Cleared when an authored member's occurrence retires.
    alive: Cell<bool>,
    kind: Kind,
    dependents_start: u32,
    dependents_end: u32,
}

/// Dependents are graph orders; this bit marks an index into `outside`
/// (occurrences that are not graph members, such as DrawTargets).
const OUTSIDE_DEPENDENT: u32 = 1 << 31;

pub(crate) struct ComponentGraph {
    // Shared with each member's DirtBinding so Components read their own
    // dirt directly.
    nodes: Rc<[Node]>,
    dependents: Box<[u32]>,
    // Cold, per member: only real onDirty/update work and fallbacks read these.
    on_dirty: Box<[Option<DirtyHandler>]>,
    cells: Box<[Option<NodeCells>]>,
    update: Box<[Option<UpdateHandler>]>,
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
        let mut kinds = Vec::with_capacity(order.len());
        let mut on_dirty = Vec::with_capacity(order.len());
        let mut cells = Vec::with_capacity(order.len());
        let mut update = Vec::with_capacity(order.len());
        let mut dirt = Vec::with_capacity(order.len());
        for occurrence in order {
            let (kind, handler, node_cells, update_handler) = match occurrence {
                ComponentOccurrenceHandle::Authored(handle) if handle == root => {
                    (Kind::Outside, None, None, None)
                }
                ComponentOccurrenceHandle::Authored(handle) => handle
                    .with(|object| {
                        let update_handler = object.component_update_handler();
                        if let Some(path) = object.as_path() {
                            // Every Path type runs Path::onDirty.
                            let cells = NodeCells::Path(Rc::clone(path.cells()));
                            (Kind::Cells, None, Some(cells), update_handler)
                        } else if let Some(handler) = object.component_on_dirty_handler() {
                            (Kind::Handler, Some(handler), None, update_handler)
                        } else if object.component_on_dirty_is_noop() {
                            (Kind::Noop, None, None, update_handler)
                        } else {
                            (Kind::Receiver, None, None, update_handler)
                        }
                    })
                    .unwrap_or((Kind::Outside, None, None, None)),
                ComponentOccurrenceHandle::PathComposer(composer) => (
                    Kind::Helper,
                    None,
                    composer
                        .upgrade()
                        .map(|composer| NodeCells::Composer(Rc::clone(composer.borrow().cells_ref()))),
                    None,
                ),
                ComponentOccurrenceHandle::TextVariationHelper(_) => {
                    (Kind::Helper, None, None, None)
                }
            };
            kinds.push(kind);
            on_dirty.push(handler);
            cells.push(node_cells);
            update.push(update_handler);
            dirt.push(
                occurrence
                    .with_component(|component| component.dirt())
                    .unwrap_or(ComponentDirt::NONE),
            );
        }
        // Graph order is the dependency-order index for every member.
        let member_index = |occurrence: &ComponentOccurrenceHandle| -> Option<u32> {
            let index = occurrence
                .with_component(|component| component.graph_order())
                .filter(|&value| value != u32::MAX)?;
            (order.get(index as usize) == Some(occurrence)
                && kinds[index as usize] != Kind::Outside)
                .then_some(index)
        };
        let mut nodes = Vec::with_capacity(order.len());
        let mut dependents = Vec::new();
        let mut outside = Vec::new();
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
                kind: kinds[index],
                dependents_start: start,
                dependents_end: dependents.len() as u32,
            });
        }
        let graph = Rc::new(Self {
            nodes: Rc::from(nodes),
            dependents: dependents.into_boxed_slice(),
            on_dirty: on_dirty.into_boxed_slice(),
            cells: cells.into_boxed_slice(),
            update: update.into_boxed_slice(),
            occurrences: order.to_vec().into_boxed_slice(),
            outside: outside.into_boxed_slice(),
            artboard,
        });
        for (index, occurrence) in order.iter().enumerate() {
            if graph.nodes[index].kind == Kind::Outside {
                continue;
            }
            occurrence.with_component_mut(|component| {
                component.bind_graph_dirt(Rc::clone(&graph), index as u32)
            });
            if let ComponentOccurrenceHandle::Authored(handle) = occurrence {
                handle.bind_component_graph(Rc::clone(&graph));
            }
        }
        graph
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
    pub(crate) fn add_dirt(&self, index: u32, value: ComponentDirt, recurse: bool) -> bool {
        #[cfg(feature = "tools")]
        trace::event(self, 'A', index, value, recurse);
        let i = index as usize;
        if self.nodes[i].kind == Kind::Outside {
            return self.occurrences[i].add_dirt(value, recurse);
        }
        if !self.mark(i, value) {
            return false;
        }
        if recurse {
            self.cascade(index, value);
        }
        true
    }

    /// Component::addDirt's own work for member `i`: the duplicate-bit return,
    /// the dirt write, onDirty and the Artboard notification. Returns whether
    /// it added dirt (so the caller continues into its dependents).
    #[inline]
    fn mark(&self, i: usize, value: ComponentDirt) -> bool {
        let node = &self.nodes[i];
        let live = match node.kind {
            Kind::Helper => self.occurrences[i].is_live_helper(),
            // Retired authored members behave like a stale handle.
            _ => node.alive.get(),
        };
        if !live {
            return false;
        }
        let dirt = node.dirt.get();
        if dirt.contains(value) {
            return false;
        }
        let dirt = dirt | value;
        node.dirt.set(dirt);
        match node.kind {
            Kind::Noop => {}
            Kind::Handler => {
                let handler = self.on_dirty[i].expect("handler kind records its handler");
                handler(self.authored(i), dirt)
            }
            Kind::Receiver => {
                self.authored(i).with_mut(|object| {
                    object.component_on_dirty(dirt);
                });
            }
            Kind::Cells => match &self.cells[i] {
                Some(NodeCells::Path(path)) => path.on_dirty(self.authored(i), dirt),
                _ => unreachable!("cells kind records its cells"),
            },
            Kind::Helper => match &self.cells[i] {
                Some(NodeCells::Composer(shape)) => shape.composer_on_dirty(),
                _ => self.occurrences[i].on_dirty_helper(dirt),
            },
            Kind::Outside => unreachable!("outside occurrences keep their own path"),
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

    /// DependencyHelper::addDirtToDependents over the static edges, as an
    /// explicit depth-first walk: each dependent's whole cascade finishes
    /// before its next sibling, exactly as upstream's recursion orders it.
    pub(crate) fn add_dirt_to_dependents(&self, index: u32, value: ComponentDirt) {
        #[cfg(feature = "tools")]
        trace::event(self, 'P', index, value, true);
        self.cascade(index, value);
    }

    fn cascade(&self, index: u32, value: ComponentDirt) {
        let node = &self.nodes[index as usize];
        let (mut position, mut end) = (node.dependents_start, node.dependents_end);
        let mut stack = CascadeStack::new();
        loop {
            if position == end {
                let Some((resume, resume_end)) = stack.pop() else {
                    return;
                };
                (position, end) = (resume, resume_end);
                continue;
            }
            let dependent = self.dependents[position as usize];
            position += 1;
            if dependent & OUTSIDE_DEPENDENT != 0 {
                self.outside[(dependent & !OUTSIDE_DEPENDENT) as usize].add_dirt(value, true);
                continue;
            }
            let member = dependent as usize;
            if self.nodes[member].kind == Kind::Outside {
                self.occurrences[member].add_dirt(value, true);
                continue;
            }
            if !self.mark(member, value) {
                continue;
            }
            let child = &self.nodes[member];
            if child.dependents_start != child.dependents_end {
                stack.push((position, end));
                (position, end) = (child.dependents_start, child.dependents_end);
            }
        }
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
            for i in 0..count {
                dirty.set_depth(i as u32);
                if !self.update_entry(i) {
                    continue;
                }
                if dirty.depth() < i as u32 {
                    break;
                }
            }
            step += 1;
        }
    }

    fn update_entry(&self, i: usize) -> bool {
        let node = &self.nodes[i];
        match node.kind {
            Kind::Outside => return self.occurrences[i].update_if_dirty(),
            Kind::Helper => {
                let dirt = node.dirt.get();
                if !needs_update(dirt) {
                    return false;
                }
                node.dirt.set(ComponentDirt::NONE);
                self.occurrences[i].update_helper(dirt);
                return true;
            }
            Kind::Noop | Kind::Handler | Kind::Receiver | Kind::Cells => {}
        }
        assert!(node.alive.get(), "live component in dependency graph");
        let dirt = node.dirt.get();
        if !needs_update(dirt) {
            return false;
        }
        node.dirt.set(ComponentDirt::NONE);
        let handle = self.authored(i);
        if let Some(update) = self.update[i] {
            update(handle, dirt);
        } else {
            handle.with_mut(|object| object.component_update(dirt));
        }
        true
    }
}

#[inline]
fn needs_update(dirt: ComponentDirt) -> bool {
    dirt != ComponentDirt::NONE && !dirt.contains(ComponentDirt::COLLAPSED)
}

/// Resume points for add_dirt_to_dependents: inline for typical depths,
/// spilling to the heap only for unusually deep dependency chains.
struct CascadeStack {
    inline: [(u32, u32); 32],
    len: usize,
    spill: Vec<(u32, u32)>,
}

impl CascadeStack {
    #[inline]
    fn new() -> Self {
        Self {
            inline: [(0, 0); 32],
            len: 0,
            spill: Vec::new(),
        }
    }

    #[inline]
    fn push(&mut self, entry: (u32, u32)) {
        if self.len < self.inline.len() {
            self.inline[self.len] = entry;
            self.len += 1;
        } else {
            self.spill.push(entry);
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<(u32, u32)> {
        if let Some(entry) = self.spill.pop() {
            return Some(entry);
        }
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(self.inline[self.len])
    }
}

/// One member's dirt in its graph's node array. The graph order doubles as the
/// index; edges and order are fixed once the graph is built.
pub(crate) struct DirtBinding {
    nodes: Rc<[Node]>,
    index: u32,
}

impl DirtBinding {
    #[inline]
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
    use super::{ComponentDirt, ComponentGraph, Kind};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::fmt::Write as _;

    struct Recorder {
        out: String,
        graphs: HashMap<*const ComponentGraph, u32>,
    }

    thread_local! {
        static RECORDER: RefCell<Option<Recorder>> = const { RefCell::new(None) };
    }

    pub fn start() {
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
        std::fs::write(path, out.unwrap_or_default())
    }

    pub(super) fn event(
        graph: &ComponentGraph,
        kind: char,
        index: u32,
        value: ComponentDirt,
        recurse: bool,
    ) {
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
                    let kind = match node.kind {
                        Kind::Outside => 0,
                        Kind::Noop => 1,
                        // Replays treat cells dispatch as a handler call.
                        Kind::Handler | Kind::Cells => 2,
                        Kind::Receiver => 3,
                        Kind::Helper => 4,
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
