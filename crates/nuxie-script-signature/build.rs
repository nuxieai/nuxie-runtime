use std::{env, fs, path::Path, process::Command};

use sha2::{Digest, Sha256};

fn main() {
    println!("cargo:rerun-if-changed=c");
    for line in fs::read_to_string("c/upstream.sha256")
        .expect("vendored source manifest")
        .lines()
    {
        let (expected, path) = line.split_once("  ").expect("SHA256 manifest row");
        let source =
            fs::read(Path::new("c/upstream").join(path)).expect("pinned libhydrogen source");
        assert_eq!(
            format!("{:x}", Sha256::digest(source)),
            expected,
            "modified libhydrogen source: {path}"
        );
    }

    let wasm = env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32")
        && env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("unknown");
    if !wasm && env::var_os("CARGO_FEATURE_FREESTANDING_TESTS").is_none() {
        return;
    }

    let mut build = cc::Build::new();
    build
        .file("c/verify.c")
        .opt_level(3)
        .flag_if_supported("-ffunction-sections");
    if wasm {
        // Clang supplies its freestanding integer/size headers. The local
        // headers only declare the C memory/trap functions; no libc, WASI,
        // Emscripten, random provider, or host imports are linked here.
        build.flag("-ffreestanding").include("c/freestanding");
        // Apple's clang deliberately omits the WebAssembly backend. Respect
        // cc-rs's explicit compiler choices; otherwise use the LLVM toolchain
        // advertised on PATH (or by LLVM_CONFIG_PATH), without a host sysroot.
        println!("cargo:rerun-if-env-changed=LLVM_CONFIG_PATH");
        // pnpm and fnm add per-invocation PATH entries unrelated to LLVM.
        // Track explicit toolchain selections and the selected tool files,
        // not the ambient search path, so they do not invalidate every build.
        for kind in ["CC", "AR"] {
            for key in [
                format!("{kind}_wasm32-unknown-unknown"),
                format!("{kind}_wasm32_unknown_unknown"),
                format!("TARGET_{kind}"),
                kind.to_owned(),
            ] {
                println!("cargo:rerun-if-env-changed={key}");
            }
        }
        let llvm_config = env::var_os("LLVM_CONFIG_PATH").unwrap_or_else(|| "llvm-config".into());
        if let Ok(output) = Command::new(llvm_config).arg("--bindir").output() {
            if output.status.success() {
                let directory = String::from_utf8(output.stdout).expect("LLVM bindir is UTF-8");
                let directory = Path::new(directory.trim());
                let explicit = |kind: &str| {
                    [
                        format!("{kind}_wasm32-unknown-unknown"),
                        format!("{kind}_wasm32_unknown_unknown"),
                        format!("TARGET_{kind}"),
                        kind.to_owned(),
                    ]
                    .iter()
                    .any(|key| env::var_os(key).is_some())
                };
                if !explicit("CC") {
                    let compiler = directory.join("clang");
                    println!("cargo:rerun-if-changed={}", compiler.display());
                    build.compiler(compiler);
                }
                if !explicit("AR") {
                    let archiver = directory.join("llvm-ar");
                    println!("cargo:rerun-if-changed={}", archiver.display());
                    build.archiver(archiver);
                }
            }
        }
    }
    build.compile("nuxie_script_signature_verify");
}
