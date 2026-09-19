use nuxie_binary::{
    FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, encode_runtime_file,
};
use nuxie_render_api::{Factory, PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    File, RuntimeFactoryHandle,
    video::{Video, VideoAsset, playback::*},
};
use std::rc::Rc;

fn property(owner: &str, name: &str, value: FixtureValue) -> FixtureProperty {
    let def = nuxie_schema::definition_by_name(owner).unwrap();
    let key = std::iter::once(def.name)
        .chain(def.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|d| d.properties)
        .find(|p| p.name == name)
        .unwrap()
        .key
        .int;
    FixtureProperty { key, value }
}
fn record(owner: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
    FixtureRecord {
        type_key: nuxie_schema::definition_by_name(owner)
            .unwrap()
            .type_key
            .int,
        properties,
    }
}
fn scene_records(embedded: bool) -> Vec<FixtureRecord> {
    let mut records = vec![
        record("Backboard", vec![]),
        record(
            "VideoAsset",
            vec![
                property(
                    "VideoAsset",
                    "sourceKey",
                    FixtureValue::String("assets/sha256/test.mp4".into()),
                ),
                property("VideoAsset", "width", FixtureValue::Double(64.0)),
                property("VideoAsset", "height", FixtureValue::Double(32.0)),
            ],
        ),
    ];
    if embedded {
        records.push(record(
            "FileAssetContents",
            vec![property(
                "FileAssetContents",
                "bytes",
                FixtureValue::Bytes(vec![0, 0, 0, 20, 102, 116, 121, 112]),
            )],
        ));
    }
    records.push(record(
        "Artboard",
        vec![
            property("Artboard", "width", FixtureValue::Double(200.0)),
            property("Artboard", "height", FixtureValue::Double(200.0)),
        ],
    ));
    records.push(record(
        "Video",
        vec![
            property("Video", "parentId", FixtureValue::Uint(0)),
            property("Video", "assetId", FixtureValue::Uint(0)),
            property("Video", "autoplay", FixtureValue::Uint(1)),
            property("Video", "volume", FixtureValue::Double(0.4)),
            property("Video", "x", FixtureValue::Double(80.0)),
        ],
    ));
    records
}
fn scene(embedded: bool) -> Vec<u8> {
    encode_runtime_file(&RuntimeFile::from_fixture_records(scene_records(embedded)).unwrap())
        .unwrap()
}
#[test]
fn extended_stream_roundtrip_preserves_keys_and_embedded_bytes() {
    for embedded in [false, true] {
        let bytes = scene(embedded);
        assert_eq!(&bytes[..4], b"RIVE");
        let file = nuxie_binary::read_runtime_file(&bytes).unwrap();
        assert!(file.objects.iter().flatten().any(|o| o.type_key == 60000));
        assert!(file.objects.iter().flatten().any(|o| o.type_key == 60001));
        assert_eq!(encode_runtime_file(&file).unwrap(), bytes);
    }
}
#[test]
fn real_import_instancing_and_scene_drawing_use_video_owners() {
    for embedded in [false, true] {
        let bytes = scene(embedded);
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let asset = file.with_file(|f| f.assets()[0].clone());
        asset
            .with_downcast::<VideoAsset, _>(|a| {
                assert_eq!(a.source_key, "assets/sha256/test.mp4");
                assert_eq!(a.encoded_bytes().is_some(), embedded);
            })
            .unwrap();
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        artboard.update_pass(true);
        let video = artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                    .cloned()
            })
            .unwrap();
        let mut png = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png, 64, 32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&vec![255; 64 * 32 * 4])
                .unwrap();
        }
        let image = Rc::from(factory.decode_image(&png).unwrap());
        video
            .with_downcast_mut::<Video, _>(|v| {
                assert_eq!(v.playback.settings().volume, 0.4);
                assert!(v.playback.settings().autoplay);
                assert!(v.asset().is_some());
                assert!(v.present(0, image, 0.0));
            })
            .unwrap();
        // Authored layout and pointer semantics use the same frame geometry.
        video.with_mut(|object| {
            use nuxie_runtime::source::{
                layout::layout_enums::{LayoutDirection, LayoutScaleType},
                math::vec2d::Vec2D,
            };
            object
                .as_intrinsically_sizeable_mut()
                .unwrap()
                .control_size(
                    Vec2D::new(128.0, 64.0),
                    LayoutScaleType::Fixed,
                    LayoutScaleType::Fixed,
                    LayoutDirection::Ltr,
                );
            assert_eq!(
                object.semantic_provider_local_bounds().unwrap().width(),
                64.0
            );
        });
        artboard.update_pass(true);
        video.with_mut(|object| {
            use nuxie_runtime::source::{
                hit_info::HitInfo,
                math::{aabb::IAabb, mat2d::Mat2D},
            };
            let world = object.as_transform_component().unwrap().world_transform();
            assert_eq!(world[0], 2.0);
            assert_eq!(world[3], 2.0);
            let mut inside = HitInfo {
                area: IAabb {
                    left: 80,
                    top: 0,
                    right: 81,
                    bottom: 1,
                },
                mounts: vec![],
            };
            assert!(
                object
                    .drawable_hit_test(&mut inside, &Mat2D::default())
                    .is_some()
            );
            let mut outside = HitInfo {
                area: IAabb {
                    left: 200,
                    top: 200,
                    right: 201,
                    bottom: 201,
                },
                mounts: vec![],
            };
            assert!(
                object
                    .drawable_hit_test(&mut outside, &Mat2D::default())
                    .is_none()
            );
        });
        let mut renderer = factory.borrow().make_renderer();
        artboard.draw(&mut renderer);
        assert!(
            factory
                .borrow()
                .canonical_recording()
                .stream()
                .contains("drawImage"),
            "{}",
            factory.borrow().canonical_recording().stream()
        );
    }
}
#[test]
fn lifecycle_wins_over_ordered_play_intent() {
    let mut p = Playback::default();
    p.opened(0, 10.0);
    p.enqueue(Command::Play).unwrap();
    p.enqueue(Command::Suspend(true)).unwrap();
    assert!(!p.drain_actions().contains(&DecoderAction::Play));
    assert!(p.wants_play());
    p.enqueue(Command::Suspend(false)).unwrap();
    assert_eq!(p.drain_actions(), vec![DecoderAction::Play]);
    p.enqueue(Command::Play).unwrap();
    p.enqueue(Command::Pause).unwrap();
    assert_eq!(p.drain_actions(), vec![DecoderAction::Pause]);
}
#[test]
fn seek_dispose_and_loops_reject_obsolete_decoder_callbacks() {
    let mut p = Playback::new(PlaybackSettings {
        looping: true,
        autoplay: true,
        ..Default::default()
    });
    p.opened(0, 10.0);
    p.enqueue(Command::Seek(5.0)).unwrap();
    assert_eq!(
        p.drain_actions(),
        vec![DecoderAction::Seek {
            seconds: 5.0,
            generation: 1
        }]
    );
    assert!(!p.accept_frame(0, 1.0));
    assert!(p.accept_frame(1, 5.0));
    assert_eq!(
        p.ended(1),
        vec![
            DecoderAction::Seek {
                seconds: 0.0,
                generation: 2
            },
            DecoderAction::Play
        ]
    );
    assert!(p.ended(1).is_empty());
    p.enqueue(Command::Dispose).unwrap();
    assert_eq!(p.drain_actions(), vec![DecoderAction::Dispose]);
    assert!(!p.accept_frame(2, 0.0));
    assert_eq!(p.enqueue(Command::Play), Err(PlaybackError::Disposed));
}
#[test]
fn invalid_commands_do_not_reach_decoder() {
    let mut p = Playback::default();
    for c in [
        Command::Seek(f64::NAN),
        Command::Seek(-1.0),
        Command::Rate(0.0),
        Command::Volume(2.0),
    ] {
        assert_eq!(p.enqueue(c), Err(PlaybackError::InvalidValue));
    }
    assert!(p.drain_actions().is_empty());
}
#[test]
fn failed_or_duplicate_open_cannot_restart_playback() {
    let mut p = Playback::default();
    p.opened(0, 10.0);
    p.enqueue(Command::Seek(5.0)).unwrap();
    p.drain_actions();
    assert!(p.opened(1, 10.0).is_empty());
    assert_eq!(p.state(), PlaybackState::Seeking);
    p.failed(1);
    assert!(p.opened(1, 10.0).is_empty());
    assert!(!p.accept_frame(1, 5.0));
    assert_eq!(p.state(), PlaybackState::Failed);
}
#[test]
fn decoder_saturation_preserves_foreground_and_bounds_software() {
    use nuxie_runtime::video::resources::*;
    let make = |id, priority, visible| DecoderRequest {
        id,
        priority,
        visible,
        hardware_supported: true,
        managed_supported: false,
        software_supported: true,
        pixels_per_second: 100,
    };
    assert_eq!(
        allocate(
            &[
                make(1, 0, true),
                make(2, 10, true),
                make(3, 3, true),
                make(4, 20, false)
            ],
            DecoderBudget {
                max_players: 2,
                managed_players: 0,
                managed_pixels_per_second: 0,
                hardware_players: 1,
                software_pixels_per_second: 100
            }
        ),
        vec![
            (4, Allocation::Poster),
            (2, Allocation::Hardware),
            (3, Allocation::Software),
            (1, Allocation::Poster)
        ]
    );
}
#[test]
fn unsupported_host_rejects_video_before_asset_loading() {
    use nuxie_runtime::source::file::ImportAdmission;
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let admission = nuxie_runtime::video::admission::VideoAdmission::new(false, 1024, None);
    let result = File::import_with_admission(
        &scene(true),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
        admission.clone(),
    );
    assert!(result.is_none());
    assert!(admission.is_rejected());
}
#[test]
fn explicit_play_retries_autoplay_rejection_without_losing_intent() {
    let mut p = Playback::new(PlaybackSettings {
        autoplay: true,
        ..Default::default()
    });
    assert!(p.opened(0, 10.0).contains(&DecoderAction::Play));
    p.observed_play_blocked(0);
    assert!(p.wants_play());
    assert_eq!(p.state(), PlaybackState::Paused);
    p.enqueue(Command::Play).unwrap();
    assert_eq!(p.drain_actions(), vec![DecoderAction::Play]);
    p.enqueue(Command::Suspend(true)).unwrap();
    p.enqueue(Command::Play).unwrap();
    assert_eq!(p.drain_actions(), vec![DecoderAction::Pause]);
}
#[test]
fn captions_follow_seek_and_have_exclusive_end_boundaries() {
    use nuxie_runtime::video::captions::*;
    let track = CaptionTrack::new(
        "en".into(),
        vec![
            Cue {
                start: 0.0,
                end: 1.0,
                text: "Hello".into(),
            },
            Cue {
                start: 1.0,
                end: 2.0,
                text: "Again".into(),
            },
        ],
    )
    .unwrap();
    assert_eq!(track.text(0.0), "Hello");
    assert_eq!(track.text(1.0), "Again");
    assert_eq!(track.text(2.0), "");
    assert_eq!(track.text(0.5), "Hello");
    assert!(
        CaptionTrack::new(
            "en".into(),
            vec![Cue {
                start: f64::NAN,
                end: 2.0,
                text: String::new()
            }]
        )
        .is_err()
    );
}
#[test]
fn ending_an_interruption_does_not_resume_a_hidden_scene() {
    let mut p = Playback::new(PlaybackSettings {
        autoplay: true,
        ..Default::default()
    });
    p.opened(0, 10.0);
    p.enqueue(Command::SuspendReason {
        reason: SuspensionReason::Hidden,
        suspended: true,
    })
    .unwrap();
    p.enqueue(Command::SuspendReason {
        reason: SuspensionReason::Interruption,
        suspended: true,
    })
    .unwrap();
    assert_eq!(p.drain_actions(), vec![DecoderAction::Pause]);
    p.enqueue(Command::SuspendReason {
        reason: SuspensionReason::Interruption,
        suspended: false,
    })
    .unwrap();
    assert!(!p.drain_actions().contains(&DecoderAction::Play));
    p.enqueue(Command::SuspendReason {
        reason: SuspensionReason::Hidden,
        suspended: false,
    })
    .unwrap();
    assert_eq!(p.drain_actions(), vec![DecoderAction::Play]);
}

