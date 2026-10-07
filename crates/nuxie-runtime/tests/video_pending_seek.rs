use nuxie_runtime::{
    source::{core::CoreArena, generated::core_registry::CoreRegistry},
    video::{
        Video,
        properties::{LOOP, POSITION, Write},
    },
};

#[test]
fn pending_loop_seek_reads_the_position_the_decoder_will_receive() {
    // Each independently specified case checks both the pending binding value
    // and the final playback position, not one implementation against another.
    for (writes, expected) in [
        (
            vec![
                Write::Bool(LOOP, true),
                Write::LoopRange {
                    start: 2.0,
                    end: 5.0,
                },
                Write::Number(POSITION, 8.0),
            ],
            2.0,
        ),
        (
            vec![
                Write::Bool(LOOP, true),
                Write::LoopRange {
                    start: 2.0,
                    end: 5.0,
                },
                Write::Number(POSITION, 1.0),
            ],
            2.0,
        ),
        (
            vec![
                Write::Bool(LOOP, true),
                Write::LoopRange {
                    start: 2.0,
                    end: 5.0,
                },
                Write::Number(POSITION, 4.0),
            ],
            4.0,
        ),
        (
            vec![
                Write::Number(POSITION, 8.0),
                Write::LoopRange {
                    start: 2.0,
                    end: 5.0,
                },
                Write::Bool(LOOP, true),
            ],
            2.0,
        ),
        (
            vec![
                Write::Bool(LOOP, true),
                Write::LoopRange {
                    start: 2.0,
                    end: 5.0,
                },
                Write::Bool(LOOP, false),
                Write::Number(POSITION, 8.0),
            ],
            8.0,
        ),
        (vec![Write::Number(POSITION, 20.0)], 12.5),
    ] {
        let arena = CoreArena::default();
        let video = arena.insert(Video::default());
        video.with_downcast_mut::<Video, _>(|v| {
            v.playback.opened(0, 12.5);
            for write in writes {
                v.write_property(write).unwrap();
            }
        });
        assert_eq!(
            CoreRegistry::get_double_handle(&video, POSITION.into()),
            Some(expected)
        );
        video.with_downcast_mut::<Video, _>(|v| {
            v.playback.drain_actions();
            assert_eq!(v.playback.position(), f64::from(expected));
        });
        assert_eq!(
            CoreRegistry::get_double_handle(&video, POSITION.into()),
            Some(expected)
        );
    }
}
