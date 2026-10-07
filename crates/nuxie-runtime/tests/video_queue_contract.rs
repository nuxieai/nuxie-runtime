use nuxie_runtime::{
    source::{core::CoreArena, generated::core_registry::CoreRegistry},
    video::{
        Video,
        playback::PlaybackError,
        properties::{VOLUME, Write},
    },
};

#[test]
fn a_full_video_queue_keeps_the_last_value_and_accepts_writes_after_drain() {
    let arena = CoreArena::default();
    let video = arena.insert(Video::default());
    for index in 0..256 {
        CoreRegistry::set_double_handle(
            &video,
            VOLUME.into(),
            if index % 2 == 0 { 0.25 } else { 0.75 },
        );
    }
    CoreRegistry::set_double_handle(&video, VOLUME.into(), 0.5);
    assert_eq!(
        CoreRegistry::get_double_handle(&video, VOLUME.into()),
        Some(0.75)
    );
    video.with_downcast_mut::<Video, _>(|v| {
        assert!(matches!(
            v.write_property(Write::Number(VOLUME, 0.5)),
            Err(PlaybackError::QueueFull)
        ));
        assert_eq!(v.playback.drain_actions().len(), 256);
        v.write_property(Write::Number(VOLUME, 0.5)).unwrap();
    });
    assert_eq!(
        CoreRegistry::get_double_handle(&video, VOLUME.into()),
        Some(0.5)
    );
}
