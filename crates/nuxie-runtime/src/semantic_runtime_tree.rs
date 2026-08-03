//! Retained semantic domain state shared by mounted artboard occurrences
//! beneath a state-machine instance; interface stays on `StateMachineInstance`.

use std::collections::{BTreeMap, BTreeSet};

use crate::artboard::ArtboardInstance;
use crate::components::Mat2D;
use crate::semantic_data::{RuntimeSemanticData, SemanticNodeHandle};
use crate::state_machine::semantic_listener_group;
use crate::state_machine::state_machine_instance::closest_semantic_node;
use crate::semantic_manager::SemanticManager;
use crate::state_machine::state_machine_instance::{RuntimeSemanticOccurrenceKey, RuntimeSemanticRoute};

/// One retained semantic domain shared by every mounted artboard occurrence
/// beneath a state-machine instance. The interface stays on
/// `StateMachineInstance`; occurrence keys and recursive Artboard traversal
/// remain implementation details at this seam.
#[derive(Debug, Default)]
pub(crate) struct RuntimeSemanticTree {
    pub(crate) manager: SemanticManager,
    pub(crate) data: BTreeMap<RuntimeSemanticOccurrenceKey, RuntimeSemanticData>,
    boundaries: BTreeMap<u64, SemanticNodeHandle>,
    pub(crate) routes: BTreeMap<u32, RuntimeSemanticRoute>,
    pub(crate) registered_listener_groups: BTreeSet<(RuntimeSemanticOccurrenceKey, usize)>,
    pub(crate) pending_focus_scroll: Option<RuntimeSemanticRoute>,
}


impl RuntimeSemanticTree {
    pub(crate) fn synchronize(
        &mut self,
        artboard: &mut ArtboardInstance,
        listener_groups: &[semantic_listener_group::RuntimeSemanticListenerGroup],
    ) {
        let root_owner_identity = artboard.instance_identity();
        let mut live = BTreeSet::new();
        let mut live_boundaries = BTreeSet::new();
        self.visit_artboard(
            artboard,
            None,
            Mat2D::IDENTITY,
            false,
            &mut live,
            &mut live_boundaries,
        );

        let stale = self
            .data
            .keys()
            .filter(|key| !live.contains(*key))
            .copied()
            .collect::<Vec<_>>();
        for key in stale {
            if let Some(mut data) = self.data.remove(&key) {
                data.detach(&mut self.manager);
            }
        }
        let stale_boundaries = self
            .boundaries
            .keys()
            .filter(|identity| !live_boundaries.contains(*identity))
            .copied()
            .collect::<Vec<_>>();
        for identity in stale_boundaries {
            if let Some(boundary) = self.boundaries.remove(&identity) {
                self.manager.remove_child(&boundary);
            }
        }
        self.registered_listener_groups
            .retain(|(key, _)| live.contains(key));
        for (group_index, group) in listener_groups.iter().enumerate() {
            let key = RuntimeSemanticOccurrenceKey {
                owner_identity: root_owner_identity,
                data_local_id: group.semantic_data_local_id,
            };
            if self.data.contains_key(&key)
                && self.registered_listener_groups.insert((key, group_index))
            {
                group.register(
                    self.data
                        .get_mut(&key)
                        .expect("registered semantic data remains retained"),
                );
            }
        }
        self.rebuild_routes();
    }