#[test]
fn lifecycle_pause_cannot_be_lost_to_a_full_script_command_queue() {
    let mut p = Playback::new(PlaybackSettings {
        autoplay: true,
        ..Default::default()
    });
    p.opened(0, 2.0);
    p.observed_playing(0);
    for _ in 0..256 {
        p.enqueue(Command::Play).unwrap();
    }
    assert_eq!(p.enqueue(Command::Play), Err(PlaybackError::QueueFull));
    assert_eq!(
        p.update_suspension(SuspensionReason::Interruption, true),
        vec![DecoderAction::Pause]
    );
    assert!(!p.drain_actions().contains(&DecoderAction::Play));
    assert!(p.wants_play());
    assert_eq!(
        p.update_suspension(SuspensionReason::Interruption, false),
        vec![DecoderAction::Play]
    );
}

#[test]
fn source_replacement_invalidates_old_decode_callbacks_and_preserves_suspension() {
    let settings = PlaybackSettings {
        autoplay: true,
        ..Default::default()
    };
    let mut p = Playback::new(settings);
    p.opened(0, 2.0);
    p.update_suspension(SuspensionReason::Background, true);
    let generation = p.replace_source(settings).unwrap();
    assert_ne!(generation, 0);
    assert!(!p.accept_frame(0, 1.0));
    assert!(p.opened(0, 2.0).is_empty());
    p.failed(0);
    assert_eq!(p.state(), PlaybackState::Opening);
    assert!(!p.opened(generation, 3.0).contains(&DecoderAction::Play));
    assert_eq!(
        p.update_suspension(SuspensionReason::Background, false),
        vec![DecoderAction::Play]
    );
    p.failed(generation);
    assert_eq!(p.enqueue(Command::Seek(0.0)), Err(PlaybackError::Failed));
    assert!(!p.accept_frame(generation, 0.0));
    let recovered = p.replace_source(settings).unwrap();
    assert!(p.opened(recovered, 3.0).contains(&DecoderAction::Play));
}

