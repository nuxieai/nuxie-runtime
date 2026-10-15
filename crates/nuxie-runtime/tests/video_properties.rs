use nuxie_runtime::{
    source::{core::CoreArena, generated::core_registry::CoreRegistry},
    video::{Video, playback::DecoderAction},
};

#[test]
fn video_properties_write_the_decoder_queue_and_reject_invalid_volume() {
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    video.with_downcast_mut::<Video, _>(|v| v.playback.opened(0, 12.5));
    CoreRegistry::set_bool_handle(&video, 60005, true);
    assert_eq!(CoreRegistry::get_bool_handle(&video, 60005), Some(true));
    video.with_downcast_mut::<Video, _>(|v| {
        assert_eq!(v.playback.drain_actions(), vec![DecoderAction::Volume(0.0)]);
    });
    CoreRegistry::set_bool_handle(&video, 60005, false);
    CoreRegistry::set_double_handle(&video, 60006, 0.5);
    CoreRegistry::set_double_handle(&video, 60006, 2.0);
    assert_eq!(CoreRegistry::get_double_handle(&video, 60006), Some(0.5));
    video.with_downcast_mut::<Video, _>(|v| {
        assert_eq!(
            v.playback.drain_actions(),
            vec![DecoderAction::Volume(1.0), DecoderAction::Volume(0.5)]
        );
    });
}

#[test]
fn video_properties_expose_decoder_metadata_and_state() {
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    assert_eq!(CoreRegistry::get_bool_handle(&video, 60016), Some(true));
    assert_eq!(CoreRegistry::get_double_handle(&video, 60018), Some(0.0));
    video.with_downcast_mut::<Video, _>(|v| {
        v.playback.opened(0, 12.5);
        v.playback.ended(0);
    });
    assert_eq!(CoreRegistry::get_double_handle(&video, 60018), Some(12.5));
    assert_eq!(CoreRegistry::get_double_handle(&video, 60020), Some(6.0));
    assert_eq!(CoreRegistry::get_bool_handle(&video, 60019), Some(true));
    CoreRegistry::set_double_handle(&video, 60018, 1.0);
    assert_eq!(CoreRegistry::get_double_handle(&video, 60018), Some(12.5));
}

#[test]
fn schema_typed_video_boolean_writes_reach_playback() {
    use nuxie_schema::{CoreRegistryFieldKind, definition_by_name};
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    let definition = definition_by_name("Video").unwrap();
    for key in [60004, 60005] {
        let property = definition
            .properties
            .iter()
            .find(|p| p.key.int == key)
            .unwrap();
        // Dispatch as schema consumers do, rather than assuming the field family.
        match CoreRegistryFieldKind::from_property(property).unwrap() {
            CoreRegistryFieldKind::Bool => {
                CoreRegistry::set_bool_handle(&video, key.into(), true);
            }
            CoreRegistryFieldKind::Uint => {
                CoreRegistry::set_uint_handle(&video, key.into(), 1);
            }
            other => panic!("unexpected video flag type: {other:?}"),
        }
        assert_eq!(
            CoreRegistry::get_bool_handle(&video, key.into()),
            Some(true),
            "schema write to {key}"
        );
    }
}

#[test]
fn video_loop_bounds_reject_empty_ranges_and_zero_end_uses_duration() {
    use nuxie_runtime::video::properties::{LOOP, LOOP_END, LOOP_START, PAUSED, Write};
    let mut video = Video::default();
    video.playback.opened(0, 12.5);
    for start in [12.5, 13.0] {
        assert!(
            video
                .write_property(Write::Number(LOOP_START, start))
                .is_err()
        );
    }
    video
        .write_property(Write::Number(LOOP_START, 3.0))
        .unwrap();
    for end in [2.0, 3.0] {
        assert!(video.write_property(Write::Number(LOOP_END, end)).is_err());
    }
    video.write_property(Write::Number(LOOP_END, 0.0)).unwrap();
    video.write_property(Write::Bool(LOOP, true)).unwrap();
    video.write_property(Write::Bool(PAUSED, false)).unwrap();
    video.playback.drain_actions();
    let generation = video.playback.generation();
    assert!(video.playback.accept_frame(generation, 12.4));
    assert!(!video.playback.accept_frame(generation, 12.5));
    let actions = video.playback.drain_actions();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, DecoderAction::Seek { seconds: 3.0, .. }))
    );
}

