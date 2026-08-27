use std::{cell::Cell, ptr::NonNull};

use crate::mechanical_port::source::{
    assets::script_asset::{ScriptInput, ScriptInputBehavior},
    core_context::CoreContext,
    custom_property::CustomProperty,
    data_bind::data_context::DataContext,
    data_bind_path_referencer::DataBindPathReferencer,
    generated::{
        script_input_viewmodel_property_base::{
            ScriptInputViewModelPropertyBase, ScriptInputViewModelPropertyBaseCallbacks,
        },
        scripted::scripted_drawable_base::ScriptedDrawableBase,
    },
    importers::{import_stack::ImportStack, scripted_object_importer::ScriptedObjectImporter},
    status_code::StatusCode,
    viewmodel::viewmodel_instance_value::ViewModelInstanceValue,
};

#[repr(C)]
pub struct ScriptInputViewModelProperty {
    pub base: ScriptInputViewModelPropertyBase,
    script_input: ScriptInput,
    data_bind_path_referencer: DataBindPathReferencer,
    view_model_instance_value: Cell<Option<NonNull<ViewModelInstanceValue>>>,
}

impl Default for ScriptInputViewModelProperty {
    fn default() -> Self {
        Self {
            base: ScriptInputViewModelPropertyBase::default(),
            script_input: ScriptInput::default(),
            data_bind_path_referencer: DataBindPathReferencer::default(),
            view_model_instance_value: Cell::new(None),
        }
    }
}

impl ScriptInputViewModelProperty {
    fn custom_property_mut(&mut self) -> &mut CustomProperty {
        &mut self.base.base
    }

    fn name(&self) -> &str {
        self.base.base.base.base.base.name()
    }

    pub fn decode_data_bind_path_ids(&mut self, value: &[u8]) {
        self.data_bind_path_referencer.decode_data_bind_path(value);
    }

    pub fn copy_data_bind_path_ids(&mut self, object: &ScriptInputViewModelPropertyBase) {
        let object = unsafe {
            &*(object as *const ScriptInputViewModelPropertyBase
                as *const ScriptInputViewModelProperty)
        };
        self.data_bind_path_referencer
            .copy_data_bind_path(object.data_bind_path_referencer.data_bind_path());
    }

    pub fn init_scripted_value(&mut self) {
        let Some(view_model_instance_value) = self.view_model_instance_value.get() else {
            return;
        };
        let name = self.name().to_owned();
        if let Some(mut object) = self.script_input.scripted_object() {
            unsafe { object.as_mut() }
                .set_view_model_input(name, view_model_instance_value.as_ptr() as usize);
        }
    }

    pub fn validate_for_script_init(&self) -> bool {
        self.view_model_instance_value.set(None);
        true
    }

    pub fn validate_for_cold_script_init(&self) -> bool {
        self.view_model_instance_value.set(None);
        true
    }

    pub fn validate_hydration_prerequisites(&self) -> bool {
        let Some(object) = self.script_input.scripted_object() else {
            return false;
        };
        let Some(data_context) = unsafe { object.as_ref() }.data_context() else {
            return false;
        };
        let Some(data_bind_path) = self.data_bind_path_referencer.data_bind_path() else {
            return false;
        };
        let Some(instance_value) = unsafe { (data_context as *mut DataContext).as_mut() }
            .and_then(|context| context.get_view_model_property(data_bind_path))
        else {
            return false;
        };
        instance_value.view_model_property().is_some()
    }

    pub fn hydrate_script_input(&mut self) -> bool {
        self.view_model_instance_value.set(None);
        let Some(object) = self.script_input.scripted_object() else {
            return false;
        };
        let Some(data_context) = unsafe { object.as_ref() }.data_context() else {
            return false;
        };
        let Some(data_bind_path) = self.data_bind_path_referencer.data_bind_path() else {
            return false;
        };
        let Some(instance_value) = unsafe { (data_context as *mut DataContext).as_mut() }
            .and_then(|context| context.get_view_model_property(data_bind_path))
        else {
            return false;
        };
        if instance_value.view_model_property().is_none() {
            return false;
        }
        self.view_model_instance_value
            .set(Some(NonNull::from(instance_value.as_ref())));
        self.init_scripted_value();
        true
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.data_bind_path_referencer
            .import_data_bind_path(import_stack);
        let Some(importer) =
            import_stack.latest::<ScriptedObjectImporter>(ScriptedDrawableBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        importer.add_input(
            NonNull::from(self.custom_property_mut()),
            ScriptInputViewModelPropertyBase::TYPE_KEY.into(),
        );

        if self
            .script_input
            .scripted_object()
            .is_some_and(|object| unsafe { object.as_ref() }.component().is_some())
        {
            return self.base.base.base.base.import(import_stack);
        }
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.base.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }

        let property = self.custom_property_mut() as *mut CustomProperty;
        if let Some(parent) = self
            .base
            .base
            .base
            .base
            .parent_mut()
            .and_then(|parent| parent.as_scripted_object_mut())
        {
            parent.add_property(property);
        }
        StatusCode::Ok
    }
}

impl ScriptInputBehavior for ScriptInputViewModelProperty {
    fn script_input(&self) -> &ScriptInput {
        &self.script_input
    }

    fn script_input_mut(&mut self) -> &mut ScriptInput {
        &mut self.script_input
    }

    fn validate_for_script_init(&self) -> bool {
        ScriptInputViewModelProperty::validate_for_script_init(self)
    }

    fn init_scripted_value(&mut self) {
        ScriptInputViewModelProperty::init_scripted_value(self);
    }

    fn validate_for_cold_script_init(&self) -> bool {
        ScriptInputViewModelProperty::validate_for_cold_script_init(self)
    }

    fn hydrate_script_input(&mut self) -> bool {
        ScriptInputViewModelProperty::hydrate_script_input(self)
    }

    fn validate_hydration_prerequisites(&self) -> bool {
        ScriptInputViewModelProperty::validate_hydration_prerequisites(self)
    }
}

impl ScriptInputViewModelPropertyBaseCallbacks for ScriptInputViewModelProperty {
    fn decode_data_bind_path_ids(&mut self, value: &[u8]) {
        ScriptInputViewModelProperty::decode_data_bind_path_ids(self, value);
    }

    fn copy_data_bind_path_ids(&mut self, object: &ScriptInputViewModelPropertyBase) {
        ScriptInputViewModelProperty::copy_data_bind_path_ids(self, object);
    }

    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}

impl Drop for ScriptInputViewModelProperty {
    fn drop(&mut self) {
        let property = self.custom_property_mut() as *mut CustomProperty;
        if let Some(mut object) = self.script_input.scripted_object() {
            unsafe { object.as_mut() }.remove_property(property);
        }
    }
}
