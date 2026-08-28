use crate::mechanical_port::source::{
    assets::file_asset::FileAsset,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    factory::RuntimeFactoryHandle,
    generated::assets::{
        asset_base::AssetBaseCallbacks,
        file_asset_base::{FileAssetBase, FileAssetBaseCallbacks},
        font_asset_base::FontAssetBase,
    },
    text::font_hb::HbFont,
    text_engine::FontRef,
};

pub struct FontAsset {
    pub base: FontAssetBase,
    font: Option<FontRef>,
}

impl AssetBaseCallbacks for FontAsset {
    fn notify_property_changed(&mut self, property_key: u16) {
        AssetBaseCallbacks::notify_property_changed(&mut self.base.base, property_key);
    }
}

impl FileAssetBaseCallbacks for FontAsset {
    fn notify_property_changed(&mut self, property_key: u16) {
        AssetBaseCallbacks::notify_property_changed(self, property_key);
    }

    fn decode_cdn_uuid(&mut self, value: &[u8]) {
        FileAsset::decode_cdn_uuid(&mut self.base.base, value);
    }

    fn copy_cdn_uuid(&mut self, object: &FileAssetBase) {
        FileAsset::copy_cdn_uuid(&mut self.base.base, object);
    }
}

impl std::ops::Deref for FontAsset {
    type Target = FontAssetBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FontAsset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Default for FontAsset {
    fn default() -> Self {
        Self {
            base: FontAssetBase::default(),
            font: None,
        }
    }
}

impl FontAsset {
    pub(crate) fn restore_host_font(&mut self, font: Option<FontRef>) {
        self.font = font;
    }
    pub fn set_font_occurrence(owner: &CoreHandle, font: Option<FontRef>) {
        let referencers = owner
            .with_downcast_mut::<Self, _>(|owner| {
                owner.font = font;
                owner.base.file_asset().file_asset_referencers().to_vec()
            })
            .expect("retained FontAsset");
        for referencer in referencers {
            referencer
                .with_mut(|referencer| {
                    referencer.component_add_dirt(ComponentDirt::TEXT_SHAPE, false)
                })
                .expect("retained FontAsset referencer");
        }
    }
    pub fn decode(&mut self, data: &[u8], factory: &RuntimeFactoryHandle) -> bool {
        let font = factory.with_factory_mut(|factory| {
            factory
                .decode_font(data)
                .ok()
                .and_then(|decoded| HbFont::decode(decoded.bytes()))
        });
        self.set_font(font);
        self.font.is_some()
    }

    pub fn file_extension(&self) -> &'static str {
        "ttf"
    }

    pub fn font(&self) -> Option<FontRef> {
        self.font.clone()
    }

    pub fn set_font(&mut self, font: Option<FontRef>) {
        self.font = font;
        let referencers: Vec<CoreHandle> = self.base.file_asset().file_asset_referencers().to_vec();
        for referencer in referencers {
            referencer
                .with_mut(|referencer| {
                    referencer.component_add_dirt(ComponentDirt::TEXT_SHAPE, false)
                })
                .expect("FontAsset referencers are TextStyle instances");
        }
    }
}
