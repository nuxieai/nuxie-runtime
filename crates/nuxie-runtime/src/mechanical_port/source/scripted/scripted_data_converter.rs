use crate::mechanical_port::source::{
    core::{CoreHandle, CoreObject},
    data_bind::{
        data_context::RuntimeDataContextHandle,
        data_values::{
            data_type::DataType,
            data_value::{DataValue, EmptyDataValue},
            data_value_boolean::DataValueBoolean,
            data_value_color::DataValueColor,
            data_value_number::DataValueNumber,
            data_value_string::DataValueString,
        },
    },
    generated::scripted::scripted_data_converter_base::ScriptedDataConverterBase,
    importers::import_stack::ImportStack,
    scripted::scripted_object::{ScriptProtocol, ScriptUpdateRequestHost, ScriptedObject},
    status_code::StatusCode,
};
use crate::scripting::{ScriptDataConverterMethod, ScriptDataConverterOptionalCall, ScriptValue};

impl crate::mechanical_port::source::generated::core_registry::DataConverterCapability
    for ScriptedDataConverter
{
    fn convert(
        &mut self,
        value: &dyn DataValue,
        _bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(Self::convert(self, value));
    }
    fn reverse_convert(
        &mut self,
        value: &dyn DataValue,
        _bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(Self::reverse_convert(self, value));
    }
    fn output_type(&self) -> DataType {
        DataType::Any
    }
    fn bind_context_handler(&self) -> crate::mechanical_port::source::data_bind::converters::data_converter::ConverterBindContextHandler{
        Self::bind_from_context_occurrence
    }
    fn unbind(&mut self) {
        self.base.base.unbind();
    }
    fn update(&mut self) {
        self.base.base.update();
    }
    fn reset(&mut self) {
        self.base.base.reset();
    }
    fn advance(&mut self, elapsed: f32) -> bool {
        Self::advance(self, elapsed)
    }
}

#[derive(Default)]
pub struct ScriptedDataConverter {
    pub base: ScriptedDataConverterBase,
    pub scripted: ScriptedObject,
    data_context: Option<RuntimeDataContextHandle>,
    data_value: Option<Box<dyn DataValue>>,
    pub properties: Vec<CoreHandle>,
}

impl Drop for ScriptedDataConverter {
    fn drop(&mut self) {
        ScriptedObject::dispose_owned_script_inputs(&mut self.properties);
    }
}

impl ScriptedDataConverter {
    pub fn asset_id(&self) -> u32 {
        self.base.script_asset_id()
    }
    pub fn output_type(&self) -> DataType {
        DataType::Any
    }
    pub fn component(&self) -> Option<CoreHandle> {
        None
    }
    pub fn data_context(&self) -> Option<RuntimeDataContextHandle> {
        self.data_context.clone()
    }

    pub fn did_hydrate_script_inputs(&mut self) {
        self.base.base.mark_converter_dirty();
    }

    fn script_value(value: &dyn DataValue) -> Option<ScriptValue> {
        if let Some(value) = value.as_any().downcast_ref::<DataValueNumber>() {
            Some(ScriptValue::Number(f64::from(value.value())))
        } else if let Some(value) = value.as_any().downcast_ref::<DataValueString>() {
            Some(ScriptValue::String(value.value().to_owned()))
        } else if let Some(value) = value.as_any().downcast_ref::<DataValueBoolean>() {
            Some(ScriptValue::Bool(value.value()))
        } else {
            value
                .as_any()
                .downcast_ref::<DataValueColor>()
                .map(|value| ScriptValue::Color(value.value()))
        }
    }

    /// Preserve the cached DataValue allocation while its dynamic type agrees.
    fn store<T: DataValue + Default>(&mut self, value: impl FnOnce(&mut T)) {
        if !self
            .data_value
            .as_ref()
            .is_some_and(|cache| cache.as_any().is::<T>())
        {
            self.data_value = Some(Box::<T>::default());
        }
        value(
            self.data_value
                .as_mut()
                .unwrap()
                .as_any_mut()
                .downcast_mut::<T>()
                .unwrap(),
        );
    }

