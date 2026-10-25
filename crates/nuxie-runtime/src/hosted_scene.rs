//! Nuxie hosted-instance index. Membership edits rebuild the row index once;
//! frame paths iterate active occurrences and never enter collapsed rows.
use crate::{
    Artboard, CoreHandle, RuntimeArtboardInstanceHandle,
    source::{
        artboard_component_list::ArtboardComponentList,
        core::{CoreObject, CoreType},
    },
};
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
};
// Only a root runtime occurrence in a file with published records owns this.
// Descendants resolve the owner through their existing host ancestry. There is
// no global route, revision or retained-handle store to sweep later.
pub(crate) struct SceneState {
    root: CoreHandle,
    // Classification belongs to this Page occurrence, not shared definitions.
    screen_names: HashSet<String>,
    revision: Cell<u64>,
    reconciled_revision: Cell<Option<u64>>,
    pub(crate) plan: RefCell<crate::live_move::plan::Plan>,
    index: RefCell<CachedIndex>,
    pub(crate) loans: RefCell<HashMap<CoreHandle, HashSet<CoreHandle>>>,
    // Unit identity -> storage of its explicit VM. Only the scene can retain
    // another row; destination slots must never form row-to-row owner cycles.
    pub(crate) unit_vm_storage: RefCell<HashMap<CoreHandle, RuntimeArtboardInstanceHandle>>,
}
impl SceneState {
    pub(crate) fn is_root(&self, board: &CoreHandle) -> bool {
        self.root == *board
    }
    pub(crate) fn prepare(&self) -> anyhow::Result<()> {
        let root = self
            .root
            .runtime_arena_owner()
            .ok_or_else(|| anyhow::anyhow!("retired scene"))?;
        self.plan.borrow_mut().prepare(&root)
    }
    /// Native structure is authoritative. All entry points share this revision
    /// check; no lifecycle path has to know which live-move caches it invalidates.
    pub(crate) fn reconcile(&self) {
        let revision = self.revision.get();
        if self.reconciled_revision.get() == Some(revision) {
            return;
        }
        let Some(root) = self.root.runtime_arena_owner() else {
            return;
        };
        let (boards, hosts) = {
            let mut index = self.index.borrow_mut();
            index.active(&root);
            (
                index.retained.iter().cloned().collect::<HashSet<_>>(),
                index.hosts.clone(),
            )
        };
        let mounted_hosts = hosts.iter().cloned().collect::<HashSet<_>>();
        for host in hosts {
            crate::live_move::reconcile_slot(&host, &boards, &mounted_hosts);
            host.with_downcast_mut::<ArtboardComponentList, _>(|list| {
                list.live_move_rebuild_collapse_mask()
            });
        }
        self.loans.borrow_mut().retain(|list, slots| {
            slots.retain(|slot| mounted_hosts.contains(slot));
            mounted_hosts.contains(list) && !slots.is_empty()
        });
        // Keep removed storage alive until the map borrow ends. Its destructor
        // can run native cleanup, including queries back into this occurrence.
        let mut released = Vec::new();
        self.unit_vm_storage.borrow_mut().retain(|unit, owner| {
            if boards.contains(unit) {
                true
            } else {
                released.push(owner.clone());
                false
            }
        });
        drop(released);
        self.reconciled_revision.set(Some(revision));
    }
    pub(crate) fn new(root: CoreHandle) -> std::rc::Rc<Self> {
        let screen_names = root
            .with_downcast::<Artboard, _>(|b| {
                b.find_all_handles::<crate::live_move::records::LiveStep>()
            })
            .unwrap_or_default()
            .iter()
            .filter_map(|step| {
                step.with_downcast::<crate::live_move::records::LiveStep, _>(|s| {
                    [s.values.from_artboard.clone(), s.values.to_artboard.clone()]
                })
            })
            .flatten()
            .collect();
        let state = std::rc::Rc::new(Self {
            root,
            screen_names,
            revision: Cell::new(0),
            reconciled_revision: Cell::new(None),
            plan: RefCell::new(Default::default()),
            index: RefCell::new(Default::default()),
            loans: RefCell::new(HashMap::new()),
            unit_vm_storage: RefCell::new(HashMap::new()),
        });
        #[cfg(feature = "test-support")]
        SCENES.with(|s| s.borrow_mut().push(std::rc::Rc::downgrade(&state)));
        state
    }
}
impl Drop for SceneState {
    fn drop(&mut self) {
        crate::live_move::frame::clear_for(&self.root);
    }
}
pub(crate) fn scene_for_object(object: &dyn CoreObject) -> Option<std::rc::Rc<SceneState>> {
    if let Some(board) = object.as_artboard() {
        if board.host().is_none() && board.parent_artboard().is_none() {
            if let Some(Some(scene)) = &board.live_move_scene {
                return Some(scene.clone());
            }
        }
        return board.parent_artboard().and_then(|b| scene_for(&b));
    }
    object
        .as_component()?
        .artboard_handle()
        .and_then(|b| scene_for(&b))
}
pub(crate) fn scene_for(owner: &CoreHandle) -> Option<std::rc::Rc<SceneState>> {
    owner.with(scene_for_object).flatten()
}
pub fn enabled(root: &RuntimeArtboardInstanceHandle) -> bool {
    root_scene(root).is_some()
}
/// Only explicit live operations may opt a Page in. Native structure hooks
/// resolve an existing enclosing scene, never create state on ordinary content.
pub(crate) fn scene_for_operation(owner: &CoreHandle) -> Option<std::rc::Rc<SceneState>> {
    if let Some(scene) = scene_for(owner) {
        return Some(scene);
    }
    let mut board = owner
        .with(|o| {
            if o.as_artboard().is_some() {
                Some(owner.clone())
            } else {
                o.as_component().and_then(|c| c.artboard_handle())
            }
        })
        .flatten()?;
    loop {
        let parent = board
            .with_downcast::<Artboard, _>(Artboard::parent_artboard)
            .flatten();
        match parent {
            Some(parent) => board = parent,
            None => return root_scene(&board.runtime_arena_owner()?),
        }
    }
}
pub(crate) fn is_screen_list(list: &CoreHandle) -> bool {
    let Some(scene) = scene_for(list) else {
        return false;
    };
    list.with_downcast::<ArtboardComponentList, _>(|l| {
        l.name() == "screens" && l.artboard_handle().is_some_and(|b| scene.is_root(&b))
    })
    .unwrap_or(false)
}
pub(crate) fn is_screen(board: &CoreHandle) -> bool {
    let Some(scene) = scene_for(board) else {
        return false;
    };
    board
        .with_downcast::<Artboard, _>(|b| {
            !scene.is_root(board)
                && b.find_handle::<ArtboardComponentList>("screens").is_none()
                && scene.screen_names.contains(b.name())
        })
        .unwrap_or(false)
}
pub(crate) fn is_live_slot(slot: &CoreHandle) -> bool {
    let Some(board) = slot
        .with(|o| {
            o.as_nested_artboard()
                .filter(|s| {
                    s.children()
                        .iter()
                        .any(|c| c.is_type_of(crate::live_move::records::LiveSlot::TYPE_KEY))
                })
                .and_then(|s| s.parent_artboard_handle())
        })
        .flatten()
    else {
        return false;
    };
    let Some(host) = board.with_downcast::<Artboard, _>(Artboard::host).flatten() else {
        return false;
    };
    is_screen(&board)
        && is_screen_list(&host)
        && host
            .with_downcast::<ArtboardComponentList, _>(|list| {
                (0..list.artboard_count()).any(|i| {
                    #[cfg(feature = "test-support")]
                    record_walk(Walk::ListLookup);
                    list.artboard_instance(i as i32)
                        .is_some_and(|row| row.core_handle() == board)
                })
            })
            .unwrap_or(false)
}
pub(crate) fn membership_changed(object: &dyn CoreObject) {
    if let Some(scene) = scene_for_object(object) {
        scene.revision.set(scene.revision.get().wrapping_add(1));
    }
}
pub(crate) fn membership_changed_for(owner: &CoreHandle) {
    owner.with(membership_changed);
}
pub fn membership_revision(root: &CoreHandle) -> u64 {
    scene_for(root).map_or(0, |s| s.revision.get())
}

