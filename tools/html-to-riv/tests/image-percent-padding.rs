//! Compiler admission is separate from native visual qualification.
use nuxie_html_to_riv::{Asset, CompileInput, compile};
fn input(css: &str) -> CompileInput {
    let mut input = CompileInput { html: "<div id=parent><img id=image src=picture></div>".into(),
        css: format!("#parent{{width:200px;height:160px}}#image{{{css}}}"),
        width:390.,height:320.,assets:Default::default() };
    input.assets.insert("picture".into(), Asset::Image { bytes: include_bytes!("../fixtures/images/ordinary-r1/opaque.png").to_vec() });
    input
}
#[test]
fn public_percentage_padding_retains_ordinary_assets_determinism_and_computed_inheritance() {
    for css in ["width:120px;height:80px;padding:5%", "box-sizing:content-box;width:120px;height:80px;padding:5%",
        "box-sizing:content-box;width:50%;height:80px;padding:5% 0",
        "box-sizing:content-box;width:120px;height:50%;padding:0 5%"] {
        let request = input(css);
        let output = compile(&request).unwrap_or_else(|e|panic!("{css}: {e:?}"));
        assert_eq!(output,compile(&request).unwrap());
        let Asset::Image {bytes}=&request.assets["picture"];
        assert_eq!(output.riv.windows(bytes.len()).filter(|w|*w==bytes).count(),1);
    }
    let mut request = input("width:120px;height:80px;padding:inherit");
    request.css.push_str("#parent{padding:5%}");
    let inherited=compile(&request).unwrap();
    request.css=request.css.replace("padding:inherit","padding:5%");
    assert_eq!(inherited,compile(&request).unwrap());
}
#[test]
fn public_percentage_padding_keeps_evidenced_and_unresolved_boundaries() {
    for (css,row,message) in [
        ("box-sizing:content-box;width:50%;height:80px;padding:5%",false,"same-axis"),
        ("box-sizing:content-box;width:120px;height:50%;padding:5%",false,"same-axis"),
        ("width:auto;height:80px;padding:5%",true,"intrinsic-main"),
        ("width:auto;height:auto;align-self:flex-start;padding:5%",true,"intrinsic-main"),
        ("box-sizing:content-box;width:auto;height:auto;align-self:flex-start;padding:5%",true,"intrinsic-main"),
        ("box-sizing:content-box;width:120px;height:80px;padding:5%",true,"intrinsic-main"),
        ("box-sizing:content-box;width:120px;height:auto;padding:5%",true,"intrinsic-main"),
    ] {
        let mut request=input(css);
        if row {request.css.push_str("#parent{flex-direction:row}");}
        let error=compile(&request).unwrap_err();
        assert_eq!(error[0].code,"unsupported-target-semantics");
        assert!(error[0].message.contains(message),"{css}: {error:?}");
    }
}
#[test]
fn final_content_plus_percentage_padding_overflow_is_a_compile_only_diagnostic() {
    let mut bytes=Vec::new();
    {let mut encoder=png::Encoder::new(&mut bytes,1,8192);encoder.set_color(png::ColorType::Rgba);
    let mut writer=encoder.write_header().unwrap();writer.write_image_data(&vec![255;8192*4]).unwrap();}
    let mut request=input("box-sizing:content-box;padding:1000000% 0;align-self:stretch");
    request.assets.insert("picture".into(),Asset::Image{bytes});
    request.html=format!("{}<div id=parent><img id=image src=picture></div>{}","<div class=grow>".repeat(7),"</div>".repeat(7));
    request.css.push_str(".grow{width:1000000%}#parent{width:10000%;height:auto}");
    let error=compile(&request).unwrap_err();
    assert!(error[0].message.contains("content plus padding"),"{error:?}");
    request.css=request.css.replace("padding:1000000%","padding:100000%");
    assert!(compile(&request).is_ok());
}

#[test]
fn row_ratio_diagnostic_advice_covers_opposite_axis_padding() {
    let mut request = input("width:auto;height:80px;padding:5% 0");
    request.css.push_str("#parent{flex-direction:row}");
    let error = compile(&request).unwrap_err();
    assert!(error[0].message.contains("remove percentage padding"));
    request.css = request.css.replace("padding:5% 0", "padding:0");
    assert!(compile(&request).is_ok());
}
