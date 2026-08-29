//! Retained host adapters for assets owned by translated files.
use crate::mechanical_port::source::{
    assets::{audio_asset::AudioAsset, font_asset::FontAsset, image_asset::ImageAsset},
    audio::audio_source::AudioSource,
    core::CoreHandle,
    file::RuntimeFileWeakHandle,
    text::font_hb::HbFont,
};
use nuxie_render_api::{Factory, ImageDecodeError, RenderImage};
use std::{collections::BTreeMap, rc::Rc, sync::Arc};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeFileAssetKind {
    Image,
    Font,
    Audio,
}
fn native_catalog(
    file: &RuntimeFileWeakHandle,
    kind: RuntimeFileAssetKind,
) -> BTreeMap<u32, CoreHandle> {
    let Some(file) = file.upgrade() else {
        return BTreeMap::new();
    };
    file.with_file(|file| {
        file.assets()
            .iter()
            .filter_map(|asset| {
                asset
                    .with(|owner| {
                        let matches = match kind {
                            RuntimeFileAssetKind::Image => owner.as_any().is::<ImageAsset>(),
                            RuntimeFileAssetKind::Font => owner.as_any().is::<FontAsset>(),
                            RuntimeFileAssetKind::Audio => owner.as_any().is::<AudioAsset>(),
                        };
                        matches.then(|| {
                            (
                                owner
                                    .as_file_asset()
                                    .expect("native FileAsset")
                                    .file_asset_base()
                                    .asset_id(),
                                asset.clone(),
                            )
                        })
                    })
                    .flatten()
            })
            .collect()
    })
}
#[derive(Clone)]
pub struct RuntimeImageAssetOwners {
    assets: BTreeMap<u32, CoreHandle>,
}
impl std::fmt::Debug for RuntimeImageAssetOwners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeImageAssetOwners")
            .field("assets", &self.assets)
            .finish()
    }
}
impl RuntimeImageAssetOwners {
    pub fn from_native_file(file: RuntimeFileWeakHandle) -> Self {
        Self {
            assets: native_catalog(&file, RuntimeFileAssetKind::Image),
        }
    }
    pub fn get(&self, id: u32) -> Option<Rc<dyn RenderImage>> {
        self.assets
            .get(&id)?
            .with_downcast::<ImageAsset, _>(|asset| asset.render_image().cloned())
            .flatten()
    }
    pub fn insert(&self, id: u32, image: Box<dyn RenderImage>) {
        ImageAsset::set_render_image_occurrence(
            self.assets.get(&id).expect("known native ImageAsset"),
            Some(Rc::from(image)),
        );
    }
    pub fn decode(
        &self,
        id: u32,
        bytes: &[u8],
        factory: &mut dyn Factory,
    ) -> Result<(), ImageDecodeError> {
        let image = factory.decode_image(bytes)?;
        self.insert(id, image);
        Ok(())
    }
}
#[derive(Clone)]
pub struct RuntimeFontAssetOwners {
    assets: BTreeMap<u32, CoreHandle>,
}
impl std::fmt::Debug for RuntimeFontAssetOwners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeFontAssetOwners")
            .field("assets", &self.assets)
            .finish()
    }
}
impl RuntimeFontAssetOwners {
    pub fn from_native_file(file: RuntimeFileWeakHandle) -> Self {
        Self {
            assets: native_catalog(&file, RuntimeFileAssetKind::Font),
        }
    }
    pub fn get(&self, id: u32) -> Option<Arc<[u8]>> {
        let font = self
            .assets
            .get(&id)?
            .with_downcast::<FontAsset, _>(FontAsset::font)
            .flatten()?;
        font.as_any()
            .downcast_ref::<HbFont>()
            .map(HbFont::source_bytes)
    }
    pub fn native_font(
        &self,
        id: u32,
    ) -> Option<crate::mechanical_port::source::text_engine::FontRef> {
        self.assets
            .get(&id)?
            .with_downcast::<FontAsset, _>(FontAsset::font)
            .flatten()
    }
    pub fn insert(&self, id: u32, font: crate::RawTextFont) {
        FontAsset::set_font_occurrence(
            self.assets.get(&id).expect("known native FontAsset"),
            Some(font.native_handle()),
        );
    }
    pub fn decode(&self, id: u32, bytes: &[u8], factory: &mut dyn Factory) -> bool {
        let Some(asset) = self.assets.get(&id) else {
            return false;
        };
        let font = factory
            .decode_font(bytes)
            .ok()
            .and_then(|font| HbFont::decode(font.bytes()));
        let success = font.is_some();
        FontAsset::set_font_occurrence(asset, font);
        success
    }
}
#[derive(Clone)]
pub struct RuntimeAudioAssetOwners {
    assets: BTreeMap<u32, CoreHandle>,
}
impl std::fmt::Debug for RuntimeAudioAssetOwners {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeAudioAssetOwners")
            .field("assets", &self.assets)
            .finish()
    }
}
impl RuntimeAudioAssetOwners {
    pub fn from_native_file(file: RuntimeFileWeakHandle) -> Self {
        Self {
            assets: native_catalog(&file, RuntimeFileAssetKind::Audio),
        }
    }
    pub fn get(&self, id: u32) -> Option<Arc<nuxie_audio::AudioSource>> {
        self.assets
            .get(&id)?
            .with_downcast::<AudioAsset, _>(AudioAsset::audio_source)
            .flatten()?
            .backend()
    }
    pub fn insert(&self, id: u32, source: Arc<nuxie_audio::AudioSource>) {
        self.assets
            .get(&id)
            .expect("known native AudioAsset")
            .with_downcast_mut::<AudioAsset, _>(|asset| {
                asset.set_audio_source(Some(AudioSource::from_backend(source)))
            });
    }
    pub fn decode(&self, id: u32, bytes: &[u8], _factory: &mut dyn Factory) -> bool {
        let Some(asset) = self.assets.get(&id) else {
            return false;
        };
        asset.with_downcast_mut::<AudioAsset, _>(|asset| {
            asset.set_audio_source(AudioSource::from_encoded(bytes))
        });
        true
    }
}
