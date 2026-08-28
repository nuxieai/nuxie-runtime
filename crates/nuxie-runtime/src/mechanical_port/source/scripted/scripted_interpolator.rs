use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::scripted::scripted_interpolator_base::ScriptedInterpolatorBase,
    scripted::scripted_object::{ScriptProtocol, ScriptedObject, ScriptedObjectClone},
};
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

    pub fn transform(&mut self, factor: f32) -> f32 {
        self.scripted
            .call_number("transform", &[factor])
            .unwrap_or(factor)
    }
    pub fn transform_value(&mut self, from: f32, to: f32, factor: f32) -> f32 {
        self.scripted
            .call_number("transformValue", &[from, to, factor])
            .unwrap_or(from + (to - from) * factor)
    }
    pub fn add_property(&mut self, p: CoreHandle) {
        self.properties.push(p)
    }
    pub fn remove_property(&mut self, property: &CoreHandle) {
        self.properties.retain(|item| item != property)
    }

    /// Clone the template into the same Core arena and clone each scripted
    /// input binding for the runtime host that owns the stateful occurrence.
    pub fn clone_scripted_object(&self) -> Option<ScriptedObjectClone> {
        let template = self.base.base.base.base.handle()?;
        let owner = template.clone_occurrence()?;
        let asset = self.scripted.script_asset();
        let mut cloned_properties = Vec::with_capacity(self.properties.len());
        let mut data_binds = Vec::new();

        for property in &self.properties {
            let Some(cloned_property) = property.clone_occurrence() else {
                continue;
            };
            cloned_property.with_mut(|property| {
                property.script_input_set_scripted_object(Some(owner.clone()));
            });

            let source_bind = property
                .with(|property| property.script_input_data_bind())
                .flatten();
            if let Some(source_bind) = source_bind
                && let Some(cloned_bind) = source_bind.clone_occurrence()
            {
                let file = source_bind
                    .with(|bind| bind.as_data_bind().map(|bind| bind.file()))
                    .flatten();
                cloned_bind.with_mut(|bind| {
                    if let Some(bind) = bind.as_data_bind_mut() {
                        bind.set_target(Some(cloned_property.clone()));
                        if let Some(file) = file {
                            bind.set_file(file);
                        }
                    }
                });
                cloned_property.with_mut(|property| {
                    property.script_input_set_data_bind(Some(cloned_bind.clone()), true);
                });
                data_binds.push(cloned_bind);
            }
            cloned_properties.push(cloned_property);
        }

        owner.with_downcast_mut::<ScriptedInterpolator, _>(|clone| {
            clone.properties = cloned_properties;
            clone.scripted.set_asset(owner.clone(), asset);
            clone.scripted.reinit();
        })?;

        Some(ScriptedObjectClone { owner, data_binds })
    }
    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::Interpolator
    }
}
