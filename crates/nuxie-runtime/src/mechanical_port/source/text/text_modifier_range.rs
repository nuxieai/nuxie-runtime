use super::{glyph_lookup::GlyphLookup, text_value_run::TextValueRun};
use crate::mechanical_port::source::{
    animation::cubic_interpolator_component::CubicInterpolatorComponent,
    component::Component,
    core::CoreHandle,
    core_context::CoreContext,
    generated::text::text_modifier_range_base::TextModifierRangeBase,
    status_code::StatusCode,
    text_engine::{GlyphLine, Paragraph},
};
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextRangeUnits {
    Characters,
    CharactersExcludingSpaces,
    Words,
    Lines,
    Unknown(u32),
}
#[derive(Clone, Copy)]
pub enum TextRangeMode {
    Add,
    Subtract,
    Multiply,
    Min,
    Max,
    Difference,
    Unknown(u32),
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextRangeType {
    Percentage,
    UnitIndex,
    Unknown(u32),
}
#[repr(u8)]
pub enum TextRangeInterpolator {
    Linear,
    Cubic,
}
#[derive(Default)]
pub struct RangeMapper {
    unit_character_indices: Vec<u32>,
    unit_lengths: Vec<u32>,
}
impl RangeMapper {
    pub fn unit_count(&self) -> u32 {
        self.unit_lengths.len() as u32
    }
    pub fn unit_character_index_count(&self) -> u32 {
        self.unit_character_indices.len() as u32
    }
    pub fn clear(&mut self) {
        self.unit_character_indices.clear();
        self.unit_lengths.clear();
    }
    pub fn empty(&self) -> bool {
        self.unit_lengths.is_empty()
    }
    pub fn unit_character_index(&self, at: u32) -> u32 {
        self.unit_character_indices[at as usize]
    }
    pub fn unit_length(&self, at: u32) -> u32 {
        self.unit_lengths[at as usize]
    }
    pub fn unit_to_character_range(&self, unit: f32) -> f32 {
        if self.unit_character_indices.is_empty() {
            return 0.0;
        }
        let clamped = unit.clamp(0.0, (self.unit_character_indices.len() - 1) as f32);
        let index = clamped as usize;
        let mut chars = self.unit_character_indices[index] as f32;
        if index < self.unit_lengths.len() {
            chars += self.unit_lengths[index] as f32 * (clamped - index as f32);
        }
        chars
    }
    pub fn add_range(&mut self, from: u32, to: u32, start: u32, end: u32) {
        if to > start && end > from {
            let actual_start = start.max(from);
            let actual_end = end.min(to);
            if actual_end > actual_start {
                self.unit_character_indices.push(actual_start);
                self.unit_lengths.push(actual_end - actual_start);
            }
        }
    }
    pub fn from_words(&mut self, text: &[u32], start: u32, end: u32) {
        if text.is_empty() {
            return;
        }
        let mut want_space = false;
        let mut count = 0;
        let mut index = 0;
        let mut from = 0;
        for &unit in text {
            if want_space == crate::mechanical_port::source::text_engine::is_white_space(unit) {
                if !want_space {
                    from = index;
                } else {
                    self.add_range(from, from + count, start, end);
                    count = 0;
                }
                want_space = !want_space;
            }
            if want_space {
                count += 1;
            }
            index += 1;
        }
        if count > 0 {
            self.add_range(from, from + count, start, end);
        }
        self.unit_character_indices.push(end);
    }
    pub fn from_characters(
        &mut self,
        text: &[u32],
        start: u32,
        end: u32,
        lookup: &GlyphLookup,
        without_spaces: bool,
    ) {
        if text.is_empty() {
            return;
        }
        let mut i = start;
        while i < end {
            if without_spaces
                && crate::mechanical_port::source::text_engine::is_white_space(text[i as usize])
            {
                i += 1;
                continue;
            }
            let count = lookup.count(i);
            self.unit_character_indices.push(i);
            self.unit_lengths.push(count);
            i += count;
        }
        self.unit_character_indices.push(end);
    }
    pub fn from_lines(
        &mut self,
        text: &[u32],
        start: u32,
        end: u32,
        shape: &[Paragraph],
        lines: &[Vec<GlyphLine>],
        lookup: &GlyphLookup,
    ) {
        if text.is_empty() {
            return;
        }
        for (paragraph, lines) in shape.iter().zip(lines) {
            for line in lines {
                let first = &paragraph.runs[line.start_run_index as usize];
                let from = first.text_indices[line.start_glyph_index as usize];
                let last = &paragraph.runs[line.end_run_index as usize];
                let glyph = if line.end_glyph_index == 0 {
                    0
                } else {
                    line.end_glyph_index - 1
                };
                let mut to = last.text_indices[glyph as usize];
                to += lookup.count(to);
                self.add_range(from, to, start, end);
            }
        }
        self.unit_character_indices.push(end);
    }
}
pub struct TextModifierRange {
    pub base: TextModifierRangeBase,
    mapper: RangeMapper,
    index_from: f32,
    index_to: f32,
    index_falloff_from: f32,
    index_falloff_to: f32,
    interpolator: Option<CoreHandle>,
    run: Option<CoreHandle>,
}
impl TextModifierRange {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        if self.base.run_id() != crate::mechanical_port::source::core::EMPTY_ID {
            let Some(run) = context.resolve(self.base.run_id()).filter(|run| {
                run.with(|run| run.as_text_value_run().is_some())
                    .unwrap_or(false)
            }) else {
                return StatusCode::MissingObject;
            };
            self.run = Some(run);
        }
        let (Some(group), Some(this)) = (self.base.parent_handle(), self.base.handle()) else {
            return StatusCode::MissingObject;
        };
        let added = group
            .with_mut(|group| {
                group
                    .as_text_modifier_group_mut()
                    .map(|group| group.add_modifier_range(this))
            })
            .flatten()
            .is_some();
        if !added {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
    pub fn add_child(&mut self, component: &mut Component) {
        let Some(component) = component.handle() else {
            return;
        };
        self.base.add_child(component.clone());
        if component
            .with_downcast::<CubicInterpolatorComponent, _>(|_| ())
            .is_some()
        {
            self.interpolator = Some(component);
        }
    }
    pub fn clear_range_map(&mut self) {
        self.mapper.clear();
    }
    pub fn units(&self) -> TextRangeUnits {
        match self.base.units_value() {
            1 => TextRangeUnits::CharactersExcludingSpaces,
            2 => TextRangeUnits::Words,
            3 => TextRangeUnits::Lines,
            value @ 4.. => TextRangeUnits::Unknown(value),
            0 => TextRangeUnits::Characters,
        }
    }
    pub fn range_type(&self) -> TextRangeType {
        match self.base.type_value() {
            1 => TextRangeType::UnitIndex,
            value @ 2.. => TextRangeType::Unknown(value),
            0 => TextRangeType::Percentage,
        }
    }
    pub fn mode(&self) -> TextRangeMode {
        match self.base.mode_value() {
            1 => TextRangeMode::Subtract,
            2 => TextRangeMode::Multiply,
            3 => TextRangeMode::Min,
            4 => TextRangeMode::Max,
            5 => TextRangeMode::Difference,
            value @ 6.. => TextRangeMode::Unknown(value),
            0 => TextRangeMode::Add,
        }
    }
    pub fn compute_range(
        &mut self,
        text: &[u32],
        shape: &[Paragraph],
        lines: &[Vec<GlyphLine>],
        lookup: &GlyphLookup,
    ) {
        if !self.mapper.empty() {
            return;
        }
        let (mut start, mut end) = (0, text.len() as u32);
        if let Some(run) = &self.run {
            if let Some((offset, length)) = run
                .with_mut(|run| {
                    run.as_text_value_run_mut()
                        .map(|run| (run.offset(), run.length()))
                })
                .flatten()
            {
                start = offset;
                end = start + length;
            }
        }
        match self.units() {
            TextRangeUnits::CharactersExcludingSpaces => {
                self.mapper.from_characters(text, start, end, lookup, true)
            }
            TextRangeUnits::Words => self.mapper.from_words(text, start, end),
            TextRangeUnits::Lines => self
                .mapper
                .from_lines(text, start, end, shape, lines, lookup),
            TextRangeUnits::Characters => {
                self.mapper.from_characters(text, start, end, lookup, false)
            }
            TextRangeUnits::Unknown(_) => {
                self.mapper.from_characters(text, start, end, lookup, false)
            }
        }
    }
    fn coverage_at(&self, t: f32) -> f32 {
        if self.index_to < self.index_from || t < self.index_from || t > self.index_to {
            return 0.0;
        }
        let mut c = if t < self.index_falloff_from {
            let range = (self.index_falloff_from - self.index_from).max(0.0);
            if range == 0.0 {
                1.0
            } else {
                (t - self.index_from).max(0.0) / range
            }
        } else if t > self.index_falloff_to {
            let range = (self.index_to - self.index_falloff_to).max(0.0);
            if range == 0.0 {
                1.0
            } else {
                1.0 - ((t - self.index_falloff_to) / range).min(1.0)
            }
        } else {
            1.0
        };
        if (t < self.index_falloff_from || t > self.index_falloff_to)
            && let Some(interpolator) = &self.interpolator
        {
            c = interpolator
                .with_downcast::<CubicInterpolatorComponent, _>(|interpolator| {
                    interpolator.transform(c)
                })
                .unwrap_or(c);
        }
        c
    }
    fn offset_modify_from(&self) -> f32 {
        self.base.modify_from() + self.base.offset()
    }
    fn offset_modify_to(&self) -> f32 {
        self.base.modify_to() + self.base.offset()
    }
    fn offset_falloff_from(&self) -> f32 {
        self.base.falloff_from() + self.base.offset()
    }
    fn offset_falloff_to(&self) -> f32 {
        self.base.falloff_to() + self.base.offset()
    }
    pub fn compute_coverage(&mut self, coverage: &mut [f32]) {
        if self.mapper.empty() {
            return;
        }
        let count = self.mapper.unit_count();
        match self.range_type() {
            TextRangeType::Percentage => {
                self.index_from = count as f32 * self.offset_modify_from();
                self.index_to = count as f32 * self.offset_modify_to();
                self.index_falloff_from = count as f32 * self.offset_falloff_from();
                self.index_falloff_to = count as f32 * self.offset_falloff_to();
            }
            TextRangeType::UnitIndex => {
                self.index_from = self.offset_modify_from();
                self.index_to = self.offset_modify_to();
                self.index_falloff_from = self.offset_falloff_from();
                self.index_falloff_to = self.offset_falloff_to();
            }
            TextRangeType::Unknown(_) => {}
        }
        for unit in 0..count {
            let len = self.mapper.unit_length(unit);
            let index = self.mapper.unit_character_index(unit);
            let c = self.base.strength() * self.coverage_at(unit as f32 + 0.5);
            for i in 0..len {
                let current = &mut coverage[(index + i) as usize];
                *current = match self.mode() {
                    TextRangeMode::Add => *current + c,
                    TextRangeMode::Subtract => *current - c,
                    TextRangeMode::Max => current.max(c),
                    TextRangeMode::Min => current.min(c),
                    TextRangeMode::Multiply => *current * c,
                    TextRangeMode::Difference => (*current - c).abs(),
                    TextRangeMode::Unknown(_) => *current,
                };
                if self.base.clamp() {
                    *current = current.clamp(0.0, 1.0);
                }
            }
            if unit + 1 < self.mapper.unit_character_index_count() {
                for i in index + len..self.mapper.unit_character_index(unit + 1) {
                    coverage[i as usize] = 0.0;
                }
            }
        }
    }
    fn range_changed(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_mut(|parent| {
                if let Some(parent) = parent.as_text_modifier_group_mut() {
                    parent.range_changed();
                }
            });
        }
    }
    pub fn modify_from_changed(&mut self) {
        self.range_changed();
    }
    pub fn modify_to_changed(&mut self) {
        self.range_changed();
    }
    pub fn strength_changed(&mut self) {
        self.range_changed();
    }
    pub fn units_value_changed(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_mut(|parent| {
                if let Some(parent) = parent.as_text_modifier_group_mut() {
                    parent.range_type_changed();
                }
            });
        }
    }
    pub fn type_value_changed(&mut self) {
        self.range_changed();
    }
    pub fn mode_value_changed(&mut self) {
        self.range_changed();
    }
    pub fn clamp_changed(&mut self) {
        self.range_changed();
    }
    pub fn falloff_from_changed(&mut self) {
        self.range_changed();
    }
    pub fn falloff_to_changed(&mut self) {
        self.range_changed();
    }
    pub fn offset_changed(&mut self) {
        self.range_changed();
    }
    pub fn needs_shape(&self) -> bool {
        self.units() == TextRangeUnits::Lines
    }
}
