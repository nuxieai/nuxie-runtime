use super::SemanticCollectionData;
use crate::source::semantic::{
    semantic_manager::SemanticManager, semantic_role::SemanticRole,
    semantic_snapshot::SemanticsDiffNode,
};
use std::collections::{HashMap, HashSet};

/// Owned values from one semantic revision. Unknown and zero remain distinct.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SemanticCollectionMetadata {
    pub collection_id: Option<u32>,
    pub item_count: Option<u32>,
    pub item_position: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticCollectionError {
    InvalidMetadata,
    LimitExceeded,
}

/// Capture logical metadata from its authored owners, never from visible counts.
/// The caller must supply the manager's current, complete flattened snapshot.
pub fn capture_semantic_collections(
    manager: &SemanticManager,
    nodes: &[SemanticsDiffNode],
) -> Result<Vec<SemanticCollectionMetadata>, SemanticCollectionError> {
    use SemanticCollectionError::{InvalidMetadata, LimitExceeded};
    if nodes.len() > 16_384 {
        return Err(LimitExceeded);
    }
    let by_id: HashMap<_, _> = nodes.iter().map(|node| (node.id, node)).collect();
    if by_id.len() != nodes.len() {
        return Err(InvalidMetadata);
    }
    let mut authored = HashMap::new();
    for node in nodes {
        let live = manager.node_by_id(node.id).ok_or(InvalidMetadata)?;
        let data = live.borrow().semantic_data.clone();
        if let Some(data) = data {
            if let Some(metadata) = data.with_downcast::<SemanticCollectionData, _>(|data| {
                data.valid_role()
                    .then_some((data.item_count(), data.item_position()))
            }) {
                authored.insert(node.id, metadata.ok_or(InvalidMetadata)?);
            }
        }
    }
    let mut owners: HashMap<usize, Option<u32>> = HashMap::new();
    let mut positions = HashSet::new();
    let mut exposed_members: HashMap<u32, usize> = HashMap::new();
    let mut captured = Vec::with_capacity(nodes.len());
    for node in nodes {
        let mut metadata = SemanticCollectionMetadata::default();
        if node.role == SemanticRole::List as u32 {
            metadata.item_count = authored.get(&node.id).and_then(|value| value.0);
        } else if node.role == SemanticRole::ListItem as u32 {
            metadata.item_position = authored.get(&node.id).and_then(|value| value.1);
            let mut ancestor = manager
                .node_by_id(node.id)
                .ok_or(InvalidMetadata)?
                .borrow()
                .parent();
            let mut visited = HashSet::new();
            let mut path = Vec::new();
            while let Some(parent) = ancestor {
                let identity = std::rc::Rc::as_ptr(&parent) as usize;
                if let Some(owner) = owners.get(&identity) {
                    metadata.collection_id = *owner;
                    break;
                }
                if !visited.insert(identity) || visited.len() > 16_384 {
                    return Err(InvalidMetadata);
                }
                path.push(identity);
                let parent = parent.borrow();
                if parent.role == SemanticRole::List as u32 {
                    metadata.collection_id = by_id.get(&parent.id()).map(|node| node.id);
                    // Upstream label absorption may omit an ordinary list while
                    // preserving an actionable item. Its owner stays unknown;
                    // explicitly authored collection meaning must not be lost.
                    if metadata.collection_id.is_none()
                        && parent
                            .semantic_data
                            .as_ref()
                            .is_some_and(|data| data.is_type_of(SemanticCollectionData::TYPE_KEY))
                    {
                        return Err(InvalidMetadata);
                    }
                    break;
                }
                ancestor = parent.parent();
            }
            for identity in path {
                owners.insert(identity, metadata.collection_id);
            }
            if let Some(owner) = metadata.collection_id {
                let count = authored.get(&owner).and_then(|value| value.0);
                if let Some(position) = metadata.item_position {
                    if count.is_some_and(|count| position >= count)
                        || !positions.insert((owner, position))
                    {
                        return Err(InvalidMetadata);
                    }
                }
                let members = exposed_members.entry(owner).or_default();
                *members += 1;
                if count.is_some_and(|count| *members > count as usize) {
                    return Err(InvalidMetadata);
                }
            }
            if authored.contains_key(&node.id) && metadata.collection_id.is_none() {
                return Err(InvalidMetadata);
            }
        }
        captured.push(metadata);
    }
    Ok(captured)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{
        core::{CoreArena, CoreHandle, CoreObject, binary_reader::BinaryReader},
        generated::core_registry::{CoreField, CoreRegistryObject},
        semantic::{
            semantic_data::SemanticData, semantic_manager::RuntimeSemanticManagerHandle,
            semantic_node::SemanticNodeRef,
        },
    };

    fn add(
        arena: &CoreArena,
        manager: &RuntimeSemanticManagerHandle,
        parent: Option<SemanticNodeRef>,
        role: u32,
        value: Option<u32>,
    ) -> (CoreHandle, SemanticNodeRef) {
        let mut data = SemanticCollectionData::default();
        data.set_uint(CoreField::SemanticDataRole, role);
        if let Some(value) = value {
            let mut bytes = Vec::new();
            let mut remaining = value;
            loop {
                let byte = (remaining & 127) as u8;
                remaining >>= 7;
                bytes.push(byte | if remaining == 0 { 0 } else { 128 });
                if remaining == 0 {
                    break;
                }
            }
            assert!(data.deserialize(
                if role == 10 { 60016 } else { 60017 },
                &mut BinaryReader::new(&bytes)
            ));
        }
        let handle = arena.insert(data);
        let node = handle
            .with_semantic_data_mut(SemanticData::semantic_node)
            .unwrap();
        manager.add_child(parent, node.clone());
        (handle, node)
    }
    fn capture(
        manager: &RuntimeSemanticManagerHandle,
    ) -> Result<Vec<SemanticCollectionMetadata>, SemanticCollectionError> {
        manager.with_semantic_manager_mut(|manager| {
            let nodes = manager.snapshot().to_vec();
            capture_semantic_collections(manager, &nodes)
        })
    }
    fn manager() -> RuntimeSemanticManagerHandle {
        RuntimeSemanticManagerHandle::new(SemanticManager::new())
    }

    #[test]
    fn upstream_label_absorption_does_not_reject_surviving_list_item() {
        assert_absorbed_owner(false, false);
    }

    #[test]
    fn absorbed_owner_cannot_discard_explicit_collection_metadata() {
        assert_absorbed_owner(true, false);
        assert_absorbed_owner(false, true);
        assert_absorbed_owner(true, true);
    }

    fn assert_absorbed_owner(owned_list: bool, owned_item: bool) {
        let arena = CoreArena::default();
        let manager = manager();
        let add = |parent, role, label: &str, owned| {
            let mut data = SemanticData::default();
            data.set_role(role);
            data.set_label(label.into());
            let handle = if owned {
                let mut data = SemanticCollectionData::default();
                data.set_uint(CoreField::SemanticDataRole, role);
                data.set_string(CoreField::SemanticDataLabel, label.into());
                arena.insert(data)
            } else {
                arena.insert(data)
            };
            let node = handle
                .with_semantic_data_mut(SemanticData::semantic_node)
                .unwrap();
            manager.add_child(parent, node.clone());
            node
        };
        let button = add(None, 1, "", false);
        add(Some(button.clone()), 7, "Choose", false);
        let list = add(Some(button), 10, "", owned_list);
        let item = add(Some(list.clone()), 11, "Option", owned_item);
        manager.with_semantic_manager_mut(|manager| {
            let nodes = manager.snapshot().to_vec();
            assert!(!nodes.iter().any(|node| node.id == list.borrow().id()));
            let index = nodes
                .iter()
                .position(|node| node.id == item.borrow().id())
                .unwrap();
            let metadata = capture_semantic_collections(manager, &nodes);
            if owned_list || owned_item {
                assert_eq!(metadata, Err(SemanticCollectionError::InvalidMetadata));
            } else {
                assert_eq!(
                    metadata.unwrap()[index],
                    SemanticCollectionMetadata::default()
                );
            }
        });
    }

    #[test]
    fn three_exposed_items_keep_ten_item_total_and_logical_positions() {
        let arena = CoreArena::default();
        let manager = manager();
        let (_, list) = add(&arena, &manager, None, 10, Some(10));
        for position in [4, 5, 6] {
            add(&arena, &manager, Some(list.clone()), 11, Some(position));
        }
        let frozen = capture(&manager).unwrap();
        assert_eq!(frozen[0].item_count, Some(10));
        assert_eq!(
            frozen[1..]
                .iter()
                .map(|m| m.item_position.unwrap())
                .collect::<Vec<_>>(),
            vec![4, 5, 6]
        );
        assert!(
            frozen[1..]
                .iter()
                .all(|m| m.collection_id == Some(list.borrow().id()))
        );
        // Removing a presented member does not change the authored total or old capture.
        let removed = list.borrow().children()[0].clone();
        manager.remove_child(&removed);
        assert_eq!(capture(&manager).unwrap()[0].item_count, Some(10));
        assert_eq!(frozen.len(), 4);
    }

    #[test]
    fn nested_lists_have_independent_membership_and_unknown_is_not_zero() {
        let arena = CoreArena::default();
        let manager = manager();
        let (_, outer) = add(&arena, &manager, None, 10, Some(1));
        let (_, item) = add(&arena, &manager, Some(outer.clone()), 11, Some(0));
        let (_, inner) = add(&arena, &manager, Some(item), 10, None);
        add(&arena, &manager, Some(inner.clone()), 11, Some(9));
        let data = capture(&manager).unwrap();
        assert_eq!(data[0].item_count, Some(1));
        assert_eq!(data[1].collection_id, Some(outer.borrow().id()));
        assert_eq!(data[2].item_count, None);
        assert_eq!(data[3].collection_id, Some(inner.borrow().id()));
        assert_eq!(data[3].item_position, Some(9));
    }

    #[test]
    fn invalid_owner_positions_and_empty_collection_members_fail_capture() {
        for (count, positions) in [
            (0, vec![None]),
            (2, vec![Some(2)]),
            (3, vec![Some(1), Some(1)]),
        ] {
            let arena = CoreArena::default();
            let manager = manager();
            let (_, list) = add(&arena, &manager, None, 10, Some(count));
            for position in positions {
                add(&arena, &manager, Some(list.clone()), 11, position);
            }
            assert_eq!(
                capture(&manager),
                Err(SemanticCollectionError::InvalidMetadata)
            );
        }
        let arena = CoreArena::default();
        let manager = manager();
        add(&arena, &manager, None, 11, Some(0));
        assert_eq!(
            capture(&manager),
            Err(SemanticCollectionError::InvalidMetadata)
        );
    }
}
