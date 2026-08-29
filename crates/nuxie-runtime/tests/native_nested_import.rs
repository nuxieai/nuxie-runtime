//! The source/instance distinction in pinned NestedArtboard::nest.

use nuxie_runtime::source::{artboard::Artboard, core::CoreArena, nested_artboard::NestedArtboard};

#[test]
fn recording_an_authored_reference_does_not_create_an_instance() {
    let arena = CoreArena::default();
    let source = arena.insert(Artboard::default());
    let mut nested = NestedArtboard::default();

    // An import-time source does not require an instance factory: upstream
    // records the reference and returns without initializing or mutating it.
    nested.referenced_artboard(Some(source.clone()));

    assert!(nested.artboard_instance_handle(0).is_none());
    assert_eq!(nested.source_artboard(), Some(source.clone()));
    assert_eq!(
        source.with_downcast::<Artboard, _>(Artboard::is_instance),
        Some(false)
    );
}
