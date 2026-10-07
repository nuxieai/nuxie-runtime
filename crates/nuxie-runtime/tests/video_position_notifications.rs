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
fn a_loop_write_keeps_its_pending_position_notification() {
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
    assert!(Video::flush_property_changes(&video));
    DataBind::update_data_bind_handle(&binding, true);
    assert_eq!(CoreRegistry::get_double_handle(&number, 575), Some(2.0));
}
