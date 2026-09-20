use nuxie_runtime::source::data_bind::{
    converters::{
        data_converter_operation::ArithmeticOperation,
        data_converter_operation_value::DataConverterOperationValue,
        data_converter_system_degs_to_rads::DataConverterSystemDegsToRads,
        data_converter_system_normalizer::DataConverterSystemNormalizer,
        data_converter_trigger::DataConverterTrigger,
    },
    data_values::{
        data_value::DataValue, data_value_integer::DataValueInteger,
        data_value_number::DataValueNumber, data_value_trigger::DataValueTrigger,
    },
};

// C++ data_converter_trigger.cpp uses is<DataValueInteger>(), including derived
// Trigger values, and performs unsigned 32-bit wrapping addition.
#[test]
fn trigger_accepts_integer_subtypes_and_wraps() {
    let mut converter = DataConverterTrigger::default();
    for value in [0, 41, u32::MAX] {
        for input in [
            Box::new(DataValueInteger::new(value)) as Box<dyn DataValue>,
            Box::new(DataValueTrigger::new(value)),
        ] {
            let output = converter.convert(input.as_ref());
            assert_eq!(
                output
                    .as_any()
                    .downcast_ref::<DataValueTrigger>()
                    .unwrap()
                    .value(),
                value.wrapping_add(1)
            );
        }
    }
    assert_eq!(
        converter
            .convert(&DataValueNumber::new(41.0))
            .as_any()
            .downcast_ref::<DataValueTrigger>()
            .unwrap()
            .value(),
        0
    );
}

fn number(value: &dyn DataValue) -> f32 {
    value
        .as_any()
        .downcast_ref::<DataValueNumber>()
        .unwrap()
        .value()
}

// Both upstream system converters mask Direction (bit 0). TwoWay, Once,
// precedence, and name-based flags do not change which operation is applied.
#[test]
fn system_adapters_use_direction_in_both_conversion_paths() {
    let mut degrees = DataConverterSystemDegsToRads::default();
    degrees.base.base = DataConverterOperationValue::new(ArithmeticOperation::Multiply, 2.0);
    let mut normalizer = DataConverterSystemNormalizer::default();
    normalizer.base.base = DataConverterOperationValue::new(ArithmeticOperation::Multiply, 2.0);
    let input = DataValueNumber::new(10.0);
    for flags in 0..32 {
        let expected = if flags & 1 == 0 { 20.0 } else { 5.0 };
        assert_eq!(
            number(degrees.convert(&input, flags).as_ref()),
            expected,
            "degrees forward {flags}"
        );
        assert_eq!(
            number(degrees.reverse_convert(&input, flags).as_ref()),
            expected,
            "degrees reverse {flags}"
        );
        assert_eq!(
            number(normalizer.convert(&input, flags).as_ref()),
            expected,
            "normalizer forward {flags}"
        );
        assert_eq!(
            number(normalizer.reverse_convert(&input, flags).as_ref()),
            expected,
            "normalizer reverse {flags}"
        );
    }
}
