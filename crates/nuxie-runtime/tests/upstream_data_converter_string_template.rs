//! All 22 cases in upstream runtime/data_converter_string_template_test.cpp
//! at c63a5ba0. Native imports and bindings exercise the production runtime.
#[path = "support/riv_bytes.rs"]
mod riv_bytes;
use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::source::{
    core::{CoreArena, CoreHandle},
    custom_property_string::CustomPropertyString,
    data_bind::{
        converters::{
            data_converter_string_template::DataConverterStringTemplate,
            string_template::{
                string_template_argument_number::StringTemplateArgumentNumber,
                string_template_argument_string::StringTemplateArgumentString,
                string_template_format::{StringTemplateFormat, StringTemplateSegmentKind},
            },
        },
        data_bind_context::DataBindContext,
        data_values::{
            data_type::DataType, data_value_number::DataValueNumber,
            data_value_string::DataValueString,
        },
    },
    generated::{
        backboard_base::BackboardBase,
        core_registry::CoreRegistry,
        data_bind::{
            converters::{
                data_converter_rounder_base::DataConverterRounderBase,
                data_converter_string_template_base::DataConverterStringTemplateBase,
                string_template::{
                    string_template_argument_base::StringTemplateArgumentBase,
                    string_template_argument_number_base::StringTemplateArgumentNumberBase,
                    string_template_argument_string_base::StringTemplateArgumentStringBase,
                },
            },
            data_bind_base::DataBindBase,
        },
    },
};
use nuxie_runtime::{File, RuntimeFactoryHandle, RuntimeFileHandle};
use riv_bytes::{RivBytes, write_artboard_object};

