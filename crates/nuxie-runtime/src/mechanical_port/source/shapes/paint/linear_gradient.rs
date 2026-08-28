use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    core::{CoreContext, CoreHandle, StatusCode},
    math::{mat2d::Mat2D, vec2d::Vec2D},
    node::Node,
    renderer::RenderPaint,
    shapes::{
        deformer::PointDeformer,
        paint::{
            color::{ColorInt, color_modulate_opacity, color_opacity},
            gradient_stop::GradientStop,
            shape_paint::ShapePaint,
            shape_paint_mutator::{MutatorFlags, ShapePaintMutator},
        },
        path_flags::PathFlags,
        shape_paint_container::ShapePaintContainer,
    },
};

pub struct LinearGradient {
    pub base: LinearGradientBase,
    stops: Vec<CoreHandle>,
    shape_paint_container: Option<Node>,
    deformer: Option<PointDeformer>,
    color_storage: Vec<ColorInt>,
    flags: MutatorFlags,
}

impl LinearGradient {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.base.init_paint_mutator(self)
    }

    pub fn build_dependencies(&mut self) {
        if let Some(parent) = self.base.parent().and_then(|parent| parent.parent()) {
            let grand_parent = ShapePaintContainer::from_component(parent)
                .expect("LinearGradient grandparent must be a ShapePaintContainer");
            self.shape_paint_container = grand_parent.first_node_ancestor();
            if let Some(node) = self.shape_paint_container.as_mut() {
                node.add_dependent(self);
            } else {
                grand_parent.add_dependent(self);
            }
        }
        self.update_deformer();
    }

    fn update_deformer(&mut self) {
        let Some(container) = self.shape_paint_container.as_ref() else {
            return;
        };
        if let Some(shape) = container.as_shape() {
            if let Some(deformer) = shape.deformer() {
                self.deformer = PointDeformer::from_component(deformer.as_component());
            }
        }
    }

    pub fn add_stop(&mut self, stop: CoreHandle) {
        self.stops.push(stop);
    }

    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::STOPS) {
            self.stops.sort_by(|a, b| {
                let a = a
                    .with_downcast::<GradientStop, _>(|stop| stop.base.position())
                    .unwrap_or_default();
                let b = b
                    .with_downcast::<GradientStop, _>(|stop| stop.base.position())
                    .unwrap_or_default();
                a.total_cmp(&b)
            });
        }
        let paints_in_world_space = self
            .base
            .parent_as::<ShapePaint>()
            .is_flagged(PathFlags::WORLD);
        let rebuild_gradient = has_dirt(
            value,
            ComponentDirt::PAINT
                | ComponentDirt::RENDER_OPACITY
                | ComponentDirt::TRANSFORM
                | ComponentDirt::N_SLICER,
        ) || (paints_in_world_space
            && has_dirt(value, ComponentDirt::WORLD_TRANSFORM));
        if rebuild_gradient {
            self.apply_to(self.base.render_paint(), 1.0);
            self.flags = MutatorFlags::NONE;
            for color in self.color_storage.iter().take(self.stops.len()) {
                let opacity = color_opacity(*color);
                if opacity > 0.0 {
                    self.flags |= MutatorFlags::VISIBLE;
                }
                if opacity < 1.0 {
                    self.flags |= MutatorFlags::TRANSLUCENT;
                }
            }
        }
    }

    pub fn apply_to(&mut self, render_paint: &mut RenderPaint, opacity_modifier: f32) {
        let paints_in_world_space = self
            .base
            .parent_as::<ShapePaint>()
            .is_flagged(PathFlags::WORLD);
        let mut start = Vec2D::new(self.base.start_x(), self.base.start_y());
        let mut end = Vec2D::new(self.base.end_x(), self.base.end_y());
        if paints_in_world_space {
            if let Some(container) = self.shape_paint_container.as_ref() {
                let world = container.world_transform();
                start = world * start;
                end = world * end;
                if let Some(deformer) = self.deformer.as_ref() {
                    start = deformer.deform_world_point(start);
                    end = deformer.deform_world_point(end);
                }
            }
        } else if let (Some(deformer), Some(container)) =
            (self.deformer.as_ref(), self.shape_paint_container.as_ref())
        {
            let world = container.world_transform();
            if let Some(inverse_world) = Mat2D::invert(world) {
                start = deformer.deform_local_point(start, world, inverse_world);
                end = deformer.deform_local_point(end, world, inverse_world);
            }
        }

        let opacity = self.base.opacity() * self.base.render_opacity() * opacity_modifier;
        self.color_storage.resize(self.stops.len(), 0);
        let mut positions = vec![0.0; self.stops.len()];
        for (index, stop) in self.stops.iter().enumerate() {
            let (color, position) = stop
                .with_downcast::<GradientStop, _>(|stop| {
                    (stop.base.color_value(), stop.base.position())
                })
                .unwrap_or_default();
            self.color_storage[index] = color_modulate_opacity(color, opacity);
            positions[index] = position.clamp(0.0, 1.0);
        }
        self.make_gradient(render_paint, start, end, &self.color_storage, &positions);
    }

    pub fn make_gradient(
        &self,
        render_paint: &mut RenderPaint,
        start: Vec2D,
        end: Vec2D,
        colors: &[ColorInt],
        stops: &[f32],
    ) {
        let factory = self
            .base
            .artboard()
            .factory()
            .expect("LinearGradient requires its Artboard renderer factory");
        let shader = factory.with_factory_mut(|factory| {
            factory.make_linear_gradient(start.x, start.y, end.x, end.y, colors, stops)
        });
        render_paint.shader(Some(shader.as_ref()));
    }

    pub fn mark_gradient_dirty(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT);
    }

    pub fn mark_stops_dirty(&mut self) {
        self.base
            .add_dirt(ComponentDirt::PAINT | ComponentDirt::STOPS);
    }

    pub fn render_opacity_changed(&mut self) {
        self.mark_gradient_dirty();
    }

    pub fn start_x_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::TRANSFORM);
    }

    pub fn start_y_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::TRANSFORM);
    }

    pub fn end_x_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::TRANSFORM);
    }

    pub fn end_y_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::TRANSFORM);
    }

    pub fn opacity_changed(&mut self) {
        self.mark_gradient_dirty();
    }
}
