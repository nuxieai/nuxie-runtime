//! A host that drives the player's artboard from outside the file.
//!
//! Inside one ordinary step the player advances, hands the step's reported
//! events to the driver, applies the property writes the driver answers with,
//! lays out and advances by zero within the same frame, and asks again with
//! whatever the settle reported, up to [`MAX_DRIVER_ROUNDS`]. The frame drawn
//! after the step already shows every write.
use super::OwnedPlayerEvent;
/// One step of a [`SceneObject`]'s path, the type a reported event's path
/// is made of.
pub use crate::RuntimeGeometryHitPathSegment;
/// One keyed list row of a [`SceneObject`]'s path, the type a reported
/// event's row keys are made of.
pub use crate::RuntimeListRowKey;
use crate::mechanical_port::source::animation::{
    nested_state_machine::NestedStateMachine,
    state_machine_instance::RuntimeStateMachineInstanceHandle,
};
use crate::mechanical_port::source::artboard_component_list::{
    ArtboardComponentList, KeyedRowRefusal,
};
use crate::mechanical_port::source::{
    core::field_types::{
        core_bool_type::CoreBoolType, core_color_type::CoreColorType,
        core_double_type::CoreDoubleType, core_string_type::CoreStringType,
        core_uint_type::CoreUintType,
    },
    generated::core_registry::CoreRegistry,
};
use crate::{ArtboardInstance, CoreHandle, RuntimeNestedArtboardOccurrence, StateMachineInstance};

/// The most driver rounds one step runs. A driver still writing in the last
/// round is cut off: its writes are laid out, the step returns unsettled, and
/// whatever the last settle reported is handed to the driver at the start of
/// the next step.
pub const MAX_DRIVER_ROUNDS: usize = 8;

/// One object of the player's artboard, or of one nested artboard
/// occurrence under it.
///
/// `path` leads from the player's artboard down to the occurrence that holds
/// the object: the host object of each nested artboard or keyed list on the
/// way, each by its local id in the artboard the path has reached so far,
/// with that artboard's file-global id. `rows` names the row of each keyed
/// list step, in path order: the list's host, as its step names it, and the
/// key the row was mounted by. These are the path and the row keys a
/// reported event carries (`StateMachineEventContext::path` and `rows`), so
/// a driver can hand back what it heard. An empty path names the player's
/// own artboard. `local_id` is the object's local id in the artboard the
/// path leads to.
///
/// A path through a list that is not keyed leads nowhere.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct SceneObject {
    pub local_id: usize,
    pub path: Vec<RuntimeGeometryHitPathSegment>,
    pub rows: Vec<RuntimeListRowKey>,
}

