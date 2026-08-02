use super::{
    ScriptGamepadInputChange, ScriptGamepadMappingKind, ScriptGamepadSnapshot,
    ScriptListenerInvocation, StateMachineInstance,
};
use crate::ArtboardInstance;

/// Little-endian wire version accepted by
/// [`StateMachineInstance::submit_gamepads_from_buffer`].
pub const GAMEPAD_BATCH_WIRE_VERSION: u32 = 2;
/// Maximum button count carried by a connected record.
pub const GAMEPAD_BATCH_MAX_BUTTONS: u8 = 32;
/// Maximum axis count carried by a connected record.
pub const GAMEPAD_BATCH_MAX_AXES: u8 = 16;

const RECORD_CONNECTED: u8 = 0;
const RECORD_UPDATE: u8 = 1;
const RECORD_DISCONNECTED: u8 = 2;
const STANDARD_BUTTON_START: u8 = 16;
const STANDARD_AXIS_RIGHT_TRIGGER: u8 = 5;

impl StateMachineInstance {
    /// Decode and dispatch one little-endian embedder gamepad batch.
    pub fn submit_gamepads_from_buffer(
        &mut self,
        artboard: &mut ArtboardInstance,
        data: &[u8],
    ) -> bool {
        let mut reader = GamepadBatchReader::new(data);
        if reader.read_u32() != Some(GAMEPAD_BATCH_WIRE_VERSION) {
            return false;
        }

        while !reader.is_empty() {
            match reader.read_u8() {
                Some(RECORD_CONNECTED) => {
                    let Some(snapshot) = read_connected_snapshot(&mut reader) else {
                        return false;
                    };
                    self.embedder_gamepads
                        .insert(snapshot.device_id, snapshot.clone());
                    let _ = self.gamepad_dispatch(
                        artboard,
                        ScriptListenerInvocation::GamepadConnected { snapshot },
                    );
                }
                Some(RECORD_UPDATE) => {
                    let Some(device_id) = reader.read_i32() else {
                        return false;
                    };
                    let Some(change_count) = reader.read_u8() else {
                        return false;
                    };
                    if !self.embedder_gamepads.contains_key(&device_id) {
                        return false;
                    }
                    let Some(changes) = read_changes(&mut reader, change_count) else {
                        return false;
                    };
                    let Some(final_state) = apply_changes(
                        self.embedder_gamepads.get_mut(&device_id),
                        changes.as_slice(),
                    ) else {
                        return false;
                    };
                    for change in changes {
                        let (standard_button_intent, standard_axis_intent) =
                            standard_intents(final_state.mapping, change);
                        let _ = self.gamepad_dispatch(
                            artboard,
                            ScriptListenerInvocation::GamepadEvent {
                                full_state: final_state.clone(),
                                change,
                                standard_button_intent,
                                standard_axis_intent,
                            },
                        );
                    }
                }
                Some(RECORD_DISCONNECTED) => {
                    let Some(device_id) = reader.read_i32() else {
                        return false;
                    };
                    self.embedder_gamepads.remove(&device_id);
                    let _ = self.gamepad_dispatch(
                        artboard,
                        ScriptListenerInvocation::GamepadDisconnected { device_id },
                    );
                }
                _ => return false,
            }
        }
        true
    }
}

fn read_connected_snapshot(reader: &mut GamepadBatchReader<'_>) -> Option<ScriptGamepadSnapshot> {
    let device_id = reader.read_i32()?;
    let mapping = reader.read_u8()?;
    let button_count = reader.read_u8()?;
    let axis_count = reader.read_u8()?;
    let _padding = reader.read_u8()?;
    if button_count > GAMEPAD_BATCH_MAX_BUTTONS || axis_count > GAMEPAD_BATCH_MAX_AXES {
        return None;
    }

    let mut button_values = Vec::with_capacity(usize::from(button_count));
    for _ in 0..button_count {
        button_values.push(reader.read_f32()?);
    }
    let mut axes = Vec::with_capacity(usize::from(axis_count));
    for _ in 0..axis_count {
        axes.push(reader.read_f32()?);
    }
    Some(ScriptGamepadSnapshot {
        device_id,
        button_mask: button_mask(&button_values),
        button_values,
        axes,
        mapping: if mapping == 0 {
            ScriptGamepadMappingKind::Standard
        } else {
            ScriptGamepadMappingKind::Unknown
        },
    })
}

fn read_changes(
    reader: &mut GamepadBatchReader<'_>,
    change_count: u8,
) -> Option<Vec<ScriptGamepadInputChange>> {
    let mut changes = Vec::with_capacity(usize::from(change_count));
    for _ in 0..change_count {
        let kind = reader.read_u8()?;
        let index = reader.read_u8()?;
        let value = reader.read_f32()?;
        changes.push(if kind == 0 {
            ScriptGamepadInputChange::Button { index, value }
        } else {
            ScriptGamepadInputChange::Axis { index, value }
        });
    }
    Some(changes)
}

fn apply_changes(
    snapshot: Option<&mut ScriptGamepadSnapshot>,
    changes: &[ScriptGamepadInputChange],
) -> Option<ScriptGamepadSnapshot> {
    let snapshot = snapshot?;
    for change in changes {
        match *change {
            ScriptGamepadInputChange::Button { index, value } => {
                if index >= GAMEPAD_BATCH_MAX_BUTTONS {
                    return None;
                }
                let index = usize::from(index);
                if snapshot.button_values.len() <= index {
                    snapshot.button_values.resize(index + 1, 0.0);
                }
                *snapshot.button_values.get_mut(index)? = value;
                snapshot.button_mask = button_mask(&snapshot.button_values);
            }
            ScriptGamepadInputChange::Axis { index, value } => {
                if index >= GAMEPAD_BATCH_MAX_AXES {
                    return None;
                }
                let index = usize::from(index);
                if snapshot.axes.len() <= index {
                    snapshot.axes.resize(index + 1, 0.0);
                }
                *snapshot.axes.get_mut(index)? = value;
            }
        }
    }
    Some(snapshot.clone())
}

fn button_mask(values: &[f32]) -> u64 {
    values
        .iter()
        .enumerate()
        .filter(|(_, value)| **value >= 0.5)
        .fold(0_u64, |mask, (index, _)| mask | (1_u64 << index))
}

fn standard_intents(
    mapping: ScriptGamepadMappingKind,
    change: ScriptGamepadInputChange,
) -> (Option<u32>, Option<u32>) {
    if mapping != ScriptGamepadMappingKind::Standard {
        return (None, None);
    }
    match change {
        ScriptGamepadInputChange::Button { index, .. } if index <= STANDARD_BUTTON_START => {
            (Some(u32::from(index)), None)
        }
        ScriptGamepadInputChange::Axis { index, .. } if index <= STANDARD_AXIS_RIGHT_TRIGGER => {
            (None, Some(u32::from(index)))
        }
        _ => (None, None),
    }
}

struct GamepadBatchReader<'a> {
    remaining: &'a [u8],
}

impl<'a> GamepadBatchReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { remaining: data }
    }

    fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }

    fn read_bytes<const N: usize>(&mut self) -> Option<[u8; N]> {
        let value = self.remaining.get(..N)?;
        self.remaining = self.remaining.get(N..)?;
        value.try_into().ok()
    }

    fn read_u8(&mut self) -> Option<u8> {
        Some(self.read_bytes::<1>()?[0])
    }

    fn read_u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.read_bytes()?))
    }

    fn read_i32(&mut self) -> Option<i32> {
        Some(i32::from_le_bytes(self.read_bytes()?))
    }

    fn read_f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.read_bytes()?))
    }
}
