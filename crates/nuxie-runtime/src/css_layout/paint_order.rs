//! CSS painting order within stacking contexts (docs/CSS_LAYOUT.md, 4.4).
//! Non-clipping CSS boxes let positioned descendants and nested contexts
//! join their nearest context. Clips and non-CSS hosts stay atomic, keeping
//! Rive's clip save and restore brackets and translated drawing order.
use super::keys;
use crate::source::{
    core::CoreHandle, drawable::RuntimeDrawableOccurrence,
    generated::foreground_layout_drawable_base::ForegroundLayoutDrawableBase,
};
use std::{collections::HashMap, rc::Rc};

/// The Rive properties that author a CSS transform.
pub(crate) fn is_transform_property(key: u32) -> bool {
    use crate::source::generated::{
        node_base::NodeBase, transform_component_base::TransformComponentBase,
    };
    [
        NodeBase::X_PROPERTY_KEY,
        NodeBase::Y_PROPERTY_KEY,
        TransformComponentBase::ROTATION_PROPERTY_KEY,
        TransformComponentBase::SCALE_X_PROPERTY_KEY,
        TransformComponentBase::SCALE_Y_PROPERTY_KEY,
    ]
    .into_iter()
    .any(|property| u32::from(property) == key)
}

/// How a container orders its children.
#[derive(Clone, Copy)]
pub(crate) struct Container {
    /// A CSS box or CSS artboard: CSS painting order.
    pub css: bool,
    /// Its children are flex or grid items, whose z-index applies even
    /// when they are not positioned.
    pub flex_or_grid: bool,
}

/// The component a drawable draws for: a proxy draws its owner's paint.
fn owner_of(drawable: &RuntimeDrawableOccurrence) -> Option<CoreHandle> {
    match drawable {
        RuntimeDrawableOccurrence::Authored(handle) => Some(handle.clone()),
        RuntimeDrawableOccurrence::RuntimeProxy(proxy) => Some(proxy.borrow().owner_handle()),
    }
}

/// Whether `item` is `container`'s own paint: a box's proxy, which draws
/// its fills, strokes, borders, shadows and gradients and then opens its
/// content clip. It paints below the box's children however the drawables
/// arrive: a retained remount appends re-created children after it.
fn is_own_paint(container: &CoreHandle, item: &RuntimeDrawableOccurrence) -> bool {
    let RuntimeDrawableOccurrence::RuntimeProxy(proxy) = item else {
        return false;
    };
    container
        .with(|object| {
            object
                .as_layout_component()
                .and_then(|layout| layout.existing_proxy())
                .is_some_and(|own| Rc::ptr_eq(own, proxy))
        })
        .unwrap_or(false)
}

/// The child of `container` that `component` is, or is inside. None for the
/// container itself, its foreground paint, or anything outside it.
fn child_of(container: &CoreHandle, component: CoreHandle) -> Option<CoreHandle> {
    if &component == container {
        return None;
    }
    let mut current = component;
    loop {
        let parent = current
            .with(|object| object.component_parent_handle())
            .flatten()?;
        if &parent == container {
            return (!current.is_type_of(ForegroundLayoutDrawableBase::TYPE_KEY))
                .then_some(current);
        }
        current = parent;
    }
}

/// How `component`, a CSS container's child, orders as a container itself.
pub(crate) fn container_of(component: &CoreHandle) -> Container {
    component
        .with(|object| {
            object.as_layout_component().map(|layout| Container {
                css: layout.is_css_mode(),
                flex_or_grid: layout
                    .with_css(|css| matches!(css.choice(keys::DISPLAY), 1 | 2))
                    .unwrap_or(false),
            })
        })
        .flatten()
        .unwrap_or(Container {
            css: false,
            flex_or_grid: false,
        })
}

