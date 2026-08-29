use crate::mechanical_port::source::{
    animation::state_machine_instance::{
        RuntimeStateMachineLayerInstanceWeakHandle, StateMachineInstance,
    },
    core::{CoreHandle, CoreObject},
    data_bind::data_bind_container::DataBindContainer,
    generated::animation::{
        scripted_transition_condition_base::ScriptedTransitionConditionBase,
        state_machine_base::StateMachineBase,
    },
    importers::{import_stack::ImportStack, state_machine_importer::StateMachineImporter},
    scripted::scripted_object::{ScriptProtocol, ScriptedObject},
    status_code::StatusCode,
};
use crate::scripting::ScriptHost;

#[derive(Default)]
pub struct ScriptedTransitionCondition {
    pub base: ScriptedTransitionConditionBase,
    pub scripted: ScriptedObject,
    pub properties: Vec<CoreHandle>,
}

impl Drop for ScriptedTransitionCondition {
    fn drop(&mut self) {
        self.dispose_script_inputs();
    }
}

impl ScriptedTransitionCondition {
    pub fn dispose_script_inputs(&mut self) {
        ScriptedObject::dispose_owned_script_inputs(&mut self.properties);
    }

    pub fn evaluate_stateful(owner: &CoreHandle, host: &mut dyn ScriptHost) -> bool {
        ScriptedObject::evaluate_condition(owner, host)
    }

    pub fn evaluate(
        &mut self,
        machine: &StateMachineInstance,
        _layer: &RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool {
        let Some(source) = CoreObject::core(self).handle() else {
            return false;
        };
        machine
            .scripted_object(&source)
            .is_some_and(|stateful| machine.evaluate_scripted_condition(&stateful))
    }

    pub fn asset_id(&self) -> u32 {
        self.base.script_asset_id()
    }

    pub fn add_scripted_dirt(&mut self, _value: u32, _recurse: bool) -> bool {
        false
    }

    pub fn component(&self) -> Option<CoreHandle> {
        None
    }

    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::TransitionCondition
    }

    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(owner) = CoreObject::core(self).handle() else {
            return StatusCode::MissingObject;
        };
        let result = self.scripted.register_referencer(owner.clone(), stack);
        if result != StatusCode::Ok {
            return result;
        }
        let Some(importer) = stack.latest::<StateMachineImporter>(StateMachineBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        importer.add_scripted_object(owner);
        self.base.base.import(stack)
    }

    pub fn add_property(&mut self, property: CoreHandle) {
        let owner = CoreObject::core(self).handle();
        property.with_mut(|property| {
            property.script_input_set_scripted_object(owner);
        });
        if !self.properties.contains(&property) {
            self.properties.push(property);
        }
    }

    pub(crate) fn add_property_from_input(
        &mut self,
        property: CoreHandle,
        input: &mut crate::mechanical_port::source::assets::script_asset::ScriptInput,
    ) {
        input.attach_to_container(
            CoreObject::core(self).handle(),
            property,
            &mut self.properties,
        );
    }

    pub fn remove_property(&mut self, property: &CoreHandle) {
        if let Some(index) = self
            .properties
            .iter()
            .position(|candidate| candidate == property)
        {
            self.properties.remove(index);
        }
    }

    pub fn clone_definition(&self) -> Self {
        let mut clone = Self::default();
        let mut base = std::mem::take(&mut clone.base);
        base.copy(&self.base, &mut clone);
        clone.base = base;
        clone
            .scripted
            .file_asset_referencer_mut()
            .set_asset_unattached(self.scripted.script_asset());
        clone
    }

    pub fn clone_scripted_occurrence(
        source: &CoreHandle,
        container: &mut DataBindContainer,
        host: &mut dyn ScriptHost,
    ) -> Option<CoreHandle> {
        let (definition, properties) = source.with_downcast::<Self, _>(|source| {
            (source.clone_definition(), source.properties.clone())
        })?;
        let owner = source.insert_sibling(definition)?;
        let cloned_properties = ScriptedObject::clone_properties(&properties, &owner, container);
        ScriptedObject::reinit_occurrence(&owner, &cloned_properties, host);
        Some(owner)
    }
}