#[test]
fn authored_paused_and_position_survive_later_fields_and_instancing() {
    use nuxie_runtime::source::core::{CoreObject, binary_reader::BinaryReader};
    let mut video = Video::default();
    assert!(video.deserialize(60016, &mut BinaryReader::new(&[0])));
    assert!(video.deserialize(60017, &mut BinaryReader::new(&3.0_f32.to_le_bytes())));
    assert!(video.deserialize(60007, &mut BinaryReader::new(&2.0_f32.to_le_bytes())));
    let mut cloned = video.clone_boxed().unwrap();
    assert!(!CoreRegistry::get_bool(&mut *cloned, 60016));
    assert_eq!(CoreRegistry::get_double(&mut *cloned, 60017), 3.0);
    assert_eq!(CoreRegistry::get_double(&mut *cloned, 60007), 2.0);
}

#[test]
fn seeking_to_the_last_frame_still_publishes_the_changed_media_clock_position() {
    use nuxie_runtime::{
        source::{
            data_bind::data_bind::{DataBind, TO_SOURCE},
            viewmodel::viewmodel_instance_number::ViewModelInstanceNumber,
        },
        video::{
            properties::{POSITION, Write},
            sync::{MediaClock, report_media_clock},
        },
    };
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    video.with_downcast_mut::<Video, _>(|v| {
        v.playback.opened(0, 12.5);
        assert!(v.playback.accept_frame(0, 3.0));
    });
    let number = arena.insert(ViewModelInstanceNumber::default());
    let binding = arena.insert(DataBind::new(TO_SOURCE, 60017, 0));
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
    assert_eq!(CoreRegistry::get_double_handle(&number, 575), Some(3.2));
    video.with_downcast_mut::<Video, _>(|v| {
        v.write_property(Write::Number(POSITION, 3.0)).unwrap();
    });
    Video::flush_property_changes(&video);
    DataBind::update_data_bind_handle(&binding, true);
    assert_eq!(CoreRegistry::get_double_handle(&number, 575), Some(3.0));
}

#[test]
fn synchronization_correction_publishes_follower_position() {
    use nuxie_runtime::{
        source::{
            data_bind::data_bind::{DataBind, TO_SOURCE},
            viewmodel::viewmodel_instance_number::ViewModelInstanceNumber,
        },
        video::{
            playback::Command,
            sync::{MediaClock, SynchronizationGroup, report_media_clock},
        },
    };
    let arena = CoreArena::default();
    let leader = arena.insert(Video::default());
    let follower = arena.insert(Video::default());
    for member in [&leader, &follower] {
        member.with_downcast_mut::<Video, _>(|v| {
            v.playback.opened(0, 12.5);
            v.playback.enqueue(Command::Play).unwrap();
            v.playback.drain_actions();
            v.playback.observed_playing(0);
        });
    }
    let number = arena.insert(ViewModelInstanceNumber::default());
    let binding = arena.insert(DataBind::new(TO_SOURCE, 60017, 0));
    binding.with_mut(|b| {
        let b = b.as_data_bind_mut().unwrap();
        b.set_target(Some(follower.clone()));
        b.set_source(number.clone());
    });
    DataBind::bind_handle(&binding);
    let clock = |seconds| MediaClock {
        generation: 0,
        seconds,
        rate: 1.0,
        playing: true,
    };
    report_media_clock(&follower, 1.0, Some(clock(0.7))).unwrap();
    DataBind::update_data_bind_handle(&binding, true);
    assert_eq!(CoreRegistry::get_double_handle(&number, 575), Some(0.7));
    let mut group =
        SynchronizationGroup::new(leader.clone(), vec![follower.clone()], 0.05, 0.1).unwrap();
    let measurements = group
        .correct(1.0, |member| {
            Some(clock(if member.identity_key() == leader.identity_key() {
                1.0
            } else {
                0.7
            }))
        })
        .unwrap();
    assert!(measurements[0].corrected);
    DataBind::update_data_bind_handle(&binding, true);
    assert_eq!(CoreRegistry::get_double_handle(&number, 575), Some(1.0));
}
