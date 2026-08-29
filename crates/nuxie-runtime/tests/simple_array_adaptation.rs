//! The translated SimpleArray/Builder own every array under test. Rust's
//! allocator does not promise C++ malloc/realloc/free call counts.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::animation::state_machine::StateMachine;
use nuxie_runtime::source::simple_array::{SimpleArray, SimpleArrayBuilder};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle};
use std::{mem::size_of_val, path::PathBuf};

#[test]
fn empty_owned_and_borrowed_storage_has_the_simple_array_empty_contract() {
    let owned = SimpleArray::<u8>::default();
    let borrowed = owned.as_slice();
    assert!(owned.is_empty());
    assert_eq!(owned.size_bytes(), 0);
    assert_eq!(owned.as_slice().iter().count(), 0);
    assert!(borrowed.is_empty());
    assert_eq!(
        SimpleArray::from_slice(borrowed).as_slice(),
        owned.as_slice()
    );
}
#[test]
fn owned_storage_preserves_size_bytes_order_and_iteration() {
    let source = (0..10).collect::<Vec<u8>>();
    let owned = SimpleArray::from_slice(&source);
    assert_eq!(owned.size(), 10);
    assert_eq!(owned.size_bytes(), 10 * size_of::<u8>());
    assert_eq!(owned.as_slice().iter().copied().sum::<u8>(), 45);
    assert_eq!(owned.as_slice(), source.as_slice());
    let powers = SimpleArray::from_slice(&[2, 4, 8, 16]);
    assert_eq!(
        powers.as_slice().iter().copied().collect::<Vec<_>>(),
        [2, 4, 8, 16]
    );
}
#[test]
fn vec_builder_and_reset_preserve_only_initialized_values() {
    let mut builder = SimpleArrayBuilder::with_reserve(2);
    builder.add(1);
    builder.add(2);
    builder.add(3);
    assert_eq!(builder.as_slice(), [1, 2, 3]);
    assert_eq!(builder.as_slice().iter().count(), 3);
    let compact = builder.into_simple_array();
    assert_eq!(compact.as_slice(), [1, 2, 3]);
    let mut resettable = SimpleArrayBuilder::new();
    for value in [1_u32, 2, 3] {
        resettable.add(value);
    }
    resettable = SimpleArrayBuilder::with_reserve(4);
    resettable.add(3);
    resettable.add(2);
    let reset = resettable.into_simple_array();
    assert_eq!(
        reset
            .as_slice()
            .iter()
            .copied()
            .map(f64::from)
            .collect::<Vec<_>>(),
        [3.0, 2.0]
    );
}
#[test]
fn nested_owned_storage_moves_without_aliasing_payloads() {
    let mut numbers_a = SimpleArray::from_slice(&[33.0, 22.0, 44.0, 66.0]);
    let mut numbers_b = SimpleArray::from_slice(&[1.0, 2.0, 3.0]);
    let mut nested = SimpleArrayBuilder::new();
    nested.add(std::mem::take(&mut numbers_a));
    nested.add(std::mem::take(&mut numbers_b));
    let nested = nested.into_simple_array();
    assert!(numbers_a.is_empty());
    assert!(numbers_b.is_empty());
    assert_eq!(nested.size(), 2);
    assert_eq!(nested[0].as_slice(), [33.0, 22.0, 44.0, 66.0]);

    // Preserve the additional runtime ownership assertion against the actual
    // immutable StateMachine definition handles, not a parallel Arc<Vec> graph.
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        PathBuf::from(root).join("tests/unit_tests/assets/state_machine_transition.riv"),
    )
    .expect("fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("import");
    let artboard = file.with_file(|file| file.artboard()).expect("Artboard");
    let borrowed = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.state_machine_handles().to_vec())
        .unwrap();
    assert!(!borrowed.is_empty());
    let definition = borrowed[0].clone();
    assert_eq!(definition, borrowed[0]);
    let contents = |owner: &nuxie_runtime::CoreHandle| {
        owner
            .with_downcast::<StateMachine, _>(|machine| {
                (
                    machine.input_count(),
                    machine.layer_count(),
                    (0..machine.input_count())
                        .map(|index| machine.input(index))
                        .collect::<Vec<_>>(),
                    (0..machine.layer_count())
                        .map(|index| machine.layer(index))
                        .collect::<Vec<_>>(),
                )
            })
            .unwrap()
    };
    let original = contents(&borrowed[0]);
    let cloned = contents(&definition);
    assert_eq!(cloned.2, original.2, "exact native input identities");
    assert_eq!(cloned.3, original.3, "exact native layer identities");
    assert_eq!(cloned.0, original.0);
    assert_eq!(cloned.1, original.1);
}
#[test]
fn upstream_builder_arrays_of_arrays_work_under_the_rust_allocator_adaptation() {
    let mut structs = SimpleArrayBuilder::with_reserve(2);
    for _ in 0..3 {
        structs.add(SimpleArray::from_slice(&[33_u32, 22, 44, 66]));
    }
    assert_eq!(structs.size(), 3);
    assert!(
        structs
            .as_slice()
            .iter()
            .all(|numbers| numbers.as_slice() == [33, 22, 44, 66])
    );
}
#[test]
fn upstream_oom_construction_returns_empty_under_the_rust_allocator_adaptation() {
    let array = SimpleArray::<u8>::new(usize::MAX);
    assert!(array.is_empty());
    assert_eq!(array.size(), 0);
    assert!(array.as_slice().is_empty());
}
#[test]
fn fallible_growth_rejects_overflow_and_leaves_storage_composable() {
    let failed = SimpleArray::<u64>::new(usize::MAX / 4);
    assert!(failed.is_empty());
    assert_eq!(failed.as_slice().iter().count(), 0);
    let moved = failed;
    assert!(moved.is_empty());
    let copied = moved.clone();
    assert!(copied.is_empty());
    let normal = SimpleArray::<u8>::new(8);
    assert_eq!(normal.size(), 8);
    assert_eq!(size_of_val(normal.as_slice()), 8 * size_of::<u8>());
}
