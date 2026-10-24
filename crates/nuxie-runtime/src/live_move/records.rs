//! Nuxie-owned published live-move records. Keys come from defs/nuxie.
use crate::source::{
    component::Component,
    container_component::ContainerComponent,
    core::{
        Core, CoreHandle, CoreObject, CoreType, PropertySetterCompletion,
        binary_reader::BinaryReader,
        field_types::{core_callback_type::CallbackData, core_uint_type::CoreUintType},
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

#[derive(Clone, Default)]
pub struct Values {
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
    pub flight_key: u32,
    pub fade: f32,
}
// Shared by import and step preflight. A single-version pair names one flight;
// every other key belongs to just one mapping in the complete step.
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
            (unit.incoming_unit_key == unit.outgoing_unit_key)
                == (unit.incoming_flight_key == unit.outgoing_flight_key),
            "single-version unit must name one flight"
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
impl<const TYPE: u16> Default for LiveRecord<TYPE> {
    fn default() -> Self {
        Self {
            component: ContainerComponent::default(),
            values: Values {
                fade: 1.0,
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
            62000 => (62000..=62005).contains(&key) || (62013..=62014).contains(&key),
            62001 => (62006..=62008).contains(&key),
            62002 => (62009..=62012).contains(&key),
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
                    && (self.values.role != 0
                        || (self.values.progress == 0.0
                            && self.values.path_x == 0.0
                            && self.values.path_y == 0.0
                            && self.values.flight_key == 0
                            && self.values.fade == 1.0))
            }
            62001 => {
                !self.values.name.is_empty()
                    && !self.values.from_artboard.is_empty()
                    && !self.values.to_artboard.is_empty()
            }
            62002 => {
                (self.values.incoming_unit_key == self.values.outgoing_unit_key)
                    == (self.values.incoming_flight_key == self.values.outgoing_flight_key)
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
                FLIGHT_KEY => self.values.flight_key = CoreUintType::deserialize(reader),
                FADE => self.values.fade = reader.read_float32(),
                PROGRESS => self.values.progress = reader.read_float32(),
                PATH_X => self.values.path_x = reader.read_float32(),
                PATH_Y => self.values.path_y = reader.read_float32(),
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
        62000..=62002 | 62009..=62013 => Some(0),
        62003..=62005 | 62014 => Some(2),
        62006..=62008 => Some(1),
        _ => None,
    }
}
pub(crate) fn owner(key: u32) -> Option<u16> {
    match key {
        62000..=62005 | 62013..=62014 => Some(62000),
        62006..=62008 => Some(62001),
        62009..=62012 => Some(62002),
        _ => None,
    }
}
pub(crate) fn set_double(
    object: &mut dyn Any,
    key: i32,
    value: f32,
    completion: &mut PropertySetterCompletion,
) {
    if let Some(record) = object.downcast_mut::<LiveSlot>() {
        let target = match key {
            62003 => &mut record.values.progress,
            62004 => &mut record.values.path_x,
            62005 => &mut record.values.path_y,
            62014 => &mut record.values.fade,
            _ => return,
        };
        if *target != value {
            *target = value;
            completion.record(CoreObject::core(record), key as u16);
        }
    }
}
pub(crate) fn get_double(object: &dyn Any, key: i32) -> f32 {
    object
        .downcast_ref::<LiveSlot>()
        .map_or(0.0, |r| match key {
            62003 => r.values.progress,
            62004 => r.values.path_x,
            62005 => r.values.path_y,
            62014 => r.values.fade,
            _ => 0.0,
        })
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

// Validate the compiler's declarations once, using the native definition tables.
// This is shared by whole-file import and the tools reader. Runtime instances
// still check their actual screens path, since a valid Screen definition can
// also be instantiated as ordinary paint or nested content.
pub(crate) fn validate_import_shape(file: &crate::File, replacement: Option<&CoreHandle>) -> bool {
    use crate::{Artboard, source::artboard_component_list::ArtboardComponentList};
    use std::collections::HashSet;
    if !file.core_arena().contains_types(62000..=62002) {
        return true;
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
                Some(62000) => is_screen,
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
                if let Err(error) = validate_step_units(&units) {
                    eprintln!("live moves: invalid step {name}: {error}");
                    return false;
                }
            }
        }
    }
    true
}
