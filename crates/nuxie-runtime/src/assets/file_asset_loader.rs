use crate::{RuntimeFontAssetOwners, RuntimeImageAssetOwners};
use nuxie_binary::{RuntimeFile, RuntimeObject};
use nuxie_render_api::Factory as RenderFactory;
use std::sync::Arc;

/// The supported concrete owner behind a loader-visible FileAsset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeFileAssetKind {
    Image,
    Font,
}

#[derive(Clone)]
enum RuntimeFileAssetOwner {
    Image(Arc<RuntimeImageAssetOwners>),
    Font(Arc<RuntimeFontAssetOwners>),
}

/// A retained FileAsset handle passed to [`RuntimeFileAssetLoader`].
///
/// Cloning this value is the safe-Rust counterpart of retaining the C++ asset
/// for asynchronous completion. Calling [`Self::decode`] later replaces the
/// asset-owned resource and notifies every live referencer.
#[derive(Clone)]
pub struct RuntimeFileAsset {
    descriptor: RuntimeObject,
    owner: RuntimeFileAssetOwner,
}

impl std::fmt::Debug for RuntimeFileAsset {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeFileAsset")
            .field("global_id", &self.descriptor.id)
            .field("type_name", &self.descriptor.type_name)
            .finish()
    }
}

impl RuntimeFileAsset {
    pub fn descriptor(&self) -> &RuntimeObject {
        &self.descriptor
    }

    pub fn kind(&self) -> RuntimeFileAssetKind {
        match self.owner {
            RuntimeFileAssetOwner::Image(_) => RuntimeFileAssetKind::Image,
            RuntimeFileAssetOwner::Font(_) => RuntimeFileAssetKind::Font,
        }
    }

    pub fn decode(&self, bytes: &[u8], factory: &mut dyn RenderFactory) -> bool {
        match &self.owner {
            RuntimeFileAssetOwner::Image(owners) => {
                owners.decode(self.descriptor.id, bytes, factory).is_ok()
            }
            RuntimeFileAssetOwner::Font(owners) => owners.decode(self.descriptor.id, bytes),
        }
    }
}

/// Import-time host callback mirroring C++ `FileAssetLoader::loadContents`.
///
/// Returning `true` claims responsibility and suppresses in-band fallback.
/// The loader may decode immediately or retain a cloned asset handle and
/// complete it asynchronously.
pub trait RuntimeFileAssetLoader {
    fn load_contents(
        &mut self,
        asset: &RuntimeFileAsset,
        in_band: &[u8],
        factory: &mut dyn RenderFactory,
    ) -> bool;
}

impl<F> RuntimeFileAssetLoader for F
where
    F: FnMut(&RuntimeFileAsset, &[u8], &mut dyn RenderFactory) -> bool,
{
    fn load_contents(
        &mut self,
        asset: &RuntimeFileAsset,
        in_band: &[u8],
        factory: &mut dyn RenderFactory,
    ) -> bool {
        self(asset, in_band, factory)
    }
}

/// File-owned ImageAsset and FontAsset resources imported through one loader
/// seam. Audio and command-queue resource types intentionally remain outside
/// this self-contained A1 module.
#[derive(Debug)]
pub struct RuntimeFileAssetOwners {
    images: Arc<RuntimeImageAssetOwners>,
    fonts: Arc<RuntimeFontAssetOwners>,
    loader_owns_images: bool,
}

impl RuntimeFileAssetOwners {
    pub fn from_runtime(
        runtime: &RuntimeFile,
        max_retained_decoded_image_bytes: Option<usize>,
    ) -> Self {
        Self {
            images: Arc::new(RuntimeImageAssetOwners::with_max_retained_decoded_bytes(
                max_retained_decoded_image_bytes,
            )),
            fonts: Arc::new(RuntimeFontAssetOwners::from_runtime(runtime)),
            loader_owns_images: false,
        }
    }

    pub fn import_with_loader(
        runtime: &RuntimeFile,
        max_retained_decoded_image_bytes: Option<usize>,
        factory: &mut dyn RenderFactory,
        loader: &mut dyn RuntimeFileAssetLoader,
    ) -> Self {
        let owners = Self {
            images: Arc::new(RuntimeImageAssetOwners::with_max_retained_decoded_bytes(
                max_retained_decoded_image_bytes,
            )),
            fonts: Arc::new(RuntimeFontAssetOwners::default()),
            loader_owns_images: true,
        };
        for entry in runtime.imported_file_assets_with_contents() {
            let owner = match entry.asset.type_name {
                "ImageAsset" => RuntimeFileAssetOwner::Image(Arc::clone(&owners.images)),
                "FontAsset" => RuntimeFileAssetOwner::Font(Arc::clone(&owners.fonts)),
                _ => continue,
            };
            let asset = RuntimeFileAsset {
                descriptor: entry.asset.clone(),
                owner,
            };
            let in_band = entry.contents.unwrap_or_default();
            if !loader.load_contents(&asset, in_band, factory) && !in_band.is_empty() {
                let _ = asset.decode(in_band, factory);
            }
            if asset.kind() == RuntimeFileAssetKind::Image {
                owners.images.mark_import_resolved(asset.descriptor.id);
            }
        }
        owners
    }

    pub fn image_assets(&self) -> Arc<RuntimeImageAssetOwners> {
        Arc::clone(&self.images)
    }

    pub(crate) fn loader_image_assets(&self) -> Option<Arc<RuntimeImageAssetOwners>> {
        self.loader_owns_images.then(|| Arc::clone(&self.images))
    }

    pub fn font_assets(&self) -> Arc<RuntimeFontAssetOwners> {
        Arc::clone(&self.fonts)
    }

    pub fn asset(&self, descriptor: &RuntimeObject) -> Option<RuntimeFileAsset> {
        let owner = match descriptor.type_name {
            "ImageAsset" => RuntimeFileAssetOwner::Image(Arc::clone(&self.images)),
            "FontAsset" => RuntimeFileAssetOwner::Font(Arc::clone(&self.fonts)),
            _ => return None,
        };
        Some(RuntimeFileAsset {
            descriptor: descriptor.clone(),
            owner,
        })
    }
}
