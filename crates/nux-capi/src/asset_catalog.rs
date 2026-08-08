//! Portable, read-only metadata for the exact file-asset catalog.

use super::*;
use nuxie::{FileAsset, FileAssetKind};

pub const NUX_FILE_ASSET_PROVIDER_EXTERNAL_BYTES: u32 = 1 << 0;
pub const NUX_FILE_ASSET_PROVIDER_IMAGE_DECODE: u32 = 1 << 1;
pub const NUX_FILE_ASSET_CATALOG_HARD_MAX: usize = 4_096;
const EXPECTED_ASSET_TEXT_BYTES_HARD_MAX: usize = 4 * 1024 * 1024;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuxFileAssetKind {
    Image = 0,
    Font = 1,
    Audio = 2,
    Blob = 3,
    Script = 4,
    Shader = 5,
}

pub const NUX_FILE_ASSET_KIND_IMAGE: u32 = NuxFileAssetKind::Image as u32;
pub const NUX_FILE_ASSET_KIND_FONT: u32 = NuxFileAssetKind::Font as u32;
pub const NUX_FILE_ASSET_KIND_AUDIO: u32 = NuxFileAssetKind::Audio as u32;
pub const NUX_FILE_ASSET_KIND_BLOB: u32 = NuxFileAssetKind::Blob as u32;
pub const NUX_FILE_ASSET_KIND_SCRIPT: u32 = NuxFileAssetKind::Script as u32;
pub const NUX_FILE_ASSET_KIND_SHADER: u32 = NuxFileAssetKind::Shader as u32;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFileAssetDescriptorView {
    pub struct_size: u32,
    pub ordinal: usize,
    pub kind: u32,
    pub has_authored_id: u32,
    pub authored_id: u32,
    pub name: NuxStringView,
    pub file_extension: NuxStringView,
    pub is_embedded: u32,
    pub has_contents_record: u32,
    pub required_provider_flags: u32,
}

impl Default for NuxFileAssetDescriptorView {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            ordinal: 0,
            kind: NUX_FILE_ASSET_KIND_BLOB,
            has_authored_id: 0,
            authored_id: 0,
            name: NuxStringView::default(),
            file_extension: NuxStringView::default(),
            is_embedded: 0,
            has_contents_record: 0,
            required_provider_flags: 0,
        }
    }
}

pub const NUX_FILE_ASSET_DESCRIPTOR_VIEW_V3_MIN_SIZE: usize =
    std::mem::offset_of!(NuxFileAssetDescriptorView, required_provider_flags)
        + std::mem::size_of::<u32>();

/// Exact descriptor expected by a configured import. This is a fixed-stride
/// array element: `struct_size` must equal `sizeof(NuxExpectedFileAssetDescriptor)`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxExpectedFileAssetDescriptor {
    pub struct_size: u32,
    pub ordinal: usize,
    pub kind: u32,
    pub has_authored_id: u32,
    pub authored_id: u32,
    pub name: NuxStringView,
    pub file_extension: NuxStringView,
    pub is_embedded: u32,
    pub has_contents_record: u32,
    pub required_provider_flags: u32,
}

impl Default for NuxExpectedFileAssetDescriptor {
    fn default() -> Self {
        Self {
            struct_size: u32::try_from(std::mem::size_of::<Self>()).unwrap_or(u32::MAX),
            ordinal: 0,
            kind: NUX_FILE_ASSET_KIND_BLOB,
            has_authored_id: 0,
            authored_id: 0,
            name: NuxStringView::default(),
            file_extension: NuxStringView::default(),
            is_embedded: 0,
            has_contents_record: 0,
            required_provider_flags: 0,
        }
    }
}

pub const NUX_EXPECTED_FILE_ASSET_DESCRIPTOR_V3_SIZE: usize =
    std::mem::size_of::<NuxExpectedFileAssetDescriptor>();

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpectedFileAssetDescriptor {
    ordinal: usize,
    kind: u32,
    authored_id: Option<u32>,
    name: Vec<u8>,
    file_extension: Vec<u8>,
    is_embedded: bool,
    has_contents_record: bool,
    required_provider_flags: u32,
}

fn kind(kind: FileAssetKind) -> u32 {
    match kind {
        FileAssetKind::Image => NUX_FILE_ASSET_KIND_IMAGE,
        FileAssetKind::Font => NUX_FILE_ASSET_KIND_FONT,
        FileAssetKind::Audio => NUX_FILE_ASSET_KIND_AUDIO,
        FileAssetKind::Blob => NUX_FILE_ASSET_KIND_BLOB,
        FileAssetKind::Script => NUX_FILE_ASSET_KIND_SCRIPT,
        FileAssetKind::Shader => NUX_FILE_ASSET_KIND_SHADER,
    }
}