/// A child's paint layer, bottom first, and whether descendants can escape.
/// Participants order by their host's record. Translated absolute children
/// remain positioned with z-index auto.
fn placement(child: &CoreHandle, parent: Container) -> ((u8, i32), bool) {
    use crate::source::layout::layout_style_applier::YGPositionType;
    child
        .with(|object| {
            let read = |css: super::CssValues<'_>| (css.choice(keys::POSITION) != 0, css.z_index());
            let layout = object.as_layout_component();
            let css_box = layout.is_some_and(|layout| layout.is_css_mode());
            let slot_record = layout
                .is_none()
                .then(|| object.as_container_component())
                .flatten()
                .and_then(|container| super::record::find_in(container.children()));
            let values = match layout {
                Some(layout) if css_box => layout.with_css(read),
                Some(layout) => layout
                    .with_style(|style| style.position_type() == YGPositionType::Absolute)
                    .filter(|absolute| *absolute)
                    .map(|_| (true, None)),
                None => slot_record
                    .as_ref()
                    .and_then(|record| super::record::with_values(record, read)),
            };
            let (positioned, z) = values.unwrap_or((false, None));
            let z = z.filter(|_| positioned || parent.flex_or_grid);
            // A CSS slot takes the layer of the frame it places, and stays
            // atomic (css_layout/hosted_frame.rs).
            let context = (css_box
                && (z.is_some()
                    || layout.is_some_and(|layout| layout.css_effect_stacking_context())))
                || (slot_record.is_some() && super::hosted_frame::slot_makes_context(object));
            let layer = match z {
                Some(z) if z < 0 => (0, z),
                Some(z) if z > 0 => (3, z),
                _ if positioned || context || z.is_some() => (2, 0),
                _ => (1, 0),
            };
            // Only CSS layout boxes are transparent to this walk. In
            // particular, participant hosts, lists and nested artboards keep
            // their complete internal drawing order.
            // Mask and mask-source brackets also require contiguous subtrees.
            let masked = object.as_drawable().is_some_and(|drawable| {
                !drawable.layer_masks().is_empty() || !drawable.source_of_layer_masks().is_empty()
            });
            let atomic = !css_box || masked || layout.is_some_and(|layout| layout.clips());
            (layer, context || atomic)
        })
        .unwrap_or(((1, 0), true))
}

struct Parts {
    top: Vec<RuntimeDrawableOccurrence>,
    paint: Vec<RuntimeDrawableOccurrence>,
    children: Vec<(CoreHandle, Vec<RuntimeDrawableOccurrence>)>,
}

fn parts(
    container: &CoreHandle,
    items: Vec<RuntimeDrawableOccurrence>,
    file_order: &HashMap<CoreHandle, usize>,
) -> Parts {
    let mut result = Parts {
        top: Vec::new(),
        paint: Vec::new(),
        children: Vec::new(),
    };
    for item in items {
        match owner_of(&item).and_then(|owner| child_of(container, owner)) {
            // This container is CSS. Its proxy owns its paint, including
            // when the artboard itself is borrowed by the sorter.
            None if matches!(&item, RuntimeDrawableOccurrence::RuntimeProxy(proxy)
                if &proxy.borrow().owner_handle() == container) =>
            {
                result.paint.push(item)
            }
            None => result.top.push(item),
            Some(child) => match result.children.iter_mut().find(|(key, _)| key == &child) {
                Some((_, group)) => group.push(item),
                None => result.children.push((child, vec![item])),
            },
        }
    }
    result
        .children
        .sort_by_key(|(child, _)| file_order.get(child).copied().unwrap_or(usize::MAX));
    result
}

/// Units are collected in tree preorder, including across static wrappers.
/// Each unit holds draw-list order (top first); the context sorts units in
/// paint order, then reverses them when flattening back into the draw list.
#[derive(Default)]
struct Context {
    units: Vec<((u8, i32), Vec<RuntimeDrawableOccurrence>)>,
}

impl Context {
    fn collect(
        &mut self,
        child: &CoreHandle,
        parent: Container,
        items: Vec<RuntimeDrawableOccurrence>,
        file_order: &HashMap<CoreHandle, usize>,
    ) -> Vec<RuntimeDrawableOccurrence> {
        let (layer, atomic) = placement(child, parent);
        if atomic {
            let items = order(child, container_of(child), items, file_order);
            if layer.0 == 1 {
                return items;
            }
            self.units.push((layer, items));
            return Vec::new();
        }
        let kind = container_of(child);
        let Parts {
            top,
            paint,
            children,
        } = parts(child, items, file_order);
        // Reserve the pseudo-context's tree position before collecting its
        // descendants, which can join this same positioned phase.
        let slot = (layer.0 != 1).then(|| {
            let index = self.units.len();
            self.units.push((layer, Vec::new()));
            index
        });
        let mut flow = Vec::new();
        for (descendant, items) in children {
            flow.push(self.collect(&descendant, kind, items, file_order));
        }
        let items = top
            .into_iter()
            .chain(flow.into_iter().rev().flatten())
            .chain(paint)
            .collect();
        if let Some(slot) = slot {
            self.units[slot].1 = items;
            Vec::new()
        } else {
            items
        }
    }
}

