fn main() {
    for name in [
        "NUX_RUNTIME_SOURCE_REVISION",
        "NUX_RUNTIME_BUILD_INPUTS_HASH",
        "NUX_RUNTIME_CONTRACT_FINGERPRINT",
        "NUX_RUNTIME_BUILD_PROFILE",
        "NUX_RUNTIME_RUSTC_VERSION",
    ] {
        println!("cargo:rerun-if-env-changed={name}");
    }

    let required =
        |name: &str| std::env::var(name).unwrap_or_else(|_| format!("unqualified:{name}"));
    let feature = std::env::var_os("CARGO_FEATURE_APPLE_RUNTIME")
        .is_some()
        .then_some("apple-runtime")
        .unwrap_or_default();
    let profile =
        std::env::var("NUX_RUNTIME_BUILD_PROFILE").unwrap_or_else(|_| required("PROFILE"));
    let provenance = format!(
        r#"{{"schemaVersion":6,"rootPackage":"nux-apple-product-extension","runtimeVersion":"{}","buildSourceRevision":"{}","target":"{}","profile":"{}","features":"{}","rustc":"{}","buildInputsHash":"{}","contractFingerprint":"{}"}}"#,
        required("CARGO_PKG_VERSION"),
        required("NUX_RUNTIME_SOURCE_REVISION"),
        required("TARGET"),
        profile,
        feature,
        required("NUX_RUNTIME_RUSTC_VERSION").replace('"', ""),
        required("NUX_RUNTIME_BUILD_INPUTS_HASH"),
        required("NUX_RUNTIME_CONTRACT_FINGERPRINT"),
    );
    println!("cargo:rustc-env=NUX_APPLE_PRODUCT_EXTENSION_BUILD_PROVENANCE={provenance}");
}
