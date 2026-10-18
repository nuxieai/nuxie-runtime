//! CSS painting order among a box's children (docs/CSS_LAYOUT.md, 4.4):
//! our layer over Rive's draw order. Rive draws an artboard's drawables in
//! reverse file order, so a box's first child draws on top. A CSS box paints
//! as CSS 2.1 Appendix E does within one box: its own paint, then children
//! positioned with a negative z-index, then in-flow children in tree order
//! (later on top), then positioned children with z-index auto or 0 in tree
//! order, then positive z-index ascending. Each child keeps its subtree
//! together, so a positioned descendant does not leave its parent's box.
//! A participant orders by its host's record, as a box does. Translated
//! (Rive) boxes keep Rive's order.
use super::keys;
use crate::source::{
    core::CoreHandle, drawable::RuntimeDrawableOccurrence,
    generated::foreground_layout_drawable_base::ForegroundLayoutDrawableBase,
};
use std::collections::HashMap;

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

/// A child's paint layer, bottom first, and its z-index within the layer.
/// A box reads its record; a participant's host reads the record among
/// its children.
fn layer_of(child: &CoreHandle, parent: Container) -> (u8, i32) {
    let read = |css: super::CssValues<'_>| (css.choice(keys::POSITION) != 0, css.z_index());
    let css = child
        .with(|object| match object.as_layout_component() {
            Some(layout) => layout.with_css(read),
            None => object
                .as_container_component()
                .and_then(|container| super::record::find_in(container.children()))
                .and_then(|record| super::record::with_values(&record, read)),
        })
        .flatten();
    let Some((positioned, z)) = css else {
        return (1, 0);
    };
    // z-index applies to positioned boxes, and to flex and grid items.
    let z = z.filter(|_| positioned || parent.flex_or_grid);
    match z {
        Some(z) if z < 0 => (0, z),
        None if !positioned => (1, 0),
        Some(z) if z > 0 => (3, z),
        _ => (2, 0),
    }
}

/// `items`, the drawables of `container`'s subtree in draw-list order (the
/// first is drawn last, on top), reordered for CSS containers.
pub(crate) fn order(
    container: &CoreHandle,
    kind: Container,
    items: Vec<RuntimeDrawableOccurrence>,
    file_order: &HashMap<CoreHandle, usize>,
) -> Vec<RuntimeDrawableOccurrence> {
    let mut prefix = Vec::new();
    let mut suffix = Vec::new();
    let mut groups: Vec<(CoreHandle, Vec<RuntimeDrawableOccurrence>)> = Vec::new();
    for item in items {
        match owner_of(&item).and_then(|owner| child_of(container, owner)) {
            None if groups.is_empty() => prefix.push(item),
            None => suffix.push(item),
            Some(child) => match groups.iter_mut().find(|(key, _)| key == &child) {
                Some((_, group)) => group.push(item),
                None => groups.push((child, vec![item])),
            },
        }
    }
    for (child, group) in &mut groups {
        if group.len() > 1 {
            *group = order(
                child,
                container_of(child),
                std::mem::take(group),
                file_order,
            );
        }
    }
    let tree = |child: &CoreHandle| file_order.get(child).copied().unwrap_or(usize::MAX);
    if kind.css {
        // Paint order, bottom first; the draw list holds the top first.
        groups.sort_by_cached_key(|(child, _)| {
            let (layer, z) = layer_of(child, kind);
            (layer, z, tree(child))
        });
        groups.reverse();
    } else {
        // Rive's order: file order, the first child on top.
        groups.sort_by_cached_key(|(child, _)| tree(child));
    }
    prefix
        .into_iter()
        .chain(groups.into_iter().flat_map(|(_, group)| group))
        .chain(suffix)
        .collect()
}
