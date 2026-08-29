//! Bounded, product-neutral asset hooks for the native C distribution.

use super::{
    HandleKind, NuxByteView, NuxCapiResult, NuxFile, NuxStatus, PendingHandlePublication,
    ffi_guard_with_handle_result, publish_result, register_handle, struct_size_supports,
    with_platform_callback,
};
use nuxie::render_api::RawPath;
use nuxie::{
    AudioAsset, ColorInt, CoreHandle, Factory, FileAssetLoader, FileAssetLoaderRef, FillRule,
    FontAsset, GpuCanvasError, GpuCanvasPipelineShaders, GpuCanvasPlan, GpuCanvasShader,
    GpuCanvasShaderArtifact, GpuCanvasShaderLoad, GpuCanvasShaderProfile, ImageAsset,
    ImageDecodeError, PersistentFactory, RenderBuffer, RenderBufferFlags, RenderBufferType,
    RenderCanvas, RenderCanvasError, RenderGpuCanvasShader, RenderImage, RenderPaint, RenderPath,
    RenderShader, RuntimeFactoryHandle,
};
use std::collections::HashMap;
use std::ffi::c_void;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::sync::Arc;
use std::thread;

pub type NuxRetainCallback = unsafe extern "C" fn(owner: *mut c_void);
pub type NuxReleaseCallback = unsafe extern "C" fn(owner: *mut c_void);

/// Callback-produced bytes with an explicit borrowed-owner retain cycle.
///
/// When `struct_size` covers the callback fields and both callbacks are
/// present, Rust invokes `retain(owner)` exactly once before inspecting `data`,
/// and `release(owner)` exactly once after copying or rejecting it. A short
/// prefix or a missing half of the pair transfers no ownership and is rejected
/// without invoking either callback.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxRetainedBytes {
    pub struct_size: u32,
    pub data: *const u8,
    pub len: usize,
    pub owner: *mut c_void,
    pub retain: Option<unsafe extern "C" fn(owner: *mut c_void)>,
    pub release: Option<unsafe extern "C" fn(owner: *mut c_void)>,
}

pub const NUX_RETAINED_BYTES_V3_MIN_SIZE: usize = std::mem::offset_of!(NuxRetainedBytes, release)
    + std::mem::size_of::<Option<unsafe extern "C" fn(owner: *mut c_void)>>();

impl Default for NuxRetainedBytes {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            data: ptr::null(),
            len: 0,
            owner: ptr::null_mut(),
            retain: None,
            release: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RetainedBytesError {
    InvalidStructSize,
    InvalidOwnership,
    NullData,
    LimitExceeded,
}

struct RetainedBytesLease {
    bytes: NuxRetainedBytes,
    release: NuxReleaseCallback,
}

impl RetainedBytesLease {
    fn as_slice(&self) -> &[u8] {
        if self.bytes.len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.bytes.data, self.bytes.len) }
        }
    }
}

impl Drop for RetainedBytesLease {
    fn drop(&mut self) {
        let owner = self.bytes.owner;
        let release = self.release;
        with_platform_callback(|| unsafe { release(owner) });
    }
}

fn retain_bytes(
    bytes: NuxRetainedBytes,
    maximum: usize,
) -> Result<RetainedBytesLease, RetainedBytesError> {
    if !struct_size_supports(bytes.struct_size, NUX_RETAINED_BYTES_V3_MIN_SIZE) {
        return Err(RetainedBytesError::InvalidStructSize);
    }
    let (Some(retain), Some(release)) = (bytes.retain, bytes.release) else {
        return Err(RetainedBytesError::InvalidOwnership);
    };
    let owner = bytes.owner;
    with_platform_callback(|| unsafe { retain(owner) });
    let lease = RetainedBytesLease { bytes, release };
    if lease.bytes.len > maximum {
        return Err(RetainedBytesError::LimitExceeded);
    }
    if lease.bytes.data.is_null() && lease.bytes.len != 0 {
        return Err(RetainedBytesError::NullData);
    }
    Ok(lease)
}

fn copy_retained_bytes(
    bytes: NuxRetainedBytes,
    maximum: usize,
) -> Result<Vec<u8>, RetainedBytesError> {
    let lease = retain_bytes(bytes, maximum)?;
    Ok(lease.as_slice().to_vec())
}

pub type NuxPixelFormat = u32;
pub const NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB: NuxPixelFormat = 1;

/// Host-decoded image pixels. The only accepted format is RGBA8,
/// premultiplied-alpha, sRGB; Rust validates and tightly repacks each row before
/// any renderer upload.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxDecodedImage {
    pub struct_size: u32,
    pub width: u32,
    pub height: u32,
    pub row_bytes: u32,
    pub pixel_format: NuxPixelFormat,
    pub pixels: NuxRetainedBytes,
}

pub const NUX_DECODED_IMAGE_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxDecodedImage, pixels) + std::mem::size_of::<NuxRetainedBytes>();

impl Default for NuxDecodedImage {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            width: 0,
            height: 0,
            row_bytes: 0,
            pixel_format: 0,
            pixels: NuxRetainedBytes::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CanonicalImage {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) row_bytes: u32,
    pub(crate) pixels: Arc<[u8]>,
}

#[derive(Debug, Default)]
pub(crate) struct AssetCatalog {
    pub(crate) decoded_images: HashMap<Vec<u8>, CanonicalImage>,
}

pub(crate) trait AssetUploadFactory: Factory + 'static {
    fn upload_rgba8_premul_srgb(
        &mut self,
        width: u32,
        height: u32,
        row_bytes: u32,
        pixels: &[u8],
    ) -> Result<Box<dyn RenderImage>, ImageDecodeError>;
}

// Asset-policy unit tests exercise the same factory-at-import ownership path
// as production without depending on a host GPU being available. The null
// factory is test-only; no factory-free C entry point is exported.
#[cfg(test)]
impl AssetUploadFactory for nuxie::render_api::NullFactory {
    fn upload_rgba8_premul_srgb(
        &mut self,
        _width: u32,
        _height: u32,
        _row_bytes: u32,
        _pixels: &[u8],
    ) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.decode_image(&[])
    }
}

pub(crate) struct AssetFactory<F> {
    inner: F,
    decoded_images: HashMap<Vec<u8>, CanonicalImage>,
}

impl<F> AssetFactory<F> {
    pub(crate) fn new(inner: F) -> Self {
        Self {
            inner,
            decoded_images: HashMap::new(),
        }
    }

    pub(crate) fn install_catalog(&mut self, catalog: &AssetCatalog) {
        self.decoded_images.extend(
            catalog
                .decoded_images
                .iter()
                .map(|(encoded, image)| (encoded.clone(), image.clone())),
        );
    }

    pub(crate) fn replace_inner(&mut self, inner: F) {
        self.inner = inner;
    }
}

impl<F> Deref for AssetFactory<F> {
    type Target = F;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<F> DerefMut for AssetFactory<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<F: AssetUploadFactory> Factory for AssetFactory<F> {
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

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        self.inner.make_render_path(raw_path, fill_rule)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.inner.make_empty_render_path()
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.inner.make_render_paint()
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        let Some(image) = self.decoded_images.get(data) else {
            return self.inner.decode_image(data);
        };
        self.inner.upload_rgba8_premul_srgb(
            image.width,
            image.height,
            image.row_bytes,
            &image.pixels,
        )
    }

    fn make_render_canvas(
        &mut self,
        width: u32,
        height: u32,
    ) -> Result<Box<dyn RenderCanvas>, RenderCanvasError> {
        self.inner.make_render_canvas(width, height)
    }

    fn make_gpu_canvas_image_view(
        &mut self,
        image: std::rc::Rc<dyn RenderImage>,
    ) -> Result<std::rc::Rc<dyn RenderImage>, GpuCanvasError> {
        self.inner.make_gpu_canvas_image_view(image)
    }

    fn make_gpu_canvas_shader(
        &mut self,
        shader: &GpuCanvasShader,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.inner.make_gpu_canvas_shader(shader)
    }

    fn load_gpu_canvas_shader(&mut self, shader: &GpuCanvasShader) -> GpuCanvasShaderLoad {
        self.inner.load_gpu_canvas_shader(shader)
    }

    fn gpu_canvas_shader_profile(&self) -> GpuCanvasShaderProfile {
        self.inner.gpu_canvas_shader_profile()
    }

    fn make_gpu_canvas_shader_artifact(
        &mut self,
        shader: &GpuCanvasShaderArtifact,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.inner.make_gpu_canvas_shader_artifact(shader)
    }

    fn load_gpu_canvas_shader_artifact(
        &mut self,
        shader: &GpuCanvasShaderArtifact,
    ) -> GpuCanvasShaderLoad {
        self.inner.load_gpu_canvas_shader_artifact(shader)
    }

