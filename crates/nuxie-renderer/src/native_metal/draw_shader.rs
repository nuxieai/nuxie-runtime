//! Typed inventory and retained library loader for the pinned native Metal draw shader.
//!
//! The parent `native_metal` module wires this library into the later pipeline
//! slice. Keeping the source inventory beside its device-specific library owner
//! makes the source, hash, and compiled-function oracles independently testable.
//!
//! Upstream: `rive-app/rive-runtime` at
//! Current minified sources: `7aa93402a27c800db8a36acc8672612c100ea9b1`.
//! Primary source: `renderer/src/shaders/metal/draw.metal:1-42`.

use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{msg_send, rc::Retained};
use objc2_foundation::{NSError, NSString};
use objc2_metal::{MTLDevice, MTLFunction, MTLLibrary};
use std::ffi::c_void;
use std::fmt;
use std::ptr::NonNull;

/// Revision used to regenerate the current minified GLSL batch.
/// The unchanged draw-combination capture retains its separately recorded provenance.
pub const UPSTREAM_SHA: &str = "7aa93402a27c800db8a36acc8672612c100ea9b1";

/// The Cargo build-script output consumed by [`DrawShaderLibrary`]. This name
/// intentionally differs from the existing tracer artifact.
pub const DRAW_METALLIB_FILE_NAME: &str = "native_metal_draw.metallib";

const DRAW_METALLIB: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/native_metal_draw.metallib"));

#[link(name = "System")]
extern "C" {
    #[link_name = "dispatch_data_create"]
    fn create_dispatch_data(
        buffer: NonNull<c_void>,
        size: usize,
        queue: Option<NonNull<c_void>>,
        destructor: *mut c_void,
    ) -> *mut AnyObject;

    #[link_name = "dispatch_release"]
    fn release_dispatch_object(object: *mut AnyObject);
}

/// Role of an input in the offline draw-shader include graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArtifactKind {
    EntryPoint,
    DirectInclude,
    GeneratedCombination,
    CombinationInclude,
}

/// One immutable source artifact and its pinned provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShaderArtifact {
    pub name: &'static str,
    pub kind: ArtifactKind,
    pub upstream_path: &'static str,
    pub byte_len: usize,
    pub sha256: &'static str,
    pub bytes: &'static [u8],
}

