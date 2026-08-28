use crate::mechanical_port::source::{
    animation::nested_state_machine::NestedStateMachine, core::CoreHandle,
    core_context::CoreContext, generated::animation::nested_input_base::NestedInputBase,
    status_code::StatusCode,
};

pub struct NestedInput {
    pub base: NestedInputBase,
    parent_state_machine: Option<CoreHandle>,
}

impl Default for NestedInput {
    fn default() -> Self {
        Self {
            base: NestedInputBase::default(),
            parent_state_machine: None,
        }
    }
}

impl NestedInput {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let result = self.base.base.on_added_dirty(context);
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        if let Some(parent) = context.resolve(self.base.base.base.parent_id()) {
            parent.with_downcast_mut::<NestedStateMachine, _>(|state_machine| {
                state_machine.add_nested_input(this)
            });
            self.parent_state_machine = Some(parent);
        }
        result
    }

    pub fn apply_value(&mut self) {}

    pub fn name(&self, context: &dyn CoreContext) -> String {
        let parent = context.resolve(self.base.base.base.parent_id());
        parent
            .and_then(|parent| {
                parent.with_downcast::<NestedStateMachine, _>(|nested| {
                    nested.input_name(self.base.input_id())
                })
            })
            .flatten()
            .unwrap_or_default()
    }

    pub(crate) fn input_name(&self) -> String {
        self.parent_state_machine
            .as_ref()
            .and_then(|parent| {
                parent.with_downcast::<NestedStateMachine, _>(|nested| {
                    nested.input_name(self.base.input_id())
                })
            })
            .flatten()
            .unwrap_or_default()
    }

    pub(crate) fn bool_value(&self) -> Option<bool> {
        self.parent_state_machine
            .as_ref()?
            .with_downcast::<NestedStateMachine, _>(|nested| {
                nested.bool_input_value(self.base.input_id())
            })?
    }

    pub(crate) fn set_bool_value(&self, value: bool) {
        if let Some(parent) = &self.parent_state_machine {
            parent.with_downcast_mut::<NestedStateMachine, _>(|nested| {
                nested.set_bool_input(self.base.input_id(), value)
            });
        }
    }

    pub(crate) fn number_value(&self) -> Option<f32> {
        self.parent_state_machine
            .as_ref()?
            .with_downcast::<NestedStateMachine, _>(|nested| {
                nested.number_input_value(self.base.input_id())
            })?
    }

    pub(crate) fn set_number_value(&self, value: f32) {
        if let Some(parent) = &self.parent_state_machine {
            parent.with_downcast_mut::<NestedStateMachine, _>(|nested| {
                nested.set_number_input(self.base.input_id(), value)
            });
        }
    }

    pub(crate) fn fire_trigger(&self) {
        if let Some(parent) = &self.parent_state_machine {
            parent.with_downcast_mut::<NestedStateMachine, _>(|nested| {
                nested.fire_trigger_input(self.base.input_id())
            });
        }
    }
}
impl std::ops::Deref for NestedInput {
    type Target = NestedInputBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for NestedInput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
    for NestedInput
{
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
}
impl crate::mechanical_port::source::generated::animation::nested_input_base::NestedInputBaseCallbacks for NestedInput { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
