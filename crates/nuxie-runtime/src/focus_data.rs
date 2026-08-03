//! Direct FocusData retained ownership port of pinned src/focus_data.cpp (B6-0209).

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[cfg(test)]
use crate::input::FocusEvent;
use crate::input::{
    FocusBounds, FocusEdgeBehavior, FocusEventKind, FocusManager, FocusNode, FocusNodeId,
    FocusPoint, RuntimeFocusable,
};
use crate::parent_traversal::{ParentTraversal, ParentTraversalFrame};
use crate::properties::property_key_for_name;
use crate::{ArtboardInstance, Mat2D};

// Retained authored FocusData ownership. FocusManager deliberately owns no
// Rive-object knowledge; this layer mirrors Artboard::buildFocusTree and keeps
// occurrence identity stable while nested artboards and component-list rows
// are rebuilt or reordered.

const FOCUS_KEY_ROOT: u64 = 1;
const FOCUS_KEY_NESTED: u64 = 2;
const FOCUS_KEY_LIST_SCOPE: u64 = 3;
const FOCUS_KEY_LIST_ROW: u64 = 4;
const FOCUS_KEY_AUTHORED: u64 = 5;
const FOCUS_KEY_NESTED_CHILD: u64 = 6;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RuntimeFocusOccurrenceKey(Vec<u64>);

impl RuntimeFocusOccurrenceKey {
    fn root(graph_global_id: u32, owner_identity: u64) -> Self {
        Self(vec![
            FOCUS_KEY_ROOT,
            u64::from(graph_global_id),
            owner_identity,
        ])
    }

    fn child(&self, tag: u64, first: u64, second: u64) -> Self {
        let mut value = self.0.clone();
        value.extend([tag, first, second]);
        Self(value)
    }

    fn is_within(&self, root: &Self) -> bool {
        self.0.starts_with(&root.0)
    }
}

#[derive(Debug, Clone, Default)]
struct RuntimeFocusDomain {
    manager: FocusManager,
    /// Stable owner identities for `FocusData::m_focusNode` plus structural
    /// nested/list scopes. The registry lives beside the shared manager so an
    /// Artboard mutation callback can update its retained subtree directly.
    retained_nodes: BTreeMap<RuntimeFocusOccurrenceKey, FocusNodeId>,
    retained_parents: BTreeMap<RuntimeFocusOccurrenceKey, Option<RuntimeFocusOccurrenceKey>>,
    focus_nodes: BTreeMap<(u64, usize), FocusNodeId>,
    focus_targets: BTreeMap<(u64, usize), FocusNodeId>,
    mounts: BTreeMap<u64, RuntimeFocusMount>,
}

impl RuntimeFocusDomain {
    fn create_node(&mut self, node: FocusNode) -> FocusNodeId {
        let focusable = node.focusable();
        let node_id = self.manager.create_node(node);
        if let Some(focusable) = focusable {
            self.focus_nodes.insert(
                (focusable.owner_identity, focusable.focus_data_local),
                node_id,
            );
            self.focus_targets
                .insert((focusable.owner_identity, focusable.target_local), node_id);
        }
        node_id
    }
}

#[derive(Debug, Clone)]
struct RuntimeFocusMount {
    occurrence_key: RuntimeFocusOccurrenceKey,
    parent_focus: Option<RuntimeFocusOccurrenceKey>,
    inherited_eligible: bool,
    root_transform: Mat2D,
    sibling_index: usize,
}

fn replace_focusable(
    domain: &mut RuntimeFocusDomain,
    node_id: FocusNodeId,
    mut next: Option<RuntimeFocusable>,
) {
    let previous = domain.manager.focusable(node_id);
    if let (Some(previous), Some(next_focusable)) = (previous, next.as_mut())
        && previous.owner_identity == next_focusable.owner_identity
        && previous.target_local == next_focusable.target_local
        && previous.focus_data_local == next_focusable.focus_data_local
    {
        next_focusable.accepts_keyboard_input = previous.accepts_keyboard_input;
    }
    if let Some(previous) = previous {
        domain
            .focus_nodes
            .remove(&(previous.owner_identity, previous.focus_data_local));
        domain
            .focus_targets
            .remove(&(previous.owner_identity, previous.target_local));
    }
    domain.manager.set_node_focusable(node_id, next);
    if let Some(next) = next {
        domain
            .focus_nodes
            .insert((next.owner_identity, next.focus_data_local), node_id);
        domain
            .focus_targets
            .insert((next.owner_identity, next.target_local), node_id);
    }
}

/// One state-machine instance's authored focus domain.
///
/// The keys describe concrete mounted occurrences, not just file-global
/// objects. A component-list row therefore retains its FocusNode when moved,
/// while a genuinely removed row is blurred and discarded.
#[derive(Debug)]
pub(crate) struct RuntimeFocusTree {
    inert: bool,
    domain: Rc<RefCell<RuntimeFocusDomain>>,
    owner_identity: u64,
}

impl Default for RuntimeFocusTree {
    fn default() -> Self {
        Self {
            inert: false,
            domain: Rc::new(RefCell::new(RuntimeFocusDomain::default())),
            owner_identity: 0,
        }
    }
}

impl Clone for RuntimeFocusTree {
    fn clone(&self) -> Self {
        // A public Rust state-machine snapshot is a new occurrence. Copy the
        // retained focus domain rather than aliasing focus mutations back to
        // the source occurrence. Nested machines are reattached to the new
        // root domain when their owning parent instance is constructed.
        let domain = self.domain.borrow().clone();
        // Public Clone is Rust's explicit state snapshot. Preserve owned
        // pending focus/blur values in the new non-aliased manager, just as
        // StateMachineInstance preserves callbacks already translated into
        // its own queue. A cold remount still starts empty through `default`.
        Self {
            inert: self.inert,
            domain: Rc::new(RefCell::new(domain)),
            owner_identity: self.owner_identity,
        }
    }
}

impl RuntimeFocusTree {
    pub(crate) fn owner_identity(&self) -> u64 {
        self.owner_identity
    }

