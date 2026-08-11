use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=NUX_RUNTIME_SOURCE_REVISION");
    println!("cargo:rerun-if-env-changed=NUX_RUNTIME_BUILD_INPUTS_HASH");
    println!("cargo:rerun-if-env-changed=NUX_RUNTIME_CONTRACT_FINGERPRINT");
    println!("cargo:rerun-if-env-changed=NUX_RUNTIME_BUILD_PROFILE");
    println!("cargo:rerun-if-env-changed=NUX_RUNTIME_RUSTC_VERSION");
    println!("cargo:rerun-if-env-changed=NUX_RUNTIME_DISTRIBUTION_ROOT_PACKAGE");
    println!("cargo:rerun-if-env-changed=NUX_CAPI_UPDATE_HEADER");
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=src");

    let revision = std::env::var("NUX_RUNTIME_SOURCE_REVISION")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(git_revision)
        .unwrap_or_else(|| "unknown".to_owned());

    println!("cargo:rustc-env=NUX_RUNTIME_SOURCE_REVISION={revision}");
    emit_build_provenance(&revision);

    verify_generated_header();
}

fn emit_build_provenance(revision: &str) {
    let distribution_root = std::env::var("NUX_RUNTIME_DISTRIBUTION_ROOT_PACKAGE")
        .unwrap_or_else(|_| "nux-capi".to_owned());
    if distribution_root != "nux-capi" {
        println!("cargo:rustc-env=NUX_CAPI_BUILD_PROVENANCE=dependency-of:{distribution_root}");
        return;
    }
    let required =
        |name: &str| std::env::var(name).unwrap_or_else(|_| format!("unqualified:{name}"));
    let target = required("TARGET");
    let profile =
        std::env::var("NUX_RUNTIME_BUILD_PROFILE").unwrap_or_else(|_| required("PROFILE"));
    let build_inputs_hash = required("NUX_RUNTIME_BUILD_INPUTS_HASH");
    let contract_fingerprint = required("NUX_RUNTIME_CONTRACT_FINGERPRINT");
    let rustc = required("NUX_RUNTIME_RUSTC_VERSION");
    let features = [
        (
            "apple-metal",
            std::env::var_os("CARGO_FEATURE_APPLE_METAL").is_some(),
        ),
        (
            "scripting",
            std::env::var_os("CARGO_FEATURE_SCRIPTING").is_some(),
        ),
    ]
    .into_iter()
    .filter_map(|(name, enabled)| enabled.then_some(name))
    .collect::<Vec<_>>()
    .join(",");
    let provenance = format!(
        "{{\"schemaVersion\":6,\"rootPackage\":\"nux-capi\",\"runtimeVersion\":\"{}\",\"buildSourceRevision\":\"{}\",\"target\":\"{}\",\"profile\":\"{}\",\"features\":\"{}\",\"rustc\":\"{}\",\"buildInputsHash\":\"{}\",\"contractFingerprint\":\"{}\"}}",
        std::env::var("CARGO_PKG_VERSION").unwrap_or_default(),
        revision,
        target,
        profile,
        features,
        rustc.replace('"', ""),
        build_inputs_hash,
        contract_fingerprint,
    );
    println!("cargo:rustc-env=NUX_CAPI_BUILD_PROVENANCE={provenance}");
}

fn verify_generated_header() {
    let crate_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let config = cbindgen::Config::from_file(crate_dir.join("cbindgen.toml"))
        .expect("read crates/nux-capi/cbindgen.toml");
    let bindings = cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config)
        .generate()
        .expect("generate nux-capi C header");
    let generated_path =
        PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR")).join("nux_capi.generated.h");
    bindings.write_to_file(&generated_path);

    let committed_path = crate_dir.join("include/nux_capi.generated.h");
    if std::env::var_os("NUX_CAPI_UPDATE_HEADER").is_some() {
        std::fs::copy(&generated_path, &committed_path).expect("update committed generated header");
        return;
    }

    let generated = std::fs::read(&generated_path).expect("read generated header");
    let committed = std::fs::read(&committed_path).unwrap_or_default();
    assert_eq!(
        committed, generated,
        "generated C header is stale; run NUX_CAPI_UPDATE_HEADER=1 cargo build -p nux-capi"
    );
}

fn git_revision() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
