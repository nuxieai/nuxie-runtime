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
    text::font_hb::{HbFont, LetterSpacingMode, ShapingPrecision},
    text_engine::FontRef,
};

pub struct FontAsset {
    pub base: FontAssetBase,
    font: Option<FontRef>,
    shaping_precision: Option<ShapingPrecision>,
    letter_spacing_mode: Option<LetterSpacingMode>,
    experimental_space_breaks: Option<bool>,
    experimental_css_tabs: Option<bool>,
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
            shaping_precision: None,
            letter_spacing_mode: None,
            experimental_space_breaks: None,
            experimental_css_tabs: None,
        }
    }
}

impl FontAsset {
    fn prepare_font(&self, font: Option<FontRef>) -> Option<FontRef> {
        if self.shaping_precision.is_none()
            && self.letter_spacing_mode.is_none()
            && self.experimental_space_breaks.is_none()
            && self.experimental_css_tabs.is_none()
        {
            return font;
        }
        let mut font = font?;
        if let Some(precision) = self.shaping_precision {
            font = font
                .as_any()
                .downcast_ref::<HbFont>()?
                .with_shaping_precision(precision);
        }
        if let Some(mode) = self.letter_spacing_mode {
            font = font
                .as_any()
                .downcast_ref::<HbFont>()?
                .with_letter_spacing_mode(mode);
        }
        if let Some(enabled) = self.experimental_space_breaks {
            font = font
                .as_any()
                .downcast_ref::<HbFont>()?
                .with_experimental_space_breaks(enabled);
        }
        if let Some(enabled) = self.experimental_css_tabs {
            font = font
                .as_any()
                .downcast_ref::<HbFont>()?
                .with_experimental_css_tabs(enabled);
        }
        Some(font)
    }

    /// Retain shaping precision across replacement, decode and restore.
    pub fn set_shaping_precision_occurrence(
        owner: &CoreHandle,
        precision: ShapingPrecision,
    ) -> bool {
        let font = owner
            .with_downcast_mut::<Self, _>(|asset| {
                if asset
                    .font
                    .as_ref()
                    .is_some_and(|f| !f.as_any().is::<HbFont>())
                {
                    return None;
                }
                asset.shaping_precision = Some(precision);
                Some(asset.font.clone())
            })
            .flatten();
        let Some(font) = font else {
            return false;
        };
        Self::set_font_occurrence(owner, font);
        true
    }

    /// Retain preserved-space break policy across replacement, decode and restore.
    pub fn set_experimental_space_breaks_occurrence(owner: &CoreHandle, enabled: bool) -> bool {
        let font = owner
            .with_downcast_mut::<Self, _>(|asset| {
                if asset
                    .font
                    .as_ref()
                    .is_some_and(|f| !f.as_any().is::<HbFont>())
                {
                    return None;
                }
                asset.experimental_space_breaks = Some(enabled);
                Some(asset.font.clone())
            })
            .flatten();
        let Some(font) = font else {
            return false;
        };
        Self::set_font_occurrence(owner, font);
        true
    }

    /// Retain CSS tab-stop policy across replacement, decode and restore.
    pub fn set_experimental_css_tabs_occurrence(owner: &CoreHandle, enabled: bool) -> bool {
        let font = owner
            .with_downcast_mut::<Self, _>(|asset| {
                if asset
                    .font
                    .as_ref()
                    .is_some_and(|f| !f.as_any().is::<HbFont>())
                {
                    return None;
                }
                asset.experimental_css_tabs = Some(enabled);
                Some(asset.font.clone())
            })
            .flatten();
        let Some(font) = font else {
            return false;
        };
        Self::set_font_occurrence(owner, font);
        true
    }

    /// Select a retained host policy without changing ordinary Rive imports.
    /// Returns false without mutation if the current font has a foreign backend.
    pub fn set_letter_spacing_mode_occurrence(owner: &CoreHandle, mode: LetterSpacingMode) -> bool {
        let font = owner
            .with_downcast_mut::<Self, _>(|asset| {
                if asset
                    .font
                    .as_ref()
                    .is_some_and(|f| !f.as_any().is::<HbFont>())
                {
                    return None;
                }
                asset.letter_spacing_mode = Some(mode);
                Some(asset.font.clone())
            })
            .flatten();
        let Some(font) = font else {
            return false;
        };
        Self::set_font_occurrence(owner, font);
        true
    }
    pub(crate) fn restore_host_font(&mut self, font: Option<FontRef>) {
        self.font = self.prepare_font(font);
    }
    pub fn set_font_occurrence(owner: &CoreHandle, font: Option<FontRef>) {
        let referencers = owner
            .with_downcast_mut::<Self, _>(|owner| {
                owner.font = owner.prepare_font(font);
                owner.base.file_asset().file_asset_referencers().to_vec()
            })
            .expect("retained FontAsset");
        for referencer in referencers {
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                referencer,
            )
            .add_dirt(ComponentDirt::TEXT_SHAPE, false);
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
        self.font = self.prepare_font(font);
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
