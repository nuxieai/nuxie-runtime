//! CSS overflow clips follow containing blocks, independently of paint order.
use super::keys;
use crate::source::{
    core::CoreHandle,
    drawable::{DrawableProxy, ProxyDrawing, RuntimeDrawableOccurrence},
    factory::RuntimeFactoryHandle,
    layout_component::LayoutComponent,
    renderer::Renderer,
};
use std::{cell::RefCell, rc::Rc};

/// State while walking from a painted box towards the root. An absolute
/// box skips overflow until its containing block.
#[derive(Default)]
pub(crate) struct OverflowFilter {
    position: u8,
}

impl OverflowFilter {
    pub(crate) fn for_position(position: u8) -> Self {
        let mut filter = Self::default();
        filter.leave(position);
        filter
    }

    pub(crate) fn enter(&mut self, position: u8, artboard: bool) -> bool {
        if self.position == 2 && (position != 0 || artboard) {
            self.position = 0;
        }
        self.position == 0
    }

    pub(crate) fn leave(&mut self, position: u8) {
        if position == 2 {
            self.position = position;
        }
    }
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

/// Content clips and their enclosing shape clips, outside first. The root
/// is supplied separately because sort_draw_order already borrows it.
pub(crate) fn draw_chain(
    drawable: &RuntimeDrawableOccurrence,
    root: &CoreHandle,
    root_overflow: bool,
    root_shapes: &[CoreHandle],
) -> (Vec<DrawClip>, bool) {
    let owner = match drawable {
        RuntimeDrawableOccurrence::Authored(owner) => owner.clone(),
        RuntimeDrawableOccurrence::RuntimeProxy(proxy) => proxy.borrow().owner_handle(),
    };
    let owner = paint_owner(&owner);
    let mut filter = OverflowFilter::default();
    let mut current = Some(owner.clone());
    let mut levels = Vec::new();
    let mut css_scope = root_overflow;
    while let Some(handle) = current {
        if &handle == root {
            let mut clips = Vec::new();
            if root_overflow {
                clips.extend(root_shapes.iter().cloned().map(DrawClip::Shape));
                if filter.enter(0, true) {
                    clips.push(DrawClip::Overflow(handle));
                }
            }
            levels.push(clips);
            break;
        }
        let Some((parent, clips)) = handle.with(|object| {
            let mut clips = Vec::new();
            if let Some(layout) = object.as_layout_component() {
                let position = position(layout);
                css_scope |= layout.is_css_mode() && layout.clips();
                if handle != owner {
                    let include = filter.enter(position, false);
                    if layout.is_css_mode() && layout.clips() {
                        clips.extend(
                            layout
                                .clipping_shapes()
                                .iter()
                                .cloned()
                                .map(DrawClip::Shape),
                        );
                        if include {
                            clips.push(DrawClip::Overflow(handle.clone()));
                        }
                    }
                }
                filter.leave(position);
            }
            (object.component_parent_handle(), clips)
        }) else {
            break;
        };
        levels.push(clips);
        current = parent;
    }
    let mut chain = Vec::new();
    for clip in levels.into_iter().rev().flatten().chain(
        drawable
            .with(|drawable| drawable.clipping_shapes().to_vec())
            .unwrap_or_default()
            .into_iter()
            .map(DrawClip::Shape),
    ) {
        if !chain.contains(&clip) {
            chain.push(clip);
        }
    }
    (chain, css_scope)
}

struct OverflowOperation {
    owner: CoreHandle,
    factory: RuntimeFactoryHandle,
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
                renderer.clip_path(paths.borrow_mut().clip.render_path(&self.factory));
            }
        } else if needs_save_operation {
            renderer.restore();
        }
    }
}

pub(crate) fn operation(
    owner: CoreHandle,
    start: bool,
    factory: RuntimeFactoryHandle,
) -> RuntimeDrawableOccurrence {
    RuntimeDrawableOccurrence::runtime_proxy(Rc::new(RefCell::new(DrawableProxy::new(Box::new(
        OverflowOperation {
            owner,
            start,
            factory,
        },
    )))))
}
