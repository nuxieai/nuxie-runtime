use nuxie_html_to_riv::{Asset, CompileInput, compile};

fn input(html: &str, css: &str) -> CompileInput {
    CompileInput {
        html: html.into(),
        css: css.into(),
        width: 390.0,
        height: 320.0,
        ..Default::default()
    }
}

#[test]
fn normal_line_height_and_font_defaults_preserve_font_dependent_inheritance() {
    let scene = |css: &str| {
        let mut source = input(
            "<div id=root><p id=a>Normal line height</p><p id=b>Inherited metrics</p></div>",
            css,
        );
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source).unwrap().riv
    };
    for css in [
        "#root {font:16px Inter}",
        "#root {font:16px/normal Inter}",
        "#root {line-height:normal}",
        "#root {line-height:initial}",
    ] {
        assert_eq!(scene(css), scene("#root {line-height:normal}"));
    }
    assert_eq!(
        scene("#root {line-height:40px;font:16px Inter} #b {font-size:20px}"),
        scene("#root {font-size:16px;line-height:normal} #b {font-size:20px;line-height:normal}")
    );
    for keyword in ["inherit", "unset"] {
        assert_eq!(
            scene(&format!(
                "#root {{line-height:normal}} #b {{font-size:20px;line-height:{keyword}}}"
            )),
            scene("#root {line-height:normal} #b {font-size:20px;line-height:normal}")
        );
    }
}

#[test]
fn font_shorthand_expands_with_cascade_and_inherited_multiplier() {
    let scene = |css: &str| {
        let mut source = input(
            "<div id=root><p id=a>Text sample</p><p id=b>Inherited text</p></div>",
            css,
        );
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source).unwrap().riv
    };
    for shorthand in [
        "16px/1.5 Inter",
        "normal 16px / 24px \"Inter\"",
        "normal normal normal 400 16px/1.5 Inter",
        "400 NORMAL 16PX / 1.5 Inter",
    ] {
        assert_eq!(
            scene(&format!(
                "#root {{font-weight:700;font-size:20px;line-height:40px;font:{shorthand}}}"
            )),
            scene("#root {font-family:Inter;font-size:16px;font-weight:400;line-height:24px}")
        );
    }
    assert_eq!(
        scene("#root {font:16px/1.5 Inter} #b {font-size:20px}"),
        scene("#root {font-size:16px;line-height:24px} #b {font-size:20px;line-height:30px}")
    );
    for keyword in ["inherit", "unset"] {
        assert_eq!(
            scene(&format!(
                "#root {{font:16px/1.5 Inter}} #b {{font-size:20px;line-height:40px;font:{keyword}}}"
            )),
            scene("#root {font-size:16px;line-height:24px}")
        );
    }
    assert_eq!(
        scene("#root {line-height:40px!important;font:16px/1.5 Inter}"),
        scene("#root {font-size:16px;line-height:40px}")
    );
    assert_eq!(
        scene("#root {font:16px/1.5 Inter!important;line-height:40px}"),
        scene("#root {font-size:16px;line-height:24px}")
    );
    for value in [
        "italic 16px/24px Inter",
        "small-caps 16px/24px Inter",
        "condensed 16px/24px Inter",
        "400 700 16px/24px Inter",
        "normal normal normal normal normal 16px/24px Inter",
        "16px/24px",
        "16px/24px Inter,Arial",
        "16px//24px Inter",
        "menu",
        "initial",
    ] {
        assert!(
            compile(&input("<div></div>", &format!(".absent {{font:{value}}}"))).is_err(),
            "{value}"
        );
    }
}

#[test]
fn unitless_line_height_inherits_the_multiplier_not_parent_pixels() {
    let scene = |css: &str| {
        let mut source = input(
            "<div id=root><p id=a>Small text</p><p id=b>Large text</p></div>",
            css,
        );
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source).unwrap().riv
    };
    assert_eq!(
        scene("#root {font-size:16px;line-height:1.5} #b {font-size:20px}"),
        scene("#a {line-height:24px} #b {font-size:20px;line-height:30px}")
    );
    assert_eq!(
        scene("#root {line-height:1.5} #b {line-height:inherit;font-size:20px}"),
        scene("#a {line-height:24px} #b {font-size:20px;line-height:30px}")
    );
    assert_eq!(
        scene("#root {line-height:1.5} #b {font-size:20px;line-height:unset}"),
        scene("#a {line-height:24px} #b {font-size:20px;line-height:30px}")
    );
    assert_eq!(
        scene("#root {line-height:30px} #b {font-size:20px}"),
        scene("#a {line-height:30px} #b {font-size:20px;line-height:30px}")
    );
    // Baseline rounding must not reject a valid, tightly spaced line box.
    scene("#root {font-size:22px;line-height:27px}");
    for (size, multiplier) in [(16, 1), (200, 10000)] {
        let mut source = input(
            "<p>Text</p>",
            &format!("p {{font-size:{size}px;line-height:{multiplier}}}"),
        );
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        assert_eq!(
            compile(&source).unwrap_err()[0].code,
            "unsupported-line-height"
        );
    }
    for value in ["0", "-1", "NaN", "infinity", "10001", "1.5 2"] {
        assert!(
            compile(&input(
                "<div></div>",
                &format!(".absent {{line-height:{value}}}")
            ))
            .is_err(),
            "{value}"
        );
    }
}

#[test]
fn solid_background_shorthand_participates_in_cascade_and_resets() {
    let scene = |css: &str| {
        compile(&input("<div id=parent><div id=child></div></div>",
        &format!("#parent {{color:#369;background-color:currentcolor;padding:12px}} #child {{height:30px;color:#a53;{css}}}"))).unwrap().riv
    };
    for (authored, expected) in [
        ("background:red", "background-color:red"),
        (
            "background:rgb(51 102 153 / .5)",
            "background-color:rgb(51 102 153 / .5)",
        ),
        ("background:hsl(210 50% 40%)", "background-color:#369"),
        ("background:currentColor", "background-color:currentcolor"),
        (
            "background:#f00;background:none",
            "background-color:transparent",
        ),
        (
            "background:#f00;background:initial",
            "background-color:transparent",
        ),
        (
            "background:#f00;background:unset",
            "background-color:transparent",
        ),
        ("background:inherit", "background-color:currentcolor"),
        (
            "background:red !important;background-color:blue",
            "background-color:red",
        ),
        (
            "background:red;background-color:blue",
            "background-color:blue",
        ),
    ] {
        assert_eq!(scene(authored), scene(expected), "{authored}");
    }
    for value in [
        "url(x.png)",
        "radial-gradient(red,blue)",
        "red,blue",
        "red blue",
        "center / cover red",
        "padding-box red",
        "none none",
    ] {
        assert!(
            compile(&input(
                "<div></div>",
                &format!(".absent {{background:{value}}}")
            ))
            .is_err(),
            "{value}"
        );
    }
}

#[test]
fn initial_and_unset_use_css_defaults_instead_of_profile_defaults() {
    let scene = |property: &str, value: &str| {
        compile(&input(
        "<div id=parent><div id=child><div id=leaf></div></div></div>",
        &format!("#parent {{padding:8px;background-color:#def}} #child {{{property}:{value}}} #leaf {{width:10px;height:10px;background-color:#369}}"),
    )).unwrap().riv
    };
    for (property, equivalent) in [
        ("width", "auto"),
        ("height", "auto"),
        ("padding", "0"),
        ("margin", "0"),
        ("gap", "0"),
        ("flex-direction", "row"),
        ("flex-wrap", "nowrap"),
        ("flex-grow", "0"),
        ("flex-basis", "auto"),
        ("align-items", "stretch"),
        ("justify-content", "flex-start"),
        ("background-color", "transparent"),
        ("border-radius", "0"),
    ] {
        assert_eq!(
            scene(property, "INITIAL"),
            scene(property, equivalent),
            "{property}"
        );
        assert_eq!(
            scene(property, "unset"),
            scene(property, equivalent),
            "{property}"
        );
    }
    assert_eq!(
        scene("padding", "12px;padding:initial;padding-left:3px"),
        scene("padding", "0 0 0 3px")
    );
    assert_eq!(
        scene("max-width", "40px;max-width:initial"),
        scene("padding", "0")
    );
    assert_eq!(
        scene("flex", "initial;flex-shrink:0"),
        scene("flex", "none")
    );
    for property in [
        "display",
        "min-width",
        "min-height",
        "font-family",
        "color",
    ] {
        let errors = compile(&input(
            "<div></div>",
            &format!(".absent {{{property}:initial}}"),
        ))
        .unwrap_err();
        assert_eq!(errors[0].code, "unsupported-initial-value", "{property}");
    }
    for css in [
        ".absent {grid-template-columns:unset}",
        ".absent {padding:initial 1px}",
    ] {
        assert!(compile(&input("<div></div>", css)).is_err());
    }
}

