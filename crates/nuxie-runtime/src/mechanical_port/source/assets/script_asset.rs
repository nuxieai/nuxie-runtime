use std::collections::HashSet;

use crate::mechanical_port::source::data_bind::data_bind::BindScriptInput;
use crate::mechanical_port::source::{
    core::{CoreHandle, CoreTypeKey},
    factory::RuntimeFactoryHandle,
    file::RuntimeFileWeakHandle,
    generated::assets::script_asset_base::ScriptAssetBase,
    scripted::scripted_object::ScriptedObject,
    signed_content_header::SignedContentHeader,
};

use crate::mechanical_port::source::{
    importers::text_asset_importer::SCRIPT_VERIFICATION_PUBLIC_KEY,
    lua::scripting_vm::RuntimeScriptingVmHandle,
};

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScriptProtocol {
    Utility,
    Node,
    Layout,
    Converter,
    PathEffect,
    ListenerAction,
    TransitionCondition,
    Interpolator,
}

pub struct ScriptInput {
    scripted_object: Option<CoreHandle>,
    data_bind: Option<CoreHandle>,
    owns_data_bind: bool,
}

impl Default for ScriptInput {
    fn default() -> Self {
        Self {
            scripted_object: None,
            data_bind: None,
            owns_data_bind: false,
        }
    }
}

impl Drop for ScriptInput {
    fn drop(&mut self) {
        if self.owns_data_bind {
            if let Some(bind) = self.data_bind.take() {
                crate::mechanical_port::source::data_bind::data_bind::DataBind::unbind_handle(
                    &bind,
                );
                bind.remove_occurrence();
            }
        }
    }
}

impl ScriptInput {
    pub fn from(component: CoreHandle, type_key: CoreTypeKey) -> Option<CoreHandle> {
        match type_key {
            621 | 631 | 626 | 611 | 627 | 618 | 612 => Some(component),
            _ => None,
        }
    }

    pub fn data_bind(&self) -> Option<CoreHandle> {
        self.data_bind.clone()
    }

    pub fn set_data_bind(&mut self, data_bind: Option<CoreHandle>, owns_data_bind: bool) {
        self.data_bind = data_bind;
        self.owns_data_bind = owns_data_bind;
    }

    pub fn scripted_object(&self) -> Option<CoreHandle> {
        self.scripted_object.clone()
    }

    pub fn set_scripted_object(&mut self, object: Option<CoreHandle>) {
        self.scripted_object = object;
    }

    /// The input can already be borrowed during import/onAddedClean. The
    /// concrete scripted owner passes its real container here, preserving the
    /// source backlink-before-unique-insertion order without reborrowing input.
    pub(crate) fn attach_to_container(
        &mut self,
        owner: Option<CoreHandle>,
        property: CoreHandle,
        properties: &mut Vec<CoreHandle>,
    ) {
        self.set_scripted_object(owner);
        if !properties.contains(&property) {
            properties.push(property);
        }
    }
}

pub trait ScriptInputBehavior {
    fn script_input(&self) -> &ScriptInput;
    fn script_input_mut(&mut self) -> &mut ScriptInput;
    fn validate_for_script_init(&self) -> bool;

    fn init_scripted_value(&mut self) {}

    fn validate_for_cold_script_init(&self) -> bool {
        self.validate_for_script_init()
    }

    fn hydrate_script_input(&mut self) -> bool {
        self.init_scripted_value();
        true
    }

    fn validate_hydration_prerequisites(&self) -> bool {
        true
    }
}

impl<T: ScriptInputBehavior> BindScriptInput for T {
    fn scripted_object(&self) -> Option<CoreHandle> {
        self.script_input().scripted_object()
    }

    fn set_scripted_object(&mut self, object: Option<CoreHandle>) {
        self.script_input_mut().set_scripted_object(object);
    }

    fn data_bind(&self) -> Option<CoreHandle> {
        self.script_input().data_bind()
    }

    fn set_data_bind(&mut self, bind: Option<CoreHandle>, owns_data_bind: bool) {
        self.script_input_mut().set_data_bind(bind, owns_data_bind);
    }
}

#[derive(Default)]
pub struct OptionalScriptedMethods {
    implemented_methods: i32,
}

