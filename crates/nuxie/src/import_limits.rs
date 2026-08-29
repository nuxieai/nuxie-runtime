use anyhow::{Context, Result, bail, ensure};
use std::cell::{Cell, RefCell};

/// Optional host admission limits around the native source importer.
/// Serialized allocation budgets are checked before import; actual imported
/// assets are checked before their loader or decoder and before scripts run.
///
/// [`Self::new`] and [`Default::default`] are deliberately bounded. Hosts that
/// accept larger trusted artifacts can raise individual ceilings explicitly;
/// [`Self::unbounded`] is reserved for already-authenticated, host-controlled
/// inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileImportLimits {
    max_input_bytes: Option<usize>,
    max_runtime_objects: Option<usize>,
    max_runtime_properties: Option<usize>,
    max_imported_file_assets: Option<usize>,
    max_file_asset_content_bytes: Option<usize>,
    max_total_file_asset_content_bytes: Option<usize>,
    max_retained_decoded_image_bytes: Option<usize>,
}

impl FileImportLimits {
    const DEFAULT_MAX_INPUT_BYTES: usize = 128 * 1024 * 1024;
    const DEFAULT_MAX_RUNTIME_OBJECTS: usize = 1_000_000;
    const DEFAULT_MAX_RUNTIME_PROPERTIES: usize = 1_000_000;
    const DEFAULT_MAX_IMPORTED_FILE_ASSETS: usize = 16_384;
    const DEFAULT_MAX_FILE_ASSET_CONTENT_BYTES: usize = 64 * 1024 * 1024;
    const DEFAULT_MAX_TOTAL_FILE_ASSET_CONTENT_BYTES: usize = 128 * 1024 * 1024;
    const DEFAULT_MAX_RETAINED_DECODED_IMAGE_BYTES: usize = 64 * 1024 * 1024;

    pub const fn new() -> Self {
        Self {
            max_input_bytes: Some(Self::DEFAULT_MAX_INPUT_BYTES),
            max_runtime_objects: Some(Self::DEFAULT_MAX_RUNTIME_OBJECTS),
            max_runtime_properties: Some(Self::DEFAULT_MAX_RUNTIME_PROPERTIES),
            max_imported_file_assets: Some(Self::DEFAULT_MAX_IMPORTED_FILE_ASSETS),
            max_file_asset_content_bytes: Some(Self::DEFAULT_MAX_FILE_ASSET_CONTENT_BYTES),
            max_total_file_asset_content_bytes: Some(
                Self::DEFAULT_MAX_TOTAL_FILE_ASSET_CONTENT_BYTES,
            ),
            max_retained_decoded_image_bytes: Some(Self::DEFAULT_MAX_RETAINED_DECODED_IMAGE_BYTES),
        }
    }

    pub const fn unbounded() -> Self {
        Self {
            max_input_bytes: None,
            max_runtime_objects: None,
            max_runtime_properties: None,
            max_imported_file_assets: None,
            max_file_asset_content_bytes: None,
            max_total_file_asset_content_bytes: None,
            max_retained_decoded_image_bytes: None,
        }
    }

    pub const fn with_max_input_bytes(mut self, maximum: usize) -> Self {
        self.max_input_bytes = Some(maximum);
        self
    }

    pub const fn with_max_runtime_objects(mut self, maximum: usize) -> Self {
        self.max_runtime_objects = Some(maximum);
        self
    }

    /// Bound every serialized property occurrence decoded by the binary
    /// parser, including skipped/unknown/duplicate properties and properties
    /// on objects that ultimately become null slots. The same aggregate also
    /// covers header property-table entries and manifest name, path-entry, and
    /// path-component declarations.
    pub const fn with_max_runtime_properties(mut self, maximum: usize) -> Self {
        self.max_runtime_properties = Some(maximum);
        self
    }

    pub const fn with_max_imported_file_assets(mut self, maximum: usize) -> Self {
        self.max_imported_file_assets = Some(maximum);
        self
    }

