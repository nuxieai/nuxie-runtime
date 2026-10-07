use nuxie_runtime::{
    source::{
        core::CoreArena,
        data_bind::data_bind::{DataBind, TO_SOURCE},
        generated::core_registry::CoreRegistry,
        viewmodel::viewmodel_instance_number::ViewModelInstanceNumber,
    },
    video::{
        Video,
        sync::{MediaClock, report_media_clock},
    },
};

#[test]
fn a_frame_does_not_notify_position_when_the_bound_clock_value_is_unchanged() {
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    video.with_downcast_mut::<Video, _>(|v| {
        v.playback.opened(0, 12.5);
    });
    Video::flush_property_changes(&video);
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
    assert_eq!(binding.with(|b| b.as_data_bind().unwrap().dirt()), Some(0));
    video.with_downcast_mut::<Video, _>(|v| {
        assert!(v.playback.accept_frame(0, 3.1));
    });
    assert!(!Video::flush_property_changes(&video));
    assert_eq!(binding.with(|b| b.as_data_bind().unwrap().dirt()), Some(0));
    assert_eq!(CoreRegistry::get_double_handle(&number, 575), Some(3.2));
}

#[test]
fn property_flush_reports_no_binding_dirt_for_an_unbound_video() {
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    video.with_downcast_mut::<Video, _>(|v| {
        v.playback.opened(0, 12.5);
    });
    assert!(!Video::flush_property_changes(&video));
}

#[test]
fn clock_changes_below_binding_precision_do_not_dirty_position_again() {
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    video.with_downcast_mut::<Video, _>(|v| {
        v.playback.opened(0, 12.5);
    });
    Video::flush_property_changes(&video);
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
    assert_eq!(binding.with(|b| b.as_data_bind().unwrap().dirt()), Some(0));
    assert_eq!(3.2_f64 as f32, 3.20000001_f64 as f32);
    report_media_clock(
        &video,
        1.1,
        Some(MediaClock {
            generation: 0,
            seconds: 3.20000001,
            rate: 1.0,
            playing: true,
        }),
    )
    .unwrap();
    assert_eq!(binding.with(|b| b.as_data_bind().unwrap().dirt()), Some(0));
    assert_eq!(CoreRegistry::get_double_handle(&number, 575), Some(3.2));
}

#[test]
fn registry_position_completion_does_not_notify_twice() {
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    let number = arena.insert(ViewModelInstanceNumber::default());
    let binding = arena.insert(DataBind::new(TO_SOURCE, 60017, 0));
    binding.with_mut(|b| {
        let b = b.as_data_bind_mut().unwrap();
        b.set_target(Some(video.clone()));
        b.set_source(number.clone());
    });
    DataBind::bind_handle(&binding);
    DataBind::update_data_bind_handle(&binding, true);
    CoreRegistry::set_double_handle(&video, 60017, 3.0);
    DataBind::update_data_bind_handle(&binding, true);
    assert_eq!(CoreRegistry::get_double_handle(&number, 575), Some(3.0));
    assert!(!Video::flush_property_changes(&video));
    assert_eq!(binding.with(|b| b.as_data_bind().unwrap().dirt()), Some(0));
}

#[test]
fn a_registry_loop_write_notifies_position_without_a_host_flush() {
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    video.with_downcast_mut::<Video, _>(|v| {
        v.playback.opened(0, 12.5);
    });
    Video::flush_property_changes(&video);
    let number = arena.insert(ViewModelInstanceNumber::default());
    let binding = arena.insert(DataBind::new(TO_SOURCE, 60017, 0));
    binding.with_mut(|b| {
        let b = b.as_data_bind_mut().unwrap();
        b.set_target(Some(video.clone()));
        b.set_source(number.clone());
    });
    DataBind::bind_handle(&binding);
    DataBind::update_data_bind_handle(&binding, true);
    CoreRegistry::set_double_handle(&video, 60015, 5.0);
    CoreRegistry::set_double_handle(&video, 60014, 2.0);
    CoreRegistry::set_bool_handle(&video, 60004, true);
    assert_ne!(binding.with(|b| b.as_data_bind().unwrap().dirt()), Some(0));
    DataBind::update_data_bind_handle(&binding, true);
    assert_eq!(CoreRegistry::get_double_handle(&number, 575), Some(2.0));
    assert!(!Video::flush_property_changes(&video));
}

