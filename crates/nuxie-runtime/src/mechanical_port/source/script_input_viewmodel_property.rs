use std::cell::RefCell;

use crate::mechanical_port::source::{
    assets::script_asset::{ScriptInput, ScriptInputBehavior},
    core_context::CoreContext,
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
    view_model_instance_value: RefCell<Option<crate::mechanical_port::source::core::CoreHandle>>,
}

impl Default for ScriptInputViewModelProperty {
    fn default() -> Self {
        Self {
            base: ScriptInputViewModelPropertyBase::default(),
            script_input: ScriptInput::default(),
            data_bind_path_referencer: DataBindPathReferencer::default(),
            view_model_instance_value: RefCell::new(None),
        }
    }
}

impl ScriptInputViewModelProperty {
    fn name(&self) -> &str {
        self.base.base.base.base.base.name()
    }

    pub fn decode_data_bind_path_ids(&mut self, value: &[u8]) {
        self.data_bind_path_referencer.decode_data_bind_path(value);
    }

    pub fn clone_core(&self) -> Self {
        let mut cloned = Self::default();
        cloned
            .data_bind_path_referencer
            .copy_data_bind_path(&self.data_bind_path_referencer);
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(&self.base, &mut cloned);
        cloned.base = base;
        cloned
    }

    pub fn init_scripted_value(&mut self) {
        let Some(view_model_instance_value) = self.view_model_instance_value.borrow().clone()
        else {
            return;
        };
        let name = self.name().to_owned();
        if let Some(object) = self.script_input.scripted_object() {
            object.with_mut(|object| {
                if let Some(object) = object.as_scripted_object_mut() {
                    object.set_view_model_input(name, view_model_instance_value);
                }
            });
        }
    }

    pub fn validate_for_script_init(&self) -> bool {
        *self.view_model_instance_value.borrow_mut() = None;
        true
    }

    pub fn validate_for_cold_script_init(&self) -> bool {
        *self.view_model_instance_value.borrow_mut() = None;
        true
    }

    pub fn validate_hydration_prerequisites(&self) -> bool {
        let Some(object) = self.script_input.scripted_object() else {
            return false;
        };
        let Some(data_context) = object
            .with(|object| {
                object
                    .as_scripted_object()
                    .and_then(|object| object.data_context())
            })
            .flatten()
        else {
            return false;
        };
        let Some(data_bind_path) = self
            .data_bind_path_referencer
            .with_data_bind_path(|path| path.path().to_vec())
        else {
            return false;
        };
        let Some(instance_value) = data_context
            .borrow()
            .get_view_model_property(&data_bind_path)
        else {
            return false;
        };
        instance_value
            .with(|value| {
                value
                    .as_view_model_instance_value()
                    .is_some_and(|value| value.view_model_property().is_some())
            })
            .unwrap_or(false)
    }

    pub fn hydrate_script_input(&mut self) -> bool {
        *self.view_model_instance_value.borrow_mut() = None;
        let Some(object) = self.script_input.scripted_object() else {
            return false;
        };
        let Some(data_context) = object
            .with(|object| {
                object
                    .as_scripted_object()
                    .and_then(|object| object.data_context())
            })
            .flatten()
        else {
            return false;
        };
        let Some(data_bind_path) = self
            .data_bind_path_referencer
            .with_data_bind_path(|path| path.path().to_vec())
        else {
            return false;
        };
        let Some(instance_value) = data_context
            .borrow()
            .get_view_model_property(&data_bind_path)
        else {
            return false;
        };
        if !instance_value
            .with(|value| {
                value
                    .as_view_model_instance_value()
                    .is_some_and(|value| value.view_model_property().is_some())
            })
            .unwrap_or(false)
        {
            return false;
        }
        *self.view_model_instance_value.borrow_mut() = Some(instance_value);
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
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_input(this, ScriptInputViewModelPropertyBase::TYPE_KEY.into());

        if self.script_input.scripted_object().is_some_and(|object| {
            object
                .with(|object| object.as_component().is_some())
                .unwrap_or(false)
        }) {
            return self.base.base.base.base.import(import_stack);
        }
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.base.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }

        if let (Some(this), Some(parent)) = (self.base.handle(), self.base.parent_handle()) {
            parent.with_mut(|parent| parent.scripted_object_add_property(this));
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
        if let (Some(this), Some(object)) =
            (self.base.handle(), self.script_input.scripted_object())
        {
            object.with_mut(|object| object.scripted_object_remove_property(&this));
        }
    }
}

impl std::ops::Deref for ScriptInputViewModelProperty {
    type Target = ScriptInputViewModelPropertyBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ScriptInputViewModelProperty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
