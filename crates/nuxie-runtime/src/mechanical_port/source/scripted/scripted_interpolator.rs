use crate::mechanical_port::source::{
    core::{CoreHandle, CoreObject},
    generated::scripted::scripted_interpolator_base::ScriptedInterpolatorBase,
    importers::import_stack::ImportStack,
    scripted::scripted_object::{ScriptProtocol, ScriptUpdateRequestHost, ScriptedObject},
    status_code::StatusCode,
};
use crate::scripting::ScriptInterpolatorMethod;

#[derive(Default)]
pub struct ScriptedInterpolator {
    pub base: ScriptedInterpolatorBase,
    pub scripted: ScriptedObject,
    pub properties: Vec<CoreHandle>,
}

impl ScriptedInterpolator {
    pub fn asset_id(&self) -> u32 {
        self.base.script_asset_id()
    }

    pub fn transform(&self, factor: f32) -> f32 {
        self.scripted
            .call_interpolator(ScriptInterpolatorMethod::Transform, &[factor])
            .unwrap_or(factor)
    }

    pub fn transform_value(&mut self, from: f32, to: f32, factor: f32) -> f32 {
        self.scripted
            .call_interpolator(
                ScriptInterpolatorMethod::TransformValue,
                &[from, to, factor],
            )
            .unwrap_or(from + (to - from) * factor)
    }

    pub fn add_scripted_dirt(&mut self, _value: u32, _recurse: bool) -> bool {
        false
    }

    pub fn component(&self) -> Option<CoreHandle> {
        None
    }

    pub fn add_property(&mut self, property: CoreHandle) {
        let owner = CoreObject::core(self).handle();
        property.with_mut(|property| {
            property.script_input_set_scripted_object(owner);
        });
        self.properties.push(property);
    }

    pub fn remove_property(&mut self, property: &CoreHandle) {
        if let Some(index) = self.properties.iter().position(|item| item == property) {
            self.properties.remove(index);
        }
    }

    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(owner) = CoreObject::core(self).handle() else {
            return StatusCode::MissingObject;
        };
        let result = self.scripted.register_referencer(owner, stack);
        if result != StatusCode::Ok {
            return result;
        }
        self.base.base.import(stack)
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

    /// A short-borrow attachment callback preserves addDataBind's position
    /// before the input backlink and before script initialization.
    pub fn clone_scripted_occurrence(
        source: &CoreHandle,
        add_data_bind: impl FnMut(CoreHandle),
    ) -> Option<CoreHandle> {
        let (definition, properties) = source.with_downcast::<Self, _>(|source| {
            (source.clone_definition(), source.properties.clone())
        })?;
        let owner = source.insert_sibling(definition)?;
        let properties = ScriptedObject::clone_properties_with(&properties, &owner, add_data_bind);
        let mut host = ScriptUpdateRequestHost::default();
        ScriptedObject::reinit_occurrence(&owner, &properties, &mut host);
        // ScriptedInterpolator inherits the empty markNeedsUpdate.
        Some(owner)
    }

    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::Interpolator
    }
}