#[test]
fn explicit_inheritance_copies_supported_computed_properties() {
    for (property, value) in [
        ("width", "50%"),
        ("height", "80px"),
        ("padding", "2px 3px 4px 5px"),
        ("margin", "1px 2px 3px 4px"),
        ("gap", "3px 7px"),
        ("min-width", "20px"),
        ("max-width", "200px"),
        ("min-height", "10px"),
        ("max-height", "150px"),
        ("padding-left", "6px"),
        ("margin-bottom", "7px"),
        ("row-gap", "5px"),
        ("column-gap", "9px"),
        ("background-color", "#369"),
        ("border-radius", "8px"),
        ("display", "flex"),
        ("flex-direction", "row"),
        ("flex-wrap", "wrap"),
        ("align-items", "center"),
        ("justify-content", "space-between"),
        ("box-sizing", "border-box"),
        ("flex", "0 0 30px"),
        ("flex-basis", "20px"),
        ("flex-grow", "0"),
        ("flex-shrink", "0"),
    ] {
        let scene = |child: &str| {
            compile(&input(
            "<div id=parent><div id=child><div id=leaf></div></div></div>",
            &format!("#parent {{{property}:{value}}} #child {{{property}:{child}}} #leaf {{width:10px;height:10px;background-color:#f80}}"),
        )).unwrap().riv
        };
        assert_eq!(scene("INHERIT"), scene(value), "{property}");
    }
    for css in [
        ".absent {grid-template-columns:inherit}",
        ".absent {padding:inherit 2px}",
        ".absent {color:inherit red}",
    ] {
        assert!(compile(&input("<div></div>", css)).is_err(), "{css}");
    }
}

#[test]
fn hsl_colors_match_known_srgb_values() {
    let scene = |value: &str| {
        compile(&input(
            "<div></div>",
            &format!("div {{width:40px;height:30px;background-color:{value}}}"),
        ))
        .unwrap()
        .riv
    };
    for (value, hex) in [
        ("hsl(0, 100%, 50%)", "#ff0000"),
        ("hsl(0 100 50)", "#ff0000"),
        ("HSLA(120 100% 50% / 50%)", "#00ff0080"),
        ("hsl(240deg 100% 50%)", "#0000ff"),
        ("hsl(-.5turn 100% 50%)", "#00ffff"),
        ("hsl(100grad 100% 50%)", "#80ff00"),
        ("hsl(0rad 100% 50%)", "#ff0000"),
        ("hsla(720, 150%, 50%, 2)", "#ff0000"),
        ("hsl(0 -20% 50% / -1)", "#80808000"),
        ("hsl(20 30% 200%)", "#ffffff"),
        ("hsl(210 50% 40%) !important", "#336699 !important"),
    ] {
        assert_eq!(scene(value), scene(hex), "{value}");
    }
    for value in [
        "hsl(0, 50, 50)",
        "hsl(0px 50% 50%)",
        "hsl(0,50%,50%/.5)",
        "hsl(0 50% 50% .5)",
        "hsl(0 50%)",
        "hsl(none 50% 50%)",
        "hsl(0 50% 50%",
        "hsl(calc(10 + 20) 50% 50%)",
        "hsl(0 50% 50% / NaN)",
        "hsl(1e999deg 50% 50%)",
    ] {
        assert!(
            compile(&input("<div></div>", &format!(".absent {{color:{value}}}"))).is_err(),
            "{value}"
        );
    }
}

#[test]
fn named_colors_match_known_srgb_values_and_aliases() {
    let scene = |value: &str| {
        compile(&input(
            "<div></div>",
            &format!("div {{width:40px;height:30px;background-color:{value}}}"),
        ))
        .unwrap()
        .riv
    };
    for (name, hex) in [
        ("ReBeccAPurple", "#663399"),
        ("AliceBlue", "#f0f8ff"),
        ("red", "#ff0000"),
        ("green", "#008000"),
        ("lime", "#00ff00"),
        ("darkslategray", "#2f4f4f"),
        ("darkslategrey", "#2f4f4f"),
        ("aqua", "#00ffff"),
        ("cyan", "#00ffff"),
        ("magenta", "#ff00ff"),
        ("fuchsia", "#ff00ff"),
        ("transparent", "#0000"),
    ] {
        assert_eq!(scene(name), scene(hex), "{name}");
    }
    for value in [
        "notacolor",
        "CanvasText",
        "ButtonFace",
        "light blue",
        "'red'",
        "red blue",
    ] {
        assert!(
            compile(&input("<div></div>", &format!(".absent {{color:{value}}}"))).is_err(),
            "{value}"
        );
    }
}

#[test]
fn text_alignment_inherits_and_obeys_the_cascade() {
    let scene = |css: &str| {
        let mut source = input("<div id=root><p id=a>Aligned text</p></div>", css);
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source).unwrap().riv
    };
    for alignment in ["left", "center", "right"] {
        assert_eq!(
            scene(&format!("#root {{text-align:{alignment}}}")),
            scene(&format!("#a {{text-align:{alignment}}}"))
        );
    }
    assert_eq!(
        scene("#root {text-align:center} #a {text-align:right !important;text-align:left}"),
        scene("#a {text-align:right}")
    );
    for value in [
        "justify",
        "start",
        "end",
        "match-parent",
        "left right",
        "center !wrong",
    ] {
        assert!(
            compile(&input(
                "<div></div>",
                &format!(".absent {{text-align:{value}}}")
            ))
            .is_err(),
            "{value}"
        );
    }
}

#[test]
fn currentcolor_resolves_after_cascade_and_inherits_for_color_property() {
    for (authored, expected) in [
        (
            "background-color:currentColor;color:#369",
            "background-color:#369;color:#369",
        ),
        (
            "color:#369;background-color:currentcolor",
            "background-color:#369;color:#369",
        ),
        (
            "color:#f00;color:currentcolor;background-color:currentcolor",
            "color:#123456;background-color:#123456",
        ),
        (
            "background-color:currentcolor;background-color:#fff;color:#369",
            "background-color:#fff;color:#369",
        ),
        (
            "background-color:currentcolor !important;background-color:#fff;color:#369",
            "background-color:#369;color:#369",
        ),
        (
            "background-color:currentcolor;color:transparent",
            "background-color:transparent;color:transparent",
        ),
    ] {
        let scene = |declarations| {
            compile(&input(
            "<div id=root><div id=a><div id=b></div></div></div>",
            &format!("#root {{color:#123456}} #a {{width:80px;height:40px;{declarations}}} #b {{width:20px;height:20px;background-color:currentcolor}}"),
        )).unwrap()
        };
        assert_eq!(scene(authored).riv, scene(expected).riv, "{authored}");
    }
}

#[test]
fn invalid_and_deferred_color_functions_are_rejected() {
    for value in [
        "rgb(1, 2%, 3)",
        "rgb(1,2,3 / .5)",
        "rgb(1 2 3, .5)",
        "rgb(1 2)",
        "rgb(1 2 3 4)",
        "rgb(1 2 3 /)",
        "rgb(1 2 3 / .5 / .5)",
        "rgb(1,,3)",
        "rgb(1,2,3,)",
        "rgb(1px 2 3)",
        "rgb(NaN 2 3)",
        "rgb(1e999 2 3)",
        "rgb(none 0 0)",
        "rgb(from #fff r g b)",
        "rgb(calc(1 + 2) 0 0)",
        "rgb(env(unknown) 0 0)",
        "rgb(rgb(0 0 0) 0 0)",
        "rgb(1 2 3",
        "rgb(1 2 3))",
        "rgb(1 2 3) garbage",
        "rgb(1 2 3 !important)",
        "color(display-p3 1 0 0)",
    ] {
        for css in [format!(".absent {{color:{value}}}"), String::new()] {
            let html = if css.is_empty() {
                format!("<div style='color:{value}'></div>")
            } else {
                "<div></div>".into()
            };
            assert!(compile(&input(&html, &css)).is_err(), "{value}");
        }
    }
}

