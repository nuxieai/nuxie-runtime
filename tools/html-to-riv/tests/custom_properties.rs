use nuxie_html_to_riv::{compile,CompileInput};
fn input(css:&str)->CompileInput {CompileInput{html:"<div id=root><div id=a></div><div id=b style='--local: 35px'></div></div>".into(),css:format!("#root{{width:100%;gap:8px}}#a,#b{{height:20px;background:red}}{css}"),width:390.0,height:200.0,..Default::default()}}
fn equal(actual:&str,expected:&str){assert!(compile(&input(actual)).unwrap().riv==compile(&input(expected)).unwrap().riv,"{actual}");}
#[test]
fn custom_values_cascade_inherit_and_keep_name_case() {
    equal("#root{--Width:40px;--width:60px}#a{width:var(--Width)}#b{width:var(--width)}","#a{width:40px}#b{width:60px}");
    equal("#root{--w:40px;--alias:var(--w)}#a{--w:80px;width:var(--alias)}#b{width:var(--local)}","#a{width:40px}#b{width:35px}");
    equal("#a{--w:40px!important}div{--w:60px}#a{width:var(--w)}#b{--local:70px!important;width:var(--local)}","#a{width:40px}#b{width:70px}");
    equal("#root{--w:40px}#a{--w:initial;width:var(--w,80px)}#b{--w:unset;width:var(--w)}","#a{width:80px}#b{width:40px}");
}
#[test]
fn cycles_short_circuit_unused_fallbacks_and_allow_external_recovery() {
    equal("#root{--safe:20px;--x:var(--safe,var(--y));--y:var(--x)}#a{width:var(--x,40px)}#b{width:var(--y,60px)}","#a{width:20px}#b{width:20px}");
    equal("#a{--x:var(--x,20px);width:var(--x,40px)}","#a{width:40px}");
}
#[test]
fn substitutions_preserve_shorthand_order_and_font_relative_lengths() {
    equal("#root{--p:8px 12px;--w:2em}#a{padding-left:30px;padding:var(--p);padding-right:20px;font-size:20px;width:var(--w)}","#a{padding:8px 12px;padding-right:20px;font-size:20px;width:40px}");
    equal("#a{--c:0,128,0;background:rgb(var(--c))}","#a{background:green}");
}
#[test]
fn missing_variables_reset_instead_of_restoring_earlier_values() {
    equal("#a{width:40px;width:var(--absent);background:var(--absent)}","#a{width:auto;background:transparent}");
    equal("#a{padding:20px;padding:var(--absent);padding-left:8px}","#a{padding:0;padding-left:8px}");
}
#[test]
fn inert_values_and_escaped_names_are_preserved_without_rendering_them() {
    equal(r"#a{--unused:foo('var(--bad)') [a,b] {c:d};--\77:40px;width:var(--w)}","#a{width:40px}");
    equal("#a{--empty:;--unused:'!important';width:var(--absent,var(--also-absent,40px))}","#a{width:40px}");
}
#[test]
fn unsupported_properties_and_rendering_features_still_fail() {
    for css in [".missing{grid-template-columns:var(--x)}","#a{--x:grid;display:var(--x)}","#a{--x:calc(10px + 2px);width:var(--x)}","#a{--:20px}","#a{--x:revert}",".missing{--x:revert-layer}"] {assert!(compile(&input(css)).is_err(),"{css}");}
}
#[test]
fn empty_substitution_is_invalid_and_does_not_use_fallback_or_previous_value() {
    for empty in ["", " ", "/**/"] {
        equal(&format!("#a{{--empty:{empty};width:40px;width:var(--empty,80px);background:var(--empty,green)}}"),"#a{width:auto;background:transparent}");
    }
    equal("#a{width:40px;width:var(--absent,)}","#a{width:auto}");
    equal("#a{--empty:;padding:20px;padding:var(--empty);padding-left:8px}","#a{padding:0;padding-left:8px}");
    equal("#root{color:green}#a{--empty:;color:red;color:var(--empty,blue);background:currentcolor}","#root{color:green}#a{background:green}");
    equal("#a{--empty:;width:40px;width:var(--empty);width:60px}","#a{width:60px}");
}
#[test]
fn invalid_substituted_sizes_and_shorthands_reset_without_using_fallbacks() {
    for invalid in ["20", "-10px", "-20%", "'40px'", "10px 20px"] {
        equal(&format!("#a{{--bad:{invalid};width:40px;width:var(--bad,80px)}}"),"#a{width:auto}");
    }
    equal("#a{--bad:-2px;padding:20px;padding:var(--bad);padding-left:8px}","#a{padding:0;padding-left:8px}");
    equal("#a{--bad:1px 2px 3px 4px 5px;margin:20px;margin:var(--bad)}","#a{margin:0}");
    equal("#a{--n:20;width:var(--n)px}","#a{width:auto}");
}
#[test]
fn invalid_substituted_color_tokens_use_inheritance_or_initial() {
    for invalid in ["20px", "4", "50%", "'red'", "#ggg", "red blue"] {
        equal(&format!("#root{{color:green}}#a{{--bad:{invalid};color:red;color:var(--bad,blue);background:currentcolor}}"),"#root{color:green}#a{background:green}");
        equal(&format!("#a{{--bad:{invalid};background-color:var(--bad,green)}}"),"#a{background:transparent}");
    }
}
#[test]
fn invalidation_does_not_enable_unsupported_valid_css() {
    for css in ["#a{--x:10vw;width:var(--x)}","#a{--x:Canvas;color:var(--x)}","#a{--x:min-content;width:var(--x)}","#a{--x:calc(20px + 1px);width:var(--x)}"] {assert!(compile(&input(css)).is_err(),"{css}");}
}
#[test]
fn full_property_limit_handles_dependency_chains_in_either_name_order() {
    for reverse in [false,true] {
        let names:Vec<_>=(0..512).map(|i|format!("--n{:04}",if reverse{511-i}else{i})).collect();
        let mut css=String::new();
        for chunk in (0..512).collect::<Vec<_>>().chunks(128) {
            css.push_str("#root{");
            for &i in chunk {
                css.push_str(&format!("{}:{};",names[i],if i==511{"40px".into()}else{format!("var({})",names[i+1])}));
            }
            css.push('}');
        }
        css.push_str(&format!("#root{{width:var({});height:20px;background:green}}",names[0]));
        let make=|css:String|CompileInput{html:"<div id=root></div>".into(),css,width:390.0,height:200.0,..Default::default()};
        assert!(compile(&make(css)).unwrap().riv==compile(&make("#root{width:40px;height:20px;background:green}".into())).unwrap().riv);
    }
}
#[test]
fn nested_inert_values_across_long_chains_do_not_use_the_call_stack() {
    let mut css=String::new();
    for chunk in (0..512).collect::<Vec<_>>().chunks(128) {
        css.push_str("#root{");
        for &i in chunk {
            let inner=if i==511{"40px".into()}else{format!("var(--n{:04})",i+1)};
            css.push_str(&format!("--n{i:04}:{}{}{};","f(".repeat(32),inner,")".repeat(32)));
        }
        css.push('}');
    }
    css.push_str("#root{width:40px;height:20px;background:green}");
    let make=|css:String|CompileInput{html:"<div id=root></div>".into(),css,width:390.0,height:200.0,..Default::default()};
    assert!(compile(&make(css)).unwrap().riv==compile(&make("#root{width:40px;height:20px;background:green}".into())).unwrap().riv);
}
#[test]
fn property_count_and_expansion_limits_fail_or_recover_explicitly() {
    let make=|css:String|CompileInput{html:"<div id=root></div>".into(),css,width:390.0,height:200.0,..Default::default()};
    let mut css=String::new();
    for chunk in (0..513).collect::<Vec<_>>().chunks(128) {
        css.push_str("#root{");for &i in chunk {css.push_str(&format!("--n{i}:1px;"));}css.push('}');
    }
    let errors=compile(&make(css)).unwrap_err();assert_eq!(errors[0].code,"input-limit");
    let mut css=String::from("#root{--n0:1px;");
    for i in 1..24 {css.push_str(&format!("--n{i}:var(--n{}) var(--n{});",i-1,i-1));}
    css.push_str("width:var(--n23,60px);height:20px;background:green}");
    assert!(compile(&make(css)).unwrap().riv==compile(&make("#root{width:60px;height:20px;background:green}".into())).unwrap().riv);
}
#[test]
fn invalid_color_and_length_keywords_reset_but_valid_profile_exclusions_do_not() {
    for bad in ["banana","none","auto"] {
        equal(&format!("#root{{color:green}}#a{{--bad:{bad};color:var(--bad);background:currentcolor}}"),"#root{color:green}#a{background:green}");
        equal(&format!("#a{{--bad:{bad};background-color:var(--bad)}}"),"#a{background:transparent}");
    }
    for (property,bad,reset) in [("width","banana","auto"),("width","none","auto"),("padding","auto","0"),("padding","inherit 2px","0"),("gap","auto","0"),("margin","banana","0")] {
        equal(&format!("#a{{--bad:{bad};{property}:var(--bad)}}"),&format!("#a{{{property}:{reset}}}"));
    }
    for css in ["#a{--x:ActiveBorder;color:var(--x)}","#a{--x:SelectedItem;background-color:var(--x)}","#a{--x:-webkit-text;color:var(--x)}","#a{--x:stretch;width:var(--x)}","#a{--x:content;flex-basis:var(--x)}"] {assert!(compile(&input(css)).is_err(),"{css}");}
    equal("#root{color:green}#a{color:var(--missing,inherit);background:currentcolor}","#root{color:green}#a{background:green}");
}
#[test]
fn invalid_scalar_color_functions_reset_after_substitution() {
    for bad in ["rgb()","rgb(1 2)","rgb(1,2,3,4,5)","rgb(1% 2 3 /)","rgb(1%,2,3)","hsl(20px 30% 40%)","hsl(1,2,3)","hsl(1 2 / 3)","rgba(1 2 3 / / 1)"] {
        equal(&format!("#root{{color:green}}#a{{--bad:{bad};color:var(--bad,blue);background:currentcolor}}"),"#root{color:green}#a{background:green}");
        equal(&format!("#a{{--bad:{bad};background:var(--bad,green)}}"),"#a{background:transparent}");
    }
    equal("#a{--channels:1 2;background-color:rgb(var(--channels))}","#a{background:transparent}");
}
#[test]
fn valid_but_unimplemented_color_functions_keep_diagnostics() {
    for bad in ["rgb(none 20 30)","hsl(none 20% 30%)","rgb(from red r g b)","rgb(calc(1 + 2) 0 0)","color(display-p3 1 0 0)"] {
        assert!(compile(&input(&format!("#a{{--x:{bad};color:var(--x)}}"))).is_err(),"{bad}");
    }
    equal("#a{--x:rgb(300 -10 0);background:var(--x)}","#a{background:red}");
}
#[test]
fn invalid_flex_and_overflow_substitutions_reset_to_initial() {
    for bad in ["banana","20px","0","row column"] {
        equal(&format!("#root{{--bad:{bad};flex-direction:column;flex-direction:var(--bad)}}#a,#b{{width:40px}}"),"#root{flex-direction:row}#a,#b{width:40px}");
        equal(&format!("#root{{--bad:{bad};flex-wrap:wrap;flex-wrap:var(--bad)}}"),"#root{flex-wrap:nowrap}");
    }
    for bad in ["banana","20px","visible clip hidden"] {
        equal(&format!("#root{{--bad:{bad};height:10px;overflow:hidden;overflow:var(--bad)}}"),"#root{height:10px;overflow:visible}");
        equal(&format!("#a{{--bad:{bad};text-overflow:var(--bad)}}"),"#a{text-overflow:clip}");
    }
}
#[test]
fn valid_excluded_flex_and_overflow_values_keep_diagnostics() {
    equal("#root{--value:hidden clip;overflow:var(--value)}", "#root{overflow:hidden}");
    for (property,value) in [("overflow","scroll"),("overflow","auto"),("overflow","hidden visible"),("text-overflow","'custom'"),("text-overflow","clip ellipsis"),("text-overflow","fade")] {
        assert!(compile(&input(&format!("#root{{--value:{value};{property}:var(--value)}}"))).is_err(),"{property}:{value}");
    }
}
#[test]
fn invalid_flex_factors_reset_without_restoring_previous_factors() {
    for bad in ["-1","20px","20%","banana","'2'","1 2"] {
        equal(&format!("#root{{flex-direction:row}}#a{{--bad:{bad};width:40px;flex-grow:2;flex-grow:var(--bad)}}#b{{width:40px}}"),"#root{flex-direction:row}#a,#b{width:40px}");
        equal(&format!("#root{{flex-direction:row;width:60px}}#a{{--bad:{bad};width:40px;flex-grow:1;flex-shrink:0;flex-shrink:var(--bad)}}#b{{width:40px}}"),"#root{flex-direction:row;width:60px}#a,#b{width:40px}#a{flex-grow:1;flex-shrink:1}");
    }
    equal("#a{height:auto;--bad:-1;flex-grow:var(--bad);flex-shrink:1}","#a{height:auto;flex-grow:0;flex-shrink:1}");
    equal("#a{width:40px;height:40px;--bad:-1;flex-grow:var(--bad);flex-shrink:1}","#a{width:40px;height:40px;flex-grow:0;flex-shrink:1}");
    equal("#a{--factor:2.5;flex-grow:var(--factor);flex-shrink:var(--factor)}","#a{flex-grow:2.5;flex-shrink:2.5}");
}

