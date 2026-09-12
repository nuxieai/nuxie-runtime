//! Pure authoring color tests; importing this module does not render or mutate a runtime.
use nuxie_html_to_riv::Diagnostic;
#[path = "../src/css_whitespace.rs"]
mod css_whitespace;
#[path = "../src/color.rs"]
mod color;

#[test]
fn only_css_whitespace_separates_color_components_or_pads_values() {
    for separator in ['\t', '\n', '\u{c}', '\r', ' '] {
        for text in [format!("{separator}red{separator}"),
            format!("rgb(255{separator}0{separator}0)"),
            format!("rgb({separator}255,0,0)"),
            format!("hsl(0,100%,50%,{separator}1{separator})")] {
            assert_eq!(color::parse(&text, "test").unwrap(), 0xffff0000, "{text:?}");
        }
    }
    for separator in ['\u{b}', '\u{85}', '\u{a0}', '\u{1680}', '\u{2003}',
        '\u{2028}', '\u{2029}', '\u{202f}', '\u{205f}', '\u{3000}', '\u{feff}'] {
        for text in [format!("{separator}red"), format!("red{separator}"),
            format!("#f00{separator}"), format!("rgb({separator}255,0,0)"),
            format!("rgb(255{separator}0{separator}0)"),
            format!("rgb(255 0 0 / {separator}1)"),
            format!("hsl({separator}0,100%,50%)"),
            format!("hsl(0,100%,50%,1{separator})")] {
            assert!(color::parse(&text, "test").is_err(), "{text:?}");
        }
    }
}

#[test]
fn named_colors_hex_alpha_and_case_are_preserved() {
    for (input, expected) in [
        ("aliceblue", 0xfff0f8ff),
        ("rebeccapurple", 0xff663399),
        ("darkslategray", 0xff2f4f4f),
        ("darkslategrey", 0xff2f4f4f),
        ("papayawhip", 0xffffefd5),
        ("lightgoldenrodyellow", 0xfffafad2),
        ("mediumaquamarine", 0xff66cdaa),
        ("  BlAnChEdAlMoNd ", 0xffffebcd),
        ("red", 0xffff0000),
        ("lime", 0xff00ff00),
        ("blue", 0xff0000ff),
        ("transparent", 0),
        ("#abc", 0xffaabbcc),
        ("#abcd", 0xddaabbcc),
        ("#123456", 0xff123456),
        ("#12345678", 0x78123456),
        ("#ff000000", 0x00ff0000),
    ] {
        assert_eq!(color::parse(input, "color").unwrap(), expected, "{input}");
    }
    // Complete CSS named-color vocabulary; numeric expectations above remain
    // independent controls for byte ordering and alias behavior.
    let names="aliceblue antiquewhite aqua aquamarine azure beige bisque black blanchedalmond blue blueviolet brown burlywood cadetblue chartreuse chocolate coral cornflowerblue cornsilk crimson cyan darkblue darkcyan darkgoldenrod darkgray darkgreen darkgrey darkkhaki darkmagenta darkolivegreen darkorange darkorchid darkred darksalmon darkseagreen darkslateblue darkslategray darkslategrey darkturquoise darkviolet deeppink deepskyblue dimgray dimgrey dodgerblue firebrick floralwhite forestgreen fuchsia gainsboro ghostwhite gold goldenrod gray green greenyellow grey honeydew hotpink indianred indigo ivory khaki lavender lavenderblush lawngreen lemonchiffon lightblue lightcoral lightcyan lightgoldenrodyellow lightgray lightgreen lightgrey lightpink lightsalmon lightseagreen lightskyblue lightslategray lightslategrey lightsteelblue lightyellow lime limegreen linen magenta maroon mediumaquamarine mediumblue mediumorchid mediumpurple mediumseagreen mediumslateblue mediumspringgreen mediumturquoise mediumvioletred midnightblue mintcream mistyrose moccasin navajowhite navy oldlace olive olivedrab orange orangered orchid palegoldenrod palegreen paleturquoise palevioletred papayawhip peachpuff peru pink plum powderblue purple rebeccapurple red rosybrown royalblue saddlebrown salmon sandybrown seagreen seashell sienna silver skyblue slateblue slategray slategrey snow springgreen steelblue tan teal thistle tomato turquoise violet wheat white whitesmoke yellow yellowgreen";
    assert_eq!(names.split_whitespace().count(), 148);
    for name in names.split_whitespace() {
        assert!(color::parse(name, "named").is_ok(), "{name}");
    }
}

#[test]
fn rgb_supports_legacy_and_modern_channels_clamps_and_quantizes() {
    for (input, expected) in [
        ("rgb(255, 0, 128)", 0xffff0080),
        ("rgba(100%, 0%, 50%, 50%)", 0x80ff0080),
        ("rgb(255 0% 128 / .25)", 0x40ff0080),
        ("rgba(255 0 0)", 0xffff0000),
        ("rgb(-20 999 0 / 2)", 0xff00ff00),
        ("rgb(255 0 0 / -1)", 0x00ff0000),
        ("RGB(100% 0% 0% / 100%)", 0xffff0000),
    ] {
        assert_eq!(color::parse(input, "rgb").unwrap(), expected, "{input}");
    }
}

#[test]
fn hsl_angles_alpha_and_legacy_modern_forms_are_equivalent() {
    for (input, expected) in [
        ("hsl(0, 100%, 50%)", 0xffff0000),
        ("hsla(120,100%,50%,.5)", 0x8000ff00),
        ("hsl(240 100% 50% / 25%)", 0x400000ff),
        ("hsl(-120deg 100% 50%)", 0xff0000ff),
        ("hsl(200grad 100% 50%)", 0xff00ffff),
        ("hsl(.5turn 100% 50%)", 0xff00ffff),
        ("hsl(3.141592653589793rad 100% 50%)", 0xff00ffff),
        ("hsl(720 100 50)", 0xffff0000),
        ("HSL(60 100% 50% / 1)", 0xffffff00),
        ("hsl(0 -10% 50%)", 0xff808080),
        ("hsl(0 999% -10%)", 0xff000000),
        ("hsl(0 100% 120% / 2)", 0xffffffff),
        ("hsla(0 100% 50% / -1)", 0x00ff0000),
    ] {
        assert_eq!(color::parse(input, "hsl").unwrap(), expected, "{input}");
    }
}

#[test]
fn unsupported_or_malformed_values_preserve_diagnostic_source() {
    for value in [
        "unknown-color",
        "currentColor",
        "inherit",
        "#12",
        "#ggg",
        "red blue",
        "rgb(1,2)",
        "rgb(1,2,3,4,5)",
        "rgb(1,20%,3)",
        "rgb(1 2 3 / .5 / .5)",
        "rgb(none 0 0)",
        "rgb(calc(2) 0 0)",
        "rgb(1e999 0 0)",
        "rgb(1 2 3) junk",
        "hsl(0,100,50)",
        "hsl(0% 100% 50%)",
        "hsl(20px 100% 50%)",
        "hsl(0 100%)",
        "hsl(none 100% 50%)",
        "hsl(1e999 100% 50%)",
        "hsl(0 100% 50% / calc(.5))",
        "hsl(0 100% 50% / .5 / .5)",
        "lab(50% 0 0)",
        "oklch(50% .2 0)",
        "color(display-p3 1 0 0)",
        "color-mix(in srgb,red,blue)",
    ] {
        let error = color::parse(value, "stylesheet:42").unwrap_err();
        assert_eq!(error.code, "unsupported-color", "{value}");
        assert_eq!(error.source, "stylesheet:42", "{value}");
        assert!(!error.message.is_empty());
    }
}