#[test]
fn malformed_video_asset_reference_is_rejected_without_panicking() {
    let bytes = scene(false);
    let mut decoded = nuxie_binary::read_runtime_file(&bytes).unwrap();
    let video = decoded
        .objects
        .iter_mut()
        .flatten()
        .find(|o| o.type_key == Video::TYPE_KEY)
        .unwrap();
    let asset = video.properties.iter_mut().find(|p| p.key == 206).unwrap();
    asset.value = nuxie_binary::FieldValue::Uint(999);
    let malformed = encode_runtime_file(&decoded).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    assert!(
        File::import(
            &malformed,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None
        )
        .is_none()
    );
}

#[test]
fn authored_poster_is_available_before_decode_and_after_frame_release() {
    let mut png = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png, 64, 32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&vec![255; 64 * 32 * 4])
            .unwrap();
    }
    let records = vec![
        record("Backboard", vec![]),
        record(
            "VideoAsset",
            vec![
                property("VideoAsset", "width", FixtureValue::Double(64.0)),
                property("VideoAsset", "height", FixtureValue::Double(32.0)),
            ],
        ),
        record(
            "ImageAsset",
            vec![
                property("ImageAsset", "width", FixtureValue::Double(64.0)),
                property("ImageAsset", "height", FixtureValue::Double(32.0)),
            ],
        ),
        record(
            "FileAssetContents",
            vec![property(
                "FileAssetContents",
                "bytes",
                FixtureValue::Bytes(png.clone()),
            )],
        ),
        record(
            "Artboard",
            vec![
                property("Artboard", "width", FixtureValue::Double(64.0)),
                property("Artboard", "height", FixtureValue::Double(32.0)),
            ],
        ),
        record(
            "Video",
            vec![
                property("Video", "parentId", FixtureValue::Uint(0)),
                property("Video", "assetId", FixtureValue::Uint(0)),
                property("Video", "posterAssetId", FixtureValue::Uint(1)),
            ],
        ),
    ];
    let bytes = encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(|f| f.artboard_default()).unwrap();
    artboard.update_pass(true);
    let video = artboard
        .with_artboard(|a| {
            a.objects()
                .iter()
                .flatten()
                .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                .cloned()
        })
        .unwrap();
    assert!(video.with(|v| v.drawable_will_draw()).unwrap());
    let frame: Rc<dyn nuxie_render_api::RenderImage> =
        Rc::from(factory.decode_image(&png).unwrap());
    video
        .with_downcast_mut::<Video, _>(|v| {
            let poster = v.render_image().expect("authored poster snapshot");
            assert!(v.present(0, frame.clone(), 0.0));
            assert!(Rc::ptr_eq(&frame, &v.render_image().unwrap()));
            assert!(!Rc::ptr_eq(&poster, &frame));
            v.clear_frame();
            assert!(Rc::ptr_eq(&poster, &v.render_image().unwrap()));
        })
        .unwrap();
    assert!(video.with(|v| v.drawable_will_draw()).unwrap());
}

