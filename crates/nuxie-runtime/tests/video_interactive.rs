use nuxie_runtime::video::playback::{
    Command, DecoderAction, Playback, PlaybackError, RequestState,
};

fn opened() -> Playback {
    let mut playback = Playback::default();
    playback.opened(0, 10.0);
    playback
}

#[test]
fn finite_range_holds_last_in_range_frame_and_rejects_late_delivery() {
    let mut p = opened();
    let id = p.play_range(2.0, 4.0).unwrap();
    let actions = p.drain_actions();
    assert!(actions.contains(&DecoderAction::Play));
    let generation = p.generation();
    assert!(!p.accept_frame(generation, 1.9));
    assert!(p.accept_frame(generation, 2.0));
    assert!(p.accept_frame(generation, 3.9));
    assert!(!p.accept_frame(generation, 4.0));
    assert_eq!(p.position(), 3.9);
    assert_eq!(p.request_status().unwrap().id, id);
    assert_eq!(p.request_status().unwrap().state, RequestState::Completed);
    assert_eq!(p.drain_actions(), vec![DecoderAction::Pause]);
    assert!(!p.accept_frame(generation, 4.1));
    assert!(!p.accept_frame(generation, 3.8));
}

#[test]
fn range_accepts_first_valid_frame_delayed_by_decoder_load() {
    let mut p = opened();
    p.play_range(2.0, 4.0).unwrap();
    p.drain_actions();
    let generation = p.generation();
    assert!(!p.accept_frame(generation, 1.9));
    assert!(p.accept_frame(generation, 2.067));
    assert_eq!(p.request_status().unwrap().state, RequestState::Playing);
    assert!(p.accept_frame(generation, 3.9));
    assert!(!p.accept_frame(generation, 4.0));
    assert_eq!(p.request_status().unwrap().state, RequestState::Completed);
    assert_eq!(p.position(), 3.9);
}

#[test]
fn scrub_burst_is_one_latest_seek_and_one_settled_frame() {
    let mut p = opened();
    let mut id = 0;
    for index in 0..1000 {
        id = p.scrub(index as f64 / 1000.0, 0.0, 10.0).unwrap();
    }
    let actions = p.drain_actions();
    assert_eq!(
        actions
            .iter()
            .filter(|a| matches!(a, DecoderAction::Seek { .. }))
            .count(),
        1
    );
    assert_eq!(p.position(), 9.99);
    assert!(!p.accept_frame(p.generation() - 1, 1.0));
    assert!(p.accept_frame(p.generation(), 9.99));
    assert_eq!(p.request_status().unwrap().id, id);
    assert_eq!(p.request_status().unwrap().state, RequestState::Settled);
    assert!(!p.accept_frame(p.generation(), 10.0));
}

#[test]
fn supersession_does_not_adopt_old_frame_before_drain() {
    let mut p = opened();
    p.play_range(0.0, 3.0).unwrap();
    p.drain_actions();
    let old = p.generation();
    let id = p.play_range(4.0, 8.0).unwrap();
    assert!(!p.accept_frame(old, 2.0));
    p.drain_actions();
    assert!(!p.accept_frame(old, 2.0));
    assert!(p.accept_frame(p.generation(), 4.0));
    assert_eq!(p.request_status().unwrap().id, id);
}

#[test]
fn invalid_requests_are_atomic_and_ordinary_commands_cancel_ownership() {
    let mut p = opened();
    let id = p.play_range(2.0, 3.0).unwrap();
    assert_eq!(
        p.scrub(f64::NAN, 0.0, 2.0),
        Err(PlaybackError::InvalidValue)
    );
    assert_eq!(p.play_range(3.0, 11.0), Err(PlaybackError::InvalidValue));
    assert_eq!(p.request_status().unwrap().id, id);
    p.drain_actions();
    p.enqueue(Command::Seek(6.0)).unwrap();
    p.drain_actions();
    assert_eq!(p.request_status().unwrap().state, RequestState::Cancelled);
    assert!(p.accept_frame(p.generation(), 6.0));
}

#[test]
fn metadata_failure_and_decoder_failure_set_request_failed() {
    let mut p = Playback::default();
    p.play_range(3.0, 9.0).unwrap();
    p.drain_actions();
    p.opened(p.generation(), 5.0);
    assert_eq!(p.request_status().unwrap().state, RequestState::Failed);
    let mut p = opened();
    p.scrub(0.5, 0.0, 10.0).unwrap();
    p.drain_actions();
    p.failed(p.generation());
    assert_eq!(p.request_status().unwrap().state, RequestState::Failed);
}

#[test]
fn stale_end_before_drain_cannot_complete_new_request() {
    let mut p = opened();
    p.play_range(0.0, 2.0).unwrap();
    p.drain_actions();
    let old = p.generation();
    p.play_range(3.0, 4.0).unwrap();
    assert!(p.ended(old).is_empty());
    assert_eq!(p.request_status().unwrap().state, RequestState::Pending);
}

