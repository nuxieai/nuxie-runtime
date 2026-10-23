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
impl std::ops::Deref for TextStyle {
    type Target = TextStyleBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextStyle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TextStyle {
    pub const TYPE_KEY: u16 = TextStyleBase::TYPE_KEY;
}

pub struct TextStyle {
    pub base: TextStyleBase,
    variation_helper: Option<RuntimeTextVariationHelperHandle>,
    file_asset_referencer: FileAssetReferencer,
    variable_font: Option<FontRef>,
    variable_font_base: Option<FontRef>,
    coords: Vec<FontCoord>,
    variations: Vec<CoreHandle>,
    style_features: Vec<CoreHandle>,
    features: Vec<FontFeature>,
    text: Option<CoreHandle>,
    css_input_options_generation: Option<std::rc::Rc<std::cell::Cell<u64>>>,
    css_input_font_key: Option<CssInputFontKey>,
    #[cfg(test)]
    pub(crate) css_input_font_refresh_count: u64,
}

// Retain the font instances, rather than pointer integers which an allocator
// could reuse after an asset replacement. The options generation covers tags
// and values, including changes which bypass component dirt.
#[derive(Clone)]
pub(super) struct CssInputFontKey {
    asset: Option<CoreHandle>,
    asset_id: u32,
    base_font: Option<FontRef>,
    pub(super) font: Option<FontRef>,
    options_generation: u64,
}

impl PartialEq for CssInputFontKey {
    fn eq(&self, other: &Self) -> bool {
        let same_font = |a: &Option<FontRef>, b: &Option<FontRef>| match (a, b) {
            (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        self.asset == other.asset
            && self.asset_id == other.asset_id
            && same_font(&self.base_font, &other.base_font)
            && same_font(&self.font, &other.font)
            && self.options_generation == other.options_generation
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            base: TextStyleBase::default(),
            variation_helper: None,
            file_asset_referencer: FileAssetReferencer::default(),
            variable_font: None,
            variable_font_base: None,
            coords: Vec::new(),
            variations: Vec::new(),
            style_features: Vec::new(),
            features: Vec::new(),
            text: None,
            css_input_options_generation: None,
            css_input_font_key: None,
            #[cfg(test)]
            css_input_font_refresh_count: 0,
        }
    }
}
impl TextStyle {
    fn font_asset(&self) -> Option<CoreHandle> {
        self.file_asset_referencer
            .asset()
            .filter(|asset| asset.is_type_of(crate::mechanical_port::source::generated::assets::font_asset_base::FontAssetBase::TYPE_KEY))
    }

    /// The lazily-created owned graph node, if this style needed one.
    pub fn variation_helper(&self) -> Option<&RuntimeTextVariationHelperHandle> {
        self.variation_helper.as_ref()
    }

    pub(super) fn css_input_options_generation(&self) -> Option<std::rc::Rc<std::cell::Cell<u64>>> {
        self.css_input_options_generation.clone()
    }
    pub fn add_variation(&mut self, axis: CoreHandle) {
        if let Some(generation) = &self.css_input_options_generation {
            generation.set(generation.get().wrapping_add(1));
        }
        self.variations.push(axis);
    }
    pub fn add_feature(&mut self, feature: CoreHandle) {
        if let Some(generation) = &self.css_input_options_generation {
            generation.set(generation.get().wrapping_add(1));
        }
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

    pub(crate) fn add_dirt_occurrence(
        owner: &CoreHandle,
        value: ComponentDirt,
        recurse: bool,
    ) -> bool {
        crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
            owner.clone(),
        )
        .add_dirt(value, recurse)
    }

