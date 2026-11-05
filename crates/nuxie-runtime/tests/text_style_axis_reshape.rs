//! A live `TextStyleAxis` change reshapes its text, as upstream's
//! `TextStyleAxis::tagChanged` and `axisValueChanged` do through
//! `parent()->addDirt(ComponentDirt::TextShape)`, which reaches the style's
//! virtual `onDirty` (text_style_axis.cpp, text_style.cpp).
//!
//! The oracle is a text built fresh at the new axis value: its axis value
//! comes from the file at import and never goes through the change path.
//! The font is a Roboto Flex subset with an `opsz` axis (fixtures/fonts).
use std::cell::Cell;
use std::rc::Rc;

use nuxie_binary::{
    FieldValue, FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, encode_runtime_file,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    generated::core_registry::CoreRegistry,
    text::{
        text::Text, text_style::TextStyleOwner, text_style_axis::TextStyleAxis,
        text_style_paint::TextStylePaint,
    },
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};

const FONT: &[u8] = include_bytes!("../../../fixtures/fonts/roboto-flex-awake.ttf");
const FONT_SIZE: f32 = 30.0;
/// `opsz`, big-endian: 0x6F70737A.
const OPSZ: u32 = u32::from_be_bytes(*b"opsz");
/// A tag the font has no axis for.
const UNKNOWN: u32 = u32::from_be_bytes(*b"XXXX");
const AXIS_VALUE_KEY: i32 = 288;
const AXIS_TAG_KEY: i32 = 289;

fn property(owner: &str, name: &str, value: FixtureValue) -> FixtureProperty {
    let def = nuxie_schema::definition_by_name(owner).unwrap();
    let key = std::iter::once(def.name)
        .chain(def.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|d| d.properties)
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("{owner}.{name}"))
        .key
        .int;
    FixtureProperty { key, value }
}

fn record(owner: &str, properties: Vec<(&str, FixtureValue)>) -> FixtureRecord {
    FixtureRecord {
        type_key: nuxie_schema::definition_by_name(owner)
            .unwrap()
            .type_key
            .int,
        properties: properties
            .into_iter()
            .map(|(name, value)| property(owner, name, value))
            .collect(),
    }
}

fn head() -> Vec<FixtureRecord> {
    vec![
        record("Backboard", vec![]),
        record("FontAsset", vec![("assetId", FixtureValue::Uint(0))]),
        record(
            "FileAssetContents",
            vec![("bytes", FixtureValue::Bytes(FONT.to_vec()))],
        ),
        record(
            "Artboard",
            vec![
                ("width", FixtureValue::Double(400.0)),
                ("height", FixtureValue::Double(100.0)),
            ],
        ),
    ]
}

/// A style (artboard-local id `parent + 1`) with one fill and one axis.
fn style(parent: u64, tag: u32, value: f32) -> Vec<FixtureRecord> {
    let style = parent + 1;
    vec![
        record(
            "TextStylePaint",
            vec![
                ("parentId", FixtureValue::Uint(parent)),
                ("fontSize", FixtureValue::Double(FONT_SIZE)),
                ("fontAssetId", FixtureValue::Uint(0)),
            ],
        ),
        record("Fill", vec![("parentId", FixtureValue::Uint(style))]),
        record(
            "SolidColor",
            vec![("parentId", FixtureValue::Uint(style + 1))],
        ),
        record(
            "TextStyleAxis",
            vec![
                ("parentId", FixtureValue::Uint(style)),
                ("tag", FixtureValue::Uint(u64::from(tag))),
                ("axisValue", FixtureValue::Double(value)),
            ],
        ),
    ]
}

