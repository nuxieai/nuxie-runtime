use super::{
    glyph_lookup::GlyphLookup,
    text::{StyledText, Text},
    text_follow_path_modifier::TextFollowPathModifier,
    text_modifier_flags::TextModifierFlags,
    text_modifier_range::TextModifierRange,
    text_variation_modifier::TextVariationModifier,
};
use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::CoreContext,
    generated::core_registry::CoreCapabilities,
    generated::text::text_modifier_group_base::TextModifierGroupBase,
    math::{mat2d::Mat2D, transform_components::TransformComponents, vec2d::Vec2D},
    status_code::StatusCode,
    text_engine::{FontCoord, FontRef, GlyphLine, Paragraph, TextRun},
};
use std::collections::HashMap;
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
impl std::ops::Deref for TextModifierGroup {
    type Target = TextModifierGroupBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextModifierGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TextModifierGroup {
    pub const TYPE_KEY: u16 = TextModifierGroupBase::TYPE_KEY;
}

#[derive(Default)]
pub struct TextModifierGroup {
    pub base: TextModifierGroupBase,
    ranges: Vec<CoreHandle>,
    modifiers: Vec<CoreHandle>,
    shape_modifiers: Vec<CoreHandle>,
    follow_path_modifiers: Vec<CoreHandle>,
    coverage: Vec<f32>,
    variable_font: Option<FontRef>,
    variation_coords: Vec<FontCoord>,
    next_text_runs: Vec<TextRun>,
}
impl TextModifierGroup {
    pub fn text_component(&self) -> Option<CoreHandle> {
        self.base.parent_handle().filter(|parent| {
            parent.is_type_of(
                crate::mechanical_port::source::generated::text::text_base::TextBase::TYPE_KEY,
            )
        })
    }
    pub fn on_added_dirty(&mut self, c: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(c);
        if code != StatusCode::Ok {
            return code;
        }
        let (Some(text), Some(this)) = (self.text_component(), self.base.handle()) else {
            return StatusCode::MissingObject;
        };
        let added = text
            .with_mut(|text| text.as_text_mut().map(|text| text.add_modifier_group(this)))
            .flatten()
            .is_some();
        if !added {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
    pub fn add_modifier_range(&mut self, range: CoreHandle) {
        self.ranges.push(range);
    }
    pub fn add_modifier(&mut self, modifier: CoreHandle) {
        self.modifiers.push(modifier.clone());
        if modifier
            .is_type_of(crate::mechanical_port::source::generated::text::text_shape_modifier_base::TextShapeModifierBase::TYPE_KEY)
        {
            self.shape_modifiers.push(modifier.clone());
        }
        if modifier
            .is_type_of(crate::mechanical_port::source::generated::text::text_follow_path_modifier_base::TextFollowPathModifierBase::TYPE_KEY)
        {
            self.follow_path_modifiers.push(modifier);
        }
    }
    pub fn range_type_changed(&mut self) {
        self.with_text_mut(Text::modifier_shape_dirty);
        self.base.add_dirt(ComponentDirt::TEXT_COVERAGE, true);
    }
    pub fn shape_modifier_changed(&mut self) {
        self.with_text_mut(Text::mark_shape_dirty);
    }
    pub fn range_changed(&mut self) {
        if self.shape_modifiers.is_empty() {
            self.with_text_mut(Text::mark_paint_dirty);
        } else {
            self.with_text_mut(Text::modifier_shape_dirty);
        }
        self.base.add_dirt(ComponentDirt::TEXT_COVERAGE, true);
    }
    pub fn clear_range_maps(&mut self) {
        for r in &mut self.ranges {
            r.with_downcast_mut::<TextModifierRange, _>(TextModifierRange::clear_range_map);
        }
        self.base.add_dirt(ComponentDirt::TEXT_COVERAGE, true);
    }
    pub fn compute_range_map(
        &mut self,
        owner_text: &Text,
        text: &[u32],
        shape: &[Paragraph],
        lines: &[Vec<GlyphLine>],
        lookup: &GlyphLookup,
    ) {
        for r in &mut self.ranges {
            r.with_downcast_mut::<TextModifierRange, _>(|range| {
                range.compute_range(owner_text, text, shape, lines, lookup)
            });
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
            r.with_downcast_mut::<TextModifierRange, _>(|range| {
                range.compute_coverage(&mut self.coverage)
            });
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
    pub fn on_text_world_transform_dirty(owner: &CoreHandle, parent_text: &mut Text) {
        let (follows_path, text) = owner
            .with(|owner| {
                let group = owner
                    .as_text_modifier_group()
                    .expect("TextModifierGroup owner");
                (
                    !group.follow_path_modifiers.is_empty(),
                    group.text_component(),
                )
            })
            .expect("live TextModifierGroup");
        if follows_path {
            assert_eq!(
                parent_text.base.handle(),
                text,
                "actual TextModifierGroup parent"
            );
            // addDirt can re-enter Text::onDirty and this same modifier group.
            // Keep the live Text borrow, but release the group before callback.
            parent_text.component_add_dirt(ComponentDirt::PATH, false);
        }
    }
    pub fn reset_text_follow_path(&mut self, parent_text: &Text) {
        let Some(text) = self.text_component() else {
            return;
        };
        assert_eq!(
            parent_text.base.handle().as_ref(),
            Some(&text),
            "resetTextFollowPath receives its actual parent Text"
        );
        let mut inverse = Mat2D::default();
        if !parent_text.base.world_transform().invert(&mut inverse) {
            return;
        }
        for modifier in &mut self.follow_path_modifiers {
            modifier.with_downcast_mut::<TextFollowPathModifier, _>(|modifier| {
                modifier.reset(&inverse)
            });
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
    pub fn modifies_opacity(&self) -> bool {
        self.has(TextModifierFlags::MODIFY_OPACITY)
    }
    pub fn modifies_rotation(&self) -> bool {
        self.has(TextModifierFlags::MODIFY_ROTATION)
    }
    pub fn modifies_translation(&self) -> bool {
        self.has(TextModifierFlags::MODIFY_TRANSLATION)
    }
    pub fn modifies_scale(&self) -> bool {
        self.has(TextModifierFlags::MODIFY_SCALE)
    }
    pub fn modifies_origin(&self) -> bool {
        self.has(TextModifierFlags::MODIFY_ORIGIN)
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
            let mut tc = TransformComponents::default();
            tc.set_x(arg.origin_position.x);
            tc.set_y(arg.origin_position.y);
            if self.has(TextModifierFlags::MODIFY_TRANSLATION) {
                arg.offset = Vec2D::new(self.base.x(), self.base.y());
            }
            for m in &self.follow_path_modifiers {
                tc = m
                    .with_downcast::<TextFollowPathModifier, _>(|modifier| {
                        modifier.transform_glyph(tc, arg)
                    })
                    .unwrap_or(tc);
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
        let transform = Mat2D::compose(&parts);
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
        self.with_text_mut(Text::mark_paint_dirty);
    }

    fn with_text_mut(&self, use_text: impl FnOnce(&mut Text)) {
        if let Some(text) = self.text_component() {
            text.with_mut(|text| {
                if let Some(text) = text.as_text_mut() {
                    use_text(text);
                }
            });
        }
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
        let Some(font) = style
            .with_mut(|style| style.as_text_style_mut().and_then(|style| style.font()))
            .flatten()
        else {
            return run;
        };
        let mut variations = HashMap::new();
        let mut size = run.size;
        for modifier in &self.shape_modifiers {
            size = modifier
                .with_downcast::<TextVariationModifier, _>(|modifier| {
                    modifier.modify(font.as_ref(), &mut variations, size, strength)
                })
                .unwrap_or(size);
        }
        if variations.is_empty() {
            self.variable_font = None;
            return run;
        }
        self.variation_coords.clear();
        for (tag, value) in variations {
            self.variation_coords.push(FontCoord { axis: tag, value });
        }
        self.variable_font = Some(font.make_at_coords(&self.variation_coords));
        TextRun {
            font: self.variable_font.clone(),
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
            || self.ranges.iter().any(|range| {
                range
                    .with_downcast::<TextModifierRange, _>(TextModifierRange::needs_shape)
                    .unwrap_or(false)
            })
    }
}
