use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=NUX_RUNTIME_BUILD_PROFILE");
    println!("cargo:rerun-if-env-changed=NUX_RUNTIME_SOURCE_REVISION");
    println!("cargo:rerun-if-env-changed=NUX_RUNTIME_UPDATE_HEADER");
    println!("cargo:rerun-if-changed=cbindgen.toml");
    println!("cargo:rerun-if-changed=src");

    let revision = std::env::var("NUX_RUNTIME_SOURCE_REVISION")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(git_revision)
        .unwrap_or_else(|| "unknown".to_owned());
    let rustc = command_output(
        std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned()),
        &["--version"],
    )
    .unwrap_or_else(|| "unknown".to_owned());
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_owned());
    let profile = std::env::var("NUX_RUNTIME_BUILD_PROFILE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| std::env::var("PROFILE").ok())
        .unwrap_or_else(|| "unknown".to_owned());
    let features = if std::env::var_os("CARGO_FEATURE_APPLE_PRODUCT").is_some() {
        "apple-product"
    } else {
        "none"
    };
    let luaur_version = if std::env::var_os("CARGO_FEATURE_APPLE_PRODUCT").is_some() {
        "\"0.1.8\""
    } else {
        "null"
    };
    let provenance = format!(
        concat!(
            "{{\"schemaVersion\":1,\"runtimeVersion\":\"{}\",",
            "\"runtimeAbiMajor\":1,\"runtimeAbiMinor\":4,",
            "\"flowSessionAbiMinor\":4,",
            "\"sourceRevision\":\"{}\",\"target\":\"{}\",",
            "\"profile\":\"{}\",\"rustc\":\"{}\",",
            "\"features\":\"{}\",\"wgpuVersion\":\"30.0.0\",",
            "\"luaurVersion\":{}}}"
        ),
        env!("CARGO_PKG_VERSION"),
        json_escape(&revision),
        json_escape(&target),
        json_escape(&profile),
        json_escape(&rustc),
        features,
        luaur_version,
    );
    println!("cargo:rustc-env=NUX_RUNTIME_BUILD_PROVENANCE={provenance}");

    verify_generated_header();
}

fn git_revision() -> Option<String> {
    command_output("git".to_owned(), &["rev-parse", "--verify", "HEAD"])
}

fn command_output(command: String, arguments: &[&str]) -> Option<String> {
    let output = Command::new(command).args(arguments).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn verify_generated_header() {
    let crate_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let config = cbindgen::Config::from_file(crate_dir.join("cbindgen.toml"))
        .expect("read nux-apple-runtime cbindgen config");
    let bindings = cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config)
        .generate()
        .expect("generate Nuxie runtime C header");
    let generated_path = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"))
        .join("nux_runtime.generated.h");
    bindings.write_to_file(&generated_path);

    let committed_path = crate_dir.join("include/nux_runtime.generated.h");
    if std::env::var_os("NUX_RUNTIME_UPDATE_HEADER").is_some() {
        std::fs::copy(&generated_path, &committed_path).expect("update generated runtime header");
        return;
    }
    assert_eq!(
        std::fs::read(&committed_path).unwrap_or_default(),
        std::fs::read(&generated_path).expect("read generated runtime header"),
        "generated runtime header is stale; run NUX_RUNTIME_UPDATE_HEADER=1 cargo build -p nux-apple-runtime --features apple-product"
    );
}