#[test]
fn rgb_colors_compile_like_equivalent_hex_colors() {
    for (value, hex) in [
        ("rgb(51, 102, 153)", "#336699"),
        ("RGBA(20%, 40%, 60%, 50%)", "#33669980"),
        ("rgb(51 40% 153 / .5)", "#33669980"),
        ("rgba(51 102 153)", "#336699"),
        ("rgb(-20, 300, 0, 2)", "#00ff00"),
        ("rgb(1e2 0 0 / -1)", "#64000000"),
        ("rgb(51/**/ 102 153) !important", "#336699 !important"),
    ] {
        let scene = |color| {
            compile(&input(
                "<div id=a></div>",
                &format!("#a {{width:80px;height:40px;background-color:{color}}}"),
            ))
            .unwrap_or_else(|errors| panic!("{color}: {errors:?}"))
        };
        assert_eq!(scene(value).riv, scene(hex).riv, "{value}");
    }
}

#[test]
fn flex_shorthand_and_longhand_cascade_produce_the_same_scene() {
    for (shorthand, expanded) in [
        ("flex:1", "flex-grow:1;flex-shrink:1;flex-basis:0%"),
        ("flex:2 2 80px", "flex-grow:2;flex-shrink:2;flex-basis:80px"),
        ("flex:80px 2 2", "flex-grow:2;flex-shrink:2;flex-basis:80px"),
        ("flex:auto", "flex-grow:1;flex-shrink:1;flex-basis:auto"),
        (
            "flex:1;flex:none",
            "flex-grow:0;flex-shrink:0;flex-basis:auto",
        ),
        (
            "flex:1;flex-basis:30%",
            "flex-grow:1;flex-shrink:1;flex-basis:30%",
        ),
    ] {
        let a = compile(&input(
            "<div id=a></div>",
            &format!("#a {{height:80px;{shorthand}}}"),
        ))
        .unwrap();
        let b = compile(&input(
            "<div id=a></div>",
            &format!("#a {{height:80px;{expanded}}}"),
        ))
        .unwrap();
        assert_eq!(a.riv, b.riv, "{shorthand}");
    }
}

#[test]
fn unrepresentable_flex_and_malformed_values_are_rejected() {
    let nested = "<div id=root><div id=parent><div id=item></div></div></div>";
    let uncertain = "#root {width:100%;align-items:flex-start} #parent {flex-direction:row} #item {flex:auto;width:50%;height:20px}";
    let percentage_auto = compile(&input(nested, uncertain)).unwrap();
    assert_eq!(percentage_auto.runtime_requirements.version, 12);
    assert!(percentage_auto.runtime_requirements.capabilities.contains(
        &nuxie_html_to_riv::RuntimeCapability::LayoutCssIndefiniteBasisV1));
    assert!(
        compile(&input(
            nested,
            &uncertain.replace("align-items:flex-start", "align-items:stretch")
        ))
        .is_ok()
    );
    for css in [
        "flex-grow:1",
        "flex-shrink:1",
        "flex:2",
        "flex:initial",
        "flex:1 0 auto",
    ] {
        let output = compile(&input("<div></div>", &format!("div {{width:40px;height:40px;{css}}}"))).unwrap();
        assert_eq!(output.runtime_requirements.version, 9, "{css}");
        assert_eq!(output.runtime_requirements.layout_flex_factors.len(), 1, "{css}");
    }
    for css in [
        "flex-grow:-1",
        "flex-shrink:NaN",
        "flex-grow:10001",
        "flex-basis:content",
        "flex-basis:-1px",
        "flex:1 1 1 1",
        "flex:1px 2px",
        "flex:1 auto 1",
        "flex:1 1 2",
    ] {
        assert!(
            compile(&input("<div></div>", &format!("div {{{css}}}"))).is_err(),
            "{css}"
        );
    }
}

#[test]
fn deferred_corpus_rejections_remain_explicit() {
    let deferred: serde_json::Value =
        serde_json::from_str(include_str!("../validation/deferred-cases.json")).unwrap();
    for case in deferred.as_array().unwrap() {
        let mut source = input(
            case["html"].as_str().unwrap(),
            case["css"].as_str().unwrap(),
        );
        if case["font"].as_bool() == Some(true) {
            source.assets.insert(
                "inter".into(),
                Asset::Font {
                    family: "Inter".into(),
                    weight: 400,
                    bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
                },
            );
        }
        assert_eq!(
            compile(&source).unwrap_err()[0].code,
            case["diagnostic"].as_str().unwrap()
        );
    }
}

#[test]
fn unsupported_syntax_is_rejected_even_when_its_selector_does_not_match() {
    for css in [
        ".absent { display:grid }",
        ".absent { grid-template-columns:1fr 1fr }",
        ".absent { position:fixed }",
        ".absent { transform:rotate(10deg) }",
        "div:hover { color:#fff }",
        "div::before { content:'x' }",
        "@media (min-width:300px) { div { width:30px } }",
        "@import 'other.css';",
        "div { --width:20px; width:env(unknown) }",
        "div { width:calc(100% - 10px) }",
        "div { width:1ex }",
        "div { width:-1px }",
        "div { width:NaNpx }",
        "div { opacity:calc(.5) }",
        "div { filter:blur(1px) }",
        "div { box-shadow:0 1px 2px #000 }",
        "div { float:left }",
        "div { display:table }",
        "div { box-sizing:padding-box }",
        "div { font-family:Inter,Arial }",
        "div { background-color:notacolor }",
        "div { font-size:0 }",
        "div { padding:1px 2px 3px 4px 5px }",
        "div { width:10px !wrong }",
        "div { height: }",
        "div { width 10px }",
        "div { background-image:url(https://example.com/x.png) }",
    ] {
        let errors = compile(&input("<div></div>", css)).expect_err(css);
        assert!(!errors.is_empty(), "{css}");
        assert!(
            errors
                .iter()
                .all(|e| !e.code.is_empty() && !e.message.is_empty() && !e.source.is_empty()),
            "diagnostics must identify the unsupported source: {css}"
        );
    }
}

#[test]
fn unsupported_html_is_never_silently_discarded() {
    for html in [
        "<script>alert(1)</script>",
        "<iframe></iframe>",
        "<svg></svg>",
        "<canvas></canvas>",
        "<input>",
        "<button>Buy</button>",
        "<a href='/'>Link</a>",
        "<div onclick='x()'></div>",
        "<div hidden></div>",
        "<p>Hello <span>world</span></p>",
        "<div id=a></div><div id=a></div>",
        "<div id=a data-nuxie-id=one></div><div id=a data-nuxie-id=two></div>",
        "unwrapped text<div></div>",
        "<img src='https://example.com/image.png'>",
    ] {
        assert!(compile(&input(html, "")).is_err(), "must reject {html}");
    }
}

#[test]
fn malformed_html_reports_a_diagnostic_instead_of_publishing_a_repaired_scene() {
    for html in [
        "<div id=a id=b></div>",
        "<div><p></div></p>",
        "<div x=\"unterminated>",
    ] {
        assert!(compile(&input(html, "")).is_err(), "must reject {html}");
    }
}

#[test]
fn authored_ids_are_stable_across_sibling_insertions_and_recompilation_is_deterministic() {
    let a = input(
        "<div data-nuxie-id=stable><div id=child></div></div>",
        "div { width:30px; height:20px }",
    );
    let first = compile(&a).unwrap();
    assert_eq!(first.riv, compile(&a).unwrap().riv);
    let b = input(
        "<div id=new></div><div data-nuxie-id=stable><div id=child></div></div>",
        &a.css,
    );
    let next = compile(&b).unwrap();
    for id in ["stable", "child"] {
        assert!(first.source_map.iter().any(|n| n.id == id));
        assert!(next.source_map.iter().any(|n| n.id == id));
    }
    assert_ne!(
        first
            .source_map
            .iter()
            .find(|n| n.id == "stable")
            .unwrap()
            .object_id,
        next.source_map
            .iter()
            .find(|n| n.id == "stable")
            .unwrap()
            .object_id
    );
}

#[test]
fn nested_element_depth_boundary_uses_default_thread_stack() {
    // Top-level elements have depth zero; depth 64 is accepted, 65 is rejected.
    for levels in [65, 66] {
        let html = format!("{}{}", "<div>".repeat(levels), "</div>".repeat(levels));
        let result = compile(&input(&html, ""));
        if levels == 65 {
            assert_eq!(result.unwrap().source_map.len(), levels);
        } else {
            assert_eq!(result.unwrap_err()[0].code, "input-limit");
        }
    }
}

