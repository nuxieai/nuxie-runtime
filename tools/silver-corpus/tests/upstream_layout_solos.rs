//! The complete dynamic enum loop in solo_test.cpp at 86fc70a7.
use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::PathBuf;

#[test]
fn solo_children_of_a_layout_render_for_every_state() -> anyhow::Result<()> {
    let runtime = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let manifest =
        read_manifest(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../silver-corpus.toml"))?;
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == "layout_solos")
        .ok_or_else(|| anyhow::anyhow!("missing layout_solos producer"))?;
    // Execution reads the imported DataEnum's full value list and requires
    // every index setter to succeed before advance/apply, draw, and addFrame.
    let execution = Execution::run(case, &runtime)?;
    let actual = parse_sriv(execution.bytes())?;
    let expected = parse_sriv(&std::fs::read(resolve_expected(&runtime, case))?)?;
    compare_sriv(&expected, &actual)
        .map_err(|difference| anyhow::anyhow!("layout_solos: {difference}"))
}
