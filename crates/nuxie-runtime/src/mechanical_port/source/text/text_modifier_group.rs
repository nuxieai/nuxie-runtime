use super::{
    glyph_lookup::GlyphLookup, text::Text, text_follow_path_modifier::TextFollowPathModifier,
    text_modifier::TextModifier, text_modifier_flags::TextModifierFlags,
    text_modifier_range::TextModifierRange, text_shape_modifier::TextShapeModifier,
};
use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core_context::CoreContext,
    generated::text::text_modifier_group_base::TextModifierGroupBase,
    math::{mat2d::Mat2D, transform_components::TransformComponents, vec2d::Vec2D},
    refcnt::RiveRc,
    status_code::StatusCode,
    text_engine::{Font, FontCoord, GlyphLine, Paragraph, StyledText, TextRun},
};
use std::{collections::HashMap, ptr::NonNull};
pub struct TransformGlyphArg<'a> {
    pub position: Vec2D,
    pub origin_position: Vec2D,
    pub offset: Vec2D,
    pub center_x: f32,
    pub line_index_in_paragraph: i32,
    pub paragraph_lines: &'a [GlyphLine],
}
impl<'a> TransformGlyphArg<'a> {
    pub fn new(position: Vec2D, center_x: f32, line: i32, lines: &'a [GlyphLine]) -> Self {
        Self {
            position,
            origin_position: Vec2D::new(position.x + center_x, position.y),
            offset: Vec2D::new(0.0, 0.0),
            center_x,
            line_index_in_paragraph: line,
            paragraph_lines: lines,
        }
    }
}
pub struct TextModifierGroup {
    pub base: TextModifierGroupBase,
    ranges: Vec<NonNull<TextModifierRange>>,
    modifiers: Vec<NonNull<TextModifier>>,
    shape_modifiers: Vec<NonNull<dyn TextShapeModifier>>,
    follow_path_modifiers: Vec<NonNull<TextFollowPathModifier>>,
    coverage: Vec<f32>,
    variable_font: Option<RiveRc<Font>>,
    variation_coords: Vec<FontCoord>,
    next_text_runs: Vec<TextRun>,
}
impl TextModifierGroup {
    pub fn text_component(&self) -> Option<NonNull<Text>> {
        self.base.parent_as_text()
    }
    pub fn on_added_dirty(&mut self, c: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(c);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(mut text) = self.text_component() else {
            return StatusCode::MissingObject;
        };
        unsafe { text.as_mut() }.add_modifier_group(self);
        StatusCode::Ok
    }
    pub fn add_modifier_range(&mut self, r: &mut TextModifierRange) {
        self.ranges.push(NonNull::from(r));
    }
    pub fn add_modifier(&mut self, m: &mut TextModifier) {
        let ptr = NonNull::from(&mut *m);
        self.modifiers.push(ptr);
        if let Some(shape) = m.base.as_text_shape_modifier() {
            self.shape_modifiers.push(shape);
        }
        if let Some(path) = m.base.as_text_follow_path_modifier() {
            self.follow_path_modifiers.push(path);
        }
    }
    pub fn range_type_changed(&mut self) {
        self.base.parent_text_mut().modifier_shape_dirty();
        self.base.add_dirt(ComponentDirt::TEXT_COVERAGE);
    }
    pub fn shape_modifier_changed(&mut self) {
        self.base.parent_text_mut().mark_shape_dirty();
    }
    pub fn range_changed(&mut self) {
        if self.shape_modifiers.is_empty() {
            self.base.parent_text_mut().mark_paint_dirty();
        } else {
            self.base.parent_text_mut().modifier_shape_dirty();
        }
        self.base.add_dirt(ComponentDirt::TEXT_COVERAGE);
    }
    pub fn clear_range_maps(&mut self) {
        for r in &mut self.ranges {
            unsafe { r.as_mut() }.clear_range_map();
        }
        self.base.add_dirt(ComponentDirt::TEXT_COVERAGE);
    }
    pub fn compute_range_map(
        &mut self,
        text: &[u32],
        shape: &[Paragraph],
        lines: &[Vec<GlyphLine>],
        lookup: &GlyphLookup,
    ) {
        for r in &mut self.ranges {
            unsafe { r.as_mut() }.compute_range(text, shape, lines, lookup);
        }
    }
    pub fn compute_coverage(&mut self, size: u32) {
        if !self.base.has_dirt(ComponentDirt::TEXT_COVERAGE) {
            return;
        }
        self.base.set_dirt(ComponentDirt::NONE);
        self.coverage.clear();
        self.coverage.resize(size as usize, 0.0);
        for r in &mut self.ranges {
            unsafe { r.as_mut() }.compute_coverage(&mut self.coverage);
        }
    }
    pub fn coverage(&self, i: u32) -> f32 {
        self.coverage[i as usize]
    }
    pub fn glyph_coverage(&self, index: u32, count: u32) -> f32 {
        assert!(count >= 1);
        let mut c = self.coverage(index);
        for i in 1..count {
            c += self.coverage(index + i);
        }
        c / count as f32
    }
    pub fn on_text_world_transform_dirty(&mut self) {
        if !self.follow_path_modifiers.is_empty() {
            if let Some(mut text) = self.text_component() {
                unsafe { text.as_mut() }.base.add_dirt(ComponentDirt::PATH);
            }
        }
    }
    pub fn reset_text_follow_path(&mut self) {
        let Some(text) = self.text_component() else {
            return;
        };
        let Some(inverse) = unsafe { text.as_ref() }.base.world_transform().inverse() else {
            return;
        };
        for modifier in &mut self.follow_path_modifiers {
            unsafe { modifier.as_mut() }.reset(&inverse);
        }
    }
    pub fn modifies_transform(&self) -> bool {
        self.base.modifier_flags()
            & ((TextModifierFlags::MODIFY_TRANSLATION
                | TextModifierFlags::MODIFY_ROTATION
                | TextModifierFlags::MODIFY_SCALE
                | TextModifierFlags::MODIFY_ORIGIN)
                .0 as u32)
            != 0
    }
    fn has(&self, f: TextModifierFlags) -> bool {
        self.base.modifier_flags() & f.0 as u32 != 0
    }
    pub fn transform(&mut self, amount: f32, ctm: &mut Mat2D, arg: &mut TransformGlyphArg) {
        let follows = !self.follow_path_modifiers.is_empty();
        if amount == 0.0 || (!self.modifies_transform() && !follows) {
            return;
        }
        let mut parts = TransformComponents::default();
        if follows {
            let mut tc = TransformComponents::from_xy(arg.origin_position.x, arg.origin_position.y);
            if self.has(TextModifierFlags::MODIFY_TRANSLATION) {
                arg.offset = Vec2D::new(self.base.x(), self.base.y());
            }
            for m in &self.follow_path_modifiers {
                tc = unsafe { m.as_ref() }.transform_glyph(tc, arg);
            }
            let diff = tc.translation() - arg.origin_position;
            parts.set_rotation(parts.rotation() + tc.rotation() * amount);
            parts.set_x(diff.x * amount);
            parts.set_y(diff.y * amount);
        } else if self.has(TextModifierFlags::MODIFY_TRANSLATION) {
            parts.set_x(self.base.x() * amount);
            parts.set_y(self.base.y() * amount);
        }
        if self.has(TextModifierFlags::MODIFY_SCALE) {
            let inv = 1.0 - amount;
            parts.set_scale_x(inv + self.base.scale_x() * amount);
            parts.set_scale_y(inv + self.base.scale_y() * amount);
        }
        if self.has(TextModifierFlags::MODIFY_ROTATION) {
            parts.set_rotation(parts.rotation() + self.base.rotation() * amount);
        }
        let transform = Mat2D::compose(parts);
        let origin = self.has(TextModifierFlags::MODIFY_ORIGIN);
        if origin {
            ctm[4] += self.base.origin_x();
            ctm[5] += self.base.origin_y();
        }
        *ctm = transform * *ctm;
        if origin {
            ctm[4] -= self.base.origin_x();
            ctm[5] -= self.base.origin_y();
        }
    }
    pub fn compute_opacity(&self, current: f32, t: f32) -> f32 {
        if self.has(TextModifierFlags::INVERT_OPACITY) {
            current * (1.0 - t) + self.base.opacity() * t
        } else {
            current * self.base.opacity() * t
        }
    }
    fn mark_paint(&mut self) {
        self.base.parent_text_mut().mark_paint_dirty();
    }
    pub fn modifier_flags_changed(&mut self) {
        self.mark_paint();
    }
    pub fn origin_x_changed(&mut self) {
        self.mark_paint();
    }
    pub fn origin_y_changed(&mut self) {
        self.mark_paint();
    }
    pub fn opacity_changed(&mut self) {
        self.mark_paint();
    }
    pub fn x_changed(&mut self) {
        self.mark_paint();
    }
    pub fn y_changed(&mut self) {
        self.mark_paint();
    }
    pub fn rotation_changed(&mut self) {
        self.mark_paint();
    }
    pub fn scale_x_changed(&mut self) {
        self.mark_paint();
    }
    pub fn scale_y_changed(&mut self) {
        self.mark_paint();
    }
    fn copy_run(run: &TextRun, count: u32) -> TextRun {
        TextRun {
            font: run.font.clone(),
            size: run.size,
            line_height: run.line_height,
            letter_spacing: run.letter_spacing,
            unichar_count: count,
            script: run.script,
            style_id: run.style_id,
            level: run.level,
        }
    }
    pub fn modify_shape(&mut self, text: &Text, run: TextRun, strength: f32) -> TextRun {
        let Some(style) = text.style_from_shaper_id(run.style_id) else {
            return run;
        };
        let Some(font) = unsafe { style.as_ref() }.base.font() else {
            return run;
        };
        let mut variations = HashMap::new();
        let mut size = run.size;
        for modifier in &self.shape_modifiers {
            size = unsafe { modifier.as_ref() }.modify(&font, &mut variations, size, strength);
        }
        if variations.is_empty() {
            self.variable_font = None;
            return run;
        }
        self.variation_coords.clear();
        for (tag, value) in variations {
            self.variation_coords.push(FontCoord { axis: tag, value });
        }
        self.variable_font = font.make_at_coords(&self.variation_coords);
        TextRun {
            font: self.variable_font.clone().expect("variable font"),
            ..run
        }
    }
    pub fn apply_shape_modifiers(&mut self, text: &Text, styled: &mut StyledText) {
        if self.shape_modifiers.is_empty() {
            return;
        }
        self.next_text_runs.clear();
        self.next_text_runs.reserve(styled.runs().len());
        let (mut index, mut last, mut extract) = (0, f32::MAX, 0);
        for run in styled.runs() {
            let end = index + run.unichar_count;
            while index < end && index < self.coverage.len() as u32 {
                let coverage = self.coverage[index as usize];
                if coverage != last {
                    if index - extract != 0 {
                        let copy = Self::copy_run(run, index - extract);
                        let next = if last == 0.0 {
                            copy
                        } else {
                            self.modify_shape(text, copy, last)
                        };
                        self.next_text_runs.push(next);
                    }
                    last = coverage;
                    extract = index;
                }
                index += 1;
            }
            assert_ne!(extract, end);
            let copy = Self::copy_run(run, end - extract);
            let next = if last == 0.0 {
                copy
            } else {
                self.modify_shape(text, copy, last)
            };
            self.next_text_runs.push(next);
            extract = end;
        }
        styled.swap_runs(&mut self.next_text_runs);
    }
    pub fn needs_shape(&self) -> bool {
        !self.shape_modifiers.is_empty()
            || self
                .ranges
                .iter()
                .any(|r| unsafe { r.as_ref() }.needs_shape())
    }
}
