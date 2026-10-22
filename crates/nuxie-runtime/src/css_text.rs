//! CSS text processing for text owned by a CSS box.
use crate::source::{
    component::Component,
    core::{
        Core, CoreHandle, CoreObject, CoreType, PropertySetterCompletion,
        binary_reader::BinaryReader,
        field_types::{core_callback_type::CallbackData, core_uint_type::CoreUintType},
    },
    core_context::CoreContext,
    generated::{
        component_base::ComponentBase,
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
        text::text_style_paint_base::TextStylePaintBase,
    },
    importers::import_stack::ImportStack,
    status_code::StatusCode,
    text::text_style_paint::TextStylePaint,
};
use std::any::Any;

#[path = "css_titlecase_data.rs"]
mod titlecase_data;

pub const TEXT_TRANSFORM_PROPERTY_KEY: u16 = 61104;
pub const WHITE_SPACE_PROPERTY_KEY: u16 = 61105;

pub fn is_property(key: i32) -> bool {
    matches!(key, 61104 | 61105)
}

#[derive(Default)]
pub struct TextStyleCssText {
    component: Component,
    text_transform: u32,
    white_space: u32,
}

impl TextStyleCssText {
    pub const TYPE_KEY: u16 = 61005;

    fn subtype(key: u16) -> bool {
        key == Self::TYPE_KEY || ComponentBase::is_type_of(key)
    }

    pub fn text_transform(&self) -> u32 {
        self.text_transform
    }
    pub fn white_space(&self) -> u32 {
        self.white_space
    }
    pub fn property(&self, key: u16) -> u32 {
        match key {
            TEXT_TRANSFORM_PROPERTY_KEY => self.text_transform,
            WHITE_SPACE_PROPERTY_KEY => self.white_space,
            _ => 0,
        }
    }
    pub fn set_property(&mut self, key: u16, value: u32) -> bool {
        let field = match key {
            TEXT_TRANSFORM_PROPERTY_KEY => &mut self.text_transform,
            WHITE_SPACE_PROPERTY_KEY => &mut self.white_space,
            _ => return false,
        };
        let value = value as u8 as u32;
        if *field == value {
            return false;
        }
        *field = value;
        if let Some(text) = self.component.parent_handle().and_then(|style| {
            style
                .with(|style| style.as_component().and_then(|style| style.parent_handle()))
                .flatten()
        }) {
            text.with_mut(|text| {
                if let Some(text) = text.as_text_mut() {
                    text.mark_shape_dirty();
                }
            });
        }
        true
    }

    fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.component.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let (Some(parent), Some(this)) = (
            self.component.parent_handle(),
            CoreObject::core(self).handle(),
        ) else {
            return StatusCode::InvalidObject;
        };
        if parent
            .with_downcast_mut::<TextStylePaint, _>(|style| style.set_css_text(this))
            .is_none()
        {
            return StatusCode::InvalidObject;
        }
        StatusCode::Ok
    }
}

impl CoreType for TextStyleCssText {
    const TYPE_KEY: u16 = TextStyleCssText::TYPE_KEY;
}

impl CoreObject for TextStyleCssText {
    fn core(&self) -> &Core {
        &self.component.base.base
    }
    fn core_mut(&mut self) -> &mut Core {
        &mut self.component.base.base
    }
    fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
        let mut cloned = Self {
            text_transform: self.text_transform,
            white_space: self.white_space,
            ..Self::default()
        };
        cloned
            .component
            .base
            .copy(&self.component.base, &mut Component::default());
        Some(Box::new(cloned))
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        if is_property(i32::from(key)) {
            let value = CoreUintType::deserialize(reader) as u8 as u32;
            match key {
                TEXT_TRANSFORM_PROPERTY_KEY => self.text_transform = value,
                _ => self.white_space = value,
            }
            return true;
        }
        let mut base = std::mem::take(&mut self.component.base);
        let handled = base.deserialize(key, reader, &mut self.component);
        self.component.base = base;
        handled
    }
}

