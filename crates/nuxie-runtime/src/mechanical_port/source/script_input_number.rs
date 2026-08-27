use std::ptr::NonNull;

use crate::mechanical_port::source::{
    assets::script_asset::{ScriptInput, ScriptInputBehavior},
    core_context::CoreContext,
    custom_property::CustomProperty,
    generated::{
        custom_property_number_base::CustomPropertyNumberBaseCallbacks,
        script_input_number_base::ScriptInputNumberBase,
        scripted::scripted_drawable_base::ScriptedDrawableBase,
    },
    importers::{import_stack::ImportStack, scripted_object_importer::ScriptedObjectImporter},
    status_code::StatusCode,
};

pub struct ScriptInputNumber {
    pub base: ScriptInputNumberBase,
    script_input: ScriptInput,
}

impl Default for ScriptInputNumber {
    fn default() -> Self {
        Self {
            base: ScriptInputNumberBase::default(),
            script_input: ScriptInput::default(),
        }
    }
}

impl ScriptInputNumber {
    fn custom_property_mut(&mut self) -> &mut CustomProperty {
        &mut self.base.base.base.base
    }

    fn name(&self) -> &str {
        self.base.base.base.base.base.base.base.name()
    }

    fn property_value(&self) -> f32 {
        self.base.base.base.property_value()
    }

    pub fn init_scripted_value(&mut self) {
        let name = self.name().to_owned();
        let value = self.property_value();
        if let Some(mut object) = self.script_input.scripted_object() {
            unsafe { object.as_mut() }.set_number_input(name, value);
        }
    }

    pub fn validate_for_script_init(&self) -> bool {
        true
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) =
            import_stack.latest::<ScriptedObjectImporter>(ScriptedDrawableBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        importer.add_input(
            NonNull::from(self.custom_property_mut()),
            ScriptInputNumberBase::TYPE_KEY.into(),
        );

        if self
            .script_input
            .scripted_object()
            .is_some_and(|object| unsafe { object.as_ref() }.component().is_some())
        {
            return self.base.base.base.base.base.base.import(import_stack);
        }
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.base.base.base.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }

        let property = self.custom_property_mut() as *mut CustomProperty;
        if let Some(parent) = self
            .base
            .base
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

    pub fn property_value_changed(&mut self) {
        let name = self.name().to_owned();
        let value = self.property_value();
        if let Some(mut object) = self.script_input.scripted_object() {
            unsafe { object.as_mut() }.set_number_input(name, value);
        }
    }
}

impl ScriptInputBehavior for ScriptInputNumber {
    fn script_input(&self) -> &ScriptInput {
        &self.script_input
    }

    fn script_input_mut(&mut self) -> &mut ScriptInput {
        &mut self.script_input
    }

    fn validate_for_script_init(&self) -> bool {
        ScriptInputNumber::validate_for_script_init(self)
    }

    fn init_scripted_value(&mut self) {
        ScriptInputNumber::init_scripted_value(self);
    }
}

impl CustomPropertyNumberBaseCallbacks for ScriptInputNumber {
    fn property_value_changed(&mut self) {
        ScriptInputNumber::property_value_changed(self);
    }

    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}

impl Drop for ScriptInputNumber {
    fn drop(&mut self) {
        let property = self.custom_property_mut() as *mut CustomProperty;
        if let Some(mut object) = self.script_input.scripted_object() {
            unsafe { object.as_mut() }.remove_property(property);
        }
    }
}
