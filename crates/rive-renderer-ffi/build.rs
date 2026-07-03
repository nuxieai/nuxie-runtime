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
    let root_lib_dir = runtime_dir.join("out").join(&profile);
    let root_lib = root_lib_dir.join("librive.a");
    if !root_lib.exists() {
        panic!(
            "missing {}; build the C++ runtime first, e.g. `cd {} && premake5 gmake2 && make config={}`",
            root_lib.display(),
            runtime_dir.display(),
            profile
        );
    }

    let renderer_out_dir = runtime_dir.join("renderer").join("out").join(&profile);
    let generated_include_dir = renderer_out_dir.join("include");
    let renderer_lib = runtime_dir
        .join("renderer")
        .join("out")
        .join(&profile)
        .join("librive_pls_renderer.a");

    let mut build = cc::Build::new();
    build
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
        .flag_if_supported("-fno-rtti")
        .flag_if_supported("-Wno-shorten-64-to-32");

    if !renderer_lib.exists() {
        if !generated_include_dir.exists() {
            panic!(
                "missing {} and {}; build the C++ renderer first, e.g. `cd {}/renderer && PATH=\"{}/build:$PATH\" build_rive.sh debug -- rive_pls_renderer`",
                renderer_lib.display(),
                generated_include_dir.display(),
                runtime_dir.display(),
                runtime_dir.display(),
            );
        }
        println!(
            "cargo:warning=missing {}; compiling the null renderer bridge from C++ renderer sources",
            renderer_lib.display()
        );
        build.include(generated_include_dir).files(
            [
                "draw.cpp",
                "gpu.cpp",
                "gpu_resource.cpp",
                "gr_triangulator.cpp",
                "gradient.cpp",
                "intersection_board.cpp",
                "render_context.cpp",
                "render_context_helper_impl.cpp",
                "rive_render_factory.cpp",
                "rive_render_image.cpp",
                "rive_render_paint.cpp",
                "rive_render_path.cpp",
                "rive_renderer.cpp",
                "sk_rectanizer_skyline.cpp",
            ]
            .map(|file| runtime_dir.join("renderer/src").join(file)),
        );
    }

    build.compile("rive_renderer_ffi");

    println!(
        "cargo:rustc-link-search=native={}",
        renderer_out_dir.display()
    );
    println!("cargo:rustc-link-search=native={}", root_lib_dir.display());
    println!("cargo:rustc-link-lib=static=rive_renderer_ffi");
    if renderer_lib.exists() {
        println!("cargo:rustc-link-lib=static=rive_pls_renderer");
    }
    println!("cargo:rustc-link-lib=static=rive");

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=QuartzCore");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=IOKit");
    }
}