    fn apply_conversion<'a>(
        &'a mut self,
        input: &'a dyn DataValue,
        method: ScriptDataConverterMethod,
    ) -> &'a dyn DataValue {
        if self.scripted.self_ref() == 0 {
            return input;
        }
        let Some(instance) = self.scripted.runtime_instance() else {
            return input;
        };
        // The backend resolves the field before testing input support, once.
        match instance
            .borrow_mut()
            .call_optional_data_converter(method, Self::script_value(input))
        {
            Ok(ScriptDataConverterOptionalCall::Missing) => return input,
            Ok(ScriptDataConverterOptionalCall::Returned(ScriptValue::Number(value))) => {
                self.store::<DataValueNumber>(|cache| cache.set_value(value as f32))
            }
            Ok(ScriptDataConverterOptionalCall::Returned(ScriptValue::Bool(value))) => {
                self.store::<DataValueBoolean>(|cache| cache.set_value(value))
            }
            Ok(ScriptDataConverterOptionalCall::Returned(ScriptValue::String(value))) => {
                self.store::<DataValueString>(|cache| cache.set_value(value))
            }
            Ok(ScriptDataConverterOptionalCall::Returned(ScriptValue::Color(value))) => {
                self.store::<DataValueColor>(|cache| cache.set_value(value))
            }
            // An unsupported input/output or protected error preserves the cache.
            _ => {}
        }
        self.data_value
            .get_or_insert_with(|| Box::new(EmptyDataValue))
            .as_ref()
    }

    pub fn convert<'a>(&'a mut self, value: &'a dyn DataValue) -> &'a dyn DataValue {
        if !self.scripted.data_converts() {
            return value;
        }
        self.apply_conversion(value, ScriptDataConverterMethod::Convert)
    }

    pub fn reverse_convert<'a>(&'a mut self, value: &'a dyn DataValue) -> &'a dyn DataValue {
        if !self.scripted.data_reverse_converts() {
            return value;
        }
        self.apply_conversion(value, ScriptDataConverterMethod::ReverseConvert)
    }

    /// Full virtual bind operation. The clone's Core borrow is released before
    /// reinit, because input hydration resolves this same occurrence.
    pub fn bind_from_context_occurrence(
        owner: &CoreHandle,
        context: RuntimeDataContextHandle,
        data_bind: CoreHandle,
    ) {
        let properties = owner
            .with_downcast_mut::<Self, _>(|converter| {
                converter.data_context = Some(context.clone());
                converter.scripted.set_data_context(Some(context.clone()));
                converter
                    .base
                    .base
                    .bind_from_context(context.clone(), data_bind);
                converter.properties.clone()
            })
            .expect("a retained scripted converter keeps its type");
        let mut host = ScriptUpdateRequestHost::default();
        ScriptedObject::reinit_occurrence(owner, &properties, &mut host);
        for property in properties {
            if let Some(bind) = property
                .with(|property| property.script_input_data_bind())
                .flatten()
            {
                bind.with_mut(|bind| {
                    bind.as_data_bind_context_mut()
                        .expect("script input bindings are DataBindContext")
                        .bind_from_context(Some(context.clone()));
                });
            }
        }
    }

    pub fn advance_component(&mut self, elapsed: f32, advance_nested: bool) -> bool {
        self.advance(if advance_nested { elapsed } else { 0.0 })
    }

    pub fn advance(&mut self, elapsed: f32) -> bool {
        if elapsed == 0.0 {
            return false;
        }
        let advanced = self.scripted.script_advance(elapsed);
        if advanced {
            self.base.base.mark_converter_dirty();
        }
        advanced
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

    pub fn add_scripted_dirt(&mut self, _value: u32, _recurse: bool) -> bool {
        self.base.base.mark_converter_dirty();
        true
    }

    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::Converter
    }

    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(owner) = CoreObject::core(self).handle() else {
            return StatusCode::MissingObject;
        };
        let result = self.scripted.register_referencer(owner, stack);
        if result != StatusCode::Ok {
            return result;
        }
        self.base.base.import_stack(stack)
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

    /// Completes DataConverter::copy, then the ScriptedDataConverter override.
    /// The destination already has its final identity before binding targets.
    pub fn complete_clone(source: &CoreHandle, clone: &CoreHandle) -> bool {
        let Some((source_binds, source_properties)) = source.with_downcast::<Self, _>(|source| {
            (source.base.base.data_binds(), source.properties.clone())
        }) else {
            return false;
        };
        for source_bind in &source_binds {
            let bind = source_bind
                .clone_occurrence()
                .expect("a retained DataBind has a clone");
            let file = source_bind
                .with(|bind| bind.as_data_bind().unwrap().file())
                .unwrap();
            bind.with_mut(|bind| {
                let bind = bind.as_data_bind_mut().unwrap();
                bind.set_target(Some(clone.clone()));
                bind.set_file(file);
            });
            clone
                .with_downcast_mut::<Self, _>(|clone| clone.base.base.add_data_bind(bind))
                .expect("the converter clone remains live");
        }
        let twin_binds = clone
            .with_downcast::<Self, _>(|clone| clone.base.base.data_binds())
            .unwrap();
        for property in source_properties {
            let cloned_property = property
                .clone_occurrence()
                .expect("a retained custom property has a clone");
            clone
                .with_downcast_mut::<Self, _>(|clone| clone.add_property(cloned_property.clone()))
                .unwrap();
            let source_has_bind = property
                .with(|property| property.script_input_data_bind().is_some())
                .unwrap();
            if source_has_bind {
                for (index, source_bind) in source_binds.iter().enumerate() {
                    let targets_property = source_bind
                        .with(|bind| {
                            bind.as_data_bind().unwrap().target().as_ref() == Some(&property)
                        })
                        .unwrap();
                    if targets_property && let Some(bind) = twin_binds.get(index) {
                        bind.with_mut(|bind| {
                            bind.as_data_bind_mut()
                                .unwrap()
                                .set_target(Some(cloned_property.clone()))
                        });
                        cloned_property.with_mut(|input| {
                            input.script_input_set_data_bind(Some(bind.clone()), true);
                        });
                    }
                }
            }
        }
        true
    }

    pub fn add_data_bind_from_scripted_object(&mut self, data_bind: CoreHandle) -> bool {
        self.base.base.add_data_bind(data_bind);
        true
    }
}
