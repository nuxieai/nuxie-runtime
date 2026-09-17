//! Nuxie-owned scene video controls. Commands share the runtime queue used by
//! Journey/SDK hosts; Luau never owns a decoder or source acquisition.
use luaur_rt::{Error, UserData, UserDataMethods};
use nuxie_runtime::{
    source::core::CoreHandle,
    video::{
        Video,
        playback::{Command, PlaybackEvent, PlaybackState, SuspensionReason},
    },
};
use std::{cell::Cell, rc::Rc};

pub(super) struct ScriptVideo {
    handle: CoreHandle,
    alive: Rc<Cell<bool>>,
    needs_update: Rc<Cell<bool>>,
}
impl ScriptVideo {
    pub(super) fn new(
        handle: CoreHandle,
        alive: Rc<Cell<bool>>,
        needs_update: Rc<Cell<bool>>,
    ) -> Self {
        Self {
            handle,
            alive,
            needs_update,
        }
    }
    fn with<R>(&self, operation: impl FnOnce(&mut Video) -> R) -> Result<R, Error> {
        if !self.alive.get() {
            return Err(Error::RuntimeError(
                "video context has been disposed".into(),
            ));
        }
        self.handle
            .with_downcast_mut::<Video, _>(operation)
            .ok_or_else(|| Error::RuntimeError("video occurrence is unavailable".into()))
    }
    fn command(&self, command: Command) -> Result<(), Error> {
        self.with(|v| v.playback.enqueue(command))?
            .map_err(|error| Error::RuntimeError(format!("video command: {error:?}")))?;
        self.needs_update.set(true);
        Ok(())
    }
}
pub(super) struct ScriptVideoGroup {
    group: Rc<nuxie_runtime::video::sync::RegisteredSynchronizationGroup>,
    needs_update: Rc<Cell<bool>>,
}
impl ScriptVideoGroup {
    pub(super) fn new(
        members: Vec<CoreHandle>,
        alive: Rc<Cell<bool>>,
        needs_update: Rc<Cell<bool>>,
    ) -> Result<Self, Error> {
        let mut members = members.into_iter();
        let leader = members.next().ok_or_else(|| {
            Error::RuntimeError("video group requires a leader and followers".into())
        })?;
        let group = nuxie_runtime::video::sync::RegisteredSynchronizationGroup::new(
            leader,
            members.collect(),
            0.06,
            0.25,
            alive,
        )
        .map_err(|error| Error::RuntimeError(format!("video group: {error:?}")))?;
        Ok(Self {
            group,
            needs_update,
        })
    }
    fn command(&self, command: Command) -> Result<(), Error> {
        self.group
            .command(command)
            .map_err(|error| Error::RuntimeError(format!("video group: {error:?}")))?;
        self.needs_update.set(true);
        Ok(())
    }
}
impl UserData for ScriptVideoGroup {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("play", |_, this, ()| this.command(Command::Play));
        methods.add_method("pause", |_, this, ()| this.command(Command::Pause));
        methods.add_method("seek", |_, this, seconds: f64| {
            this.command(Command::Seek(seconds))
        });
        methods.add_method("setRate", |_, this, rate: f32| {
            this.command(Command::Rate(rate))
        });
        methods.add_method("setMuted", |_, this, muted: bool| {
            this.command(Command::Mute(muted))
        });
        methods.add_method("setVolume", |_, this, volume: f32| {
            this.command(Command::Volume(volume))
        });
        methods.add_method("setLooping", |_, this, enabled: bool| {
            this.command(Command::Loop(enabled))
        });
        methods.add_method("setLoopRange", |_, this, (start, end): (f64, f64)| {
            this.command(Command::LoopRange { start, end })
        });
        methods.add_method("takeError", |_, this, ()| {
            Ok(this.group.take_error().map(|error| format!("{error:?}")))
        });
    }
}

