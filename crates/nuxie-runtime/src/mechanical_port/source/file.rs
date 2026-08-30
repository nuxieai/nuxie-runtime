use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::{Rc, Weak},
};

use crate::mechanical_port::source::{
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    assets::script_asset::ScriptAsset,
    bindable_artboard::RuntimeBindableArtboardHandle,
    core::{
        Core, CoreArena, CoreHandle,
        binary_reader::BinaryReader,
        field_types::{
            core_color_type::CoreColorType, core_double_type::CoreDoubleType,
            core_string_type::CoreStringType, core_uint_type::CoreUintType,
        },
    },
    data_resolver::DataResolver,
    factory::RuntimeFactoryHandle,
    file_asset_loader::FileAssetLoaderRef,
    generated::{
        artboard_base::ArtboardBase, assets::file_asset_base::FileAssetBase,
        backboard_base::BackboardBase, core_registry::CoreRegistry,
    },
    importers::{
        artboard_importer::ArtboardImporter,
        backboard_importer::BackboardImporter,
        bindable_property_importer::BindablePropertyImporter,
        data_bind_path_importer::DataBindPathImporter,
        data_converter_formula_importer::DataConverterFormulaImporter,
        data_converter_group_importer::DataConverterGroupImporter,
        enum_importer::EnumImporter,
        file_asset_importer::FileAssetImporter,
        import_stack::{ImportStack, ImportStackObject},
        keyed_object_importer::KeyedObjectImporter,
        keyed_property_importer::KeyedPropertyImporter,
        layer_state_importer::LayerStateImporter,
        linear_animation_importer::LinearAnimationImporter,
        listener_input_type_gamepad_importer::ListenerInputTypeGamepadImporter,
        listener_input_type_keyboard_importer::ListenerInputTypeKeyboardImporter,
        listener_input_type_semantic_importer::ListenerInputTypeSemanticImporter,
        scripted_object_importer::ScriptedObjectImporter,
        state_machine_importer::StateMachineImporter,
        state_machine_layer_component_importer::StateMachineLayerComponentImporter,
        state_machine_layer_importer::StateMachineLayerImporter,
        state_machine_listener_importer::StateMachineListenerImporter,
        state_transition_importer::StateTransitionImporter,
        transition_viewmodel_condition_importer::TransitionViewModelConditionImporter,
        viewmodel_importer::ViewModelImporter,
        viewmodel_instance_importer::ViewModelInstanceImporter,
        viewmodel_instance_list_importer::ViewModelInstanceListImporter,
    },
    lua::scripting_vm::RuntimeScriptingVmHandle,
    runtime_header::RuntimeHeader,
    status_code::StatusCode,
    view_model_type::ViewModelType,
    viewmodel::{
        runtime::viewmodel_runtime::RuntimeViewModelHandle, viewmodel::ViewModel,
        viewmodel_instance::ViewModelInstance,
    },
};
use crate::scripting::ScriptAssetRegistration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImportResult {
    #[default]
    Success,
    UnsupportedVersion,
    Malformed,
}

/// Explicit host resource admission. Ordinary upstream imports do not install
/// this policy. Checks observe source-accepted objects and precede asset loaders
/// and decoders; rejection aborts import without registering scripts.
pub trait ImportAdmission {
    fn admit_object(&self, object: &CoreHandle) -> bool;
    fn admit_asset_bytes(&self, asset: &CoreHandle, bytes: &[u8]) -> bool;
    fn admit_loaded_asset(&self, asset: &CoreHandle) -> bool;
    fn is_rejected(&self) -> bool;
}

pub type ImportAdmissionRef = Rc<dyn ImportAdmission>;

/// Shared identity for the one non-Core File occurrence that owns an imported
/// runtime graph. Consumers may only borrow it for the duration of a closure.
#[derive(Clone)]
pub struct RuntimeFileHandle(Rc<RefCell<File>>, Rc<RefCell<Vec<CoreHandle>>>);

#[derive(Clone, Default)]
pub struct RuntimeFileWeakHandle(Weak<RefCell<File>>, Weak<RefCell<Vec<CoreHandle>>>);

impl RuntimeFileHandle {
    pub fn new(file: File) -> Self {
        // The File's one canonical model table is also reachable during its
        // synchronous import callbacks, without reborrowing File::read.
        let models = file.view_models.clone();
        let handle = Self(Rc::new(RefCell::new(file)), models);
        handle.0.borrow_mut().self_handle = handle.downgrade();
        handle
    }

    pub fn downgrade(&self) -> RuntimeFileWeakHandle {
        RuntimeFileWeakHandle(Rc::downgrade(&self.0), Rc::downgrade(&self.1))
    }

    pub fn with_file<R>(&self, f: impl FnOnce(&File) -> R) -> R {
        f(&self.0.borrow())
    }

    pub fn with_file_mut<R>(&self, f: impl FnOnce(&mut File) -> R) -> R {
        f(&mut self.0.borrow_mut())
    }

    pub fn complete_view_model_properties(&self, instance: &CoreHandle) {
        File::complete_view_model_properties_in(&self.1, instance);
    }

    pub fn view_model(&self, index: usize) -> Option<CoreHandle> {
        self.1.borrow().get(index).cloned()
    }
}

impl RuntimeFileWeakHandle {
    pub fn upgrade(&self) -> Option<RuntimeFileHandle> {
        Some(RuntimeFileHandle(self.0.upgrade()?, self.1.upgrade()?))
    }

    pub fn complete_view_model_properties(&self, instance: &CoreHandle) {
        if let Some(file) = self.upgrade() {
            file.complete_view_model_properties(instance);
        }
    }

    pub fn view_model(&self, index: usize) -> Option<CoreHandle> {
        self.upgrade()?.view_model(index)
    }

    pub fn with_file<R>(&self, f: impl FnOnce(&File) -> R) -> Option<R> {
        self.upgrade().map(|file| file.with_file(f))
    }

    pub fn with_file_mut<R>(&self, f: impl FnOnce(&mut File) -> R) -> Option<R> {
        self.upgrade().map(|file| file.with_file_mut(f))
    }
}

pub static DETERMINISTIC_MODE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(debug_assertions)]
pub static DEBUG_TOTAL_FILE_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(feature = "tools")]
pub trait ViewModelInstanceRegistrar {
    fn register_instance(&mut self, instance: CoreHandle);
    fn contains(&self, instance: &CoreHandle) -> bool;
    fn clear(&mut self);
}

#[cfg(feature = "tools")]
#[derive(Clone)]
pub struct ViewModelInstanceRegistrarHandle(Rc<RefCell<Box<dyn ViewModelInstanceRegistrar>>>);

#[cfg(feature = "tools")]
impl ViewModelInstanceRegistrarHandle {
    pub fn new(registrar: Box<dyn ViewModelInstanceRegistrar>) -> Self {
        Self(Rc::new(RefCell::new(registrar)))
    }

    fn with_mut<R>(
        &self,
        use_registrar: impl FnOnce(&mut dyn ViewModelInstanceRegistrar) -> R,
    ) -> R {
        use_registrar(self.0.borrow_mut().as_mut())
    }
}

fn read_runtime_object(
    reader: &mut BinaryReader<'_>,
    header: &RuntimeHeader,
) -> Option<Box<dyn crate::mechanical_port::source::core::CoreObject>> {
    let core_object_key = reader.read_var_uint_as::<i32>();
    let mut object = CoreRegistry::make_core_box(core_object_key);
    loop {
        let property_key = reader.read_var_uint_as::<u16>();
        if property_key == 0 {
            break;
        }
        if reader.has_error() {
            return None;
        }
        let handled = object
            .as_deref_mut()
            .is_some_and(|object| object.deserialize(property_key, reader));
        if !handled {
            let mut field_id = CoreRegistry::property_field_id(property_key as i32);
            if field_id == -1 {
                field_id = header.property_field_id(property_key as i32);
            }
            if field_id == -1 {
                eprintln!(
                    "Unknown property key {}, missing from property ToC.",
                    property_key
                );
                return None;
            }
            match field_id {
                CoreUintType::ID => {
                    // Uint64 shares the uint field id, so skip its full range.
                    reader.read_var_uint64();
                }
                CoreStringType::ID => {
                    CoreStringType::deserialize(reader);
                }
                CoreDoubleType::ID => {
                    CoreDoubleType::deserialize(reader);
                }
                CoreColorType::ID => {
                    CoreColorType::deserialize(reader);
                }
                _ => {}
            }
        }
    }
    object
}

