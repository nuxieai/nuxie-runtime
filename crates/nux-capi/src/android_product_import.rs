//! Narrow trusted authored-data seam for the Android product distribution.
//!
//! Android ships `nux-capi` directly, so its explicit trusted-WGSL entrypoint
//! lives in this feature-gated leaf. The portable configured-import symbols
//! remain product-neutral and cannot assert trusted-exporter provenance.

#[cfg(feature = "android-authored-wgsl")]
use super::{NuxAndroidVulkanRenderer, NuxCapiResult, NuxFile, NuxFileImportConfig, NuxStatus};

/// Import caller-authenticated product bytes after enabling the authored-data
/// converter format and trusted WGSL-exporter authority.
///
/// This function performs no package or signature verification. The Android
/// product caller must verify the signed release envelope before selecting
/// this entrypoint, and must establish that every shader payload came from the
/// trusted exporter. A signature over arbitrary WGSL is insufficient.
///
/// # Safety
///
/// The pointers and lengths must satisfy the same contract as
/// [`super::nux_file_import_android_vulkan`].
#[unsafe(no_mangle)]
#[cfg(feature = "android-authored-wgsl")]
pub unsafe extern "C" fn nux_file_import_android_vulkan_with_trusted_wgsl(
    renderer: *mut NuxAndroidVulkanRenderer,
    bytes: *const u8,
    len: usize,
    config: *const NuxFileImportConfig,
    out_file: *mut *mut NuxFile,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    unsafe {
        super::android_vulkan::import_android_vulkan_file_with_authority(
            renderer,
            bytes,
            len,
            config,
            out_file,
            out_result,
            super::NativeShaderImportAuthority::TrustedExporter,
            Some(nuxie_project_data_scripting::ProjectDataScriptProgramAdapter::shared()),
        )
    }
}

#[cfg(all(test, feature = "android-authored-wgsl"))]
mod tests {
    use std::any::Any;
    use std::cell::Cell;
    use std::ptr;
    use std::rc::Rc;
    use std::sync::Arc;

    use luaur_compiler::functions::luau_compile::luau_compile;
    use nuxie::render_api::RawPath;
    use nuxie::{
        ArtboardInstance, ColorInt, Factory, FillRule, GpuCanvasError, GpuCanvasShader,
        ImageDecodeError, PersistentFactory, RenderBuffer, RenderBufferFlags, RenderBufferType,
        RenderGpuCanvasShader, RenderImage, RenderPaint, RenderPath, RenderShader,
        ScriptedDrawable,
    };
    use nuxie_render_api::RecordingFactory;
    use nuxie_schema::definition_by_name;
    use sha2::{Digest, Sha256};

    use super::*;
    use crate::{NativeShaderImportAuthority, NuxHostCommandImportConfig, NuxStringView};

    const SCRIPT: &[u8] = br#"
return function(context)
    local shader = context:shader("scene")
    local pipeline = GPUPipeline.new {
        vertex = { module = shader, entryPoint = "vs_main" },
        fragment = { module = shader, entryPoint = "fs_main" },
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
    }
    return { draw = function(self, renderer) end }
end
"#;

    const PINNED_GPU_CANVAS_WGSL: &str =
        include_str!("../tests/fixtures/univ-2781-gpu-canvas.wgsl");
    const PINNED_GPU_CANVAS_BINDING_MAP: &[u8] = &[
        0x02, 0x01, 0x0e, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0xff, 0xff,
        0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00,
    ];

    fn compile_luau(source: &[u8]) -> Vec<u8> {
        luaur_common::set_all_flags(true);
        let mut output_size = 0;
        let output = luau_compile(
            source.as_ptr().cast(),
            source.len(),
            ptr::null_mut(),
            &mut output_size,
        );
        assert!(!output.is_null());
        // SAFETY: luaur returns a valid allocation containing output_size bytes.
        unsafe { std::slice::from_raw_parts(output.cast(), output_size) }.to_vec()
    }

    fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn property_key(type_name: &str, property_name: &str) -> u16 {
        let definition = definition_by_name(type_name).expect("fixture type exists");
        definition
            .properties
            .iter()
            .chain(definition.ancestors.iter().flat_map(|ancestor| {
                definition_by_name(ancestor)
                    .expect("fixture ancestor exists")
                    .properties
                    .iter()
            }))
            .find(|property| property.name == property_name)
            .expect("fixture property exists")
            .key
            .int
    }

    fn push_object(bytes: &mut Vec<u8>, type_name: &str, body: impl FnOnce(&mut Vec<u8>)) {
        push_var_uint(
            bytes,
            u64::from(
                definition_by_name(type_name)
                    .expect("fixture type exists")
                    .type_key
                    .int,
            ),
        );
        body(bytes);
        push_var_uint(bytes, 0);
    }

