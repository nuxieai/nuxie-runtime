use super::{text::Text, text_style_paint::TextStylePaint, utf::Utf};
use crate::mechanical_port::source::{
    component::Component,
    core::CoreHandle,
    core_context::CoreContext,
    generated::text::text_value_run_base::{TextValueRunBase, TextValueRunBaseCallbacks},
    hittest_command_path::HitTestCommandPath,
    math::{aabb::Aabb, mat2d::Mat2D, rectangles_to_contour::RectanglesToContour, vec2d::Vec2D},
    status_code::StatusCode,
};
pub struct TextValueRun {
    pub base: TextValueRunBase,
    rectangles: Option<Box<RectanglesToContour>>,
    local_bounds: Aabb,
    is_hit_target: bool,
    glyph_hit_rects: Vec<Aabb>,
    style: Option<CoreHandle>,
    length: u32,
    text_component: Option<CoreHandle>,
}

impl Default for TextValueRun {
    fn default() -> Self {
        Self {
            base: TextValueRunBase::default(),
            rectangles: None,
            local_bounds: Aabb::default(),
            is_hit_target: false,
            glyph_hit_rects: Vec::new(),
            style: None,
            length: u32::MAX,
            text_component: None,
        }
    }
}

impl TextValueRunBaseCallbacks for TextValueRun {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn style_id_changed(&mut self) {
        Self::style_id_changed(self);
    }

    fn text_changed(&mut self) {
        Self::text_changed(self);
    }
}

impl TextValueRun {
    pub fn set_bound_text(&mut self, value: String) {
        if !self.base.set_text_value(value) {
            return;
        }
        self.text_changed();
        self.base
            .base
            .base
            .base
            .notify_property_changed(TextValueRunBase::TEXT_PROPERTY_KEY);
    }
    pub fn text_changed(&mut self) {
        self.length = u32::MAX;
        if let Some(text) = self.text_component() {
            text.with_mut(|text| {
                if let Some(text) = text.as_text_mut() {
                    text.mark_shape_dirty();
                }
            });
        }
    }
    pub fn text_component(&self) -> Option<CoreHandle> {
        self.text_component.clone().or_else(|| {
            self.base.parent_handle().filter(|parent| {
                parent
                    .with(|parent| parent.as_text().is_some())
                    .unwrap_or(false)
            })
        })
    }
    pub fn set_text_component(&mut self, value: CoreHandle) {
        self.text_component = Some(value);
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        let (Some(text), Some(this)) = (self.text_component(), self.base.handle()) else {
            return StatusCode::MissingObject;
        };
        let added = text
            .with_mut(|text| text.as_text_mut().map(|text| text.add_run(this)))
            .flatten()
            .is_some();
        if !added {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(style) = context.resolve(self.base.style_id()).filter(|style| {
            style
                .with(|style| style.as_text_style().is_some())
                .unwrap_or(false)
        }) else {
            return StatusCode::MissingObject;
        };
        self.style = Some(style);
        StatusCode::Ok
    }
    pub fn style_id_changed(&mut self) {
        if let Some(style) = self
            .base
            .with_artboard(|artboard| {
                artboard.resolve(self.base.style_id()).filter(|style| {
                    style
                        .with(|style| style.as_text_style().is_some())
                        .unwrap_or(false)
                })
            })
            .flatten()
        {
            self.style = Some(style);
            if let Some(text) = self.text_component() {
                text.with_mut(|text| {
                    if let Some(text) = text.as_text_mut() {
                        text.mark_shape_dirty();
                    }
                });
            }
        }
    }
    pub fn style(&self) -> Option<CoreHandle> {
        self.style.clone()
    }
    pub fn set_style(&mut self, value: CoreHandle) {
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
        let Some(this) = self.base.handle() else {
            return 0;
        };
        let mut offset = 0;
        let runs = self
            .text_component()
            .and_then(|text| text.with(|text| text.as_text().map(|text| text.runs().to_vec())))
            .flatten()
            .unwrap_or_default();
        for run in runs {
            if run == this {
                break;
            }
            offset += run
                .with_mut(|run| run.as_text_value_run_mut().map(TextValueRun::length))
                .flatten()
                .unwrap_or(0);
        }
        offset
    }
    fn can_hit_test(&self) -> bool {
        self.is_hit_target
            && (self.text_component.is_some() || self.base.parent_as_text().is_some())
            && !self.local_bounds.is_empty_or_nan()
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
        self.text_component()
            .and_then(|text| {
                text.with(|text| {
                    let text = text.as_text()?;
                    if !text.overflow_visible() {
                        let inv = text.world_transform().inverse()?;
                        if !text.local_bounds().contains(inv * position) {
                            return Some(false);
                        }
                    }
                    let world = *text.world_transform() * text.internal_transform();
                    Some(
                        world
                            .inverse()
                            .is_some_and(|inv| self.local_bounds.contains(inv * position)),
                    )
                })
            })
            .flatten()
            .flatten()
            .unwrap_or(false)
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
        let Some(transform) = self
            .text_component()
            .and_then(|text| {
                text.with(|text| {
                    text.as_text()
                        .map(|text| *text.world_transform() * text.internal_transform())
                })
            })
            .flatten()
        else {
            return false;
        };
        let mut tester = HitTestCommandPath::new(area);
        tester.set_xform(transform);
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
