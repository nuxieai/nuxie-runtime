//! Direct Rust home for `include/rive/text/text_variation_helper.hpp` and
//! `src/text/text_variation_helper.cpp`.
//!
//! Artboard retains authored-order orchestration, while this module owns the
//! helper's occurrence attachment and concrete update callback.

use anyhow::{Context, Result};
use nuxie_graph::ArtboardGraph;

use crate::ArtboardInstance;
use crate::components::{ComponentDirt, ComponentHandle, RuntimeComponent};
use crate::objects::InstanceObjectArena;

/// Attach every TextVariationHelper at its owning TextStyle's authored
/// `onAddedClean` point. The caller preserves the surrounding Component order;
/// this delegate owns the concrete helper allocation/relink operation
/// (`src/text/text_style.cpp:45-70`).
pub(crate) fn attach_occurrences(
    objects: &mut InstanceObjectArena,
    graph: &ArtboardGraph,
    root: ComponentHandle,
) -> Result<()> {
    for component in &graph.components {
        let Some(helper) = graph
            .text_variation_helpers
            .iter()
            .find(|helper| helper.text_style_local == component.local_id)
        else {
            continue;
        };
        let handle = if objects
            .text_variation_helper_handle(helper.text_style_local)
            .is_some()
        {
            objects
                .relink_text_variation_helper_owner(helper.text_style_local)
                .context("TextVariationHelper cannot retain its rebuilt TextStyle parent")?
        } else {
            objects
                .attach_text_variation_helper(
                    helper.text_style_local,
                    RuntimeComponent::embedded(
                        helper.text_style_local,
                        helper.text_style_global,
                        "TextVariationHelper",
                    ),
                )
                .context("TextStyle cannot own its TextVariationHelper")?
        };
        if !objects.link_parent(handle, root) {
            anyhow::bail!("TextVariationHelper parent link could not be retained");
        }
    }
    Ok(())
}

pub(crate) fn update(instance: &mut ArtboardInstance, text: ComponentHandle, dirt: ComponentDirt) {
    if !dirt.contains(ComponentDirt::TEXT_SHAPE) {
        return;
    }
    if let Some(text_local) = instance.component_local_id(text) {
        // C++ rebuilds the variation-bearing Font on the helper update
        // (`src/text/text_variation_helper.cpp:14-17`,
        // `src/text/text_style.cpp:98-124`). Rust's retained text owner
        // rebuilds lazily from the same live axis values, so invalidate
        // precisely that Text occurrence here.
        instance
            .runtime_drawables
            .mark_text_resource_dirty_for_local(text_local);
    }
}
