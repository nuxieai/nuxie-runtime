use crate::mechanical_port::source::data_bind::data_values::{
    data_type::DataType, data_value::DataValue, data_value_number::DataValueNumber,
    data_value_symbol_list_index::DataValueSymbolListIndex,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArithmeticOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    SquareRoot,
    Power,
    Exp,
    Log,
    Cosine,
    Sine,
    Tangent,
    Acosine,
    Asine,
    Atangent,
    Atangent2,
    Round,
    Floor,
    Ceil,
}
pub struct DataConverterOperation {
    operation: ArithmeticOperation,
    output: DataValueNumber,
    dirty: bool,
}
impl DataConverterOperation {
    pub fn new(operation: ArithmeticOperation) -> Self {
        Self {
            operation,
            output: DataValueNumber::default(),
            dirty: false,
        }
    }
    pub fn output_type(&self) -> DataType {
        DataType::Number
    }
    fn input_number(input: &dyn DataValue) -> Option<f32> {
        input
            .as_any()
            .downcast_ref::<DataValueNumber>()
            .map(DataValueNumber::value)
            .or_else(|| {
                input
                    .as_any()
                    .downcast_ref::<DataValueSymbolListIndex>()
                    .map(|value| value.value() as f32)
            })
    }
    pub fn convert_value(&mut self, input: &dyn DataValue, value: f32) -> &DataValueNumber {
        let result = Self::input_number(input).map_or(DataValueNumber::DEFAULT_VALUE, |input| {
            match self.operation {
                ArithmeticOperation::Add => input + value,
                ArithmeticOperation::Subtract => input - value,
                ArithmeticOperation::Multiply => input * value,
                ArithmeticOperation::Divide => input / value,
                ArithmeticOperation::Modulo => {
                    let range = value.abs();
                    let mut result = input % range;
                    if result < 0.0 {
                        result += range;
                    }
                    result
                }
                ArithmeticOperation::SquareRoot => input.sqrt(),
                ArithmeticOperation::Power => input.powf(value),
                ArithmeticOperation::Exp => input.exp(),
                ArithmeticOperation::Log => input.ln(),
                ArithmeticOperation::Cosine => input.cos(),
                ArithmeticOperation::Sine => input.sin(),
                ArithmeticOperation::Tangent => input.tan(),
                ArithmeticOperation::Acosine => input.acos(),
                ArithmeticOperation::Asine => input.asin(),
                ArithmeticOperation::Atangent => input.atan(),
                ArithmeticOperation::Atangent2 => input.atan2(value),
                ArithmeticOperation::Round => input.round(),
                ArithmeticOperation::Floor => input.floor(),
                ArithmeticOperation::Ceil => input.ceil(),
            }
        });
        self.output.set_value(result);
        &self.output
    }
    pub fn reverse_convert_value(&self, input: &dyn DataValue, value: f32) -> DataValueNumber {
        let mut output = DataValueNumber::default();
        if let Some(input) = input
            .as_any()
            .downcast_ref::<DataValueNumber>()
            .map(DataValueNumber::value)
        {
            let result = match self.operation {
                ArithmeticOperation::Add => input - value,
                ArithmeticOperation::Subtract => input + value,
                ArithmeticOperation::Multiply => input / value,
                ArithmeticOperation::Divide => input * value,
                ArithmeticOperation::Modulo => input,
                ArithmeticOperation::SquareRoot => input.powf(2.0),
                ArithmeticOperation::Power => input.powf(1.0 / value),
                ArithmeticOperation::Exp => input.ln(),
                ArithmeticOperation::Log => input.exp(),
                ArithmeticOperation::Cosine => input.acos(),
                ArithmeticOperation::Sine => input.asin(),
                ArithmeticOperation::Tangent => input.atan(),
                ArithmeticOperation::Acosine => input.cos(),
                ArithmeticOperation::Asine => input.sin(),
                ArithmeticOperation::Atangent => input.tan(),
                ArithmeticOperation::Atangent2
                | ArithmeticOperation::Round
                | ArithmeticOperation::Floor
                | ArithmeticOperation::Ceil => input,
            };
            output.set_value(result);
        }
        output
    }
    pub fn mark_converter_dirty(&mut self) {
        self.dirty = true
    }
}
