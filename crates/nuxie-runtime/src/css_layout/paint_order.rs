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
            let values = match layout {
                Some(layout) if css_box => layout.with_css(read),
                Some(layout) => layout
                    .with_style(|style| style.position_type() == YGPositionType::Absolute)
                    .filter(|absolute| *absolute)
                    .map(|_| (true, None)),
                None => object
                    .as_container_component()
                    .and_then(|container| super::record::find_in(container.children()))
                    .and_then(|record| super::record::with_values(&record, read)),
            };
            let (positioned, z) = values.unwrap_or((false, None));
            let z = z.filter(|_| positioned || parent.flex_or_grid);
            let context = css_box
                && (z.is_some()
                    || layout.is_some_and(|layout| layout.css_effect_stacking_context()));
            let layer = match z {
                Some(z) if z < 0 => (0, z),
                Some(z) if z > 0 => (3, z),
                _ if positioned || context || z.is_some() => (2, 0),
                _ => (1, 0),
            };
            // Only CSS layout boxes are transparent to this walk. In
            // particular, participant hosts, lists and nested artboards keep
            // their complete internal drawing order.
            let atomic = !css_box || layout.is_some_and(|layout| layout.clips());
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