impl OptionalScriptedMethods {
    const ADVANCES_BIT: i32 = 1 << 0;
    const UPDATES_BIT: i32 = 1 << 1;
    const MEASURES_BIT: i32 = 1 << 2;
    const WANTS_POINTER_DOWN_BIT: i32 = 1 << 3;
    const WANTS_POINTER_MOVE_BIT: i32 = 1 << 4;
    const WANTS_POINTER_UP_BIT: i32 = 1 << 5;
    const WANTS_POINTER_EXIT_BIT: i32 = 1 << 6;
    const WANTS_POINTER_CANCEL_BIT: i32 = 1 << 7;
    const DRAWS_BIT: i32 = 1 << 8;
    const INITS_BIT: i32 = 1 << 9;
    const DATA_CONVERTS_BIT: i32 = 1 << 10;
    const DATA_REVERSE_CONVERTS_BIT: i32 = 1 << 11;
    const RESIZES_BIT: i32 = 1 << 12;
    const LISTENER_PERFORMS_BIT: i32 = 1 << 13;
    const LISTENER_PERFORMS_ACTION_BIT: i32 = 1 << 14;
    const DRAWS_CANVAS_BIT: i32 = 1 << 15;
    const WANTS_KEYBOARD_INPUT_BIT: i32 = 1 << 16;
    const WANTS_TEXT_INPUT_BIT: i32 = 1 << 17;
    const WANTS_GAMEPAD_CONNECT_BIT: i32 = 1 << 18;
    const WANTS_GAMEPAD_DISCONNECT_BIT: i32 = 1 << 19;
    const WANTS_GAMEPAD_EVENT_BIT: i32 = 1 << 20;

    pub const METHOD_MASK: u32 = (1 << 21) - 1;

    pub fn implemented_methods(&self) -> i32 {
        self.implemented_methods
    }

    pub fn set_implemented_methods(&mut self, implemented: i32) {
        self.implemented_methods = implemented;
    }

    fn has(&self, bit: i32) -> bool {
        self.implemented_methods & bit != 0
    }

    pub fn listens_to_pointer_events(&self) -> bool {
        self.has(
            Self::WANTS_POINTER_DOWN_BIT
                | Self::WANTS_POINTER_MOVE_BIT
                | Self::WANTS_POINTER_UP_BIT
                | Self::WANTS_POINTER_EXIT_BIT
                | Self::WANTS_POINTER_CANCEL_BIT
                | Self::WANTS_GAMEPAD_CONNECT_BIT
                | Self::WANTS_GAMEPAD_DISCONNECT_BIT
                | Self::WANTS_GAMEPAD_EVENT_BIT,
        )
    }

    pub fn advances(&self) -> bool {
        self.has(Self::ADVANCES_BIT)
    }
    pub fn updates(&self) -> bool {
        self.has(Self::UPDATES_BIT)
    }
    pub fn measures(&self) -> bool {
        self.has(Self::MEASURES_BIT)
    }
    pub fn resizes(&self) -> bool {
        self.has(Self::RESIZES_BIT)
    }
    pub fn performs(&self) -> bool {
        self.has(Self::LISTENER_PERFORMS_BIT)
    }
    pub fn performs_action(&self) -> bool {
        self.has(Self::LISTENER_PERFORMS_ACTION_BIT)
    }
    pub fn wants_pointer_down(&self) -> bool {
        self.has(Self::WANTS_POINTER_DOWN_BIT)
    }
    pub fn wants_pointer_move(&self) -> bool {
        self.has(Self::WANTS_POINTER_MOVE_BIT)
    }
    pub fn wants_pointer_up(&self) -> bool {
        self.has(Self::WANTS_POINTER_UP_BIT)
    }
    pub fn wants_pointer_exit(&self) -> bool {
        self.has(Self::WANTS_POINTER_EXIT_BIT)
    }
    pub fn wants_pointer_cancel(&self) -> bool {
        self.has(Self::WANTS_POINTER_CANCEL_BIT)
    }
    pub fn wants_gamepad_connect(&self) -> bool {
        self.has(Self::WANTS_GAMEPAD_CONNECT_BIT)
    }
    pub fn wants_gamepad_disconnect(&self) -> bool {
        self.has(Self::WANTS_GAMEPAD_DISCONNECT_BIT)
    }
    pub fn wants_gamepad_event(&self) -> bool {
        self.has(Self::WANTS_GAMEPAD_EVENT_BIT)
    }
    pub fn draws(&self) -> bool {
        self.has(Self::DRAWS_BIT)
    }
    pub fn inits(&self) -> bool {
        self.has(Self::INITS_BIT)
    }
    pub fn data_converts(&self) -> bool {
        self.has(Self::DATA_CONVERTS_BIT)
    }
    pub fn data_reverse_converts(&self) -> bool {
        self.has(Self::DATA_REVERSE_CONVERTS_BIT)
    }
    pub fn draws_canvas(&self) -> bool {
        self.has(Self::DRAWS_CANVAS_BIT)
    }
    pub fn wants_keyboard_input(&self) -> bool {
        self.has(Self::WANTS_KEYBOARD_INPUT_BIT)
    }
    pub fn wants_text_input(&self) -> bool {
        self.has(Self::WANTS_TEXT_INPUT_BIT)
    }
}

