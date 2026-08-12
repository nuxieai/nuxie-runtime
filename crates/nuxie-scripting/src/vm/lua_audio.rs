//! Direct owner for pinned `src/lua/lua_audio.cpp`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use luaur_rt::{Lua, Result, Table, UserData, UserDataFields, UserDataMethods, Value};
use nuxie_runtime::{AudioEngine, AudioSound, AudioSource, RuntimeAudioAssetOwners};

struct ScriptedAudioSource(Arc<AudioSource>);

impl UserData for ScriptedAudioSource {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("duration", |_, this| Ok(this.0.duration()));
    }
}

struct ScriptedAudioSound(AudioSound);

impl UserData for ScriptedAudioSound {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("volume", |_, this| Ok(this.0.volume()));
        fields.add_field_method_set("volume", |_, this, value: f32| {
            this.0.set_volume(value);
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("play", |_, this, ()| {
            this.0.play();
            Ok(())
        });
        methods.add_method("pause", |_, this, ()| {
            this.0.pause();
            Ok(())
        });
        methods.add_method("resume", |_, this, ()| {
            this.0.resume();
            Ok(())
        });
        methods.add_method("stop", |_, this, fade_frames: Option<f64>| {
            this.0.stop(number_to_frame(fade_frames.unwrap_or(0.0)));
            Ok(())
        });
        methods.add_method("seek", |_, this, seconds: f32| {
            Ok(this.0.seek_seconds(seconds))
        });
        methods.add_method("seekFrame", |_, this, frame: f64| {
            Ok(this.0.seek(number_to_frame(frame)))
        });
        methods.add_method("completed", |_, this, ()| Ok(this.0.completed()));
        methods.add_method("time", |_, this, ()| Ok(this.0.time_in_seconds()));
        methods.add_method(
            "timeFrame",
            |_, this, ()| Ok(this.0.time_in_frames() as f64),
        );
    }
}

#[derive(Clone, Default)]
pub(super) struct ScriptedAudioAssets {
    direct: Rc<RefCell<BTreeMap<String, Vec<Arc<AudioSource>>>>>,
    file_assets: Rc<RefCell<BTreeMap<String, Vec<u32>>>>,
    owners: Rc<RefCell<Option<Arc<RuntimeAudioAssetOwners>>>>,
}

impl ScriptedAudioAssets {
    pub(super) fn install(lua: &Lua) -> Self {
        let assets = Self::default();
        lua.set_app_data(assets.clone());
        assets
    }

    pub(super) fn register(&self, name: &str, source: Arc<AudioSource>) {
        self.direct
            .borrow_mut()
            .entry(name.to_owned())
            .or_default()
            .push(source);
    }

    pub(super) fn set_file_owners(&self, owners: Arc<RuntimeAudioAssetOwners>) {
        *self.owners.borrow_mut() = Some(owners);
    }

    pub(super) fn register_file_asset(&self, name: &str, global_id: u32) {
        self.file_assets
            .borrow_mut()
            .entry(name.to_owned())
            .or_default()
            .push(global_id);
    }