#[test]
fn completed_range_can_restore_held_image_after_decoder_reclamation() {
    let mut p = opened();
    p.play_range(2.0, 4.0).unwrap();
    p.drain_actions();
    assert!(p.accept_frame(p.generation(), 2.0));
    assert!(p.accept_frame(p.generation(), 3.9));
    assert!(!p.accept_frame(p.generation(), 4.0));
    p.drain_actions();
    let generation = p.reclaim_decoder().unwrap();
    p.opened(generation, 10.0);
    assert!(p.accept_frame(generation, 3.9));
    assert_eq!(p.request_status().unwrap().state, RequestState::Completed);
    assert!(!p.accept_frame(generation, 4.1));
}

#[test]
fn cancellation_is_request_owned_and_does_not_pause_replacement() {
    let mut p = opened();
    let first = p.play_range(0.0, 2.0).unwrap();
    let second = p.play_range(2.0, 4.0).unwrap();
    assert!(!p.cancel_request(first).unwrap());
    assert_eq!(p.request_status().unwrap().id, second);
    assert!(p.cancel_request(second).unwrap());
    assert!(
        !p.drain_actions()
            .iter()
            .any(|action| matches!(action, DecoderAction::Seek { .. } | DecoderAction::Play))
    );
    assert_eq!(p.request_status().unwrap().state, RequestState::Cancelled);
}

#[test]
fn interactive_override_does_not_mutate_authored_loop_settings() {
    let mut p = Playback::new(nuxie_runtime::video::playback::PlaybackSettings {
        looping: true,
        loop_start: 0.0,
        loop_end: 2.0,
        ..Default::default()
    });
    p.opened(0, 10.0);
    p.play_range(4.0, 6.0).unwrap();
    p.drain_actions();
    assert_eq!(p.position(), 4.0);
    assert!(p.settings().looping);
    assert!(p.accept_frame(p.generation(), 4.0));
    assert!(p.accept_frame(p.generation(), 5.0));
    p.enqueue(Command::Seek(8.0)).unwrap();
    p.drain_actions();
    assert_eq!(p.position(), 0.0);
}