// Stateless adapter. Ordinary files use a transient native host walk; they
// never create an index, a stamp or any retained live-move identity storage.
#[derive(Default)]
pub struct HostedIndex;
impl HostedIndex {
    pub fn active(&mut self, root: &RuntimeArtboardInstanceHandle) -> Vec<CoreHandle> {
        if let Some(scene) = scene_for(&root.core_handle()) {
            return scene.index.borrow_mut().active(root).to_vec();
        }
        native_boards(root)
    }
    pub fn retained(&mut self, root: &RuntimeArtboardInstanceHandle) -> Vec<CoreHandle> {
        if let Some(scene) = scene_for(&root.core_handle()) {
            let mut index = scene.index.borrow_mut();
            index.active(root);
            return index.retained.clone();
        }
        native_boards(root)
    }
    /// Inline VM components belong to the active scene even when their slot's
    /// provisional child is discarded. Index them only on membership changes.
    /// Ordinary files retain the existing context-only scope and pay nothing.
    pub fn active_owned_models(&mut self, root: &RuntimeArtboardInstanceHandle) -> Vec<CoreHandle> {
        let Some(scene) = scene_for(&root.core_handle()) else {
            return Vec::new();
        };
        let mut index = scene.index.borrow_mut();
        index.active(root);
        index.owned_models.clone()
    }
    pub(crate) fn inactive_models(
        &self,
        root: &RuntimeArtboardInstanceHandle,
    ) -> HashSet<CoreHandle> {
        scene_for(&root.core_handle())
            .map_or_else(HashSet::new, |s| s.index.borrow().inactive_models.clone())
    }
    pub(crate) fn has_collapsed_rows(&self, root: &RuntimeArtboardInstanceHandle) -> bool {
        scene_for(&root.core_handle()).is_some_and(|s| s.index.borrow().has_collapsed_rows)
    }
}
fn native_boards(root: &RuntimeArtboardInstanceHandle) -> Vec<CoreHandle> {
    let mut result = Vec::new();
    let mut pending = vec![root.clone()];
    let mut seen = HashSet::new();
    while let Some(board) = pending.pop() {
        if !seen.insert(board.core_handle()) {
            continue;
        }
        result.push(board.core_handle());
        for host in host_nodes(&board) {
            host.with(|o| pending.extend(active_instances(o)));
        }
    }
    result
}

