//! A host's settle solves the layout it is owed within the frame. A write
//! to a text style's font asset reshapes its text after the frame's first
//! solve has measured it in the old font (the style's variable font is
//! rebuilt by its `TextVariationHelper`, which updates after the solve), so
//! the text asks for another solve. The settle loop keeps going while the
//! artboard has a dirty layout, as upstream's
//! `StateMachineInstance::advanceAndApply` does, so that solve lands in the
//! same frame.
//!
//! Oracle: each face's own tables, its `cmap` and `hmtx` read by
//! [`advanced`], never the runtime's measure. The text is eight A's, which
//! neither face kerns: before the write in the variable face at its default
//! instance (the style's `wght` axis at 400 is the face's default), after it
//! in the static face.
use super::*;
use crate::source::{
    generated::{core_registry::CoreRegistry, text::text_style_base::TextStyleBase},
    layout_component::LayoutComponent,
    text::text_style_paint::TextStylePaint,
};
use crate::{ArtboardInstance, File, RuntimeFactoryHandle};
use nuxie_binary::{
    FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, encode_runtime_file,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};

/// A Roboto Flex subset with `wght` and `opsz` axes: the file's asset 0.
const VARIABLE: &[u8] = include_bytes!("../../../../fixtures/fonts/roboto-flex-awake.ttf");
/// A Roboto Regular subset, static: the file's asset 1.
const REGULAR: &[u8] = include_bytes!("../../../../fixtures/fonts/roboto-input.ttf");
const WORDS: &str = "AAAAAAAA";
const FONT_SIZE: f32 = 30.0;

fn record(owner: &str, values: Vec<(&str, FixtureValue)>) -> FixtureRecord {
    let def = nuxie_schema::definition_by_name(owner).unwrap();
    FixtureRecord {
        type_key: def.type_key.int,
        properties: values
            .into_iter()
            .map(|(name, value)| {
                let key = std::iter::once(def.name)
                    .chain(def.ancestors.iter().copied())
                    .filter_map(nuxie_schema::definition_by_name)
                    .flat_map(|d| d.properties)
                    .find(|p| p.name == name)
                    .unwrap_or_else(|| panic!("{owner}.{name}"))
                    .key
                    .int;
                FixtureProperty { key, value }
            })
            .collect(),
    }
}

/// A CSS flex row holding one box, "Words", sized by its Text: eight A's at
/// 30 points in asset 0, its style with a `wght` axis at 400. Artboard-local
/// ids: Artboard 0, its styles 1 and 2, the box 3, its styles 4 and 5, Text
/// 6, style 7, fill 8, color 9, axis 10, run 11.
fn words_file() -> Vec<u8> {
    use FixtureValue::{Bytes, Double, String as Str, Uint};
    let records = vec![
        record("Backboard", vec![]),
        record("FontAsset", vec![("assetId", Uint(0))]),
        record(
            "FileAssetContents",
            vec![("bytes", Bytes(VARIABLE.to_vec()))],
        ),
        record("FontAsset", vec![("assetId", Uint(1))]),
        record(
            "FileAssetContents",
            vec![("bytes", Bytes(REGULAR.to_vec()))],
        ),
        record(
            "Artboard",
            vec![
                ("width", Double(400.0)),
                ("height", Double(100.0)),
                ("styleId", Uint(1)),
            ],
        ),
        record("LayoutComponentStyle", vec![("parentId", Uint(0))]),
        record(
            "LayoutCssStyle",
            vec![("parentId", Uint(0)), ("display", Uint(1))],
        ),
        record(
            "LayoutComponent",
            vec![
                ("parentId", Uint(0)),
                ("name", Str("Words".into())),
                ("styleId", Uint(4)),
            ],
        ),
        record("LayoutComponentStyle", vec![("parentId", Uint(3))]),
        record("LayoutCssStyle", vec![("parentId", Uint(3))]),
        record("Text", vec![("parentId", Uint(3))]),
        record(
            "TextStylePaint",
            vec![
                ("parentId", Uint(6)),
                ("fontSize", Double(FONT_SIZE)),
                ("fontAssetId", Uint(0)),
            ],
        ),
        record("Fill", vec![("parentId", Uint(7))]),
        record("SolidColor", vec![("parentId", Uint(8))]),
        record(
            "TextStyleAxis",
            vec![
                ("parentId", Uint(7)),
                ("tag", Uint(u64::from(u32::from_be_bytes(*b"wght")))),
                ("axisValue", Double(400.0)),
            ],
        ),
        record(
            "TextValueRun",
            vec![
                ("parentId", Uint(6)),
                ("styleId", Uint(7)),
                ("text", Str(WORDS.into())),
            ],
        ),
    ];
    encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap()
}

fn u16_at(face: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([face[at], face[at + 1]])
}

fn u32_at(face: &[u8], at: usize) -> usize {
    let bytes = [face[at], face[at + 1], face[at + 2], face[at + 3]];
    u32::from_be_bytes(bytes) as usize
}