#[test]
fn invalid_viewports_and_resource_limits_fail_before_emission() {
    for width in [0.0, -1.0, f32::NAN, f32::INFINITY, 16385.0] {
        let mut i = input("<div></div>", "");
        i.width = width;
        assert_eq!(compile(&i).unwrap_err()[0].code, "invalid-viewport");
    }
    let mut i = input("<div></div>", "");
    i.html = " ".repeat(1_048_577);
    assert_eq!(compile(&i).unwrap_err()[0].code, "input-limit");
    let deep = format!("{}{}", "<div>".repeat(66), "</div>".repeat(66));
    assert_eq!(
        compile(&input(&deep, "")).unwrap_err()[0].code,
        "input-limit"
    );
    assert_eq!(
        compile(&input(&"<div></div>".repeat(2049), "")).unwrap_err()[0].code,
        "input-limit"
    );
}

#[test]
fn missing_corrupt_and_mismatched_assets_fail_with_explicit_diagnostics() {
    assert_eq!(
        compile(&input("<p>Hello</p>", "")).unwrap_err()[0].code,
        "missing-font"
    );
    assert_eq!(
        compile(&input(
            "<img src=asset:missing>",
            "img{width:10px;height:10px}"
        ))
        .unwrap_err()[0]
            .code,
        "missing-image"
    );
    let mut i = input("<div></div>", "");
    i.assets.insert(
        "bad".into(),
        Asset::Font {
            family: "Inter".into(),
            weight: 400,
            bytes: vec![0; 64],
        },
    );
    assert_eq!(compile(&i).unwrap_err()[0].code, "invalid-font");
    i.assets
        .insert("bad".into(), Asset::Image { bytes: vec![0; 64] });
    assert_eq!(compile(&i).unwrap_err()[0].code, "invalid-image");
    i.assets.insert(
        "bad".into(),
        Asset::Font {
            family: "Inter".into(),
            weight: 700,
            bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
        },
    );
    assert_eq!(compile(&i).unwrap_err()[0].code, "unsupported-font");
}

#[test]
fn every_accepted_corpus_file_imports_without_dropping_records() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("../validation/cases.json")).unwrap();
    for case in corpus.as_array().unwrap() {
        let mut i = input(
            case["html"].as_str().unwrap(),
            case["css"].as_str().unwrap(),
        );
        if case["font"].as_bool() == Some(true) {
            i.assets.insert(
                "inter".into(),
                Asset::Font {
                    family: case["fontFamily"].as_str().unwrap_or("Inter").into(),
                    weight: 400,
                    bytes: if case["fontAsset"] == "OpenSans-Regular.ttf" {
                        include_bytes!("assets/OpenSans-Regular.ttf").to_vec()
                    } else if case["fontAsset"] == "NotoSansOgham-Regular.ttf" {
                        include_bytes!("assets/NotoSansOgham-Regular.ttf").to_vec()
                    } else if case["fontAsset"] == "NuxieJapaneseFixture-Regular.otf" {
                        include_bytes!("assets/NuxieJapaneseFixture-Regular.otf").to_vec()
                    } else {
                        include_bytes!("assets/Inter-Regular.ttf").to_vec()
                    },
                },
            );
        }
        if case["image"].as_bool() == Some(true) {
            i.assets.insert(
                "photo".into(),
                Asset::Image {
                    bytes: include_bytes!("assets/quadrants.png").to_vec(),
                },
            );
        }
        let out = compile(&i).unwrap_or_else(|e| panic!("{}: {e:?}", case["name"]));
        let parsed = nuxie_binary::read_runtime_file(&out.riv).expect("wire format parses");
        assert_eq!(
            parsed.imported_object_count(),
            parsed.object_count(),
            "{} must not silently lose objects",
            case["name"]
        );
        assert_eq!(parsed.known_object_count(), parsed.object_count());
    }
}

#[test]
fn stylesheet_complexity_limits_fail_with_diagnostics() {
    let declarations = "padding:0;".repeat(257);
    assert!(compile(&input("<div></div>", &format!("div {{{declarations}}}"))).is_err());
    let rules = "div {padding:0}".repeat(1025);
    assert_eq!(
        compile(&input("<div></div>", &rules)).unwrap_err()[0].code,
        "input-limit"
    );
}

#[test]
fn color_managed_png_is_rejected_instead_of_rendering_different_colors() {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_gamma(png::ScaledFloat::new(1.0));
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[20, 40, 60])
            .unwrap();
    }
    let mut i = input("<img src=asset:photo style='width:1px;height:1px'>", "");
    i.assets.insert("photo".into(), Asset::Image { bytes });
    assert_eq!(compile(&i).unwrap_err()[0].code, "unsupported-image");
}

#[test]
fn em_lengths_use_parent_font_size_and_final_cascaded_element_size() {
    let scene = |css: &str| {
        compile(&input(
            "<div id=root><div id=child></div><div id=sibling></div></div>",
            css,
        ))
        .unwrap()
        .riv
    };
    let relative = "#root{font-size:20px;width:100%;padding:1em;gap:.5em}#child{width:4em;height:2em;margin:.25em;padding:.5em;min-width:2em;max-width:8em;border-radius:.25em;font-size:2em;font-size:1.5em}#sibling{font-size:.5em;width:3em;height:2em}";
    let absolute = "#root{font-size:20px;width:100%;padding:20px;gap:10px}#child{width:120px;height:60px;margin:7.5px;padding:15px;min-width:60px;max-width:240px;border-radius:7.5px;font-size:30px}#sibling{font-size:10px;width:30px;height:20px}";
    assert_eq!(scene(relative), scene(absolute));
    // Preserve an exact px control for the fractional-edge visual failure.
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("../validation/cases.json")).unwrap();
    let fixture = corpus
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == "em-layout-cascade")
        .unwrap();
    let html = fixture["html"].as_str().unwrap();
    let px = "#root{width:100%;padding:20px;gap:10px;font-size:20px;background:#eef}#a{width:120px;height:60px;padding:7.5px;margin:3.75px;border-radius:7.5px;min-width:60px;max-width:180px;background:#369;font-size:30px}#b{width:60px;height:20px;background:coral;font-size:10px}";
    assert_eq!(
        compile(&input(html, fixture["css"].as_str().unwrap()))
            .unwrap()
            .riv,
        compile(&input(html, px)).unwrap().riv
    );
    assert_eq!(
        scene("#root{font-size:20px}#child{width:2em;font-size:2em!important;font-size:3em}"),
        scene("#root{font-size:20px}#child{width:80px;font-size:40px}")
    );
}

#[test]
fn em_line_height_is_inherited_as_a_length_and_font_shorthand_uses_final_size() {
    let scene = |css: &str| {
        let mut source = input(
            "<div id=root><p id=a>Parent line</p><p id=b>Child line</p></div>",
            css,
        );
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source).unwrap().riv
    };
    assert_eq!(
        scene("#root{font:1.25em/2em Inter}#b{font-size:.5em}"),
        scene("#root{font:20px/40px Inter}#b{font-size:10px}")
    );
    assert_eq!(
        scene("#root{font:1.25em/2em Inter;font-size:1em}"),
        scene("#root{font:16px/32px Inter}")
    );
    assert_eq!(
        scene("#root{font-size:20px;line-height:2em}#b{font-size:.5em;line-height:inherit}"),
        scene("#root{font-size:20px;line-height:40px}#b{font-size:10px;line-height:40px}")
    );
}

#[test]
fn em_lengths_preserve_global_inheritance_and_strict_resource_validation() {
    let scene = |css: &str| {
        compile(&input("<div id=root><div id=child></div></div>", css))
            .unwrap()
            .riv
    };
    for keyword in ["inherit", "unset"] {
        assert_eq!(
            scene(&format!(
                "#root{{font-size:20px}}#child{{font-size:{keyword};width:2em}}"
            )),
            scene("#root{font-size:20px}#child{font-size:20px;width:40px}")
        );
    }
    assert_eq!(
        scene(
            "#root{font-size:20px;padding:1em}#child{font-size:initial;width:2em;padding:inherit}"
        ),
        scene("#root{font-size:20px;padding:20px}#child{font-size:16px;width:32px;padding:20px}")
    );
    assert_eq!(
        scene("#root{font-size:.000001px;width:1000000em}"),
        scene("#root{font-size:.000001px;width:1px}")
    );
    for value in [
        "-1em",
        "1.em",
        "1 em",
        "1000001em",
        "1ex",
        "calc(1em + 1px)",
    ] {
        for selector in ["#root", ".unmatched"] {
            assert!(
                compile(&input(
                    "<div id=root></div>",
                    &format!("{selector}{{width:{value}}}")
                ))
                .is_err(),
                "{selector}: {value}"
            );
        }
    }
    assert!(
        compile(&input(
            "<div id=root></div>",
            "#root{font-size:100px;width:10001em}"
        ))
        .is_err()
    );
    assert!(compile(&input("<div id=root></div>", "#root{font-size:0em}")).is_err());
}

