//! Explicit leader-clock synchronization. Independent players remain independent
//! unless a host creates a group. Hosts sample live media clocks on their owning
//! context. An asynchronously sampled native clock may be extrapolated over a
//! bounded freshness window; frame PTS or unbounded cached positions are invalid.
use crate::{
    source::core::CoreHandle,
    video::{
        Video,
        playback::{Command, PlaybackError, PlaybackState, SuspensionReason},
    },
};

/// Current host media time, sampled on its owning context or extrapolated from
/// a bounded fresh native timestamp. This is distinct from a decoded frame PTS.
#[derive(Debug, Clone, Copy)]
pub struct MediaClock {
    pub generation: u64,
    pub seconds: f64,
    pub rate: f64,
    pub playing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncError {
    InvalidPolicy,
    InvalidMember,
    DuplicateMember,
    InvalidTime,
    Command(PlaybackError),
}
#[derive(Debug, Clone, Copy)]
pub struct DriftMeasurement {
    /// Zero-based index in the group's followers, not a serialized scene ID.
    pub follower: usize,
    pub seconds: f64,
    pub corrected: bool,
    pub seek_prediction_seconds: f64,
}
pub struct SynchronizationGroup {
    leader: CoreHandle,
    followers: Vec<CoreHandle>,
    tolerance: f64,
    cooldown: f64,
    last_correction: Vec<Option<f64>>,
    last_update: Option<f64>,
    seek_compensation: Vec<f64>,
    pending_seek: Vec<Option<u64>>,
}
impl SynchronizationGroup {
    /// The leader supplies the timeline, normally the audible foreground video.
    /// Members keep their own decoders. This controller corrects follower seeks;
    /// group play/pause/rate use explicit commands; audio policy remains host-owned.
    /// Members must use the same playback rate. Rate-mismatched, suspended,
    /// seeking, stale-generation or unavailable clocks receive no correction.
    pub fn new(
        leader: CoreHandle,
        followers: Vec<CoreHandle>,
        tolerance: f64,
        cooldown: f64,
    ) -> Result<Self, SyncError> {
        if !tolerance.is_finite() || tolerance <= 0.0 || !cooldown.is_finite() || cooldown <= 0.0 {
            return Err(SyncError::InvalidPolicy);
        }
        if followers.is_empty() || followers.len() > 63 {
            return Err(SyncError::InvalidMember);
        }
        let mut identities = std::collections::BTreeSet::new();
        for member in std::iter::once(&leader).chain(&followers) {
            if member.with_downcast::<Video, _>(|_| ()).is_none() {
                return Err(SyncError::InvalidMember);
            }
            if !identities.insert(member.identity_key()) {
                return Err(SyncError::DuplicateMember);
            }
        }
        let last_correction = vec![None; followers.len()];
        let seek_compensation = vec![0.0; followers.len()];
        let pending_seek = vec![None; followers.len()];
        Ok(Self {
            leader,
            followers,
            tolerance,
            cooldown,
            last_correction,
            last_update: None,
            seek_compensation,
            pending_seek,
        })
    }
    /// Queue an authored command for the entire group, or change no queues.
    /// Lifecycle callbacks still use each managed player's immediate suspension
    /// API, because rendering may stop before an authored command is drained.
    pub fn command(&mut self, command: Command) -> Result<(), SyncError> {
        for member in std::iter::once(&self.leader).chain(&self.followers) {
            member
                .with_downcast::<Video, _>(|v| v.playback.can_enqueue(command))
                .ok_or(SyncError::InvalidMember)?
                .map_err(SyncError::Command)?;
        }
        // No external callbacks or async work occur between these two phases.
        for member in std::iter::once(&self.leader).chain(&self.followers) {
            member
                .with_downcast_mut::<Video, _>(|v| v.playback.enqueue(command))
                .ok_or(SyncError::InvalidMember)?
                .map_err(SyncError::Command)?;
        }
        self.pending_seek.fill(None);
        self.last_correction.fill(None);
        Ok(())
    }
    /// Call after host command/observation processing. `now` is a monotonic
    /// seconds clock. The callback returns each decoder's current media clock,
    /// sampled directly or extrapolated from a bounded fresh native timestamp.
    /// It returns None while opening/seeking or without a reliable clock.
    /// Corrections enter the same ordered runtime queue as Luau/Journey commands.
    pub fn correct(
        &mut self,
        now: f64,
        mut clock: impl FnMut(&CoreHandle) -> Option<MediaClock>,
    ) -> Result<Vec<DriftMeasurement>, SyncError> {
        if !now.is_finite() || now < 0.0 || self.last_update.is_some_and(|previous| now < previous)
        {
            return Err(SyncError::InvalidTime);
        }
        self.last_update = Some(now);
        let Some(leader) = usable_clock(&self.leader, &mut clock)? else {
            return Ok(Vec::new());
        };
        let mut measurements = Vec::new();
        let mut corrections = Vec::new();
        for (index, member) in self.followers.iter().enumerate() {
            let Some(follower) = usable_clock(member, &mut clock)? else {
                continue;
            };
            if (leader.rate - follower.rate).abs() > 0.001 {
                continue;
            }
            let drift = follower.seconds - leader.seconds;
            let correct = drift.abs() > self.tolerance
                && self.last_correction[index]
                    .is_none_or(|previous| now - previous >= self.cooldown);
            let mut compensation = self.seek_compensation[index];
            if correct {
                // The first post-seek sample can precede platform preroll:
                // media time may hold while the native player reports playing.
                // Learn residual delay at the next correction opportunity, after
                // the cooldown, instead of treating that first sample as settled.
                if self.pending_seek[index] == Some(follower.generation) {
                    compensation = (compensation - drift / leader.rate).clamp(0.0, 1.0);
                }
                corrections.push((
                    index,
                    Command::Seek(leader.seconds + compensation * leader.rate),
                    compensation,
                    follower.generation.checked_add(1),
                ));
            }
            measurements.push(DriftMeasurement {
                follower: index,
                seconds: drift,
                corrected: correct,
                seek_prediction_seconds: compensation,
            });
        }
        // Validate every correction before changing any queue or predictor.
        // All host clock callbacks have finished before this atomic phase.
        for (index, command, _, _) in &corrections {
            self.followers[*index]
                .with_downcast::<Video, _>(|v| v.playback.can_enqueue(*command))
                .ok_or(SyncError::InvalidMember)?
                .map_err(SyncError::Command)?;
        }
        for (index, command, compensation, generation) in corrections {
            self.followers[index]
                .with_downcast_mut::<Video, _>(|v| v.playback.enqueue(command))
                .ok_or(SyncError::InvalidMember)?
                .map_err(SyncError::Command)?;
            self.last_correction[index] = Some(now);
            self.seek_compensation[index] = compensation;
            self.pending_seek[index] = generation;
        }
        Ok(measurements)
    }
}
fn usable_clock(
    member: &CoreHandle,
    clock: &mut impl FnMut(&CoreHandle) -> Option<MediaClock>,
) -> Result<Option<MediaClock>, SyncError> {
    let state = member
        .with_downcast::<Video, _>(|video| {
            let playback = &video.playback;
            let suspended = [
                SuspensionReason::Hidden,
                SuspensionReason::Background,
                SuspensionReason::Interruption,
                SuspensionReason::Resources,
            ]
            .into_iter()
            .any(|reason| playback.is_suspended(reason));
            (
                playback.generation(),
                playback.wants_play()
                    && !suspended
                    && matches!(
                        playback.state(),
                        PlaybackState::Playing | PlaybackState::Ready
                    ),
            )
        })
        .ok_or(SyncError::InvalidMember)?;
    if !state.1 {
        return Ok(None);
    }
    Ok(clock(member).filter(|sample| {
        sample.generation == state.0
            && sample.playing
            && sample.seconds.is_finite()
            && sample.seconds >= 0.0
            && sample.rate.is_finite()
            && sample.rate > 0.0
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        source::core::CoreArena,
        video::playback::{DecoderAction, Playback, PlaybackSettings},
    };
    fn playing(arena: &CoreArena) -> CoreHandle {
        let mut video = Video::default();
        video.playback = Playback::new(PlaybackSettings {
            autoplay: true,
            ..Default::default()
        });
        video.playback.opened(0, 10.0);
        video.playback.observed_playing(0);
        arena.insert(video)
    }
    #[test]
    fn measured_drift_queues_a_seek_and_cooldown_prevents_seek_storms() {
        let arena = CoreArena::default();
        let leader = playing(&arena);
        let follower = playing(&arena);
        let mut group =
            SynchronizationGroup::new(leader.clone(), vec![follower.clone()], 0.05, 0.5).unwrap();
        let clock = |member: &CoreHandle| {
            Some(MediaClock {
                generation: member
                    .with_downcast::<Video, _>(|v| v.playback.generation())
                    .unwrap(),
                seconds: if member.identity_key() == leader.identity_key() {
                    1.0
                } else {
                    0.7
                },
                rate: 1.0,
                playing: true,
            })
        };
        let result = group.correct(0.0, clock).unwrap();
        assert!(result[0].corrected);
        assert!((result[0].seconds + 0.3).abs() < 0.001);
        follower
            .with_downcast_mut::<Video, _>(|v| {
                assert!(
                    v.playback.drain_actions().iter().any(
                        |a| matches!(a, DecoderAction::Seek { seconds, .. } if *seconds == 1.0)
                    )
                );
                v.playback.accept_frame(v.playback.generation(), 0.7);
            })
            .unwrap();
        assert!(!group.correct(0.1, clock).unwrap()[0].corrected);
        assert!(group.correct(0.6, clock).unwrap()[0].corrected);
        follower.with_downcast_mut::<Video, _>(|v| {
            assert!(v.playback.drain_actions().iter().any(|action|
                matches!(action, DecoderAction::Seek { seconds, .. } if (*seconds - 1.3).abs() < 0.001)));
        }).unwrap();
        assert_eq!(
            group.correct(0.5, clock).unwrap_err(),
            SyncError::InvalidTime
        );
    }
    #[test]
    fn suspension_and_stale_clocks_never_override_author_intent() {
        let arena = CoreArena::default();
        let leader = playing(&arena);
        let follower = playing(&arena);
        let mut group =
            SynchronizationGroup::new(leader.clone(), vec![follower.clone()], 0.05, 0.5).unwrap();
        follower
            .with_downcast_mut::<Video, _>(|v| {
                v.playback
                    .update_suspension(SuspensionReason::Background, true);
            })
            .unwrap();
        let clock = |_: &CoreHandle| {
            Some(MediaClock {
                generation: 0,
                seconds: 1.0,
                rate: 1.0,
                playing: true,
            })
        };
        assert!(group.correct(0.0, clock).unwrap().is_empty());
        follower
            .with_downcast_mut::<Video, _>(|v| {
                assert!(v.playback.drain_actions().is_empty());
                v.playback
                    .update_suspension(SuspensionReason::Background, false);
                v.playback.reclaim_decoder().unwrap();
                v.playback.opened(v.playback.generation(), 10.0);
                v.playback.observed_playing(v.playback.generation());
            })
            .unwrap();
        assert!(group.correct(0.1, clock).unwrap().is_empty());
        assert!(SynchronizationGroup::new(leader.clone(), vec![leader], 0.05, 0.5).is_err());
    }
    #[test]
    fn group_commands_are_atomic_when_one_member_queue_is_full() {
        let arena = CoreArena::default();
        let leader = playing(&arena);
        let follower = playing(&arena);
        let mut group =
            SynchronizationGroup::new(leader.clone(), vec![follower.clone()], 0.05, 0.5).unwrap();
        follower
            .with_downcast_mut::<Video, _>(|v| {
                for _ in 0..256 {
                    v.playback.enqueue(Command::Play).unwrap();
                }
            })
            .unwrap();
        assert_eq!(
            group.command(Command::Pause),
            Err(SyncError::Command(PlaybackError::QueueFull))
        );
        leader
            .with_downcast_mut::<Video, _>(|v| {
                assert!(v.playback.drain_actions().is_empty());
                assert!(v.playback.wants_play());
            })
            .unwrap();
        follower
            .with_downcast_mut::<Video, _>(|v| {
                v.playback.drain_actions();
            })
            .unwrap();
        group.command(Command::Pause).unwrap();
        for member in [leader, follower] {
            member
                .with_downcast_mut::<Video, _>(|v| {
                    assert!(v.playback.drain_actions().contains(&DecoderAction::Pause));
                    assert!(!v.playback.wants_play());
                })
                .unwrap();
        }
    }
    #[test]
    fn correction_batch_is_atomic_when_a_later_follower_queue_is_full() {
        let arena = CoreArena::default();
        let leader = playing(&arena);
        let first = playing(&arena);
        let full = playing(&arena);
        let mut group =
            SynchronizationGroup::new(leader.clone(), vec![first.clone(), full.clone()], 0.05, 0.5)
                .unwrap();
        full.with_downcast_mut::<Video, _>(|v| {
            for _ in 0..256 {
                v.playback.enqueue(Command::Play).unwrap();
            }
        })
        .unwrap();
        let clock = |member: &CoreHandle| {
            Some(MediaClock {
                generation: 0,
                seconds: if member.identity_key() == leader.identity_key() {
                    1.0
                } else {
                    0.5
                },
                rate: 1.0,
                playing: true,
            })
        };
        assert_eq!(
            group.correct(0.0, clock).unwrap_err(),
            SyncError::Command(PlaybackError::QueueFull)
        );
        first
            .with_downcast_mut::<Video, _>(|v| assert!(v.playback.drain_actions().is_empty()))
            .unwrap();
        full.with_downcast_mut::<Video, _>(|v| v.playback.drain_actions())
            .unwrap();
        let result = group.correct(0.01, clock).unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|measurement| measurement.corrected));
    }
    #[test]
    fn post_seek_preroll_is_measured_after_the_cooldown() {
        let arena = CoreArena::default();
        let leader = playing(&arena);
        let follower = playing(&arena);
        let mut group =
            SynchronizationGroup::new(leader.clone(), vec![follower.clone()], 0.05, 0.5).unwrap();
        let follower_time = std::cell::Cell::new(0.5);
        let leader_time = std::cell::Cell::new(1.0);
        let clock = |member: &CoreHandle| {
            Some(MediaClock {
                generation: member
                    .with_downcast::<Video, _>(|v| v.playback.generation())
                    .unwrap(),
                seconds: if member.identity_key() == leader.identity_key() {
                    leader_time.get()
                } else {
                    follower_time.get()
                },
                rate: 1.0,
                playing: true,
            })
        };
        assert!(group.correct(0.0, clock).unwrap()[0].corrected);
        follower
            .with_downcast_mut::<Video, _>(|v| {
                v.playback.drain_actions();
                v.playback.accept_frame(v.playback.generation(), 1.0);
            })
            .unwrap();
        follower_time.set(1.0);
        assert!(!group.correct(0.1, clock).unwrap()[0].corrected);
        leader_time.set(1.15);
        let correction = group.correct(0.6, clock).unwrap()[0];
        assert!(correction.corrected);
        assert!((correction.seek_prediction_seconds - 0.15).abs() < 0.001);
        follower.with_downcast_mut::<Video, _>(|v| {
            assert!(v.playback.drain_actions().iter().any(|action|
                matches!(action, DecoderAction::Seek { seconds, .. } if (*seconds - 1.3).abs() < 0.001)));
        }).unwrap();
    }
}

/// An authored group retained by its script owner. Videos retain only weak
/// registrations, so group/controller/member ownership cannot form a cycle.
pub struct RegisteredSynchronizationGroup {
    controller: std::cell::RefCell<SynchronizationGroup>,
    alive: std::rc::Rc<std::cell::Cell<bool>>,
    error: std::cell::Cell<Option<SyncError>>,
}
impl RegisteredSynchronizationGroup {
    pub fn new(
        leader: CoreHandle,
        followers: Vec<CoreHandle>,
        tolerance: f64,
        cooldown: f64,
        alive: std::rc::Rc<std::cell::Cell<bool>>,
    ) -> Result<std::rc::Rc<Self>, SyncError> {
        let controller = SynchronizationGroup::new(leader.clone(), followers, tolerance, cooldown)?;
        let group = std::rc::Rc::new(Self {
            controller: std::cell::RefCell::new(controller),
            alive,
            error: std::cell::Cell::new(None),
        });
        leader
            .with_downcast_mut::<Video, _>(|video| {
                video
                    .sync_groups
                    .retain(|weak| weak.upgrade().is_some_and(|group| group.alive.get()));
                if video.sync_groups.len() >= 64 {
                    return Err(SyncError::InvalidPolicy);
                }
                video.sync_groups.push(std::rc::Rc::downgrade(&group));
                Ok(())
            })
            .ok_or(SyncError::InvalidMember)??;
        Ok(group)
    }
    pub fn command(&self, command: Command) -> Result<(), SyncError> {
        if !self.alive.get() {
            return Err(SyncError::InvalidMember);
        }
        self.controller.borrow_mut().command(command)
    }
    /// Consume the most recent asynchronous correction error, if any.
    pub fn take_error(&self) -> Option<SyncError> {
        self.error.take()
    }
}

/// Publish a current native clock for authored groups. All scene members must
/// use the same monotonic seconds domain. Clocks older than 100ms are ignored;
/// fresh running clocks are projected to this update's instant. A leader report
/// corrects its groups. No script callbacks or decoder operations run here.
pub fn report_media_clock(
    member: &CoreHandle,
    now: f64,
    sample: Option<MediaClock>,
) -> Result<bool, SyncError> {
    if !now.is_finite() || now < 0.0 {
        return Err(SyncError::InvalidTime);
    }
    if sample.is_some_and(|clock| {
        !clock.seconds.is_finite()
            || clock.seconds < 0.0
            || !clock.rate.is_finite()
            || clock.rate < 0.0
    }) {
        return Err(SyncError::InvalidTime);
    }
    let groups = member
        .with_downcast_mut::<Video, _>(|video| {
            if video.sync_clock.is_some_and(|(_, previous)| now < previous) {
                return Err(SyncError::InvalidTime);
            }
            video.sync_clock = Some((sample, now));
            video
                .sync_groups
                .retain(|weak| weak.upgrade().is_some_and(|group| group.alive.get()));
            Ok(video
                .sync_groups
                .iter()
                .filter_map(std::rc::Weak::upgrade)
                .collect::<Vec<_>>())
        })
        .ok_or(SyncError::InvalidMember)??;
    let mut corrected = false;
    let mut error = None;
    for group in groups {
        let result = group.controller.borrow_mut().correct(now, |member| {
            member
                .with_downcast::<Video, _>(|video| {
                    let (clock, sampled_at) = video.sync_clock?;
                    let mut clock = clock?;
                    let elapsed = now - sampled_at;
                    if !(0.0..=0.1).contains(&elapsed) {
                        return None;
                    }
                    if clock.playing {
                        clock.seconds += elapsed * clock.rate;
                    }
                    Some(clock)
                })
                .flatten()
        });
        match result {
            Ok(measurements) => {
                corrected |= measurements.iter().any(|measurement| measurement.corrected)
            }
            Err(value) => {
                group.error.set(Some(value));
                error.get_or_insert(value);
            }
        }
    }
    match error {
        Some(error) => Err(error),
        None => Ok(corrected),
    }
}

#[cfg(test)]
mod registered_tests {
    use super::*;
    use crate::source::core::CoreArena;
    use crate::video::playback::{Playback, PlaybackSettings};
    use std::{cell::Cell, rc::Rc};
    fn playing(arena: &CoreArena) -> CoreHandle {
        let mut video = Video::default();
        video.playback = Playback::new(PlaybackSettings {
            autoplay: true,
            ..Default::default()
        });
        video.playback.opened(0, 10.0);
        video.playback.observed_playing(0);
        arena.insert(video)
    }
    fn clock(seconds: f64) -> Option<MediaClock> {
        Some(MediaClock {
            generation: 0,
            seconds,
            rate: 1.0,
            playing: true,
        })
    }
    #[test]
    fn registered_groups_require_fresh_clocks_and_expire_with_script_owner() {
        let arena = CoreArena::default();
        let leader = playing(&arena);
        let follower = playing(&arena);
        let alive = Rc::new(Cell::new(true));
        let group = RegisteredSynchronizationGroup::new(
            leader.clone(),
            vec![follower.clone()],
            0.05,
            0.25,
            alive.clone(),
        )
        .unwrap();
        report_media_clock(&follower, 0.0, clock(0.1)).unwrap();
        assert!(!report_media_clock(&leader, 0.2, clock(1.0)).unwrap());
        report_media_clock(&follower, 0.21, clock(0.1)).unwrap();
        assert!(report_media_clock(&leader, 0.22, clock(1.0)).unwrap());
        follower
            .with_downcast_mut::<Video, _>(|v| {
                assert!(
                    v.playback
                        .drain_actions()
                        .iter()
                        .any(|action| matches!(action,
                super::super::playback::DecoderAction::Seek { seconds, .. } if *seconds == 1.0))
                );
            })
            .unwrap();
        alive.set(false);
        assert!(group.command(Command::Play).is_err());
        assert!(!report_media_clock(&leader, 0.23, clock(1.0)).unwrap());
        assert_eq!(
            leader.with_downcast::<Video, _>(|v| v.sync_groups.len()),
            Some(0)
        );
    }
    #[test]
    fn fresh_clock_projection_does_not_invent_drift_from_update_order() {
        let arena = CoreArena::default();
        let leader = playing(&arena);
        let follower = playing(&arena);
        let _group = RegisteredSynchronizationGroup::new(
            leader.clone(),
            vec![follower.clone()],
            0.01,
            0.25,
            Rc::new(Cell::new(true)),
        )
        .unwrap();
        report_media_clock(&follower, 1.0, clock(1.0)).unwrap();
        assert!(!report_media_clock(&leader, 1.05, clock(1.05)).unwrap());
        assert_eq!(
            report_media_clock(&follower, 0.9, clock(1.0)),
            Err(SyncError::InvalidTime)
        );
    }
}
