use std::{collections::HashSet, ptr::NonNull};

use crate::mechanical_port::source::{
    core::{CoreHandle, CoreTypeKey},
    data_bind::data_bind::DataBind,
    factory::Factory,
    file::File,
    generated::assets::script_asset_base::ScriptAssetBase,
    scripted::scripted_object::ScriptedObject,
    signed_content_header::SignedContentHeader,
};

#[cfg(feature = "rive_scripting")]
use crate::mechanical_port::source::scripting::{ScriptingVm, verify_hydrogen_signature};

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
    scripted_object: Option<NonNull<ScriptedObject>>,
    data_bind: Option<NonNull<DataBind>>,
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
        if self.owns_data_bind
            && let Some(data_bind) = self.data_bind.take()
        {
            // This is the direct representation of the source's conditional
            // raw-pointer ownership transfer.
            unsafe { drop(Box::from_raw(data_bind.as_ptr())) };
        }
    }
}

impl ScriptInput {
    pub fn from(component: CoreHandle, type_key: CoreTypeKey) -> Option<CoreHandle> {
        match type_key.value() {
            621 | 631 | 626 | 611 | 627 | 618 | 612 => Some(component),
            _ => None,
        }
    }

    pub fn data_bind(&self) -> Option<NonNull<DataBind>> {
        self.data_bind
    }

    pub fn set_data_bind(&mut self, data_bind: Option<NonNull<DataBind>>, owns_data_bind: bool) {
        self.data_bind = data_bind;
        self.owns_data_bind = owns_data_bind;
    }

    pub fn scripted_object(&self) -> Option<NonNull<ScriptedObject>> {
        self.scripted_object
    }

    pub fn set_scripted_object(&mut self, object: Option<NonNull<ScriptedObject>>) {
        self.scripted_object = object;
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

pub struct ScriptAsset {
    pub base: ScriptAssetBase,
    optional_methods: OptionalScriptedMethods,
    module_details: ModuleDetails,
    file: Option<NonNull<File>>,
    #[cfg(feature = "rive_scripting")]
    script_registered: bool,
    #[cfg(feature = "rive_scripting")]
    bytecode: Vec<u8>,
    #[cfg(feature = "rive_scripting")]
    initted: bool,
}

impl Default for ScriptAsset {
    fn default() -> Self {
        Self {
            base: ScriptAssetBase::default(),
            optional_methods: OptionalScriptedMethods::default(),
            module_details: ModuleDetails::default(),
            file: None,
            #[cfg(feature = "rive_scripting")]
            script_registered: false,
            #[cfg(feature = "rive_scripting")]
            bytecode: Vec::new(),
            #[cfg(feature = "rive_scripting")]
            initted: false,
        }
    }
}

impl ScriptAsset {
    #[cfg(feature = "rive_scripting")]
    pub fn verified(&self) -> bool {
        self.base.text_asset().verified()
    }

    #[cfg(feature = "rive_scripting")]
    pub fn module_bytecode(&mut self) -> &mut [u8] {
        &mut self.bytecode
    }

    pub fn init_scripted_object(&mut self, object: &mut ScriptedObject) -> bool {
        #[cfg(feature = "rive_scripting")]
        {
            if self.scripting_vm().is_none() {
                return false;
            }
            return self.init_scripted_object_with(object);
        }
        #[cfg(not(feature = "rive_scripting"))]
        {
            let _ = object;
            false
        }
    }

    pub fn decode(&mut self, data: &mut Vec<u8>, _factory: &mut Factory) -> bool {
        #[cfg(feature = "rive_scripting")]
        {
            self.base.text_asset_mut().set_verified(false);
            let header = SignedContentHeader::new(data.as_slice());
            if !header.is_valid() {
                return false;
            }
            self.bytecode = header.content().to_vec();
        }
        true
    }

    pub fn bytecode(&mut self, data: &mut [u8]) -> bool {
        #[cfg(feature = "rive_scripting")]
        {
            let header = SignedContentHeader::new(data);
            if !header.is_valid() {
                self.base.text_asset_mut().set_verified(false);
                return false;
            }
            let bytecode = header.content();
            if !header.is_signed() {
                self.base.text_asset_mut().set_verified(false);
                self.bytecode = bytecode.to_vec();
                return true;
            }
            if !verify_hydrogen_signature(header.signature(), bytecode, b"RiveCode") {
                self.base.text_asset_mut().set_verified(false);
                return false;
            }
            self.base.text_asset_mut().set_verified(true);
            self.bytecode = bytecode.to_vec();
        }
        true
    }

    pub fn file_extension(&self) -> &'static str {
        "lua"
    }

    pub fn set_file(&mut self, value: Option<NonNull<File>>) {
        self.file = value;
    }

    pub fn file(&self) -> Option<NonNull<File>> {
        self.file
    }

    #[cfg(feature = "rive_scripting")]
    pub fn scripting_vm(&mut self) -> Option<&mut ScriptingVm> {
        let mut file = self.file?;
        unsafe { file.as_mut().scripting_vm() }
    }

    #[cfg(feature = "rive_scripting")]
    pub fn registration_complete(&mut self, reference: i32) {
        if self.base.is_module() {
            self.script_registered = true;
        } else {
            self.base.set_generator_function_ref(reference as u32);
            self.initted = false;
        }
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

    fn init_scripted_object_with(&mut self, object: &mut ScriptedObject) -> bool {
        #[cfg(feature = "rive_scripting")]
        {
            let mut reference = self.base.generator_function_ref() as i32;
            #[cfg(feature = "rive_tools")]
            {
                let generator_reference = self.base.generator_function_ref();
                if let Some(tool_reference) = self
                    .scripting_vm()
                    .and_then(|vm| vm.tool_generator_reference(generator_reference))
                {
                    reference = tool_reference;
                }
            }
            if reference == 0 {
                eprintln!(
                    "ScriptAsset doesn't have a generator function {}",
                    self.base.name()
                );
                return false;
            }
            if !self
                .scripting_vm()
                .expect("initScriptedObject checked the VM")
                .push_reference(reference)
            {
                return false;
            }
            if !self.initted {
                self.optional_methods.set_implemented_methods(
                    (self.base.serialized_implemented_methods()
                        & OptionalScriptedMethods::METHOD_MASK) as i32,
                );
                self.initted = true;
            }
            object.set_implemented_methods(self.optional_methods.implemented_methods());
            return object.ensure_script_initialized(
                self.scripting_vm()
                    .expect("the ScriptAsset VM remains available during initialization"),
            );
        }
        #[cfg(not(feature = "rive_scripting"))]
        {
            let _ = object;
            false
        }
    }

    pub fn optional_methods(&self) -> &OptionalScriptedMethods {
        &self.optional_methods
    }

    pub fn module_details(&self) -> &ModuleDetails {
        &self.module_details
    }

    pub fn module_details_mut(&mut self) -> &mut ModuleDetails {
        &mut self.module_details
    }
}
