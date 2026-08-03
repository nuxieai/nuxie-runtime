// Pinned C++ correspondence (4ac7b327):
// src/semantic/semantic_provider.cpp:1-148 and
// include/rive/semantic/semantic_provider.hpp:1-36.

use crate::ArtboardInstance;
use crate::components::Mat2D;
use crate::semantic_data::{SemanticBounds, SemanticNodeHandle};
use crate::semantic_inference_registry::{resolve_inferred_semantics, supports_inferred_semantics};
use nuxie_schema::definition_by_name;

fn component_is_a(artboard: &ArtboardInstance, component_local_id: usize, base_type: &str) -> bool {
    artboard
        .runtime_object_type_name(component_local_id)
        .and_then(definition_by_name)
        .is_some_and(|definition| definition.is_a(base_type))
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedSemanticData {
    pub has_semantics: bool,
    pub role: u32,
    pub label: String,
}

pub struct SemanticProvider;

impl SemanticProvider {
    pub fn can_infer_semantics(artboard: &ArtboardInstance, component_local_id: usize) -> bool {
        supports_inferred_semantics(artboard, component_local_id)
    }

    pub fn resolve_semantic_data(
        artboard: &ArtboardInstance,
        component_local_id: usize,
    ) -> ResolvedSemanticData {
        let mut out = ResolvedSemanticData::default();
        if !component_is_a(artboard, component_local_id, "Node") {
            return out;
        }
        let Some(component) = artboard.component(component_local_id) else {
            return out;
        };
        for child in &component.children {
            let Some(child_local) = artboard.component_local_id(*child) else {
                continue;
            };
            if artboard.runtime_object_type_name(child_local) != Some("SemanticData") {
                continue;
            }
            out.has_semantics = true;
            out.role = semantic_uint_property(artboard, child_local, "role");
            out.label = semantic_string_property(artboard, child_local, "label");
            return out;
        }
        resolve_inferred_semantics(artboard, component_local_id, &mut out);
        out
    }

    /// Compute root-space semantic bounds using the live geometry owner.
    pub fn semantic_bounds(
        artboard: &mut ArtboardInstance,
        component_local_id: usize,
    ) -> SemanticBounds {
        Self::semantic_bounds_with_root_transform(artboard, component_local_id, Mat2D::IDENTITY)
    }

    /// Compute semantic bounds in the outermost artboard's coordinate space.
    ///
    /// Pinned C++ maps all four corners through `Artboard::rootTransform`, so
    /// nested rotation and shear must be re-enclosed rather than represented
    /// by translating only the local bounds origin.
    pub(crate) fn semantic_bounds_with_root_transform(
        artboard: &mut ArtboardInstance,
        component_local_id: usize,
        root_transform: Mat2D,
    ) -> SemanticBounds {
        if !component_is_a(artboard, component_local_id, "Node") {
            return SemanticBounds::default();
        }
        // C++ `SemanticProvider::semanticBounds` first asks the target Node
        // for its own `localBounds` and maps that box through its world/root
        // transforms. A LayoutComponent owns its solved border box even when
        // it has no drawable path, so do not replace that box with a merge of
        // inset visual descendants.
        if let Some((min_x, min_y, max_x, max_y)) = artboard
            .semantic_layout_world_bounds(component_local_id)
            .or_else(|| artboard.layout_world_bounds(component_local_id))
        {
            return root_transform_bounds(
                root_transform,
                SemanticBounds::new(min_x, min_y, max_x, max_y),
            );
        }
        if let Some(bounds) = artboard.object_world_bounds(component_local_id) {
            return root_transform_bounds(
                root_transform,
                SemanticBounds::new(bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y),
            );
        }

        // Pinned containers merge all visual descendants. The retained arena
        // already stores the complete child topology, so walk it in authored
        // order and merge every live geometry owner.
        if component_is_a(artboard, component_local_id, "ContainerComponent") {
            let mut merged = SemanticBounds::for_expansion();
            let mut has_descendant_bounds = false;
            let mut stack = artboard
                .component(component_local_id)
                .map(|component| component.children.clone())
                .unwrap_or_default();
            while let Some(child) = stack.pop() {
                let Some(child_local) = artboard.component_local_id(child) else {
                    continue;
                };
                if let Some(component) = artboard.component(child_local) {
                    stack.extend(component.children.iter().rev().copied());
                }
                if !component_is_a(artboard, child_local, "Node") {
                    continue;
                }
                if let Some(bounds) = artboard.object_world_bounds(child_local) {
                    merged.expand(SemanticBounds::new(
                        bounds.min_x,
                        bounds.min_y,
                        bounds.max_x,
                        bounds.max_y,
                    ));
                    has_descendant_bounds = true;
                }
            }
            if has_descendant_bounds {
                return root_transform_bounds(root_transform, merged);
            }
        }

        let Some(transform) = artboard.object_world_transform(component_local_id) else {
            return SemanticBounds::default();
        };
        let (x, y) = root_transform.transform_point(transform.0[4], transform.0[5]);
        SemanticBounds::new(x, y, x, y)
    }

    pub(crate) fn direct_semantic_data_child(
        artboard: &ArtboardInstance,
        component_local_id: usize,
    ) -> Option<usize> {
        artboard
            .component(component_local_id)?
            .children
            .iter()
            .filter_map(|child| artboard.component_local_id(*child))
            .find(|local| artboard.runtime_object_type_name(*local) == Some("SemanticData"))
    }

    pub(crate) fn closest_parent_semantic_node(
        artboard: &ArtboardInstance,
        mut component_local_id: usize,
        nodes_by_data_local: &std::collections::BTreeMap<usize, SemanticNodeHandle>,
    ) -> Option<SemanticNodeHandle> {
        loop {
            let parent = artboard.component_parent_local(component_local_id)?;
            if let Some(data_local) = Self::direct_semantic_data_child(artboard, parent)
                && let Some(node) = nodes_by_data_local.get(&data_local)
            {
                return Some(node.clone());
            }
            component_local_id = parent;
        }
    }
}

fn root_transform_bounds(transform: Mat2D, bounds: SemanticBounds) -> SemanticBounds {
    let corners = [
        transform.transform_point(bounds.min_x, bounds.min_y),
        transform.transform_point(bounds.max_x, bounds.min_y),
        transform.transform_point(bounds.max_x, bounds.max_y),
        transform.transform_point(bounds.min_x, bounds.max_y),
    ];
    let (mut min_x, mut min_y) = corners[0];
    let (mut max_x, mut max_y) = corners[0];
    for (x, y) in corners.into_iter().skip(1) {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    SemanticBounds::new(min_x, min_y, max_x, max_y)
}

pub(crate) fn semantic_uint_property(
    artboard: &ArtboardInstance,
    local_id: usize,
    name: &str,
) -> u32 {
    crate::properties::property_key_for_name("SemanticData", name)
        .and_then(|key| artboard.uint_property(local_id, key))
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(0)
}

pub(crate) fn semantic_string_property(
    artboard: &ArtboardInstance,
    local_id: usize,
    name: &str,
) -> String {
    crate::properties::property_key_for_name("SemanticData", name)
        .and_then(|key| artboard.string_property(local_id, key))
        .map(String::from_utf8_lossy)
        .map(std::borrow::Cow::into_owned)
        .unwrap_or_default()
}