    /// Owner-safe identity comparison for the StateMachineInstance selection
    /// seam. This observes shared ownership only; it does not expose or port
    /// RECORDED focus-manager internals from manifest row B6-0238.
    pub(crate) fn shares_manager(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.domain, &other.domain)
    }

    /// Create the focus-manager identity used by one state-machine occurrence
    /// without building the authored focus topology yet.
    ///
    /// Pinned C++ constructs `m_focusManager` with the
    /// `StateMachineInstance`, runs every layer's initial entry callbacks, and
    /// only then calls `Artboard::buildFocusTree`
    /// (`state_machine_instance.cpp:1747-1752,2123-2127`). Entry-time focus
    /// actions therefore see an empty manager. `FocusActionTarget` may still
    /// lazily create its one target `FocusNode` through
    /// `FocusData::focusNode`, but that node has no attached descendants until
    /// the final build pass (`focus_action_target.cpp:14-40`;
    /// `focus_data.cpp:55-69`).
    pub(crate) fn new_unsynchronized(artboard: &ArtboardInstance) -> Self {
        Self {
            owner_identity: artboard.instance_identity(),
            ..Self::default()
        }
    }

    /// Perform the first complete authored-tree build after initial layer
    /// callbacks have finished.
    pub(crate) fn synchronize_after_layer_initialization(&mut self, artboard: &ArtboardInstance) {
        self.build_focus_tree(artboard);
        // An empty tree cannot gain authored focus content later: lists
        // and data-bound nested hosts contribute persistent structural scopes
        // even while empty. Keep the common no-focus advance path O(1).
        self.inert = self.domain.borrow().retained_nodes.is_empty();
    }

    /// Install the same manager used by the parent occurrence while retaining
    /// the child occurrence's own authored target namespace. Pinned C++ calls
    /// `setExternalFocusManager` before `syncNestedStateMachine` so the child
    /// contributes to one traversal domain without copying manager state.
    pub(crate) fn external_for_owner(&self, owner_identity: u64) -> Self {
        Self {
            inert: self.inert,
            domain: Rc::clone(&self.domain),
            owner_identity,
        }
    }

    #[inline]
    pub(crate) fn is_inert(&self) -> bool {
        self.inert
    }

    /// Direct port of `Artboard::buildFocusTree`: walk authored child order and
    /// mutate the retained nodes/scopes in place. Unlike the retired `sync`
    /// projection, this runs only at the pinned construction/rebuild sites.
    pub(crate) fn build_focus_tree(&self, artboard: &ArtboardInstance) {
        if self.inert {
            return;
        }
        let root_key =
            RuntimeFocusOccurrenceKey::root(artboard.graph_global_id, artboard.instance_identity());
        self.rebuild_mounted_subtree(
            artboard,
            RuntimeFocusMount {
                occurrence_key: root_key,
                parent_focus: None,
                inherited_eligible: true,
                root_transform: Mat2D::IDENTITY,
                sibling_index: 0,
            },
        );
    }

    fn rebuild_mounted_subtree(&self, artboard: &ArtboardInstance, mount: RuntimeFocusMount) {
        let mut active = BTreeSet::new();
        let mut sibling_counts = BTreeMap::new();
        sibling_counts.insert(mount.parent_focus.clone(), mount.sibling_index);
        build_artboard_focus_tree(
            self,
            artboard,
            &mount.occurrence_key,
            mount.parent_focus.clone(),
            mount.inherited_eligible,
            mount.root_transform,
            &mut active,
            &mut sibling_counts,
        );
        let removed = self
            .domain
            .borrow()
            .retained_nodes
            .keys()
            .filter(|key| key.is_within(&mount.occurrence_key) && !active.contains(*key))
            .cloned()
            .collect::<BTreeSet<_>>();
        let removed_roots = removed
            .iter()
            .filter(|key| {
                self.domain
                    .borrow()
                    .retained_parents
                    .get(*key)
                    .and_then(Option::as_ref)
                    .is_none_or(|parent| !removed.contains(parent))
            })
            .cloned()
            .collect::<Vec<_>>();
        for key in removed_roots {
            let node_id = self.domain.borrow().retained_nodes.get(&key).copied();
            if let Some(node_id) = node_id {
                self.domain.borrow_mut().manager.remove_subtree(node_id);
            }
        }
        let mut domain = self.domain.borrow_mut();
        let live_nodes = domain
            .manager
            .nodes
            .keys()
            .copied()
            .collect::<BTreeSet<_>>();
        domain
            .retained_nodes
            .retain(|_, node_id| live_nodes.contains(node_id));
        let retained_keys = domain
            .retained_nodes
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        domain
            .retained_parents
            .retain(|key, _| retained_keys.contains(key));
        domain
            .focus_nodes
            .retain(|_, node_id| live_nodes.contains(node_id));
        domain
            .focus_targets
            .retain(|_, node_id| live_nodes.contains(node_id));
        domain.mounts.retain(|_, candidate| {
            retained_keys
                .iter()
                .any(|key| key.is_within(&candidate.occurrence_key))
        });
        drop(domain);
        self.domain.borrow_mut().manager.drop_focus_if_ineligible();
    }

    /// Rebuild exactly the mounted occurrence that published the structural
    /// callback. The mount retains its parent scope and root-space transform,
    /// so a nested/list mutation cannot disturb siblings in the shared domain.
    pub(crate) fn rebuild_after_structure_change(&self, artboard: &ArtboardInstance) {
        let mount = self
            .domain
            .borrow()
            .mounts
            .get(&artboard.instance_identity())
            .cloned();
        if let Some(mount) = mount {
            self.rebuild_mounted_subtree(artboard, mount);
        }
    }

    /// Direct `Artboard::cleanupFocusTree` ownership: remove this mounted
    /// occurrence from its current manager before selecting another manager.
    pub(crate) fn cleanup_focus_tree(&self, artboard: &ArtboardInstance) {
        debug_assert_eq!(self.owner_identity, artboard.instance_identity());
        self.cleanup_owner_occurrence();
    }

    pub(crate) fn cleanup_owner_occurrence(&self) {
        let Some(mount) = self
            .domain
            .borrow()
            .mounts
            .get(&self.owner_identity)
            .cloned()
        else {
            return;
        };
        let keys = self
            .domain
            .borrow()
            .retained_nodes
            .keys()
            .filter(|key| key.is_within(&mount.occurrence_key))
            .cloned()
            .collect::<BTreeSet<_>>();
        let roots = keys
            .iter()
            .filter(|key| {
                self.domain
                    .borrow()
                    .retained_parents
                    .get(*key)
                    .and_then(Option::as_ref)
                    .is_none_or(|parent| !keys.contains(parent))
            })
            .filter_map(|key| self.domain.borrow().retained_nodes.get(key).copied())
            .collect::<Vec<_>>();
        for node in roots {
            self.domain.borrow_mut().manager.remove_subtree(node);
        }
        let mut domain = self.domain.borrow_mut();
        domain.retained_nodes.retain(|key, _| !keys.contains(key));
        domain.retained_parents.retain(|key, _| !keys.contains(key));
        let live_nodes = domain
            .manager
            .nodes
            .keys()
            .copied()
            .collect::<BTreeSet<_>>();
        domain
            .focus_nodes
            .retain(|_, node| live_nodes.contains(node));
        domain
            .focus_targets
            .retain(|_, node| live_nodes.contains(node));
        domain
            .mounts
            .retain(|_, candidate| !candidate.occurrence_key.is_within(&mount.occurrence_key));
    }

    /// Rebuild this private domain from the owner's live subtree in another
    /// retained manager. C++ can rebuild through its stored Artboard pointer;
    /// the public Rust manager-switch API deliberately carries no Artboard
    /// borrow, so clone the already-retained occurrence at that rare switch
    /// boundary instead. This never runs from the per-frame focus query.
    pub(crate) fn replace_with_owner_occurrence_from(&mut self, source: &Self) -> bool {
        if self.shares_manager(source) || self.owner_identity != source.owner_identity {
            return false;
        }
        let source_domain = source.domain.borrow();
        let Some(root_mount) = source_domain.mounts.get(&source.owner_identity).cloned() else {
            return false;
        };
        let keys = source_domain
            .retained_nodes
            .keys()
            .filter(|key| key.is_within(&root_mount.occurrence_key))
            .cloned()
            .collect::<BTreeSet<_>>();
        let old_to_key = keys
            .iter()
            .filter_map(|key| {
                source_domain
                    .retained_nodes
                    .get(key)
                    .copied()
                    .map(|node_id| (node_id, key.clone()))
            })
            .collect::<BTreeMap<_, _>>();
        let mut roots = old_to_key
            .keys()
            .copied()
            .filter(|node_id| {
                source_domain
                    .manager
                    .parent(*node_id)
                    .is_none_or(|parent| !old_to_key.contains_key(&parent))
            })
            .collect::<Vec<_>>();
        roots.sort_by_key(|node_id| {
            source_domain
                .manager
                .parent(*node_id)
                .and_then(|parent| source_domain.manager.children(parent))
                .unwrap_or(source_domain.manager.roots())
                .iter()
                .position(|candidate| candidate == node_id)
                .unwrap_or(usize::MAX)
        });

        let mut rebuilt = RuntimeFocusDomain::default();
        let mut old_to_new = BTreeMap::new();
        let mut new_to_key = BTreeMap::new();
        let mut pending = roots
            .iter()
            .enumerate()
            .rev()
            .map(|(index, node_id)| (*node_id, None, index))
            .collect::<Vec<_>>();
        while let Some((old_id, new_parent, index)) = pending.pop() {
            let Some(mut node) = source_domain.manager.node(old_id).cloned() else {
                continue;
            };
            node.parent = None;
            node.children.clear();
            node.has_focus = false;
            let new_id = rebuilt.create_node(node);
            rebuilt.manager.insert_child(new_parent, new_id, index);
            old_to_new.insert(old_id, new_id);

            let Some(key) = old_to_key.get(&old_id).cloned() else {
                continue;
            };
            let parent_key = new_parent.and_then(|parent| new_to_key.get(&parent).cloned());
            rebuilt.retained_nodes.insert(key.clone(), new_id);
            rebuilt.retained_parents.insert(key.clone(), parent_key);
            new_to_key.insert(new_id, key);

            let children = source_domain
                .manager
                .children(old_id)
                .unwrap_or_default()
                .iter()
                .copied()
                .filter(|child| old_to_key.contains_key(child))
                .collect::<Vec<_>>();
            pending.extend(
                children
                    .iter()
                    .enumerate()
                    .rev()
                    .map(|(index, child)| (*child, Some(new_id), index)),
            );
        }

        for (identity, old_id) in &source_domain.focus_nodes {
            if let Some(new_id) = old_to_new.get(old_id) {
                rebuilt.focus_nodes.insert(*identity, *new_id);
            }
        }
        for (identity, old_id) in &source_domain.focus_targets {
            if let Some(new_id) = old_to_new.get(old_id) {
                rebuilt.focus_targets.insert(*identity, *new_id);
            }
        }
        for (owner, mount) in &source_domain.mounts {
            if mount.occurrence_key.is_within(&root_mount.occurrence_key) {
                let mut mount = mount.clone();
                if mount
                    .parent_focus
                    .as_ref()
                    .is_some_and(|parent| !keys.contains(parent))
                {
                    mount.parent_focus = None;
                    mount.sibling_index = 0;
                }
                rebuilt.mounts.insert(*owner, mount);
            }
        }
        drop(source_domain);
        self.inert = rebuilt.retained_nodes.is_empty();
        self.domain = Rc::new(RefCell::new(rebuilt));
        true
    }

    fn place_retained_node(
        &self,
        key: RuntimeFocusOccurrenceKey,
        parent_key: Option<RuntimeFocusOccurrenceKey>,
        node: FocusNode,
        active: &mut BTreeSet<RuntimeFocusOccurrenceKey>,
        sibling_counts: &mut BTreeMap<Option<RuntimeFocusOccurrenceKey>, usize>,
    ) -> FocusNodeId {
        active.insert(key.clone());
        let retained_node = self.domain.borrow().retained_nodes.get(&key).copied();
        let node_id = match retained_node {
            Some(node_id) => {
                self.domain.borrow_mut().manager.update_node(node_id, &node);
                replace_focusable(&mut self.domain.borrow_mut(), node_id, node.focusable());
                node_id
            }
            None => {
                let node_id = self.domain.borrow_mut().create_node(node);
                self.domain
                    .borrow_mut()
                    .retained_nodes
                    .insert(key.clone(), node_id);
                let focusable = self.domain.borrow().manager.focusable(node_id);
                replace_focusable(&mut self.domain.borrow_mut(), node_id, focusable);
                node_id
            }
        };
        let parent = parent_key
            .as_ref()
            .and_then(|parent| self.domain.borrow().retained_nodes.get(parent).copied());
        let sibling_index = sibling_counts.entry(parent_key.clone()).or_insert(0);
        self.domain
            .borrow_mut()
            .manager
            .insert_child(parent, node_id, *sibling_index);
        *sibling_index += 1;
        self.domain
            .borrow_mut()
            .retained_parents
            .insert(key, parent_key);
        node_id
    }

    pub(crate) fn has_focusable_content(&self) -> bool {
        self.domain.borrow().manager.has_focusable_content()
    }

    pub(crate) fn set_focus_target_before_topology(
        &mut self,
        artboard: &ArtboardInstance,
        target_local: usize,
        focus_data_local: usize,
    ) -> bool {
        self.ensure_unattached_target(artboard, target_local, focus_data_local);
        self.set_focus_target(target_local)
    }

    pub(crate) fn set_focus_target(&mut self, target_local: usize) -> bool {
        let mut domain = self.domain.borrow_mut();
        domain
            .focus_targets
            .get(&(self.owner_identity, target_local))
            .copied()
            .is_some_and(|node_id| domain.manager.set_focus(node_id))
    }

    /// Mirror the constructor-time `FocusData::focusNode()` path without
    /// attaching any other authored node or descendant. The later full build
    /// reuses this exact occurrence identity and places it into the completed
    /// tree.
    fn ensure_unattached_target(
        &mut self,
        artboard: &ArtboardInstance,
        target_local: usize,
        focus_data_local: usize,
    ) {
        let target = (self.owner_identity, target_local);
        let focusable = RuntimeFocusable::new(target.0, target.1, focus_data_local);
        if self
            .domain
            .borrow()
            .focus_nodes
            .contains_key(&(target.0, focus_data_local))
        {
            return;
        }

        let root_key =
            RuntimeFocusOccurrenceKey::root(artboard.graph_global_id, artboard.instance_identity());
        let focus_global_id = artboard.runtime_graph().and_then(|graph| {
            graph
                .components
                .iter()
                .find(|component| component.local_id == focus_data_local)
                .map(|component| component.global_id)
        });
        let Some(focus_global_id) = focus_global_id else {
            return;
        };
        let key = root_key.child(
            FOCUS_KEY_AUTHORED,
            focus_data_local as u64,
            u64::from(focus_global_id),
        );
        let retained_node = self.domain.borrow().retained_nodes.get(&key).copied();
        let node_id = match retained_node {
            Some(node_id) => node_id,
            None => {
                let node_id = self.domain.borrow_mut().create_node(authored_focus_node(
                    artboard,
                    focus_data_local,
                    true,
                    Mat2D::IDENTITY,
                ));
                self.domain
                    .borrow_mut()
                    .retained_nodes
                    .insert(key.clone(), node_id);
                replace_focusable(&mut self.domain.borrow_mut(), node_id, Some(focusable));
                node_id
            }
        };
        self.domain.borrow_mut().retained_parents.insert(key, None);
        debug_assert_eq!(
            self.domain.borrow().manager.focusable(node_id),
            Some(focusable)
        );
    }

    pub(crate) fn clear_focus(&mut self) -> bool {
        self.domain.borrow_mut().manager.clear_focus()
    }

    pub(crate) fn traverse(&mut self, traversal_kind: u64) -> bool {
        let mut domain = self.domain.borrow_mut();
        match traversal_kind {
            0 => domain.manager.focus_next(),
            1 => domain.manager.focus_previous(),
            2 => domain.manager.focus_up(),
            3 => domain.manager.focus_down(),
            4 => domain.manager.focus_left(),
            5 => domain.manager.focus_right(),
            _ => domain.manager.focus_next(),
        }
    }

    pub(crate) fn refresh_after_property_change(
        &self,
        artboard: &ArtboardInstance,
        local_id: usize,
        property_key: u16,
    ) {
        let Some(component) = artboard.component(local_id) else {
            return;
        };
        let type_name = component.type_name;
        if type_name == "FocusData" {
            let root_transform = self
                .domain
                .borrow()
                .mounts
                .get(&artboard.instance_identity())
                .map_or(Mat2D::IDENTITY, |mount| mount.root_transform);
            self.refresh_focus_data_update(artboard, local_id, root_transform);
            return;
        }
        let affects_visibility = property_key_for_name(type_name, "opacity") == Some(property_key)
            || property_key_for_name(type_name, "drawableFlags") == Some(property_key)
            || (matches!(
                type_name,
                "NestedArtboard" | "NestedArtboardLayout" | "NestedArtboardLeaf"
            ) && property_key_for_name("NestedArtboard", "isPaused") == Some(property_key));
        if affects_visibility {
            self.refresh_visibility_change(artboard);
        }
    }

    pub(crate) fn refresh_visibility_change(&self, artboard: &ArtboardInstance) {
        let mount = self
            .domain
            .borrow()
            .mounts
            .get(&artboard.instance_identity())
            .cloned();
        if let Some(mount) = mount {
            refresh_artboard_focusables(
                self,
                artboard,
                mount.inherited_eligible,
                mount.root_transform,
            );
        }
    }

    pub(crate) fn refresh_focus_data_update(
        &self,
        artboard: &ArtboardInstance,
        focus_data_local: usize,
        root_transform: Mat2D,
    ) {
        let mount = self
            .domain
            .borrow()
            .mounts
            .get(&artboard.instance_identity())
            .cloned();
        let Some(mount) = mount else {
            return;
        };
        refresh_focus_data_node(
            self,
            artboard,
            focus_data_local,
            mount.inherited_eligible,
            root_transform,
        );
    }

    pub(crate) fn drop_hidden_focus_target(&self) -> bool {
        self.domain.borrow_mut().manager.drop_focus_if_ineligible()
    }

    pub(crate) fn set_accepts_keyboard_input(
        &self,
        focus_data_local: usize,
        accepts_keyboard_input: bool,
    ) {
        let mut domain = self.domain.borrow_mut();
        let node_id = domain
            .focus_nodes
            .get(&(self.owner_identity, focus_data_local))
            .copied();
        let Some(node_id) = node_id else {
            return;
        };
        let Some(mut focusable) = domain.manager.focusable(node_id) else {
            return;
        };
        focusable.accepts_keyboard_input = accepts_keyboard_input;
        domain.manager.set_node_focusable(node_id, Some(focusable));
    }

    pub(crate) fn clear_keyboard_input_capabilities(&self) {
        let mut domain = self.domain.borrow_mut();
        let node_ids = domain
            .focus_nodes
            .iter()
            .filter_map(|((owner, _), node_id)| (*owner == self.owner_identity).then_some(*node_id))
            .collect::<Vec<_>>();
        for node_id in node_ids {
            let Some(mut focusable) = domain.manager.focusable(node_id) else {
                continue;
            };
            if focusable.accepts_keyboard_input {
                focusable.accepts_keyboard_input = false;
                domain.manager.set_node_focusable(node_id, Some(focusable));
            }
        }
    }

    pub(crate) fn primary_accepts_keyboard_input(&self) -> bool {
        let domain = self.domain.borrow();
        domain
            .manager
            .primary_focus()
            .and_then(|node_id| domain.manager.focusable(node_id))
            .is_some_and(|focusable| focusable.accepts_keyboard_input)
    }

    pub(crate) fn target_has_focus(&self, target_local: usize) -> bool {
        let domain = self.domain.borrow();
        domain
            .focus_targets
            .get(&(self.owner_identity, target_local))
            .copied()
            .is_some_and(|node_id| domain.manager.has_focus(node_id))
    }

    /// Owner-safe existence query for StateMachineInstance::setFocus.
    ///
    /// This distinguishes a retained FocusData occurrence from the C++
    /// null-FocusData/null-node branch without exposing manager internals.
    pub(crate) fn has_focus_target(&self, target_local: usize) -> bool {
        self.domain
            .borrow()
            .focus_targets
            .contains_key(&(self.owner_identity, target_local))
    }

    /// Cheap owner-safe query used by host FocusState polling.
    pub(crate) fn has_primary_focus(&self) -> bool {
        self.domain.borrow().manager.primary_focus().is_some()
    }

    /// Return focused listener targets from the primary leaf toward the root.
    ///
    /// C++ `FocusManager::{keyInput,textInput,gamepadDispatch}` bubbles along
    /// this exact node chain and only then applies registration order within
    /// each `FocusData` (`focus_manager.cpp:702-751`).
    pub(crate) fn focused_listener_chain(&self) -> Vec<(u64, usize, usize)> {
        let domain = self.domain.borrow();
        let Some(primary) = domain.manager.primary_focus() else {
            return Vec::new();
        };
        domain
            .manager
            .ancestor_chain(primary)
            .into_iter()
            .filter_map(|node_id| {
                let focusable = domain.manager.focusable(node_id)?;
                Some((
                    focusable.owner_identity,
                    focusable.target_local,
                    focusable.focus_data_local,
                ))
            })
            .collect()
    }

    /// Drain this occurrence's focus callbacks as authored target ids.
    ///
    /// `FocusManager` owns the concrete node identities while listener groups
    /// retain `FocusData` occurrences. Pinned C++ delivers the manager
    /// callback to those groups, which enqueue an occurrence-owned record on
    /// the state machine. Translating back through the retained target table
    /// preserves that ownership without rediscovering the artboard graph.
    pub(crate) fn take_owner_events(&mut self) -> Vec<(usize, usize, FocusEventKind)> {
        let mut domain = self.domain.borrow_mut();
        let events = std::mem::take(&mut domain.manager.pending_events);
        let mut owner_events = Vec::new();
        for event in events {
            let Some(focusable) = domain.manager.focusable(event.node_id) else {
                // Structural scopes have no FocusData callback in C++.
                continue;
            };
            if focusable.owner_identity != self.owner_identity {
                domain.manager.pending_events.push(event);
                continue;
            }
            owner_events.push((
                focusable.target_local,
                focusable.focus_data_local,
                event.kind,
            ));
        }
        owner_events
    }

    /// Drop focus notifications produced before listener groups exist.
    ///
    /// Pinned C++ initializes every layer (including entry focus actions)
    /// before it constructs `FocusListenerGroup` occurrences. Those earlier
    /// callbacks therefore have no registered recipient and are not replayed
    /// after registration (`state_machine_instance.cpp:1747-1752,1829-1891`).
    pub(crate) fn discard_unregistered_events(&mut self) {
        self.domain.borrow_mut().manager.pending_events.clear();
    }
}