/// The flat part of a scene occurrence's index, rebuilt on structural edits. Handles
/// are weak observations. The scene graph remains the sole lifetime owner.
#[derive(Default)]
struct CachedIndex {
    revision: Option<u64>,
    root: Option<CoreHandle>,
    boards: Vec<CoreHandle>,
    retained: Vec<CoreHandle>,
    hosts: Vec<CoreHandle>,
    owned_models: Vec<CoreHandle>,
    inactive_models: HashSet<CoreHandle>,
    has_collapsed_rows: bool,
}
impl CachedIndex {
    pub fn active(&mut self, root: &RuntimeArtboardInstanceHandle) -> &[CoreHandle] {
        let revision = membership_revision(&root.core_handle());
        if self.revision == Some(revision) && self.root.as_ref() == Some(&root.core_handle()) {
            return &self.boards;
        }
        #[cfg(feature = "test-support")]
        record_walk(Walk::IndexBuild);
        self.boards.clear();
        self.retained.clear();
        self.hosts.clear();
        self.owned_models.clear();
        self.inactive_models.clear();
        self.has_collapsed_rows = false;
        let mut active_models = HashSet::new();
        // Structural work only. Discover retained model heads once so frame
        // checkpoint/subscription walks can skip a sleeping graph at its edge.
        let mut pending = vec![(root.clone(), true)];
        let mut seen = HashSet::new();
        let mut retained = HashSet::new();
        while let Some((board, active)) = pending.pop() {
            #[cfg(feature = "test-support")]
            record_walk(Walk::IndexNode);
            if !seen.insert((board.core_handle(), active)) {
                continue;
            }
            let first = retained.insert(board.core_handle());
            if first {
                self.retained.push(board.core_handle());
            }
            if active {
                self.boards.push(board.core_handle());
                let models = board.with_artboard(|board| {
                    board.find_all_handles::<crate::source::viewmodel::viewmodel_instance::ViewModelInstance>()
                });
                for model in models {
                    // An empty or transported slot's old inline VM still
                    // belongs to this scene, but no longer drives a child.
                    // Keep its identity in scope without walking it each frame.
                    let dormant = model
                        .with(|object| object.component_parent_handle())
                        .flatten()
                        .and_then(|parent| {
                            parent
                                .with(|object| {
                                    object.as_nested_artboard().map(|slot| {
                                        slot.active_view_model_instance.as_ref() != Some(&model)
                                            && slot.view_model_instance.as_ref() != Some(&model)
                                    })
                                })
                                .flatten()
                        })
                        .unwrap_or(false);
                    if dormant {
                        self.inactive_models.insert(model.clone());
                    }
                    self.owned_models.push(model);
                }
            }
            if let Some(model) = board
                .data_context()
                .and_then(|c| c.with_context(|c| c.main_view_model_instance()))
            {
                if active {
                    active_models.insert(model);
                } else {
                    self.inactive_models.insert(model);
                }
            }
            let hosts = host_nodes(&board);
            if first {
                self.hosts.extend(hosts.iter().cloned());
            }
            for host in hosts {
                let children = host
                    .with(|object| {
                        let Some(host) = object.as_artboard_host() else {
                            return Vec::new();
                        };
                        if let Some(list) = object
                            .as_registry_any()
                            .downcast_ref::<ArtboardComponentList>()
                        {
                            if list.live_move_has_sleeping_rows() {
                                self.has_collapsed_rows = true;
                            }
                            return (0..list.artboard_count())
                                .filter_map(|i| {
                                    list.artboard_instance(i as i32).map(|b| {
                                        (b, active && !list.live_move_row_collapsed(i as i32))
                                    })
                                })
                                .collect();
                        }
                        (0..host.artboard_count())
                            .filter_map(|i| host.artboard_instance(i as i32).map(|b| (b, active)))
                            .collect()
                    })
                    .unwrap_or_default();
                pending.extend(children);
            }
        }
        self.inactive_models.retain(|m| !active_models.contains(m));
        self.revision = Some(revision);
        self.root = Some(root.core_handle());
        &self.boards
    }
}
// Structural discovery includes every existing ArtboardHost, including paint
// artboards. The steady frame reads only the resulting flat occurrence index.
fn host_nodes(board: &RuntimeArtboardInstanceHandle) -> Vec<CoreHandle> {
    board.with_artboard(|b| {
        b.objects()
            .iter()
            .flatten()
            .filter(|h| {
                #[cfg(feature = "test-support")]
                record_walk(Walk::IndexNode);
                h.with(|o| o.as_artboard_host().is_some()).unwrap_or(false)
            })
            .cloned()
            .collect()
    })
}
/// Host edges use the same row membership as advancement, layout and hits.
/// Ordinary hosts retain upstream enumeration, including whole-list collapse.
pub fn active_instances(
    object: &dyn CoreObject,
) -> impl Iterator<Item = RuntimeArtboardInstanceHandle> + '_ {
    let host = object.as_artboard_host();
    let indices = object
        .as_registry_any()
        .downcast_ref::<ArtboardComponentList>()
        .filter(|list| list.live_move_has_sleeping_rows())
        .map(ArtboardComponentList::live_move_frame_indices);
    let count = indices.map_or_else(|| host.map_or(0, |h| h.artboard_count()), |rows| rows.len());
    (0..count)
        .filter_map(move |i| host?.artboard_instance(indices.map_or(i as i32, |rows| rows[i])))
}
// Release the host borrow before invoking component callbacks, as the native
// realized-row walker does. Ordinary lists use that native path immediately.
pub(crate) fn for_each_active_row(
    owner: &CoreHandle,
    mut body: impl FnMut(RuntimeArtboardInstanceHandle),
) {
    let sleeping = owner
        .with_downcast::<ArtboardComponentList, _>(
            ArtboardComponentList::live_move_has_sleeping_rows,
        )
        .unwrap_or(false);
    if !sleeping {
        return ArtboardComponentList::for_each_realized_occurrence(owner, |_, row| body(row));
    }
    let mut position = 0;
    while let Some(index) = owner
        .with_downcast::<ArtboardComponentList, _>(|list| {
            list.live_move_frame_indices().get(position).copied()
        })
        .flatten()
    {
        if let Some(row) = owner
            .with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(index))
            .flatten()
        {
            body(row);
        }
        position += 1;
    }
}
pub(crate) fn root_scene(root: &RuntimeArtboardInstanceHandle) -> Option<std::rc::Rc<SceneState>> {
    let source = root.with_artboard(|b| {
        if b.host().is_some() || b.parent_artboard().is_some() {
            return None;
        }
        Some((b.live_move_scene.clone(), b.artboard_source_handle()))
    })?;
    if let Some(decision) = source.0 {
        return decision;
    }
    // Decide once for this occurrence, including a negative answer. Native
    // hooks only read the decision; explicit entry points initialize it here.
    // A Screen used by paint or an ordinary nested host gets no scene state.
    let records = source.1.as_ref().is_some_and(definition_has_records);
    // Only a root with a screens list opts in: a $nuxie.page by shape, or a
    // root whose file declares live-move steps. Misplaced steps elsewhere
    // stay ignored.
    let declared = root.with_artboard(|b| {
        b.find_handle::<ArtboardComponentList>("screens").is_some()
            && (b.name() == "$nuxie.page"
                || (records
                    && !b
                        .find_all_handles::<crate::live_move::records::LiveStep>()
                        .is_empty()))
    });
    let scene = declared.then(|| SceneState::new(root.core_handle()));
    root.with_artboard_mut(|b| b.live_move_scene = Some(scene.clone()));
    scene
}
pub(crate) fn is_active_artboard(board: &CoreHandle) -> bool {
    board
        .with(|o| !crate::live_move::effectively_hidden(o, None))
        .unwrap_or(false)
}

