use crate::mechanical_port::source::{
    component::ComponentDirt, core::CoreHandle, core_context::CoreContext, factory::Factory,
    generated::assets::font_asset_base::FontAssetBase, text_engine::FontRef,
};

pub struct FontAsset {
    pub base: FontAssetBase,
    font: Option<FontRef>,
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
    pub fn decode(
        &mut self,
        data: &[u8],
        factory: &mut Factory,
        context: &mut CoreContext,
    ) -> bool {
        let font = factory.decode_font(data);
        self.set_font(font, context);
        self.font.is_some()
    }

    pub fn file_extension(&self) -> &'static str {
        "ttf"
    }

    pub fn font(&self) -> Option<FontRef> {
        self.font.clone()
    }

    pub fn set_font(&mut self, font: Option<FontRef>, context: &mut CoreContext) {
        self.font = font;
        let referencers: Vec<CoreHandle> = self.base.file_asset().file_asset_referencers().to_vec();
        for referencer in referencers {
            context
                .text_style_mut(referencer)
                .expect("FontAsset referencers are TextStyle instances")
                .add_dirt(ComponentDirt::TEXT_SHAPE);
        }
    }
}
