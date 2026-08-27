//! Concrete Artboard-property comparator identity.

use nuxie_binary::RuntimeObject;

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionPropertyArtboardComparator {
    property_type: u64,
}

impl RuntimeTransitionPropertyArtboardComparator {
    pub(super) fn from_object(object: &RuntimeObject) -> Option<Self> {
        if object.type_name != "TransitionPropertyArtboardComparator" {
            return None;
        }
        Some(Self {
            property_type: object.uint_property("propertyType").unwrap_or(0),
        })
    }

    pub(super) fn property_type(self) -> u64 {
        self.property_type
    }
}
