use crate::mechanical_port::source::input::{
    gamepad_snapshot::{
        GamepadInputChange, GamepadInputChangeKind, GamepadMappingKind, GamepadSnapshot,
    },
    standard_gamepad::{StandardGamepadAxis, StandardGamepadButton},
};
use std::collections::HashMap;
pub const GAMEPAD_BATCH_WIRE_VERSION: u32 = 2;
pub const GAMEPAD_BATCH_MAX_BUTTONS: u8 = 32;
pub const GAMEPAD_BATCH_MAX_AXES: u8 = 16;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum GamepadRecordType {
    Connected = 0,
    Update = 1,
    Disconnected = 2,
}
#[derive(Clone, Debug)]
pub enum GamepadInvocation {
    Connected(GamepadSnapshot),
    Event(GamepadEventInvocation),
    Disconnected(i32),
}
#[derive(Clone, Debug)]
pub struct GamepadEventInvocation {
    pub full_state: GamepadSnapshot,
    pub change: GamepadInputChange,
    pub standard_button: Option<StandardGamepadButton>,
    pub standard_axis: Option<StandardGamepadAxis>,
}
pub trait GamepadDispatcher {
    fn dispatch(&mut self, invocation: GamepadInvocation);
}
pub struct GamepadBatchState {
    pub gamepads: HashMap<i32, GamepadSnapshot>,
}
impl Default for GamepadBatchState {
    fn default() -> Self {
        Self {
            gamepads: HashMap::new(),
        }
    }
}
struct Reader<'a> {
    b: &'a [u8],
    p: usize,
}
impl<'a> Reader<'a> {
    fn u8(&mut self) -> Option<u8> {
        let v = *self.b.get(self.p)?;
        self.p += 1;
        Some(v)
    }
    fn u32(&mut self) -> Option<u32> {
        let v = u32::from_le_bytes(self.b.get(self.p..self.p + 4)?.try_into().ok()?);
        self.p += 4;
        Some(v)
    }
    fn f32(&mut self) -> Option<f32> {
        Some(f32::from_bits(self.u32()?))
    }
}
fn mask(s: &mut GamepadSnapshot) {
    s.button_mask = 0;
    for (i, v) in s.button_values.iter().enumerate() {
        if *v >= 0.5 {
            s.button_mask |= 1u64 << i;
        }
    }
}
impl GamepadBatchState {
    pub fn submit(&mut self, data: Option<&[u8]>, dispatcher: &mut dyn GamepadDispatcher) -> bool {
        let Some(data) = data else { return false };
        let mut r = Reader { b: data, p: 0 };
        if r.u32() != Some(GAMEPAD_BATCH_WIRE_VERSION) {
            return false;
        }
        while r.p < data.len() {
            match r.u8() {
                Some(0) => {
                    let Some(id) = r.u32() else { return false };
                    let (Some(mapping), Some(nb), Some(na), Some(_)) =
                        (r.u8(), r.u8(), r.u8(), r.u8())
                    else {
                        return false;
                    };
                    if nb > GAMEPAD_BATCH_MAX_BUTTONS || na > GAMEPAD_BATCH_MAX_AXES {
                        return false;
                    }
                    let mut s = GamepadSnapshot {
                        device_id: id as i32,
                        mapping: if mapping == 0 {
                            GamepadMappingKind::Standard
                        } else {
                            GamepadMappingKind::Unknown
                        },
                        ..Default::default()
                    };
                    for _ in 0..nb {
                        let Some(v) = r.f32() else { return false };
                        s.button_values.push(v)
                    }
                    for _ in 0..na {
                        let Some(v) = r.f32() else { return false };
                        s.axes.push(v)
                    }
                    mask(&mut s);
                    self.gamepads.insert(s.device_id, s.clone());
                    dispatcher.dispatch(GamepadInvocation::Connected(s));
                }
                Some(1) => {
                    let (Some(id), Some(n)) = (r.u32(), r.u8()) else {
                        return false;
                    };
                    if !self.gamepads.contains_key(&(id as i32)) {
                        return false;
                    }
                    let mut changes = Vec::new();
                    for _ in 0..n {
                        let (Some(k), Some(i), Some(v)) = (r.u8(), r.u8(), r.f32()) else {
                            return false;
                        };
                        changes.push(GamepadInputChange {
                            kind: if k == 0 {
                                GamepadInputChangeKind::Button
                            } else {
                                GamepadInputChangeKind::Axis
                            },
                            index: i,
                            value: v,
                        });
                    }
                    let s = self.gamepads.get_mut(&(id as i32)).unwrap();
                    for c in &changes {
                        match c.kind {
                            GamepadInputChangeKind::Button => {
                                if c.index >= GAMEPAD_BATCH_MAX_BUTTONS {
                                    return false;
                                }
                                if s.button_values.len() <= c.index as usize {
                                    s.button_values.resize(c.index as usize + 1, 0.0)
                                }
                                s.button_values[c.index as usize] = c.value;
                                mask(s)
                            }
                            GamepadInputChangeKind::Axis => {
                                if c.index >= GAMEPAD_BATCH_MAX_AXES {
                                    return false;
                                }
                                if s.axes.len() <= c.index as usize {
                                    s.axes.resize(c.index as usize + 1, 0.0)
                                }
                                s.axes[c.index as usize] = c.value;
                            }
                        }
                    }
                    let final_s = s.clone();
                    for c in changes {
                        let (sb, sa) = if final_s.mapping == GamepadMappingKind::Standard {
                            match c.kind {
                                GamepadInputChangeKind::Button if c.index <= 16 => {
                                    (button(c.index), None)
                                }
                                GamepadInputChangeKind::Axis if c.index <= 5 => {
                                    (None, axis(c.index))
                                }
                                _ => (None, None),
                            }
                        } else {
                            (None, None)
                        };
                        dispatcher.dispatch(GamepadInvocation::Event(GamepadEventInvocation {
                            full_state: final_s.clone(),
                            change: c,
                            standard_button: sb,
                            standard_axis: sa,
                        }));
                    }
                }
                Some(2) => {
                    let Some(id) = r.u32() else { return false };
                    self.gamepads.remove(&(id as i32));
                    dispatcher.dispatch(GamepadInvocation::Disconnected(id as i32));
                }
                _ => return false,
            }
        }
        true
    }
}
fn button(v: u8) -> Option<StandardGamepadButton> {
    StandardGamepadButton::from_raw(v)
}
fn axis(v: u8) -> Option<StandardGamepadAxis> {
    StandardGamepadAxis::from_raw(v)
}
