//! The layout hold of an element flying in place (docs/live-moves.md, Flying
//! in place).
//!
//! While a matched element flies in place in Layout mode, its parent keeps
//! seeing the element as it rests, so its neighbors don't move, and its
//! inside is laid out at the flight size each frame. The element stays its
//! parent's own Taffy child with its own style: the parent pass lays it out
//! exactly as at rest. A flight pass then lays the element out again, alone,
//! as its parent last asked but at the flight size
//! (`TaffyTree::relayout_at_size`, vendor/taffy-0.15.0-nuxie-css/NUXIE_PATCH.md).
//! That pass writes only the element's descendants; the element's own Taffy
//! layout stays the place its parent gave it, its resting slot.
//!
//! Two readers turn that into what is published, and the layout checker
//! reads the same two: `resting_slot` (the element's place and size in its
//! parent) and `flight_layout` (that place at the flight's size and
//! content). No Rive object and no Taffy node is added: Taffy is driven
//! differently only in the flight pass of a held element.
//!
//! A held box that carries a component (`carry`) lays that component's root
//! out in the same pass, at the blend of its sizes at the two ends. Taffy
//! takes a child's size from its style, so the pass gives the root's Taffy
//! style that size, with no minimum or maximum to floor it, lets the box
//! place it by its own rules, and puts the style back. No Rive object sees
//! it, and the root itself is not held, so it is nowhere held on once its
//! box's hold ends. While it is carried, and in the solve after, it takes its
//! layout at once, as its box does: the flight drives it.
//!
//! The element is a layout node's owner: a box, or a participant (a text,
//! an image or a video). Its rest follows its parent through the step: a
//! percentage or stretched size follows a parent resize, and its neighbors
//! move as they would at rest when its content changes. While held, and in
//! the frame the hold ends, the element's own box does not interpolate: the
//! flight drives it.
use crate::mechanical_port::source::{
    core::CoreHandle,
    layout_component::{Layout, LayoutComponent},
};
use taffy::TraversePartialTree;
use taffy::prelude::{BoxSizing, Dimension, LengthPercentageAuto, NodeId, Size, Style, TaffyTree};
use taffy::tree::{LayoutInput, LayoutOutput};

/// How a held element flies this frame.
#[derive(Clone, Debug)]
pub(crate) struct Flying {
    /// The element's border box while held: what a flight's ends measure,
    /// whatever the element's box-sizing.
    size: (f32, f32),
    /// The component it carries.
    carried: Option<CarriedRoot>,
}

/// The root of the component a held box carries.
#[derive(Clone, Debug)]
struct CarriedRoot {
    root: CoreHandle,
    /// Its border box less its box's at the end it left, as it took off. At
    /// the arriving end both rest in the tree, where each solve reads them.
    left: (f32, f32),
    progress: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct FlightHold {
    flying: Flying,
    /// The element's resting slot, relative to its layout parent, as the
    /// last solve published it: its own published layout, were it at rest.
    pub(crate) rest: Layout,
}

impl Flying {
    pub(crate) fn carried_root(&self) -> Option<&CoreHandle> {
        self.carried.as_ref().map(|carried| &carried.root)
    }
}

impl FlightHold {
    pub(crate) fn flying(&self) -> &Flying {
        &self.flying
    }
}

/// The element's resting slot: its place and size among its parent's
/// children, as the parent pass laid it out. The flight pass leaves it as
/// it is.
pub(crate) fn resting_slot<T>(taffy: &TaffyTree<T>, node: NodeId) -> Option<taffy::Layout> {
    taffy.layout(node).ok().copied()
}

/// The element's flight layout: its resting slot's place, at the size and
/// with the content the flight pass gave it.
pub(crate) fn flight_layout(slot: &taffy::Layout, flight: &LayoutOutput) -> taffy::Layout {
    taffy::Layout {
        size: flight.size,
        scrollable_overflow_rect: flight.scrollable_overflow_rect,
        ..*slot
    }
}

/// The flight pass of one held element: lay its inside out with its border
/// box at its flight size, as its parent last asked, and the root it carries
/// at its own. None before the element's first full layout. `owner` is what
/// a node's context says it lays out.
pub(crate) fn fly<T, M>(
    taffy: &mut TaffyTree<T>,
    node: NodeId,
    flying: &Flying,
    owner: impl Fn(&T) -> &CoreHandle,
    measure: M,
) -> Option<LayoutOutput>
where
    M: FnMut(LayoutInput, NodeId, Option<&mut T>, &Style) -> LayoutOutput,
{
    let slot = resting_slot(taffy, node)?;
    let (width, height) = flying.size;
    let root = flying.carried.as_ref().and_then(|carried| {
        let root = taffy.child_ids(node).find(|&child| {
            taffy
                .get_node_context(child)
                .is_some_and(|context| owner(context) == &carried.root)
        })?;
        // The parent pass has just laid both out at rest.
        let rest = resting_slot(taffy, root)?.size;
        let blend = |box_size: f32, left: f32, arriving: f32| {
            (box_size + left + (arriving - left) * carried.progress).max(0.0)
        };
        Some((
            root,
            Size {
                width: blend(width, carried.left.0, rest.width - slot.size.width),
                height: blend(height, carried.left.1, rest.height - slot.size.height),
            },
        ))
    });
    let size = Size { width, height };
    match root {
        Some((root, root_size)) => Resized::new(taffy, node, root, root_size)
            .taffy
            .relayout_at_size(node, size, measure),
        None => taffy.relayout_at_size(node, size, measure),
    }
}

/// A carried root's Taffy style, sized for the flight pass of the box it is
/// in; its own style is put back when this is dropped, however that is.
struct Resized<'a, T> {
    taffy: &'a mut TaffyTree<T>,
    element: NodeId,
    root: NodeId,
    style: Option<Style>,
}

