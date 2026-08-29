use crate::mechanical_port::source::{
    assets::script_asset::{ScriptInput, ScriptInputBehavior},
    core_context::CoreContext,
    generated::{
        custom_property_boolean_base::CustomPropertyBooleanBaseCallbacks,
        script_input_boolean_base::ScriptInputBooleanBase,
        scripted::scripted_drawable_base::ScriptedDrawableBase,
    },
    importers::{import_stack::ImportStack, scripted_object_importer::ScriptedObjectImporter},
    status_code::StatusCode,
};

pub struct ScriptInputBoolean {
    pub base: ScriptInputBooleanBase,
    script_input: ScriptInput,
}

impl Default for ScriptInputBoolean {
    fn default() -> Self {
        Self {
            base: ScriptInputBooleanBase::default(),
            script_input: ScriptInput::default(),
        }
    }
}

impl ScriptInputBoolean {
    fn name(&self) -> &str {
        self.base.base.base.base.base.base.base.name()
    }

    fn property_value(&self) -> bool {
        self.base.base.base.property_value()
    }

    pub fn init_scripted_value(&mut self) {
        let name = self.name().to_owned();
        let value = self.property_value();
        if let Some(object) = self.script_input.scripted_object() {
            crate::mechanical_port::source::scripted::scripted_object::ScriptedObject::set_primitive_input(
                &object, name,
                crate::mechanical_port::source::scripted::scripted_object::ScriptValue::Boolean(value),
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
            ScriptInputBooleanBase::TYPE_KEY.into(),
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
                crate::mechanical_port::source::scripted::scripted_object::ScriptValue::Boolean(value),
            );
        }
    }
}

impl ScriptInputBehavior for ScriptInputBoolean {
    fn script_input(&self) -> &ScriptInput {
        &self.script_input
    }

    fn script_input_mut(&mut self) -> &mut ScriptInput {
        &mut self.script_input
    }

    fn validate_for_script_init(&self) -> bool {
        ScriptInputBoolean::validate_for_script_init(self)
    }

    fn init_scripted_value(&mut self) {
        ScriptInputBoolean::init_scripted_value(self);
    }
}

impl CustomPropertyBooleanBaseCallbacks for ScriptInputBoolean {
    fn property_value_changed(&mut self) {
        ScriptInputBoolean::property_value_changed(self);
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

impl Drop for ScriptInputBoolean {
    fn drop(&mut self) {
        if let (Some(this), Some(object)) =
            (self.base.handle(), self.script_input.scripted_object())
        {
            object.with_mut(|object| object.scripted_object_remove_property(&this));
        }
    }
}

impl std::ops::Deref for ScriptInputBoolean {
    type Target = ScriptInputBooleanBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScriptInputBoolean {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
