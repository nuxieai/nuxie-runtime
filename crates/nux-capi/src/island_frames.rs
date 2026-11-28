//! Where a host places a platform view over a box, read from a step.

use super::player_driver::read_scene_object;
use super::*;
use nuxie::page_player::{IslandClip, IslandReadError, island_frame};
use nuxie::runtime::semantic::semantic_provider::SemanticGeometryError;

/// The most boxes one read takes.
const NUX_PLAYER_ISLAND_FRAMES_MAX: usize = 4096;

/// Where a host places a platform view over one box, copied out of a
/// step. Matrices use [a, b, c, d, tx, ty]: x' = a*x + c*y + tx.
/// `world_transform` maps the box's layout space into the root
/// artboard's; `frame_*` and `content_*` are in the box's layout space.
/// `clip` is 0 when nothing clips the box, 1 when `clip_*` is the
/// rectangle that does, in root space, and 2 when a clip is not a
/// rectangle. `placeable` is 0 when the transform rotates, skews or
/// mirrors, or `clip` is 2. A box under a copy flying in place reads
/// as the flight draws it: under its pose, at its opacity, and cut by
/// the clips it draws under. When `visible` is 0, the clip rectangle
/// carries no meaning, and a host must not read it.
/// Canonical 0/1 flags.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NuxIslandFrame {
    pub struct_size: u32,
    pub render_revision: u64,
    pub world_transform: [f32; 6],
    pub frame_min_x: f32,
    pub frame_min_y: f32,
    pub frame_max_x: f32,
    pub frame_max_y: f32,
    pub content_min_x: f32,
    pub content_min_y: f32,
    pub content_max_x: f32,
    pub content_max_y: f32,
    pub visible: u32,
    pub opacity: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub clip: u32,
    pub clip_min_x: f32,
    pub clip_min_y: f32,
    pub clip_max_x: f32,
    pub clip_max_y: f32,
    pub placeable: u32,
}

impl NuxIslandFrame {
    /// Every number it carries.
    fn numbers(&self) -> impl Iterator<Item = f32> + '_ {
        self.world_transform.iter().copied().chain([
            self.frame_min_x,
            self.frame_min_y,
            self.frame_max_x,
            self.frame_max_y,
            self.content_min_x,
            self.content_min_y,
            self.content_max_x,
            self.content_max_y,
            self.opacity,
            self.scale_x,
            self.scale_y,
            self.clip_min_x,
            self.clip_min_y,
            self.clip_max_x,
            self.clip_max_y,
        ])
    }
}

/// Read `count` boxes' island frames from the state named by `step`, in
/// order, into `out`, as `nux_player_text_run_geometry` reads a text
/// run: the step must be this player's and name its current render
/// revision, or HANDLE_MISMATCH. Each box is a scene object, as a driver
/// writes one. Both arrays are fixed-stride: each `boxes[i].struct_size`
/// and `out[i].struct_size` must equal the size of its struct, or
/// INVALID_STRUCT_SIZE. NOT_FOUND when a box names nothing,
/// INVALID_ARGUMENT when it names something that is not a box. `out` is
/// written only when every box reads. Changes nothing in the scene.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_island_frames(
    player: *const NuxPlayer,
    step: *const NuxPlayerStepResult,
    boxes: *const NuxSceneObject,
    count: usize,
    out: *mut NuxIslandFrame,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _player = enter_status_handle!(player, HandleKind::Player);
        let _step = enter_status_handle!(step, HandleKind::PlayerStepResult);
        let player = unsafe { &*player };
        let step = unsafe { &*step };
        let _occurrence = match enter_occurrence(&player.artboard) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        if let Err(status) = player
            .artboard
            .refresh_renderer_domain_invalidation()
            .and_then(|()| player.artboard.refresh_bound_view_model_invalidation())
        {
            player.artboard.poisoned.set(true);
            return status;
        }
        if step.status != NuxStatus::Ok
            || !step
                .occurrence
                .upgrade()
                .is_some_and(|owner| Rc::ptr_eq(&owner, &player.artboard))
            || step.scheduling.render_revision == 0
            || step.scheduling.render_revision != player.artboard.render_revision.get()
        {
            return NuxStatus::HandleMismatch;
        }
        if count > NUX_PLAYER_ISLAND_FRAMES_MAX {
            return NuxStatus::LimitExceeded;
        }
        if count == 0 {
            return NuxStatus::Ok;
        }
        if boxes.is_null() || out.is_null() {
            return NuxStatus::NullArgument;
        }
        // Each array steps by its struct's size, so every element must be
        // exactly that size: a shorter or longer one would move the next.
        let exact = |size: u32, of: usize| usize::try_from(size).ok() == Some(of);
        for index in 0..count {
            let box_size = unsafe { boxes.add(index).cast::<u32>().read() };
            let out_size = unsafe { out.add(index).cast::<u32>().read() };
            if !exact(box_size, std::mem::size_of::<NuxSceneObject>())
                || !exact(out_size, std::mem::size_of::<NuxIslandFrame>())
            {
                return NuxStatus::InvalidStructSize;
            }
        }
        let objects = match (0..count)
            .map(|index| unsafe { read_scene_object(boxes.add(index)) })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(objects) => objects,
            Err(status) => return status,
        };
        let artboard = player.artboard.instance.borrow();
        let mut frames = Vec::with_capacity(count);
        for object in &objects {
            let island = match island_frame(&artboard, object) {
                Ok(island) => island,
                Err(IslandReadError::NotFound) => return NuxStatus::NotFound,
                Err(IslandReadError::NotABox) => return NuxStatus::InvalidArgument,
                Err(IslandReadError::Geometry(SemanticGeometryError::LimitExceeded)) => {
                    return NuxStatus::LimitExceeded;
                }
                Err(IslandReadError::Geometry(SemanticGeometryError::InvalidPath)) => {
                    return NuxStatus::RuntimeError;
                }
            };
            let (clip, [clip_min_x, clip_min_y, clip_max_x, clip_max_y]) = match island.clip {
                IslandClip::None => (0, [0.0; 4]),
                IslandClip::Rect(rect) => (1, [rect.min_x, rect.min_y, rect.max_x, rect.max_y]),
                IslandClip::NotRect => (2, [0.0; 4]),
            };
            let frame = NuxIslandFrame {
                struct_size: std::mem::size_of::<NuxIslandFrame>() as u32,
                render_revision: step.scheduling.render_revision,
                world_transform: *island.world.values(),
                frame_min_x: island.frame.min_x,
                frame_min_y: island.frame.min_y,
                frame_max_x: island.frame.max_x,
                frame_max_y: island.frame.max_y,
                content_min_x: island.content.min_x,
                content_min_y: island.content.min_y,
                content_max_x: island.content.max_x,
                content_max_y: island.content.max_y,
                visible: u32::from(island.visible),
                opacity: island.opacity,
                scale_x: island.scale.0,
                scale_y: island.scale.1,
                clip,
                clip_min_x,
                clip_min_y,
                clip_max_x,
                clip_max_y,
                placeable: u32::from(island.placeable),
            };
            if !frame.numbers().all(f32::is_finite) {
                return NuxStatus::RuntimeError;
            }
            frames.push(frame);
        }
        for (index, frame) in frames.iter().enumerate() {
            unsafe { out.add(index).write(*frame) };
        }
        NuxStatus::Ok
    })
}

#[cfg(test)]
#[path = "island_frames_tests.rs"]
mod island_frames_tests;
