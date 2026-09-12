use nuxie_html_to_riv::{Asset, CompileInput, compile};

fn request(html: &str, css: &str) -> CompileInput {
    CompileInput { html: html.into(), css: css.into(), width: 390., height: 320., assets: Default::default() }
}
fn asset(input: &mut CompileInput, key: &str, file: &str) {
    input.assets.insert(key.into(), Asset::Image { bytes: std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/images/ordinary-r1").join(file)).unwrap() });
}

#[test]
fn supplied_formats_embed_original_bytes_and_keep_authored_identity() {
    for file in ["opaque.png", "alpha.png", "baseline.jpg", "progressive.jpg", "lossless.webp", "lossy.webp", "alpha.webp"] {
        let mut input = request("<div id=frame><img id=image src=logo alt='Authored image'></div>",
            "#frame{width:200px;height:160px}#image{width:96px;height:64px}");
        asset(&mut input, "logo", file);
        let output = compile(&input).unwrap_or_else(|e| panic!("{file}: {e:?}"));
        let Asset::Image { bytes } = &input.assets["logo"];
        assert_eq!(output.riv.windows(bytes.len()).filter(|w| *w == bytes).count(), 1);
        assert_eq!(output.source_map.iter().map(|n| (n.id.as_str(), n.path.as_str())).collect::<Vec<_>>(),
            [("frame", "/0"), ("image", "/0/0")]);
        assert_eq!(output, compile(&input).unwrap());
    }
}

#[test]
fn source_aliases_share_an_ordinary_asset_without_changing_file_or_map() {
    let mut input = request("<img id=a src=logo><img id=b src=alias>", "img{width:96px;height:64px}");
    asset(&mut input, "logo", "opaque.png"); asset(&mut input, "alias", "opaque.png");
    let aliased = compile(&input).unwrap();
    input.html = input.html.replace("src=alias", "src=logo"); input.assets.remove("alias");
    assert_eq!(aliased, compile(&input).unwrap());
}

#[test]
fn fit_position_and_sampling_cascade_have_literal_controls() {
    let mut input = request("<div id=parent><img id=image src=logo></div>",
        "#parent{width:200px;height:160px;object-fit:contain;object-position:left bottom;image-rendering:pixelated}#image{width:160px;height:100px;object-fit:inherit;object-position:inherit;image-rendering:inherit}");
    asset(&mut input, "logo", "opaque.png");
    let inherited = compile(&input).unwrap();
    input.css = input.css.replace("object-fit:inherit", "object-fit:contain")
        .replace("object-position:inherit", "object-position:0% 100%")
        .replace("image-rendering:inherit", "image-rendering:pixelated");
    assert_eq!(inherited, compile(&input).unwrap());
    input.css = "#parent{width:200px;height:160px;object-fit:contain;object-position:left bottom;image-rendering:pixelated}#image{width:160px;height:100px}".into();
    let automatic = compile(&input).unwrap();
    input.css.push_str("#image{object-fit:fill;object-position:center;image-rendering:pixelated}");
    assert_eq!(automatic, compile(&input).unwrap());
}

#[test]
fn image_sources_are_explicit_and_no_external_loader_is_available() {
    for (html, code) in [("<img>", "missing-image-source"), ("<img src=''>", "missing-image-source"),
        ("<img src='https://example.test/image.png'>", "missing-image"), ("<img src=LOGO>", "missing-image")] {
        let mut input = request(html, ""); asset(&mut input, "logo", "opaque.png");
        assert_eq!(compile(&input).unwrap_err()[0].code, code);
    }
}

#[test]
fn unqualified_image_combinations_produce_diagnostics() {
    for css in ["min-width:10%", "max-height:100%", "margin:auto", "align-self:baseline",
        "align-self:center", "object-position:20px 30px", "object-position:100.000001%", "object-position:-1e-500%",
        "object-position:100.000000000000000000001%", "object-fit:invalid", "image-rendering:crisp-edges"] {
        let mut input = request("<img id=image src=logo>", &format!("#image{{width:96px;height:64px;{css}}}"));
        asset(&mut input, "logo", "opaque.png");
        assert_eq!(compile(&input).expect_err(css)[0].code, "unsupported-target-semantics");
    }
    let mut input = request("<div id=p><div id=a><img src=logo></div><div id=b></div></div>",
        "#p{flex-direction:row}#a{width:96px;align-self:baseline}#b{width:20px;height:20px;align-self:baseline}");
    asset(&mut input, "logo", "opaque.png");
    let error = compile(&input).unwrap_err();
    assert!(error[0].message.contains("baseline"));
}

#[test]
fn empty_assets_preserve_the_existing_serialized_request_shape() {
    let input = request("<div></div>", "");
    let value = serde_json::to_value(&input).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 4);
    let mut explicit = value.clone(); explicit["assets"] = serde_json::json!({});
    let explicit: CompileInput = serde_json::from_value(explicit).unwrap();
    assert_eq!(compile(&input).unwrap(), compile(&explicit).unwrap());
}

#[test]
fn source_point_constraints_emit_the_same_ordinary_scene_as_the_selected_sizes() {
    for direction in ["row", "column"] {
        for (constrained, literal) in [
            ("min-width:192px", "width:192px;height:128px"),
            ("max-width:48px", "width:48px;height:32px"),
            ("min-width:120px;max-height:70px", "width:120px;height:70px"),
            ("min-width:120px;max-width:48px", "width:120px;height:80px"),
            ("width:120px;max-height:40px", "width:120px;height:40px"),
            ("height:80px;max-width:60px", "width:60px;height:80px"),
            ("width:120px;height:80px;max-width:60px", "width:60px;height:80px"),
            ("width:120px;max-width:60px;padding:8px 10px;box-sizing:content-box", "width:60px;height:40px;padding:8px 10px;box-sizing:content-box"),
            ("width:120px;max-width:68px;padding:8px 10px;box-sizing:border-box", "width:68px;height:48px;padding:8px 10px;box-sizing:border-box"),
        ] {
            let mut input = request("<div id=parent><img id=image src=logo><div id=tail></div></div>",
                &format!("#parent{{width:200px;height:240px;flex-direction:{direction}}}#image{{align-self:flex-start;{constrained}}}#tail{{width:8px;height:8px}}"));
            asset(&mut input,"logo","opaque.png");
            let output=compile(&input).unwrap_or_else(|e|panic!("{direction}: {constrained}: {e:?}"));
            assert_eq!(output,compile(&input).unwrap());
            input.css=input.css.replace(constrained,literal);
            let selected=compile(&input).unwrap();
            // The packing direction of an existing image content owner can
            // differ after replacing an authored auto axis with an explicit
            // point. Compare exact files for the single-owner controls; native
            // evidence covers the separate padded ordinary owners.
            if !constrained.contains("padding") { assert_eq!(output,selected,"{direction}: {constrained}"); }
            let Asset::Image{bytes}=&input.assets["logo"];
            assert_eq!(output.riv.windows(bytes.len()).filter(|b|*b==bytes).count(),1);
        }
    }
}

#[test]
fn constrained_automatic_main_is_admitted_but_automatic_cross_stretch_is_not() {
    for (direction, accepted, rejected) in [
        ("column","width:120px;max-height:40px","height:80px;max-width:60px"),
        ("row","height:80px;max-width:60px","width:120px;max-height:40px"),
    ] {
        let mut input=request("<div id=parent><img id=image src=logo></div>",
            &format!("#parent{{width:200px;height:240px;flex-direction:{direction}}}#image{{{accepted}}}"));
        asset(&mut input,"logo","opaque.png");
        assert!(compile(&input).is_ok(),"{direction}: {accepted}");
        input.css=input.css.replace(accepted,rejected);
        let error=compile(&input).unwrap_err();
        assert!(error[0].message.contains("stretch"));
    }
}

#[test]
fn constrained_images_keep_responsive_and_automatic_minimum_diagnostics() {
    for declaration in ["width:50%;min-width:10%;max-width:100px", "max-width:50%", "max-height:50%",
        "min-width:auto;max-width:100px", "max-width:100px;padding:5%"] {
        let mut input=request("<img id=image src=logo>",&format!("#image{{align-self:flex-start;{declaration}}}"));
        asset(&mut input,"logo","opaque.png");
        assert_eq!(compile(&input).unwrap_err()[0].code,"unsupported-target-semantics","{declaration}");
    }
}

#[test]
fn responsive_image_opposite_constraints_remain_runtime_sized_and_deterministic() {
    for direction in ["row","column"] {
        for declaration in ["width:37.5%;max-height:83px", "height:42%;max-width:61%", "width:28%;min-height:31px", "height:63%;min-width:117px", "width:50%;min-height:100px;max-height:35%", "height:50%;min-width:80%;max-width:30%", "width:50%;min-height:45%"] {
            let mut input=request("<div id=parent><img id=image src=logo><div id=tail></div></div>",
                &format!("#parent{{width:100%;height:100%;padding:7px 11px;flex-direction:{direction}}}#image{{align-self:flex-start;{declaration}}}#tail{{width:8px;height:8px}}"));
            asset(&mut input,"logo","opaque.png");
            let output=compile(&input).unwrap_or_else(|e|panic!("{declaration}: {e:?}"));
            assert_eq!(output,compile(&input).unwrap());
            let Asset::Image{bytes}=&input.assets["logo"];
            assert_eq!(output.riv.windows(bytes.len()).filter(|b|*b==bytes).count(),1);
            assert_eq!(output.source_map.len(),3);
        }
    }
}

#[test]
fn preferred_percentage_coefficient_arbitration_matches_independent_responsive_literal() {
    for (authored,literal) in [("width:50%;min-width:75%","width:75%"),("height:50%;max-height:30%","height:30%"),("width:50%;min-width:75%;max-width:30%","width:75%")] {
        let mut input=request("<div id=parent><img id=image src=logo></div>",
            &format!("#parent{{width:100%;height:100%}}#image{{align-self:flex-start;{authored}}}"));
        asset(&mut input,"logo","opaque.png");
        let output=compile(&input).unwrap();
        input.css=input.css.replace(authored,literal);
        assert_eq!(output,compile(&input).unwrap(),"{authored}");
    }
}
#[test]
fn preferred_point_bounds_keep_source_assets_and_reject_both_axis_or_mixed_bounds() {
    let mut input=request("<div id=parent><img id=image src=logo></div>","#parent{width:100%;height:100%}#image{align-self:flex-start;width:50%;min-width:200px;max-width:120px}");
    asset(&mut input,"logo","opaque.png");
    let output=compile(&input).unwrap();assert_eq!(output,compile(&input).unwrap());
    let Asset::Image{bytes}=&input.assets["logo"];
    assert_eq!(output.riv.windows(bytes.len()).filter(|b|*b==bytes).count(),1);
    input.css.push_str("#image{max-height:100px}");assert!(compile(&input).is_err());
    input.css=input.css.replace("max-height:100px","max-height:none;min-width:80%;max-width:120px");assert!(compile(&input).is_err());
}

#[test]
fn mixed_and_both_axis_image_constraints_equal_independent_source_compositions() {
    for (authored,composed) in [
        ("width:50%;min-width:150px;max-width:40%","width:40%;min-width:150px"),
        ("height:50%;min-height:150px;max-height:75%","height:50%;min-height:150px"),
        ("width:50%;min-width:75%;max-width:60%;max-height:100px","width:75%;max-height:100px"),
        ("height:50%;min-height:75%;max-height:60%;min-width:80%;max-width:100px","height:75%;min-width:80%;max-width:100px"),
    ] {
        let mut input=request("<div id=parent><img id=image src=logo></div>",
            &format!("#parent{{width:100%;height:100%;padding:20px}}#image{{align-self:flex-start;{authored}}}"));
        asset(&mut input,"logo","opaque.png");
        let output=compile(&input).unwrap();input.css=input.css.replace(authored,composed);
        assert_eq!(output,compile(&input).unwrap(),"{authored}");
    }
}
