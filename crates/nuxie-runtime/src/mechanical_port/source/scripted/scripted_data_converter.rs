use crate::mechanical_port::source::scripted::scripted_object::{
    ScriptProtocol, ScriptValue, ScriptedObject,
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
    pub scripted: ScriptedObject,
    data_context: Option<usize>,
    data_value: Option<DataValue>,
    properties: Vec<usize>,
    advance_active: bool,
    converter_dirty: bool,
}
impl ScriptedDataConverter {
    fn number(v: &DataValue) -> Option<f32> {
        match v {
            DataValue::Number(v) => Some(*v),
            DataValue::Integer(v) => Some(*v as f32),
            _ => None,
        }
    }
    fn apply_conversion(&mut self, v: DataValue, m: &str) -> DataValue {
        if let Some(n) = Self::number(&v) {
            self.scripted
                .call_number(m, &[n])
                .map(DataValue::Number)
                .unwrap_or(v)
        } else {
            v
        }
    }
    pub fn convert(&mut self, v: DataValue) -> DataValue {
        let out = self.apply_conversion(v, "convert");
        self.data_value = Some(out.clone());
        out
    }
    pub fn reverse_convert(&mut self, v: DataValue) -> DataValue {
        let out = self.apply_conversion(v, "reverseConvert");
        self.data_value = Some(out.clone());
        out
    }
    pub fn bind_from_context(&mut self, c: Option<usize>) {
        self.data_context = c;
        self.scripted.set_data_context(c);
        self.converter_dirty = true
    }
    pub fn advance_component(&mut self, e: f32, animate: bool) -> bool {
        if !animate {
            return false;
        }
        self.advance(e)
    }
    pub fn advance(&mut self, e: f32) -> bool {
        self.advance_active = self.scripted.script_advance(e);
        self.advance_active
    }
    pub fn add_property(&mut self, p: usize) {
        self.properties.push(p)
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
}
