#[path = "src/mechanical_port/source/renderer/src/shaders/minify_py.rs"]
mod minify_py;
#[path = "src/mechanical_port/source/renderer/src/shaders/metal/generate_draw_combinations_py.rs"]
mod translated_draw_combinations;
#[path = "src/mechanical_port/source/renderer/src/shaders/makefile.rs"]
mod translated_makefile;

use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output};

fn main() {
    for source in TRANSLATED_SHADER_RUST_PATHS {
        println!("cargo:rerun-if-changed={source}");
    }
    for source in TRANSLATED_BUILD_RULE_PATHS {
        println!("cargo:rerun-if-changed={source}");
    }
    if env::var_os("CARGO_FEATURE_NATIVE_VULKAN_EXPERIMENTAL").is_some() {
        generate_vulkan_spirv_module()
            .unwrap_or_else(|error| panic!("materialize frozen Vulkan SPIR-V module: {error}"));
    }
    if env::var_os("CARGO_FEATURE_NATIVE_METAL_EXPERIMENTAL").is_none() {
        return;
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("Cargo provides target OS");
    if !matches!(target_os.as_str(), "ios" | "macos" | "tvos" | "visionos") {
        return;
    }

    let target_abi = env::var("CARGO_CFG_TARGET_ABI").unwrap_or_default();
    let (sdk, deployment_target, metal_standard, source_metallib_name, family) =
        match (target_os.as_str(), target_abi.as_str()) {
            ("ios", "sim") => (
                "iphonesimulator",
                "-miphonesimulator-version-min=13",
                "-std=ios-metal2.2",
                "rive_pls_ios_simulator.metallib",
                translated_makefile::AppleFamily::IosSimulator,
            ),
            ("ios", _) => (
                "iphoneos",
                "-mios-version-min=13",
                "-std=ios-metal2.2",
                "rive_pls_ios.metallib",
                translated_makefile::AppleFamily::Ios,
            ),
            ("macos", _) => (
                "macosx",
                "-mmacosx-version-min=11.0",
                "-std=macos-metal2.3",
                "rive_pls_macosx.metallib",
                translated_makefile::AppleFamily::MacOs,
            ),
            ("visionos", "sim") => (
                "xrsimulator",
                "--target=air64-apple-xros1.0-simulator",
                "-std=metal3.1",
                "rive_renderer_xros_simulator.metallib",
                translated_makefile::AppleFamily::XrOsSimulator,
            ),
            ("visionos", _) => (
                "xros",
                "--target=air64-apple-xros1.0",
                "-std=metal3.1",
                "rive_renderer_xros.metallib",
                translated_makefile::AppleFamily::XrOs,
            ),
            ("tvos", "sim") => (
                "appletvsimulator",
                "-mappletvsimulator-version-min=16.0",
                "-std=metal3.0",
                "rive_renderer_appletvsimulator.metallib",
                translated_makefile::AppleFamily::AppleTvOsSimulator,
            ),
            ("tvos", _) => (
                "appletvos",
                "-mappletvos-version-min=16.0",
                "-std=metal3.0",
                "rive_renderer_appletvos.metallib",
                translated_makefile::AppleFamily::AppleTvOs,
            ),
            _ => unreachable!(),
        };
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // The translated pinned shader batch is the build authority.  Materialize
    // its embedded source constants into an upstream-shaped tree, then invoke
    // the translated minifier/Make rules against that tree.  The old
    // native_metal copies remain only as source-shaped compatibility inputs
    // for dormant/test consumers; they are never used to produce these
    // production libraries.
    let translated_shader_dir = output.join("mechanical_shader_sources");
    let translated_generated_dir = output.join("mechanical_shader_generated");
    materialize_translated_shader_sources(&translated_shader_dir)
        .unwrap_or_else(|error| panic!("materialize translated shader sources: {error}"));
    let shader_inputs = shader_batch_inputs(&translated_shader_dir)
        .unwrap_or_else(|error| panic!("enumerate translated shader inputs: {error}"));
    let shader_outputs = minifier_outputs(&shader_inputs, &translated_generated_dir);
    // Cargo reruns this build owner when any embedded translated source or
    // rule changes. Invalidate the Make-style stamp for that invocation so a
    // persistent OUT_DIR cannot reuse minified bytes from the prior source
    // snapshot merely because every output file still exists.
    match fs::remove_file(translated_generated_dir.join("glsl.stamp")) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => panic!("invalidate translated shader minify stamp: {error}"),
    }
    translated_makefile::ensure_minified_outputs(
        "python3",
        &translated_shader_dir,
        &translated_generated_dir,
        &shader_inputs,
        &shader_outputs,
    )
    .unwrap_or_else(|error| panic!("translate shader minification: {error}"));
    materialize_runtime_shader_exports(&translated_generated_dir)
        .unwrap_or_else(|error| panic!("materialize runtime shader exports: {error}"));
    materialize_runtime_shader_fragments(&shader_outputs)
        .unwrap_or_else(|error| panic!("materialize runtime shader fragments: {error}"));
    translated_draw_combinations::run([
        "generate_draw_combinations.py".to_owned(),
        translated_generated_dir
            .join("draw_combinations.metal")
            .to_string_lossy()
            .into_owned(),
    ])
    .unwrap_or_else(|error| panic!("generate translated draw combinations: {error}"));

    let draw_source = translated_shader_dir.join("metal/draw.metal");
    let draw_include_dir = translated_generated_dir.clone();
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
            translated_shader_dir.join("metal/color_ramp.metal"),
            output.join("native_metal_color_ramp.air"),
        ),
        (
            translated_shader_dir.join("metal/tessellate.metal"),
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

    let translated_source_c = translated_makefile::build_apple_metallib(
        family,
        &translated_shader_dir,
        &translated_generated_dir,
    )
    .unwrap_or_else(|error| panic!("build translated Apple shader family: {error}"));
    let translated_source_metallib = translated_generated_dir
        .join(family.rule().intermediate_dir)
        .join(family.rule().metallib_name);
    assert!(
        translated_source_c.is_file(),
        "translated C shader owner was not emitted"
    );
    fs::copy(
        translated_source_metallib,
        output.join(source_metallib_name),
    )
    .unwrap_or_else(|error| panic!("publish translated source metallib: {error}"));
}