pub struct File {
    self_handle: RuntimeFileWeakHandle,
    core_arena: CoreArena,
    backboard: Option<CoreHandle>,
    file_assets: Vec<CoreHandle>,
    data_converters: Vec<CoreHandle>,
    keyframe_interpolators: Vec<CoreHandle>,
    scripted_interpolators: Vec<CoreHandle>,
    scroll_physics: Vec<CoreHandle>,
    artboards: Vec<CoreHandle>,
    view_models: Rc<RefCell<Vec<CoreHandle>>>,
    view_model_instances: Rc<RefCell<Vec<CoreHandle>>>,
    enums: Vec<CoreHandle>,
    factory: RuntimeFactoryHandle,
    asset_loader: Option<FileAssetLoaderRef>,
    scripting_vm: Option<RuntimeScriptingVmHandle>,
    #[cfg(feature = "tools")]
    view_model_instance_registrar: Option<ViewModelInstanceRegistrarHandle>,
    manifest: Option<CoreHandle>,
    has_audio: bool,
}

impl Drop for File {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        DEBUG_TOTAL_FILE_COUNT.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        self.cleanup_scripting_vm();
        self.artboards.clear();
        self.view_models.borrow_mut().clear();
        #[cfg(feature = "tools")]
        {
            self.view_model_instance_registrar = None;
        }
        self.enums.clear();
        self.data_converters.clear();
        self.keyframe_interpolators.clear();
        self.scroll_physics.clear();
        self.backboard = None;
    }
}

impl File {
    pub const MAJOR_VERSION: i32 = 7;
    pub const MINOR_VERSION: i32 = 2;