fn build_artboard_focus_tree(
    tree: &RuntimeFocusTree,
    artboard: &ArtboardInstance,
    occurrence_key: &RuntimeFocusOccurrenceKey,
    parent_focus: Option<RuntimeFocusOccurrenceKey>,
    inherited_eligible: bool,
    root_transform: Mat2D,
    active: &mut BTreeSet<RuntimeFocusOccurrenceKey>,
    sibling_counts: &mut BTreeMap<Option<RuntimeFocusOccurrenceKey>, usize>,
) {
    let sibling_index = sibling_counts.get(&parent_focus).copied().unwrap_or(0);
    tree.domain.borrow_mut().mounts.insert(
        artboard.instance_identity(),
        RuntimeFocusMount {
            occurrence_key: occurrence_key.clone(),
            parent_focus: parent_focus.clone(),
            inherited_eligible,
            root_transform,
            sibling_index,
        },
    );
    let Some(graph) = artboard.runtime_graph() else {
        return;
    };
    let Some(root_local) = graph
        .components
        .iter()
        .find(|component| component.type_name == "Artboard" && component.parent_local.is_none())
        .map(|component| component.local_id)
    else {
        return;
    };
    build_component_focus_tree(
        tree,
        artboard,
        root_local,
        occurrence_key,
        parent_focus,
        inherited_eligible,
        root_transform,
        active,
        sibling_counts,
    );
}