impl CoreRegistryObject for TextStyleCssText {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn set_uint_with_completion(
        &mut self,
        field: CoreField,
        value: u32,
        completion: &mut PropertySetterCompletion,
    ) {
        if field == CoreField::ComponentParentId && self.component.base.set_parent_id_value(value) {
            completion.record(
                CoreObject::core(self),
                ComponentBase::PARENT_ID_PROPERTY_KEY,
            );
        }
    }
    fn set_string_with_completion(
        &mut self,
        field: CoreField,
        value: String,
        completion: &mut PropertySetterCompletion,
    ) {
        if field == CoreField::ComponentName && self.component.base.set_name_value(value) {
            completion.record(CoreObject::core(self), ComponentBase::NAME_PROPERTY_KEY);
        }
    }
    fn set_color_with_completion(
        &mut self,
        _: CoreField,
        _: i32,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_bool_with_completion(
        &mut self,
        _: CoreField,
        _: bool,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_double_with_completion(
        &mut self,
        _: CoreField,
        _: f32,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_int_with_completion(&mut self, _: CoreField, _: i32, _: &mut PropertySetterCompletion) {}
    fn set_callback_with_completion(
        &mut self,
        _: CoreField,
        _: CallbackData<'_>,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn get_uint(&mut self, field: CoreField) -> u32 {
        match field {
            CoreField::ComponentParentId => self.component.base.parent_id(),
            _ => 0,
        }
    }
    fn get_string(&mut self, field: CoreField) -> String {
        match field {
            CoreField::ComponentName => self.component.base.name().to_owned(),
            _ => String::new(),
        }
    }
    fn get_color(&mut self, _: CoreField) -> i32 {
        0
    }
    fn get_bool(&mut self, _: CoreField) -> bool {
        false
    }
    fn get_double(&mut self, _: CoreField) -> f32 {
        0.0
    }
    fn get_int(&mut self, _: CoreField) -> i32 {
        0
    }
}

impl CoreCapabilities for TextStyleCssText {
    fn component_on_dirty(&mut self, _: crate::source::component_dirt::ComponentDirt) -> bool {
        true
    }
    fn component_on_dirty_is_noop(&self) -> bool {
        true
    }
    fn lifecycle_validate(&mut self, context: &mut dyn CoreContext) -> Option<bool> {
        Some(
            self.text_transform <= 3
                && self.white_space <= 4
                && self.component.validate(context)
                && context
                    .resolve(self.component.base.parent_id())
                    .is_some_and(|parent| parent.is_type_of(TextStylePaintBase::TYPE_KEY)),
        )
    }
    fn lifecycle_on_added_dirty(&mut self, context: &mut dyn CoreContext) -> Option<StatusCode> {
        Some(self.on_added_dirty(context))
    }
    fn lifecycle_import(&mut self, stack: &mut ImportStack) -> Option<StatusCode> {
        Some(self.component.import(stack))
    }
    fn as_component(&self) -> Option<&Component> {
        Some(&self.component)
    }
    fn as_component_mut(&mut self) -> Option<&mut Component> {
        Some(&mut self.component)
    }
}

/// The compiler writes the inherited values on every participating style.
pub(crate) fn style_options(style: &CoreHandle) -> Option<(u32, u32)> {
    style
        .with_downcast::<TextStylePaint, _>(TextStylePaint::css_text)
        .flatten()
        .and_then(|record| {
            record.with_downcast::<TextStyleCssText, _>(|record| {
                (
                    if record.text_transform <= 3 {
                        record.text_transform
                    } else {
                        0
                    },
                    if record.white_space <= 4 {
                        record.white_space
                    } else {
                        0
                    },
                )
            })
        })
}

/// Source code-point boundaries mapped to display code-point boundaries.
/// Equal boundaries represent removed whitespace; a jump represents a full
/// Unicode mapping such as one sharp s becoming two capital letters.
#[derive(Clone, Default)]
pub struct TextIndexMap {
    source: Vec<u32>,
    boundaries: Vec<u32>,
    collapsible: Vec<bool>,
}
impl TextIndexMap {
    pub fn source(&self) -> &[u32] {
        &self.source
    }
    pub fn to_display(&self, source: u32) -> u32 {
        self.boundaries
            .get(source as usize)
            .copied()
            .unwrap_or_else(|| self.boundaries.last().copied().unwrap_or(0))
    }
    pub fn to_source(&self, display: u32) -> u32 {
        let after = self
            .boundaries
            .partition_point(|&boundary| boundary <= display);
        let before = after.saturating_sub(1);
        if let Some(&next) = self.boundaries.get(after)
            && display - self.boundaries[before] >= next - display
        {
            return after as u32;
        }
        before as u32
    }
    pub(crate) fn is_collapsible(&self, display: u32) -> bool {
        self.collapsible
            .get(display as usize)
            .copied()
            .unwrap_or(false)
    }
}

fn titlecase(ch: char) -> Vec<char> {
    let exceptions = titlecase_data::TITLECASE_EXCEPTIONS;
    match exceptions.binary_search_by_key(&ch, |&(source, _)| source) {
        Ok(index) => exceptions[index].1.chars().collect(),
        Err(_) => ch.to_uppercase().collect(),
    }
}

/// Process the complete source so collapse and word state cross style runs.
pub(crate) fn display_text(
    source: &[u32],
    options: &[Option<(u32, u32)>],
) -> (Vec<u32>, TextIndexMap) {
    use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
    let chars: Vec<char> = source
        .iter()
        .map(|&c| char::from_u32(c).unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect();
    // str's full default lowercase includes contextual final sigma. The number
    // of output scalars per source scalar is unchanged by that context rule.
    let lower: Vec<char> = chars
        .iter()
        .collect::<String>()
        .to_lowercase()
        .chars()
        .collect();
    let mut lower_offset = 0;
    let mut in_word = false;
    let mut pieces: Vec<(usize, u32, bool)> = Vec::new();
    for (i, &ch) in chars.iter().enumerate() {
        let lower_len = ch.to_lowercase().count();
        let lowercase = &lower[lower_offset..lower_offset + lower_len];
        lower_offset += lower_len;
        let first = !in_word && ch.is_alphabetic();
        let extends_word = matches!(
            ch.general_category(),
            GeneralCategory::NonspacingMark
                | GeneralCategory::SpacingMark
                | GeneralCategory::EnclosingMark
                | GeneralCategory::Format
                | GeneralCategory::ConnectorPunctuation
        );
        if ch.is_alphanumeric() {
            in_word = true;
        } else if !extends_word
            && !(matches!(ch, '\'' | '\u{2019}')
                && in_word
                && chars.get(i + 1).is_some_and(|c| c.is_alphabetic()))
        {
            in_word = false;
        }
        let Some((transform, whitespace)) = options[i] else {
            pieces.push((i, ch as u32, false));
            continue;
        };
        let collapse = matches!(whitespace, 0 | 3 | 4);
        // CSS segment breaks normalize CRLF and CR before whitespace processing.
        if ch == '\n' && i > 0 && chars[i - 1] == '\r' && options[i - 1].is_some() {
            continue;
        }
        let ch = if ch == '\r' || ch == '\u{c}' {
            '\n'
        } else {
            ch
        };
        if collapse && (ch == ' ' || ch == '\t' || (ch == '\n' && whitespace != 3)) {
            if pieces
                .last()
                .is_none_or(|&(_, cp, collapsible)| cp == 10 || collapsible)
            {
                continue;
            }
            pieces.push((i, 32, true));
        } else if ch == '\n' {
            while pieces
                .last()
                .is_some_and(|&(_, _, collapsible)| collapsible)
            {
                pieces.pop();
            }
            pieces.push((i, 10, false));
        } else {
            let mapped: Vec<char> = match transform {
                1 => ch.to_uppercase().collect(),
                2 => lowercase.to_vec(),
                3 if first && ch.is_lowercase() => titlecase(ch),
                _ => vec![ch],
            };
            pieces.extend(mapped.into_iter().map(|ch| (i, ch as u32, false)));
        }
    }
    while pieces
        .last()
        .is_some_and(|&(_, _, collapsible)| collapsible)
    {
        pieces.pop();
    }
    let mut map = TextIndexMap {
        source: source.to_vec(),
        boundaries: vec![0; source.len() + 1],
        collapsible: Vec::with_capacity(pieces.len()),
    };
    let mut display = Vec::with_capacity(pieces.len());
    for (source, cp, collapsible) in pieces {
        map.boundaries[source + 1] += 1;
        map.collapsible.push(collapsible);
        display.push(cp);
    }
    for i in 1..map.boundaries.len() {
        map.boundaries[i] += map.boundaries[i - 1];
    }
    (display, map)
}

/// Trim only CSS-collapsible glyphs at the logical edges. The display and
/// its source map stay intact, including the boundaries skipped at a wrap.
pub(crate) fn trim_line(
    line: &mut crate::source::text_engine::GlyphLine,
    runs: &[crate::source::text_engine::GlyphRun],
    map: &TextIndexMap,
) {
    let mut start = (
        line.start_run_index as usize,
        line.start_glyph_index as usize,
    );
    let mut end = (line.end_run_index as usize, line.end_glyph_index as usize);
    while end > start {
        let previous = if end.1 == 0 {
            let run = end.0 - 1;
            (run, runs[run].glyphs.len().saturating_sub(1))
        } else {
            (end.0, end.1 - 1)
        };
        if !runs[previous.0]
            .text_indices
            .get(previous.1)
            .is_some_and(|&index| map.is_collapsible(index))
        {
            break;
        }
        end = previous;
    }
    while start < end {
        if start.1 == runs[start.0].glyphs.len() {
            start = (start.0 + 1, 0);
            continue;
        }
        if !map.is_collapsible(runs[start.0].text_indices[start.1]) {
            break;
        }
        start.1 += 1;
    }
    line.start_run_index = start.0 as u32;
    line.start_glyph_index = start.1 as u32;
    line.end_run_index = end.0 as u32;
    line.end_glyph_index = end.1 as u32;
}

#[cfg(test)]
#[path = "css_text_tests.rs"]
mod tests;
