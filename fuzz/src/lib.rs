//! Shared drivers for the negative-input fuzz targets.
//!
//! These functions mirror the exact call sequence that
//! `tools/rust-golden-runner` performs against the translated native owners in
//! `nuxie-runtime` (the non-`scripting` path), but they deliberately swallow
//! every recoverable `Result`/`Option` error. The *only* thing a fuzz target
//! cares about is a panic: the runtime ships inside customer apps under
//! `panic = "abort"`, so a reachable panic is a host-app kill. Any input that
//! the importer *accepts* must be drivable through instantiate -> advance ->
//! draw without panicking.

use nuxie_binary::read_runtime_file;
use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::source::math::vec2d::Vec2D;
use nuxie_runtime::{
    File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeStateMachineInstanceHandle,
};

/// A pointer event replayed against the default state machine.
#[derive(Clone, Copy)]
pub struct PointerEvent {
    pub kind: PointerKind,
    pub x: f32,
    pub y: f32,
    pub pointer_id: i32,
}

#[derive(Clone, Copy)]
pub enum PointerKind {
    Down,
    Move,
    Up,
    Exit,
}

/// Parser-hardening target: just import the bytes. `read_runtime_file` must
/// never panic on arbitrary input; it either returns a `RuntimeFile` or an
/// `Err`.
#[inline]
pub fn run_import(data: &[u8]) {
    let _ = read_runtime_file(data);
}

/// Runtime target: if native import ACCEPTS, instantiate the default artboard,
/// advance the default scene twice (t=0 and a small dt), and draw through the
/// null renderer.
#[inline]
pub fn run_runtime(data: &[u8]) {
    let _ = drive(data, &[]);
}

/// Pointer target: same as `run_runtime`, but also replays a handful of pointer
/// events (derived from the input tail) against the default state machine
/// between the two advances.
#[inline]
pub fn run_pointer(data: &[u8]) {
    let events = derive_pointer_events(data);
    let _ = drive(data, &events);
}

fn drive(data: &[u8], pointer_events: &[PointerEvent]) -> Option<()> {
    // 1. Import. If the importer rejects, there is nothing for the runtime to
    //    assume about, so we are done.
    let mut factory = PersistentFactory::new(NullFactory::new());
    let retained_factory = RuntimeFactoryHandle::from_factory(&mut factory)?;
    let file = File::import(data, retained_factory, None, None, None)?;

    // 2. Select and instantiate the default (first) artboard, mirroring the
    //    native golden runner.
    let artboard = file.with_file(File::artboard_default)?;

    // 3. Bind the artboard's default view-model occurrence before constructing
    //    its default state-machine occurrence, matching the native loader.
    let view_model = file.with_file_mut(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
            .or_else(|| file.create_view_model_instance_for_artboard(artboard.core_handle()))
    });
    artboard.bind_view_model_instance(view_model.clone());
    let state_machine = artboard.default_state_machine();
    if let (Some(state_machine), Some(view_model)) = (&state_machine, view_model) {
        state_machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model));
    }

    let mut renderer = factory.borrow().make_renderer();

    // 4. Advance twice (t=0.0 then a small dt), replaying pointer events after
    //    the first advance, drawing after each.
    let mut current_seconds = 0.0f32;
    for (step, &target_seconds) in [0.0f32, 0.016f32].iter().enumerate() {
        advance_scene_to(
            &artboard,
            state_machine.as_ref(),
            target_seconds,
            &mut current_seconds,
        );

        if step == 0 {
            for event in pointer_events {
                apply_pointer_event(event, state_machine.as_ref());
            }
        }

        artboard.draw(&mut renderer);
    }

    Some(())
}

fn advance_scene_to(
    artboard: &RuntimeArtboardInstanceHandle,
    state_machine: Option<&RuntimeStateMachineInstanceHandle>,
    target_seconds: f32,
    current_seconds: &mut f32,
) {
    let elapsed_seconds = (target_seconds - *current_seconds).max(0.0);
    if let Some(state_machine) = state_machine {
        state_machine.advance_and_apply(elapsed_seconds);
    } else {
        artboard.advance_default(elapsed_seconds);
    }
    *current_seconds = target_seconds;
}

fn apply_pointer_event(
    event: &PointerEvent,
    state_machine: Option<&RuntimeStateMachineInstanceHandle>,
) {
    let Some(state_machine) = state_machine else {
        return;
    };
    let position = Vec2D::new(event.x, event.y);
    state_machine.with_instance_mut(|state_machine| match event.kind {
        PointerKind::Down => state_machine.pointer_down(position, event.pointer_id),
        PointerKind::Move => state_machine.pointer_move(position, 0.016, event.pointer_id),
        PointerKind::Up => state_machine.pointer_up(position, event.pointer_id),
        PointerKind::Exit => state_machine.pointer_exit(position, event.pointer_id),
    });
}

/// Derive up to four pointer events from the tail of the input so that
/// libFuzzer's mutations steer both the file bytes *and* the pointer stream.
fn derive_pointer_events(data: &[u8]) -> Vec<PointerEvent> {
    // Use the last few bytes so mutating the .riv body and the pointer stream
    // stay largely independent.
    let tail = if data.len() > 16 {
        &data[data.len() - 16..]
    } else {
        data
    };
    let mut events = Vec::new();
    for chunk in tail.chunks(4) {
        if chunk.len() < 4 {
            break;
        }
        let kind = match chunk[0] & 0b11 {
            0 => PointerKind::Down,
            1 => PointerKind::Move,
            2 => PointerKind::Up,
            _ => PointerKind::Exit,
        };
        // Map bytes to a mix of in-bounds and out-of-bounds coordinates so we
        // exercise hit-testing edges without going fully degenerate.
        let x = f32::from(chunk[1]) * 4.0 - 128.0;
        let y = f32::from(chunk[2]) * 4.0 - 128.0;
        let pointer_id = i32::from(chunk[3] & 0b11);
        events.push(PointerEvent {
            kind,
            x,
            y,
            pointer_id,
        });
    }
    events
}
