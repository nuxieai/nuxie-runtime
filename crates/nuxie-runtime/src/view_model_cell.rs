//! Retained-identity view-model property cells — #RB-1 slice (a)+(b).
//!
//! Ports the C++ core that the flattened data-bind design discarded:
//!
//! - `ViewModelInstanceValue` (`viewmodel_instance_value.cpp`): one shared,
//!   typed, mutable property value with a changed flag and a dependent list.
//! - `DependencyHelper` (`dependency_helper.hpp`): mutation cascades dirt to
//!   every registered dependent; nothing else happens inline.
//! - `SuppressDelegation` (`viewmodel_instance_value.hpp`): scoped delegate
//!   suppression so internal writes (e.g. trigger `advanced()` zeroing) do
//!   not notify hosts.
//!
//! `Rc<RefCell<..>>` is the Rust analog of C++ `rcp`: cloning a
//! [`RuntimeViewModelCell`] shares identity, exactly like retaining the same
//! `ViewModelInstanceValue*`. The dirt cascade deliberately does nothing but
//! set bits on dependent sinks — mirroring C++ `addDirt`, and keeping the
//! cascade re-entrancy-free while listener actions mutate other cells.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

/// Dirt bits a cell mutation cascades to its dependents.
///
/// C++ cascades `ComponentDirt::Bindings`; the rebuild keeps one bit per
/// concern so the update cycle can cheaply ask "which of my binds observed a
/// source change since the last pass".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeCellDirt(u8);

impl RuntimeCellDirt {
    pub const NONE: Self = Self(0);
    /// The bound source value changed (C++ `ComponentDirt::Bindings`).
    pub const BINDINGS: Self = Self(1 << 0);

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::NONE)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// One dependent's dirt sink.
///
/// C++ stores a raw `DataBind*` and calls `addDirt` on it. The Rust analog
/// is a shared bit-cell owned by the dependent (a retained bind, listener,
/// or converter operand); the cell holds only a weak reference, so a dropped
/// dependent unregisters itself implicitly, mirroring `~DataBind`'s
/// `removeDependent` without manual lifetime bookkeeping.
#[derive(Clone, Default)]
pub struct RuntimeCellDirtSink {
    bits: Rc<Cell<u8>>,
}

impl RuntimeCellDirtSink {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_dirt(&self, dirt: RuntimeCellDirt) {
        self.bits.set(self.bits.get() | dirt.0);
    }

    pub fn take_dirt(&self) -> RuntimeCellDirt {
        RuntimeCellDirt(self.bits.replace(0))
    }

    pub fn peek_dirt(&self) -> RuntimeCellDirt {
        RuntimeCellDirt(self.bits.get())
    }

    fn downgrade(&self) -> Weak<Cell<u8>> {
        Rc::downgrade(&self.bits)
    }
}

/// The typed payload of one cell.
///
/// Mirrors the concrete `ViewModelInstance*` C++ subclasses. `ViewModel` and
/// `List` payloads retain child cells/instances by identity in later #RB-1
/// slices; they enter here as the type lattice so the cell layer is complete
/// from the start.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeViewModelCellValue {
    Number(f32),
    Boolean(bool),
    String(Vec<u8>),
    Color(u32),
    Enum(u32),
    /// Fired-counter semantics: the value is the cumulative fire count while
    /// live; C++ `ViewModelInstanceTrigger::advanced()` zeroes it each
    /// advance under suppressed delegation.
    Trigger(u64),
    SymbolListIndex(u32),
    AssetImage(u32),
    AssetFont(u32),
    Artboard(u32),
}

