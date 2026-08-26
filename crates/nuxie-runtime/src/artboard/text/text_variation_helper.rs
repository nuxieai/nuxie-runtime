use super::super::ArtboardInstance;
use crate::properties::property_key_for_name;

impl ArtboardInstance {
    pub(crate) fn text_variation_modifier_tag(&self, local_id: usize, authored_tag: u32) -> u32 {
        // Pinned `TextVariationModifier::modify` calls the generated getter on
        // every invocation. `axisTagChanged` remains intentionally empty, but
        // the next independently requested reshape must observe the live tag.
        property_key_for_name("TextVariationModifier", "axisTag")
            .and_then(|key| self.uint_property(local_id, key))
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(authored_tag)
    }
}
