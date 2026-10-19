//! CSS painting order among a box's children (docs/CSS_LAYOUT.md, 4.4):
//! our layer over Rive's draw order. Rive draws an artboard's drawables in
//! reverse file order, so a box's first child draws on top. A CSS box paints
//! as CSS 2.1 Appendix E does within one box: its own paint, then children
//! positioned with a negative z-index, then in-flow children in tree order
//! (later on top), then positioned children with z-index auto or 0 in tree
//! order, then positive z-index ascending. Each child keeps its subtree
//! together, so a positioned descendant does not leave its parent's box.
//! A participant orders by its host's record, as a box does, and a
//! translated child placed absolutely is positioned. Translated (Rive)
//! boxes keep Rive's order among their own children.
use super::keys;
use crate::source::{
    core::CoreHandle, drawable::RuntimeDrawableOccurrence,
    generated::foreground_layout_drawable_base::ForegroundLayoutDrawableBase,
};
use std::{collections::HashMap, rc::Rc};

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

/// A child's paint layer, bottom first, and its z-index within the layer.
/// A box reads its record; a participant's host reads the record among
/// its children. A translated box is positioned when layout places it
/// absolutely, with z-index auto.
fn layer_of(child: &CoreHandle, parent: Container) -> (u8, i32) {
    use crate::source::layout::layout_style_applier::YGPositionType;
    let read = |css: super::CssValues<'_>| (css.choice(keys::POSITION) != 0, css.z_index());
    let css = child
        .with(|object| match object.as_layout_component() {
            Some(layout) if layout.is_css_mode() => layout.with_css(read),
            Some(layout) => layout
                .with_style(|style| style.position_type() == YGPositionType::Absolute)
                .filter(|absolute| *absolute)
                .map(|_| (true, None)),
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
            // The box's own paint is the bottom of its stack.
            None if is_own_paint(container, &item) => suffix.push(item),
            // Its own drawable closes the clip above its children, and
            // foreground drawables draw over them.
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
