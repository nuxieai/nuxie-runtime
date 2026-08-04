use std::sync::Arc;

use super::super::ArtboardInstance;
use crate::components::ComponentDirt;
use crate::properties::property_key_for_name;
use crate::view_model::RuntimeFontAssetValue;

#[derive(Debug, Clone, Copy)]
pub(in crate::artboard) struct RuntimeTextStyleFeatureOption {
    pub(in crate::artboard) shape_revision: u64,
    pub(in crate::artboard) tag: u32,
    pub(in crate::artboard) value: u32,
}

impl ArtboardInstance {
    pub(crate) fn text_style_font_override(
        &self,
        local_id: usize,
    ) -> Option<&RuntimeFontAssetValue> {
        self.text_style_font_overrides.get(&local_id)
    }

    pub(crate) fn text_style_feature_option(
        &self,
        local_id: usize,
        authored_tag: u32,
        authored_value: u32,
    ) -> (u32, u32) {
        let shape_revision = self.text_shape_revision;
        let mut options = self.text_style_feature_options.borrow_mut();
        let option = options
            .entry(local_id)
            .or_insert_with(|| RuntimeTextStyleFeatureOption {
                shape_revision,
                tag: property_key_for_name("TextStyleFeature", "tag")
                    .and_then(|key| self.uint_property(local_id, key))
                    .and_then(|value| u32::try_from(value).ok())
                    .unwrap_or(authored_tag),
                value: property_key_for_name("TextStyleFeature", "featureValue")
                    .and_then(|key| self.uint_property(local_id, key))
                    .and_then(|value| u32::try_from(value).ok())
                    .unwrap_or(authored_value),
            });
        if option.shape_revision != shape_revision {
            option.tag = property_key_for_name("TextStyleFeature", "tag")
                .and_then(|key| self.uint_property(local_id, key))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(authored_tag);
            option.value = property_key_for_name("TextStyleFeature", "featureValue")
                .and_then(|key| self.uint_property(local_id, key))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(authored_value);
            option.shape_revision = shape_revision;
        }
        (option.tag, option.value)
    }

    pub(crate) fn set_text_style_font_override(
        &mut self,
        local_id: usize,
        value: RuntimeFontAssetValue,
    ) -> bool {
        let unchanged = self
            .text_style_font_overrides
            .get(&local_id)
            .is_some_and(|current| {
                current.file_asset_index() == value.file_asset_index()
                    && match (current.live_font_bytes_arc(), value.live_font_bytes_arc()) {
                        (Some(current), Some(next)) => {
                            Arc::ptr_eq(current, next) || current.as_ref() == next.as_ref()
                        }
                        (None, None) => true,
                        _ => false,
                    }
            });
        if unchanged {
            return false;
        }
        self.text_style_font_overrides.insert(local_id, value);
        self.mark_text_style_shape_dirty(local_id);
        self.mark_path_changed();
        self.mark_layout_changed();
        true
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn debug_set_text_style_font_bytes(&mut self, local_id: usize, bytes: Vec<u8>) -> bool {
        let mut value = RuntimeFontAssetValue::default();
        value.set_live_font_bytes(Some(Arc::from(bytes)));
        self.set_text_style_font_override(local_id, value)
    }

    pub(crate) fn mark_text_style_shape_dirty(&mut self, style_local_id: usize) -> bool {
        let Some(parent_key) = property_key_for_name("Component", "parentId") else {
            return false;
        };
        let Some(text_local) = self
            .uint_property(style_local_id, parent_key)
            .and_then(|parent_id| usize::try_from(parent_id).ok())
        else {
            return false;
        };
        if !matches!(
            self.slot(text_local).and_then(|slot| slot.type_name),
            Some("Text" | "TextInput")
        ) {
            return false;
        }

        self.add_dirt(style_local_id, ComponentDirt::TEXT_SHAPE, false)
            | crate::text_owner::mark_shape_dirty(self, text_local)
    }
}
