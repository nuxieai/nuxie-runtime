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
    for css in ["padding:1px", "min-width:10px", "max-height:100px", "margin:auto", "align-self:baseline",
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
