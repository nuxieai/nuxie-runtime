//! API-contract coverage for the AF-7 `SimpleArray<T>` ownership adaptation.
//!
//! Pinned C++ keeps the production implementation in `simple_array.hpp`; its
//! `simple_array.cpp` translation unit only defines `TESTING` allocator
//! counters. Runtime-loaded Rust data uses `Vec<T>`, borrowed slices, and
//! `Arc<[T]>` instead. Exact malloc/realloc/free counts are allocator-specific
//! and intentionally have no Rust parity contract.

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_runtime::{ArtboardInstance, ProjectDataValue, ScriptCoreString};
use std::mem::size_of_val;
use std::path::PathBuf;
use std::sync::Arc;

fn upstream_fixture(name: &str) -> PathBuf {
    PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets")
    .join(name)
}

fn state_machine_fixture() -> ArtboardInstance {
    let bytes = std::fs::read(upstream_fixture("state_machine_transition.riv"))
        .expect("read pinned state-machine fixture");
    let file = read_runtime_file(&bytes).expect("import pinned state-machine fixture");
    let graph = GraphFile::from_runtime_file(&file).expect("build state-machine graph");
    ArtboardInstance::from_graph_with_artboards(&file, &graph.artboards[0], &graph.artboards)
        .expect("instantiate state-machine fixture")
}

#[test]
fn empty_owned_and_borrowed_storage_has_the_simple_array_empty_contract() {
    // `array initializes as expected` and the safe-Rust projection of
    // `delegating ctor accepts (nullptr, 0) without UB`.
    let owned = ScriptCoreString::default();
    let borrowed = owned.as_bytes();

    assert!(owned.as_bytes().is_empty());
    assert_eq!(size_of_val(owned.as_bytes()), 0);
    assert_eq!(owned.as_bytes().iter().count(), 0);
    assert!(borrowed.is_empty());
    assert_eq!(ScriptCoreString::from_bytes(borrowed), owned);
}

#[test]
fn owned_storage_preserves_size_bytes_order_and_iteration() {
    // `simple array can be created` and `can iterate simple array`.
    let source = (0..10).collect::<Vec<u8>>();
    let owned = ScriptCoreString::from_bytes(source.clone());

    assert_eq!(owned.as_bytes().len(), 10);
    assert_eq!(size_of_val(owned.as_bytes()), 10 * size_of::<u8>());
    assert_eq!(owned.as_bytes().iter().copied().sum::<u8>(), 45);
    assert_eq!(owned.as_bytes(), source.as_slice());

    let powers = ScriptCoreString::from_bytes([2, 4, 8, 16]);
    let powers = powers.into_bytes();
    assert_eq!(powers.into_iter().collect::<Vec<_>>(), [2, 4, 8, 16]);
}

#[test]
fn vec_builder_and_reset_preserve_only_initialized_values() {
    // Meaningful API portions of `can build up a simple array` and
    // `builders can be reset`; capacity and allocator counts are inapplicable.
    let mut builder = Vec::with_capacity(2);
    builder.push(1);
    builder.push(2);
    builder.push(3);

    assert_eq!(builder.as_slice(), [1, 2, 3]);
    assert_eq!(builder.iter().count(), 3);

    let compact = ScriptCoreString::from_bytes(builder);
    assert_eq!(compact.as_bytes(), [1, 2, 3]);

    let mut resettable = vec![1_u32, 2, 3];
    resettable = Vec::with_capacity(4);
    resettable.extend([3, 2]);
    let reset = ProjectDataValue::List(
        resettable
            .into_iter()
            .map(|value| ProjectDataValue::Number(f64::from(value)))
            .collect(),
    );
    assert_eq!(
        reset,
        ProjectDataValue::List(vec![
            ProjectDataValue::Number(3.0),
            ProjectDataValue::Number(2.0),
        ])
    );
}

#[test]
fn nested_owned_storage_moves_without_aliasing_payloads() {
    // Meaningful API portions of `arrays of arrays work` and
    // `builder arrays of arrays work`; allocation counts are inapplicable.
    let mut numbers_a = vec![33.0, 22.0, 44.0, 66.0];
    let mut numbers_b = vec![1.0, 2.0, 3.0];
    let nested = ProjectDataValue::List(vec![
        ProjectDataValue::List(
            std::mem::take(&mut numbers_a)
                .into_iter()
                .map(ProjectDataValue::Number)
                .collect(),
        ),
        ProjectDataValue::List(
            std::mem::take(&mut numbers_b)
                .into_iter()
                .map(ProjectDataValue::Number)
                .collect(),
        ),
    ]);

    assert!(numbers_a.is_empty());
    assert!(numbers_b.is_empty());
    let ProjectDataValue::List(rows) = nested else {
        unreachable!("constructed a list")
    };
    assert_eq!(rows.len(), 2);
    assert_eq!(
        rows[0],
        ProjectDataValue::List(vec![
            ProjectDataValue::Number(33.0),
            ProjectDataValue::Number(22.0),
            ProjectDataValue::Number(44.0),
            ProjectDataValue::Number(66.0),
        ])
    );

    // Exercise a production Arc<Vec<T>> owner and its borrowed-slice view.
    // A clone must retain the exact immutable definition allocation.
    let artboard = state_machine_fixture();
    let borrowed = artboard.state_machines();
    assert!(!borrowed.is_empty());
    let definition = borrowed[0].clone();
    assert!(Arc::ptr_eq(&definition.inputs, &borrowed[0].inputs));
    assert!(Arc::ptr_eq(&definition.layers, &borrowed[0].layers));
    assert_eq!(definition.input_count(), borrowed[0].input_count());
    assert_eq!(definition.layer_count(), borrowed[0].layer_count());
}

#[test]
fn fallible_growth_rejects_overflow_and_leaves_storage_composable() {
    // `ctor returns empty array on size*sizeof(T) overflow`, `delegating ctor
    // stays safe on overflow`, and `overflow-failed array stays composable`.
    // A slice cannot encode C++'s invalid `(pointer, huge_len)` pair, so the
    // separately fallible allocation preflight is the safe-Rust contract.
    let mut failed = Vec::<u64>::new();
    let huge = usize::MAX / 4;
    assert!(failed.try_reserve_exact(huge).is_err());
    assert!(failed.is_empty());
    assert_eq!(failed.iter().count(), 0);

    let moved = ProjectDataValue::List(
        failed
            .into_iter()
            .map(|value| ProjectDataValue::Number(value as f64))
            .collect(),
    );
    assert_eq!(moved, ProjectDataValue::List(Vec::new()));
    let copied = moved.clone();
    assert_eq!(copied, ProjectDataValue::List(Vec::new()));

    // `ctor still works for normal sizes after overflow guard`.
    let normal = ScriptCoreString::from_bytes(vec![0_u8; 8]);
    assert_eq!(normal.as_bytes().len(), 8);
    assert_eq!(size_of_val(normal.as_bytes()), 8 * size_of::<u8>());
}
