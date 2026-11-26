//! A host's reveal of a semantic node and move of a scroll region, through
//! the C edge, on pages whose buttons sit in a CSS scroll box 100 tall over
//! 300 points of content (`scroll_reveal_fixture`).
//!
//! Oracle: the fixture's CSS arithmetic for where each button is drawn, and
//! the smallest move that shows a box in a viewport: a box below the fold
//! comes to rest with its bottom edge on the viewport's.
use super::*;
#[path = "scroll_reveal_fixture.rs"]
mod fixture;
use fixture::{
    BOTTOM, DEEP, FAR, MIDDLE, Page, Physics, ROW_BUTTON, ROWS, START, TOP, USED_BUTTON,
    scroll_reveal_fixture,
};
use std::ffi::c_void;

/// How many rows the driver mounts in its first round.
struct Mount {
    rows: usize,
    mounted: bool,
}

unsafe extern "C" fn mount_rows(
    context: *mut c_void,
    round: *const NuxPlayerDriverRound,
    writer: *mut NuxSceneWriter,
) -> NuxStatus {
    let mount = unsafe { &mut *context.cast::<Mount>() };
    if mount.mounted || unsafe { (*round).index } != 0 {
        return NuxStatus::Ok;
    }
    mount.mounted = true;
    let list = NuxSceneObject {
        struct_size: std::mem::size_of::<NuxSceneObject>() as u32,
        local_id: ROWS,
        path: ptr::null(),
        path_len: 0,
        row_keys: ptr::null(),
        row_key_count: 0,
    };
    for row in 0..mount.rows {
        let key = format!("row {row}");
        let status = unsafe {
            nux_scene_writer_mount_row(
                writer,
                &list,
                NuxByteView {
                    data: key.as_ptr(),
                    len: key.len(),
                },
                ptr::null_mut(),
            )
        };
        if status != NuxStatus::Ok {
            return status;
        }
    }
    NuxStatus::Ok
}

/// What plays the page.
#[derive(Clone, Copy, PartialEq)]
enum Plays {
    Machine,
    Nothing,
}

/// A page playing with semantics on, presented.
struct Host {
    file: *mut NuxFile,
    instance: *mut NuxArtboardInstance,
    player: *mut NuxPlayer,
    _mount: Box<Mount>,
}

/// The most steps a move is given to settle: ten seconds at 60 a second.
const MAX_SETTLE_STEPS: usize = 600;

impl Host {
    fn new(page: Page, physics: Physics) -> Self {
        Self::played(page, physics, Plays::Machine)
    }

