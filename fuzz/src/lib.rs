//! Shared drivers for the negative-input fuzz targets.
//!
//! These functions mirror the exact call sequence that
//! `tools/rust-golden-runner` performs against `rive-runtime`/`rive-graph`
//! (the non-`scripting` path), but they deliberately swallow every recoverable
//! `Result`/`Option` error. The *only* thing a fuzz target cares about is a
//! panic: the runtime ships inside customer apps under `panic = "abort"`, so a
//! reachable panic is a host-app kill. Any input that the importer *accepts*
//! must be drivable through instantiate -> advance -> draw without panicking.

use rive_binary::read_runtime_file;
use rive_graph::{ArtboardGraph, GraphFile};
use rive_render_api::NullFactory;
use rive_runtime::{
    preallocate_render_paint_cache_for_artboard_tree, ArtboardInstance,
    RuntimeOwnedViewModelInstance, RuntimeRenderPathCache, StateMachineInstance,
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

/// Runtime target: if import ACCEPTS, build the graph, instantiate the default
/// artboard, advance the default scene twice (t=0 and a small dt), and draw
/// through the null renderer.
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
    let runtime = read_runtime_file(data).ok()?;
    let graph = GraphFile::from_runtime_file(&runtime).ok()?;

    // 2. Select the default (first) artboard, mirroring the golden runner.
    let artboard_index = 0usize;
    let artboard = graph.artboards.first()?;

    // 3. Instantiate.
    let mut instance =
        ArtboardInstance::from_graph_with_artboards(&runtime, artboard, &graph.artboards).ok()?;
    let mut factory = NullFactory::new();
    let mut paint_cache = preallocate_render_paint_cache_for_artboard_tree(
        &runtime,
        artboard,
        &graph.artboards,
        &mut factory,
    );
    let mut path_cache = RuntimeRenderPathCache::default();
    let mut renderer = factory.make_renderer();

    // 4. Default scene selection + state machine instantiation.
    let scene_sm_index = default_state_machine_index(&runtime, artboard_index, artboard);
    let mut state_machine: Option<StateMachineInstance> =
        scene_sm_index.and_then(|index| instance.state_machine_instance(index));

    // 5. View-model + data-context binding (the "importer accepts, runtime
    //    assumes" surface is dense here).
    let mut owned_context = owned_view_model_context(&runtime, artboard_index);
    instance.bind_default_view_model_artboard_list_context(&runtime);
    if let Some(state_machine) = state_machine.as_mut() {
        if let Some(context) = owned_context.as_ref() {
            state_machine.bind_owned_view_model_context(context);
        }
        state_machine.advance_data_context();
    }
    if let Some(context) = owned_context.as_ref() {
        instance.bind_owned_view_model_nested_artboard_contexts(&runtime, context);
    }

    // 6. Advance twice (t=0.0 then a small dt), replaying pointer events after
    //    the first advance, drawing after each.
    let mut current_seconds = 0.0f32;
    for (step, &target_seconds) in [0.0f32, 0.016f32].iter().enumerate() {
        advance_scene_to(
            &mut instance,
            &runtime,
            state_machine.as_mut(),
            owned_context.as_ref(),
            target_seconds,
            &mut current_seconds,
        );

        if step == 0 {
            for event in pointer_events {
                apply_pointer_event(
                    event,
                    &instance,
                    state_machine.as_mut(),
                    owned_context.as_mut(),
                );
            }
        }

        // prepare + draw through the null renderer.
        if instance
            .prepare_static_artboard_tree_paints(
                &runtime,
                artboard,
                &graph.artboards,
                &mut factory,
                &mut paint_cache,
                &mut path_cache,
            )
            .is_err()
        {
            return Some(());
        }
        let _ = instance.draw_prepared_static_artboard_with_render_cache(
            &runtime,
            artboard,
            &graph.artboards,
            &mut factory,
            &mut renderer,
            &mut paint_cache,
            &mut path_cache,
        );
    }

    Some(())
}

