//! A host that drives the player's artboard from outside the file.
//!
//! Inside one ordinary step the player advances, hands the step's reported
//! events to the driver, applies the property writes the driver answers with,
//! lays out and advances by zero within the same frame, and asks again with
//! whatever the settle reported, up to [`MAX_DRIVER_ROUNDS`]. The frame drawn
//! after the step already shows every write.
use super::OwnedPlayerEvent;
use crate::mechanical_port::source::{
    core::field_types::{
        core_bool_type::CoreBoolType, core_color_type::CoreColorType,
        core_double_type::CoreDoubleType, core_string_type::CoreStringType,
        core_uint_type::CoreUintType,
    },
    generated::core_registry::CoreRegistry,
};
use crate::{ArtboardInstance, CoreHandle, StateMachineInstance};

/// The most driver rounds one step runs. A driver still writing in the last
/// round is cut off: its writes are laid out, the step returns unsettled, and
/// whatever the last settle reported is handed to the driver at the start of
/// the next step.
pub const MAX_DRIVER_ROUNDS: usize = 8;

/// One object of the player's artboard, by its local id in that artboard.
///
/// Today an address names an object of the root artboard only. The type is
/// non-exhaustive so an address of one nested occurrence or one list row can
/// be added as a path beside `local_id` without changing these callers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct SceneObject {
    pub local_id: usize,
}

impl SceneObject {
    /// An object of the root artboard.
    pub fn root(local_id: usize) -> Self {
        Self { local_id }
    }
}

/// Why a write or reset was refused. A refused call changes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneWriteError {
    /// The artboard has no object with this local id.
    NotFound,
    /// The object has no property of this key and kind.
    UnknownProperty,
    /// The value cannot be written: a number that is not finite.
    InvalidValue,
}

/// Writes into the player's artboard, valid for one driver round.
///
/// A write goes through the property's own setter, so it marks layout and
/// paint exactly as an animation or a bind writing the same property would.
/// Writing the value a property already holds changes nothing and does not
/// count as a change.
pub struct SceneWriter<'a> {
    artboard: &'a ArtboardInstance,
    machine: Option<&'a StateMachineInstance>,
    changes: usize,
}

impl<'a> SceneWriter<'a> {
    pub(crate) fn new(
        artboard: &'a ArtboardInstance,
        machine: Option<&'a StateMachineInstance>,
    ) -> Self {
        Self {
            artboard,
            machine,
            changes: 0,
        }
    }

    /// How many calls in this round changed the scene.
    pub fn changes(&self) -> usize {
        self.changes
    }

    fn property(
        &self,
        object: &SceneObject,
        key: u16,
        field: i32,
    ) -> Result<CoreHandle, SceneWriteError> {
        let handle = self
            .artboard
            .object_handle(object.local_id)
            .ok_or(SceneWriteError::NotFound)?;
        let supported = CoreRegistry::property_field_id(i32::from(key)) == field
            && handle
                .with(|object| CoreRegistry::object_supports_property(object, u32::from(key)))
                .unwrap_or(false);
        if !supported {
            return Err(SceneWriteError::UnknownProperty);
        }
        Ok(handle)
    }

    fn written<T: PartialEq>(
        &mut self,
        handle: &CoreHandle,
        value: T,
        get: impl Fn(&CoreHandle) -> Option<T>,
        set: impl FnOnce(&CoreHandle, T) -> bool,
    ) -> Result<bool, SceneWriteError> {
        let before = get(handle).ok_or(SceneWriteError::NotFound)?;
        if before == value {
            return Ok(false);
        }
        set(handle, value);
        let changed = get(handle).is_some_and(|after| after != before);
        if changed {
            self.changes += 1;
        }
        Ok(changed)
    }

    /// Set a number property. Whether its value changed.
    pub fn set_number(
        &mut self,
        object: &SceneObject,
        key: u16,
        value: f32,
    ) -> Result<bool, SceneWriteError> {
        if !value.is_finite() {
            return Err(SceneWriteError::InvalidValue);
        }
        let handle = self.property(object, key, CoreDoubleType::ID)?;
        let key = i32::from(key);
        self.written(
            &handle,
            value,
            |h| CoreRegistry::get_double_handle(h, key),
            |h, v| CoreRegistry::set_double_handle(h, key, v),
        )
    }

