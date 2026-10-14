//! Ordinary video properties share playback's validated, ordered decoder queue.
use super::playback::{Command, Playback, PlaybackError, PlaybackState};

pub const LOOP: u16 = 60004;
pub const MUTED: u16 = 60005;
pub const VOLUME: u16 = 60006;
pub const RATE: u16 = 60007;
pub const LOOP_START: u16 = 60014;
pub const LOOP_END: u16 = 60015;
pub const PAUSED: u16 = 60016;
pub const POSITION: u16 = 60017;
pub const DURATION: u16 = 60018;
pub const ENDED: u16 = 60019;
pub const STATE: u16 = 60020;
/// Only the scripts that read the video draw it. A starting attribute, not a
/// live property.
pub const DRAWN_BY_SCRIPT: u16 = 60021;
pub(crate) const KEYS: [u16; 11] = [
    LOOP, MUTED, VOLUME, RATE, LOOP_START, LOOP_END, PAUSED, POSITION, DURATION, ENDED, STATE,
];

pub fn field_id(key: i32) -> Option<i32> {
    match u16::try_from(key).ok()? {
        LOOP | MUTED | PAUSED | ENDED => Some(4),
        VOLUME | RATE | LOOP_START | LOOP_END | POSITION | DURATION | STATE => Some(2),
        _ => None,
    }
}

/// An ephemeral read of applied playback plus its pending property writes.
/// No values are stored beside the playback state and command queue.
#[derive(Clone, Copy)]
pub(crate) struct Values(pub [f64; 11]);
impl Values {
    pub fn get(self, key: u16) -> Option<f64> {
        KEYS.iter().position(|k| *k == key).map(|i| self.0[i])
    }
    pub fn changed(self, after: Self) -> u16 {
        self.0
            .iter()
            .zip(after.0)
            .enumerate()
            .fold(0, |mask, (i, (a, b))| {
                mask | if *a != b { 1 << i } else { 0 }
            })
    }
}

/// The shared entry point for registry, script, and loop-range host writes.
#[derive(Clone, Copy)]
pub enum Write {
    Bool(u16, bool),
    Number(u16, f64),
    LoopRange { start: f64, end: f64 },
}
impl Write {
    pub(crate) fn command(self, playback: &Playback) -> Result<Command, PlaybackError> {
        Ok(match self {
            Self::Bool(LOOP, value) => Command::Loop(value),
            Self::Bool(MUTED, value) => Command::Mute(value),
            Self::Bool(PAUSED, true) => Command::Pause,
            Self::Bool(PAUSED, false) => Command::Play,
            Self::Number(POSITION, value) => Command::Seek(value),
            Self::Number(VOLUME, value) if value.is_finite() && (0.0..=1.0).contains(&value) => {
                Command::Volume(value as f32)
            }
            Self::Number(RATE, value)
                if value.is_finite() && value > 0.0 && value <= f64::from(f32::MAX) =>
            {
                Command::Rate(value as f32)
            }
            Self::Number(LOOP_START, start) => Command::LoopRange {
                start,
                end: playback.property_values().0[5],
            },
            Self::Number(LOOP_END, end) => Command::LoopRange {
                start: playback.property_values().0[4],
                end,
            },
            Self::LoopRange { start, end } => Command::LoopRange { start, end },
            _ => return Err(PlaybackError::InvalidValue),
        })
    }
}

impl Playback {
    pub(crate) fn property_values(&self) -> Values {
        let mut settings = self.settings();
        let mut paused = !self.wants_play();
        let mut position = self.position();
        for command in self.property_commands() {
            match *command {
                Command::Play => paused = false,
                Command::Pause | Command::Dispose => paused = true,
                Command::Seek(value) => position = self.duration().map_or(value, |d| value.min(d)),
                Command::Rate(value) => settings.rate = value,
                Command::Volume(value) => settings.volume = value,
                Command::Mute(value) => settings.muted = value,
                Command::Loop(value) => settings.looping = value,
                Command::LoopRange { start, end } => {
                    settings.loop_start = start;
                    settings.loop_end = end;
                }
                Command::Reenter if settings.reentry == 0 => {
                    position = 0.0;
                    paused = !settings.autoplay;
                }
                _ => {}
            }
        }
        Values([
            u8::from(settings.looping).into(),
            u8::from(settings.muted).into(),
            settings.volume.into(),
            settings.rate.into(),
            settings.loop_start,
            settings.loop_end,
            u8::from(paused).into(),
            position,
            self.duration().unwrap_or(0.0),
            u8::from(self.state() == PlaybackState::Ended).into(),
            self.state() as u8 as f64,
        ])
    }
}
