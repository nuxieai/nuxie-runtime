//! Translation of text_style_background.hpp/.cpp at upstream 1f04919a.

use crate::mechanical_port::source::{
    core::CoreHandle,
    core_context::CoreContext,
    generated::text::text_style_background_base::{
        TextStyleBackgroundBase, TextStyleBackgroundBaseCallbacks,
    },
    math::{aabb::Aabb, mat2d::Mat2D},
    shapes::{paint::shape_paint_path::ShapePaintPath, shape_paint_container::ShapePaintContainer},
    status_code::StatusCode,
    text::{text_selection_path::TextSelectionPath, text_style_paint::TextStylePaint},
};
use nuxie_render_api::{BlendMode, FillRule, Renderer};

pub struct TextStyleBackground {
    pub base: TextStyleBackgroundBase,
    pub paints: ShapePaintContainer,
    rects: Vec<Aabb>,
    path: TextSelectionPath,
}

impl Default for TextStyleBackground {
    fn default() -> Self {
        Self {
            base: TextStyleBackgroundBase::default(),
            paints: ShapePaintContainer::default(),
            rects: Vec::new(),
            // Upstream intentionally makes background contours winding-agnostic.
            path: TextSelectionPath::new(true, FillRule::EvenOdd),
        }
    }
}

impl TextStyleBackground {
    pub const TYPE_KEY: u16 = TextStyleBackgroundBase::TYPE_KEY;

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code == StatusCode::Ok {
            let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) else {
                return StatusCode::InvalidObject;
            };
            if parent
                .with_downcast_mut::<TextStylePaint, _>(|style| style.set_background(this))
                .is_none()
            {
                return StatusCode::InvalidObject;
            }
        }
        code
    }

    fn style(&self) -> CoreHandle {
        self.base
            .parent_handle()
            .expect("TextStyleBackground parent")
    }

    pub fn reset_path(&mut self) {
        self.rects.clear();
        self.path.path.rewind();
    }

    pub fn add_rect(&mut self, rect: Aabb) {
        self.rects.push(rect);
    }

    pub fn update_path(&mut self) {
        self.path.update(&self.rects, self.base.corner_radius());
    }

    pub fn propagate_opacity(&mut self, opacity: f32) {
        self.paints.propagate_opacity(opacity);
    }

    // Text supplies its already-borrowed transform/blend, as TextStylePaint does.
    // Reading Text through the parent chain here would reborrow that same owner.
    pub fn draw(&mut self, renderer: &mut dyn Renderer, world: &Mat2D, blend: BlendMode) {
        if self.rects.is_empty() {
            return;
        }
        for handle in self.paints.shape_paints().iter().cloned() {
            handle.with_mut(|object| {
                let Some(paint) = object.as_shape_paint_behavior_mut() else {
                    return;
                };
                if !paint.should_draw() {
                    return;
                }
                let fill_rule = paint.fill_rule();
                paint.shape_paint_mut().blend_mode(blend);
                paint.shape_paint_mut().draw_with_fill_rule(
                    renderer,
                    &mut self.path.path,
                    *world,
                    true,
                    None,
                    true,
                    fill_rule,
                );
            });
        }
    }

    pub fn shape_world_transform(&self) -> Mat2D {
        self.style()
            .with_downcast::<TextStylePaint, _>(TextStylePaint::shape_world_transform)
            .expect("TextStyleBackground parent is TextStylePaint")
    }

    pub fn path_builder(&self) -> CoreHandle {
        self.style()
            .with_downcast::<TextStylePaint, _>(TextStylePaint::path_builder)
            .expect("TextStyleBackground parent is TextStylePaint")
    }

    pub fn local_path(&mut self) -> &mut ShapePaintPath {
        &mut self.path.path
    }

    pub fn local_clockwise_path(&mut self) -> &mut ShapePaintPath {
        &mut self.path.path
    }

    pub fn corner_radius_changed(&mut self) {
        let text = self.base.parent_handle().and_then(|parent| {
            parent
                .with(|parent| {
                    parent
                        .as_component()
                        .and_then(|parent| parent.parent_handle())
                })
                .flatten()
        });
        if let Some(text) = text {
            text.with_mut(|text| {
                if let Some(text) = text.as_text_mut() {
                    text.mark_paint_dirty();
                }
            });
        }
    }

    pub fn set_corner_radius(&mut self, value: f32) {
        if self.base.set_corner_radius_value(value) {
            self.corner_radius_changed();
            TextStyleBackgroundBaseCallbacks::notify_property_changed(
                self,
                TextStyleBackgroundBase::CORNER_RADIUS_PROPERTY_KEY,
            );
        }
    }

    pub fn clone_value(&self) -> Box<Self> {
        Box::new(self.base.clone_into(&mut Self::default()))
    }
}

impl std::ops::Deref for TextStyleBackground {
    type Target = TextStyleBackgroundBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextStyleBackground {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
