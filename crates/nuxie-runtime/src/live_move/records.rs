//! Nuxie-owned published live-move records. Keys come from defs/nuxie.
use crate::source::{
    component::Component,
    container_component::ContainerComponent,
    core::{
        Core, CoreHandle, CoreObject, CoreType, PropertySetterCompletion,
        binary_reader::BinaryReader,
        field_types::{
            core_bool_type::CoreBoolType, core_callback_type::CallbackData,
            core_uint_type::CoreUintType,
        },
    },
    core_context::CoreContext,
    generated::{
        component_base::{ComponentBase, ComponentBaseCallbacks},
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
    },
    importers::import_stack::ImportStack,
    status_code::StatusCode,
};
use std::any::Any;

pub const UNIT_KEY: u16 = 62000;
pub const ROLE: u16 = 62001;
pub const SIZE_MODE: u16 = 62002;
pub const PROGRESS: u16 = 62003;
pub const PATH_X: u16 = 62004;
pub const PATH_Y: u16 = 62005;
pub const STEP_NAME: u16 = 62006;
pub const FROM_ARTBOARD: u16 = 62007;
pub const TO_ARTBOARD: u16 = 62008;
pub const INCOMING_UNIT_KEY: u16 = 62009;
pub const OUTGOING_UNIT_KEY: u16 = 62010;
pub const INCOMING_FLIGHT_KEY: u16 = 62011;
pub const OUTGOING_FLIGHT_KEY: u16 = 62012;
pub const FLIGHT_KEY: u16 = 62013;
pub const FADE: u16 = 62014;
pub const CORNER_TOP_LEFT: u16 = 62015;
pub const CORNER_TOP_RIGHT: u16 = 62016;
pub const CORNER_BOTTOM_RIGHT: u16 = 62017;
pub const CORNER_BOTTOM_LEFT: u16 = 62018;
pub const SHADOW_ALPHA: u16 = 62019;
pub const SHADOW_RED: u16 = 62020;
pub const SHADOW_GREEN: u16 = 62021;
pub const SHADOW_BLUE: u16 = 62022;
pub const SHADOW_BLUR: u16 = 62023;
pub const SHADOW_OFFSET_X: u16 = 62024;
pub const SHADOW_OFFSET_Y: u16 = 62025;
pub const ZOOM: u16 = 62026;
pub const TRANSFERS: u16 = 62029;
pub const FLIGHT_ANIMATION_ID: u16 = 62030;
/// A `LiveFlight`'s `key`: unique and nonzero among its Screen's flight
/// records; a step unit's flight keys name it.
pub const RECORD_KEY: u16 = 62031;
/// A `LiveFlight`'s `sizeMode`: 0 scaled, 1 laid out at the flight size.
pub const RECORD_SIZE_MODE: u16 = 62032;
/// A `LiveFlight`'s first control, `progress`. Its fifteen controls follow
/// in `Values::control`'s order to 62047.
pub const RECORD_PROGRESS: u16 = 62033;
/// The keyable controls a flight slot or a flight record carries.
pub const CONTROLS: usize = 15;
/// The seven shadow controls with an alpha below 0: no shadow.
pub const NO_SHADOW: [f32; 7] = [-1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

#[derive(Clone, Default)]
pub struct Values {
    /// A flight record's `key`.
    pub key: u32,
    pub unit_key: u32,
    pub role: u32,
    pub size_mode: u32,
    pub progress: f32,
    pub path_x: f32,
    pub path_y: f32,
    pub name: String,
    pub from_artboard: String,
    pub to_artboard: String,
    pub incoming_unit_key: u32,
    pub outgoing_unit_key: u32,
    pub incoming_flight_key: u32,
    pub outgoing_flight_key: u32,
    /// How a step unit's two versions fly: 0 apart, 1 as one box leaving the
    /// source, 2 as one box returning.
    pub zoom: u32,
    /// A step unit's `transfers`: one instance flies, from the leaving
    /// Screen's slot to the arriving Screen's. Unset, each Screen's own copy
    /// flies in its own flight slot and lands back in its own slot.
    pub transfers: bool,
    /// A step's `flightAnimationId`: its flight timeline's index among the
    /// animations of the Screen holding its flight slots. `u32::MAX`, Rive's
    /// missing id, when the step names none.
    pub flight_animation: u32,
    pub flight_key: u32,
    pub fade: f32,
    /// Top left, top right, bottom right, bottom left; -1 sets no corner.
    pub corners: [f32; 4],
    /// Alpha, red, green, blue, blur, offset x, offset y.
    pub shadow: [f32; 7],
}
impl Values {
    /// A flight control by its index: progress, path x, path y, fade, the
    /// four corners, then the seven shadow controls.
    pub fn control(&self, index: usize) -> f32 {
        match index {
            0 => self.progress,
            1 => self.path_x,
            2 => self.path_y,
            3 => self.fade,
            4..=7 => self.corners[index - 4],
            _ => self.shadow[index - 8],
        }
    }
    fn control_mut(&mut self, index: usize) -> &mut f32 {
        match index {
            0 => &mut self.progress,
            1 => &mut self.path_x,
            2 => &mut self.path_y,
            3 => &mut self.fade,
            4..=7 => &mut self.corners[index - 4],
            _ => &mut self.shadow[index - 8],
        }
    }
    /// Whether a step unit flies one instance, from the leaving Screen's slot
    /// to the arriving Screen's: its own `transfers`. Otherwise each Screen's
    /// copy flies in its own flight slot, whether or not their keys agree.
    pub(super) fn transfers(&self) -> bool {
        self.transfers
    }
    /// A step unit's `zoom` as the plan reads it: 1 or 2, and otherwise 0, the
    /// implicit default. It is read only for a unit that doesn't transfer.
    /// Validity doesn't check it, so a record with any other value still
    /// flies, apart.
    pub(super) fn read_zoom(&self) -> u32 {
        match self.zoom {
            1 | 2 => self.zoom,
            _ => 0,
        }
    }
}
// Shared by import and step preflight. A unit that transfers names one
// flight, and two copies name two; every other key belongs to just one
// mapping in the complete step.
pub(super) fn validate_step_units<'a>(
    units: impl IntoIterator<Item = &'a Values>,
) -> anyhow::Result<()> {
    let (mut incoming, mut outgoing, mut flights) = (
        std::collections::HashSet::new(),
        std::collections::HashSet::new(),
        std::collections::HashSet::new(),
    );
    for unit in units {
        anyhow::ensure!(
            unit.transfers() == (unit.incoming_flight_key == unit.outgoing_flight_key),
            "a transferred unit names one flight, two copies name two"
        );
        anyhow::ensure!(
            outgoing.insert(unit.outgoing_unit_key),
            "duplicate outgoing unit in step"
        );
        anyhow::ensure!(
            incoming.insert(unit.incoming_unit_key),
            "duplicate incoming unit in step"
        );
        anyhow::ensure!(
            flights.insert(unit.outgoing_flight_key),
            "overlapping flight keys in step"
        );
        if unit.incoming_flight_key != unit.outgoing_flight_key {
            anyhow::ensure!(
                flights.insert(unit.incoming_flight_key),
                "overlapping flight keys in step"
            );
        }
    }
    Ok(())
}
pub struct LiveRecord<const TYPE: u16> {
    component: ContainerComponent,
    pub values: Values,
}
pub type LiveSlot = LiveRecord<62000>;
pub type LiveStep = LiveRecord<62001>;
pub type LiveStepUnit = LiveRecord<62002>;
/// One flying copy of one in-place step: the controls its flight timeline
/// keys (docs/live-moves.md, Flying in place). A child of the Screen that
/// holds the step's flight timeline; it hosts nothing.
pub type LiveFlight = LiveRecord<62003>;
/// A flight control's index (`Values::control`) for a key of a flight slot
/// or a flight record.
fn control_index(key: u16) -> Option<usize> {
    match key {
        PROGRESS..=PATH_Y => Some(usize::from(key - PROGRESS)),
        FADE..=SHADOW_OFFSET_Y => Some(usize::from(key - FADE) + 3),
        RECORD_PROGRESS..=62047 => Some(usize::from(key - RECORD_PROGRESS)),
        _ => None,
    }
}
impl<const TYPE: u16> Default for LiveRecord<TYPE> {
    fn default() -> Self {
        Self {
            component: ContainerComponent::default(),
            values: Values {
                fade: 1.0,
                flight_animation: u32::MAX,
                corners: [-1.0; 4],
                shadow: NO_SHADOW,
                ..Values::default()
            },
        }
    }
}
fn subtype<const TYPE: u16>(key: u16) -> bool {
    key == TYPE
        || crate::source::generated::container_component_base::ContainerComponentBase::is_type_of(
            key,
        )
}
struct NoCallbacks;
impl ComponentBaseCallbacks for NoCallbacks {
    fn notify_property_changed(&mut self, _: u16) {}
}
impl<const TYPE: u16> LiveRecord<TYPE> {
    fn owns(key: u16) -> bool {
        match TYPE {
            62000 => (62000..=62005).contains(&key) || (62013..=62025).contains(&key),
            62001 => (62006..=62008).contains(&key) || key == FLIGHT_ANIMATION_ID,
            62002 => (62009..=62012).contains(&key) || key == ZOOM || key == TRANSFERS,
            62003 => (RECORD_KEY..=62047).contains(&key),
            _ => false,
        }
    }
    fn valid_values(&self) -> bool {
        match TYPE {
            62000 => {
                self.values.role <= 1
                    && self.values.size_mode <= 1
                    && self.values.progress.is_finite()
                    && self.values.path_x.is_finite()
                    && self.values.path_y.is_finite()
                    && self.values.fade.is_finite()
                    && self.values.corners.iter().all(|corner| corner.is_finite())
                    && self.values.shadow.iter().all(|value| value.is_finite())
                    && (self.values.role != 0
                        || (self.values.progress == 0.0
                            && self.values.path_x == 0.0
                            && self.values.path_y == 0.0
                            && self.values.flight_key == 0
                            && self.values.fade == 1.0
                            && self.values.corners == [-1.0; 4]
                            && self.values.shadow == NO_SHADOW))
            }
            62001 => {
                !self.values.name.is_empty()
                    && !self.values.from_artboard.is_empty()
                    && !self.values.to_artboard.is_empty()
            }
            // A step unit's flight rule is the whole step's, checked at
            // import, where a breach refuses the file (validate_step_units).
            62002 => true,
            62003 => {
                self.values.key != 0
                    && self.values.size_mode <= 1
                    && (0..CONTROLS).all(|index| self.values.control(index).is_finite())
            }
            _ => false,
        }
    }
    pub fn parent(&self) -> Option<CoreHandle> {
        self.component.parent_handle()
    }
}
impl<const TYPE: u16> CoreType for LiveRecord<TYPE> {
    const TYPE_KEY: u16 = TYPE;
}
impl<const TYPE: u16> CoreObject for LiveRecord<TYPE> {
    fn core(&self) -> &Core {
        &self.component.base.base.base.base
    }
    fn core_mut(&mut self) -> &mut Core {
        &mut self.component.base.base.base.base
    }
    fn core_type(&self) -> u16 {
        TYPE
    }
    fn is_type_of(&self, key: u16) -> bool {
        subtype::<TYPE>(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        subtype::<TYPE>
    }
    fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
        let mut copy = Self::default();
        copy.component
            .base
            .base
            .base
            .copy(&self.component.base.base.base, &mut NoCallbacks);
        copy.values = self.values.clone();
        Some(Box::new(copy))
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        if Self::owns(key) {
            match key {
                UNIT_KEY => self.values.unit_key = CoreUintType::deserialize(reader),
                ROLE => self.values.role = CoreUintType::deserialize(reader),
                SIZE_MODE => self.values.size_mode = CoreUintType::deserialize(reader),
                INCOMING_UNIT_KEY => {
                    self.values.incoming_unit_key = CoreUintType::deserialize(reader)
                }
                OUTGOING_UNIT_KEY => {
                    self.values.outgoing_unit_key = CoreUintType::deserialize(reader)
                }
                INCOMING_FLIGHT_KEY => {
                    self.values.incoming_flight_key = CoreUintType::deserialize(reader)
                }
                OUTGOING_FLIGHT_KEY => {
                    self.values.outgoing_flight_key = CoreUintType::deserialize(reader)
                }
                ZOOM => self.values.zoom = CoreUintType::deserialize(reader),
                TRANSFERS => self.values.transfers = CoreBoolType::deserialize(reader),
                FLIGHT_ANIMATION_ID => {
                    self.values.flight_animation = CoreUintType::deserialize(reader)
                }
                FLIGHT_KEY => self.values.flight_key = CoreUintType::deserialize(reader),
                RECORD_KEY => self.values.key = CoreUintType::deserialize(reader),
                RECORD_SIZE_MODE => self.values.size_mode = CoreUintType::deserialize(reader),
                _ if control_index(key).is_some() => {
                    let index = control_index(key).unwrap();
                    *self.values.control_mut(index) = reader.read_float32();
                }
                STEP_NAME => self.values.name = reader.read_string(),
                FROM_ARTBOARD => self.values.from_artboard = reader.read_string(),
                TO_ARTBOARD => self.values.to_artboard = reader.read_string(),
                _ => unreachable!(),
            }
            return true;
        }
        self.component
            .base
            .base
            .base
            .deserialize(key, reader, &mut NoCallbacks)
    }
}

