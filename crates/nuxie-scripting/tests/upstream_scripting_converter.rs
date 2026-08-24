//! Direct ports of the six inline cases in pinned
//! `tests/unit_tests/runtime/scripting/scripting_converter_test.cpp`.
#![cfg(feature = "luau")]

use luaur_rt::{Function, Table};
use nuxie_runtime::{ScriptDataConverterMethod, ScriptInstance, ScriptValue};
use nuxie_scripting::vm::{LuaScriptInstance, ScriptVm};

mod support;
use support::ScriptVmSourceTestExt as _;

fn converter(source: &str) -> LuaScriptInstance {
    let vm = ScriptVm::new();
    vm.install_rive_globals().expect("install Rive globals");
    let factory: Function = vm.eval(source).expect("evaluate exact converter source");
    let table: Table = factory.call(()).expect("construct converter table");
    LuaScriptInstance::new(table)
}

#[test]
fn scripted_string_converter_support_data_types() {
    let mut converter = converter(
        r#"type InputTypes = DataValueString | DataValueBoolean | DataValueNumber

type StringConverter = {}
function convert(self: StringConverter, input: InputTypes): DataValueString
  local inputString: string = ''
  local dv: DataValueString = DataValue.string()
  if input:isString() then
    inputString = (input :: DataValueString).value
    dv.value = (input :: DataValueString).value .. ' - suffix'
  elseif input:isBoolean() then
    if (input :: DataValueBoolean).value then
      inputString = 'True'
    else
      inputString = 'False'
    end
  elseif input:isNumber() then
    inputString = tostring((input :: DataValueNumber).value)
  end

  dv.value = inputString .. ' - suffix'
  return dv
end

function reverseConvert(
  self: StringConverter,
  input: InputTypes
): DataValueString
  local dv: DataValueString = DataValue.string()
  if input:isString() then
    dv.value = (input :: DataValueString).value
  end
  return dv
end

function init(self: StringConverter): boolean
  return true
end

return function(): Converter<StringConverter, InputTypes, DataValueString>
  return { convert = convert, reverseConvert = reverseConvert, init = init }
end

"#,
    );

    assert_eq!(
        converter
            .call_data_converter(
                ScriptDataConverterMethod::Convert,
                ScriptValue::String("input".to_owned()),
            )
            .expect("convert string"),
        ScriptValue::String("input - suffix".to_owned())
    );
    assert_eq!(
        converter
            .call_data_converter(ScriptDataConverterMethod::Convert, ScriptValue::Bool(true),)
            .expect("convert boolean"),
        ScriptValue::String("True - suffix".to_owned())
    );
    assert_eq!(
        converter
            .call_data_converter(ScriptDataConverterMethod::Convert, ScriptValue::Number(1.0),)
            .expect("convert number"),
        ScriptValue::String("1 - suffix".to_owned())
    );
    assert_eq!(
        converter
            .call_data_converter(
                ScriptDataConverterMethod::ReverseConvert,
                ScriptValue::String("input as output".to_owned()),
            )
            .expect("reverse-convert string"),
        ScriptValue::String("input as output".to_owned())
    );
}

#[test]
fn scripted_number_converter_support_data_types() {
    let mut converter = converter(
        r#"type NumberConverter = {}
function convert(self: NumberConverter, input: DataValueNumber): DataValueNumber
  local dv: DataValueNumber = DataValue.number()
  if input:isNumber() then
    dv.value = (input :: DataValueNumber).value + 250
  end
  return dv
end

function reverseConvert(
  self: NumberConverter,
  input: DataValueNumber
): DataValueNumber
  local dv: DataValueNumber = DataValue.number()
  return dv
end

function init(self: NumberConverter): boolean
  return true
end

return function(): Converter<NumberConverter, DataValueNumber, DataValueNumber>
  return { convert = convert, reverseConvert = reverseConvert, init = init }
end

"#,
    );
    assert_eq!(
        converter
            .call_data_converter(ScriptDataConverterMethod::Convert, ScriptValue::Number(1.0),)
            .expect("convert number"),
        ScriptValue::Number(251.0)
    );
}

