//! CSS overflow clips follow containing blocks, independently of paint order.
use super::keys;
use crate::source::{
    core::{CoreHandle, CoreObject},
    drawable::{Drawable, DrawableProxy, ProxyDrawing, RuntimeDrawableOccurrence},
    layout_component::LayoutComponent,
    renderer::Renderer,
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

/// Masked content keeps ancestor overflow outside its cached canvas.
pub(crate) fn keeps_full_chain(drawable: &Drawable) -> bool {
    !drawable.layer_masks().is_empty() || !drawable.source_of_layer_masks().is_empty()
}

/// A component's `position` for the containing-block rule: a layout's as
/// `position` reads it, a participant's from the record among its children
/// (as paint order reads it), and static for anything else.
pub(crate) fn object_position(object: &dyn CoreObject) -> u8 {
    if let Some(layout) = object.as_layout_component() {
        return position(layout);
    }
    object
        .as_container_component()
        .and_then(|container| super::record::find_in(container.children()))
        .and_then(|record| super::record::with_values(&record, |css| css.choice(keys::POSITION)))
        .unwrap_or(0)
}

pub(crate) fn position(layout: &LayoutComponent) -> u8 {
    if layout.is_artboard() {
        return 0;
    }
    layout
        .with_css(|css| css.choice(keys::POSITION))
        .unwrap_or(1)
}

/// Keep interaction targets, translated clips and artboard host mapping.
/// CSS overflow has already been tested from the hit owner's clip list.
pub(crate) fn hit_test_ancestors(
    mut parent: Option<CoreHandle>,
    point: &crate::source::math::vec2d::Vec2D,
) -> bool {
    while let Some(owner) = parent {
        let Some((hit, next)) = owner.with_mut(|object| {
            let next = object.component_parent_handle();
            if let Some(artboard) = object.as_artboard_mut() {
                return (
                    artboard.hit_test_point_filtered(point, true, false, false),
                    None,
                );
            }
            let hit = if let Some(layout) = object.as_layout_component() {
                !layout.is_hidden() && layout.hit_test_contains(point, true, false, None, false)
            } else {
                let visible = object
                    .as_drawable()
                    .is_none_or(|drawable| !drawable.is_hidden());
                // A field's own target, which the ancestor chain checked for
                // anything inside it too.
                visible
                    && object
                        .as_text_input_mut()
                        .is_none_or(|input| input.contains_point(*point))
            };
            (hit, next)
        }) else {
            // Match Component's parent chain during a reentrant borrow.
            return true;
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
    css_context: bool,
    full_chain: bool,
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

    pub(crate) fn set_full_chain(&mut self, full: bool) {
        self.full_chain = full;
    }

    pub(crate) fn is_css_context(&self) -> bool {
        self.css_context
    }

    /// Read the same absolute-escape marks as drawing. A clip owner may
    /// already be borrowed by scrolling or text input, as in the hit chain.
    pub(crate) fn contains(&self, point: &crate::source::math::vec2d::Vec2D) -> bool {
        self.applicable(false).all(|clip| match clip {
            DrawClip::Overflow(owner) => owner
                .with(|object| {
                    let Some(layout) = object.as_layout_component() else {
                        return true;
                    };
                    let origin = object.as_artboard().map(|artboard| {
                        crate::source::math::vec2d::Vec2D::new(
                            artboard.origin_x() * artboard.layout_width(),
                            artboard.origin_y() * artboard.layout_height(),
                        )
                    });
                    layout.hit_test_contains(point, true, false, origin, true)
                })
                .unwrap_or(true),
            DrawClip::Shape(owner) => owner
                .with_downcast_mut::<crate::source::shapes::clipping_shape::ClippingShape, _>(
                    |shape| {
                        !shape.base.is_visible()
                            || shape.path().is_some_and(|path| {
                                crate::source::semantic::semantic_clip::path_contains(
                                    path.raw_path(),
                                    path.fill_rule(),
                                    *point,
                                )
                            })
                    },
                )
                .unwrap_or(true),
        })
    }

    pub(crate) fn root(root: &CoreHandle, overflow: bool, shapes: &[CoreHandle]) -> Self {
        let mut list = Self {
            scope: overflow,
            css_context: overflow,
            ..Self::default()
        };
        if overflow {
            // Keep shapes around the CSS overflow save. Without that
            // scope, Rive's shape membership belongs to each occurrence,
            // including translated paint proxies with no shape clips.
            for shape in shapes {
                list.push(DrawClip::Shape(shape.clone()));
            }
            list.push(DrawClip::Overflow(root.clone()));
        }
        list
    }

    fn applicable(&self, full: bool) -> impl Iterator<Item = &DrawClip> {
        self.entries
            .iter()
            .filter(move |entry| full || self.full_chain || !entry.skipped)
            .map(|entry| &entry.clip)
    }

    pub(crate) fn chain(&self, full: bool, shapes: &[CoreHandle]) -> (Vec<DrawClip>, bool) {
        let mut chain = self.applicable(full).cloned().collect::<Vec<_>>();
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
                            position: object_position(object),
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
                        paint: root_list.clone(),
                        children: root_list.clone(),
                        positioned_end: root_list.entries.len(),
                        css_box: false,
                    });
                    if node.position == 2 {
                        for entry in &mut state.children.entries[state.positioned_end..] {
                            if matches!(entry.clip, DrawClip::Overflow(_)) {
                                entry.skipped = true;
                            }
                        }
                    }
                    state.children.scope |= node.clips;
                    state.children.css_context |= node.css_box;
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
        .unwrap_or_else(|| root_list.chain(full, &shapes))
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
