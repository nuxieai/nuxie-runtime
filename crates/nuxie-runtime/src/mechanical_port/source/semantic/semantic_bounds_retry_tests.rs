//! A semantic record whose bounds are empty when it is first placed retries
//! them once (UNIV-7008), as upstream's comment on the retry says
//! (`src/semantic/semantic_data.cpp`, "re-dirts once if the initial bounds
//! are degenerate"). Each page is played by a state-machine page player with
//! the semantic tree built, as the SDKs play it, and read through the
//! player's `settled`.
//!
//! Oracle: a page with nothing moving reports itself settled within a few
//! frames, as the same page does without the tree; the text's bounds follow
//! from its font size and line height (20 and 24 points) and from where it
//! is placed (the artboard's top left); a shape's bounds are its rectangle.
use super::*;
use crate::mechanical_port::source::{
    generated::{core_registry::CoreRegistry, text::text_value_run_base::TextValueRunBase},
    semantic::{semantic_manager::SemanticManager, semantic_snapshot::Bounds},
    text::text_value_run::TextValueRun,
};
use crate::page_player::{Output, PagePlayer, Playback, Step};
use crate::{ArtboardInstance, File, RuntimeFactoryHandle, RuntimeFileHandle};
use nuxie_binary::{
    FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, encode_runtime_file,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};

/// The most frames a still page is given to settle. The same page without
/// the tree settles on its second; a page that settles at all does so
/// within a handful.
const SETTLE_FRAMES: usize = 10;

/// The promise for a still page: it settles within this many frames.
const SETTLED_WITHIN: usize = 5;

/// The frames a settled still page is stepped further, each of which must
/// report it settled: dirt that never ends would show in one of them.
const STAYS_SETTLED_FRAMES: usize = 120;

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

/// A state machine with one empty layer, so a page player plays it.
fn machine() -> Vec<FixtureRecord> {
    use FixtureValue::String as Str;
    vec![
        record("StateMachine", vec![("name", Str("machine".into()))]),
        record("StateMachineLayer", vec![("name", Str("layer".into()))]),
        record("EntryState", vec![]),
        record("AnyState", vec![]),
        record("ExitState", vec![]),
    ]
}

/// A column holding one auto-sized text, its one run `words`, with a
/// semantic record on the text (a read text, labeled from its words).
fn text_page(words: &str) -> Vec<u8> {
    use FixtureValue::{Bytes, Double, String as Str, Uint};
    let mut records = vec![
        record("Backboard", vec![]),
        record("FontAsset", vec![("assetId", Uint(0))]),
        record(
            "FileAssetContents",
            vec![(
                "bytes",
                Bytes(include_bytes!("../../../../../../fixtures/fonts/roboto-input.ttf").to_vec()),
            )],
        ),
        // 0
        record(
            "Artboard",
            vec![
                ("width", Double(320.0)),
                ("height", Double(400.0)),
                ("styleId", Uint(1)),
            ],
        ),
        record("LayoutComponentStyle", vec![("parentId", Uint(0))]), // 1
        record("LayoutCssStyle", vec![("parentId", Uint(0))]),       // 2
        record(
            "LayoutComponent",
            vec![
                ("name", Str("column".into())),
                ("parentId", Uint(0)),
                ("styleId", Uint(4)),
            ],
        ), // 3
        record("LayoutComponentStyle", vec![("parentId", Uint(3))]), // 4
        record("LayoutCssStyle", vec![("parentId", Uint(3))]),       // 5
        record("Text", vec![("parentId", Uint(3))]),                 // 6
        record(
            "TextStylePaint",
            vec![
                ("parentId", Uint(6)),
                ("fontSize", Double(20.0)),
                ("fontAssetId", Uint(0)),
                ("lineHeight", Double(24.0)),
            ],
        ), // 7
        record("Fill", vec![("parentId", Uint(7))]),                 // 8
        record("SolidColor", vec![("parentId", Uint(8))]),           // 9
        record(
            "TextValueRun",
            vec![
                ("parentId", Uint(6)),
                ("styleId", Uint(7)),
                ("text", Str(words.into())),
            ],
        ), // 10
        record("SemanticData", vec![("parentId", Uint(6))]),         // 11
    ];
    records.extend(machine());
    encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap()
}

