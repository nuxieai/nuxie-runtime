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
//! animated move are upstream's. One difference: upstream's focus reveal
//! moves nothing under clamped physics, whose `scrollToPosition` is the
//! empty default; a host reveal sets that scroller's offset at once.
use crate::mechanical_port::source::{
    constraints::scrolling::scroll_constraint::ScrollConstraint, focus_data::FocusData,
    math::vec2d::Vec2D, parent_traversal::ParentTraversal,
    semantic::semantic_node::SemanticNodeRef,
};

/// Scroll every scroller above `node` by the smallest move that shows it,
/// through nested scrollers and the artboards hosting it, as a focus
/// reveals itself. True when any scroller's effective offset (its target
/// while it settles) changed; false for a node already in view, one with
/// no scroller above it, and a boundary node, which has no `SemanticData`.
/// The caller checks the node may be revealed
/// (`SemanticNode::is_reveal_eligible`). With `animate` unset, for a player
/// whose time does not pass, every scroller moves at once, physics or not.
pub fn semantic_reveal(node: &SemanticNodeRef, animate: bool) -> bool {
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
    // A host move, so a scroller whose physics cannot animate to a target
    // (clamped) still moves, at once (`host_scroll_to_position`).
    let move_to: fn(&mut ScrollConstraint, Vec2D) = if animate {
        |constraint, target| {
            let _ = constraint.host_scroll_to_position(target, true);
        }
    } else {
        |constraint, target| {
            let _ = constraint.host_scroll_to_position(target, false);
        }
    };
    FocusData::scroll_into_view(parent, traversal, move_to)
}
