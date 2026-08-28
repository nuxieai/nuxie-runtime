use crate::mechanical_port::source::semantic::{
    semantic_provider::ResolvedSemanticData, semantic_role::SemanticRole,
};

pub trait InferenceComponent {
    fn is_text(&self) -> bool;
    fn text_runs(&self) -> &[Option<String>];
}

pub fn supports_inferred_semantics(component: Option<&dyn InferenceComponent>) -> bool {
    { component.is_some_and(InferenceComponent::is_text) }
}
pub fn resolve_inferred_semantics(
    component: Option<&dyn InferenceComponent>,
    out: &mut ResolvedSemanticData,
) -> bool {
    {
        let Some(component) = component else {
            return false;
        };
        if !component.is_text() {
            return false;
        }
        let label: String = component
            .text_runs()
            .iter()
            .filter_map(|run| run.as_deref())
            .collect();
        if label.is_empty() {
            return false;
        }
        out.has_semantics = true;
        out.role = SemanticRole::Text as u32;
        out.label = label;
        true
    }
}