/// A host artboard using a component, the component's one rectangle 100 by
/// 40 at (10, 20) carrying a semantic record: upstream's case for the retry,
/// a nested artboard whose shapes may build their paths after the record is
/// first placed.
fn nested_page() -> Vec<u8> {
    use FixtureValue::{Double, String as Str, Uint};
    let mut records = vec![
        record("Backboard", vec![]),
        record(
            "Artboard",
            vec![("width", Double(320.0)), ("height", Double(400.0))],
        ),
        record(
            "NestedArtboard",
            vec![
                ("parentId", Uint(0)),
                ("artboardId", Uint(1)),
                ("x", Double(30.0)),
                ("y", Double(50.0)),
            ],
        ),
    ];
    records.extend(machine());
    records.extend(card());
    encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap()
}

/// The component: one rectangle 100 by 40, its top left at (10, 20),
/// carrying a semantic record.
fn card() -> Vec<FixtureRecord> {
    use FixtureValue::{Double, String as Str, Uint};
    vec![
        record(
            "Artboard",
            vec![
                ("name", Str("Card".into())),
                ("width", Double(200.0)),
                ("height", Double(100.0)),
                ("originX", Double(0.0)),
                ("originY", Double(0.0)),
            ],
        ), // 0
        record(
            "Shape",
            vec![
                ("parentId", Uint(0)),
                ("x", Double(60.0)),
                ("y", Double(40.0)),
            ],
        ), // 1
        record(
            "Rectangle",
            vec![
                ("parentId", Uint(1)),
                ("width", Double(100.0)),
                ("height", Double(40.0)),
            ],
        ), // 2
        record("Fill", vec![("parentId", Uint(1))]), // 3
        record("SolidColor", vec![("parentId", Uint(3))]), // 4
        record(
            "SemanticData",
            vec![
                ("parentId", Uint(1)),
                ("role", Uint(1)),
                ("label", Str("Card button".into())),
            ],
        ), // 5
    ]
}

/// A still page of shapes: a box carrying a semantic record, which a node
/// clips to and follows (a clip and a constraint, both dependents of the
/// box), the clipped node holding a circle, and the component of
/// `nested_page` placed below.
fn shapes_page() -> Vec<u8> {
    use FixtureValue::{Double, String as Str, Uint};
    let mut records = vec![
        record("Backboard", vec![]),
        record(
            "Artboard",
            vec![("width", Double(320.0)), ("height", Double(400.0))],
        ), // 0
        record(
            "Shape",
            vec![
                ("parentId", Uint(0)),
                ("x", Double(100.0)),
                ("y", Double(60.0)),
            ],
        ), // 1
        record(
            "Rectangle",
            vec![
                ("parentId", Uint(1)),
                ("width", Double(100.0)),
                ("height", Double(40.0)),
            ],
        ), // 2
        record("Fill", vec![("parentId", Uint(1))]), // 3
        record("SolidColor", vec![("parentId", Uint(3))]), // 4
        record(
            "SemanticData",
            vec![
                ("parentId", Uint(1)),
                ("role", Uint(1)),
                ("label", Str("Box".into())),
            ],
        ), // 5
        record(
            "Node",
            vec![("parentId", Uint(0)), ("name", Str("clipped".into()))],
        ), // 6
        record(
            "ClippingShape",
            vec![("parentId", Uint(6)), ("sourceId", Uint(1))],
        ), // 7
        record(
            "TranslationConstraint",
            vec![("parentId", Uint(6)), ("targetId", Uint(1))],
        ), // 8
        record(
            "Shape",
            vec![
                ("parentId", Uint(6)),
                ("x", Double(20.0)),
                ("y", Double(20.0)),
            ],
        ), // 9
        record(
            "Ellipse",
            vec![
                ("parentId", Uint(9)),
                ("width", Double(60.0)),
                ("height", Double(60.0)),
            ],
        ), // 10
        record("Fill", vec![("parentId", Uint(9))]), // 11
        record("SolidColor", vec![("parentId", Uint(11))]), // 12
        record(
            "NestedArtboard",
            vec![
                ("parentId", Uint(0)),
                ("artboardId", Uint(1)),
                ("x", Double(30.0)),
                ("y", Double(200.0)),
            ],
        ), // 13
    ];
    records.extend(machine());
    records.extend(card());
    encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap()
}

