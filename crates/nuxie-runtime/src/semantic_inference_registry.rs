// Pinned C++ correspondence (4ac7b327):
// src/semantic/semantic_inference_registry.cpp:1-97.

use crate::ArtboardInstance;
use crate::semantic_data::SemanticRole;
use nuxie_schema::definition_by_name;

use super::semantic_provider::ResolvedSemanticData;

type InferSemanticFn = fn(&ArtboardInstance, usize, &mut ResolvedSemanticData) -> bool;

struct InferenceRule {
    // Rust's generated schema ancestry is the arena equivalent of the C++
    // `typeKey` passed to `Component::isTypeOf`.
    type_name: &'static str,
    infer: InferSemanticFn,
}

fn inferred_text_label(artboard: &ArtboardInstance, component_local_id: usize) -> String {
    let Some(component) = artboard.component(component_local_id) else {
        return String::new();
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
    for run in artboard.text_list_runs(component_local_id) {
        if let Some(text) = run.text {
            label.push_str(&String::from_utf8_lossy(&text));
        }
    }
    label
}

fn infer_text_semantics(
    artboard: &ArtboardInstance,
    component_local_id: usize,
    out: &mut ResolvedSemanticData,
) -> bool {
    let label = inferred_text_label(artboard, component_local_id);
    if label.is_empty() {
        return false;
    }

    out.has_semantics = true;
    out.role = SemanticRole::Text as u32;
    out.label = label;
    true
}

const INFERENCE_RULES: &[InferenceRule] = &[InferenceRule {
    type_name: "Text",
    infer: infer_text_semantics,
}];

fn component_is_type_of(
    artboard: &ArtboardInstance,
    component_local_id: usize,
    type_name: &str,
) -> bool {
    artboard.component(component_local_id).is_some()
        && artboard
            .runtime_object_type_name(component_local_id)
            .and_then(definition_by_name)
            .is_some_and(|definition| definition.is_a(type_name))
}

pub(crate) fn supports_inferred_semantics(
    artboard: &ArtboardInstance,
    component_local_id: usize,
) -> bool {
    INFERENCE_RULES
        .iter()
        .any(|rule| component_is_type_of(artboard, component_local_id, rule.type_name))
}

pub(crate) fn resolve_inferred_semantics(
    artboard: &ArtboardInstance,
    component_local_id: usize,
    out: &mut ResolvedSemanticData,
) -> bool {
    INFERENCE_RULES.iter().any(|rule| {
        component_is_type_of(artboard, component_local_id, rule.type_name)
            && (rule.infer)(artboard, component_local_id, out)
    })
}
