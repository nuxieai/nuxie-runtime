use nuxie_runtime::{
    source::{
        core::CoreArena,
        data_bind::data_bind::{DataBind, TO_SOURCE},
        generated::core_registry::CoreRegistry,
        viewmodel::viewmodel_instance_number::ViewModelInstanceNumber,
    },
    video::{
        Video,
        playback::{Command, DecoderAction, Playback, PlaybackSettings},
        properties::{LOOP, POSITION, VOLUME, Write},
        sync::{MediaClock, report_media_clock},
    },
};

#[test]
fn loop_repairs_invalidate_the_clock_only_when_the_queue_seeks() {
    for (start, end, expected, seeks) in [(4.0, 5.0, 4.0, true), (2.0, 5.0, 3.2, false)] {
        let arena = CoreArena::default();
        let video = arena.insert(Video::default());
        video.with_downcast_mut::<Video, _>(|v| {
            v.playback.opened(0, 12.5);
            assert!(v.playback.accept_frame(0, 3.0));
        });
        Video::flush_property_changes(&video);
        let number = arena.insert(ViewModelInstanceNumber::default());
        let binding = arena.insert(DataBind::new(TO_SOURCE, POSITION as u32, 0));
        binding.with_mut(|b| {
            let b = b.as_data_bind_mut().unwrap();
            b.set_target(Some(video.clone()));
            b.set_source(number.clone());
        });
        DataBind::bind_handle(&binding);
        report_media_clock(
            &video,
            1.0,
            Some(MediaClock {
                generation: 0,
                seconds: 3.2,
                rate: 1.0,
                playing: true,
            }),
        )
        .unwrap();
        DataBind::update_data_bind_handle(&binding, true);
        video.with_downcast_mut::<Video, _>(|v| {
            v.write_property(Write::LoopRange { start, end }).unwrap();
            v.write_property(Write::Bool(LOOP, true)).unwrap();
        });
        assert_eq!(
            CoreRegistry::get_double_handle(&video, POSITION.into()),
            Some(expected)
        );
        assert_eq!(Video::flush_property_changes(&video), seeks);
        DataBind::update_data_bind_handle(&binding, true);
        assert_eq!(
            CoreRegistry::get_double_handle(&number, 575),
            Some(expected)
        );
        video.with_downcast_mut::<Video, _>(|v| {
            let actions = v.playback.drain_actions();
            if seeks {
                assert!(actions.contains(&DecoderAction::Seek {
                    seconds: 4.0,
                    generation: 1
                }));
            } else {
                assert!(
                    !actions
                        .iter()
                        .any(|a| matches!(a, DecoderAction::Seek { .. }))
                );
            }
        });
        assert_eq!(
            CoreRegistry::get_double_handle(&video, POSITION.into()),
            Some(expected)
        );
    }
}

#[test]
fn pending_disposal_discards_later_property_writes() {
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    video.with_downcast_mut::<Video, _>(|v| {
        v.playback.enqueue(Command::Dispose).unwrap();
        v.write_property(Write::Number(VOLUME, 0.5)).unwrap();
    });
    assert_eq!(
        CoreRegistry::get_double_handle(&video, VOLUME.into()),
        Some(1.0)
    );
    video.with_downcast_mut::<Video, _>(|v| {
        assert_eq!(v.playback.drain_actions(), vec![DecoderAction::Dispose]);
    });
    assert_eq!(
        CoreRegistry::get_double_handle(&video, VOLUME.into()),
        Some(1.0)
    );
}

#[test]
fn reentry_that_keeps_playback_does_not_replace_the_current_clock() {
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    video.with_downcast_mut::<Video, _>(|v| {
        v.playback = Playback::new(PlaybackSettings {
            reentry: 1,
            ..PlaybackSettings::default()
        });
        v.playback.opened(0, 12.5);
        assert!(v.playback.accept_frame(0, 3.0));
    });
    report_media_clock(
        &video,
        1.0,
        Some(MediaClock {
            generation: 0,
            seconds: 3.2,
            rate: 1.0,
            playing: true,
        }),
    )
    .unwrap();
    video.with_downcast_mut::<Video, _>(|v| v.playback.enqueue(Command::Reenter).unwrap());
    assert_eq!(
        CoreRegistry::get_double_handle(&video, POSITION.into()),
        Some(3.2)
    );
    video.with_downcast_mut::<Video, _>(|v| {
        assert!(
            !v.playback
                .drain_actions()
                .iter()
                .any(|a| matches!(a, DecoderAction::Seek { .. }))
        );
    });
    assert_eq!(
        CoreRegistry::get_double_handle(&video, POSITION.into()),
        Some(3.2)
    );
}

#[test]
fn replay_from_ended_projects_its_seek_before_the_decoder_runs() {
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    video.with_downcast_mut::<Video, _>(|v| {
        v.playback.opened(0, 12.5);
        assert!(v.playback.accept_frame(0, 12.5));
        v.playback.ended(0);
    });
    report_media_clock(
        &video,
        1.0,
        Some(MediaClock {
            generation: 0,
            seconds: 12.5,
            rate: 1.0,
            playing: false,
        }),
    )
    .unwrap();
    video.with_downcast_mut::<Video, _>(|v| v.playback.enqueue(Command::Play).unwrap());
    assert_eq!(
        CoreRegistry::get_double_handle(&video, POSITION.into()),
        Some(0.0)
    );
    video.with_downcast_mut::<Video, _>(|v| {
        assert!(v.playback.drain_actions().contains(&DecoderAction::Seek {
            seconds: 0.0,
            generation: 1
        }));
    });
    assert_eq!(
        CoreRegistry::get_double_handle(&video, POSITION.into()),
        Some(0.0)
    );
}
