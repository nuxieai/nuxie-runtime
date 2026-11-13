//! PROBE (one shape for a Screen): the placed lift.
//!
//! While a matched element flies in place, each Screen's copy stays in its
//! own Screen's tree. Its own Screen's draw leaves it out (`skips`), and the
//! upper Screen draws it at its anchor's place in its own paint order: just
//! before the anchor, the upper Screen's own copy, so the upper copy stays
//! on top and everything the upper Screen paints under or over the anchor
//! stays under or over the flight. A copy that belongs to the other Screen
//! is drawn from that Screen's draw list, under that Screen's transform,
//! with none of its ancestors' clips: only its own entries and the clips
//! they open draw (`only`).
//!
//! Probe limits, each named in result.md: the anchor is the upper copy's own
//! box paint (its proxy), so an anchor inside a clip of the upper Screen
//! would draw the lifted copy inside that clip (today's `place` steps out of
//! such spans); the rows' relative transform is passed in, not read from the
//! Screen host; ancestors' opacity is not undone.
use crate::source::{core::CoreHandle, drawable::RuntimeDrawableOccurrence, math::mat2d::Mat2D};

/// A copy drawn by the upper Screen at its anchor.
#[derive(Clone)]
pub struct PlacedLift {
    /// The upper Screen's box the copy is drawn just before.
    pub anchor: CoreHandle,
    /// The Screen artboard that holds the copy.
    pub row: CoreHandle,
    /// The copy: a box in `row`.
    pub element: CoreHandle,
    /// From `row`'s drawing space to the upper Screen's, times the flight's
    /// pose, composed outside layout.
    pub transform: Mat2D,
}

/// Which entries a draw of a Screen keeps while copies are lifted.
#[derive(Clone, Default)]
pub(crate) struct LiftFilter {
    /// Only what draws for this box: the lifted draw of a copy.
    pub(crate) only: Option<CoreHandle>,
    /// Never what draws for these boxes: the Screen's own draw.
    pub(crate) skip: Vec<CoreHandle>,
}

impl LiftFilter {
    pub(crate) fn keeps(&self, entry: &RuntimeDrawableOccurrence) -> bool {
        let Some(node) = entry_node(entry) else {
            return self.only.is_none();
        };
        if let Some(only) = &self.only {
            return within(&node, only);
        }
        !self.skip.iter().any(|skip| within(&node, skip))
    }
}

/// Whether `entry` is `anchor`'s own paint, its box proxy (not a clip
/// close or another proxy the box owns).
pub(crate) fn is_anchor(entry: &RuntimeDrawableOccurrence, anchor: &CoreHandle) -> bool {
    let RuntimeDrawableOccurrence::RuntimeProxy(proxy) = entry else {
        return false;
    };
    anchor
        .with(|object| {
            object
                .as_layout_component()
                .and_then(|layout| layout.existing_proxy())
                .is_some_and(|own| std::rc::Rc::ptr_eq(own, proxy))
        })
        .unwrap_or(false)
}

fn entry_node(entry: &RuntimeDrawableOccurrence) -> Option<CoreHandle> {
    match entry {
        RuntimeDrawableOccurrence::Authored(handle) => Some(handle.clone()),
        RuntimeDrawableOccurrence::RuntimeProxy(proxy) => Some(proxy.borrow().owner_handle()),
    }
}

fn within(node: &CoreHandle, element: &CoreHandle) -> bool {
    let mut current = Some(node.clone());
    while let Some(at) = current {
        if &at == element {
            return true;
        }
        current = at.with(|object| object.component_parent_handle()).flatten();
    }
    false
}
