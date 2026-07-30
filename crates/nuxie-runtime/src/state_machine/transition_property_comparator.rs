//! Property transition-comparator definitions.
//!
//! Mirrors pinned C++ `src/animation/transition_property_comparator.cpp` and
//! retains the concrete component/artboard property identity consumed by
//! `TransitionViewModelCondition::initialize`.

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
