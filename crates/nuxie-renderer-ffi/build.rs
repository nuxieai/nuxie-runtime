use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi.cpp");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi.h");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi_private.hpp");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi_metal.mm");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi_dawn.cpp");
    println!("cargo:rerun-if-env-changed=RIVE_RUNTIME_DIR");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");

    if env::var_os("CARGO_FEATURE_NATIVE").is_none() {
        return;
    }

    let target_is_macos = env::var("CARGO_CFG_TARGET_OS")
        .map(|target_os| target_os == "macos")
        .unwrap_or(false);
    let has_dawn = env::var_os("CARGO_FEATURE_DAWN").is_some();

    let runtime_dir = env::var("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));

    let profile = env::var("PROFILE").unwrap_or_else(|_| String::from("debug"));
    if has_dawn && (!target_is_macos || profile != "release") {
        panic!("the Dawn renderer bridge requires a release build on macOS");
    }
    let renderer_out_dir = if has_dawn {
        runtime_dir
            .join("renderer")
            .join("out")
            .join("cpp-atlas-mask-oracle")
    } else {
        runtime_dir.join("renderer").join("out").join(&profile)
    };
    let root_lib_dir = if has_dawn {
        renderer_out_dir.clone()
    } else {
        runtime_dir.join("out").join(&profile)
    };
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

    let generated_include_dir = renderer_out_dir.join("include");
    let renderer_lib = renderer_out_dir.join("librive_pls_renderer.a");
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
        if has_dawn {
            build
                .define("RIVE_DESKTOP_GL", None)
                .define("RIVE_DAWN", None);
        } else {
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
        }
        build.define("YOGA_EXPORT", Some(""));
        if profile == "debug" {
            build.define("DEBUG", None);
        } else {
            build.define("RELEASE", None).define("NDEBUG", None);
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

    let dawn_dir = runtime_dir.join("renderer/dependencies/dawn");
    let dawn_out_dir = dawn_dir.join("out/release");
    if has_dawn {
        build
            .file("cpp/rive_renderer_ffi_dawn.cpp")
            .include(dawn_dir.join("include"))
            .include(dawn_out_dir.join("gen/include"))
            .define("RIVE_FFI_HAS_DAWN", None);
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
    if env::var_os("CARGO_FEATURE_DECODE_ORACLE").is_some() {
        build.define("RIVE_FFI_DECODE_ORACLE", None);
    }

    let clang_runtime_dir = has_dawn.then(|| {
        let compiler = build.get_compiler();
        let output = Command::new(compiler.path())
            .arg("-print-resource-dir")
            .output()
            .expect("failed to query the C++ compiler resource directory");
        if !output.status.success() {
            panic!("the C++ compiler did not report its resource directory");
        }
        let directory = PathBuf::from(
            String::from_utf8(output.stdout)
                .expect("the C++ compiler resource directory was not UTF-8")
                .trim(),
        )
        .join("lib/darwin");
        let archive = directory.join("libclang_rt.osx.a");
        if !archive.exists() {
            panic!("missing compiler runtime archive {}", archive.display());
        }
        directory
    });

    build.compile("nuxie_renderer_ffi");

    println!(
        "cargo:rustc-link-search=native={}",
        renderer_out_dir.display()
    );
    println!("cargo:rustc-link-search=native={}", root_lib_dir.display());
    println!("cargo:rustc-link-lib=static=nuxie_renderer_ffi");
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

    if has_dawn {
        for directory in [
            dawn_out_dir.join("obj/src/dawn"),
            dawn_out_dir.join("obj/src/dawn/native"),
            dawn_out_dir.join("obj/src/dawn/platform"),
        ] {
            println!("cargo:rustc-link-search=native={}", directory.display());
        }
        for lib in [
            "webgpu_dawn",
            "dawn_native_static",
            "dawn_proc_static",
            "dawn_platform_static",
        ] {
            println!("cargo:rustc-link-lib=static={lib}");
        }
        let clang_runtime_dir = clang_runtime_dir.expect("Dawn builds require the Clang runtime");
        println!(
            "cargo:rustc-link-search=native={}",
            clang_runtime_dir.display()
        );
        println!("cargo:rustc-link-lib=static=clang_rt.osx");
    }

    if target_is_macos {
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=QuartzCore");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=IOKit");
        if has_dawn {
            println!("cargo:rustc-link-lib=framework=IOSurface");
            println!("cargo:rustc-link-lib=framework=Security");
        }
    }
}
