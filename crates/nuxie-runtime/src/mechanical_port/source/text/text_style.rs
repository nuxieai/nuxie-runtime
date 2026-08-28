use super::{
    text_interface::TextInterface, text_style_axis::TextStyleAxis,
    text_style_feature::TextStyleFeature, text_variation_helper::RuntimeTextVariationHelperHandle,
};
use crate::mechanical_port::source::{
    assets::{file_asset_referencer::FileAssetReferencer, font_asset::FontAsset},
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::CoreContext,
    generated::text::text_style_base::TextStyleBase,
    importers::import_stack::ImportStack,
    status_code::StatusCode,
    text_engine::{FontCoord, FontFeature, FontRef},
};
pub struct TextStyle {
    pub base: TextStyleBase,
    variation_helper: Option<RuntimeTextVariationHelperHandle>,
    file_asset_referencer: FileAssetReferencer,
    variable_font: Option<FontRef>,
    coords: Vec<FontCoord>,
    variations: Vec<CoreHandle>,
    style_features: Vec<CoreHandle>,
    features: Vec<FontFeature>,
    text: Option<CoreHandle>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            base: TextStyleBase::default(),
            variation_helper: None,
            file_asset_referencer: FileAssetReferencer::default(),
            variable_font: None,
            coords: Vec::new(),
            variations: Vec::new(),
            style_features: Vec::new(),
            features: Vec::new(),
            text: None,
        }
    }
}
impl TextStyle {
    fn font_asset(&self) -> Option<CoreHandle> {
        self.file_asset_referencer
            .asset()
            .filter(|asset| asset.with_downcast::<FontAsset, _>(|_| ()).is_some())
    }

    pub fn add_variation(&mut self, axis: CoreHandle) {
        self.variations.push(axis);
    }
    pub fn add_feature(&mut self, feature: CoreHandle) {
        self.style_features.push(feature);
    }
    pub fn on_dirty(&mut self, dirt: ComponentDirt) {
        if let Some(text) = self.text.as_ref() {
            if dirt.contains(ComponentDirt::TEXT_SHAPE) {
                text.with_mut(|text| text.text_interface_mark_shape_dirty());
                if let Some(helper) = &mut self.variation_helper {
                    helper
                        .occurrence()
                        .add_dirt(ComponentDirt::TEXT_SHAPE, false);
                }
            }
        }
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        self.text = TextInterface::from_core(self.base.parent_handle());
        let mut code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        if !self.variations.is_empty() || !self.style_features.is_empty() {
            let Some(this) = self.base.handle() else {
                return StatusCode::MissingObject;
            };
            self.variation_helper = Some(RuntimeTextVariationHelperHandle::new(this));
        }
        if let Some(helper) = &mut self.variation_helper {
            code = helper.with_mut(|helper| helper.component.on_added_dirty_runtime(context));
            if code != StatusCode::Ok {
                return code;
            }
            code = helper.with_mut(|helper| helper.component.on_added_clean(context));
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn font(&mut self) -> Option<FontRef> {
        if self.variable_font.is_some() {
            return self.variable_font.clone();
        }
        if !self.variations.is_empty() || !self.style_features.is_empty() {
            self.update_variable_font();
            if self.variable_font.is_some() {
                return self.variable_font.clone();
            }
        }
        self.font_asset()
            .and_then(|asset| asset.with_downcast::<FontAsset, _>(FontAsset::font))
            .flatten()
    }
    pub fn update_variable_font(&mut self) {
        let Some(base_font) = self
            .font_asset()
            .and_then(|asset| asset.with_downcast::<FontAsset, _>(FontAsset::font))
            .flatten()
        else {
            return;
        };
        if !self.variations.is_empty() || !self.style_features.is_empty() {
            self.coords.clear();
            for axis in &self.variations {
                axis.with_downcast::<TextStyleAxis, _>(|axis| {
                    self.coords.push(FontCoord {
                        axis: axis.base.tag(),
                        value: axis.base.axis_value(),
                    });
                });
            }
            self.features.clear();
            for feature in &self.style_features {
                feature.with_downcast::<TextStyleFeature, _>(|feature| {
                    self.features.push(FontFeature {
                        tag: feature.base.tag(),
                        value: feature.base.feature_value(),
                    });
                });
            }
            self.variable_font = base_font.with_options(&self.coords, &self.features);
        } else {
            self.variable_font = None;
        }
    }
    pub fn build_dependencies(&mut self) {
        if let Some(helper) = &mut self.variation_helper {
            let text = self.base.parent_handle().expect("TextStyle parent");
            helper.with_mut(|helper| helper.build_dependencies_for_text(text));
        }
        if let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) {
            parent.with_mut(|parent| parent.component_add_dependent(this));
        }
        self.base.build_dependencies();
    }
    pub fn asset_id(&self) -> u32 {
        self.base.font_asset_id()
    }
    pub fn set_asset(&mut self, asset: Option<CoreHandle>) {
        if asset.as_ref().is_some_and(|asset| {
            asset
                .with_downcast::<FontAsset, _>(|_| true)
                .unwrap_or(false)
        }) {
            if let Some(this) = self.base.handle() {
                self.file_asset_referencer.set_asset(this, asset);
            }
            if self.text.is_some() {
                self.base.add_dirt(ComponentDirt::TEXT_SHAPE);
            }
        }
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        let result = self.file_asset_referencer.register_referencer(this, stack);
        if result != StatusCode::Ok {
            return result;
        }
        self.base.import(stack)
    }
    pub fn font_size_changed(&mut self) {
        self.text
            .as_ref()
            .expect("TextStyle text")
            .with_mut(|text| text.text_interface_mark_shape_dirty());
    }
    pub fn line_height_changed(&mut self) {
        self.text
            .as_ref()
            .expect("TextStyle text")
            .with_mut(|text| text.text_interface_mark_shape_dirty());
    }
    pub fn letter_spacing_changed(&mut self) {
        self.text
            .as_ref()
            .expect("TextStyle text")
            .with_mut(|text| text.text_interface_mark_shape_dirty());
    }
    pub fn clone_value(&self) -> Box<Self> {
        let mut twin = Box::new(Self::default());
        let mut base = std::mem::take(&mut twin.base);
        base.copy(&self.base, twin.as_mut());
        twin.base = base;
        if let Some(asset) = self.file_asset_referencer.asset() {
            twin.file_asset_referencer.set_asset_unattached(Some(asset));
        }
        twin
    }
    pub fn validate(&self, context: &dyn CoreContext) -> bool {
        TextInterface::from_core(context.resolve(self.base.parent_id())).is_some()
    }
}
