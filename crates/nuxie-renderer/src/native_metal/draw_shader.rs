//! Typed inventory and retained library loader for the pinned native Metal draw shader.
//!
//! The parent `native_metal` module wires this library into the later pipeline
//! slice. Keeping the source inventory beside its device-specific library owner
//! makes the source, hash, and compiled-function oracles independently testable.
//!
//! Upstream: `rive-app/rive-runtime` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
//! Primary source: `renderer/src/shaders/metal/draw.metal:1-42`.

use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{msg_send, rc::Retained};
use objc2_foundation::{NSError, NSString};
use objc2_metal::{MTLDevice, MTLFunction, MTLLibrary};
use std::ffi::c_void;
use std::fmt;
use std::ptr::NonNull;

/// The exact upstream revision from which every checked-in source byte was
/// captured.
pub const UPSTREAM_SHA: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";

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
        byte_len: 7194,
        sha256: "e4845f27a7f9f293c139eebc9babdb03f30a3eb9c3fd5699b2a97abcd517c0a8",
        bytes: include_bytes!("shaders/metal.minified.glsl"),
    },
    ShaderArtifact {
        name: "constants.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/constants.minified.glsl",
        byte_len: 2120,
        sha256: "0644011079b560f56ba301fb9e8163212ddc910cf9d592105a2c75eef839c597",
        bytes: include_bytes!("shaders/constants.minified.glsl"),
    },
    ShaderArtifact {
        name: "flush_uniforms.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/flush_uniforms.minified.glsl",
        byte_len: 375,
        sha256: "644d2092ddf84a3cb929dfab1b75bdd6d1cd0ec2675b124fea68038c4ad855c0",
        bytes: include_bytes!("shaders/flush_uniforms.minified.glsl"),
    },
    ShaderArtifact {
        name: "common.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/common.minified.glsl",
        byte_len: 4826,
        sha256: "536c2f979d42703bf8c14d2f5f310344c418853b9b23e955621fb1b39d048134",
        bytes: include_bytes!("shaders/common.minified.glsl"),
    },
    ShaderArtifact {
        name: "draw_path_common.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/draw_path_common.minified.glsl",
        byte_len: 6467,
        sha256: "c044a173c950871b487f8d121ad82154236bda2ab0d4af0c8f35369328f18616",
        bytes: include_bytes!("shaders/draw_path_common.minified.glsl"),
    },
    ShaderArtifact {
        name: "render_atlas.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/render_atlas.minified.glsl",
        byte_len: 2361,
        sha256: "1c880793e4e30b79b447b8bedb88c589304ba264ef5cfd2ef1f35d15f94623e2",
        bytes: include_bytes!("shaders/render_atlas.minified.glsl"),
    },
    ShaderArtifact {
        name: "advanced_blend.minified.glsl",
        kind: ArtifactKind::DirectInclude,
        upstream_path: "renderer/src/shaders/out/generated/advanced_blend.minified.glsl",
        byte_len: 2259,
        sha256: "5a3c63e1a6af349758dbc75cd9fff42b282000463b40fc1e8fdaad22e17178c9",
        bytes: include_bytes!("shaders/advanced_blend.minified.glsl"),
    },
    ShaderArtifact {
        name: "draw_combinations.metal",
        kind: ArtifactKind::GeneratedCombination,
        upstream_path: "renderer/src/shaders/out/generated/draw_combinations.metal",
        byte_len: 5363,
        sha256: "9f33bdcd7b8831c0654848677d5270698069f98ceebf698c28790d4b32ffed7c",
        bytes: include_bytes!("shaders/draw_combinations.metal"),
    },
    ShaderArtifact {
        name: "draw_path.minified.vert",
        kind: ArtifactKind::CombinationInclude,
        upstream_path: "renderer/src/shaders/out/generated/draw_path.minified.vert",
        byte_len: 4618,
        sha256: "0a478c0880dc2de6a51799e8b4a725e42f022f7313d91bfdfa192bee98bc6531",
        bytes: include_bytes!("shaders/draw_path.minified.vert"),
    },
    ShaderArtifact {
        name: "draw_raster_order_path.minified.frag",
        kind: ArtifactKind::CombinationInclude,
        upstream_path: "renderer/src/shaders/out/generated/draw_raster_order_path.minified.frag",
        byte_len: 1831,
        sha256: "90100677be8092452ae382ca9ca3745830bcc8135d54a451375176d91c5730d0",
        bytes: include_bytes!("shaders/draw_raster_order_path.minified.frag"),
    },
    ShaderArtifact {
        name: "draw_mesh.minified.frag",
        kind: ArtifactKind::CombinationInclude,
        upstream_path: "renderer/src/shaders/out/generated/draw_mesh.minified.frag",
        byte_len: 2098,
        sha256: "290b59763daa435a8f0d9fdfb9716258d2c0c18cfbf145b8fd8bc19bc105e1a0",
        bytes: include_bytes!("shaders/draw_mesh.minified.frag"),
    },
    ShaderArtifact {
        name: "draw_image_mesh.minified.vert",
        kind: ArtifactKind::CombinationInclude,
        upstream_path: "renderer/src/shaders/out/generated/draw_image_mesh.minified.vert",
        byte_len: 1439,
        sha256: "cef8caabf852ac991b5f8fe3a3f38994e603f670df0f893aa537e3c9e30e4bcb",
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

        let result: Result<Retained<ProtocolObject<dyn MTLLibrary>>, Retained<NSError>> =
            // SAFETY: `device` is a live MTLDevice protocol object and `data`
            // is a valid +1 dispatch-data object containing the complete
            // metallib. The selector consumes it synchronously and returns
            // retained library/error ownership through objc2.
            unsafe { msg_send![device, newLibraryWithData: data, error: _] };
        // SAFETY: This balances the +1 create_dispatch_data result after the
        // synchronous Metal call; the embedded bytes remain static and a
        // successful MTLLibrary is independently retained in `result`.
        unsafe { release_dispatch_object(data) };
        let library = result.map_err(|error| DrawShaderLibraryError::MetalLibraryLoad {
            domain: error.domain().to_string(),
            code: error.code(),
            description: error.localizedDescription().to_string(),
        })?;
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
                "p1111000000",
                "p1111111100",
                "c1111111100",
                "p1111000010",
                "p1111111110",
                "c1111111110",
                "p1110000011",
                "p1110001111",
                "m1110000000",
                "m1110001100",
            ]
        );
    }

    #[test]
    fn compiled_library_has_the_exact_function_inventory() {
        let device = MTLCreateSystemDefaultDevice().expect("create system Metal device");
        let library = DrawShaderLibrary::load(&device).expect("load embedded draw metallib");
        let mut names = library.function_names();
        names.sort();
        assert_eq!(
            names,
            [
                "RF",
                "UE",
                "VE",
                "c1111111100::JB",
                "c1111111110::JB",
                "m1110000000::GC",
                "m1110001100::JB",
                "p1110000011::GC",
                "p1110001111::JB",
                "p1111000000::GC",
                "p1111000010::GC",
                "p1111111100::JB",
                "p1111111110::JB",
            ]
        );
    }

    #[test]
    fn compiled_library_resolves_representative_functions() {
        let device = MTLCreateSystemDefaultDevice().expect("create system Metal device");
        let library = DrawShaderLibrary::load(&device).expect("load embedded draw metallib");
        for name in ["RF", "p1111000000::GC", "p1111111100::JB"] {
            library
                .function(name)
                .unwrap_or_else(|error| panic!("{error}"));
        }
        assert_eq!(
            library.function("not-a-draw-function").unwrap_err(),
            DrawShaderLibraryError::MissingFunction("not-a-draw-function".to_owned())
        );
    }
}
