//! A page whose one semantic node sits over an empty text, captured through
//! the C edge (UNIV-7138): the node's box is the empty text's from the first
//! capture on, and the tree version holds while nothing changes.
//!
//! Oracle: an auto-sized text with no words, at the top left of a block
//! column at the page's top left, has an empty box at the page's origin,
//! (0, 0, 0, 0); a page where nothing moves keeps the tree version a host
//! captured.
use super::*;
use nuxie_binary::{
    FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, encode_runtime_file,
};

/// The still frames over which a captured tree version must hold.
const STILL_FRAMES: usize = 120;

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

/// A column holding one auto-sized text with no words and a semantic record
/// on the text, played by a state machine named "machine" with one empty
/// layer.
fn empty_text_page() -> Vec<u8> {
    use FixtureValue::{Bytes, Double, String as Str, Uint};
    let records = vec![
        record("Backboard", vec![]),
        record("FontAsset", vec![("assetId", Uint(0))]),
        record(
            "FileAssetContents",
            vec![(
                "bytes",
                Bytes(include_bytes!("../../../fixtures/fonts/roboto-input.ttf").to_vec()),
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
                ("text", Str(String::new())),
            ],
        ), // 10
        record("SemanticData", vec![("parentId", Uint(6))]),         // 11
        record("StateMachine", vec![("name", Str("machine".into()))]),
        record("StateMachineLayer", vec![("name", Str("layer".into()))]),
        record("EntryState", vec![]),
        record("AnyState", vec![]),
        record("ExitState", vec![]),
    ];
    encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap()
}

/// The page played by its machine with semantics on.
struct Host {
    file: *mut NuxFile,
    instance: *mut NuxArtboardInstance,
    player: *mut NuxPlayer,
}

impl Host {
    fn new() -> Self {
        let bytes = empty_text_page();
        let mut file = ptr::null_mut();
        let mut instance = ptr::null_mut();
        let mut player = ptr::null_mut();
        let machine = "machine";
        unsafe {
            assert_eq!(
                nux_file_import(
                    bytes.as_ptr(),
                    bytes.len(),
                    &NuxRenderCallbacks::default(),
                    &mut file
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut instance),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_new_state_machine_named(
                    instance,
                    NuxStringView {
                        data: machine.as_ptr().cast(),
                        len: machine.len(),
                    },
                    &mut player
                ),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
        }
        Self {
            file,
            instance,
            player,
        }
    }

    /// One step of `elapsed_seconds`, presented.
    fn present(&self, elapsed_seconds: f32) {
        let step = NuxPlayerStep {
            elapsed_seconds,
            ..NuxPlayerStep::default()
        };
        let mut result = ptr::null_mut();
        let mut scheduling = NuxPlayerSchedulingInfo {
            struct_size: std::mem::size_of::<NuxPlayerSchedulingInfo>() as u32,
            ..Default::default()
        };
        unsafe {
            assert_eq!(
                nux_player_step(self.player, &step, &mut result),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_step_result_scheduling(result, &mut scheduling),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_acknowledge_presented(self.player, scheduling.render_revision),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
        }
    }

    /// A capture's tree version and its one node's box.
    fn capture(&self) -> (u64, [f32; 4]) {
        let mut snapshot = ptr::null_mut();
        let mut info = NuxSemanticSnapshotInfo {
            struct_size: std::mem::size_of::<NuxSemanticSnapshotInfo>() as u32,
            ..Default::default()
        };
        let mut node = NuxSemanticNodeView {
            struct_size: std::mem::size_of::<NuxSemanticNodeView>() as u32,
            ..Default::default()
        };
        unsafe {
            assert_eq!(
                nux_player_semantic_snapshot(self.player, &mut snapshot),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_semantic_snapshot_info(snapshot, &mut info),
                NuxStatus::Ok
            );
            assert_eq!(info.node_count, 1, "one node");
            assert_eq!(
                nux_semantic_snapshot_node(snapshot, 0, &mut node),
                NuxStatus::Ok
            );
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
        }
        (
            info.tree_version,
            [node.min_x, node.min_y, node.max_x, node.max_y],
        )
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(nux_player_free(self.player), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(self.instance), NuxStatus::Ok);
            assert_eq!(nux_file_free(self.file), NuxStatus::Ok);
        }
    }
}

/// From the first capture after the first presented frame, the empty
/// text's node has its empty box at the origin and the tree version holds
/// over 120 still frames, played by zero-second steps and by timed ones.
#[test]
fn an_empty_texts_capture_holds_its_version_and_box() {
    for (label, seconds) in [("zero-second steps", 0.0), ("timed steps", 1.0 / 60.0)] {
        let host = Host::new();
        host.present(seconds);
        let (version, node_box) = host.capture();
        assert_eq!(node_box, [0.0; 4], "{label}: the first capture's box");
        for frame in 1..=STILL_FRAMES {
            host.present(seconds);
            assert_eq!(
                host.capture(),
                (version, node_box),
                "{label}: the capture after {frame} more still frames"
            );
        }
    }
}