impl<'a, T> Resized<'a, T> {
    /// `size` is the root's border box. Its place stays its box's to give:
    /// its position, insets, margins and alignment are as they were.
    fn new(taffy: &'a mut TaffyTree<T>, element: NodeId, root: NodeId, size: Size<f32>) -> Self {
        let style = taffy.style(root).ok().cloned();
        if let Some(style) = &style {
            let mut flying = style.clone();
            flying.box_sizing = BoxSizing::BorderBox;
            flying.size = size.map(Dimension::length);
            flying.min_size = Size {
                width: LengthPercentageAuto::length(0.0),
                height: LengthPercentageAuto::length(0.0),
            };
            flying.max_size = Size {
                width: LengthPercentageAuto::auto(),
                height: LengthPercentageAuto::auto(),
            };
            flying.aspect_ratio = None;
            flying.flex_basis = Dimension::auto();
            flying.flex_grow = 0.0;
            flying.flex_shrink = 0.0;
            let _ = taffy.set_style(root, flying);
        }
        Self {
            taffy,
            element,
            root,
            style,
        }
    }
}

impl<T> Drop for Resized<'_, T> {
    fn drop(&mut self) {
        if let Some(style) = self.style.take() {
            let _ = self.taffy.set_style(self.root, style);
        }
        // What the pass cached answers for the flight's style: the next
        // parent pass lays the box's inside out at rest again.
        let _ = self.taffy.mark_dirty(self.element);
    }
}

/// Whether `begin` would hold `element`: it is laid out by a parent, has
/// been laid out at a finite size, and is not held. Reads only.
pub(crate) fn can_begin(element: &CoreHandle) -> bool {
    element
        .with_mut(|object| {
            LayoutComponent::layout_parent_of(object).filter(|parent| parent != element)?;
            let data = LayoutComponent::layout_data_of(object)?;
            let rest = data.solved_layout;
            (data.flight_hold.is_none()
                && data.node().is_some()
                && rest.width().is_finite()
                && rest.height().is_finite())
            .then_some(())
        })
        .flatten()
        .is_some()
}

/// Starts holding `element`'s place in its parent's layout; from the next
/// solve its inside is laid out with its border box at `size`. False, and nothing held, for an element that is not
/// laid out by a parent, has not been laid out, or is already held.
pub(crate) fn begin(element: &CoreHandle, size: (f32, f32)) -> bool {
    let begun = element
        .with_mut(|object| {
            LayoutComponent::layout_parent_of(object).filter(|parent| parent != element)?;
            let data = LayoutComponent::layout_data_of(object)?;
            if data.flight_hold.is_some() || data.node().is_none() {
                return None;
            }
            let rest = data.solved_layout;
            if !(rest.width().is_finite() && rest.height().is_finite()) {
                return None;
            }
            data.flight_hold = Some(Box::new(FlightHold {
                flying: Flying {
                    size,
                    carried: None,
                },
                rest,
            }));
            data.request_solve();
            Some(())
        })
        .flatten()
        .is_some();
    if !begun {
        return false;
    }
    #[cfg(any(test, feature = "test-support"))]
    HOLDS_BEGUN.with(|count| count.set(count.get() + 1));
    LayoutComponent::request_layout_solve(element);
    true
}

/// Lays the held element's inside out at `size` from the next solve. The
/// parent's layout is not invalidated: it doesn't read the flight.
pub(crate) fn set_size(element: &CoreHandle, size: (f32, f32)) {
    change(element, |flying| {
        if flying.size == size {
            return false;
        }
        flying.size = size;
        true
    });
}

/// From the next solve, the held box `element` lays `root`, the root of the
/// component it holds, out with it: at the box's flight size plus the blend,
/// at the flight's progress (`set_progress`), of the root's border box less
/// its box's at each end, `left` at the end it left. Nothing when it isn't
/// held.
pub(crate) fn carry(element: &CoreHandle, root: &CoreHandle, left: (f32, f32)) {
    change(element, |flying| {
        flying.carried = Some(CarriedRoot {
            root: root.clone(),
            left,
            progress: 0.0,
        });
        true
    });
}