struct Page {
    _file: RuntimeFileHandle,
    _factory: PersistentFactory<RecordingFactory>,
    board: ArtboardInstance,
    playback: Playback,
    player: PagePlayer,
}

impl Page {
    /// `bytes`' first artboard played by its state machine, with the
    /// semantic tree built when `semantics` is set, as
    /// `nux_player_enable_semantics` builds it.
    fn open(bytes: &[u8], semantics: bool) -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("the page imports");
        let mut board = ArtboardInstance::from_native(file.clone(), 0).unwrap();
        let machine = board
            .state_machine_instance(0)
            .expect("the page's state machine");
        if semantics {
            board.native_handle().build_semantic_tree(
                Some(RuntimeSemanticManagerHandle::new(SemanticManager::new())),
                None,
            );
        }
        Self {
            _file: file,
            _factory: factory,
            board,
            playback: Playback::StateMachine(Box::new(machine)),
            player: PagePlayer::default(),
        }
    }

    fn step(&mut self) -> Output {
        self.player
            .step(
                &mut self.board,
                &mut self.playback,
                Step {
                    elapsed_seconds: 1.0 / 60.0,
                    ..Default::default()
                },
                &[],
                None,
            )
            .unwrap_or_else(|error| panic!("a step: {:?}: {}", error.kind, error.message))
    }

    /// The frame, counted from 1, on which the page first reports itself
    /// settled, within `SETTLE_FRAMES`.
    fn frames_to_settle(&mut self) -> Option<usize> {
        (1..=SETTLE_FRAMES).find(|_| self.step().settled)
    }

    /// The bounds of the semantic node of the page's one semantic record,
    /// a nested artboard's included.
    fn bounds(&self) -> Bounds {
        let mut found = Vec::new();
        collect_semantic_data(&self.board.native_handle().core_handle(), &mut found);
        assert_eq!(found.len(), 1, "one semantic record");
        found[0]
            .with_downcast::<SemanticData, _>(|data| {
                data.semantic_node
                    .as_ref()
                    .expect("the record is in the tree")
                    .borrow()
                    .bounds
                    .clone()
            })
            .unwrap()
    }

    /// The bounds of every semantic record's node on the page.
    fn all_bounds(&self) -> Vec<Bounds> {
        let mut found = Vec::new();
        collect_semantic_data(&self.board.native_handle().core_handle(), &mut found);
        found
            .iter()
            .map(|data| {
                data.with_downcast::<SemanticData, _>(|data| {
                    data.semantic_node
                        .as_ref()
                        .expect("the record is in the tree")
                        .borrow()
                        .bounds
                        .clone()
                })
                .unwrap()
            })
            .collect()
    }

    /// The page settles within `SETTLED_WITHIN` frames, then reports itself
    /// settled on each of the next `STAYS_SETTLED_FRAMES`. Returns the frame
    /// it settled on.
    fn assert_settles_and_stays_settled(&mut self, label: &str) -> usize {
        let frame = self.frames_to_settle();
        assert!(
            frame.is_some_and(|frame| frame <= SETTLED_WITHIN),
            "{label}: settled on frame {frame:?}, the promise is within {SETTLED_WITHIN}"
        );
        let frame = frame.unwrap();
        for later in 1..=STAYS_SETTLED_FRAMES {
            assert!(
                self.step().settled,
                "{label}: settled on frame {frame}, then busy again {later} frames later"
            );
        }
        frame
    }

    fn set_words(&mut self, words: &str) {
        let runs = self
            .board
            .native_handle()
            .with_artboard(|artboard| artboard.find_all_handles::<TextValueRun>());
        assert_eq!(runs.len(), 1);
        assert!(CoreRegistry::set_string_handle(
            &runs[0],
            TextValueRunBase::TEXT_PROPERTY_KEY.into(),
            words.into(),
        ));
    }
}

/// Every semantic record in `artboard` and the artboards it hosts.
fn collect_semantic_data(artboard: &CoreHandle, found: &mut Vec<CoreHandle>) {
    let (records, hosted) = artboard
        .with(|object| {
            let artboard = object.as_artboard().expect("an artboard");
            (
                artboard.find_all_handles::<SemanticData>(),
                artboard.find_all_handles::<crate::mechanical_port::source::nested_artboard::NestedArtboard>(),
            )
        })
        .unwrap();
    found.extend(records);
    for host in hosted {
        if let Some(inner) = host
            .with(|object| object.as_artboard_host()?.artboard_instance(0))
            .flatten()
        {
            collect_semantic_data(&inner.core_handle(), found);
        }
    }
}