fn generate_vulkan_spirv_module() -> io::Result<()> {
    let source_dir = PathBuf::from("src/mechanical_port/vulkan/generated/spirv");
    println!("cargo:rerun-if-changed={}", source_dir.display());

    let declaration = regex::Regex::new(r"(?s)const uint32_t ([A-Za-z0-9_]+)\[\]\s*=\s*\{(.*?)\};")
        .expect("static generated-header declaration regex");
    let word = regex::Regex::new(r"0x[0-9a-fA-F]{8}").expect("static SPIR-V word regex");
    let mut headers = fs::read_dir(&source_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("h"))
        .collect::<Vec<_>>();
    headers.sort();
    if headers.len() != 93 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected 93 frozen SPIR-V headers, found {}", headers.len()),
        ));
    }

    let mut generated = String::from(
        "// Generated from the frozen upstream SPIR-V headers.\n\
         // Do not edit: tools/backend-port/import_vulkan_spirv_headers.py owns the inputs.\n",
    );
    let mut names = std::collections::BTreeSet::new();
    for header in headers {
        let source = fs::read_to_string(&header)?;
        let captures = declaration.captures(&source).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("missing uint32_t shader array in {}", header.display()),
            )
        })?;
        let name = captures.get(1).unwrap().as_str();
        if !names.insert(name.to_owned()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("duplicate frozen SPIR-V symbol {name}"),
            ));
        }
        let words = word
            .find_iter(captures.get(2).unwrap().as_str())
            .map(|value| value.as_str())
            .collect::<Vec<_>>();
        if words.first() != Some(&"0x07230203") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{name} does not begin with the SPIR-V magic word"),
            ));
        }
        generated.push_str("pub(crate) static ");
        generated.push_str(name);
        generated.push_str(": &[u32] = &[\n");
        for chunk in words.chunks(8) {
            generated.push_str("    ");
            generated.push_str(&chunk.join(", "));
            generated.push_str(",\n");
        }
        generated.push_str("];\n");
    }

    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::write(output.join("vulkan_spirv_embedded.rs"), generated)
}

fn shader_batch_inputs(shader_dir: &std::path::Path) -> io::Result<Vec<PathBuf>> {
    let mut inputs = fs::read_dir(shader_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|extension| extension.to_str()),
                Some("glsl" | "vert" | "frag")
            )
        })
        .collect::<Vec<_>>();
    inputs.sort_by(|left, right| {
        let rank = |path: &std::path::Path| match path.extension().and_then(|value| value.to_str())
        {
            Some("glsl") => 0,
            Some("vert") => 1,
            Some("frag") => 2,
            _ => 3,
        };
        rank(left).cmp(&rank(right)).then_with(|| left.cmp(right))
    });
    Ok(inputs)
}

