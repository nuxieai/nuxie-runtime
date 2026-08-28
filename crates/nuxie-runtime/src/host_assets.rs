//! Retained asset adapters. Descriptors are immutable host metadata; payloads
//! and referencer notifications belong to the translated FileAsset occurrences.
use crate::mechanical_port::source::{
    assets::{audio_asset::AudioAsset, font_asset::FontAsset, image_asset::ImageAsset},
    audio::audio_source::AudioSource,
    core::CoreHandle,
    file::{RuntimeFileHandle, RuntimeFileWeakHandle},
    text::font_hb::HbFont,
};
use nuxie_binary::{RuntimeFile, RuntimeObject};
use nuxie_render_api::{Factory, ImageDecodeError, RenderImage};
use std::{collections::BTreeMap, rc::Rc, sync::Arc};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeFileAssetKind {
    Image,
    Font,
    Audio,
}
#[derive(Clone)]
pub struct RuntimeFileAsset {
    file: RuntimeFileHandle,
    asset: CoreHandle,
    descriptor: RuntimeObject,
    kind: RuntimeFileAssetKind,
}
impl std::fmt::Debug for RuntimeFileAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeFileAsset")
            .field("asset", &self.asset)
            .field("descriptor", &self.descriptor)
            .finish()
    }
}
impl RuntimeFileAsset {
    pub fn from_native(
        file: RuntimeFileHandle,
        asset: CoreHandle,
        descriptor: RuntimeObject,
    ) -> Option<Self> {
        let kind = asset
            .with(|asset| {
                if asset.as_any().is::<ImageAsset>() {
                    Some(RuntimeFileAssetKind::Image)
                } else if asset.as_any().is::<FontAsset>() {
                    Some(RuntimeFileAssetKind::Font)
                } else if asset.as_any().is::<AudioAsset>() {
                    Some(RuntimeFileAssetKind::Audio)
                } else {
                    None
                }
            })
            .flatten()?;
        Some(Self {
            file,
            asset,
            descriptor,
            kind,
        })
    }
    pub fn native_handle(&self) -> CoreHandle {
        self.asset.clone()
    }
    pub fn descriptor(&self) -> &RuntimeObject {
        &self.descriptor
    }
    pub fn kind(&self) -> RuntimeFileAssetKind {
        self.kind
    }
    pub fn decode(&self, bytes: &[u8], factory: &mut dyn Factory) -> bool {
        match self.kind {
            RuntimeFileAssetKind::Image => {
                let image = factory.decode_image(bytes).ok().map(Rc::from);
                let success = image.is_some();
                ImageAsset::set_render_image_occurrence(&self.asset, image);
                success
            }
            RuntimeFileAssetKind::Font => {
                let font = factory
                    .decode_font(bytes)
                    .ok()
                    .and_then(|font| HbFont::decode(font.bytes()));
                let success = font.is_some();
                FontAsset::set_font_occurrence(&self.asset, font);
                success
            }
            RuntimeFileAssetKind::Audio => {
                let source = AudioSource::from_encoded(bytes);
                self.asset
                    .with_downcast_mut::<AudioAsset, _>(|asset| asset.set_audio_source(source));
                true
            }
        }
    }
    pub fn audio_source(&self) -> Option<Arc<nuxie_audio::AudioSource>> {
        self.asset
            .with_downcast::<AudioAsset, _>(AudioAsset::audio_source)
            .flatten()?
            .backend()
    }
    pub fn set_render_image(&self, image: Box<dyn RenderImage>) -> bool {
        if self.kind != RuntimeFileAssetKind::Image {
            return false;
        }
        ImageAsset::set_render_image_occurrence(&self.asset, Some(Rc::from(image)));
        true
    }
    pub fn set_font(&self, font: crate::RawTextFont) -> bool {
        if self.kind != RuntimeFileAssetKind::Font {
            return false;
        }
        FontAsset::set_font_occurrence(&self.asset, Some(font.native_handle()));
        true
    }
    pub fn set_audio_source(&self, source: Arc<nuxie_audio::AudioSource>) -> bool {
        if self.kind != RuntimeFileAssetKind::Audio {
            return false;
        }
        self.asset
            .with_downcast_mut::<AudioAsset, _>(|asset| {
                asset.set_audio_source(Some(AudioSource::from_backend(source)))
            })
            .is_some()
    }
}
pub trait RuntimeFileAssetLoader {
    fn load_contents(
        &mut self,
        asset: &RuntimeFileAsset,
        in_band: &[u8],
        factory: &mut dyn Factory,
    ) -> bool;
}
impl<F> RuntimeFileAssetLoader for F
where
    F: FnMut(&RuntimeFileAsset, &[u8], &mut dyn Factory) -> bool,
{
    fn load_contents(
        &mut self,
        asset: &RuntimeFileAsset,
        in_band: &[u8],
        factory: &mut dyn Factory,
    ) -> bool {
        self(asset, in_band, factory)
    }
}
pub trait RuntimeImageAssetLoader {
    fn load_contents(
        &mut self,
        asset: &RuntimeObject,
        in_band: &[u8],
        factory: &mut dyn Factory,
    ) -> bool;
}
impl<F> RuntimeImageAssetLoader for F
where
    F: FnMut(&RuntimeObject, &[u8], &mut dyn Factory) -> bool,
{
    fn load_contents(
        &mut self,
        asset: &RuntimeObject,
        in_band: &[u8],
        factory: &mut dyn Factory,
    ) -> bool {
        self(asset, in_band, factory)
    }
}
#[derive(Clone, Debug)]
pub struct RuntimeFileAssetOwners {
    assets: BTreeMap<u32, RuntimeFileAsset>,
    images: Arc<RuntimeImageAssetOwners>,
    fonts: Arc<RuntimeFontAssetOwners>,
    audio: Arc<RuntimeAudioAssetOwners>,
}
impl RuntimeFileAssetOwners {
    pub fn from_native(file: RuntimeFileHandle, descriptors: Arc<RuntimeFile>) -> Self {
        let native = file.with_file(|file| file.assets().to_vec());
        let descriptor_assets = descriptors.file_assets();
        assert_eq!(
            native.len(),
            descriptor_assets.len(),
            "native and descriptor FileAsset order must correspond"
        );
        let assets = descriptor_assets
            .into_iter()
            .zip(native)
            .filter_map(|(descriptor, asset)| {
                RuntimeFileAsset::from_native(file.clone(), asset, descriptor.clone())
                    .map(|asset| (descriptor.id, asset))
            })
            .collect::<BTreeMap<_, _>>();
        let images = Arc::new(RuntimeImageAssetOwners {
            assets: assets
                .iter()
                .filter(|(_, a)| a.kind == RuntimeFileAssetKind::Image)
                .map(|(id, a)| (*id, a.asset.clone()))
                .collect(),
            file: Some(file.clone()),
        });
        let fonts = Arc::new(RuntimeFontAssetOwners {
            assets: assets
                .iter()
                .filter(|(_, a)| a.kind == RuntimeFileAssetKind::Font)
                .map(|(id, a)| (*id, a.asset.clone()))
                .collect(),
            file: Some(file.clone()),
        });
        let audio = Arc::new(RuntimeAudioAssetOwners {
            assets: assets
                .iter()
                .filter(|(_, a)| a.kind == RuntimeFileAssetKind::Audio)
                .map(|(id, a)| (*id, a.asset.clone()))
                .collect(),
            file: Some(file.clone()),
        });
        Self {
            assets,
            images,
            fonts,
            audio,
        }
    }
    pub fn image_assets(&self) -> Arc<RuntimeImageAssetOwners> {
        self.images.clone()
    }
    pub fn font_assets(&self) -> Arc<RuntimeFontAssetOwners> {
        self.fonts.clone()
    }
    pub fn audio_assets(&self) -> Arc<RuntimeAudioAssetOwners> {
        self.audio.clone()
    }
    pub fn asset(&self, descriptor: &RuntimeObject) -> Option<RuntimeFileAsset> {
        self.assets.get(&descriptor.id).cloned()
    }
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
    file: Option<RuntimeFileHandle>,
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
            file: None,
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
    file: Option<RuntimeFileHandle>,
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
            file: None,
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
    file: Option<RuntimeFileHandle>,
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
            file: None,
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