pub(crate) fn field_id(key: i32) -> Option<i32> {
    match key {
        62000..=62002 | 62009..=62013 | 62026 | 62030..=62032 => Some(0),
        62003..=62005 | 62014..=62025 | 62033..=62047 => Some(2),
        62006..=62008 => Some(1),
        62029 => Some(CoreBoolType::ID),
        _ => None,
    }
}
pub(crate) fn owner(key: u32) -> Option<u16> {
    match key {
        62000..=62005 | 62013..=62025 => Some(62000),
        62006..=62008 | 62030 => Some(62001),
        62009..=62012 | 62026 | 62029 => Some(62002),
        62031..=62047 => Some(62003),
        _ => None,
    }
}
pub(crate) fn set_double(
    object: &mut dyn Any,
    key: i32,
    value: f32,
    completion: &mut PropertySetterCompletion,
) {
    fn set<const TYPE: u16>(
        record: &mut LiveRecord<TYPE>,
        key: u16,
        value: f32,
        completion: &mut PropertySetterCompletion,
    ) {
        let Some(index) = control_index(key).filter(|_| LiveRecord::<TYPE>::owns(key)) else {
            return;
        };
        let target = record.values.control_mut(index);
        if *target != value {
            *target = value;
            completion.record(CoreObject::core(record), key);
        }
    }
    let Ok(key) = u16::try_from(key) else {
        return;
    };
    if let Some(record) = object.downcast_mut::<LiveSlot>() {
        set(record, key, value, completion);
    } else if let Some(record) = object.downcast_mut::<LiveFlight>() {
        set(record, key, value, completion);
    }
}
pub(crate) fn get_double(object: &dyn Any, key: i32) -> f32 {
    fn get<const TYPE: u16>(record: &LiveRecord<TYPE>, key: u16) -> f32 {
        control_index(key)
            .filter(|_| LiveRecord::<TYPE>::owns(key))
            .map_or(0.0, |index| record.values.control(index))
    }
    let Ok(key) = u16::try_from(key) else {
        return 0.0;
    };
    if let Some(record) = object.downcast_ref::<LiveSlot>() {
        get(record, key)
    } else if let Some(record) = object.downcast_ref::<LiveFlight>() {
        get(record, key)
    } else {
        0.0
    }
}
impl<const TYPE: u16> CoreRegistryObject for LiveRecord<TYPE> {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        subtype::<TYPE>(key)
    }
    fn set_uint_with_completion(
        &mut self,
        field: CoreField,
        value: u32,
        completion: &mut PropertySetterCompletion,
    ) {
        if field == CoreField::ComponentParentId
            && self.component.base.base.base.set_parent_id_value(value)
        {
            completion.record(
                CoreObject::core(self),
                ComponentBase::PARENT_ID_PROPERTY_KEY,
            );
        }
    }
    fn set_string_with_completion(
        &mut self,
        field: CoreField,
        value: String,
        completion: &mut PropertySetterCompletion,
    ) {
        if field == CoreField::ComponentName && self.component.base.base.base.set_name_value(value)
        {
            completion.record(CoreObject::core(self), ComponentBase::NAME_PROPERTY_KEY);
        }
    }
    fn set_color_with_completion(
        &mut self,
        _: CoreField,
        _: i32,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_bool_with_completion(
        &mut self,
        _: CoreField,
        _: bool,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_double_with_completion(
        &mut self,
        _: CoreField,
        _: f32,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_int_with_completion(&mut self, _: CoreField, _: i32, _: &mut PropertySetterCompletion) {}
    fn set_callback_with_completion(
        &mut self,
        _: CoreField,
        _: CallbackData<'_>,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn get_uint(&mut self, field: CoreField) -> u32 {
        match field {
            CoreField::ComponentParentId => self.component.base.base.base.parent_id(),
            _ => 0,
        }
    }
    fn get_string(&mut self, field: CoreField) -> String {
        match field {
            CoreField::ComponentName => self.component.base.base.base.name().to_owned(),
            _ => String::new(),
        }
    }
    fn get_color(&mut self, _: CoreField) -> i32 {
        0
    }
    fn get_bool(&mut self, _: CoreField) -> bool {
        false
    }
    fn get_double(&mut self, _: CoreField) -> f32 {
        0.0
    }
    fn get_int(&mut self, _: CoreField) -> i32 {
        0
    }
}

impl<const TYPE: u16> CoreCapabilities for LiveRecord<TYPE> {
    fn as_container_component(&self) -> Option<&ContainerComponent> {
        Some(&self.component)
    }
    fn as_container_component_mut(&mut self) -> Option<&mut ContainerComponent> {
        Some(&mut self.component)
    }

    fn component_on_dirty(&mut self, _: crate::source::component_dirt::ComponentDirt) -> bool {
        true
    }
    fn component_on_dirty_is_noop(&self) -> bool {
        true
    }
    fn lifecycle_validate(&mut self, context: &mut dyn CoreContext) -> Option<bool> {
        let parent = context.resolve(self.component.base.base.base.parent_id());
        let valid = self.component.validate(context)
            && parent.is_some_and(|p| match TYPE {
                62000 => p
                    .with(|o| o.as_nested_artboard().is_some())
                    .unwrap_or(false),
                62001 => p.is_type_of(crate::Artboard::TYPE_KEY),
                62002 => p.is_type_of(62001),
                62003 => p.is_type_of(crate::Artboard::TYPE_KEY),
                _ => false,
            })
            && self.valid_values();
        if !valid {
            eprintln!("live moves: dropping invalid record type {TYPE} (parent or values)");
        }
        Some(valid)
    }
    fn lifecycle_on_added_dirty(&mut self, context: &mut dyn CoreContext) -> Option<StatusCode> {
        Some(self.component.on_added_dirty(context))
    }
    fn lifecycle_on_added_clean(&mut self, context: &mut dyn CoreContext) -> Option<StatusCode> {
        let code = self.core_mut().on_added_clean(context);
        if TYPE == 62000
            && self.values.role == 1
            && code == StatusCode::Ok
            && let Some(host) = self.component.parent_handle()
        {
            flight_slot_out_of_flow(self.component.base.base.artboard_handle(), &host);
        }
        Some(code)
    }
    fn lifecycle_import(&mut self, stack: &mut ImportStack) -> Option<StatusCode> {
        Some(self.component.import(stack))
    }
    fn as_component(&self) -> Option<&Component> {
        Some(&self.component.base.base)
    }
    fn as_component_mut(&mut self) -> Option<&mut Component> {
        Some(&mut self.component.base.base)
    }
}

/// A flight never pushes content (live-moves.md). A CSS flight slot leaves the
/// flow as its Screen occurrence is built, before that occurrence is laid out.
/// The write is to occurrence state only: definitions stay as imported.
fn flight_slot_out_of_flow(artboard: Option<CoreHandle>, host: &CoreHandle) {
    let occurrence = artboard
        .and_then(|board| board.with_downcast::<crate::Artboard, _>(crate::Artboard::is_instance))
        .unwrap_or(false);
    if occurrence && let Some(record) = flight_slot_record(host) {
        crate::source::generated::core_registry::CoreRegistry::set_uint_handle(
            &record,
            i32::from(crate::css_layout::keys::POSITION),
            2,
        );
    }
}
/// A layout flight slot's item record, which its unit's size and placement
/// come from. A native layout flight slot has none.
fn flight_slot_record(host: &CoreHandle) -> Option<CoreHandle> {
    use crate::source::generated::nested_artboard_layout_base::NestedArtboardLayoutBase;
    if !host.is_type_of(NestedArtboardLayoutBase::TYPE_KEY) {
        return None;
    }
    host.with(|o| {
        o.as_container_component()
            .and_then(|host| crate::css_layout::record::find_in(host.children()))
    })
    .flatten()
}

// Validate the compiler's declarations once, using the native definition tables.
// This is shared by whole-file import and the tools reader. Runtime instances
// still check their actual screens path, since a valid Screen definition can
// also be instantiated as ordinary paint or nested content.
pub(crate) fn validate_import_shape(file: &crate::File, replacement: Option<&CoreHandle>) -> bool {
    use crate::{Artboard, source::artboard_component_list::ArtboardComponentList};
    use std::collections::HashSet;
    if !file.core_arena().contains_types(62000..=62003) {
        return true;
    }
    // A file flies its matched elements one way: in place when it holds any
    // flight record, otherwise through flight slots (docs/live-moves.md,
    // Flying in place).
    let in_place = file.core_arena().contains_types(62003..=62003);
    if in_place && file.core_arena().contains_types(62000..=62000) {
        eprintln!("live moves: a file holds both flight slots and flight records");
        return false;
    }
    let mut boards = file.artboards();
    if let Some(replacement) = replacement {
        let name = replacement.with_downcast::<Artboard, _>(|b| b.name().to_owned());
        boards.retain(|b| b.with_downcast::<Artboard, _>(|b| b.name().to_owned()) != name);
        boards.push(replacement.clone());
    }
    let mut pages = HashSet::new();
    let mut screen_names = HashSet::new();
    for board in &boards {
        if !board
            .with_downcast::<Artboard, _>(|b| {
                b.find_handle::<ArtboardComponentList>("screens").is_some()
            })
            .unwrap_or(false)
        {
            continue;
        }
        pages.insert(board.clone());
        let steps = board
            .with_downcast::<Artboard, _>(|b| b.find_all_handles::<LiveStep>())
            .unwrap();
        for step in steps {
            step.with_downcast::<LiveStep, _>(|s| {
                screen_names.insert(s.values.from_artboard.clone());
                screen_names.insert(s.values.to_artboard.clone());
            });
        }
    }
    for board in boards {
        let (is_screen, objects) = board
            .with_downcast::<Artboard, _>(|b| {
                (
                    !pages.contains(&board) && screen_names.contains(b.name()),
                    b.objects().to_vec(),
                )
            })
            .unwrap();
        // Validation is read-only, including definitions shared with an
        // installed Page during tools replacement. Diagnose here; the mounted
        // Page's endpoint classification and path checks ignore these at use.
        for record in objects.iter().flatten() {
            let valid = match record.core_type() {
                Some(62000 | 62003) => is_screen,
                Some(62001 | 62002) => pages.contains(&board),
                _ => continue,
            };
            if !valid {
                eprintln!(
                    "live moves: ignoring record outside page/screens/Screen slots in {}",
                    board
                        .with_downcast::<Artboard, _>(|b| b.name().to_owned())
                        .unwrap()
                );
            }
        }
        // On one Screen, a nonzero flight key names one element and a
        // flight record's key one record.
        if is_screen && let Err(error) = unique_flight_keys(&objects) {
            eprintln!(
                "live moves: invalid Screen {}: {error}",
                board
                    .with_downcast::<Artboard, _>(|b| b.name().to_owned())
                    .unwrap()
            );
            return false;
        }
        // Validate plans only after the supported shape has been selected.
        // A declaration on ordinary paint/nested content is ignored as a unit.
        if pages.contains(&board) {
            let steps = board
                .with_downcast::<Artboard, _>(|b| b.find_all_handles::<LiveStep>())
                .unwrap();
            for step in steps {
                let (name, units) = step
                    .with_downcast::<LiveStep, _>(|s| {
                        (
                            s.values.name.clone(),
                            s.component
                                .children()
                                .iter()
                                .filter_map(|u| {
                                    u.with_downcast::<LiveStepUnit, _>(|u| u.values.clone())
                                })
                                .collect::<Vec<_>>(),
                        )
                    })
                    .unwrap();
                let in_place_rule = || -> anyhow::Result<()> {
                    anyhow::ensure!(
                        !in_place || units.iter().all(|unit| !unit.transfers()),
                        "a step unit of an in-place file transfers"
                    );
                    Ok(())
                };
                if let Err(error) = in_place_rule().and_then(|()| validate_step_units(&units)) {
                    eprintln!("live moves: invalid step {name}: {error}");
                    return false;
                }
            }
        }
    }
    true
}

/// A Screen's nonzero element flight keys and its flight records' keys are
/// each unique on it.
fn unique_flight_keys(objects: &[Option<CoreHandle>]) -> anyhow::Result<()> {
    use crate::css_layout::record::LayoutCssStyle;
    use std::collections::HashSet;
    let (mut elements, mut records) = (HashSet::new(), HashSet::new());
    for object in objects.iter().flatten() {
        if let Some(key) = object.with_downcast::<LayoutCssStyle, _>(LayoutCssStyle::flight_key) {
            anyhow::ensure!(
                key == 0 || elements.insert(key),
                "duplicate flight key {key}"
            );
        } else if let Some(key) = object.with_downcast::<LiveFlight, _>(|r| r.values.key) {
            anyhow::ensure!(records.insert(key), "duplicate flight record key {key}");
        }
    }
    Ok(())
}