#[test]
fn first_frame_timeout_is_bounded_and_stale_frames_cannot_override_fallback() {
    use nuxie_runtime::video::readiness::*;
    let mut gate = FirstFrameGate::new(4, 10.0, 2.0, true, false).unwrap();
    assert_eq!(
        gate.evaluate(3, 11.0, true, false, true),
        Readiness::Waiting
    );
    assert_eq!(
        gate.evaluate(4, 11.9, false, false, true),
        Readiness::Waiting
    );
    assert_eq!(
        gate.evaluate(4, 12.0, false, false, true),
        Readiness::Poster
    );
    assert_eq!(gate.evaluate(4, 12.1, true, false, true), Readiness::Poster);
    let mut required = FirstFrameGate::new(4, 0.0, 1.0, true, false).unwrap();
    assert_eq!(
        required.evaluate(4, 1.0, false, false, false),
        Readiness::Unavailable
    );
    assert!(FirstFrameGate::new(0, 0.0, f64::INFINITY, true, false).is_none());
}

#[test]
fn serialized_captions_roundtrip_import_and_follow_independent_instance_clocks() {
    use nuxie_runtime::video::captions::*;
    let track = CaptionTrack::new(
        "en".into(),
        vec![
            Cue {
                start: 0.0,
                end: 1.0,
                text: "Hello 👋".into(),
            },
            Cue {
                start: 1.0,
                end: 2.0,
                text: "Welcome back".into(),
            },
        ],
    )
    .unwrap();
    let json = track.to_json().unwrap();
    assert_eq!(
        CaptionTrack::from_json(&json).unwrap().text(0.0),
        "Hello 👋"
    );
    let mut decoded = nuxie_binary::read_runtime_file(&scene(false)).unwrap();
    decoded
        .objects
        .iter_mut()
        .flatten()
        .find(|o| o.type_key == Video::TYPE_KEY)
        .unwrap()
        .properties
        .push(nuxie_binary::RuntimeProperty {
            key: 60013,
            name: "captions",
            owner: "Video",
            value: nuxie_binary::FieldValue::String(nuxie_binary::StringValue {
                raw: json.as_bytes().to_vec(),
                value: Some(json),
            }),
        });
    let bytes = encode_runtime_file(&decoded).unwrap();
    assert_eq!(
        encode_runtime_file(&nuxie_binary::read_runtime_file(&bytes).unwrap()).unwrap(),
        bytes
    );
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let first_artboard = file.with_file(|f| f.artboard_default()).unwrap();
    let second_artboard = file.with_file(|f| f.artboard_default()).unwrap();
    let find = |artboard: &nuxie_runtime::RuntimeArtboardInstanceHandle| {
        artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                    .cloned()
            })
            .unwrap()
    };
    let first = find(&first_artboard);
    let second = find(&second_artboard);
    first
        .with_downcast_mut::<Video, _>(|v| {
            assert_eq!(v.caption_language(), "en");
            assert_eq!(v.caption_text(), "Hello 👋");
            v.playback.enqueue(Command::Seek(1.0)).unwrap();
            v.playback.drain_actions();
            assert_eq!(v.caption_text(), "Welcome back");
        })
        .unwrap();
    assert_eq!(
        second
            .with_downcast::<Video, _>(|v| v.caption_text())
            .unwrap(),
        "Hello 👋"
    );
}

