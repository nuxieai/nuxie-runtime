use nuxie_html_to_riv::{compile,CompileInput};
fn nested(depth:usize,css:&str,width:f32)->CompileInput {
    CompileInput { assets: Default::default(),html:format!("{}{}",(0..depth).map(|i|format!("<div id=n{i}>")).collect::<String>(),"</div>".repeat(depth)),css:format!("div{{width:1000000%;height:1px}}{css}"),width,height:32.}
}
#[test]
fn reproduced_nested_overflow_rejects_across_the_entire_viewport_domain() {
    for viewport in [1.,240.,16384.] {
        let errors=compile(&nested(9,"",viewport)).unwrap_err();
        assert_eq!(errors[0].code,"unsupported-target-semantics");
        assert!(errors[0].message.contains("binary32"));
    }
}
#[test]
fn large_finite_chains_and_effective_clamps_are_retained() {
    assert!(compile(&nested(8,"",16384.)).is_ok());
    assert!(compile(&nested(30,"div{max-width:1000px}",16384.)).is_ok());
    assert!(compile(&nested(30,"div{max-width:1px;min-width:20000px}",16384.)).is_ok());
    assert!(compile(&nested(9,"#n0{width:1px}",16384.)).is_ok());
}
#[test]
fn finite_max_caps_preferred_overflow_but_minimum_wins() {
    assert!(compile(&nested(9,"#n8{max-width:1px}",16384.)).is_ok());
    let mut minimum=nested(9,"div{width:1px;min-width:1000000%;max-width:1px}",16384.);
    assert!(compile(&minimum).is_err());
    minimum.css="div{width:0px;height:1px;max-width:1000000%}".into();
    assert!(compile(&minimum).is_ok());
}

#[test]
fn both_axes_zero_and_midchain_pixel_resets_follow_the_same_rules() {
    for vertical in [false,true] {
        let convert=|mut request:CompileInput| {
            if vertical { request.css=request.css.replace("width","AXIS").replace("height","width").replace("AXIS","height"); }
            request
        };
        assert!(compile(&convert(nested(9,"",1.))).is_err());
        assert!(compile(&convert(nested(9,"#n0{width:0px}",16384.))).is_ok());
        assert!(compile(&convert(nested(9,"#n4{width:1px}",16384.))).is_ok());
        assert!(compile(&convert(nested(9,"#n8{max-width:100px}",16384.))).is_ok());
    }
}

#[test]
fn automatic_minimum_cannot_hide_known_preferred_overflow() {
    for vertical in [false,true] {
        let convert=|mut request:CompileInput| {
            if vertical { request.css=request.css.replace("width","AXIS").replace("height","width").replace("AXIS","height"); }
            request
        };
        assert!(compile(&convert(nested(8,"div{min-width:auto}",16384.))).is_ok());
        assert!(compile(&convert(nested(9,"div{min-width:auto}",16384.))).is_err());
        // A finite max can cap preferred infinity, but an automatic minimum
        // leaves the upper bound unknown rather than claiming a fixed size.
        assert!(compile(&convert(nested(9,"div{min-width:auto}#n8{max-width:100px}",16384.))).is_ok());
    }
}
