//! CSS overflow clips follow containing blocks, independently of paint order.
use super::keys;
use crate::source::{
    core::CoreHandle,
    drawable::{Drawable, DrawableProxy, ProxyDrawing, RuntimeDrawableOccurrence},
    layout_component::LayoutComponent,
    renderer::Renderer,
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

/// State while walking from a painted box towards the root. An absolute
/// box outside a masked subtree skips overflow until its containing block.
#[derive(Default)]
pub(crate) struct OverflowFilter {
    position: u8,
    full_chain: bool,
}

impl OverflowFilter {
    pub(crate) fn for_layout(layout: &LayoutComponent) -> Self {
        let mut filter = Self {
            full_chain: keeps_full_chain(layout),
            ..Self::default()
        };
        filter.leave(position(layout));
        filter
    }

    pub(crate) fn keep_full_chain(&mut self) {
        self.full_chain = true;
        self.position = 0;
    }

    pub(crate) fn enter(&mut self, position: u8, artboard: bool) -> bool {
        if self.position == 2 && (position != 0 || artboard) {
            self.position = 0;
        }
        self.position == 0
    }

    pub(crate) fn leave(&mut self, position: u8) {
        if !self.full_chain && position == 2 {
            self.position = position;
        }
    }
}

/// Masked content keeps ancestor overflow outside its cached canvas.
pub(crate) fn keeps_full_chain(drawable: &Drawable) -> bool {
    !drawable.layer_masks().is_empty() || !drawable.source_of_layer_masks().is_empty()
}

pub(crate) fn position(layout: &LayoutComponent) -> u8 {
    if layout.is_artboard() {
        return 0;
    }
    layout
        .with_css(|css| css.choice(keys::POSITION))
        .unwrap_or(1)
}

/// Keep interaction's own targets and artboard host mapping, filtering
/// only the CSS overflow tests along the parent chain.
pub(crate) fn hit_test_ancestors(
    mut parent: Option<CoreHandle>,
    point: &crate::source::math::vec2d::Vec2D,
    mut filter: OverflowFilter,
) -> bool {
    while let Some(owner) = parent {
        let Some((hit, next)) = owner.with_mut(|object| {
            let next = object.component_parent_handle();
            if object.as_drawable().is_some_and(keeps_full_chain) {
                filter.keep_full_chain();
            }
            let position = object.as_layout_component().map(position).unwrap_or(0);
            let include = filter.enter(position, object.as_artboard().is_some());
            if let Some(artboard) = object.as_artboard_mut() {
                return (
                    artboard.hit_test_point_filtered(point, true, false, include),
                    None,
                );
            }
            let hit = if let Some(layout) = object.as_layout_component() {
                !layout.is_hidden() && layout.hit_test_contains(point, true, false, None, include)
            } else {
                object
                    .as_drawable()
                    .is_none_or(|drawable| !drawable.is_hidden())
            };
            filter.leave(position);
            (hit, next)
        }) else {
            return false;
        };
        if !hit {
            return false;
        }
        parent = next;
    }
    true
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum DrawClip {
    Shape(CoreHandle),
    Overflow(CoreHandle),
}

/// A foreground drawable paints its CSS box after the box's content.
/// It uses the box's ancestor clips, leaving out that box's overflow.
pub(crate) fn paint_owner(component: &CoreHandle) -> CoreHandle {
    use crate::source::generated::foreground_layout_drawable_base::ForegroundLayoutDrawableBase;
    if component.is_type_of(ForegroundLayoutDrawableBase::TYPE_KEY)
        && let Some(parent) = component
            .with(|object| object.component_parent_handle())
            .flatten()
        && parent
            .with(|object| {
                object
                    .as_layout_component()
                    .is_some_and(|layout| layout.is_css_mode() && !layout.is_artboard())
            })
            .unwrap_or(false)
    {
        return parent;
    }
    component.clone()
}

/// Membership for a paint owner, in tree order. Masked content reads the
/// skipped entries too, without changing the cached membership.
#[derive(Clone, Default)]
pub(crate) struct ClipList {
    entries: Vec<ClipEntry>,
    scope: bool,
}

#[derive(Clone)]
struct ClipEntry {
    clip: DrawClip,
    skipped: bool,
}

impl ClipList {
    fn push(&mut self, clip: DrawClip) {
        if !self.entries.iter().any(|entry| entry.clip == clip) {
            self.entries.push(ClipEntry {
                clip,
                skipped: false,
            });
        }
    }

    pub(crate) fn root(root: &CoreHandle, overflow: bool, shapes: &[CoreHandle]) -> Self {
        let mut list = Self {
            scope: overflow,
            ..Self::default()
        };
        if overflow {
            for shape in shapes {
                list.push(DrawClip::Shape(shape.clone()));
            }
            list.push(DrawClip::Overflow(root.clone()));
        }
        list
    }

    fn chain(&self, full: bool, shapes: &[CoreHandle]) -> (Vec<DrawClip>, bool) {
        let mut chain = self
            .entries
            .iter()
            .filter(|entry| full || !entry.skipped)
            .map(|entry| entry.clip.clone())
            .collect::<Vec<_>>();
        for shape in shapes {
            let clip = DrawClip::Shape(shape.clone());
            if !chain.contains(&clip) {
                chain.push(clip);
            }
        }
        (chain, self.scope)
    }
}

#[derive(Clone, Default)]
struct BuildState {
    paint: ClipList,
    children: ClipList,
    // Entries before this boundary have a positioned box between them
    // and the next child. Older skips remain in force.
    positioned_end: usize,
    css_box: bool,
}

struct BuildNode {
    parent: Option<CoreHandle>,
    position: u8,
    css_box: bool,
    clips: bool,
    shapes: Vec<CoreHandle>,
    foreground: bool,
}

enum BuildStep {
    Enter(CoreHandle),
    Store(CoreHandle, BuildNode),
}

/// Resolve each parent once and store it before its children. Imported
/// objects normally already have that order; late owners need not have it.
/// Proxy owners join the same pass even when they are not in the object table.
pub(crate) fn build_lists(
    root: &CoreHandle,
    root_list: &ClipList,
    objects: &[Option<CoreHandle>],
    drawables: &[RuntimeDrawableOccurrence],
) -> usize {
    use crate::source::generated::foreground_layout_drawable_base::ForegroundLayoutDrawableBase;
    let mut states = HashMap::from([(
        root.clone(),
        BuildState {
            paint: root_list.clone(),
            children: root_list.clone(),
            positioned_end: root_list.entries.len(),
            css_box: false,
        },
    )]);
    let mut scheduled = HashSet::new();
    let mut parent_steps = 0;
    let owners = drawables.iter().map(|drawable| match drawable {
        RuntimeDrawableOccurrence::Authored(owner) => owner.clone(),
        RuntimeDrawableOccurrence::RuntimeProxy(proxy) => proxy.borrow().owner_handle(),
    });
    for owner in objects.iter().flatten().cloned().chain(owners) {
        let mut pending = vec![BuildStep::Enter(owner)];
        while let Some(step) = pending.pop() {
            match step {
                BuildStep::Enter(owner) => {
                    if states.contains_key(&owner) || !scheduled.insert(owner.clone()) {
                        continue;
                    }
                    let Some(node) = owner.with(|object| {
                        let layout = object.as_layout_component();
                        let css_box = layout.is_some_and(LayoutComponent::is_css_mode);
                        let clips = css_box && layout.is_some_and(LayoutComponent::clips);
                        BuildNode {
                            parent: object.component_parent_handle(),
                            position: layout.map(position).unwrap_or(0),
                            css_box,
                            clips,
                            shapes: if clips {
                                layout.unwrap().clipping_shapes().to_vec()
                            } else {
                                Vec::new()
                            },
                            foreground: owner.is_type_of(ForegroundLayoutDrawableBase::TYPE_KEY),
                        }
                    }) else {
                        continue;
                    };
                    parent_steps += usize::from(node.parent.is_some());
                    let parent = node.parent.clone();
                    pending.push(BuildStep::Store(owner, node));
                    if let Some(parent) = parent {
                        pending.push(BuildStep::Enter(parent));
                    }
                }
                BuildStep::Store(owner, node) => {
                    let parent = node.parent.as_ref().and_then(|parent| states.get(parent));
                    let mut state = parent.cloned().unwrap_or_else(|| BuildState {
                        children: ClipList {
                            scope: root_list.scope,
                            ..ClipList::default()
                        },
                        ..BuildState::default()
                    });
                    if node.position == 2 {
                        for entry in &mut state.children.entries[state.positioned_end..] {
                            if matches!(entry.clip, DrawClip::Overflow(_)) {
                                entry.skipped = true;
                            }
                        }
                    }
                    state.children.scope |= node.clips;
                    state.paint = if node.foreground && parent.is_some_and(|parent| parent.css_box)
                    {
                        parent.unwrap().paint.clone()
                    } else {
                        state.children.clone()
                    };
                    owner.with_mut(|object| {
                        if let Some(drawable) = object.as_drawable_mut() {
                            drawable.set_css_clip_list(state.paint.clone());
                        }
                    });
                    if node.clips {
                        for shape in node.shapes {
                            state.children.push(DrawClip::Shape(shape));
                        }
                        state.children.push(DrawClip::Overflow(owner.clone()));
                    }
                    if node.position != 0 {
                        state.positioned_end = state.children.entries.len();
                    }
                    state.css_box = node.css_box;
                    states.insert(owner, state);
                }
            }
        }
    }
    parent_steps
}

/// Sorting reads membership and the drawable's own shape list. It never
/// resolves a parent, including for foreground paint and layout proxies.
pub(crate) fn read_chain(
    drawable: &RuntimeDrawableOccurrence,
    root: &CoreHandle,
    root_list: &ClipList,
) -> (Vec<DrawClip>, bool) {
    let owner = match drawable {
        RuntimeDrawableOccurrence::Authored(owner) => owner.clone(),
        RuntimeDrawableOccurrence::RuntimeProxy(proxy) => proxy.borrow().owner_handle(),
    };
    let shapes = drawable
        .with(|drawable| drawable.clipping_shapes().to_vec())
        .unwrap_or_default();
    // The sort already holds the root's mutable receiver.
    if &owner == root {
        return root_list.chain(true, &shapes);
    }
    let full = !drawable.layer_masks().is_empty() || !drawable.source_of_layer_masks().is_empty();
    owner
        .with(|object| {
            object
                .as_drawable()
                .map(|drawable| drawable.css_clip_list().chain(full, &shapes))
        })
        .flatten()
        .unwrap_or_else(|| {
            ClipList {
                scope: root_list.scope,
                ..ClipList::default()
            }
            .chain(full, &shapes)
        })
}

struct OverflowOperation {
    owner: CoreHandle,
    start: bool,
}

impl ProxyDrawing for OverflowOperation {
    fn owner_handle(&self) -> CoreHandle {
        self.owner.clone()
    }
    fn hittable_component(&self) -> Option<CoreHandle> {
        None
    }
    fn is_proxy_hidden(&self) -> bool {
        false
    }
    fn is_clip_start(&self) -> bool {
        self.start
    }
    fn is_clip_end(&self) -> bool {
        !self.start
    }
    fn will_clip(&self) -> bool {
        true
    }
    fn draw_proxy(&mut self, renderer: &mut Renderer, needs_save_operation: bool) {
        if self.start {
            if needs_save_operation {
                renderer.save();
            }
            let paths = self
                .owner
                .with(|object| {
                    object
                        .as_layout_component()
                        .and_then(LayoutComponent::render_paths_handle)
                })
                .flatten();
            if let Some(paths) = paths {
                // Like a shape paint's path, the clip path is made through the
                // artboard's factory at its first draw, which a renderer callback
                // such as the save above may have replaced since sorting.
                let owner = &self.owner;
                let factory = || {
                    owner
                        .with(|object| {
                            let layout = object.as_layout_component()?;
                            layout
                                .with_artboard(|artboard| artboard.factory())
                                .flatten()
                        })
                        .flatten()
                        .expect("a drawing CSS overflow box has its imported Artboard factory")
                };
                let mut paths = paths.borrow_mut();
                renderer.clip_path(paths.clip.render_path_with_deferred_factory(factory));
            }
        } else if needs_save_operation {
            renderer.restore();
        }
    }
}

pub(crate) fn operation(owner: CoreHandle, start: bool) -> RuntimeDrawableOccurrence {
    RuntimeDrawableOccurrence::runtime_proxy(Rc::new(RefCell::new(DrawableProxy::new(Box::new(
        OverflowOperation { owner, start },
    )))))
}
