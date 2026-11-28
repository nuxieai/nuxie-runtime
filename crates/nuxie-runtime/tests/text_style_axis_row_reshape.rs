//! A host driver's write to a `TextStyleAxis` value reshapes and relays out
//! a text inside a keyed list's row, in the step of the write, as it does
//! for the same text played as the root artboard.
//!
//! The text is "AWAKE" in a CSS box sized by the text ("Words"), at 35
//! points in a Roboto Flex subset with a `wght` axis, written 800 to 900.
//! The row artboard is played five ways: as the player's root; as a row of
//! a driver page's list (a keyed list flagged `isPageRows` under a plain
//! Node of the root); as a row of the same keyed list without the flag;
//! as a row of a keyed list under a CSS layout box of the root; and as a
//! Screen of the older `screens` page (a view-model list named `screens`
//! under a plain Node of a root named `$nuxie.page`), where no driver path
//! leads into a row, so the axis is written through the same generated
//! setter a driver write calls, between steps.
//!
//! A row of a list with no layout parent lays itself out, in its own update
//! pass, which only its list runs. The text reshapes in the rebuilt font
//! after that solve and marks the row's layout again, so the list runs the
//! row's update pass once more in the step's settle. Each shape settles the
//! step of the write in fewer update passes than the settle loop's limit,
//! and the steps after it solve no layout. The same holds for a row that
//! has gone quiet before the write, and for one row written among a few
//! hundred. A sleeping page row keeps the write and is laid out when it
//! wakes.
//!
//! Oracle: the same shape built fresh with the axis at 900, whose value
//! comes from the file at import and never goes through the change path.
use std::cell::RefCell;
use std::rc::Rc;

use nuxie_binary::{
    FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, encode_runtime_file,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::page_player::{
    DriverFault, DriverRound, HostDriver, PagePlayer, Playback, SceneObject, SceneWriter, Step,
};
use nuxie_runtime::source::{
    artboard_component_list::ArtboardComponentList, layout_component::LayoutComponent,
    text::text::Text,
};
use nuxie_runtime::{
    Artboard, ArtboardInstance, CoreHandle, File, RuntimeArtboardInstanceHandle,
    RuntimeFactoryHandle, RuntimeFileHandle, RuntimeGeometryHitPathSegment, RuntimeListRowKey,
    RuntimeOwnedViewModelHandle, measure_settles, settle_pass_limit,
};

const FONT: &[u8] = include_bytes!("../../../fixtures/fonts/roboto-flex-awake.ttf");
const FONT_SIZE: f32 = 35.0;
const WGHT: u32 = u32::from_be_bytes(*b"wght");
const AXIS_VALUE_KEY: u16 = 288;
const FROM: f32 = 800.0;
const TO: f32 = 900.0;
const FRAME: f32 = 1.0 / 60.0;
const FRAMES: usize = 6;
/// The root artboard's file-global id: the fourth record.
const PAGE_GLOBAL_ID: u32 = 3;
/// The row artboard's local ids: its box "Words", and the text's axis.
const WORDS: usize = 3;
const AXIS: usize = 10;
const KEY: &str = "a";
/// The rows of the long list: the written row and the others.
const LONG_LIST_ROWS: usize = 300;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    /// The row artboard played as the player's root.
    Root,
    /// A row of a driver page: a keyed list flagged `isPageRows` under a
    /// plain Node of the root.
    PageRows,
    /// A row of the same keyed list under a plain Node, without the flag.
    NodeKeyed,
    /// A row of a keyed list under a CSS layout box of the root.
    LayoutKeyed,
    /// A Screen of a `screens` page: a bound view-model list under a plain
    /// Node of `$nuxie.page`.
    Screens,
}

impl Shape {
    /// The list's local id in the root artboard.
    fn list(self) -> usize {
        match self {
            Shape::Root | Shape::Screens => unreachable!("no keyed list"),
            Shape::PageRows | Shape::NodeKeyed => 2,
            Shape::LayoutKeyed => 6,
        }
    }
}

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