#[test]
fn invalid_flex_shorthand_resets_all_components_before_later_longhands() {
    for bad in ["banana", "-1", "1 -2", "20px 30px", "1 2 3px 4", "1 20px 2", "none 1", "'auto'"] {
        equal(&format!("#a{{--bad:{bad};width:40px;flex:2 2 80px;flex:var(--bad,none);flex-shrink:0}}"), "#a{width:40px;flex:0 0 auto}");
        equal(&format!("#a{{--bad:{bad};width:40px;flex:var(--bad)!important;flex-shrink:0!important;flex:2 2 80px}}"), "#a{width:40px;flex:0 0 auto}");
    }
    equal("#a{--f:2 2 30%;flex-basis:80px;flex:var(--f);flex-basis:40px}","#a{flex:2 2 40px}");
    for value in ["1 1 min-content", "1 1 20vw"] {
        assert!(compile(&input(&format!("#a{{--f:{value};flex:var(--f);flex-shrink:0}}"))).is_err(), "{value}");
    }
}

#[test]
fn invalid_alignment_substitutions_reset_or_inherit() {
    for bad in ["banana", "20px", "0", "center center", "safe stretch", "first center", "center safe"] {
        equal(&format!("#root{{--bad:{bad};height:100px;align-items:center;align-items:var(--bad)}}"),"#root{height:100px;align-items:stretch}");
        equal(&format!("#root{{--bad:{bad};height:100px;justify-content:center;justify-content:var(--bad)}}"),"#root{height:100px;justify-content:flex-start}");
    }
    for bad in ["banana", "20px", "0", "left right"] {
        equal(&format!("#root{{text-align:right}}#a{{--bad:{bad};text-align:center;text-align:var(--bad)}}"),"#root{text-align:right}#a{text-align:right}");
    }
}
#[test]
fn valid_excluded_alignment_values_keep_diagnostics() {
    for (property,value) in [("align-items","first baseline"),("align-items","safe center"),("align-items","anchor-center"),("justify-content","unsafe right"),("text-align","justify"),("text-align","match-parent")] {
        assert!(compile(&input(&format!("#root{{--x:{value};{property}:var(--x)}}"))).is_err(),"{property}:{value}");
    }
}