    /// Set an unsigned property: a count, an id or an enum such as a layout
    /// record's `display`. Whether its value changed.
    pub fn set_unsigned(
        &mut self,
        object: &SceneObject,
        key: u16,
        value: u32,
    ) -> Result<bool, SceneWriteError> {
        let handle = self.property(object, key, CoreUintType::ID)?;
        let key = i32::from(key);
        self.written(
            &handle,
            value,
            |h| CoreRegistry::get_uint_handle(h, key),
            |h, v| CoreRegistry::set_uint_handle(h, key, v),
        )
    }

    /// Set a boolean property. Whether its value changed.
    pub fn set_bool(
        &mut self,
        object: &SceneObject,
        key: u16,
        value: bool,
    ) -> Result<bool, SceneWriteError> {
        let handle = self.property(object, key, CoreBoolType::ID)?;
        let key = i32::from(key);
        self.written(
            &handle,
            value,
            |h| CoreRegistry::get_bool_handle(h, key),
            |h, v| CoreRegistry::set_bool_handle(h, key, v),
        )
    }

    /// Set a color property, as 0xAARRGGBB. Whether its value changed.
    pub fn set_color(
        &mut self,
        object: &SceneObject,
        key: u16,
        value: u32,
    ) -> Result<bool, SceneWriteError> {
        let handle = self.property(object, key, CoreColorType::ID)?;
        let key = i32::from(key);
        self.written(
            &handle,
            value as i32,
            |h| CoreRegistry::get_color_handle(h, key),
            |h, v| CoreRegistry::set_color_handle(h, key, v),
        )
    }

    /// Set a text property, such as a text run's text. Whether its value
    /// changed.
    pub fn set_text(
        &mut self,
        object: &SceneObject,
        key: u16,
        value: &str,
    ) -> Result<bool, SceneWriteError> {
        let handle = self.property(object, key, CoreStringType::ID)?;
        let key = i32::from(key);
        self.written(
            &handle,
            value.to_owned(),
            |h| CoreRegistry::get_string_handle(h, key),
            |h, v| CoreRegistry::set_string_handle(h, key, v),
        )
    }

    /// Reset what the engine holds under `object`, the object included.
    ///
    /// After a reset inside a step, everything the engine holds under that
    /// object is what a freshly made instance of the same file would hold
    /// after its first settle, and the frame drawn after that same step
    /// shows it. Property values a host wrote are the only difference, and
    /// they stay.
    ///
    /// So scroll positions go back to the offsets the file gave them, typed
    /// text to the text last written to its field, the machines of hosted
    /// artboards (of any host type, and of list rows) to their entry states
    /// with their states' animations applied at the start, simple animations
    /// to their starting pose, and the presses and hovers the player's
    /// listeners hold there are dropped.
    ///
    /// A reset does not return, on a hosted state machine, the input values
    /// its listeners set or its pointer latches (its scroll latch, drag
    /// holders, drag owners and disabled pointer ids), nor the inputs of the
    /// player's own machine, which is under no object. Whether anything was
    /// reset.
    pub fn reset(&mut self, object: &SceneObject) -> Result<bool, SceneWriteError> {
        let handle = self
            .artboard
            .object_handle(object.local_id)
            .ok_or(SceneWriteError::NotFound)?;
        let changed = super::reset::reset_subtree(self.artboard, self.machine, &handle);
        if changed {
            self.changes += 1;
        }
        Ok(changed)
    }
}

/// One round of a step, as the driver sees it.
pub struct DriverRound<'a> {
    /// 0 for the step's first round.
    pub index: usize,
    /// The events reported since the previous round: in the first round, the
    /// step's own reports and any the previous step's last settle made after
    /// its driver was cut off.
    pub reports: &'a [OwnedPlayerEvent],
}

/// A driver's failure. The step fails and the occurrence is poisoned, since
/// the step's advance and any earlier writes cannot be undone.
#[derive(Clone, Debug)]
pub struct DriverFault(pub String);

/// The host side of the player's step. A driver set on a [`super::PagePlayer`]
/// is asked once in every ordinary step, after the advance, and again after
/// each round that changed the scene.
pub trait HostDriver {
    fn drive(
        &mut self,
        round: DriverRound<'_>,
        scene: &mut SceneWriter<'_>,
    ) -> Result<(), DriverFault>;
}