    fn make_gpu_canvas_shader_occurrence(
        &mut self,
        prepared: &Arc<dyn RenderGpuCanvasShader>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.inner.make_gpu_canvas_shader_occurrence(prepared)
    }

    fn make_gpu_canvas_image(
        &mut self,
        vertex_shader: &Arc<dyn RenderGpuCanvasShader>,
        fragment_shader: &Arc<dyn RenderGpuCanvasShader>,
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        self.inner
            .make_gpu_canvas_image(vertex_shader, fragment_shader, plan)
    }

    fn make_gpu_canvas_image_with_pipelines(
        &mut self,
        pipelines: &[GpuCanvasPipelineShaders],
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        self.inner
            .make_gpu_canvas_image_with_pipelines(pipelines, plan)
    }
}

pub type NuxAssetCallbackStatus = u32;
pub const NUX_ASSET_CALLBACK_STATUS_OK: NuxAssetCallbackStatus = 0;
pub const NUX_ASSET_CALLBACK_STATUS_NOT_FOUND: NuxAssetCallbackStatus = 1;
pub const NUX_ASSET_CALLBACK_STATUS_FAILED: NuxAssetCallbackStatus = 2;

pub type NuxAssetKind = u32;
pub const NUX_ASSET_KIND_IMAGE: NuxAssetKind = 1;
pub const NUX_ASSET_KIND_FONT: NuxAssetKind = 2;
pub const NUX_ASSET_KIND_AUDIO: NuxAssetKind = 3;
pub const NUX_ASSET_ID_NONE: u32 = u32::MAX;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxExternalAssetRequest {
    pub struct_size: u32,
    pub kind: NuxAssetKind,
    pub asset_index: usize,
    pub asset_id: u32,
    pub name: super::NuxStringView,
    pub file_extension: super::NuxStringView,
}

pub const NUX_EXTERNAL_ASSET_REQUEST_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxExternalAssetRequest, file_extension)
        + std::mem::size_of::<super::NuxStringView>();

pub type NuxLookupExternalAssetCallback = unsafe extern "C" fn(
    context: *mut c_void,
    request: *const NuxExternalAssetRequest,
    out_bytes: *mut NuxRetainedBytes,
) -> NuxAssetCallbackStatus;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxImageDecodeRequest {
    pub struct_size: u32,
    /// Encoded bytes borrowed only for the synchronous callback invocation.
    pub encoded: NuxByteView,
    pub maximum_dimension: u32,
    pub maximum_decoded_bytes: usize,
}

pub const NUX_IMAGE_DECODE_REQUEST_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxImageDecodeRequest, maximum_decoded_bytes)
        + std::mem::size_of::<usize>();

impl Default for NuxImageDecodeRequest {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            encoded: NuxByteView::default(),
            maximum_dimension: 8_192,
            maximum_decoded_bytes: 256 * 1024 * 1024,
        }
    }
}

pub type NuxDecodeImageCallback = unsafe extern "C" fn(
    context: *mut c_void,
    request: *const NuxImageDecodeRequest,
    out_image: *mut NuxDecodedImage,
) -> NuxAssetCallbackStatus;

/// One synchronous, versioned asset import surface. The table and all
/// request views are borrowed only until `nux_file_import_with_assets`
/// returns; callback-owned outputs use `NuxRetainedBytes` instead.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxAssetHooks {
    pub struct_size: u32,
    pub context: *mut c_void,
    pub lookup_external_asset: Option<
        unsafe extern "C" fn(
            context: *mut c_void,
            request: *const NuxExternalAssetRequest,
            out_bytes: *mut NuxRetainedBytes,
        ) -> NuxAssetCallbackStatus,
    >,
    pub decode_image: Option<
        unsafe extern "C" fn(
            context: *mut c_void,
            request: *const NuxImageDecodeRequest,
            out_image: *mut NuxDecodedImage,
        ) -> NuxAssetCallbackStatus,
    >,
    pub maximum_external_asset_bytes: usize,
    pub maximum_total_external_asset_bytes: usize,
    pub maximum_image_dimension: u32,
    pub maximum_decoded_image_bytes: usize,
    pub maximum_total_decoded_image_bytes: usize,
}

pub const NUX_ASSET_HOOKS_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxAssetHooks, maximum_total_decoded_image_bytes)
        + std::mem::size_of::<usize>();

impl Default for NuxAssetHooks {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            context: ptr::null_mut(),
            lookup_external_asset: None,
            decode_image: None,
            maximum_external_asset_bytes: 64 * 1024 * 1024,
            maximum_total_external_asset_bytes: 256 * 1024 * 1024,
            maximum_image_dimension: 8_192,
            maximum_decoded_image_bytes: 256 * 1024 * 1024,
            maximum_total_decoded_image_bytes: 512 * 1024 * 1024,
        }
    }
}

/// Reads only the caller-provided prefix. Reading `struct_size` first avoids
/// forming a reference to the runtime's larger current table when an older or
/// malformed caller supplied a shorter allocation.
unsafe fn read_asset_hooks(hooks: *const NuxAssetHooks) -> Result<NuxAssetHooks, NuxStatus> {
    if hooks.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let caller_size = unsafe { hooks.cast::<u32>().read() };
    if !struct_size_supports(caller_size, NUX_ASSET_HOOKS_V3_MIN_SIZE) {
        return Err(NuxStatus::InvalidStructSize);
    }
    let mut value = NuxAssetHooks::default();
    let read_len = usize::try_from(caller_size)
        .unwrap_or(usize::MAX)
        .min(std::mem::size_of::<NuxAssetHooks>());
    unsafe {
        ptr::copy_nonoverlapping(
            hooks.cast::<u8>(),
            (&mut value as *mut NuxAssetHooks).cast::<u8>(),
            read_len,
        );
    }
    Ok(value)
}

#[derive(Debug)]
enum AssetImportError {
    InvalidHooks,
    DecodeCallbackFailed,
    Decode(DecodedImageError),
    TotalLimitExceeded,
    LookupCallbackFailed,
    InvalidExternalAsset,
}

impl AssetImportError {
    fn status(&self) -> NuxStatus {
        match self {
            Self::InvalidHooks => NuxStatus::InvalidStructSize,
            Self::TotalLimitExceeded | Self::Decode(DecodedImageError::LimitExceeded) => {
                NuxStatus::LimitExceeded
            }
            Self::DecodeCallbackFailed
            | Self::LookupCallbackFailed
            | Self::InvalidExternalAsset
            | Self::Decode(_) => NuxStatus::ImportError,
        }
    }
}