    pub(super) fn lookup(lua: &Lua, name: &str) -> Result<Value> {
        let source = lua.app_data_ref::<Self>().and_then(|assets| {
            let from_file =
                assets.owners.borrow().as_ref().and_then(|owners| {
                    assets.file_assets.borrow().get(name).and_then(|matches| {
                        matches.iter().find_map(|global_id| owners.get(*global_id))
                    })
                });
            from_file.or_else(|| {
                assets
                    .direct
                    .borrow()
                    .get(name)
                    .and_then(|v| v.first())
                    .cloned()
            })
        });
        match source {
            Some(source) => lua
                .create_userdata(ScriptedAudioSource(source))
                .map(Value::UserData),
            None => Ok(Value::Nil),
        }
    }
}

pub(super) fn install_audio_global(lua: &Lua) -> Result<()> {
    let audio = lua.create_table();
    audio.set(
        "time",
        lua.create_function(|_, ()| Ok(AudioEngine::runtime_engine().time_in_seconds()))?,
    )?;
    audio.set(
        "timeFrame",
        lua.create_function(|_, ()| Ok(AudioEngine::runtime_engine().time_in_frames() as f64))?,
    )?;
    audio.set(
        "sampleRate",
        lua.create_function(|_, ()| Ok(AudioEngine::runtime_engine().sample_rate()))?,
    )?;
    install_play_method(lua, &audio, "play", false, true, false)?;
    install_play_method(lua, &audio, "playAtTime", false, false, true)?;
    install_play_method(lua, &audio, "playInTime", false, true, true)?;
    install_play_method(lua, &audio, "playAtFrame", true, false, true)?;
    install_play_method(lua, &audio, "playInFrame", true, true, true)?;
    lua.globals().set("Audio", audio)
}

fn install_play_method(
    lua: &Lua,
    audio: &Table,
    name: &str,
    frames: bool,
    relative: bool,
    has_time: bool,
) -> Result<()> {
    let function = lua.create_function(move |lua, arguments: luaur_rt::MultiValue| {
        let Some(Value::UserData(source)) = arguments.front() else {
            return Ok(Value::Nil);
        };
        let Ok(source) = source.borrow::<ScriptedAudioSource>() else {
            return Ok(Value::Nil);
        };
        let source = Arc::clone(&source.0);
        let time = if has_time {
            arguments
                .get(1)
                .and_then(Value::as_number)
                .unwrap_or_default()
        } else {
            0.0
        };
        play_source(lua, source, time, frames, relative)
    })?;
    audio.set(name, function)
}

fn play_source(
    lua: &Lua,
    source: Arc<AudioSource>,
    time: f64,
    frames: bool,
    relative: bool,
) -> Result<Value> {
    let engine = AudioEngine::runtime_engine();
    let sound = if frames {
        let mut start = number_to_frame(time);
        if relative {
            start = start.saturating_add(engine.time_in_frames());
        }
        engine.play(source, start, 0, 0, None)
    } else {
        let mut start = time as f32;
        if relative {
            start += engine.time_in_seconds();
        }
        engine.play_seconds(source, start, 0, 0, None)
    };
    match sound {
        Some(sound) => lua
            .create_userdata(ScriptedAudioSound(sound))
            .map(Value::UserData),
        None => Ok(Value::Nil),
    }
}

fn number_to_frame(value: f64) -> u64 {
    if value.is_nan() || value <= 0.0 {
        0
    } else if value >= u64::MAX as f64 {
        u64::MAX
    } else {
        value as u64
    }
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::*;

    #[test]
    fn audio_source_sound_and_static_queries_match_the_pinned_lua_surface() {
        let lua = Lua::new();
        let assets = ScriptedAudioAssets::install(&lua);
        install_audio_global(&lua).expect("Audio global");
        let engine = AudioEngine::make_and_store(1, 4).expect("runtime engine");
        assets.register(
            "tone",
            Arc::new(AudioSource::from_buffered(vec![0.25, 0.5, 0.75, 1.0], 1, 4).expect("source")),
        );
        lua.globals()
            .set(
                "source",
                match ScriptedAudioAssets::lookup(&lua, "tone").expect("lookup") {
                    Value::UserData(source) => source,
                    _ => panic!("audio source userdata"),
                },
            )
            .expect("source global");

        let (duration, sample_rate, volume, seek, frame): (f32, u32, f32, bool, f64) = lua
            .load(
                "local sound = Audio.play(source)\n\
                 sound.volume = 0.1\n\
                 return source.duration, Audio.sampleRate(), sound.volume, sound:seekFrame(2), sound:timeFrame()",
            )
            .eval()
            .expect("audio script");
        assert_eq!(duration, 1.0);
        assert_eq!(sample_rate, 4);
        assert_eq!(volume, 0.1);
        assert!(seek);
        assert_eq!(frame, 0.0);
        assert_eq!(engine.playing_sound_count(), 1);
    }
}