#[test]
fn invalid_serialized_caption_tracks_fail_scene_import() {
    for json in [
        r#"{"version":2,"language":"en","cues":[]}"#,
        r#"{"version":1,"language":"en","cues":[{"start":2,"end":1,"text":"bad"}]}"#,
        r#"{"version":1,"language":"en","cues":[],"html":true}"#,
        "not json",
    ] {
        let mut decoded = nuxie_binary::read_runtime_file(&scene(false)).unwrap();
        decoded
            .objects
            .iter_mut()
            .flatten()
            .find(|o| o.type_key == Video::TYPE_KEY)
            .unwrap()
            .properties
            .push(nuxie_binary::RuntimeProperty {
                key: 60013,
                name: "captions",
                owner: "Video",
                value: nuxie_binary::FieldValue::String(nuxie_binary::StringValue {
                    raw: json.as_bytes().to_vec(),
                    value: Some(json.into()),
                }),
            });
        let bytes = encode_runtime_file(&decoded).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        assert!(
            File::import(
                &bytes,
                RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
                None,
                None,
                None
            )
            .is_none(),
            "accepted invalid caption track: {json}"
        );
    }
}

#[test]
fn decoder_reclamation_preserves_intent_and_restores_seek_before_play() {
    let mut playback = Playback::new(PlaybackSettings {
        autoplay: true,
        ..Default::default()
    });
    playback.opened(0, 10.0);
    assert!(playback.accept_frame(0, 3.0));
    playback.update_suspension(SuspensionReason::Resources, true);
    let generation = playback.reclaim_decoder().unwrap();
    assert!(!playback.accept_frame(0, 4.0));
    assert!(playback.wants_play());
    assert_eq!(playback.position(), 3.0);
    playback.enqueue(Command::Seek(5.0)).unwrap();
    playback.drain_actions();
    assert!(playback.generation() > generation);
    playback.update_suspension(SuspensionReason::Resources, false);
    let actions = playback.opened(playback.generation(), 10.0);
    assert_eq!(
        &actions[2..],
        &[
            DecoderAction::Seek {
                seconds: 5.0,
                generation: playback.generation()
            },
            DecoderAction::Play,
        ]
    );
}

#[test]
fn looping_range_excludes_end_rejects_stale_callbacks_and_obeys_suspension() {
    let mut playback = Playback::new(PlaybackSettings {
        autoplay: true,
        looping: true,
        loop_start: 1.0,
        loop_end: 2.0,
        ..Default::default()
    });
    let actions = playback.opened(0, 5.0);
    assert!(actions.contains(&DecoderAction::Seek {
        seconds: 1.0,
        generation: 0
    }));
    assert!(!playback.accept_frame(0, 0.5));
    assert!(playback.accept_frame(0, 1.0));
    assert!(playback.accept_frame(0, 1.999));
    assert!(!playback.accept_frame(0, 2.0));
    let generation = playback.generation();
    assert!(generation > 0);
    assert!(!playback.accept_frame(0, 1.5));
    assert!(playback.ended(0).is_empty());
    playback.enqueue(Command::Suspend(true)).unwrap();
    let actions = playback.drain_actions();
    assert!(actions.contains(&DecoderAction::Seek {
        seconds: 1.0,
        generation
    }));
    assert!(!actions.contains(&DecoderAction::Play));
    assert!(actions.contains(&DecoderAction::Pause));
    let mut loops = 0;
    while let Some(event) = playback.pop_event() {
        if event == PlaybackEvent::Looped {
            loops += 1;
        }
    }
    assert_eq!(loops, 1);
    assert!(
        playback
            .enqueue(Command::LoopRange {
                start: 2.0,
                end: 1.0
            })
            .is_err()
    );
    assert!(
        playback
            .enqueue(Command::LoopRange {
                start: f64::NAN,
                end: 0.0
            })
            .is_err()
    );
    assert!(
        playback
            .enqueue(Command::LoopRange {
                start: 6.0,
                end: 0.0
            })
            .is_err()
    );
    playback.enqueue(Command::Loop(false)).unwrap();
    playback.drain_actions();
    assert!(playback.accept_frame(generation, 3.0));
}

