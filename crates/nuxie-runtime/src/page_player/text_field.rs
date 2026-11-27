//! What a host needs to draw an engine text field as the platform's own:
//! its look, its placeholder's color and its texts, read from the settled
//! scene.
use super::driver::{SceneObject, scene_object_handle};
use crate::ArtboardInstance;
use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::{
        semantic::semantic_data_base::SemanticDataBase,
        text::text_input_text_base::TextInputTextBase,
    },
    semantic::semantic_data::SemanticData,
    text::{
        text::Text, text_input::TextInput, text_input_text::TextInputText,
        text_style_paint::TextStylePaint,
    },
};

/// The weight axis's tag, `wght`, big-endian.
const WEIGHT: u32 = u32::from_be_bytes(*b"wght");

/// What a text paints when nothing names its color, as the draw has it.
const BLACK: u32 = 0xff00_0000;

/// One text input's look.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextFieldLook {
    /// The index, in the file's assets, of the font it draws with: the
    /// one a font binding chose, else the one its style names. None when
    /// it draws with none, or with one the host gave.
    pub font_asset: Option<u32>,
    /// Whether the font it draws with was given by the host and has no
    /// index in the file's assets.
    pub font_from_host: bool,
    pub font_size: f32,
    /// Its style's `wght` axis value, or None when it sets none.
    pub weight: Option<f32>,
    pub letter_spacing: f32,
    /// ARGB: the color its text paints.
    pub color: u32,
    /// ARGB: the color the placeholder's text paints, when one was named.
    pub placeholder_color: Option<u32>,
    /// 0 left, 1 right, 2 center, as `align_value` holds it.
    pub align: u32,
    pub obscured: bool,
    /// 0 when it has no limit, as `max_characters` holds it.
    pub max_characters: u32,
    /// Whether its semantics say it is disabled: the disabled state of the
    /// nearest SemanticData on it or above it, which for an `input` is the
    /// record on its box. Never its FocusData.
    pub disabled: bool,
    /// UTF-8 lengths of its value and the placeholder's words.
    pub value_len: usize,
    pub placeholder_len: usize,
}

/// Why a text field or a text could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextFieldReadError {
    /// An object names nothing.
    NotFound,
    /// The field is not a TextInput.
    NotATextInput,
    /// The placeholder is not a Text.
    NotAText,
    /// The object is neither a TextInput nor a Text.
    NotText,
}

/// `field`'s look, with `placeholder`'s color and words when given.
/// NotFound when either names nothing; NotATextInput when `field` is
/// not a TextInput; NotAText when `placeholder` is not a Text.
pub fn text_field_look(
    artboard: &ArtboardInstance,
    field: &SceneObject,
    placeholder: Option<&SceneObject>,
) -> Result<TextFieldLook, TextFieldReadError> {
    let handle = scene_object_handle(artboard, field).map_err(|_| TextFieldReadError::NotFound)?;
    let Some((style, drawn, value_len, align, obscured, max_characters)) = handle
        .with_downcast::<TextInput, _>(|input| {
            let drawn = input
                .base
                .children()
                .iter()
                .find(|child| child.is_type_of(TextInputTextBase::TYPE_KEY))
                .cloned();
            (
                input.text_style().cloned(),
                drawn,
                input.base.text().len(),
                input.base.align_value(),
                input.base.obscured(),
                input.base.max_characters(),
            )
        })
    else {
        return Err(TextFieldReadError::NotATextInput);
    };
    // A field with no style shapes nothing: no font, no size.
    let (font, font_size, letter_spacing, weight) = style
        .and_then(|style| {
            style
                .with(|object| {
                    object.as_text_style().map(|style| {
                        (
                            style.asset(),
                            style.base.font_size(),
                            style.base.letter_spacing(),
                            style.axis_value(WEIGHT),
                        )
                    })
                })
                .flatten()
        })
        .unwrap_or((None, 0.0, 0.0, None));
    // The font it draws with, as a font binding left it: one of the file's
    // assets, or one the host gave, which has no index there.
    let font_asset = font.as_ref().and_then(|font| {
        artboard
            .native_file()
            .with_file(|file| file.assets().iter().position(|asset| asset == font))
    });
    let font_from_host = font.is_some() && font_asset.is_none();
    // A field with no text drawable draws no text; black, as one with no
    // fill paints.
    let color = drawn
        .and_then(|drawn| {
            drawn.with_downcast::<TextInputText, _>(|text| text.base.base.fill_color())
        })
        .unwrap_or(BLACK);
    let (placeholder_color, placeholder_len) = match placeholder {
        None => (None, 0),
        Some(placeholder) => {
            let handle = scene_object_handle(artboard, placeholder)
                .map_err(|_| TextFieldReadError::NotFound)?;
            let Some((run, words)) = handle.with_downcast::<Text, _>(|text| {
                (text.runs().first().cloned(), text.settled_text_value())
            }) else {
                return Err(TextFieldReadError::NotAText);
            };
            // Its first run's style paints it; a text with no run, or a run
            // with no style, paints black.
            let color = run
                .and_then(|run| run.with(|run| run.style()).flatten())
                .and_then(|style| {
                    style
                        .with(|object| {
                            object
                                .as_text_style_paint()
                                .map(TextStylePaint::foreground_color)
                        })
                        .flatten()
                })
                .unwrap_or(BLACK);
            (Some(color), words.len())
        }
    };
    Ok(TextFieldLook {
        font_asset: font_asset.map(|index| index as u32),
        font_from_host,
        font_size,
        weight,
        letter_spacing,
        color,
        placeholder_color,
        align,
        obscured,
        max_characters,
        disabled: disabled(&handle),
        value_len,
        placeholder_len,
    })
}

/// The words `object` holds: a TextInput's text, or a Text's runs
/// joined. NotFound, or NotText for anything else.
pub fn scene_text(
    artboard: &ArtboardInstance,
    object: &SceneObject,
) -> Result<String, TextFieldReadError> {
    let handle = scene_object_handle(artboard, object).map_err(|_| TextFieldReadError::NotFound)?;
    if let Some(text) = handle.with_downcast::<TextInput, _>(|input| input.base.text().to_owned()) {
        return Ok(text);
    }
    handle
        .with_downcast::<Text, _>(Text::settled_text_value)
        .ok_or(TextFieldReadError::NotText)
}

/// Whether the nearest SemanticData on `field` or above it, in its own
/// artboard, is disabled: the record the compiler writes on an `input`'s
/// box. False when there is none.
fn disabled(field: &CoreHandle) -> bool {
    let mut current = Some(field.clone());
    while let Some(component) = current {
        let record = component
            .with(|component| {
                component.as_node()?.children().iter().find_map(|child| {
                    child
                        .is_type_of(SemanticDataBase::TYPE_KEY)
                        .then(|| child.clone())
                })
            })
            .flatten();
        if let Some(record) = record {
            return record
                .with(|record| {
                    record
                        .as_semantic_data()
                        .is_some_and(SemanticData::is_disabled)
                })
                .unwrap_or(false);
        }
        current = component
            .with(|component| component.component_parent_handle())
            .flatten();
    }
    false
}