    pub(crate) fn visit_artboard(
        &mut self,
        artboard: &mut ArtboardInstance,
        inherited_parent: Option<SemanticNodeHandle>,
        root_transform: Mat2D,
        needs_boundary: bool,
        live: &mut BTreeSet<RuntimeSemanticOccurrenceKey>,
        live_boundaries: &mut BTreeSet<u64>,
    ) {
        let owner_identity = artboard.instance_identity();
        let effective_parent = if needs_boundary {
            live_boundaries.insert(owner_identity);
            let boundary = self
                .boundaries
                .entry(owner_identity)
                .or_insert_with(|| {
                    let node = SemanticNodeHandle::new(0);
                    node.borrow_mut().set_boundary_node(true);
                    node
                })
                .clone();
            let current_parent_id = boundary.borrow().parent_id();
            let desired_parent_id = inherited_parent
                .as_ref()
                .map(|parent| parent.borrow().id());
            if boundary.borrow().manager_identity() == Some(self.manager.identity())
                && current_parent_id != desired_parent_id
            {
                self.manager.remove_child(&boundary);
            }
            if boundary.borrow().manager_identity().is_none() {
                self.manager.add_child(inherited_parent.as_ref(), boundary.clone());
            }
            Some(boundary)
        } else {
            inherited_parent
        };
        let semantic_locals = artboard
            .components()
            .iter()
            .filter(|component| component.type_name == "SemanticData")
            .map(|component| component.local_id)
            .collect::<Vec<_>>();

        for local_id in &semantic_locals {
            let key = RuntimeSemanticOccurrenceKey {
                owner_identity,
                data_local_id: *local_id,
            };
            live.insert(key);
            if !self.data.contains_key(&key) {
                let mut data = RuntimeSemanticData::from_artboard(artboard, *local_id);
                data.prepare_for_tree(artboard);
                self.data.insert(key, data);
            }
        }

        let nodes_by_target = semantic_locals
            .iter()
            .filter_map(|local_id| {
                let key = RuntimeSemanticOccurrenceKey {
                    owner_identity,
                    data_local_id: *local_id,
                };
                let data = self.data.get(&key)?;
                Some((data.parent_local_id?, data.node_handle()?))
            })
            .collect::<BTreeMap<_, _>>();

        for local_id in &semantic_locals {
            let key = RuntimeSemanticOccurrenceKey {
                owner_identity,
                data_local_id: *local_id,
            };
            let target_local = self.data.get(&key).and_then(|data| data.parent_local_id);
            let parent = target_local
                .and_then(|target| artboard.component_parent_local(target))
                .and_then(|parent| closest_semantic_node(artboard, parent, &nodes_by_target))
                .or_else(|| effective_parent.clone());
            let data = self.data.get_mut(&key).expect("semantic data was retained");
            data.synchronize_from_artboard(artboard, &mut self.manager, root_transform);
            data.reconcile_tree_membership(
                &mut self.manager,
                parent.as_ref(),
                artboard,
                root_transform,
            );
        }

        let nested_hosts = artboard
            .nested_artboards
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for host_local in nested_hosts {
            let parent = closest_semantic_node(artboard, host_local, &nodes_by_target)
                .or_else(|| effective_parent.clone());
            let host_world = artboard.runtime_component_world_transform_with_scroll(host_local);
            if let Some(nested) = artboard.nested_artboards.get_mut(&host_local) {
                let child_root = nested
                    .child
                    .mounted_root_transform(root_transform.multiply(host_world));
                self.visit_artboard(
                    &mut nested.child,
                    parent,
                    child_root,
                    true,
                    live,
                    live_boundaries,
                );
            }
        }

        let list_locals = artboard.component_list_locals().collect::<Vec<_>>();
        let list_root_transforms =
            artboard.runtime_component_list_child_root_transforms(root_transform);
        for list_local in list_locals {
            let parent = closest_semantic_node(artboard, list_local, &nodes_by_target)
                .or_else(|| effective_parent.clone());
            let Some(items) = artboard.component_list_items_mut(list_local) else {
                continue;
            };
            for (item_index, item) in items.iter_mut().enumerate() {
                let child_root = list_root_transforms
                    .get(&list_local)
                    .and_then(|roots| roots.get(item_index))
                    .copied()
                    .unwrap_or(root_transform);
                self.visit_artboard(
                    &mut item.child,
                    parent.clone(),
                    child_root,
                    true,
                    live,
                    live_boundaries,
                );
            }
        }
    }

    pub(crate) fn rebuild_routes(&mut self) {
        self.routes.clear();
        for (key, data) in &self.data {
            let Some(target_local_id) = data.parent_local_id else {
                continue;
            };
            let id = data.semantic_id();
            if id != 0 {
                self.routes.insert(
                    id,
                    RuntimeSemanticRoute {
                        owner_identity: key.owner_identity,
                        target_local_id,
                        data_local_id: key.data_local_id,
                    },
                );
            }
        }
    }
}