#[test]
fn authored_loop_range_is_clamped_to_duration_and_rejects_empty_source_interval() {
    let mut playback = Playback::new(PlaybackSettings {
        autoplay: true,
        looping: true,
        loop_start: 1.0,
        loop_end: 10.0,
        ..Default::default()
    });
    playback.opened(0, 3.0);
    assert!(playback.accept_frame(0, 2.9));
    assert!(!playback.accept_frame(0, 3.0));
    assert!(playback.drain_actions().contains(&DecoderAction::Seek {
        seconds: 1.0,
        generation: 1
    }));
    let mut invalid = Playback::new(PlaybackSettings {
        looping: true,
        loop_start: 5.0,
        ..Default::default()
    });
    assert_eq!(invalid.opened(0, 3.0), vec![DecoderAction::Dispose]);
    assert_eq!(invalid.state(), PlaybackState::Failed);
}

#[test]
fn loop_fields_roundtrip_and_invalid_authored_range_rejects_import() {
    for (start, end, valid) in [(1.0, 2.0, true), (1.0, 0.0, true), (2.0, 1.0, false)] {
        let mut file = nuxie_binary::read_runtime_file(&scene(false)).unwrap();
        let video = file
            .objects
            .iter_mut()
            .flatten()
            .find(|o| o.type_key == Video::TYPE_KEY)
            .unwrap();
        for (key, name, value) in [(60014, "loopStart", start), (60015, "loopEnd", end)] {
            video.properties.push(nuxie_binary::RuntimeProperty {
                key,
                name,
                owner: "Video",
                value: nuxie_binary::FieldValue::Double(value),
            });
        }
        let bytes = encode_runtime_file(&file).unwrap();
        assert_eq!(
            encode_runtime_file(&nuxie_binary::read_runtime_file(&bytes).unwrap()).unwrap(),
            bytes
        );
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        assert_eq!(
            File::import(
                &bytes,
                RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
                None,
                None,
                None
            )
            .is_some(),
            valid
        );
    }
}

#[test]
fn cropped_video_import_draws_its_authored_uv_mesh() {
    for mesh_offset in [0.0, 100.0] {
        let mut records = scene_records(false);
        records.push(record(
            "Mesh",
            vec![
                property("Mesh", "parentId", FixtureValue::Uint(1)),
                property(
                    "Mesh",
                    "triangleIndexBytes",
                    FixtureValue::Bytes(vec![0, 1, 2, 0, 2, 3]),
                ),
            ],
        ));
        for (x, y, u, v) in [
            (0.0, 0.0, 0.5, 0.0),
            (1.0, 0.0, 1.0, 0.0),
            (1.0, 1.0, 1.0, 1.0),
            (0.0, 1.0, 0.5, 1.0),
        ] {
            records.push(record(
                "MeshVertex",
                vec![
                    property("MeshVertex", "parentId", FixtureValue::Uint(2)),
                    property("MeshVertex", "x", FixtureValue::Double(x + mesh_offset)),
                    property("MeshVertex", "y", FixtureValue::Double(y)),
                    property("MeshVertex", "u", FixtureValue::Double(u)),
                    property("MeshVertex", "v", FixtureValue::Double(v)),
                ],
            ));
        }
        let bytes =
            encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        artboard.update_pass(true);
        let video = artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                    .cloned()
            })
            .unwrap();
        use nuxie_runtime::source::semantic::semantic_snapshot::Bounds;
        use nuxie_runtime::video::visibility::is_visible_in;
        let viewport = Bounds {
            min_x: 80.2 + mesh_offset,
            min_y: 0.2,
            max_x: 80.8 + mesh_offset,
            max_y: 0.8,
        };
        assert!(
            is_visible_in(&video, viewport).unwrap(),
            "Authored mesh is visible before decoding"
        );
        assert!(
            !is_visible_in(
                &video,
                Bounds {
                    min_x: 90.0,
                    min_y: 0.2,
                    max_x: 91.0,
                    max_y: 0.8
                }
            )
            .unwrap(),
            "Nominal video rectangle must not create decoder demand outside mesh triangles"
        );
        let mut png = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png, 64, 32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&vec![255; 64 * 32 * 4])
                .unwrap();
        }
        let image = Rc::from(factory.decode_image(&png).unwrap());
        video
            .with_downcast_mut::<Video, _>(|v| assert!(v.present(0, image, 0.0)))
            .unwrap();
        artboard.update_pass(true);
        let mut renderer = factory.borrow().make_renderer();
        artboard.draw(&mut renderer);
        let stream = factory.borrow().canonical_recording().stream().to_string();
        let mesh = stream
            .lines()
            .find(|line| line.starts_with("drawImageMesh "))
            .unwrap_or_else(|| panic!("cropped video must draw a mesh: {stream}"));
        // Independent authored oracle: right-half UVs and two complete triangles.
        assert!(
            mesh.contains("data=0000003f000000000000803f000000000000803f0000803f0000003f0000803f}"),
            "{mesh}"
        );
        assert!(mesh.contains("vertexCount=4 indexCount=6"), "{mesh}");
        assert!(!mesh.contains("indices=0"), "{mesh}");
    }
}