/// The root artboard (file artboard 0, global id 3) holding the list
/// `shape` names, then the row artboard (file artboard 1): a CSS flex
/// artboard holding the box "Words" sized by its Text. The row's
/// artboard-local ids: Artboard 0, its styles 1 and 2, the box 3, its
/// styles 4 and 5, Text 6, style 7, fill 8, color 9, axis 10, run 11.
fn file(shape: Shape, value: f32) -> Vec<u8> {
    use FixtureValue::{Bool, Bytes, Double, String as Str, Uint};
    let mut records = vec![
        record("Backboard", vec![]),
        record("FontAsset", vec![("assetId", Uint(0))]),
        record("FileAssetContents", vec![("bytes", Bytes(FONT.to_vec()))]),
    ];
    let list = |parent: u64, flagged: bool| {
        let mut values = vec![
            ("name", Str("rows".into())),
            ("parentId", Uint(parent)),
            ("rowArtboardId", Uint(1)),
        ];
        if flagged {
            values.push(("isPageRows", Bool(true)));
        }
        record("ArtboardComponentList", values)
    };
    match shape {
        Shape::Root => {
            // A small page nothing plays, so the row artboard is file
            // artboard 1 in every shape.
            records.push(record(
                "Artboard",
                vec![
                    ("name", Str("Page".into())),
                    ("width", Double(400.0)),
                    ("defaultStateMachineId", Uint(0)),
                ],
            ));
        }
        Shape::PageRows | Shape::NodeKeyed => {
            records.push(record(
                "Artboard",
                vec![
                    ("name", Str("Page".into())),
                    ("width", Double(400.0)),
                    ("height", Double(300.0)),
                    ("defaultStateMachineId", Uint(0)),
                ],
            ));
            records.push(record(
                "Node",
                vec![("name", Str("holder".into())), ("parentId", Uint(0))],
            )); // 1
            records.push(list(1, shape == Shape::PageRows)); // 2
        }
        Shape::Screens => {
            // The row's model (view model 0, its instance 0), and the
            // page's, whose list holds one item of the row's model.
            records.extend([
                record("ViewModel", vec![("name", Str("RowModel".into()))]),
                record("ViewModelPropertyNumber", vec![("name", Str("n".into()))]),
                record("ViewModelInstance", vec![("viewModelId", Uint(0))]),
                record(
                    "ViewModelInstanceNumber",
                    vec![
                        ("viewModelPropertyId", Uint(0)),
                        ("propertyValue", Double(0.0)),
                    ],
                ),
                record("ViewModel", vec![("name", Str("PageModel".into()))]),
                record(
                    "ViewModelPropertyList",
                    vec![("name", Str("screens".into()))],
                ),
                record("ViewModelInstance", vec![("viewModelId", Uint(1))]),
                record(
                    "ViewModelInstanceList",
                    vec![("viewModelPropertyId", Uint(0))],
                ),
                record(
                    "ViewModelInstanceListItem",
                    vec![("viewModelId", Uint(0)), ("viewModelInstanceId", Uint(0))],
                ),
                record(
                    "Artboard",
                    vec![
                        ("name", Str("$nuxie.page".into())),
                        ("width", Double(400.0)),
                        ("height", Double(300.0)),
                        ("viewModelId", Uint(1)),
                        ("defaultStateMachineId", Uint(0)),
                    ],
                ),
                record(
                    "Node",
                    vec![("name", Str("overlap".into())), ("parentId", Uint(0))],
                ), // 1
                record(
                    "ArtboardComponentList",
                    vec![("name", Str("screens".into())), ("parentId", Uint(1))],
                ), // 2
                record(
                    "DataBindContext",
                    vec![
                        ("propertyKey", Uint(800)),
                        ("sourcePathIds", Bytes(vec![1, 0])),
                        ("flags", Uint(0)),
                    ],
                ),
            ]);
        }
        Shape::LayoutKeyed => {
            records.push(record(
                "Artboard",
                vec![
                    ("name", Str("Page".into())),
                    ("width", Double(400.0)),
                    ("height", Double(300.0)),
                    ("styleId", Uint(1)),
                    ("defaultStateMachineId", Uint(0)),
                ],
            ));
            records.push(record("LayoutComponentStyle", vec![("parentId", Uint(0))])); // 1
            records.push(record(
                "LayoutCssStyle",
                vec![("parentId", Uint(0)), ("display", Uint(1))],
            )); // 2
            records.push(record(
                "LayoutComponent",
                vec![
                    ("parentId", Uint(0)),
                    ("name", Str("column".into())),
                    ("styleId", Uint(4)),
                ],
            )); // 3
            records.push(record("LayoutComponentStyle", vec![("parentId", Uint(3))])); // 4
            records.push(record("LayoutCssStyle", vec![("parentId", Uint(3))])); // 5
            records.push(list(3, false)); // 6
        }
    }
    records.push(record("StateMachine", vec![("name", Str("run".into()))]));
    records.extend([
        record(
            "Artboard",
            vec![
                ("name", Str("Row".into())),
                ("width", Double(400.0)),
                ("height", Double(100.0)),
                ("styleId", Uint(1)),
                ("defaultStateMachineId", Uint(0)),
                // The row's model, which a `screens` item picks it by.
                ("viewModelId", Uint(0)),
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
                ("tag", Uint(u64::from(WGHT))),
                ("axisValue", Double(value)),
            ],
        ),
        record(
            "TextValueRun",
            vec![
                ("parentId", Uint(6)),
                ("styleId", Uint(7)),
                ("text", Str("AWAKE".into())),
            ],
        ),
        record("StateMachine", vec![("name", Str("run".into()))]),
    ]);
    encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap()
}

