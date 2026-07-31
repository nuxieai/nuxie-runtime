use crate::{ArtboardInstance, ComponentDirt, properties::property_key_for_name};

pub(crate) fn bool_property_changed(
    artboard: &mut ArtboardInstance,
    _local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("ClippingShape", "isVisible") != Some(property_key) {
        return None;
    }
    Some(artboard.add_dirt(0, ComponentDirt::CLIPPING, false))
}
