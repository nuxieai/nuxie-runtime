use crate::mechanical_port::source::{
    core::CoreHandle, semantic::semantic_provider::ResolvedSemanticData,
};

pub fn supports_inferred_semantics(component: Option<&CoreHandle>) -> bool {
    component
        .and_then(|component| component.with(|component| component.semantic_provider_can_infer()))
        .unwrap_or(false)
}

pub fn resolve_inferred_semantics(
    component: Option<&CoreHandle>,
    out: &mut ResolvedSemanticData,
) -> bool {
    let Some(inferred) = component
        .and_then(|component| {
            component.with(|component| component.semantic_provider_inferred_data())
        })
        .flatten()
    else {
        return false;
    };
    *out = inferred;
    true
}