    pub fn set_deterministic_mode(value: bool) {
        DETERMINISTIC_MODE.store(value, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn deterministic_mode() -> bool {
        DETERMINISTIC_MODE.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn new(factory: RuntimeFactoryHandle, asset_loader: Option<FileAssetLoaderRef>) -> Self {
        #[cfg(debug_assertions)]
        DEBUG_TOTAL_FILE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self {
            self_handle: RuntimeFileWeakHandle::default(),
            core_arena: CoreArena::default(),
            backboard: None,
            file_assets: Vec::new(),
            data_converters: Vec::new(),
            keyframe_interpolators: Vec::new(),
            scripted_interpolators: Vec::new(),
            scroll_physics: Vec::new(),
            artboards: Vec::new(),
            view_models: Rc::default(),
            view_model_instances: Rc::default(),
            enums: Vec::new(),
            factory,
            asset_loader,
            scripting_vm: None,
            #[cfg(feature = "tools")]
            view_model_instance_registrar: None,
            manifest: None,
            has_audio: false,
        }
    }

    pub fn import(
        bytes: &[u8],
        factory: RuntimeFactoryHandle,
        result: Option<&mut ImportResult>,
        asset_loader: Option<FileAssetLoaderRef>,
        scripting_vm: Option<RuntimeScriptingVmHandle>,
    ) -> Option<RuntimeFileHandle> {
        Self::import_internal(bytes, factory, result, asset_loader, scripting_vm, None)
    }

    pub fn import_with_loader(
        bytes: &[u8],
        factory: RuntimeFactoryHandle,
        result: Option<&mut ImportResult>,
        asset_loader: FileAssetLoaderRef,
        scripting_vm: Option<RuntimeScriptingVmHandle>,
    ) -> Option<RuntimeFileHandle> {
        Self::import_internal(
            bytes,
            factory,
            result,
            Some(asset_loader),
            scripting_vm,
            None,
        )
    }

    pub fn import_with_admission(
        bytes: &[u8],
        factory: RuntimeFactoryHandle,
        result: Option<&mut ImportResult>,
        asset_loader: Option<FileAssetLoaderRef>,
        scripting_vm: Option<RuntimeScriptingVmHandle>,
        admission: ImportAdmissionRef,
    ) -> Option<RuntimeFileHandle> {
        Self::import_internal(
            bytes,
            factory,
            result,
            asset_loader,
            scripting_vm,
            Some(admission),
        )
    }

    fn import_internal(
        bytes: &[u8],
        factory: RuntimeFactoryHandle,
        mut result: Option<&mut ImportResult>,
        asset_loader: Option<FileAssetLoaderRef>,
        scripting_vm: Option<RuntimeScriptingVmHandle>,
        admission: Option<ImportAdmissionRef>,
    ) -> Option<RuntimeFileHandle> {
        if scripting_vm
            .as_ref()
            .is_some_and(|vm| vm.install_render_factory(&factory).is_err())
        {
            if let Some(result) = result.as_deref_mut() {
                *result = ImportResult::Malformed;
            }
            return None;
        }
        let mut reader = BinaryReader::new(bytes);
        let mut header = RuntimeHeader::default();
        if !RuntimeHeader::read(&mut reader, &mut header) {
            eprintln!("Bad header");
            if let Some(result) = result.as_deref_mut() {
                *result = ImportResult::Malformed;
            }
            return None;
        }
        if header.major_version() != Self::MAJOR_VERSION {
            eprintln!(
                "Unsupported version {}.{} expected {}.{}.",
                header.major_version(),
                header.minor_version(),
                Self::MAJOR_VERSION,
                Self::MINOR_VERSION
            );
            if let Some(result) = result.as_deref_mut() {
                *result = ImportResult::UnsupportedVersion;
            }
            return None;
        }

        let file = RuntimeFileHandle::new(File::new(factory, asset_loader));
        let (read_result, registration_ready) = file.with_file_mut(|file| {
            file.set_scripting_vm(scripting_vm);
            file.read(&mut reader, &header, admission.clone())
        });
        if registration_ready
            && !admission
                .as_ref()
                .is_some_and(|policy| policy.is_rejected())
        {
            Self::register_scripts(&file);
        }
        if let Some(result) = result.as_deref_mut() {
            *result = read_result;
        }
        if read_result != ImportResult::Success {
            return None;
        }
        Some(file)
    }

    fn read(
        &mut self,
        reader: &mut BinaryReader<'_>,
        header: &RuntimeHeader,
        admission: Option<ImportAdmissionRef>,
    ) -> (ImportResult, bool) {
        let mut import_stack = ImportStack::default();
        import_stack.set_version(header.major_version(), header.minor_version());
        let in_band_content = Rc::new(RefCell::new(Vec::new()));
        // Core has no type key, so the most recent non-bind object remains the
        // target for an immediately following DataBind.
        let mut last_bindable_object: Option<CoreHandle> = None;
        // Host source identity is the dense order of concrete Core records
        // accepted from the file. Importers may allocate synthetic owners in
        // the same arena, so an arena slot is not an authored global id.
        let mut source_global_id = 0_u32;

        while !reader.reached_end() {
            let Some(object) = read_runtime_object(reader, header) else {
                import_stack.read_null_object();
                continue;
            };
            let object_source_global_id = source_global_id;
            let Some(next_source_global_id) = source_global_id.checked_add(1) else {
                return (ImportResult::Malformed, false);
            };
            source_global_id = next_source_global_id;
            let object = self.core_arena.insert_boxed(object);
            if !object.set_source_global_id(object_source_global_id) {
                return (ImportResult::Malformed, false);
            }
            let object_type = object.core_type().unwrap_or_default();
            if !object.is_type_of(
                crate::mechanical_port::source::generated::data_bind::data_bind_base::DataBindBase::TYPE_KEY,
            ) {
                last_bindable_object = Some(object.clone());
            } else if let Some(target) = last_bindable_object.as_ref() {
                object.with_mut(|object| {
                    if let Some(bind) = object.as_data_bind_mut() {
                        bind.set_target(Some(target.clone()));
                    }
                });
            }

            let import_result = if object
                .with(|object| object.as_data_bind().is_some())
                .unwrap_or(false)
            {
                crate::mechanical_port::source::data_bind::data_bind::DataBind::import_handle(
                    &object,
                    &mut import_stack,
                )
            } else if object_type
                == crate::mechanical_port::source::generated::assets::file_asset_contents_base::FileAssetContentsBase::TYPE_KEY
            {
                crate::mechanical_port::source::assets::file_asset_contents::FileAssetContents::import_handle(
                    &object,
                    &mut import_stack,
                )
            } else {
                object
                    .with_mut(|object| object.import(&mut import_stack))
                    .unwrap_or(StatusCode::MissingObject)
            };
            if import_result == StatusCode::Ok {
                if admission
                    .as_ref()
                    .is_some_and(|policy| !policy.admit_object(&object))
                {
                    return (ImportResult::Malformed, false);
                }
                match object_type {
                    BackboardBase::TYPE_KEY => {
                        self.backboard = Some(object.clone());
                    }
                    ArtboardBase::TYPE_KEY => {
                        let factory = self.factory.clone();
                        object.with_downcast_mut::<Artboard, _>(|artboard| {
                            artboard.set_core_arena(self.core_arena.clone());
                            artboard.set_factory(factory);
                            artboard.set_file(self.self_handle.clone());
                            artboard.set_scripting_vm(self.scripting_vm.clone());
                        });
                        self.artboards.push(object.clone());
                    }
                    crate::mechanical_port::source::generated::assets::image_asset_base::ImageAssetBase::TYPE_KEY
                    | crate::mechanical_port::source::generated::assets::font_asset_base::FontAssetBase::TYPE_KEY
                    | crate::mechanical_port::source::generated::assets::audio_asset_base::AudioAssetBase::TYPE_KEY
                    | crate::mechanical_port::source::generated::assets::blob_asset_base::BlobAssetBase::TYPE_KEY
                    | crate::mechanical_port::source::generated::assets::script_asset_base::ScriptAssetBase::TYPE_KEY
                    | crate::mechanical_port::source::generated::assets::shader_asset_base::ShaderAssetBase::TYPE_KEY => {
                        self.file_assets.push(object.clone());
                        if object_type
                            == crate::mechanical_port::source::generated::assets::audio_asset_base::AudioAssetBase::TYPE_KEY
                        {
                            self.has_audio = true;
                        }
                    }
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_base::ViewModelBase::TYPE_KEY => {
                        self.view_models.borrow_mut().push(object.clone());
                    }
                    crate::mechanical_port::source::generated::viewmodel::data_enum_base::DataEnumBase::TYPE_KEY
                    | crate::mechanical_port::source::generated::viewmodel::data_enum_custom_base::DataEnumCustomBase::TYPE_KEY => {
                        self.enums.push(object.clone());
                    }
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_enum_custom_base::ViewModelPropertyEnumCustomBase::TYPE_KEY => {
                        object.with_downcast_mut::<crate::mechanical_port::source::viewmodel::viewmodel_property_enum_custom::ViewModelPropertyEnumCustom, _>(|property| {
                            if let Some(data_enum) = self.enums.get(property.base.enum_id() as usize) {
                                property.base.base.set_data_enum(data_enum.clone());
                            }
                        });
                    }
                    _ => {}
                }
            } else {
                if last_bindable_object.as_ref() == Some(&object) {
                    last_bindable_object = None;
                }
                eprintln!("Failed to import object of type {}", object_type);
                // File::read deletes an object immediately when import fails.
                drop(self.core_arena.remove(&object));
                continue;
            }

            let mut stack_object: Option<Box<dyn ImportStackObject>> = None;
            let mut stack_type = object_type;
            match stack_type {
                BackboardBase::TYPE_KEY => {
                    let mut importer = Box::new(BackboardImporter::new(object.clone()));
                    importer.set_file(
                        self.self_handle.clone(),
                        self.view_models.clone(),
                        self.view_model_instances.clone(),
                    );
                    stack_object = Some(importer);
                }
                ArtboardBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ArtboardImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::viewmodel::data_enum_custom_base::DataEnumCustomBase::TYPE_KEY => {
                    stack_object = Some(Box::new(EnumImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::animation::linear_animation_base::LinearAnimationBase::TYPE_KEY => {
                    stack_object = Some(Box::new(LinearAnimationImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::animation::keyed_object_base::KeyedObjectBase::TYPE_KEY => {
                    stack_object = Some(Box::new(KeyedObjectImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::animation::keyed_property_base::KeyedPropertyBase::TYPE_KEY => {
                    let Some(importer) = import_stack.latest::<LinearAnimationImporter>(
                        crate::mechanical_port::source::generated::animation::linear_animation_base::LinearAnimationBase::TYPE_KEY,
                    ) else { return (ImportResult::Malformed, false); };
                    stack_object = Some(Box::new(KeyedPropertyImporter::new(importer.animation(), object.clone())));
                }
                crate::mechanical_port::source::generated::animation::state_machine_base::StateMachineBase::TYPE_KEY => {
                    stack_object = Some(Box::new(StateMachineImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::animation::state_machine_layer_base::StateMachineLayerBase::TYPE_KEY => {
                    let Some(importer) = import_stack.latest::<ArtboardImporter>(
                        crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
                    ) else { return (ImportResult::Malformed, false); };
                    stack_object = Some(Box::new(StateMachineLayerImporter::new(object.clone(), importer.artboard())));
                }
                crate::mechanical_port::source::generated::animation::entry_state_base::EntryStateBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::exit_state_base::ExitStateBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::any_state_base::AnyStateBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::animation_state_base::AnimationStateBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::blend_state_1d_viewmodel_base::BlendState1DViewModelBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::blend_state_1d_input_base::BlendState1DInputBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::blend_state_direct_base::BlendStateDirectBase::TYPE_KEY => {
                    stack_object = Some(Box::new(LayerStateImporter::new(object.clone())));
                    stack_type = crate::mechanical_port::source::generated::animation::layer_state_base::LayerStateBase::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::animation::state_transition_base::StateTransitionBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::blend_state_transition_base::BlendStateTransitionBase::TYPE_KEY => {
                    stack_object = Some(Box::new(StateTransitionImporter::new(object.clone())));
                    stack_type = crate::mechanical_port::source::generated::animation::state_transition_base::StateTransitionBase::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::animation::state_machine_listener_base::StateMachineListenerBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::state_machine_listener_single_base::StateMachineListenerSingleBase::TYPE_KEY => {
                    stack_object = Some(Box::new(StateMachineListenerImporter::new(object.clone())));
                    stack_type = crate::mechanical_port::source::generated::animation::state_machine_listener_base::StateMachineListenerBase::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::assets::image_asset_base::ImageAssetBase::TYPE_KEY
                | crate::mechanical_port::source::generated::assets::font_asset_base::FontAssetBase::TYPE_KEY
                | crate::mechanical_port::source::generated::assets::audio_asset_base::AudioAssetBase::TYPE_KEY
                | crate::mechanical_port::source::generated::assets::blob_asset_base::BlobAssetBase::TYPE_KEY => {
                    stack_object = Some(Box::new(FileAssetImporter::new(
                        object.clone(),
                        self.asset_loader.clone(),
                        self.factory.clone(),
                    ).with_admission(admission.clone())));
                    stack_type = crate::mechanical_port::source::generated::assets::file_asset_base::FileAssetBase::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::animation::listener_types::listener_input_type_keyboard_base::ListenerInputTypeKeyboardBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ListenerInputTypeKeyboardImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::animation::listener_types::listener_input_type_gamepad_base::ListenerInputTypeGamepadBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ListenerInputTypeGamepadImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::animation::listener_types::listener_input_type_semantic_base::ListenerInputTypeSemanticBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ListenerInputTypeSemanticImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::assets::script_asset_base::ScriptAssetBase::TYPE_KEY => {
                    object.with_downcast_mut::<ScriptAsset, _>(|script| {
                        script.set_file(Some(self.self_handle.clone()));
                        script.set_scripting_vm(self.scripting_vm.clone());
                    });
                    stack_object = Some(Box::new(
                        crate::mechanical_port::source::importers::text_asset_importer::TextAssetImporter::new(
                            object.clone(),
                            self.asset_loader.clone(),
                            self.factory.clone(),
                            in_band_content.clone(),
                        ).with_admission(admission.clone()),
                    ));
                    stack_type = crate::mechanical_port::source::generated::assets::file_asset_base::FileAssetBase::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::assets::shader_asset_base::ShaderAssetBase::TYPE_KEY => {
                    stack_object = Some(Box::new(
                        crate::mechanical_port::source::importers::text_asset_importer::TextAssetImporter::new(
                            object.clone(),
                            self.asset_loader.clone(),
                            self.factory.clone(),
                            in_band_content.clone(),
                        ).with_admission(admission.clone()),
                    ));
                    stack_type = crate::mechanical_port::source::generated::assets::file_asset_base::FileAssetBase::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::assets::manifest_asset_base::ManifestAssetBase::TYPE_KEY => {
                    stack_object = Some(Box::new(FileAssetImporter::new(
                        object.clone(),
                        self.asset_loader.clone(),
                        self.factory.clone(),
                    ).with_admission(admission.clone())));
                    stack_type = FileAssetBase::TYPE_KEY;
                    self.manifest = Some(object.clone());
                }
                crate::mechanical_port::source::generated::viewmodel::viewmodel_base::ViewModelBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ViewModelImporter::new(object.clone())));
                    stack_type = crate::mechanical_port::source::generated::viewmodel::viewmodel_base::ViewModelBase::TYPE_KEY;
                    let file = self.self_handle.clone();
                    object.with_downcast_mut::<ViewModel, _>(|view_model| view_model.set_file(file));
                }
                crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ViewModelInstanceImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_list_base::ViewModelInstanceListBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ViewModelInstanceListImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::animation::transition_viewmodel_condition_base::TransitionViewModelConditionBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::transition_artboard_condition_base::TransitionArtboardConditionBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::transition_focus_condition_base::TransitionFocusConditionBase::TYPE_KEY => {
                    stack_object = Some(Box::new(TransitionViewModelConditionImporter::new(object.clone())));
                    stack_type = crate::mechanical_port::source::generated::animation::transition_viewmodel_condition_base::TransitionViewModelConditionBase::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::data_bind::bindable_property_number_base::BindablePropertyNumberBase::TYPE_KEY
                | crate::mechanical_port::source::generated::data_bind::bindable_property_string_base::BindablePropertyStringBase::TYPE_KEY
                | crate::mechanical_port::source::generated::data_bind::bindable_property_color_base::BindablePropertyColorBase::TYPE_KEY
                | crate::mechanical_port::source::generated::data_bind::bindable_property_enum_base::BindablePropertyEnumBase::TYPE_KEY
                | crate::mechanical_port::source::generated::data_bind::bindable_property_boolean_base::BindablePropertyBooleanBase::TYPE_KEY
                | crate::mechanical_port::source::generated::data_bind::bindable_property_asset_base::BindablePropertyAssetBase::TYPE_KEY
                | crate::mechanical_port::source::generated::data_bind::bindable_property_viewmodel_base::BindablePropertyViewModelBase::TYPE_KEY
                | crate::mechanical_port::source::generated::data_bind::bindable_property_artboard_base::BindablePropertyArtboardBase::TYPE_KEY
                | crate::mechanical_port::source::generated::data_bind::bindable_property_trigger_base::BindablePropertyTriggerBase::TYPE_KEY
                | crate::mechanical_port::source::generated::data_bind::bindable_property_integer_base::BindablePropertyIntegerBase::TYPE_KEY
                | crate::mechanical_port::source::generated::data_bind::bindable_property_list_base::BindablePropertyListBase::TYPE_KEY => {
                    stack_object = Some(Box::new(BindablePropertyImporter::new(object.clone())));
                    stack_type = crate::mechanical_port::source::generated::data_bind::bindable_property_base::BindablePropertyBase::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::data_bind::converters::data_converter_group_base::DataConverterGroupBase::TYPE_KEY => {
                    stack_object = Some(Box::new(DataConverterGroupImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::data_bind::converters::data_converter_formula_base::DataConverterFormulaBase::TYPE_KEY => {
                    stack_object = Some(Box::new(DataConverterFormulaImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::data_bind::converters::data_converter_number_to_list_base::DataConverterNumberToListBase::TYPE_KEY => {
                    let file = self.self_handle.clone();
                    object.with_downcast_mut::<crate::mechanical_port::source::data_bind::converters::data_converter_number_to_list::DataConverterNumberToList, _>(|converter| converter.set_file(Some(file)));
                }
                crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase::TYPE_KEY => {
                    object.with_mut(|object| {
                        if let Some(list) = object.as_artboard_component_list_mut() {
                            list.set_file(Some(self.self_handle.clone()));
                        }
                    });
                }
                crate::mechanical_port::source::generated::nested_artboard_base::NestedArtboardBase::TYPE_KEY
                | crate::mechanical_port::source::generated::nested_artboard_layout_base::NestedArtboardLayoutBase::TYPE_KEY
                | crate::mechanical_port::source::generated::nested_artboard_leaf_base::NestedArtboardLeafBase::TYPE_KEY => {
                    object.with_mut(|object| {
                        if let Some(nested) = object.as_nested_artboard_mut() {
                            nested.set_file(self.self_handle.clone());
                        }
                    });
                }
                crate::mechanical_port::source::generated::scripted::scripted_data_converter_base::ScriptedDataConverterBase::TYPE_KEY
                | crate::mechanical_port::source::generated::scripted::scripted_drawable_base::ScriptedDrawableBase::TYPE_KEY
                | crate::mechanical_port::source::generated::scripted::scripted_layout_base::ScriptedLayoutBase::TYPE_KEY
                | crate::mechanical_port::source::generated::scripted::scripted_path_effect_base::ScriptedPathEffectBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::scripted_listener_action_base::ScriptedListenerActionBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::scripted_transition_condition_base::ScriptedTransitionConditionBase::TYPE_KEY
                | crate::mechanical_port::source::generated::scripted::scripted_interpolator_base::ScriptedInterpolatorBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ScriptedObjectImporter::new(object.clone())));
                    stack_type = crate::mechanical_port::source::generated::scripted::scripted_drawable_base::ScriptedDrawableBase::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::data_bind::data_bind_path_base::DataBindPathBase::TYPE_KEY => {
                    stack_object = Some(Box::new(DataBindPathImporter::new(object.clone())));
                }
                crate::mechanical_port::source::generated::script_input_artboard_base::ScriptInputArtboardBase::TYPE_KEY => {
                    object.with_downcast_mut::<crate::mechanical_port::source::script_input_artboard::ScriptInputArtboard, _>(|input| {
                        input.set_file(Some(self.self_handle.clone()));
                    });
                }
                _ => {}
            }

            if import_stack.make_latest(stack_type, stack_object) != StatusCode::Ok {
                return (ImportResult::Malformed, false);
            }
            if object.is_type_of(crate::mechanical_port::source::generated::animation::state_machine_layer_component_base::StateMachineLayerComponentBase::TYPE_KEY) {
                if import_stack.make_latest(
                    crate::mechanical_port::source::generated::animation::state_machine_layer_component_base::StateMachineLayerComponentBase::TYPE_KEY,
                    Some(Box::new(StateMachineLayerComponentImporter::new(object.clone()))),
                ) != StatusCode::Ok
                {
                    return (ImportResult::Malformed, false);
                }
            }
            if object.is_type_of(crate::mechanical_port::source::generated::data_bind::converters::data_converter_base::DataConverterBase::TYPE_KEY) {
                self.data_converters.push(object.clone());
            } else if object.is_type_of(crate::mechanical_port::source::generated::animation::keyframe_interpolator_base::KeyFrameInterpolatorBase::TYPE_KEY) {
                let artboard_importer = import_stack.latest::<ArtboardImporter>(
                    crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
                );
                if artboard_importer.is_none() {
                    self.keyframe_interpolators.push(object.clone());
                }
                if object.is_type_of(crate::mechanical_port::source::generated::scripted::scripted_interpolator_base::ScriptedInterpolatorBase::TYPE_KEY) {
                    self.scripted_interpolators.push(object.clone());
                }
            } else if object.is_type_of(crate::mechanical_port::source::generated::constraints::scrolling::scroll_physics_base::ScrollPhysicsBase::TYPE_KEY) {
                self.scroll_physics.push(object.clone());
            }
        }

        let resolved = import_stack.resolve();
        if !reader.has_error() && resolved == StatusCode::Ok {
            (ImportResult::Success, true)
        } else {
            (ImportResult::Malformed, true)
        }
    }

    pub fn add_file_view_model_instance(&mut self, instance: CoreHandle) {
        self.view_model_instances.borrow_mut().push(instance);
    }

    fn register_scripts(file: &RuntimeFileHandle) {
        let (assets, vm, interpolators, models) = file.with_file(|file| {
            (
                file.file_assets.clone(),
                file.scripting_vm.clone(),
                file.scripted_interpolators.clone(),
                file.view_models.borrow().clone(),
            )
        });
        let scripts: Vec<_> = assets
            .iter()
            .filter(|asset| {
                asset
                    .with_downcast::<ScriptAsset, _>(|_| true)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        if scripts.is_empty() {
            return;
        }
        let Some(vm) = vm else {
            return;
        };
        if let Err(error) = vm.with_vm_mut(|vm| vm.install_native_file_assets(file.downgrade())) {
            eprintln!("{error}");
        }
        if !vm.with_vm_mut(|vm| vm.initializes_data_global_externally()) {
            let models = models
                .into_iter()
                .map(|model| {
                    let name = model
                        .with_downcast::<ViewModel, _>(|model| model.base.name().to_owned())
                        .expect("File owns ViewModel definitions");
                    (
                        name,
                        crate::scripting::ScriptViewModel::from_native_file_definition(model, file),
                    )
                })
                .collect();
            if let Err(error) = vm.with_vm_mut(|vm| vm.initialize_data_global(models)) {
                eprintln!("{error}");
            }
        }

        let owned_modules: Vec<(String, Vec<u8>, CoreHandle, bool, Vec<String>)> = scripts
            .iter()
            .filter_map(|script| {
                script
                    .with_downcast::<ScriptAsset, _>(|script_asset| {
                        #[cfg(not(feature = "tools"))]
                        if !script_asset.verified() {
                            return None;
                        }
                        Some((
                            script_asset.module_name(),
                            script_asset.module_bytecode().to_vec(),
                            script.clone(),
                            script_asset.is_module(),
                            script_asset
                                .module_details()
                                .missing_dependencies()
                                .into_iter()
                                .collect(),
                        ))
                    })
                    .flatten()
            })
            .collect();
        let modules: Vec<_> = owned_modules
            .iter()
            .map(
                |(name, bytes, _, is_module, dependencies)| ScriptAssetRegistration {
                    name,
                    bytecode: bytes,
                    is_protocol: !is_module,
                    missing_dependencies: dependencies.clone(),
                },
            )
            .collect();
        let results = vm.with_vm_mut(|vm| vm.register_script_assets(&modules));
        assert_eq!(
            results.len(),
            owned_modules.len(),
            "registration returns each asset's result"
        );
        for ((name, _, script, _, _), result) in owned_modules.iter().zip(results) {
            if let Some(error) = &result.error {
                eprintln!("{name}: {error}");
            }
            script.with_downcast_mut::<ScriptAsset, _>(|script| {
                for dependency in script.module_details().missing_dependencies() {
                    script
                        .module_details_mut()
                        .clear_missing_dependency(&dependency);
                }
                for dependency in result.missing_dependencies {
                    script
                        .module_details_mut()
                        .add_missing_dependency(dependency);
                }
                if result.completed {
                    script.registration_complete_native(result.program);
                }
            });
        }

        for interpolator in interpolators {
            let script = interpolator
                .with(|interpolator| {
                    interpolator
                        .as_scripted_object()
                        .and_then(|interpolator| interpolator.script_asset())
                })
                .flatten();
            if script.is_some() {
                use crate::mechanical_port::source::scripted::scripted_object::{
                    ScriptUpdateRequestHost, ScriptedObject,
                };
                let properties = ScriptedObject::custom_properties(&interpolator);
                let mut host = ScriptUpdateRequestHost::default();
                // File initializes shared interpolators but does not hydrate
                // their inputs here; per-instance clones hydrate later.
                ScriptedObject::initialize_occurrence(&interpolator, &properties, &mut host);
            }
        }
    }

    pub fn set_scripting_vm(&mut self, vm: Option<RuntimeScriptingVmHandle>) {
        self.scripting_vm = vm;
    }

    fn cleanup_scripting_vm(&mut self) {
        self.scripting_vm = None;
    }

    pub fn scripting_vm(&self) -> Option<RuntimeScriptingVmHandle> {
        self.scripting_vm.clone()
    }

    #[cfg(feature = "tools")]
    pub fn clear_scripting_vm(&mut self) {
        self.cleanup_scripting_vm();
    }

    #[cfg(feature = "tools")]
    pub fn has_vm(&self) -> bool {
        self.scripting_vm.is_some()
    }

    pub fn artboard_named_source(&self, name: &str) -> Option<CoreHandle> {
        self.artboards
            .iter()
            .find(|artboard| {
                artboard
                    .with_downcast::<Artboard, _>(|artboard| artboard.base.name() == name)
                    .unwrap_or(false)
            })
            .cloned()
    }

    pub fn artboard(&self) -> Option<CoreHandle> {
        self.artboards.first().cloned()
    }

    pub fn artboard_handle(&self, index: usize) -> Option<CoreHandle> {
        self.artboards.get(index).cloned()
    }

    pub fn artboard_at_source(&self, index: usize) -> Option<CoreHandle> {
        self.artboard_handle(index)
    }

    pub fn artboard_name_at(&self, index: usize) -> String {
        self.artboard_at_source(index)
            .and_then(|artboard| {
                artboard.with_downcast::<Artboard, _>(|artboard| artboard.base.name().to_owned())
            })
            .unwrap_or_default()
    }

    fn instance_artboard(
        &self,
        artboard: Option<CoreHandle>,
    ) -> Option<RuntimeArtboardInstanceHandle> {
        let source = artboard?;
        let instance = Artboard::instance_from_handle(&source)?;
        instance.with_artboard_mut(|instance| {
            instance.set_scripting_vm(self.scripting_vm.clone());
            instance.set_file(Some(self.self_handle.clone()));
        });
        Some(instance)
    }

    pub fn artboard_default(&self) -> Option<RuntimeArtboardInstanceHandle> {
        self.instance_artboard(self.artboard())
    }

    pub fn artboard_at(&self, index: usize) -> Option<RuntimeArtboardInstanceHandle> {
        self.instance_artboard(self.artboard_at_source(index))
    }

    pub fn artboard_named(&self, name: &str) -> Option<RuntimeArtboardInstanceHandle> {
        self.instance_artboard(self.artboard_named_source(name))
    }

    pub fn bindable_artboard_named(&self, name: &str) -> Option<RuntimeBindableArtboardHandle> {
        let source = self.artboard_named_source(name)?;
        let artboard = self.instance_artboard(Some(source.clone()))?;
        Some(RuntimeBindableArtboardHandle::new(
            Some(self.self_handle.clone()),
            artboard,
            Some(source),
        ))
    }

    pub fn bindable_artboard_default(&self) -> Option<RuntimeBindableArtboardHandle> {
        let source = self.artboard()?;
        let artboard = self.instance_artboard(Some(source.clone()))?;
        Some(RuntimeBindableArtboardHandle::new(
            Some(self.self_handle.clone()),
            artboard,
            Some(source),
        ))
    }

    pub fn internal_bindable_artboard_from_artboard(
        &self,
        source: Option<CoreHandle>,
    ) -> Option<RuntimeBindableArtboardHandle> {
        let source = source?;
        let artboard = Artboard::instance_from_handle(&source)?;
        Some(RuntimeBindableArtboardHandle::new(
            None,
            artboard,
            Some(source),
        ))
    }

    pub fn complete_view_model_instance(&self, instance: &CoreHandle) {
        self.complete_view_model_instance_with_map(instance, &mut HashMap::new());
    }

    fn complete_view_model_instance_with_map(
        &self,
        instance: &CoreHandle,
        instances: &mut HashMap<CoreHandle, CoreHandle>,
    ) {
        let Some((view_model_id, values)) = instance
            .with(|instance| {
                let instance = instance.as_view_model_instance()?;
                Some((
                    instance.base.view_model_id() as usize,
                    instance.property_values().to_vec(),
                ))
            })
            .flatten()
        else {
            return;
        };
        let Some(view_model) = self.view_model_handle(view_model_id) else {
            return;
        };

        for value in values {
            let nested_index = value
                .with(|value| {
                    value
                        .as_view_model_instance_view_model()
                        .map(|nested| nested.base.property_value())
                })
                .flatten();
            if let Some(nested_index) = nested_index {
                let property = Self::view_model_property_for(&view_model, &value);
                let reference_id = property.with_downcast::<crate::mechanical_port::source::viewmodel::viewmodel_property_viewmodel::ViewModelPropertyViewModel, _>(|property| property.base.view_model_reference_id());
                if let Some(reference_id) = reference_id {
                    let source = self
                        .view_model_handle(reference_id as usize)
                        .and_then(|model| {
                            model
                                .with_downcast::<ViewModel, _>(|model| {
                                    model.instance_at(nested_index as usize)
                                })
                                .flatten()
                        });
                    value.with_mut(|value| {
                        value
                            .as_view_model_instance_view_model_mut()
                            .expect("nested VMI value")
                            .set_parent_view_model_instance(Some(instance.clone()));
                    });
                    if let Some(source) = source {
                        let copied = if let Some(copied) = instances.get(&source) {
                            Some(copied.clone())
                        } else {
                            let copied = self.copy_view_model_instance(&source, instances);
                            if let Some(copied) = &copied {
                                instances.insert(source, copied.clone());
                            }
                            copied
                        };
                        if let Some(copied) = copied {
                            value.with_mut(|value| {
                                value
                                    .as_view_model_instance_view_model_mut()
                                    .expect("nested VMI value")
                                    .set_reference_view_model_instance(Some(copied));
                            });
                        }
                    }
                }
            } else {
                let items = value
                    .with_mut(|value| {
                        let list = value.as_view_model_instance_list_mut()?;
                        list.set_parent_view_model_instance(Some(instance.clone()));
                        Some(list.list_items().to_vec())
                    })
                    .flatten();
                for item in items.into_iter().flatten() {
                    let ids = item
                        .with(|item| {
                            let item = item.as_view_model_instance_list_item()?;
                            Some((
                                item.base.view_model_id() as usize,
                                item.base.view_model_instance_id() as usize,
                            ))
                        })
                        .flatten();
                    let Some((model_id, source_id)) = ids else {
                        continue;
                    };
                    let source = self.view_model_handle(model_id).and_then(|model| {
                        model
                            .with(|model| {
                                model
                                    .as_view_model()
                                    .and_then(|model| model.instance_at(source_id))
                            })
                            .flatten()
                    });
                    let Some(source) = source else {
                        continue;
                    };
                    let copied = if let Some(copied) = instances.get(&source) {
                        Some(copied.clone())
                    } else {
                        let copied = self.copy_view_model_instance(&source, instances);
                        if let Some(copied) = &copied {
                            instances.insert(source, copied.clone());
                        }
                        copied
                    };
                    if let Some(copied) = copied {
                        item.with_mut(|item| {
                            if let Some(item) = item.as_view_model_instance_list_item_mut() {
                                item.set_view_model_instance(Some(copied));
                            }
                        });
                    }
                }
            }
            let property = Self::view_model_property_for(&view_model, &value);
            value.with_mut(|value| {
                value
                    .as_view_model_instance_value_mut()
                    .expect("VMI property value")
                    .set_view_model_property(property);
            });
        }
    }

    fn view_model_property_for(model: &CoreHandle, value: &CoreHandle) -> CoreHandle {
        let property_id = value
            .with(|value| {
                value
                    .as_view_model_instance_value()
                    .expect("VMI property value")
                    .base
                    .view_model_property_id() as usize
            })
            .expect("live VMI property value");
        model
            .with_downcast::<ViewModel, _>(|model| model.property_at(property_id))
            .flatten()
            .expect("valid ViewModel property index")
    }

    pub fn complete_view_model_properties(&self, instance: &CoreHandle) {
        Self::complete_view_model_properties_in(&self.view_models, instance);
    }

    fn complete_view_model_properties_in(
        models: &Rc<RefCell<Vec<CoreHandle>>>,
        instance: &CoreHandle,
    ) {
        let (model_id, values) = instance
            .with_downcast::<ViewModelInstance, _>(|instance| {
                (
                    instance.base.view_model_id() as usize,
                    instance.property_values().to_vec(),
                )
            })
            .expect("completeViewModelProperties requires a live ViewModelInstance");
        let model = models.borrow()[model_id].clone();
        for value in values {
            let nested_index = value
                .with(|value| {
                    value
                        .as_view_model_instance_view_model()
                        .map(|value| value.base.property_value())
                })
                .flatten();
            if let Some(nested_index) = nested_index {
                let property = Self::view_model_property_for(&model, &value);
                if let Some(reference_id) = property.with_downcast::<crate::mechanical_port::source::viewmodel::viewmodel_property_viewmodel::ViewModelPropertyViewModel, _>(|property| property.base.view_model_reference_id()) {
                    let referenced_model = models.borrow()[reference_id as usize].clone();
                    let referenced = referenced_model.with_downcast::<ViewModel, _>(|model| {
                        model.instance_at(nested_index as usize)
                    }).flatten();
                    if let Some(referenced) = referenced {
                        Self::complete_view_model_properties_in(models, &referenced);
                    }
                }
            } else if let Some(items) = value
                .with(|value| {
                    value
                        .as_view_model_instance_list()
                        .map(|list| list.list_items().to_vec())
                })
                .flatten()
            {
                for item in items {
                    let (model_id, instance_id) = item
                        .with(|item| {
                            let item = item
                                .as_view_model_instance_list_item()
                                .expect("VMI list item");
                            (
                                item.base.view_model_id() as usize,
                                item.base.view_model_instance_id() as usize,
                            )
                        })
                        .expect("live VMI list item");
                    let model = models.borrow()[model_id].clone();
                    let referenced = model
                        .with_downcast::<ViewModel, _>(|model| model.instance_at(instance_id))
                        .flatten();
                    if let Some(referenced) = referenced {
                        Self::complete_view_model_properties_in(models, &referenced);
                    }
                }
            }
            // Source binds this property's metadata after walking its nested
            // values. This operation never clones instances or installs refs.
            let property = Self::view_model_property_for(&model, &value);
            value.with_mut(|value| {
                value
                    .as_view_model_instance_value_mut()
                    .expect("VMI property value")
                    .set_view_model_property(property);
            });
        }
    }

    fn copy_view_model_instance(
        &self,
        instance: &CoreHandle,
        instances: &mut HashMap<CoreHandle, CoreHandle>,
    ) -> Option<CoreHandle> {
        let copied = ViewModelInstance::clone_instance(instance)?;
        self.complete_view_model_instance_with_map(&copied, instances);
        #[cfg(feature = "tools")]
        self.register_view_model_instance(copied.clone());
        Some(copied)
    }

    pub fn create_view_model_instance_for_name(&self, model_name: &str) -> Option<CoreHandle> {
        self.create_view_model_instance(self.view_model_named(model_name)?)
    }

    pub fn create_view_model_instance_named(
        &self,
        model_name: &str,
        instance_name: &str,
    ) -> Option<CoreHandle> {
        let model = self.view_model_named(model_name)?;
        let source = model
            .with(|model| {
                model
                    .as_view_model()
                    .and_then(|model| model.instance_named(instance_name))
            })
            .flatten()?;
        self.copy_view_model_instance(&source, &mut HashMap::new())
    }

    pub fn create_view_model_instance_at(
        &self,
        model_index: usize,
        instance_index: usize,
    ) -> Option<CoreHandle> {
        let model = self.view_model_handle(model_index)?;
        let source = model
            .with(|model| {
                model
                    .as_view_model()
                    .and_then(|model| model.instance_at(instance_index))
            })
            .flatten()?;
        self.copy_view_model_instance(&source, &mut HashMap::new())
    }

    fn find_view_model_id(&self, search: &CoreHandle) -> u32 {
        let models = self.view_models.borrow();
        models
            .iter()
            .position(|model| model == search)
            .unwrap_or(models.len()) as u32
    }

    #[cfg(feature = "tools")]
    pub fn set_view_model_instance_registrar(
        &mut self,
        registrar: Option<ViewModelInstanceRegistrarHandle>,
    ) {
        self.view_model_instance_registrar = registrar;
    }

    #[cfg(feature = "tools")]
    pub fn register_view_model_instance(&self, instance: CoreHandle) {
        if let Some(registrar) = &self.view_model_instance_registrar {
            registrar.with_mut(|registrar| registrar.register_instance(instance));
        }
    }

    #[cfg(feature = "tools")]
    pub fn contains_view_model_instance(&self, instance: &CoreHandle) -> bool {
        self.view_model_instance_registrar
            .as_ref()
            .is_some_and(|registrar| registrar.with_mut(|registrar| registrar.contains(instance)))
    }

    #[cfg(feature = "tools")]
    pub fn clear_runtime_view_model_instances(&mut self) {
        if let Some(registrar) = &self.view_model_instance_registrar {
            registrar.with_mut(|registrar| registrar.clear());
        }
    }

    pub fn create_view_model_instance(&self, view_model: CoreHandle) -> Option<CoreHandle> {
        let instance = self.core_arena.insert(ViewModelInstance::default());
        let view_model_id = self.find_view_model_id(&view_model);
        CoreRegistry::set_uint_handle(
            &instance,
            crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase::VIEW_MODEL_ID_PROPERTY_KEY as i32,
            view_model_id,
        );
        instance.with_downcast_mut::<ViewModelInstance, _>(|instance| {
            instance.view_model(view_model.clone());
        });
        let properties = view_model.with_downcast::<ViewModel, _>(ViewModel::properties)?;
        for (property_id, property) in properties.into_iter().enumerate() {
            let property_type = property.core_type()?;
            let value_type =
                match property_type {
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_string_base::ViewModelPropertyStringBase::TYPE_KEY => crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_string_base::ViewModelInstanceStringBase::TYPE_KEY,
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_number_base::ViewModelPropertyNumberBase::TYPE_KEY => crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase::TYPE_KEY,
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_boolean_base::ViewModelPropertyBooleanBase::TYPE_KEY => crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_boolean_base::ViewModelInstanceBooleanBase::TYPE_KEY,
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_color_base::ViewModelPropertyColorBase::TYPE_KEY => crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_color_base::ViewModelInstanceColorBase::TYPE_KEY,
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_list_base::ViewModelPropertyListBase::TYPE_KEY => crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_list_base::ViewModelInstanceListBase::TYPE_KEY,
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_enum_system_base::ViewModelPropertyEnumSystemBase::TYPE_KEY
                    | crate::mechanical_port::source::generated::viewmodel::viewmodel_property_enum_custom_base::ViewModelPropertyEnumCustomBase::TYPE_KEY
                    | crate::mechanical_port::source::generated::viewmodel::viewmodel_property_enum_base::ViewModelPropertyEnumBase::TYPE_KEY => crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_enum_base::ViewModelInstanceEnumBase::TYPE_KEY,
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_trigger_base::ViewModelPropertyTriggerBase::TYPE_KEY => crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_trigger_base::ViewModelInstanceTriggerBase::TYPE_KEY,
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_viewmodel_base::ViewModelPropertyViewModelBase::TYPE_KEY => {
                        crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_viewmodel_base::ViewModelInstanceViewModelBase::TYPE_KEY
                    }
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_asset_image_base::ViewModelPropertyAssetImageBase::TYPE_KEY => crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_asset_image_base::ViewModelInstanceAssetImageBase::TYPE_KEY,
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_asset_font_base::ViewModelPropertyAssetFontBase::TYPE_KEY => crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_asset_font_base::ViewModelInstanceAssetFontBase::TYPE_KEY,
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_asset_blob_base::ViewModelPropertyAssetBlobBase::TYPE_KEY => crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_asset_blob_base::ViewModelInstanceAssetBlobBase::TYPE_KEY,
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_symbol_list_index_base::ViewModelPropertySymbolListIndexBase::TYPE_KEY => crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_symbol_list_index_base::ViewModelInstanceSymbolListIndexBase::TYPE_KEY,
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_artboard_base::ViewModelPropertyArtboardBase::TYPE_KEY => crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_artboard_base::ViewModelInstanceArtboardBase::TYPE_KEY,
                    _ => {
                        eprintln!("Missing view model property type");
                        return None;
                    }
                };
            let value = self
                .core_arena
                .insert_boxed(CoreRegistry::make_core_box(value_type as i32)?);
            value.with_mut(|value| {
                if let Some(list) = value.as_view_model_instance_list_mut() {
                    list.set_parent_view_model_instance(Some(instance.clone()));
                }
            });
            if value
                .with(|value| value.as_view_model_instance_view_model().is_some())
                .unwrap_or(false)
            {
                let reference_id = property.with_downcast::<crate::mechanical_port::source::viewmodel::viewmodel_property_viewmodel::ViewModelPropertyViewModel, _>(|property| property.base.view_model_reference_id());
                let nested = reference_id
                    .and_then(|id| self.view_model_handle(id as usize))
                    .and_then(|model| self.create_view_model_instance(model));
                if let Some(nested) = nested {
                    value.with_mut(|value| {
                        if let Some(value) = value.as_view_model_instance_view_model_mut() {
                            value.set_parent_view_model_instance(Some(instance.clone()));
                            value.set_reference_view_model_instance(Some(nested));
                        }
                    });
                }
            }
            value.with_mut(|value| {
                if let Some(value) = value.as_view_model_instance_value_mut() {
                    value.set_view_model_property(property.clone());
                }
            });
            CoreRegistry::set_uint_handle(
                &value,
                crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_value_base::ViewModelInstanceValueBase::VIEW_MODEL_PROPERTY_ID_PROPERTY_KEY as i32,
                property_id as u32,
            );
            instance
                .with_downcast_mut::<ViewModelInstance, _>(|instance| instance.add_value(value));
        }
        #[cfg(feature = "tools")]
        self.register_view_model_instance(instance.clone());
        Some(instance)
    }

    pub fn create_view_model_instance_for_artboard(
        &self,
        artboard: CoreHandle,
    ) -> Option<CoreHandle> {
        let id = artboard
            .with_downcast::<Artboard, _>(|artboard| artboard.base.view_model_id() as usize)?;
        self.create_view_model_instance(self.view_model_handle(id)?)
    }

    pub fn create_default_view_model_instance_for_artboard(
        &self,
        artboard: CoreHandle,
    ) -> Option<CoreHandle> {
        let id = artboard
            .with_downcast::<Artboard, _>(|artboard| artboard.base.view_model_id() as usize)?;
        self.create_default_view_model_instance(self.view_model_handle(id)?)
    }

    pub fn create_default_view_model_instance(&self, view_model: CoreHandle) -> Option<CoreHandle> {
        let source = view_model.with_downcast::<ViewModel, _>(ViewModel::default_instance)?;
        if let Some(source) = source {
            return self.copy_view_model_instance(&source, &mut HashMap::new());
        }
        self.create_view_model_instance(view_model)
    }

    pub fn view_model_instance_list_item(&mut self, instance: CoreHandle) -> Option<CoreHandle> {
        let view_model_id = instance
            .with_downcast::<ViewModelInstance, _>(|instance| instance.base.view_model_id())?;
        let artboard = self
            .artboards
            .iter()
            .find(|artboard| {
                artboard
                    .with_downcast::<Artboard, _>(|artboard| {
                        artboard.base.view_model_id() == view_model_id
                    })
                    .unwrap_or(false)
            })?
            .clone();
        Some(self.view_model_instance_list_item_for_artboard(instance, artboard))
    }

    pub fn view_model_instance_list_item_for_artboard(
        &mut self,
        instance: CoreHandle,
        artboard: CoreHandle,
    ) -> CoreHandle {
        let item = self.core_arena.insert(
            crate::mechanical_port::source::viewmodel::viewmodel_instance_list_item::ViewModelInstanceListItem::default(),
        );
        item.with_mut(|item| {
            if let Some(item) = item.as_view_model_instance_list_item_mut() {
                item.set_view_model_instance(Some(instance));
                item.set_artboard(Some(artboard));
            }
        });
        item
    }

    pub fn view_model_named(&self, name: &str) -> Option<CoreHandle> {
        self.view_models
            .borrow()
            .iter()
            .find(|model| {
                model
                    .with_downcast::<ViewModel, _>(|model| model.base.name() == name)
                    .unwrap_or(false)
            })
            .cloned()
    }

    pub fn view_model_handle(&self, index: usize) -> Option<CoreHandle> {
        self.view_models.borrow().get(index).cloned()
    }

    pub fn view_model(&self, index: usize) -> Option<CoreHandle> {
        self.view_model_handle(index)
    }

    pub fn view_model_id(&self, name: &str) -> u32 {
        let models = self.view_models.borrow();
        models
            .iter()
            .position(|model| {
                model
                    .with_downcast::<ViewModel, _>(|model| model.base.name() == name)
                    .unwrap_or(false)
            })
            .unwrap_or(models.len()) as u32
    }

    pub fn global_view_models(&self) -> Vec<CoreHandle> {
        self.view_models
            .borrow()
            .iter()
            .cloned()
            .filter(|model| {
                model
                    .with_downcast::<ViewModel, _>(|model| {
                        model.base.view_model_type() == ViewModelType::Global as u32
                    })
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn global_view_model_names(&self) -> Vec<String> {
        self.global_view_models()
            .iter()
            .filter_map(|model| {
                model.with_downcast::<ViewModel, _>(|model| model.base.name().to_owned())
            })
            .collect()
    }

    pub fn view_model_by_index(&self, index: usize) -> Option<RuntimeViewModelHandle> {
        if let Some(model) = self.view_model_handle(index) {
            return self.create_view_model_runtime(model);
        }
        eprintln!(
            "Could not find View Model. Index {} is out of range.",
            index
        );
        None
    }

    pub fn view_model_by_name(&self, name: &str) -> Option<RuntimeViewModelHandle> {
        for model in self.view_models.borrow().iter() {
            let matches = model
                .with_downcast::<ViewModel, _>(|model| model.base.name() == name)
                .unwrap_or(false);
            if matches {
                return self.create_view_model_runtime(model.clone());
            }
        }
        eprintln!("Could not find View Model named {}.", name);
        None
    }

    pub fn default_artboard_view_model(
        &self,
        artboard: Option<CoreHandle>,
    ) -> Option<RuntimeViewModelHandle> {
        let artboard = artboard?;
        let (id, artboard_name) = artboard.with_downcast::<Artboard, _>(|artboard| {
            (
                artboard.base.view_model_id() as usize,
                artboard.base.name().to_owned(),
            )
        })?;
        if let Some(model) = self.view_model_handle(id) {
            return self.create_view_model_runtime(model);
        }
        eprintln!(
            "Could not find a View Model linked to Artboard {}.",
            artboard_name
        );
        None
    }

    fn create_view_model_runtime(&self, model: CoreHandle) -> Option<RuntimeViewModelHandle> {
        RuntimeViewModelHandle::new(model, self.self_handle.clone())
    }

    pub fn assets(&self) -> &[CoreHandle] {
        &self.file_assets
    }

    pub fn enums(&self) -> &[CoreHandle] {
        &self.enums
    }

    #[cfg(feature = "tools")]
    pub fn strip_assets(
        bytes: &[u8],
        type_keys: &HashSet<u16>,
        mut result: Option<&mut ImportResult>,
    ) -> Vec<u8> {
        let mut stripped = Vec::with_capacity(bytes.len());
        let mut reader = BinaryReader::new(bytes);
        let mut header = RuntimeHeader::default();
        if !RuntimeHeader::read(&mut reader, &mut header) {
            if let Some(result) = result.as_deref_mut() {
                *result = ImportResult::Malformed;
            }
            return stripped;
        }
        if header.major_version() != Self::MAJOR_VERSION {
            if let Some(result) = result.as_deref_mut() {
                *result = ImportResult::UnsupportedVersion;
            }
            return stripped;
        }

        let header_length = bytes.len() - reader.position().len();
        stripped.extend_from_slice(&bytes[..header_length]);
        let mut from = header_length;
        let mut to = header_length;
        let mut last_asset_type = 0u16;
        while !reader.reached_end() {
            let object = read_runtime_object(&mut reader, &header);
            let Some(object) = object else {
                continue;
            };
            if crate::mechanical_port::source::core::CoreObject::is_type_of(
                object.as_ref(),
                FileAssetBase::TYPE_KEY,
            ) {
                last_asset_type = object.core_type();
            }
            if crate::mechanical_port::source::core::CoreObject::is_type_of(
                object.as_ref(),
                crate::mechanical_port::source::generated::assets::file_asset_contents_base::FileAssetContentsBase::TYPE_KEY,
            ) && type_keys.contains(&last_asset_type) {
                if from != to {
                    stripped.extend_from_slice(&bytes[from..to]);
                }
                from = bytes.len() - reader.position().len();
            }
            to = bytes.len() - reader.position().len();
        }
        if from != to {
            stripped.extend_from_slice(&bytes[from..to]);
        }
        // Preserve upstream's unconditional success write through result.
        *result.expect("strip_assets success requires a result pointer") = ImportResult::Success;
        stripped
    }

    pub fn core_arena(&self) -> &CoreArena {
        &self.core_arena
    }

    pub fn asset(&self, index: usize) -> Option<CoreHandle> {
        self.file_assets.get(index).cloned()
    }

    pub fn backboard(&self) -> Option<CoreHandle> {
        self.backboard.clone()
    }

    pub fn artboard_count(&self) -> usize {
        self.artboards.len()
    }

    pub fn view_model_count(&self) -> usize {
        self.view_models.borrow().len()
    }

    pub fn artboards(&self) -> Vec<CoreHandle> {
        self.artboards.clone()
    }

    pub fn has_audio(&self) -> bool {
        self.has_audio
    }

    pub fn manifest(&self) -> Option<CoreHandle> {
        self.manifest.clone()
    }

    pub fn factory(&self) -> RuntimeFactoryHandle {
        self.factory.clone()
    }

    pub fn asset_loader(&self) -> Option<FileAssetLoaderRef> {
        self.asset_loader.clone()
    }
}
