use crate::mechanical_port::source::{
    animation::{
        listener_input_value::ListenerInputValue, listener_invocation::ListenerInvocation,
        state_machine_instance::StateMachineInstance,
    },
    component_dirt::ComponentDirt,
    core::CoreHandle,
    data_bind::bindable_property_viewmodel::BindablePropertyViewModel,
    generated::animation::listener_viewmodel_change_base::ListenerViewModelChangeBase,
    importers::{bindable_property_importer::BindablePropertyImporter, import_stack::ImportStack},
    status_code::StatusCode,
};
use crate::mechanical_port::source::{
    generated::{
        core_registry::CoreRegistry,
        data_bind::{
            bindable_property_boolean_base::BindablePropertyBooleanBase,
            bindable_property_number_base::BindablePropertyNumberBase,
            bindable_property_string_base::BindablePropertyStringBase,
        },
    },
    input::gamepad_snapshot::GamepadSnapshot,
};

fn gamepad_snapshot(invocation: &ListenerInvocation) -> Option<&GamepadSnapshot> {
    invocation
        .as_gamepad_event()
        .map(|event| &event.full_state)
        .or_else(|| {
            invocation
                .as_gamepad_connected()
                .map(|event| &event.snapshot)
        })
}
fn gamepad_value_at(values: &[f32], index: usize) -> f32 {
    values.get(index).copied().unwrap_or(0.0)
}
fn apply_number(bindable: &CoreHandle, value: f32) -> bool {
    bindable.is_type_of(BindablePropertyNumberBase::TYPE_KEY)
        && CoreRegistry::set_double_handle(
            bindable,
            i32::from(BindablePropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY),
            value,
        )
}
fn apply_boolean(bindable: &CoreHandle, value: bool) -> bool {
    bindable.is_type_of(BindablePropertyBooleanBase::TYPE_KEY)
        && CoreRegistry::set_bool_handle(
            bindable,
            i32::from(BindablePropertyBooleanBase::PROPERTY_VALUE_PROPERTY_KEY),
            value,
        )
}
fn apply_string(bindable: &CoreHandle, value: &str) -> bool {
    bindable.is_type_of(BindablePropertyStringBase::TYPE_KEY)
        && CoreRegistry::set_string_handle(
            bindable,
            i32::from(BindablePropertyStringBase::PROPERTY_VALUE_PROPERTY_KEY),
            value.to_owned(),
        )
}
#[derive(Default)]
pub struct ListenerViewModelChange {
    pub base: ListenerViewModelChangeBase,
    bindable_property: Option<CoreHandle>,
}
impl Drop for ListenerViewModelChange {
    fn drop(&mut self) {
        if let Some(property) = self.bindable_property.take() {
            property.remove_occurrence();
        }
    }
}

impl std::ops::Deref for ListenerViewModelChange {
    type Target = ListenerViewModelChangeBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ListenerViewModelChange {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl ListenerViewModelChange {
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<BindablePropertyImporter>(crate::mechanical_port::source::generated::data_bind::bindable_property_base::BindablePropertyBase::TYPE_KEY) else { return StatusCode::MissingObject };
        self.bindable_property = importer.bindable_property();
        self.base.base.import(stack)
    }
    fn apply_input_value(&self, bindable: &CoreHandle, invocation: &ListenerInvocation) -> bool {
        let index = self.input_value_index() as usize;
        let Some(value) = ListenerInputValue::from_value(self.input_value() as u32) else {
            return false;
        };
        match value {
            ListenerInputValue::None => true,
            ListenerInputValue::PointerX
            | ListenerInputValue::PointerY
            | ListenerInputValue::PointerDeltaX
            | ListenerInputValue::PointerDeltaY => {
                let Some(pointer) = invocation.as_pointer() else {
                    return false;
                };
                let delta = pointer.position - pointer.previous_position;
                apply_number(
                    bindable,
                    match value {
                        ListenerInputValue::PointerX => pointer.position.x,
                        ListenerInputValue::PointerY => pointer.position.y,
                        ListenerInputValue::PointerDeltaX => delta.x,
                        _ => delta.y,
                    },
                )
            }
            ListenerInputValue::KeyPressed => invocation
                .as_keyboard()
                .is_some_and(|keyboard| apply_boolean(bindable, keyboard.is_pressed)),
            ListenerInputValue::Text => invocation
                .as_text_input()
                .is_some_and(|text| apply_string(bindable, &text.text)),
            ListenerInputValue::Focused => invocation
                .as_focus()
                .is_some_and(|focus| apply_boolean(bindable, focus.is_focus)),
            ListenerInputValue::GamepadButtonPressed => {
                gamepad_snapshot(invocation).is_some_and(|snapshot| {
                    apply_boolean(
                        bindable,
                        index < 64 && snapshot.button_mask & (1u64 << index) != 0,
                    )
                })
            }
            ListenerInputValue::GamepadButtonValue => {
                gamepad_snapshot(invocation).is_some_and(|snapshot| {
                    apply_number(bindable, gamepad_value_at(&snapshot.button_values, index))
                })
            }
            ListenerInputValue::GamepadAxis => {
                gamepad_snapshot(invocation).is_some_and(|snapshot| {
                    apply_number(bindable, gamepad_value_at(&snapshot.axes, index))
                })
            }
            ListenerInputValue::GamepadChangedValue => invocation
                .as_gamepad_event()
                .is_some_and(|event| apply_number(bindable, event.change.value)),
        }
    }

    pub fn perform(&self, machine: &mut StateMachineInstance, invocation: &ListenerInvocation) {
        let Some(property) = self.bindable_property.as_ref() else {
            return;
        };
        let Some(instance) = machine.bindable_property_instance(property) else {
            return;
        };
        if self.input_value() != 0 && !self.apply_input_value(&instance, invocation) {
            return;
        }
        let data_bind = machine.bindable_data_bind_to_source(&instance);
        let to_target = machine.bindable_data_bind_to_target(&instance);
        if let Some(data_bind) = data_bind {
            if let Some(target) = data_bind
                .with(|bind| bind.as_data_bind().and_then(|bind| bind.target()))
                .flatten()
            {
                if target
                    .is_type_of(crate::mechanical_port::source::generated::data_bind::bindable_property_viewmodel_base::BindablePropertyViewModelBase::TYPE_KEY)
                {
                    if let Some(context) = machine.data_context() {
                        let value =
                            context.with_context(|context| context.main_view_model_instance());
                        target.with_downcast_mut::<BindablePropertyViewModel, _>(|target| {
                            target.set_view_model_instance_value(value.clone())
                        });
                        let key = crate::mechanical_port::source::viewmodel::viewmodel_instance::ViewModelInstance::pointer_key(value.as_ref());
                        crate::mechanical_port::source::generated::core_registry::CoreRegistry::set_uint_handle(&target,
                            crate::mechanical_port::source::generated::data_bind::bindable_property_id_base::BindablePropertyIdBase::PROPERTY_VALUE_PROPERTY_KEY as i32, key);
                    }
                }
            }
            crate::mechanical_port::source::data_bind::data_bind::DataBind::update_source_binding_handle(
                &data_bind, true,
            );
        }
        if let Some(to_target) = to_target {
            crate::source::data_bind::data_bind::DataBind::add_dirt_handle(
                &to_target,
                ComponentDirt::BINDINGS.0 as u32,
                true,
            );
        }
    }
}