/// The complete source closure required by `draw.metal` and its generated
/// combination source. The four `CombinationInclude` entries are required by
/// namespaces emitted in `draw_combinations.metal` even though they are not
/// named by the first 42 lines of `draw.metal`.
pub static DRAW_SHADER_ARTIFACTS: &[ShaderArtifact] = &[
    ShaderArtifact {
        name: "draw.metal",
        kind: ArtifactKind::EntryPoint,
        upstream_path: "renderer/src/shaders/metal/draw.metal",
        byte_len: 947,
        sha256: "1111713584059e5d2b6469d45200b5c11949de17d7dcb7ffe62529c96c6269bd",
        bytes: include_bytes!("shaders/draw.metal"),
    },
    ShaderArtifact {
        name: "metal.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/metal.minified.glsl",
        byte_len: 7327,
        sha256: "932b94a01b7040f1a84bc958dc46d513bb3ba175151b8fdf9d7a6c601e4064f7",
        bytes: include_bytes!("shaders/metal.minified.glsl"),
    },
    ShaderArtifact {
        name: "constants.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/constants.minified.glsl",
        byte_len: 2525,
        sha256: "53f853c9b547b1ba0b8f94f0e2ec2c708ee5135e85ef0559e162ab21502b92c1",
        bytes: include_bytes!("shaders/constants.minified.glsl"),
    },
    ShaderArtifact {
        name: "flush_uniforms.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/flush_uniforms.minified.glsl",
        byte_len: 418,
        sha256: "4226f7cee7c3fd94843dbfd1de2e293a883a98b379ee624836971fd4f05d410e",
        bytes: include_bytes!("shaders/flush_uniforms.minified.glsl"),
    },
    ShaderArtifact {
        name: "common.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/common.minified.glsl",
        byte_len: 4525,
        sha256: "328a364d4f79657f8a8adf792e7d4c8caa962813427d0ecd8d7552b156cc1351",
        bytes: include_bytes!("shaders/common.minified.glsl"),
    },
    ShaderArtifact {
        name: "draw_path_common.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/draw_path_common.minified.glsl",
        byte_len: 6676,
        sha256: "3c681684ec68a2e3b3ad6bb99969230fc228b5a911def25bcfffd727f3ee97ff",
        bytes: include_bytes!("shaders/draw_path_common.minified.glsl"),
    },
    ShaderArtifact {
        name: "render_atlas.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/render_atlas.minified.glsl",
        byte_len: 2366,
        sha256: "f84bae6dc804c6f3fa782950b042de95aa62ca6738ce6cef564cbe4d6ac859f1",
        bytes: include_bytes!("shaders/render_atlas.minified.glsl"),
    },
    ShaderArtifact {
        name: "advanced_blend.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/advanced_blend.minified.glsl",
        byte_len: 2263,
        sha256: "d7970cbdb75191e7d9888eb302e712c81bf06a43fce0ec4ee9db4d7ea2341ac4",
        bytes: include_bytes!("shaders/advanced_blend.minified.glsl"),
    },
    ShaderArtifact {
        name: "draw_combinations.metal",
        kind: ArtifactKind::GeneratedCombination,
        upstream_path: "renderer/src/shaders/out/generated/draw_combinations.metal",
        byte_len: 5877,
        sha256: "d3ec0fcb16802b4081d04356e03560d244afb4799c408b0a7b633e591679e06d",
        bytes: include_bytes!("shaders/draw_combinations.metal"),
    },
    ShaderArtifact {
        name: "draw_path.minified.vert",
        kind: ArtifactKind::CombinationInclude,
        upstream_path: "renderer/src/shaders/out/generated/draw_path.minified.vert",
        byte_len: 5192,
        sha256: "0dfb3c1db0e9432fe49e50095dd8f54fa9a65ba4be5ed5ec2cb90224e36c4dbf",
        bytes: include_bytes!("shaders/draw_path.minified.vert"),
    },
    ShaderArtifact {
        name: "draw_raster_order_path.minified.frag",
        kind: ArtifactKind::CombinationInclude,
        upstream_path: "renderer/src/shaders/out/generated/draw_raster_order_path.minified.frag",
        byte_len: 2098,
        sha256: "7f8b973edef743421340082bd07c646e8d7e8d4aa99f5810308afac3cb47224d",
        bytes: include_bytes!("shaders/draw_raster_order_path.minified.frag"),
    },
    ShaderArtifact {
        name: "draw_mesh.minified.frag",
        kind: ArtifactKind::CombinationInclude,
        upstream_path: "renderer/src/shaders/out/generated/draw_mesh.minified.frag",
        byte_len: 2169,
        sha256: "5108f104eecd34dabb311c83589fd2e63d39137950ed901e2c9cf7d7a2571dcd",
        bytes: include_bytes!("shaders/draw_mesh.minified.frag"),
    },
    ShaderArtifact {
        name: "draw_image_mesh.minified.vert",
        kind: ArtifactKind::CombinationInclude,
        upstream_path: "renderer/src/shaders/out/generated/draw_image_mesh.minified.vert",
        byte_len: 1419,
        sha256: "bbd72ebb15855438a94d7ffc2bd686ba076b0c72c1be2772159616ef7bb37657",
        bytes: include_bytes!("shaders/draw_image_mesh.minified.vert"),
    },
];

/// A typed view of the offline source closure.
#[derive(Clone, Copy, Debug)]
pub struct DrawShaderSource {
    pub entry_point: &'static ShaderArtifact,
    pub artifacts: &'static [ShaderArtifact],
}

impl DrawShaderSource {
    pub const fn load() -> Self {
        Self {
            entry_point: &DRAW_SHADER_ARTIFACTS[0],
            artifacts: DRAW_SHADER_ARTIFACTS,
        }
    }

    pub fn artifact(self, name: &str) -> Option<&'static ShaderArtifact> {
        self.artifacts.iter().find(|artifact| artifact.name == name)
    }

    pub fn namespace_inventory(self) -> impl Iterator<Item = &'static str> {
        self.artifact("draw_combinations.metal")
            .into_iter()
            .flat_map(|artifact| {
                std::str::from_utf8(artifact.bytes)
                    .unwrap_or_default()
                    .lines()
                    .filter_map(|line| line.strip_prefix("namespace "))
            })
    }
}