fn minifier_outputs(inputs: &[PathBuf], generated_dir: &std::path::Path) -> Vec<PathBuf> {
    let mut outputs = Vec::with_capacity(inputs.len() * 3);
    for input in inputs {
        let filename = input
            .file_name()
            .expect("translated shader input has a filename")
            .to_string_lossy();
        outputs.push(generated_dir.join(format!("{filename}.exports.h")));
        let minified = match input.extension().and_then(|extension| extension.to_str()) {
            Some("glsl") => filename.replace(".glsl", ".minified.glsl"),
            Some("vert") => filename.replace(".vert", ".minified.vert"),
            Some("frag") => filename.replace(".frag", ".minified.frag"),
            _ => unreachable!("shader batch only contains GLSL, vertex, and fragment inputs"),
        };
        outputs.push(generated_dir.join(minified));
        outputs.push(generated_dir.join(format!("{filename}.hpp")));
    }
    outputs
}

/// Publish the exact C++ embedded-string payload for runtime inclusion.
/// `minify.py` deliberately emits two different token streams: the offline
/// `.minified.*` files preserve exported preprocessor identifiers for command
/// line compilation, while the `.hpp` embedded strings rewrite them for the
/// runtime macro dictionary. BackgroundShaderCompiler includes the latter.
fn materialize_runtime_shader_fragments(outputs: &[PathBuf]) -> io::Result<()> {
    for output_group in outputs.chunks_exact(3) {
        let minified = &output_group[1];
        let embedded = fs::read_to_string(&output_group[2])?;
        let payload_start_marker = "R\"===(";
        let payload_end_marker = ")===";
        let payload_start = embedded
            .find(payload_start_marker)
            .map(|index| index + payload_start_marker.len())
            .ok_or_else(|| io::Error::other("generated shader header has no raw payload"))?;
        let payload_end = embedded[payload_start..]
            .find(payload_end_marker)
            .map(|index| payload_start + index)
            .ok_or_else(|| io::Error::other("generated shader header has no raw terminator"))?;
        let runtime_name = format!(
            "{}.runtime",
            minified
                .file_name()
                .expect("minified output has a filename")
                .to_string_lossy()
        );
        fs::write(
            minified.with_file_name(runtime_name),
            &embedded.as_bytes()[payload_start..payload_end],
        )?;
    }
    Ok(())
}