impl RuntimeViewModelCellValue {
    fn same_kind(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

struct RuntimeViewModelCellState {
    value: RuntimeViewModelCellValue,
    /// C++ `ValueFlags::valueChanged`: set by writes, cleared by `advanced`.
    value_changed: bool,
    /// C++ `SuppressDelegation` depth: internal writes still cascade dirt to
    /// dependents but do not count as host-visible changes.
    suppress_depth: u32,
    dependents: Vec<Weak<Cell<u8>>>,
}

impl RuntimeViewModelCellState {
    fn cascade(&mut self, dirt: RuntimeCellDirt) {
        // Prune dead sinks while cascading; C++ relies on explicit
        // removeDependent in ~DataBind, which weak references replace.
        self.dependents.retain(|dependent| {
            let Some(bits) = dependent.upgrade() else {
                return false;
            };
            bits.set(bits.get() | dirt.0);
            true
        });
    }
}

/// One shared, retained, typed view-model property value.
#[derive(Clone)]
pub struct RuntimeViewModelCell {
    state: Rc<RefCell<RuntimeViewModelCellState>>,
}

impl RuntimeViewModelCell {
    pub fn new(value: RuntimeViewModelCellValue) -> Self {
        Self {
            state: Rc::new(RefCell::new(RuntimeViewModelCellState {
                value,
                value_changed: false,
                suppress_depth: 0,
                dependents: Vec::new(),
            })),
        }
    }

    /// Shared-identity check: two handles retain the same C++ object.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }

    pub fn value(&self) -> RuntimeViewModelCellValue {
        self.state.borrow().value.clone()
    }

    /// C++ `ViewModelInstanceValue::hasChanged` (`ValueFlags::valueChanged`).
    pub fn has_changed(&self) -> bool {
        self.state.borrow().value_changed
    }

    /// Register a dependent's dirt sink (C++ `addDependent`). A bind created
    /// with C++ `bindsOnce()` simply never registers.
    pub fn add_dependent(&self, sink: &RuntimeCellDirtSink) {
        let mut state = self.state.borrow_mut();
        let weak = sink.downgrade();
        if !state
            .dependents
            .iter()
            .any(|existing| existing.ptr_eq(&weak))
        {
            state.dependents.push(weak);
        }
    }

    /// C++ `removeDependent` (used by `clearSource`; drop also suffices).
    pub fn remove_dependent(&self, sink: &RuntimeCellDirtSink) {
        let weak = sink.downgrade();
        self.state
            .borrow_mut()
            .dependents
            .retain(|existing| !existing.ptr_eq(&weak));
    }

    /// Write a same-kind value. Mirrors the concrete C++ setters
    /// (`ViewModelInstanceNumber::propertyValue(..)` →
    /// `propertyValueChanged()`): a genuine change sets the changed flag
    /// (unless delegation is suppressed) and cascades `BINDINGS` dirt to
    /// every dependent. Returns whether the stored value changed.
    ///
    /// A kind mismatch is a programming error in the caller (C++ types this
    /// statically per subclass) and is rejected without mutation.
    pub fn set_value(&self, value: RuntimeViewModelCellValue) -> bool {
        let mut state = self.state.borrow_mut();
        if !state.value.same_kind(&value) {
            debug_assert!(
                false,
                "view-model cell kind mismatch: {:?} <- {:?}",
                state.value, value
            );
            return false;
        }
        if state.value == value {
            return false;
        }
        state.value = value;
        if state.suppress_depth == 0 {
            state.value_changed = true;
        }
        state.cascade(RuntimeCellDirt::BINDINGS);
        true
    }

    /// C++ `ViewModelInstanceTrigger::trigger()` analog: increment the fire
    /// counter. Cascades like any other write.
    pub fn fire_trigger(&self) -> bool {
        let current = {
            let state = self.state.borrow();
            match state.value {
                RuntimeViewModelCellValue::Trigger(count) => count,
                _ => {
                    debug_assert!(false, "fire_trigger on non-trigger cell");
                    return false;
                }
            }
        };
        self.set_value(RuntimeViewModelCellValue::Trigger(
            current.saturating_add(1),
        ))
    }

    /// C++ `ViewModelInstanceValue::advanced()`: acknowledge one update pass.
    /// Clears the changed flag; a trigger cell additionally zeroes its
    /// counter under suppressed delegation
    /// (`ViewModelInstanceTrigger::advanced()` → `propertyValue(0)` inside
    /// `SuppressDelegation`), still cascading dirt to dependents exactly as
    /// the C++ setter does.
    pub fn advanced(&self) {
        let is_trigger = {
            let state = self.state.borrow();
            matches!(state.value, RuntimeViewModelCellValue::Trigger(_))
        };
        if is_trigger {
            self.with_suppressed_delegation(|cell| {
                cell.set_value(RuntimeViewModelCellValue::Trigger(0));
            });
        }
        self.state.borrow_mut().value_changed = false;
    }

    /// C++ `SuppressDelegation` scope: writes inside `body` cascade dirt but
    /// do not mark the cell host-visibly changed.
    pub fn with_suppressed_delegation(&self, body: impl FnOnce(&Self)) {
        self.state.borrow_mut().suppress_depth += 1;
        body(self);
        let mut state = self.state.borrow_mut();
        debug_assert!(state.suppress_depth > 0);
        state.suppress_depth = state.suppress_depth.saturating_sub(1);
    }

    #[cfg(test)]
    fn dependent_count(&self) -> usize {
        self.state.borrow().dependents.len()
    }
}


/// Property-slot classification, from the `ViewModelProperty*` definition
/// object type names (C++ subclasses of `ViewModelProperty`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeViewModelCellKind {
    Number,
    Boolean,
    String,
    Color,
    Enum,
    Trigger,
    SymbolListIndex,
    AssetImage,
    AssetFont,
    Artboard,
    ViewModel,
    List,
}

impl RuntimeViewModelCellKind {
    pub fn from_property_type_name(type_name: &str) -> Option<Self> {
        Some(match type_name {
            "ViewModelPropertyNumber" => Self::Number,
            "ViewModelPropertyBoolean" => Self::Boolean,
            "ViewModelPropertyString" => Self::String,
            "ViewModelPropertyColor" => Self::Color,
            "ViewModelPropertyEnum"
            | "ViewModelPropertyEnumCustom"
            | "ViewModelPropertyEnumSystem" => Self::Enum,
            "ViewModelPropertyTrigger" => Self::Trigger,
            "ViewModelPropertySymbolListIndex" => Self::SymbolListIndex,
            "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage" => Self::AssetImage,
            "ViewModelPropertyAssetFont" => Self::AssetFont,
            "ViewModelPropertyArtboard" => Self::Artboard,
            "ViewModelPropertyViewModel" => Self::ViewModel,
            "ViewModelPropertyList" => Self::List,
            _ => return None,
        })
    }