fn required_provider_flags(asset: FileAsset<'_>) -> u32 {
    let external = asset.contents().is_none();
    match asset.kind() {
        FileAssetKind::Image => {
            NUX_FILE_ASSET_PROVIDER_IMAGE_DECODE
                | if external {
                    NUX_FILE_ASSET_PROVIDER_EXTERNAL_BYTES
                } else {
                    0
                }
        }
        FileAssetKind::Font | FileAssetKind::Audio if external => {
            NUX_FILE_ASSET_PROVIDER_EXTERNAL_BYTES
        }
        FileAssetKind::Font
        | FileAssetKind::Audio
        | FileAssetKind::Blob
        | FileAssetKind::Script
        | FileAssetKind::Shader => 0,
    }
}

fn borrowed_string(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

fn descriptor_view(asset: FileAsset<'_>) -> NuxFileAssetDescriptorView {
    let authored_id = asset.asset_id();
    NuxFileAssetDescriptorView {
        ordinal: asset.index(),
        kind: kind(asset.kind()),
        has_authored_id: u32::from(authored_id.is_some()),
        authored_id: authored_id.unwrap_or(0),
        name: borrowed_string(asset.name().unwrap_or_default()),
        file_extension: borrowed_string(asset.file_extension()),
        is_embedded: u32::from(asset.contents().is_some()),
        has_contents_record: u32::from(asset.has_contents_record()),
        required_provider_flags: required_provider_flags(asset),
        ..NuxFileAssetDescriptorView::default()
    }
}

pub(crate) unsafe fn copy_expected_descriptors(
    descriptors: *const NuxExpectedFileAssetDescriptor,
    count: usize,
) -> Result<Vec<ExpectedFileAssetDescriptor>, (NuxStatus, &'static str)> {
    if count > NUX_FILE_ASSET_CATALOG_HARD_MAX {
        return Err((
            NuxStatus::LimitExceeded,
            "expected asset count exceeds its hard bound",
        ));
    }
    if descriptors.is_null() && count != 0 {
        return Err((
            NuxStatus::NullArgument,
            "nonempty expected asset array is null",
        ));
    }
    let input = if count == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(descriptors, count) }
    };
    let mut total_text = 0usize;
    let mut copied = Vec::with_capacity(input.len());
    for descriptor in input {
        if usize::try_from(descriptor.struct_size).ok()
            != Some(NUX_EXPECTED_FILE_ASSET_DESCRIPTOR_V3_SIZE)
        {
            return Err((
                NuxStatus::InvalidStructSize,
                "expected asset descriptor stride is invalid",
            ));
        }
        if descriptor.ordinal != copied.len()
            || descriptor.kind > NUX_FILE_ASSET_KIND_SHADER
            || descriptor.has_authored_id > 1
            || descriptor.is_embedded > 1
            || descriptor.has_contents_record > 1
            || descriptor.required_provider_flags
                & !(NUX_FILE_ASSET_PROVIDER_EXTERNAL_BYTES | NUX_FILE_ASSET_PROVIDER_IMAGE_DECODE)
                != 0
        {
            return Err((
                NuxStatus::InvalidArgument,
                "expected asset descriptor is invalid",
            ));
        }
        total_text = total_text
            .checked_add(descriptor.name.len)
            .and_then(|value| value.checked_add(descriptor.file_extension.len))
            .filter(|value| *value <= EXPECTED_ASSET_TEXT_BYTES_HARD_MAX)
            .ok_or((
                NuxStatus::LimitExceeded,
                "expected asset text exceeds its hard bound",
            ))?;
        let name = with_utf8_view(descriptor.name, |value| value.as_bytes().to_vec())
            .map_err(|status| (status, "expected asset name is invalid"))?;
        let file_extension =
            with_utf8_view(descriptor.file_extension, |value| value.as_bytes().to_vec())
                .map_err(|status| (status, "expected asset extension is invalid"))?;
        copied.push(ExpectedFileAssetDescriptor {
            ordinal: descriptor.ordinal,
            kind: descriptor.kind,
            authored_id: (descriptor.has_authored_id == 1).then_some(descriptor.authored_id),
            name,
            file_extension,
            is_embedded: descriptor.is_embedded == 1,
            has_contents_record: descriptor.has_contents_record == 1,
            required_provider_flags: descriptor.required_provider_flags,
        });
    }
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_catalog_array_is_exact_stride_validated_and_bounded_before_copy() {
        assert_eq!(
            unsafe { copy_expected_descriptors(std::ptr::null(), 0) },
            Ok(Vec::new())
        );
        assert_eq!(
            unsafe { copy_expected_descriptors(std::ptr::null(), 1) }
                .expect_err("nonnull array required")
                .0,
            NuxStatus::NullArgument
        );
        assert_eq!(
            unsafe {
                copy_expected_descriptors(std::ptr::dangling(), NUX_FILE_ASSET_CATALOG_HARD_MAX + 1)
            }
            .expect_err("count is bounded before pointer access")
            .0,
            NuxStatus::LimitExceeded
        );

        let mut descriptor = NuxExpectedFileAssetDescriptor::default();
        descriptor.struct_size -= 1;
        assert_eq!(
            unsafe { copy_expected_descriptors(&descriptor, 1) }
                .expect_err("array stride is exact")
                .0,
            NuxStatus::InvalidStructSize
        );
        descriptor = NuxExpectedFileAssetDescriptor {
            required_provider_flags: u32::MAX,
            ..NuxExpectedFileAssetDescriptor::default()
        };
        assert_eq!(
            unsafe { copy_expected_descriptors(&descriptor, 1) }
                .expect_err("unknown flags fail closed")
                .0,
            NuxStatus::InvalidArgument
        );

        descriptor = NuxExpectedFileAssetDescriptor {
            name: NuxStringView {
                data: std::ptr::dangling(),
                len: EXPECTED_ASSET_TEXT_BYTES_HARD_MAX + 1,
            },
            ..NuxExpectedFileAssetDescriptor::default()
        };
        assert_eq!(
            unsafe { copy_expected_descriptors(&descriptor, 1) }
                .expect_err("oversize text rejects before borrowed pointer access")
                .0,
            NuxStatus::LimitExceeded
        );

        let invalid_utf8 = [0xff];
        descriptor = NuxExpectedFileAssetDescriptor {
            name: NuxStringView {
                data: invalid_utf8.as_ptr().cast(),
                len: invalid_utf8.len(),
            },
            ..NuxExpectedFileAssetDescriptor::default()
        };
        assert_eq!(
            unsafe { copy_expected_descriptors(&descriptor, 1) }
                .expect_err("catalog text must be UTF-8")
                .0,
            NuxStatus::InvalidArgument
        );
    }
}

