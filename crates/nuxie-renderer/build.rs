use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};

fn main() {
    println!("cargo:rerun-if-changed=src/native_metal/tracer.metal");
    for source in DRAW_SHADER_SOURCES {
        println!("cargo:rerun-if-changed={source}");
    }
    for source in RESOURCE_SHADER_SOURCES {
        println!("cargo:rerun-if-changed={source}");
    }
    if env::var_os("CARGO_FEATURE_NATIVE_METAL_EXPERIMENTAL").is_none() {
        return;
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Cargo provides target OS");
    if target_os != "ios" && target_os != "macos" {
        return;
    }

    let (sdk, deployment_target, metal_standard) = match (
        target_os.as_str(),
        env::var("CARGO_CFG_TARGET_ABI").as_deref(),
    ) {
        ("ios", Ok("sim")) => (
            "iphonesimulator",
            "-miphonesimulator-version-min=15.0",
            "-std=ios-metal2.2",
        ),
        ("ios", _) => ("iphoneos", "-mios-version-min=15.0", "-std=ios-metal2.2"),
        ("macos", _) => ("macosx", "-mmacosx-version-min=12.0", "-std=macos-metal2.3"),
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

    let draw_source = manifest.join("src/native_metal/shaders/draw.metal");
    let draw_include_dir = manifest.join("src/native_metal/shaders");
    let draw_air = output.join("native_metal_draw.air");
    let draw_metallib = output.join("native_metal_draw.metallib");

    checked(
        Command::new("xcrun")
            .args(["-sdk", sdk, "metal", "-c"])
            .arg(deployment_target)
            .arg(metal_standard)
            .arg("-I")
            .arg(&draw_include_dir)
            .args([
                "-ffast-math",
                "-ffp-contract=fast",
                "-fpreserve-invariance",
                "-fvisibility=hidden",
            ])
            .arg(&draw_source)
            .arg("-o")
            .arg(&draw_air)
            .output(),
        "compile native Metal offline draw shader",
    );
    checked(
        Command::new("xcrun")
            .args(["-sdk", sdk, "metallib"])
            .arg(&draw_air)
            .arg("-o")
            .arg(&draw_metallib)
            .output(),
        "link native Metal offline draw shader library",
    );

    let resource_sources = [
        (
            manifest.join("src/native_metal/shaders/color_ramp.metal"),
            output.join("native_metal_color_ramp.air"),
        ),
        (
            manifest.join("src/native_metal/shaders/tessellate.metal"),
            output.join("native_metal_tessellate.air"),
        ),
    ];
    let resource_metallib = output.join("native_metal_resources.metallib");
    for (source, air) in &resource_sources {
        checked(
            Command::new("xcrun")
                .args(["-sdk", sdk, "metal", "-c"])
                .arg(deployment_target)
                .arg(metal_standard)
                .arg("-I")
                .arg(&draw_include_dir)
                .args([
                    "-ffast-math",
                    "-ffp-contract=fast",
                    "-fpreserve-invariance",
                    "-fvisibility=hidden",
                ])
                .arg(source)
                .arg("-o")
                .arg(air)
                .output(),
            "compile native Metal color-ramp/tessellate resource shader",
        );
    }
    let mut resource_link = Command::new("xcrun");
    resource_link
        .args(["-sdk", sdk, "metallib"])
        .args(resource_sources.iter().map(|(_, air)| air))
        .arg("-o")
        .arg(&resource_metallib);
    checked(
        resource_link.output(),
        "link native Metal color-ramp/tessellate resource shader library",
    );
}

const DRAW_SHADER_SOURCES: &[&str] = &[
    "src/native_metal/shaders/draw.metal",
    "src/native_metal/shaders/metal.minified.glsl",
    "src/native_metal/shaders/constants.minified.glsl",
    "src/native_metal/shaders/flush_uniforms.minified.glsl",
    "src/native_metal/shaders/common.minified.glsl",
    "src/native_metal/shaders/draw_path_common.minified.glsl",
    "src/native_metal/shaders/render_atlas.minified.glsl",
    "src/native_metal/shaders/advanced_blend.minified.glsl",
    "src/native_metal/shaders/draw_combinations.metal",
    "src/native_metal/shaders/draw_path.minified.vert",
    "src/native_metal/shaders/draw_raster_order_path.minified.frag",
    "src/native_metal/shaders/draw_mesh.minified.frag",
    "src/native_metal/shaders/draw_image_mesh.minified.vert",
];

const RESOURCE_SHADER_SOURCES: &[&str] = &[
    "src/native_metal/shaders/color_ramp.metal",
    "src/native_metal/shaders/tessellate.metal",
    "src/native_metal/shaders/metal.minified.glsl",
    "src/native_metal/shaders/constants.minified.glsl",
    "src/native_metal/shaders/flush_uniforms.minified.glsl",
    "src/native_metal/shaders/common.minified.glsl",
    "src/native_metal/shaders/color_ramp.minified.glsl",
    "src/native_metal/shaders/bezier_utils.minified.glsl",
    "src/native_metal/shaders/tessellate.minified.glsl",
];

fn checked(output: std::io::Result<Output>, operation: &str) {
    let output = output.unwrap_or_else(|error| panic!("{operation}: {error}"));
    if !output.status.success() {
        panic!(
            "{operation} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
