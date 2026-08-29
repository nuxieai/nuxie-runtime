use crate::mechanical_port::source::{
    assets::script_asset::{ScriptInput, ScriptInputBehavior},
    core_context::CoreContext,
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
    fn name(&self) -> &str {
        self.base.base.base.base.base.base.base.name()
    }

    fn property_value(&self) -> f32 {
        self.base.base.base.property_value()
    }

    pub fn init_scripted_value(&mut self) {
        let name = self.name().to_owned();
        let value = self.property_value();
        if let Some(object) = self.script_input.scripted_object() {
            crate::mechanical_port::source::scripted::scripted_object::ScriptedObject::set_primitive_input(
                &object, name,
                crate::mechanical_port::source::scripted::scripted_object::ScriptValue::Number(value),
            );
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
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_input(
            this,
            ScriptInputNumberBase::TYPE_KEY.into(),
            &mut self.script_input,
        );

        if self.script_input.scripted_object().is_some_and(|object| {
            object
                .with(|object| object.as_component().is_some())
                .unwrap_or(false)
        }) {
            return self.base.base.base.base.base.base.import(import_stack);
        }
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.base.base.base.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }

        if let (Some(this), Some(parent)) = (self.base.handle(), self.base.parent_handle()) {
            parent.with_mut(|parent| {
                parent.scripted_object_add_property_from_input(this, &mut self.script_input)
            });
        }
        StatusCode::Ok
    }

    pub fn property_value_changed(&mut self) {
        let name = self.name().to_owned();
        let value = self.property_value();
        if let Some(object) = self.script_input.scripted_object() {
            crate::mechanical_port::source::scripted::scripted_object::ScriptedObject::set_primitive_input(
                &object, name,
                crate::mechanical_port::source::scripted::scripted_object::ScriptValue::Number(value),
            );
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
        if let (Some(this), Some(object)) =
            (self.base.handle(), self.script_input.scripted_object())
        {
            object.with_mut(|object| object.scripted_object_remove_property(&this));
        }
    }
}

impl std::ops::Deref for ScriptInputNumber {
    type Target = ScriptInputNumberBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScriptInputNumber {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