#[test]
fn rem_lengths_use_the_fixed_host_root_even_inside_resized_fonts() {
    let scene = |css: &str| {
        let mut source = input(
            "<div id=root><p id=a>Root units</p><p id=b>Inherited leading</p></div>",
            css,
        );
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source).unwrap().riv
    };
    assert_eq!(
        scene(
            "#root{font:2rem/3rem Inter;padding:.5rem 1em;gap:.5REM}#a{font-size:1.25rem;width:10rem;min-width:2rem;max-width:12rem;margin:.25rem;border-radius:.5rem}#b{font-size:1rem;padding:inherit;line-height:inherit}"
        ),
        scene(
            "#root{font:32px/48px Inter;padding:8px 32px;gap:8px}#a{font-size:20px;width:160px;min-width:32px;max-width:192px;margin:4px;border-radius:8px}#b{font-size:16px;padding:inherit;line-height:inherit}"
        )
    );
    assert_eq!(
        scene(
            "#root{font-size:40px;line-height:64px}#a{font:2rem/3rem Inter;font-size:1rem!important;padding:1em 1rem}"
        ),
        scene("#root{font-size:40px;line-height:64px}#a{font:16px/48px Inter;padding:16px}")
    );
    for value in [
        "-1rem",
        "1.rem",
        "1 rem",
        "62501rem",
        "1000001rem",
        "1rrem",
        "calc(1rem + 1px)",
    ] {
        for selector in ["#root", ".unmatched"] {
            assert!(
                compile(&input(
                    "<div id=root></div>",
                    &format!("{selector}{{width:{value}}}")
                ))
                .is_err(),
                "{selector}: {value}"
            );
        }
    }
    assert!(compile(&input("<div></div>", "div{font-size:0rem}")).is_err());
    assert!(compile(&input("<div></div>", "div{width:62500rem}")).is_ok());
}

#[test]
fn percentage_limits_compile_with_strict_value_validation() {
    compile(&input("<div id=root><div id=a></div></div>", "#root{width:100%;height:200px}#a{width:120px;height:20px;min-width:25%;max-width:75%;min-height:10%;max-height:50%}")).unwrap();
    for property in ["min-width", "max-width", "min-height", "max-height"] {
        for value in ["-1%", "10001%", "NaN%", "auto", "min-content"] {
            assert!(
                compile(&input(
                    "<div></div>",
                    &format!(".unmatched{{{property}:{value}}}")
                ))
                .is_err()
            );
        }
        assert!(compile(&input("<div></div>", &format!("div{{{property}:0%}}"))).is_ok());
    }
}

#[test]
fn letter_spacing_inherits_computed_lengths_and_survives_font_shorthand() {
    let scene = |css: &str| {
        let mut source = input("<div id=root><p id=a>Spaced text</p></div>", css);
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source).unwrap().riv
    };
    assert_eq!(
        scene(
            "#root{font-size:20px;line-height:64px;letter-spacing:.1em}#a{font-size:40px;letter-spacing:inherit}"
        ),
        scene(
            "#root{font-size:20px;line-height:64px;letter-spacing:2px}#a{font-size:40px;letter-spacing:2px}"
        )
    );
    assert_eq!(
        scene("#root{letter-spacing:-.125rem}#a{font:20px Inter;letter-spacing:unset}"),
        scene("#root{letter-spacing:-2px}#a{font:20px Inter;letter-spacing:-2px}")
    );
    assert_eq!(
        scene("#root{letter-spacing:normal}#a{letter-spacing:initial}"),
        scene("#root{letter-spacing:0}#a{letter-spacing:0px}")
    );
    for value in [
        "10%",
        "1",
        "1ex",
        "1.px",
        "NaNpx",
        "1000001px",
        "-1000001px",
        "calc(1px + 1px)",
    ] {
        assert!(
            compile(&input(
                "<div></div>",
                &format!(".missing{{letter-spacing:{value}}}")
            ))
            .is_err(),
            "{value}"
        );
    }
}

#[test]
fn runtime_requirements_are_explicit_versioned_and_content_independent() {
    use nuxie_html_to_riv::{RuntimeCapability, RuntimeRequirements};
    let output = |text: &str, spacing: &str| {
        let mut source = input(
            &format!("<p>{text}</p>"),
            &format!("p{{letter-spacing:{spacing}}}"),
        );
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source).unwrap()
    };
    let plain = output("A", "normal");
    assert_eq!(
        plain.runtime_requirements.capabilities,
        [RuntimeCapability::LayoutCssIntrinsicSizingV1, RuntimeCapability::TextCssShapingPrecisionV1, RuntimeCapability::TextCssNormalWrapV1]
            .into_iter()
            .collect()
    );
    let spaced = output("A", "2px");
    assert_eq!(
        spaced.runtime_requirements,
        output("a\u{301}\u{327}", "2px").runtime_requirements
    );
    assert!(spaced.runtime_requirements.ensure_supported(&[]).is_err());
    assert!(
        spaced
            .runtime_requirements
            .ensure_supported(&[
                RuntimeCapability::LayoutCssIntrinsicSizingV1, RuntimeCapability::TextCssShapingPrecisionV1, RuntimeCapability::TextCssNormalWrapV1,
                RuntimeCapability::TextClusterSpacingV1
            ])
            .is_err(),
        "a cluster-only host cannot claim CSS optional-ligature behavior"
    );
    spaced
        .runtime_requirements
        .ensure_supported(&[
            RuntimeCapability::LayoutCssIntrinsicSizingV1, RuntimeCapability::TextCssShapingPrecisionV1, RuntimeCapability::TextCssNormalWrapV1,
            RuntimeCapability::TextCssLetterSpacingV1,
        ])
        .unwrap();
    let encoded = serde_json::to_value(&spaced.runtime_requirements).unwrap();
    assert_eq!(
        serde_json::from_value::<RuntimeRequirements>(encoded).unwrap(),
        spaced.runtime_requirements
    );
    let mut future = spaced.runtime_requirements.clone();
    future.version = 3;
    assert!(
        future
            .ensure_supported(&[
                RuntimeCapability::LayoutCssIntrinsicSizingV1, RuntimeCapability::TextCssShapingPrecisionV1, RuntimeCapability::TextCssNormalWrapV1,
                RuntimeCapability::TextCssLetterSpacingV1
            ])
            .is_err()
    );
    assert!(
        serde_json::from_str::<RuntimeRequirements>(
            r#"{"version":1,"capabilities":["future-capability"]}"#
        )
        .is_err()
    );
}

#[test]
fn word_spacing_cascade_lengths_and_normal_are_explicit() {
    let scene = |css: &str| {
        let mut source = input(
            "<div id=root><p id=a>one two three</p><p id=b>four five</p></div>",
            css,
        );
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source)
    };
    assert_eq!(
        scene("p{word-spacing:normal}").unwrap().riv,
        scene("p{word-spacing:0}").unwrap().riv
    );
    for keyword in ["inherit", "unset"] {
        assert_eq!(scene(&format!("#root{{font-size:20px;word-spacing:.2em}}p{{font:16px Inter;word-spacing:{keyword}}}")).unwrap().riv,
            scene("#root{font-size:20px}p{font:16px Inter;word-spacing:4px}").unwrap().riv);
    }
    assert_eq!(
        scene("p{word-spacing:-.125rem}").unwrap().riv,
        scene("p{word-spacing:-2px}").unwrap().riv
    );
    assert_eq!(
        scene("#root{word-spacing:4px}p{word-spacing:initial}")
            .unwrap()
            .riv,
        scene("p{word-spacing:normal}").unwrap().riv
    );
    for value in [
        "1%",
        "calc(2px)",
        "2",
        "NaNpx",
        "1000001px",
        "-1000001px",
        "1px 2px",
    ] {
        assert!(
            scene(&format!(".unmatched{{word-spacing:{value}}}")).is_err(),
            "{value}"
        );
    }
}