#[derive(Default)]
pub struct ModuleDetails {
    dependencies: HashSet<String>,
}

impl ModuleDetails {
    pub fn registration_complete(&mut self, _reference: i32) {}

    pub fn module_bytecode(&mut self) -> &mut [u8] {
        &mut []
    }

    pub fn verified(&self) -> bool {
        false
    }

    pub fn add_missing_dependency(&mut self, name: String) {
        self.dependencies.insert(name);
    }

    pub fn clear_missing_dependency(&mut self, name: &str) {
        self.dependencies.remove(name);
    }

    pub fn missing_dependencies(&self) -> HashSet<String> {
        self.dependencies.clone()
    }
}

/// Object-safe form of the pinned `ModuleDetails` virtual interface.
///
/// `ModuleDetails` owns only the dependency bookkeeping inherited by a module
/// owner. Calls that are virtual in C++ must be made through this behavior so
/// they reach `ScriptAsset`, rather than the embedded state object's defaults.
pub trait ModuleDetailsBehavior {
    fn module_name(&self) -> String;
    fn registration_complete(&mut self, reference: i32);
    fn module_bytecode(&self) -> &[u8];
    fn is_protocol_script(&self) -> bool;
    fn verified(&self) -> bool;
    fn add_missing_dependency(&mut self, name: String);
    fn clear_missing_dependency(&mut self, name: &str);
    fn missing_dependencies(&self) -> HashSet<String>;
}

/// Stable arena identity for an object that implements `ModuleDetails`.
///
/// The handle never exposes a borrow outside its callback, so Lua module
/// registration cannot retain a pointer into movable arena storage.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RuntimeModuleDetailsHandle {
    owner: CoreHandle,
}

impl RuntimeModuleDetailsHandle {
    pub fn new(owner: CoreHandle) -> Option<Self> {
        owner.with_downcast::<ScriptAsset, _>(|_| Self {
            owner: owner.clone(),
        })
    }

    pub fn owner(&self) -> CoreHandle {
        self.owner.clone()
    }

    pub fn with_module<R>(
        &self,
        callback: impl FnOnce(&dyn ModuleDetailsBehavior) -> R,
    ) -> Option<R> {
        self.owner
            .with_downcast::<ScriptAsset, _>(|module| callback(module))
    }

    pub fn with_module_mut<R>(
        &self,
        callback: impl FnOnce(&mut dyn ModuleDetailsBehavior) -> R,
    ) -> Option<R> {
        self.owner
            .with_downcast_mut::<ScriptAsset, _>(|module| callback(module))
    }
}

pub struct ScriptAsset {
    pub base: ScriptAssetBase,
    optional_methods: OptionalScriptedMethods,
    module_details: ModuleDetails,
    file: Option<RuntimeFileWeakHandle>,
    scripting_vm: Option<RuntimeScriptingVmHandle>,
    generator: Option<crate::scripting::RuntimeScriptProgram>,
    script_registered: bool,
    bytecode: Vec<u8>,
    initted: bool,
}

impl Default for ScriptAsset {
    fn default() -> Self {
        Self {
            base: ScriptAssetBase::default(),
            optional_methods: OptionalScriptedMethods::default(),
            module_details: ModuleDetails::default(),
            file: None,
            scripting_vm: None,
            generator: None,
            script_registered: false,
            bytecode: Vec::new(),
            initted: false,
        }
    }
}

impl ScriptAsset {
    pub fn generator_function_ref(&self) -> u32 {
        self.base.generator_function_ref()
    }

    pub fn set_generator_function_ref(&mut self, value: u32) {
        if self.base.set_generator_function_ref_value(value) {
            self.base
                .base
                .base
                .base
                .base
                .base
                .base
                .base
                .notify_property_changed(ScriptAssetBase::GENERATOR_FUNCTION_REF_PROPERTY_KEY);
        }
    }