/// Reorder a subtree in draw-list order (the first drawable is on top).
/// Non-CSS containers retain Rive's child order and form an escape boundary.
pub(crate) fn order(
    container: &CoreHandle,
    kind: Container,
    items: Vec<RuntimeDrawableOccurrence>,
    file_order: &HashMap<CoreHandle, usize>,
) -> Vec<RuntimeDrawableOccurrence> {
    if !kind.css {
        // Keep the translated path's prefix and suffix behavior intact.
        let mut prefix = Vec::new();
        let mut suffix = Vec::new();
        let mut groups: Vec<(CoreHandle, Vec<RuntimeDrawableOccurrence>)> = Vec::new();
        for item in items {
            match owner_of(&item).and_then(|owner| child_of(container, owner)) {
                None if is_own_paint(container, &item) => suffix.push(item),
                None if groups.is_empty() => prefix.push(item),
                None => suffix.push(item),
                Some(child) => match groups.iter_mut().find(|(key, _)| key == &child) {
                    Some((_, group)) => group.push(item),
                    None => groups.push((child, vec![item])),
                },
            }
        }
        groups.sort_by_key(|(child, _)| file_order.get(child).copied().unwrap_or(usize::MAX));
        return prefix
            .into_iter()
            .chain(groups.into_iter().flat_map(|(child, items)| {
                if items.len() > 1 {
                    order(&child, container_of(&child), items, file_order)
                } else {
                    items
                }
            }))
            .chain(suffix)
            .collect();
    }
    let Parts {
        top,
        paint,
        children,
    } = parts(container, items, file_order);
    let mut context = Context::default();
    for (child, items) in children {
        // Reserve the in-flow unit in tree order too. Flex and grid items
        // paint as units here, while their positioned descendants escape.
        let slot = context.units.len();
        context.units.push(((1, 0), Vec::new()));
        let flow = context.collect(&child, kind, items, file_order);
        context.units[slot].1 = flow;
    }
    context.units.sort_by_key(|(layer, _)| *layer);
    top.into_iter()
        .chain(context.units.into_iter().rev().flat_map(|(_, items)| items))
        .chain(paint)
        .collect()
}

/// Whether two entries of a draw list are the same drawable.
pub(crate) fn same(a: &RuntimeDrawableOccurrence, b: &RuntimeDrawableOccurrence) -> bool {
    match (a, b) {
        (RuntimeDrawableOccurrence::Authored(a), RuntimeDrawableOccurrence::Authored(b)) => a == b,
        (
            RuntimeDrawableOccurrence::RuntimeProxy(a),
            RuntimeDrawableOccurrence::RuntimeProxy(b),
        ) => Rc::ptr_eq(a, b),
        _ => false,
    }
}

/// The runs of a draw list (top first) that a flight never goes inside, as
/// (top, bottom) indices: a clipping layout's bracket, from its own
/// drawable (the restore) to its proxy (the save and clip), CSS or not; and
/// a mask's member run and source run. Each is contiguous, since the order
/// keeps clips and masked groups atomic. Nothing here borrows the root.
fn spans(root: &CoreHandle, list: &[RuntimeDrawableOccurrence]) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    for (own, item) in list.iter().enumerate() {
        let RuntimeDrawableOccurrence::Authored(layout) = item else {
            continue;
        };
        if layout == root
            || !layout
                .with(|object| object.as_layout_component().is_some_and(|l| l.clips()))
                .unwrap_or(false)
        {
            continue;
        }
        let proxy = list.iter().position(|item| {
            matches!(item, RuntimeDrawableOccurrence::RuntimeProxy(proxy)
                if &proxy.borrow().owner_handle() == layout)
        });
        if let Some(proxy) = proxy {
            spans.push((own.min(proxy), own.max(proxy)));
        }
    }
    let mut runs: HashMap<(CoreHandle, bool), (usize, usize)> = HashMap::new();
    for (index, item) in list.iter().enumerate() {
        if owner_of(item).as_ref() == Some(root) {
            continue;
        }
        for (masks, source) in [
            (item.layer_masks(), false),
            (item.source_of_layer_masks(), true),
        ] {
            for mask in masks {
                runs.entry((mask, source))
                    .and_modify(|run| run.1 = index)
                    .or_insert((index, index));
            }
        }
    }
    spans.extend(runs.into_values());
    spans
}

