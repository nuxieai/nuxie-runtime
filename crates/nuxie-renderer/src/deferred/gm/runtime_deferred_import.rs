//! Exercise goldens_shared.hpp's RIVLoader deferred-import route on the six
//! ordinary runtime fixtures introduced with e949498e. The API-scene GMs do
//! not cover file-owned paints/paths surviving snapshot/reset across frames.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{
    deferred_replayer::{take_frame, DeferredFrameSink, DeferredReplayer},
    deferred_session::DeferredSession,
};
use nuxie_runtime::{source::static_scene::StaticScene, File, RuntimeFactoryHandle};

fn frames(bytes: &[u8], deferred: bool) -> Vec<Vec<u8>> {
    let mut host = GmHost::with_screen(0xff202028, false);
    // Session precedes file/resources so their destruction still records into
    // the live stream (the source RIVLoader has the same lifetime ordering).
    let mut session = DeferredSession::new(Some(host.ore.clone()));
    session.bind_render_context(host.factory.persistent_context());
    let mut import = PersistentFactory::new(session.clone());
    let factory = if deferred {
        RuntimeFactoryHandle::from_factory(&mut import).unwrap()
    } else {
        RuntimeFactoryHandle::from_factory(&mut host.factory).unwrap()
    };
    let file = File::import(bytes, factory, None, None, None).expect("parity fixture import");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard instance");
    let model =
        file.with_file(|file| file.create_view_model_instance_for_artboard(artboard.core_handle()));
    artboard.bind_view_model_instance(model.clone());
    let machine = artboard.default_state_machine();
    if let (Some(machine), Some(model)) = (&machine, model) {
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(model));
    }
    let mut static_scene = machine
        .is_none()
        .then(|| StaticScene::new(artboard.downgrade()));
    let mut replay = DeferredReplayer::default();
    let mut output = Vec::new();
    for seconds in [0.0, 1.0 / 60.0, 1.0 / 60.0] {
        if let Some(machine) = &machine {
            machine.advance_and_apply(seconds);
        } else {
            static_scene.as_mut().unwrap().advance_and_apply(seconds);
        }
        let dimensions = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
        let scale = (SIZE as f32 / dimensions.0).min(SIZE as f32 / dimensions.1);
        let transform = Mat2D([
            scale,
            0.0,
            0.0,
            scale,
            (SIZE as f32 - scale * dimensions.0) * 0.5,
            (SIZE as f32 - scale * dimensions.1) * 0.5,
        ]);
        if deferred {
            session.record_ore_replay_marker();
            let mut renderer = session.make_screen_renderer(0);
            renderer.save();
            renderer.transform(transform);
            artboard.draw(renderer.as_mut());
            renderer.restore();
            let frame = take_frame(&mut session);
            assert!(
                !frame.commands.is_empty(),
                "runtime must record real drawing"
            );
            replay.replay_frame(&frame, &mut host);
            assert_eq!(
                replay.dropped_draws(),
                0,
                "all imported resources must resolve"
            );
        } else {
            let renderer = host.begin_screen_frame(0).unwrap();
            renderer.borrow_mut().save();
            renderer.borrow_mut().transform(transform);
            artboard.draw(renderer.borrow_mut().as_mut());
            renderer.borrow_mut().restore();
        }
        output.push(host.finish_frame());
    }
    output
}

#[test]
fn runtime_file_import_snapshot_replay_matches_immediate_frames() {
    for name in [
        "Halloween_v3",
        "Knight_square_2",
        "Tom_Morello",
        "UI_Swipe_left_to_delete",
        "falling",
        "popsicle_loader",
    ] {
        let bytes = fixture(&format!("parity/{name}.riv"));
        let immediate = frames(&bytes, false);
        let deferred = frames(&bytes, true);
        for (index, (expected, actual)) in immediate.iter().zip(&deferred).enumerate() {
            assert_eq!(expected.len(), (SIZE * SIZE * 4) as usize);
            assert_eq!(actual.len(), expected.len());
            let differing_channels = expected.iter().zip(actual).filter(|(a, b)| a != b).count();
            let max_channel_delta = expected
                .iter()
                .zip(actual)
                .map(|(a, b)| a.abs_diff(*b))
                .max()
                .unwrap_or(0);
            assert_eq!(
                expected.iter().zip(actual).position(|(a, b)| a != b),
                None,
                "{name} frame {index}: deferred-import pixels differ; {differing_channels} differing channels, maximum channel delta {max_channel_delta}"
            );
        }
    }
}
