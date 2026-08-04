use super::super::ArtboardInstance;
use crate::properties::property_key_for_name;

impl ArtboardInstance {
    pub(crate) fn text_variation_modifier_tag(&self, local_id: usize, authored_tag: u32) -> u32 {
        let shape_revision = self.text_shape_revision;
        let mut tags = self.text_variation_modifier_tags.borrow_mut();
        let (revision, tag) = tags.entry(local_id).or_insert_with(|| {
            (
                shape_revision,
                property_key_for_name("TextVariationModifier", "axisTag")
                    .and_then(|key| self.uint_property(local_id, key))
                    .and_then(|value| u32::try_from(value).ok())
                    .unwrap_or(authored_tag),
            )
        });
        if *revision != shape_revision {
            *tag = property_key_for_name("TextVariationModifier", "axisTag")
                .and_then(|key| self.uint_property(local_id, key))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(authored_tag);
            *revision = shape_revision;
        }
        *tag
    }
}