    fn push_uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
        push_var_uint(bytes, u64::from(property_key(type_name, name)));
        push_var_uint(bytes, value);
    }

    fn push_f32(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: f32) {
        push_var_uint(bytes, u64::from(property_key(type_name, name)));
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
        push_var_uint(bytes, u64::from(property_key(type_name, name)));
        push_var_uint(bytes, value.len() as u64);
        bytes.extend_from_slice(value);
    }

    fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
        push_blob(bytes, type_name, name, value.as_bytes());
    }

    fn put_string(bytes: &mut Vec<u8>, value: &str) {
        bytes.extend_from_slice(&(value.len() as u16).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }

    fn shader_payload() -> Vec<u8> {
        let mut source = vec![2, 0];
        put_string(&mut source, "vs_main");
        put_string(&mut source, "vs_main");
        source.push(1);
        put_string(&mut source, "fs_main");
        put_string(&mut source, "fs_main");
        source.extend_from_slice(&(PINNED_GPU_CANVAS_WGSL.len() as u32).to_le_bytes());
        source.extend_from_slice(PINNED_GPU_CANVAS_WGSL.as_bytes());

        let mut payload = vec![0];
        payload.extend_from_slice(&0x5253_5442u32.to_le_bytes());
        payload.extend_from_slice(&4u16.to_le_bytes());
        payload.extend_from_slice(&[2, 0, 0]);
        payload.extend_from_slice(&0u32.to_le_bytes());
        payload.extend_from_slice(&(source.len() as u32).to_le_bytes());
        payload.push(16);
        payload.extend_from_slice(&(source.len() as u32).to_le_bytes());
        payload.extend_from_slice(&(PINNED_GPU_CANVAS_BINDING_MAP.len() as u32).to_le_bytes());
        payload.extend(source);
        payload.extend_from_slice(PINNED_GPU_CANVAS_BINDING_MAP);
        payload
    }

    fn imported_file() -> Vec<u8> {
        let mut script_payload = vec![0];
        script_payload.extend(compile_luau(SCRIPT));
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 991);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "ShaderAsset", |bytes| {
            push_uint(bytes, "ShaderAsset", "assetId", 0);
            push_string(bytes, "ShaderAsset", "name", "scene");
        });
        push_object(&mut bytes, "FileAssetContents", |bytes| {
            push_blob(bytes, "FileAssetContents", "bytes", &shader_payload());
        });
        push_object(&mut bytes, "ScriptAsset", |bytes| {
            push_uint(bytes, "ScriptAsset", "assetId", 1);
            push_string(bytes, "ScriptAsset", "name", "ShaderProbe");
        });
        push_object(&mut bytes, "FileAssetContents", |bytes| {
            push_blob(bytes, "FileAssetContents", "bytes", &script_payload);
        });
        push_object(&mut bytes, "Artboard", |bytes| {
            push_f32(bytes, "Artboard", "width", 32.0);
            push_f32(bytes, "Artboard", "height", 24.0);
        });
        push_object(&mut bytes, "ScriptedDrawable", |bytes| {
            push_uint(bytes, "ScriptedDrawable", "parentId", 0);
            push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 1);
        });
        bytes
    }

    struct ShaderOccurrence;

    impl RenderGpuCanvasShader for ShaderOccurrence {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    struct ShaderProbeFactory {
        inner: RecordingFactory,
        shader_count: Rc<Cell<usize>>,
    }

    impl Factory for ShaderProbeFactory {
        fn make_render_buffer(
            &mut self,
            buffer_type: RenderBufferType,
            flags: RenderBufferFlags,
            size_in_bytes: usize,
        ) -> Box<dyn RenderBuffer> {
            self.inner
                .make_render_buffer(buffer_type, flags, size_in_bytes)
        }

        fn make_linear_gradient(
            &mut self,
            sx: f32,
            sy: f32,
            ex: f32,
            ey: f32,
            colors: &[ColorInt],
            stops: &[f32],
        ) -> Box<dyn RenderShader> {
            self.inner
                .make_linear_gradient(sx, sy, ex, ey, colors, stops)
        }

        fn make_radial_gradient(
            &mut self,
            cx: f32,
            cy: f32,
            radius: f32,
            colors: &[ColorInt],
            stops: &[f32],
        ) -> Box<dyn RenderShader> {
            self.inner
                .make_radial_gradient(cx, cy, radius, colors, stops)
        }

        fn make_render_path(&mut self, path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
            self.inner.make_render_path(path, fill_rule)
        }

        fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
            self.inner.make_empty_render_path()
        }

        fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
            self.inner.make_render_paint()
        }

        fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
            self.inner.decode_image(data)
        }

        fn make_gpu_canvas_shader(
            &mut self,
            _shader: &GpuCanvasShader,
        ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
            self.shader_count.set(
                self.shader_count
                    .get()
                    .checked_add(1)
                    .expect("fixture shader count overflow"),
            );
            Ok(Arc::new(ShaderOccurrence))
        }
    }

    fn string_view(value: &str) -> NuxStringView {
        NuxStringView {
            data: value.as_ptr().cast(),
            len: value.len(),
        }
    }

    fn import_and_draw(
        bytes: &[u8],
        authority: NativeShaderImportAuthority,
    ) -> (Result<bool, String>, usize) {
        let host = NuxHostCommandImportConfig {
            module_name: string_view("bridge"),
            ..NuxHostCommandImportConfig::default()
        };
        let prepared = unsafe { crate::prepare_optional_host_command_import(&host) }
            .expect("valid host import config");
        let shader_count = Rc::new(Cell::new(0));
        let mut factory = PersistentFactory::new(ShaderProbeFactory {
            inner: RecordingFactory::new(),
            shader_count: Rc::clone(&shader_count),
        });
        let imported = match crate::import_file_with_prepared_host_commands(
            bytes,
            &mut factory,
            None,
            prepared,
            authority,
            None,
        ) {
            Ok(imported) => imported,
            Err(error) => return (Err(error), shader_count.get()),
        };
        let _scripted = imported.scripted;
        let mut instance = match ArtboardInstance::from_native(imported.file, 0) {
            Ok(instance) => instance,
            Err(error) => return (Err(format!("{error:#}")), shader_count.get()),
        };
        let initialized = (0..instance.object_count())
            .filter_map(|index| instance.object_handle(index))
            .filter_map(|object| {
                object.with_downcast::<ScriptedDrawable, _>(|drawable| {
                    drawable.scripted.self_ref() != 0
                })
            })
            .any(|initialized| initialized);
        if let Err(error) = instance.advance(0.0) {
            return (Err(format!("{error:#}")), shader_count.get());
        }
        let mut renderer = factory.borrow().inner.make_renderer();
        instance.draw(&mut renderer);
        (Ok(initialized), shader_count.get())
    }

    #[test]
    fn only_android_product_import_builds_the_pinned_gpu_canvas_pipeline() {
        let source_digest: [u8; 32] = Sha256::digest(PINNED_GPU_CANVAS_WGSL.as_bytes()).into();
        assert_eq!(
            source_digest,
            [
                0xae, 0x1a, 0xa9, 0x70, 0xef, 0x1f, 0xf3, 0x92, 0x6f, 0x18, 0xed, 0xbb, 0xa5, 0xae,
                0x51, 0x9c, 0x62, 0xff, 0xb3, 0x20, 0x36, 0x0e, 0xe3, 0x71, 0x55, 0x33, 0x02, 0xb1,
                0x6b, 0x2b, 0xdb, 0x8b,
            ],
            "the local shader must stay byte-identical to the pinned exact-source fixture"
        );
        let payload = shader_payload();
        let payload_digest: [u8; 32] = Sha256::digest(&payload).into();
        assert_eq!(payload.len(), 739);
        assert_eq!(
            payload_digest,
            [
                0xba, 0x47, 0x8e, 0x0e, 0xdb, 0xc5, 0xf7, 0xbf, 0xaa, 0x73, 0x20, 0x90, 0xb1, 0x30,
                0xb1, 0xc9, 0xd0, 0x6c, 0x1d, 0x60, 0x40, 0x54, 0x99, 0x56, 0xd1, 0xcc, 0xe0, 0x1e,
                0xbe, 0xdf, 0x87, 0xca,
            ],
            "the target-0/target-16 RSTB payload must remain pinned"
        );
        let bytes = imported_file();

        let (generic_initialization, generic_shader_count) =
            import_and_draw(&bytes, NativeShaderImportAuthority::Denied);
        assert_eq!(
            generic_initialization.expect("generic exact import must remain script-safe"),
            false,
            "the exact runtime must leave a scripted drawable inert when its shader lacks native provenance"
        );
        assert_eq!(generic_shader_count, 0);

        let (product_initialization, product_shader_count) =
            import_and_draw(&bytes, NativeShaderImportAuthority::TrustedExporter);
        assert!(
            product_initialization
                .expect("trusted product import must expose a working Shader object"),
            "the exact artboard constructor must initialize the trusted scripted drawable"
        );
        assert_eq!(product_shader_count, 1);
    }
}