    fn default_value(self) -> Option<RuntimeViewModelCellValue> {
        Some(match self {
            Self::Number => RuntimeViewModelCellValue::Number(0.0),
            Self::Boolean => RuntimeViewModelCellValue::Boolean(false),
            Self::String => RuntimeViewModelCellValue::String(Vec::new()),
            Self::Color => RuntimeViewModelCellValue::Color(0),
            Self::Enum => RuntimeViewModelCellValue::Enum(0),
            Self::Trigger => RuntimeViewModelCellValue::Trigger(0),
            Self::SymbolListIndex => RuntimeViewModelCellValue::SymbolListIndex(0),
            Self::AssetImage => RuntimeViewModelCellValue::AssetImage(0),
            Self::AssetFont => RuntimeViewModelCellValue::AssetFont(0),
            Self::Artboard => RuntimeViewModelCellValue::Artboard(0),
            Self::ViewModel | Self::List => return None,
        })
    }
}

/// One property slot of a cell-backed instance, indexed by definition-order
/// property index (the unit of C++ property paths).
#[derive(Clone)]
pub enum RuntimeViewModelCellSlot {
    Value(RuntimeViewModelCell),
    /// Nested `ViewModelPropertyViewModel` reference; `None` when the
    /// definition declares no referenced model or the reference is
    /// unresolvable (C++ `referenceViewModelInstance() == nullptr`).
    ViewModel(Option<RuntimeViewModelInstanceCells>),
    /// `ViewModelPropertyList` items, each a retained child instance.
    List(Vec<RuntimeViewModelInstanceCells>),
    /// Property type this port does not model yet; lookups return `None`
    /// exactly like C++ returns nullptr for a failed downcast.
    Unsupported,
}

struct RuntimeViewModelInstanceCellsState {
    view_model_index: usize,
    slots: Vec<RuntimeViewModelCellSlot>,
}

/// A retained, shared, cell-backed view-model instance — the Rust analog of
/// C++ `rcp<ViewModelInstance>`: cloning shares identity; every property is
/// a shared [`RuntimeViewModelCell`] (or a retained child instance).
#[derive(Clone)]
pub struct RuntimeViewModelInstanceCells {
    state: Rc<RefCell<RuntimeViewModelInstanceCellsState>>,
}

impl RuntimeViewModelInstanceCells {
    /// C++ `File::createViewModelInstance(viewModel)`: definition defaults,
    /// nested references instantiated recursively (cycle-guarded).
    pub fn from_definition_defaults(
        file: &nuxie_binary::RuntimeFile,
        view_model_index: usize,
    ) -> Option<Self> {
        let mut visiting = Vec::new();
        Self::build(file, view_model_index, None, &mut visiting)
    }