fn is_empty(bounds: &Bounds) -> bool {
    !(bounds.max_x > bounds.min_x && bounds.max_y > bounds.min_y)
}

/// The text's box with words: in a block column it is the column's width,
/// the artboard's 320, and one line of 24 points high, at the top left.
fn assert_one_line_of_words(bounds: &Bounds) {
    assert_eq!(
        *bounds,
        Bounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 320.0,
            max_y: 24.0,
        }
    );
}

/// The case from Nux: a column holding a read text with no words settles
/// within 5 frames and stays settled for a long run after (120 frames),
/// with the tree built and without, and its record's bounds are empty.
#[test]
fn a_still_page_over_an_empty_text_settles() {
    let mut without = Page::open(&text_page(""), false);
    without.assert_settles_and_stays_settled("without the tree");
    let mut page = Page::open(&text_page(""), true);
    page.assert_settles_and_stays_settled("with the tree");
    assert!(is_empty(&page.bounds()), "{:?}", page.bounds());
}

/// The retry's purpose is kept: the same text gets its bounds once it has
/// words, and the page settles again.
#[test]
fn an_empty_text_gets_its_bounds_once_it_has_words() {
    let mut page = Page::open(&text_page(""), true);
    assert!(page.frames_to_settle().is_some(), "settles while empty");
    page.set_words("Hello");
    assert!(page.frames_to_settle().is_some(), "settles with words");
    assert_one_line_of_words(&page.bounds());
}

/// Upstream's case for the retry: a nested artboard's record gets its
/// shape's bounds, the rectangle moved by the shape's and the component's
/// places (100 by 40 at (30 + 60 - 50, 50 + 40 - 20), a rectangle being
/// drawn about its middle), and the page settles.
#[test]
fn a_nested_record_gets_its_shapes_bounds() {
    let mut page = Page::open(&nested_page(), true);
    assert!(page.frames_to_settle().is_some(), "the page settles");
    assert_eq!(
        page.bounds(),
        Bounds {
            min_x: 40.0,
            min_y: 70.0,
            max_x: 140.0,
            max_y: 110.0,
        }
    );
}

/// A text that gains words and loses them again settles each time, its
/// record's bounds empty, then one line of the words, then empty again.
#[test]
fn a_text_that_gains_and_loses_words_settles_each_time() {
    let mut page = Page::open(&text_page(""), true);
    assert!(page.frames_to_settle().is_some(), "settles while empty");
    assert!(is_empty(&page.bounds()));
    page.set_words("Hello");
    assert!(page.frames_to_settle().is_some(), "settles with words");
    assert_one_line_of_words(&page.bounds());
    page.set_words("");
    assert!(page.frames_to_settle().is_some(), "settles empty again");
    assert!(is_empty(&page.bounds()), "{:?}", page.bounds());
}

/// A still page of shapes with dependents (a clip and a constraint on the
/// box) and a component's shape settles on the same frame with the tree
/// built and without, and then stays settled, so the retry adds no endless
/// dirt; with the tree, the box's record and the component's record have
/// their rectangles.
#[test]
fn a_still_page_of_shapes_settles_with_and_without_the_tree() {
    let mut without = Page::open(&shapes_page(), false);
    let frame = without.assert_settles_and_stays_settled("without the tree");
    let mut page = Page::open(&shapes_page(), true);
    assert_eq!(
        page.assert_settles_and_stays_settled("with the tree"),
        frame,
        "with the tree, against without"
    );
    let mut bounds = page.all_bounds();
    bounds.sort_by(|a, b| a.min_y.total_cmp(&b.min_y));
    assert_eq!(
        bounds,
        [
            Bounds {
                min_x: 50.0,
                min_y: 40.0,
                max_x: 150.0,
                max_y: 80.0,
            },
            Bounds {
                min_x: 40.0,
                min_y: 220.0,
                max_x: 140.0,
                max_y: 260.0,
            },
        ]
    );
}