/// Materialize the generated `GLSL_*` export names as Rust constants and
/// Objective-C static literals. The pinned Metal sources consume the same
/// `*.exports.h` macros for both dynamic-source preprocessor keys and
/// `newFunctionWithName:` lookups; spelling the pre-minification identifiers
/// in Rust would compile a library with no exported entry points.
fn materialize_runtime_shader_exports(generated_dir: &std::path::Path) -> io::Result<()> {
    use std::collections::BTreeMap;

    const REQUIRED_EXPORTS: &[&str] = &[
        "GLSL_VERTEX",
        "GLSL_FRAGMENT",
        "GLSL_PLS_IMPL_DEVICE_BUFFER",
        "GLSL_PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED",
        "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
        "GLSL_CLOCKWISE_FILL",
        "GLSL_ENABLE_INSTANCE_INDEX",
        "GLSL_DRAW_PATH",
        "GLSL_DRAW_INTERIOR_TRIANGLES",
        "GLSL_FEATHER_ATLAS_BLIT",
        "GLSL_DRAW_IMAGE",
        "GLSL_DRAW_IMAGE_RECT",
        "GLSL_DRAW_IMAGE_MESH",
        "GLSL_DRAW_RENDER_TARGET_UPDATE_BOUNDS",
        "GLSL_INITIALIZE_PLS",
        "GLSL_STORE_COLOR_CLEAR",
        "GLSL_SWIZZLE_COLOR_BGRA_TO_RGBA",
        "GLSL_RESOLVE_PLS",
        "GLSL_COALESCED_PLS_RESOLVE_AND_TRANSFER",
        "GLSL_ENABLE_CLIPPING",
        "GLSL_ENABLE_CLIP_RECT",
        "GLSL_ENABLE_ADVANCED_BLEND",
        "GLSL_ENABLE_FEATHER",
        "GLSL_ENABLE_EVEN_ODD",
        "GLSL_ENABLE_NESTED_CLIPPING",
        "GLSL_ENABLE_HSL_BLEND_MODES",
        "GLSL_ENABLE_DITHER",
        "GLSL_colorRampVertexMain",
        "GLSL_colorRampFragmentMain",
        "GLSL_tessellateVertexMain",
        "GLSL_tessellateFragmentMain",
        "GLSL_atlasVertexMain",
        "GLSL_atlasFillFragmentMain",
        "GLSL_atlasStrokeFragmentMain",
        "GLSL_drawVertexMain",
        "GLSL_drawFragmentMain",
    ];
    const MACRO_LITERAL_EXPORTS: &[&str] = &[
        "GLSL_VERTEX",
        "GLSL_FRAGMENT",
        "GLSL_PLS_IMPL_DEVICE_BUFFER",
        "GLSL_PLS_IMPL_DEVICE_BUFFER_RASTER_ORDERED",
        "GLSL_FIXED_FUNCTION_COLOR_OUTPUT",
        "GLSL_CLOCKWISE_FILL",
        "GLSL_ENABLE_INSTANCE_INDEX",
        "GLSL_DRAW_PATH",
        "GLSL_DRAW_INTERIOR_TRIANGLES",
        "GLSL_FEATHER_ATLAS_BLIT",
        "GLSL_DRAW_IMAGE",
        "GLSL_DRAW_IMAGE_RECT",
        "GLSL_DRAW_IMAGE_MESH",
        "GLSL_DRAW_RENDER_TARGET_UPDATE_BOUNDS",
        "GLSL_INITIALIZE_PLS",
        "GLSL_STORE_COLOR_CLEAR",
        "GLSL_SWIZZLE_COLOR_BGRA_TO_RGBA",
        "GLSL_RESOLVE_PLS",
        "GLSL_COALESCED_PLS_RESOLVE_AND_TRANSFER",
    ];

    let exports = fs::read_to_string(generated_dir.join("constants.glsl.exports.h"))?;
    let mut values = BTreeMap::new();
    for line in exports.lines() {
        let Some(rest) = line.strip_prefix("#define ") else {
            continue;
        };
        let Some((name, quoted)) = rest.split_once(' ') else {
            continue;
        };
        if name.ends_with("_raw") || !quoted.starts_with('"') || !quoted.ends_with('"') {
            continue;
        }
        values.insert(name, &quoted[1..quoted.len() - 1]);
    }
    for name in REQUIRED_EXPORTS {
        if !values.contains_key(name) {
            return Err(io::Error::other(format!(
                "translated minifier did not export {name}"
            )));
        }
    }

    let mut rust =
        String::from("// @generated by build.rs from constants.glsl.exports.h; do not edit.\n");
    for name in REQUIRED_EXPORTS {
        rust.push_str(&format!("pub const {name}: &str = {:?};\n", values[name]));
    }
    let mut literal_values = vec!["", "1", "true"];
    literal_values.extend(MACRO_LITERAL_EXPORTS.iter().map(|name| values[name]));
    rust.push_str(&format!(
        "pub const SOURCE_MACRO_LITERAL_TEXTS: [&str; {}] = [\n",
        literal_values.len()
    ));
    for value in &literal_values {
        rust.push_str(&format!("    {:?},\n", value));
    }
    rust.push_str("] ;\n");
    rust.push_str(
        "#[cfg(target_vendor = \"apple\")]\n\
         pub fn source_macro_literal(text: &str) -> &'static objc2_foundation::NSString {\n\
             match text {\n",
    );
    for value in &literal_values {
        rust.push_str(&format!(
            "        {:?} => objc2_foundation::ns_string!({:?}),\n",
            value, value
        ));
    }
    rust.push_str(
        "        _ => panic!(\"generated shader export has no static NSString authority: {text}\"),\n\
         \t}\n\
         }\n",
    );
    fs::write(generated_dir.join("runtime_shader_exports.rs"), rust)
}

fn materialize_translated_shader_sources(shader_dir: &std::path::Path) -> io::Result<()> {
    for rust_source in TRANSLATED_SHADER_SOURCES {
        let upstream_path = embedded_source_path(rust_source)?;
        let source = embedded_source_literal(rust_source)?;
        let relative = upstream_path
            .strip_prefix("renderer/src/shaders/")
            .ok_or_else(|| {
                io::Error::other(format!("unexpected shader source path {upstream_path}"))
            })?;
        let destination = shader_dir.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(destination, source)?;
    }
    fs::create_dir_all(shader_dir.join("metal"))?;
    Ok(())
}

fn embedded_source_path(rust_source: &str) -> io::Result<&str> {
    let marker = "pub const PINNED_SOURCE_PATH: &str";
    let declaration = rust_source
        .find(marker)
        .map(|index| &rust_source[index + marker.len()..])
        .ok_or_else(|| io::Error::other("translated shader has no pinned source path"))?;
    let value = declaration
        .find('=')
        .map(|index| declaration[index + 1..].trim_start())
        .and_then(|value| value.strip_prefix('"'))
        .ok_or_else(|| io::Error::other("malformed pinned shader source path"))?;
    let end = value
        .find('"')
        .ok_or_else(|| io::Error::other("unterminated pinned shader source path"))?;
    Ok(&value[..end])
}

