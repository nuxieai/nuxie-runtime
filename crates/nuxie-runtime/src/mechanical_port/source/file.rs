use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    ptr::NonNull,
};

use crate::mechanical_port::source::{
    animation::keyframe_interpolator::KeyFrameInterpolator,
    artboard::{Artboard, ArtboardInstance},
    assets::{file_asset::FileAsset, manifest_asset::ManifestAsset},
    backboard::Backboard,
    bindable_artboard::BindableArtboard,
    constraints::scrolling::scroll_physics::ScrollPhysics,
    core::{
        Core,
        binary_reader::BinaryReader,
        field_types::{
            core_color_type::CoreColorType, core_double_type::CoreDoubleType,
            core_string_type::CoreStringType, core_uint_type::CoreUintType,
        },
    },
    data_bind::converters::data_converter::DataConverter,
    data_resolver::DataResolver,
    factory::Factory,
    file_asset_loader::FileAssetLoader,
    generated::core_registry::CoreRegistry,
    importers::{
        ImportStackObject, artboard_importer::ArtboardImporter,
        backboard_importer::BackboardImporter,
        bindable_property_importer::BindablePropertyImporter,
        data_bind_path_importer::DataBindPathImporter,
        data_converter_formula_importer::DataConverterFormulaImporter,
        data_converter_group_importer::DataConverterGroupImporter, enum_importer::EnumImporter,
        file_asset_importer::FileAssetImporter, import_stack::ImportStack,
        keyed_object_importer::KeyedObjectImporter, keyed_property_importer::KeyedPropertyImporter,
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
    refcnt::{Rcp, RefCnt, RefCounted, make_rcp, ref_rcp},
    runtime_header::RuntimeHeader,
    status_code::StatusCode,
    view_model_type::ViewModelType,
    viewmodel::{
        data_enum::DataEnum, runtime::viewmodel_runtime::ViewModelRuntime, viewmodel::ViewModel,
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImportResult {
    #[default]
    Success,
    UnsupportedVersion,
    Malformed,
}

pub static DETERMINISTIC_MODE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
#[cfg(debug_assertions)]
pub static DEBUG_TOTAL_FILE_COUNT: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(feature = "rive_tools")]
pub trait ViewModelInstanceRegistrar {
    fn register_instance(
        &mut self,
        pointer: NonNull<ViewModelInstance>,
        reference: Rcp<ViewModelInstance>,
    );
    fn contains(&self, pointer: NonNull<ViewModelInstance>) -> bool;
    fn clear(&mut self);
}

fn read_runtime_object(reader: &mut BinaryReader<'_>, header: &RuntimeHeader) -> Option<Box<Core>> {
    let core_object_key = reader.read_var_uint_as::<i32>();
    let mut object = CoreRegistry::make_core_instance(core_object_key);
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
            let mut field_id = CoreRegistry::property_field_id(property_key);
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
    ref_count: RefCnt,
    backboard: Option<NonNull<Backboard>>,
    file_assets: Vec<Rcp<FileAsset>>,
    data_converters: Vec<NonNull<DataConverter>>,
    keyframe_interpolators: Vec<NonNull<KeyFrameInterpolator>>,
    scripted_interpolators: Vec<
        NonNull<
            crate::mechanical_port::source::scripted::scripted_interpolator::ScriptedInterpolator,
        >,
    >,
    scroll_physics: Vec<NonNull<ScrollPhysics>>,
    artboards: Vec<NonNull<Artboard>>,
    view_models: Vec<NonNull<ViewModel>>,
    view_model_instances: Vec<Rcp<ViewModelInstance>>,
    view_model_runtimes: RefCell<Vec<Rcp<ViewModelRuntime>>>,
    enums: Vec<NonNull<DataEnum>>,
    factory: NonNull<Factory>,
    asset_loader: Rcp<FileAssetLoader>,
    #[cfg(feature = "rive_scripting")]
    scripting_vm: Option<Rcp<crate::mechanical_port::source::lua::scripting_vm::ScriptingVm>>,
    #[cfg(feature = "rive_tools")]
    view_model_instance_registrar: Option<NonNull<dyn ViewModelInstanceRegistrar>>,
    manifest: Option<Rcp<FileAsset>>,
    has_audio: bool,
}

unsafe impl RefCounted for File {
    fn ref_count(&self) -> &RefCnt {
        &self.ref_count
    }
}

impl Drop for File {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        DEBUG_TOTAL_FILE_COUNT.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        #[cfg(feature = "rive_scripting")]
        self.cleanup_scripting_vm();
        for artboard in self.artboards.drain(..) {
            unsafe { drop(Box::from_raw(artboard.as_ptr())) };
        }
        for mut view_model in self.view_models.drain(..) {
            unsafe { view_model.as_mut() }.base.unref();
        }
        #[cfg(feature = "rive_tools")]
        {
            self.view_model_instance_registrar = None;
        }
        for mut data_enum in self.enums.drain(..) {
            unsafe { data_enum.as_mut() }.base.unref();
        }
        for converter in self.data_converters.drain(..) {
            unsafe { drop(Box::from_raw(converter.as_ptr())) };
        }
        for interpolator in self.keyframe_interpolators.drain(..) {
            unsafe { drop(Box::from_raw(interpolator.as_ptr())) };
        }
        for physics in self.scroll_physics.drain(..) {
            unsafe { drop(Box::from_raw(physics.as_ptr())) };
        }
        if let Some(backboard) = self.backboard.take() {
            unsafe { drop(Box::from_raw(backboard.as_ptr())) };
        }
    }
}

impl File {
    pub const MAJOR_VERSION: i32 = 7;
    pub const MINOR_VERSION: i32 = 2;

    pub fn new(factory: NonNull<Factory>, asset_loader: Rcp<FileAssetLoader>) -> Self {
        #[cfg(debug_assertions)]
        DEBUG_TOTAL_FILE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self {
            ref_count: RefCnt::new(),
            backboard: None,
            file_assets: Vec::new(),
            data_converters: Vec::new(),
            keyframe_interpolators: Vec::new(),
            scripted_interpolators: Vec::new(),
            scroll_physics: Vec::new(),
            artboards: Vec::new(),
            view_models: Vec::new(),
            view_model_instances: Vec::new(),
            view_model_runtimes: RefCell::new(Vec::new()),
            enums: Vec::new(),
            factory,
            asset_loader,
            #[cfg(feature = "rive_scripting")]
            scripting_vm: None,
            #[cfg(feature = "rive_tools")]
            view_model_instance_registrar: None,
            manifest: None,
            has_audio: false,
        }
    }

    pub fn import(
        bytes: &[u8],
        factory: NonNull<Factory>,
        mut result: Option<&mut ImportResult>,
        asset_loader: Option<NonNull<FileAssetLoader>>,
        #[cfg(feature = "rive_scripting")] scripting_vm: Option<
            NonNull<crate::mechanical_port::source::lua::scripting_vm::ScriptingVm>,
        >,
    ) -> Option<Rcp<File>> {
        let loader = unsafe { ref_rcp(asset_loader.map_or(std::ptr::null_mut(), NonNull::as_ptr)) };
        Self::import_with_loader(
            bytes,
            factory,
            result,
            loader,
            #[cfg(feature = "rive_scripting")]
            scripting_vm,
        )
    }

    pub fn import_with_loader(
        bytes: &[u8],
        factory: NonNull<Factory>,
        mut result: Option<&mut ImportResult>,
        asset_loader: Rcp<FileAssetLoader>,
        #[cfg(feature = "rive_scripting")] scripting_vm: Option<
            NonNull<crate::mechanical_port::source::lua::scripting_vm::ScriptingVm>,
        >,
    ) -> Option<Rcp<File>> {
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

        let mut file = make_rcp(File::new(factory, asset_loader));
        #[cfg(feature = "rive_scripting")]
        if let Some(vm) = scripting_vm {
            file.set_scripting_vm(unsafe { ref_rcp(vm.as_ptr()) });
        }
        let read_result = file.read(&mut reader, &header);
        if let Some(result) = result.as_deref_mut() {
            *result = read_result;
        }
        if read_result != ImportResult::Success {
            file.reset(None);
            return None;
        }
        Some(file)
    }

    fn read(&mut self, reader: &mut BinaryReader<'_>, header: &RuntimeHeader) -> ImportResult {
        let mut import_stack = ImportStack::default();
        import_stack.set_version(header.major_version(), header.minor_version());
        #[cfg(feature = "rive_scripting")]
        let mut in_band_content = Vec::new();
        // Core has no type key, so the most recent non-bind object remains the
        // target for an immediately following DataBind.
        let mut last_bindable_object: Option<NonNull<Core>> = None;

        while !reader.reached_end() {
            let Some(mut object) = read_runtime_object(reader, header) else {
                import_stack.read_null_object();
                continue;
            };
            let object_pointer = NonNull::from(object.as_mut());
            if !object.is_data_bind() {
                last_bindable_object = Some(object_pointer);
            } else if let Some(target) = last_bindable_object {
                object.as_data_bind_mut().unwrap().set_target(Some(target));
            }

            if object.import(&mut import_stack) == StatusCode::Ok {
                match object.core_type() {
                    Backboard::TYPE_KEY => {
                        self.backboard = object.as_backboard_mut().map(NonNull::from);
                    }
                    Artboard::TYPE_KEY => {
                        let mut artboard = object.as_artboard_mut().unwrap();
                        unsafe { artboard.as_mut() }.set_factory(self.factory);
                        self.artboards.push(artboard);
                    }
                    crate::mechanical_port::source::assets::image_asset::ImageAsset::TYPE_KEY
                    | crate::mechanical_port::source::assets::font_asset::FontAsset::TYPE_KEY
                    | crate::mechanical_port::source::assets::audio_asset::AudioAsset::TYPE_KEY
                    | crate::mechanical_port::source::assets::blob_asset::BlobAsset::TYPE_KEY
                    | crate::mechanical_port::source::assets::script_asset::ScriptAsset::TYPE_KEY
                    | crate::mechanical_port::source::assets::shader_asset::ShaderAsset::TYPE_KEY => {
                        let asset = object.as_file_asset_mut().unwrap();
                        self.file_assets.push(unsafe { Rcp::from_raw(asset.as_ptr()) });
                        if object.core_type()
                            == crate::mechanical_port::source::assets::audio_asset::AudioAsset::TYPE_KEY
                        {
                            self.has_audio = true;
                        }
                    }
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_base::ViewModelBase::TYPE_KEY => {
                        self.view_models.push(object.as_view_model_mut().unwrap());
                    }
                    crate::mechanical_port::source::generated::viewmodel::data_enum_base::DataEnumBase::TYPE_KEY
                    | crate::mechanical_port::source::generated::viewmodel::data_enum_custom_base::DataEnumCustomBase::TYPE_KEY => {
                        self.enums.push(object.as_data_enum_mut().unwrap());
                    }
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_enum_custom_base::ViewModelPropertyEnumCustomBase::TYPE_KEY => {
                        let value = object.as_view_model_property_enum_custom_mut().unwrap();
                        let enum_id = unsafe { value.as_ref() }.base.enum_id() as usize;
                        if enum_id < self.enums.len() {
                            unsafe { value.as_mut() }.set_data_enum(Some(self.enums[enum_id]));
                        }
                    }
                    _ => {}
                }
            } else {
                if last_bindable_object == Some(object_pointer) {
                    last_bindable_object = None;
                }
                eprintln!("Failed to import object of type {}", object.core_type());
                continue;
            }

            let mut stack_object: Option<Box<dyn ImportStackObject>> = None;
            let mut stack_type = object.core_type();
            match stack_type {
                Backboard::TYPE_KEY => {
                    let mut importer = Box::new(BackboardImporter::new(object.as_backboard_mut().unwrap()));
                    importer.set_file(NonNull::from(&mut *self));
                    stack_object = Some(importer);
                }
                Artboard::TYPE_KEY => {
                    stack_object = Some(Box::new(ArtboardImporter::new(object.as_artboard_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::viewmodel::data_enum_custom_base::DataEnumCustomBase::TYPE_KEY => {
                    stack_object = Some(Box::new(EnumImporter::new(object.as_data_enum_custom_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::animation::linear_animation_base::LinearAnimationBase::TYPE_KEY => {
                    stack_object = Some(Box::new(LinearAnimationImporter::new(object.as_linear_animation_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::animation::keyed_object_base::KeyedObjectBase::TYPE_KEY => {
                    stack_object = Some(Box::new(KeyedObjectImporter::new(object.as_keyed_object_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::animation::keyed_property_base::KeyedPropertyBase::TYPE_KEY => {
                    let Some(importer) = import_stack.latest::<LinearAnimationImporter>(
                        crate::mechanical_port::source::generated::animation::linear_animation_base::LinearAnimationBase::TYPE_KEY,
                    ) else { return ImportResult::Malformed; };
                    stack_object = Some(Box::new(KeyedPropertyImporter::new(importer.animation(), object.as_keyed_property_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::animation::state_machine_base::StateMachineBase::TYPE_KEY => {
                    stack_object = Some(Box::new(StateMachineImporter::new(object.as_state_machine_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::animation::state_machine_layer_base::StateMachineLayerBase::TYPE_KEY => {
                    let Some(importer) = import_stack.latest::<ArtboardImporter>(
                        crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
                    ) else { return ImportResult::Malformed; };
                    stack_object = Some(Box::new(StateMachineLayerImporter::new(object.as_state_machine_layer_mut().unwrap(), importer.artboard())));
                }
                crate::mechanical_port::source::generated::animation::entry_state_base::EntryStateBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::exit_state_base::ExitStateBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::any_state_base::AnyStateBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::animation_state_base::AnimationStateBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::blend_state_1d_viewmodel_base::BlendState1DViewModelBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::blend_state_1d_input_base::BlendState1DInputBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::blend_state_direct_base::BlendStateDirectBase::TYPE_KEY => {
                    stack_object = Some(Box::new(LayerStateImporter::new(object.as_layer_state_mut().unwrap())));
                    stack_type = crate::mechanical_port::source::generated::animation::layer_state_base::LayerStateBase::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::animation::state_transition_base::StateTransitionBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::blend_state_transition_base::BlendStateTransitionBase::TYPE_KEY => {
                    stack_object = Some(Box::new(StateTransitionImporter::new(object.as_state_transition_mut().unwrap())));
                    stack_type = crate::mechanical_port::source::generated::animation::state_transition_base::StateTransitionBase::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::animation::state_machine_listener_base::StateMachineListenerBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::state_machine_listener_single_base::StateMachineListenerSingleBase::TYPE_KEY => {
                    stack_object = Some(Box::new(StateMachineListenerImporter::new(object.as_state_machine_listener_mut().unwrap())));
                    stack_type = crate::mechanical_port::source::generated::animation::state_machine_listener_base::StateMachineListenerBase::TYPE_KEY;
                }
                crate::mechanical_port::source::assets::image_asset::ImageAsset::TYPE_KEY
                | crate::mechanical_port::source::assets::font_asset::FontAsset::TYPE_KEY
                | crate::mechanical_port::source::assets::audio_asset::AudioAsset::TYPE_KEY
                | crate::mechanical_port::source::assets::blob_asset::BlobAsset::TYPE_KEY => {
                    stack_object = Some(Box::new(FileAssetImporter::new(
                        object.as_file_asset_mut().unwrap(),
                        self.asset_loader.clone(),
                        self.factory,
                    )));
                    stack_type = crate::mechanical_port::source::assets::file_asset::FileAsset::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::animation::listener_types::listener_input_type_keyboard_base::ListenerInputTypeKeyboardBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ListenerInputTypeKeyboardImporter::new(object.as_listener_input_type_keyboard_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::animation::listener_types::listener_input_type_gamepad_base::ListenerInputTypeGamepadBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ListenerInputTypeGamepadImporter::new(object.as_listener_input_type_gamepad_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::animation::listener_types::listener_input_type_semantic_base::ListenerInputTypeSemanticBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ListenerInputTypeSemanticImporter::new(object.as_listener_input_type_semantic_mut().unwrap())));
                }
                #[cfg(feature = "rive_scripting")]
                crate::mechanical_port::source::assets::script_asset::ScriptAsset::TYPE_KEY => {
                    let script = object.as_script_asset_mut().unwrap();
                    stack_object = Some(Box::new(crate::mechanical_port::source::importers::text_asset_importer::TextAssetImporter::new(
                        script.cast(), self.asset_loader.clone(), self.factory, &mut in_band_content,
                    )));
                    stack_type = FileAsset::TYPE_KEY;
                    unsafe { script.as_mut() }.set_file(Some(NonNull::from(&mut *self)));
                }
                #[cfg(feature = "rive_scripting")]
                crate::mechanical_port::source::assets::shader_asset::ShaderAsset::TYPE_KEY => {
                    stack_object = Some(Box::new(crate::mechanical_port::source::importers::text_asset_importer::TextAssetImporter::new(
                        object.as_shader_asset_mut().unwrap().cast(), self.asset_loader.clone(), self.factory, &mut in_band_content,
                    )));
                    stack_type = FileAsset::TYPE_KEY;
                }
                crate::mechanical_port::source::assets::manifest_asset::ManifestAsset::TYPE_KEY => {
                    let asset = object.as_file_asset_mut().unwrap();
                    stack_object = Some(Box::new(FileAssetImporter::new(asset, self.asset_loader.clone(), self.factory)));
                    stack_type = FileAsset::TYPE_KEY;
                    self.manifest = Some(unsafe { Rcp::from_raw(asset.as_ptr()) });
                }
                crate::mechanical_port::source::generated::viewmodel::viewmodel_base::ViewModelBase::TYPE_KEY => {
                    let mut view_model = object.as_view_model_mut().unwrap();
                    stack_object = Some(Box::new(ViewModelImporter::new(view_model)));
                    stack_type = crate::mechanical_port::source::generated::viewmodel::viewmodel_base::ViewModelBase::TYPE_KEY;
                    unsafe { view_model.as_mut() }.set_file(Some(NonNull::from(&mut *self)));
                }
                crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ViewModelInstanceImporter::new(object.as_view_model_instance_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_list_base::ViewModelInstanceListBase::TYPE_KEY => {
                    stack_object = Some(Box::new(ViewModelInstanceListImporter::new(object.as_view_model_instance_list_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::animation::transition_viewmodel_condition_base::TransitionViewModelConditionBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::transition_artboard_condition_base::TransitionArtboardConditionBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::transition_focus_condition_base::TransitionFocusConditionBase::TYPE_KEY => {
                    stack_object = Some(Box::new(TransitionViewModelConditionImporter::new(object.as_transition_viewmodel_condition_mut().unwrap())));
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
                    stack_object = Some(Box::new(BindablePropertyImporter::new(object.as_bindable_property_mut().unwrap())));
                    stack_type = crate::mechanical_port::source::generated::data_bind::bindable_property_base::BindablePropertyBase::TYPE_KEY;
                }
                crate::mechanical_port::source::generated::data_bind::converters::data_converter_group_base::DataConverterGroupBase::TYPE_KEY => {
                    stack_object = Some(Box::new(DataConverterGroupImporter::new(object.as_data_converter_group_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::data_bind::converters::data_converter_formula_base::DataConverterFormulaBase::TYPE_KEY => {
                    stack_object = Some(Box::new(DataConverterFormulaImporter::new(object.as_data_converter_formula_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::data_bind::converters::data_converter_number_to_list_base::DataConverterNumberToListBase::TYPE_KEY => {
                    object.as_data_converter_number_to_list_mut().unwrap().set_file(Some(NonNull::from(&mut *self)));
                }
                crate::mechanical_port::source::generated::artboard_component_list_base::ArtboardComponentListBase::TYPE_KEY => {
                    object.as_artboard_component_list_mut().unwrap().set_file(Some(NonNull::from(&mut *self)));
                }
                crate::mechanical_port::source::generated::nested_artboard_base::NestedArtboardBase::TYPE_KEY
                | crate::mechanical_port::source::generated::nested_artboard_layout_base::NestedArtboardLayoutBase::TYPE_KEY
                | crate::mechanical_port::source::generated::nested_artboard_leaf_base::NestedArtboardLeafBase::TYPE_KEY => {
                    object
                        .as_nested_artboard_mut()
                        .unwrap()
                        .set_file(Some(NonNull::from(&mut *self)));
                }
                crate::mechanical_port::source::generated::scripted::scripted_data_converter_base::ScriptedDataConverterBase::TYPE_KEY
                | crate::mechanical_port::source::generated::scripted::scripted_drawable_base::ScriptedDrawableBase::TYPE_KEY
                | crate::mechanical_port::source::generated::scripted::scripted_layout_base::ScriptedLayoutBase::TYPE_KEY
                | crate::mechanical_port::source::generated::scripted::scripted_path_effect_base::ScriptedPathEffectBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::scripted_listener_action_base::ScriptedListenerActionBase::TYPE_KEY
                | crate::mechanical_port::source::generated::animation::scripted_transition_condition_base::ScriptedTransitionConditionBase::TYPE_KEY
                | crate::mechanical_port::source::generated::scripted::scripted_interpolator_base::ScriptedInterpolatorBase::TYPE_KEY => {
                    if let Some(scripted) = object.as_scripted_object_mut() {
                        stack_object = Some(Box::new(ScriptedObjectImporter::new(scripted)));
                        stack_type = crate::mechanical_port::source::generated::scripted::scripted_drawable_base::ScriptedDrawableBase::TYPE_KEY;
                    }
                }
                crate::mechanical_port::source::generated::data_bind::data_bind_path_base::DataBindPathBase::TYPE_KEY => {
                    stack_object = Some(Box::new(DataBindPathImporter::new(object.as_data_bind_path_mut().unwrap())));
                }
                crate::mechanical_port::source::generated::script_input_artboard_base::ScriptInputArtboardBase::TYPE_KEY => {
                    object.as_script_input_artboard_mut().unwrap().set_file(Some(NonNull::from(&mut *self)));
                }
                _ => {}
            }

            if import_stack.make_latest(stack_type, stack_object) != StatusCode::Ok {
                return ImportResult::Malformed;
            }
            if let Some(component) = object.as_state_machine_layer_component_mut() {
                if import_stack.make_latest(
                    crate::mechanical_port::source::generated::animation::state_machine_layer_component_base::StateMachineLayerComponentBase::TYPE_KEY,
                    Some(Box::new(StateMachineLayerComponentImporter::new(component))),
                ) != StatusCode::Ok
                {
                    return ImportResult::Malformed;
                }
            }
            if let Some(converter) = object.as_data_converter_mut() {
                self.data_converters.push(converter);
            } else if let Some(interpolator) = object.as_keyframe_interpolator_mut() {
                let artboard_importer = import_stack.latest::<ArtboardImporter>(
                    crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
                );
                if artboard_importer.is_none() {
                    self.keyframe_interpolators.push(interpolator);
                }
                if let Some(scripted) = object.as_scripted_interpolator_mut() {
                    self.scripted_interpolators.push(scripted);
                }
            } else if let Some(physics) = object.as_scroll_physics_mut() {
                self.scroll_physics.push(physics);
            }
            Box::leak(object);
        }

        let resolved = import_stack.resolve();
        #[cfg(feature = "rive_scripting")]
        self.register_scripts();
        if !reader.has_error() && resolved == StatusCode::Ok {
            ImportResult::Success
        } else {
            ImportResult::Malformed
        }
    }

    pub fn add_file_view_model_instance(&mut self, instance: NonNull<ViewModelInstance>) {
        self.view_model_instances
            .push(unsafe { Rcp::from_raw(instance.as_ptr()) });
    }

    #[cfg(feature = "rive_scripting")]
    fn register_scripts(&mut self) {
        let scripts: Vec<_> = self
            .file_assets
            .iter()
            .filter_map(|asset| asset.as_script_asset())
            .collect();
        if scripts.is_empty() {
            return;
        }
        if self.scripting_vm.is_none() {
            self.make_scripting_vm();
        }
        if let Some(vm) = self.scripting_vm.as_deref_mut() {
            if !vm.context().initializes_data_global_externally() {
                crate::mechanical_port::source::lua::rive_lua_libs::initialize_lua_data(
                    vm.state(),
                    &self.view_models,
                );
            }
            for script in scripts {
                #[cfg(feature = "rive_tools")]
                vm.add_module(script);
                #[cfg(not(feature = "rive_tools"))]
                if unsafe { script.as_ref() }.base.verified() {
                    vm.add_module(script);
                }
            }
            vm.perform_registration();
        }
        for interpolator in &mut self.scripted_interpolators {
            if let Some(mut script) = unsafe { interpolator.as_ref() }.script_asset() {
                unsafe { script.as_mut() }.init_scripted_object(interpolator.cast());
            }
        }
    }

    #[cfg(feature = "rive_scripting")]
    fn make_scripting_vm(&mut self) {
        self.cleanup_scripting_vm();
        let context = Box::new(
            crate::mechanical_port::source::lua::rive_lua_libs::CppRuntimeScriptingContext::new(
                self.factory,
            ),
        );
        self.scripting_vm = Some(make_rcp(
            crate::mechanical_port::source::lua::scripting_vm::ScriptingVm::new(context),
        ));
    }

    #[cfg(feature = "rive_scripting")]
    pub fn scripting_state(
        &mut self,
    ) -> Option<NonNull<crate::mechanical_port::source::lua::lua_state::LuaState>> {
        self.scripting_vm.as_deref_mut().map(|vm| vm.state())
    }

    #[cfg(feature = "rive_scripting")]
    pub fn set_scripting_vm(
        &mut self,
        vm: Rcp<crate::mechanical_port::source::lua::scripting_vm::ScriptingVm>,
    ) {
        #[cfg(feature = "rive_tools")]
        if let Some(current) = self.scripting_vm.as_deref_mut() {
            if let Some(context) = current.context_mut() {
                context.dispose_orphan_scripted_properties();
            }
        }
        self.scripting_vm = Some(vm);
    }

    #[cfg(feature = "rive_scripting")]
    fn cleanup_scripting_vm(&mut self) {
        #[cfg(feature = "rive_tools")]
        if let Some(current) = self.scripting_vm.as_deref_mut() {
            if let Some(context) = current.context_mut() {
                context.dispose_orphan_scripted_properties();
            }
        }
        self.scripting_vm = None;
    }

    #[cfg(feature = "rive_scripting")]
    pub fn scripting_vm(
        &mut self,
    ) -> Option<NonNull<crate::mechanical_port::source::lua::scripting_vm::ScriptingVm>> {
        self.scripting_vm.as_deref_mut().map(NonNull::from)
    }

    #[cfg(all(feature = "rive_scripting", feature = "rive_tools"))]
    pub fn clear_scripting_vm(&mut self) {
        self.cleanup_scripting_vm();
    }

    #[cfg(all(feature = "rive_scripting", feature = "rive_tools"))]
    pub fn has_vm(&self) -> bool {
        self.scripting_vm.is_some()
    }

    pub fn artboard_named_source(&self, name: &str) -> Option<NonNull<Artboard>> {
        self.artboards
            .iter()
            .copied()
            .find(|artboard| unsafe { artboard.as_ref().base.name() == name })
    }

    pub fn artboard(&self) -> Option<NonNull<Artboard>> {
        self.artboards.first().copied()
    }

    pub fn artboard_at_source(&self, index: usize) -> Option<NonNull<Artboard>> {
        self.artboards.get(index).copied()
    }

    pub fn artboard_name_at(&self, index: usize) -> String {
        self.artboard_at_source(index)
            .map(|artboard| unsafe { artboard.as_ref() }.base.name().to_owned())
            .unwrap_or_default()
    }

    fn instance_artboard(
        &self,
        artboard: Option<NonNull<Artboard>>,
    ) -> Option<Box<ArtboardInstance>> {
        let artboard = artboard?;
        let mut instance = unsafe { artboard.as_ref() }.instance()?;
        #[cfg(feature = "rive_scripting")]
        instance.set_scripting_vm(self.scripting_vm.clone());
        instance.set_file(Some(unsafe { ref_rcp(self as *const Self as *mut Self) }));
        Some(instance)
    }

    pub fn artboard_default(&self) -> Option<Box<ArtboardInstance>> {
        self.instance_artboard(self.artboard())
    }

    pub fn artboard_at(&self, index: usize) -> Option<Box<ArtboardInstance>> {
        self.instance_artboard(self.artboard_at_source(index))
    }

    pub fn artboard_named(&self, name: &str) -> Option<Box<ArtboardInstance>> {
        self.instance_artboard(self.artboard_named_source(name))
    }

    pub fn bindable_artboard_named(&self, name: &str) -> Option<Rcp<BindableArtboard>> {
        let artboard = self.artboard_named(name)?;
        Some(make_rcp(BindableArtboard::new(
            Some(unsafe { ref_rcp(self as *const Self as *mut Self) }),
            artboard,
        )))
    }

    pub fn bindable_artboard_default(&self) -> Option<Rcp<BindableArtboard>> {
        let artboard = self.artboard_default()?;
        Some(make_rcp(BindableArtboard::new(
            Some(unsafe { ref_rcp(self as *const Self as *mut Self) }),
            artboard,
        )))
    }

    pub fn internal_bindable_artboard_from_artboard(
        &self,
        artboard: Option<NonNull<Artboard>>,
    ) -> Option<Rcp<BindableArtboard>> {
        let artboard = unsafe { artboard?.as_ref() }.instance()?;
        Some(make_rcp(BindableArtboard::new(None, artboard)))
    }

    pub fn complete_view_model_instance(&self, instance: Rcp<ViewModelInstance>) {
        let mut instances = HashMap::new();
        self.complete_view_model_instance_with_map(instance, &mut instances);
    }

    pub fn complete_view_model_instance_with_map(
        &self,
        instance: Rcp<ViewModelInstance>,
        instances: &mut HashMap<NonNull<ViewModelInstance>, Rcp<ViewModelInstance>>,
    ) {
        let view_model = self.view_models[instance.base.view_model_id() as usize];
        for value in instance.property_values() {
            if let Some(mut nested) = value.base.as_view_model_instance_viewmodel() {
                let property =
                    unsafe { view_model.as_ref() }.property(value.base.view_model_property_id());
                if let Some(property) = property.and_then(|property| unsafe {
                    property.as_ref().base.as_view_model_property_viewmodel()
                }) {
                    let referenced_model = self.view_models
                        [unsafe { property.as_ref() }.base.view_model_reference_id() as usize];
                    let source = unsafe { referenced_model.as_ref() }
                        .instance(unsafe { nested.as_ref() }.base.property_value());
                    unsafe { nested.as_mut() }
                        .set_parent_view_model_instance(Some(NonNull::from(&*instance)));
                    if let Some(source) = source {
                        let copied = if let Some(existing) = instances.get(&source) {
                            existing.clone()
                        } else {
                            let copied = self.copy_view_model_instance(source, instances);
                            instances.insert(source, copied.clone());
                            copied
                        };
                        unsafe { nested.as_mut() }.set_reference_view_model_instance(Some(copied));
                    }
                }
            } else if let Some(mut list) = value.base.as_view_model_instance_list() {
                unsafe { list.as_mut() }
                    .set_parent_view_model_instance(Some(NonNull::from(&*instance)));
                for item in unsafe { list.as_mut() }.list_items_mut() {
                    let model = self.view_models[item.base.view_model_id() as usize];
                    let source =
                        unsafe { model.as_ref() }.instance(item.base.view_model_instance_id());
                    if let Some(source) = source {
                        let copied = if let Some(existing) = instances.get(&source) {
                            existing.clone()
                        } else {
                            let copied = self.copy_view_model_instance(source, instances);
                            instances.insert(source, copied.clone());
                            copied
                        };
                        item.set_view_model_instance(Some(copied));
                    }
                }
            }
            value.set_view_model_property(
                unsafe { view_model.as_ref() }.property(value.base.view_model_property_id()),
            );
        }
    }

    pub fn complete_view_model_properties(&self, instance: NonNull<ViewModelInstance>) {
        let view_model =
            self.view_models[unsafe { instance.as_ref() }.base.view_model_id() as usize];
        for value in unsafe { instance.as_ref() }.property_values() {
            if let Some(nested) = value.base.as_view_model_instance_viewmodel() {
                let property =
                    unsafe { view_model.as_ref() }.property(value.base.view_model_property_id());
                if let Some(property) = property.and_then(|property| unsafe {
                    property.as_ref().base.as_view_model_property_viewmodel()
                }) {
                    let referenced_model = self.view_models
                        [unsafe { property.as_ref() }.base.view_model_reference_id() as usize];
                    if let Some(source) = unsafe { referenced_model.as_ref() }
                        .instance(unsafe { nested.as_ref() }.base.property_value())
                    {
                        self.complete_view_model_properties(source);
                    }
                }
            } else if let Some(mut list) = value.base.as_view_model_instance_list() {
                for item in unsafe { list.as_mut() }.list_items_mut() {
                    let model = self.view_models[item.base.view_model_id() as usize];
                    if let Some(source) =
                        unsafe { model.as_ref() }.instance(item.base.view_model_instance_id())
                    {
                        self.complete_view_model_properties(source);
                    }
                }
            }
            value.set_view_model_property(
                unsafe { view_model.as_ref() }.property(value.base.view_model_property_id()),
            );
        }
    }

    fn copy_view_model_instance(
        &self,
        instance: NonNull<ViewModelInstance>,
        instances: &mut HashMap<NonNull<ViewModelInstance>, Rcp<ViewModelInstance>>,
    ) -> Rcp<ViewModelInstance> {
        let cloned = unsafe { instance.as_ref() }.clone_core();
        let copied = unsafe { Rcp::from_raw(Box::into_raw(cloned)) };
        self.complete_view_model_instance_with_map(copied.clone(), instances);
        #[cfg(feature = "rive_tools")]
        self.register_view_model_instance(NonNull::from(&*copied), copied.clone());
        copied
    }

    pub fn create_view_model_instance_named(&self, name: &str) -> Option<Rcp<ViewModelInstance>> {
        self.view_models.iter().find_map(|model| {
            (unsafe { model.as_ref() }.base.name() == name)
                .then(|| self.create_view_model_instance_for_model(*model))
                .flatten()
        })
    }

    pub fn create_view_model_instance_by_instance_name(
        &self,
        model_name: &str,
        instance_name: &str,
    ) -> Option<Rcp<ViewModelInstance>> {
        for model in &self.view_models {
            if unsafe { model.as_ref() }.base.name() == model_name {
                if let Some(instance) = unsafe { model.as_ref() }.instance_named(instance_name) {
                    return Some(self.copy_view_model_instance(instance, &mut HashMap::new()));
                }
            }
        }
        None
    }

    pub fn create_view_model_instance_at(
        &self,
        model_index: usize,
        instance_index: usize,
    ) -> Option<Rcp<ViewModelInstance>> {
        let model = *self.view_models.get(model_index)?;
        let instance = unsafe { model.as_ref() }.instance(instance_index as u32)?;
        Some(self.copy_view_model_instance(instance, &mut HashMap::new()))
    }

    fn find_view_model_id(&self, search: NonNull<ViewModel>) -> u32 {
        self.view_models
            .iter()
            .position(|model| *model == search)
            .unwrap_or(self.view_models.len()) as u32
    }

    #[cfg(feature = "rive_tools")]
    pub fn set_view_model_instance_registrar(
        &mut self,
        registrar: Option<NonNull<dyn ViewModelInstanceRegistrar>>,
    ) {
        self.view_model_instance_registrar = registrar;
    }

    #[cfg(feature = "rive_tools")]
    pub fn register_view_model_instance(
        &self,
        pointer: NonNull<ViewModelInstance>,
        reference: Rcp<ViewModelInstance>,
    ) {
        if let Some(mut registrar) = self.view_model_instance_registrar {
            unsafe { registrar.as_mut() }.register_instance(pointer, reference);
        }
    }

    #[cfg(feature = "rive_tools")]
    pub fn contains_view_model_instance(&self, pointer: NonNull<ViewModelInstance>) -> bool {
        self.view_model_instance_registrar
            .is_some_and(|registrar| unsafe { registrar.as_ref() }.contains(pointer))
    }

    #[cfg(feature = "rive_tools")]
    pub fn clear_runtime_view_model_instances(&mut self) {
        if let Some(mut registrar) = self.view_model_instance_registrar {
            unsafe { registrar.as_mut() }.clear();
        }
    }

    pub fn create_view_model_instance_for_model(
        &self,
        view_model: NonNull<ViewModel>,
    ) -> Option<Rcp<ViewModelInstance>> {
        let mut instance = Box::<ViewModelInstance>::default();
        instance
            .base
            .set_view_model_id(self.find_view_model_id(view_model));
        instance.set_view_model(view_model);
        for (property_id, property) in unsafe { view_model.as_ref() }
            .properties()
            .iter()
            .copied()
            .enumerate()
        {
            let property_type = unsafe { property.as_ref() }.core_type();
            let value: Option<NonNull<crate::mechanical_port::source::viewmodel::viewmodel_instance_value::ViewModelInstanceValue>> =
                match property_type {
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_string_base::ViewModelPropertyStringBase::TYPE_KEY => Some(NonNull::from(Box::leak(Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_string::ViewModelInstanceString::default()))).cast()),
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_number_base::ViewModelPropertyNumberBase::TYPE_KEY => Some(NonNull::from(Box::leak(Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_number::ViewModelInstanceNumber::default()))).cast()),
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_boolean_base::ViewModelPropertyBooleanBase::TYPE_KEY => Some(NonNull::from(Box::leak(Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_boolean::ViewModelInstanceBoolean::default()))).cast()),
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_color_base::ViewModelPropertyColorBase::TYPE_KEY => Some(NonNull::from(Box::leak(Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_color::ViewModelInstanceColor::default()))).cast()),
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_list_base::ViewModelPropertyListBase::TYPE_KEY => {
                        let mut list = Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_list::ViewModelInstanceList::default());
                        list.set_parent_view_model_instance(Some(NonNull::from(instance.as_mut())));
                        Some(NonNull::from(Box::leak(list)).cast())
                    }
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_enum_system_base::ViewModelPropertyEnumSystemBase::TYPE_KEY
                    | crate::mechanical_port::source::generated::viewmodel::viewmodel_property_enum_custom_base::ViewModelPropertyEnumCustomBase::TYPE_KEY
                    | crate::mechanical_port::source::generated::viewmodel::viewmodel_property_enum_base::ViewModelPropertyEnumBase::TYPE_KEY => Some(NonNull::from(Box::leak(Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_enum::ViewModelInstanceEnum::default()))).cast()),
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_trigger_base::ViewModelPropertyTriggerBase::TYPE_KEY => Some(NonNull::from(Box::leak(Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_trigger::ViewModelInstanceTrigger::default()))).cast()),
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_viewmodel_base::ViewModelPropertyViewModelBase::TYPE_KEY => {
                        let mut nested = Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_viewmodel::ViewModelInstanceViewModel::default());
                        let nested_property = unsafe { property.as_ref() }.base.as_view_model_property_viewmodel().unwrap();
                        let referenced_model = self.view_models[unsafe { nested_property.as_ref() }.base.view_model_reference_id() as usize];
                        if let Some(referenced_instance) = self.create_view_model_instance_for_model(referenced_model) {
                            nested.set_parent_view_model_instance(Some(NonNull::from(instance.as_mut())));
                            nested.set_reference_view_model_instance(Some(referenced_instance));
                        }
                        Some(NonNull::from(Box::leak(nested)).cast())
                    }
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_asset_image_base::ViewModelPropertyAssetImageBase::TYPE_KEY => Some(NonNull::from(Box::leak(Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_asset_image::ViewModelInstanceAssetImage::default()))).cast()),
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_asset_font_base::ViewModelPropertyAssetFontBase::TYPE_KEY => Some(NonNull::from(Box::leak(Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_asset_font::ViewModelInstanceAssetFont::default()))).cast()),
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_asset_blob_base::ViewModelPropertyAssetBlobBase::TYPE_KEY => Some(NonNull::from(Box::leak(Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_asset_blob::ViewModelInstanceAssetBlob::default()))).cast()),
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_symbol_list_index_base::ViewModelPropertySymbolListIndexBase::TYPE_KEY => Some(NonNull::from(Box::leak(Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_symbol_list_index::ViewModelInstanceSymbolListIndex::default()))).cast()),
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_property_artboard_base::ViewModelPropertyArtboardBase::TYPE_KEY => Some(NonNull::from(Box::leak(Box::new(crate::mechanical_port::source::viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard::default()))).cast()),
                    _ => {
                        eprintln!("Missing view model property type");
                        None
                    }
                };
            if let Some(mut value) = value {
                unsafe { value.as_mut() }.set_view_model_property(Some(property));
                unsafe { value.as_mut() }
                    .base
                    .set_view_model_property_id(property_id as u32);
            }
            instance.add_value(value);
        }
        let instance = unsafe { Rcp::from_raw(Box::into_raw(instance)) };
        #[cfg(feature = "rive_tools")]
        self.register_view_model_instance(NonNull::from(&*instance), instance.clone());
        Some(instance)
    }

    pub fn create_view_model_instance_for_artboard(
        &self,
        artboard: NonNull<Artboard>,
    ) -> Option<Rcp<ViewModelInstance>> {
        let id = unsafe { artboard.as_ref() }.base.view_model_id() as usize;
        let model = *self.view_models.get(id)?;
        self.create_view_model_instance_for_model(model)
    }

    pub fn create_default_view_model_instance_for_artboard(
        &self,
        artboard: NonNull<Artboard>,
    ) -> Option<Rcp<ViewModelInstance>> {
        let id = unsafe { artboard.as_ref() }.base.view_model_id() as usize;
        self.create_default_view_model_instance(*self.view_models.get(id)?)
    }

    pub fn create_default_view_model_instance(
        &self,
        view_model: NonNull<ViewModel>,
    ) -> Option<Rcp<ViewModelInstance>> {
        if let Some(instance) = unsafe { view_model.as_ref() }.instance(0) {
            let copy =
                unsafe { Rcp::from_raw(Box::into_raw(unsafe { instance.as_ref() }.clone_core())) };
            self.complete_view_model_instance(copy.clone());
            #[cfg(feature = "rive_tools")]
            self.register_view_model_instance(NonNull::from(&*copy), copy.clone());
            return Some(copy);
        }
        self.create_view_model_instance_for_model(view_model)
    }

    pub fn view_model_instance_list_item(
        &mut self,
        instance: Rcp<ViewModelInstance>,
    ) -> Option<Box<ViewModelInstanceListItem>> {
        for artboard in &self.artboards {
            if unsafe { artboard.as_ref() }.base.view_model_id() == instance.base.view_model_id() {
                return Some(self.view_model_instance_list_item_for_artboard(instance, *artboard));
            }
        }
        None
    }

    pub fn view_model_instance_list_item_for_artboard(
        &self,
        instance: Rcp<ViewModelInstance>,
        artboard: NonNull<Artboard>,
    ) -> Box<ViewModelInstanceListItem> {
        let mut item = Box::<ViewModelInstanceListItem>::default();
        item.set_view_model_instance(Some(instance));
        item.set_artboard(Some(artboard));
        item
    }

    pub fn view_model_named(&self, name: &str) -> Option<NonNull<ViewModel>> {
        self.view_models
            .iter()
            .copied()
            .find(|model| unsafe { model.as_ref().base.name() == name })
    }

    pub fn view_model(&self, index: usize) -> Option<NonNull<ViewModel>> {
        self.view_models.get(index).copied()
    }

    pub fn view_model_id(&self, name: &str) -> u32 {
        self.view_models
            .iter()
            .position(|model| unsafe { model.as_ref().base.name() == name })
            .unwrap_or(self.view_models.len()) as u32
    }

    pub fn global_view_models(&self) -> Vec<NonNull<ViewModel>> {
        self.view_models
            .iter()
            .copied()
            .filter(|model| {
                ViewModelType::from_u32(unsafe { model.as_ref() }.base.view_model_type())
                    == Some(ViewModelType::Global)
            })
            .collect()
    }

    pub fn global_view_model_names(&self) -> Vec<String> {
        self.global_view_models()
            .iter()
            .map(|model| unsafe { model.as_ref() }.base.name().to_owned())
            .collect()
    }

    pub fn view_model_by_index(&self, index: usize) -> Option<NonNull<ViewModelRuntime>> {
        if let Some(model) = self.view_models.get(index) {
            return Some(NonNull::from(&*self.create_view_model_runtime(*model)));
        }
        eprintln!(
            "Could not find View Model. Index {} is out of range.",
            index
        );
        None
    }

    pub fn view_model_by_name(&self, name: &str) -> Option<NonNull<ViewModelRuntime>> {
        for model in &self.view_models {
            if unsafe { model.as_ref() }.base.name() == name {
                return Some(NonNull::from(&*self.create_view_model_runtime(*model)));
            }
        }
        eprintln!("Could not find View Model named {}.", name);
        None
    }

    pub fn default_artboard_view_model(
        &self,
        artboard: Option<NonNull<Artboard>>,
    ) -> Option<NonNull<ViewModelRuntime>> {
        let artboard = artboard?;
        let id = unsafe { artboard.as_ref() }.base.view_model_id() as usize;
        if let Some(model) = self.view_models.get(id) {
            return Some(NonNull::from(&*self.create_view_model_runtime(*model)));
        }
        eprintln!(
            "Could not find a View Model linked to Artboard {}.",
            unsafe { artboard.as_ref() }.base.name()
        );
        None
    }

    fn create_view_model_runtime(&self, model: NonNull<ViewModel>) -> Rcp<ViewModelRuntime> {
        let runtime = make_rcp(ViewModelRuntime::new(model, NonNull::from(self)));
        self.view_model_runtimes.borrow_mut().push(runtime.clone());
        runtime
    }

    pub fn assets(&self) -> &[Rcp<FileAsset>] {
        &self.file_assets
    }

    pub fn enums(&self) -> &[NonNull<DataEnum>] {
        &self.enums
    }

    #[cfg(feature = "rive_tools")]
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
            if object.is_file_asset() {
                last_asset_type = object.core_type();
            }
            if object.is_file_asset_contents() && type_keys.contains(&last_asset_type) {
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

    pub fn asset(&self, index: usize) -> Option<Rcp<FileAsset>> {
        self.file_assets.get(index).cloned()
    }

    pub fn backboard(&self) -> Option<NonNull<Backboard>> {
        self.backboard
    }

    pub fn artboard_count(&self) -> usize {
        self.artboards.len()
    }

    pub fn view_model_count(&self) -> usize {
        self.view_models.len()
    }

    pub fn artboards(&self) -> Vec<NonNull<Artboard>> {
        self.artboards.clone()
    }

    pub fn has_audio(&self) -> bool {
        self.has_audio
    }

    pub fn data_resolver(&mut self) -> Option<NonNull<dyn DataResolver>> {
        self.manifest
            .as_deref_mut()
            .and_then(|asset| asset.as_manifest_asset_mut())
            .map(|manifest: &mut ManifestAsset| {
                NonNull::from(manifest) as NonNull<dyn DataResolver>
            })
    }

    #[cfg(test)]
    pub fn testing_get_asset_loader(&mut self) -> Option<NonNull<FileAssetLoader>> {
        NonNull::new(self.asset_loader.get())
    }
}
