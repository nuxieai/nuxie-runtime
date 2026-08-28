use crate::mechanical_port::source::{
    component_dirt::{ComponentDirt, has_dirt},
    core::CoreHandle,
    core_context::{CoreContext, StatusCode},
    generated::shapes::paint::linear_gradient_base::LinearGradientBase,
    layout::n_sliced_node::NSlicedNode,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    renderer::RenderPaint,
    shapes::{
        paint::{
            color::{ColorInt, color_modulate_opacity, color_opacity},
            gradient_stop::GradientStop,
            shape_paint_mutator::{MutatorFlags, ShapePaintMutator, ShapePaintMutatorState},
        },
        path_flags::PathFlags,
    },
};

#[derive(Clone, Copy)]
pub(crate) enum GradientKind {
    Linear,
    Radial,
}

#[derive(Default)]
pub struct LinearGradient {
    pub base: LinearGradientBase,
    stops: Vec<CoreHandle>,
    shape_paint_container: Option<CoreHandle>,
    deformer: Option<CoreHandle>,
    color_storage: Vec<ColorInt>,
    pub(crate) mutator: ShapePaintMutatorState,
}

impl LinearGradient {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        self.init_paint_mutator(this, self.base.parent_handle())
    }

    pub fn build_dependencies(&mut self) {
        let grand_parent = self.base.parent_handle().and_then(|parent| {
            parent
                .with(|parent| parent.component_parent_handle())
                .flatten()
        });
        if let Some(grand_parent) = grand_parent {
            assert!(
                grand_parent
                    .with(|parent| parent.as_shape_paint_container().is_some())
                    .unwrap_or(false)
            );
            self.shape_paint_container = None;
            let mut container = Some(grand_parent.clone());
            while let Some(candidate) = container {
                if candidate
                    .with(|candidate| candidate.as_node().is_some())
                    .unwrap_or(false)
                {
                    self.shape_paint_container = Some(candidate);
                    break;
                }
                container = candidate
                    .with(|candidate| candidate.component_parent_handle())
                    .flatten();
            }
            let dependency = self.shape_paint_container.as_ref().unwrap_or(&grand_parent);
            if let Some(this) = self.base.handle() {
                dependency.with_mut(|dependency| dependency.component_add_dependent(this));
            }
        }
        self.update_deformer();
    }

    fn update_deformer(&mut self) {
        if let Some(container) = self.shape_paint_container.as_ref() {
            if let Some(deformer) = container
                .with(|container| container.as_shape().and_then(|shape| shape.deformer()))
                .flatten()
            {
                self.deformer = deformer.with_downcast::<NSlicedNode, _>(|_| deformer.clone());
            }
        }
    }

    pub fn add_stop(&mut self, stop: CoreHandle) {
        self.stops.push(stop);
    }

    fn paints_in_world_space(&self) -> bool {
        self.base
            .parent_handle()
            .and_then(|parent| {
                parent
                    .with(|parent| {
                        parent
                            .as_shape_paint_behavior()
                            .map(|paint| !(paint.path_flags() & PathFlags::WORLD).is_empty())
                    })
                    .flatten()
            })
            .expect("gradient parent is a ShapePaint")
    }

    pub fn update(&mut self, value: ComponentDirt) {
        self.update_with_kind(value, GradientKind::Linear);
    }

    pub(crate) fn update_with_kind(&mut self, value: ComponentDirt, kind: GradientKind) {
        if has_dirt(value, ComponentDirt::STOPS) {
            self.stops.sort_by(|a, b| {
                let a = a
                    .with_downcast::<GradientStop, _>(|stop| stop.base.position())
                    .expect("gradient stop");
                let b = b
                    .with_downcast::<GradientStop, _>(|stop| stop.base.position())
                    .expect("gradient stop");
                a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        let rebuild = has_dirt(
            value,
            ComponentDirt::PAINT
                | ComponentDirt::RENDER_OPACITY
                | ComponentDirt::TRANSFORM
                | ComponentDirt::N_SLICER,
        ) || (self.paints_in_world_space()
            && has_dirt(value, ComponentDirt::WORLD_TRANSFORM));
        if rebuild {
            let paint = self
                .mutator
                .render_paint_handle()
                .expect("initialized gradient paint");
            self.apply_to_with_kind(
                paint.borrow_mut().as_mut(),
                1.0,
                kind,
                self.paints_in_world_space(),
            );
            self.mutator.flags = MutatorFlags::NONE;
            for color in &self.color_storage {
                let opacity = color_opacity(*color);
                if opacity > 0.0 {
                    self.mutator.flags |= MutatorFlags::VISIBLE;
                }
                if opacity < 1.0 {
                    self.mutator.flags |= MutatorFlags::TRANSLUCENT;
                }
            }
        }
    }

    pub fn apply_to(&mut self, paint: &mut RenderPaint, opacity: f32) {
        self.apply_to_with_kind(
            paint,
            opacity,
            GradientKind::Linear,
            self.paints_in_world_space(),
        );
    }

    pub(crate) fn apply_to_with_kind(
        &mut self,
        paint: &mut RenderPaint,
        modifier: f32,
        kind: GradientKind,
        paints_in_world_space: bool,
    ) {
        let mut start = Vec2D::new(self.base.start_x(), self.base.start_y());
        let mut end = Vec2D::new(self.base.end_x(), self.base.end_y());
        let world = self.shape_paint_container.as_ref().and_then(|container| {
            container
                .with(|container| {
                    container
                        .as_world_transform_component()
                        .map(|container| *container.world_transform())
                })
                .flatten()
        });
        if paints_in_world_space {
            if let Some(world) = world {
                start = world * start;
                end = world * end;
                if let Some(deformer) = self.deformer.as_ref() {
                    deformer.with_downcast::<NSlicedNode, _>(|deformer| {
                        start = deformer.deform_world_point(start);
                        end = deformer.deform_world_point(end);
                    });
                }
            }
        } else if let (Some(deformer), Some(world)) = (self.deformer.as_ref(), world) {
            let mut inverse = Mat2D::identity();
            if world.invert(&mut inverse) {
                deformer.with_downcast::<NSlicedNode, _>(|deformer| {
                    start = deformer.deform_local_point(start, &world, &inverse);
                    end = deformer.deform_local_point(end, &world, &inverse);
                });
            }
        }
        let opacity = self.base.opacity() * self.render_opacity() * modifier;
        self.color_storage.resize(self.stops.len(), 0);
        // C++ overlays the stop floats after its color array; two owned Rust
        // vectors preserve the values without type-punning live storage.
        let mut positions = vec![0.0; self.stops.len()];
        for (index, stop) in self.stops.iter().enumerate() {
            let (color, position) = stop
                .with_downcast::<GradientStop, _>(|stop| {
                    (stop.base.color_value() as u32, stop.base.position())
                })
                .expect("gradient stop");
            self.color_storage[index] = color_modulate_opacity(color, opacity);
            // std::max(0, std::min(position, 1)) returns 0 for NaN.
            positions[index] = if position.is_nan() {
                0.0
            } else {
                position.clamp(0.0, 1.0)
            };
        }
        self.make_gradient_with_kind(paint, start, end, &self.color_storage, &positions, kind);
    }

    pub fn make_gradient(
        &self,
        paint: &mut RenderPaint,
        start: Vec2D,
        end: Vec2D,
        colors: &[ColorInt],
        stops: &[f32],
    ) {
        self.make_gradient_with_kind(paint, start, end, colors, stops, GradientKind::Linear);
    }

    pub(crate) fn make_gradient_with_kind(
        &self,
        paint: &mut RenderPaint,
        start: Vec2D,
        end: Vec2D,
        colors: &[ColorInt],
        stops: &[f32],
        kind: GradientKind,
    ) {
        let factory = self
            .base
            .with_artboard(|artboard| artboard.factory())
            .flatten()
            .expect("imported gradient factory");
        let shader = factory.with_factory_mut(|factory| match kind {
            GradientKind::Linear => {
                factory.make_linear_gradient(start.x, start.y, end.x, end.y, colors, stops)
            }
            GradientKind::Radial => factory.make_radial_gradient(
                start.x,
                start.y,
                Vec2D::distance(start, end),
                colors,
                stops,
            ),
        });
        paint.shader(Some(shader.as_ref()));
    }

    pub fn mark_gradient_dirty(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT, false);
    }
    pub fn mark_stops_dirty(&mut self) {
        self.base
            .add_dirt(ComponentDirt::PAINT | ComponentDirt::STOPS, false);
    }
    pub fn render_opacity_changed(&mut self) {
        self.mark_gradient_dirty();
    }
    pub fn start_x_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::TRANSFORM, false);
    }
    pub fn start_y_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::TRANSFORM, false);
    }
    pub fn end_x_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::TRANSFORM, false);
    }
    pub fn end_y_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::TRANSFORM, false);
    }
    pub fn opacity_changed(&mut self) {
        self.mark_gradient_dirty();
    }
}
impl ShapePaintMutator for LinearGradient {
    fn mutator_state(&self) -> &ShapePaintMutatorState {
        &self.mutator
    }
    fn mutator_state_mut(&mut self) -> &mut ShapePaintMutatorState {
        &mut self.mutator
    }
    fn render_opacity_changed(&mut self) {
        LinearGradient::render_opacity_changed(self);
    }
    fn apply_to(&mut self, paint: &mut RenderPaint, opacity: f32, flags: PathFlags) {
        self.apply_to_with_kind(
            paint,
            opacity,
            GradientKind::Linear,
            !(flags & PathFlags::WORLD).is_empty(),
        );
    }
}
impl std::ops::Deref for LinearGradient {
    type Target = LinearGradientBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for LinearGradient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