#[test]
fn clamped_position_write_seeks_without_notifying_an_unchanged_value() {
    use nuxie_runtime::video::{
        playback::{DecoderAction, Playback, PlaybackSettings},
        properties::POSITION,
    };
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    video.with_downcast_mut::<Video, _>(|v| {
        v.playback = Playback::new(PlaybackSettings {
            looping: true,
            loop_start: 2.0,
            loop_end: 5.0,
            ..PlaybackSettings::default()
        });
        v.playback.opened(0, 12.5);
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
    DataBind::update_data_bind_handle(&binding, true);
    assert_eq!(CoreRegistry::get_double_handle(&number, 575), Some(2.0));
    assert_eq!(binding.with(|b| b.as_data_bind().unwrap().dirt()), Some(0));
    CoreRegistry::set_double_handle(&video, POSITION.into(), 8.0);
    assert_eq!(
        CoreRegistry::get_double_handle(&video, POSITION.into()),
        Some(2.0)
    );
    assert_eq!(binding.with(|b| b.as_data_bind().unwrap().dirt()), Some(0));
    assert!(!Video::flush_property_changes(&video));
    video.with_downcast_mut::<Video, _>(|v| {
        assert!(v.playback.drain_actions().contains(&DecoderAction::Seek {
            seconds: 2.0,
            generation: 1
        }));
    });
}

fn assert_registry_position_change(replay: bool, borrowed: bool) {
    use nuxie_runtime::video::playback::{Playback, PlaybackSettings};

    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    let (initial, expected) = if replay { (12.5, 0.0) } else { (3.2, 4.0) };
    video.with_downcast_mut::<Video, _>(|v| {
        if !replay {
            v.playback = Playback::new(PlaybackSettings {
                looping: true,
                loop_start: 2.0,
                loop_end: 5.0,
                ..PlaybackSettings::default()
            });
        }
        v.playback.opened(0, 12.5);
        assert!(v.playback.accept_frame(0, initial));
        if replay {
            v.playback.ended(0);
        }
    });
    report_media_clock(
        &video,
        1.0,
        Some(MediaClock {
            generation: 0,
            seconds: initial,
            rate: 1.0,
            playing: !replay,
        }),
    )
    .unwrap();
    let number = arena.insert(ViewModelInstanceNumber::default());
    let binding = arena.insert(DataBind::new(TO_SOURCE, 60017, 0));
    binding.with_mut(|b| {
        let b = b.as_data_bind_mut().unwrap();
        b.set_target(Some(video.clone()));
        b.set_source(number.clone());
    });
    DataBind::bind_handle(&binding);
    DataBind::update_data_bind_handle(&binding, true);
    assert_eq!(
        CoreRegistry::get_double_handle(&number, 575),
        Some(initial as f32)
    );
    assert_eq!(binding.with(|b| b.as_data_bind().unwrap().dirt()), Some(0));
    if borrowed {
        video.with_downcast_mut::<Video, _>(|v| {
            if replay {
                CoreRegistry::set_bool(v, 60016, false);
            } else {
                CoreRegistry::set_double(v, 60014, 4.0);
            }
        });
    } else if replay {
        CoreRegistry::set_bool_handle(&video, 60016, false);
    } else {
        CoreRegistry::set_double_handle(&video, 60014, 4.0);
    }
    assert_eq!(
        CoreRegistry::get_double_handle(&video, 60017),
        Some(expected)
    );
    assert_ne!(binding.with(|b| b.as_data_bind().unwrap().dirt()), Some(0));
    DataBind::update_data_bind_handle(&binding, true);
    assert_eq!(
        CoreRegistry::get_double_handle(&number, 575),
        Some(expected)
    );
    assert!(!Video::flush_property_changes(&video));
}

#[test]
fn registry_replay_notifies_position_without_a_host_flush() {
    assert_registry_position_change(true, false);
}

#[test]
fn registry_range_repair_notifies_position_without_a_host_flush() {
    assert_registry_position_change(false, false);
}

#[test]
fn borrowed_registry_replay_notifies_position_without_a_host_flush() {
    assert_registry_position_change(true, true);
}

#[test]
fn borrowed_registry_range_repair_notifies_position_without_a_host_flush() {
    assert_registry_position_change(false, true);
}
