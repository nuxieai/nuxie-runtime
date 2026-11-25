//! Nuxie: bring a semantic node into view, for a host whose screen reader
//! moved to a node that is scrolled out of view.
//!
//! Upstream reveals a node only by focusing it: `FocusData::focused` first
//! scrolls the focus into view (`FocusData::scroll_into_view`), walking
//! every `ScrollConstraint` above it, across the artboards hosting it, and
//! asking each for the smallest move that shows it. A node needs a
//! `FocusData` for that, and engine focus is keyboard focus. Our CSS
//! content writes a `SemanticData` for each read box and no `FocusData`,
//! and a screen reader's focus must not move keyboard focus, so a reveal
//! here starts the same walk from the node's `SemanticData` instead, as
//! `FocusData::focused_occurrence` starts it from a focus. Nothing is
//! focused and no listener hears it; the geometry, the snapping and the
//! animated move are upstream's.
use crate::mechanical_port::source::{
    focus_data::FocusData, parent_traversal::ParentTraversal,
    semantic::semantic_node::SemanticNodeRef,
};

/// Scroll every scroller above `node` by the smallest move that shows it,
/// through nested scrollers and the artboards hosting it, as a focus
/// reveals itself. True when any scroller's effective offset (its target
/// while it settles) changed; false for a node already in view, one with
/// no scroller above it, and a boundary node, which has no `SemanticData`.
/// The caller checks the node may be revealed
/// (`SemanticNode::is_reveal_eligible`).
pub fn semantic_reveal(node: &SemanticNodeRef) -> bool {
    let Some(data) = node.borrow().semantic_data.clone() else {
        return false;
    };
    let Some((parent, traversal)) = data
        .with(|object| {
            let component = object.as_component()?;
            Some((
                component.parent_handle(),
                ParentTraversal::from_component(component),
            ))
        })
        .flatten()
    else {
        return false;
    };
    // The data's arena borrow is released before any content moves, as
    // `focused_occurrence` releases its focus's.
    FocusData::scroll_into_view(parent, traversal)
}