#[test]
fn word_spacing_run_expansion_is_bounded_and_visible_separators_are_explicit() {
    let mut source = input(
        &format!("<p>{}</p>", "a ".repeat(2050)),
        "p{word-spacing:2px}",
    );
    source.assets.insert(
        "inter".into(),
        Asset::Font {
            family: "Inter".into(),
            weight: 400,
            bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
        },
    );
    assert_eq!(compile(&source).unwrap_err()[0].code, "input-limit");
    source.html = "<p>a&#x1361;b</p>".into();
    assert_eq!(compile(&source).unwrap_err()[0].code, "unsupported-text");
}

#[test]
fn preserved_space_capability_is_separate_and_covers_its_exact_character_set() {
    use nuxie_html_to_riv::RuntimeCapability;
    let mut source = input("<p></p>", "");
    source.assets.insert(
        "inter".into(),
        Asset::Font {
            family: "Inter".into(),
            weight: 400,
            bytes: include_bytes!("assets/NotoSansOgham-Regular.ttf").to_vec(),
        },
    );
    for ch in [
        '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}',
        '\u{2006}', '\u{2008}', '\u{2009}', '\u{200a}', '\u{205f}', '\u{3000}',
    ] {
        source.html = format!("<p>one{ch}two</p>");
        let req = compile(&source).unwrap().runtime_requirements;
        assert_eq!(req.capabilities.len(), 4);
        assert!(
            req.ensure_supported(&[
                RuntimeCapability::LayoutCssIntrinsicSizingV1, RuntimeCapability::TextCssShapingPrecisionV1, RuntimeCapability::TextCssNormalWrapV1,
                RuntimeCapability::TextCssLetterSpacingV1
            ])
            .is_err()
        );
        req.ensure_supported(&[
            RuntimeCapability::LayoutCssIntrinsicSizingV1, RuntimeCapability::TextCssShapingPrecisionV1, RuntimeCapability::TextCssNormalWrapV1,
            RuntimeCapability::TextPreservedSpaceBreaksV1,
        ])
        .unwrap();
    }
    for ch in [' ', '\u{a0}', '\u{2007}', '\u{202f}'] {
        source.html = format!("<p>one{ch}two</p>");
        assert!(
            compile(&source).unwrap().runtime_requirements.capabilities
                == [nuxie_html_to_riv::RuntimeCapability::LayoutCssIntrinsicSizingV1, nuxie_html_to_riv::RuntimeCapability::TextCssShapingPrecisionV1, RuntimeCapability::TextCssNormalWrapV1]
                    .into_iter()
                    .collect()
        );
    }
}

#[test]
fn explicit_breaks_have_scalar_offsets_and_reject_unmapped_contexts() {
    let scene = |html: &str, css: &str| {
        let mut source = input(html, css);
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source)
    };
    assert_eq!(
        scene(
            &format!("<p>{}</p>", "<br>".repeat(4097)),
            "p{display:block}"
        )
        .unwrap_err()[0]
            .code,
        "input-limit"
    );
    let many = format!("<p>{}</p>", "<br>".repeat(3000)).repeat(3);
    assert_eq!(
        scene(&many, "p{display:block}").unwrap_err()[0].code,
        "input-limit"
    );
    let output = scene("<p id=a> é <br id=cut> two <br></p>", "p{display:block}").unwrap();
    let map = serde_json::to_value(output.source_map).unwrap();
    assert_eq!(map[0]["text_breaks"][0]["id"], "cut");
    assert_eq!(map[0]["text_breaks"][0]["text_offset"], 1);
    assert_eq!(map[0]["text_breaks"][1]["text_offset"], 5);
    for (html, css, code) in [
        ("<p>a<br>b</p>", "", "unsupported-break-context"),
        (
            "<p>a<br style='color:red'>b</p>",
            "p{display:block}",
            "unsupported-attribute",
        ),
        (
            "<p>a<br>b</p>",
            "p{display:block}br{color:red}",
            "unsupported-break-style",
        ),
        (
            "<div><p>a</p></div>",
            "div{display:block}",
            "unsupported-block-context",
        ),
        (
            "<p id=x>a<br id=x>b</p>",
            "p{display:block}",
            "duplicate-id",
        ),
    ] {
        assert_eq!(scene(html, css).unwrap_err()[0].code, code, "{html}");
    }
}

#[test]
fn nowrap_inherits_and_normal_restores_soft_wrapping() {
    let scene = |css: &str| {
        let mut source = input(
            "<div id=root><p id=a>one two three four five</p></div>",
            css,
        );
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source)
    };
    assert_eq!(
        serde_json::to_value(scene("p{white-space:nowrap}").unwrap().runtime_requirements).unwrap(),
        serde_json::json!({"version":11,"layout_intrinsic_sizing":[4],"capabilities":["layout-css-intrinsic-sizing-v1","text-css-shaping-precision-v1","text-css-nowrap-alignment-v1"],"text_policies":[{"object_id":8,"policy":"css-nowrap-alignment-v1"}]})
    );
    assert!(
        scene("").unwrap().runtime_requirements.capabilities
            == [nuxie_html_to_riv::RuntimeCapability::LayoutCssIntrinsicSizingV1, nuxie_html_to_riv::RuntimeCapability::TextCssShapingPrecisionV1, nuxie_html_to_riv::RuntimeCapability::TextCssNormalWrapV1]
                .into_iter()
                .collect()
    );
    for keyword in ["inherit", "unset"] {
        assert_eq!(
            scene(&format!(
                "#root{{white-space:nowrap}}p{{white-space:{keyword}}}"
            ))
            .unwrap()
            .riv,
            scene("p{white-space:nowrap}").unwrap().riv
        );
    }
    for keyword in ["normal", "initial"] {
        assert_eq!(
            scene(&format!(
                "#root{{white-space:nowrap}}p{{white-space:{keyword}}}"
            ))
            .unwrap()
            .riv,
            scene("").unwrap().riv
        );
    }
    assert_ne!(
        scene("p{white-space:nowrap}").unwrap().riv,
        scene("").unwrap().riv
    );
    for value in ["break-spaces", "nowrap normal", "1"] {
        assert!(scene(&format!(".unmatched{{white-space:{value}}}")).is_err());
    }
}

#[test]
fn pre_preserves_spaces_newlines_and_break_offsets() {
    let scene = |html: &str, css: &str| {
        let mut source = input(html, css);
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source)
    };
    let html = "<div id=root><p id=a>  é  \n x<br id=cut> y  </p></div>";
    let base = scene(html, "p{display:block;white-space:pre}").unwrap();
    assert_eq!(base.source_map[1].text_breaks[0].text_offset, 8);
    for keyword in ["inherit", "unset"] {
        assert_eq!(
            base.riv,
            scene(
                html,
                &format!("#root{{white-space:pre}}p{{display:block;white-space:{keyword}}}")
            )
            .unwrap()
            .riv
        );
    }
    for keyword in ["normal", "initial"] {
        assert_eq!(
            scene(html, "p{display:block}").unwrap().riv,
            scene(
                html,
                &format!("#root{{white-space:pre}}p{{display:block;white-space:{keyword}}}")
            )
            .unwrap()
            .riv
        );
    }
    assert_ne!(
        base.riv,
        scene(html, "p{display:block;white-space:nowrap}")
            .unwrap()
            .riv
    );
    assert_eq!(
        scene("<p>a\r\nb</p>", "p{white-space:pre}").unwrap().riv,
        scene("<p>a\nb</p>", "p{white-space:pre}").unwrap().riv
    );
    assert!(
        scene("<p>a\tb</p>", "p{white-space:pre}")
            .unwrap()
            .runtime_requirements
            .capabilities
            .contains(&nuxie_html_to_riv::RuntimeCapability::TextCssTabsV1)
    );
}

