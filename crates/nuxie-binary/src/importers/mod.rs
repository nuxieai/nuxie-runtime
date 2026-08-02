use super::*;

#[derive(Default)]
pub(crate) struct ImportContext {
    pub(crate) import_stack: CppImportStack,
    pub(crate) latest_layer_state_accepts_blend_animation: bool,
    pub(crate) state_machine_inputs: Vec<Option<StateMachineInputKind>>,
    pub(crate) artboard_local_nested_inputs: Vec<Option<StateMachineInputKind>>,
}

#[derive(Default)]
pub(crate) struct CppImportStack {
    latest: BTreeSet<ImportStackKey>,
    last_added: Vec<ImportStackKey>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ImportStackKey {
    Backboard,
    Artboard,
    FileAsset,
    LinearAnimation,
    KeyedObject,
    KeyedProperty,
    StateMachine,
    StateMachineLayer,
    LayerState,
    StateTransition,
    StateMachineLayerComponent,
    StateMachineListener,
    ListenerInputTypeGamepad,
    ListenerInputTypeKeyboard,
    ListenerInputTypeSemantic,
    DataEnumCustom,
    ViewModel,
    ViewModelInstance,
    ViewModelInstanceList,
    TransitionViewModelCondition,
    BindableProperty,
    DataConverterGroup,
    DataConverterFormula,
    DataBindPath,
    ScriptedObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StateMachineInputKind {
    Bool,
    Number,
    Trigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NullObjectConsumer {
    Artboard,
    KeyedProperty,
    StateMachine,
    StateMachineLayer,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CppDataBindTarget<'a> {
    pub(crate) file_index: usize,
    pub(crate) object: &'a RuntimeObject,
}

impl ImportContext {
    pub(crate) fn latest(&self, key: ImportStackKey) -> bool {
        self.import_stack.latest(key)
    }

    fn make_latest(&mut self, key: ImportStackKey) {
        self.import_stack.make_latest(key);
    }

    pub(crate) fn read_null_object(&mut self) {
        match self.import_stack.latest_null_object_consumer() {
            Some(NullObjectConsumer::Artboard) => {
                self.artboard_local_nested_inputs.push(None);
            }
            Some(NullObjectConsumer::StateMachine) => self.state_machine_inputs.push(None),
            _ => {}
        }
    }

    pub(crate) fn read_dropped_object(&mut self, definition: &'static Definition) {
        if definition_is_cpp_artboard_local(definition) {
            self.artboard_local_nested_inputs.push(None);
        }
    }
}

pub(crate) mod artboard_importer;
mod backboard_importer;
mod bindable_property_importer;
mod data_bind_path_importer;
mod data_converter_formula_importer;
mod data_converter_group_importer;
mod enum_importer;
mod file_asset_importer;
mod keyed_object_importer;
mod layer_state_importer;
mod linear_animation_importer;
mod listener_input_type_gamepad_importer;
mod listener_input_type_keyboard_importer;
mod listener_input_type_semantic_importer;
mod scripted_object_importer;
mod state_machine_layer_component_importer;
mod state_machine_layer_importer;
mod state_machine_listener_importer;
mod text_asset_importer;
mod viewmodel_importer;
mod viewmodel_instance_importer;
mod viewmodel_instance_list_importer;

impl CppImportStack {
    fn latest(&self, key: ImportStackKey) -> bool {
        self.latest.contains(&key)
    }

    fn make_latest(&mut self, key: ImportStackKey) {
        if let Some(index) = self
            .last_added
            .iter()
            .rposition(|candidate| *candidate == key)
        {
            self.last_added.remove(index);
        }
        self.latest.insert(key);
        self.last_added.push(key);
    }

    fn latest_null_object_consumer(&self) -> Option<NullObjectConsumer> {
        self.last_added
            .iter()
            .rev()
            .find_map(|key| key.null_object_consumer())
    }
}

/// The graph projection replays only the importer keys that affect the graph
/// shape. Preserve the exact ImportStack “replace this key, then append it to
/// reverse insertion order” rule for null-object ownership.
pub(crate) fn replay_import_stack_make_latest(
    last_added: &mut Vec<ImportStackKey>,
    key: ImportStackKey,
) {
    if let Some(index) = last_added.iter().rposition(|candidate| *candidate == key) {
        last_added.remove(index);
    }
    last_added.push(key);
}

impl ImportStackKey {
    pub(crate) fn null_object_consumer(self) -> Option<NullObjectConsumer> {
        match self {
            Self::Artboard => Some(NullObjectConsumer::Artboard),
            Self::KeyedProperty => Some(NullObjectConsumer::KeyedProperty),
            Self::StateMachine => Some(NullObjectConsumer::StateMachine),
            Self::StateMachineLayer => Some(NullObjectConsumer::StateMachineLayer),
            _ => None,
        }
    }
}

pub(crate) fn compute_import_statuses(
    objects: &[Option<RuntimeObject>],
    script_assets_create_importers: bool,
) -> Vec<RuntimeImportStatus> {
    let mut context = ImportContext::default();
    objects
        .iter()
        .map(|object| {
            let Some(object) = object.as_ref() else {
                context.read_null_object();
                return RuntimeImportStatus::NullObject;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                context.read_null_object();
                return RuntimeImportStatus::NullObject;
            };

            if let Some(reason) = object_import_failure_reason(object, definition, &context) {
                context.read_dropped_object(definition);
                return RuntimeImportStatus::Dropped { reason };
            }

            update_import_context(
                object,
                definition,
                &mut context,
                script_assets_create_importers,
            );
            RuntimeImportStatus::Imported
        })
        .collect()
}

fn object_import_failure_reason(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<RuntimeImportDropReason> {
    if !object_imports_successfully(object, definition, context) {
        return Some(RuntimeImportDropReason::MissingObject);
    }

    if transition_input_condition_is_invalid(object, definition, context) {
        return Some(RuntimeImportDropReason::InvalidObject);
    }

    if listener_input_change_is_invalid(object, definition, context) {
        return Some(RuntimeImportDropReason::InvalidObject);
    }

    if blend_input_is_invalid(object, definition, context) {
        return Some(RuntimeImportDropReason::InvalidObject);
    }

    if definition.is_a("BlendAnimation") && !context.latest_layer_state_accepts_blend_animation {
        return Some(RuntimeImportDropReason::InvalidObject);
    }

    None
}

fn object_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> bool {
    if let Some(decision) = enum_importer::imports_successfully(object, definition, context) {
        return decision;
    }

    match definition.name {
        "Backboard" => {
            return backboard_importer::imports_successfully(object, definition, context)
                .expect("Backboard is owned by BackboardImporter");
        }
        "ViewModel" => {
            return viewmodel_importer::imports_successfully(object, definition, context)
                .expect("ViewModel is owned by ViewModelImporter");
        }
        "KeyedProperty" => return context.latest(ImportStackKey::KeyedObject),
        "StateMachine" => return context.latest(ImportStackKey::Artboard),
        "BlendState1DViewModel"
        | "ListenerViewModelChange"
        | "TransitionPropertyViewModelComparator" => {
            return bindable_property_importer::imports_successfully(object, definition, context)
                .expect("bindable consumer is owned by BindablePropertyImporter");
        }
        "BlendAnimationDirect" => {
            if let Some(decision) =
                bindable_property_importer::imports_successfully(object, definition, context)
            {
                return decision;
            }
        }
        "ViewModelInstance"
        | "ViewModelInstanceAsset"
        | "ViewModelInstanceAssetImage"
        | "ViewModelInstanceAssetFont" => {
            return viewmodel_instance_importer::imports_successfully(object, definition, context)
                .expect("view-model instance object is owned by ViewModelInstanceImporter");
        }
        "ViewModelInstanceListItem" => {
            return viewmodel_instance_list_importer::imports_successfully(
                object, definition, context,
            )
            .expect("list item is owned by ViewModelInstanceListImporter");
        }
        _ => {}
    }

    if let Some(decision) = artboard_importer::imports_successfully(object, definition, context) {
        return decision;
    }
    if let Some(decision) =
        scripted_object_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }
    if let Some(decision) =
        listener_input_type_gamepad_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }
    if let Some(decision) =
        listener_input_type_keyboard_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }
    if let Some(decision) =
        listener_input_type_semantic_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }
    if let Some(decision) =
        linear_animation_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }
    if let Some(decision) = keyed_object_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }
    if let Some(decision) =
        state_machine_layer_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }
    if let Some(decision) =
        data_converter_group_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }

    if let Some(decision) = text_asset_importer::imports_successfully(object, definition, context) {
        return decision;
    }
    if let Some(decision) = file_asset_importer::imports_successfully(object, definition, context) {
        return decision;
    }

    if definition.is_a("StateTransition") {
        return context.latest(ImportStackKey::LayerState);
    }

    if definition.is_a("TransitionCondition") {
        return context.latest(ImportStackKey::StateTransition);
    }

    if definition.is_a("TransitionComparator") {
        return context.latest(ImportStackKey::TransitionViewModelCondition);
    }

    if let Some(decision) =
        state_machine_layer_component_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }

    if definition.is_a("StateMachineInput") {
        return context.latest(ImportStackKey::StateMachine);
    }

    if let Some(decision) =
        state_machine_listener_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }

    if let Some(decision) = layer_state_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }

    if definition.is_a("ViewModelProperty") {
        return viewmodel_importer::imports_successfully(object, definition, context)
            .expect("property is owned by ViewModelImporter");
    }

    if definition.is_a("ViewModelInstanceValue") {
        return viewmodel_instance_importer::imports_successfully(object, definition, context)
            .expect("value is owned by ViewModelInstanceImporter");
    }

    if let Some(decision) =
        data_bind_path_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }

    if definition.is_a("DataBind") || definition.is_a("DataConverter") {
        return backboard_importer::imports_successfully(object, definition, context)
            .expect("Backboard owns data-bind and converter objects");
    }

    if let Some(decision) =
        data_converter_formula_importer::imports_successfully(object, definition, context)
    {
        return decision;
    }

    if definition.is_a("ScrollPhysics") || definition.is_a("KeyFrameInterpolator") {
        return backboard_importer::imports_successfully(object, definition, context)
            .expect("Backboard owns file-global physics/interpolators");
    }

    if let Some(decision) = artboard_importer::component_imports_successfully(definition, context) {
        return decision;
    }

    true
}