    /// Bound each source-accepted `FileAssetContents` payload occurrence.
    /// This never substitutes a final-contents catalog for the source records.
    pub const fn with_max_file_asset_content_bytes(mut self, maximum: usize) -> Self {
        self.max_file_asset_content_bytes = Some(maximum);
        self
    }

    pub const fn with_max_total_file_asset_content_bytes(mut self, maximum: usize) -> Self {
        self.max_total_file_asset_content_bytes = Some(maximum);
        self
    }

    /// Bound decoded RGBA bytes retained by the native file's imported images.
    /// In-band image dimensions are admitted before loader/decode; images supplied
    /// by a host loader are checked when it returns. Later external asset loads
    /// remain the host's responsibility. This optional policy is not part of the
    /// upstream importer; `unbounded` disables this ceiling.
    pub const fn with_max_retained_decoded_image_bytes(mut self, maximum: usize) -> Self {
        self.max_retained_decoded_image_bytes = Some(maximum);
        self
    }

    pub const fn max_input_bytes(self) -> Option<usize> {
        self.max_input_bytes
    }

    pub const fn max_runtime_objects(self) -> Option<usize> {
        self.max_runtime_objects
    }

    pub const fn max_runtime_properties(self) -> Option<usize> {
        self.max_runtime_properties
    }

    pub const fn max_imported_file_assets(self) -> Option<usize> {
        self.max_imported_file_assets
    }

    pub const fn max_file_asset_content_bytes(self) -> Option<usize> {
        self.max_file_asset_content_bytes
    }

    pub const fn max_total_file_asset_content_bytes(self) -> Option<usize> {
        self.max_total_file_asset_content_bytes
    }

    pub const fn max_retained_decoded_image_bytes(self) -> Option<usize> {
        self.max_retained_decoded_image_bytes
    }

    pub(crate) fn validate_input(self, bytes: &[u8]) -> Result<()> {
        if let Some(maximum) = self.max_input_bytes()
            && bytes.len() > maximum
        {
            bail!(
                "Rive file is {} bytes; the import limit is {maximum} bytes",
                bytes.len()
            );
        }
        Ok(())
    }
}

impl Default for FileImportLimits {
    fn default() -> Self {
        Self::new()
    }
}

use nuxie_runtime::mechanical_port::source::{
    assets::{file_asset_contents::FileAssetContents, image_asset::ImageAsset},
    core::CoreHandle,
    file::ImportAdmission,
    generated::assets::{image_asset_base::ImageAssetBase, manifest_asset_base::ManifestAssetBase},
};

/// Host counters only; source File and Core owners remain the execution graph.
pub(crate) struct NativeImportAdmission {
    limits: FileImportLimits,
    properties: Cell<usize>,
    assets: Cell<usize>,
    content_bytes: Cell<usize>,
    decoded_image_bytes: Cell<usize>,
    error: RefCell<Option<anyhow::Error>>,
}

impl NativeImportAdmission {
    pub(crate) fn preflight(bytes: &[u8], limits: FileImportLimits) -> Result<Self> {
        limits.validate_input(bytes)?;
        let properties = if limits.max_runtime_objects().is_some()
            || limits.max_runtime_properties().is_some()
        {
            // Structural decoding enforces allocation budgets only. Do not
            // construct descriptors, run legacy lifecycle, or re-encode bytes.
            nuxie_binary::read_runtime_metadata(
                bytes,
                limits.max_runtime_objects(),
                limits.max_runtime_properties(),
            )?
            .decoded_property_count()
        } else {
            0
        };
        Ok(Self {
            limits,
            properties: Cell::new(properties),
            assets: Cell::new(0),
            content_bytes: Cell::new(0),
            decoded_image_bytes: Cell::new(0),
            error: RefCell::new(None),
        })
    }

