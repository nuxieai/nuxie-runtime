//! Complete solo_test.cpp Silver scenarios through 3b2c51e2.
use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::PathBuf;

#[test]
fn solo_children_of_a_layout_render_for_every_state() -> anyhow::Result<()> {
    run("layout_solos")
}

#[test]
fn solo_children_of_a_layout_render_fitted_to_the_layout_parent() -> anyhow::Result<()> {
    run("layout_solos_fit_to_layout_parent")
}

#[test]
fn a_leaf_parented_by_the_artboard_fits_it() -> anyhow::Result<()> {
    run("solo_nested_artboard_leaf_no_solo")
}

#[test]
fn an_opted_in_leaf_under_a_solo_fits_the_layout_above_it() -> anyhow::Result<()> {
    run("solo_nested_artboard_leaf_fits_parent_layout")
}

#[test]
fn a_legacy_leaf_under_a_solo_frames_its_own_bounds() -> anyhow::Result<()> {
    run("solo_nested_artboard_leaf_solo")
}

fn run(id: &str) -> anyhow::Result<()> {
    let runtime = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let manifest =
        read_manifest(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../silver-corpus.toml"))?;
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == id)
        .ok_or_else(|| anyhow::anyhow!("missing {id} producer"))?;
    // Execution reads the imported DataEnum's full value list and requires
    // every index setter to succeed before advance/apply, draw, and addFrame.
    let execution = Execution::run(case, &runtime)?;
    let actual = parse_sriv(execution.bytes())?;
    let expected = parse_sriv(&std::fs::read(resolve_expected(&runtime, case))?)?;
    compare_sriv(&expected, &actual).map_err(|difference| anyhow::anyhow!("{id}: {difference}"))
}