#[test]
fn drawable_retains_actual_in_range_image_on_boundary_and_stale_delivery() {
    use std::rc::Rc;
    #[derive(Clone)]
    struct Image;
    impl nuxie_render_api::RenderImage for Image {
        fn retain_image(&self) -> Rc<dyn nuxie_render_api::RenderImage> {
            Rc::new(self.clone())
        }
        fn image_identity(&self) -> usize {
            1
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn width(&self) -> u32 {
            2
        }
        fn height(&self) -> u32 {
            2
        }
        fn uv_transform(&self) -> nuxie_render_api::Mat2D {
            nuxie_render_api::Mat2D::IDENTITY
        }
    }
    let mut video = nuxie_runtime::video::Video::default();
    video.playback.opened(0, 10.0);
    video.playback.play_range(2.0, 4.0).unwrap();
    video.playback.drain_actions();
    let frame: Rc<dyn nuxie_render_api::RenderImage> = Rc::new(Image);
    let other: Rc<dyn nuxie_render_api::RenderImage> = Rc::new(Image);
    let generation = video.playback.generation();
    assert!(video.present(generation, frame.clone(), 2.0));
    assert!(video.present(generation, frame.clone(), 3.9));
    assert!(!video.present(generation, other.clone(), 4.0));
    assert!(Rc::ptr_eq(&frame, &video.render_image().unwrap()));
    video.playback.play_range(5.0, 8.0).unwrap();
    assert!(!video.present(generation, other, 6.0));
    assert!(Rc::ptr_eq(&frame, &video.render_image().unwrap()));
}

#[test]
fn lifecycle_veto_and_source_replacement_preserve_request_identity_safety() {
    use nuxie_runtime::video::playback::{PlaybackSettings, SuspensionReason};
    let mut p = opened();
    p.update_suspension(SuspensionReason::Background, true);
    let first = p.play_range(0.0, 3.0).unwrap();
    assert!(!p.drain_actions().contains(&DecoderAction::Play));
    assert!(p.wants_play());
    assert_eq!(
        p.update_suspension(SuspensionReason::Background, false),
        vec![DecoderAction::Play]
    );
    let generation = p.replace_source(PlaybackSettings::default()).unwrap();
    assert!(p.request_status().is_none());
    assert!(p.duration().is_none());
    p.opened(generation, 10.0);
    let second = p.play_range(0.0, 3.0).unwrap();
    assert!(second > first);
    assert!(!p.cancel_request(first).unwrap());
}

#[test]
fn scrub_does_not_settle_on_same_generation_preroll() {
    let mut p = opened();
    p.scrub(0.8, 0.0, 1.0).unwrap();
    p.drain_actions();
    let generation = p.generation();
    assert!(!p.accept_frame(generation, 0.1));
    assert_eq!(p.request_status().unwrap().state, RequestState::Pending);
    assert!(p.accept_frame(generation, 0.8));
    assert_eq!(p.request_status().unwrap().state, RequestState::Settled);
}

#[test]
fn no_presented_range_frame_is_failure_not_successful_hold() {
    let mut p = opened();
    p.play_range(3.0, 3.01).unwrap();
    p.drain_actions();
    assert!(!p.accept_frame(p.generation(), 3.02));
    assert_eq!(p.request_status().unwrap().state, RequestState::Failed);
    assert_eq!(p.drain_actions(), vec![DecoderAction::Pause]);
    assert!(!p.accept_frame(p.generation(), 3.005));
    let mut p = opened();
    p.play_range(3.0, 10.0).unwrap();
    p.drain_actions();
    assert_eq!(p.ended(p.generation()), vec![DecoderAction::Pause]);
    assert_eq!(p.request_status().unwrap().state, RequestState::Failed);
}

#[test]
fn reclaim_during_active_range_does_not_leave_terminal_hold_rewindable() {
    let mut p = opened();
    p.play_range(2.0, 4.0).unwrap();
    p.drain_actions();
    assert!(p.accept_frame(p.generation(), 2.0));
    let generation = p.reclaim_decoder().unwrap();
    p.opened(generation, 10.0);
    assert!(p.accept_frame(generation, 2.0));
    assert!(p.accept_frame(generation, 3.9));
    assert!(!p.accept_frame(generation, 4.0));
    assert!(!p.accept_frame(generation, 3.89));
    assert_eq!(p.position(), 3.9);
}

#[test]
fn repeated_unchanged_scrub_does_not_restart_decoder_seek() {
    let mut p = opened();
    let id = p.scrub(0.5, 0.0, 10.0).unwrap();
    p.drain_actions();
    let generation = p.generation();
    assert_eq!(p.scrub(0.5, 0.0, 10.0).unwrap(), id);
    assert!(p.drain_actions().is_empty());
    assert_eq!(p.generation(), generation);
    assert!(p.accept_frame(generation, 5.0));
    assert_eq!(p.scrub(0.5, 0.0, 10.0).unwrap(), id);
    assert!(p.drain_actions().is_empty());
}

fn endpoint_scrub() -> Playback {
    let mut p = Playback::default();
    p.opened(0, 2.022);
    p.scrub(1.0, 0.0, 2.022).unwrap();
    p.drain_actions();
    p
}

#[test]
fn endpoint_audio_tail_requires_exact_selected_seek_frame_receipt() {
    let mut p = endpoint_scrub();
    let pts = 59.0 / 30.0;
    let generation = p.generation();
    assert!(!p.accept_frame(generation, pts));
    assert_eq!(p.request_status().unwrap().state, RequestState::Pending);
    assert!(p.observe_selected_seek_frame(generation, pts));
    assert!(p.accept_frame(generation, pts));
    assert_eq!(p.position(), pts);
    assert_eq!(p.request_status().unwrap().state, RequestState::Settled);
    assert!(!p.observe_selected_seek_frame(generation, pts));
    assert!(!p.accept_frame(generation, pts));
}

#[test]
fn selected_seek_receipt_is_consumed_by_mismatch_and_rejects_stale_owners() {
    let mut p = endpoint_scrub();
    let pts = 59.0 / 30.0;
    let generation = p.generation();
    assert!(!p.observe_selected_seek_frame(generation - 1, pts));
    assert!(!p.accept_frame(generation, pts));
    assert!(p.observe_selected_seek_frame(generation, pts));
    assert!(!p.accept_frame(generation, pts - 0.1));
    assert!(!p.accept_frame(generation, pts));
    assert!(p.observe_selected_seek_frame(generation, pts));
    p.scrub(0.5, 0.0, 2.022).unwrap();
    assert!(!p.observe_selected_seek_frame(generation, pts));
    assert!(!p.accept_frame(generation, pts));
    p.drain_actions();
    assert!(!p.observe_selected_seek_frame(generation, pts));
    assert!(!p.accept_frame(p.generation(), pts));
}

#[test]
fn selected_seek_receipt_cannot_relax_interior_scrub_or_invalid_frames() {
    let mut p = endpoint_scrub();
    for pts in [f64::NAN, f64::INFINITY, -1.0, 2.1] {
        assert!(!p.observe_selected_seek_frame(p.generation(), pts));
    }
    p.scrub(1.0, 0.0, 1.0).unwrap();
    p.drain_actions();
    assert!(!p.observe_selected_seek_frame(p.generation(), 0.8));
    assert!(!p.accept_frame(p.generation(), 0.8));
    assert!(p.accept_frame(p.generation(), 1.0));
}
