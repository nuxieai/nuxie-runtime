use std::sync::Arc;

use crate::ArtboardInstance;
use crate::components::{ComponentDirt, ComponentHandle};
use crate::properties::property_key_for_name;
use crate::view_model::RuntimeFontAssetValue;

impl ArtboardInstance {
    /// Behavior-preserving owner delegate for the existing TextStyle dirty
    /// bridge. The current Rust topology only discovers the Text parent when
    /// a TextVariationHelper exists; the complete text wave must replace that
    /// compatibility limitation with C++'s retained `m_text` owner.
    pub(crate) fn dispatch_text_style_on_dirty(
        &mut self,
        handle: ComponentHandle,
        local_id: usize,
        accumulated: ComponentDirt,
    ) {
        if !accumulated.contains(ComponentDirt::TEXT_SHAPE) {
            return;
        }
        let Some(helper) = self.objects.text_variation_helper_handle(local_id) else {
            return;
        };
        let text = self
            .objects
            .component(handle)
            .and_then(|component| component.parent);
        if let Some(text) = text {
            self.add_component_dirt(text, ComponentDirt::TEXT_SHAPE, false);
        }
        self.add_component_dirt(helper, ComponentDirt::TEXT_SHAPE, false);
    }

    pub(crate) fn text_style_font_override(
        &self,
        local_id: usize,
    ) -> Option<&RuntimeFontAssetValue> {
        self.text_style_font_overrides.get(&local_id)
    }

    /// Behavior-preserving extraction of the live TextStyle font override
    /// facade. Pinned C++ keeps the selected `FontAsset` on TextStyle and
    /// dirties shaping when it changes (`src/text/text_style.cpp:138-154`).
    ///
    /// Rust still stores this compatibility value on Artboard; moving it onto
    /// a retained TextStyle owner is part of the complete text semantic wave.
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

    /// Behavior-preserving extraction of the existing Rust bridge from
    /// TextStyle metric callbacks to the retained Text owner. Pinned C++
    /// `fontSizeChanged`/`lineHeightChanged`/`letterSpacingChanged` call
    /// `Text::markShapeDirty`, while `TextStyle::onDirty` forwards TextShape to
    /// its Text and variation helper (`src/text/text_style.cpp:27-43,166-170`).
    ///
    /// The complete owner-local text dirt topology remains part of the mapped
    /// text/layout semantic closure.
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

        let mut changed = false;
        changed |= self.add_dirt(style_local_id, ComponentDirt::TEXT_SHAPE, false);
        changed |= self.add_dirt(text_local, ComponentDirt::TEXT_SHAPE, false);
        changed |= self.add_dirt(text_local, ComponentDirt::WORLD_TRANSFORM, true);
        changed
    }
}