type Write = Box<dyn FnOnce(&mut SceneWriter<'_>)>;

/// A driver that makes the queued writes in round 0 of each step.
struct Writes(Rc<RefCell<Vec<Write>>>);

impl HostDriver for Writes {
    fn drive(
        &mut self,
        round: DriverRound<'_>,
        scene: &mut SceneWriter<'_>,
    ) -> Result<(), DriverFault> {
        if round.index == 0 {
            for write in self.0.take() {
                write(scene);
            }
        }
        Ok(())
    }
}

/// What one frame shows of the row's text. Every field is printed; the
/// check reads only some of them.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
struct Frame {
    /// The axis value the style holds.
    axis: f32,
    /// The solved width of the box the text sizes.
    width: f32,
    /// The text's shaped width: its glyph advances summed.
    shaped: f32,
    /// Whether the player's artboard has a layout still to solve.
    root_owed: bool,
    /// Whether the row's artboard has a layout still to solve.
    row_owed: bool,
    /// Whether the row's artboard has Components dirt.
    row_dirty: bool,
    /// Layout solves (`Artboard::calculateLayout`) the step ran, on any
    /// artboard.
    solves: u64,
    /// The most update passes any of the step's settles ran.
    settle_passes: usize,
    /// The step's settles that the loop's limit stopped with work left.
    limit_stops: usize,
    /// The row update passes lists ran in the step.
    row_passes: u64,
    /// The layout solves those row update passes ran.
    row_solves: u64,
}

struct Scene {
    shape: Shape,
    _file: RuntimeFileHandle,
    _factory: PersistentFactory<RecordingFactory>,
    board: ArtboardInstance,
    playback: Playback,
    player: PagePlayer,
    writes: Rc<RefCell<Vec<Write>>>,
    /// The page's model, on a `screens` page.
    models: Vec<RuntimeOwnedViewModelHandle>,
}

impl Scene {
    /// `shape` with the axis at `value`, its row mounted, settled.
    fn new(shape: Shape, value: f32) -> Self {
        Self::with_rows(shape, value, 1)
    }

    /// `shape` with the axis at `value` and `rows` rows mounted in one
    /// round, the written row ("a") first, settled.
    fn with_rows(shape: Shape, value: f32, rows: usize) -> Self {
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &file(shape, value),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("the axis text imports");
        let index = if shape == Shape::Root { 1 } else { 0 };
        let mut board = ArtboardInstance::from_native(file.clone(), index).unwrap();
        let mut machine = board
            .default_state_machine_instance()
            .expect("the artboard's machine");
        let mut models = Vec::new();
        if shape == Shape::Screens {
            let root = board.native_handle();
            let model = file
                .with_file(|f| {
                    f.create_default_view_model_instance_for_artboard(root.core_handle())
                })
                .expect("the page's model");
            let model = RuntimeOwnedViewModelHandle::from_native(file.clone(), model).unwrap();
            board.bind_owned_view_model_handle(model.clone());
            machine.bind_owned_view_model_handle(model.clone());
            models.push(model);
        }
        let writes = Rc::<RefCell<Vec<Write>>>::default();
        let player = PagePlayer::default();
        player
            .set_driver(Some(Box::new(Writes(writes.clone()))))
            .expect("no step is running");
        let mut scene = Self {
            shape,
            _file: file,
            _factory: factory,
            board,
            playback: Playback::StateMachine(Box::new(machine)),
            player,
            writes,
            models,
        };
        if shape == Shape::Screens {
            scene.step(FRAME);
            scene
                .player
                .set_row_collapsed(&mut scene.board, "screens", 0, false, &scene.models, None)
                .expect("the Screen wakes");
        }
        if matches!(
            shape,
            Shape::PageRows | Shape::NodeKeyed | Shape::LayoutKeyed
        ) {
            let list = SceneObject::root(shape.list());
            scene.write(move |scene| {
                assert_eq!(scene.mount_row(&list, KEY.as_bytes()), Ok(true));
                for row in 1..rows {
                    let key = format!("r{row}");
                    assert_eq!(scene.mount_row(&list, key.as_bytes()), Ok(true));
                }
            });
        }
        for _ in 0..4 {
            scene.step(FRAME);
        }
        scene
    }

    fn root(&self) -> RuntimeArtboardInstanceHandle {
        self.board.native_handle()
    }

    fn write(&self, write: impl FnOnce(&mut SceneWriter<'_>) + 'static) {
        self.writes.borrow_mut().push(Box::new(write));
    }

    fn step(&mut self, elapsed_seconds: f32) {
        self.player
            .step(
                &mut self.board,
                &mut self.playback,
                Step {
                    elapsed_seconds,
                    ..Step::default()
                },
                &self.models,
                None,
            )
            .expect("the step");
    }

    /// The artboard the text is in.
    fn row(&self) -> RuntimeArtboardInstanceHandle {
        if self.shape == Shape::Root {
            return self.root();
        }
        let name = if self.shape == Shape::Screens {
            "screens"
        } else {
            "rows"
        };
        let list = self
            .root()
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>(name))
            .expect("the list");
        if self.shape == Shape::Screens {
            return list
                .with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(0))
                .flatten()
                .expect("the Screen");
        }
        list.with_downcast::<ArtboardComponentList, _>(|list| {
            let place = list
                .row_keys()
                .iter()
                .position(|row| *row == KEY.as_bytes())?;
            list.artboard_instance(place as i32)
        })
        .flatten()
        .expect("the mounted row")
    }

    /// The axis as a driver names it.
    fn axis_object(&self) -> SceneObject {
        if self.shape == Shape::Root {
            return SceneObject::root(AXIS);
        }
        let list = self.shape.list();
        SceneObject::in_rows(
            vec![RuntimeGeometryHitPathSegment {
                artboard_global_id: PAGE_GLOBAL_ID,
                local_id: list,
            }],
            vec![RuntimeListRowKey {
                artboard_global_id: PAGE_GLOBAL_ID,
                host_local_id: list,
                key: KEY.as_bytes().into(),
            }],
            AXIS,
        )
    }

    fn handle(&self, local_id: usize) -> CoreHandle {
        self.row()
            .with_artboard(|row| row.base.objects().get(local_id).cloned().flatten())
            .expect("an object of the row")
    }

    /// Whether the row is quiet in its list: skipped by its list's advance
    /// until something wakes it.
    fn row_quiet(&self) -> bool {
        self.row()
            .with_artboard(|row| row.quiet_host_row() != Artboard::NO_QUIET_ROW)
    }

    /// One step, recorded.
    fn recorded_step(&mut self) -> Frame {
        let solves = Artboard::layout_pass_count();
        let rows = ArtboardComponentList::row_updates();
        measure_settles(true);
        self.step(FRAME);
        let settles = measure_settles(false);
        let after = ArtboardComponentList::row_updates();
        let mut frame = self.frame(Artboard::layout_pass_count() - solves);
        frame.settle_passes = settles
            .iter()
            .map(|settle| settle.passes)
            .max()
            .unwrap_or(0);
        frame.limit_stops = settles.iter().filter(|settle| !settle.settled).count();
        frame.row_passes = after.passes - rows.passes;
        frame.row_solves = after.solves - rows.solves;
        frame
    }

    fn frame(&self, solves: u64) -> Frame {
        let row = self.row();
        let words = self.handle(WORDS);
        let text = row.with_artboard(|row| row.base.find_all_handles::<Text>())[0].clone();
        Frame {
            axis: nuxie_runtime::source::generated::core_registry::CoreRegistry::get_double_handle(
                &self.handle(AXIS),
                i32::from(AXIS_VALUE_KEY),
            )
            .unwrap(),
            width: words
                .with_downcast::<LayoutComponent, _>(|layout| layout.layout_bounds().width())
                .unwrap(),
            shaped: text
                .with_downcast::<Text, _>(|text| {
                    text.shape()
                        .iter()
                        .flat_map(|paragraph| paragraph.runs.iter())
                        .flat_map(|run| run.advances.iter().copied())
                        .sum::<f32>()
                })
                .unwrap(),
            root_owed: self.root().with_artboard(|b| b.base.has_dirty_layout()),
            row_owed: row.with_artboard(|b| b.base.has_dirty_layout()),
            row_dirty: row.with_artboard(|b| b.base.has_component_dirt()),
            solves,
            settle_passes: 0,
            limit_stops: 0,
            row_passes: 0,
            row_solves: 0,
        }
    }

    /// The driver writes the axis to `value` in the next step, and each of
    /// `FRAMES` steps is recorded, the write's first.
    fn write_axis(&mut self, value: f32) -> Vec<Frame> {
        self.queue_axis(value);
        (0..FRAMES).map(|_| self.recorded_step()).collect()
    }

    /// The axis written to `value`: by the driver in the next step, or on a
    /// `screens` page through the setter now.
    fn queue_axis(&mut self, value: f32) {
        if self.shape == Shape::Screens {
            // No driver path leads into a `screens` row: the generated
            // setter a driver write goes through, before the step.
            assert!(
                nuxie_runtime::source::generated::core_registry::CoreRegistry::set_double_handle(
                    &self.handle(AXIS),
                    i32::from(AXIS_VALUE_KEY),
                    value,
                )
            );
        } else {
            let axis = self.axis_object();
            self.write(move |scene| {
                assert_eq!(scene.set_number(&axis, AXIS_VALUE_KEY, value), Ok(true));
            });
        }
    }

    /// Sleeps or wakes the driver page's row.
    fn set_row_asleep(&mut self, asleep: bool) {
        assert_eq!(self.shape, Shape::PageRows, "only a page's row sleeps");
        self.player
            .set_keyed_row_collapsed(
                &mut self.board,
                "rows",
                KEY.as_bytes(),
                asleep,
                &self.models,
                None,
            )
            .expect("the row sleeps or wakes");
    }
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.05
}