/// Retained device-specific Metal library loaded from the build-script output.
#[derive(Clone)]
pub struct DrawShaderLibrary {
    library: Retained<ProtocolObject<dyn MTLLibrary>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawShaderLibraryError {
    EmptyEmbeddedMetallib,
    DispatchDataCreation,
    MetalLibraryLoad {
        domain: String,
        code: isize,
        description: String,
    },
    MissingFunction(String),
}

impl fmt::Display for DrawShaderLibraryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyEmbeddedMetallib => formatter.write_str("embedded draw metallib is empty"),
            Self::DispatchDataCreation => {
                formatter.write_str("failed to create dispatch data for embedded draw metallib")
            }
            Self::MetalLibraryLoad {
                domain,
                code,
                description,
            } => write!(
                formatter,
                "load embedded draw metallib: {domain} error {code}: {description}"
            ),
            Self::MissingFunction(name) => {
                write!(formatter, "draw metallib function is absent: {name}")
            }
        }
    }
}

impl std::error::Error for DrawShaderLibraryError {}

impl DrawShaderLibrary {
    pub fn load(device: &ProtocolObject<dyn MTLDevice>) -> Result<Self, DrawShaderLibraryError> {
        if DRAW_METALLIB.is_empty() {
            return Err(DrawShaderLibraryError::EmptyEmbeddedMetallib);
        }
        let buffer = NonNull::new(DRAW_METALLIB.as_ptr().cast_mut().cast::<c_void>())
            .expect("a non-empty embedded metallib has a non-null address");
        // SAFETY: `buffer` addresses immutable embedded bytes with static
        // lifetime. A null destructor selects dispatch's default handling and
        // does not transfer ownership of that static storage, which remains
        // valid through the synchronous Metal library load below.
        let data = unsafe {
            create_dispatch_data(buffer, DRAW_METALLIB.len(), None, std::ptr::null_mut())
        };
        if data.is_null() {
            return Err(DrawShaderLibraryError::DispatchDataCreation);
        }

        let mut error: Option<Retained<NSError>> = None;
        // Keep the +1 library and retained NSError writeback independent until
        // after the pinned `err != nil || library == nil` condition.
        let library: Option<Retained<ProtocolObject<dyn MTLLibrary>>> =
            // SAFETY: `device` is a live MTLDevice protocol object and `data`
            // is a valid +1 dispatch-data object containing the complete
            // metallib. The selector consumes it synchronously and publishes
            // independently retained library and error owners.
            unsafe { msg_send![device, newLibraryWithData: data, error: &mut error] };
        // SAFETY: This balances the +1 create_dispatch_data result after the
        // synchronous Metal call; the embedded bytes remain static and a
        // successful MTLLibrary is independently retained in `library`.
        unsafe { release_dispatch_object(data) };
        if error.is_some() || library.is_none() {
            return Err(match error {
                Some(error) => DrawShaderLibraryError::MetalLibraryLoad {
                    domain: error.domain().to_string(),
                    code: error.code(),
                    description: error.localizedDescription().to_string(),
                },
                None => DrawShaderLibraryError::MetalLibraryLoad {
                    domain: "<nil>".to_owned(),
                    code: 0,
                    description: "<nil>".to_owned(),
                },
            });
        }
        let library = library.expect("library checked nonnil");
        Ok(Self { library })
    }

    pub fn library(&self) -> &ProtocolObject<dyn MTLLibrary> {
        &self.library
    }

    pub fn function_names(&self) -> Vec<String> {
        let names = self.library.functionNames();
        (0..names.count())
            .map(|index| names.objectAtIndex(index).to_string())
            .collect()
    }

