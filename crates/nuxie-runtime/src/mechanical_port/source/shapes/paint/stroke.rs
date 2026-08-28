pub use crate::mechanical_port::source::generated::shapes::paint::stroke_base::StrokeBase;

use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    core::CoreHandle,
    shapes::{
        paint::{
            shape_paint::{ShapePaint, ShapePaintBehavior, ShapePaintPathKind, ShapePaintType},
            shape_paint_mutator::ShapePaintMutator,
            stroke_cap::StrokeCap,
            stroke_join::StrokeJoin,
        },
        path_flags::PathFlags,
    },
};
use nuxie_render_api::{RenderPaint, RenderPaintStyle};

#[derive(Default)]
pub struct Stroke {
    pub base: StrokeBase,
}

impl Stroke {
    pub fn path_flags(&self) -> PathFlags {
        if self.base.transform_affects_stroke() {
            PathFlags::LOCAL
        } else {
            PathFlags::WORLD
        }
    }

    pub fn init_render_paint(&mut self, mutator: CoreHandle) -> bool {
        if !self.base.init_render_paint(mutator) {
            return false;
        }
        let thickness = self.base.thickness();
        let cap: nuxie_render_api::StrokeCap = StrokeCap::from(self.base.cap()).into();
        let join: nuxie_render_api::StrokeJoin = StrokeJoin::from(self.base.join()).into();
        self.base.with_render_paint_mut(|paint| {
            paint.style(RenderPaintStyle::Stroke);
            paint.thickness(thickness);
            paint.cap(cap);
            paint.join(join);
        });
        true
    }

    pub fn apply_to(&mut self, paint: &mut dyn RenderPaint, opacity: f32) {
        paint.style(RenderPaintStyle::Stroke);
        paint.thickness(self.base.thickness());
        paint.cap(StrokeCap::from(self.base.cap()).into());
        paint.join(StrokeJoin::from(self.base.join()).into());
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

    pub fn is_visible(&self) -> bool {
        self.base.base.base.is_visible() && self.base.thickness() > 0.0
    }

    pub fn thickness_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT, false);
    }

    pub fn cap_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT, false);
    }

    pub fn join_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::PAINT, false);
    }

    pub fn update(&mut self, value: ComponentDirt) {
        let kind = self.pick_path_kind();
        self.base.base.update_with_path_kind(value, kind);
        if has_dirt(value, ComponentDirt::PAINT) {
            let thickness = self.base.thickness();
            let cap = StrokeCap::from(self.base.cap()).into();
            let join = StrokeJoin::from(self.base.join()).into();
            self.base.with_render_paint_mut(|paint| {
                paint.thickness(thickness);
                paint.cap(cap);
                paint.join(join);
            });
        }
    }

    pub fn invalidate_rendering(&mut self) {
        self.base
            .with_render_paint_mut(nuxie_render_api::RenderPaint::invalidate_stroke);
        self.base.invalidate_rendering();
    }

    pub fn build_dependencies(&mut self) {
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

impl ShapePaintBehavior for Stroke {
    fn is_visible(&self) -> bool {
        Stroke::is_visible(self)
    }
    fn shape_paint(&self) -> &ShapePaint {
        &self.base.base
    }

    fn shape_paint_mut(&mut self) -> &mut ShapePaint {
        &mut self.base.base
    }

    fn path_flags(&self) -> PathFlags {
        Stroke::path_flags(self)
    }

    fn paint_type(&self) -> ShapePaintType {
        ShapePaintType::Stroke
    }

    fn pick_path_kind(&self) -> ShapePaintPathKind {
        if self.base.transform_affects_stroke() {
            ShapePaintPathKind::Local
        } else {
            ShapePaintPathKind::World
        }
    }

    fn initialize_render_paint(&mut self, mutator: CoreHandle) -> bool {
        self.init_render_paint(mutator)
    }

    fn apply_to(&mut self, paint: &mut dyn RenderPaint, opacity: f32) {
        Stroke::apply_to(self, paint, opacity);
    }
}