impl SceneObject {
    /// An object of the player's own artboard.
    pub fn root(local_id: usize) -> Self {
        Self {
            local_id,
            path: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// An object of the nested artboard occurrence `path` leads to.
    pub fn nested(path: Vec<RuntimeGeometryHitPathSegment>, local_id: usize) -> Self {
        Self {
            local_id,
            path,
            rows: Vec::new(),
        }
    }

    /// An object of the occurrence `path` leads to through the keyed list
    /// rows `rows` names.
    pub fn in_rows(
        path: Vec<RuntimeGeometryHitPathSegment>,
        rows: Vec<RuntimeListRowKey>,
        local_id: usize,
    ) -> Self {
        Self {
            local_id,
            path,
            rows,
        }
    }
}

/// Why a write or reset was refused. A refused call changes nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneWriteError {
    /// The artboard the path leads to has no object with this local id.
    NotFound,
    /// The path leads to no mounted occurrence: a step names no object, an
    /// object that hosts no nested artboard, or an artboard other than the
    /// one the path has reached, or the occurrence it names is gone.
    PathNotFound,
    /// The object has no property of this key and kind.
    UnknownProperty,
    /// The value cannot be written: a number that is not finite.
    InvalidValue,
    /// A row operation named an object that is not a keyed list: not a list,
    /// or a list whose record names no artboard for keyed rows.
    NotKeyedList,
    /// The artboard a mount names, which the keyed list declares, is not one
    /// the file has, so no row can be made from it.
    NoRowArtboard,
    /// A mount named an artboard the keyed list does not declare for its
    /// rows: neither its `rowArtboardId` nor a `KeyedRowArtboard` child's.
    NotRowArtboard,
    /// A row with this key is already mounted in the list.
    KeyExists,
    /// No row with this key is mounted in the list.
    KeyNotFound,
    /// The place is past the list's last row.
    PlaceOutOfRange,
    /// An instance replacement named an object whose instance the driver
    /// may not replace: it hosts no nested artboard, or hosts nothing now,
    /// or a live move, a bound artboard property or a view model owns what
    /// it holds.
    NotReplaceable,
}

impl From<KeyedRowRefusal> for SceneWriteError {
    fn from(refusal: KeyedRowRefusal) -> Self {
        match refusal {
            KeyedRowRefusal::NotKeyed => Self::NotKeyedList,
            KeyedRowRefusal::NoRowArtboard => Self::NoRowArtboard,
            KeyedRowRefusal::NotRowArtboard => Self::NotRowArtboard,
            KeyedRowRefusal::KeyExists => Self::KeyExists,
            KeyedRowRefusal::KeyNotFound => Self::KeyNotFound,
            KeyedRowRefusal::PlaceOutOfRange => Self::PlaceOutOfRange,
        }
    }
}

/// Writes into the player's artboard and the nested artboard occurrences
/// under it, valid for one driver round.
///
/// An object inside an occurrence is reached through the occurrence its
/// path resolves to, and written through the same setter as an object of
/// the player's artboard. A write goes through the property's own setter,
/// so it marks layout and paint exactly as an animation or a bind writing
/// the same property would.
/// Writing the value a property already holds changes nothing and does not
/// count as a change.
///
/// Row operations on a keyed list apply in the order given. A mounted row is
/// made at once, so a write into it in the same round lands. Each list a
/// round changed takes its new order of rows once, when the round ends.
pub struct SceneWriter<'a> {
    artboard: &'a ArtboardInstance,
    machine: Option<&'a StateMachineInstance>,
    changes: usize,
    /// The keyed lists this round's row operations changed.
    row_lists: Vec<CoreHandle>,
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
            row_lists: Vec::new(),
        }
    }

    /// End the round: each keyed list a row operation changed takes its new
    /// order of rows, once.
    pub(crate) fn finish_round(&mut self) {
        for list in std::mem::take(&mut self.row_lists) {
            ArtboardComponentList::commit_keyed_rows_occurrence(&list);
        }
    }

    /// How many calls in this round changed the scene.
    pub fn changes(&self) -> usize {
        self.changes
    }

    /// The occurrence `object`'s path leads to, or None for an object of
    /// the player's own artboard.
    fn occurrence(
        &self,
        object: &SceneObject,
    ) -> Result<Option<RuntimeNestedArtboardOccurrence>, SceneWriteError> {
        if object.path.is_empty() {
            return if object.rows.is_empty() {
                Ok(None)
            } else {
                Err(SceneWriteError::PathNotFound)
            };
        }
        self.artboard
            .nested_artboard_occurrence_on_keyed_path(&object.path, &object.rows)
            .map(Some)
            .ok_or(SceneWriteError::PathNotFound)
    }

    fn object_handle(&self, object: &SceneObject) -> Result<CoreHandle, SceneWriteError> {
        match self.occurrence(object)? {
            None => self.artboard.object_handle(object.local_id),
            Some(occurrence) => occurrence.object_handle(object.local_id),
        }
        .ok_or(SceneWriteError::NotFound)
    }

    fn property(
        &self,
        object: &SceneObject,
        key: u16,
        field: i32,
    ) -> Result<CoreHandle, SceneWriteError> {
        let handle = self.object_handle(object)?;
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
    /// A reset returns the engine's state holders under the object to their
    /// start and leaves every property value alone, whoever wrote it. So a
    /// property that an animation or a state wrote, and that the starting
    /// state does not key, keeps its value until something writes it.
    ///
    /// The state holders a reset returns: the playback of nested simple and
    /// remap animations, which is rebuilt; nested machines, whose layers are
    /// rebuilt at their entry states; scroll views, whose offsets go back to
    /// the ones the file gives them; text fields, whose text goes back to the
    /// text last written to them; list rows, rebuilt with their machines; the
    /// artboard a paint paints with, rebuilt; and layouts' and layout
    /// participants' motion in flight, which ends where it was going. The
    /// reset then plays each host's first frame, so its starting states
    /// apply what they key. The presses and hovers the player's listeners
    /// hold there are dropped. A rebuilt machine layer draws from the shared
    /// random source where it stands; the source is not restarted.
    ///
    /// A reset leaves as they are, on a nested machine, the values of its
    /// inputs, which its listeners and its host set, and its pointer latches;
    /// on a scroll view, whether a drag holds it; on a text field, whether it
    /// has focus; script state; sounds already playing; video playback; text
    /// style artboards; values that binds carry from the view model,
    /// converters easing them included; and the inputs of the player's own
    /// machine, which is under no object. A nested host that skips frames
    /// (paused, collapsed, or playing at a fixed rate) skips the reset's
    /// first frame too, so what its playback keys holds its values until it
    /// next advances. The reset module lists every kind.
    ///
    /// A reset of an object inside a nested occurrence reaches only that
    /// occurrence, and there the machines its host plays on it drop the
    /// presses and hovers they hold under the object, as the player's own
    /// machine does in the player's artboard. Those machines belong to the
    /// host, one artboard up, so a reset that names the host returns them to
    /// their entry states and a reset inside the occurrence does not. Inside a
    /// keyed list's row, that is the machine the list plays on the row; a
    /// reset that names the list rebuilds each row it lays out, under its key
    /// and in its place, and keeps none for reuse. Whether anything was reset.
    pub fn reset(&mut self, object: &SceneObject) -> Result<bool, SceneWriteError> {
        let changed = match self.occurrence(object)? {
            None => {
                let handle = self
                    .artboard
                    .object_handle(object.local_id)
                    .ok_or(SceneWriteError::NotFound)?;
                let machines: Vec<_> = self
                    .machine
                    .map(|m| m.native_handle())
                    .into_iter()
                    .collect();
                super::reset::reset_subtree(&self.artboard.native_handle(), &machines, &handle)
            }
            Some(occurrence) => {
                let handle = occurrence
                    .object_handle(object.local_id)
                    .ok_or(SceneWriteError::NotFound)?;
                let (Some(artboard), Some(host)) = (occurrence.artboard(), occurrence.host())
                else {
                    return Err(SceneWriteError::PathNotFound);
                };
                let machines = hosted_machines(&host, occurrence.host_list_item().as_ref());
                super::reset::reset_subtree(&artboard, &machines, &handle)
            }
        };
        if changed {
            self.changes += 1;
        }
        Ok(changed)
    }

    /// Replace the instance the nested artboard host `host` holds with a
    /// fresh instance of the same artboard, made from the file as a list
    /// makes a row: its objects hold the file's values, its scripts run
    /// their setup, and nothing the old instance played, scrolled, typed or
    /// was written reaches it. The old instance is discarded at once, and
    /// its scripts are disposed with it.
    ///
    /// The host keeps its own records and its nested animations; each
    /// animation and machine starts again on the new instance at its entry
    /// with its authored inputs, heard by the machines that heard it before.
    /// The host's focus scope, accessibility place and layout take the new
    /// instance, which plays its first frame, as a reset's hosts do, so an
    /// object inside it can be written in the same round. A use's display,
    /// which lives on the hosted artboard's own record, is the file's again.
    ///
    /// A host a live move carries through, whose artboard a bound property
    /// chooses, or that carries a view model instance is refused
    /// ([`SceneWriteError::NotReplaceable`]). Always a change otherwise.
    pub fn replace_instance(&mut self, host: &SceneObject) -> Result<bool, SceneWriteError> {
        let (artboard, machines, handle) = match self.occurrence(host)? {
            None => (
                self.artboard.native_handle(),
                self.machine
                    .map(|m| m.native_handle())
                    .into_iter()
                    .collect::<Vec<_>>(),
                self.artboard
                    .object_handle(host.local_id)
                    .ok_or(SceneWriteError::NotFound)?,
            ),
            Some(occurrence) => {
                let handle = occurrence
                    .object_handle(host.local_id)
                    .ok_or(SceneWriteError::NotFound)?;
                let (Some(artboard), Some(owner)) = (occurrence.artboard(), occurrence.host())
                else {
                    return Err(SceneWriteError::PathNotFound);
                };
                let machines = hosted_machines(&owner, occurrence.host_list_item().as_ref());
                (artboard, machines, handle)
            }
        };
        super::renew::renew_hosted_instance(&artboard, &machines, &handle)
            .map_err(|_| SceneWriteError::NotReplaceable)?;
        self.changes += 1;
        Ok(true)
    }

    /// The keyed list `list` names.
    fn keyed_list(&self, list: &SceneObject) -> Result<CoreHandle, SceneWriteError> {
        let handle = self.object_handle(list)?;
        let keyed = handle
            .with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::is_keyed)
            .unwrap_or(false);
        if !keyed {
            return Err(SceneWriteError::NotKeyedList);
        }
        Ok(handle)
    }

    fn row_changed(&mut self, list: CoreHandle) {
        self.changes += 1;
        if !self.row_lists.contains(&list) {
            self.row_lists.push(list);
        }
    }

    /// Mount a row by `key` at the end of the keyed list `list`: a fresh
    /// instance of the list's row artboard (its `rowArtboardId`) with its
    /// machine at entry and the file's values. An object inside the row can
    /// be written in the same round. Always a change.
    pub fn mount_row(&mut self, list: &SceneObject, key: &[u8]) -> Result<bool, SceneWriteError> {
        let handle = self.keyed_list(list)?;
        let artboard = handle
            .with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::row_artboard_id)
            .ok_or(SceneWriteError::NotKeyedList)?;
        self.mount_keyed_row(handle, key, artboard)
    }

    /// Mount a row by `key` at the end of the keyed list `list`, as
    /// [`Self::mount_row`] does, made from `artboard`, an index into the
    /// file's artboards that the list declares: its `rowArtboardId` or a
    /// `KeyedRowArtboard` child's. An artboard the list does not declare is
    /// refused with [`SceneWriteError::NotRowArtboard`]. Always a change.
    pub fn mount_row_from(
        &mut self,
        list: &SceneObject,
        key: &[u8],
        artboard: u32,
    ) -> Result<bool, SceneWriteError> {
        let handle = self.keyed_list(list)?;
        self.mount_keyed_row(handle, key, artboard)
    }

    fn mount_keyed_row(
        &mut self,
        list: CoreHandle,
        key: &[u8],
        artboard: u32,
    ) -> Result<bool, SceneWriteError> {
        ArtboardComponentList::mount_keyed_row_occurrence(&list, key, artboard)?;
        self.row_changed(list);
        Ok(true)
    }

    /// Remove the row mounted by `key` from the keyed list `list`. Its
    /// instance, machine and every state they hold are discarded; a later
    /// mount of the same key makes a new row. Always a change.
    pub fn remove_row(&mut self, list: &SceneObject, key: &[u8]) -> Result<bool, SceneWriteError> {
        let handle = self.keyed_list(list)?;
        ArtboardComponentList::remove_keyed_row_occurrence(&handle, key)?;
        self.row_changed(handle);
        Ok(true)
    }

    /// Move the row mounted by `key` to `place` among the keyed list's rows,
    /// 0 being the first; the other rows keep their order. The row keeps its
    /// instance and all its state. Whether its place changed.
    pub fn move_row(
        &mut self,
        list: &SceneObject,
        key: &[u8],
        place: usize,
    ) -> Result<bool, SceneWriteError> {
        let handle = self.keyed_list(list)?;
        let moved = ArtboardComponentList::move_keyed_row_occurrence(&handle, key, place)?;
        if moved {
            self.row_changed(handle);
        }
        Ok(moved)
    }
}

/// The machines a host plays on the artboard it hosts: a nested artboard
/// host's nested machines, or the machine a list plays on the row of `row`.
fn hosted_machines(
    host: &CoreHandle,
    row: Option<&CoreHandle>,
) -> Vec<RuntimeStateMachineInstanceHandle> {
    if let Some(row) = row {
        return host
            .with_downcast::<ArtboardComponentList, _>(|list| list.row_state_machine(row))
            .flatten()
            .into_iter()
            .collect();
    }
    host.with(|host| {
        host.as_nested_artboard()
            .map(|nested| nested.nested_animations().to_vec())
    })
    .flatten()
    .unwrap_or_default()
    .iter()
    .filter_map(|animation| {
        animation
            .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
            .flatten()
    })
    .collect()
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