#[allow(clippy::too_many_arguments)]
fn build_component_focus_tree(
    tree: &RuntimeFocusTree,
    artboard: &ArtboardInstance,
    local_id: usize,
    occurrence_key: &RuntimeFocusOccurrenceKey,
    parent_focus: Option<RuntimeFocusOccurrenceKey>,
    inherited_eligible: bool,
    root_transform: Mat2D,
    active: &mut BTreeSet<RuntimeFocusOccurrenceKey>,
    sibling_counts: &mut BTreeMap<Option<RuntimeFocusOccurrenceKey>, usize>,
) {
    let Some(graph) = artboard.runtime_graph() else {
        return;
    };
    let Some(component) = graph
        .components
        .iter()
        .find(|component| component.local_id == local_id)
    else {
        return;
    };

    let mut host_parent = parent_focus.clone();
    if matches!(
        component.type_name,
        "NestedArtboard" | "NestedArtboardLayout" | "NestedArtboardLeaf"
    ) {
        let artboard_id_key = property_key_for_name("NestedArtboard", "artboardId");
        let data_bound = artboard_id_key.is_some_and(|property_key| {
            graph.data_binds.iter().any(|data_bind| {
                data_bind.target_local == Some(local_id)
                    && data_bind.property_key == u64::from(property_key)
            })
        });
        if data_bound {
            let scope_key = occurrence_key.child(
                FOCUS_KEY_NESTED,
                local_id as u64,
                u64::from(component.global_id),
            );
            tree.place_retained_node(
                scope_key.clone(),
                parent_focus.clone(),
                FocusNode::structural_scope(),
                active,
                sibling_counts,
            );
            host_parent = Some(scope_key);
        }
        if let Some(nested) = artboard.nested_artboards.get(&local_id) {
            let child_key = occurrence_key.child(
                FOCUS_KEY_NESTED_CHILD,
                local_id as u64,
                nested.child.instance_identity(),
            );
            build_artboard_focus_tree(
                tree,
                &nested.child,
                &child_key,
                host_parent.clone(),
                inherited_eligible
                    && component_and_ancestors_allow_focus(artboard, local_id)
                    && !nested_host_is_paused(artboard, local_id),
                root_transform.multiply(
                    artboard
                        .component(local_id)
                        .map_or(Mat2D::IDENTITY, |host| host.transform.world_transform),
                ),
                active,
                sibling_counts,
            );
        }
    } else if component.type_name == "ArtboardComponentList" {
        let scope_key = occurrence_key.child(
            FOCUS_KEY_LIST_SCOPE,
            local_id as u64,
            u64::from(component.global_id),
        );
        tree.place_retained_node(
            scope_key.clone(),
            parent_focus.clone(),
            FocusNode::structural_scope(),
            active,
            sibling_counts,
        );
        if let Some(items) = artboard.component_list_items(local_id) {
            let host_transform_local =
                if crate::constraints::scrolling::scroll_virtualizer::component_list_virtualization(
                    artboard, local_id,
                )
                .is_some()
                {
                    component.parent_local.unwrap_or(local_id)
                } else {
                    local_id
                };
            let host_world = artboard
                .component(host_transform_local)
                .map_or(Mat2D::IDENTITY, |host| host.transform.world_transform);
            let item_transforms = artboard
                .component_list_state(local_id)
                .map(|list| &list.item_transforms);
            for (item_index, item) in items.iter().enumerate() {
                let row_key = occurrence_key.child(
                    FOCUS_KEY_LIST_ROW,
                    local_id as u64,
                    item.occurrence_identity,
                );
                tree.place_retained_node(
                    row_key.clone(),
                    Some(scope_key.clone()),
                    FocusNode::structural_scope(),
                    active,
                    sibling_counts,
                );
                let child_key = row_key.child(
                    FOCUS_KEY_ROOT,
                    u64::from(item.child.graph_global_id),
                    item.child.instance_identity(),
                );
                build_artboard_focus_tree(
                    tree,
                    &item.child,
                    &child_key,
                    Some(row_key),
                    inherited_eligible && component_and_ancestors_allow_focus(artboard, local_id),
                    root_transform.multiply(host_world).multiply(
                        item_transforms
                            .and_then(|transforms| transforms.get(item_index))
                            .copied()
                            .unwrap_or(item.transform),
                    ),
                    active,
                    sibling_counts,
                );
            }
        }
    }

    let direct_focus = component.children.iter().copied().find(|child_local| {
        graph
            .components
            .iter()
            .find(|child| child.local_id == *child_local)
            .is_some_and(|child| child.type_name == "FocusData")
    });
    let recurse_parent = if let Some(focus_local) = direct_focus {
        let focus_key = occurrence_key.child(
            FOCUS_KEY_AUTHORED,
            focus_local as u64,
            graph
                .components
                .iter()
                .find(|child| child.local_id == focus_local)
                .map_or(0, |child| u64::from(child.global_id)),
        );
        tree.place_retained_node(
            focus_key.clone(),
            parent_focus,
            authored_focus_node(artboard, focus_local, inherited_eligible, root_transform),
            active,
            sibling_counts,
        );
        Some(focus_key)
    } else {
        parent_focus
    };

    for child_local in &component.children {
        let is_focus_data = graph
            .components
            .iter()
            .find(|child| child.local_id == *child_local)
            .is_some_and(|child| child.type_name == "FocusData");
        if !is_focus_data {
            build_component_focus_tree(
                tree,
                artboard,
                *child_local,
                occurrence_key,
                recurse_parent.clone(),
                inherited_eligible,
                root_transform,
                active,
                sibling_counts,
            );
        }
    }
}

