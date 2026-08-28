use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::data_context::RuntimeDataContextHandle,
    generated::scripted::scripted_data_converter_base::ScriptedDataConverterBase,
    scripted::scripted_object::{ScriptProtocol, ScriptValue, ScriptedObject},
};
#[derive(Clone, Debug, PartialEq)]
pub enum DataValue {
    None,
    Boolean(bool),
    Integer(i32),
    Number(f32),
    String(String),
    Color(u32),
    List(Vec<DataValue>),
}
#[derive(Default)]
pub struct ScriptedDataConverter {
    pub base: ScriptedDataConverterBase,
    pub scripted: ScriptedObject,
    data_context: Option<RuntimeDataContextHandle>,
    data_value: Option<DataValue>,
    properties: Vec<CoreHandle>,
    converter_dirty: bool,
}
impl ScriptedDataConverter {
    pub fn asset_id(&self) -> u32 {
        self.base.script_asset_id()
    }

    pub fn did_hydrate_script_inputs(&mut self) {
        self.converter_dirty = true;
    }
    fn script_value(value: &DataValue) -> Option<ScriptValue> {
        match value {
            DataValue::Boolean(value) => Some(ScriptValue::Boolean(*value)),
            DataValue::Number(value) => Some(ScriptValue::Number(*value)),
            DataValue::String(value) => Some(ScriptValue::String(value.clone())),
            DataValue::Color(value) => Some(ScriptValue::Color(*value)),
            DataValue::None | DataValue::Integer(_) | DataValue::List(_) => None,
        }
    }
    fn apply_conversion(&mut self, v: DataValue, m: &str) -> DataValue {
        if self.scripted.self_ref() == 0 {
            return v;
        }
        let Some(input) = Self::script_value(&v) else {
            return self.data_value.clone().unwrap_or(DataValue::None);
        };
        if let Some(result) = self.scripted.call_value(m, &input) {
            self.data_value = Some(match result {
                ScriptValue::Boolean(value) => DataValue::Boolean(value),
                ScriptValue::Color(value) => DataValue::Color(value),
                ScriptValue::Integer(value) => DataValue::Integer(value),
                ScriptValue::Number(value) => DataValue::Number(value),
                ScriptValue::String(value) => DataValue::String(value),
                ScriptValue::Artboard(_) | ScriptValue::ViewModel(_) | ScriptValue::Trigger => {
                    DataValue::None
                }
            });
        }
        self.data_value.clone().unwrap_or(DataValue::None)
    }
    pub fn convert(&mut self, v: DataValue) -> DataValue {
        if !self.scripted.data_converts() {
            return v;
        }
        let out = self.apply_conversion(v, "convert");
        out
    }
    pub fn reverse_convert(&mut self, v: DataValue) -> DataValue {
        if !self.scripted.data_reverse_converts() {
            return v;
        }
        let out = self.apply_conversion(v, "reverseConvert");
        out
    }
    pub fn bind_from_context(&mut self, c: Option<RuntimeDataContextHandle>) {
        self.data_context = c.clone();
        self.scripted.set_data_context(c);
        self.scripted.reinit();
        self.converter_dirty = true
    }
    pub fn advance_component(&mut self, mut e: f32, advance_nested: bool) -> bool {
        if !advance_nested {
            e = 0.0;
        }
        self.advance(e)
    }
    pub fn advance(&mut self, e: f32) -> bool {
        if e == 0.0 {
            return false;
        }
        let needs_advance = self.scripted.script_advance(e);
        if needs_advance {
            self.converter_dirty = true;
        }
        needs_advance
    }
    pub fn add_property(&mut self, p: CoreHandle) {
        self.properties.push(p)
    }
    pub fn remove_property(&mut self, property: &CoreHandle) {
        self.properties.retain(|item| item != property)
    }
    pub fn add_scripted_dirt(&mut self) {
        self.converter_dirty = true
    }
    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::Converter
    }
    pub fn push_value_as_input(&mut self, name: String, v: &DataValue) {
        match v {
            DataValue::Boolean(v) => self.scripted.set_boolean_input(name, *v),
            DataValue::Integer(v) => self.scripted.set_integer_input(name, *v),
            DataValue::Number(v) => self.scripted.set_number_input(name, *v),
            DataValue::String(v) => self.scripted.set_string_input(name, v.clone()),
            _ => self.scripted.set_string_input(name, String::new()),
        }
    }
    pub fn add_data_bind_from_scripted_object(&mut self, data_bind: CoreHandle) -> bool {
        self.base.base.add_data_bind(data_bind);
        true
    }
}
