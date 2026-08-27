use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    core::{CoreContext, StatusCode},
    drawable::{Drawable, HitInfo},
    math::mat2d::Mat2D,
    renderer::Renderer,
    shapes::{path_flags::PathFlags, shape::Shape, shape_paint_path::ShapePaintPath},
};

pub trait ClippingShapeOperation {
    fn draw(&mut self, renderer: &mut Renderer, needs_save_operation: bool);
    fn empty_clip_count(&mut self) -> i32;
    fn set_clipping_shape(&mut self, shape: *mut ClippingShape);
    fn is_start(&self) -> bool {
        false
    }
    fn is_visible(&self) -> bool {
        true
    }
}

#[derive(Default)]
pub struct ClippingShapeStart {
    clipping_shape: Option<*mut ClippingShape>,
}

impl ClippingShapeOperation for ClippingShapeStart {
    fn draw(&mut self, renderer: &mut Renderer, needs_save_operation: bool) {
        let shape = unsafe { &mut *self.clipping_shape.unwrap() };
        if !shape.base.is_visible() {
            return;
        }
        if needs_save_operation {
            renderer.save();
        }
        if let Some(path) = shape.path() {
            let render_path = path.render_path(shape);
            renderer.clip_path(render_path);
        }
    }

    fn empty_clip_count(&mut self) -> i32 {
        let Some(pointer) = self.clipping_shape else {
            return 0;
        };
        let shape = unsafe { &mut *pointer };
        if shape.base.is_visible() && shape.path().is_none() {
            return 1;
        }
        0
    }

    fn set_clipping_shape(&mut self, shape: *mut ClippingShape) {
        self.clipping_shape = Some(shape);
    }

    fn is_start(&self) -> bool {
        true
    }

    fn is_visible(&self) -> bool {
        self.clipping_shape
            .map(|shape| unsafe { (&*shape).base.is_visible() })
            .unwrap_or(false)
    }
}

#[derive(Default)]
pub struct ClippingShapeEnd {
    clipping_shape: Option<*mut ClippingShape>,
}

impl ClippingShapeOperation for ClippingShapeEnd {
    fn draw(&mut self, renderer: &mut Renderer, needs_save_operation: bool) {
        let shape = unsafe { &*self.clipping_shape.unwrap() };
        if !shape.base.is_visible() || !needs_save_operation {
            return;
        }
        renderer.restore();
    }

    fn empty_clip_count(&mut self) -> i32 {
        let Some(pointer) = self.clipping_shape else {
            return 0;
        };
        let shape = unsafe { &mut *pointer };
        if shape.base.is_visible() && shape.path().is_none() {
            return -1;
        }
        0
    }

    fn set_clipping_shape(&mut self, shape: *mut ClippingShape) {
        self.clipping_shape = Some(shape);
    }
}

pub struct ClippingShapeProxyDrawable {
    pub drawable: Drawable,
    operation: Box<dyn ClippingShapeOperation>,
}

impl ClippingShapeProxyDrawable {
    pub fn new(operation: Box<dyn ClippingShapeOperation>) -> Self {
        Self {
            drawable: Drawable::default(),
            operation,
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer) {
        self.operation
            .draw(renderer, self.drawable.needs_save_operation());
    }

    pub fn empty_clip_count(&mut self) -> i32 {
        self.operation.empty_clip_count()
    }

    pub fn is_hidden(&self) -> bool {
        false
    }

    pub fn hittable_component(&self) -> Option<&Drawable> {
        None
    }

    pub fn is_target_opaque(&self) -> bool {
        false
    }

    pub fn hit_test(&self, _info: &mut HitInfo, _transform: Mat2D) -> Option<&Core> {
        None
    }

    pub fn set_operation(&mut self, operation: Box<dyn ClippingShapeOperation>) {
        self.operation = operation;
    }

    pub fn is_proxy(&self) -> bool {
        true
    }

    pub fn is_clip_start(&self) -> bool {
        self.operation.is_start()
    }

    pub fn is_clip_end(&self) -> bool {
        !self.operation.is_start()
    }

    pub fn will_clip(&self) -> bool {
        self.operation.is_visible()
    }
}

pub struct ClippingShape {
    pub base: ClippingShapeBase,
    shapes: Vec<Shape>,
    proxy_drawables: Vec<ClippingShapeProxyDrawable>,
    pooled_proxy_drawables: Vec<ClippingShapeProxyDrawable>,
    source: Option<Node>,
    path: ShapePaintPath,
    clip_path: bool,
    pub clip_start: ClippingShapeStart,
    pub clip_end: ClippingShapeEnd,
}

impl ClippingShape {
    pub fn source(&self) -> Option<&Node> {
        self.source.as_ref()
    }

    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }

    pub fn on_added_clean(&mut self, _context: &mut CoreContext) -> StatusCode {
        self.base.parent_mut().for_all(|component| {
            if let Some(drawable) = component.as_drawable_mut() {
                drawable.add_clipping_shape(self);
            }
            true
        });
        if let Some(source) = self.source.as_mut() {
            source.for_all(|component| {
                if let Some(shape) = component.as_shape_mut() {
                    shape.add_flags(PathFlags::WORLD | PathFlags::CLIPPING);
                    self.shapes.push(shape.clone());
                }
                true
            });
        }
        StatusCode::Ok
    }

    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(source) = context
            .resolve(self.base.source_id())
            .and_then(Core::as_node)
        else {
            return StatusCode::MissingObject;
        };
        self.source = Some(source.clone());
        StatusCode::Ok
    }

    pub fn build_dependencies(&mut self) {
        for shape in &mut self.shapes {
            shape.path_composer_mut().add_dependent(&mut self.base);
        }
        let pointer = self as *mut ClippingShape;
        self.clip_start.set_clipping_shape(pointer);
        self.clip_end.set_clipping_shape(pointer);
    }

    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(
            value,
            ComponentDirt::PATH | ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER,
        ) {
            self.path.rewind_with_rule(false, self.base.fill_rule());
            self.clip_path = false;
            for shape in &mut self.shapes {
                if !shape.is_empty() {
                    if let Some(path) = shape.path_composer_mut().world_path_option() {
                        self.path.add_path(path, Mat2D::identity());
                        self.clip_path = true;
                    }
                }
            }
        }
    }

    pub fn is_visible_changed(&mut self) {
        self.base.artboard_mut().add_dirt(ComponentDirt::CLIPPING);
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
    ) -> &mut ClippingShapeProxyDrawable {
        let drawable = if let Some(mut drawable) = self.pooled_proxy_drawables.pop() {
            drawable.set_operation(operation);
            drawable.drawable.set_needs_save_operation(true);
            drawable
        } else {
            ClippingShapeProxyDrawable::new(operation)
        };
        self.proxy_drawables.push(drawable);
        self.proxy_drawables.last_mut().unwrap()
    }
}