pub(crate) fn update_import_context(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &mut ImportContext,
    script_assets_create_importers: bool,
) {
    match definition.name {
        "Backboard" => backboard_importer::update_context(definition, context),
        "KeyedProperty" => context.make_latest(ImportStackKey::KeyedProperty),
        "StateMachine" => {
            context.state_machine_inputs.clear();
            context.make_latest(ImportStackKey::StateMachine);
        }
        "ViewModel" => viewmodel_importer::update_context(definition, context),
        "ViewModelInstance" => viewmodel_instance_importer::update_context(definition, context),
        "ViewModelInstanceList" => {
            viewmodel_instance_list_importer::update_context(definition, context);
        }
        _ => {}
    }

    artboard_importer::update_context(definition, context);
    linear_animation_importer::update_context(definition, context);
    keyed_object_importer::update_context(definition, context);
    state_machine_layer_importer::update_context(definition, context);
    enum_importer::update_context(definition, context);
    data_converter_group_importer::update_context(definition, context);
    data_converter_formula_importer::update_context(definition, context);
    file_asset_importer::update_context(definition, context, script_assets_create_importers);
    listener_input_type_gamepad_importer::update_context(definition, context);
    listener_input_type_keyboard_importer::update_context(definition, context);
    listener_input_type_semantic_importer::update_context(definition, context);
    state_machine_layer_component_importer::update_context(definition, context);
    if definition.is_a("StateTransition") {
        context.make_latest(ImportStackKey::StateTransition);
    }
    layer_state_importer::update_context(definition, context);
    state_machine_listener_importer::update_context(definition, context);
    if let Some(kind) = state_machine_input_kind(definition) {
        context.state_machine_inputs.push(Some(kind));
    }
    if definition_is_cpp_artboard_local(definition) {
        context
            .artboard_local_nested_inputs
            .push(nested_input_kind(definition));
    }
    if definition.is_a("TransitionViewModelCondition") {
        context.make_latest(ImportStackKey::TransitionViewModelCondition);
    }
    if definition.is_a("BindableProperty") {
        bindable_property_importer::update_context(definition, context);
    }
    data_bind_path_importer::update_context(definition, context);
    scripted_object_importer::update_context(definition, context);

    let _ = object;
}