#[test]
fn video_visibility_is_available_before_decoding_and_respects_viewport_and_opacity() {
    use nuxie_runtime::source::semantic::semantic_snapshot::Bounds;
    use nuxie_runtime::video::visibility::is_visible_in;
    for (opacity, expected) in [(1.0, true), (0.0, false)] {
        let mut records = scene_records(false);
        records.last_mut().unwrap().properties.extend([
            property("Video", "y", FixtureValue::Double(80.0)),
            property("Video", "opacity", FixtureValue::Double(opacity)),
        ]);
        let bytes =
            encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        artboard.update_pass(true);
        let video = artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                    .cloned()
            })
            .unwrap();
        let viewport = Bounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 200.0,
            max_y: 200.0,
        };
        assert_eq!(is_visible_in(&video, viewport).unwrap(), expected);
        // This video occupies x=48..112, y=64..96 with its default center origin.
        assert!(
            !is_visible_in(
                &video,
                Bounds {
                    min_x: 120.0,
                    ..viewport
                }
            )
            .unwrap()
        );
        assert_eq!(
            is_visible_in(
                &video,
                Bounds {
                    min_x: 100.0,
                    ..viewport
                }
            )
            .unwrap(),
            expected
        );
        assert!(
            is_visible_in(
                &video,
                Bounds {
                    max_x: f32::NAN,
                    ..viewport
                }
            )
            .is_err()
        );
        assert!(
            !is_visible_in(
                &video,
                Bounds {
                    max_x: 0.0,
                    ..viewport
                }
            )
            .unwrap()
        );
    }
}

#[test]
fn video_visibility_respects_artboard_clipping_before_decoding() {
    use nuxie_runtime::source::semantic::semantic_snapshot::Bounds;
    use nuxie_runtime::video::visibility::is_visible_in;
    for clip in [false, true] {
        let mut records = scene_records(false);
        records[2]
            .properties
            .push(property("Artboard", "clip", FixtureValue::Bool(clip)));
        let x = property("Video", "x", FixtureValue::Double(300.0));
        let video = records.last_mut().unwrap();
        video.properties.retain(|p| p.key != x.key);
        video
            .properties
            .extend([x, property("Video", "y", FixtureValue::Double(80.0))]);
        let bytes =
            encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        artboard.update_pass(true);
        let video = artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                    .cloned()
            })
            .unwrap();
        // The wider host viewport includes the video, but the authored clip does not.
        let viewport = Bounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: 400.0,
            max_y: 200.0,
        };
        assert_eq!(is_visible_in(&video, viewport).unwrap(), !clip);
    }
}

#[test]
fn video_visibility_intersects_the_transformed_polygon_not_its_bounding_box() {
    use nuxie_runtime::source::semantic::semantic_snapshot::Bounds;
    use nuxie_runtime::video::visibility::is_visible_in;
    let mut records = scene_records(false);
    records.last_mut().unwrap().properties.extend([
        property("Video", "y", FixtureValue::Double(80.0)),
        property(
            "Video",
            "rotation",
            FixtureValue::Double(std::f32::consts::FRAC_PI_4),
        ),
    ]);
    let bytes = encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(|f| f.artboard_default()).unwrap();
    artboard.update_pass(true);
    let video = artboard
        .with_artboard(|a| {
            a.objects()
                .iter()
                .flatten()
                .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                .cloned()
        })
        .unwrap();
    // A 64x32 centered rectangle rotated 45 degrees has bounds approximately
    // 46..114 on both axes, but its upper-left bounding-box corner is empty.
    assert!(
        !is_visible_in(
            &video,
            Bounds {
                min_x: 47.0,
                min_y: 47.0,
                max_x: 49.0,
                max_y: 49.0
            }
        )
        .unwrap()
    );
    assert!(
        is_visible_in(
            &video,
            Bounds {
                min_x: 79.0,
                min_y: 79.0,
                max_x: 81.0,
                max_y: 81.0
            }
        )
        .unwrap()
    );
}