#[test]
fn scripted_boolean_converts_bool() {
    let mut converter = converter(
        r#"type BoolConverter = {
  coin: Input<Artboard<Data.Coin>>,
}

function convert(self: BoolConverter, input: DataValueBoolean): DataValueBoolean
  local dv: DataValueBoolean = DataValue.boolean()
  dv.value = not input.value
  return dv
end

function reverseConvert(
  self: BoolConverter,
  input: DataValueBoolean
): DataValueBoolean
  local dv: DataValueBoolean = DataValue.boolean()
  return dv
end

function init(self: BoolConverter): boolean
  return true
end

return function(): Converter<BoolConverter, DataValueBoolean, DataValueBoolean>
  return {
    convert = convert,
    coin = late(),
    reverseConvert = reverseConvert,
    init = init,
  }
end

"#,
    );
    assert_eq!(
        converter
            .call_data_converter(ScriptDataConverterMethod::Convert, ScriptValue::Bool(true),)
            .expect("convert true"),
        ScriptValue::Bool(false)
    );
    assert_eq!(
        converter
            .call_data_converter(ScriptDataConverterMethod::Convert, ScriptValue::Bool(false),)
            .expect("convert false"),
        ScriptValue::Bool(true)
    );
}

#[test]
fn scripted_color_converts_color_value() {
    let mut converter = converter(
        r#"type ColorConverter = {}
function convert(self: ColorConverter, input: DataValueColor): DataValueColor
  local dv: DataValueColor = DataValue.color()
  dv.value = input.value
  dv.red = 0
  dv.blue = 255
  return dv
end
function reverseConvert(
  self: ColorConverter,
  input: DataValueColor
): DataValueColor
  local dv: DataValueColor = DataValue.color()
  dv.value = input.value
  return dv
end
function init(self: ColorConverter): boolean
  return true
end
return function(): Converter<ColorConverter, DataValueColor, DataValueColor>
  return { convert = convert, reverseConvert = reverseConvert, init = init }
end
"#,
    );
    assert_eq!(
        converter
            .call_data_converter(
                ScriptDataConverterMethod::Convert,
                ScriptValue::Color(0xffff_ff00),
            )
            .expect("convert first color"),
        ScriptValue::Color(0xff00_ffff)
    );
    assert_eq!(
        converter
            .call_data_converter(ScriptDataConverterMethod::Convert, ScriptValue::Color(0),)
            .expect("convert zero color"),
        ScriptValue::Color(0x0000_00ff)
    );
}

#[test]
fn another_scripted_color_converter() {
    let mut converter = converter(
        r#"type ColorConverter = {}
function convert(self: ColorConverter, input: DataValueColor): DataValueColor
  local dv: DataValueColor = DataValue.color()
  if input:isColor() then
    dv.alpha = input.red
    dv.red = input.green
    dv.green = input.blue
    dv.blue = input.alpha
  end
  return dv
end
function reverseConvert(
  self: ColorConverter,
  input: DataValueColor
): DataValueColor
  local dv: DataValueColor = DataValue.color()
  dv.value = input.value
  return dv
end
function init(self: ColorConverter): boolean
  return true
end
return function(): Converter<ColorConverter, DataValueColor, DataValueColor>
  return { convert = convert, reverseConvert = reverseConvert, init = init }
end
"#,
    );
    assert_eq!(
        converter
            .call_data_converter(
                ScriptDataConverterMethod::Convert,
                ScriptValue::Color(0x1122_3344),
            )
            .expect("rotate color channels"),
        ScriptValue::Color(0x2233_4411)
    );
    assert_eq!(
        converter
            .call_data_converter(ScriptDataConverterMethod::Convert, ScriptValue::Bool(true),)
            .expect("convert non-color input"),
        ScriptValue::Color(0)
    );
}

#[test]
fn scripted_converter_survives_number_string_direction_flips() {
    let mut converter = converter(
        r#"type NumberToStringConverter = {}

function convert(self: NumberToStringConverter, input: DataValueNumber): DataValueString
  local result = DataValue.string()
  result.value = tostring(math.round(input.value))
  return result
end

function reverseConvert(self: NumberToStringConverter, input: DataValueString): DataValueNumber
  local result = DataValue.number()
  result.value = tonumber(input.value) or 0
  return result
end

return function(): Converter<NumberToStringConverter, DataValueNumber, DataValueString>
  return {
    convert = convert,
    reverseConvert = reverseConvert,
  }
end
"#,
    );

    assert!(
        converter
            .has_data_converter_method(ScriptDataConverterMethod::Convert)
            .expect("query convert")
    );
    assert!(
        converter
            .has_data_converter_method(ScriptDataConverterMethod::ReverseConvert)
            .expect("query reverseConvert")
    );

    for index in 0..3 {
        assert_eq!(
            converter
                .call_data_converter(
                    ScriptDataConverterMethod::Convert,
                    ScriptValue::Number((20_000 + index) as f64),
                )
                .expect("forward conversion"),
            ScriptValue::String((20_000 + index).to_string())
        );
        assert_eq!(
            converter
                .call_data_converter(
                    ScriptDataConverterMethod::ReverseConvert,
                    ScriptValue::String((12_345 + index).to_string()),
                )
                .expect("reverse conversion"),
            ScriptValue::Number((12_345 + index) as f64)
        );
    }
}
