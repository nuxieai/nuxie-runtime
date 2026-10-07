use nux_capi::*;
use std::ptr;

#[path = "support/layout.rs"]
mod layout;

struct LayoutHandles {
    file: *mut NuxFile,
    artboard: *mut NuxArtboardInstance,
    player: *mut NuxPlayer,
}

impl LayoutHandles {
    fn new() -> Self {
        let bytes = layout::layout_artboard();
        let mut file = ptr::null_mut();
        let mut artboard = ptr::null_mut();
        let mut player = ptr::null_mut();
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
                nux_artboard_instance_new(file, 0, &mut artboard),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_new_static(artboard, &mut player), NuxStatus::Ok);
            assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
        }
        Self {
            file,
            artboard,
            player,
        }
    }

    fn set(&self, width: f32, height: f32) -> NuxStatus {
        unsafe { nux_player_layout_size_set(self.player, width, height) }
    }

    fn step(&self) -> NuxPlayerSchedulingInfo {
        let mut result = ptr::null_mut();
        let mut scheduling = NuxPlayerSchedulingInfo::default();
        unsafe {
            assert_eq!(
                nux_player_step(self.player, &NuxPlayerStep::default(), &mut result),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_step_result_scheduling(result, &mut scheduling),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
            assert_eq!(
                nux_player_acknowledge_presented(self.player, scheduling.render_revision),
                NuxStatus::Ok
            );
        }
        scheduling
    }

    fn size(&self) -> (f32, f32) {
        let mut size = NuxPlayerLayoutSize::default();
        assert_eq!(
            unsafe { nux_player_layout_size(self.player, &mut size) },
            NuxStatus::Ok
        );
        (size.width, size.height)
    }

    fn child_width(&self) -> f32 {
        let mut snapshot = ptr::null_mut();
        let mut info = NuxSemanticSnapshotInfo {
            struct_size: std::mem::size_of::<NuxSemanticSnapshotInfo>() as u32,
            ..Default::default()
        };
        let mut width = None;
        unsafe {
            assert_eq!(
                nux_player_semantic_snapshot(self.player, &mut snapshot),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_semantic_snapshot_info(snapshot, &mut info),
                NuxStatus::Ok
            );
            for index in 0..info.node_count {
                let mut node = NuxSemanticNodeView {
                    struct_size: std::mem::size_of::<NuxSemanticNodeView>() as u32,
                    ..Default::default()
                };
                assert_eq!(
                    nux_semantic_snapshot_node(snapshot, index, &mut node),
                    NuxStatus::Ok
                );
                let label = if node.label.len == 0 {
                    &[]
                } else {
                    std::slice::from_raw_parts(node.label.data.cast::<u8>(), node.label.len)
                };
                if label == b"Full width child" {
                    width = Some(node.max_x - node.min_x);
                }
            }
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
        }
        width.expect("fixture's semantic child")
    }
}

impl Drop for LayoutHandles {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(nux_player_free(self.player), NuxStatus::Ok);
            assert_eq!(nux_artboard_instance_free(self.artboard), NuxStatus::Ok);
            assert_eq!(nux_file_free(self.file), NuxStatus::Ok);
        }
    }
}

#[test]
fn layout_size_reflows_percentage_width_through_c() {
    let scene = LayoutHandles::new();
    for width in [300.0, 500.0] {
        assert_eq!(scene.set(width, 600.0), NuxStatus::Ok);
        scene.step();
        assert_eq!(scene.size(), (width, 600.0));
        assert_eq!(scene.child_width(), width);
    }
}

#[test]
fn layout_size_refuses_invalid_inputs_without_changing_size_or_revision() {
    let scene = LayoutHandles::new();
    assert_eq!(scene.set(300.0, 600.0), NuxStatus::Ok);
    let revision = scene.step().render_revision;
    for (width, height) in [
        (0.0, 600.0),
        (-1.0, 600.0),
        (f32::NAN, 600.0),
        (f32::INFINITY, 600.0),
        (300.0, f32::NAN),
        (300.0, f32::INFINITY),
        (300.0, 0.0),
        (300.0, -1.0),
    ] {
        assert_eq!(scene.set(width, height), NuxStatus::InvalidArgument);
        assert_eq!(scene.size(), (300.0, 600.0));
        assert_eq!(scene.step().render_revision, revision);
    }
    assert_eq!(scene.set(300.0, 600.0), NuxStatus::Ok);
    assert_eq!(scene.step().render_revision, revision);
}

#[test]
fn layout_size_checks_nulls_and_output_prefix() {
    let scene = LayoutHandles::new();
    let mut size = NuxPlayerLayoutSize::default();
    unsafe {
        assert_eq!(
            nux_player_layout_size_set(ptr::null_mut(), 300.0, 600.0),
            NuxStatus::NullArgument
        );
        assert_eq!(
            nux_player_layout_size(ptr::null(), &mut size),
            NuxStatus::NullArgument
        );
        assert_eq!(
            nux_player_layout_size(scene.player, ptr::null_mut()),
            NuxStatus::NullArgument
        );
        size.struct_size = 8;
        assert_eq!(
            nux_player_layout_size(scene.player, &mut size),
            NuxStatus::InvalidStructSize
        );
        size.struct_size = 12;
        assert_eq!(
            nux_player_layout_size(scene.player, &mut size),
            NuxStatus::Ok
        );
    }
}

#[test]
fn layout_size_accepts_large_finite_dimensions() {
    let scene = LayoutHandles::new();
    assert_eq!(scene.set(1.0e20, 1.0e20), NuxStatus::Ok);
    scene.step();
    assert_eq!(scene.size(), (1.0e20, 1.0e20));
    assert_eq!(scene.set(300.0, 600.0), NuxStatus::Ok);
    scene.step();
    assert_eq!(scene.size(), (300.0, 600.0));
}