// Observability has no shipping symbols, storage or branches.
#[cfg(feature = "test-support")]
#[derive(Clone, Copy)]
pub enum Walk {
    CapabilityScan,
    IntrinsicMeasure,
    IndexBuild,
    Advance,
    Layout,
    Draw,
    Hit,
    VideoEnumeration,
    VideoLookup,
    SemanticValidation,
    OwnerCapture,
    Checkpoint,
    ValueGraph,
    EventDrain,
    LoanRow,
    ListLookup,
    RowAccess,
    Visibility,
    IndexNode,
    ObjectLookup,
    CollapseNode,
    StatusCheckpoint,
    CheckedRoot,
    CheckedScreenSearch,
}
#[cfg(feature = "test-support")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WalkCounts {
    pub capability_scan: u64,
    pub intrinsic_measure: u64,
    pub index_build: u64,
    pub advance: u64,
    pub layout: u64,
    pub draw: u64,
    pub hit: u64,
    pub video_enumeration: u64,
    pub video_lookup: u64,
    pub semantic_validation: u64,
    pub owner_capture: u64,
    pub checkpoint: u64,
    pub value_graph: u64,
    pub event_drain: u64,
    pub loan_row: u64,
    /// Entries compared by reverse row/active-membership list searches.
    pub list_lookup: u64,
    /// Constant-time indexed item, artboard and machine reads.
    pub row_access: u64,
    pub visibility: u64,
    /// Nodes/objects visited while discovering structural scene membership.
    pub index_node: u64,
    pub object_lookup: u64,
    pub collapse_node: u64,
    pub status_checkpoint: u64,
    /// Checked-script root resolver calls, including named-root fast paths.
    pub checked_root: u64,
    /// Unnamed-root searches through awake Screen contexts.
    pub checked_screen_search: u64,
}
#[cfg(feature = "test-support")]
thread_local! { static COUNTS: Cell<Option<WalkCounts>> = const { Cell::new(None) }; }
#[cfg(feature = "test-support")]
pub fn measure_walks(enabled: bool) -> WalkCounts {
    if enabled {
        PHASES.with(|p| p.borrow_mut().clear());
    }
    COUNTS.with(|v| {
        v.replace(enabled.then(WalkCounts::default))
            .unwrap_or_default()
    })
}
#[cfg(feature = "test-support")]
pub fn record_walk(walk: Walk) {
    COUNTS.with(|v| {
        if let Some(mut c) = v.get() {
            let target = match walk {
                Walk::CapabilityScan => &mut c.capability_scan,
                Walk::IntrinsicMeasure => &mut c.intrinsic_measure,
                Walk::IndexBuild => &mut c.index_build,
                Walk::Advance => &mut c.advance,
                Walk::Layout => &mut c.layout,
                Walk::Draw => &mut c.draw,
                Walk::Hit => &mut c.hit,
                Walk::VideoEnumeration => &mut c.video_enumeration,
                Walk::VideoLookup => &mut c.video_lookup,
                Walk::SemanticValidation => &mut c.semantic_validation,
                Walk::OwnerCapture => &mut c.owner_capture,
                Walk::Checkpoint => &mut c.checkpoint,
                Walk::ValueGraph => &mut c.value_graph,
                Walk::EventDrain => &mut c.event_drain,
                Walk::LoanRow => &mut c.loan_row,
                Walk::ListLookup => &mut c.list_lookup,
                Walk::RowAccess => &mut c.row_access,
                Walk::Visibility => &mut c.visibility,
                Walk::IndexNode => &mut c.index_node,
                Walk::ObjectLookup => &mut c.object_lookup,
                Walk::CollapseNode => &mut c.collapse_node,
                Walk::StatusCheckpoint => &mut c.status_checkpoint,
                Walk::CheckedRoot => &mut c.checked_root,
                Walk::CheckedScreenSearch => &mut c.checked_screen_search,
            };
            *target += 1;
            v.set(Some(c));
        }
    });
}

