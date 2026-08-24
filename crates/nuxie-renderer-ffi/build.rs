use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi.cpp");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi.h");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi_private.hpp");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi_metal.mm");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi_dawn.cpp");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi_vulkan.cpp");
    println!("cargo:rerun-if-changed=cpp/rive_renderer_ffi_webgl2.cpp");
    println!("cargo:rerun-if-env-changed=RIVE_RUNTIME_DIR");
    println!("cargo:rerun-if-env-changed=RIVE_RENDERER_OUT_DIR");
    println!("cargo:rerun-if-env-changed=EMSDK");
    println!("cargo:rerun-if-env-changed=EMDAWNWEBGPU_PORT");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");

    if env::var_os("CARGO_FEATURE_NATIVE").is_none() {
        return;
    }

    let target_is_macos = env::var("CARGO_CFG_TARGET_OS")
        .map(|target_os| target_os == "macos")
        .unwrap_or(false);
    let target_is_emscripten = env::var("CARGO_CFG_TARGET_OS")
        .map(|target_os| target_os == "emscripten")
        .unwrap_or(false);
    let has_dawn = env::var_os("CARGO_FEATURE_DAWN").is_some();
    let has_vulkan = env::var_os("CARGO_FEATURE_VULKAN").is_some();
    let has_webgl2 = env::var_os("CARGO_FEATURE_WEBGL2").is_some();
    let has_perf_counters = env::var_os("CARGO_FEATURE_PERF_COUNTERS").is_some();

    if [has_dawn, has_vulkan, has_webgl2]
        .into_iter()
        .filter(|enabled| *enabled)
        .count()
        > 1
    {
        panic!("the Dawn, Vulkan, and WebGL2 source-oracle bridges are separate build roots");
    }

    let runtime_dir = env::var("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));

    let profile = env::var("PROFILE").unwrap_or_else(|_| String::from("debug"));
    if has_dawn && (!(target_is_macos || target_is_emscripten) || profile != "release") {
        panic!("the Dawn renderer bridge requires a release build on macOS or Emscripten");
    }
    if has_vulkan && (!target_is_macos || profile != "release") {
        panic!("the Vulkan renderer bridge requires a release build on macOS");
    }
    if has_webgl2 && (!target_is_emscripten || profile != "release") {
        panic!("the WebGL2 renderer bridge requires a release wasm32-unknown-emscripten build");
    }
    let renderer_out_dir = if let Some(path) = env::var_os("RIVE_RENDERER_OUT_DIR") {
        PathBuf::from(path)
    } else if has_dawn {
        runtime_dir
            .join("renderer")
            .join("out")
            .join("cpp-atlas-mask-oracle")
    } else if has_webgl2 {
        runtime_dir
            .join("renderer")
            .join("out")
            .join("cpp-webgl2-oracle")
    } else {
        let unified = runtime_dir.join("tests").join("out").join(&profile);
        if unified.join("librive_pls_renderer.a").exists() {
            unified
        } else {
            runtime_dir.join("renderer").join("out").join(&profile)
        }
    };
    let root_lib_dir = if renderer_out_dir.join("librive.a").exists() {
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
    let renderer_makefile = renderer_out_dir.join("rive_pls_renderer.make");
    println!("cargo:rerun-if-changed={}", renderer_lib.display());
    println!("cargo:rerun-if-changed={}", renderer_makefile.display());
    let has_metal_backend = target_is_macos && renderer_lib.exists();
    let renderer_with_rive_tools = fs::read_to_string(&renderer_makefile)
        .map(|contents| contents.contains("-DWITH_RIVE_TOOLS"))
        .unwrap_or(false);

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
            let defines = if target_is_emscripten {
                [
                    "ORE_BACKEND_WGPU",
                    "RIVE_WEBGL",
                    "ORE_BACKEND_GL",
                    "RIVE_ORE",
                    "WITH_RIVE_TOOLS",
                    "WITH_RIVE_TEXT",
                    "RIVE_CANVAS",
                    "WITH_RIVE_LAYOUT",
                    "RIVE_DECODERS",
                    "RIVE_KTX2",
                ]
            } else {
                [
                    "RIVE_DESKTOP_GL",
                    "RIVE_DAWN",
                    "ORE_BACKEND_METAL",
                    "ORE_BACKEND_GL",
                    "ORE_BACKEND_WGPU",
                    "RIVE_ORE",
                    "WITH_RIVE_TEXT",
                    "RIVE_CANVAS",
                    "WITH_RIVE_LAYOUT",
                    "RIVE_DECODERS",
                ]
            };
            for define in defines {
                build.define(define, None);
            }
            if target_is_emscripten {
                build.define("RIVE_WEBGPU", Some("2"));
            } else {
                build.define("RIVE_KTX2", None);
            }
        } else if has_vulkan {
            for define in [
                "RIVE_VULKAN",
                "VK_NO_PROTOTYPES",
                "VMA_STATIC_VULKAN_FUNCTIONS=0",
                "VMA_DYNAMIC_VULKAN_FUNCTIONS=1",
                "RIVE_DESKTOP_GL",
                "RIVE_DECODERS",
                "RIVE_KTX2",
            ] {
                build.define(define, None);
            }
        } else if has_webgl2 {
            for define in [
                "RIVE_WEBGL",
                "ORE_BACKEND_GL",
                "RIVE_ORE",
                "WITH_RIVE_TEXT",
                "RIVE_CANVAS",
                "WITH_RIVE_LAYOUT",
                "RIVE_DECODERS",
                "RIVE_KTX2",
            ] {
                build.define(define, None);
            }
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
            // This changes RenderContextMetalImpl's layout, so the bridge must
            // mirror the archive before invoking inline methods such as
            // setCommandQueue(). Current tests/out builds enable it; older
            // standalone renderer builds may not.
            if renderer_with_rive_tools {
                build.define("WITH_RIVE_TOOLS", None);
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
        if target_is_emscripten {
            let emsdk = env::var("EMSDK")
                .map(PathBuf::from)
                .expect("browser Dawn builds require the pinned EMSDK path");
            let compiler = emsdk.join("upstream/emscripten/em++");
            if !compiler.exists() {
                panic!("missing pinned Emscripten compiler {}", compiler.display());
            }
            let port = env::var("EMDAWNWEBGPU_PORT")
                .map(PathBuf::from)
                .expect("browser Dawn builds require the pinned Emdawnwebgpu port");
            if !port.exists() {
                panic!("missing pinned Emdawnwebgpu port {}", port.display());
            }
            build
                .compiler(compiler)
                .flag("-msimd128")
                .flag(&format!("--use-port={}", port.display()));
        }
        build
            .file("cpp/rive_renderer_ffi_dawn.cpp")
            .define("RIVE_FFI_HAS_DAWN", None);
        if target_is_macos {
            build
                .include(dawn_dir.join("include"))
                .include(dawn_out_dir.join("gen/include"));
        }
        if has_perf_counters {
            build.define("RIVE_FFI_PERF_COUNTERS", None);
        }
    }

    if has_vulkan {
        let vulkan_headers =
            runtime_dir.join("dependencies/KhronosGroup_Vulkan-Headers_vulkan-sdk-1.4.321/include");
        let vulkan_memory_allocator = runtime_dir
            .join("dependencies/GPUOpen-LibrariesAndSDKs_VulkanMemoryAllocator_v3.3.0/include");
        let bootstrap_dir = runtime_dir.join("renderer/rive_vk_bootstrap");
        let bootstrap_sources = [
            "vulkan_debug_callbacks.cpp",
            "vulkan_device.cpp",
            "vulkan_frame_synchronizer.cpp",
            "vulkan_headless_frame_synchronizer.cpp",
            "vulkan_instance.cpp",
            "vulkan_library.cpp",
            "vulkan_swapchain.cpp",
        ];
        build
            .file("cpp/rive_renderer_ffi_vulkan.cpp")
            .include(bootstrap_dir.join("include"))
            .include(bootstrap_dir.join("src"))
            .include(vulkan_headers)
            .include(vulkan_memory_allocator)
            .define("RIVE_FFI_HAS_VULKAN", None)
            .files(bootstrap_sources.map(|source| bootstrap_dir.join("src").join(source)));
    }

    if has_webgl2 {
        let emscripten_dir =
            runtime_dir.join("build/dependencies/emsdk_3.1.61/upstream/emscripten");
        let compiler = emscripten_dir.join("em++");
        if !compiler.exists() {
            panic!("missing pinned Emscripten compiler {}", compiler.display());
        }
        build
            .compiler(compiler)
            .file("cpp/rive_renderer_ffi_webgl2.cpp")
            .define("RIVE_FFI_HAS_WEBGL2", None)
            .flag("-msimd128");
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

    let clang_runtime_dir = (has_dawn && target_is_macos).then(|| {
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

    if has_dawn && target_is_macos {
        for directory in [
            dawn_out_dir.join("obj/src/dawn"),
            dawn_out_dir.join("obj/src/dawn/native"),
            dawn_out_dir.join("obj/src/dawn/platform"),
        ] {
            println!("cargo:rustc-link-search=native={}", directory.display());
        }
        let dawn_libs = if has_perf_counters {
            [
                "dawn_proc_static",
                "webgpu_dawn",
                "dawn_native_static",
                "dawn_platform_static",
            ]
        } else {
            [
                "webgpu_dawn",
                "dawn_native_static",
                "dawn_proc_static",
                "dawn_platform_static",
            ]
        };
        for lib in dawn_libs {
            println!("cargo:rustc-link-lib=static={lib}");
        }
        let clang_runtime_dir = clang_runtime_dir.expect("Dawn builds require the Clang runtime");
        println!(
            "cargo:rustc-link-search=native={}",
            clang_runtime_dir.display()
        );
        println!("cargo:rustc-link-lib=static=clang_rt.osx");
    }
    if has_dawn && target_is_emscripten {
        let port = env::var("EMDAWNWEBGPU_PORT")
            .map(PathBuf::from)
            .expect("browser Dawn builds require the pinned Emdawnwebgpu port");
        println!("cargo:rustc-link-arg=--use-port={}", port.display());
    }

    if target_is_macos {
        println!("cargo:rustc-link-lib=framework=Metal");
        println!("cargo:rustc-link-lib=framework=QuartzCore");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=IOKit");
        if has_dawn || has_vulkan {
            println!("cargo:rustc-link-lib=framework=IOSurface");
        }
        if has_dawn {
            println!("cargo:rustc-link-lib=framework=Security");
        }
        if has_vulkan {
            let moltenvk_dir = runtime_dir.join(
                "renderer/dependencies/MoltenVK/Package/Release/MoltenVK/dynamic/dylib/macOS",
            );
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", moltenvk_dir.display());
        }
    }
}