/// Extract the first raw string assigned to a `PINNED_*_SOURCE` constant.
/// Keeping this tiny parser in the build owner lets the build consume the
/// literal translated constants without duplicating their bytes in build.rs.
fn embedded_source_literal(rust_source: &str) -> io::Result<&str> {
    let mut cursor = 0;
    while let Some(relative) = rust_source[cursor..].find("pub const PINNED_") {
        let declaration_start = cursor + relative;
        let declaration = &rust_source[declaration_start..];
        if let Some(relative_raw) = declaration.find("_SOURCE: &str = r") {
            let raw_start = declaration_start + relative_raw + "_SOURCE: &str = ".len();
            let raw = &rust_source[raw_start..];
            let quote = raw
                .find('"')
                .ok_or_else(|| io::Error::other("malformed translated shader raw delimiter"))?;
            let hashes = raw[..quote].len().saturating_sub(1);
            let body_start = raw_start + quote + 1;
            let terminator = format!("\"{}", "#".repeat(hashes));
            let body_end = rust_source[body_start..]
                .find(&terminator)
                .map(|index| body_start + index)
                .ok_or_else(|| io::Error::other("unterminated translated shader raw constant"))?;
            return Ok(&rust_source[body_start..body_end]);
        }
        cursor = declaration_start + "pub const PINNED_".len();
    }
    Err(io::Error::other(
        "translated shader has no pinned source constant",
    ))
}

const TRANSLATED_SHADER_RUST_PATHS: &[&str] = &[
    "src/mechanical_port/source/renderer/src/shaders/advanced_blend_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/atomic_draw_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/bezier_utils_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/blit_texture_as_draw_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/clear_clockwise_atomic_clip_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/color_ramp_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/common_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/constants_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_clockwise_atomic_borrowed_coverage_frag.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_clockwise_atomic_clip_frag.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_clockwise_atomic_path_frag.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_clockwise_clip_frag.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_clockwise_path_frag.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_fullscreen_quad_vert.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_image_mesh_vert.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_input_attachment_frag.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_mesh_frag.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_msaa_object_frag.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_msaa_resolve_frag.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_path_common_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_path_vert.rs",
    "src/mechanical_port/source/renderer/src/shaders/draw_raster_order_path_frag.rs",
    "src/mechanical_port/source/renderer/src/shaders/flush_uniforms_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/glsl_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/hlsl_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/init_clockwise_atomic_workaround_frag.rs",
    "src/mechanical_port/source/renderer/src/shaders/metal_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/pls_load_store_ext_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/render_atlas_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/resolve_atlas_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/rhi_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/specialization_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/stencil_draw_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/tessellate_glsl.rs",
    "src/mechanical_port/source/renderer/src/shaders/metal/color_ramp_metal.rs",
    "src/mechanical_port/source/renderer/src/shaders/metal/draw_metal.rs",
    "src/mechanical_port/source/renderer/src/shaders/metal/tessellate_metal.rs",
];

const TRANSLATED_BUILD_RULE_PATHS: &[&str] = &[
    "src/mechanical_port/source/renderer/src/shaders/makefile.rs",
    "src/mechanical_port/source/renderer/src/shaders/minify_py.rs",
    "src/mechanical_port/source/renderer/src/shaders/metal/generate_draw_combinations_py.rs",
];

const TRANSLATED_SHADER_SOURCES: &[&str] = &[
    include_str!("src/mechanical_port/source/renderer/src/shaders/advanced_blend_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/atomic_draw_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/bezier_utils_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/blit_texture_as_draw_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/clear_clockwise_atomic_clip_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/color_ramp_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/common_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/constants_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_clockwise_atomic_borrowed_coverage_frag.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_clockwise_atomic_clip_frag.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_clockwise_atomic_path_frag.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_clockwise_clip_frag.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_clockwise_path_frag.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_fullscreen_quad_vert.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_image_mesh_vert.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_input_attachment_frag.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_mesh_frag.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_msaa_object_frag.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_msaa_resolve_frag.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_path_common_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_path_vert.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/draw_raster_order_path_frag.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/flush_uniforms_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/glsl_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/hlsl_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/init_clockwise_atomic_workaround_frag.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/metal_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/pls_load_store_ext_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/render_atlas_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/resolve_atlas_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/rhi_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/specialization_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/stencil_draw_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/tessellate_glsl.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/metal/color_ramp_metal.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/metal/draw_metal.rs"),
    include_str!("src/mechanical_port/source/renderer/src/shaders/metal/tessellate_metal.rs"),
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
