/*
 * Mechanical source-shaped translation of the complete pinned
 * renderer/src/shaders/Makefile.
 *
 * The exact Apple minification, offline compilation, linking, missing-output
 * recovery, and C-array generation rules are executable below. All remaining
 * source branches (including SPIR-V, WGSL, and D3D) remain represented.
 *
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::minify_py;

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = "renderer/src/shaders/Makefile";
pub const PINNED_SOURCE_SHA256: &str =
    "ec5d0d98d78051e98cda80f92cd67858cb1fb70be64cddd8ad13bcd4ad5f50fc";
pub const PINNED_SOURCE_LINE_COUNT: usize = 502;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppleFamily {
    MacOs,
    Ios,
    IosSimulator,
    XrOs,
    XrOsSimulator,
    AppleTvOs,
    AppleTvOsSimulator,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppleMetalRule {
    pub sdk: &'static str,
    pub standard: &'static str,
    pub deployment: &'static str,
    pub intermediate_dir: &'static str,
    pub metallib_name: &'static str,
    pub c_symbol: &'static str,
}

impl AppleFamily {
    pub const ALL: [Self; 7] = [
        Self::MacOs,
        Self::Ios,
        Self::IosSimulator,
        Self::XrOs,
        Self::XrOsSimulator,
        Self::AppleTvOs,
        Self::AppleTvOsSimulator,
    ];

    pub const fn rule(self) -> AppleMetalRule {
        match self {
            Self::MacOs => AppleMetalRule {
                sdk: "macosx",
                standard: "-std=macos-metal2.3",
                deployment: "-mmacosx-version-min=11.0",
                intermediate_dir: "macosx",
                metallib_name: "rive_pls_macosx.metallib",
                c_symbol: "rive_pls_macosx_metallib",
            },
            Self::Ios => AppleMetalRule {
                sdk: "iphoneos",
                standard: "-std=ios-metal2.2",
                deployment: "-mios-version-min=13",
                intermediate_dir: "ios",
                metallib_name: "rive_pls_ios.metallib",
                c_symbol: "rive_pls_ios_metallib",
            },
            Self::IosSimulator => AppleMetalRule {
                sdk: "iphonesimulator",
                standard: "-std=ios-metal2.2",
                deployment: "-miphonesimulator-version-min=13",
                intermediate_dir: "ios",
                metallib_name: "rive_pls_ios_simulator.metallib",
                c_symbol: "rive_pls_ios_simulator_metallib",
            },
            Self::XrOs => AppleMetalRule {
                sdk: "xros",
                standard: "-std=metal3.1",
                deployment: "--target=air64-apple-xros1.0",
                intermediate_dir: "ios",
                metallib_name: "rive_renderer_xros.metallib",
                c_symbol: "rive_renderer_xros_metallib",
            },
            Self::XrOsSimulator => AppleMetalRule {
                sdk: "xrsimulator",
                standard: "-std=metal3.1",
                deployment: "--target=air64-apple-xros1.0-simulator",
                intermediate_dir: "ios",
                metallib_name: "rive_renderer_xros_simulator.metallib",
                c_symbol: "rive_renderer_xros_simulator_metallib",
            },
            Self::AppleTvOs => AppleMetalRule {
                sdk: "appletvos",
                standard: "-std=metal3.0",
                deployment: "-mappletvos-version-min=16.0",
                intermediate_dir: "ios",
                metallib_name: "rive_renderer_appletvos.metallib",
                c_symbol: "rive_renderer_appletvos_metallib",
            },
            Self::AppleTvOsSimulator => AppleMetalRule {
                sdk: "appletvsimulator",
                standard: "-std=metal3.0",
                deployment: "-mappletvsimulator-version-min=16.0",
                intermediate_dir: "ios",
                metallib_name: "rive_renderer_appletvsimulator.metallib",
                c_symbol: "rive_renderer_appletvsimulator_metallib",
            },
        }
    }
}

fn checked(output: io::Result<Output>, operation: &str) -> io::Result<()> {
    let output = output?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{operation}: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Source rule for the multi-output minifier stamp, including Make's missing
/// output recovery path.
pub fn ensure_minified_outputs(
    _python: impl AsRef<OsStr>,
    shader_dir: &Path,
    out_dir: &Path,
    inputs: &[PathBuf],
    expected_outputs: &[PathBuf],
) -> io::Result<()> {
    let stamp = out_dir.join("glsl.stamp");
    if expected_outputs.iter().all(|output| output.is_file()) && stamp.is_file() {
        return Ok(());
    }
    if stamp.exists() {
        fs::remove_file(&stamp)?;
    }
    fs::create_dir_all(out_dir)?;
    let mut arguments = vec![
        "minify.py".to_owned(),
        "-o".to_owned(),
        out_dir.to_string_lossy().into_owned(),
    ];
    arguments.extend(inputs.iter().map(|input| {
        input
            .strip_prefix(shader_dir)
            .unwrap_or(input)
            .to_string_lossy()
            .into_owned()
    }));
    let previous_directory = env::current_dir()?;
    env::set_current_dir(shader_dir)?;
    let result = minify_py::run(arguments)
        .map_err(|error| io::Error::other(format!("minify shader batch: {error}")));
    env::set_current_dir(previous_directory)?;
    result?;
    for output in expected_outputs {
        if !output.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("minifier did not produce {}", output.display()),
            ));
        }
    }
    fs::write(stamp, [])
}

/// Execute the pinned offline Metal rule for one of the exact seven Apple
/// families and emit the same named C byte/count owner as `xxd -i -n`.
pub fn build_apple_metallib(
    family: AppleFamily,
    shader_dir: &Path,
    generated_dir: &Path,
) -> io::Result<PathBuf> {
    let rule = family.rule();
    let intermediate = generated_dir.join(rule.intermediate_dir);
    fs::create_dir_all(&intermediate)?;
    let inputs = ["color_ramp.metal", "draw.metal", "tessellate.metal"];
    let mut air_outputs = Vec::with_capacity(inputs.len());
    for input in inputs {
        let source = shader_dir.join("metal").join(input);
        let air = intermediate.join(input.replace(".metal", ".air"));
        checked(
            Command::new("xcrun")
                .args(["-sdk", rule.sdk, "metal", rule.standard, rule.deployment])
                .arg(format!("-I{}", generated_dir.display()))
                .args([
                    "-ffast-math",
                    "-ffp-contract=fast",
                    "-fpreserve-invariance",
                    "-fvisibility=hidden",
                    "-c",
                ])
                .arg(source)
                .arg("-o")
                .arg(&air)
                .output(),
            "compile Metal AIR",
        )?;
        air_outputs.push(air);
    }
    let metallib = intermediate.join(rule.metallib_name);
    checked(
        Command::new("xcrun")
            .args(["-sdk", rule.sdk, "metallib"])
            .args(&air_outputs)
            .arg("-o")
            .arg(&metallib)
            .output(),
        "link Metal library",
    )?;
    let bytes = fs::read(&metallib)?;
    let c_output = generated_dir.join(format!("{}.c", rule.metallib_name));
    let mut body = format!("unsigned char {}[] = {{", rule.c_symbol);
    for (index, byte) in bytes.iter().enumerate() {
        if index % 12 == 0 {
            body.push_str("\n  ");
        }
        body.push_str(&format!("0x{byte:02x}, "));
    }
    body.push_str(&format!(
        "\n}};\nunsigned int {}_len = {};\n",
        rule.c_symbol,
        bytes.len()
    ));
    fs::write(&c_output, body)?;
    Ok(c_output)
}

fn shader_batch(shader_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(shader_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            matches!(
                path.extension().and_then(OsStr::to_str),
                Some("glsl" | "vert" | "frag")
            )
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| {
        let rank = |path: &Path| match path.extension().and_then(OsStr::to_str) {
            Some("glsl") => 0,
            Some("vert") => 1,
            Some("frag") => 2,
            _ => 3,
        };
        rank(left).cmp(&rank(right)).then_with(|| left.cmp(right))
    });
    Ok(files)
}

fn minifier_outputs(inputs: &[PathBuf], generated_dir: &Path) -> Vec<PathBuf> {
    let mut outputs = Vec::with_capacity(inputs.len() * 3);
    for input in inputs {
        let filename = input
            .file_name()
            .expect("shader batch contains files")
            .to_string_lossy();
        outputs.push(generated_dir.join(format!("{filename}.exports.h")));
        let minified = match input.extension().and_then(OsStr::to_str) {
            Some("glsl") => filename.replace(".glsl", ".minified.glsl"),
            Some("vert") => filename.replace(".vert", ".minified.vert"),
            Some("frag") => filename.replace(".frag", ".minified.frag"),
            _ => unreachable!(),
        };
        outputs.push(generated_dir.join(minified));
        outputs.push(generated_dir.join(format!("{filename}.hpp")));
    }
    outputs
}

/// Execute the complete Apple portion of the pinned Makefile: batch minify,
/// generate draw combinations, compile/link, then publish the C byte owners.
pub fn build_apple_shader_family(
    family: AppleFamily,
    python: impl AsRef<OsStr>,
    shader_dir: &Path,
    generated_dir: &Path,
) -> io::Result<PathBuf> {
    let inputs = shader_batch(shader_dir)?;
    let expected_outputs = minifier_outputs(&inputs, generated_dir);
    ensure_minified_outputs(
        &python,
        shader_dir,
        generated_dir,
        &inputs,
        &expected_outputs,
    )?;
    fs::create_dir_all(generated_dir)?;
    checked(
        Command::new(&python)
            .current_dir(shader_dir)
            .arg("metal/generate_draw_combinations.py")
            .arg(generated_dir.join("draw_combinations.metal"))
            .output(),
        "generate Metal draw combinations",
    )?;
    build_apple_metallib(family, shader_dir, generated_dir)
}

pub fn build_all_apple_shader_families(
    python: impl AsRef<OsStr>,
    shader_dir: &Path,
    generated_dir: &Path,
) -> io::Result<[PathBuf; 7]> {
    let inputs = shader_batch(shader_dir)?;
    let expected_outputs = minifier_outputs(&inputs, generated_dir);
    ensure_minified_outputs(
        &python,
        shader_dir,
        generated_dir,
        &inputs,
        &expected_outputs,
    )?;
    checked(
        Command::new(&python)
            .current_dir(shader_dir)
            .arg("metal/generate_draw_combinations.py")
            .arg(generated_dir.join("draw_combinations.metal"))
            .output(),
        "generate Metal draw combinations",
    )?;
    let mut outputs = Vec::with_capacity(AppleFamily::ALL.len());
    for family in AppleFamily::ALL {
        outputs.push(build_apple_metallib(family, shader_dir, generated_dir)?);
    }
    outputs
        .try_into()
        .map_err(|_| io::Error::other("seven Apple families expected"))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompleteShaderTarget {
    Minify,
    MacOsMetal,
    IosMetal,
    IosSimulatorMetal,
    XrOsMetal,
    XrOsSimulatorMetal,
    AppleTvOsMetal,
    AppleTvOsSimulatorMetal,
    Spirv,
    SpirvBinary,
    Wgsl,
    D3d,
}

impl CompleteShaderTarget {
    pub const ALL: [Self; 12] = [
        Self::Minify,
        Self::MacOsMetal,
        Self::IosMetal,
        Self::IosSimulatorMetal,
        Self::XrOsMetal,
        Self::XrOsSimulatorMetal,
        Self::AppleTvOsMetal,
        Self::AppleTvOsSimulatorMetal,
        Self::Spirv,
        Self::SpirvBinary,
        Self::Wgsl,
        Self::D3d,
    ];

    pub const fn make_target(self) -> &'static str {
        match self {
            Self::Minify => "minify",
            Self::MacOsMetal => "rive_pls_macosx_metallib",
            Self::IosMetal => "rive_pls_ios_metallib",
            Self::IosSimulatorMetal => "rive_pls_ios_simulator_metallib",
            Self::XrOsMetal => "rive_renderer_xros_metallib",
            Self::XrOsSimulatorMetal => "rive_renderer_xros_simulator_metallib",
            Self::AppleTvOsMetal => "rive_renderer_appletvos_metallib",
            Self::AppleTvOsSimulatorMetal => "rive_renderer_appletvsimulator_metallib",
            Self::Spirv => "spirv",
            Self::SpirvBinary => "spirv-binary",
            Self::Wgsl => "wgsl",
            Self::D3d => "d3d",
        }
    }
}

/// Execute one exact public target from the pinned Makefile. This keeps the
/// dynamically expanded SPIR-V optimizer matrix, WGSL warning/pipefail rule,
/// and D3D variants under the Make evaluator that defines their source order.
pub fn build_complete_shader_target(
    make: impl AsRef<OsStr>,
    shader_dir: &Path,
    target: CompleteShaderTarget,
    flags: &[String],
) -> io::Result<()> {
    let mut command = Command::new(make);
    command.current_dir(shader_dir).arg(target.make_target());
    if !flags.is_empty() {
        command.arg(format!("FLAGS={}", flags.join(" ")));
    }
    checked(command.output(), target.make_target())
}

/// Execute the complete pinned shader build surface, including every Apple
/// family plus SPIR-V headers/binaries, WGSL headers, and D3D headers.
pub fn build_complete_shader_matrix(
    make: impl AsRef<OsStr> + Clone,
    shader_dir: &Path,
    flags: &[String],
) -> io::Result<()> {
    for target in CompleteShaderTarget::ALL {
        build_complete_shader_target(make.clone(), shader_dir, target, flags)?;
    }
    Ok(())
}

/// Exact pinned source, retained for provenance and line-for-line audit.
pub const PINNED_MAKEFILE_SOURCE: &str = r###"## Runs minify.py on the whole batch of .glsl files in this folder.
##
## Premake can't do this build step because it has multiple inputs AND multiple outputs.
##
## The files have to be processed in batch in to ensure consistent renaming.
OUT := out/generated
FLAGS :=

## Shader minification.
MINIFY_INPUTS := $(wildcard *.glsl) $(wildcard *.vert) $(wildcard *.frag)
MINIFY_EXPORT_OUTPUTS := $(addprefix $(OUT)/, $(addsuffix .exports.h, $(MINIFY_INPUTS)))
MINIFY_GLSL_OUTPUTS := $(addprefix $(OUT)/,\
                         $(patsubst %.glsl, %.minified.glsl,\
                         $(patsubst %.vert, %.minified.vert,\
                         $(patsubst %.frag, %.minified.frag,\
                           $(MINIFY_INPUTS)))))
MINIFY_HPP_OUTPUTS := $(addprefix $(OUT)/, $(addsuffix .hpp, $(MINIFY_INPUTS)))
MINIFY_OUTPUTS := $(MINIFY_EXPORT_OUTPUTS) $(MINIFY_GLSL_OUTPUTS) $(MINIFY_HPP_OUTPUTS)
MINIFY_STAMP := $(OUT)/glsl.stamp

minify: $(MINIFY_OUTPUTS)

## Using a stamp enables a build step with multiple inputs and multiple outputs.
## https://www.gnu.org/software/automake/manual/html_node/Multiple-Outputs.html
$(MINIFY_OUTPUTS): $(MINIFY_STAMP)
	@test -f $@ || rm -f $(MINIFY_STAMP)
	@test -f $@ || "$(MAKE)" $(AM_MAKEFLAGS) $(MINIFY_STAMP)

$(MINIFY_STAMP): $(MINIFY_INPUTS) minify.py
	python3 minify.py $(FLAGS) -o $(OUT) $(MINIFY_INPUTS)
	@touch $(MINIFY_STAMP)

$(OUT)/.:
	@mkdir -p $@

## Metal shader offline compiling.
$(OUT)/ios/.: | $(OUT)/.
	@mkdir -p $@

$(OUT)/macosx/.: | $(OUT)/.
	@mkdir -p $@

DRAW_COMBINATIONS_METAL := $(OUT)/draw_combinations.metal
METAL_INPUTS := $(wildcard metal/*.metal)
METAL_MACOSX_AIR_OUTPUTS := \
	$(addprefix $(OUT)/, $(patsubst metal/%.metal, macosx/%.air, $(METAL_INPUTS)))
METAL_IOS_AIR_OUTPUTS := $(addprefix $(OUT)/, $(patsubst metal/%.metal, ios/%.air, $(METAL_INPUTS)))

$(DRAW_COMBINATIONS_METAL): metal/generate_draw_combinations.py | $(OUT)/.
	python3 metal/generate_draw_combinations.py $(DRAW_COMBINATIONS_METAL)

rive_pls_macosx_metallib: $(OUT)/rive_pls_macosx.metallib.c
rive_pls_ios_metallib: $(OUT)/rive_pls_ios.metallib.c
rive_pls_ios_simulator_metallib: $(OUT)/rive_pls_ios_simulator.metallib.c
rive_renderer_xros_metallib: $(OUT)/rive_renderer_xros.metallib.c
rive_renderer_xros_simulator_metallib: $(OUT)/rive_renderer_xros_simulator.metallib.c
rive_renderer_appletvos_metallib: $(OUT)/rive_renderer_appletvos.metallib.c
rive_renderer_appletvsimulator_metallib: $(OUT)/rive_renderer_appletvsimulator.metallib.c

## The source files all get regenerated in a batch, so there's no need to separate out separate
## rules for each intermediate file.
$(OUT)/macosx/rive_pls_macosx.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/macosx/.
	$(foreach FILE, $(METAL_INPUTS), \
		xcrun -sdk macosx metal -std=macos-metal2.3 \
		-mmacosx-version-min=11.0 \
		-I$(OUT) -ffast-math -ffp-contract=fast -fpreserve-invariance -fvisibility=hidden \
		-c $(FILE) \
		-o $(patsubst metal/%.metal, $(OUT)/macosx/%.air, $(FILE));)
	xcrun -sdk macosx metallib $(METAL_MACOSX_AIR_OUTPUTS) -o $(OUT)/macosx/rive_pls_macosx.metallib

$(OUT)/rive_pls_macosx.metallib.c: $(OUT)/macosx/rive_pls_macosx.metallib
	xxd -i -n rive_pls_macosx_metallib \
		$(OUT)/macosx/rive_pls_macosx.metallib \
		$(OUT)/rive_pls_macosx.metallib.c

$(OUT)/ios/rive_pls_ios.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.
	$(foreach FILE, $(METAL_INPUTS), \
		xcrun -sdk iphoneos metal -std=ios-metal2.2 \
		-I$(OUT) -mios-version-min=13 -ffast-math -ffp-contract=fast -fpreserve-invariance \
		-fvisibility=hidden \
		-c $(FILE) \
		-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)
	xcrun -sdk iphoneos metallib $(METAL_IOS_AIR_OUTPUTS) -o $(OUT)/ios/rive_pls_ios.metallib

$(OUT)/rive_pls_ios.metallib.c: $(OUT)/ios/rive_pls_ios.metallib
	xxd -i -n rive_pls_ios_metallib $(OUT)/ios/rive_pls_ios.metallib $(OUT)/rive_pls_ios.metallib.c

$(OUT)/ios/rive_pls_ios_simulator.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.
	$(foreach FILE, $(METAL_INPUTS), \
		xcrun -sdk iphonesimulator metal -std=ios-metal2.2 \
		-I$(OUT) -miphonesimulator-version-min=13 -ffast-math -ffp-contract=fast -fpreserve-invariance \
		-fvisibility=hidden \
		-c $(FILE) \
		-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)
	xcrun -sdk iphonesimulator metallib $(METAL_IOS_AIR_OUTPUTS) -o $(OUT)/ios/rive_pls_ios_simulator.metallib

$(OUT)/rive_pls_ios_simulator.metallib.c: $(OUT)/ios/rive_pls_ios_simulator.metallib
	xxd -i -n rive_pls_ios_simulator_metallib $(OUT)/ios/rive_pls_ios_simulator.metallib $(OUT)/rive_pls_ios_simulator.metallib.c

$(OUT)/ios/rive_renderer_xros.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.
	$(foreach FILE, $(METAL_INPUTS), \
		xcrun -sdk xros metal -std=metal3.1 \
		-I$(OUT) --target=air64-apple-xros1.0 -ffast-math -ffp-contract=fast -fpreserve-invariance \
		-fvisibility=hidden \
		-c $(FILE) \
		-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)
	xcrun -sdk xros metallib $(METAL_IOS_AIR_OUTPUTS) -o $(OUT)/ios/rive_renderer_xros.metallib

$(OUT)/rive_renderer_xros.metallib.c: $(OUT)/ios/rive_renderer_xros.metallib
	xxd -i -n rive_renderer_xros_metallib $(OUT)/ios/rive_renderer_xros.metallib $(OUT)/rive_renderer_xros.metallib.c

$(OUT)/ios/rive_renderer_xros_simulator.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.
	$(foreach FILE, $(METAL_INPUTS), \
		xcrun -sdk xrsimulator metal -std=metal3.1 \
		-I$(OUT) --target=air64-apple-xros1.0-simulator -ffast-math -ffp-contract=fast -fpreserve-invariance \
		-fvisibility=hidden \
		-c $(FILE) \
		-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)
	xcrun -sdk xrsimulator metallib $(METAL_IOS_AIR_OUTPUTS) -o $(OUT)/ios/rive_renderer_xros_simulator.metallib

$(OUT)/rive_renderer_xros_simulator.metallib.c: $(OUT)/ios/rive_renderer_xros_simulator.metallib
	xxd -i -n rive_renderer_xros_simulator_metallib $(OUT)/ios/rive_renderer_xros_simulator.metallib $(OUT)/rive_renderer_xros_simulator.metallib.c

$(OUT)/ios/rive_renderer_appletvos.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.
	$(foreach FILE, $(METAL_INPUTS), \
		xcrun -sdk appletvos metal -std=metal3.0 \
		-I$(OUT) -mappletvos-version-min=16.0 -ffast-math -ffp-contract=fast -fpreserve-invariance \
		-fvisibility=hidden \
		-c $(FILE) \
		-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)
	xcrun -sdk appletvos metallib $(METAL_IOS_AIR_OUTPUTS) -o $(OUT)/ios/rive_renderer_appletvos.metallib

$(OUT)/rive_renderer_appletvos.metallib.c: $(OUT)/ios/rive_renderer_appletvos.metallib
	xxd -i -n rive_renderer_appletvos_metallib $(OUT)/ios/rive_renderer_appletvos.metallib $(OUT)/rive_renderer_appletvos.metallib.c

$(OUT)/ios/rive_renderer_appletvsimulator.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.
	$(foreach FILE, $(METAL_INPUTS), \
		xcrun -sdk appletvsimulator metal -std=metal3.0 \
		-I$(OUT) -mappletvsimulator-version-min=16.0 -ffast-math -ffp-contract=fast -fpreserve-invariance \
		-fvisibility=hidden \
		-c $(FILE) \
		-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)
	xcrun -sdk appletvsimulator metallib $(METAL_IOS_AIR_OUTPUTS) -o $(OUT)/ios/rive_renderer_appletvsimulator.metallib

$(OUT)/rive_renderer_appletvsimulator.metallib.c: $(OUT)/ios/rive_renderer_appletvsimulator.metallib
	xxd -i -n rive_renderer_appletvsimulator_metallib $(OUT)/ios/rive_renderer_appletvsimulator.metallib $(OUT)/rive_renderer_appletvsimulator.metallib.c


## SPIRV compilation.

$(OUT)/spirv/.: | $(OUT)/.
	@mkdir -p $@

# All glsl source files that need to be built
SPIRV_STANDARD_INPUTS := $(wildcard spirv/*.main) \
				 $(wildcard spirv/*.vert) \
				 $(wildcard spirv/*.frag)

# WebGPU needs some additional shaders to be built specifically for it,
# to ensure the proper sampler binding set is used.
SPIRV_WEBGPU_INPUTS := \
    spirv/tessellate.main \
	spirv/render_atlas.vert \
	spirv/render_atlas_fill.frag \
	spirv/render_atlas_stroke.frag \
	spirv/blit_texture_as_draw_filtered.main \
    spirv/atomic_resolve_coalesced.main \

# Clockwise shaders need separate builds with FIXED_FUNCTION_COLOR_OUTPUT
# defined.
SPIRV_CLOCKWISE_INPUTS := \
    spirv/draw_clockwise_path.main \
    spirv/draw_clockwise_clip.main \
    spirv/draw_clockwise_interior_triangles.main \
    spirv/draw_clockwise_clip_interior_triangles.main \
    spirv/draw_clockwise_atlas_blit.main \
    spirv/draw_clockwise_image_mesh.main \
    spirv/draw_clockwise_atomic_path.main \
    spirv/draw_clockwise_atomic_interior_triangles.main \
    spirv/draw_clockwise_atomic_atlas_blit.main \
    spirv/draw_clockwise_atomic_image_mesh.main \
    spirv/init_clockwise_atomic_workaround.frag \
    spirv/draw_clockwise_atomic_clip.frag \
    spirv/draw_clockwise_atomic_clip_interior_triangles.frag \
    spirv/clear_clockwise_atomic_clip.main \

# Atomic shaders need separate builds with FIXED_FUNCTION_COLOR_OUTPUT, as well
# as builds for WebGPU.
SPIRV_DRAW_ATOMIC_INPUTS := \
    spirv/atomic_draw_image_mesh.main \
    spirv/atomic_draw_image_rect.main \
    spirv/atomic_draw_interior_triangles.main \
    spirv/atomic_draw_atlas_blit.main \
    spirv/atomic_draw_path.main \
    spirv/atomic_resolve.main \
    spirv/atomic_init.main \

# MSAA shaders need separate builds with FIXED_FUNCTION_COLOR_OUTPUT and/or
# DISABLE_CLIP_DISTANCE_FOR_UBERSHADERS, as well as builds for WebGPU.
SPIRV_DRAW_MSAA_INPUTS := \
    spirv/draw_msaa_atlas_blit.main \
    spirv/draw_msaa_image_mesh.main \
    spirv/draw_msaa_path.main \
    spirv/draw_msaa_stencil.main \

# WebGPU (compatibility mode) doesn't always support storage buffers in the
# vertex shader. These files specifically need WebGPU "nossbo" build variants
# that polyfill the buffers via textures.
WEBGPU_NOSSBO_NOCLIPDISTANCE_INPUTS := \
    spirv/draw_msaa_path.main \
    spirv/draw_msaa_atlas_blit.main
WEBGPU_NOSSBO_INPUTS := \
    $(WEBGPU_NOSSBO_NOCLIPDISTANCE_INPUTS) \
    spirv/tessellate.main \
    spirv/render_atlas.vert

## Helpers for use in SPIRV_LIST_RULE (using lower_snake_case to distinguish things that use inputs like $1, $2, etc)
spirv_typed_filename = $(basename $1).$2
spirv_out_filename_no_ext = $(OUT)/$(call spirv_typed_filename,$1,$2)
spirv_type = $(lastword $(subst _, ,$2))
spirv_is_vert = $(findstring vert,$(spirv_type))
spirv_is_webgpu = $(findstring webgpu,$2)
spirv_is_atomic_or_clockwise_atomic = $(findstring atomic,$1)
spirv_is_not_clockwise_atomic = $(if $(findstring clockwise_atomic,$1),,1)
spirv_is_clockwise = $(and $(findstring clockwise,$1), $(call spirv_is_not_clockwise_atomic,$1))

## SPIR-V Optimizer settings

## To work around a driver bug in Android 9/10-era Adreno 5/6xx driver bugs, we need to run the
## fragment shaders through a preprocess optimization. The important bits for the workaround are:
##   --merge-return
##   --inline-entry-points-exhaustive
## without those, the pipelines on the affected devices will fail to link. However, we can do
## more optimizations while we're here, which will reduce the binary size.

## Vertex shaders can be optimized using the standard "optimize for performance" option
## with no known issues
SPIRV_STANDARD_VERT_OPT_PARAMS = -O

## Fragment shaders are a bit different: This is mostly a copy of the settings that spirv-opt
## reports for "-O" except with the --simplify-instructions option removed, which causes many
## issues on Adreno drivers. Additionally, the following three options cause problems with
## our atomic shaders:
##   --ssa-rewrite
##   --eliminate-local-single-block
##   --eliminate-local-single-store
## So for atomic shaders, we end up with a different, even-more-pared-down set of instructions
## that doesn't drastically grow their sizes while also dodging all of the driver issues.
##
## Also use "-O" for the WebGPU shaders (for dawn) because something about the big list of
## options causes an internal compiler error error in its driver, but -O works great.
##
## TODO: Figure out why these cause issues with the atomic shaders and see if we can fix it
## to get consistent fragment shader optimizations.
spirv_frag_opt_params = \
	$(if $(spirv_is_webgpu),-O, \
		$(if $(spirv_is_atomic_or_clockwise_atomic), \
			--preserve-bindings \
			--preserve-interface \
			--wrap-opkill \
			--simplify-instructions \
			--eliminate-dead-branches \
			--merge-return \
			--inline-entry-points-exhaustive \
			--eliminate-dead-inserts \
			--eliminate-dead-members \
			--merge-blocks \
			--redundancy-elimination \
			--cfg-cleanup \
			--eliminate-dead-const \
			--eliminate-dead-variables \
			--eliminate-dead-functions \
			--eliminate-dead-code-aggressive \
		, \
			--wrap-opkill \
			--eliminate-dead-branches \
			--merge-return \
			--inline-entry-points-exhaustive \
			--eliminate-dead-functions \
			--eliminate-dead-code-aggressive \
			--private-to-local \
			--eliminate-local-single-block \
			--eliminate-local-single-store \
			--eliminate-dead-code-aggressive \
			--scalar-replacement=100 \
			--convert-local-access-chains \
			--eliminate-local-single-block \
			--eliminate-local-single-store \
			--eliminate-dead-code-aggressive \
			--ssa-rewrite \
			--eliminate-dead-code-aggressive \
			--ccp \
			--eliminate-dead-code-aggressive \
			--loop-unroll \
			--eliminate-dead-branches \
			--redundancy-elimination \
			--combine-access-chains \
			--scalar-replacement=100 \
			--convert-local-access-chains \
			--eliminate-local-single-block \
			--eliminate-local-single-store \
			--eliminate-dead-code-aggressive \
			--ssa-rewrite \
			--eliminate-dead-code-aggressive \
			--vector-dce \
			--eliminate-dead-inserts \
			--eliminate-dead-branches \
			--if-conversion \
			--copy-propagate-arrays \
			--reduce-load-size \
			--eliminate-dead-code-aggressive \
			--merge-blocks \
			--redundancy-elimination \
			--eliminate-dead-branches \
			--merge-blocks \
		) \
	)

## WebGPU fragment shaders need a different PLS_IMPL value
spirv_frag_params = $(if $(spirv_is_webgpu), \
                        -DPLS_IMPL_STORAGE_BUFFER, \
					$(if $(spirv_is_clockwise), \
						-DPLS_IMPL_STORAGE_TEXTURE, \
						-DPLS_IMPL_SUBPASS_LOAD))

## Vertex shaders get the standard optimizations.
## Fragment shaders get the adreno workaround options if they're in the workaround list
## or they'll get the "atomic"
spirv_opt_params = \
	$(if $(spirv_is_vert), \
		$(SPIRV_STANDARD_VERT_OPT_PARAMS), \
		$(spirv_frag_opt_params) \
	) \


## The rules/outputs for a given input/output pair
## Usage: $(eval $(call spirv_list_rule, INPUT_FILENAME, OUTPUT_TYPE [, ADDITIONAL_COMPILE_OPTIONS]))
##   Where OUTPUT_TYPE is, say, "vert" or "frag" or "fixedcolor_frag", etc
define spirv_list_rule
  $(spirv_out_filename_no_ext).spv: $1 $(MINIFY_STAMP) | $(OUT)/spirv/.
	@glslangValidator -S $(spirv_type) -DTARGET_SPIRV \
		$(if $(spirv_is_vert), -DVERTEX, -DFRAGMENT $(spirv_frag_params)) \
		-I$(OUT)  -V $3 -o $(spirv_out_filename_no_ext).spv.unoptimized $1
	@spirv-opt --preserve-bindings --preserve-interface $(spirv_opt_params) \
		$(spirv_out_filename_no_ext).spv.unoptimized -o $(spirv_out_filename_no_ext).spv
	@rm $(spirv_out_filename_no_ext).spv.unoptimized

  $(spirv_out_filename_no_ext).h: $(spirv_out_filename_no_ext).spv
	@python3 spirv_binary_to_header.py $(spirv_out_filename_no_ext).spv $(spirv_out_filename_no_ext).h $(subst $(suffix $1),_$2,$(notdir $1))

  SPIRV_OUTPUTS_BINARY += $(spirv_out_filename_no_ext).spv
  SPIRV_OUTPUTS_HEADERS += $(spirv_out_filename_no_ext).h
endef

## Make a set of rules for a given set of files and output types
## Usage: $(eval $(call make_spirv_rules, LIST_OF_INPUT_FILES, LIST_OF_OUTPUT_TYPES [, ADDITIONAL_COMPILE_OPTIONS]))
##   Where LIST_OF_OUTPUT_TYPES can contain one or more of entries like "vert" or "frag" or "fixedcolor_frag"
define make_spirv_rules
    ## Note that the inner foreach will filter out ".frag" files from the list for any vert targets, and vice versa.
    $(foreach type,$2,\
        $(foreach file,$(filter-out %.$(if $(findstring vert, $(type)),frag,vert),$1),\
            $(eval $(call spirv_list_rule,$(file),$(type),$3))\
        )\
    )
endef

## All .main/vert/frag files should build
$(eval $(call make_spirv_rules, $(SPIRV_STANDARD_INPUTS), vert frag))

SPIRV_COMMON_WEBGPU_PARAMS = -DTARGET_WGSL -DUSE_WEBGPU_SAMPLERS

## Each of the specialized SPIRV lists have their own associated rules
$(eval $(call make_spirv_rules, $(SPIRV_DRAW_MSAA_INPUTS), noclipdistance_vert, -DDISABLE_CLIP_DISTANCE_FOR_UBERSHADERS))
$(eval $(call make_spirv_rules, \
    $(SPIRV_CLOCKWISE_INPUTS) \
    $(SPIRV_DRAW_ATOMIC_INPUTS) \
    $(SPIRV_DRAW_MSAA_INPUTS), \
    fixedcolor_frag, \
    -DFIXED_FUNCTION_COLOR_OUTPUT))
$(eval $(call make_spirv_rules, \
    $(SPIRV_WEBGPU_INPUTS) \
    $(SPIRV_DRAW_ATOMIC_INPUTS) \
    $(SPIRV_DRAW_MSAA_INPUTS), \
    webgpu_vert, \
    $(SPIRV_COMMON_WEBGPU_PARAMS)))
$(eval $(call make_spirv_rules, \
    $(SPIRV_DRAW_MSAA_INPUTS), \
    webgpu_noclipdistance_vert, \
    $(SPIRV_COMMON_WEBGPU_PARAMS) -DDISABLE_CLIP_DISTANCE_FOR_UBERSHADERS))
$(eval $(call make_spirv_rules, \
    $(WEBGPU_NOSSBO_INPUTS), \
    webgpu_nossbo_vert, \
    $(SPIRV_COMMON_WEBGPU_PARAMS) -DDISABLE_SHADER_STORAGE_BUFFERS))
$(eval $(call make_spirv_rules, \
    $(WEBGPU_NOSSBO_NOCLIPDISTANCE_INPUTS), \
    webgpu_nossbo_noclipdistance_vert, \
    $(SPIRV_COMMON_WEBGPU_PARAMS) -DDISABLE_SHADER_STORAGE_BUFFERS -DDISABLE_CLIP_DISTANCE_FOR_UBERSHADERS))
$(eval $(call make_spirv_rules, \
    $(SPIRV_WEBGPU_INPUTS) \
    $(SPIRV_DRAW_ATOMIC_INPUTS) \
    $(SPIRV_DRAW_MSAA_INPUTS), \
    webgpu_frag, \
    $(SPIRV_COMMON_WEBGPU_PARAMS)))
$(eval $(call make_spirv_rules, \
    $(SPIRV_DRAW_ATOMIC_INPUTS) \
    $(SPIRV_DRAW_MSAA_INPUTS), \
    webgpu_fixedcolor_frag, \
    $(SPIRV_COMMON_WEBGPU_PARAMS) -DFIXED_FUNCTION_COLOR_OUTPUT))


spirv: $(SPIRV_OUTPUTS_HEADERS)
spirv-binary: $(SPIRV_OUTPUTS_BINARY)

## WGSL compilation. Translates WebGPU-flavored SPIR-V binaries to WGSL using
## naga. Some Vulkan-flavored SPIR-V files aren't compatible with WebGPU so we
## filter to the webgpu_* variants only — plus a hand-curated list of standard
## variants that translate cleanly (e.g., shaders without combined samplers).
$(OUT)/wgsl/.: | $(OUT)/.
	@mkdir -p $@

WGSL_OUTPUTS := \
    $(patsubst $(OUT)/spirv/%.spv,$(OUT)/wgsl/%.wgsl, \
        $(filter %.webgpu_vert.spv \
                 %.webgpu_noclipdistance_vert.spv \
                 %.webgpu_nossbo_vert.spv \
                 %.webgpu_nossbo_noclipdistance_vert.spv \
                 %.webgpu_frag.spv \
                 %.webgpu_fixedcolor_frag.spv, \
                 $(SPIRV_OUTPUTS_BINARY))) \
    $(OUT)/wgsl/draw_msaa_stencil.vert.wgsl \
    $(OUT)/wgsl/draw_msaa_stencil.frag.wgsl \
    $(OUT)/wgsl/color_ramp.vert.wgsl \
    $(OUT)/wgsl/color_ramp.frag.wgsl

## Filter naga's noisy "Unknown decoration RelaxedPrecision" warning — WGSL has
## no precision qualifiers, so naga always drops it. bash with pipefail ensures
## a real naga failure still fails the build (pipeline exit status reflects
## naga's, not grep's). The trailing "[ $$? = 1 ]" absorbs grep's "no matches"
## exit code on clean runs where naga emitted nothing.

## TERM=dumb forces a terminfo entry that always exists. CI sometimes sets
## TERM=unknown, which makes naga's terminal-detection code (ncurses-backed)
## abort with "Error opening terminal: unknown" before it can translate.

## --keep-coordinate-space disables naga's default Y-flip on gl_Position. naga
## assumes Vulkan-flavored SPIR-V (Y-down NDC) and flips to OpenGL/ WebGPU's Y-up
## NDC. Our GLSL is already authored for Y-up — gpu.cpp sets
## renderTargetInverseViewportY negative so the math produces WebGPU clip coords
## directly — and the extra flip would render upside down (Dawn's SPIR-V path
## didn't apply it either).
$(OUT)/wgsl/%.wgsl: $(OUT)/spirv/%.spv | $(OUT)/wgsl/.
	@echo "wgsl/$*.wgsl"
	@TERM=dumb bash -o pipefail -c 'naga --keep-coordinate-space "$<" "$@" 2>&1 | { grep -v "Unknown decoration RelaxedPrecision" || [ $$? = 1 ]; }'

## Embed each .wgsl as a C++ raw string literal in a .hpp header that downstream
## code can #include. The variable lives in the wgsl:: namespace and is named
## after the basename with dots replaced by underscores (e.g.
## wgsl::tessellate_webgpu_vert). wgsl_to_header.py minifies the source first
## unless WGSL_FLAGS includes --raw.
WGSL_HEADER_OUTPUTS := $(patsubst %.wgsl,%.hpp,$(WGSL_OUTPUTS))

$(OUT)/wgsl/%.hpp: $(OUT)/wgsl/%.wgsl wgsl_to_header.py | $(OUT)/wgsl/.
	@python3 wgsl_to_header.py $(WGSL_FLAGS) $< $@ $(subst .,_,$(basename $(notdir $<)))

wgsl: $(WGSL_HEADER_OUTPUTS)

## d3d compilation.
.PHONY: $(OUT)/d3d/render_atlas.frag.h

$(OUT)/d3d/.: | $(OUT)/.
	@mkdir -p $@


FXC_DEBUG_FLAG := $(if $(filter --human-readable,$(FLAGS)),/Zi,)

D3D_OUTPUTS := \
	 $(OUT)/d3d/root.sig.h \
	 $(addprefix $(OUT)/, $(patsubst %.hlsl, %.vert.h, $(wildcard d3d/*.hlsl))) \
	 $(addprefix $(OUT)/, $(patsubst %.hlsl, %.frag.h, $(wildcard d3d/*.hlsl))) \
	 $(OUT)/d3d/render_atlas_stroke.frag.h\
	 $(OUT)/d3d/render_atlas_fill.frag.h\

$(OUT)/d3d/%.vert.h: d3d/%.hlsl $(MINIFY_STAMP) | $(OUT)/d3d/.
	@fxc /D VERTEX /I $(OUT) $(FXC_DEBUG_FLAG) /T vs_5_0 /Fh $@ $<

$(OUT)/d3d/%.frag.h: d3d/%.hlsl $(MINIFY_STAMP) | $(OUT)/d3d/.
	@fxc /D FRAGMENT /I $(OUT) $(FXC_DEBUG_FLAG) /T ps_5_0 /Fh  $@ $<

$(OUT)/d3d/render_atlas_stroke.frag.h: d3d/render_atlas.hlsl $(MINIFY_STAMP) | $(OUT)/d3d/.
	@fxc /D FRAGMENT /D ATLAS_FEATHERED_STROKE $(FXC_DEBUG_FLAG)  /I $(OUT) /T ps_5_0 /Fh  $@ $<

$(OUT)/d3d/render_atlas_fill.frag.h: d3d/render_atlas.hlsl $(MINIFY_STAMP) | $(OUT)/d3d/.
	@fxc /D FRAGMENT /D ATLAS_FEATHERED_FILL $(FXC_DEBUG_FLAG)  /I $(OUT) /T ps_5_0 /Fh  $@ $<

$(OUT)/d3d/root.sig.h: d3d/root.sig | $(OUT)/d3d/.
	@fxc /I $(OUT) /T rootsig_1_1 /E ROOT_SIG /Fh   $@ $<

d3d: $(D3D_OUTPUTS)

## Cleaning.
clean:
	@rm -fr out
"###;

/// Return the exact pinned Makefile text without executing any recipe.
pub const fn pinned_source() -> &'static str {
    PINNED_MAKEFILE_SOURCE
}

/// A source assignment retains its spelling, operator, default/expression, and
/// complete continuation lines.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MakeVariable {
    pub source_line: u16,
    pub name: &'static str,
    pub operator: &'static str,
    pub source: &'static str,
}

/// Every assignment/default in the pinned source, in source order.
pub const MAKE_VARIABLES: &[MakeVariable] = &[
    MakeVariable {
        source_line: 6,
        name: "OUT",
        operator: ":=",
        source: "OUT := out/generated",
    },
    MakeVariable {
        source_line: 7,
        name: "FLAGS",
        operator: ":=",
        source: "FLAGS :=",
    },
    MakeVariable {
        source_line: 10,
        name: "MINIFY_INPUTS",
        operator: ":=",
        source: "MINIFY_INPUTS := $(wildcard *.glsl) $(wildcard *.vert) $(wildcard *.frag)",
    },
    MakeVariable {
        source_line: 11,
        name: "MINIFY_EXPORT_OUTPUTS",
        operator: ":=",
        source: "MINIFY_EXPORT_OUTPUTS := $(addprefix $(OUT)/, $(addsuffix .exports.h, $(MINIFY_INPUTS)))",
    },
    MakeVariable {
        source_line: 12,
        name: "MINIFY_GLSL_OUTPUTS",
        operator: ":=",
        source: "MINIFY_GLSL_OUTPUTS := $(addprefix $(OUT)/,\\\n                         $(patsubst %.glsl, %.minified.glsl,\\\n                         $(patsubst %.vert, %.minified.vert,\\\n                         $(patsubst %.frag, %.minified.frag,\\\n                           $(MINIFY_INPUTS)))))",
    },
    MakeVariable {
        source_line: 17,
        name: "MINIFY_HPP_OUTPUTS",
        operator: ":=",
        source: "MINIFY_HPP_OUTPUTS := $(addprefix $(OUT)/, $(addsuffix .hpp, $(MINIFY_INPUTS)))",
    },
    MakeVariable {
        source_line: 18,
        name: "MINIFY_OUTPUTS",
        operator: ":=",
        source: "MINIFY_OUTPUTS := $(MINIFY_EXPORT_OUTPUTS) $(MINIFY_GLSL_OUTPUTS) $(MINIFY_HPP_OUTPUTS)",
    },
    MakeVariable {
        source_line: 19,
        name: "MINIFY_STAMP",
        operator: ":=",
        source: "MINIFY_STAMP := $(OUT)/glsl.stamp",
    },
    MakeVariable {
        source_line: 43,
        name: "DRAW_COMBINATIONS_METAL",
        operator: ":=",
        source: "DRAW_COMBINATIONS_METAL := $(OUT)/draw_combinations.metal",
    },
    MakeVariable {
        source_line: 44,
        name: "METAL_INPUTS",
        operator: ":=",
        source: "METAL_INPUTS := $(wildcard metal/*.metal)",
    },
    MakeVariable {
        source_line: 45,
        name: "METAL_MACOSX_AIR_OUTPUTS",
        operator: ":=",
        source: "METAL_MACOSX_AIR_OUTPUTS := \\\n\t$(addprefix $(OUT)/, $(patsubst metal/%.metal, macosx/%.air, $(METAL_INPUTS)))",
    },
    MakeVariable {
        source_line: 47,
        name: "METAL_IOS_AIR_OUTPUTS",
        operator: ":=",
        source: "METAL_IOS_AIR_OUTPUTS := $(addprefix $(OUT)/, $(patsubst metal/%.metal, ios/%.air, $(METAL_INPUTS)))",
    },
    MakeVariable {
        source_line: 155,
        name: "SPIRV_STANDARD_INPUTS",
        operator: ":=",
        source: "SPIRV_STANDARD_INPUTS := $(wildcard spirv/*.main) \\\n\t\t\t\t $(wildcard spirv/*.vert) \\\n\t\t\t\t $(wildcard spirv/*.frag)",
    },
    MakeVariable {
        source_line: 161,
        name: "SPIRV_WEBGPU_INPUTS",
        operator: ":=",
        source: "SPIRV_WEBGPU_INPUTS := \\\n    spirv/tessellate.main \\\n\tspirv/render_atlas.vert \\\n\tspirv/render_atlas_fill.frag \\\n\tspirv/render_atlas_stroke.frag \\\n\tspirv/blit_texture_as_draw_filtered.main \\\n    spirv/atomic_resolve_coalesced.main \\\n",
    },
    MakeVariable {
        source_line: 171,
        name: "SPIRV_CLOCKWISE_INPUTS",
        operator: ":=",
        source: "SPIRV_CLOCKWISE_INPUTS := \\\n    spirv/draw_clockwise_path.main \\\n    spirv/draw_clockwise_clip.main \\\n    spirv/draw_clockwise_interior_triangles.main \\\n    spirv/draw_clockwise_clip_interior_triangles.main \\\n    spirv/draw_clockwise_atlas_blit.main \\\n    spirv/draw_clockwise_image_mesh.main \\\n    spirv/draw_clockwise_atomic_path.main \\\n    spirv/draw_clockwise_atomic_interior_triangles.main \\\n    spirv/draw_clockwise_atomic_atlas_blit.main \\\n    spirv/draw_clockwise_atomic_image_mesh.main \\\n    spirv/init_clockwise_atomic_workaround.frag \\\n    spirv/draw_clockwise_atomic_clip.frag \\\n    spirv/draw_clockwise_atomic_clip_interior_triangles.frag \\\n    spirv/clear_clockwise_atomic_clip.main \\\n",
    },
    MakeVariable {
        source_line: 189,
        name: "SPIRV_DRAW_ATOMIC_INPUTS",
        operator: ":=",
        source: "SPIRV_DRAW_ATOMIC_INPUTS := \\\n    spirv/atomic_draw_image_mesh.main \\\n    spirv/atomic_draw_image_rect.main \\\n    spirv/atomic_draw_interior_triangles.main \\\n    spirv/atomic_draw_atlas_blit.main \\\n    spirv/atomic_draw_path.main \\\n    spirv/atomic_resolve.main \\\n    spirv/atomic_init.main \\\n",
    },
    MakeVariable {
        source_line: 200,
        name: "SPIRV_DRAW_MSAA_INPUTS",
        operator: ":=",
        source: "SPIRV_DRAW_MSAA_INPUTS := \\\n    spirv/draw_msaa_atlas_blit.main \\\n    spirv/draw_msaa_image_mesh.main \\\n    spirv/draw_msaa_path.main \\\n    spirv/draw_msaa_stencil.main \\\n",
    },
    MakeVariable {
        source_line: 209,
        name: "WEBGPU_NOSSBO_NOCLIPDISTANCE_INPUTS",
        operator: ":=",
        source: "WEBGPU_NOSSBO_NOCLIPDISTANCE_INPUTS := \\\n    spirv/draw_msaa_path.main \\\n    spirv/draw_msaa_atlas_blit.main",
    },
    MakeVariable {
        source_line: 212,
        name: "WEBGPU_NOSSBO_INPUTS",
        operator: ":=",
        source: "WEBGPU_NOSSBO_INPUTS := \\\n    $(WEBGPU_NOSSBO_NOCLIPDISTANCE_INPUTS) \\\n    spirv/tessellate.main \\\n    spirv/render_atlas.vert",
    },
    MakeVariable {
        source_line: 218,
        name: "spirv_typed_filename",
        operator: "=",
        source: "spirv_typed_filename = $(basename $1).$2",
    },
    MakeVariable {
        source_line: 219,
        name: "spirv_out_filename_no_ext",
        operator: "=",
        source: "spirv_out_filename_no_ext = $(OUT)/$(call spirv_typed_filename,$1,$2)",
    },
    MakeVariable {
        source_line: 220,
        name: "spirv_type",
        operator: "=",
        source: "spirv_type = $(lastword $(subst _, ,$2))",
    },
    MakeVariable {
        source_line: 221,
        name: "spirv_is_vert",
        operator: "=",
        source: "spirv_is_vert = $(findstring vert,$(spirv_type))",
    },
    MakeVariable {
        source_line: 222,
        name: "spirv_is_webgpu",
        operator: "=",
        source: "spirv_is_webgpu = $(findstring webgpu,$2)",
    },
    MakeVariable {
        source_line: 223,
        name: "spirv_is_atomic_or_clockwise_atomic",
        operator: "=",
        source: "spirv_is_atomic_or_clockwise_atomic = $(findstring atomic,$1)",
    },
    MakeVariable {
        source_line: 224,
        name: "spirv_is_not_clockwise_atomic",
        operator: "=",
        source: "spirv_is_not_clockwise_atomic = $(if $(findstring clockwise_atomic,$1),,1)",
    },
    MakeVariable {
        source_line: 225,
        name: "spirv_is_clockwise",
        operator: "=",
        source: "spirv_is_clockwise = $(and $(findstring clockwise,$1), $(call spirv_is_not_clockwise_atomic,$1))",
    },
    MakeVariable {
        source_line: 238,
        name: "SPIRV_STANDARD_VERT_OPT_PARAMS",
        operator: "=",
        source: "SPIRV_STANDARD_VERT_OPT_PARAMS = -O",
    },
    MakeVariable {
        source_line: 255,
        name: "spirv_frag_opt_params",
        operator: "=",
        source: "spirv_frag_opt_params = \\\n\t$(if $(spirv_is_webgpu),-O, \\\n\t\t$(if $(spirv_is_atomic_or_clockwise_atomic), \\\n\t\t\t--preserve-bindings \\\n\t\t\t--preserve-interface \\\n\t\t\t--wrap-opkill \\\n\t\t\t--simplify-instructions \\\n\t\t\t--eliminate-dead-branches \\\n\t\t\t--merge-return \\\n\t\t\t--inline-entry-points-exhaustive \\\n\t\t\t--eliminate-dead-inserts \\\n\t\t\t--eliminate-dead-members \\\n\t\t\t--merge-blocks \\\n\t\t\t--redundancy-elimination \\\n\t\t\t--cfg-cleanup \\\n\t\t\t--eliminate-dead-const \\\n\t\t\t--eliminate-dead-variables \\\n\t\t\t--eliminate-dead-functions \\\n\t\t\t--eliminate-dead-code-aggressive \\\n\t\t, \\\n\t\t\t--wrap-opkill \\\n\t\t\t--eliminate-dead-branches \\\n\t\t\t--merge-return \\\n\t\t\t--inline-entry-points-exhaustive \\\n\t\t\t--eliminate-dead-functions \\\n\t\t\t--eliminate-dead-code-aggressive \\\n\t\t\t--private-to-local \\\n\t\t\t--eliminate-local-single-block \\\n\t\t\t--eliminate-local-single-store \\\n\t\t\t--eliminate-dead-code-aggressive \\\n\t\t\t--scalar-replacement=100 \\\n\t\t\t--convert-local-access-chains \\\n\t\t\t--eliminate-local-single-block \\\n\t\t\t--eliminate-local-single-store \\\n\t\t\t--eliminate-dead-code-aggressive \\\n\t\t\t--ssa-rewrite \\\n\t\t\t--eliminate-dead-code-aggressive \\\n\t\t\t--ccp \\\n\t\t\t--eliminate-dead-code-aggressive \\\n\t\t\t--loop-unroll \\\n\t\t\t--eliminate-dead-branches \\\n\t\t\t--redundancy-elimination \\\n\t\t\t--combine-access-chains \\\n\t\t\t--scalar-replacement=100 \\\n\t\t\t--convert-local-access-chains \\\n\t\t\t--eliminate-local-single-block \\\n\t\t\t--eliminate-local-single-store \\\n\t\t\t--eliminate-dead-code-aggressive \\\n\t\t\t--ssa-rewrite \\\n\t\t\t--eliminate-dead-code-aggressive \\\n\t\t\t--vector-dce \\\n\t\t\t--eliminate-dead-inserts \\\n\t\t\t--eliminate-dead-branches \\\n\t\t\t--if-conversion \\\n\t\t\t--copy-propagate-arrays \\\n\t\t\t--reduce-load-size \\\n\t\t\t--eliminate-dead-code-aggressive \\\n\t\t\t--merge-blocks \\\n\t\t\t--redundancy-elimination \\\n\t\t\t--eliminate-dead-branches \\\n\t\t\t--merge-blocks \\\n\t\t) \\\n\t)",
    },
    MakeVariable {
        source_line: 320,
        name: "spirv_frag_params",
        operator: "=",
        source: "spirv_frag_params = $(if $(spirv_is_webgpu), \\\n                        -DPLS_IMPL_STORAGE_BUFFER, \\\n\t\t\t\t\t$(if $(spirv_is_clockwise), \\\n\t\t\t\t\t\t-DPLS_IMPL_STORAGE_TEXTURE, \\\n\t\t\t\t\t\t-DPLS_IMPL_SUBPASS_LOAD))",
    },
    MakeVariable {
        source_line: 329,
        name: "spirv_opt_params",
        operator: "=",
        source: "spirv_opt_params = \\\n\t$(if $(spirv_is_vert), \\\n\t\t$(SPIRV_STANDARD_VERT_OPT_PARAMS), \\\n\t\t$(spirv_frag_opt_params) \\\n\t) \\\n",
    },
    MakeVariable {
        source_line: 370,
        name: "SPIRV_COMMON_WEBGPU_PARAMS",
        operator: "=",
        source: "SPIRV_COMMON_WEBGPU_PARAMS = -DTARGET_WGSL -DUSE_WEBGPU_SAMPLERS",
    },
    MakeVariable {
        source_line: 421,
        name: "WGSL_OUTPUTS",
        operator: ":=",
        source: "WGSL_OUTPUTS := \\\n    $(patsubst $(OUT)/spirv/%.spv,$(OUT)/wgsl/%.wgsl, \\\n        $(filter %.webgpu_vert.spv \\\n                 %.webgpu_noclipdistance_vert.spv \\\n                 %.webgpu_nossbo_vert.spv \\\n                 %.webgpu_nossbo_noclipdistance_vert.spv \\\n                 %.webgpu_frag.spv \\\n                 %.webgpu_fixedcolor_frag.spv, \\\n                 $(SPIRV_OUTPUTS_BINARY))) \\\n    $(OUT)/wgsl/draw_msaa_stencil.vert.wgsl \\\n    $(OUT)/wgsl/draw_msaa_stencil.frag.wgsl \\\n    $(OUT)/wgsl/color_ramp.vert.wgsl \\\n    $(OUT)/wgsl/color_ramp.frag.wgsl",
    },
    MakeVariable {
        source_line: 460,
        name: "WGSL_HEADER_OUTPUTS",
        operator: ":=",
        source: "WGSL_HEADER_OUTPUTS := $(patsubst %.wgsl,%.hpp,$(WGSL_OUTPUTS))",
    },
    MakeVariable {
        source_line: 474,
        name: "FXC_DEBUG_FLAG",
        operator: ":=",
        source: "FXC_DEBUG_FLAG := $(if $(filter --human-readable,$(FLAGS)),/Zi,)",
    },
    MakeVariable {
        source_line: 476,
        name: "D3D_OUTPUTS",
        operator: ":=",
        source: "D3D_OUTPUTS := \\\n\t $(OUT)/d3d/root.sig.h \\\n\t $(addprefix $(OUT)/, $(patsubst %.hlsl, %.vert.h, $(wildcard d3d/*.hlsl))) \\\n\t $(addprefix $(OUT)/, $(patsubst %.hlsl, %.frag.h, $(wildcard d3d/*.hlsl))) \\\n\t $(OUT)/d3d/render_atlas_stroke.frag.h\\\n\t $(OUT)/d3d/render_atlas_fill.frag.h\\\n",
    },
];

/// A Make target declaration retains its complete dependency expression.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MakeRule {
    pub source_line: u16,
    pub family: &'static str,
    pub target: &'static str,
    pub dependencies: &'static str,
    pub declaration: &'static str,
}

/// Every target declaration (including pattern, alias, directory, and phony
/// rules), in source order.
pub const MAKE_RULES: &[MakeRule] = &[
    MakeRule {
        source_line: 21,
        family: "minify",
        target: "minify",
        dependencies: "$(MINIFY_OUTPUTS)",
        declaration: "minify: $(MINIFY_OUTPUTS)",
    },
    MakeRule {
        source_line: 25,
        family: "minify",
        target: "$(MINIFY_OUTPUTS)",
        dependencies: "$(MINIFY_STAMP)",
        declaration: "$(MINIFY_OUTPUTS): $(MINIFY_STAMP)",
    },
    MakeRule {
        source_line: 29,
        family: "minify",
        target: "$(MINIFY_STAMP)",
        dependencies: "$(MINIFY_INPUTS) minify.py",
        declaration: "$(MINIFY_STAMP): $(MINIFY_INPUTS) minify.py",
    },
    MakeRule {
        source_line: 33,
        family: "minify",
        target: "$(OUT)/.",
        dependencies: "",
        declaration: "$(OUT)/.:",
    },
    MakeRule {
        source_line: 37,
        family: "minify",
        target: "$(OUT)/ios/.",
        dependencies: "| $(OUT)/.",
        declaration: "$(OUT)/ios/.: | $(OUT)/.",
    },
    MakeRule {
        source_line: 40,
        family: "minify",
        target: "$(OUT)/macosx/.",
        dependencies: "| $(OUT)/.",
        declaration: "$(OUT)/macosx/.: | $(OUT)/.",
    },
    MakeRule {
        source_line: 49,
        family: "metal",
        target: "$(DRAW_COMBINATIONS_METAL)",
        dependencies: "metal/generate_draw_combinations.py | $(OUT)/.",
        declaration: "$(DRAW_COMBINATIONS_METAL): metal/generate_draw_combinations.py | $(OUT)/.",
    },
    MakeRule {
        source_line: 52,
        family: "metal",
        target: "rive_pls_macosx_metallib",
        dependencies: "$(OUT)/rive_pls_macosx.metallib.c",
        declaration: "rive_pls_macosx_metallib: $(OUT)/rive_pls_macosx.metallib.c",
    },
    MakeRule {
        source_line: 53,
        family: "metal",
        target: "rive_pls_ios_metallib",
        dependencies: "$(OUT)/rive_pls_ios.metallib.c",
        declaration: "rive_pls_ios_metallib: $(OUT)/rive_pls_ios.metallib.c",
    },
    MakeRule {
        source_line: 54,
        family: "metal",
        target: "rive_pls_ios_simulator_metallib",
        dependencies: "$(OUT)/rive_pls_ios_simulator.metallib.c",
        declaration: "rive_pls_ios_simulator_metallib: $(OUT)/rive_pls_ios_simulator.metallib.c",
    },
    MakeRule {
        source_line: 55,
        family: "metal",
        target: "rive_renderer_xros_metallib",
        dependencies: "$(OUT)/rive_renderer_xros.metallib.c",
        declaration: "rive_renderer_xros_metallib: $(OUT)/rive_renderer_xros.metallib.c",
    },
    MakeRule {
        source_line: 56,
        family: "metal",
        target: "rive_renderer_xros_simulator_metallib",
        dependencies: "$(OUT)/rive_renderer_xros_simulator.metallib.c",
        declaration: "rive_renderer_xros_simulator_metallib: $(OUT)/rive_renderer_xros_simulator.metallib.c",
    },
    MakeRule {
        source_line: 57,
        family: "metal",
        target: "rive_renderer_appletvos_metallib",
        dependencies: "$(OUT)/rive_renderer_appletvos.metallib.c",
        declaration: "rive_renderer_appletvos_metallib: $(OUT)/rive_renderer_appletvos.metallib.c",
    },
    MakeRule {
        source_line: 58,
        family: "metal",
        target: "rive_renderer_appletvsimulator_metallib",
        dependencies: "$(OUT)/rive_renderer_appletvsimulator.metallib.c",
        declaration: "rive_renderer_appletvsimulator_metallib: $(OUT)/rive_renderer_appletvsimulator.metallib.c",
    },
    MakeRule {
        source_line: 62,
        family: "metal",
        target: "$(OUT)/macosx/rive_pls_macosx.metallib",
        dependencies: "$(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/macosx/.",
        declaration: "$(OUT)/macosx/rive_pls_macosx.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/macosx/.",
    },
    MakeRule {
        source_line: 71,
        family: "metal",
        target: "$(OUT)/rive_pls_macosx.metallib.c",
        dependencies: "$(OUT)/macosx/rive_pls_macosx.metallib",
        declaration: "$(OUT)/rive_pls_macosx.metallib.c: $(OUT)/macosx/rive_pls_macosx.metallib",
    },
    MakeRule {
        source_line: 76,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios.metallib",
        dependencies: "$(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.",
        declaration: "$(OUT)/ios/rive_pls_ios.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.",
    },
    MakeRule {
        source_line: 85,
        family: "metal",
        target: "$(OUT)/rive_pls_ios.metallib.c",
        dependencies: "$(OUT)/ios/rive_pls_ios.metallib",
        declaration: "$(OUT)/rive_pls_ios.metallib.c: $(OUT)/ios/rive_pls_ios.metallib",
    },
    MakeRule {
        source_line: 88,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios_simulator.metallib",
        dependencies: "$(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.",
        declaration: "$(OUT)/ios/rive_pls_ios_simulator.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.",
    },
    MakeRule {
        source_line: 97,
        family: "metal",
        target: "$(OUT)/rive_pls_ios_simulator.metallib.c",
        dependencies: "$(OUT)/ios/rive_pls_ios_simulator.metallib",
        declaration: "$(OUT)/rive_pls_ios_simulator.metallib.c: $(OUT)/ios/rive_pls_ios_simulator.metallib",
    },
    MakeRule {
        source_line: 100,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros.metallib",
        dependencies: "$(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.",
        declaration: "$(OUT)/ios/rive_renderer_xros.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.",
    },
    MakeRule {
        source_line: 109,
        family: "metal",
        target: "$(OUT)/rive_renderer_xros.metallib.c",
        dependencies: "$(OUT)/ios/rive_renderer_xros.metallib",
        declaration: "$(OUT)/rive_renderer_xros.metallib.c: $(OUT)/ios/rive_renderer_xros.metallib",
    },
    MakeRule {
        source_line: 112,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros_simulator.metallib",
        dependencies: "$(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.",
        declaration: "$(OUT)/ios/rive_renderer_xros_simulator.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.",
    },
    MakeRule {
        source_line: 121,
        family: "metal",
        target: "$(OUT)/rive_renderer_xros_simulator.metallib.c",
        dependencies: "$(OUT)/ios/rive_renderer_xros_simulator.metallib",
        declaration: "$(OUT)/rive_renderer_xros_simulator.metallib.c: $(OUT)/ios/rive_renderer_xros_simulator.metallib",
    },
    MakeRule {
        source_line: 124,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvos.metallib",
        dependencies: "$(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.",
        declaration: "$(OUT)/ios/rive_renderer_appletvos.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.",
    },
    MakeRule {
        source_line: 133,
        family: "metal",
        target: "$(OUT)/rive_renderer_appletvos.metallib.c",
        dependencies: "$(OUT)/ios/rive_renderer_appletvos.metallib",
        declaration: "$(OUT)/rive_renderer_appletvos.metallib.c: $(OUT)/ios/rive_renderer_appletvos.metallib",
    },
    MakeRule {
        source_line: 136,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvsimulator.metallib",
        dependencies: "$(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.",
        declaration: "$(OUT)/ios/rive_renderer_appletvsimulator.metallib: $(MINIFY_GLSL_OUTPUTS) $(METAL_INPUTS) $(DRAW_COMBINATIONS_METAL) | $(OUT)/ios/.",
    },
    MakeRule {
        source_line: 145,
        family: "metal",
        target: "$(OUT)/rive_renderer_appletvsimulator.metallib.c",
        dependencies: "$(OUT)/ios/rive_renderer_appletvsimulator.metallib",
        declaration: "$(OUT)/rive_renderer_appletvsimulator.metallib.c: $(OUT)/ios/rive_renderer_appletvsimulator.metallib",
    },
    MakeRule {
        source_line: 151,
        family: "spirv",
        target: "$(OUT)/spirv/.",
        dependencies: "| $(OUT)/.",
        declaration: "$(OUT)/spirv/.: | $(OUT)/.",
    },
    MakeRule {
        source_line: 340,
        family: "spirv",
        target: "$(spirv_out_filename_no_ext).spv",
        dependencies: "$1 $(MINIFY_STAMP) | $(OUT)/spirv/.",
        declaration: "  $(spirv_out_filename_no_ext).spv: $1 $(MINIFY_STAMP) | $(OUT)/spirv/.",
    },
    MakeRule {
        source_line: 348,
        family: "spirv",
        target: "$(spirv_out_filename_no_ext).h",
        dependencies: "$(spirv_out_filename_no_ext).spv",
        declaration: "  $(spirv_out_filename_no_ext).h: $(spirv_out_filename_no_ext).spv",
    },
    MakeRule {
        source_line: 411,
        family: "wgsl",
        target: "spirv",
        dependencies: "$(SPIRV_OUTPUTS_HEADERS)",
        declaration: "spirv: $(SPIRV_OUTPUTS_HEADERS)",
    },
    MakeRule {
        source_line: 412,
        family: "wgsl",
        target: "spirv-binary",
        dependencies: "$(SPIRV_OUTPUTS_BINARY)",
        declaration: "spirv-binary: $(SPIRV_OUTPUTS_BINARY)",
    },
    MakeRule {
        source_line: 418,
        family: "wgsl",
        target: "$(OUT)/wgsl/.",
        dependencies: "| $(OUT)/.",
        declaration: "$(OUT)/wgsl/.: | $(OUT)/.",
    },
    MakeRule {
        source_line: 451,
        family: "wgsl",
        target: "$(OUT)/wgsl/%.wgsl",
        dependencies: "$(OUT)/spirv/%.spv | $(OUT)/wgsl/.",
        declaration: "$(OUT)/wgsl/%.wgsl: $(OUT)/spirv/%.spv | $(OUT)/wgsl/.",
    },
    MakeRule {
        source_line: 462,
        family: "wgsl",
        target: "$(OUT)/wgsl/%.hpp",
        dependencies: "$(OUT)/wgsl/%.wgsl wgsl_to_header.py | $(OUT)/wgsl/.",
        declaration: "$(OUT)/wgsl/%.hpp: $(OUT)/wgsl/%.wgsl wgsl_to_header.py | $(OUT)/wgsl/.",
    },
    MakeRule {
        source_line: 465,
        family: "wgsl",
        target: "wgsl",
        dependencies: "$(WGSL_HEADER_OUTPUTS)",
        declaration: "wgsl: $(WGSL_HEADER_OUTPUTS)",
    },
    MakeRule {
        source_line: 468,
        family: "wgsl",
        target: ".PHONY",
        dependencies: "$(OUT)/d3d/render_atlas.frag.h",
        declaration: ".PHONY: $(OUT)/d3d/render_atlas.frag.h",
    },
    MakeRule {
        source_line: 470,
        family: "wgsl",
        target: "$(OUT)/d3d/.",
        dependencies: "| $(OUT)/.",
        declaration: "$(OUT)/d3d/.: | $(OUT)/.",
    },
    MakeRule {
        source_line: 483,
        family: "d3d",
        target: "$(OUT)/d3d/%.vert.h",
        dependencies: "d3d/%.hlsl $(MINIFY_STAMP) | $(OUT)/d3d/.",
        declaration: "$(OUT)/d3d/%.vert.h: d3d/%.hlsl $(MINIFY_STAMP) | $(OUT)/d3d/.",
    },
    MakeRule {
        source_line: 486,
        family: "d3d",
        target: "$(OUT)/d3d/%.frag.h",
        dependencies: "d3d/%.hlsl $(MINIFY_STAMP) | $(OUT)/d3d/.",
        declaration: "$(OUT)/d3d/%.frag.h: d3d/%.hlsl $(MINIFY_STAMP) | $(OUT)/d3d/.",
    },
    MakeRule {
        source_line: 489,
        family: "d3d",
        target: "$(OUT)/d3d/render_atlas_stroke.frag.h",
        dependencies: "d3d/render_atlas.hlsl $(MINIFY_STAMP) | $(OUT)/d3d/.",
        declaration: "$(OUT)/d3d/render_atlas_stroke.frag.h: d3d/render_atlas.hlsl $(MINIFY_STAMP) | $(OUT)/d3d/.",
    },
    MakeRule {
        source_line: 492,
        family: "d3d",
        target: "$(OUT)/d3d/render_atlas_fill.frag.h",
        dependencies: "d3d/render_atlas.hlsl $(MINIFY_STAMP) | $(OUT)/d3d/.",
        declaration: "$(OUT)/d3d/render_atlas_fill.frag.h: d3d/render_atlas.hlsl $(MINIFY_STAMP) | $(OUT)/d3d/.",
    },
    MakeRule {
        source_line: 495,
        family: "d3d",
        target: "$(OUT)/d3d/root.sig.h",
        dependencies: "d3d/root.sig | $(OUT)/d3d/.",
        declaration: "$(OUT)/d3d/root.sig.h: d3d/root.sig | $(OUT)/d3d/.",
    },
    MakeRule {
        source_line: 498,
        family: "d3d",
        target: "d3d",
        dependencies: "$(D3D_OUTPUTS)",
        declaration: "d3d: $(D3D_OUTPUTS)",
    },
    MakeRule {
        source_line: 501,
        family: "d3d",
        target: "clean",
        dependencies: "",
        declaration: "clean:",
    },
];

/// Each recipe command is retained separately so command ordering and
/// continuation lines remain visible without running the toolchain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MakeRecipe {
    pub source_line: u16,
    pub family: &'static str,
    pub target: &'static str,
    pub command: &'static str,
}

pub const MAKE_RECIPES: &[MakeRecipe] = &[
    MakeRecipe {
        source_line: 26,
        family: "minify",
        target: "$(MINIFY_OUTPUTS)",
        command: "@test -f $@ || rm -f $(MINIFY_STAMP)",
    },
    MakeRecipe {
        source_line: 27,
        family: "minify",
        target: "$(MINIFY_OUTPUTS)",
        command: "@test -f $@ || \"$(MAKE)\" $(AM_MAKEFLAGS) $(MINIFY_STAMP)",
    },
    MakeRecipe {
        source_line: 30,
        family: "minify",
        target: "$(MINIFY_STAMP)",
        command: "python3 minify.py $(FLAGS) -o $(OUT) $(MINIFY_INPUTS)",
    },
    MakeRecipe {
        source_line: 31,
        family: "minify",
        target: "$(MINIFY_STAMP)",
        command: "@touch $(MINIFY_STAMP)",
    },
    MakeRecipe {
        source_line: 34,
        family: "minify",
        target: "$(OUT)/.",
        command: "@mkdir -p $@",
    },
    MakeRecipe {
        source_line: 38,
        family: "minify",
        target: "$(OUT)/ios/.",
        command: "@mkdir -p $@",
    },
    MakeRecipe {
        source_line: 41,
        family: "minify",
        target: "$(OUT)/macosx/.",
        command: "@mkdir -p $@",
    },
    MakeRecipe {
        source_line: 50,
        family: "metal",
        target: "$(DRAW_COMBINATIONS_METAL)",
        command: "python3 metal/generate_draw_combinations.py $(DRAW_COMBINATIONS_METAL)",
    },
    MakeRecipe {
        source_line: 63,
        family: "metal",
        target: "$(OUT)/macosx/rive_pls_macosx.metallib",
        command: "$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeRecipe {
        source_line: 64,
        family: "metal",
        target: "$(OUT)/macosx/rive_pls_macosx.metallib",
        command: "\txcrun -sdk macosx metal -std=macos-metal2.3 \\",
    },
    MakeRecipe {
        source_line: 65,
        family: "metal",
        target: "$(OUT)/macosx/rive_pls_macosx.metallib",
        command: "\t-mmacosx-version-min=11.0 \\",
    },
    MakeRecipe {
        source_line: 66,
        family: "metal",
        target: "$(OUT)/macosx/rive_pls_macosx.metallib",
        command: "\t-I$(OUT) -ffast-math -ffp-contract=fast -fpreserve-invariance -fvisibility=hidden \\",
    },
    MakeRecipe {
        source_line: 67,
        family: "metal",
        target: "$(OUT)/macosx/rive_pls_macosx.metallib",
        command: "\t-c $(FILE) \\",
    },
    MakeRecipe {
        source_line: 68,
        family: "metal",
        target: "$(OUT)/macosx/rive_pls_macosx.metallib",
        command: "\t-o $(patsubst metal/%.metal, $(OUT)/macosx/%.air, $(FILE));)",
    },
    MakeRecipe {
        source_line: 69,
        family: "metal",
        target: "$(OUT)/macosx/rive_pls_macosx.metallib",
        command: "xcrun -sdk macosx metallib $(METAL_MACOSX_AIR_OUTPUTS) -o $(OUT)/macosx/rive_pls_macosx.metallib",
    },
    MakeRecipe {
        source_line: 72,
        family: "metal",
        target: "$(OUT)/rive_pls_macosx.metallib.c",
        command: "xxd -i -n rive_pls_macosx_metallib \\",
    },
    MakeRecipe {
        source_line: 73,
        family: "metal",
        target: "$(OUT)/rive_pls_macosx.metallib.c",
        command: "\t$(OUT)/macosx/rive_pls_macosx.metallib \\",
    },
    MakeRecipe {
        source_line: 74,
        family: "metal",
        target: "$(OUT)/rive_pls_macosx.metallib.c",
        command: "\t$(OUT)/rive_pls_macosx.metallib.c",
    },
    MakeRecipe {
        source_line: 77,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios.metallib",
        command: "$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeRecipe {
        source_line: 78,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios.metallib",
        command: "\txcrun -sdk iphoneos metal -std=ios-metal2.2 \\",
    },
    MakeRecipe {
        source_line: 79,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios.metallib",
        command: "\t-I$(OUT) -mios-version-min=13 -ffast-math -ffp-contract=fast -fpreserve-invariance \\",
    },
    MakeRecipe {
        source_line: 80,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios.metallib",
        command: "\t-fvisibility=hidden \\",
    },
    MakeRecipe {
        source_line: 81,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios.metallib",
        command: "\t-c $(FILE) \\",
    },
    MakeRecipe {
        source_line: 82,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios.metallib",
        command: "\t-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)",
    },
    MakeRecipe {
        source_line: 83,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios.metallib",
        command: "xcrun -sdk iphoneos metallib $(METAL_IOS_AIR_OUTPUTS) -o $(OUT)/ios/rive_pls_ios.metallib",
    },
    MakeRecipe {
        source_line: 86,
        family: "metal",
        target: "$(OUT)/rive_pls_ios.metallib.c",
        command: "xxd -i -n rive_pls_ios_metallib $(OUT)/ios/rive_pls_ios.metallib $(OUT)/rive_pls_ios.metallib.c",
    },
    MakeRecipe {
        source_line: 89,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios_simulator.metallib",
        command: "$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeRecipe {
        source_line: 90,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios_simulator.metallib",
        command: "\txcrun -sdk iphonesimulator metal -std=ios-metal2.2 \\",
    },
    MakeRecipe {
        source_line: 91,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios_simulator.metallib",
        command: "\t-I$(OUT) -miphonesimulator-version-min=13 -ffast-math -ffp-contract=fast -fpreserve-invariance \\",
    },
    MakeRecipe {
        source_line: 92,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios_simulator.metallib",
        command: "\t-fvisibility=hidden \\",
    },
    MakeRecipe {
        source_line: 93,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios_simulator.metallib",
        command: "\t-c $(FILE) \\",
    },
    MakeRecipe {
        source_line: 94,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios_simulator.metallib",
        command: "\t-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)",
    },
    MakeRecipe {
        source_line: 95,
        family: "metal",
        target: "$(OUT)/ios/rive_pls_ios_simulator.metallib",
        command: "xcrun -sdk iphonesimulator metallib $(METAL_IOS_AIR_OUTPUTS) -o $(OUT)/ios/rive_pls_ios_simulator.metallib",
    },
    MakeRecipe {
        source_line: 98,
        family: "metal",
        target: "$(OUT)/rive_pls_ios_simulator.metallib.c",
        command: "xxd -i -n rive_pls_ios_simulator_metallib $(OUT)/ios/rive_pls_ios_simulator.metallib $(OUT)/rive_pls_ios_simulator.metallib.c",
    },
    MakeRecipe {
        source_line: 101,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros.metallib",
        command: "$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeRecipe {
        source_line: 102,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros.metallib",
        command: "\txcrun -sdk xros metal -std=metal3.1 \\",
    },
    MakeRecipe {
        source_line: 103,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros.metallib",
        command: "\t-I$(OUT) --target=air64-apple-xros1.0 -ffast-math -ffp-contract=fast -fpreserve-invariance \\",
    },
    MakeRecipe {
        source_line: 104,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros.metallib",
        command: "\t-fvisibility=hidden \\",
    },
    MakeRecipe {
        source_line: 105,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros.metallib",
        command: "\t-c $(FILE) \\",
    },
    MakeRecipe {
        source_line: 106,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros.metallib",
        command: "\t-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)",
    },
    MakeRecipe {
        source_line: 107,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros.metallib",
        command: "xcrun -sdk xros metallib $(METAL_IOS_AIR_OUTPUTS) -o $(OUT)/ios/rive_renderer_xros.metallib",
    },
    MakeRecipe {
        source_line: 110,
        family: "metal",
        target: "$(OUT)/rive_renderer_xros.metallib.c",
        command: "xxd -i -n rive_renderer_xros_metallib $(OUT)/ios/rive_renderer_xros.metallib $(OUT)/rive_renderer_xros.metallib.c",
    },
    MakeRecipe {
        source_line: 113,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros_simulator.metallib",
        command: "$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeRecipe {
        source_line: 114,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros_simulator.metallib",
        command: "\txcrun -sdk xrsimulator metal -std=metal3.1 \\",
    },
    MakeRecipe {
        source_line: 115,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros_simulator.metallib",
        command: "\t-I$(OUT) --target=air64-apple-xros1.0-simulator -ffast-math -ffp-contract=fast -fpreserve-invariance \\",
    },
    MakeRecipe {
        source_line: 116,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros_simulator.metallib",
        command: "\t-fvisibility=hidden \\",
    },
    MakeRecipe {
        source_line: 117,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros_simulator.metallib",
        command: "\t-c $(FILE) \\",
    },
    MakeRecipe {
        source_line: 118,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros_simulator.metallib",
        command: "\t-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)",
    },
    MakeRecipe {
        source_line: 119,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_xros_simulator.metallib",
        command: "xcrun -sdk xrsimulator metallib $(METAL_IOS_AIR_OUTPUTS) -o $(OUT)/ios/rive_renderer_xros_simulator.metallib",
    },
    MakeRecipe {
        source_line: 122,
        family: "metal",
        target: "$(OUT)/rive_renderer_xros_simulator.metallib.c",
        command: "xxd -i -n rive_renderer_xros_simulator_metallib $(OUT)/ios/rive_renderer_xros_simulator.metallib $(OUT)/rive_renderer_xros_simulator.metallib.c",
    },
    MakeRecipe {
        source_line: 125,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvos.metallib",
        command: "$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeRecipe {
        source_line: 126,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvos.metallib",
        command: "\txcrun -sdk appletvos metal -std=metal3.0 \\",
    },
    MakeRecipe {
        source_line: 127,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvos.metallib",
        command: "\t-I$(OUT) -mappletvos-version-min=16.0 -ffast-math -ffp-contract=fast -fpreserve-invariance \\",
    },
    MakeRecipe {
        source_line: 128,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvos.metallib",
        command: "\t-fvisibility=hidden \\",
    },
    MakeRecipe {
        source_line: 129,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvos.metallib",
        command: "\t-c $(FILE) \\",
    },
    MakeRecipe {
        source_line: 130,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvos.metallib",
        command: "\t-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)",
    },
    MakeRecipe {
        source_line: 131,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvos.metallib",
        command: "xcrun -sdk appletvos metallib $(METAL_IOS_AIR_OUTPUTS) -o $(OUT)/ios/rive_renderer_appletvos.metallib",
    },
    MakeRecipe {
        source_line: 134,
        family: "metal",
        target: "$(OUT)/rive_renderer_appletvos.metallib.c",
        command: "xxd -i -n rive_renderer_appletvos_metallib $(OUT)/ios/rive_renderer_appletvos.metallib $(OUT)/rive_renderer_appletvos.metallib.c",
    },
    MakeRecipe {
        source_line: 137,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvsimulator.metallib",
        command: "$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeRecipe {
        source_line: 138,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvsimulator.metallib",
        command: "\txcrun -sdk appletvsimulator metal -std=metal3.0 \\",
    },
    MakeRecipe {
        source_line: 139,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvsimulator.metallib",
        command: "\t-I$(OUT) -mappletvsimulator-version-min=16.0 -ffast-math -ffp-contract=fast -fpreserve-invariance \\",
    },
    MakeRecipe {
        source_line: 140,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvsimulator.metallib",
        command: "\t-fvisibility=hidden \\",
    },
    MakeRecipe {
        source_line: 141,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvsimulator.metallib",
        command: "\t-c $(FILE) \\",
    },
    MakeRecipe {
        source_line: 142,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvsimulator.metallib",
        command: "\t-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)",
    },
    MakeRecipe {
        source_line: 143,
        family: "metal",
        target: "$(OUT)/ios/rive_renderer_appletvsimulator.metallib",
        command: "xcrun -sdk appletvsimulator metallib $(METAL_IOS_AIR_OUTPUTS) -o $(OUT)/ios/rive_renderer_appletvsimulator.metallib",
    },
    MakeRecipe {
        source_line: 146,
        family: "metal",
        target: "$(OUT)/rive_renderer_appletvsimulator.metallib.c",
        command: "xxd -i -n rive_renderer_appletvsimulator_metallib $(OUT)/ios/rive_renderer_appletvsimulator.metallib $(OUT)/rive_renderer_appletvsimulator.metallib.c",
    },
    MakeRecipe {
        source_line: 152,
        family: "spirv",
        target: "$(OUT)/spirv/.",
        command: "@mkdir -p $@",
    },
    MakeRecipe {
        source_line: 341,
        family: "spirv",
        target: "$(spirv_out_filename_no_ext).spv",
        command: "@glslangValidator -S $(spirv_type) -DTARGET_SPIRV \\",
    },
    MakeRecipe {
        source_line: 342,
        family: "spirv",
        target: "$(spirv_out_filename_no_ext).spv",
        command: "\t$(if $(spirv_is_vert), -DVERTEX, -DFRAGMENT $(spirv_frag_params)) \\",
    },
    MakeRecipe {
        source_line: 343,
        family: "spirv",
        target: "$(spirv_out_filename_no_ext).spv",
        command: "\t-I$(OUT)  -V $3 -o $(spirv_out_filename_no_ext).spv.unoptimized $1",
    },
    MakeRecipe {
        source_line: 344,
        family: "spirv",
        target: "$(spirv_out_filename_no_ext).spv",
        command: "@spirv-opt --preserve-bindings --preserve-interface $(spirv_opt_params) \\",
    },
    MakeRecipe {
        source_line: 345,
        family: "spirv",
        target: "$(spirv_out_filename_no_ext).spv",
        command: "\t$(spirv_out_filename_no_ext).spv.unoptimized -o $(spirv_out_filename_no_ext).spv",
    },
    MakeRecipe {
        source_line: 346,
        family: "spirv",
        target: "$(spirv_out_filename_no_ext).spv",
        command: "@rm $(spirv_out_filename_no_ext).spv.unoptimized",
    },
    MakeRecipe {
        source_line: 349,
        family: "spirv",
        target: "$(spirv_out_filename_no_ext).h",
        command: "@python3 spirv_binary_to_header.py $(spirv_out_filename_no_ext).spv $(spirv_out_filename_no_ext).h $(subst $(suffix $1),_$2,$(notdir $1))",
    },
    MakeRecipe {
        source_line: 419,
        family: "wgsl",
        target: "$(OUT)/wgsl/.",
        command: "@mkdir -p $@",
    },
    MakeRecipe {
        source_line: 452,
        family: "wgsl",
        target: "$(OUT)/wgsl/%.wgsl",
        command: "@echo \"wgsl/$*.wgsl\"",
    },
    MakeRecipe {
        source_line: 453,
        family: "wgsl",
        target: "$(OUT)/wgsl/%.wgsl",
        command: "@TERM=dumb bash -o pipefail -c 'naga --keep-coordinate-space \"$<\" \"$@\" 2>&1 | { grep -v \"Unknown decoration RelaxedPrecision\" || [ $$? = 1 ]; }'",
    },
    MakeRecipe {
        source_line: 463,
        family: "wgsl",
        target: "$(OUT)/wgsl/%.hpp",
        command: "@python3 wgsl_to_header.py $(WGSL_FLAGS) $< $@ $(subst .,_,$(basename $(notdir $<)))",
    },
    MakeRecipe {
        source_line: 471,
        family: "wgsl",
        target: "$(OUT)/d3d/.",
        command: "@mkdir -p $@",
    },
    MakeRecipe {
        source_line: 484,
        family: "d3d",
        target: "$(OUT)/d3d/%.vert.h",
        command: "@fxc /D VERTEX /I $(OUT) $(FXC_DEBUG_FLAG) /T vs_5_0 /Fh $@ $<",
    },
    MakeRecipe {
        source_line: 487,
        family: "d3d",
        target: "$(OUT)/d3d/%.frag.h",
        command: "@fxc /D FRAGMENT /I $(OUT) $(FXC_DEBUG_FLAG) /T ps_5_0 /Fh  $@ $<",
    },
    MakeRecipe {
        source_line: 490,
        family: "d3d",
        target: "$(OUT)/d3d/render_atlas_stroke.frag.h",
        command: "@fxc /D FRAGMENT /D ATLAS_FEATHERED_STROKE $(FXC_DEBUG_FLAG)  /I $(OUT) /T ps_5_0 /Fh  $@ $<",
    },
    MakeRecipe {
        source_line: 493,
        family: "d3d",
        target: "$(OUT)/d3d/render_atlas_fill.frag.h",
        command: "@fxc /D FRAGMENT /D ATLAS_FEATHERED_FILL $(FXC_DEBUG_FLAG)  /I $(OUT) /T ps_5_0 /Fh  $@ $<",
    },
    MakeRecipe {
        source_line: 496,
        family: "d3d",
        target: "$(OUT)/d3d/root.sig.h",
        command: "@fxc /I $(OUT) /T rootsig_1_1 /E ROOT_SIG /Fh   $@ $<",
    },
    MakeRecipe {
        source_line: 502,
        family: "d3d",
        target: "clean",
        command: "@rm -fr out",
    },
];

/// Wildcard calls preserve expansion kind and exact source order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MakeExpression {
    pub source_line: u16,
    pub family: &'static str,
    pub expression: &'static str,
}

pub const MAKE_WILDCARD_EXPRESSIONS: &[MakeExpression] = &[
    MakeExpression {
        source_line: 10,
        family: "minify",
        expression: "MINIFY_INPUTS := $(wildcard *.glsl) $(wildcard *.vert) $(wildcard *.frag)",
    },
    MakeExpression {
        source_line: 44,
        family: "metal",
        expression: "METAL_INPUTS := $(wildcard metal/*.metal)",
    },
    MakeExpression {
        source_line: 155,
        family: "spirv",
        expression: "SPIRV_STANDARD_INPUTS := $(wildcard spirv/*.main) \\",
    },
    MakeExpression {
        source_line: 156,
        family: "spirv",
        expression: "\t\t\t\t $(wildcard spirv/*.vert) \\",
    },
    MakeExpression {
        source_line: 157,
        family: "spirv",
        expression: "\t\t\t\t $(wildcard spirv/*.frag)",
    },
    MakeExpression {
        source_line: 478,
        family: "wgsl",
        expression:
            "\t $(addprefix $(OUT)/, $(patsubst %.hlsl, %.vert.h, $(wildcard d3d/*.hlsl))) \\",
    },
    MakeExpression {
        source_line: 479,
        family: "wgsl",
        expression:
            "\t $(addprefix $(OUT)/, $(patsubst %.hlsl, %.frag.h, $(wildcard d3d/*.hlsl))) \\",
    },
];

/// Make functions and conditional/selection expressions (if, findstring,
/// filter, foreach, patsubst, eval, call, and related transforms).
pub const MAKE_CONDITIONAL_EXPRESSIONS: &[MakeExpression] = &[
    MakeExpression {
        source_line: 11,
        family: "minify",
        expression: "MINIFY_EXPORT_OUTPUTS := $(addprefix $(OUT)/, $(addsuffix .exports.h, $(MINIFY_INPUTS)))",
    },
    MakeExpression {
        source_line: 12,
        family: "minify",
        expression: "MINIFY_GLSL_OUTPUTS := $(addprefix $(OUT)/,\\",
    },
    MakeExpression {
        source_line: 13,
        family: "minify",
        expression: "                         $(patsubst %.glsl, %.minified.glsl,\\",
    },
    MakeExpression {
        source_line: 14,
        family: "minify",
        expression: "                         $(patsubst %.vert, %.minified.vert,\\",
    },
    MakeExpression {
        source_line: 15,
        family: "minify",
        expression: "                         $(patsubst %.frag, %.minified.frag,\\",
    },
    MakeExpression {
        source_line: 17,
        family: "minify",
        expression: "MINIFY_HPP_OUTPUTS := $(addprefix $(OUT)/, $(addsuffix .hpp, $(MINIFY_INPUTS)))",
    },
    MakeExpression {
        source_line: 46,
        family: "metal",
        expression: "\t$(addprefix $(OUT)/, $(patsubst metal/%.metal, macosx/%.air, $(METAL_INPUTS)))",
    },
    MakeExpression {
        source_line: 47,
        family: "metal",
        expression: "METAL_IOS_AIR_OUTPUTS := $(addprefix $(OUT)/, $(patsubst metal/%.metal, ios/%.air, $(METAL_INPUTS)))",
    },
    MakeExpression {
        source_line: 63,
        family: "metal",
        expression: "\t$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeExpression {
        source_line: 68,
        family: "metal",
        expression: "\t\t-o $(patsubst metal/%.metal, $(OUT)/macosx/%.air, $(FILE));)",
    },
    MakeExpression {
        source_line: 77,
        family: "metal",
        expression: "\t$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeExpression {
        source_line: 82,
        family: "metal",
        expression: "\t\t-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)",
    },
    MakeExpression {
        source_line: 89,
        family: "metal",
        expression: "\t$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeExpression {
        source_line: 94,
        family: "metal",
        expression: "\t\t-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)",
    },
    MakeExpression {
        source_line: 101,
        family: "metal",
        expression: "\t$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeExpression {
        source_line: 106,
        family: "metal",
        expression: "\t\t-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)",
    },
    MakeExpression {
        source_line: 113,
        family: "metal",
        expression: "\t$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeExpression {
        source_line: 118,
        family: "metal",
        expression: "\t\t-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)",
    },
    MakeExpression {
        source_line: 125,
        family: "metal",
        expression: "\t$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeExpression {
        source_line: 130,
        family: "metal",
        expression: "\t\t-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)",
    },
    MakeExpression {
        source_line: 137,
        family: "metal",
        expression: "\t$(foreach FILE, $(METAL_INPUTS), \\",
    },
    MakeExpression {
        source_line: 142,
        family: "metal",
        expression: "\t\t-o $(patsubst metal/%.metal, $(OUT)/ios/%.air, $(FILE));)",
    },
    MakeExpression {
        source_line: 218,
        family: "spirv",
        expression: "spirv_typed_filename = $(basename $1).$2",
    },
    MakeExpression {
        source_line: 219,
        family: "spirv",
        expression: "spirv_out_filename_no_ext = $(OUT)/$(call spirv_typed_filename,$1,$2)",
    },
    MakeExpression {
        source_line: 220,
        family: "spirv",
        expression: "spirv_type = $(lastword $(subst _, ,$2))",
    },
    MakeExpression {
        source_line: 221,
        family: "spirv",
        expression: "spirv_is_vert = $(findstring vert,$(spirv_type))",
    },
    MakeExpression {
        source_line: 222,
        family: "spirv",
        expression: "spirv_is_webgpu = $(findstring webgpu,$2)",
    },
    MakeExpression {
        source_line: 223,
        family: "spirv",
        expression: "spirv_is_atomic_or_clockwise_atomic = $(findstring atomic,$1)",
    },
    MakeExpression {
        source_line: 224,
        family: "spirv",
        expression: "spirv_is_not_clockwise_atomic = $(if $(findstring clockwise_atomic,$1),,1)",
    },
    MakeExpression {
        source_line: 225,
        family: "spirv",
        expression: "spirv_is_clockwise = $(and $(findstring clockwise,$1), $(call spirv_is_not_clockwise_atomic,$1))",
    },
    MakeExpression {
        source_line: 256,
        family: "spirv",
        expression: "\t$(if $(spirv_is_webgpu),-O, \\",
    },
    MakeExpression {
        source_line: 257,
        family: "spirv",
        expression: "\t\t$(if $(spirv_is_atomic_or_clockwise_atomic), \\",
    },
    MakeExpression {
        source_line: 320,
        family: "spirv",
        expression: "spirv_frag_params = $(if $(spirv_is_webgpu), \\",
    },
    MakeExpression {
        source_line: 322,
        family: "spirv",
        expression: "\t\t\t\t\t$(if $(spirv_is_clockwise), \\",
    },
    MakeExpression {
        source_line: 330,
        family: "spirv",
        expression: "\t$(if $(spirv_is_vert), \\",
    },
    MakeExpression {
        source_line: 337,
        family: "spirv",
        expression: "## Usage: $(eval $(call spirv_list_rule, INPUT_FILENAME, OUTPUT_TYPE [, ADDITIONAL_COMPILE_OPTIONS]))",
    },
    MakeExpression {
        source_line: 342,
        family: "spirv",
        expression: "\t\t$(if $(spirv_is_vert), -DVERTEX, -DFRAGMENT $(spirv_frag_params)) \\",
    },
    MakeExpression {
        source_line: 349,
        family: "spirv",
        expression: "\t@python3 spirv_binary_to_header.py $(spirv_out_filename_no_ext).spv $(spirv_out_filename_no_ext).h $(subst $(suffix $1),_$2,$(notdir $1))",
    },
    MakeExpression {
        source_line: 356,
        family: "spirv",
        expression: "## Usage: $(eval $(call make_spirv_rules, LIST_OF_INPUT_FILES, LIST_OF_OUTPUT_TYPES [, ADDITIONAL_COMPILE_OPTIONS]))",
    },
    MakeExpression {
        source_line: 360,
        family: "spirv",
        expression: "    $(foreach type,$2,\\",
    },
    MakeExpression {
        source_line: 361,
        family: "spirv",
        expression: "        $(foreach file,$(filter-out %.$(if $(findstring vert, $(type)),frag,vert),$1),\\",
    },
    MakeExpression {
        source_line: 362,
        family: "spirv",
        expression: "            $(eval $(call spirv_list_rule,$(file),$(type),$3))\\",
    },
    MakeExpression {
        source_line: 368,
        family: "spirv",
        expression: "$(eval $(call make_spirv_rules, $(SPIRV_STANDARD_INPUTS), vert frag))",
    },
    MakeExpression {
        source_line: 373,
        family: "spirv",
        expression: "$(eval $(call make_spirv_rules, $(SPIRV_DRAW_MSAA_INPUTS), noclipdistance_vert, -DDISABLE_CLIP_DISTANCE_FOR_UBERSHADERS))",
    },
    MakeExpression {
        source_line: 374,
        family: "spirv",
        expression: "$(eval $(call make_spirv_rules, \\",
    },
    MakeExpression {
        source_line: 380,
        family: "spirv",
        expression: "$(eval $(call make_spirv_rules, \\",
    },
    MakeExpression {
        source_line: 386,
        family: "spirv",
        expression: "$(eval $(call make_spirv_rules, \\",
    },
    MakeExpression {
        source_line: 390,
        family: "spirv",
        expression: "$(eval $(call make_spirv_rules, \\",
    },
    MakeExpression {
        source_line: 394,
        family: "spirv",
        expression: "$(eval $(call make_spirv_rules, \\",
    },
    MakeExpression {
        source_line: 398,
        family: "spirv",
        expression: "$(eval $(call make_spirv_rules, \\",
    },
    MakeExpression {
        source_line: 404,
        family: "spirv",
        expression: "$(eval $(call make_spirv_rules, \\",
    },
    MakeExpression {
        source_line: 422,
        family: "wgsl",
        expression: "    $(patsubst $(OUT)/spirv/%.spv,$(OUT)/wgsl/%.wgsl, \\",
    },
    MakeExpression {
        source_line: 423,
        family: "wgsl",
        expression: "        $(filter %.webgpu_vert.spv \\",
    },
    MakeExpression {
        source_line: 460,
        family: "wgsl",
        expression: "WGSL_HEADER_OUTPUTS := $(patsubst %.wgsl,%.hpp,$(WGSL_OUTPUTS))",
    },
    MakeExpression {
        source_line: 463,
        family: "wgsl",
        expression: "\t@python3 wgsl_to_header.py $(WGSL_FLAGS) $< $@ $(subst .,_,$(basename $(notdir $<)))",
    },
    MakeExpression {
        source_line: 474,
        family: "wgsl",
        expression: "FXC_DEBUG_FLAG := $(if $(filter --human-readable,$(FLAGS)),/Zi,)",
    },
    MakeExpression {
        source_line: 478,
        family: "wgsl",
        expression: "\t $(addprefix $(OUT)/, $(patsubst %.hlsl, %.vert.h, $(wildcard d3d/*.hlsl))) \\",
    },
    MakeExpression {
        source_line: 479,
        family: "wgsl",
        expression: "\t $(addprefix $(OUT)/, $(patsubst %.hlsl, %.frag.h, $(wildcard d3d/*.hlsl))) \\",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MakeFunction {
    pub source_line: u16,
    pub name: &'static str,
    pub source: &'static str,
}

pub const MAKE_FUNCTIONS: &[MakeFunction] = &[
    MakeFunction {
        source_line: 339,
        name: "spirv_list_rule",
        source: "define spirv_list_rule\n  $(spirv_out_filename_no_ext).spv: $1 $(MINIFY_STAMP) | $(OUT)/spirv/.\n\t@glslangValidator -S $(spirv_type) -DTARGET_SPIRV \\\n\t\t$(if $(spirv_is_vert), -DVERTEX, -DFRAGMENT $(spirv_frag_params)) \\\n\t\t-I$(OUT)  -V $3 -o $(spirv_out_filename_no_ext).spv.unoptimized $1\n\t@spirv-opt --preserve-bindings --preserve-interface $(spirv_opt_params) \\\n\t\t$(spirv_out_filename_no_ext).spv.unoptimized -o $(spirv_out_filename_no_ext).spv\n\t@rm $(spirv_out_filename_no_ext).spv.unoptimized\n\n  $(spirv_out_filename_no_ext).h: $(spirv_out_filename_no_ext).spv\n\t@python3 spirv_binary_to_header.py $(spirv_out_filename_no_ext).spv $(spirv_out_filename_no_ext).h $(subst $(suffix $1),_$2,$(notdir $1))\n\n  SPIRV_OUTPUTS_BINARY += $(spirv_out_filename_no_ext).spv\n  SPIRV_OUTPUTS_HEADERS += $(spirv_out_filename_no_ext).h\nendef",
    },
    MakeFunction {
        source_line: 358,
        name: "make_spirv_rules",
        source: "define make_spirv_rules\n    ## Note that the inner foreach will filter out \".frag\" files from the list for any vert targets, and vice versa.\n    $(foreach type,$2,\\\n        $(foreach file,$(filter-out %.$(if $(findstring vert, $(type)),frag,vert),$1),\\\n            $(eval $(call spirv_list_rule,$(file),$(type),$3))\\\n        )\\\n    )\nendef",
    },
];

/// Coarse target-family boundaries are source organization, not feature
/// selection. Every family remains required by the pinned campaign.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetFamily {
    pub name: &'static str,
    pub source_start: u16,
    pub source_end: u16,
    pub description: &'static str,
}

pub const TARGET_FAMILIES: &[TargetFamily] = &[
    TargetFamily {
        name: "minify",
        source_start: 1,
        source_end: 43,
        description: "Batch-expand *.glsl, *.vert, and *.frag inputs; minify once with a stamp and emit exports, minified sources, and headers.",
    },
    TargetFamily {
        name: "metal",
        source_start: 44,
        source_end: 148,
        description: "Generate draw combinations and compile the seven Apple Metal artifact families with platform SDK, language standard, and minimum-version choices.",
    },
    TargetFamily {
        name: "spirv",
        source_start: 149,
        source_end: 410,
        description: "Preserve all Vulkan/SPIR-V source lists, specialization variants, glslangValidator and spirv-opt recipes, and binary/header targets.",
    },
    TargetFamily {
        name: "wgsl",
        source_start: 411,
        source_end: 482,
        description: "Translate selected SPIR-V outputs with naga, retaining coordinate-space, TERM, warning-filter, and WGSL-header behavior.",
    },
    TargetFamily {
        name: "d3d",
        source_start: 483,
        source_end: 502,
        description: "Preserve FXC vertex, fragment, atlas specialization, root-signature, and clean targets.",
    },
];

/// The source's wildcard declarations and their expansion order.
pub const WILDCARD_SOURCE_ORDER: &[(&str, &[&str])] = &[
    ("MINIFY_INPUTS", &["*.glsl", "*.vert", "*.frag"]),
    ("METAL_INPUTS", &["metal/*.metal"]),
    (
        "SPIRV_STANDARD_INPUTS",
        &["spirv/*.main", "spirv/*.vert", "spirv/*.frag"],
    ),
    ("D3D_OUTPUTS", &["d3d/*.hlsl"]),
];

/// The explicitly enumerated source-order lists used by the SPIR-V branch.
pub const SPIRV_SOURCE_ORDER: &[(&str, &[&str])] = &[
    (
        "SPIRV_WEBGPU_INPUTS",
        &[
            "spirv/tessellate.main",
            "spirv/render_atlas.vert",
            "spirv/render_atlas_fill.frag",
            "spirv/render_atlas_stroke.frag",
            "spirv/blit_texture_as_draw_filtered.main",
            "spirv/atomic_resolve_coalesced.main",
        ],
    ),
    (
        "SPIRV_CLOCKWISE_INPUTS",
        &[
            "spirv/draw_clockwise_path.main",
            "spirv/draw_clockwise_clip.main",
            "spirv/draw_clockwise_interior_triangles.main",
            "spirv/draw_clockwise_clip_interior_triangles.main",
            "spirv/draw_clockwise_atlas_blit.main",
            "spirv/draw_clockwise_image_mesh.main",
            "spirv/draw_clockwise_atomic_path.main",
            "spirv/draw_clockwise_atomic_interior_triangles.main",
            "spirv/draw_clockwise_atomic_atlas_blit.main",
            "spirv/draw_clockwise_atomic_image_mesh.main",
            "spirv/init_clockwise_atomic_workaround.frag",
            "spirv/draw_clockwise_atomic_clip.frag",
            "spirv/draw_clockwise_atomic_clip_interior_triangles.frag",
            "spirv/clear_clockwise_atomic_clip.main",
        ],
    ),
    (
        "SPIRV_DRAW_ATOMIC_INPUTS",
        &[
            "spirv/atomic_draw_image_mesh.main",
            "spirv/atomic_draw_image_rect.main",
            "spirv/atomic_draw_interior_triangles.main",
            "spirv/atomic_draw_atlas_blit.main",
            "spirv/atomic_draw_path.main",
            "spirv/atomic_resolve.main",
            "spirv/atomic_init.main",
        ],
    ),
    (
        "SPIRV_DRAW_MSAA_INPUTS",
        &[
            "spirv/draw_msaa_atlas_blit.main",
            "spirv/draw_msaa_image_mesh.main",
            "spirv/draw_msaa_path.main",
            "spirv/draw_msaa_stencil.main",
        ],
    ),
    (
        "WEBGPU_NOSSBO_NOCLIPDISTANCE_INPUTS",
        &[
            "spirv/draw_msaa_path.main",
            "spirv/draw_msaa_atlas_blit.main",
        ],
    ),
    (
        "WEBGPU_NOSSBO_INPUTS",
        &[
            "$(WEBGPU_NOSSBO_NOCLIPDISTANCE_INPUTS)",
            "spirv/tessellate.main",
            "spirv/render_atlas.vert",
        ],
    ),
];

/// Seven-way Apple correspondence, preserving the pinned SDK, language
/// standard, target, and minimum-version distinctions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AppleArtifact {
    pub source_alias: &'static str,
    pub metallib_target: &'static str,
    pub embedded_c_target: &'static str,
    pub sdk: &'static str,
    pub language_standard: &'static str,
    pub target_or_min_version: &'static str,
    pub air_output_dir: &'static str,
}

pub const APPLE_ARTIFACTS: &[AppleArtifact] = &[
    AppleArtifact {
        source_alias: "rive_pls_macosx_metallib",
        metallib_target: "$(OUT)/macosx/rive_pls_macosx.metallib",
        embedded_c_target: "$(OUT)/rive_pls_macosx.metallib.c",
        sdk: "macosx",
        language_standard: "macos-metal2.3",
        target_or_min_version: "-mmacosx-version-min=11.0",
        air_output_dir: "$(OUT)/macosx",
    },
    AppleArtifact {
        source_alias: "rive_pls_ios_metallib",
        metallib_target: "$(OUT)/ios/rive_pls_ios.metallib",
        embedded_c_target: "$(OUT)/rive_pls_ios.metallib.c",
        sdk: "iphoneos",
        language_standard: "ios-metal2.2",
        target_or_min_version: "-mios-version-min=13",
        air_output_dir: "$(OUT)/ios",
    },
    AppleArtifact {
        source_alias: "rive_pls_ios_simulator_metallib",
        metallib_target: "$(OUT)/ios/rive_pls_ios_simulator.metallib",
        embedded_c_target: "$(OUT)/rive_pls_ios_simulator.metallib.c",
        sdk: "iphonesimulator",
        language_standard: "ios-metal2.2",
        target_or_min_version: "-miphonesimulator-version-min=13",
        air_output_dir: "$(OUT)/ios",
    },
    AppleArtifact {
        source_alias: "rive_renderer_xros_metallib",
        metallib_target: "$(OUT)/ios/rive_renderer_xros.metallib",
        embedded_c_target: "$(OUT)/rive_renderer_xros.metallib.c",
        sdk: "xros",
        language_standard: "metal3.1",
        target_or_min_version: "--target=air64-apple-xros1.0",
        air_output_dir: "$(OUT)/ios",
    },
    AppleArtifact {
        source_alias: "rive_renderer_xros_simulator_metallib",
        metallib_target: "$(OUT)/ios/rive_renderer_xros_simulator.metallib",
        embedded_c_target: "$(OUT)/rive_renderer_xros_simulator.metallib.c",
        sdk: "xrsimulator",
        language_standard: "metal3.1",
        target_or_min_version: "--target=air64-apple-xros1.0-simulator",
        air_output_dir: "$(OUT)/ios",
    },
    AppleArtifact {
        source_alias: "rive_renderer_appletvos_metallib",
        metallib_target: "$(OUT)/ios/rive_renderer_appletvos.metallib",
        embedded_c_target: "$(OUT)/rive_renderer_appletvos.metallib.c",
        sdk: "appletvos",
        language_standard: "metal3.0",
        target_or_min_version: "-mappletvos-version-min=16.0",
        air_output_dir: "$(OUT)/ios",
    },
    AppleArtifact {
        source_alias: "rive_renderer_appletvsimulator_metallib",
        metallib_target: "$(OUT)/ios/rive_renderer_appletvsimulator.metallib",
        embedded_c_target: "$(OUT)/rive_renderer_appletvsimulator.metallib.c",
        sdk: "appletvsimulator",
        language_standard: "metal3.0",
        target_or_min_version: "-mappletvsimulator-version-min=16.0",
        air_output_dir: "$(OUT)/ios",
    },
];

/// Non-Metal targets are intentionally retained because the pinned source
/// contains them and the authority requires full-source translation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NonMetalFamily {
    pub name: &'static str,
    pub target: &'static str,
    pub toolchain: &'static str,
    pub source_lines: &'static str,
}

pub const NON_METAL_FAMILIES: &[NonMetalFamily] = &[
    NonMetalFamily {
        name: "spirv",
        target: "spirv / spirv-binary",
        toolchain: "glslangValidator + spirv-opt + spirv_binary_to_header.py",
        source_lines: "149-410",
    },
    NonMetalFamily {
        name: "wgsl",
        target: "wgsl",
        toolchain: "naga --keep-coordinate-space + wgsl_to_header.py",
        source_lines: "411-482",
    },
    NonMetalFamily {
        name: "d3d",
        target: "d3d",
        toolchain: "fxc vs_5_0 + ps_5_0 + rootsig_1_1",
        source_lines: "483-502",
    },
];

/// A narrow source-shaped selector lets later queues inspect target families
/// without executing the Makefile.
pub fn target_family(source_line: u16) -> &'static str {
    match source_line {
        1..=43 => "minify",
        44..=148 => "metal",
        149..=410 => "spirv",
        411..=482 => "wgsl",
        483..=502 => "d3d",
        _ => "outside-pinned-makefile",
    }
}

// The complete source is intentionally the final payload; no generated
// The Apple artifacts and build integration are performed by the executable
// rules at the start of this owner; the records below retain the full Makefile.