    pub(crate) fn finish(&self) -> Result<()> {
        match self.error.borrow_mut().take() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn check(&self, check: impl FnOnce() -> Result<()>) -> bool {
        if self.is_rejected() {
            return false;
        }
        match check() {
            Ok(()) => true,
            Err(error) => {
                *self.error.borrow_mut() = Some(error);
                false
            }
        }
    }

    fn image_total(&self, bytes: usize) -> Result<usize> {
        let total = self
            .decoded_image_bytes
            .get()
            .checked_add(bytes)
            .context("Rive decoded image byte total overflowed usize")?;
        if let Some(maximum) = self.limits.max_retained_decoded_image_bytes() {
            ensure!(
                total <= maximum,
                "Rive images require more than {maximum} retained decoded bytes"
            );
        }
        Ok(total)
    }
}

impl ImportAdmission for NativeImportAdmission {
    fn admit_object(&self, object: &CoreHandle) -> bool {
        self.check(|| {
            let is_asset = object.with(|object| object.as_file_asset().is_some())
                .context("source import object is no longer live")?;
            if is_asset {
                let count = self.assets.get().checked_add(1)
                    .context("Rive FileAsset count overflowed usize")?;
                if let Some(maximum) = self.limits.max_imported_file_assets() {
                    ensure!(count <= maximum, "Rive file imports more than {maximum} FileAssets");
                }
                self.assets.set(count);
            }
            if let Some(bytes) = object.with_downcast_mut::<FileAssetContents, _>(|contents| contents.bytes().len()) {
                if let Some(maximum) = self.limits.max_file_asset_content_bytes() {
                    ensure!(bytes <= maximum, "Rive FileAssetContents contains {bytes} bytes; the per-content import limit is {maximum} bytes");
                }
                let total = self.content_bytes.get().checked_add(bytes)
                    .context("Rive FileAsset content byte total overflowed usize")?;
                if let Some(maximum) = self.limits.max_total_file_asset_content_bytes() {
                    ensure!(total <= maximum, "Rive FileAssets contain more than {maximum} aggregate content bytes");
                }
                self.content_bytes.set(total);
            }
            Ok(())
        })
    }

    fn admit_asset_bytes(&self, asset: &CoreHandle, bytes: &[u8]) -> bool {
        self.check(|| {
            if asset.core_type() == Some(ManifestAssetBase::TYPE_KEY) {
                let mut consumed = self.properties.get();
                nuxie_binary::validate_manifest_payload_budget(
                    bytes,
                    self.limits.max_runtime_properties(),
                    &mut consumed,
                )?;
                self.properties.set(consumed);
            }
            if asset.core_type() == Some(ImageAssetBase::TYPE_KEY)
                && !bytes.is_empty()
                && self.limits.max_retained_decoded_image_bytes().is_some()
            {
                let dimensions = nuxie_image_codec::preflight_encoded_image(bytes)
                    .context("Rive image cannot be admitted within the decoded-image budget")?;
                let size = nuxie_image_codec::decoded_rgba_len(dimensions.width, dimensions.height)
                    .context("Rive decoded image byte size is invalid")?;
                self.image_total(size)?;
            }
            Ok(())
        })
    }

    fn admit_loaded_asset(&self, asset: &CoreHandle) -> bool {
        self.check(|| {
            if self.limits.max_retained_decoded_image_bytes().is_none() {
                return Ok(());
            }
            if let Some(Some((width, height))) = asset.with_downcast::<ImageAsset, _>(|asset| {
                asset
                    .render_image()
                    .map(|image| (image.width(), image.height()))
            }) {
                let bytes = usize::try_from(width)
                    .ok()
                    .and_then(|width| {
                        usize::try_from(height)
                            .ok()
                            .and_then(|height| width.checked_mul(height))
                    })
                    .and_then(|pixels| pixels.checked_mul(4))
                    .context("Rive decoded image byte size overflowed usize")?;
                self.decoded_image_bytes.set(self.image_total(bytes)?);
            }
            Ok(())
        })
    }

    fn is_rejected(&self) -> bool {
        self.error.borrow().is_some()
    }
}