/// The step of the write reaches `fresh`, settles under the loop's limit
/// and leaves nothing owed; every step after it solves no layout.
fn assert_relaid_out(name: &str, frames: &[Frame], fresh: Frame) {
    for (index, frame) in frames.iter().enumerate() {
        eprintln!("{name} frame {index} {frame:?}");
    }
    let first = frames[0];
    assert_eq!(first.axis, TO, "{name}: the write landed");
    assert!(
        near(first.shaped, fresh.shaped),
        "{name}: in the step of the write the text is shaped at {TO}: {first:?} and {fresh:?}"
    );
    assert!(
        near(first.width, fresh.width),
        "{name}: in the step of the write the box is as wide as a fresh {TO}: {first:?} and {fresh:?}"
    );
    assert!(
        !first.root_owed && !first.row_owed,
        "{name}: no layout is left to solve: {first:?}"
    );
    assert!(
        first.limit_stops == 0 && first.settle_passes < settle_pass_limit(),
        "{name}: the step of the write settles in fewer than {} passes: {first:?}",
        settle_pass_limit()
    );
    for (index, frame) in frames.iter().enumerate().skip(1) {
        assert!(
            frame.solves == 0 && frame.row_solves == 0 && frame.limit_stops == 0,
            "{name}: step {index} after the write solves no layout: {frame:?}"
        );
    }
}

