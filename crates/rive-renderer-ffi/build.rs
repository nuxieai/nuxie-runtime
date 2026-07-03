use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi.cpp");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi.h");

    if env::var_os("CARGO_FEATURE_NATIVE").is_none() {
        return;
    }

    let runtime_dir = env::var("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));

    let profile = env::var("PROFILE").unwrap_or_else(|_| String::from("debug"));
    let renderer_lib = runtime_dir
        .join("renderer")
        .join("out")
        .join(&profile)
        .join("librive_pls_renderer.a");
    if !renderer_lib.exists() {
        panic!(
            "missing {}; build the C++ renderer first, e.g. `cd {}/renderer && premake5 gmake2 && make config={}`",
            renderer_lib.display(),
            runtime_dir.display(),
            profile
        );
    }

    cc::Build::new()
        .cpp(true)
        .std("c++17")
        .file("cpp/rive_renderer_ffi.cpp")
        .file(runtime_dir.join("tests/common/render_context_null.cpp"))
        .include("cpp")
        .include(runtime_dir.join("include"))
        .include(runtime_dir.join("renderer/include"))
        .include(runtime_dir.join("renderer/src"))
        .include(runtime_dir.join("renderer/glad"))
        .include(runtime_dir.join("renderer/glad/include"))
        .include(runtime_dir.join("decoders/include"))
        .include(runtime_dir.join("tests/common"))
        .define("_RIVE_INTERNAL_", None)
        .define("TESTING", None)
        .flag_if_supported("-Wno-shorten-64-to-32")
        .compile("rive_renderer_ffi");

    println!(
        "cargo:rustc-link-search=native={}",
        runtime_dir.join("renderer/out").join(&profile).display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        runtime_dir.join("out").join(&profile).display()
    );
    println!("cargo:rustc-link-lib=static=rive_renderer_ffi");
    println!("cargo:rustc-link-lib=static=rive_pls_renderer");
    println!("cargo:rustc-link-lib=static=rive");

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=QuartzCore");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=IOKit");
    }
}