#[test]
fn invalid_radius_substitutions_reset_without_using_fallback() {
    for bad in ["-1px", "-2%", "2", "banana", "'8px'", "1px 2px 3px 4px 5px", "/ 2px", "2px /", "2px / 3px / 4px", "2px / -3px", "2px,3px"] {
        equal(&format!("#a{{--r:{bad};border-radius:12px;border-radius:var(--r,8px)}}"),"#a{border-radius:0}");
    }
    equal("#a{--r:8px;border-radius:var(--r)}","#a{border-radius:8px}");
    for value in ["8px 4px", "8px 4px 12px", "8px 4px 12px 16px", "8px / 4px", "20%"] {
        equal(&format!("#a{{--r:{value};border-radius:var(--r)}}"), &format!("#a{{border-radius:{value}}}"));
    }
    for value in ["2vw", "calc(2px + 3px)"] {
        assert!(compile(&input(&format!("#a{{--r:{value};border-radius:var(--r)}}"))).is_err(),"{value}");
    }
}

fn text_input(css: &str) -> CompileInput {
    CompileInput {
        html: "<div id=root><div id=a>Words that wrap at a narrow width</div></div>".into(),
        css: format!("#root{{width:100%;font-family:Inter;font-size:20px;line-height:1.5}}#a{{display:block}}{css}"),
        width: 390.0, height: 320.0,
        assets: [("font".into(),nuxie_html_to_riv::Asset::Font{family:"Inter".into(),weight:400,bytes:include_bytes!("assets/Inter-Regular.ttf").to_vec()})].into(),
    }
}
#[test]
fn invalid_font_longhands_inherit_in_real_text() {
    for (property,prior,bad) in [("font-size","32px","-2px"),("font-size","32px","20"),("font-size","32px","banana"),("font-size","32px","10px 20px"),("font-weight","700","0"),("font-weight","700","1001"),("font-weight","700","400px"),("font-weight","700","banana"),("line-height","2","-1"),("line-height","2","-20%"),("line-height","2","banana"),("line-height","2","1 2")] {
        let actual=compile(&text_input(&format!("#a{{--x:{bad};{property}:{prior};{property}:var(--x)}}"))).unwrap();
        let expected=compile(&text_input("#a{font-size:inherit;font-weight:inherit;line-height:inherit}")).unwrap();
        assert!(actual.riv==expected.riv,"{property}:{bad}");
    }
}
#[test]
fn valid_excluded_font_longhands_do_not_become_inherited() {
    for (property,value) in [("font-size","0"),("font-size","120%"),("font-size","larger"),("font-weight","1"),("font-weight","1000"),("font-weight","400.5"),("font-weight","bolder"),("line-height","0"),("line-height","120%"),("line-height","calc(20px + 2px)")] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};{property}:var(--x)}}"))).is_err(),"{property}:{value}");
    }
}