    /// C++ `File::createViewModelInstance(index, instanceIndex)` →
    /// `copyViewModelInstance`: a retained copy of serialized instance
    /// `instance_index`, nested instance references followed.
    pub fn from_serialized_instance(
        file: &nuxie_binary::RuntimeFile,
        view_model_index: usize,
        instance_index: usize,
    ) -> Option<Self> {
        let view_model = file.view_model(view_model_index)?;
        let instance = view_model.instances.into_iter().nth(instance_index)?;
        let mut visiting = Vec::new();
        Self::build(file, view_model_index, Some(instance.object), &mut visiting)
    }

    fn build(
        file: &nuxie_binary::RuntimeFile,
        view_model_index: usize,
        instance: Option<&nuxie_binary::RuntimeObject>,
        visiting: &mut Vec<usize>,
    ) -> Option<Self> {
        if visiting.contains(&view_model_index) {
            return None;
        }
        visiting.push(view_model_index);
        let view_model = file.view_model(view_model_index);
        let slots = view_model
            .map(|view_model| {
                view_model
                    .properties
                    .iter()
                    .enumerate()
                    .map(|(property_index, property)| {
                        Self::build_slot(
                            file,
                            view_model_index,
                            property_index,
                            property.type_name,
                            instance,
                            visiting,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        visiting.pop();
        Some(Self {
            state: Rc::new(RefCell::new(RuntimeViewModelInstanceCellsState {
                view_model_index,
                slots,
            })),
        })
    }

    fn build_slot(
        file: &nuxie_binary::RuntimeFile,
        view_model_index: usize,
        property_index: usize,
        property_type_name: &str,
        instance: Option<&nuxie_binary::RuntimeObject>,
        visiting: &mut Vec<usize>,
    ) -> RuntimeViewModelCellSlot {
        let Some(kind) = RuntimeViewModelCellKind::from_property_type_name(property_type_name)
        else {
            return RuntimeViewModelCellSlot::Unsupported;
        };
        let path = [
            u32::try_from(view_model_index).unwrap_or(u32::MAX),
            u32::try_from(property_index).unwrap_or(u32::MAX),
        ];
        let instance_value = instance.and_then(|instance| {
            file.data_context_view_model_property_for_instance(instance, &path)
        });
        match kind {
            RuntimeViewModelCellKind::ViewModel => {
                let child = match instance {
                    Some(instance) => file
                        .data_context_view_model_instance_for_instance(instance, &path)
                        .and_then(|reference| {
                            Self::build(
                                file,
                                reference.view_model_index,
                                Some(reference.object),
                                visiting,
                            )
                        }),
                    None => file
                        .view_model(view_model_index)
                        .and_then(|view_model| {
                            view_model.properties.get(property_index).and_then(|property| {
                                property.uint_property("viewModelReferenceId")
                            })
                        })
                        .and_then(|reference| usize::try_from(reference).ok())
                        .and_then(|reference| Self::build(file, reference, None, visiting)),
                };
                RuntimeViewModelCellSlot::ViewModel(child)
            }
            RuntimeViewModelCellKind::List => {
                // Serialized list items are nested instance references; the
                // default instance starts empty (C++ default list).
                let items = instance_value
                    .map(|_| Vec::new())
                    .unwrap_or_default();
                RuntimeViewModelCellSlot::List(items)
            }
            kind => {
                let seeded = instance_value.and_then(|value| match kind {
                    RuntimeViewModelCellKind::Number => file
                        .view_model_instance_number_value_for_object(value)
                        .map(RuntimeViewModelCellValue::Number),
                    RuntimeViewModelCellKind::Boolean => file
                        .view_model_instance_boolean_value_for_object(value)
                        .map(RuntimeViewModelCellValue::Boolean),
                    RuntimeViewModelCellKind::String => file
                        .view_model_instance_string_value_bytes_for_object(value)
                        .map(|bytes| RuntimeViewModelCellValue::String(bytes.to_vec())),
                    RuntimeViewModelCellKind::Color => file
                        .view_model_instance_color_value_for_object(value)
                        .map(RuntimeViewModelCellValue::Color),
                    RuntimeViewModelCellKind::Enum => value
                        .uint_property("propertyValue")
                        .and_then(|value| u32::try_from(value).ok())
                        .map(RuntimeViewModelCellValue::Enum),
                    RuntimeViewModelCellKind::Trigger => file
                        .view_model_instance_trigger_count_for_object(value)
                        .map(RuntimeViewModelCellValue::Trigger),
                    RuntimeViewModelCellKind::SymbolListIndex => file
                        .view_model_instance_symbol_list_index_value_for_object(value)
                        .and_then(|index| u32::try_from(index).ok())
                        .map(RuntimeViewModelCellValue::SymbolListIndex),
                    RuntimeViewModelCellKind::AssetImage => file
                        .view_model_instance_asset_index_for_object(value)
                        .and_then(|index| u32::try_from(index).ok())
                        .map(RuntimeViewModelCellValue::AssetImage),
                    RuntimeViewModelCellKind::AssetFont => file
                        .view_model_instance_font_asset_index_for_object(value)
                        .and_then(|index| u32::try_from(index).ok())
                        .map(RuntimeViewModelCellValue::AssetFont),
                    RuntimeViewModelCellKind::Artboard => file
                        .view_model_instance_artboard_index_for_object(value)
                        .and_then(|index| u32::try_from(index).ok())
                        .map(RuntimeViewModelCellValue::Artboard),
                    RuntimeViewModelCellKind::ViewModel | RuntimeViewModelCellKind::List => None,
                });
                match seeded.or_else(|| kind.default_value()) {
                    Some(value) => {
                        RuntimeViewModelCellSlot::Value(RuntimeViewModelCell::new(value))
                    }
                    None => RuntimeViewModelCellSlot::Unsupported,
                }
            }
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }

    pub fn view_model_index(&self) -> usize {
        self.state.borrow().view_model_index
    }

    /// The retained cell at one property index (C++
    /// `ViewModelInstance::propertyValue(index)` downcast to a value).
    pub fn property_cell(&self, property_index: usize) -> Option<RuntimeViewModelCell> {
        match self.state.borrow().slots.get(property_index)? {
            RuntimeViewModelCellSlot::Value(cell) => Some(cell.clone()),
            _ => None,
        }
    }

    pub fn child_instance(&self, property_index: usize) -> Option<RuntimeViewModelInstanceCells> {
        match self.state.borrow().slots.get(property_index)? {
            RuntimeViewModelCellSlot::ViewModel(child) => child.clone(),
            _ => None,
        }
    }

    /// Walk a property path (already stripped of the leading viewModelId
    /// segment): intermediate segments must be nested ViewModel slots, the
    /// final segment must be a value cell — C++
    /// `DataContext::tryGetViewModelProperty`'s loop.
    pub fn cell_by_property_path(&self, path: &[usize]) -> Option<RuntimeViewModelCell> {
        let (&last, rest) = path.split_last()?;
        let mut current = self.clone();
        for &segment in rest {
            current = current.child_instance(segment)?;
        }
        current.property_cell(last)
    }
}

struct RuntimeDataContextState {
    instances: Vec<RuntimeViewModelInstanceCells>,
    parent: Option<RuntimeDataContext>,
}

/// The Rust analog of C++ `DataContext` (`data_context.cpp`): a retained
/// list of instances plus one parent link. Lookup walks own instances by
/// `path[0] == viewModelId`, then defers to the parent — replacing the
/// candidate-vector search entirely.
#[derive(Clone)]
pub struct RuntimeDataContext {
    state: Rc<RefCell<RuntimeDataContextState>>,
}

impl RuntimeDataContext {
    pub fn new(instance: Option<RuntimeViewModelInstanceCells>) -> Self {
        Self {
            state: Rc::new(RefCell::new(RuntimeDataContextState {
                instances: instance.into_iter().collect(),
                parent: None,
            })),
        }
    }

    pub fn from_instances(instances: Vec<RuntimeViewModelInstanceCells>) -> Self {
        Self {
            state: Rc::new(RefCell::new(RuntimeDataContextState {
                instances,
                parent: None,
            })),
        }
    }

    pub fn set_parent(&self, parent: Option<RuntimeDataContext>) {
        self.state.borrow_mut().parent = parent;
    }

    pub fn parent(&self) -> Option<RuntimeDataContext> {
        self.state.borrow().parent.clone()
    }

    pub fn add_instance(&self, instance: RuntimeViewModelInstanceCells) {
        self.state.borrow_mut().instances.push(instance);
    }

    /// C++ `DataContext::getViewModelProperty(path)`: `path[0]` selects the
    /// instance by view-model id; the rest walks nested properties; a miss
    /// defers to the parent context.
    pub fn view_model_property(&self, path: &[u32]) -> Option<RuntimeViewModelCell> {
        if path.len() < 2 {
            return None;
        }
        let state = self.state.borrow();
        let view_model_id = usize::try_from(path[0]).ok()?;
        let property_path = path[1..]
            .iter()
            .map(|&segment| usize::try_from(segment).ok())
            .collect::<Option<Vec<_>>>()?;
        for instance in &state.instances {
            if instance.view_model_index() != view_model_id {
                continue;
            }
            if let Some(cell) = instance.cell_by_property_path(&property_path) {
                return Some(cell);
            }
        }
        state
            .parent
            .as_ref()
            .and_then(|parent| parent.view_model_property(path))
    }

    /// C++ `DataContext::getViewModelInstance(path)` (nested instance form).
    pub fn view_model_instance(&self, path: &[u32]) -> Option<RuntimeViewModelInstanceCells> {
        if path.is_empty() {
            return None;
        }
        let state = self.state.borrow();
        let view_model_id = usize::try_from(path[0]).ok()?;
        for instance in &state.instances {
            if instance.view_model_index() != view_model_id {
                continue;
            }
            let mut current = instance.clone();
            let mut resolved = true;
            for &segment in &path[1..] {
                let Some(next) = usize::try_from(segment)
                    .ok()
                    .and_then(|segment| current.child_instance(segment))
                else {
                    resolved = false;
                    break;
                };
                current = next;
            }
            if resolved {
                return Some(current);
            }
        }
        state
            .parent
            .as_ref()
            .and_then(|parent| parent.view_model_instance(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::properties::property_key_for_name;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue, RuntimeFile};
    use nuxie_schema::definition_by_name;

    fn authoring_record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn authoring_property(
        type_name: &str,
        property_name: &str,
        value: AuthoringValue,
    ) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, property_name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{property_name}")),
            value,
        }
    }

    /// Root VM: [amount: Number, fired: Trigger, child: ViewModel->Child].
    /// Child VM: [offset: Number]. Serialized instance 0 carries
    /// amount=4022, child.offset=95 — the instance-0-vs-defaults shape from
    /// the db_health_tracker scalar divergence.
    fn cell_fixture() -> RuntimeFile {
        RuntimeFile::from_authoring_records(vec![
            authoring_record(
                "ViewModel",
                vec![authoring_property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Root".to_owned()),
                )],
            ),
            authoring_record(
                "ViewModelPropertyNumber",
                vec![authoring_property(
                    "ViewModelPropertyNumber",
                    "name",
                    AuthoringValue::String("amount".to_owned()),
                )],
            ),
            authoring_record(
                "ViewModelPropertyTrigger",
                vec![authoring_property(
                    "ViewModelPropertyTrigger",
                    "name",
                    AuthoringValue::String("fired".to_owned()),
                )],
            ),
            authoring_record(
                "ViewModelPropertyViewModel",
                vec![
                    authoring_property(
                        "ViewModelPropertyViewModel",
                        "name",
                        AuthoringValue::String("child".to_owned()),
                    ),
                    authoring_property(
                        "ViewModelPropertyViewModel",
                        "viewModelReferenceId",
                        AuthoringValue::Uint(1),
                    ),
                ],
            ),
            authoring_record(
                "ViewModel",
                vec![authoring_property(
                    "ViewModel",
                    "name",
                    AuthoringValue::String("Child".to_owned()),
                )],
            ),
            authoring_record(
                "ViewModelPropertyNumber",
                vec![authoring_property(
                    "ViewModelPropertyNumber",
                    "name",
                    AuthoringValue::String("offset".to_owned()),
                )],
            ),
            authoring_record("Backboard", Vec::new()),
            // Real .riv serialization places artboards before backboard-level
            // instances; the nested-instance resolver requires that order.
            authoring_record("Artboard", Vec::new()),
            authoring_record(
                "ViewModelInstance",
                vec![
                    authoring_property(
                        "ViewModelInstance",
                        "name",
                        AuthoringValue::String("child-instance".to_owned()),
                    ),
                    authoring_property(
                        "ViewModelInstance",
                        "viewModelId",
                        AuthoringValue::Uint(1),
                    ),
                ],
            ),
            authoring_record(
                "ViewModelInstanceNumber",
                vec![
                    authoring_property(
                        "ViewModelInstanceNumber",
                        "viewModelPropertyId",
                        AuthoringValue::Uint(0),
                    ),
                    authoring_property(
                        "ViewModelInstanceNumber",
                        "propertyValue",
                        AuthoringValue::Double(95.0),
                    ),
                ],
            ),
            authoring_record(
                "ViewModelInstance",
                vec![
                    authoring_property(
                        "ViewModelInstance",
                        "name",
                        AuthoringValue::String("root-instance".to_owned()),
                    ),
                    authoring_property(
                        "ViewModelInstance",
                        "viewModelId",
                        AuthoringValue::Uint(0),
                    ),
                ],
            ),
            authoring_record(
                "ViewModelInstanceNumber",
                vec![
                    authoring_property(
                        "ViewModelInstanceNumber",
                        "viewModelPropertyId",
                        AuthoringValue::Uint(0),
                    ),
                    authoring_property(
                        "ViewModelInstanceNumber",
                        "propertyValue",
                        AuthoringValue::Double(4022.0),
                    ),
                ],
            ),
            authoring_record(
                "ViewModelInstanceTrigger",
                vec![authoring_property(
                    "ViewModelInstanceTrigger",
                    "viewModelPropertyId",
                    AuthoringValue::Uint(1),
                )],
            ),
            authoring_record(
                "ViewModelInstanceViewModel",
                vec![
                    authoring_property(
                        "ViewModelInstanceViewModel",
                        "viewModelPropertyId",
                        AuthoringValue::Uint(2),
                    ),
                    authoring_property(
                        "ViewModelInstanceViewModel",
                        "propertyValue",
                        AuthoringValue::Uint(0),
                    ),
                ],
            ),
        ])
        .expect("cell fixture imports")
    }

    #[test]
    fn definition_defaults_and_serialized_instance_seed_differently() {
        let file = cell_fixture();

        let defaults = RuntimeViewModelInstanceCells::from_definition_defaults(&file, 0)
            .expect("defaults instance builds");
        assert_eq!(
            defaults.property_cell(0).expect("amount cell").value(),
            RuntimeViewModelCellValue::Number(0.0),
            "definition defaults zero-seed numbers"
        );

        let seeded = RuntimeViewModelInstanceCells::from_serialized_instance(&file, 0, 0)
            .expect("serialized instance builds");
        assert_eq!(
            seeded.property_cell(0).expect("amount cell").value(),
            RuntimeViewModelCellValue::Number(4022.0),
            "instance 0 seeds the serialized value"
        );
        let child = seeded.child_instance(2).expect("nested child instantiates");
        assert_eq!(child.view_model_index(), 1);
        assert_eq!(
            child.property_cell(0).expect("offset cell").value(),
            RuntimeViewModelCellValue::Number(95.0),
            "nested serialized values follow the instance reference"
        );
    }

    #[test]
    fn data_context_lookup_returns_the_retained_cell_by_identity() {
        let file = cell_fixture();
        let instance = RuntimeViewModelInstanceCells::from_serialized_instance(&file, 0, 0)
            .expect("instance builds");
        let context = RuntimeDataContext::new(Some(instance.clone()));

        let direct = instance.property_cell(0).expect("amount cell");
        let via_context = context
            .view_model_property(&[0, 0])
            .expect("context resolves [vm 0, property 0]");
        assert!(
            direct.ptr_eq(&via_context),
            "the context returns the SAME retained cell, not a copy"
        );

        // Nested path [0, 2, 0] = root vm -> child slot -> offset.
        let nested = context
            .view_model_property(&[0, 2, 0])
            .expect("context resolves the nested path");
        assert_eq!(nested.value(), RuntimeViewModelCellValue::Number(95.0));

        // A dependent registered through one handle observes writes made
        // through the other — the exact propagation the compensation family
        // reconstructed by polling.
        let bind = RuntimeCellDirtSink::new();
        via_context.add_dependent(&bind);
        assert!(direct.set_value(RuntimeViewModelCellValue::Number(7.0)));
        assert!(bind.take_dirt().contains(RuntimeCellDirt::BINDINGS));
        assert_eq!(
            context.view_model_property(&[0, 0]).unwrap().value(),
            RuntimeViewModelCellValue::Number(7.0)
        );
    }

    #[test]
    fn data_context_defers_to_parent_and_rejects_wrong_view_model_id() {
        let file = cell_fixture();
        let child_instance = RuntimeViewModelInstanceCells::from_serialized_instance(&file, 1, 0)
            .expect("child instance builds");
        let root_instance = RuntimeViewModelInstanceCells::from_serialized_instance(&file, 0, 0)
            .expect("root instance builds");

        let parent = RuntimeDataContext::new(Some(root_instance));
        let context = RuntimeDataContext::new(Some(child_instance));
        context.set_parent(Some(parent));

        // vm id 1 resolves locally; vm id 0 misses locally and walks the
        // parent link (C++ getViewModelProperty parent recursion).
        assert!(context.view_model_property(&[1, 0]).is_some());
        let inherited = context
            .view_model_property(&[0, 0])
            .expect("parent resolves vm 0");
        assert_eq!(inherited.value(), RuntimeViewModelCellValue::Number(4022.0));

        // An unknown vm id resolves nowhere.
        assert!(context.view_model_property(&[9, 0]).is_none());
    }

    #[test]
    fn shared_identity_propagates_one_write_to_every_dependent_once() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(3.0));
        let alias = cell.clone();
        assert!(cell.ptr_eq(&alias));

        let bind_a = RuntimeCellDirtSink::new();
        let bind_b = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind_a);
        cell.add_dependent(&bind_b);

        assert!(alias.set_value(RuntimeViewModelCellValue::Number(4.0)));
        assert_eq!(cell.value(), RuntimeViewModelCellValue::Number(4.0));
        assert!(bind_a.take_dirt().contains(RuntimeCellDirt::BINDINGS));
        assert!(bind_b.take_dirt().contains(RuntimeCellDirt::BINDINGS));

        // A writes; it has already consumed its own dirt, B observes exactly
        // one change — the scenario the deleted "observation cursor" polled
        // for, delivered here by the dependency edge alone.
        assert!(bind_a.peek_dirt().is_empty());
        assert!(bind_b.peek_dirt().is_empty());
    }

    #[test]
    fn unchanged_writes_do_not_cascade() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(4.0));
        let bind = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind);
        assert!(!cell.set_value(RuntimeViewModelCellValue::Number(4.0)));
        assert!(bind.peek_dirt().is_empty());
        assert!(!cell.has_changed());
    }

