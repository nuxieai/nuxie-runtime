//! Native type predicates must remain usable while their owner is borrowed.
//! Expected ancestry is the explicit switch in the pinned generated
//! ShapeBase, NodeBase, and ArtboardBase headers, not a Rust predicate oracle.

use nuxie_runtime::source::{
    artboard::{Artboard, ArtboardInstance, RuntimeArtboardInstanceHandle},
    core::{CoreArena, CoreHandle},
    node::Node,
    shapes::shape::Shape,
};

const SHAPE_TYPES: &[u16] = &[3, 13, 2, 38, 91, 11, 10];
const NODE_TYPES: &[u16] = &[2, 38, 91, 11, 10];
const ARTBOARD_TYPES: &[u16] = &[1, 409, 13, 2, 38, 91, 11, 10];

fn assert_types_during_mutable_borrow(handle: &CoreHandle, leaf: u16, ancestry: &[u16]) {
    handle
        .with_mut(|owner| {
            let embedded_core = owner.core();
            assert_eq!(handle.core_type(), Some(leaf));
            assert_eq!(embedded_core.core_type(), leaf);
            for key in 0..=u16::MAX {
                let expected = ancestry.contains(&key);
                assert_eq!(handle.is_type_of(key), expected, "handle key {key}");
                assert_eq!(embedded_core.is_type_of(key), expected, "Core key {key}");
            }
        })
        .expect("live native occurrence");
}

#[test]
fn shape_and_node_type_queries_preserve_exact_ancestry_under_owner_borrow() {
    let arena = CoreArena::default();
    let shape = arena.insert(Shape::default());
    let node = arena.insert(Node::default());

    assert_types_during_mutable_borrow(&shape, 3, SHAPE_TYPES);
    assert_types_during_mutable_borrow(&node, 2, NODE_TYPES);
}

#[test]
fn removed_handle_stays_stale_when_its_slot_is_reused_for_another_type() {
    let arena = CoreArena::default();
    let shape = arena.insert(Shape::default());
    let old_identity = shape.identity_key();
    let removed = arena.remove(&shape).expect("remove actual Shape owner");

    assert!(!shape.is_alive());
    assert_eq!(shape.core_type(), None);
    // Removing the occurrence invalidates its handle, not the returned native
    // object's concrete type. The boxed Shape still exists independently.
    assert_eq!(removed.core().core_type(), 3);
    for &key in SHAPE_TYPES {
        assert!(!shape.is_type_of(key));
        assert!(removed.core().is_type_of(key));
    }

    let node = arena.insert(Node::default());
    let new_identity = node.identity_key();
    assert_eq!(new_identity.0, old_identity.0, "same arena");
    assert_eq!(new_identity.1, old_identity.1, "the freed slot was reused");
    assert_ne!(new_identity.2, old_identity.2, "new occurrence generation");
    assert_eq!(shape.core_type(), None);
    assert_eq!(removed.core().core_type(), 3);
    for key in 0..=u16::MAX {
        assert!(!shape.is_type_of(key), "stale handle key {key}");
        assert_eq!(
            removed.core().is_type_of(key),
            SHAPE_TYPES.contains(&key),
            "live removed Shape key {key}",
        );
    }
    assert_types_during_mutable_borrow(&node, 2, NODE_TYPES);
}

#[test]
fn dropping_the_arena_invalidates_retained_type_queries() {
    let shape = {
        let arena = CoreArena::default();
        arena.insert(Shape::default())
    };

    assert!(!shape.is_alive());
    assert_eq!(shape.core_type(), None);
    for &key in SHAPE_TYPES {
        assert!(!shape.is_type_of(key));
    }
}

#[test]
fn runtime_artboard_root_uses_the_same_predicate_without_reborrowing_instance() {
    let instance = RuntimeArtboardInstanceHandle::new(ArtboardInstance {
        base: Artboard::default(),
    });
    let root = instance.core_handle();

    assert_types_during_mutable_borrow(&root, 1, ARTBOARD_TYPES);

    drop(instance);
    assert!(!root.is_alive());
    assert_eq!(root.core_type(), None);
    for &key in ARTBOARD_TYPES {
        assert!(!root.is_type_of(key));
    }
}