#[test]
fn video_layout_uses_intrinsic_dimensions_before_decode() {
    use nuxie_runtime::source::{
        layout::layout_enums::{LayoutDirection, LayoutScaleType},
        math::vec2d::Vec2D,
    };
    let bytes = scene(false);
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(|f| f.artboard_default()).unwrap();
    artboard.update_pass(true);
    let video = artboard
        .with_artboard(|a| {
            a.objects()
                .iter()
                .flatten()
                .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                .cloned()
        })
        .unwrap();
    video.with_mut(|object| {
        object
            .as_intrinsically_sizeable_mut()
            .unwrap()
            .control_size(
                Vec2D::new(128.0, 64.0),
                LayoutScaleType::Fixed,
                LayoutScaleType::Fixed,
                LayoutDirection::Ltr,
            );
    });
    artboard.update_pass(true);
    video.with(|object| {
        let world = object.as_transform_component().unwrap().world_transform();
        assert_eq!(
            world[0], 2.0,
            "128-wide layout scales the undecoded 64-wide asset"
        );
        assert_eq!(
            world[3], 2.0,
            "64-high layout scales the undecoded 32-high asset"
        );
    });
}

#[test]
fn video_visibility_uses_curved_host_clip_before_decoding() {
    use nuxie_runtime::source::{math::raw_path::RawPath, semantic::semantic_snapshot::Bounds};
    use nuxie_runtime::video::visibility::{is_visible_in, is_visible_in_host_clip};
    let viewport = Bounds {
        min_x: 0.0,
        min_y: 0.0,
        max_x: 200.0,
        max_y: 200.0,
    };
    // A circular 200x200 bezel: both candidates intersect its bounding box,
    // but only the second reaches the curved visible region.
    let mut clip = RawPath::default();
    let curve = 100.0 * 0.552_284_8;
    clip.move_to(100.0, 0.0);
    clip.cubic_to(100.0 + curve, 0.0, 200.0, 100.0 - curve, 200.0, 100.0);
    clip.cubic_to(200.0, 100.0 + curve, 100.0 + curve, 200.0, 100.0, 200.0);
    clip.cubic_to(100.0 - curve, 200.0, 0.0, 100.0 + curve, 0.0, 100.0);
    clip.cubic_to(0.0, 100.0 - curve, 100.0 - curve, 0.0, 100.0, 0.0);
    clip.close();
    for (x, expected) in [(5.0, false), (10.0, true)] {
        let mut records = scene_records(false);
        let x_key = property("Video", "x", FixtureValue::Double(x)).key;
        records
            .last_mut()
            .unwrap()
            .properties
            .retain(|property| property.key != x_key);
        records.last_mut().unwrap().properties.extend([
            property("Video", "x", FixtureValue::Double(x)),
            property("Video", "y", FixtureValue::Double(5.0)),
        ]);
        let bytes =
            encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        artboard.update_pass(true);
        let video = artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                    .cloned()
            })
            .unwrap();
        assert!(is_visible_in(&video, viewport).unwrap());
        assert_eq!(
            is_visible_in_host_clip(
                &video,
                viewport,
                Some((&clip, nuxie_render_api::FillRule::NonZero))
            )
            .unwrap(),
            expected
        );
        let mut invalid = RawPath::default();
        invalid.move_to(f32::NAN, 0.0);
        invalid.line_to(200.0, 200.0);
        invalid.close();
        assert!(
            is_visible_in_host_clip(
                &video,
                viewport,
                Some((&invalid, nuxie_render_api::FillRule::NonZero))
            )
            .is_err()
        );
    }
}

#[test]
fn preview_duration_tracks_only_current_source_metadata() {
    let mut playback = Playback::default();
    assert_eq!(playback.duration(), None);
    playback.opened(1, 9.0); // Stale decoder observation.
    playback.opened(0, f64::NAN);
    assert_eq!(playback.duration(), None);
    playback.opened(0, 2.5);
    assert_eq!(playback.duration(), Some(2.5));
    let generation = playback
        .replace_source(PlaybackSettings::default())
        .unwrap();
    assert_eq!(playback.duration(), None);
    playback.opened(0, 2.5);
    assert_eq!(playback.duration(), None);
    playback.opened(generation, 7.0);
    assert_eq!(playback.duration(), Some(7.0));
}