#[cfg(feature = "test-support")]
thread_local! { static PHASES: RefCell<std::collections::BTreeMap<&'static str, std::time::Duration>> = RefCell::new(std::collections::BTreeMap::new()); }
/// Phase timings for the host probe. No timing state exists in shipping builds.
#[cfg(feature = "test-support")]
pub struct StepTimer {
    phase: &'static str,
    start: Option<std::time::Instant>,
}
#[cfg(feature = "test-support")]
impl StepTimer {
    pub fn new(phase: &'static str) -> Self {
        Self {
            phase,
            start: COUNTS
                .with(|v| v.get().is_some())
                .then(std::time::Instant::now),
        }
    }
    pub fn mark(&mut self, phase: &'static str) {
        if let Some(start) = self.start {
            PHASES.with(|p| *p.borrow_mut().entry(self.phase).or_default() += start.elapsed());
            self.start = Some(std::time::Instant::now());
        }
        self.phase = phase;
    }
}
#[cfg(feature = "test-support")]
impl Drop for StepTimer {
    fn drop(&mut self) {
        self.mark("done");
    }
}
#[cfg(feature = "test-support")]
pub fn measured_step_phases() -> Vec<(&'static str, std::time::Duration)> {
    PHASES.with(|p| {
        p.borrow()
            .iter()
            .map(|(name, time)| (*name, *time))
            .collect()
    })
}

#[cfg(feature = "test-support")]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BookkeepingCounts {
    pub scenes: usize,
    pub routes: usize,
    pub revisions: usize,
    pub loan_lists: usize,
    pub unit_vm_storage: usize,
    pub projections: usize,
    pub active_frames: usize,
    pub indexed_handles: usize,
    pub row_indices: usize,
    pub slot_states: usize,
}
#[cfg(feature = "test-support")]
thread_local! {
    static SCENES: RefCell<Vec<std::rc::Weak<SceneState>>> = const { RefCell::new(Vec::new()) };
    static ALLOCATIONS: Cell<[usize; 2]> = const { Cell::new([0,0]) };
}
#[cfg(feature = "test-support")]
pub(crate) fn bookkeeping_allocation(kind: usize, add: bool) {
    ALLOCATIONS.with(|a| {
        let mut counts = a.get();
        if add {
            counts[kind] += 1;
        } else {
            counts[kind] -= 1;
        }
        a.set(counts);
    });
}
#[cfg(feature = "test-support")]
pub fn bookkeeping_counts() -> BookkeepingCounts {
    let mut result = BookkeepingCounts {
        active_frames: crate::live_move::frame::active_scope_count(),
        ..Default::default()
    };
    SCENES.with(|scenes| {
        scenes.borrow_mut().retain(|weak| {
            let Some(scene) = weak.upgrade() else {
                return false;
            };
            result.scenes += 1;
            result.revisions += 1;
            result.loan_lists += scene.loans.borrow().len();
            result.unit_vm_storage += scene.unit_vm_storage.borrow().len();
            result.projections += usize::from(scene.plan.borrow().has_projection());
            let index = scene.index.borrow();
            result.indexed_handles += index.boards.len()
                + index.retained.len()
                + index.hosts.len()
                + index.owned_models.len()
                + index.inactive_models.len();
            true
        })
    });
    ALLOCATIONS.with(|a| {
        let [rows, slots] = a.get();
        result.row_indices = rows;
        result.slot_states = slots;
    });
    result
}

/// Query native definition metadata before an explicit Page operation opts in.
/// The arena is shared with its file, including tools replacements. Ordinary
/// definitions return before scanning for the compiler's screens list.
pub(crate) fn definition_has_records(source: &CoreHandle) -> bool {
    let records = source
        .with_downcast::<Artboard, _>(Artboard::artboard_source_handle)
        .flatten()
        .and_then(|source| source.retain_arena())
        .is_some_and(|arena| {
            #[cfg(feature = "test-support")]
            record_walk(Walk::CapabilityScan);
            arena.contains_types(62000..=62002)
        });
    records
        && source
            .with_downcast::<Artboard, _>(|b| {
                b.find_handle::<ArtboardComponentList>("screens").is_some()
            })
            .unwrap_or(false)
}
