use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=NUX_RUNTIME_SOURCE_REVISION");
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

    verify_generated_header();
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