fn refresh_artboard_focusables(
    tree: &RuntimeFocusTree,
    artboard: &ArtboardInstance,
    inherited_eligible: bool,
    root_transform: Mat2D,
) {
    if let Some(mount) = tree
        .domain
        .borrow_mut()
        .mounts
        .get_mut(&artboard.instance_identity())
    {
        mount.inherited_eligible = inherited_eligible;
        mount.root_transform = root_transform;
    }
    let Some(graph) = artboard.runtime_graph() else {
        return;
    };

    for focus_data in graph
        .components
        .iter()
        .filter(|component| component.type_name == "FocusData")
    {
        refresh_focus_data_node(
            tree,
            artboard,
            focus_data.local_id,
            inherited_eligible,
            root_transform,
        );
    }

    for &host_local in &artboard.nested_artboard_locals {
        let Some(nested) = artboard.nested_artboards.get(&host_local) else {
            continue;
        };
        let host_world = artboard
            .component(host_local)
            .map_or(Mat2D::IDENTITY, |host| host.transform.world_transform);
        refresh_artboard_focusables(
            tree,
            &nested.child,
            inherited_eligible
                && component_and_ancestors_allow_focus(artboard, host_local)
                && !nested_host_is_paused(artboard, host_local),
            root_transform.multiply(host_world),
        );
    }

    for component in graph
        .components
        .iter()
        .filter(|component| component.type_name == "ArtboardComponentList")
    {
        let Some(items) = artboard.component_list_items(component.local_id) else {
            continue;
        };
        let host_transform_local =
            if crate::constraints::scrolling::scroll_virtualizer::component_list_virtualization(
                artboard,
                component.local_id,
            )
            .is_some()
            {
                component.parent_local.unwrap_or(component.local_id)
            } else {
                component.local_id
            };
        let host_world = artboard
            .component(host_transform_local)
            .map_or(Mat2D::IDENTITY, |host| host.transform.world_transform);
        let item_transforms = artboard
            .component_list_state(component.local_id)
            .map(|list| &list.item_transforms);
        for (item_index, item) in items.iter().enumerate() {
            refresh_artboard_focusables(
                tree,
                &item.child,
                inherited_eligible
                    && component_and_ancestors_allow_focus(artboard, component.local_id),
                root_transform.multiply(host_world).multiply(
                    item_transforms
                        .and_then(|transforms| transforms.get(item_index))
                        .copied()
                        .unwrap_or(item.transform),
                ),
            );
        }
    }
}

fn refresh_focus_data_node(
    tree: &RuntimeFocusTree,
    artboard: &ArtboardInstance,
    focus_data_local: usize,
    inherited_eligible: bool,
    root_transform: Mat2D,
) {
    let refreshed = authored_focus_node(
        artboard,
        focus_data_local,
        inherited_eligible,
        root_transform,
    );
    let Some(focusable) = refreshed.focusable() else {
        return;
    };
    let mut domain = tree.domain.borrow_mut();
    let Some(node_id) = domain
        .focus_nodes
        .get(&(focusable.owner_identity, focusable.focus_data_local))
        .copied()
    else {
        return;
    };
    domain.manager.update_node(node_id, &refreshed);
    replace_focusable(&mut domain, node_id, Some(focusable));
}

fn authored_focus_node(
    artboard: &ArtboardInstance,
    focus_local: usize,
    inherited_eligible: bool,
    root_transform: Mat2D,
) -> FocusNode {
    let mut node = FocusNode::new();
    // C++ FocusData::onAddedDirty wires the authored FocusData through a
    // Focusable into its FocusNode. A bare FocusNode starts with nullptr, but
    // a node constructed from authored FocusData is therefore backed.
    let target_local = artboard
        .component_parent_local(focus_local)
        .unwrap_or(focus_local);
    // FocusData itself implements Focusable; `Focusable::from(Core*)` is a
    // separate conversion seam for TextInput/NestedArtboard callers.
    node.set_focusable(RuntimeFocusable::new(
        artboard.instance_identity(),
        target_local,
        focus_local,
    ));
    let focus_flags = property_key_for_name("FocusData", "focusFlags")
        .and_then(|property_key| artboard.objects.uint_property(focus_local, property_key))
        .unwrap_or(7);
    node.set_can_focus(focus_flags & 1 != 0);
    node.set_can_touch(focus_flags & 2 != 0);
    node.set_can_traverse(focus_flags & 4 != 0);
    let edge_behavior = property_key_for_name("FocusData", "edgeBehaviorValue")
        .and_then(|property_key| artboard.objects.uint_property(focus_local, property_key))
        .unwrap_or(0);
    node.set_edge_behavior(match edge_behavior {
        1 => FocusEdgeBehavior::ClosedLoop,
        2 => FocusEdgeBehavior::Stop,
        _ => FocusEdgeBehavior::ParentScope,
    });
    if let Some(name) = property_key_for_name("FocusData", "name")
        .and_then(|property_key| artboard.objects.string_property(focus_local, property_key))
    {
        node.set_name(name.to_vec());
    }

    let eligible = inherited_eligible
        && artboard
            .component(focus_local)
            .is_none_or(|focus_data| !focus_data.is_collapsed());
    let parent_local = artboard.component_parent_local(focus_local);
    let eligible = eligible
        && parent_local
            .is_none_or(|parent_local| component_and_ancestors_allow_focus(artboard, parent_local));
    node.set_eligible(eligible);
    if let Some(parent) = parent_local.and_then(|parent_local| artboard.component(parent_local)) {
        let parent_to_root = root_transform.multiply(parent.transform.world_transform);
        let (x, y) = parent_to_root.transform_point(0.0, 0.0);
        node.set_position(Some(FocusPoint::new(x, y)));
        if let Some((min_x, min_y, max_x, max_y)) =
            parent_local.and_then(|local| artboard.layout_world_bounds(local))
        {
            let (min_x, min_y) = root_transform.transform_point(min_x, min_y);
            let (max_x, max_y) = root_transform.transform_point(max_x, max_y);
            node.set_bounds(Some(FocusBounds::new(min_x, min_y, max_x, max_y)));
        }
    }
    node
}

