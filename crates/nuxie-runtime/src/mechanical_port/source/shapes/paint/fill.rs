use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    generated::shapes::paint::fill_base::FillBase,
    shapes::{
        paint::{
            shape_paint::{ShapePaint, ShapePaintBehavior, ShapePaintPathKind, ShapePaintType},
            shape_paint_mutator::ShapePaintMutator,
        },
        path_flags::PathFlags,
    },
};
use nuxie_render_api::{RenderPaint, RenderPaintStyle};

#[derive(Default)]
pub struct Fill {
    pub base: FillBase,
}

impl Fill {
    pub fn update(&mut self, value: ComponentDirt) {
        let kind = self.pick_path_kind();
        let paint = crate::scripting::ScriptPaint::from_fresh(&self.base.base, None);
        self.base.base.update_with_path_kind(value, kind, paint);
    }
    pub fn path_flags(&self) -> PathFlags {
        if self.base.fill_rule() == nuxie_render_api::FillRule::Clockwise as u32 {
            PathFlags::LOCAL_CLOCKWISE
        } else {
            PathFlags::LOCAL
        }
    }

    pub fn init_render_paint(&mut self, mutator: CoreHandle) -> bool {
        if !self.base.init_render_paint(mutator) {
            return false;
        }
        self.base
            .with_render_paint_mut(|paint| paint.style(RenderPaintStyle::Fill));
        true
    }

    pub fn apply_to(&mut self, paint: &mut dyn RenderPaint, opacity: f32) {
        paint.style(RenderPaintStyle::Fill);
        paint.shader(None);
        let path_flags = self.path_flags();
        if let Some(mutator) = self.base.paint() {
            mutator.with_mut(|mutator| {
                if let Some(mutator) = mutator.as_shape_paint_mutator_mut() {
                    mutator.apply_to(paint, opacity, path_flags);
                }
            });
        }
    }

    pub fn build_dependencies(&mut self) {
        if self.base.effects_container.effects.is_empty() {
            return;
        }
        let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) else {
            return;
        };
        let builder = parent
            .with(|parent| parent.shape_paint_path_builder())
            .flatten();
        if let Some(builder) = builder {
            builder.with_component_mut(|component| component.add_dependent(this));
        }
    }
}

impl ShapePaintBehavior for Fill {
    fn is_visible(&self) -> bool {
        self.base.is_visible()
    }
    fn shape_paint(&self) -> &ShapePaint {
        &self.base.base
    }

    fn shape_paint_mut(&mut self) -> &mut ShapePaint {
        &mut self.base.base
    }

    fn path_flags(&self) -> PathFlags {
        Fill::path_flags(self)
    }

    fn paint_type(&self) -> ShapePaintType {
        ShapePaintType::Fill
    }

    fn pick_path_kind(&self) -> ShapePaintPathKind {
        if self.base.fill_rule() == nuxie_render_api::FillRule::Clockwise as u32 {
            ShapePaintPathKind::LocalClockwise
        } else {
            ShapePaintPathKind::Local
        }
    }

    fn fill_rule(&self) -> Option<u32> {
        Some(self.base.fill_rule())
    }

    fn initialize_render_paint(&mut self, mutator: CoreHandle) -> bool {
        self.init_render_paint(mutator)
    }

    fn apply_to(&mut self, paint: &mut dyn RenderPaint, opacity: f32) {
        Fill::apply_to(self, paint, opacity);
    }
}