fn string_view(value: &str) -> super::NuxStringView {
    super::NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

fn asset_kind(type_name: &str) -> Option<NuxAssetKind> {
    match type_name {
        "ImageAsset" => Some(NUX_ASSET_KIND_IMAGE),
        "FontAsset" => Some(NUX_ASSET_KIND_FONT),
        "AudioAsset" => Some(NUX_ASSET_KIND_AUDIO),
        _ => None,
    }
}

fn validate_asset_hook_policy(hooks: NuxAssetHooks) -> Result<NuxAssetHooks, AssetImportError> {
    if !struct_size_supports(hooks.struct_size, NUX_ASSET_HOOKS_V3_MIN_SIZE)
        || hooks.maximum_image_dimension == 0
        || hooks.maximum_external_asset_bytes == 0
        || hooks.maximum_total_external_asset_bytes == 0
        || hooks.maximum_decoded_image_bytes == 0
        || hooks.maximum_total_decoded_image_bytes == 0
    {
        return Err(AssetImportError::InvalidHooks);
    }
    Ok(hooks)
}

struct ExternalAssetBytes {
    kind: NuxAssetKind,
    authored_id: u32,
    name: String,
    bytes: Vec<u8>,
}

struct HookAssetLoader {
    external: Vec<ExternalAssetBytes>,
}

impl FileAssetLoader for HookAssetLoader {
    fn load_contents(
        &mut self,
        asset: CoreHandle,
        in_band_bytes: &[u8],
        factory: &RuntimeFactoryHandle,
    ) -> bool {
        if !in_band_bytes.is_empty() {
            return false;
        }
        let identity = asset
            .with_downcast::<ImageAsset, _>(|asset| {
                let base = asset.base.file_asset();
                (
                    NUX_ASSET_KIND_IMAGE,
                    base.asset_id(),
                    base.name().to_owned(),
                )
            })
            .or_else(|| {
                asset.with_downcast::<FontAsset, _>(|asset| {
                    let base = asset.base.file_asset();
                    (NUX_ASSET_KIND_FONT, base.asset_id(), base.name().to_owned())
                })
            })
            .or_else(|| {
                asset.with_downcast::<AudioAsset, _>(|asset| {
                    let base = asset.base.file_asset();
                    (
                        NUX_ASSET_KIND_AUDIO,
                        base.asset_id(),
                        base.name().to_owned(),
                    )
                })
            });
        let Some((kind, authored_id, name)) = identity else {
            return false;
        };
        let Some(index) = self.external.iter().position(|entry| {
            entry.kind == kind && entry.authored_id == authored_id && entry.name == name
        }) else {
            return false;
        };
        let mut entry = self.external.swap_remove(index);
        if kind == NUX_ASSET_KIND_IMAGE {
            return asset
                .with_downcast_mut::<ImageAsset, _>(|asset| asset.decode(&entry.bytes, factory))
                .unwrap_or(false);
        }
        if kind == NUX_ASSET_KIND_FONT {
            return asset
                .with_downcast_mut::<FontAsset, _>(|asset| asset.decode(&entry.bytes, factory))
                .unwrap_or(false);
        }
        asset
            .with_downcast_mut::<AudioAsset, _>(|asset| asset.decode(&mut entry.bytes, factory))
            .unwrap_or(false)
    }
}

pub(crate) struct PreparedAssets {
    pub(crate) catalog: Arc<AssetCatalog>,
    pub(crate) loader: Option<FileAssetLoaderRef>,
}

pub(crate) fn prepare_assets(
    bytes: &[u8],
    hooks: NuxAssetHooks,
) -> Result<PreparedAssets, AssetImportError> {
    debug_assert!(validate_asset_hook_policy(hooks).is_ok());
    let parsed = nuxie_binary::read_runtime_file_with_scripting(bytes)
        .map_err(|_| AssetImportError::InvalidExternalAsset)?;
    let assets = parsed.scripting_file_assets_with_contents();
    let mut image_sources = Vec::new();
    let mut external_assets = Vec::new();
    let mut external_total = 0usize;
    for entry in assets {
        let Some(kind) = asset_kind(entry.asset.type_name) else {
            continue;
        };
        let asset_id = entry
            .asset
            .uint_property("assetId")
            .and_then(|value| u32::try_from(value).ok());
        let name = entry
            .asset
            .string_property("name")
            .unwrap_or_default()
            .to_owned();
        let extension = entry.asset.file_asset_extension().unwrap_or_default();
        let is_external = entry.contents.is_none();
        let asset_bytes = if let Some(embedded) = entry.contents {
            Some(embedded.to_vec())
        } else if let Some(lookup) = hooks.lookup_external_asset {
            let request = NuxExternalAssetRequest {
                struct_size: u32::try_from(std::mem::size_of::<NuxExternalAssetRequest>())
                    .unwrap_or(u32::MAX),
                kind,
                asset_index: entry.ordinal,
                asset_id: asset_id.unwrap_or(NUX_ASSET_ID_NONE),
                name: string_view(&name),
                file_extension: string_view(extension),
            };
            let mut returned = NuxRetainedBytes::default();
            let status = with_platform_callback(|| unsafe {
                lookup(hooks.context, &request, &mut returned)
            });
            let complete_owner = returned.retain.is_some() && returned.release.is_some();
            match status {
                NUX_ASSET_CALLBACK_STATUS_OK => {
                    let remaining = hooks
                        .maximum_total_external_asset_bytes
                        .checked_sub(external_total)
                        .ok_or(AssetImportError::TotalLimitExceeded)?;
                    let item_budget = hooks.maximum_external_asset_bytes.min(remaining);
                    let copied =
                        copy_retained_bytes(returned, item_budget).map_err(
                            |error| match error {
                                RetainedBytesError::LimitExceeded => {
                                    AssetImportError::TotalLimitExceeded
                                }
                                _ => AssetImportError::InvalidExternalAsset,
                            },
                        )?;
                    external_total = external_total
                        .checked_add(copied.len())
                        .ok_or(AssetImportError::TotalLimitExceeded)?;
                    debug_assert!(external_total <= hooks.maximum_total_external_asset_bytes);
                    Some(copied)
                }
                NUX_ASSET_CALLBACK_STATUS_NOT_FOUND => {
                    if complete_owner {
                        let _ = retain_bytes(returned, hooks.maximum_external_asset_bytes);
                    }
                    None
                }
                _ => {
                    if complete_owner {
                        let _ = retain_bytes(returned, hooks.maximum_external_asset_bytes);
                    }
                    return Err(AssetImportError::LookupCallbackFailed);
                }
            }
        } else {
            None
        };
        let Some(asset_bytes) = asset_bytes else {
            continue;
        };
        if kind == NUX_ASSET_KIND_IMAGE {
            image_sources.push(asset_bytes.clone());
        }
        if is_external {
            external_assets.push(ExternalAssetBytes {
                kind,
                authored_id: asset_id.ok_or(AssetImportError::InvalidExternalAsset)?,
                name,
                bytes: asset_bytes,
            });
        }
    }

    let mut catalog = AssetCatalog::default();
    let mut total = 0usize;
    let Some(decode_image) = hooks.decode_image else {
        let loader = (!external_assets.is_empty()).then(|| {
            FileAssetLoaderRef::new(Box::new(HookAssetLoader {
                external: external_assets,
            }))
        });
        return Ok(PreparedAssets {
            catalog: Arc::new(catalog),
            loader,
        });
    };
    for encoded in image_sources {
        let remaining = hooks
            .maximum_total_decoded_image_bytes
            .checked_sub(total)
            .ok_or(AssetImportError::TotalLimitExceeded)?;
        let item_budget = hooks.maximum_decoded_image_bytes.min(remaining);
        let (_, _, packed_len) =
            preflight_encoded_image(&encoded, hooks.maximum_image_dimension, item_budget)
                .map_err(AssetImportError::Decode)?;
        let request = NuxImageDecodeRequest {
            encoded: NuxByteView {
                data: encoded.as_ptr(),
                len: encoded.len(),
            },
            maximum_dimension: hooks.maximum_image_dimension,
            maximum_decoded_bytes: item_budget,
            ..NuxImageDecodeRequest::default()
        };
        let mut decoded = NuxDecodedImage::default();
        let callback_status = with_platform_callback(|| unsafe {
            decode_image(hooks.context, &request, &mut decoded)
        });
        if callback_status != NUX_ASSET_CALLBACK_STATUS_OK {
            // A callback that reports failure still owns any complete returned
            // buffer pair. Acquire-and-drop balances it exactly once.
            if decoded.pixels.retain.is_some() && decoded.pixels.release.is_some() {
                let _ = retain_bytes(decoded.pixels, hooks.maximum_decoded_image_bytes);
            }
            return Err(AssetImportError::DecodeCallbackFailed);
        }
        let canonical = validate_decoded_image(
            &encoded,
            decoded,
            hooks.maximum_image_dimension,
            item_budget,
        )
        .map_err(AssetImportError::Decode)?;
        debug_assert_eq!(canonical.pixels.len(), packed_len);
        total = total
            .checked_add(canonical.pixels.len())
            .ok_or(AssetImportError::TotalLimitExceeded)?;
        debug_assert!(total <= hooks.maximum_total_decoded_image_bytes);
        catalog.decoded_images.insert(encoded, canonical);
    }
    let loader = (!external_assets.is_empty()).then(|| {
        FileAssetLoaderRef::new(Box::new(HookAssetLoader {
            external: external_assets,
        }))
    });
    Ok(PreparedAssets {
        catalog: Arc::new(catalog),
        loader,
    })
}

/// One deep import surface. Each optional child is copied and validated in
/// full before file parsing or any platform callback. A null child pointer
/// leaves that capability inert.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFileImportConfig {
    pub struct_size: u32,
    pub host_commands: *const super::NuxHostCommandImportConfig,
    pub asset_hooks: *const NuxAssetHooks,
    pub expected_assets: *const super::NuxExpectedFileAssetDescriptor,
    pub expected_asset_count: usize,
}

impl Default for NuxFileImportConfig {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            host_commands: ptr::null(),
            asset_hooks: ptr::null(),
            expected_assets: ptr::null(),
            expected_asset_count: 0,
        }
    }
}

pub const NUX_FILE_IMPORT_CONFIG_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxFileImportConfig, expected_asset_count) + std::mem::size_of::<usize>();

unsafe fn read_file_import_config(
    config: *const NuxFileImportConfig,
) -> Result<NuxFileImportConfig, NuxStatus> {
    if config.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let caller_size = unsafe { config.cast::<u32>().read() };
    if !struct_size_supports(caller_size, NUX_FILE_IMPORT_CONFIG_V3_MIN_SIZE) {
        return Err(NuxStatus::InvalidStructSize);
    }
    let mut value = NuxFileImportConfig::default();
    let read_len = usize::try_from(caller_size)
        .unwrap_or(usize::MAX)
        .min(std::mem::size_of::<NuxFileImportConfig>());
    unsafe {
        ptr::copy_nonoverlapping(
            config.cast::<u8>(),
            (&mut value as *mut NuxFileImportConfig).cast::<u8>(),
            read_len,
        );
    }
    Ok(value)
}

