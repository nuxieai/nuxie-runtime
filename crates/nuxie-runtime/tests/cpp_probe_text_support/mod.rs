//! Shared test-only observations of authored native TextInput owners.
#![allow(dead_code)]

use super::native_runtime;
use native_runtime::source::{
    assets::font_asset::FontAsset,
    component_dirt::ComponentDirt,
    core::CoreHandle,
    generated::core_registry::CoreRegistry,
    math::vec2d::Vec2D,
    text::{
        cursor::{Cursor, CursorPosition},
        font_hb::HbFont,
        text_input::TextInput,
        text_style::TextStyle,
    },
};
use native_runtime::{
    File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use std::path::PathBuf;

pub(super) fn fixture_path(relative: &str) -> PathBuf {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(relative)
}

pub(super) fn fixture() -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle, CoreHandle) {
    let bytes = std::fs::read(fixture_path("text_input.riv")).expect("pinned TextInput fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("native File import");
    let artboard = file
        .with_file(|file| file.artboard_named("Text Input - Multiline"))
        .expect("authored artboard");
    let input = artboard
        .with_artboard(|artboard| {
            artboard
                .objects()
                .iter()
                .flatten()
                .find(|object| object.is_type_of(TextInput::TYPE_KEY))
                .cloned()
        })
        .expect("authored TextInput");
    (file, artboard, input)
}

pub(super) fn with_input<R>(input: &CoreHandle, f: impl FnOnce(&mut TextInput) -> R) -> R {
    input.with_downcast_mut(f).expect("live TextInput")
}

pub(super) fn property_key(type_name: &str, property_name: &str) -> i32 {
    let definition = nuxie_schema::definition_by_name(type_name).expect("schema type");
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == property_name)
        .expect("schema property")
        .key
        .int
        .into()
}

pub(super) fn set_cursor(input: &CoreHandle, start: u32, end: u32) {
    with_input(input, |input| {
        input.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(start),
            CursorPosition::unresolved(end),
        ))
    });
}

pub(super) fn cursor(input: &CoreHandle) -> Option<(u32, u32)> {
    input.with_downcast_mut::<TextInput, _>(|input| {
        let cursor = input.raw_text_input().cursor();
        (
            cursor.start().code_point_index(),
            cursor.end().code_point_index(),
        )
    })
}

pub(super) fn clear_dirt(input: &CoreHandle) {
    input
        .with_mut(|object| {
            object
                .as_component_mut()
                .expect("Component")
                .set_dirt(ComponentDirt::NONE)
        })
        .expect("live input");
}

pub(super) fn dirt(input: &CoreHandle) -> Option<ComponentDirt> {
    input.with(|object| object.as_component().expect("Component").dirt())
}

pub(super) fn style(input: &CoreHandle) -> CoreHandle {
    with_input(input, |input| {
        input
            .base
            .children()
            .iter()
            .find(|child| child.is_type_of(TextStyle::TYPE_KEY))
            .cloned()
    })
    .expect("TextInput owns a TextStyle")
}

pub(super) fn install_font(style: &CoreHandle, name: &str) {
    let bytes = std::fs::read(fixture_path(name)).expect("pinned font bytes");
    let font = HbFont::decode(&bytes).expect("native HbFont");
    let asset = style
        .insert_sibling(FontAsset::default())
        .expect("native FontAsset occurrence");
    FontAsset::set_font_occurrence(&asset, Some(font.clone()));
    TextStyle::set_asset_occurrence(style, Some(asset));
    assert!(
        style
            .with_downcast_mut::<TextStyle, _>(|style| style.font().is_some())
            .unwrap()
    );
    // The old test helper replaced the font used by live TextInput geometry.
    // Upstream TextInput captures its font at onAddedClean, not on every update,
    // so replacing TextStyle's asset alone does not perform that test override.
    let input = style
        .with_downcast::<TextStyle, _>(|style| style.base.parent_handle())
        .flatten()
        .expect("TextStyle's authored TextInput parent");
    with_input(&input, |input| input.raw_text_input().set_font(Some(font)));
}

pub(super) fn update_raw(artboard: &RuntimeArtboardInstanceHandle, input: &CoreHandle) {
    let factory = artboard
        .with_artboard(|artboard| artboard.factory())
        .expect("retained factory");
    with_input(input, |input| input.raw_text_input().update(&factory));
}

pub(super) fn world_point(
    artboard: &RuntimeArtboardInstanceHandle,
    input: &CoreHandle,
    x: f32,
    y: f32,
) -> Vec2D {
    let point = with_input(input, |input| {
        *input.base.world_transform() * Vec2D::new(x, y)
    });
    artboard.with_artboard_mut(|artboard| artboard.root_transform(point))
}

pub(super) fn caret(input: &CoreHandle) -> Option<((f32, f32), (f32, f32))> {
    input
        .with_downcast_mut::<TextInput, _>(|input| {
            let position = input.raw_text_input().cursor_visual_position();
            position.found().then_some((
                (position.x(), position.top()),
                (position.x(), position.bottom()),
            ))
        })
        .flatten()
}

// Project the already-shaped native glyph span into authored code-point bounds,
// as the original text-only snapshot did. RawTextInput appends one U+200B at
// length() for its terminal caret; it is not part of text() or this snapshot.
// Do not change the source shape, including any sentinel-only final line.
// No line breaking, bidi ordering, glyph positioning, or cursor behavior lives here.
pub(super) fn line_metrics(input: &CoreHandle) -> Option<Vec<(usize, usize, f32, f32)>> {
    input.with_downcast_mut::<TextInput, _>(|input| {
        let raw = input.raw_text_input();
        let authored_length = raw.length();
        let shape = raw.shape();
        shape
            .ordered_lines()
            .iter()
            .filter_map(|line| {
                let mut begin = usize::MAX;
                let mut end = 0;
                let mut glyph = line.begin();
                while glyph != line.end() {
                    let index = glyph.run().text_indices[glyph.glyph_index() as usize];
                    if (index as usize) < authored_length {
                        begin = begin.min(index as usize);
                        end = end.max(
                            ((index + shape.glyph_lookup().count(index)) as usize)
                                .min(authored_length),
                        );
                    }
                    glyph.advance();
                }
                (begin != usize::MAX).then_some((
                    begin,
                    end,
                    line.y() - line.glyph_line().baseline + line.glyph_line().top,
                    line.bottom(),
                ))
            })
            .collect()
    })
}

pub(super) fn line_directions(input: &CoreHandle) -> Option<Vec<bool>> {
    input.with_downcast_mut::<TextInput, _>(|input| {
        let shape = input.raw_text_input().shape();
        shape
            .paragraphs()
            .iter()
            .zip(shape.paragraph_lines())
            .flat_map(|(paragraph, lines)| {
                std::iter::repeat_n(
                    paragraph.base_direction()
                        == native_runtime::source::text::text_engine::TextDirection::Rtl,
                    lines.len(),
                )
            })
            .collect()
    })
}

pub(super) fn measure(input: &CoreHandle, width: f32, height: f32) -> Option<(f32, f32, f32, f32)> {
    input.with_downcast_mut::<TextInput, _>(|input| {
        let bounds = input.raw_text_input().measure(width, height);
        (bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y)
    })
}

pub(super) fn measure_count(input: &CoreHandle) -> Option<usize> {
    input.with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().measure_count as usize)
}
