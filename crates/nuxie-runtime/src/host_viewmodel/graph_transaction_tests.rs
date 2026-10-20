#![allow(
    clippy::unwrap_used,
    reason = "literal public-boundary fixture assertions"
)]
use super::*;
use crate::{File, RuntimeFactoryHandle, RuntimeOwnedViewModelGraphTransaction};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
#[path = "../../tests/support/value_policy_fixture.rs"]
mod fixture;

/// A Values root with an empty `picks` list and two detached Values records.
fn values() -> (
    RuntimeOwnedViewModelHandle,
    [RuntimeOwnedViewModelHandle; 2],
    PersistentFactory<RecordingFactory>,
) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &fixture::group_fixture(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let root = RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::from_instance(file.clone(), 0, 0).unwrap(),
    );
    let records = [0, 1].map(|_| {
        RuntimeOwnedViewModelHandle::new(
            RuntimeOwnedViewModelInstance::new(file.clone(), 0).unwrap(),
        )
    });
    (root, records, factory)
}

fn number(owner: &RuntimeOwnedViewModelHandle) -> Option<f32> {
    owner.borrow().number_value_by_property_name("n")
}

/// Write `n` on `row`, then detach it from the root inside one frame.
fn write_then_detach(
    root: &RuntimeOwnedViewModelHandle,
    row: &RuntimeOwnedViewModelHandle,
    value: f32,
) -> (
    RuntimeOwnedViewModelGraphTransaction,
    RuntimeViewModelChangeCapture,
) {
    assert!(root.push_list_item_by_property_name_path("picks", row));
    let transaction =
        RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(root), 4096).unwrap();
    let capture = RuntimeViewModelChangeCapture::begin().unwrap();
    assert!(row.borrow_mut().set_number_by_property_name("n", value));
    assert!(root.remove_list_item_by_property_name_path("picks", 0));
    (transaction, capture)
}

#[test]
fn frame_resolves_a_write_by_an_owner_detached_during_the_frame() {
    let (root, [row, _], _factory) = values();
    // The live graph alone has no owner for the detached row's write.
    let (transaction, capture) = write_then_detach(&root, &row, 4.0);
    assert!(
        RuntimeOwnedViewModelHandle::resolve_change_capture_across_with_owners(
            std::slice::from_ref(&root),
            capture,
        )
        .is_none()
    );
    drop(transaction);

    let (transaction, capture) = write_then_detach(&root, &row, 5.0);
    let changes = transaction
        .resolve_change_capture_with_owners(std::slice::from_ref(&root), capture)
        .unwrap();
    assert!(changes.iter().any(|(owner, change)| {
        owner.ptr_eq(&row)
            && change.owner_instance_identity == row.instance_identity()
            && change.value == RuntimeViewModelChangeValue::Number(5.0)
    }));
    transaction.commit();
    assert_eq!(number(&row), Some(5.0));
}

#[test]
fn frame_rollback_restores_a_detached_owner() {
    let (root, [row, _], _factory) = values();
    let (transaction, capture) = write_then_detach(&root, &row, 5.0);
    assert!(
        transaction
            .resolve_change_capture_with_owners(std::slice::from_ref(&root), capture)
            .is_some()
    );
    drop(transaction);
    assert_eq!(number(&row), Some(0.0));
}

#[test]
fn frame_still_refuses_writes_outside_its_scope_and_overflowed_captures() {
    let (root, [row, outsider], _factory) = values();
    // Never part of the frame.
    {
        let transaction =
            RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096)
                .unwrap();
        let capture = RuntimeViewModelChangeCapture::begin().unwrap();
        assert!(outsider.borrow_mut().set_number_by_property_name("n", 1.0));
        assert!(
            transaction
                .resolve_change_capture_with_owners(std::slice::from_ref(&root), capture)
                .is_none()
        );
    }
    // Joined during the frame and still present: the live graph owns it.
    {
        let transaction =
            RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096)
                .unwrap();
        let capture = RuntimeViewModelChangeCapture::begin().unwrap();
        assert!(root.push_list_item_by_property_name_path("picks", &outsider));
        assert!(outsider.borrow_mut().set_number_by_property_name("n", 2.0));
        let changes = transaction
            .resolve_change_capture_with_owners(std::slice::from_ref(&root), capture)
            .unwrap();
        assert!(changes.iter().any(|(owner, _)| owner.ptr_eq(&outsider)));
        transaction.commit();
        assert!(root.remove_list_item_by_property_name_path("picks", 0));
    }
    // Joined and left within the frame: in neither the checkpoint nor the
    // final graph, so its write cannot be rolled back and still fails.
    {
        let transaction =
            RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096)
                .unwrap();
        let capture = RuntimeViewModelChangeCapture::begin().unwrap();
        assert!(root.push_list_item_by_property_name_path("picks", &outsider));
        assert!(outsider.borrow_mut().set_number_by_property_name("n", 3.0));
        assert!(root.remove_list_item_by_property_name_path("picks", 0));
        assert!(
            transaction
                .resolve_change_capture_with_owners(std::slice::from_ref(&root), capture)
                .is_none()
        );
    }
    // A bounded journal that overflowed fails even for checkpointed owners.
    {
        assert!(root.push_list_item_by_property_name_path("picks", &row));
        let transaction =
            RuntimeOwnedViewModelGraphTransaction::begin(std::slice::from_ref(&root), 4096)
                .unwrap();
        let capture = RuntimeViewModelChangeCapture::begin_bounded(1, 1024).unwrap();
        assert!(row.borrow_mut().set_number_by_property_name("n", 6.0));
        assert!(row.borrow_mut().set_number_by_property_name("n", 7.0));
        assert!(
            transaction
                .resolve_change_capture_with_owners(std::slice::from_ref(&root), capture)
                .is_none()
        );
    }
    assert_eq!(number(&row), Some(0.0));
}