#[test]
fn invalid_font_family_lists_inherit_in_real_text() {
    for bad in ["20px", "400", "Inter,", ",Inter", "Inter,,serif", "'Inter' Other", "Inter 'Other'", "Inter / Other", "default"] {
        let actual=compile(&text_input(&format!("#a{{--x:{bad};font-family:Missing;font-family:var(--x,Missing)}}"))).unwrap();
        let expected=compile(&text_input("#a{font-family:inherit}")).unwrap();
        assert!(actual.riv==expected.riv,"{bad}");
    }
    for value in ["Inter, serif", "Some Family", "'default'", "'Missing'", "serif"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};font-family:var(--x)}}"))).is_err(),"{value}");
    }
    let actual=compile(&text_input("#a{--x:'Inter';font-family:var(--x)}")).unwrap();
    assert!(actual.riv==compile(&text_input("#a{font-family:Inter}")).unwrap().riv);
}

#[test]
fn font_family_keyword_positions_follow_chromium() {
    for bad in ["serif Other", "system-ui Other", "serif serif", "default,Inter", "Inter,initial"] {
        let actual=compile(&text_input(&format!("#a{{--x:{bad};font-family:var(--x)}}"))).unwrap();
        assert!(actual.riv==compile(&text_input("#a{font-family:inherit}")).unwrap().riv,"{bad}");
    }
    for value in ["Inter inherit", "default Inter", "Other serif", "caption Other", "Other system-ui"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};font-family:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn leading_wide_words_in_substituted_family_names_inherit() {
    for value in ["inherit Inter","initial Inter","unset Inter","revert Inter","revert-layer Inter","INHERIT Inter","inherit/**/Inter"] {
        for declaration in [format!("--x:{value};font-family:var(--x)"),format!("font-family:var(--missing,{value})")] {
            let actual=compile(&text_input(&format!("#a{{font-family:Missing;{declaration}}}"))).unwrap();
            assert!(actual.riv==compile(&text_input("#a{font-family:inherit}")).unwrap().riv,"{declaration}");
        }
    }
    for value in ["default Inter","Inter inherit","'inherit Inter'"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};font-family:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn substituted_font_shorthands_preserve_longhand_cascade_with_real_text() {
    for (actual,expected) in [
        ("#a{font:700 32px/2 Missing;font:var(--missing);font-size:24px}","#a{font:400 24px/1.5 Inter}"),
        ("#a{--f:400 24px/1.5 Inter;font-size:32px;font:var(--f);line-height:2}","#a{font:400 24px/2 Inter}"),
        ("#a{--f:400 24px/1.5 Inter;font:var(--f)!important;font-size:32px}div{font-size:20px!important}","#a{font:400 24px/1.5 Inter}"),
        ("#a{--f:400 24px/1.5 Inter;font:var(--f)!important;font-size:20px!important}","#a{font:400 20px/1.5 Inter}"),
        ("#a{font:700 32px/2 Missing;font:var(--missing,);line-height:2em;font-size:24px}","#a{font:400 24px/48px Inter}"),
    ] {
        assert!(compile(&text_input(actual)).unwrap().riv==compile(&text_input(expected)).unwrap().riv,"{actual}");
    }
}

#[test]
fn malformed_supported_font_shorthand_forms_inherit() {
    for value in ["20px", "400 20px", "20px /", "20px / 2", "-2px Inter", "20px / -1 Inter", "20px / 2 Inter,", "20px 'Inter' Other", "400 500 20px Inter", "normal normal normal normal normal 20px Inter", "20px inherit"] {
        let actual=compile(&text_input(&format!("#a{{--x:{value};font:700 32px/2 Missing;font:var(--x,24px Missing);font-size:24px}}"))).unwrap();
        assert!(actual.riv==compile(&text_input("#a{font:400 24px/1.5 Inter}")).unwrap().riv,"{value}");
    }
    for value in ["italic 20px Inter", "small-caps 20px Inter", "condensed 20px Inter", "caption", "120% Inter", "0 Inter", "400.5 20px Inter", "20px / 120% Inter", "20px Inter, serif"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};font:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn unknown_font_prefixes_inherit_but_known_exclusions_diagnose() {
    for value in ["banana", "banana 20px Inter", "normal banana 20px Inter", "caption 20px Inter", "bold caption"] {
        let actual=compile(&text_input(&format!("#a{{--x:{value};font:var(--x)}}"))).unwrap();
        assert!(actual.riv==compile(&text_input("#a{font:inherit}")).unwrap().riv,"{value}");
    }
    for value in ["caption","menu","medium Inter","italic 20px Inter","oblique 10deg 20px Inter","small-caps 20px Inter","condensed 20px Inter"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};font:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn invalid_text_transform_substitutions_inherit_casing() {
    for value in ["banana","20px","0","uppercase lowercase","capitalize uppercase","none uppercase","uppercase uppercase","math-auto uppercase","full-width full-width"] {
        let actual=compile(&text_input(&format!("#root{{text-transform:uppercase}}#a{{--x:{value};text-transform:lowercase;text-transform:var(--x)}}"))).unwrap();
        assert!(actual.riv==compile(&text_input("#root{text-transform:uppercase}#a{text-transform:inherit}")).unwrap().riv,"{value}");
    }
    for value in ["full-width","full-size-kana","uppercase full-width","full-size-kana full-width lowercase","math-auto"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};text-transform:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn invalid_white_space_substitutions_inherit_preserved_text() {
    let make=|css:&str| {let mut input=text_input(css);input.html="<div id=root><div id=a>first   words\nsecond line with wrapping words</div></div>".into();input};
    for value in ["banana","20px","0","pre nowrap","normal preserve","collapse preserve","wrap nowrap","preserve preserve","none discard-before","discard-before discard-before"] {
        let actual=compile(&make(&format!("#root{{white-space:pre-wrap}}#a{{--x:{value};white-space:normal;white-space:var(--x)}}"))).unwrap();
        assert!(actual.riv==compile(&make("#root{white-space:pre-wrap}#a{white-space:inherit}")).unwrap().riv,"{value}");
    }
    for value in ["preserve wrap","break-spaces","collapse nowrap","preserve-spaces","discard-before discard-after","preserve nowrap none"] {
        assert!(compile(&make(&format!("#a{{--x:{value};white-space:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn invalid_decoration_style_and_skip_ink_substitutions_reset() {
    for bad in ["banana","20px","0","solid solid","'auto'"] {
        let actual=compile(&text_input(&format!("#root{{text-decoration-skip-ink:none}}#a{{--x:{bad};text-decoration-line:underline;text-decoration-style:var(--x);text-decoration-skip-ink:auto;text-decoration-skip-ink:var(--x)}}"))).unwrap();
        let expected=compile(&text_input("#root{text-decoration-skip-ink:none}#a{text-decoration-line:underline;text-decoration-style:solid;text-decoration-skip-ink:none}")).unwrap();
        assert!(actual.riv==expected.riv,"{bad}");
        assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap(),"{bad}");
    }
    for value in ["double","dotted","dashed","wavy"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};text-decoration-line:underline;text-decoration-style:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn invalid_decoration_lines_reset_local_lines_and_preserve_ancestor_lines() {
    for value in ["banana","20px","0","underline underline","none underline","line-through line-through","spelling-error underline"] {
        let actual=compile(&text_input(&format!("#root{{text-decoration-line:underline;text-decoration-color:red}}#a{{--x:{value};text-decoration-line:line-through;text-decoration-line:var(--x)}}"))).unwrap();
        let expected=compile(&text_input("#root{text-decoration-line:underline;text-decoration-color:red}#a{text-decoration-line:none}")).unwrap();
        assert!(actual.riv==expected.riv,"{value}");
        assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap(),"{value}");
    }
    for value in ["overline","blink","spelling-error","grammar-error","underline overline"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};text-decoration-line:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn invalid_decoration_metrics_reset_thickness_and_inherit_offset() {
    for property in ["text-decoration-thickness", "text-underline-offset"] {
        for value in ["banana", "thin", "2", "2px 3px", "'auto'"] {
            let actual=compile(&text_input(&format!("#root{{text-underline-offset:4px}}#a{{--x:{value};text-decoration-line:underline;text-decoration-thickness:3px;text-underline-offset:1px;{property}:var(--x)}}"))).unwrap();
            let reset=if property=="text-decoration-thickness" {"auto"} else {"4px"};
            let expected=compile(&text_input(&format!("#root{{text-underline-offset:4px}}#a{{text-decoration-line:underline;text-decoration-thickness:3px;text-underline-offset:1px;{property}:{reset}}}"))).unwrap();
            assert!(actual.riv==expected.riv,"{property}: {value}");
            assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap(),"{property}: {value}");
        }
    }
    let actual=compile(&text_input("#root{text-underline-offset:4px}#a{--x:from-font;text-decoration-line:underline;text-underline-offset:var(--x)}")).unwrap();
    let expected=compile(&text_input("#root{text-underline-offset:4px}#a{text-decoration-line:underline;text-underline-offset:4px}")).unwrap();
    assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap());
    // Negative thickness is valid CSS, but remains an explicit profile exclusion.
    for value in ["-2px", "-10%", "1vh", "calc(2px + 1px)"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};text-decoration-line:underline;text-decoration-thickness:var(--x)}}"))).is_err(),"{value}");
    }
    for value in ["-2px", "-10%", "0", "auto"] {
        let actual=compile(&text_input(&format!("#a{{--x:{value};text-decoration-line:underline;text-underline-offset:var(--x)}}"))).unwrap();
        let expected=compile(&text_input(&format!("#a{{text-decoration-line:underline;text-underline-offset:{value}}}"))).unwrap();
        assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap(),"{value}");
    }
}

#[test]
fn invalid_underline_position_substitutions_inherit_auto() {
    for value in ["banana", "2px", "0", "'auto'", "auto left", "auto under", "left right", "under under", "from-font under"] {
        let actual=compile(&text_input(&format!("#root{{text-underline-position:auto}}#a{{--x:{value};text-decoration:underline red 2px;text-underline-position:var(--x)}}"))).unwrap();
        let expected=compile(&text_input("#root{text-underline-position:auto}#a{text-decoration:underline red 2px;text-underline-position:inherit}")).unwrap();
        assert!(actual.riv==expected.riv,"{value}");
        assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap(),"{value}");
    }
    for value in ["under", "left", "right", "from-font", "under left", "right under", "from-font left"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};text-decoration:underline;text-underline-position:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn invalid_decoration_shorthand_resets_components_but_preserves_longhands() {
    for value in ["underline red 2px", "blue solid line-through underline from-font", "none", "underline", "#123456 underline", "rgb(20 30 40) underline 2px"] {
        let actual=compile(&text_input(&format!("#a{{--x:{value};text-decoration:var(--x)}}"))).unwrap();
        let expected=compile(&text_input(&format!("#a{{text-decoration:{value}}}"))).unwrap();
        assert!(actual.riv==expected.riv,"{value}");
        assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap(),"{value}");
    }
    for value in ["banana", "underline underline", "none underline", "underline red blue", "solid dashed", "underline 2px 3px", "underline 2", "grammar-error underline", "'underline'"] {
        for later in ["", "text-decoration-line:underline;text-decoration-color:blue"] {
            let actual=compile(&text_input(&format!("#root{{text-underline-offset:4px}}#a{{--x:{value};text-decoration:line-through red 3px;text-decoration:var(--x);{later}}}"))).unwrap();
            let expected=compile(&text_input(&format!("#root{{text-underline-offset:4px}}#a{{text-decoration:initial;{later}}}"))).unwrap();
            assert!(actual.riv==expected.riv,"{value} {later}");
            assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap(),"{value} {later}");
        }
    }
    for value in ["underline overline", "underline dashed", "underline -2px", "underline 1vh", "spelling-error red"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};text-decoration:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn invalid_decoration_color_functions_reset_entire_shorthand() {
    for value in ["underline rgb(1 2)", "rgb(1, 2, 3, 4, 5) underline", "underline hsl(20 30)", "underline rgb(1 2 3) red", "hsl(20 30% 40%) underline rgb(1 2 3)", "underline underline rgba(1,2,3,.5)", "solid dashed hsl(20 30% 40%)", "underline rgb(1 2 3) 2px 3px"] {
        let actual=compile(&text_input(&format!("#a{{--x:{value};text-decoration:line-through red 3px;text-decoration:var(--x);text-decoration-line:underline}}"))).unwrap();
        let expected=compile(&text_input("#a{text-decoration:initial;text-decoration-line:underline}")).unwrap();
        assert!(actual.riv==expected.riv,"{value}");
        assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap(),"{value}");
    }
    for value in ["underline hsl(20 30 40)", "underline rgb(1 2 3) 2px", "2px hsla(20,30%,40%,.5) underline", "rgb(1 2 3) line-through underline"] {
        let actual=compile(&text_input(&format!("#a{{--x:{value};text-decoration:var(--x)}}"))).unwrap();
        let expected=compile(&text_input(&format!("#a{{text-decoration:{value}}}"))).unwrap();
        assert!(actual.riv==expected.riv,"{value}");
        assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap(),"{value}");
    }
    for value in ["underline color(display-p3 1 0 0)", "underline rgb(none 0 0)", "underline rgb(from red r g b)", "underline rgb(calc(1 + 2) 0 0)", "underline rgb(1 2 3) calc(2px + 1px)"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};text-decoration:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn decoration_missing_empty_and_wide_fallbacks_preserve_ancestor_origins() {
    let base="#root{color:blue;text-decoration:underline blue 3px;text-underline-offset:4px;text-decoration-skip-ink:none}#a{color:green;text-decoration:line-through red 2px;text-underline-offset:1px;text-decoration-skip-ink:auto}";
    for property in ["text-decoration", "text-decoration-line", "text-decoration-style", "text-decoration-color", "text-decoration-thickness", "text-underline-offset", "text-decoration-skip-ink", "text-underline-position"] {
        for (value, expected) in [("var(--absent)","unset"),("var(--empty,inherit)","unset"),("var(--absent,)","unset"),("var(--absent,initial)","initial"),("var(--absent,inherit)","inherit"),("var(--absent,unset)","unset")] {
            let actual=compile(&text_input(&format!("{base}#a{{--empty:;{property}:{value}}}"))).unwrap();
            let expected=compile(&text_input(&format!("{base}#a{{{property}:{expected}}}"))).unwrap();
            assert!(actual.riv==expected.riv,"{property}: {value}");
            assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap(),"{property}: {value}");
        }
    }
}

#[test]
fn custom_property_wide_values_differ_from_decoration_fallback_wide_values() {
    for (wide, expected) in [("inherit","underline blue 3px"),("unset","underline blue 3px"),("initial","line-through red 2px")] {
        let actual=compile(&text_input(&format!("#root{{--x:underline blue 3px}}#a{{--x:{wide};text-decoration:var(--x,line-through red 2px)}}"))).unwrap();
        let expected=compile(&text_input(&format!("#a{{text-decoration:{expected}}}"))).unwrap();
        assert!(actual.riv==expected.riv,"{wide}");
        assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap(),"{wide}");
    }
}