    pub fn function(
        &self,
        name: &str,
    ) -> Result<Retained<ProtocolObject<dyn MTLFunction>>, DrawShaderLibraryError> {
        self.library
            .newFunctionWithName(&NSString::from_str(name))
            .ok_or_else(|| DrawShaderLibraryError::MissingFunction(name.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2_metal::MTLCreateSystemDefaultDevice;
    use sha2::{Digest, Sha256};

    fn sha256_hex(bytes: &[u8]) -> String {
        let digest = Sha256::digest(bytes);
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[test]
    fn source_hash_inventory_matches_checked_in_bytes() {
        for artifact in DRAW_SHADER_ARTIFACTS {
            assert_eq!(artifact.bytes.len(), artifact.byte_len, "{}", artifact.name);
            assert_eq!(
                sha256_hex(artifact.bytes),
                artifact.sha256,
                "{}",
                artifact.name
            );
        }
    }

    #[test]
    fn current_source_inventory_matches_regenerated_batch() {
        assert_eq!(UPSTREAM_SHA, "7aa93402a27c800db8a36acc8672612c100ea9b1");
        let inventory = include_str!(
            "../../tests/fixtures/native_metal/offline_draw_shader/source_inventory.txt"
        );
        let rows = inventory.lines()
            .filter_map(|line| line.strip_prefix("# current_source|"))
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), DRAW_SHADER_ARTIFACTS.len());
        for row in rows {
            let fields = row.split('|').collect::<Vec<_>>();
            assert_eq!(fields.len(), 3);
            let artifact = DRAW_SHADER_ARTIFACTS.iter()
                .find(|artifact| artifact.name == fields[0]).unwrap();
            assert_eq!(artifact.bytes.len(), fields[1].parse::<usize>().unwrap());
            assert_eq!(sha256_hex(artifact.bytes), fields[2], "{}", artifact.name);
        }
    }

    #[test]
    fn draw_source_preserves_upstream_include_order() {
        let source = std::str::from_utf8(DrawShaderSource::load().entry_point.bytes).unwrap();
        let includes: Vec<_> = source
            .lines()
            .filter(|line| line.starts_with("#include"))
            .collect();
        assert_eq!(
            includes,
            [
                "#include <metal_stdlib>",
                "#include \"metal.minified.glsl\"",
                "#include \"constants.minified.glsl\"",
                "#include \"flush_uniforms.minified.glsl\"",
                "#include \"common.minified.glsl\"",
                "#include \"draw_path_common.minified.glsl\"",
                "#include \"render_atlas.minified.glsl\"",
                "#include \"advanced_blend.minified.glsl\"",
                "#include \"draw_combinations.metal\"",
            ]
        );
    }

    #[test]
    fn generated_namespace_inventory_has_the_pinned_namespaces() {
        let names: Vec<_> = DrawShaderSource::load().namespace_inventory().collect();
        assert_eq!(names.len(), 10);
        assert_eq!(
            names,
            [
                "p11110000100",
                "p11111111100",
                "c11111111100",
                "p11110000110",
                "p11111111110",
                "c11111111110",
                "p11100000111",
                "p11100011111",
                "m11100000000",
                "m11100011000",
            ]
        );
    }

    #[test]
    fn compiled_library_has_the_exact_function_inventory() {
        use crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::runtime_generated_shader_exports as exports;
        let device = MTLCreateSystemDefaultDevice().expect("create system Metal device");
        let library = DrawShaderLibrary::load(&device).expect("load embedded draw metallib");
        let mut names = library.function_names();
        names.sort();
        let mut expected: Vec<String> = [
            exports::GLSL_atlasVertexMain,
            exports::GLSL_atlasFillFragmentMain,
            exports::GLSL_atlasStrokeFragmentMain,
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        for namespace in [
            "m11100000000",
            "p11100000111",
            "p11110000100",
            "p11110000110",
        ] {
            expected.push(format!("{namespace}::{}", exports::GLSL_drawVertexMain));
        }
        for namespace in [
            "c11111111100",
            "c11111111110",
            "m11100011000",
            "p11100011111",
            "p11111111100",
            "p11111111110",
        ] {
            expected.push(format!("{namespace}::{}", exports::GLSL_drawFragmentMain));
        }
        expected.sort();
        assert_eq!(names, expected);
    }

    #[test]
    fn compiled_library_resolves_representative_functions() {
        use crate::mechanical_port::source::renderer::src::metal::background_shader_compiler_mm::runtime_generated_shader_exports as exports;
        let device = MTLCreateSystemDefaultDevice().expect("create system Metal device");
        let library = DrawShaderLibrary::load(&device).expect("load embedded draw metallib");
        for name in [
            exports::GLSL_atlasVertexMain.to_owned(),
            format!("p11110000100::{}", exports::GLSL_drawVertexMain),
            format!("p11111111100::{}", exports::GLSL_drawFragmentMain),
        ] {
            library
                .function(&name)
                .unwrap_or_else(|error| panic!("{error}"));
        }
        assert_eq!(
            library.function("not-a-draw-function").unwrap_err(),
            DrawShaderLibraryError::MissingFunction("not-a-draw-function".to_owned())
        );
    }
}