    pub(crate) fn on_dirty_occurrence(owner: &CoreHandle, dirt: ComponentDirt) {
        let text = owner
            .with(|object| object.as_text_style().expect("TextStyle").text.clone())
            .flatten();
        if let Some(text) = text.filter(|_| dirt.contains(ComponentDirt::TEXT_SHAPE)) {
            if text.is_type_of(
                crate::mechanical_port::source::generated::text::text_base::TextBase::TYPE_KEY,
            ) {
                super::text::Text::mark_shape_dirty_occurrence(&text, true);
            } else {
                text.with_mut(|object| object.text_interface_mark_shape_dirty());
            }
            let helper = owner
                .with(|object| {
                    object
                        .as_text_style()
                        .expect("TextStyle")
                        .variation_helper
                        .as_ref()
                        .map(|helper| helper.occurrence())
                })
                .flatten();
            if let Some(helper) = helper {
                helper.add_dirt(ComponentDirt::TEXT_SHAPE, false);
            }
        }
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        self.text = <dyn TextInterface>::from_core(self.base.parent_handle());
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
    /// CSS reads through current variation inputs before any layout solve.
    /// The translated font() entry retains upstream's helper update order.
    pub(super) fn css_input_font_key(&mut self) -> CssInputFontKey {
        let generation = self
            .css_input_options_generation
            .get_or_insert_with(|| {
                let generation = std::rc::Rc::new(std::cell::Cell::new(0u64));
                for axis in &self.variations {
                    axis.with_downcast_mut::<TextStyleAxis, _>(|a| {
                        a.base.bind_css_input_options_generation(generation.clone())
                    });
                }
                for feature in &self.style_features {
                    feature.with_downcast_mut::<TextStyleFeature, _>(|f| {
                        f.base.bind_css_input_options_generation(generation.clone())
                    });
                }
                generation
            })
            .get();
        let asset = self.font_asset();
        let base_font = asset
            .as_ref()
            .and_then(|a| a.with_downcast::<FontAsset, _>(FontAsset::font))
            .flatten();
        // The helper can replace this font without going through our setters.
        let font = self.variable_font.clone().or_else(|| base_font.clone());
        let mut key = CssInputFontKey {
            asset,
            asset_id: self.base.font_asset_id(),
            base_font,
            font,
            options_generation: generation,
        };
        if self.css_input_font_key.as_ref() != Some(&key) {
            #[cfg(test)]
            {
                self.css_input_font_refresh_count += 1;
            }
            self.update_variable_font();
            key.font = self.font();
            self.css_input_font_key = Some(key.clone());
        }
        key
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
            // Not ready yet, or the font was cleared. Drop the variable font
            // cached from the previous base so font() cannot return it.
            self.variable_font = None;
            self.variable_font_base = None;
            return;
        };
        if !self.variations.is_empty() || !self.style_features.is_empty() {
            if self.variable_font.is_some()
                && self
                    .variable_font_base
                    .as_ref()
                    .is_some_and(|cached| std::sync::Arc::ptr_eq(cached, &base_font))
                && self.same_options()
            {
                return;
            }
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
            self.variable_font = Some(base_font.with_options(&self.coords, &self.features));
            self.variable_font_base = Some(base_font);
        } else {
            self.variable_font = None;
            self.variable_font_base = None;
        }
    }
    fn same_options(&self) -> bool {
        if self.coords.len() != self.variations.len()
            || self.features.len() != self.style_features.len()
        {
            return false;
        }
        for (coord, axis) in self.coords.iter().zip(&self.variations) {
            if !axis
                .with_downcast::<TextStyleAxis, _>(|axis| {
                    coord.axis == axis.base.tag() && coord.value == axis.base.axis_value()
                })
                .expect("live TextStyleAxis")
            {
                return false;
            }
        }
        for (cached, feature) in self.features.iter().zip(&self.style_features) {
            if !feature
                .with_downcast::<TextStyleFeature, _>(|feature| {
                    cached.tag == feature.base.tag() && cached.value == feature.base.feature_value()
                })
                .expect("live TextStyleFeature")
            {
                return false;
            }
        }
        true
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
    pub(crate) fn file_asset_referencer_mut(&mut self) -> &mut FileAssetReferencer {
        &mut self.file_asset_referencer
    }
    pub fn set_asset_occurrence(owner: &CoreHandle, asset: Option<CoreHandle>) {
        if !asset.as_ref().is_some_and(|asset| {
            asset.is_type_of(crate::mechanical_port::source::generated::assets::font_asset_base::FontAssetBase::TYPE_KEY)
        }) {
            return;
        }
        let has_text = owner
            .with_mut(|object| {
                let style = object
                    .as_text_style_mut()
                    .expect("TextStyle asset referencer");
                style.file_asset_referencer.set_asset(owner.clone(), asset);
                style.text.is_some()
            })
            .expect("live TextStyle");
        if has_text {
            // Upstream setAsset calls the most-derived onDirty through addDirt.
            // Release the style before Text dirt traverses its dependents.
            Self::add_dirt_occurrence(owner, ComponentDirt::TEXT_SHAPE, false);
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
    pub(crate) fn set_double_occurrence(owner: &CoreHandle, key: u16, value: f32) -> bool {
        let Some(changed) = owner.with_mut(|object| {
            let style = object
                .as_text_style_mut()
                .expect("TextStyle property owner");
            match key {
                TextStyleBase::FONT_SIZE_PROPERTY_KEY => style.base.set_font_size_value(value),
                TextStyleBase::LINE_HEIGHT_PROPERTY_KEY => style.base.set_line_height_value(value),
                TextStyleBase::LETTER_SPACING_PROPERTY_KEY => {
                    style.base.set_letter_spacing_value(value)
                }
                _ => unreachable!("TextStyle generated numeric property"),
            }
        }) else {
            return false;
        };
        if !changed {
            return true;
        }

        // All three pinned changed callbacks call m_text->markShapeDirty().
        // That callback invalidates this same TextStylePaint's stroke effects,
        // so the property owner borrow must end before entering Text.
        let text = owner
            .with(|object| object.as_text_style().expect("TextStyle").text.clone())
            .flatten()
            .expect("TextStyle text");
        if text.is_type_of(
            crate::mechanical_port::source::generated::text::text_base::TextBase::TYPE_KEY,
        ) {
            super::text::Text::mark_shape_dirty_occurrence(&text, true);
        } else {
            text.with_mut(|object| object.text_interface_mark_shape_dirty());
        }
        owner.notify_property_changed(key);
        true
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
        <dyn TextInterface>::from_core(context.resolve(self.base.parent_id())).is_some()
    }
}