#[test]
fn invalid_simple_background_substitutions_reset_to_transparent() {
    for value in ["banana", "'red'", "red blue", "none none", "red none blue", "#ggg", "#123 #456", "red, blue", ",red", "red,", "none,,none"] {
        equal(&format!("#a{{--x:{value};background:green;background:var(--x,blue)}}"),"#a{background:transparent}");
    }
    for value in ["red none", "none red", "none, red", "left red", "red center / cover", "repeat red", "border-box red", "url(example.png)"] {
        assert!(compile(&input(&format!("#a{{--x:{value};background:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn invalid_background_color_function_combinations_reset() {
    for value in ["rgb(1 2 3) red", "none rgb(1 2)", "none hsl(20 30)", "rgb(1 2 3), none", "none, rgb(1 2 3), none", "rgb(1 2 3) hsl(20 30 40)", "none none rgb(1 2 3)"] {
        equal(&format!("#a{{--x:{value};background:green;background:var(--x,blue)}}"),"#a{background:transparent}");
    }
    for value in ["rgb(1 2 3)", "hsl(20 30 40)", "rgba(20,30,40,.5)"] {
        equal(&format!("#a{{--x:{value};background:var(--x)}}"),&format!("#a{{background:{value}}}"));
    }
    for value in ["none rgb(1 2 3)", "none, hsl(20 30 40)", "left rgb(1 2 3)", "radial-gradient(red,blue)", "rgb(from red r g b)", "color(display-p3 1 0 0)"] {
        assert!(compile(&input(&format!("#a{{--x:{value};background:var(--x)}}"))).is_err(),"{value}");
    }
}

#[test]
fn themed_compositions_equal_explicit_design_values() {
    let cases:serde_json::Value=serde_json::from_str(include_str!("../validation/cases.json")).unwrap();
    for (theme,canvas,surface,ink,muted,accent,button) in [
        ("light","#e8edf4","white","#1b2940","#53647c","hsl(220 75% 40%)","white"),
        ("dark","#101827","#1b2940","#eff4ff","#b0bfd5","hsl(190 85% 65%)","#101827"),
    ] {
        let name=format!("custom-composition-{theme}");
        let case=cases.as_array().unwrap().iter().find(|c|c["name"]==name).unwrap();
        let mut actual=text_input("");actual.html=case["html"].as_str().unwrap().into();actual.css=case["css"].as_str().unwrap().into();
        let mut expected=actual.clone();
        expected.css=format!("#root{{width:100%;padding:12px;background:{canvas};font-family:Inter;font-size:14px;line-height:1.4;color:{ink}}}.card{{width:100%;max-width:460px;padding:16px;gap:12px;border-radius:12px;background:{surface}}}.eyebrow,.title,.copy,.note,.action{{display:block}}.eyebrow{{font-size:10px;letter-spacing:1px;color:{muted}}}.title{{font:24px/1.4 Inter}}.copy{{color:{ink}}}.note{{font-size:12px;text-decoration:underline {accent} 1px;text-underline-offset:3px}}.actions{{flex-direction:row;gap:8px}}.action{{flex:1 1 0px;padding:8px 4px;border-radius:6px;text-align:center;font-size:12px}}.primary{{background:{accent};color:{button}}}.secondary{{background:transparent;color:{accent}}}");
        let actual=compile(&actual).unwrap();let expected=compile(&expected).unwrap();
        assert!(actual.riv==expected.riv,"{theme}");
        assert_eq!(serde_json::to_value(actual.runtime_requirements).unwrap(),serde_json::to_value(expected.runtime_requirements).unwrap(),"{theme}");
    }
}

#[test]
fn non_length_dimension_substitutions_reset_layout_values() {
    for unit in ["deg", "grad", "rad", "turn", "s", "ms", "Hz", "kHz", "dpi", "dpcm", "dppx", "x", "fr"] {
        for property in ["width", "max-width", "padding", "margin", "gap", "border-radius", "flex-basis"] {
            equal(&format!("#a{{--x:2{unit};{property}:12px;{property}:var(--x,8px)}}"), &format!("#a{{{property}:unset}}"));
        }
        equal(&format!("#a{{--x:1 1 2{unit};flex:var(--x);flex-shrink:0}}"), "#a{flex:unset;flex-shrink:0}");
        let error=compile(&input(&format!("#a{{--x:2{unit};min-width:var(--x)}}"))).unwrap_err();
        assert_eq!(error[0].code,"unsupported-initial-value");
    }
    for value in ["2vw", "2cqw", "2ch", "2lh", "2cm"] {
        assert!(compile(&input(&format!("#a{{--x:{value};width:var(--x)}}"))).is_err(), "valid excluded length {value}");
    }
}

#[test]
fn non_length_dimension_substitutions_reset_text_metrics() {
    for unit in ["deg", "ms", "Hz", "dpi", "fr"] {
        for property in ["font-size", "line-height", "letter-spacing", "word-spacing", "text-decoration-thickness", "text-underline-offset"] {
            let base="#root{text-underline-offset:5px}#a{text-decoration-line:underline}";
            let actual=compile(&text_input(&format!("{base}#a{{--x:2{unit};{property}:12px;{property}:var(--x,8px)}}"))).unwrap();
            let expected=compile(&text_input(&format!("{base}#a{{{property}:unset}}"))).unwrap();
            assert!(actual.riv==expected.riv,"{property}:2{unit}");
            assert_eq!(actual.runtime_requirements,expected.runtime_requirements,"{property}:2{unit}");
        }
        for value in [format!("2{unit} Inter"),format!("20px/2{unit} Inter")] {
            let actual=compile(&text_input(&format!("#a{{--x:{value};font:var(--x)}}"))).unwrap();
            let expected=compile(&text_input("#a{font:unset}")).unwrap();
            assert!(actual.riv==expected.riv,"font:{value}");
        }
        let actual=compile(&text_input(&format!("#a{{--x:underline 2{unit} red;text-decoration:var(--x)}}"))).unwrap();
        let expected=compile(&text_input("#a{text-decoration:unset}")).unwrap();
        assert!(actual.riv==expected.riv,"decoration:2{unit}");
        assert_eq!(actual.runtime_requirements,expected.runtime_requirements);
    }
}

#[test]
fn substituted_background_keyword_groups_follow_css_grammar() {
    for value in ["repeat repeat repeat", "repeat-x no-repeat", "repeat-y repeat-x", "repeat red repeat", "fixed scroll", "local local", "border-box padding-box content-box", "none repeat repeat repeat", "repeat, fixed scroll", "red repeat, none"] {
        equal(&format!("#a{{--x:{value};background:green;background:var(--x,blue)}}"), "#a{background:unset}");
    }
    for value in ["repeat", "repeat no-repeat", "repeat-x", "fixed", "border-box padding-box", "border-box red padding-box", "repeat, fixed", "none repeat no-repeat fixed border-box padding-box red"] {
        assert!(compile(&input(&format!("#a{{--x:{value};background:var(--x)}}"))).is_err(), "valid excluded background {value}");
    }
}

#[test]
fn substituted_font_prefix_groups_reset_invalid_combinations() {
    for value in ["italic italic 20px Inter", "italic oblique 20px Inter", "small-caps small-caps 20px Inter", "condensed expanded 20px Inter", "italic bold 700 20px Inter", "normal normal normal italic bold 20px Inter", "italic 20px", "italic 20px/ Inter", "italic medium/2", "small-caps -2px Inter", "oblique 91deg 20px Inter", "oblique 20deg italic 20px Inter", "oblique 20deg 30deg 20px Inter"] {
        let actual=compile(&text_input(&format!("#a{{--x:{value};font:32px/2 Missing;font:var(--x,24px Missing)}}"))).unwrap();
        let expected=compile(&text_input("#a{font:unset}")).unwrap();
        assert!(actual.riv==expected.riv,"{value}");
        assert_eq!(actual.runtime_requirements,expected.runtime_requirements,"{value}");
    }
    for value in ["italic 20px Inter", "oblique 20px Inter", "oblique 20deg 20px Inter", "oblique -0.25turn 20px Inter", "oblique 50grad 20px Inter", "small-caps 20px Inter", "condensed 20px Inter", "normal normal normal italic 20px Inter", "italic small-caps bold condensed 20px/2 Inter", "italic medium Inter", "normal italic normal normal 20px Inter"] {
        assert!(compile(&text_input(&format!("#a{{--x:{value};font:var(--x)}}"))).is_err(),"valid excluded prefix {value}");
    }
}

#[test]
fn substituted_bare_blocks_reset_layout_and_text_values() {
    for value in ["(20px)", "[20px]", "{20px}", "calc(20px) [20px]", "[20px] calc(20px)"] {
        for property in ["width", "padding", "margin", "gap", "border-radius", "background", "box-sizing"] {
            equal(&format!("#a{{--x:{value};{property}:var(--x,12px)}}"), &format!("#a{{{property}:unset}}"));
        }
        for property in ["color", "font-size", "line-height", "font", "text-decoration"] {
            let actual=compile(&text_input(&format!("#a{{--x:{value};{property}:var(--x)}}"))).unwrap();
            let expected=compile(&text_input(&format!("#a{{{property}:unset}}"))).unwrap();
            assert!(actual.riv==expected.riv,"{property}:{value}");
            assert_eq!(actual.runtime_requirements,expected.runtime_requirements);
        }
        for property in ["display"] {
            let error=compile(&input(&format!("#a{{--x:{value};{property}:var(--x)}}"))).unwrap_err();
            assert_eq!(error[0].code,"unsupported-initial-value","{property}:{value}");
        }
    }
    // Parentheses nested inside a function, or bracket characters inside a
    // string, must not be mistaken for top-level grammar blocks.
    for (property,value) in [("width","calc((20px + 10px))"),("color","rgb(from red r g b)"),("font-family","'[Font]'"),("text-overflow","'(...)'")] {
        let actual=compile(&input(&format!("#a{{--x:{value};{property}:var(--x)}}")));
        let expected=compile(&input(&format!("#a{{{property}:{value}}}")));
        match (actual,expected) {
            (Ok(actual),Ok(expected)) => assert!(actual.riv==expected.riv,"{property}:{value}"),
            // Direct and substituted syntax fail at different parser phases.
            (Err(actual),Err(_)) => assert_ne!(actual[0].code,"unsupported-initial-value","{property}:{value}"),
            _ => panic!("substitution changed {property}:{value}"),
        }
    }
}

#[test]
fn substituted_background_position_and_size_follow_css_grammar() {
    let cases:serde_json::Value=serde_json::from_str(include_str!("../validation/background-position-cases.json")).unwrap();
    for value in cases["invalid"].as_array().unwrap() {
        let value=value.as_str().unwrap();
        equal(&format!("#a{{--x:{value};background:green;background:var(--x,blue)}}"),"#a{background:unset}");
    }
    for value in cases["valid"].as_array().unwrap() {
        let value=value.as_str().unwrap();
        assert!(compile(&input(&format!("#a{{--x:{value};background:var(--x)}}"))).is_err(),"valid excluded background {value}");
    }
}

#[test]
fn substituted_function_types_must_match_the_property() {
    let cases:serde_json::Value=serde_json::from_str(include_str!("../validation/function-type-cases.json")).unwrap();
    for group in cases["groups"].as_array().unwrap() {
        for property in group["properties"].as_array().unwrap() {
            let property=property.as_str().unwrap();
            for value in group["values"].as_array().unwrap() {
                let value=value.as_str().unwrap();
                let actual=compile(&text_input(&format!("#a{{--x:{value};{property}:var(--x)}}")));
                if matches!(property,"display") {
                    let Err(error)=actual else {panic!("expected unrepresentable initial value")};
                    assert_eq!(error[0].code,"unsupported-initial-value","{property}:{value}");
                } else {
                    let actual=actual.unwrap();
                    let expected=compile(&text_input(&format!("#a{{{property}:unset}}"))).unwrap();
                    assert!(actual.riv==expected.riv,"{property}:{value}");
                    assert_eq!(actual.runtime_requirements,expected.runtime_requirements);
                }
            }
        }
    }
    for (property,value) in [("color","rgb(1 2 3)"),("background","hsl(20 30% 40%)"),("width","calc(20px)"),("background","url(test.png)"),("color","color-mix(in srgb,red,blue)"),("font","calc(20px) Inter"),("text-decoration","underline calc(2px)"),("width","future-function(20px)")] {
        let actual=compile(&text_input(&format!("#a{{--x:{value};{property}:var(--x)}}")));
        let expected=compile(&text_input(&format!("#a{{{property}:{value}}}")));
        match (actual,expected) {
            (Ok(a),Ok(b))=>assert!(a.riv==b.riv,"{property}:{value}"),
            (Err(_),Err(_))=>{},
            _=>panic!("function profile changed {property}:{value}"),
        }
    }
}
