use crate::ArtboardInstance;
use crate::components::ComponentDirt;
use crate::properties::property_key_for_name;

impl ArtboardInstance {
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