#[test]
fn pre_wrap_preserves_source_but_restores_soft_wrapping() {
    let scene = |css: &str, html: &str| {
        let mut source = input(html, css);
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source)
    };
    let html = "<div id=root><p id=a>  one  two\n three <br id=cut>four </p></div>";
    let base = scene("p{display:block;white-space:pre-wrap}", html).unwrap();
    let pre = scene("p{display:block;white-space:pre}", html).unwrap();
    use nuxie_html_to_riv::{RuntimeCapability, RuntimeRequirements, TextPolicy};
    assert_eq!(base.runtime_requirements.version, 11);
    assert_eq!(base.runtime_requirements.text_policies.len(), 1);
    assert_eq!(
        base.runtime_requirements.text_policies[0].policy,
        TextPolicy::CssPreWrapV1
    );
    assert!(base.runtime_requirements.ensure_supported(&[]).is_err());
    base.runtime_requirements
        .ensure_supported(&[
            RuntimeCapability::LayoutCssIntrinsicSizingV1, RuntimeCapability::TextCssShapingPrecisionV1,
            RuntimeCapability::TextCssPreWrapV1,
        ])
        .unwrap();
    for version in [1, 2] {
        let incomplete: RuntimeRequirements = serde_json::from_value(serde_json::json!({
            "version":version,"capabilities":["text-css-pre-wrap-v1"]
        }))
        .unwrap();
        assert_eq!(
            incomplete
                .ensure_supported(&[
                    RuntimeCapability::LayoutCssIntrinsicSizingV1, RuntimeCapability::TextCssShapingPrecisionV1,
                    RuntimeCapability::TextCssPreWrapV1
                ])
                .unwrap_err()
                .code,
            "invalid-text-policies"
        );
    }

    assert_eq!(
        serde_json::to_value(&base.source_map[1].text_breaks).unwrap(),
        serde_json::to_value(&pre.source_map[1].text_breaks).unwrap()
    );
    assert_ne!(base.riv, pre.riv);
    for keyword in ["inherit", "unset"] {
        assert_eq!(
            base.riv,
            scene(
                &format!("#root{{white-space:pre-wrap}}p{{display:block;white-space:{keyword}}}"),
                html
            )
            .unwrap()
            .riv
        );
    }
    for keyword in ["normal", "initial"] {
        assert_eq!(
            scene("p{display:block}", html).unwrap().riv,
            scene(
                &format!("#root{{white-space:pre-wrap}}p{{display:block;white-space:{keyword}}}"),
                html
            )
            .unwrap()
            .riv
        );
    }
    let tabs = scene("p{white-space:pre-wrap}", "<p>one\ttwo</p>").unwrap();
    tabs.runtime_requirements
        .ensure_supported(&[
            RuntimeCapability::LayoutCssIntrinsicSizingV1, RuntimeCapability::TextCssShapingPrecisionV1,
            RuntimeCapability::TextCssPreWrapV1,
            RuntimeCapability::TextCssTabsV1,
            RuntimeCapability::TextCssWrappedTabsV1,
        ])
        .unwrap();
    assert_eq!(
        tabs.runtime_requirements
            .ensure_supported(&[
                RuntimeCapability::LayoutCssIntrinsicSizingV1, RuntimeCapability::TextCssShapingPrecisionV1,
                RuntimeCapability::TextCssPreWrapV1,
                RuntimeCapability::TextCssTabsV1,
            ])
            .unwrap_err()
            .code,
        "missing-runtime-capability",
        "older hosts must reject the new combination"
    );
    let mut invalid_tabs = tabs.runtime_requirements.clone();
    invalid_tabs
        .capabilities
        .remove(&RuntimeCapability::TextCssTabsV1);
    assert_eq!(
        invalid_tabs
            .ensure_supported(&[
                RuntimeCapability::LayoutCssIntrinsicSizingV1, RuntimeCapability::TextCssShapingPrecisionV1,
                RuntimeCapability::TextCssPreWrapV1,
                RuntimeCapability::TextCssWrappedTabsV1,
            ])
            .unwrap_err()
            .code,
        "invalid-text-policies"
    );
    let manifest = serde_json::to_value(tabs.runtime_requirements).unwrap();
    assert!(
        manifest["capabilities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c == "text-css-wrapped-tabs-v1")
    );
}

#[test]
fn text_occurrence_policies_are_explicit_and_checked() {
    use nuxie_html_to_riv::{RuntimeCapability, RuntimeRequirements};
    let mut source = input(
        "<div><p id=a>nowrap</p><p id=b>normal</p><p id=c>pre</p></div>",
        "#a{white-space:nowrap}#c{white-space:pre}",
    );
    source.assets.insert(
        "inter".into(),
        Asset::Font {
            family: "Inter".into(),
            weight: 400,
            bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
        },
    );
    let output = compile(&source).unwrap();
    let mut visited = Vec::new();
    output
        .runtime_requirements
        .ensure_text_targets(|id| {
            visited.push(id);
            true
        })
        .unwrap();
    assert_eq!(visited.len(), 3);
    assert_eq!(
        output
            .runtime_requirements
            .ensure_text_targets(|_| false)
            .unwrap_err()
            .code,
        "invalid-text-policy-target"
    );
    let encoded = serde_json::to_value(&output.runtime_requirements).unwrap();
    assert_eq!(encoded["version"], 11);
    let policies = encoded["text_policies"].as_array().unwrap();
    assert_eq!(policies.len(), 3, "normal text has its own wrap policy");
    assert_eq!(policies.iter().filter(|p|p["policy"]=="css-nowrap-alignment-v1").count(),2);
    assert_eq!(policies.iter().filter(|p|p["policy"]=="css-normal-wrap-v1").count(),1);
    assert!(policies[0]["object_id"].as_u64() < policies[1]["object_id"].as_u64());
    let supported = [
        RuntimeCapability::LayoutCssPaintOrderV1,
        RuntimeCapability::LayoutCssIntrinsicSizingV1, RuntimeCapability::TextCssShapingPrecisionV1,
        RuntimeCapability::TextCssNormalWrapV1,
        RuntimeCapability::TextCssNowrapAlignmentV1,
    ];
    output
        .runtime_requirements
        .ensure_supported(&supported)
        .unwrap();
    for invalid in [
        serde_json::json!({"version":2,"capabilities":["text-css-nowrap-alignment-v1"]}),
        serde_json::json!({"version":2,"capabilities":[],"text_policies":policies}),
        serde_json::json!({"version":1,"capabilities":["text-css-nowrap-alignment-v1"],"text_policies":policies}),
        serde_json::json!({"version":2,"capabilities":["text-css-nowrap-alignment-v1"],"text_policies":[policies[0],policies[0]]}),
    ] {
        let requirements: RuntimeRequirements = serde_json::from_value(invalid).unwrap();
        assert_eq!(
            requirements.ensure_supported(&supported).unwrap_err().code,
            "invalid-text-policies"
        );
    }
    let legacy: RuntimeRequirements = serde_json::from_value(serde_json::json!({
        "version":1,"capabilities":["text-css-nowrap-alignment-v1"]
    }))
    .unwrap();
    legacy.ensure_supported(&supported).unwrap();
}

#[test]
fn pre_line_preserves_newlines_and_collapses_spaces_tabs_with_break_identity() {
    let scene = |html: &str, css: &str| {
        let mut source = input(html, css);
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source)
    };
    let html = "<p id=a> \té  one \n\t two \t<br id=cut> \n three \t</p>";
    let compiled = scene(html, "p{display:block;white-space:pre-line}").unwrap();
    let canonical = scene(
        "<p id=a>é one\ntwo<br id=cut>\nthree</p>",
        "p{display:block;white-space:pre-wrap}",
    )
    .unwrap();
    assert_eq!(compiled.riv, canonical.riv);
    assert_eq!(compiled.source_map[0].text_breaks[0].text_offset, 9);
    let manifest = serde_json::to_value(&compiled.runtime_requirements).unwrap();
    assert_eq!(manifest["version"], 11);
    assert_eq!(
        manifest["capabilities"],
        serde_json::json!(["layout-css-intrinsic-sizing-v1", "text-css-shaping-precision-v1", "text-css-pre-line-v1"])
    );
    assert_eq!(manifest["text_policies"][0]["policy"], "css-pre-line-v1");
    for keyword in ["inherit", "unset"] {
        assert_eq!(
            scene("<div><p> a \n b </p></div>", "p{white-space:pre-line}")
                .unwrap()
                .riv,
            scene(
                "<div><p> a \n b </p></div>",
                &format!("div{{white-space:pre-line}}p{{white-space:{keyword}}}")
            )
            .unwrap()
            .riv,
        );
    }
    for keyword in ["normal", "initial"] {
        assert_eq!(
            scene("<div><p> a \n b </p></div>", "").unwrap().riv,
            scene(
                "<div><p> a \n b </p></div>",
                &format!("div{{white-space:pre-line}}p{{white-space:{keyword}}}")
            )
            .unwrap()
            .riv,
        );
    }
    assert_eq!(
        scene("<p>a\r\nb</p>", "p{white-space:pre-line}")
            .unwrap()
            .riv,
        scene("<p>a\nb</p>", "p{white-space:pre-line}").unwrap().riv
    );
    for html in ["<p> \t </p>", "<p></p>", "<p> \n \n </p>"] {
        scene(html, "p{display:block;white-space:pre-line}").unwrap();
    }
}

