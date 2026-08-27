use super::{
    text_interface::TextInterface, text_style_axis::TextStyleAxis,
    text_style_feature::TextStyleFeature, text_variation_helper::TextVariationHelper,
};
use crate::mechanical_port::source::{
    assets::{file_asset::FileAsset, font_asset::FontAsset},
    component_dirt::ComponentDirt,
    core_context::CoreContext,
    generated::text::text_style_base::TextStyleBase,
    importers::import_stack::ImportStack,
    refcnt::RiveRc,
    status_code::StatusCode,
    text_engine::{Font, FontCoord, FontFeature},
};
use std::ptr::NonNull;
pub struct TextStyle {
    pub base: TextStyleBase,
    variation_helper: Option<Box<TextVariationHelper>>,
    variable_font: Option<RiveRc<Font>>,
    coords: Vec<FontCoord>,
    variations: Vec<NonNull<TextStyleAxis>>,
    style_features: Vec<NonNull<TextStyleFeature>>,
    features: Vec<FontFeature>,
    text: Option<NonNull<dyn TextInterface>>,
}
impl TextStyle {
    pub fn add_variation(&mut self, axis: &mut TextStyleAxis) {
        self.variations.push(NonNull::from(axis));
    }
    pub fn add_feature(&mut self, feature: &mut TextStyleFeature) {
        self.style_features.push(NonNull::from(feature));
    }
    pub fn on_dirty(&mut self, dirt: ComponentDirt) {
        if let Some(mut text) = self.text {
            if dirt.contains(ComponentDirt::TEXT_SHAPE) {
                unsafe { text.as_mut() }.mark_shape_dirty();
                if let Some(helper) = &mut self.variation_helper {
                    helper.component.add_dirt(ComponentDirt::TEXT_SHAPE);
                }
            }
        }
    }
    pub fn on_added_clean(&mut self, context: &mut CoreContext) -> StatusCode {
        self.text = TextInterface::from_core(self.base.parent());
        let mut code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        if !self.variations.is_empty() || !self.style_features.is_empty() {
            self.variation_helper = Some(Box::new(TextVariationHelper::new(NonNull::from(
                &mut *self,
            ))));
        }
        if let Some(helper) = &mut self.variation_helper {
            code = helper.component.on_added_dirty(context);
            if code != StatusCode::Ok {
                return code;
            }
            code = helper.component.on_added_clean(context);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn font(&mut self) -> Option<RiveRc<Font>> {
        if self.variable_font.is_some() {
            return self.variable_font.clone();
        }
        if !self.variations.is_empty() || !self.style_features.is_empty() {
            self.update_variable_font();
            if self.variable_font.is_some() {
                return self.variable_font.clone();
            }
        }
        self.base
            .font_asset()
            .and_then(|asset| unsafe { asset.as_ref() }.font())
    }
    pub fn update_variable_font(&mut self) {
        let Some(base_font) = self
            .base
            .font_asset()
            .and_then(|asset| unsafe { asset.as_ref() }.font())
        else {
            return;
        };
        if !self.variations.is_empty() || !self.style_features.is_empty() {
            self.coords.clear();
            for axis in &self.variations {
                let axis = unsafe { axis.as_ref() };
                self.coords.push(FontCoord {
                    axis: axis.base.tag(),
                    value: axis.base.axis_value(),
                });
            }
            self.features.clear();
            for feature in &self.style_features {
                let feature = unsafe { feature.as_ref() };
                self.features.push(FontFeature {
                    tag: feature.base.tag(),
                    value: feature.base.feature_value(),
                });
            }
            self.variable_font = base_font.with_options(&self.coords, &self.features);
        } else {
            self.variable_font = None;
        }
    }
    pub fn build_dependencies(&mut self) {
        if let Some(helper) = &mut self.variation_helper {
            helper.build_dependencies();
        }
        self.base.parent_mut().add_dependent(self);
        self.base.build_dependencies();
    }
    pub fn asset_id(&self) -> u32 {
        self.base.font_asset_id()
    }
    pub fn set_asset(&mut self, asset: Option<RiveRc<FileAsset>>) {
        if asset.as_ref().is_some_and(|asset| asset.is_font_asset()) {
            self.base.set_file_asset(asset);
            if self.text.is_some() {
                self.base.add_dirt(ComponentDirt::TEXT_SHAPE);
            }
        }
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let result = self.base.register_referencer(stack);
        if result != StatusCode::Ok {
            return result;
        }
        self.base.import(stack)
    }
    pub fn font_size_changed(&mut self) {
        unsafe { self.text.expect("TextStyle text").as_mut() }.mark_shape_dirty();
    }
    pub fn line_height_changed(&mut self) {
        unsafe { self.text.expect("TextStyle text").as_mut() }.mark_shape_dirty();
    }
    pub fn letter_spacing_changed(&mut self) {
        unsafe { self.text.expect("TextStyle text").as_mut() }.mark_shape_dirty();
    }
    pub fn clone_value(&self) -> Box<Self> {
        let mut twin = self.base.clone_text_style();
        if let Some(asset) = self.base.file_asset() {
            twin.set_asset(Some(asset.clone()));
        }
        twin
    }
    pub fn validate(&self, context: &CoreContext) -> bool {
        TextInterface::from_core(context.resolve(self.base.parent_id())).is_some()
    }
}
