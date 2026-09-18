//! Dispatch Nuxie-owned records without changing upstream factory allocations.
use crate::source::{core::CoreObject, generated::core_registry::CoreRegistry};
use crate::{
    collection_semantics::SemanticCollectionData,
    video::{Video, VideoAsset},
};
pub(crate) fn make_core(type_key: i32) -> Option<Box<dyn CoreObject>> {
    match type_key {
        60000 => Some(Box::new(VideoAsset::default())),
        60001 => Some(Box::new(Video::default())),
        60002 => Some(Box::new(SemanticCollectionData::default())),
        _ => CoreRegistry::make_core_box(type_key),
    }
}
