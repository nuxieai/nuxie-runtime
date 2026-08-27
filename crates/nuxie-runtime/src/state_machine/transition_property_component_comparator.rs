//! Concrete component-property comparator identity.

use nuxie_binary::RuntimeObject;

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionPropertyComponentComparator {
    local_id: usize,
    property_key: u16,
}

impl RuntimeTransitionPropertyComponentComparator {
    pub(super) fn from_object(object: &RuntimeObject) -> Option<Self> {
        if object.type_name != "TransitionPropertyComponentComparator" {
            return None;
        }
        Some(Self {
            local_id: usize::try_from(object.uint_property("objectId")?).ok()?,
            property_key: u16::try_from(object.uint_property("propertyKey")?).ok()?,
        })
    }

    pub(super) fn local_id(self) -> usize {
        self.local_id
    }

    pub(super) fn property_key(self) -> u16 {
        self.property_key
    }
}
