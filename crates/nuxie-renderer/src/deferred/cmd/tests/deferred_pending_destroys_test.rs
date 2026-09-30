//! Complete deferred_pending_destroys_test.cpp translation at 912312dc.
use super::super::{deferred_replayer::*, deferred_session::DeferredSession};
use super::*;
use nuxie_ore_metal::{
    context::ContextApi,
    ore_cmd::{
        ore_command_buffer::OreCommandReader,
        ore_commands::{CommandType, DestroyResourcePOD},
    },
    types::{BufferDesc, BufferUsage},
};

fn replay_frame(
    session: &mut DeferredSession,
    replayer: &mut DeferredReplayer,
    sink: &mut TestSink,
) {
    let frame = take_frame(session);
    replayer.replay_frame(&frame, sink);
}

#[test]
fn an_idle_session_strands_destroys_until_they_are_flushed() {
    let mut session = DeferredSession::with_caps(Default::default());
    let mut replayer = DeferredReplayer::default();
    let mut sink = TestSink::default();
    let paint = session.make_render_paint();
    let path = session.make_empty_render_path();
    session
        .screen_renderer(0)
        .borrow_mut()
        .draw_path(path.as_ref(), paint.as_ref());
    replay_frame(&mut session, &mut replayer, &mut sink);
    assert_eq!(replayer.gpu_census().paths, 1);
    assert_eq!(replayer.gpu_census().paints, 1);

    // The handles die without a subsequent frame.
    drop(paint);
    drop(path);
    assert_eq!(replayer.gpu_census().paths, 1);
    assert_eq!(replayer.gpu_census().paints, 1);
    let pending = session.take_pending_destroys();
    assert!(!pending.empty());
    replayer.replay_destroys(&pending.commands, &pending.ore_commands);
    assert_eq!(replayer.gpu_census().paths, 0);
    assert_eq!(replayer.gpu_census().paints, 0);
}

#[test]
fn nothing_queued_takes_nothing() {
    let mut session = DeferredSession::with_caps(Default::default());
    assert!(session.take_pending_destroys().empty());
}

#[test]
fn an_ore_handles_destroy_is_taken_and_replays_outside_a_frame() {
    let mut session = DeferredSession::with_caps(Default::default());
    let mut replayer = DeferredReplayer::default();
    let buffer = session.ore_context.borrow_mut().makeBuffer(&BufferDesc {
        usage: BufferUsage::uniform,
        size: 64,
        data: None,
        immutable: false,
        label: None,
    });
    assert!(buffer.is_some());
    // Creation remains in the stream; only the destroy tail may be taken.
    assert!(session.take_pending_destroys().empty());
    drop(buffer);
    let pending = session.take_pending_destroys();
    assert!(pending.commands.is_empty());
    assert!(!pending.ore_commands.is_empty());

    let mut reader = OreCommandReader::new(&pending.ore_commands, &[]);
    assert_eq!(reader.next(), Some(CommandType::destroyResource));
    let _: DestroyResourcePOD = reader.read();
    assert_eq!(reader.next::<CommandType>(), None);

    // There is no GPU and the buffer was never resident. Both passes still
    // consume the destroy stream cleanly.
    replayer.replay_destroys(&pending.commands, &pending.ore_commands);
    replayer.replay_destroys(&pending.commands, &pending.ore_commands);
    assert_eq!(replayer.gpu_census().live_objects(), 0);
}

#[test]
fn flushed_destroys_replay_again_with_the_next_frame_harmlessly() {
    let mut session = DeferredSession::with_caps(Default::default());
    let mut replayer = DeferredReplayer::default();
    let mut sink = TestSink::default();
    let paint = session.make_render_paint();
    let path = session.make_empty_render_path();
    session
        .screen_renderer(0)
        .borrow_mut()
        .draw_path(path.as_ref(), paint.as_ref());
    replay_frame(&mut session, &mut replayer, &mut sink);
    drop(paint);
    drop(path);
    let pending = session.take_pending_destroys();
    replayer.replay_destroys(&pending.commands, &pending.ore_commands);
    assert_eq!(replayer.gpu_census().live_objects(), 0);

    // Old-generation destroys remain in the stream. Reused IDs must survive
    // replaying those destroys in the next frame.
    let paint2 = session.make_render_paint();
    let path2 = session.make_empty_render_path();
    session
        .screen_renderer(0)
        .borrow_mut()
        .draw_path(path2.as_ref(), paint2.as_ref());
    replay_frame(&mut session, &mut replayer, &mut sink);
    assert_eq!(replayer.gpu_census().paths, 1);
    assert_eq!(replayer.gpu_census().paints, 1);
}