fn advance_scene_to(
    instance: &mut ArtboardInstance,
    runtime: &rive_binary::RuntimeFile,
    state_machine: Option<&mut StateMachineInstance>,
    owned_context: Option<&RuntimeOwnedViewModelInstance>,
    target_seconds: f32,
    current_seconds: &mut f32,
) {
    let elapsed_seconds = (target_seconds - *current_seconds).max(0.0);
    if let Some(state_machine) = state_machine {
        instance.advance_state_machine_instance(state_machine, elapsed_seconds);
        if instance.advance_nested_artboards_with_state_machine(elapsed_seconds, state_machine) {
            instance.advance_state_machine_instance(state_machine, 0.0);
        }
    } else {
        instance.advance_nested_artboards(elapsed_seconds);
    }
    if let Some(context) = owned_context {
        instance.bind_owned_view_model_nested_artboard_contexts(runtime, context);
    }
    instance.advance_artboard_data_binds_with_elapsed(elapsed_seconds);
    let _ = instance.update_script_instances();
    instance.update_pass();
    *current_seconds = target_seconds;
}

fn apply_pointer_event(
    event: &PointerEvent,
    instance: &ArtboardInstance,
    state_machine: Option<&mut StateMachineInstance>,
    owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
) {
    let Some(state_machine) = state_machine else {
        return;
    };
    match (event.kind, owned_context) {
        (PointerKind::Down, Some(context)) => {
            state_machine.pointer_down_with_owned_view_model_context(
                instance,
                event.x,
                event.y,
                event.pointer_id,
                context,
            );
        }
        (PointerKind::Down, None) => {
            state_machine.pointer_down(instance, event.x, event.y, event.pointer_id);
        }
        (PointerKind::Move, Some(context)) => {
            state_machine.pointer_move_with_owned_view_model_context(
                instance,
                event.x,
                event.y,
                0.016,
                event.pointer_id,
                context,
            );
        }
        (PointerKind::Move, None) => {
            state_machine.pointer_move(instance, event.x, event.y, 0.016, event.pointer_id);
        }
        (PointerKind::Up, Some(context)) => {
            state_machine.pointer_up_with_owned_view_model_context(
                instance,
                event.x,
                event.y,
                event.pointer_id,
                context,
            );
        }
        (PointerKind::Up, None) => {
            state_machine.pointer_up(instance, event.x, event.y, event.pointer_id);
        }
        (PointerKind::Exit, Some(context)) => {
            state_machine.pointer_exit_with_owned_view_model_context(
                instance,
                event.x,
                event.y,
                event.pointer_id,
                context,
            );
        }
        (PointerKind::Exit, None) => {
            state_machine.pointer_exit(instance, event.x, event.y, event.pointer_id);
        }
    }
}

/// Mirrors `select_scene`'s default branch in the golden runner.
fn default_state_machine_index(
    runtime: &rive_binary::RuntimeFile,
    artboard_index: usize,
    artboard: &ArtboardGraph,
) -> Option<usize> {
    runtime
        .artboard(artboard_index)
        .and_then(|object| {
            object
                .property("defaultStateMachineId")
                .and_then(|_| object.uint_property("defaultStateMachineId"))
        })
        .and_then(|index| usize::try_from(index).ok())
        .filter(|index| artboard.state_machines.get(*index).is_some())
}

/// Mirrors `selected_artboard_owned_view_model_context` (non-scripting).
fn owned_view_model_context(
    runtime: &rive_binary::RuntimeFile,
    artboard_index: usize,
) -> Option<RuntimeOwnedViewModelInstance> {
    let view_model_index = runtime
        .artboard(artboard_index)
        .and_then(|object| object.uint_property("viewModelId"))
        .and_then(|index| usize::try_from(index).ok())?;
    RuntimeOwnedViewModelInstance::new(runtime, view_model_index)
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