    pub fn is_module(&self) -> bool {
        self.base.is_module()
    }

    pub fn set_is_module(&mut self, value: bool) {
        if self.base.set_is_module_value(value) {
            self.base
                .base
                .base
                .base
                .base
                .base
                .base
                .base
                .notify_property_changed(ScriptAssetBase::IS_MODULE_PROPERTY_KEY);
        }
    }

    pub fn serialized_implemented_methods(&self) -> u32 {
        self.base.serialized_implemented_methods()
    }

    pub fn set_serialized_implemented_methods(&mut self, value: u32) {
        if self.base.set_serialized_implemented_methods_value(value) {
            self.base
                .base
                .base
                .base
                .base
                .base
                .base
                .base
                .notify_property_changed(
                    ScriptAssetBase::SERIALIZED_IMPLEMENTED_METHODS_PROPERTY_KEY,
                );
        }
    }

    pub fn verified(&self) -> bool {
        self.base.base.verified()
    }

    pub fn module_bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    pub fn init_scripted_object(
        object: &CoreHandle,
        host: &mut dyn crate::scripting::ScriptHost,
    ) -> bool {
        let properties = ScriptedObject::custom_properties(object);
        ScriptedObject::initialize_occurrence(object, &properties, host)
    }

    pub fn decode(&mut self, data: &mut Vec<u8>, _factory: &RuntimeFactoryHandle) -> bool {
        {
            self.base.base.set_verified(false);
            let header = SignedContentHeader::new(data.as_slice());
            if !header.is_valid() {
                return false;
            }
            self.bytecode = header.content().to_vec();
        }
        true
    }

    /// Decode code admitted by the host's enclosing-artifact authorization.
    /// This is a host trust boundary, not verification of a Rive signature.
    /// Ordinary source imports continue to use `decode` and signature checks.
    ///
    /// # Safety
    /// The caller must have authorized execution of these exact original
    /// in-band bytes. A capability for another artifact, or bytes substituted
    /// by an external asset loader, does not satisfy this requirement.
    pub unsafe fn decode_with_host_authorization(
        &mut self,
        data: &mut Vec<u8>,
        factory: &RuntimeFactoryHandle,
    ) -> bool {
        let decoded = self.decode(data, factory);
        self.base.base.set_verified(decoded);
        decoded
    }

    pub fn bytecode(&mut self, data: &mut [u8]) -> bool {
        {
            let header = SignedContentHeader::new(data);
            if !header.is_valid() {
                self.base.base.set_verified(false);
                return false;
            }
            let bytecode = header.content();
            if !header.is_signed() {
                self.base.base.set_verified(false);
                self.bytecode = bytecode.to_vec();
                return true;
            }
            let Ok(signature): Result<[u8; libhydrogen::sign::BYTES], _> =
                header.signature().try_into()
            else {
                self.base.base.set_verified(false);
                return false;
            };
            let signature = libhydrogen::sign::Signature::from(signature);
            let public_key = libhydrogen::sign::PublicKey::from(SCRIPT_VERIFICATION_PUBLIC_KEY);
            let context = libhydrogen::sign::Context::from("RiveCode");
            if libhydrogen::sign::verify(&signature, bytecode, &context, &public_key).is_err() {
                self.base.base.set_verified(false);
                return false;
            }
            self.base.base.set_verified(true);
            self.bytecode = bytecode.to_vec();
        }
        true
    }

