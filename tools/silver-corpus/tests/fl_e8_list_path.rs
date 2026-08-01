use silver_corpus::{Execution, compare_sriv, parse_sriv, read_manifest, resolve_expected};
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("silver-corpus workspace root")
        .to_path_buf()
}

fn runtime_root() -> PathBuf {
    std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
}

#[test]
fn d_lp_upstream_eight_phase_scenario_is_byte_exact() -> anyhow::Result<()> {
    // D-LP-INIT, D-LP-XY, D-LP-RD, D-LP-DETACHED, D-LP-POINT,
    // D-LP-INVALID, D-LP-PARTIAL, and D-LP-LIVE (including all 60 frames)
    // are one action stream derived from data_binding_test.cpp:1585-1819.
    let manifest = read_manifest(&workspace_root().join("silver-corpus.toml"))?;
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == "list_to_path")
        .ok_or_else(|| anyhow::anyhow!("missing exact list_to_path silver case"))?;
    let runtime = runtime_root();
    let actual = parse_sriv(Execution::run(case, &runtime)?.bytes())?;
    let expected = parse_sriv(&std::fs::read(resolve_expected(&runtime, case))?)?;
    compare_sriv(&expected, &actual)
        .map_err(|difference| anyhow::anyhow!("list_to_path: {difference}"))
}
