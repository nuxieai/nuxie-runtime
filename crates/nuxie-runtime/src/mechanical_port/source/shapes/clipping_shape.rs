use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    container_component::ContainerComponent,
    core::CoreHandle,
    core_context::CoreContext,
    drawable::{DrawableProxy, ProxyDrawing, RuntimeDrawableOccurrence},
    generated::shapes::clipping_shape_base::ClippingShapeBase,
    math::mat2d::Mat2D,
    shapes::{
        paint::{
            effects_container::EffectsContainer, fill::Fill, shape_paint::ShapePaintPathKind,
            shape_paint_path::ShapePaintPath,
        },
        path_flags::PathFlags,
    },
    status_code::StatusCode,
};
use nuxie_render_api::Renderer;

pub trait ClippingShapeOperation {
    fn draw(&mut self, renderer: &mut dyn Renderer, needs_save_operation: bool);
    fn empty_clip_count(&mut self) -> i32;
    fn set_clipping_shape(&mut self, shape: CoreHandle);
    fn is_start(&self) -> bool {
        false
    }
    fn is_visible(&self) -> bool {
        true
    }
}

#[derive(Default)]
pub struct ClippingShapeStart {
    clipping_shape: Option<CoreHandle>,
}

impl ClippingShapeOperation for ClippingShapeStart {
    fn draw(&mut self, renderer: &mut dyn Renderer, needs_save_operation: bool) {
        if let Some(shape) = self.clipping_shape.as_ref() {
            shape.with_downcast_mut::<ClippingShape, _>(|shape| {
                if !shape.base.is_visible() {
                    return;
                }
                if needs_save_operation {
                    renderer.save();
                }
                if shape.clip_path {
                    let factory = shape
                        .base
                        .with_artboard(|artboard| artboard.factory())
                        .flatten();
                    if let Some(factory) = factory {
                        renderer.clip_path(shape.path.render_path(&factory));
                    }
                }
            });
        }
    }

    fn empty_clip_count(&mut self) -> i32 {
        let Some(shape) = self.clipping_shape.as_ref() else {
            return 0;
        };
        shape
            .with_downcast_mut::<ClippingShape, _>(|shape| {
                i32::from(shape.base.is_visible() && shape.path().is_none())
            })
            .unwrap_or_default()
    }

    fn set_clipping_shape(&mut self, shape: CoreHandle) {
        self.clipping_shape = Some(shape);
    }

    fn is_start(&self) -> bool {
        true
    }

    fn is_visible(&self) -> bool {
        self.clipping_shape
            .as_ref()
            .and_then(|shape| {
                shape.with_downcast::<ClippingShape, _>(|shape| shape.base.is_visible())
            })
            .unwrap_or(false)
    }
}

#[derive(Default)]
pub struct ClippingShapeEnd {
    clipping_shape: Option<CoreHandle>,
}

impl ClippingShapeOperation for ClippingShapeEnd {
    fn draw(&mut self, renderer: &mut dyn Renderer, needs_save_operation: bool) {
        if self
            .clipping_shape
            .as_ref()
            .and_then(|shape| {
                shape.with_downcast::<ClippingShape, _>(|shape| shape.base.is_visible())
            })
            .unwrap_or(false)
            && needs_save_operation
        {
            renderer.restore();
        }
    }

    fn empty_clip_count(&mut self) -> i32 {
        let Some(shape) = self.clipping_shape.as_ref() else {
            return 0;
        };
        shape
            .with_downcast_mut::<ClippingShape, _>(|shape| {
                if shape.base.is_visible() && shape.path().is_none() {
                    -1
                } else {
                    0
                }
            })
            .unwrap_or_default()
    }

    fn set_clipping_shape(&mut self, shape: CoreHandle) {
        self.clipping_shape = Some(shape);
    }
}

pub struct ClippingShapeProxyDrawing {
    owner: CoreHandle,
    operation: Rc<RefCell<Box<dyn ClippingShapeOperation>>>,
}

impl ProxyDrawing for ClippingShapeProxyDrawing {
    fn draw_proxy(
        &mut self,
        renderer: &mut crate::mechanical_port::source::renderer::Renderer<'_>,
        needs_save_operation: bool,
    ) {
        self.operation
            .borrow_mut()
            .draw(renderer, needs_save_operation);
    }