/// The width of `words` at `size` in `face` at its default instance: each
/// letter's glyph from its `cmap` (the Windows Unicode table, format 4),
/// that glyph's advance from its `hmtx`, the advances summed and scaled by
/// `size` over the face's units per em (`head`). Kerning is not read, so
/// `words` should be letters the face does not kern.
fn advanced(face: &[u8], words: &str, size: f32) -> f32 {
    let table = |tag: &[u8]| {
        let count = usize::from(u16_at(face, 4));
        let entry = (0..count)
            .map(|at| 12 + 16 * at)
            .find(|at| &face[*at..*at + 4] == tag)
            .expect("the table");
        u32_at(face, entry + 8)
    };
    let head = table(b"head");
    let hhea = table(b"hhea");
    let hmtx = table(b"hmtx");
    let cmap = table(b"cmap");
    let units_per_em = f32::from(u16_at(face, head + 18));
    let metrics = usize::from(u16_at(face, hhea + 34));
    let unicode = (0..usize::from(u16_at(face, cmap + 2)))
        .map(|at| cmap + 4 + 8 * at)
        .find(|at| {
            let ids = (u16_at(face, *at), u16_at(face, *at + 2));
            ids == (3, 1) && u16_at(face, cmap + u32_at(face, *at + 4)) == 4
        })
        .map(|at| cmap + u32_at(face, at + 4))
        .expect("a format 4 table for Unicode");
    let segments = usize::from(u16_at(face, unicode + 6) / 2);
    // The glyph's row in `hmtx`: glyphs past the last full row share it.
    let glyph = |letter: char| -> usize {
        let code = u16::try_from(u32::from(letter)).expect("a letter of the first plane");
        let at = (0..segments)
            .find(|at| u16_at(face, unicode + 14 + 2 * at) >= code)
            .expect("a segment");
        let start = u16_at(face, unicode + 16 + 2 * segments + 2 * at);
        let delta = u16_at(face, unicode + 16 + 4 * segments + 2 * at);
        let range_at = unicode + 16 + 6 * segments + 2 * at;
        let range = usize::from(u16_at(face, range_at));
        assert!(start <= code, "`{letter}` is in the face");
        let found = if range == 0 {
            code
        } else {
            u16_at(face, range_at + range + 2 * usize::from(code - start))
        };
        usize::from(found.wrapping_add(delta)).min(metrics - 1)
    };
    let units: u32 = words
        .chars()
        .map(|letter| u32::from(u16_at(face, hmtx + 4 * glyph(letter))))
        .sum();
    units as f32 * size / units_per_em
}

/// Whether `handle` is the component named `name`.
fn is_named(handle: &CoreHandle, name: &str) -> bool {
    let named = handle.with(|o| o.as_component().map(|c| c.base.name() == name));
    named.flatten().unwrap_or(false)
}

struct Scene {
    _file: RuntimeFileHandle,
    board: ArtboardInstance,
}

impl Scene {
    /// The file played until its first layout is solved.
    fn new() -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &words_file(),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("the text imports");
        let board = ArtboardInstance::from_native(file.clone(), 0).unwrap();
        let scene = Self { _file: file, board };
        for _ in 0..3 {
            Artboard::advance_handle(
                &scene.root().core_handle(),
                0.0,
                AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
            );
        }
        assert!(!scene.owed(), "the first layout is solved");
        scene
    }

    fn root(&self) -> RuntimeArtboardInstanceHandle {
        self.board.native_handle()
    }

    /// The solved width of the box the text sizes.
    fn width(&self) -> f32 {
        let root = self.root();
        let boxes = root.with_artboard(|board| board.find_all_handles::<LayoutComponent>());
        let words = boxes.into_iter().find(|handle| is_named(handle, "Words"));
        let words = words.expect("the box");
        let bounds = words.with_downcast::<LayoutComponent, _>(|layout| layout.layout_bounds());
        bounds.expect("a live box").width()
    }

    /// Whether the artboard has a layout still to solve.
    fn owed(&self) -> bool {
        let root = self.root();
        root.with_artboard(|board| board.base.has_dirty_layout())
    }

    /// The text's style draws in asset 1, written through the registry's
    /// setter, as a host's driver writes it (`SceneWriter::set_unsigned`).
    fn write_regular_face(&self) {
        let root = self.root();
        let styles = root.with_artboard(|board| board.find_all_handles::<TextStylePaint>());
        assert_eq!(styles.len(), 1);
        assert!(CoreRegistry::set_uint_handle(
            &styles[0],
            i32::from(TextStyleBase::FONT_ASSET_ID_PROPERTY_KEY),
            1
        ));
    }
}

/// The widths before and after the write, each from its face's own tables,
/// which differ.
fn widths() -> (f32, f32) {
    let variable = advanced(VARIABLE, WORDS, FONT_SIZE);
    let regular = advanced(REGULAR, WORDS, FONT_SIZE);
    assert!(
        (variable - regular).abs() > 1.0,
        "the faces differ: {variable} and {regular}"
    );
    (variable, regular)
}

fn near(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() < 0.05
}

/// The page player's settle after a driver's round
/// (`StateMachineInstance::settle_within_frame`): one settle after a write
/// to the style's font asset leaves no layout to solve, and the box is as
/// wide as the new face's own advances.
#[test]
fn a_settle_solves_the_layout_a_font_asset_write_owes() {
    let (variable, regular) = widths();
    let mut scene = Scene::new();
    assert!(
        near(scene.width(), variable),
        "before the write, the variable face: {} and {variable}",
        scene.width()
    );
    scene.write_regular_face();
    StateMachineInstance::settle_within_frame(&mut scene.board, None).expect("the settle");
    assert!(!scene.owed(), "no layout is left to solve");
    assert!(
        near(scene.width(), regular),
        "in that settle, the regular face: {} and {regular} (the variable face {variable})",
        scene.width()
    );
}

/// The same through the advance that shares the settle loop
/// (`advance_native_state_machines`): one advance after the write.
#[test]
fn an_advance_solves_the_layout_a_font_asset_write_owes() {
    let (variable, regular) = widths();
    let scene = Scene::new();
    assert!(
        near(scene.width(), variable),
        "before the write, the variable face: {} and {variable}",
        scene.width()
    );
    scene.write_regular_face();
    advance_native_state_machines(&scene.root(), &[], 0.0, true);
    assert!(!scene.owed(), "no layout is left to solve");
    assert!(
        near(scene.width(), regular),
        "in that advance, the regular face: {} and {regular} (the variable face {variable})",
        scene.width()
    );
}
