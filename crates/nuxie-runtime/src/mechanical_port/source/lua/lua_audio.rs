#![cfg(feature = "rive_scripting")]
use crate::mechanical_port::source::lua::rive_lua_libs::*;
#[cfg(feature = "rive_audio")]
use crate::mechanical_port::source::{
    artboard::Artboard,
    audio::{
        audio_engine::{AudioEngine, AudioEngineRef},
        audio_sound::AudioSoundRef,
        audio_source::AudioSource,
    },
};
#[cfg(feature = "rive_audio")]
use std::rc::Rc;
#[cfg(feature = "rive_audio")]
impl ScriptedAudioSource {
    pub fn set_source(&mut self, value: Rc<AudioSource>) {
        self.source = Some(value)
    }
    fn initialize_sound(
        &self,
        s: &mut LuaState,
        sound: Option<AudioSoundRef>,
        artboard: Option<*mut Artboard>,
    ) -> i32 {
        if let Some(sound) = sound {
            if let Some(a) = artboard {
                sound.borrow_mut().set_volume(unsafe { &*a }.volume());
            }
            s.new_rive(ScriptedAudioSound {
                sound: Some(sound),
                artboard,
            });
            1
        } else {
            0
        }
    }
    pub fn play(
        &self,
        s: &mut LuaState,
        engine: &AudioEngineRef,
        time: f64,
        relative: bool,
    ) -> i32 {
        let Some(source) = self.source.as_ref() else {
            return 0;
        };
        #[cfg(feature = "rive_tools")]
        if !s.thread_data::<dyn ScriptingContext>().is_playing() {
            return 0;
        }
        let start = time as f32
            + if relative {
                engine.borrow().time_in_seconds()
            } else {
                0.0
            };
        let sound = AudioEngine::play_seconds(engine, source.clone(), start, 0, 0, None);
        self.initialize_sound(s, sound, None)
    }
    pub fn play_frame(
        &self,
        s: &mut LuaState,
        engine: &AudioEngineRef,
        time: u64,
        relative: bool,
    ) -> i32 {
        let Some(source) = self.source.as_ref() else {
            return 0;
        };
        #[cfg(feature = "rive_tools")]
        if !s.thread_data::<dyn ScriptingContext>().is_playing() {
            return 0;
        }
        let start = time
            + if relative {
                engine.borrow().time_in_frames()
            } else {
                0
            };
        let sound = AudioEngine::play(engine, source.clone(), start, 0, 0, None);
        self.initialize_sound(s, sound, None)
    }
}
#[cfg(feature = "rive_audio")]
fn source_index(s: &mut LuaState) -> i32 {
    let (key, atom) = s.to_string_atom(2);
    if key.is_none() {
        return s.type_error(2, s.type_name(LuaType::String));
    }
    let source = s.to_rive::<ScriptedAudioSource>(1);
    let Some(audio) = source.source.as_ref() else {
        return 0;
    };
    if atom == LuaAtoms::Duration {
        s.push_number(audio.duration() as f64);
        1
    } else {
        s.error(format!(
            "'{}' is not a valid index of {}",
            key.unwrap(),
            ScriptedAudioSource::LUA_NAME
        ))
    }
}
#[cfg(feature = "rive_audio")]
fn sound_namecall(s: &mut LuaState) -> i32 {
    let (name, atom) = s.namecall_atom();
    let Some(sound) = s
        .to_rive_mut::<ScriptedAudioSound>(1)
        .sound
        .as_ref()
        .cloned()
    else {
        return 0;
    };
    match atom {
        LuaAtoms::Play => sound.borrow_mut().play(),
        LuaAtoms::Pause => sound.borrow_mut().pause(),
        LuaAtoms::Resume => sound.borrow_mut().resume(),
        LuaAtoms::Stop => sound.borrow_mut().stop(if s.top() >= 2 && s.is_number(2) {
            s.to_number(2).unwrap() as u64
        } else {
            0
        }),
        LuaAtoms::Seek => {
            let v = sound.borrow_mut().seek_seconds(s.check_number(2) as f32);
            s.push_boolean(v);
            return 1;
        }
        LuaAtoms::SeekFrame => {
            let v = sound.borrow_mut().seek(s.check_number(2) as u64);
            s.push_boolean(v);
            return 1;
        }
        LuaAtoms::Completed => {
            s.push_boolean(sound.borrow().completed());
            return 1;
        }
        LuaAtoms::Time => {
            s.push_number(sound.borrow().time_in_seconds() as f64);
            return 1;
        }
        LuaAtoms::TimeFrame => {
            s.push_number(sound.borrow().time_in_frames() as f64);
            return 1;
        }
        _ => {
            return s.error(format!(
                "{} is not a valid method of {}",
                name.unwrap_or_default(),
                ScriptedAudioSound::LUA_NAME
            ));
        }
    }
    0
}
#[cfg(feature = "rive_audio")]
fn sound_index(s: &mut LuaState) -> i32 {
    let (key, atom) = s.to_string_atom(2);
    if key.is_none() {
        return s.type_error(2, s.type_name(LuaType::String));
    }
    if atom == LuaAtoms::Volume {
        let v = s
            .to_rive::<ScriptedAudioSound>(1)
            .sound
            .as_ref()
            .unwrap()
            .borrow()
            .volume();
        s.push_number(v as f64);
        1
    } else {
        s.error(format!(
            "'{}' is not a valid index of {}",
            key.unwrap(),
            ScriptedAudioSound::LUA_NAME
        ))
    }
}
#[cfg(feature = "rive_audio")]
fn sound_newindex(s: &mut LuaState) -> i32 {
    let (key, atom) = s.to_string_atom(2);
    if key.is_none() {
        return s.type_error(2, s.type_name(LuaType::String));
    }
    if atom == LuaAtoms::Volume {
        let mut v = s.check_number(3) as f32;
        let sound = s.to_rive_mut::<ScriptedAudioSound>(1);
        if let Some(a) = sound.artboard {
            v *= unsafe { &*a }.volume();
        }
        sound.sound.as_ref().unwrap().borrow_mut().set_volume(v);
        0
    } else {
        s.error(format!(
            "'{}' is not a valid index of {}",
            key.unwrap(),
            ScriptedAudioSound::LUA_NAME
        ))
    }
}
#[cfg(feature = "rive_audio")]
fn with_engine(s: &mut LuaState, kind: u8) -> i32 {
    let Some(engine) = AudioEngine::runtime_engine(true) else {
        s.push_nil();
        return 1;
    };
    let source = s.to_rive::<ScriptedAudioSource>(1) as *const ScriptedAudioSource;
    if source.is_null() {
        s.push_nil();
        return 1;
    }
    let time = s.to_number(2).unwrap_or(0.0);
    unsafe {
        match kind {
            0 => (*source).play(s, &engine, 0.0, true),
            1 => (*source).play(s, &engine, time, false),
            2 => (*source).play(s, &engine, time, true),
            3 => (*source).play_frame(s, &engine, time as u64, false),
            _ => (*source).play_frame(s, &engine, time as u64, true),
        }
    }
}
#[cfg(not(feature = "rive_audio"))]
fn with_engine(s: &mut LuaState, _kind: u8) -> i32 {
    s.push_nil();
    1
}
fn play(s: &mut LuaState) -> i32 {
    with_engine(s, 0)
}
fn play_at_time(s: &mut LuaState) -> i32 {
    with_engine(s, 1)
}
fn play_in_time(s: &mut LuaState) -> i32 {
    with_engine(s, 2)
}
fn play_at_frame(s: &mut LuaState) -> i32 {
    with_engine(s, 3)
}
fn play_in_frame(s: &mut LuaState) -> i32 {
    with_engine(s, 4)
}
fn time(s: &mut LuaState) -> i32 {
    #[cfg(feature = "rive_audio")]
    s.push_number(
        AudioEngine::runtime_engine(true)
            .map(|v| v.borrow().time_in_seconds() as f64)
            .unwrap_or(0.0),
    );
    #[cfg(not(feature = "rive_audio"))]
    s.push_number(0.0);
    1
}
fn time_frame(s: &mut LuaState) -> i32 {
    #[cfg(feature = "rive_audio")]
    s.push_integer(
        AudioEngine::runtime_engine(true)
            .map(|v| v.borrow().time_in_frames() as i64)
            .unwrap_or(0),
    );
    #[cfg(not(feature = "rive_audio"))]
    s.push_integer(0);
    1
}
fn sample_rate(s: &mut LuaState) -> i32 {
    #[cfg(feature = "rive_audio")]
    s.push_integer(
        AudioEngine::runtime_engine(true)
            .map(|v| v.borrow().sample_rate() as i64)
            .unwrap_or(0),
    );
    #[cfg(not(feature = "rive_audio"))]
    s.push_integer(0);
    1
}
pub fn luaopen_rive_audio(s: &mut LuaState) -> i32 {
    #[cfg(feature = "rive_audio")]
    {
        s.register_rive::<ScriptedAudioSource>();
        s.push_function(source_index);
        s.set_field(-2, "__index");
        s.set_readonly(-1, true);
        s.pop(1);
        s.register_audio_source_duration();
        s.register_rive::<ScriptedAudioSound>();
        for (name, f) in [
            ("__namecall", sound_namecall as LuaFunction),
            ("__index", sound_index),
            ("__newindex", sound_newindex),
        ] {
            s.push_function(f);
            s.set_field(-2, name);
        }
        s.set_readonly(-1, true);
        s.pop(1);
        s.register_audio_sound_volume();
        s.register(
            ScriptedAudio::LUA_NAME,
            &[
                LuaReg::new("time", time),
                LuaReg::new("timeFrame", time_frame),
                LuaReg::new("sampleRate", sample_rate),
                LuaReg::new("play", play),
                LuaReg::new("playAtTime", play_at_time),
                LuaReg::new("playInTime", play_in_time),
                LuaReg::new("playAtFrame", play_at_frame),
                LuaReg::new("playInFrame", play_in_frame),
                LuaReg::END,
            ],
        );
    }
    0
}