#[test]
fn case_transforms_preserve_source_and_map_expanding_break_offsets() {
    let scene = |html: &str, css: &str| {
        let mut source = input(html, css);
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source)
    };
    let output = scene(
        "<p id=a>straße<br id=cut>café</p>",
        "p{display:block;text-transform:uppercase}",
    )
    .unwrap();
    let canonical = scene("<p id=a>STRASSE<br id=cut>CAFÉ</p>", "p{display:block}").unwrap();
    assert_eq!(output.riv, canonical.riv);
    assert_eq!(output.source_map[0].text_breaks[0].text_offset, 7);
    let map = serde_json::to_value(&output.source_map[0]).unwrap();
    assert_eq!(map["text_transform"]["source"], "straße\ncafé");
    assert_eq!(map["text_transform"]["rendered"], "STRASSE\nCAFÉ");
    assert_eq!(
        map["text_transform"]["scalar_offsets"],
        serde_json::json!([0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12])
    );
    assert_eq!(
        scene("<p>ΟΣ ΟΣΑ İ</p>", "p{text-transform:lowercase}")
            .unwrap()
            .riv,
        scene("<p>ος οσα i\u{307}</p>", "").unwrap().riv
    );
    for mode in ["inherit", "unset"] {
        assert_eq!(
            scene(
                "<div><p>café</p></div>",
                &format!("div{{text-transform:uppercase}}p{{text-transform:{mode}}}")
            )
            .unwrap()
            .riv,
            scene("<div><p>CAFÉ</p></div>", "").unwrap().riv
        );
    }
    for mode in ["initial", "none"] {
        let output = scene(
            "<div><p>café</p></div>",
            &format!("div{{text-transform:uppercase}}p{{text-transform:{mode}}}"),
        )
        .unwrap();
        assert_eq!(output.riv, scene("<div><p>café</p></div>", "").unwrap().riv);
        assert!(
            serde_json::to_value(output.source_map).unwrap()[1]
                .get("text_transform")
                .is_none()
        );
    }
}

#[test]
fn capitalize_follows_browser_boundaries_and_preserves_other_letters() {
    for (source, expected) in [
        ("hello WORLD café", "Hello WORLD Café"),
        ("straße ßETA ǳUNGLE", "Straße ßETA ǲUNGLE"),
        (
            "hello-world foo_bar foo.bar foo/bar",
            "Hello-World Foo_bar Foo.Bar Foo/Bar",
        ),
        ("don't o'neill l’esprit", "Don't O'neill L’esprit"),
        (
            "2fast 123abc abc123def 3.14abc",
            "2fast 123abc Abc123def 3.14abc",
        ),
        ("a:b a;b a,b a!b a?b", "A:B A;B A,B A!B A?B"),
        ("a\u{a0}b a\u{2003}b", "A\u{a0}B A\u{2003}B"),
    ] {
        let scene = |text: &str, css: &str| {
            let mut source = input(&format!("<p>{text}</p>"), css);
            source.assets.insert(
                "inter".into(),
                Asset::Font {
                    family: "Inter".into(),
                    weight: 400,
                    bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
                },
            );
            compile(&source)
        };
        let output = scene(source, "p{text-transform:capitalize}").unwrap();
        assert!(
            output.riv == scene(expected, "").unwrap().riv,
            "Rive bytes differ for {source}"
        );
        let mapping = output.source_map[0].text_transform.as_ref().unwrap();
        assert_eq!(mapping.source, source);
        assert_eq!(mapping.rendered, expected);
        assert_eq!(
            mapping.scalar_offsets,
            (0..=source.chars().count() as u32).collect::<Vec<_>>()
        );
    }
}

#[test]
fn language_inheritance_and_contextual_case_offsets_are_preserved() {
    let compile_text = |html: &str, css: &str| {
        let mut source = input(html, css);
        source.assets.insert(
            "inter".into(),
            Asset::Font {
                family: "Inter".into(),
                weight: 400,
                bytes: include_bytes!("assets/Inter-Regular.ttf").to_vec(),
            },
        );
        compile(&source).unwrap()
    };
    let output = compile_text(
        "<div lang=tr><p id=a>I&#x307;<br id=b lang=en>İ</p><p id=c lang=''>İ</p><p id=d lang=en>I</p></div>",
        "p{display:block;text-transform:lowercase}",
    );
    let map = &output.source_map[1];
    let trace = map.text_transform.as_ref().unwrap();
    assert_eq!(trace.source, "I\u{307}\nİ");
    assert_eq!(trace.rendered, "i\ni");
    assert_eq!(trace.scalar_offsets, vec![0, 1, 1, 2, 3]);
    assert_eq!(map.text_breaks[0].text_offset, 1);
    assert_eq!(
        output.source_map[2]
            .text_transform
            .as_ref()
            .unwrap()
            .rendered,
        "i\u{307}"
    );
    assert_eq!(
        output.source_map[3]
            .text_transform
            .as_ref()
            .unwrap()
            .rendered,
        "i"
    );
    let output = compile_text("<p lang=lt>I&#x301;</p>", "p{text-transform:lowercase}");
    let trace = output.source_map[0].text_transform.as_ref().unwrap();
    assert_eq!(trace.rendered, "i\u{307}\u{301}");
    assert_eq!(trace.scalar_offsets, vec![0, 2, 3]);
    let expected = compile_text("<p>i&#x307;&#x301;</p>", "");
    assert!(
        output.riv == expected.riv,
        "locale casing must produce canonical scene bytes"
    );
}

#[test]
fn japanese_reference_font_covers_all_oracle_text_without_fallback() {
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("../validation/width-kana-reference.json")).unwrap();
    for case in reference["cases"].as_array().unwrap() {
        for key in ["source", "expected"] {
            let text = case[key].as_str().unwrap();
            let escaped = text.replace('&', "&amp;").replace('<', "&lt;");
            let mut source = input(
                &format!("<p>{escaped}</p>"),
                "p{font-family:Fixture;white-space:pre;font-size:24px;line-height:40px}",
            );
            source.assets.insert(
                "fixture".into(),
                Asset::Font {
                    family: "Fixture".into(),
                    weight: 400,
                    bytes: include_bytes!("assets/NuxieJapaneseFixture-Regular.otf").to_vec(),
                },
            );
            assert!(
                compile(&source).is_ok(),
                "Reference {} {key} must compile with the pinned font and no fallback",
                case["name"]
            );
        }
    }
}

#[test]
fn ellipsis_occurrence_contract_requires_precision_and_explicit_host_support() {
    use nuxie_html_to_riv::{RuntimeCapability, RuntimeRequirements};
    let valid = serde_json::json!({
        "version":2,
        "capabilities":["text-css-single-line-ellipsis-v1","text-css-shaping-precision-v1"],
        "text_policies":[{"object_id":8,"policy":"css-single-line-ellipsis-v1"}]
    });
    let supported = [RuntimeCapability::TextCssSingleLineEllipsisV1, RuntimeCapability::TextCssShapingPrecisionV1];
    let requirements: RuntimeRequirements = serde_json::from_value(valid.clone()).unwrap();
    requirements.ensure_supported(&supported).unwrap();
    assert_eq!(requirements.ensure_supported(&supported[1..]).unwrap_err().code, "missing-runtime-capability");
    requirements.ensure_text_targets(|id| id == 8).unwrap();
    assert_eq!(requirements.ensure_text_targets(|_| false).unwrap_err().code, "invalid-text-policy-target");
    for version in [1, 2] {
        let mut missing = valid.clone();
        missing["version"] = version.into();
        missing["text_policies"] = serde_json::json!([]);
        let requirements: RuntimeRequirements = serde_json::from_value(missing).unwrap();
        assert_eq!(requirements.ensure_supported(&supported).unwrap_err().code, "invalid-text-policies");
    }
    for capabilities in [serde_json::json!([]), serde_json::json!(["text-css-single-line-ellipsis-v1"]), serde_json::json!(["text-css-shaping-precision-v1"])] {
        let mut invalid = valid.clone();
        invalid["capabilities"] = capabilities;
        let requirements: RuntimeRequirements = serde_json::from_value(invalid).unwrap();
        assert_eq!(requirements.ensure_supported(&supported).unwrap_err().code, "invalid-text-policies");
    }
    let mut duplicate = valid;
    duplicate["text_policies"].as_array_mut().unwrap().push(serde_json::json!({"object_id":8,"policy":"css-single-line-ellipsis-v1"}));
    let requirements: RuntimeRequirements = serde_json::from_value(duplicate).unwrap();
    assert_eq!(requirements.ensure_supported(&supported).unwrap_err().code, "invalid-text-policies");
}