fn component_and_ancestors_allow_focus(artboard: &ArtboardInstance, start_local: usize) -> bool {
    let drawable_flags_key = property_key_for_name("Drawable", "drawableFlags");
    let allows_focus = |artboard: &ArtboardInstance,
                        component: crate::components::ComponentHandle| {
        let Some(local_id) = artboard.component_local_id(component) else {
            return true;
        };
        let component = artboard.component_at(component);
        let is_hidden = drawable_flags_key
            .and_then(|property_key| artboard.objects.uint_property(local_id, property_key))
            .is_some_and(|flags| flags & 1 != 0);
        !(component.is_collapsed()
            || is_hidden
            || (component.capabilities.transform && component.transform.render_opacity <= 0.0))
    };
    let Some(start) = artboard.component_handle(start_local) else {
        return true;
    };
    if !allows_focus(artboard, start) {
        return false;
    }

    let frames = [ParentTraversalFrame {
        artboard,
        host_component_in_parent: None,
    }];
    let mut traversal = ParentTraversal::new(&frames, start);
    while let Some(parent) = traversal.next() {
        if !allows_focus(parent.artboard, parent.component) {
            return false;
        }
    }
    true
}

fn nested_host_is_paused(artboard: &ArtboardInstance, local_id: usize) -> bool {
    property_key_for_name("NestedArtboard", "isPaused")
        .and_then(|property_key| artboard.objects.bool_property(local_id, property_key))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_focus_node_defaults_and_property_setters() {
        let mut node = FocusNode::new();
        assert!(node.can_focus());
        assert!(node.can_touch());
        assert!(node.can_traverse());
        assert_eq!(node.tab_index(), 0);
        assert_eq!(node.name(), b"");
        assert_eq!(node.edge_behavior(), FocusEdgeBehavior::ParentScope);
        assert!(node.parent.is_none());
        assert!(node.children.is_empty());
        assert!(!node.has_focus());

        node.set_can_focus(false);
        assert!(!node.can_focus());
        node.set_can_touch(false);
        assert!(!node.can_touch());
        node.set_can_traverse(false);
        assert!(!node.can_traverse());
        node.set_tab_index(42);
        assert_eq!(node.tab_index(), 42);
        node.set_name(b"button".to_vec());
        assert_eq!(node.name(), b"button");
        node.set_edge_behavior(FocusEdgeBehavior::ClosedLoop);
        assert_eq!(node.edge_behavior(), FocusEdgeBehavior::ClosedLoop);
        node.set_edge_behavior(FocusEdgeBehavior::Stop);
        assert_eq!(node.edge_behavior(), FocusEdgeBehavior::Stop);
    }

    #[test]
    fn upstream_focus_node_fresh_focusable_defaults_to_null() {
        let node = FocusNode::new();
        assert!(
            node.focusable().is_none(),
            "focus_test.cpp:88 expects a fresh FocusNode::focusable() to be null"
        );
    }

    #[test]
    fn upstream_focus_node_retains_and_replaces_focusable_identity() {
        let first = RuntimeFocusable::new(7, 11, 12);
        let replacement = RuntimeFocusable::new(7, 21, 22);
        let mut node = FocusNode::new();

        node.set_focusable(first);
        assert_eq!(node.focusable(), Some(first));
        node.set_focusable(replacement);
        assert_eq!(node.focusable(), Some(replacement));
        node.clear_focusable();
        assert_eq!(node.focusable(), None);
    }

    #[test]
    fn focusing_child_notifies_leaf_and_ancestors() {
        let mut manager = FocusManager::new();
        let parent = manager.create_node(FocusNode::new());
        let child = manager.create_node(FocusNode::new());

        assert!(manager.add_child(None, parent));
        assert!(manager.add_child(Some(parent), child));
        assert!(manager.set_focus(child));

        assert_eq!(manager.primary_focus(), Some(child));
        assert!(!manager.has_primary_focus(parent));
        assert!(manager.has_primary_focus(child));
        assert!(manager.has_focus(child));
        assert!(manager.has_focus(parent));
        assert_eq!(
            manager.take_events(),
            vec![
                FocusEvent::new(child, FocusEventKind::Focused),
                FocusEvent::new(parent, FocusEventKind::Focused),
            ]
        );
    }

    #[test]
    fn clearing_focus_blurs_leaf_and_ancestors() {
        let mut manager = FocusManager::new();
        let parent = manager.create_node(FocusNode::new());
        let child = manager.create_node(FocusNode::new());
        manager.add_child(None, parent);
        manager.add_child(Some(parent), child);
        manager.set_focus(child);
        manager.take_events();

        assert!(manager.clear_focus());

        assert_eq!(manager.primary_focus(), None);
        assert!(!manager.has_focus(child));
        assert!(!manager.has_focus(parent));
        assert_eq!(
            manager.take_events(),
            vec![
                FocusEvent::new(child, FocusEventKind::Blurred),
                FocusEvent::new(parent, FocusEventKind::Blurred),
            ]
        );
    }

    #[test]
    fn moving_between_siblings_does_not_renotify_the_common_ancestor() {
        let mut manager = FocusManager::new();
        let parent = manager.create_node(FocusNode::new());
        let first = manager.create_node(FocusNode::new());
        let second = manager.create_node(FocusNode::new());
        manager.add_child(None, parent);
        manager.add_child(Some(parent), first);
        manager.add_child(Some(parent), second);
        manager.set_focus(first);
        manager.take_events();

        assert!(manager.set_focus(second));

        assert_eq!(manager.primary_focus(), Some(second));
        assert!(manager.has_focus(parent));
        assert!(!manager.has_focus(first));
        assert!(manager.has_focus(second));
        assert_eq!(
            manager.take_events(),
            vec![
                FocusEvent::new(first, FocusEventKind::Blurred),
                FocusEvent::new(second, FocusEventKind::Focused),
            ]
        );
    }

    #[test]
    fn inserting_an_existing_subtree_reorders_without_blurring() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::new());
        let first = manager.create_node(FocusNode::new());
        let second = manager.create_node(FocusNode::new());
        manager.add_child(None, scope);
        manager.add_child(Some(scope), first);
        manager.add_child(Some(scope), second);
        manager.set_focus(second);
        manager.take_events();

        assert!(manager.insert_child(Some(scope), second, 0));

        assert_eq!(manager.children(scope), Some(&[second, first][..]));
        assert_eq!(manager.primary_focus(), Some(second));
        assert!(manager.take_events().is_empty());
    }

    #[test]
    fn inserting_an_ancestor_below_its_descendant_is_rejected_without_mutation() {
        let mut manager = FocusManager::new();
        let root = manager.create_node(FocusNode::new());
        let middle = manager.create_node(FocusNode::new());
        let leaf = manager.create_node(FocusNode::new());
        manager.add_child(None, root);
        manager.add_child(Some(root), middle);
        manager.add_child(Some(middle), leaf);

        assert!(!manager.insert_child(Some(leaf), root, 0));

        assert_eq!(manager.roots(), &[root]);
        assert_eq!(manager.parent(root), None);
        assert_eq!(manager.parent(middle), Some(root));
        assert_eq!(manager.parent(leaf), Some(middle));
    }

    #[test]
    fn detaching_a_focused_subtree_preserves_focus_for_reattachment() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::new());
        let row = manager.create_node(FocusNode::new());
        let leaf = manager.create_node(FocusNode::new());
        manager.add_child(None, scope);
        manager.add_child(Some(scope), row);
        manager.add_child(Some(row), leaf);
        manager.set_focus(leaf);
        manager.take_events();

        assert!(manager.detach_subtree(row));
        assert!(!manager.is_attached(row));
        assert_eq!(manager.primary_focus(), Some(leaf));
        assert!(manager.take_events().is_empty());

        assert!(manager.insert_child(Some(scope), row, 0));
        assert!(manager.is_attached(row));
        assert_eq!(manager.primary_focus(), Some(leaf));
        assert!(manager.take_events().is_empty());
    }

    #[test]
    fn removing_a_focused_subtree_blurs_and_invalidates_every_node() {
        let mut manager = FocusManager::new();
        let parent = manager.create_node(FocusNode::new());
        let child = manager.create_node(FocusNode::new());
        manager.add_child(None, parent);
        manager.add_child(Some(parent), child);
        manager.set_focus(child);
        manager.take_events();

        assert!(manager.remove_subtree(parent));

        assert_eq!(manager.primary_focus(), None);
        assert!(!manager.contains(parent));
        assert!(!manager.contains(child));
        assert_eq!(
            manager.take_events(),
            vec![
                FocusEvent::new(child, FocusEventKind::Blurred),
                FocusEvent::new(parent, FocusEventKind::Blurred),
            ]
        );
    }

    #[test]
    fn migrating_a_subtree_preserves_ids_after_the_old_manager_is_dropped() {
        let mut parent_manager = FocusManager::new();
        let parent = parent_manager.create_node(FocusNode::new());
        parent_manager.add_child(None, parent);

        let (scope, leaf) = {
            let mut internal_manager = FocusManager::new();
            let scope = internal_manager.create_node(FocusNode::new());
            let leaf = internal_manager.create_node(FocusNode::new());
            internal_manager.add_child(None, scope);
            internal_manager.add_child(Some(scope), leaf);

            assert!(parent_manager.migrate_subtree_from(
                &mut internal_manager,
                scope,
                Some(parent),
                0,
            ));
            assert!(internal_manager.roots().is_empty());
            assert!(!internal_manager.contains(scope));
            (scope, leaf)
        };

        assert!(parent_manager.contains(scope));
        assert!(parent_manager.contains(leaf));
        assert_eq!(parent_manager.parent(scope), Some(parent));
        assert_eq!(parent_manager.children(scope), Some(&[leaf][..]));
    }

    #[test]
    fn migrating_a_focused_subtree_transfers_focus_and_ancestry_events() {
        let mut source = FocusManager::new();
        let scope = source.create_node(FocusNode::new());
        let leaf = source.create_node(FocusNode::new());
        source.add_child(None, scope);
        source.add_child(Some(scope), leaf);
        source.set_focus(leaf);
        source.take_events();

        let mut target = FocusManager::new();
        let parent = target.create_node(FocusNode::new());
        target.add_child(None, parent);

        assert!(target.migrate_subtree_from(&mut source, scope, Some(parent), 0));

        assert_eq!(source.primary_focus(), None);
        assert_eq!(target.primary_focus(), Some(leaf));
        assert_eq!(
            source.take_events(),
            vec![
                FocusEvent::new(leaf, FocusEventKind::Blurred),
                FocusEvent::new(scope, FocusEventKind::Blurred),
            ]
        );
        assert_eq!(
            target.take_events(),
            vec![
                FocusEvent::new(leaf, FocusEventKind::Focused),
                FocusEvent::new(scope, FocusEventKind::Focused),
                FocusEvent::new(parent, FocusEventKind::Focused),
            ]
        );
    }

    #[test]
    fn focusable_content_ignores_empty_structural_scopes_but_counts_authored_nodes() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::structural_scope());
        manager.add_child(None, scope);
        assert!(!manager.has_focusable_content());

        let mut authored = FocusNode::new();
        authored.set_focusable(RuntimeFocusable::new(1, 2, 3));
        authored.set_can_focus(false);
        authored.set_can_traverse(false);
        authored.set_eligible(false);
        let authored = manager.create_node(authored);
        manager.add_child(Some(scope), authored);

        assert!(manager.has_focusable_content());
    }

    #[test]
    fn direct_focus_on_a_scope_resolves_to_its_first_traversable_leaf() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::new());
        let mut later = FocusNode::new();
        later.set_tab_index(1);
        let later = manager.create_node(later);
        let mut first = FocusNode::new();
        first.set_tab_index(-1);
        let first = manager.create_node(first);
        manager.add_child(None, scope);
        manager.add_child(Some(scope), later);
        manager.add_child(Some(scope), first);

        assert!(manager.set_focus(scope));

        assert_eq!(manager.primary_focus(), Some(first));
        assert!(manager.has_focus(scope));
        assert!(manager.has_focus(first));
        assert!(!manager.has_focus(later));
    }

    #[test]
    fn next_and_previous_traversal_follow_stable_tab_order_and_rest_on_leaves() {
        let mut manager = FocusManager::new();
        let scope = manager.create_node(FocusNode::new());
        let mut second = FocusNode::new();
        second.set_tab_index(1);
        let second = manager.create_node(second);
        let mut first = FocusNode::new();
        first.set_tab_index(-1);
        let first = manager.create_node(first);
        let tied = manager.create_node(FocusNode::new());
        manager.add_child(None, scope);
        manager.add_child(Some(scope), second);
        manager.add_child(Some(scope), first);
        manager.add_child(Some(scope), tied);

        assert!(manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(first));
        assert!(manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(tied));
        assert!(manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(second));
        assert!(manager.focus_previous());
        assert_eq!(manager.primary_focus(), Some(tied));
    }

    #[test]
    fn closed_loop_scope_wraps_at_both_edges() {
        let mut manager = FocusManager::new();
        let mut scope = FocusNode::new();
        scope.set_edge_behavior(FocusEdgeBehavior::ClosedLoop);
        let scope = manager.create_node(scope);
        let first = manager.create_node(FocusNode::new());
        let last = manager.create_node(FocusNode::new());
        manager.add_child(None, scope);
        manager.add_child(Some(scope), first);
        manager.add_child(Some(scope), last);
        manager.set_focus(last);

        assert!(manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(first));
        assert!(manager.focus_previous());
        assert_eq!(manager.primary_focus(), Some(last));
    }

    #[test]
    fn root_sequential_edges_clear_focus_like_cpp_find_next_focusable() {
        let mut manager = FocusManager::new();
        let first = manager.create_node(FocusNode::new());
        let last = manager.create_node(FocusNode::new());
        manager.add_child(None, first);
        manager.add_child(None, last);

        manager.set_focus(last);
        assert!(!manager.focus_next());
        assert_eq!(manager.primary_focus(), None);

        manager.set_focus(first);
        assert!(!manager.focus_previous());
        assert_eq!(manager.primary_focus(), None);
    }

    #[test]
    fn stop_scope_does_not_move_past_its_boundary() {
        let mut manager = FocusManager::new();
        let root = manager.create_node(FocusNode::new());
        let mut scope = FocusNode::new();
        scope.set_edge_behavior(FocusEdgeBehavior::Stop);
        let scope = manager.create_node(scope);
        let leaf = manager.create_node(FocusNode::new());
        let after = manager.create_node(FocusNode::new());
        manager.add_child(None, root);
        manager.add_child(Some(root), scope);
        manager.add_child(Some(scope), leaf);
        manager.add_child(Some(root), after);
        manager.set_focus(leaf);

        assert!(!manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(leaf));
    }

    #[test]
    fn parent_scope_edges_continue_with_the_scopes_siblings() {
        let mut manager = FocusManager::new();
        let root = manager.create_node(FocusNode::new());
        let before = manager.create_node(FocusNode::new());
        let scope = manager.create_node(FocusNode::new());
        let inner = manager.create_node(FocusNode::new());
        let after = manager.create_node(FocusNode::new());
        manager.add_child(None, root);
        manager.add_child(Some(root), before);
        manager.add_child(Some(root), scope);
        manager.add_child(Some(scope), inner);
        manager.add_child(Some(root), after);

        manager.set_focus(inner);
        assert!(manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(after));

        manager.set_focus(inner);
        assert!(manager.focus_previous());
        assert_eq!(manager.primary_focus(), Some(before));
    }

    #[test]
    fn only_unbacked_structural_scopes_are_transparent_to_traversal() {
        let mut manager = FocusManager::new();
        let mut authored_scope = FocusNode::new();
        authored_scope.set_focusable(RuntimeFocusable::new(1, 2, 3));
        authored_scope.set_can_focus(false);
        let authored_scope = manager.create_node(authored_scope);
        let blocked_leaf = manager.create_node(FocusNode::new());
        let structural_scope = manager.create_node(FocusNode::structural_scope());
        let reachable_leaf = manager.create_node(FocusNode::new());
        manager.add_child(None, authored_scope);
        manager.add_child(Some(authored_scope), blocked_leaf);
        manager.add_child(None, structural_scope);
        manager.add_child(Some(structural_scope), reachable_leaf);

        assert!(manager.focus_next());
        assert_eq!(manager.primary_focus(), Some(reachable_leaf));
        assert!(!manager.has_focus(blocked_leaf));
        assert!(manager.has_focus(structural_scope));
    }

    #[test]
    fn direct_focus_on_an_ineligible_scope_does_not_reach_its_child() {
        let mut manager = FocusManager::new();
        let mut scope = FocusNode::new();
        scope.set_eligible(false);
        let scope = manager.create_node(scope);
        let child = manager.create_node(FocusNode::new());
        manager.add_child(None, scope);
        manager.add_child(Some(scope), child);

        assert!(!manager.set_focus(scope));
        assert_eq!(manager.primary_focus(), None);
        assert!(manager.take_events().is_empty());
    }

    #[test]
    fn focus_is_dropped_when_the_primary_node_becomes_ineligible() {
        let mut manager = FocusManager::new();
        let node = manager.create_node(FocusNode::new());
        manager.add_child(None, node);
        manager.set_focus(node);
        manager.take_events();
        manager
            .node_mut(node)
            .expect("focus node")
            .set_eligible(false);

        assert!(manager.drop_focus_if_ineligible());

        assert_eq!(manager.primary_focus(), None);
        assert_eq!(
            manager.take_events(),
            vec![FocusEvent::new(node, FocusEventKind::Blurred)]
        );
    }

    #[test]
    fn directional_scoring_prefers_axis_alignment_over_off_axis_nearness() {
        let mut manager = FocusManager::new();
        let mut current = FocusNode::new();
        current.set_bounds(Some(FocusBounds::from_xywh(0.0, 0.0, 10.0, 10.0)));
        let current = manager.create_node(current);
        let mut aligned = FocusNode::new();
        aligned.set_bounds(Some(FocusBounds::from_xywh(20.0, 0.0, 10.0, 10.0)));
        let aligned = manager.create_node(aligned);
        let mut off_axis = FocusNode::new();
        off_axis.set_bounds(Some(FocusBounds::from_xywh(11.0, 100.0, 10.0, 10.0)));
        let off_axis = manager.create_node(off_axis);
        manager.add_child(None, current);
        manager.add_child(None, off_axis);
        manager.add_child(None, aligned);
        manager.set_focus(current);

        assert!(manager.focus_right());
        assert_eq!(manager.primary_focus(), Some(aligned));
    }

    #[test]
    fn directional_scoring_falls_back_to_root_space_points() {
        let mut manager = FocusManager::new();
        let mut current = FocusNode::new();
        current.set_position(Some(FocusPoint::new(0.0, 0.0)));
        let current = manager.create_node(current);
        let mut aligned = FocusNode::new();
        aligned.set_position(Some(FocusPoint::new(20.0, 0.0)));
        let aligned = manager.create_node(aligned);
        let mut off_axis = FocusNode::new();
        off_axis.set_position(Some(FocusPoint::new(1.0, 100.0)));
        let off_axis = manager.create_node(off_axis);
        manager.add_child(None, current);
        manager.add_child(None, off_axis);
        manager.add_child(None, aligned);
        manager.set_focus(current);

        assert!(manager.focus_right());
        assert_eq!(manager.primary_focus(), Some(aligned));
    }

    #[test]
    fn empty_bounds_are_unavailable_for_directional_navigation() {
        let mut node = FocusNode::new();

        node.set_bounds(Some(FocusBounds::from_xywh(10.0, 20.0, 0.0, 5.0)));

        assert_eq!(node.bounds(), None);
    }

    #[test]
    fn directional_navigation_supports_all_four_directions() {
        let mut manager = FocusManager::new();
        let bounded = |x, y| {
            let mut node = FocusNode::new();
            node.set_bounds(Some(FocusBounds::from_xywh(x, y, 10.0, 10.0)));
            node
        };
        let center = manager.create_node(bounded(0.0, 0.0));
        let left = manager.create_node(bounded(-20.0, 0.0));
        let right = manager.create_node(bounded(20.0, 0.0));
        let up = manager.create_node(bounded(0.0, -20.0));
        let down = manager.create_node(bounded(0.0, 20.0));
        for node_id in [center, left, right, up, down] {
            manager.add_child(None, node_id);
        }

        manager.set_focus(center);
        assert!(manager.focus_left());
        assert_eq!(manager.primary_focus(), Some(left));
        manager.set_focus(center);
        assert!(manager.focus_right());
        assert_eq!(manager.primary_focus(), Some(right));
        manager.set_focus(center);
        assert!(manager.focus_up());
        assert_eq!(manager.primary_focus(), Some(up));
        manager.set_focus(center);
        assert!(manager.focus_down());
        assert_eq!(manager.primary_focus(), Some(down));
    }

    #[test]
    fn nested_occurrence_uses_parent_domain_but_snapshot_clone_isolated() {
        let root = RuntimeFocusTree {
            owner_identity: 11,
            ..RuntimeFocusTree::default()
        };
        let child_target = {
            let mut domain = root.domain.borrow_mut();
            let mut node = FocusNode::new();
            node.set_focusable(RuntimeFocusable::new(22, 7, 8));
            let target = domain.create_node(node);
            domain.manager.add_child(None, target);
            target
        };
        let mut child = root.external_for_owner(22);

        assert!(child.set_focus_target(7));
        assert!(root.domain.borrow().manager.has_focus(child_target));
        let mut root = root;
        assert!(
            root.take_owner_events().is_empty(),
            "the parent occurrence must not consume a nested owner's callback"
        );
        assert_eq!(child.take_owner_events(), [(7, 8, FocusEventKind::Focused)]);

        let mut snapshot = child.clone();
        assert!(snapshot.clear_focus());
        assert!(root.domain.borrow().manager.has_focus(child_target));
    }

    #[test]
    fn listener_registration_does_not_replay_constructor_focus_callbacks() {
        let mut tree = RuntimeFocusTree {
            owner_identity: 11,
            ..RuntimeFocusTree::default()
        };
        let target = {
            let mut domain = tree.domain.borrow_mut();
            let mut node = FocusNode::new();
            node.set_focusable(RuntimeFocusable::new(11, 7, 8));
            let target = domain.create_node(node);
            domain.manager.add_child(None, target);
            target
        };

        assert!(tree.set_focus_target(7));
        assert!(tree.domain.borrow().manager.has_focus(target));
        tree.discard_unregistered_events();

        assert!(tree.take_owner_events().is_empty());
        assert!(tree.target_has_focus(7));
    }

    #[test]
    fn snapshot_clone_preserves_untranslated_focus_callbacks_without_aliasing() {
        let mut tree = RuntimeFocusTree {
            owner_identity: 11,
            ..RuntimeFocusTree::default()
        };
        {
            let mut domain = tree.domain.borrow_mut();
            let mut node = FocusNode::new();
            node.set_focusable(RuntimeFocusable::new(11, 7, 8));
            let target = domain.create_node(node);
            domain.manager.add_child(None, target);
        }
        assert!(tree.set_focus_target(7));

        let mut snapshot = tree.clone();
        assert_eq!(
            snapshot.take_owner_events(),
            [(7, 8, FocusEventKind::Focused)]
        );
        assert_eq!(
            tree.take_owner_events(),
            [(7, 8, FocusEventKind::Focused)],
            "draining the snapshot may not consume the source callback"
        );
    }
}
