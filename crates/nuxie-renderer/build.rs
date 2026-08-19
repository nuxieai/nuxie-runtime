use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};

fn main() {
    println!("cargo:rerun-if-changed=src/native_metal/tracer.metal");
    if env::var_os("CARGO_FEATURE_NATIVE_METAL_EXPERIMENTAL").is_none() {
        return;
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Cargo provides target OS");
    if target_os != "ios" && target_os != "macos" {
        return;
    }

    let (sdk, deployment_target) = match (
        target_os.as_str(),
        env::var("CARGO_CFG_TARGET_ABI").as_deref(),
    ) {
        ("ios", Ok("sim")) => ("iphonesimulator", "-mios-version-min=15.0"),
        ("ios", _) => ("iphoneos", "-mios-version-min=15.0"),
        ("macos", _) => ("macosx", "-mmacosx-version-min=12.0"),
        _ => unreachable!(),
    };
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let source = manifest.join("src/native_metal/tracer.metal");
    let air = output.join("native_metal_tracer.air");
    let metallib = output.join("native_metal_tracer.metallib");

    checked(
        Command::new("xcrun")
            .args(["-sdk", sdk, "metal", "-c"])
            .arg(deployment_target)
            .arg(&source)
            .arg("-o")
            .arg(&air)
            .output(),
        "compile native Metal tracer shader",
    );
    checked(
        Command::new("xcrun")
            .args(["-sdk", sdk, "metallib"])
            .arg(&air)
            .arg("-o")
            .arg(&metallib)
            .output(),
        "link native Metal tracer library",
    );
}

fn checked(output: std::io::Result<Output>, operation: &str) {
    let output = output.unwrap_or_else(|error| panic!("{operation}: {error}"));
    if !output.status.success() {
        panic!(
            "{operation} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