fn parse_with(text: &str, open: &str, close: &str) -> String {
    let mut segments = Vec::new();
    StringTemplateFormat::parse(text, open, close, &mut segments);
    let mut result = String::new();
    for segment in segments {
        if segment.is_placeholder() {
            result.push_str(&match segment.kind {
                StringTemplateSegmentKind::Position => format!("[#{}", segment.index),
                StringTemplateSegmentKind::Next => format!("[+{}", segment.index),
                _ => format!("[{}", segment.name),
            });
            if !segment.spec.is_empty() {
                result.push(':');
                result.push_str(&segment.spec);
            }
            result.push(']');
        } else {
            result.push('<');
            result.push_str(&segment.text);
            result.push('>');
        }
    }
    result
}
fn parse(text: &str) -> String {
    parse_with(text, "${", "}")
}
fn number_argument(arena: &CoreArena, name: &str, value: f32) -> CoreHandle {
    arena.insert(StringTemplateArgumentNumber::new(name.into(), value))
}
fn string_argument(arena: &CoreArena, name: &str, value: &str) -> CoreHandle {
    arena.insert(StringTemplateArgumentString::new(name.into(), value.into()))
}
fn convert(converter: &mut DataConverterStringTemplate, text: &str) -> String {
    let input = DataValueString::new(text.into());
    converter
        .convert(&input)
        .as_any()
        .downcast_ref::<DataValueString>()
        .expect("string output")
        .value()
        .into()
}
#[test]
fn string_template_parses_placeholders() {
    for (input, expected) in [
        (
            "You got ${numerical_score} right",
            "<You got >[numerical_score]< right>",
        ),
        ("${a:%.1f}", "[a:%.1f]"),
        ("${a}${b}", "[a][b]"),
        ("${_a1:x}", "[_a1:x]"),
        ("${a:}", "[a]"),
        ("", ""),
    ] {
        assert_eq!(parse(input), expected, "{input}");
    }
}
#[test]
fn string_template_placeholders_by_index_and_by_order() {
    for (input, expected) in [
        ("${1} scored ${0:%.1f}", "[#1]< scored >[#0:%.1f]"),
        ("${} and ${}", "[+0]< and >[+1]"),
        ("${:%d}", "[+0:%d]"),
        ("${01}", "[#1]"),
        ("${} ${name} ${5} ${}", "[+0]< >[name]< >[#5]< >[+1]"),
        ("${99999999999}", "[#999999999]"),
        ("${1a}", "<${1a}>"),
        ("${-1}", "<${-1}>"),
    ] {
        assert_eq!(parse(input), expected, "{input}");
    }
    assert_eq!(parse_with("50%% off", "%", "%"), "<50>[+0]< off>");
    assert_eq!(parse_with(r"50\%% off", "%", "%"), "<50%% off>");
}
#[test]
fn string_template_keeps_what_isnt_a_placeholder_as_text() {
    for text in [
        "${1a}", "${a b}", "${ }", "${a", "${a:%d", "$a {b}", "${", "$", "costs $5",
    ] {
        assert_eq!(parse(text), format!("<{text}>"));
    }
    assert_eq!(parse("¡Hola ${nombre}! ✓"), "<¡Hola >[nombre]<! ✓>");
}
#[test]
fn string_template_escapes() {
    for (input, expected) in [
        (r"\${a}", "<${a}>"),
        (r"\\${a}", r"<\>[a]"),
        (r"C:\path\", r"<C:\path\>"),
        (r"\n", r"<\n>"),
        (r"\$5", r"<\$5>"),
    ] {
        assert_eq!(parse(input), expected);
    }
}
#[test]
fn string_template_with_other_delimiters() {
    for (text, open, close, expected) in [
        (
            "You got {score} right",
            "{",
            "}",
            "<You got >[score]< right>",
        ),
        ("{a:%.1f}", "{", "}", "[a:%.1f]"),
        (r"\{a}", "{", "}", "<{a}>"),
        ("{{a}}", "{", "}", "<{>[a]<}>"),
        ("{ a } {1}", "{", "}", "<{ a } >[#1]"),
        ("${a}", "{", "}", "<$>[a]"),
        ("{{a}} {{b:%d}}", "{{", "}}", "[a]< >[b:%d]"),
        ("{a}", "{{", "}}", "<{a}>"),
        ("{{{a}}}", "{{", "}}", "<{>[a]<}>"),
        (r"\{{a}}", "{{", "}}", "<{{a}}>"),
        ("%a% of %b:.1f%", "%", "%", "[a]< of >[b:.1f]"),
        ("100% sure", "%", "%", "<100% sure>"),
        (r"\%a%", "%", "%", "<%a%>"),
        ("%a:%.1f%", "%", "%", "[a]<.1f%>"),
        ("<a> & <b>", "<", ">", "[a]< & >[b]"),
        ("[[a]]", "[[", "]]", "[a]"),
        ("@@a@@", "@@", "@@", "[a]"),
    ] {
        assert_eq!(parse_with(text, open, close), expected, "{text}");
    }
}
#[test]
fn string_template_delimiters_must_be_ascii_symbols() {
    for d in ["${", "{", "}}", "%", "<", "[[", "@@", "~~~~", "!", "`", "$"] {
        assert!(StringTemplateFormat::is_valid_delimiter(d), "{d}");
    }
    for d in [
        "", "a", "A", "1", "_", ":", "\\", " ", "\t", "{{{{{", "$a", "«",
    ] {
        assert!(!StringTemplateFormat::is_valid_delimiter(d), "{d}");
        assert_eq!(parse_with("${a} {a}", d, "}"), "<${a} {a}>");
        assert_eq!(parse_with("${a} {a}", "${", d), "<${a} {a}>");
    }
    assert_eq!(parse_with("", "", ""), "");
}
#[test]
fn string_template_parses_long_malformed_input() {
    let text = "${a:".repeat(200000);
    assert_eq!(parse(&text), format!("<{text}>"));
    assert_eq!(
        parse(&(text.clone() + "}${b}")),
        format!("[a:{}][b]", &text[4..])
    );
}
#[test]
fn string_template_default_number_format() {
    for (value, expected) in [
        (7.0, "7"),
        (0.5, "0.5"),
        (-12.25, "-12.25"),
        (1.0 / 3.0, "0.333333"),
        (0.0, "0"),
        (-0.0, "0"),
        (-0.0000001, "0"),
        (1234567.0, "1234567"),
    ] {
        assert_eq!(StringTemplateFormat::format_number(value, ""), expected);
    }
}
#[test]
fn string_template_printf_number_specs() {
    for (value, spec, expected) in [
        (3.14159, "%.2f", "3.14"),
        (3.14159, ".2f", "3.14"),
        (3.0, "%.0f", "3"),
        (3.5, "%6.2f", "  3.50"),
        (3.5, "%-6.1f", "3.5   "),
        (3.5, "%+.1f", "+3.5"),
        (3.5, "% .1f", " 3.5"),
        (3.5, "%07.2f", "0003.50"),
        (2.0, "%#.0f", "2."),
        (1500000.0, "%e", "1.500000e+06"),
        (1500000.0, "%.2E", "1.50E+06"),
        (0.0001, "%g", "0.0001"),
        (2.5, "%F", "2.500000"),
        (2.5, "%d", "3"),
        (2.4, "%i", "2"),
        (-2.5, "%d", "-3"),
        (42.0, "%05d", "00042"),
        (42.0, "%-5d", "42   "),
        (42.0, "%+d", "+42"),
        (7.0, "%.3d", "007"),
        (255.0, "%x", "ff"),
        (255.0, "%X", "FF"),
        (255.0, "%#x", "0xff"),
        (255.0, "%o", "377"),
        (255.0, "%u", "255"),
        (-5.0, "%u", "0"),
        (-5.0, "%x", "0"),
        (1.0e30, "%d", "9000000000000000000"),
        (-1.0e30, "%d", "-9000000000000000000"),
    ] {
        assert_eq!(
            StringTemplateFormat::format_number(value, spec),
            expected,
            "{value} {spec}"
        );
    }
}
#[test]
fn string_template_thousands_separators() {
    for (value, spec, expected) in [
        (1234567.0, "%,d", "1,234,567"),
        (-1234.0, "%,d", "-1,234"),
        (999.0, "%,d", "999"),
        (1234567.5, "%,.1f", "1,234,567.5"),
        (1234.0, "%,+d", "+1,234"),
        (1234.0, "%,8d", "   1,234"),
        (1234.0, "%-,8d", "1,234   "),
        (1234.0, "%,3d", "1,234"),
    ] {
        assert_eq!(
            StringTemplateFormat::format_number(value, spec),
            expected,
            "{value} {spec}"
        );
    }
}
#[test]
fn string_template_invalid_specs_use_the_default_format() {
    for spec in [
        "%s", "%n", "%p", "%c", "%", "%5", "%dd", "%ld", "%lld", "%*d", "%100d", "%.100f", "%#d",
        "%#u", "%,x", "%,e", "%0,d", "abc", "%%",
    ] {
        assert!(!StringTemplateFormat::is_valid_number_spec(spec), "{spec}");
        assert_eq!(StringTemplateFormat::format_number(7.5, spec), "7.5");
    }
    assert!(StringTemplateFormat::is_valid_number_spec("%99.99f"));
    assert!(StringTemplateFormat::is_valid_number_spec("d"));
}
#[test]
fn string_template_non_finite_numbers() {
    for (value, spec, expected) in [
        (f32::NAN, "", "nan"),
        (-f32::NAN, "%d", "nan"),
        (f32::INFINITY, "%.1f", "inf"),
        (-f32::INFINITY, "%,d", "-inf"),
    ] {
        assert_eq!(StringTemplateFormat::format_number(value, spec), expected);
    }
}

#[test]
fn string_template_converter_replaces_placeholders() {
    let arena = CoreArena::default();
    let mut converter = DataConverterStringTemplate::default();
    converter.add_argument(string_argument(&arena, "player", "Ana"));
    converter.add_argument(number_argument(&arena, "score", 6.6));
    converter.add_argument(number_argument(&arena, "total", 10.0));
    assert_eq!(converter.output_type(), DataType::String);
    assert_eq!(
        convert(
            &mut converter,
            "${player} got ${score:%d} of ${total} right"
        ),
        "Ana got 7 of 10 right"
    );
    assert_eq!(convert(&mut converter, "${player:%5d}!"), "Ana!");
    assert_eq!(
        convert(&mut converter, "[${missing:%.1f}] and ${score}"),
        "[] and 6.6"
    );
    converter.arguments()[1]
        .with_downcast_mut::<StringTemplateArgumentNumber, _>(|arg| arg.set_property_value(12.0))
        .unwrap();
    assert_eq!(convert(&mut converter, "${score}"), "12");
    assert_eq!(convert(&mut converter, ""), "");
}
#[test]
fn string_template_converter_matches_by_index_order_and_name() {
    let arena = CoreArena::default();
    let mut converter = DataConverterStringTemplate::default();
    converter.add_argument(number_argument(&arena, "score", 7.25));
    converter.add_argument(string_argument(&arena, "player", "Ana"));
    for (input, expected) in [
        ("${1} scored ${0:%.1f}", "Ana scored 7.2"),
        ("${0}${0}", "7.257.25"),
        ("${} and ${}", "7.25 and Ana"),
        ("${}|${}|${}|${5}", "7.25|Ana||"),
        ("${player}: ${:%d} (${1})", "Ana: 7 (Ana)"),
    ] {
        assert_eq!(convert(&mut converter, input), expected);
    }
}
#[test]
fn string_template_converter_only_takes_string_input() {
    let arena = CoreArena::default();
    let mut converter = DataConverterStringTemplate::default();
    converter.add_argument(number_argument(&arena, "a", 1.0));
    let number = DataValueNumber::new(3.0);
    let output = converter.convert(&number);
    assert_eq!(
        output
            .as_any()
            .downcast_ref::<DataValueString>()
            .expect("string output")
            .value(),
        ""
    );
    assert_eq!(convert(&mut converter, "${a}"), "1");
}
#[test]
fn string_template_converter_follows_its_delimiters() {
    let arena = CoreArena::default();
    let mut converter = DataConverterStringTemplate::default();
    assert_eq!(converter.base.open_delimiter(), "${");
    assert_eq!(converter.base.close_delimiter(), "}");
    converter.add_argument(number_argument(&arena, "a", 1.0));
    assert_eq!(convert(&mut converter, "{a} ${a}"), "{a} 1");
    converter.set_open_delimiter("{".into());
    assert_eq!(convert(&mut converter, "{a} ${a}"), "1 $1");
    converter.set_open_delimiter("%".into());
    converter.set_close_delimiter("%".into());
    assert_eq!(convert(&mut converter, "{a} ${a}"), "{a} ${a}");
    assert_eq!(convert(&mut converter, "%a% %0:.1f%"), "1 1.0");
    converter.set_close_delimiter("".into());
    assert_eq!(convert(&mut converter, "%a% %0:.1f%"), "%a% %0:.1f%");
}
#[test]
fn string_template_converter_uses_the_first_argument_with_a_name() {
    let arena = CoreArena::default();
    let mut converter = DataConverterStringTemplate::default();
    converter.add_argument(number_argument(&arena, "a", 1.0));
    converter.add_argument(number_argument(&arena, "a", 2.0));
    assert_eq!(convert(&mut converter, "${a}"), "1");
}

fn argument_handles(converter: &CoreHandle) -> Vec<CoreHandle> {
    converter
        .with_downcast::<DataConverterStringTemplate, _>(|c| c.arguments().to_vec())
        .unwrap()
}
fn convert_handle(converter: &CoreHandle, text: &str) -> String {
    converter
        .with_downcast_mut::<DataConverterStringTemplate, _>(|c| convert(c, text))
        .unwrap()
}
fn bound_to(arena: &CoreArena, target: &CoreHandle, property: u16) -> CoreHandle {
    let mut bind = DataBindContext::default();
    CoreRegistry::set_uint(
        &mut bind,
        i32::from(DataBindBase::PROPERTY_KEY_PROPERTY_KEY),
        u32::from(property),
    );
    let bind = arena.insert(bind);
    bind.with_mut(|b| {
        b.as_data_bind_mut()
            .unwrap()
            .set_target(Some(target.clone()))
    })
    .unwrap();
    bind
}
#[test]
fn string_template_clone_retargets_argument_binds() {
    let arena = CoreArena::default();
    let mut converter = DataConverterStringTemplate::default();
    converter.set_open_delimiter("<<".into());
    converter.set_close_delimiter(">>".into());
    converter.add_argument(number_argument(&arena, "a", 1.0));
    converter.add_argument(string_argument(&arena, "b", "x"));
    let converter = arena.insert(converter);
    let arguments = argument_handles(&converter);
    let converter_bind = bound_to(
        &arena,
        &converter,
        DataConverterStringTemplateBase::OPEN_DELIMITER_PROPERTY_KEY,
    );
    converter
        .data_bind_container()
        .unwrap()
        .add_data_bind(converter_bind);
    let argument_bind = bound_to(
        &arena,
        &arguments[1],
        StringTemplateArgumentStringBase::PROPERTY_VALUE_PROPERTY_KEY,
    );
    converter
        .data_bind_container()
        .unwrap()
        .add_data_bind(argument_bind);
    let cloned = converter.clone_occurrence().unwrap();
    let cloned_arguments = argument_handles(&cloned);
    assert_eq!(cloned_arguments.len(), 2);
    assert_ne!(cloned_arguments[0], arguments[0]);
    assert_eq!(
        cloned_arguments[0]
            .with_downcast::<StringTemplateArgumentNumber, _>(|a| a.name().to_owned())
            .unwrap(),
        "a"
    );
    cloned
        .with_downcast::<DataConverterStringTemplate, _>(|c| {
            assert_eq!(c.base.open_delimiter(), "<<");
            assert_eq!(c.base.close_delimiter(), ">>");
        })
        .unwrap();
    assert_eq!(
        cloned_arguments[1]
            .with_downcast::<StringTemplateArgumentString, _>(|a| a.name().to_owned())
            .unwrap(),
        "b"
    );
    let binds = cloned.data_bind_container().unwrap().data_binds().to_vec();
    assert_eq!(binds.len(), 2);
    assert_eq!(
        binds[0]
            .with(|b| b.as_data_bind().unwrap().target())
            .unwrap(),
        Some(cloned.clone())
    );
    assert_eq!(
        binds[1]
            .with(|b| b.as_data_bind().unwrap().target())
            .unwrap(),
        Some(cloned_arguments[1].clone())
    );
    assert_eq!(convert_handle(&cloned, "<<a>> <<b>>"), "1 x");
}

fn import(bytes: &[u8]) -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(NullFactory);
    File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("successful native import")
}
fn converters(file: &RuntimeFileHandle) -> Vec<CoreHandle> {
    file.with_file(|f| f.testing_data_converters().to_vec())
}
#[test]
fn string_template_and_its_arguments_import() {
    let mut riv = RivBytes::default();
    riv.object(BackboardBase::TYPE_KEY);
    riv.end();
    riv.object(DataConverterStringTemplateBase::TYPE_KEY);
    riv.end();
    riv.object(StringTemplateArgumentNumberBase::TYPE_KEY);
    riv.prop_string(StringTemplateArgumentBase::NAME_PROPERTY_KEY, "score");
    riv.prop_float(
        StringTemplateArgumentNumberBase::PROPERTY_VALUE_PROPERTY_KEY,
        7.25,
    );
    riv.end();
    riv.object(DataBindBase::TYPE_KEY);
    riv.prop_uint(
        DataBindBase::PROPERTY_KEY_PROPERTY_KEY,
        u64::from(StringTemplateArgumentNumberBase::PROPERTY_VALUE_PROPERTY_KEY),
    );
    riv.end();
    riv.object(StringTemplateArgumentStringBase::TYPE_KEY);
    riv.prop_string(StringTemplateArgumentBase::NAME_PROPERTY_KEY, "player");
    riv.prop_string(
        StringTemplateArgumentStringBase::PROPERTY_VALUE_PROPERTY_KEY,
        "Ana",
    );
    riv.end();
    write_artboard_object(&mut riv);
    let file = import(&riv.bytes());
    let converters = converters(&file);
    assert_eq!(converters.len(), 1);
    assert!(converters[0].is_type_of(DataConverterStringTemplateBase::TYPE_KEY));
    let converter = &converters[0];
    let args = argument_handles(converter);
    assert_eq!(args.len(), 2);
    let binds = converter
        .data_bind_container()
        .unwrap()
        .data_binds()
        .to_vec();
    assert_eq!(binds.len(), 1);
    assert_eq!(
        binds[0]
            .with(|b| b.as_data_bind().unwrap().target())
            .unwrap(),
        Some(args[0].clone())
    );
    assert_eq!(
        convert_handle(converter, "${player} got ${score:%.1f}"),
        "Ana got 7.2"
    );
}
#[test]
fn string_template_delimiters_import() {
    let mut riv = RivBytes::default();
    riv.object(BackboardBase::TYPE_KEY);
    riv.end();
    riv.object(DataConverterStringTemplateBase::TYPE_KEY);
    riv.prop_string(
        DataConverterStringTemplateBase::OPEN_DELIMITER_PROPERTY_KEY,
        "%",
    );
    riv.prop_string(
        DataConverterStringTemplateBase::CLOSE_DELIMITER_PROPERTY_KEY,
        "%",
    );
    riv.end();
    riv.object(StringTemplateArgumentNumberBase::TYPE_KEY);
    riv.prop_string(StringTemplateArgumentBase::NAME_PROPERTY_KEY, "score");
    riv.prop_float(
        StringTemplateArgumentNumberBase::PROPERTY_VALUE_PROPERTY_KEY,
        7.5,
    );
    riv.end();
    write_artboard_object(&mut riv);
    let file = import(&riv.bytes());
    let converters = converters(&file);
    assert_eq!(converters.len(), 1);
    assert_eq!(
        convert_handle(&converters[0], "%score:05.1f% and ${score}"),
        "007.5 and ${score}"
    );
}
#[test]
fn string_template_arguments_without_a_template_are_skipped() {
    let mut riv = RivBytes::default();
    riv.object(BackboardBase::TYPE_KEY);
    riv.end();
    riv.object(StringTemplateArgumentNumberBase::TYPE_KEY);
    riv.prop_string(StringTemplateArgumentBase::NAME_PROPERTY_KEY, "a");
    riv.end();
    riv.object(DataConverterRounderBase::TYPE_KEY);
    riv.end();
    riv.object(StringTemplateArgumentStringBase::TYPE_KEY);
    riv.prop_string(StringTemplateArgumentBase::NAME_PROPERTY_KEY, "b");
    riv.end();
    write_artboard_object(&mut riv);
    let file = import(&riv.bytes());
    let artboard = file.with_file(File::artboard_default).expect("artboard");
    assert_eq!(artboard.with_artboard(|a| a.name().to_owned()), "A");
    let converters = converters(&file);
    assert_eq!(converters.len(), 1);
    assert!(converters[0].is_type_of(DataConverterRounderBase::TYPE_KEY));
}
#[test]
fn string_template_arguments_follow_their_view_model_properties() {
    let file = import(include_bytes!("../../../fixtures/sync/string_template.riv"));
    let artboard = file
        .with_file(File::artboard_default)
        .expect("artboard instance");
    let view_model = file.with_file(|f| f.view_model_by_name("VM")).expect("VM");
    let instance = view_model.create_default_instance();
    artboard.bind_view_model_instance(Some(instance.instance()));
    artboard.advance_default(0.0);
    let result = artboard
        .with_artboard(|a| a.find_handle::<CustomPropertyString>("result"))
        .expect("result");
    let value = || {
        result
            .with_downcast::<CustomPropertyString, _>(|s| s.property_value().to_owned())
            .unwrap()
    };
    assert_eq!(value(), "Ana got 7 right");
    let score = instance
        .property_number("numerical_score")
        .expect("numerical_score");
    let player = instance.property_string("player").expect("player");
    score.set_value(8.6);
    artboard.advance_default(0.0);
    assert_eq!(value(), "Ana got 9 right");
    player.set_value("Bo");
    artboard.advance_default(0.0);
    assert_eq!(value(), "Bo got 9 right");
    let message = instance.property_string("message").expect("message");
    message.set_value("{1} has {} points");
    artboard.advance_default(0.0);
    assert_eq!(value(), "Bo has 8.6 points");
}