fn state_name(state: PlaybackState) -> &'static str {
    match state {
        PlaybackState::Opening => "opening",
        PlaybackState::Ready => "ready",
        PlaybackState::Playing => "playing",
        PlaybackState::Paused => "paused",
        PlaybackState::Seeking => "seeking",
        PlaybackState::Buffering => "buffering",
        PlaybackState::Ended => "ended",
        PlaybackState::Failed => "failed",
        PlaybackState::Disposed => "disposed",
    }
}
impl UserData for ScriptVideo {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("play", |_, this, ()| this.command(Command::Play));
        methods.add_method("pause", |_, this, ()| this.command(Command::Pause));
        methods.add_method("seek", |_, this, seconds: f64| {
            this.command(Command::Seek(seconds))
        });
        methods.add_method("setRate", |_, this, rate: f32| {
            this.command(Command::Rate(rate))
        });
        methods.add_method("setVolume", |_, this, volume: f32| {
            this.command(Command::Volume(volume))
        });
        methods.add_method("setMuted", |_, this, muted: bool| {
            this.command(Command::Mute(muted))
        });
        methods.add_method("setLooping", |_, this, enabled: bool| {
            this.command(Command::Loop(enabled))
        });
        methods.add_method("setLoopRange", |_, this, (start, end): (f64, f64)| {
            this.command(Command::LoopRange { start, end })
        });
        methods.add_method("caption", |_, this, ()| this.with(|v| v.caption_text()));
        methods.add_method("position", |_, this, ()| {
            this.with(|v| v.playback.position())
        });
        methods.add_method("state", |_, this, ()| {
            this.with(|v| state_name(v.playback.state()))
        });
        methods.add_method("isResourceLimited", |_, this, ()| {
            this.with(|v| v.playback.is_suspended(SuspensionReason::Resources))
        });
        methods.add_method("wantsPlay", |_, this, ()| {
            this.with(|v| v.playback.wants_play())
        });
        methods.add_method("nextEvent", |_, this, ()| {
            this.with(|v| {
                v.playback.pop_event().map(|event| match event {
                    PlaybackEvent::State(state) => state_name(state),
                    PlaybackEvent::PlayBlocked => "playBlocked",
                    PlaybackEvent::FirstFrame => "firstFrame",
                    PlaybackEvent::Looped => "looped",
                    PlaybackEvent::Error => "error",
                    PlaybackEvent::ResourceLimited(true) => "resourceLimited",
                    PlaybackEvent::ResourceLimited(false) => "resourceRestored",
                })
            })
        });
    }
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::*;
    #[test]
    fn luau_group_commands_and_host_clocks_share_registered_occurrences() {
        use nuxie_runtime::video::sync::{MediaClock, report_media_clock};
        let arena = nuxie_runtime::source::core::CoreArena::default();
        let members = [
            arena.insert(Video::default()),
            arena.insert(Video::default()),
        ];
        for member in &members {
            member
                .with_downcast_mut::<Video, _>(|v| v.playback.opened(0, 10.0))
                .unwrap();
        }
        let alive = Rc::new(Cell::new(true));
        let needs_update = Rc::new(Cell::new(false));
        let lua = luaur_rt::Lua::new();
        lua.globals()
            .set(
                "group",
                lua.create_userdata(
                    ScriptVideoGroup::new(members.to_vec(), alive.clone(), needs_update.clone())
                        .unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        lua.load("group:play(); group:setMuted(true)")
            .exec()
            .unwrap();
        assert!(needs_update.get());
        for member in &members {
            member
                .with_downcast_mut::<Video, _>(|v| {
                    v.playback.drain_actions();
                    v.playback.observed_playing(0);
                    assert!(v.playback.wants_play());
                    assert!(v.playback.settings().muted);
                })
                .unwrap();
        }
        let sample = |seconds| {
            Some(MediaClock {
                generation: 0,
                seconds,
                rate: 1.0,
                playing: true,
            })
        };
        report_media_clock(&members[1], 0.0, sample(0.5)).unwrap();
        assert!(report_media_clock(&members[0], 0.0, sample(1.0)).unwrap());
        members[1].with_downcast_mut::<Video, _>(|v| {
            assert!(v.playback.drain_actions().iter().any(|action| matches!(action,
                nuxie_runtime::video::playback::DecoderAction::Seek { seconds, .. } if *seconds == 1.0)));
        }).unwrap();
        lua.load("group:pause(); assert(group:takeError() == nil)")
            .exec()
            .unwrap();
        for member in &members {
            member
                .with_downcast_mut::<Video, _>(|v| {
                    v.playback.drain_actions();
                    assert!(!v.playback.wants_play());
                })
                .unwrap();
        }
        alive.set(false);
        assert!(lua.load("group:play()").exec().is_err());
    }
    #[test]
    fn luau_controls_use_the_live_video_queue_and_expire_with_context() {
        let arena = nuxie_runtime::source::core::CoreArena::default();
        let handle = arena.insert(Video::default());
        handle
            .with_downcast_mut::<Video, _>(|v| v.playback.opened(0, 10.0))
            .unwrap();
        let alive = Rc::new(Cell::new(true));
        let needs_update = Rc::new(Cell::new(false));
        let lua = luaur_rt::Lua::new();
        lua.globals()
            .set(
                "video",
                lua.create_userdata(ScriptVideo::new(
                    handle.clone(),
                    alive.clone(),
                    needs_update.clone(),
                ))
                .unwrap(),
            )
            .unwrap();
        lua.load("video:play(); video:seek(3); video:setMuted(true)")
            .exec()
            .unwrap();
        assert!(needs_update.get());
        handle
            .with_downcast_mut::<Video, _>(|v| {
                use nuxie_runtime::video::playback::DecoderAction;
                assert_eq!(
                    v.playback.drain_actions(),
                    vec![
                        DecoderAction::Seek {
                            seconds: 3.0,
                            generation: 1
                        },
                        DecoderAction::Volume(0.0),
                        DecoderAction::Play
                    ]
                );
            })
            .unwrap();
        assert_eq!(
            lua.load("return video:position()").eval::<f64>().unwrap(),
            3.0
        );
        assert!(lua.load("video:seek(-1)").exec().is_err());
        lua.load("video:setLoopRange(2, 4); video:setLooping(true)")
            .exec()
            .unwrap();
        handle
            .with_downcast_mut::<Video, _>(|v| {
                v.playback.drain_actions();
                assert!(v.playback.settings().looping);
                assert_eq!(v.playback.settings().loop_start, 2.0);
                assert_eq!(v.playback.settings().loop_end, 4.0);
            })
            .unwrap();
        assert!(lua.load("video:setLoopRange(4, 2)").exec().is_err());
        handle
            .with_downcast_mut::<Video, _>(|v| {
                while v.playback.pop_event().is_some() {}
                v.apply_allocation(nuxie_runtime::video::resources::Allocation::Poster);
                v.apply_allocation(nuxie_runtime::video::resources::Allocation::Poster);
            })
            .unwrap();
        lua.load(
            "assert(video:isResourceLimited()); assert(video:nextEvent() == 'resourceLimited')",
        )
        .exec()
        .unwrap();
        handle
            .with_downcast_mut::<Video, _>(|v| {
                while v.playback.pop_event().is_some() {}
                v.apply_allocation(nuxie_runtime::video::resources::Allocation::Hardware);
            })
            .unwrap();
        lua.load("assert(not video:isResourceLimited()); assert(video:nextEvent() == 'resourceRestored')").exec().unwrap();
        alive.set(false);
        assert!(lua.load("video:play()").exec().is_err());
    }
}