    fn played(page: Page, physics: Physics, plays: Plays) -> Self {
        let bytes = scroll_reveal_fixture(page, physics);
        let mut file = ptr::null_mut();
        let mut instance = ptr::null_mut();
        let mut player = ptr::null_mut();
        let mut mount = Box::new(Mount {
            rows: if page == Page::Hosted { 2 } else { 0 },
            mounted: false,
        });
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
            let made = match plays {
                Plays::Machine => nux_player_new_default(instance, &mut player),
                Plays::Nothing => nux_player_new_static(instance, &mut player),
            };
            assert_eq!(made, NuxStatus::Ok);
            assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
            let driver = NuxPlayerDriver {
                struct_size: std::mem::size_of::<NuxPlayerDriver>() as u32,
                context: (&raw mut *mount).cast(),
                drive: Some(mount_rows),
                release: None,
            };
            assert_eq!(nux_player_driver_set(player, &driver), NuxStatus::Ok);
        }
        let scene = Self {
            file,
            instance,
            player,
            _mount: mount,
        };
        for _ in 0..10 {
            scene.present();
        }
        assert!(scene.present().settled, "the page starts at rest");
        scene
    }

    /// One step of a sixtieth of a second, presented; its scheduling.
    fn present(&self) -> NuxPlayerSchedulingInfo {
        let step = NuxPlayerStep {
            elapsed_seconds: 1.0 / 60.0,
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
        scheduling
    }

    /// Steps until the player reports itself settled; how many it took.
    fn settle(&self) -> usize {
        (1..=MAX_SETTLE_STEPS)
            .find(|_| self.present().settled)
            .expect("the move settles within ten seconds")
    }

    fn capture(&self) -> Result<Capture, NuxStatus> {
        let mut snapshot = ptr::null_mut();
        let status = unsafe { nux_player_semantic_snapshot(self.player, &mut snapshot) };
        if status != NuxStatus::Ok {
            assert!(snapshot.is_null(), "a failure publishes no capture");
            return Err(status);
        }
        Ok(Capture(snapshot))
    }

    fn reveal(&self, capture: &Capture, node: u32) -> (NuxStatus, bool) {
        let mut moved = true;
        let status =
            unsafe { nux_player_reveal_semantic_node(self.player, capture.0, node, &mut moved) };
        (status, moved)
    }

    fn scroll_by(&self, capture: &Capture, region: u32, dx: f32, dy: f32) -> (NuxStatus, bool) {
        let mut moved = true;
        let status = unsafe {
            nux_player_scroll_region_by(self.player, capture.0, region, dx, dy, &mut moved)
        };
        (status, moved)
    }

    /// Reveal the first node labeled `label` from a fresh capture.
    fn reveal_label(&self, label: &str) -> bool {
        let capture = self.capture().expect("a capture");
        let (status, moved) = self.reveal(&capture, capture.node(label).id);
        assert_eq!(status, NuxStatus::Ok);
        moved
    }

    /// Move region `region` from a fresh capture.
    fn scroll_label(&self, region: u32, dx: f32, dy: f32) -> bool {
        let capture = self.capture().expect("a capture");
        let (status, moved) = self.scroll_by(&capture, region, dx, dy);
        assert_eq!(status, NuxStatus::Ok);
        moved
    }

    /// The live semantic node `id`.
    fn live_node(&self, id: u32) -> nuxie::runtime::semantic::semantic_node::SemanticNodeRef {
        unsafe { &*self.player }
            .artboard
            .instance
            .borrow()
            .native_handle()
            .with_artboard(|artboard| artboard.semantic_manager())
            .expect("semantics are on")
            .with_semantic_manager(|manager| manager.node_by_id(id))
            .expect("a live node")
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

struct Capture(*mut NuxSemanticSnapshot);

impl Capture {
    fn info(&self) -> NuxSemanticSnapshotInfo {
        let mut info = NuxSemanticSnapshotInfo {
            struct_size: std::mem::size_of::<NuxSemanticSnapshotInfo>() as u32,
            ..Default::default()
        };
        assert_eq!(
            unsafe { nux_semantic_snapshot_info(self.0, &mut info) },
            NuxStatus::Ok
        );
        info
    }

    fn nodes(&self) -> Vec<(String, NuxSemanticNodeView)> {
        (0..self.info().node_count)
            .map(|index| {
                let mut node = NuxSemanticNodeView {
                    struct_size: std::mem::size_of::<NuxSemanticNodeView>() as u32,
                    ..Default::default()
                };
                assert_eq!(
                    unsafe { nux_semantic_snapshot_node(self.0, index, &mut node) },
                    NuxStatus::Ok
                );
                let label = unsafe {
                    std::slice::from_raw_parts(node.label.data.cast::<u8>(), node.label.len)
                };
                (String::from_utf8_lossy(label).into_owned(), node)
            })
            .collect()
    }

    /// Every node labeled `label`, in tree order.
    fn labeled(&self, label: &str) -> Vec<NuxSemanticNodeView> {
        self.nodes()
            .into_iter()
            .filter(|(name, _)| name == label)
            .map(|(_, node)| node)
            .collect()
    }

    fn node(&self, label: &str) -> NuxSemanticNodeView {
        *self
            .labeled(label)
            .first()
            .unwrap_or_else(|| panic!("a node labeled {label}"))
    }

    fn regions(&self) -> Vec<NuxScrollRegionView> {
        (0..self.info().scroll_region_count)
            .map(|index| {
                let mut view = NuxScrollRegionView {
                    struct_size: std::mem::size_of::<NuxScrollRegionView>() as u32,
                    ..Default::default()
                };
                assert_eq!(
                    unsafe { nux_semantic_snapshot_scroll_region(self.0, index, &mut view) },
                    NuxStatus::Ok
                );
                view
            })
            .collect()
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        assert_eq!(unsafe { nux_semantic_snapshot_free(self.0) }, NuxStatus::Ok);
    }
}

fn node_box(node: &NuxSemanticNodeView) -> [f32; 4] {
    [node.min_x, node.min_y, node.max_x, node.max_y]
}

fn region_box(region: &NuxScrollRegionView) -> [f32; 4] {
    [region.min_x, region.min_y, region.max_x, region.max_y]
}

/// `[scroll_x, scroll_y, range_x, range_y]`.
fn position(region: &NuxScrollRegionView) -> [f32; 4] {
    [
        region.scroll_x,
        region.scroll_y,
        region.range_x,
        region.range_y,
    ]
}

const TAP: u32 = 1;
const CAN_TOP: u32 = 1 << 0;
const CAN_BOTTOM: u32 = 1 << 1;

/// The baseline: the bottom button is listed in the box's region, below
/// it, with no actions, and the region stands at its start with a range of
/// 200 (300 of content in 100), along its one axis.
#[test]
fn a_button_below_the_fold_is_listed_in_its_region() {
    let scene = Host::new(Page::Buttons, Physics::Elastic);
    let capture = scene.capture().expect("a capture");
    let regions = capture.regions();
    assert_eq!(regions.len(), 1);
    assert_eq!(region_box(&regions[0]), [0.0, 0.0, 200.0, 100.0]);
    assert_eq!(position(&regions[0]), [0.0, 0.0, 0.0, 200.0]);
    let bottom = capture.node(BOTTOM);
    assert_eq!(node_box(&bottom), [0.0, 220.0, 100.0, 260.0]);
    assert_eq!((bottom.scroll_region, bottom.actions), (0, 0));
    let top = capture.node(TOP);
    assert_eq!((top.scroll_region, top.actions), (0, TAP));
}

/// Revealed, the bottom button comes to rest with its bottom edge on the
/// viewport's, 160 points along, and takes its tap. The scroller animates:
/// the call answers that it moved and makes the capture stale; the first
/// capture after the next presented step already reads the target, 160,
/// while the button's box is still in flight; the player reports itself
/// unsettled on every step until the move rests, so a host that draws on
/// demand keeps getting frames.
#[test]
fn a_reveal_animates_the_box_until_the_button_shows() {
    let scene = Host::new(Page::Buttons, Physics::Elastic);
    let capture = scene.capture().expect("a capture");
    let bottom = capture.node(BOTTOM).id;
    let (status, moved) = scene.reveal(&capture, bottom);
    assert_eq!((status, moved), (NuxStatus::Ok, true));
    assert_eq!(
        unsafe { nux_player_validate_semantic_snapshot(scene.player, capture.0) },
        NuxStatus::HandleMismatch,
        "a move makes the capture stale"
    );
    assert_eq!(
        scene.capture().err(),
        Some(NuxStatus::HandleMismatch),
        "nothing new is captured until the move is presented"
    );
    drop(capture);

    let first = scene.present();
    assert!(!first.settled, "a settling move is not settled");
    let capture = scene.capture().expect("a capture");
    assert_eq!(position(&capture.regions()[0]), [0.0, 160.0, 0.0, 200.0]);
    let flying = capture.node(BOTTOM);
    assert!(
        flying.min_y > 60.0 && flying.min_y < 220.0,
        "in flight after one step: {}",
        flying.min_y
    );
    drop(capture);

    let steps = scene.settle();
    assert!(steps > 1, "the move takes frames: {steps}");
    let capture = scene.capture().expect("a capture");
    let shown = capture.node(BOTTOM);
    assert_eq!(node_box(&shown), [0.0, 60.0, 100.0, 100.0]);
    assert_eq!(shown.actions, TAP);
    assert_eq!(position(&capture.regions()[0]), [0.0, 160.0, 0.0, 200.0]);
    assert_eq!(capture.regions()[0].can_scroll, CAN_TOP | CAN_BOTTOM);
}

/// Under clamped physics, whose `scrollToPosition` is upstream's empty
/// default, a reveal still moves: the offset is set at once, the call
/// answers that it moved, the first presented step draws the button where
/// it rests, and the step after it is settled. A second reveal moves
/// nothing and says so.
#[test]
fn a_reveal_under_clamped_physics_moves_at_once() {
    let scene = Host::new(Page::Buttons, Physics::Clamped);
    let capture = scene.capture().expect("a capture");
    let bottom = capture.node(BOTTOM).id;
    assert_eq!(scene.reveal(&capture, bottom), (NuxStatus::Ok, true));
    drop(capture);
    scene.present();
    let capture = scene.capture().expect("a capture");
    assert_eq!(node_box(&capture.node(BOTTOM)), [0.0, 60.0, 100.0, 100.0]);
    assert_eq!(position(&capture.regions()[0]), [0.0, 160.0, 0.0, 200.0]);
    let bottom = capture.node(BOTTOM).id;
    assert_eq!(scene.reveal(&capture, bottom), (NuxStatus::Ok, false));
    assert!(scene.present().settled, "no animation to wait for");
}

/// A static player passes no time on its steps, so an animated reveal
/// would never arrive: there a reveal moves even an elastic scroller at
/// once, the first presented step draws the button where it rests and
/// reports itself settled, and a second reveal moves nothing. Its capture
/// lists no regions (only a state-machine player's does), so nothing
/// there can be moved by region.
#[test]
fn a_static_player_reveals_at_once() {
    let still = Host::played(Page::Buttons, Physics::Elastic, Plays::Nothing);
    let capture = still.capture().expect("a capture");
    assert_eq!(capture.info().scroll_region_count, 0);
    assert_eq!(node_box(&capture.node(BOTTOM)), [0.0, 220.0, 100.0, 260.0]);
    let bottom = capture.node(BOTTOM).id;
    assert_eq!(still.reveal(&capture, bottom), (NuxStatus::Ok, true));
    drop(capture);
    assert!(still.present().settled, "nothing left to animate");
    let capture = still.capture().expect("a capture");
    assert_eq!(node_box(&capture.node(BOTTOM)), [0.0, 60.0, 100.0, 100.0]);
    let bottom = capture.node(BOTTOM).id;
    assert_eq!(still.reveal(&capture, bottom), (NuxStatus::Ok, false));
}

/// A button already in view moves nothing: the call answers false and the
/// capture stays valid.
#[test]
fn revealing_a_button_in_view_moves_nothing() {
    let scene = Host::new(Page::Buttons, Physics::Elastic);
    let capture = scene.capture().expect("a capture");
    let (status, moved) = scene.reveal(&capture, capture.node(TOP).id);
    assert_eq!((status, moved), (NuxStatus::Ok, false));
    assert_eq!(
        unsafe { nux_player_validate_semantic_snapshot(scene.player, capture.0) },
        NuxStatus::Ok
    );
    assert!(scene.present().settled);
    assert_eq!(position(&capture.regions()[0]), [0.0, 0.0, 0.0, 200.0]);
}

/// A move by a negative `dy` shows content below, by exactly that much,
/// and a positive one content above; a move past the end stops at it and
/// a move further answers false, with the bottom bit cleared. A sideways
/// move of a vertical region is ignored. Without physics, and with
/// clamped physics, which cannot animate to a target, the first presented
/// step draws the content where it rests, and the step after it (a changed
/// step asks for one more, as any changed advance does) is settled.
#[test]
fn scroll_by_moves_by_the_amount_and_stops_at_the_end() {
    for physics in [Physics::Elastic, Physics::None, Physics::Clamped] {
        let scene = Host::new(Page::Buttons, physics);
        let mut expected = 0.0;
        for (dy, rests_at) in [(-80.0, 80.0), (-80.0, 160.0), (-80.0, 200.0), (50.0, 150.0)] {
            assert!(scene.scroll_label(0, 0.0, dy), "{physics:?}: {dy} moves");
            if physics == Physics::Elastic {
                scene.settle();
            } else {
                scene.present();
            }
            let capture = scene.capture().expect("a capture");
            assert_eq!(position(&capture.regions()[0]), [0.0, rests_at, 0.0, 200.0]);
            assert_eq!(node_box(&capture.node(TOP))[1], -rests_at);
            expected = rests_at;
            if rests_at == 200.0 {
                assert_eq!(capture.regions()[0].can_scroll, CAN_TOP);
                drop(capture);
                assert!(!scene.scroll_label(0, 0.0, -1.0), "past the end");
                assert!(scene.present().settled);
            } else if physics != Physics::Elastic {
                assert!(scene.present().settled, "{physics:?}: nothing animates");
            }
        }
        assert_eq!(expected, 150.0);
        assert!(!scene.scroll_label(0, -50.0, 0.0), "no sideways axis");
        assert!(!scene.scroll_label(0, 0.0, 0.0), "no move");
        let capture = scene.capture().expect("a capture");
        assert_eq!(position(&capture.regions()[0]), [0.0, 150.0, 0.0, 200.0]);
        assert_eq!(capture.node(MIDDLE).min_y, 0.0);
    }
}

/// Moves made before a settling one rests add up: a second move starts
/// from the first's target, not from where the content is drawn.
#[test]
fn a_move_while_one_settles_starts_from_its_target() {
    let scene = Host::new(Page::Buttons, Physics::Elastic);
    assert!(scene.scroll_label(0, 0.0, -80.0));
    assert!(!scene.present().settled);
    assert!(scene.scroll_label(0, 0.0, -80.0));
    scene.settle();
    let capture = scene.capture().expect("a capture");
    assert_eq!(position(&capture.regions()[0]), [0.0, 160.0, 0.0, 200.0]);
    assert_eq!(node_box(&capture.node(BOTTOM)), [0.0, 60.0, 100.0, 100.0]);
}

/// A button inside an inner scroller below the fold: one reveal moves both
/// scrollers. The inner viewport starts clipped away by the outer one, so
/// it is not yet a region and the button names the outer region. Each move
/// is worked out from where the button is drawn
/// before either moves: the inner one by 80, which shows it at the inner
/// viewport's bottom, and the outer one by 190, which would show it had
/// the inner one not moved. So it rests 20 points above the outer
/// viewport's top, partly shown, and a second reveal brings it whole.
#[test]
fn one_reveal_moves_nested_scrollers() {
    let scene = Host::new(Page::Nested, Physics::Elastic);
    let capture = scene.capture().expect("a capture");
    let regions = capture.regions();
    assert_eq!(regions.len(), 1, "the inner viewport is clipped away");
    assert_eq!(position(&regions[0]), [0.0, 0.0, 0.0, 200.0]);
    let deep = capture.node(DEEP);
    assert_eq!(deep.scroll_region, 0);
    assert_eq!(node_box(&deep), [0.0, 250.0, 100.0, 290.0]);
    drop(capture);

    assert!(scene.reveal_label(DEEP));
    scene.settle();
    let capture = scene.capture().expect("a capture");
    let regions = capture.regions();
    assert_eq!(regions.len(), 2);
    assert_eq!(regions[1].parent, 0);
    assert_eq!(position(&regions[0]), [0.0, 190.0, 0.0, 200.0]);
    assert_eq!(position(&regions[1]), [0.0, 80.0, 0.0, 80.0]);
    let deep = capture.node(DEEP);
    assert_eq!(deep.scroll_region, 1);
    assert_eq!(node_box(&deep), [0.0, -20.0, 100.0, 20.0]);
    assert_eq!(deep.actions, TAP, "partly shown, so pressable");
    drop(capture);

    assert!(scene.reveal_label(DEEP), "the second reveal finishes it");
    scene.settle();
    let capture = scene.capture().expect("a capture");
    assert_eq!(position(&capture.regions()[0]), [0.0, 170.0, 0.0, 200.0]);
    assert_eq!(node_box(&capture.node(DEEP)), [0.0, 0.0, 100.0, 40.0]);
    drop(capture);
    assert!(!scene.reveal_label(DEEP), "whole, nothing more moves");
}

/// A button in a used component, and one in a keyed list's row, each in the
/// scroll box: a reveal crosses the host to the scroller above it.
#[test]
fn a_reveal_crosses_a_used_component_and_a_list_row() {
    let scene = Host::new(Page::Hosted, Physics::Elastic);
    let capture = scene.capture().expect("a capture");
    let used = capture.node(USED_BUTTON);
    assert_eq!(node_box(&used), [0.0, 150.0, 200.0, 190.0]);
    let rows = capture.labeled(ROW_BUTTON);
    assert_eq!(rows.len(), 2);
    assert_eq!(node_box(&rows[1]), [0.0, 260.0, 200.0, 300.0]);
    assert_eq!((used.scroll_region, rows[1].scroll_region), (0, 0));
    drop(capture);

    assert!(scene.reveal_label(USED_BUTTON));
    scene.settle();
    let capture = scene.capture().expect("a capture");
    assert_eq!(position(&capture.regions()[0]), [0.0, 90.0, 0.0, 200.0]);
    assert_eq!(
        node_box(&capture.node(USED_BUTTON)),
        [0.0, 60.0, 200.0, 100.0]
    );
    let second_row = capture.labeled(ROW_BUTTON)[1].id;
    let (status, moved) = scene.reveal(&capture, second_row);
    assert_eq!((status, moved), (NuxStatus::Ok, true));
    drop(capture);
    scene.settle();
    let capture = scene.capture().expect("a capture");
    assert_eq!(position(&capture.regions()[0]), [0.0, 200.0, 0.0, 200.0]);
    assert_eq!(
        node_box(&capture.labeled(ROW_BUTTON)[1]),
        [0.0, 60.0, 200.0, 100.0]
    );
}

/// Right to left, sideways: the start is the viewport's right edge, so the
/// far button is 200 points to its left. A reveal brings it to the
/// viewport's left edge, the region's scroll counting from the start. A
/// negative `dx` moves the content left, showing what is to its right,
/// back toward the start; at the start a move that way answers false.
#[test]
fn right_to_left_reveal_and_scroll_count_from_the_start() {
    let scene = Host::new(Page::Sideways, Physics::Elastic);
    let capture = scene.capture().expect("a capture");
    assert_eq!(region_box(&capture.regions()[0]), [0.0, 0.0, 100.0, 40.0]);
    assert_eq!(position(&capture.regions()[0]), [0.0, 0.0, 200.0, 0.0]);
    assert_eq!(node_box(&capture.node(START)), [0.0, 0.0, 100.0, 40.0]);
    assert_eq!(node_box(&capture.node(FAR)), [-200.0, 0.0, -100.0, 40.0]);
    let (status, moved) = scene.scroll_by(&capture, 0, -10.0, 0.0);
    assert_eq!((status, moved), (NuxStatus::Ok, false), "at the start");
    drop(capture);

    assert!(scene.reveal_label(FAR));
    scene.settle();
    let capture = scene.capture().expect("a capture");
    assert_eq!(position(&capture.regions()[0]), [200.0, 0.0, 200.0, 0.0]);
    assert_eq!(node_box(&capture.node(FAR)), [0.0, 0.0, 100.0, 40.0]);
    drop(capture);

    assert!(scene.scroll_label(0, -50.0, 0.0));
    scene.settle();
    let capture = scene.capture().expect("a capture");
    assert_eq!(position(&capture.regions()[0]), [150.0, 0.0, 200.0, 0.0]);
    assert_eq!(node_box(&capture.node(FAR)), [-50.0, 0.0, 50.0, 40.0]);
}

/// A disabled button out of view has no actions and is still revealed.
#[test]
fn a_disabled_button_is_revealed() {
    use nuxie::runtime::semantic::semantic_data::SemanticData;
    let scene = Host::new(Page::Buttons, Physics::Elastic);
    let capture = scene.capture().expect("a capture");
    let data = scene
        .live_node(capture.node(BOTTOM).id)
        .borrow()
        .semantic_data
        .clone()
        .expect("the button's data");
    drop(capture);
    data.with_downcast_mut::<SemanticData, _>(|data| data.set_is_disabled(true))
        .expect("semantic data");
    scene.present();
    let capture = scene.capture().expect("a capture");
    let bottom = capture.node(BOTTOM);
    assert_eq!(bottom.actions, 0);
    let (status, moved) = scene.reveal(&capture, bottom.id);
    assert_eq!((status, moved), (NuxStatus::Ok, true));
    drop(capture);
    scene.settle();
    let capture = scene.capture().expect("a capture");
    let bottom = capture.node(BOTTOM);
    assert_eq!(node_box(&bottom), [0.0, 60.0, 100.0, 100.0]);
    assert_eq!(bottom.actions, 0, "still disabled");
}

/// What each call refuses, and that a refusal moves nothing.
#[test]
fn refusals() {
    let scene = Host::new(Page::Hosted, Physics::Elastic);
    let capture = scene.capture().expect("a capture");
    let count = capture.info().scroll_region_count as u32;
    assert_eq!(count, 1);
    let missing = capture
        .nodes()
        .iter()
        .map(|(_, node)| node.id)
        .max()
        .unwrap()
        + 1;
    assert_eq!(
        scene.reveal(&capture, missing),
        (NuxStatus::NotFound, false)
    );
    // A boundary node, standing for a hosted artboard, has no semantic data
    // and would be refused, but a capture lists none: the used component's
    // and the rows' nodes are listed in their own right.
    assert!(
        capture
            .nodes()
            .iter()
            .all(|(_, node)| !scene.live_node(node.id).borrow().is_boundary_node)
    );
    assert_eq!(
        scene.scroll_by(&capture, count, 0.0, -10.0),
        (NuxStatus::NotFound, false)
    );
    for (dx, dy) in [
        (f32::NAN, 0.0),
        (0.0, f32::INFINITY),
        (f32::NEG_INFINITY, 0.0),
    ] {
        assert_eq!(
            scene.scroll_by(&capture, 0, dx, dy),
            (NuxStatus::InvalidArgument, false)
        );
    }
    unsafe {
        let bottom = capture.labeled(ROW_BUTTON)[1].id;
        assert_eq!(
            nux_player_reveal_semantic_node(scene.player, capture.0, bottom, ptr::null_mut()),
            NuxStatus::NullArgument
        );
        assert_eq!(
            nux_player_scroll_region_by(scene.player, capture.0, 0, 0.0, -1.0, ptr::null_mut()),
            NuxStatus::NullArgument
        );
    }
    assert!(scene.present().settled, "no refusal moved anything");

    // A capture a move made stale is refused by both calls.
    let capture = scene.capture().expect("a capture");
    let row = capture.labeled(ROW_BUTTON)[1].id;
    assert_eq!(scene.reveal(&capture, row), (NuxStatus::Ok, true));
    assert_eq!(
        scene.reveal(&capture, row),
        (NuxStatus::HandleMismatch, false)
    );
    assert_eq!(
        scene.scroll_by(&capture, 0, 0.0, -10.0),
        (NuxStatus::HandleMismatch, false)
    );
    drop(capture);

    // A static player's capture has no regions to move.
    let still = Host::played(Page::Buttons, Physics::Elastic, Plays::Nothing);
    let capture = still.capture().expect("a capture");
    assert_eq!(capture.info().scroll_region_count, 0);
    assert_eq!(
        still.scroll_by(&capture, 0, 0.0, -10.0),
        (NuxStatus::NotFound, false)
    );
}

/// The region view's layout, and an older caller: a view of the size
/// before the position fields reads the rest, leaves its bytes past them
/// alone, and learns the runtime's size.
#[test]
fn region_view_layout_and_older_callers() {
    use std::mem::{offset_of, size_of};
    assert_eq!(size_of::<NuxScrollRegionView>(), 48);
    assert_eq!(
        [
            offset_of!(NuxScrollRegionView, scroll_x),
            offset_of!(NuxScrollRegionView, scroll_y),
            offset_of!(NuxScrollRegionView, range_x),
            offset_of!(NuxScrollRegionView, range_y),
        ],
        [32, 36, 40, 44]
    );
    assert_eq!(NUX_SCROLL_REGION_VIEW_MIN_SIZE, 32);

    let scene = Host::new(Page::Buttons, Physics::Elastic);
    let capture = scene.capture().expect("a capture");
    let mut old = NuxScrollRegionView {
        struct_size: NUX_SCROLL_REGION_VIEW_MIN_SIZE as u32,
        scroll_x: 7.0,
        scroll_y: 7.0,
        range_x: 7.0,
        range_y: 7.0,
        ..Default::default()
    };
    assert_eq!(
        unsafe { nux_semantic_snapshot_scroll_region(capture.0, 0, &mut old) },
        NuxStatus::Ok
    );
    assert_eq!(old.struct_size, 48, "the runtime's size");
    assert_eq!(region_box(&old), [0.0, 0.0, 200.0, 100.0]);
    assert_eq!(position(&old), [7.0; 4], "past the caller's size");
}

/// Upstream's reveal arithmetic, kept: a reveal works out each move from
/// where the node is drawn but adds it to where the scroller is going. So
/// revealing the bottom button again one step into its first reveal, while
/// it is drawn at 180 and the scroller is going to 160, adds the 120 points
/// it still has to travel to 160: past the range, so the content runs to
/// its end and the button rests 40 points higher than one reveal puts it.
/// A host reveals again only once the player reports itself settled.
#[test]
fn a_second_reveal_while_one_settles_overshoots() {
    let scene = Host::new(Page::Buttons, Physics::Elastic);
    assert!(scene.reveal_label(BOTTOM));
    scene.present();
    let capture = scene.capture().expect("a capture");
    let flying = capture.node(BOTTOM);
    assert_eq!(node_box(&flying), [0.0, 180.0, 100.0, 220.0]);
    assert_eq!(position(&capture.regions()[0]), [0.0, 160.0, 0.0, 200.0]);
    assert_eq!(scene.reveal(&capture, flying.id), (NuxStatus::Ok, true));
    drop(capture);
    scene.settle();
    let capture = scene.capture().expect("a capture");
    assert_eq!(position(&capture.regions()[0]), [0.0, 200.0, 0.0, 200.0]);
    assert_eq!(node_box(&capture.node(BOTTOM)), [0.0, 20.0, 100.0, 60.0]);
}