pub(crate) fn validate_expected_descriptors(
    file: &File,
    expected: &[ExpectedFileAssetDescriptor],
) -> Result<(), &'static str> {
    if file.asset_count() != expected.len() {
        return Err("file asset count does not match the expected catalog");
    }
    for (asset, expected) in file.assets().zip(expected) {
        let actual = ExpectedFileAssetDescriptor {
            ordinal: asset.index(),
            kind: kind(asset.kind()),
            authored_id: asset.asset_id(),
            name: asset.name().unwrap_or_default().as_bytes().to_vec(),
            file_extension: asset.file_extension().as_bytes().to_vec(),
            is_embedded: asset.contents().is_some(),
            has_contents_record: asset.has_contents_record(),
            required_provider_flags: required_provider_flags(asset),
        };
        if actual != *expected {
            return Err("file asset descriptor does not match the expected catalog");
        }
    }
    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_asset_count(
    file: *const NuxFile,
    out_count: *mut usize,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_count.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_count = 0 };
        let _call = match enter_handle(file, HandleKind::File) {
            Ok(call) => call,
            Err(status) => return status,
        };
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        unsafe { *out_count = file.file.asset_count() };
        NuxStatus::Ok
    })
}

/// Returns metadata borrowed from `file`; string views remain valid until the
/// file handle is freed. Callers must copy them before another runtime call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_asset_descriptor(
    file: *const NuxFile,
    index: usize,
    out_descriptor: *mut NuxFileAssetDescriptorView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = match enter_handle(file, HandleKind::File) {
            Ok(call) => call,
            Err(status) => return status,
        };
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        let Some(asset) = file.file.asset(index) else {
            return NuxStatus::NotFound;
        };
        let view = descriptor_view(asset);
        unsafe {
            write_caller_struct(
                out_descriptor,
                &view,
                NUX_FILE_ASSET_DESCRIPTOR_VIEW_V3_MIN_SIZE,
            )
        }
        .map_or_else(|status| status, |()| NuxStatus::Ok)
    })
}
