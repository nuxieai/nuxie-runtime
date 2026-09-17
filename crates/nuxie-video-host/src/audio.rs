//! Native interruption edges can arrive while rendering is stopped.
use nuxie_runtime::video::playback::{DecoderAction, Playback, SuspensionReason};

pub(crate) fn reconcile_interruption(
    playback: &mut Playback,
    active: bool,
    ended: bool,
) -> Vec<DecoderAction> {
    let mut actions = Vec::new();
    if ended {
        actions.extend(playback.update_suspension(SuspensionReason::Interruption, true));
    }
    actions.extend(playback.update_suspension(SuspensionReason::Interruption, active));
    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_runtime::video::playback::{Command, PlaybackSettings};

    fn playing() -> Playback {
        let mut p = Playback::new(PlaybackSettings {
            autoplay: true,
            ..Default::default()
        });
        p.opened(0, 10.0);
        p.drain_actions();
        p.observed_playing(0);
        p
    }
    #[test]
    fn complete_interruption_between_ticks_retries_requested_play() {
        let mut p = playing();
        assert_eq!(
            reconcile_interruption(&mut p, false, true),
            vec![DecoderAction::Pause, DecoderAction::Play]
        );
        assert!(p.wants_play());
        assert!(reconcile_interruption(&mut p, false, false).is_empty());
    }
    #[test]
    fn recovery_preserves_other_vetoes_and_explicit_pause() {
        let mut p = playing();
        p.update_suspension(SuspensionReason::Background, true);
        assert!(reconcile_interruption(&mut p, false, true).is_empty());
        assert_eq!(
            p.update_suspension(SuspensionReason::Background, false),
            vec![DecoderAction::Play]
        );
        p.enqueue(Command::Pause).unwrap();
        p.drain_actions();
        assert!(reconcile_interruption(&mut p, false, true).is_empty());
        assert!(!p.wants_play());
    }
    #[test]
    fn full_author_queue_cannot_prevent_native_interruption() {
        let mut p = playing();
        for _ in 0..256 {
            p.enqueue(Command::Play).unwrap();
        }
        assert_eq!(
            reconcile_interruption(&mut p, true, false),
            vec![DecoderAction::Pause]
        );
        assert!(!p.drain_actions().contains(&DecoderAction::Play));
        assert_eq!(
            reconcile_interruption(&mut p, false, true),
            vec![DecoderAction::Play]
        );
    }
}