/// Each shape's frames after the write, against the shape built fresh at
/// the new value. Every frame is printed, so a failing shape shows where
/// the second solve was lost.
fn check(shape: Shape) {
    let before = Scene::new(shape, FROM).frame(0);
    let fresh = Scene::new(shape, TO).frame(0);
    eprintln!("{shape:?} before {before:?}");
    eprintln!("{shape:?} fresh  {fresh:?}");
    assert!(
        fresh.width - before.width > 2.0,
        "{shape:?}: the heavier weight is wider: {before:?} and {fresh:?}"
    );
    let mut scene = Scene::new(shape, FROM);
    let frames = scene.write_axis(TO);
    assert_relaid_out(&format!("{shape:?}"), &frames, fresh);
    assert!(
        !before.row_owed && !fresh.row_owed,
        "{shape:?}: at rest the row owes no layout: {before:?} and {fresh:?}"
    );
}

#[test]
fn an_axis_write_relays_out_the_text_played_as_the_root() {
    check(Shape::Root);
}

#[test]
fn an_axis_write_relays_out_a_row_of_a_keyed_list_under_a_layout() {
    check(Shape::LayoutKeyed);
}

#[test]
fn an_axis_write_relays_out_a_row_of_a_driver_page() {
    check(Shape::PageRows);
}