/// Paints each flight slot that carries a placement right where its anchor
/// paints (docs/CSS_LAYOUT.md, 4.4), on the root's draw list after `order`:
/// directly under its coverer, at the top when it has none, and directly
/// above its anchor when the coverer no longer paints after the anchor.
/// Flights keep their anchors' order among themselves, flights at one gap
/// stack in placing order, and none goes inside a clip or mask span. A list
/// with no placed flight is left untouched.
pub(crate) fn place(root: &CoreHandle, drawables: &mut Vec<RuntimeDrawableOccurrence>) {
    let placement = |item: &RuntimeDrawableOccurrence| match item {
        RuntimeDrawableOccurrence::Authored(slot) if slot != root => slot
            .with(|object| {
                object
                    .as_nested_artboard()
                    .and_then(crate::source::nested_artboard::NestedArtboard::live_move_paint)
            })
            .flatten(),
        _ => None,
    };
    let has_anchor = |anchor: &CoreHandle| {
        drawables
            .iter()
            .any(|item| matches!(item, RuntimeDrawableOccurrence::Authored(slot) if slot == anchor))
    };
    let placements: Vec<_> = drawables
        .iter()
        .map(|item| placement(item).filter(|(anchor, _)| has_anchor(anchor)))
        .collect();
    if placements.iter().all(Option::is_none) {
        return;
    }
    // Take the placed flights out, keeping their relative order.
    let mut rest = Vec::with_capacity(drawables.len());
    let mut flights = Vec::new();
    for (item, placement) in std::mem::take(drawables).into_iter().zip(placements) {
        match placement {
            Some((anchor, coverer)) => flights.push((item, anchor, coverer)),
            None => rest.push(item),
        }
    }
    let index_of =
        |target: &RuntimeDrawableOccurrence| rest.iter().position(|item| same(item, target));
    let mut flights: Vec<_> = flights
        .into_iter()
        .filter_map(|(item, anchor, coverer)| {
            let anchor = index_of(&RuntimeDrawableOccurrence::Authored(anchor))?;
            Some((item, anchor, coverer))
        })
        .collect();
    // By anchor, top first; flights that share an anchor keep file order.
    flights.sort_by_key(|(_, anchor, _)| *anchor);
    // The root's own top items stay above every unit and every flight.
    let top = rest
        .iter()
        .take_while(|item| {
            owner_of(item)
                .and_then(|owner| child_of(root, owner))
                .is_none()
        })
        .count();
    let spans = spans(root, &rest);
    // Each flight's gap: it goes directly above rest[gap].
    let mut gaps = Vec::with_capacity(flights.len());
    let mut floor = 0;
    for (_, anchor, coverer) in &flights {
        let anchor = *anchor;
        let mut gap = match coverer.as_ref().map(|coverer| index_of(coverer)) {
            None => top.min(anchor),
            Some(Some(coverer)) if coverer < anchor => coverer + 1,
            // The coverer no longer paints after the anchor: the next read
            // finds the new one.
            Some(_) => anchor,
        };
        // A flight never goes higher than the flight placed before it.
        gap = gap.max(floor);
        // Out of every span: above one that holds the anchor, else under it.
        for _ in 0..=spans.len() {
            let Some(&(first, last)) = spans
                .iter()
                .find(|&&(first, last)| first < gap && gap <= last)
            else {
                break;
            };
            gap = if (first..=last).contains(&anchor) {
                first
            } else {
                last + 1
            };
        }
        floor = gap;
        gaps.push(gap);
    }
    let mut placed = flights
        .into_iter()
        .map(|(item, _, _)| item)
        .zip(gaps)
        .peekable();
    let mut list = Vec::with_capacity(rest.len() + placed.len());
    for (index, item) in rest.into_iter().enumerate() {
        while let Some((flight, _)) = placed.next_if(|(_, gap)| *gap == index) {
            list.push(flight);
        }
        list.push(item);
    }
    list.extend(placed.map(|(flight, _)| flight));
    *drawables = list;
}
