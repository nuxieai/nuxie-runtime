use crate::mechanical_port::source::{
    core::CoreHandle,
    core_context::CoreContext,
    generated::text::text_input_drawable_base::TextInputDrawableBase,
    math::mat2d::Mat2D,
    renderer::Renderer,
    shapes::{
        paint::{shape_paint::ShapePaintPathKind, shape_paint_path::ShapePaintPath},
        shape_paint_container::ShapePaintContainer,
    },
    status_code::StatusCode,
};

#[derive(Default)]
pub struct TextInputDrawable {
    pub base: TextInputDrawableBase,
    pub paints: ShapePaintContainer,
}
impl TextInputDrawable {
    pub fn text_input_handle(&self) -> CoreHandle {
        self.base
            .parent_handle()
            .expect("TextInputDrawable TextInput parent")
    }
    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        if self.base.parent_handle().is_some_and(|parent| {
            parent
                .is_type_of(crate::mechanical_port::source::generated::text::text_input_base::TextInputBase::TYPE_KEY)
        }) {
            StatusCode::Ok
        } else {
            StatusCode::InvalidObject
        }
    }
    pub fn shape_world_transform(&self) -> Mat2D {
        *self.base.world_transform()
    }
    pub fn path_builder(&self) -> CoreHandle {
        self.text_input_handle()
    }

    /// The concrete cursor/selection/text owner supplies its virtual path.
    /// Paths stay borrowed inside the parent TextInput occurrence.
    pub fn draw_with_path(
        &mut self,
        renderer: &mut Renderer,
        path: Option<&mut ShapePaintPath>,
        world: Mat2D,
    ) {
        let Some(path) = path else {
            return;
        };
        for handle in self.paints.shape_paints().iter().cloned() {
            handle.with_mut(|object| {
                let Some(paint) = object.as_shape_paint_behavior_mut() else {
                    return;
                };
                if !paint.is_visible() {
                    return;
                }
                if paint.pick_path_kind() == ShapePaintPathKind::World {
                    unreachable!("TextInputDrawable::worldPath is unreachable upstream");
                }
                let fill_rule = paint.fill_rule();
                paint
                    .shape_paint_mut()
                    .draw_with_fill_rule(renderer, path, world, false, None, true, fill_rule);
            });
        }
    }
    pub fn will_draw(&self) -> bool {
        self.base.will_draw() && self.base.render_opacity() != 0.0
    }
}
impl std::ops::Deref for TextInputDrawable {
    type Target = TextInputDrawableBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TextInputDrawable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