/// "AWAKE" in an auto-width, unwrapped Text. Artboard-local ids: Text 1,
/// style 2, fill 3, color 4, axis 5, run 6.
fn text_file(tag: u32, value: f32) -> Vec<u8> {
    let mut records = head();
    records.push(record(
        "Text",
        vec![
            ("parentId", FixtureValue::Uint(0)),
            ("sizingValue", FixtureValue::Uint(0)),
            ("wrapValue", FixtureValue::Uint(1)),
        ],
    ));
    records.extend(style(1, tag, value));
    records.push(record(
        "TextValueRun",
        vec![
            ("parentId", FixtureValue::Uint(1)),
            ("text", FixtureValue::String("AWAKE".into())),
            ("styleId", FixtureValue::Uint(2)),
        ],
    ));
    encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap()
}

/// A global style's source shape: a style directly under the Artboard, no
/// Text. Upstream's `TextStyle::validate` takes an Artboard parent
/// (text_style.cpp), as ours does, but the fixture builder's validator only
/// takes a Text, so the style is built under an empty Text and moved to the
/// Artboard in the encoded records. Artboard-local ids: Text 1, style 2,
/// fill 3, color 4, axis 5.
fn global_style_file(value: f32) -> Vec<u8> {
    let mut records = head();
    records.push(record("Text", vec![("parentId", FixtureValue::Uint(0))]));
    records.extend(style(1, OPSZ, value));
    let mut file = RuntimeFile::from_fixture_records(records).unwrap();
    let parent_id = property("TextStylePaint", "parentId", FixtureValue::Uint(0)).key;
    let style = file
        .objects
        .iter_mut()
        .flatten()
        .find(|object| object.type_name == "TextStylePaint")
        .unwrap();
    style
        .properties
        .iter_mut()
        .find(|property| property.key == parent_id)
        .unwrap()
        .value = FieldValue::Uint(0);
    encode_runtime_file(&file).unwrap()
}

struct Scene {
    _file: RuntimeFileHandle,
    board: RuntimeArtboardInstanceHandle,
}

impl Scene {
    fn import(bytes: &[u8]) -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("axis text imports");
        // An instance clones every object, so this also proves the clone.
        let board = file.with_file(File::artboard_default).unwrap();
        let scene = Self { _file: file, board };
        scene.advance();
        scene
    }

    fn root(&self) -> CoreHandle {
        self.board.core_handle()
    }

    fn advance(&self) {
        Artboard::advance_handle(
            &self.root(),
            0.0,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        );
    }

    fn axis(&self) -> CoreHandle {
        let axes = self
            .board
            .with_artboard(|b| b.find_all_handles::<TextStyleAxis>());
        assert_eq!(axes.len(), 1);
        axes[0].clone()
    }

    fn style(&self) -> CoreHandle {
        let styles = self
            .board
            .with_artboard(|b| b.find_all_handles::<TextStylePaint>());
        assert_eq!(styles.len(), 1);
        styles[0].clone()
    }

    fn advances(&self) -> Vec<f32> {
        let texts = self.board.with_artboard(|b| b.find_all_handles::<Text>());
        assert_eq!(texts.len(), 1);
        let advances = texts[0]
            .with_downcast::<Text, _>(|text| {
                text.shape()
                    .iter()
                    .flat_map(|paragraph| paragraph.runs.iter())
                    .flat_map(|run| run.advances.iter().copied())
                    .collect::<Vec<f32>>()
            })
            .unwrap();
        assert!(advances.len() >= 5, "AWAKE shaped: {advances:?}");
        advances
    }
}

fn fresh(tag: u32, value: f32) -> Vec<f32> {
    Scene::import(&text_file(tag, value)).advances()
}

fn total(advances: &[f32]) -> f32 {
    advances.iter().sum()
}