/// How far along a carried root's blend is (`carry`).
pub(crate) fn set_progress(element: &CoreHandle, progress: f32) {
    change(element, |flying| match &mut flying.carried {
        Some(carried) if carried.progress != progress => {
            carried.progress = progress;
            true
        }
        _ => false,
    });
}

/// Changes how a held `element` flies, and asks for a solve when that says
/// it did.
fn change(element: &CoreHandle, change: impl FnOnce(&mut Flying) -> bool) {
    let changed = element
        .with_mut(|object| {
            let data = LayoutComponent::layout_data_of(object)?;
            if !change(&mut data.flight_hold.as_mut()?.flying) {
                return None;
            }
            data.request_solve();
            Some(())
        })
        .flatten();
    if changed.is_some() {
        LayoutComponent::request_layout_solve(element);
    }
}

/// Ends the hold: the next solve lays the element's inside out at rest
/// again, and its own box takes its resting layout at once, as does the root
/// it carried.
pub(crate) fn end(element: &CoreHandle) {
    let ended = element
        .with_mut(|object| {
            let data = LayoutComponent::layout_data_of(object)?;
            let hold = data.flight_hold.take()?;
            data.land();
            Some(hold.flying)
        })
        .flatten();
    let Some(flying) = ended else {
        return;
    };
    LayoutComponent::request_layout_solve(element);
    if let Some(root) = flying.carried_root() {
        root.with_mut(|object| {
            if let Some(data) = LayoutComponent::layout_data_of(object) {
                data.land();
            }
        });
        LayoutComponent::request_layout_solve(root);
    }
}

/// Where `element` rests, relative to its layout parent, as layout last
/// solved it: while held, its resting slot; otherwise its own layout. The
/// one reader of a resting box that a flight's size, its placement and a
/// zoom's ends read.
pub(crate) fn resting_box(element: &CoreHandle) -> Option<Layout> {
    element
        .with_mut(|object| {
            let data = LayoutComponent::layout_data_of(object)?;
            Some(
                data.flight_hold
                    .as_ref()
                    .map_or(data.solved_layout, |hold| hold.rest),
            )
        })
        .flatten()
}

/// Where a held `element` rests, relative to its layout parent; None while
/// it isn't held, when its own layout is its rest.
pub(crate) fn held_rest(element: &CoreHandle) -> Option<Layout> {
    element
        .with_mut(|object| {
            let data = LayoutComponent::layout_data_of(object)?;
            data.flight_hold.as_ref().map(|hold| hold.rest)
        })
        .flatten()
}

#[cfg(any(test, feature = "test-support"))]
thread_local! {
    static HOLDS_BEGUN: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The holds begun on this thread.
#[cfg(any(test, feature = "test-support"))]
pub fn holds_begun() -> usize {
    HOLDS_BEGUN.with(std::cell::Cell::get)
}

#[cfg(test)]
mod tests {
    use super::*;
    use taffy::prelude::TaffyMaxContent;

    /// A carried root's own style comes back whether its box's flight pass
    /// finishes or panics, and the next parent pass lays both out at rest.
    #[test]
    fn a_resized_root_gets_its_style_back_on_every_path() {
        let length = |width, height| Size {
            width: Dimension::length(width),
            height: Dimension::length(height),
        };
        for panics in [false, true] {
            let mut taffy = TaffyTree::<()>::new();
            // As a layout tree does: the pass writes unrounded layouts.
            taffy.disable_rounding();
            let root = taffy
                .new_leaf(Style {
                    size: length(100.0, 80.0),
                    min_size: Size {
                        width: LengthPercentageAuto::length(100.0),
                        height: LengthPercentageAuto::length(80.0),
                    },
                    flex_shrink: 0.5,
                    ..Default::default()
                })
                .unwrap();
            let element = taffy
                .new_with_children(
                    Style {
                        size: length(200.0, 140.0),
                        ..Default::default()
                    },
                    &[root],
                )
                .unwrap();
            taffy.compute_layout(element, Size::MAX_CONTENT).unwrap();
            let (style, rest) = (
                taffy.style(root).unwrap().clone(),
                *taffy.layout(root).unwrap(),
            );
            let size = Size {
                width: 60.0,
                height: 40.0,
            };
            let flown = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let resized = Resized::new(&mut taffy, element, root, size);
                assert_ne!(resized.taffy.style(root).unwrap(), &style);
                resized.taffy.relayout_at_size(
                    element,
                    Size {
                        width: 120.0,
                        height: 90.0,
                    },
                    |inputs, _, _, style| {
                        assert!(!panics, "the pass stops half way");
                        taffy::compute_leaf_layout(inputs, style, |_, _| 0.0, |_, _| Size::ZERO)
                    },
                )
            }));
            assert_eq!(flown.is_err(), panics);
            if !panics {
                assert_eq!(taffy.layout(root).unwrap().size, size);
            }
            assert_eq!(taffy.style(root).unwrap(), &style);
            taffy.compute_layout(element, Size::MAX_CONTENT).unwrap();
            assert_eq!(*taffy.layout(root).unwrap(), rest);
        }
    }
}
