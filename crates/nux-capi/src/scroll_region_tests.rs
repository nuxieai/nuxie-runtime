//! The scroll regions a semantic capture carries, through the C edge, on
//! keyed list rows that each hold a CSS scroll viewport 60 points tall over
//! a 200 point line (`keyed_rows_fixture`).
//!
//! Oracle: where the layout put each row and its viewport, and a real drag
//! through the player: a region can move a way exactly when its content is
//! not at that end of its range.
use super::*;
use nuxie::keyed_rows_fixture::{
    BUTTON_LABEL, Cards, LINE_LABEL, RUN_TEXT, STILL, card, keyed_plans_fixture, page,
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
        local_id: page::CARDS,
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
#[derive(Clone, Copy)]
enum Plays {
    /// Its state machine, the default player.
    Machine,
    /// Nothing: a static player.
    Nothing,
    /// Its animation, [`STILL`].
    Animation,
}

/// A plans page playing with `rows` cards mounted and semantics on.
struct Page {
    file: *mut NuxFile,
    instance: *mut NuxArtboardInstance,
    player: *mut NuxPlayer,
    _mount: Box<Mount>,
}

impl Page {
    fn new(cards: Cards, rows: usize) -> Self {
        Self::played(cards, rows, Plays::Machine)
    }

    fn played(cards: Cards, rows: usize, plays: Plays) -> Self {
        let bytes = keyed_plans_fixture(cards);
        let mut file = ptr::null_mut();
        let mut instance = ptr::null_mut();
        let mut player = ptr::null_mut();
        let mut mount = Box::new(Mount {
            rows,
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
                Plays::Animation => nux_player_new_linear_animation_named(
                    instance,
                    NuxStringView {
                        data: STILL.as_ptr().cast(),
                        len: STILL.len(),
                    },
                    &mut player,
                ),
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
        let page = Self {
            file,
            instance,
            player,
            _mount: mount,
        };
        for _ in 0..10 {
            page.present(&[]);
        }
        page
    }

    /// One step with these pointer events, presented.
    fn present(&self, pointers: &[NuxPlayerPointerEvent]) {
        let step = NuxPlayerStep {
            pointers: pointers.as_ptr(),
            pointer_count: pointers.len(),
            elapsed_seconds: 1.0 / 60.0,
            ..NuxPlayerStep::default()
        };
        let mut result = ptr::null_mut();
        unsafe {
            assert_eq!(
                nux_player_step(self.player, &step, &mut result),
                NuxStatus::Ok
            );
            let mut scheduling = NuxPlayerSchedulingInfo {
                struct_size: std::mem::size_of::<NuxPlayerSchedulingInfo>() as u32,
                ..Default::default()
            };
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

    fn capture(&self) -> Result<Capture, NuxStatus> {
        let mut snapshot = ptr::null_mut();
        let status = unsafe { nux_player_semantic_snapshot(self.player, &mut snapshot) };
        if status != NuxStatus::Ok {
            assert!(snapshot.is_null(), "a failure publishes no capture");
            return Err(status);
        }
        Ok(Capture(snapshot))
    }

    /// Row `index`'s artboard.
    fn row(&self, index: i32) -> nuxie::RuntimeArtboardInstanceHandle {
        use nuxie::runtime::artboard_component_list::ArtboardComponentList;
        let native = unsafe { &*self.player }
            .artboard
            .instance
            .borrow()
            .native_handle();
        let list = native
            .with_artboard(|board| board.objects()[page::CARDS].clone())
            .expect("the list");
        list.with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(index))
            .flatten()
            .expect("a row")
    }

    /// Row `index`'s scroll offset, from its constraint.
    fn scroll_offset(&self, index: i32) -> f32 {
        use nuxie::runtime::constraints::scrolling::scroll_constraint::ScrollConstraint;
        self.row(index)
            .with_artboard(|board| board.objects()[card::SCROLL].clone())
            .expect("the constraint")
            .with_downcast::<ScrollConstraint, _>(|scroll| scroll.offset_y())
            .expect("a scroll constraint")
    }

    /// A held drag 30 points up from the middle of row `index`'s viewport.
    fn drag_up(&self, index: i32) {
        let [left, top, right, bottom] = self.scroller_box(index);
        let (x, y) = ((left + right) / 2.0, (top + bottom) / 2.0);
        let pointer = |kind, dy: f32, step: usize| NuxPlayerPointerEvent {
            kind,
            x,
            y: y + dy,
            pointer_id: 0,
            timestamp_seconds: step as f32 / 60.0,
        };
        self.present(&[pointer(NUX_PLAYER_POINTER_KIND_DOWN, 0.0, 0)]);
        for step in 1..=3 {
            self.present(&[pointer(
                NUX_PLAYER_POINTER_KIND_MOVE,
                -10.0 * step as f32,
                step,
            )]);
        }
    }

    /// Row `index`'s viewport, "Scroller", in page space, from the layout.
    fn scroller_box(&self, index: i32) -> [f32; 4] {
        use nuxie::runtime::layout_component::LayoutComponent;
        let row = self.row(index);
        let scroller = row
            .with_artboard(|board| board.objects()[card::SCROLLER].clone())
            .expect("the scroller")
            .with_downcast::<LayoutComponent, _>(|scroller| scroller.layout_bounds())
            .expect("a box");
        let (x, y) = row.with_artboard(|row| (row.layout_x(), row.layout_y()));
        [
            x + scroller.left(),
            y + scroller.top(),
            x + scroller.left() + scroller.width(),
            y + scroller.top() + scroller.height(),
        ]
    }
}

impl Drop for Page {
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
                assert_eq!(view.id as usize, index);
                view
            })
            .collect()
    }

    /// Each node's label and scroll region.
    fn node_regions(&self) -> Vec<(String, i32)> {
        (0..self.info().node_count)
            .map(|index| {
                let mut node = NuxSemanticNodeView {
                    struct_size: std::mem::size_of::<NuxSemanticNodeView>() as u32,
                    scroll_region: 77,
                    ..Default::default()
                };
                assert_eq!(
                    unsafe { nux_semantic_snapshot_node(self.0, index, &mut node) },
                    NuxStatus::Ok
                );
                let label = unsafe {
                    std::slice::from_raw_parts(node.label.data.cast::<u8>(), node.label.len)
                };
                (
                    String::from_utf8_lossy(label).into_owned(),
                    node.scroll_region,
                )
            })
            .collect()
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        assert_eq!(unsafe { nux_semantic_snapshot_free(self.0) }, NuxStatus::Ok);
    }
}

fn region_box(region: &NuxScrollRegionView) -> [f32; 4] {
    [region.min_x, region.min_y, region.max_x, region.max_y]
}

const TOP: u32 = 1 << 0;
const BOTTOM: u32 = 1 << 1;

/// Each row's viewport is a region in page space, in row order, at its top
/// so able to move toward its bottom only. The line it scrolls names its
/// row's region; the button and the words beside it name none. Dragged 30
/// points up, the first row's region can move both ways, and the second
/// row's is as it was.
#[test]
fn each_rows_scroller_is_a_region_and_follows_a_drag() {
    let page = Page::new(Cards::DescribedLine, 2);
    let capture = page.capture().expect("a capture");
    let regions = capture.regions();
    assert_eq!(regions.len(), 2);
    for (index, region) in regions.iter().enumerate() {
        assert_eq!(region_box(region), page.scroller_box(index as i32));
        assert_eq!(region.parent, -1);
        assert_eq!(region.can_scroll, BOTTOM);
    }
    let nodes = capture.node_regions();
    let named = |label: &str| {
        nodes
            .iter()
            .filter(|(name, _)| name == label)
            .map(|(_, region)| *region)
            .collect::<Vec<_>>()
    };
    assert_eq!(named(LINE_LABEL), [0, 1]);
    assert_eq!(named(BUTTON_LABEL), [-1, -1]);
    assert_eq!(named(RUN_TEXT), [-1, -1]);
    drop(capture);

    page.drag_up(0);
    let regions = page.capture().expect("a capture").regions();
    assert_eq!(regions[0].can_scroll, TOP | BOTTOM);
    assert_eq!(regions[1].can_scroll, BOTTOM);
    assert_eq!(region_box(&regions[0]), page.scroller_box(0));
}

/// Only a state-machine player sends its pointers on, so a page played by
/// nothing or by an animation has no region, though its rows and their
/// scrollers are there, and its nodes name none. A drag where a row's
/// region would be moves nothing, where the same drag through the page's
/// state machine scrolls the row.
#[test]
fn a_page_no_machine_plays_has_no_region() {
    let machine = Page::played(Cards::DescribedLine, 2, Plays::Machine);
    assert_eq!(machine.capture().expect("a capture").regions().len(), 2);
    assert_eq!(machine.scroll_offset(0), 0.0);
    machine.drag_up(0);
    assert!(machine.scroll_offset(0) < 0.0, "a machine's drag scrolls");
    drop(machine);

    for plays in [Plays::Nothing, Plays::Animation] {
        let page = Page::played(Cards::DescribedLine, 2, plays);
        let capture = page.capture().expect("a capture");
        assert_eq!(capture.info().scroll_region_count, 0);
        assert!(capture.regions().is_empty());
        let lines = capture
            .node_regions()
            .into_iter()
            .filter(|(name, _)| name == LINE_LABEL)
            .map(|(_, region)| region)
            .collect::<Vec<_>>();
        assert_eq!(lines, [-1, -1], "both rows' lines, in no region");
        assert!(
            capture
                .node_regions()
                .iter()
                .all(|(_, region)| *region == -1)
        );
        drop(capture);
        page.drag_up(0);
        assert_eq!(page.scroll_offset(0), 0.0);
    }
}

/// A scroller's clip past the geometry budget fails the capture as a
/// node's would, though no semantic node sits under it: the region is not
/// dropped as if clipped away. Put back to one rectangle, the capture lists
/// it again.
#[test]
fn a_scroller_clip_past_the_geometry_budget_fails_the_capture() {
    use nuxie::runtime::{layout_component::LayoutComponent, math::path_types::PathDirection};
    let page = Page::new(Cards::Keyed, 1);
    assert_eq!(page.capture().expect("a capture").regions().len(), 1);
    let rebuild = |rectangles: usize| {
        page.row(0)
            .with_artboard(|board| board.objects()[card::SCROLLER].clone())
            .expect("the scroller")
            .with_downcast_mut::<LayoutComponent, _>(|scroller| {
                let mut path = scroller.clip_world_path().expect("a scroller clips");
                let rect = path.raw_path().bounds();
                path.rewind();
                for _ in 0..rectangles {
                    path.mutable_raw_path()
                        .add_rect(rect, PathDirection::Clockwise);
                }
            })
            .expect("a layout box");
    };
    // 5,000 rectangles are 20,000 points, past the 16,384 a clip may have.
    rebuild(5000);
    assert_eq!(page.capture().err(), Some(NuxStatus::LimitExceeded));
    rebuild(1);
    assert_eq!(page.capture().expect("a capture").regions().len(), 1);
}

/// A page whose cards are not mounted has no region, and its nodes none.
#[test]
fn a_page_without_scrollers_has_no_region() {
    let page = Page::new(Cards::Keyed, 0);
    let capture = page.capture().expect("a capture");
    assert_eq!(capture.info().scroll_region_count, 0);
    assert!(
        capture
            .node_regions()
            .iter()
            .all(|(_, region)| *region == -1)
    );
    let mut view = NuxScrollRegionView {
        struct_size: std::mem::size_of::<NuxScrollRegionView>() as u32,
        ..Default::default()
    };
    assert_eq!(
        unsafe { nux_semantic_snapshot_scroll_region(capture.0, 0, &mut view) },
        NuxStatus::NotFound
    );
}

/// 256 regions are captured; one more fails the capture as a whole, for a
/// state-machine player. A static player lists no region, so the same 257
/// rows capture.
///
/// A region is listed only where a drag can reach it, and the page clips
/// rows below its 640 point height, so the page is first made tall enough
/// to show every row: 257 cards of 160 points under the heading.
#[test]
fn more_than_256_regions_fail_the_capture() {
    let tall = |rows, plays| {
        let page = Page::played(Cards::Keyed, rows, plays);
        assert_eq!(
            unsafe { nux_player_layout_size_set(page.player, 320.0, 42_000.0) },
            NuxStatus::Ok
        );
        page.present(&[]);
        page
    };
    // The page as built shows only the rows that fit in it.
    let short = Page::new(Cards::Keyed, 256);
    let shown = short
        .capture()
        .expect("a capture")
        .info()
        .scroll_region_count;
    assert!(shown > 0 && shown < 256, "{shown} regions on a short page");
    drop(short);

    let page = tall(256, Plays::Machine);
    assert_eq!(
        page.capture()
            .expect("a capture")
            .info()
            .scroll_region_count,
        256
    );
    drop(page);
    let page = tall(257, Plays::Machine);
    assert_eq!(page.capture().err(), Some(NuxStatus::LimitExceeded));
    drop(page);
    let page = tall(257, Plays::Nothing);
    let capture = page.capture().expect("a static page's capture");
    assert_eq!(capture.info().scroll_region_count, 0);
    assert!(capture.info().node_count > 257, "every row's nodes");
}

/// The views' layouts, and callers of each older or shorter size: an info
/// or node view of the size before the scroll fields reads the rest and
/// leaves the caller's bytes past it alone; a region view shorter than its
/// fields, or an info shorter than its first version, is refused.
#[test]
fn view_sizes_and_older_callers() {
    use std::mem::{offset_of, size_of};
    assert_eq!(size_of::<NuxScrollRegionView>(), 32);
    assert_eq!(
        [
            offset_of!(NuxScrollRegionView, id),
            offset_of!(NuxScrollRegionView, min_x),
            offset_of!(NuxScrollRegionView, min_y),
            offset_of!(NuxScrollRegionView, max_x),
            offset_of!(NuxScrollRegionView, max_y),
            offset_of!(NuxScrollRegionView, can_scroll),
            offset_of!(NuxScrollRegionView, parent),
        ],
        [4, 8, 12, 16, 20, 24, 28]
    );
    assert_eq!(NUX_SCROLL_REGION_VIEW_MIN_SIZE, 32);
    assert_eq!(size_of::<NuxSemanticSnapshotInfo>(), 40);
    assert_eq!(offset_of!(NuxSemanticSnapshotInfo, scroll_region_count), 32);
    assert_eq!(NUX_SEMANTIC_SNAPSHOT_INFO_MIN_SIZE, 32);
    assert_eq!(size_of::<NuxSemanticNodeView>(), 104);
    assert_eq!(offset_of!(NuxSemanticNodeView, scroll_region), 100);
    assert_eq!(NUX_SEMANTIC_NODE_VIEW_MIN_SIZE, 100);

    let page = Page::new(Cards::DescribedLine, 1);
    let capture = page.capture().expect("a capture");
    let full = capture.info();
    assert_eq!(full.scroll_region_count, 1);

    let mut old_info = NuxSemanticSnapshotInfo {
        struct_size: NUX_SEMANTIC_SNAPSHOT_INFO_MIN_SIZE as u32,
        scroll_region_count: 999,
        ..Default::default()
    };
    assert_eq!(
        unsafe { nux_semantic_snapshot_info(capture.0, &mut old_info) },
        NuxStatus::Ok
    );
    assert_eq!(old_info.node_count, full.node_count);
    assert_eq!(old_info.scroll_region_count, 999, "past the caller's size");
    old_info.struct_size = NUX_SEMANTIC_SNAPSHOT_INFO_MIN_SIZE as u32 - 1;
    assert_eq!(
        unsafe { nux_semantic_snapshot_info(capture.0, &mut old_info) },
        NuxStatus::InvalidStructSize
    );

    let line = (0..full.node_count)
        .find(|&index| capture.node_regions()[index].0 == LINE_LABEL)
        .expect("the line");
    let mut old_node = NuxSemanticNodeView {
        struct_size: NUX_SEMANTIC_NODE_VIEW_MIN_SIZE as u32,
        scroll_region: 77,
        ..Default::default()
    };
    assert_eq!(
        unsafe { nux_semantic_snapshot_node(capture.0, line, &mut old_node) },
        NuxStatus::Ok
    );
    assert_eq!(old_node.scroll_region, 77, "past the caller's size");
    assert_eq!(capture.node_regions()[line].1, 0);

    let mut short = NuxScrollRegionView {
        struct_size: NUX_SCROLL_REGION_VIEW_MIN_SIZE as u32 - 1,
        ..Default::default()
    };
    assert_eq!(
        unsafe { nux_semantic_snapshot_scroll_region(capture.0, 0, &mut short) },
        NuxStatus::InvalidStructSize
    );
    assert_eq!(
        unsafe { nux_semantic_snapshot_scroll_region(capture.0, 0, ptr::null_mut()) },
        NuxStatus::NullArgument
    );
}
