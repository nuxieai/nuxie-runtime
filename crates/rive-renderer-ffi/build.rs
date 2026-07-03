use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi.cpp");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi.h");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi_private.hpp");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi_metal.mm");
    println!("cargo:rerun-if-env-changed=RIVE_RUNTIME_DIR");

    if env::var_os("CARGO_FEATURE_NATIVE").is_none() {
        return;
    }

    let target_is_macos = env::var("CARGO_CFG_TARGET_OS")
        .map(|target_os| target_os == "macos")
        .unwrap_or(false);

    let runtime_dir = env::var("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));

    let profile = env::var("PROFILE").unwrap_or_else(|_| String::from("debug"));
    let root_lib_dir = runtime_dir.join("out").join(&profile);
    let root_lib = root_lib_dir.join("librive.a");
    println!("cargo:rerun-if-changed={}", root_lib.display());
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
    println!("cargo:rerun-if-changed={}", renderer_lib.display());
    let has_metal_backend = target_is_macos && renderer_lib.exists();

    let renderer_static_libs = [
        ("rive_pls_renderer", renderer_lib.clone()),
        ("rive_decoders", renderer_out_dir.join("librive_decoders.a")),
        ("libwebp", renderer_out_dir.join("liblibwebp.a")),
        ("libpng", renderer_out_dir.join("liblibpng.a")),
        ("zlib", renderer_out_dir.join("libzlib.a")),
        ("libjpeg", renderer_out_dir.join("liblibjpeg.a")),
        ("rive_harfbuzz", renderer_out_dir.join("librive_harfbuzz.a")),
        (
            "rive_sheenbidi",
            renderer_out_dir.join("librive_sheenbidi.a"),
        ),
        ("rive_yoga", renderer_out_dir.join("librive_yoga.a")),
    ];
    for (_, archive) in &renderer_static_libs {
        println!("cargo:rerun-if-changed={}", archive.display());
    }

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++17")
        .file("cpp/rive_renderer_ffi.cpp")
        .file(runtime_dir.join("tests/common/render_context_null.cpp"))
        .include("cpp")
        .include(runtime_dir.join("dependencies"))
        .include(
            runtime_dir
                .join("dependencies")
                .join("rive-app_harfbuzz_rive_13.1.1/src"),
        )
        .include(
            runtime_dir
                .join("dependencies")
                .join("Tehreer_SheenBidi_v2.6/Headers"),
        )
        .include(
            runtime_dir
                .join("dependencies")
                .join("rive-app_miniaudio_rive_changes_5"),
        )
        .include(
            runtime_dir
                .join("dependencies")
                .join("rive-app_yoga_rive_changes_v2_0_1_2"),
        )
        .include(runtime_dir.join("include"))
        .include(runtime_dir.join("renderer/dependencies"))
        .include(runtime_dir.join("renderer/include"))
        .include(runtime_dir.join("renderer/src"))
        .include(runtime_dir.join("renderer/glad"))
        .include(runtime_dir.join("renderer/glad/include"))
        .include(runtime_dir.join("decoders/include"))
        .include(runtime_dir.join("tests/common"))
        .define("_RIVE_INTERNAL_", None)
        .define("TESTING", None)
        .flag_if_supported("-fno-exceptions")
        .flag_if_supported("-fno-rtti")
        .flag_if_supported("-Wno-shorten-64-to-32");

    if renderer_lib.exists() {
        for define in [
            "RIVE_DESKTOP_GL",
            "RIVE_ORE",
            "ORE_BACKEND_METAL",
            "ORE_BACKEND_GL",
            "WITH_RIVE_TEXT",
            "RIVE_CANVAS",
            "WITH_RIVE_LAYOUT",
            "RIVE_DECODERS",
            "RIVE_KTX2",
        ] {
            build.define(define, None);
        }
        build.define("YOGA_EXPORT", Some(""));
        if profile == "debug" {
            build.define("DEBUG", None);
        }
        if target_is_macos {
            build.define("RIVE_MACOSX", None);
        }
    }

    if has_metal_backend {
        build
            .file("cpp/rive_renderer_ffi_metal.mm")
            .define("RIVE_FFI_HAS_METAL", None)
            .flag_if_supported("-fobjc-arc");
    }

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
    } else {
        let missing_archives = renderer_static_libs
            .iter()
            .filter_map(|(_, archive)| (!archive.exists()).then(|| archive.display().to_string()))
            .collect::<Vec<_>>();
        if !missing_archives.is_empty() {
            panic!(
                "missing renderer dependency archives:\n{}\nbuild them with `cd {}/renderer && PATH=\"{}/build:$PATH\" build_rive.sh {} -- rive_decoders libpng zlib libjpeg libwebp rive_harfbuzz rive_sheenbidi rive_yoga`",
                missing_archives.join("\n"),
                runtime_dir.display(),
                runtime_dir.display(),
                profile,
            );
        }
    }

    build.compile("rive_renderer_ffi");

    println!(
        "cargo:rustc-link-search=native={}",
        renderer_out_dir.display()
    );
    println!("cargo:rustc-link-search=native={}", root_lib_dir.display());
    println!("cargo:rustc-link-lib=static=rive_renderer_ffi");
    if renderer_lib.exists() {
        for lib in [
            "rive_pls_renderer",
            "rive_decoders",
            "libwebp",
            "libpng",
            "zlib",
            "libjpeg",
        ] {
            println!("cargo:rustc-link-lib=static={lib}");
        }
    }
    println!("cargo:rustc-link-lib=static=rive");
    if renderer_lib.exists() {
        for lib in ["rive_harfbuzz", "rive_sheenbidi", "rive_yoga"] {
            println!("cargo:rustc-link-lib=static={lib}");
        }
    }

    if target_is_macos {
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=QuartzCore");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=IOKit");
    }
}
