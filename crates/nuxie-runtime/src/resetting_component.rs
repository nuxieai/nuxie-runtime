use nuxie_graph::{ArtboardGraph, ResettingComponentKind};

use super::{ArtboardInstance, reset_component_list_instances};
use crate::components::ComponentHandle;
use crate::objects::InstanceObjectArena;
use crate::properties::property_key_for_name;

/// Occurrence-local counterpart of C++ `ResettingComponent*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeResettingComponent {
    pub(crate) local_id: usize,
    pub(crate) component: ComponentHandle,
    pub(crate) kind: ResettingComponentKind,
}

pub(super) fn build_runtime_resetting_components(
    objects: &InstanceObjectArena,
    graph: &ArtboardGraph,
) -> Vec<RuntimeResettingComponent> {
    graph
        .resetting_components
        .iter()
        .filter_map(|entry| {
            Some(RuntimeResettingComponent {
                local_id: entry.local_id,
                component: objects.component_handle(entry.local_id)?,
                kind: entry.kind,
            })
        })
        .collect()
}

impl ArtboardInstance {
    pub(super) fn reset_retained_components(&mut self) {
        if self.resetting_components.is_empty() {
            return;
        }
        for index in 0..self.resetting_components.len() {
            let entry = self.resetting_components[index];
            match entry.kind {
                ResettingComponentKind::NestedArtboard => {
                    let Some(nested) = self.nested_artboards.get_mut(&entry.local_id) else {
                        continue;
                    };
                    nested.child.reset_retained_components();
                    if let Some(context) = nested.stateful_view_model_context.as_mut() {
                        context.advanced_data_context();
                    }
                }
                ResettingComponentKind::ArtboardComponentList => {
                    let should_reset_instances =
                        self.artboard_component_list_should_reset_instances(entry.local_id);
                    let Some(list) = self.component_list_state_mut(entry.local_id) else {
                        continue;
                    };
                    reset_component_list_instances(list, should_reset_instances);
                }
                ResettingComponentKind::CustomPropertyTrigger => {
                    let Some(property_value_key) =
                        property_key_for_name("CustomPropertyTrigger", "propertyValue")
                    else {
                        continue;
                    };
                    let _ = self.set_uint_property(entry.local_id, property_value_key, 0);
                }
            }
        }
    }
}