    pub fn file_extension(&self) -> &'static str {
        "lua"
    }

    pub fn set_file(&mut self, value: Option<RuntimeFileWeakHandle>) {
        self.file = value;
    }

    pub fn file(&self) -> Option<RuntimeFileWeakHandle> {
        self.file.clone()
    }

    pub fn set_scripting_vm(&mut self, vm: Option<RuntimeScriptingVmHandle>) {
        if self.scripting_vm.as_ref() != vm.as_ref() {
            self.generator = None;
        }
        self.scripting_vm = vm;
    }

    pub fn scripting_vm(&self) -> Option<RuntimeScriptingVmHandle> {
        self.scripting_vm.clone()
    }

    pub fn registration_complete(&mut self, reference: i32) {
        if self.base.is_module() {
            self.script_registered = true;
        } else {
            self.set_generator_function_ref(reference as u32);
            self.initted = false;
        }
    }

    /// The approved VM stores a real generator token in place of a Lua registry
    /// integer. Serialized generatorFunctionRef remains the editor's asset key.
    pub fn registration_complete_native(
        &mut self,
        program: Option<crate::scripting::RuntimeScriptProgram>,
    ) {
        if self.base.is_module() {
            self.script_registered = true;
        } else {
            self.generator = program;
            self.initted = false;
        }
    }

    pub fn has_generator(&self) -> bool {
        self.generator.is_some()
    }

    pub fn module_name(&self) -> String {
        let folder_path = self.base.folder_path();
        if folder_path.is_empty() {
            self.base.name().to_owned()
        } else {
            format!("{folder_path}/{}", self.base.name())
        }
    }

    pub fn is_protocol_script(&self) -> bool {
        !self.base.is_module()
    }

    pub fn optional_methods(&self) -> &OptionalScriptedMethods {
        &self.optional_methods
    }

    pub fn prepare_implemented_methods(&mut self) -> u32 {
        if !self.initted {
            self.optional_methods.set_implemented_methods(
                (self.base.serialized_implemented_methods() & OptionalScriptedMethods::METHOD_MASK)
                    as i32,
            );
            self.initted = true;
        }
        self.optional_methods.implemented_methods() as u32
    }

    /// Snapshot the asset before invoking the generator. User code may resolve
    /// the asset and its owning scripted occurrence through the real host.
    pub fn instantiate_for_occurrence(
        asset: &CoreHandle,
        owner: &CoreHandle,
        host: &mut dyn crate::scripting::ScriptHost,
    ) -> Option<(Box<dyn crate::scripting::ScriptInstance>, u32)> {
        let (vm, generator, file, methods) = asset.with_downcast_mut::<Self, _>(|asset| {
            let vm = asset.scripting_vm()?;
            let generator = asset.generator.clone()?;
            if !asset.initted {
                asset.optional_methods.set_implemented_methods(
                    (asset.base.serialized_implemented_methods()
                        & OptionalScriptedMethods::METHOD_MASK) as i32,
                );
                asset.initted = true;
            }
            Some((
                vm,
                generator,
                asset.file(),
                asset.optional_methods.implemented_methods() as u32,
            ))
        })??;
        let context = owner
            .with(|owner| {
                owner
                    .as_scripted_object()
                    .and_then(|owner| owner.data_context())
            })
            .flatten();
        let context_present = context.is_some();
        let mut chain = Vec::new();
        let mut current = context;
        while let Some(context) = current {
            let (instance, parent) = context
                .with_context(|context| (context.main_view_model_instance(), context.parent()));
            let model = if let Some(instance) = instance {
                Some(crate::scripting::ScriptViewModel::from_native(
                    instance,
                    file.as_ref()?.upgrade()?,
                )?)
            } else {
                None
            };
            chain.push(model);
            current = parent;
        }
        let view_model = if chain.is_empty() {
            None
        } else {
            chain.remove(0)
        };
        let instance = vm
            .with_vm_mut(|vm| {
                vm.instantiate_program(&generator, context_present, view_model, chain, host)
            })
            .ok()?;
        Some((instance, methods))
    }

    pub fn module_details(&self) -> &ModuleDetails {
        &self.module_details
    }

    pub fn module_details_mut(&mut self) -> &mut ModuleDetails {
        &mut self.module_details
    }

    pub fn module_details_handle(&self) -> Option<RuntimeModuleDetailsHandle> {
        RuntimeModuleDetailsHandle::new(self.base.base.base.base.base.base.base.base.handle()?)
    }
}

impl ModuleDetailsBehavior for ScriptAsset {
    fn module_name(&self) -> String {
        ScriptAsset::module_name(self)
    }

    fn registration_complete(&mut self, reference: i32) {
        ScriptAsset::registration_complete(self, reference);
    }

    fn module_bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    fn is_protocol_script(&self) -> bool {
        ScriptAsset::is_protocol_script(self)
    }

    fn verified(&self) -> bool {
        ScriptAsset::verified(self)
    }

    fn add_missing_dependency(&mut self, name: String) {
        self.module_details.add_missing_dependency(name);
    }

    fn clear_missing_dependency(&mut self, name: &str) {
        self.module_details.clear_missing_dependency(name);
    }

    fn missing_dependencies(&self) -> HashSet<String> {
        self.module_details.missing_dependencies()
    }
}