    #[test]
    fn duplicate_registration_is_ignored_and_drop_unregisters() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Boolean(false));
        let bind = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind);
        cell.add_dependent(&bind);
        assert_eq!(cell.dependent_count(), 1);

        drop(bind);
        cell.set_value(RuntimeViewModelCellValue::Boolean(true));
        assert_eq!(cell.dependent_count(), 0, "dead sinks prune on cascade");
    }

    #[test]
    fn explicit_remove_dependent_stops_cascade() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Color(0xff000000));
        let bind = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind);
        cell.remove_dependent(&bind);
        cell.set_value(RuntimeViewModelCellValue::Color(0xffffffff));
        assert!(bind.peek_dirt().is_empty());
    }

    #[test]
    fn trigger_advanced_zeroes_counter_without_host_visible_change() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Trigger(0));
        let bind = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind);

        assert!(cell.fire_trigger());
        assert!(cell.fire_trigger());
        assert_eq!(cell.value(), RuntimeViewModelCellValue::Trigger(2));
        assert!(cell.has_changed());
        assert!(bind.take_dirt().contains(RuntimeCellDirt::BINDINGS));

        // C++ ViewModelInstanceTrigger::advanced(): propertyValue(0) under
        // SuppressDelegation, then the changed flag clears. Dependents still
        // observe the zeroing write as dirt.
        cell.advanced();
        assert_eq!(cell.value(), RuntimeViewModelCellValue::Trigger(0));
        assert!(!cell.has_changed());
        assert!(bind.take_dirt().contains(RuntimeCellDirt::BINDINGS));
    }

    #[test]
    fn suppressed_writes_cascade_dirt_but_stay_host_invisible() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let bind = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind);

        cell.with_suppressed_delegation(|cell| {
            assert!(cell.set_value(RuntimeViewModelCellValue::Number(2.0)));
        });
        assert!(!cell.has_changed(), "suppressed write is not host-visible");
        assert!(
            bind.take_dirt().contains(RuntimeCellDirt::BINDINGS),
            "dependents still observe suppressed writes"
        );
    }

    #[test]
    fn kind_mismatch_is_rejected_without_mutation() {
        let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let bind = RuntimeCellDirtSink::new();
        cell.add_dependent(&bind);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cell.set_value(RuntimeViewModelCellValue::Boolean(true))
        }));
        // Debug builds assert; release builds reject silently. Either way the
        // stored value and dirt are untouched.
        if let Ok(changed) = result {
            assert!(!changed);
            assert_eq!(cell.value(), RuntimeViewModelCellValue::Number(1.0));
            assert!(bind.peek_dirt().is_empty());
        }
    }
}
