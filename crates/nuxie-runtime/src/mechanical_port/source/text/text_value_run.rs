use super::{text::Text, text_style_paint::TextStylePaint, utf::Utf};
use crate::mechanical_port::source::{
    component::Component,
    core_context::CoreContext,
    generated::text::text_value_run_base::TextValueRunBase,
    hittest_command_path::HitTestCommandPath,
    math::{aabb::Aabb, mat2d::Mat2D, rectangles_to_contour::RectanglesToContour, vec2d::Vec2D},
    status_code::StatusCode,
};
use std::ptr::NonNull;
pub struct TextValueRun {
    pub base: TextValueRunBase,
    rectangles: Option<Box<RectanglesToContour>>,
    local_bounds: Aabb,
    is_hit_target: bool,
    glyph_hit_rects: Vec<Aabb>,
    style: Option<NonNull<TextStylePaint>>,
    length: u32,
    text_component: Option<NonNull<Text>>,
}
impl TextValueRun {
    pub fn text_changed(&mut self) {
        self.length = u32::MAX;
        unsafe { self.text_component().as_mut() }.mark_shape_dirty();
    }
    pub fn text_component(&self) -> NonNull<Text> {
        self.text_component.unwrap_or_else(|| {
            self.base
                .parent_as_text()
                .expect("TextValueRun Text parent")
        })
    }
    pub fn set_text_component(&mut self, value: NonNull<Text>) {
        self.text_component = Some(value);
    }
    pub fn on_added_clean(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(mut text) = self.base.parent_as_text() else {
            return StatusCode::MissingObject;
        };
        unsafe { text.as_mut() }.add_run(self);
        StatusCode::Ok
    }
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(style) = context
            .resolve(self.base.style_id())
            .and_then(|value| value.as_text_style_paint())
        else {
            return StatusCode::MissingObject;
        };
        self.style = Some(style);
        StatusCode::Ok
    }
    pub fn style_id_changed(&mut self) {
        if let Some(style) = self
            .base
            .artboard()
            .resolve(self.base.style_id())
            .and_then(|value| value.as_text_style_paint())
        {
            self.style = Some(style);
            unsafe { self.text_component().as_mut() }.mark_shape_dirty();
        }
    }
    pub fn style(&self) -> Option<NonNull<TextStylePaint>> {
        self.style
    }
    pub fn set_style(&mut self, value: NonNull<TextStylePaint>) {
        self.style = Some(value);
    }
    pub fn length(&mut self) -> u32 {
        if self.length == u32::MAX {
            let mut bytes = self.base.text().as_bytes();
            let mut n = 0;
            while !bytes.is_empty() && bytes[0] != 0 {
                Utf::next_utf8(&mut bytes);
                n += 1;
            }
            self.length = n;
        }
        self.length
    }
    pub fn offset(&mut self) -> u32 {
        #[cfg(feature = "rive_text")]
        {
            let this = self as *const _;
            let mut offset = 0;
            for mut run in unsafe { self.text_component().as_ref() }
                .runs()
                .iter()
                .copied()
            {
                if run.as_ptr() == this.cast_mut() {
                    break;
                }
                offset += unsafe { run.as_mut() }.length();
            }
            offset
        }
        #[cfg(not(feature = "rive_text"))]
        {
            0
        }
    }
    fn can_hit_test(&self) -> bool {
        self.is_hit_target && !self.local_bounds.is_empty_or_nan()
    }
    pub fn reset_hit_test(&mut self) {
        self.glyph_hit_rects.clear();
        self.local_bounds = Aabb::for_expansion();
    }
    pub fn add_hit_rect(&mut self, rect: Aabb) {
        Aabb::expand_to(&mut self.local_bounds, rect.min());
        Aabb::expand_to(&mut self.local_bounds, rect.max());
        self.glyph_hit_rects.push(rect);
    }
    pub fn compute_hit_contours(&mut self) {
        let contour = self
            .rectangles
            .get_or_insert_with(|| Box::new(RectanglesToContour::default()));
        contour.reset();
        for rect in &self.glyph_hit_rects {
            contour.add_rect(*rect);
        }
        contour.compute_contours();
    }
    pub fn hit_test_aabb(&self, position: Vec2D) -> bool {
        if !self.can_hit_test() {
            return false;
        }
        let text = unsafe { self.text_component().as_ref() };
        if !text.overflow_visible() {
            let Some(inv) = text.world_transform().inverse() else {
                return false;
            };
            if !text.local_bounds().contains(inv * position) {
                return false;
            }
        }
        let world = *text.world_transform() * text.internal_transform();
        world
            .inverse()
            .is_some_and(|inv| self.local_bounds.contains(inv * position))
    }
    pub fn hit_test_hi_fi(&self, position: Vec2D, radius: f32) -> bool {
        if !self.can_hit_test() {
            return false;
        }
        let area = Aabb::new(
            position.x - radius,
            position.y - radius,
            position.x + radius,
            position.y + radius,
        )
        .round();
        let text = unsafe { self.text_component().as_ref() };
        let mut tester = HitTestCommandPath::new(area);
        tester.set_xform(*text.world_transform() * text.internal_transform());
        for contour in self
            .rectangles
            .as_ref()
            .expect("computed hit contours")
            .iter()
        {
            let mut points = contour.iter();
            let first = *points.next().expect("non-empty hit contour");
            tester.move_to(first.x, first.y);
            for point in points {
                tester.line_to(point.x, point.y);
            }
            tester.close();
        }
        tester.was_hit()
    }
    pub fn set_is_hit_target(&mut self, value: bool) {
        self.is_hit_target = value;
    }
    pub fn is_hit_target(&self) -> bool {
        self.is_hit_target
    }
    pub fn hit_test_point(&self, position: Vec2D, skip: bool, primary: bool) -> bool {
        self.hit_test_aabb(position)
            && self.base.component_hit_test_point(position, skip, primary)
            && self.hit_test_hi_fi(position, 2.0)
    }
}