#[test]
fn a_live_opsz_change_reshapes_to_a_fresh_texts_advances() {
    let at_22 = fresh(OPSZ, 22.0);
    let at_30 = fresh(OPSZ, 30.0);
    // The font follows the axis, so a stale font can't pass.
    assert!(
        (total(&at_22) - total(&at_30)).abs() > 3.0,
        "opsz 22 {} and opsz 30 {} differ by more than 3 px",
        total(&at_22),
        total(&at_30)
    );

    let scene = Scene::import(&text_file(OPSZ, 22.0));
    assert_eq!(scene.advances(), at_22);

    // The path keyframes, blends and data binds take.
    assert!(CoreRegistry::set_double_handle(
        &scene.axis(),
        AXIS_VALUE_KEY,
        30.0
    ));
    scene.advance();
    assert_eq!(
        scene.advances(),
        at_30,
        "live opsz 22 -> 30 (total {}) shapes like a fresh opsz 30 text (total {})",
        total(&scene.advances()),
        total(&at_30)
    );

    // And back again: not a one-shot.
    assert!(CoreRegistry::set_double_handle(
        &scene.axis(),
        AXIS_VALUE_KEY,
        22.0
    ));
    scene.advance();
    assert_eq!(
        scene.advances(),
        at_22,
        "live opsz 30 -> 22 (total {}) shapes like a fresh opsz 22 text (total {})",
        total(&scene.advances()),
        total(&at_22)
    );
}

#[test]
fn a_live_axis_tag_change_reshapes_too() {
    let at_30 = fresh(OPSZ, 30.0);
    let scene = Scene::import(&text_file(UNKNOWN, 30.0));
    assert_ne!(
        scene.advances(),
        at_30,
        "an unknown tag keeps the default opsz"
    );

    assert!(CoreRegistry::set_uint_handle(
        &scene.axis(),
        AXIS_TAG_KEY,
        OPSZ
    ));
    scene.advance();
    assert_eq!(
        scene.advances(),
        at_30,
        "live tag XXXX -> opsz at 30 (total {}) shapes like a fresh opsz 30 text (total {})",
        total(&scene.advances()),
        total(&at_30)
    );
}

#[test]
fn a_global_style_axis_change_updates_its_owner_at_once() {
    let scene = Scene::import(&global_style_file(22.0));
    let style = scene.style();
    let opsz = |style: &CoreHandle| {
        style
            .with_downcast_mut::<TextStylePaint, _>(|style| {
                style.font().expect("variable font").get_axis_value(OPSZ)
            })
            .unwrap()
    };
    assert_eq!(opsz(&style), 22.0);

    // Shaped like SharedNotifier (text_style_share_cache.rs): the owner
    // runs the source instance's update pass at once, which rebuilds the
    // variable font and so reads every axis again.
    let calls = Rc::new(Cell::new(0));
    let font_in_owner = Rc::new(Cell::new(None::<f32>));
    let owner: Rc<dyn TextStyleOwner> = {
        let calls = calls.clone();
        let font_in_owner = font_in_owner.clone();
        let root = scene.root();
        let style = style.clone();
        Rc::new(move || {
            calls.set(calls.get() + 1);
            Artboard::update_pass_handle(&root, false);
            font_in_owner.set(Some(opsz(&style)));
        })
    };
    style
        .with_downcast_mut::<TextStylePaint, _>(|style| {
            style.set_owner(Some(Rc::downgrade(&owner)));
        })
        .unwrap();

    assert!(CoreRegistry::set_double_handle(
        &scene.axis(),
        AXIS_VALUE_KEY,
        30.0
    ));
    assert_eq!(calls.get(), 1, "the owner runs once, at the change");
    assert_eq!(
        font_in_owner.get(),
        Some(30.0),
        "the owner's update pass rebuilt the variable font at opsz 30"
    );
    assert_eq!(opsz(&style), 30.0);
}

/// Independent of our code: HarfBuzz 14.2.1's `hb-shape` on the same subset
/// at 30 px ("AWAKE", GPOS kerning included).
#[test]
fn fresh_texts_match_harfbuzz() {
    for (value, expected) in [(22.0, 94.375), (30.0, 90.739)] {
        let shaped = total(&fresh(OPSZ, value));
        assert!(
            (shaped - expected).abs() < 0.01,
            "opsz {value}: {shaped} against HarfBuzz {expected}"
        );
    }
}