#[test]
fn an_axis_write_relays_out_a_row_of_a_keyed_list_under_a_node() {
    check(Shape::NodeKeyed);
}

#[test]
fn an_axis_write_relays_out_a_screen_of_a_screens_page() {
    check(Shape::Screens);
}

/// A row that solved what it owed goes quiet; the write wakes it and it is
/// laid out in the step of the write.
fn check_quiet(shape: Shape) {
    let fresh = Scene::new(shape, TO).frame(0);
    let mut scene = Scene::new(shape, FROM);
    for _ in 0..4 {
        if scene.row_quiet() {
            break;
        }
        scene.step(FRAME);
    }
    assert!(scene.row_quiet(), "{shape:?}: the row has gone quiet");
    let frames = scene.write_axis(TO);
    assert_relaid_out(&format!("{shape:?} quiet"), &frames, fresh);
}

#[test]
fn an_axis_write_relays_out_a_quiet_row_of_a_driver_page() {
    check_quiet(Shape::PageRows);
}

#[test]
fn an_axis_write_relays_out_a_quiet_row_of_a_keyed_list_under_a_node() {
    check_quiet(Shape::NodeKeyed);
}

#[test]
fn an_axis_write_relays_out_a_quiet_screen_of_a_screens_page() {
    check_quiet(Shape::Screens);
}

/// A sleeping page row keeps the write: it is not laid out while it sleeps,
/// and the settles do not spin on it. It is laid out when it wakes.
#[test]
fn an_axis_write_into_a_sleeping_page_row_is_laid_out_when_it_wakes() {
    let before = Scene::new(Shape::PageRows, FROM).frame(0);
    let fresh = Scene::new(Shape::PageRows, TO).frame(0);
    let mut scene = Scene::new(Shape::PageRows, FROM);
    scene.set_row_asleep(true);
    let asleep = scene.write_axis(TO);
    for (index, frame) in asleep.iter().enumerate() {
        eprintln!("asleep frame {index} {frame:?}");
        assert_eq!(frame.axis, TO, "the write lands in the sleeping row");
        assert!(
            near(frame.width, before.width),
            "a sleeping row is not laid out: {frame:?}"
        );
        assert!(
            frame.limit_stops == 0 && frame.settle_passes < settle_pass_limit(),
            "a sleeping row's owed layout does not hold the settle: {frame:?}"
        );
        assert_eq!(
            frame.row_solves, 0,
            "a sleeping row solves nothing: {frame:?}"
        );
    }
    scene.set_row_asleep(false);
    // The wake runs an update pass itself, which may lay the row out; the
    // first step after it shows the fresh width either way.
    let woken = scene.frame(0);
    eprintln!("woken before its first step {woken:?}");
    let frames = (0..FRAMES)
        .map(|_| scene.recorded_step())
        .collect::<Vec<_>>();
    assert_relaid_out("woken", &frames, fresh);
}

/// One row written among `LONG_LIST_ROWS` rows of a keyed list under a
/// plain Node. Only the written row owes a layout, so only it runs the
/// extra update pass. The counts are printed.
#[test]
fn an_axis_write_relays_out_one_row_of_a_long_list_under_a_node() {
    let fresh = Scene::new(Shape::NodeKeyed, TO).frame(0);
    let mut scene = Scene::with_rows(Shape::NodeKeyed, FROM, LONG_LIST_ROWS);
    let rest = scene.recorded_step();
    eprintln!("long list at rest {rest:?}");
    let frames = scene.write_axis(TO);
    assert_relaid_out(&format!("long list of {LONG_LIST_ROWS}"), &frames, fresh);
}