pub(super) unsafe fn nux_file_import_configured_with_factory<F>(
    bytes: *const u8,
    len: usize,
    config: *const NuxFileImportConfig,
    out_file: *mut *mut NuxFile,
    out_result: *mut *mut NuxCapiResult,
    factory: PersistentFactory<AssetFactory<F>>,
    renderer_domain: super::RendererDomainBinding,
    native_shader_authority: super::NativeShaderImportAuthority,
    program_adapter: Option<Arc<dyn nuxie::ScriptProgramAdapter>>,
) -> NuxStatus
where
    F: AssetUploadFactory,
{
    ffi_guard_with_handle_result(out_file, out_result, HandleKind::File, || {
        if out_file.is_null() || out_result.is_null() || config.is_null() {
            if !out_result.is_null() {
                publish_result(
                    out_result,
                    NuxStatus::NullArgument,
                    "an input or output pointer is null",
                );
            }
            return NuxStatus::NullArgument;
        }
        if bytes.is_null() && len != 0 {
            publish_result(out_result, NuxStatus::NullArgument, "bytes is null");
            return NuxStatus::NullArgument;
        }
        let config = match unsafe { read_file_import_config(config) } {
            Ok(config) => config,
            Err(status) => {
                publish_result(out_result, status, "file import config prefix is invalid");
                return status;
            }
        };
        let host_commands =
            match unsafe { super::prepare_optional_host_command_import(config.host_commands) } {
                Ok(config) => config,
                Err((status, message)) => {
                    publish_result(out_result, status, message);
                    return status;
                }
            };
        let hooks = if config.asset_hooks.is_null() {
            None
        } else {
            match unsafe { read_asset_hooks(config.asset_hooks) }
                .map_err(|status| (status, "asset hook prefix is invalid"))
                .and_then(|hooks| {
                    validate_asset_hook_policy(hooks)
                        .map_err(|error| (error.status(), "asset hook policy is invalid"))
                }) {
                Ok(hooks) => Some(hooks),
                Err((status, message)) => {
                    publish_result(out_result, status, message);
                    return status;
                }
            }
        };
        let validates_expected_assets = !config.expected_assets.is_null();
        let expected_assets = match unsafe {
            super::asset_catalog::copy_expected_descriptors(
                config.expected_assets,
                config.expected_asset_count,
            )
        } {
            Ok(expected) => expected,
            Err((status, message)) => {
                publish_result(out_result, status, message);
                return status;
            }
        };
        let bytes = if len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(bytes, len) }
        };
        if validates_expected_assets {
            let metadata = match super::FileMetadataCatalog::assets_only_from_bytes(bytes) {
                Ok(metadata) => metadata,
                Err(error) => {
                    publish_result(out_result, NuxStatus::ImportError, error);
                    return NuxStatus::ImportError;
                }
            };
            if let Err(message) =
                super::asset_catalog::validate_expected_descriptors(&metadata, &expected_assets)
            {
                publish_result(out_result, NuxStatus::HandleMismatch, message);
                return NuxStatus::HandleMismatch;
            }
        }
        let prepared_assets = match hooks {
            Some(hooks) => match prepare_assets(bytes, hooks) {
                Ok(assets) => Some(assets),
                Err(error) => {
                    let status = error.status();
                    publish_result(
                        out_result,
                        status,
                        format!("asset import failed: {error:?}"),
                    );
                    return status;
                }
            },
            None => None,
        };
        if let Some(assets) = prepared_assets.as_ref() {
            factory.borrow_mut().install_catalog(&assets.catalog);
        }
        let loader = prepared_assets
            .as_ref()
            .and_then(|assets| assets.loader.clone());
        let mut import_factory = factory.clone();
        let imported = match super::import_file_with_prepared_host_commands(
            bytes,
            &mut import_factory,
            loader,
            host_commands,
            native_shader_authority,
            program_adapter,
        ) {
            Ok(file) => file,
            Err(error) => {
                publish_result(out_result, NuxStatus::ImportError, error.to_string());
                return NuxStatus::ImportError;
            }
        };
        let metadata = match super::FileMetadataCatalog::from_file(&imported.file, bytes) {
            Ok(metadata) => metadata,
            Err(error) => {
                publish_result(out_result, NuxStatus::ImportError, error);
                return NuxStatus::ImportError;
            }
        };
        let view_model_catalog = match super::data_binding::build_catalog_from_bytes(bytes) {
            Ok(catalog) => Arc::new(catalog),
            Err(status) => {
                publish_result(out_result, status, "view-model metadata import failed");
                return status;
            }
        };
        let pending = PendingHandlePublication::new(
            NuxFile {
                file: imported.file,
                metadata,
                view_model_catalog,
                #[cfg(feature = "scripting")]
                scripted: imported.scripted.map(std::rc::Rc::new),
                owner_thread: thread::current().id(),
                data_binding_provenance: Arc::new(()),
                renderer_domain,
                asset_hooks: prepared_assets.map(|assets| assets.catalog),
            },
            HandleKind::File,
        );
        register_handle(pending.handle, HandleKind::File, thread::current().id());
        unsafe { *out_file = pending.handle };
        let _ = pending.finish();
        publish_result(out_result, NuxStatus::Ok, "");
        NuxStatus::Ok
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecodedImageError {
    InvalidStructSize,
    InvalidEncodedImage,
    DimensionMismatch,
    DimensionLimit,
    InvalidPixelFormat,
    InvalidRowBytes,
    InvalidPixels,
    LimitExceeded,
}

fn preflight_encoded_image(
    encoded: &[u8],
    maximum_dimension: u32,
    maximum_decoded_bytes: usize,
) -> Result<(u32, u32, usize), DecodedImageError> {
    let dimensions = nuxie_image_codec::preflight_encoded_image(encoded)
        .ok_or(DecodedImageError::InvalidEncodedImage)?;
    if dimensions.width == 0
        || dimensions.height == 0
        || dimensions.width > maximum_dimension
        || dimensions.height > maximum_dimension
    {
        return Err(DecodedImageError::DimensionLimit);
    }
    let tight_row = dimensions
        .width
        .checked_mul(4)
        .ok_or(DecodedImageError::LimitExceeded)?;
    let packed_len = usize::try_from(tight_row)
        .ok()
        .and_then(|row| {
            usize::try_from(dimensions.height)
                .ok()
                .and_then(|height| row.checked_mul(height))
        })
        .ok_or(DecodedImageError::LimitExceeded)?;
    if packed_len > maximum_decoded_bytes {
        return Err(DecodedImageError::LimitExceeded);
    }
    Ok((dimensions.width, dimensions.height, packed_len))
}

fn validate_decoded_image(
    encoded: &[u8],
    decoded: NuxDecodedImage,
    maximum_dimension: u32,
    maximum_decoded_bytes: usize,
) -> Result<CanonicalImage, DecodedImageError> {
    if !struct_size_supports(decoded.struct_size, NUX_DECODED_IMAGE_V3_MIN_SIZE) {
        return Err(DecodedImageError::InvalidStructSize);
    }

    // Acquire callback ownership before semantic validation so every complete
    // returned owner pair is balanced on success and every rejection path.
    let pixels =
        retain_bytes(decoded.pixels, maximum_decoded_bytes).map_err(|error| match error {
            RetainedBytesError::LimitExceeded => DecodedImageError::LimitExceeded,
            RetainedBytesError::InvalidStructSize
            | RetainedBytesError::InvalidOwnership
            | RetainedBytesError::NullData => DecodedImageError::InvalidPixels,
        })?;
    let (encoded_width, encoded_height, packed_len) =
        preflight_encoded_image(encoded, maximum_dimension, maximum_decoded_bytes)?;
    if (decoded.width, decoded.height) != (encoded_width, encoded_height) {
        return Err(DecodedImageError::DimensionMismatch);
    }
    if decoded.pixel_format != NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB {
        return Err(DecodedImageError::InvalidPixelFormat);
    }
    let tight_row = decoded
        .width
        .checked_mul(4)
        .ok_or(DecodedImageError::InvalidRowBytes)?;
    if decoded.row_bytes < tight_row {
        return Err(DecodedImageError::InvalidRowBytes);
    }
    let height = usize::try_from(decoded.height).map_err(|_| DecodedImageError::LimitExceeded)?;
    let source_row =
        usize::try_from(decoded.row_bytes).map_err(|_| DecodedImageError::LimitExceeded)?;
    let required_source = source_row
        .checked_mul(height)
        .ok_or(DecodedImageError::LimitExceeded)?;
    if pixels.as_slice().len() < required_source {
        return Err(DecodedImageError::InvalidPixels);
    }
    let tight_row_usize =
        usize::try_from(tight_row).map_err(|_| DecodedImageError::LimitExceeded)?;
    debug_assert_eq!(tight_row_usize.checked_mul(height), Some(packed_len));
    let mut packed = Vec::with_capacity(packed_len);
    for row in 0..height {
        let start = row
            .checked_mul(source_row)
            .ok_or(DecodedImageError::LimitExceeded)?;
        let end = start
            .checked_add(tight_row_usize)
            .ok_or(DecodedImageError::LimitExceeded)?;
        packed.extend_from_slice(&pixels.as_slice()[start..end]);
    }
    Ok(CanonicalImage {
        width: decoded.width,
        height: decoded.height,
        row_bytes: tight_row,
        pixels: Arc::from(packed),
    })
}

// The production ABI has no factory-free configured import. This helper keeps
// policy/ownership unit tests on the exact factory-at-import path while using
// a deterministic null factory rather than requiring a platform GPU.
#[cfg(test)]
pub(crate) unsafe fn nux_file_import_configured(
    bytes: *const u8,
    len: usize,
    config: *const NuxFileImportConfig,
    out_file: *mut *mut NuxFile,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    if super::platform_callback_active() {
        if !out_file.is_null() {
            unsafe { *out_file = ptr::null_mut() };
        }
        if !out_result.is_null() {
            unsafe { *out_result = ptr::null_mut() };
        }
        return NuxStatus::ReentrantCall;
    }
    let callbacks = super::NuxRenderCallbacks::default();
    let table = Arc::new(callbacks);
    let callback_factory = PersistentFactory::new(super::CallbackFactory::new(callbacks));
    let factory = PersistentFactory::new(AssetFactory::new(nuxie::render_api::NullFactory::new()));
    unsafe {
        nux_file_import_configured_with_factory(
            bytes,
            len,
            config,
            out_file,
            out_result,
            factory,
            super::RendererDomainBinding::Callbacks {
                table,
                factory: callback_factory,
            },
            super::NativeShaderImportAuthority::Denied,
            None,
        )
    }
}

#[cfg(test)]
pub(crate) unsafe fn nux_file_import_with_assets(
    bytes: *const u8,
    len: usize,
    hooks: *const NuxAssetHooks,
    out_file: *mut *mut NuxFile,
    out_result: *mut *mut NuxCapiResult,
) -> NuxStatus {
    let config = NuxFileImportConfig {
        asset_hooks: hooks,
        ..NuxFileImportConfig::default()
    };
    unsafe { nux_file_import_configured(bytes, len, &config, out_file, out_result) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NUX_CAPI_ABI_VERSION, NuxStatus};
    use std::cell::RefCell;
    use std::ffi::c_void;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Default)]
    struct OwnershipProbe {
        retains: AtomicUsize,
        releases: AtomicUsize,
    }

    unsafe extern "C" fn retain_probe(owner: *mut c_void) {
        let probe = unsafe { &*owner.cast::<OwnershipProbe>() };
        probe.retains.fetch_add(1, Ordering::Relaxed);
    }

    unsafe extern "C" fn release_probe(owner: *mut c_void) {
        let probe = unsafe { &*owner.cast::<OwnershipProbe>() };
        probe.releases.fetch_add(1, Ordering::Relaxed);
    }

    fn retained<'a>(bytes: &'a [u8], probe: &'a OwnershipProbe) -> NuxRetainedBytes {
        NuxRetainedBytes {
            data: bytes.as_ptr(),
            len: bytes.len(),
            owner: std::ptr::from_ref(probe).cast_mut().cast(),
            retain: Some(retain_probe),
            release: Some(release_probe),
            ..NuxRetainedBytes::default()
        }
    }

    #[test]
    fn retained_bytes_balance_success_zero_limit_and_panic_paths() {
        let success_probe = OwnershipProbe::default();
        assert_eq!(
            copy_retained_bytes(retained(b"asset", &success_probe), 5).expect("valid bytes"),
            b"asset"
        );
        assert_eq!(success_probe.retains.load(Ordering::Relaxed), 1);
        assert_eq!(success_probe.releases.load(Ordering::Relaxed), 1);

        let zero_probe = OwnershipProbe::default();
        let mut zero = retained(&[], &zero_probe);
        zero.data = std::ptr::null();
        assert!(
            copy_retained_bytes(zero, 0)
                .expect("empty bytes")
                .is_empty()
        );
        assert_eq!(zero_probe.retains.load(Ordering::Relaxed), 1);
        assert_eq!(zero_probe.releases.load(Ordering::Relaxed), 1);

        let limit_probe = OwnershipProbe::default();
        assert_eq!(
            copy_retained_bytes(retained(b"too large", &limit_probe), 3),
            Err(RetainedBytesError::LimitExceeded)
        );
        assert_eq!(limit_probe.retains.load(Ordering::Relaxed), 1);
        assert_eq!(limit_probe.releases.load(Ordering::Relaxed), 1);

        let null_probe = OwnershipProbe::default();
        let mut null = retained(b"nonnull", &null_probe);
        null.data = std::ptr::null();
        assert_eq!(
            copy_retained_bytes(null, usize::MAX),
            Err(RetainedBytesError::NullData)
        );
        assert_eq!(null_probe.retains.load(Ordering::Relaxed), 1);
        assert_eq!(null_probe.releases.load(Ordering::Relaxed), 1);

        let incomplete_probe = OwnershipProbe::default();
        let mut incomplete = retained(b"incomplete", &incomplete_probe);
        incomplete.release = None;
        assert_eq!(
            copy_retained_bytes(incomplete, usize::MAX),
            Err(RetainedBytesError::InvalidOwnership)
        );
        assert_eq!(incomplete_probe.retains.load(Ordering::Relaxed), 0);
        assert_eq!(incomplete_probe.releases.load(Ordering::Relaxed), 0);

        let short_probe = OwnershipProbe::default();
        let mut short = retained(b"short", &short_probe);
        short.struct_size = 0;
        assert_eq!(
            copy_retained_bytes(short, usize::MAX),
            Err(RetainedBytesError::InvalidStructSize)
        );
        assert_eq!(short_probe.retains.load(Ordering::Relaxed), 0);
        assert_eq!(short_probe.releases.load(Ordering::Relaxed), 0);

        let panic_probe = OwnershipProbe::default();
        let panic = catch_unwind(AssertUnwindSafe(|| {
            let lease = retain_bytes(retained(b"panic", &panic_probe), 5)
                .expect("lease acquired before panic");
            assert_eq!(lease.as_slice(), b"panic");
            panic!("injected after retain");
        }));
        assert!(panic.is_err());
        assert_eq!(panic_probe.retains.load(Ordering::Relaxed), 1);
        assert_eq!(panic_probe.releases.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn short_hook_prefix_at_a_guard_page_is_rejected_without_tail_read() {
        unsafe extern "C" {
            fn getpagesize() -> std::ffi::c_int;
            fn mmap(
                address: *mut c_void,
                length: usize,
                protection: std::ffi::c_int,
                flags: std::ffi::c_int,
                descriptor: std::ffi::c_int,
                offset: i64,
            ) -> *mut c_void;
            fn mprotect(
                address: *mut c_void,
                length: usize,
                protection: std::ffi::c_int,
            ) -> std::ffi::c_int;
            fn munmap(address: *mut c_void, length: usize) -> std::ffi::c_int;
        }
        const PROT_NONE: std::ffi::c_int = 0;
        const PROT_READ_WRITE: std::ffi::c_int = 0x1 | 0x2;
        const MAP_PRIVATE_ANON: std::ffi::c_int = 0x0002 | 0x1000;

        let page_size = usize::try_from(unsafe { getpagesize() }).expect("positive page size");
        let mapping = unsafe {
            mmap(
                ptr::null_mut(),
                page_size * 2,
                PROT_READ_WRITE,
                MAP_PRIVATE_ANON,
                -1,
                0,
            )
        };
        assert_ne!(mapping, usize::MAX as *mut c_void, "mmap succeeds");
        let guard_page = unsafe { mapping.cast::<u8>().add(page_size).cast::<c_void>() };
        assert_eq!(unsafe { mprotect(guard_page, page_size, PROT_NONE) }, 0);
        let hooks = unsafe {
            mapping
                .cast::<u8>()
                .add(page_size - std::mem::size_of::<u32>())
                .cast::<u32>()
        };
        unsafe { hooks.write(std::mem::size_of::<u32>() as u32) };

        let mut file = ptr::dangling_mut();
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_file_import_with_assets(
                    ptr::null(),
                    0,
                    hooks.cast::<NuxAssetHooks>(),
                    &mut file,
                    &mut result,
                )
            },
            NuxStatus::InvalidStructSize
        );
        assert!(file.is_null());
        assert!(!result.is_null());
        assert_eq!(
            unsafe { super::super::nux_capi_result_free(result) },
            NuxStatus::Ok
        );
        assert_eq!(unsafe { munmap(mapping, page_size * 2) }, 0);
    }

    #[test]
    fn decoded_image_contract_copies_tight_premultiplied_srgb_pixels() {
        let encoded = include_bytes!("../tests/fixtures/external-image.png");
        let dimensions = nuxie_image_codec::preflight_encoded_image(encoded)
            .expect("checked-in external image is valid");
        let tight_row = dimensions.width.checked_mul(4).expect("row bytes");
        let padded_row = tight_row.checked_add(8).expect("padded row bytes");
        let pixel_len = usize::try_from(padded_row)
            .expect("row fits usize")
            .checked_mul(usize::try_from(dimensions.height).expect("height fits usize"))
            .expect("pixel allocation fits");
        let pixels = vec![0x7b; pixel_len];
        let probe = OwnershipProbe::default();
        let decoded = NuxDecodedImage {
            width: dimensions.width,
            height: dimensions.height,
            row_bytes: padded_row,
            pixel_format: NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB,
            pixels: retained(&pixels, &probe),
            ..NuxDecodedImage::default()
        };

        let canonical = validate_decoded_image(
            encoded,
            decoded,
            dimensions.width.max(dimensions.height),
            usize::MAX,
        )
        .expect("valid decoded image");
        assert_eq!(
            (canonical.width, canonical.height),
            (dimensions.width, dimensions.height)
        );
        assert_eq!(canonical.row_bytes, tight_row);
        assert_eq!(
            canonical.pixels.len(),
            usize::try_from(tight_row).unwrap() * usize::try_from(dimensions.height).unwrap()
        );
        assert!(canonical.pixels.iter().all(|pixel| *pixel == 0x7b));
        assert_eq!(probe.retains.load(Ordering::Relaxed), 1);
        assert_eq!(probe.releases.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn decoded_image_rejects_dimension_format_row_and_byte_limits_after_balancing_owner() {
        let encoded = include_bytes!("../tests/fixtures/external-image.png");
        let dimensions = nuxie_image_codec::preflight_encoded_image(encoded).expect("dimensions");
        let row = dimensions.width * 4;
        let pixels = vec![0x55; row as usize * dimensions.height as usize];
        for (expected, mutate) in [
            (
                DecodedImageError::DimensionMismatch,
                (
                    dimensions.width + 1,
                    dimensions.height,
                    row,
                    NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB,
                    pixels.len(),
                ),
            ),
            (
                DecodedImageError::InvalidPixelFormat,
                (dimensions.width, dimensions.height, row, 999, pixels.len()),
            ),
            (
                DecodedImageError::InvalidRowBytes,
                (
                    dimensions.width,
                    dimensions.height,
                    row - 1,
                    NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB,
                    pixels.len(),
                ),
            ),
            (
                DecodedImageError::InvalidPixels,
                (
                    dimensions.width,
                    dimensions.height,
                    row,
                    NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB,
                    pixels.len() - 1,
                ),
            ),
        ] {
            let probe = OwnershipProbe::default();
            let decoded = NuxDecodedImage {
                width: mutate.0,
                height: mutate.1,
                row_bytes: mutate.2,
                pixel_format: mutate.3,
                pixels: retained(&pixels[..mutate.4], &probe),
                ..NuxDecodedImage::default()
            };
            assert_eq!(
                validate_decoded_image(encoded, decoded, 8_192, usize::MAX),
                Err(expected)
            );
            assert_eq!(probe.retains.load(Ordering::Relaxed), 1);
            assert_eq!(probe.releases.load(Ordering::Relaxed), 1);
        }

        let probe = OwnershipProbe::default();
        let decoded = NuxDecodedImage {
            width: dimensions.width,
            height: dimensions.height,
            row_bytes: row,
            pixel_format: NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB,
            pixels: retained(&pixels, &probe),
            ..NuxDecodedImage::default()
        };
        assert_eq!(
            validate_decoded_image(encoded, decoded, dimensions.width - 1, usize::MAX),
            Err(DecodedImageError::DimensionLimit)
        );
        assert_eq!(probe.retains.load(Ordering::Relaxed), 1);
        assert_eq!(probe.releases.load(Ordering::Relaxed), 1);

        let probe = OwnershipProbe::default();
        let decoded = NuxDecodedImage {
            width: dimensions.width,
            height: dimensions.height,
            row_bytes: row,
            pixel_format: NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB,
            pixels: retained(&pixels, &probe),
            ..NuxDecodedImage::default()
        };
        assert_eq!(
            validate_decoded_image(encoded, decoded, 8_192, pixels.len() - 1),
            Err(DecodedImageError::LimitExceeded)
        );
        assert_eq!(probe.retains.load(Ordering::Relaxed), 1);
        assert_eq!(probe.releases.load(Ordering::Relaxed), 1);
    }

    struct DecodeProbe {
        ownership: OwnershipProbe,
        calls: usize,
        nested_status: NuxStatus,
        maximum_decoded_bytes: usize,
        pixels: RefCell<Vec<u8>>,
    }

    unsafe extern "C" fn decode_fixture_image(
        context: *mut c_void,
        request: *const NuxImageDecodeRequest,
        out_image: *mut NuxDecodedImage,
    ) -> NuxAssetCallbackStatus {
        let probe = unsafe { &mut *context.cast::<DecodeProbe>() };
        let request = unsafe { &*request };
        probe.calls += 1;
        probe.nested_status = unsafe { super::super::nux_capi_require_abi(NUX_CAPI_ABI_VERSION) };
        probe.maximum_decoded_bytes = request.maximum_decoded_bytes;
        let encoded =
            unsafe { std::slice::from_raw_parts(request.encoded.data, request.encoded.len) };
        let decoded = nuxie_image_codec::decode_image_rgba(encoded).expect("fixture decodes");
        let mut pixels = probe.pixels.borrow_mut();
        *pixels = decoded.pixels;
        unsafe {
            *out_image = NuxDecodedImage {
                width: decoded.width,
                height: decoded.height,
                row_bytes: decoded.width * 4,
                pixel_format: NUX_PIXEL_FORMAT_RGBA8_PREMULTIPLIED_SRGB,
                pixels: retained(&pixels, &probe.ownership),
                ..NuxDecodedImage::default()
            };
        }
        NUX_ASSET_CALLBACK_STATUS_OK
    }

    unsafe extern "C" fn decode_fixture_then_fail(
        context: *mut c_void,
        request: *const NuxImageDecodeRequest,
        out_image: *mut NuxDecodedImage,
    ) -> NuxAssetCallbackStatus {
        let _ = unsafe { decode_fixture_image(context, request, out_image) };
        NUX_ASSET_CALLBACK_STATUS_FAILED
    }

    #[test]
    fn configured_import_composes_trust_assets_and_exact_catalog_before_callbacks() {
        let root = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        );
        let bytes = std::fs::read(root.join("tests/unit_tests/assets/in_band_asset.riv"))
            .expect("read in-band image fixture");
        let mut probe = DecodeProbe {
            ownership: OwnershipProbe::default(),
            calls: 0,
            nested_status: NuxStatus::Ok,
            maximum_decoded_bytes: 0,
            pixels: RefCell::new(Vec::new()),
        };
        let hooks = NuxAssetHooks {
            context: std::ptr::from_mut(&mut probe).cast(),
            decode_image: Some(decode_fixture_image),
            ..NuxAssetHooks::default()
        };
        let module = b"bridge";
        let host = super::super::NuxHostCommandImportConfig {
            module_name: super::super::NuxStringView {
                data: module.as_ptr().cast(),
                len: module.len(),
            },
            ..super::super::NuxHostCommandImportConfig::default()
        };
        let name = b"1x1.png";
        let extension = b"png";
        let expected = super::super::NuxExpectedFileAssetDescriptor {
            ordinal: 0,
            kind: super::super::NUX_FILE_ASSET_KIND_IMAGE,
            has_authored_id: 1,
            authored_id: 45_023,
            name: super::super::NuxStringView {
                data: name.as_ptr().cast(),
                len: name.len(),
            },
            file_extension: super::super::NuxStringView {
                data: extension.as_ptr().cast(),
                len: extension.len(),
            },
            is_embedded: 1,
            has_contents_record: 1,
            required_provider_flags: super::super::NUX_FILE_ASSET_PROVIDER_IMAGE_DECODE,
            ..super::super::NuxExpectedFileAssetDescriptor::default()
        };
        let config = NuxFileImportConfig {
            host_commands: &host,
            asset_hooks: &hooks,
            expected_assets: &expected,
            expected_asset_count: 1,
            ..NuxFileImportConfig::default()
        };
        let mut file = ptr::dangling_mut();
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_file_import_configured(
                    bytes.as_ptr(),
                    bytes.len(),
                    &config,
                    &mut file,
                    &mut result,
                )
            },
            NuxStatus::HandleMismatch
        );
        assert!(file.is_null());
        assert_eq!(
            probe.calls, 0,
            "catalog mismatch precedes every provider callback"
        );
        assert_eq!(
            unsafe { super::super::nux_capi_result_free(result) },
            NuxStatus::Ok
        );

        let expected = super::super::NuxExpectedFileAssetDescriptor {
            authored_id: 45_022,
            ..expected
        };
        let config = NuxFileImportConfig {
            expected_assets: &expected,
            ..config
        };
        result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_file_import_configured(
                    bytes.as_ptr(),
                    bytes.len(),
                    &config,
                    &mut file,
                    &mut result,
                )
            },
            NuxStatus::Ok
        );
        assert_eq!(probe.calls, 1);
        assert_eq!(probe.nested_status, NuxStatus::ReentrantCall);
        assert_eq!(probe.ownership.retains.load(Ordering::Relaxed), 1);
        assert_eq!(probe.ownership.releases.load(Ordering::Relaxed), 1);
        assert_eq!(
            unsafe { super::super::nux_capi_result_free(result) },
            NuxStatus::Ok
        );
        assert_eq!(unsafe { super::super::nux_file_free(file) }, NuxStatus::Ok);
    }

    #[test]
    fn configured_import_rejects_short_prefix_and_aliased_outputs_before_import() {
        let config = NuxFileImportConfig {
            struct_size: u32::try_from(NUX_FILE_IMPORT_CONFIG_V3_MIN_SIZE - 1)
                .expect("minimum fits u32"),
            ..NuxFileImportConfig::default()
        };
        let mut file = ptr::dangling_mut();
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_file_import_configured(ptr::null(), 0, &config, &mut file, &mut result) },
            NuxStatus::InvalidStructSize
        );
        assert!(file.is_null());
        assert_eq!(
            unsafe { super::super::nux_capi_result_free(result) },
            NuxStatus::Ok
        );

        let config = NuxFileImportConfig::default();
        let mut shared = ptr::dangling_mut::<NuxFile>();
        let shared_slot = std::ptr::from_mut(&mut shared);
        assert_eq!(
            unsafe {
                nux_file_import_configured(ptr::null(), 0, &config, shared_slot, shared_slot.cast())
            },
            NuxStatus::InvalidArgument
        );
        assert!(
            shared.is_null(),
            "aliased publication storage is cleared once"
        );
    }

    #[test]
    fn configured_import_validates_every_nested_policy_before_file_or_provider_work() {
        let invalid_bytes = b"not a rive file";
        let mut probe = DecodeProbe {
            ownership: OwnershipProbe::default(),
            calls: 0,
            nested_status: NuxStatus::Ok,
            maximum_decoded_bytes: 0,
            pixels: RefCell::new(Vec::new()),
        };
        let invalid_hooks = NuxAssetHooks {
            context: std::ptr::from_mut(&mut probe).cast(),
            decode_image: Some(decode_fixture_image),
            maximum_decoded_image_bytes: 0,
            ..NuxAssetHooks::default()
        };
        let config = NuxFileImportConfig {
            asset_hooks: &invalid_hooks,
            ..NuxFileImportConfig::default()
        };
        let mut file = ptr::dangling_mut();
        let mut result = ptr::null_mut();
        super::super::test_reset_file_import_calls();
        assert_eq!(
            unsafe {
                nux_file_import_configured(
                    invalid_bytes.as_ptr(),
                    invalid_bytes.len(),
                    &config,
                    &mut file,
                    &mut result,
                )
            },
            NuxStatus::InvalidStructSize,
            "invalid nested hook policy wins before invalid file parsing"
        );
        assert!(file.is_null());
        assert_eq!(super::super::test_file_import_calls(), 0);
        assert_eq!(probe.calls, 0);
        assert_eq!(
            unsafe { super::super::nux_capi_result_free(result) },
            NuxStatus::Ok
        );

        let invalid_host = super::super::NuxHostCommandImportConfig::default();
        let config = NuxFileImportConfig {
            host_commands: &invalid_host,
            asset_hooks: &invalid_hooks,
            ..NuxFileImportConfig::default()
        };
        file = ptr::dangling_mut();
        result = ptr::null_mut();
        super::super::test_reset_file_import_calls();
        assert_eq!(
            unsafe {
                nux_file_import_configured(
                    invalid_bytes.as_ptr(),
                    invalid_bytes.len(),
                    &config,
                    &mut file,
                    &mut result,
                )
            },
            NuxStatus::InvalidArgument,
            "the first invalid child policy wins without touching the file or later providers"
        );
        assert!(file.is_null());
        assert_eq!(super::super::test_file_import_calls(), 0);
        assert_eq!(probe.calls, 0);
        assert_eq!(
            unsafe { super::super::nux_capi_result_free(result) },
            NuxStatus::Ok
        );
    }

    #[test]
    fn asset_hook_import_decodes_embedded_image_and_owns_pixels() {
        let root = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        );
        let bytes = std::fs::read(root.join("tests/unit_tests/assets/in_band_asset.riv"))
            .expect("read in-band image fixture");
        let mut probe = DecodeProbe {
            ownership: OwnershipProbe::default(),
            calls: 0,
            nested_status: NuxStatus::Ok,
            maximum_decoded_bytes: 0,
            pixels: RefCell::new(Vec::new()),
        };
        let hooks = NuxAssetHooks {
            context: std::ptr::from_mut(&mut probe).cast(),
            decode_image: Some(decode_fixture_image),
            ..NuxAssetHooks::default()
        };
        let mut file = std::ptr::null_mut();
        let mut result = std::ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_file_import_with_assets(
                    bytes.as_ptr(),
                    bytes.len(),
                    &hooks,
                    &mut file,
                    &mut result,
                )
            },
            NuxStatus::Ok
        );
        assert_eq!(probe.calls, 1);
        assert_eq!(probe.nested_status, NuxStatus::ReentrantCall);
        assert_eq!(probe.ownership.retains.load(Ordering::Relaxed), 1);
        assert_eq!(probe.ownership.releases.load(Ordering::Relaxed), 1);
        let canonical_before = unsafe { file.as_ref() }
            .and_then(|file| file.asset_hooks.as_ref())
            .and_then(|assets| assets.decoded_images.values().next())
            .map(|image| Arc::clone(&image.pixels))
            .expect("canonical pixels");
        probe.pixels.borrow_mut().fill(0);
        let assets = unsafe { file.as_ref() }
            .and_then(|file| file.asset_hooks.as_ref())
            .expect("file retains canonical assets");
        assert_eq!(assets.decoded_images.len(), 1);
        assert_eq!(
            assets
                .decoded_images
                .values()
                .next()
                .map(|image| image.pixels.as_ref()),
            Some(canonical_before.as_ref())
        );
        assert_eq!(
            unsafe { super::super::nux_capi_result_free(result) },
            NuxStatus::Ok
        );
        assert_eq!(unsafe { super::super::nux_file_free(file) }, NuxStatus::Ok);
    }

    #[test]
    fn image_preflight_rejects_one_over_aggregate_before_callback_and_passes_remaining_budget() {
        let root = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        );
        let bytes = std::fs::read(root.join("tests/unit_tests/assets/in_band_asset.riv"))
            .expect("asset fixture");
        let imported = nuxie_binary::read_runtime_file_with_scripting(&bytes)
            .expect("fixture metadata imports");
        let encoded = imported
            .scripting_file_assets_with_contents()
            .into_iter()
            .find(|asset| asset.asset.type_name == "ImageAsset")
            .and_then(|asset| asset.contents)
            .expect("fixture image bytes");
        let dimensions = nuxie_image_codec::preflight_encoded_image(encoded).expect("dimensions");
        let packed_len = usize::try_from(dimensions.width)
            .expect("width")
            .checked_mul(4)
            .and_then(|row| {
                usize::try_from(dimensions.height)
                    .ok()
                    .and_then(|height| row.checked_mul(height))
            })
            .expect("packed length");

        let mut rejected = DecodeProbe {
            ownership: OwnershipProbe::default(),
            calls: 0,
            nested_status: NuxStatus::Ok,
            maximum_decoded_bytes: 0,
            pixels: RefCell::new(Vec::new()),
        };
        let rejected_hooks = NuxAssetHooks {
            context: std::ptr::from_mut(&mut rejected).cast(),
            decode_image: Some(decode_fixture_image),
            maximum_decoded_image_bytes: packed_len,
            maximum_total_decoded_image_bytes: packed_len - 1,
            ..NuxAssetHooks::default()
        };
        let mut file = ptr::dangling_mut();
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_file_import_with_assets(
                    bytes.as_ptr(),
                    bytes.len(),
                    &rejected_hooks,
                    &mut file,
                    &mut result,
                )
            },
            NuxStatus::LimitExceeded
        );
        assert!(file.is_null());
        assert_eq!(rejected.calls, 0, "preflight runs before host decode");
        assert_eq!(rejected.ownership.retains.load(Ordering::Relaxed), 0);
        assert_eq!(rejected.ownership.releases.load(Ordering::Relaxed), 0);
        assert_eq!(
            unsafe { super::super::nux_capi_result_free(result) },
            NuxStatus::Ok
        );

        let mut accepted = DecodeProbe {
            ownership: OwnershipProbe::default(),
            calls: 0,
            nested_status: NuxStatus::Ok,
            maximum_decoded_bytes: 0,
            pixels: RefCell::new(Vec::new()),
        };
        let accepted_hooks = NuxAssetHooks {
            context: std::ptr::from_mut(&mut accepted).cast(),
            decode_image: Some(decode_fixture_image),
            maximum_decoded_image_bytes: packed_len,
            maximum_total_decoded_image_bytes: packed_len,
            ..NuxAssetHooks::default()
        };
        let mut file = ptr::null_mut();
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_file_import_with_assets(
                    bytes.as_ptr(),
                    bytes.len(),
                    &accepted_hooks,
                    &mut file,
                    &mut result,
                )
            },
            NuxStatus::Ok
        );
        assert_eq!(accepted.calls, 1);
        assert_eq!(accepted.maximum_decoded_bytes, packed_len);
        assert_eq!(accepted.ownership.retains.load(Ordering::Relaxed), 1);
        assert_eq!(accepted.ownership.releases.load(Ordering::Relaxed), 1);
        assert_eq!(
            unsafe { super::super::nux_capi_result_free(result) },
            NuxStatus::Ok
        );
        assert_eq!(unsafe { super::super::nux_file_free(file) }, NuxStatus::Ok);
    }

    #[test]
    fn callback_failure_and_result_publication_panic_balance_pixels_and_publish_no_file() {
        let root = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        );
        let bytes = std::fs::read(root.join("tests/unit_tests/assets/in_band_asset.riv"))
            .expect("asset fixture");
        for (callback, expected) in [
            (
                decode_fixture_then_fail as NuxDecodeImageCallback,
                NuxStatus::ImportError,
            ),
            (
                decode_fixture_image as NuxDecodeImageCallback,
                NuxStatus::RuntimeError,
            ),
        ] {
            let mut probe = DecodeProbe {
                ownership: OwnershipProbe::default(),
                calls: 0,
                nested_status: NuxStatus::Ok,
                maximum_decoded_bytes: 0,
                pixels: RefCell::new(Vec::new()),
            };
            let hooks = NuxAssetHooks {
                context: std::ptr::from_mut(&mut probe).cast(),
                decode_image: Some(callback),
                ..NuxAssetHooks::default()
            };
            if expected == NuxStatus::RuntimeError {
                super::super::panic_after_next_result_publication();
            }
            let mut file = ptr::dangling_mut();
            let mut result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_file_import_with_assets(
                        bytes.as_ptr(),
                        bytes.len(),
                        &hooks,
                        &mut file,
                        &mut result,
                    )
                },
                expected
            );
            assert!(file.is_null(), "failure cannot publish a file handle");
            assert_eq!(probe.ownership.retains.load(Ordering::Relaxed), 1);
            assert_eq!(probe.ownership.releases.load(Ordering::Relaxed), 1);
            assert!(!result.is_null());
            assert_eq!(
                unsafe { super::super::nux_capi_result_free(result) },
                NuxStatus::Ok
            );
        }
    }

    struct LookupProbe {
        ownership: OwnershipProbe,
        bytes: Vec<u8>,
        expected_kind: NuxAssetKind,
        calls: usize,
        nested_status: NuxStatus,
    }

    unsafe extern "C" fn lookup_fixture_asset(
        context: *mut c_void,
        request: *const NuxExternalAssetRequest,
        out_bytes: *mut NuxRetainedBytes,
    ) -> NuxAssetCallbackStatus {
        let probe = unsafe { &mut *context.cast::<LookupProbe>() };
        let request = unsafe { &*request };
        assert!(struct_size_supports(
            request.struct_size,
            NUX_EXTERNAL_ASSET_REQUEST_V3_MIN_SIZE
        ));
        assert_eq!(request.kind, probe.expected_kind);
        probe.calls += 1;
        probe.nested_status = unsafe { super::super::nux_file_free(usize::MAX as *mut NuxFile) };
        unsafe { *out_bytes = retained(&probe.bytes, &probe.ownership) };
        NUX_ASSET_CALLBACK_STATUS_OK
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
        let definition = nuxie_schema::definition_by_name(type_name).expect("fixture type");
        definition
            .properties
            .iter()
            .chain(definition.ancestors.iter().flat_map(|ancestor| {
                nuxie_schema::definition_by_name(ancestor)
                    .expect("fixture ancestor")
                    .properties
                    .iter()
            }))
            .find(|property| property.name == property_name)
            .expect("fixture property")
            .key
            .int
    }

    fn external_asset_file(type_name: &str, asset_id: u32) -> Vec<u8> {
        fn push_object(
            bytes: &mut Vec<u8>,
            type_name: &str,
            properties: impl FnOnce(&mut Vec<u8>),
        ) {
            push_var_uint(
                bytes,
                u64::from(
                    nuxie_schema::definition_by_name(type_name)
                        .expect("fixture type")
                        .type_key
                        .int,
                ),
            );
            properties(bytes);
            push_var_uint(bytes, 0);
        }
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 1_824);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, type_name, |bytes| {
            push_var_uint(bytes, u64::from(property_key(type_name, "assetId")));
            push_var_uint(bytes, u64::from(asset_id));
        });
        bytes
    }

    #[test]
    fn generic_external_image_font_and_audio_bytes_balance_every_import_cycle() {
        let root = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets");
        let png = include_bytes!("../tests/fixtures/external-image.png").to_vec();
        let font = std::fs::read(root.join("fonts/Inter_18pt-Regular.ttf")).expect("font fixture");
        let sound_bytes = std::fs::read(root.join("sound.riv")).expect("sound fixture");
        let sound = nuxie_binary::read_runtime_file_with_scripting(&sound_bytes)
            .expect("sound metadata imports");
        let audio = sound
            .scripting_file_assets_with_contents()
            .into_iter()
            .find(|asset| asset.asset.type_name == "AudioAsset")
            .and_then(|asset| asset.contents.map(<[u8]>::to_vec))
            .expect("embedded audio bytes");

        for (type_name, kind, payload) in [
            ("ImageAsset", NUX_ASSET_KIND_IMAGE, png),
            ("FontAsset", NUX_ASSET_KIND_FONT, font),
            ("AudioAsset", NUX_ASSET_KIND_AUDIO, audio),
        ] {
            let rive = external_asset_file(type_name, 7);
            let mut probe = LookupProbe {
                ownership: OwnershipProbe::default(),
                bytes: payload,
                expected_kind: kind,
                calls: 0,
                nested_status: NuxStatus::Ok,
            };
            let rejected_hooks = NuxAssetHooks {
                context: std::ptr::from_mut(&mut probe).cast(),
                lookup_external_asset: Some(lookup_fixture_asset),
                maximum_external_asset_bytes: probe.bytes.len(),
                maximum_total_external_asset_bytes: probe.bytes.len() - 1,
                ..NuxAssetHooks::default()
            };
            let mut file = ptr::dangling_mut();
            let mut result = ptr::null_mut();
            assert_eq!(
                unsafe {
                    nux_file_import_with_assets(
                        rive.as_ptr(),
                        rive.len(),
                        &rejected_hooks,
                        &mut file,
                        &mut result,
                    )
                },
                NuxStatus::LimitExceeded,
                "{type_name} aggregate one-over"
            );
            assert!(file.is_null());
            assert_eq!(probe.calls, 1);
            assert_eq!(probe.ownership.retains.load(Ordering::Relaxed), 1);
            assert_eq!(probe.ownership.releases.load(Ordering::Relaxed), 1);
            assert_eq!(
                unsafe { super::super::nux_capi_result_free(result) },
                NuxStatus::Ok
            );

            for _ in 0..3 {
                let hooks = NuxAssetHooks {
                    context: std::ptr::from_mut(&mut probe).cast(),
                    lookup_external_asset: Some(lookup_fixture_asset),
                    ..NuxAssetHooks::default()
                };
                let mut file = ptr::null_mut();
                let mut result = ptr::null_mut();
                assert_eq!(
                    unsafe {
                        nux_file_import_with_assets(
                            rive.as_ptr(),
                            rive.len(),
                            &hooks,
                            &mut file,
                            &mut result,
                        )
                    },
                    NuxStatus::Ok,
                    "{type_name} import"
                );
                assert_eq!(
                    unsafe { super::super::nux_capi_result_free(result) },
                    NuxStatus::Ok
                );
                assert_eq!(unsafe { super::super::nux_file_free(file) }, NuxStatus::Ok);
            }
            assert_eq!(probe.calls, 4, "{type_name} lookup count");
            assert_eq!(probe.nested_status, NuxStatus::ReentrantCall);
            assert_eq!(probe.ownership.retains.load(Ordering::Relaxed), 4);
            assert_eq!(probe.ownership.releases.load(Ordering::Relaxed), 4);
        }
    }
}