    fn is_proxy_hidden(&self) -> bool {
        false
    }

    fn owner_handle(&self) -> CoreHandle {
        self.owner.clone()
    }

    fn empty_clip_count(&mut self) -> i32 {
        self.operation.borrow_mut().empty_clip_count()
    }

    fn is_clip_start(&self) -> bool {
        self.operation.borrow().is_start()
    }

    fn is_clip_end(&self) -> bool {
        !self.operation.borrow().is_start()
    }

    fn will_clip(&self) -> bool {
        self.operation.borrow().is_visible()
    }
}

struct ClippingShapeProxy {
    drawable: Rc<RefCell<DrawableProxy>>,
    operation: Rc<RefCell<Box<dyn ClippingShapeOperation>>>,
}

impl std::ops::Deref for ClippingShape {
    type Target = ClippingShapeBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ClippingShape {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ClippingShape {
    pub const TYPE_KEY: u16 = ClippingShapeBase::TYPE_KEY;
}

pub struct ClippingShape {
    pub base: ClippingShapeBase,
    shapes: Vec<CoreHandle>,
    proxy_drawables: Vec<ClippingShapeProxy>,
    pooled_proxy_drawables: Vec<ClippingShapeProxy>,
    source: Option<CoreHandle>,
    path: ShapePaintPath,
    clip_path: bool,
    pub clip_start: ClippingShapeStart,
    pub clip_end: ClippingShapeEnd,
}

impl Default for ClippingShape {
    fn default() -> Self {
        Self {
            base: ClippingShapeBase::default(),
            shapes: Vec::new(),
            proxy_drawables: Vec::new(),
            pooled_proxy_drawables: Vec::new(),
            source: None,
            path: ShapePaintPath::new(false),
            clip_path: false,
            clip_start: ClippingShapeStart::default(),
            clip_end: ClippingShapeEnd::default(),
        }
    }
}

impl ClippingShape {
    pub fn source(&self) -> Option<CoreHandle> {
        self.source.clone()
    }

    pub fn shapes(&self) -> &[CoreHandle] {
        &self.shapes
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        if let Some(parent) = self.base.parent_handle() {
            ContainerComponent::for_all(&parent, |component| {
                if component
                    .is_type_of(crate::mechanical_port::source::generated::drawable_base::DrawableBase::TYPE_KEY)
                {
                    component.with_mut(|component| {
                        component
                            .as_drawable_mut()
                            .expect("Drawable subtype")
                            .add_clipping_shape(this.clone());
                    });
                }
                true
            });
        }
        if let Some(source) = self.source.clone() {
            ContainerComponent::for_all(&source, |component| {
                if component
                    .is_type_of(crate::mechanical_port::source::generated::shapes::shape_base::ShapeBase::TYPE_KEY)
                {
                    let handle = component.clone();
                    component.with_mut(|component| {
                        component
                            .as_shape_mut()
                            .expect("Shape subtype")
                            .add_flags(PathFlags::WORLD | PathFlags::CLIPPING);
                    });
                    self.shapes.push(handle);
                }
                true
            });
        }
        StatusCode::Ok
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(source) = context
            .resolve_handle(self.base.source_id())
            .filter(|source| {
                source.is_type_of(
                    crate::mechanical_port::source::generated::node_base::NodeBase::TYPE_KEY,
                )
            })
        else {
            return StatusCode::MissingObject;
        };
        self.source = Some(source);
        StatusCode::Ok
    }

    pub fn build_dependencies(&mut self) {
        let Some(this) = self.base.handle() else {
            return;
        };
        for shape in &self.shapes {
            let paints = shape
                .with_mut(|shape| {
                    let shape = shape.as_shape_mut()?;
                    shape.path_composer_mut().add_dependent(this.clone());
                    Some(shape.paint_container.shape_paints().to_vec())
                })
                .flatten()
                .unwrap_or_default();
            // Cached fill effects must finish updating before the clip reads them.
            for paint in paints {
                paint.with_downcast_mut::<Fill, _>(|fill| {
                    if fill.base.base.has_effects() {
                        fill.base.add_dependent(this.clone());
                    }
                });
            }
        }
        self.clip_start.set_clipping_shape(this.clone());
        self.clip_end.set_clipping_shape(this);
    }

    fn add_fill_paths(&mut self, shape: &CoreHandle) -> bool {
        let Some((paints, world)) = shape
            .with(|shape| {
                let shape = shape.as_shape()?;
                Some((
                    shape.paint_container.shape_paints().to_vec(),
                    *shape.base.world_transform(),
                ))
            })
            .flatten()
        else {
            return false;
        };
        let mut added_effected = false;
        let mut needs_raw_path = false;
        for paint in paints {
            // Stroke effect outputs are centerlines, not filled clip geometry.
            let Some(effected) = paint.with_downcast_mut::<Fill, _>(|fill| {
                let paint = &mut fill.base.base;
                if !paint.has_effects() {
                    None
                } else {
                    let provider = *paint.path_provider();
                    paint.last_effect_path(&provider)
                }
            }) else {
                continue;
            };
            let Some(effected) = effected else {
                needs_raw_path = true;
                continue;
            };
            let effected = effected.borrow();
            if !effected.empty() {
                let transform = if effected.is_local() {
                    world
                } else {
                    Mat2D::identity()
                };
                self.path.add_shape_paint_path(&effected, Some(&transform));
            }
            // An existing empty effect still contributes an empty clip; never
            // replace it with the unaffected shape unless another fill needs it.
            added_effected = true;
        }
        if added_effected && !needs_raw_path {
            return true;
        }
        shape
            .with(|shape| {
                let Some(shape) = shape.as_shape() else {
                    return added_effected;
                };
                shape.with_path_mut(ShapePaintPathKind::World, |path| {
                    self.path
                        .add_shape_paint_path(path, Some(&Mat2D::identity()));
                });
                true
            })
            .unwrap_or(added_effected)
    }

    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(
            value,
            ComponentDirt::PATH | ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER,
        ) {
            let fill_rule = match self.base.fill_rule() {
                0 => nuxie_render_api::FillRule::NonZero,
                1 => nuxie_render_api::FillRule::EvenOdd,
                2 => nuxie_render_api::FillRule::Clockwise,
                value => panic!("unsupported renderer fill rule {value}"),
            };
            self.path.rewind_as(false, fill_rule);
            self.clip_path = false;
            for index in 0..self.shapes.len() {
                let shape = self.shapes[index].clone();
                let nonempty = shape
                    .with(|shape| shape.as_shape().is_some_and(|shape| !shape.is_empty()))
                    .unwrap_or(false);
                if nonempty && self.add_fill_paths(&shape) {
                    self.clip_path = true;
                }
            }
        }
    }

    pub fn is_visible_changed(&mut self) {
        self.base.with_artboard_mut(|artboard| {
            artboard.add_dirt(ComponentDirt::CLIPPING, false);
        });
    }

    pub fn path(&mut self) -> Option<&mut ShapePaintPath> {
        self.clip_path.then_some(&mut self.path)
    }

    pub fn reset_drawables(&mut self) {
        self.pooled_proxy_drawables
            .append(&mut self.proxy_drawables);
    }

    pub fn create_proxy_drawable(
        &mut self,
        operation: Box<dyn ClippingShapeOperation>,
    ) -> Option<RuntimeDrawableOccurrence> {
        let proxy = if let Some(proxy) = self.pooled_proxy_drawables.pop() {
            *proxy.operation.borrow_mut() = operation;
            proxy
                .drawable
                .borrow_mut()
                .base
                .set_needs_save_operation(true);
            proxy
        } else {
            let owner = self.base.handle()?;
            let operation = Rc::new(RefCell::new(operation));
            let drawable = Rc::new(RefCell::new(DrawableProxy::new(Box::new(
                ClippingShapeProxyDrawing {
                    owner,
                    operation: operation.clone(),
                },
            ))));
            ClippingShapeProxy {
                drawable,
                operation,
            }
        };
        let drawable = proxy.drawable.clone();
        self.proxy_drawables.push(proxy);
        Some(RuntimeDrawableOccurrence::runtime_proxy(drawable))
    }
}
