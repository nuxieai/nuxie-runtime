// Pinned C++ correspondence (4ac7b327):
// src/semantic/semantic_inference_registry.cpp:1-97.

use crate::ArtboardInstance;
use crate::semantic_data::SemanticRole;

use super::semantic_provider::ResolvedSemanticData;

pub(crate) fn supports_inferred_semantics(
    artboard: &ArtboardInstance,
    component_local_id: usize,
) -> bool {
    artboard.runtime_object_type_name(component_local_id) == Some("Text")
}

pub(crate) fn resolve_inferred_semantics(
    artboard: &ArtboardInstance,
    component_local_id: usize,
    out: &mut ResolvedSemanticData,
) -> bool {
    if !supports_inferred_semantics(artboard, component_local_id) {
        return false;
    }
    let Some(component) = artboard.component(component_local_id) else {
        return false;
    };
    let mut label = String::new();
    for child in &component.children {
        let Some(child_local) = artboard.component_local_id(*child) else {
            continue;
        };
        if artboard.runtime_object_type_name(child_local) != Some("TextValueRun") {
            continue;
        }
        let Some(key) = crate::properties::property_key_for_name("TextValueRun", "text") else {
            continue;
        };
        let Some(text) = artboard.string_property(child_local, key) else {
            continue;
        };
        label.push_str(&String::from_utf8_lossy(text));
    }
    for (text, _) in artboard.text_list_runs(component_local_id) {
        label.push_str(&String::from_utf8_lossy(&text));
    }
    if label.is_empty() {
        return false;
    }
    out.has_semantics = true;
    out.role = SemanticRole::Text as u32;
    out.label = label;
    true
}
