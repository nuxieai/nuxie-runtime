//! Public first-baseline bytes and admission. Pixel qualification is separate.
use nuxie_html_to_riv::{compile, CompileInput, CompileOutput};
use nuxie_schema::{definition_by_type_key,FieldKind};
use serde_json::{Value,json};
use std::collections::BTreeMap;
fn input(css:&str)->CompileInput {CompileInput{html:"<div id=p><div id=a></div><div id=b><div id=leaf></div></div><div id=c></div></div>".into(),css:css.into(),width:240.,height:160.}}
fn scene(css:&str)->CompileOutput {compile(&input(css)).unwrap()}
const FIXED:&str="#p{flex-direction:row;height:100px}#a,#b,#c{align-self:baseline}#a{height:20px}#b{height:60px}#leaf{height:10px}#c{height:40px}";
fn uint(bytes:&[u8],i:&mut usize)->u32 {let mut result=0;for shift in (0..35).step_by(7){let b=bytes[*i];*i+=1;result|=((b&127)as u32)<<shift;if b<128{return result}}panic!("bad varuint")}
fn decoded(output:&CompileOutput)->Vec<(String,BTreeMap<String,Value>)> {
    let bytes=&output.riv;assert_eq!(&bytes[..7],b"RIVE\x07\x03\x00");let mut i=7;let mut fields=0;
    while uint(bytes,&mut i)!=0{fields+=1}i+=((fields+3)/4)*4;
    let mut result=Vec::new();
    while i<bytes.len(){let definition=definition_by_type_key(uint(bytes,&mut i)as u16).unwrap();let mut values=BTreeMap::new();loop{let key=uint(bytes,&mut i)as u16;if key==0{break}let property=definition.property_by_key_in_hierarchy(key).unwrap();let value=match property.runtime_type{
        FieldKind::Uint=>json!(uint(bytes,&mut i)),FieldKind::Double=>{let value=f32::from_le_bytes(bytes[i..i+4].try_into().unwrap());i+=4;assert!(value.is_finite());json!(value)},FieldKind::Color=>{let value=u32::from_le_bytes(bytes[i..i+4].try_into().unwrap());i+=4;json!(value)},FieldKind::String|FieldKind::Bytes=>{let len=uint(bytes,&mut i)as usize;let value=&bytes[i..i+len];i+=len;json!(String::from_utf8_lossy(value))},FieldKind::Bool=>{let value=bytes[i]!=0;i+=1;json!(value)},other=>panic!("unexpected {other:?}")};values.insert(property.name.to_owned(),value);}result.push((definition.name.to_owned(),values));}
    result
}
#[test]
fn nested_authored_metrics_emit_extent_and_landmark_not_fixture_constants() {
    for (a,b,leaf,c,extent,ascent) in [(20,60,10,40,90.,40.),(30,80,20,50,110.,50.)] {
        let css=format!("#p{{flex-direction:row;height:auto}}#a,#b,#c{{align-self:first baseline}}#a{{height:{a}px}}#b{{height:{b}px}}#leaf{{height:{leaf}px}}#c{{height:{c}px}}");
        let output=scene(&css);let records=decoded(&output);
        let helper=records.iter().filter(|(kind,_)|kind=="LayoutComponent").last().unwrap();
        assert_eq!(helper.1["width"],json!(0.));assert_eq!(helper.1["height"],json!(extent));
        let target=records.iter().find(|(kind,_)|kind=="TransformConstraint").unwrap();
        assert!((target.1["originY"].as_f64().unwrap()-ascent/extent).abs()<1e-6);
        assert_eq!(records.iter().filter(|(kind,_)|kind=="TranslationConstraint").count(),3);
        assert_eq!(output.source_map.iter().map(|n|n.id.as_str()).collect::<Vec<_>>(),["p","a","b","leaf","c"]);
        assert_eq!(output,scene(&css));
    }
}
#[test]
fn responsive_maximum_stays_an_original_parent_percentage() {
    let mut request=input("#p{flex-direction:row;height:100px}#a,#c{align-self:baseline}#a{height:50%;min-height:30px}#c{height:30%;min-height:60px}");
    request.html="<div id=p><div id=a></div><div id=c></div></div>".into();
    let output=compile(&request).unwrap();let records=decoded(&output);
    let helper=records.iter().rposition(|(kind,_)|kind=="LayoutComponent").unwrap();
    assert_eq!(records[helper].1["height"],json!(50.));
    assert_eq!(records[helper+1].1["heightUnitsValue"],json!(2));
    assert_eq!(records[helper+1].1["minHeight"],json!(60.));
    assert_eq!(records[helper].1["parentId"],json!(output.source_map[0].object_id));
}
#[test]
fn aliases_variables_and_css_wide_values_keep_context_rules() {
    assert_eq!(scene(FIXED),scene(&FIXED.replace("align-self:baseline","align-self:first baseline")));
    assert_eq!(scene(FIXED),scene(&FIXED.replace("align-self:baseline","--v:first baseline;align-self:var(--v)")));
    for value in ["initial","unset"] {assert_eq!(scene(&format!("{FIXED}#a{{align-self:{value}}}")),scene(&format!("{FIXED}#a{{align-self:auto}}")));}
    assert!(compile(&input("#p{align-self:baseline;flex-direction:row;height:100px}#a{height:20px;align-self:inherit}")).is_err());
}
#[test]
fn zero_and_fixed_bounds_produce_finite_anchors() {
    let mut request=input("#p{flex-direction:row}#a,#c{height:0;align-self:baseline}");request.html="<div id=p><div id=a></div><div id=c></div></div>".into();
    let records=decoded(&compile(&request).unwrap());
    for (kind,values) in &records {if kind=="ComponentOrigin" || kind=="TransformConstraint" {assert_eq!(values["originY"],json!(0.));}}
    request.css="#p{flex-direction:row}#a{height:100px;min-height:20px;max-height:40px;align-self:baseline}#c{height:20px;min-height:60px;max-height:40px;align-self:baseline}".into();
    let output=compile(&request).unwrap();let records=decoded(&output);assert_eq!(records.iter().filter(|(kind,_)|kind=="LayoutComponent").last().unwrap().1["height"],json!(60.));
}
#[test]
fn unresolved_expressions_and_excessive_helper_extent_reject() {
    for css in [
        "#a{height:20px;align-self:baseline}",
        "#p{flex-direction:row}#a{height:auto;min-height:auto;align-self:baseline}",
        "#p{flex-direction:row;height:100px}#a{height:50%;max-height:60px;align-self:baseline}",
        "#p{flex-direction:row;height:100px}#a{height:50%;align-self:baseline}#b{height:60px;align-self:baseline}#leaf{height:10px}",
        "#p{flex-direction:row}#b{height:0;align-self:baseline}#leaf{height:10px}",
        "#p{flex-direction:row}#b{height:60px;flex-direction:column-reverse;align-self:baseline}#leaf{height:10px}",
        "#p{flex-direction:row}#a{height:1000000px;align-self:baseline}#b{height:1000000px;align-self:baseline}#leaf{height:1px}",
    ] {let errors=compile(&input(css)).expect_err(css);assert_eq!(errors[0].code,"unsupported-target-semantics");}
    for value in ["safe last baseline","unsafe last baseline","last","last baseline extra","safe baseline","unsafe baseline","safe first baseline","first","first baseline extra"] {assert!(compile(&input(&format!("#never{{align-self:{value}}}"))).is_err());}
}

#[test]
fn intrinsic_empty_and_column_metrics_use_fixed_used_heights() {
    for (bounds, expected_extent, expected_ascent) in [
        ("", 80., 40.),
        ("min-height:90px", 110., 40.),
        ("max-height:30px", 50., 40.),
        ("min-height:90px;max-height:30px", 110., 40.),
    ] {
        let request = CompileInput {
            html: "<div id=p><div id=a><div id=x></div><div id=y></div></div><div id=b></div></div>".into(),
            css: format!("#p{{flex-direction:row}}#a{{align-self:baseline;{bounds}}}#x{{height:20px;order:-1}}#y{{height:40px}}#b{{height:40px;align-self:first baseline}}"),
            width:240., height:160.,
        };
        let output = compile(&request).unwrap();
        let records = decoded(&output);
        let helper = records.iter().filter(|(kind,_)|kind=="LayoutComponent").last().unwrap();
        assert_eq!(helper.1["height"],json!(expected_extent));
        let target = records.iter().find(|(kind,_)|kind=="TransformConstraint").unwrap();
        assert!((target.1["originY"].as_f64().unwrap()-expected_ascent/expected_extent).abs()<1e-6);
    }
    for (bounds, height) in [("",0.), ("min-height:30px",30.), ("min-height:30px;max-height:10px",30.)] {
        let request = CompileInput { html:"<div id=p><div id=a></div></div>".into(), css:format!("#p{{flex-direction:row}}#a{{align-self:baseline;{bounds}}}"),width:240.,height:160. };
        let records=decoded(&compile(&request).unwrap());
        assert_eq!(records.iter().filter(|(kind,_)|kind=="LayoutComponent").last().unwrap().1["height"],json!(height));
    }
}
#[test]
fn unresolved_intrinsic_metrics_and_oversized_sums_reject() {
    for extra in [
        "#a{min-height:auto}",
        "#a{min-height:10%}",
        "#a{max-height:10%}",
        "#a{max-height:10px}",
        "#x{height:60%}",
        "#x{height:600000px}#y{height:600000px}#a{max-height:100px}",
        "#a{flex-direction:column-reverse}",
    ] {
        let request=CompileInput {html:"<div id=p><div id=a><div id=x></div><div id=y></div></div></div>".into(),css:format!("#p{{flex-direction:row}}#a{{align-self:baseline}}#x{{height:20px}}#y{{height:40px}}{extra}"),width:240.,height:160.};
        assert!(compile(&request).is_err(),"{extra}");
    }
}

#[test]
fn intrinsic_column_baseline_uses_first_order_modified_descendant() {
    let request=CompileInput {html:"<div id=p><div id=a><div id=x></div><div id=y></div></div><div id=b></div></div>".into(),css:"#p{flex-direction:row}#a,#b{align-self:baseline}#x{height:20px}#y{height:40px;order:-1}#b{height:40px}".into(),width:240.,height:160.};
    let output=compile(&request).unwrap();let records=decoded(&output);
    assert_eq!(records.iter().filter(|(kind,_)|kind=="LayoutComponent").last().unwrap().1["height"],json!(60.));
    let anchor=records.iter().find(|(kind,values)|kind=="ComponentOrigin" && values["parentId"]==json!(output.source_map[1].object_id)).unwrap();
    assert!((anchor.1["originY"].as_f64().unwrap()-40./60.).abs()<1e-6);
    assert_eq!(output.source_map.iter().map(|node|node.id.as_str()).collect::<Vec<_>>(),["p","a","x","y","b"]);
}

#[test]
fn last_baseline_targets_parent_bottom_with_derived_descent() {
    let css=FIXED.replace("align-self:baseline","align-self:last baseline");
    let output=scene(&css);let records=decoded(&output);
    let helper=records.iter().filter(|(kind,_)|kind=="LayoutComponent").last().unwrap();
    assert_eq!(helper.1["height"],json!(90.));
    let target=records.iter().find(|(kind,_)|kind=="TransformConstraint").unwrap();
    assert_eq!(target.1["targetId"],json!(output.source_map[0].object_id));
    assert_eq!(target.1["originY"],json!(1.));
    let offset=records.iter().find(|(kind,values)|kind=="TranslationConstraint" && values.get("offset")==Some(&json!(true))).unwrap();
    assert_eq!(offset.1["doesCopy"],json!(false));
    let node=&records[offset.1["parentId"].as_u64().unwrap() as usize+1];
    assert_eq!(node.0,"Node");assert_eq!(node.1["y"],json!(-50.));
    assert_eq!(records.iter().filter(|(kind,_)|kind=="TranslationConstraint").count(),4);
    assert_eq!(output,scene(&css));
    assert_eq!(output,scene(&css.replace("align-self:last baseline","--v:last baseline;align-self:var(--v)")));
    for value in ["initial","unset"] {assert_eq!(scene(&format!("{css}#a{{align-self:{value}}}")),scene(&format!("{css}#a{{align-self:auto}}")));}
    let nested=CompileInput {html:"<div id=p><div id=a><div id=b></div></div></div>".into(),css:"#p{flex-direction:row;height:100px}#a{align-self:last baseline;flex-direction:row;height:60px}#b{align-self:inherit;height:10px}".into(),width:240.,height:160.};
    // Grammar preserves inherited last-baseline identity; the outer row
    // participant itself still requires a separately proven nested metric.
    assert!(compile(&nested).is_err());
    assert!(compile(&input("#never{align-self:last baseline}")).is_ok());
}

#[test]
fn last_metrics_follow_deep_order_modified_descendants() {
    for (extra,ascent) in [("",35.),("#x{order:2}",50.)] {
        let request=CompileInput {html:"<div id=p><div id=a><div id=x></div><div id=y><div id=z></div></div></div></div>".into(),css:format!("#p{{flex-direction:row}}#a{{height:100px;align-self:last baseline}}#x{{height:30px}}#y{{height:20px}}#z{{height:5px}}{extra}"),width:240.,height:160.};
        let output=compile(&request).unwrap();let records=decoded(&output);
        let anchor=records.iter().find(|(kind,values)|kind=="ComponentOrigin" && values["parentId"]==json!(output.source_map[1].object_id)).unwrap();
        assert!((anchor.1["originY"].as_f64().unwrap()-ascent/100.).abs()<1e-6);
        assert_eq!(records.iter().find(|(kind,values)|kind=="Node" && values.contains_key("y")).unwrap().1["y"],json!(-(100.-ascent)));
    }
}

#[test]
fn first_and_last_groups_measure_independent_extents() {
    let request=CompileInput {html:"<div id=p><div id=a><div id=x></div></div><div id=b></div><div id=c><div id=y></div></div><div id=d></div></div>".into(),css:"#p{flex-direction:row}#a,#b{align-self:first baseline}#c,#d{align-self:last baseline}#a{height:60px}#x{height:10px}#b{height:20px}#c{height:30px}#y{height:10px}#d{height:40px}".into(),width:240.,height:160.};
    let records=decoded(&compile(&request).unwrap());
    let layouts:Vec<_>=records.iter().filter(|(kind,_)|kind=="LayoutComponent").collect();
    let helpers=&layouts[layouts.len()-2..];
    assert_eq!(helpers.len(),2);assert_eq!(helpers[0].1["height"],json!(70.));assert_eq!(helpers[1].1["height"],json!(60.));
    assert_eq!(records.iter().filter(|(kind,_)|kind=="TransformConstraint").count(),2);
    assert_eq!(records.iter().filter(|(kind,_)|kind=="ComponentOrigin").count(),4);
}

#[test]
fn last_intrinsic_bounds_zero_and_unresolved_profiles() {
    for (bounds,descent) in [("",0.),("min-height:90px",30.),("max-height:90px",0.),("min-height:90px;max-height:20px",30.)] {
        let request=CompileInput{html:"<div id=p><div id=a><div id=x></div><div id=y></div></div></div>".into(),css:format!("#p{{flex-direction:row}}#a{{align-self:last baseline;{bounds}}}#x{{height:20px}}#y{{height:40px}}"),width:240.,height:160.};
        let records=decoded(&compile(&request).unwrap());
        assert_eq!(records.iter().find(|(kind,values)|kind=="Node" && values.contains_key("y")).unwrap().1["y"],json!(-descent));
    }
    let zero=CompileInput{html:"<div id=p><div id=a></div></div>".into(),css:"#p{flex-direction:row}#a{align-self:last baseline;height:0}".into(),width:240.,height:160.};
    let records=decoded(&compile(&zero).unwrap());
    assert_eq!(records.iter().find(|(kind,_)|kind=="ComponentOrigin").unwrap().1["originY"],json!(0.));
    for css in [
        "#p{flex-direction:row;height:100px}#a{height:50%;align-self:last baseline}",
        "#p{flex-direction:row}#b{height:5px;align-self:last baseline}#leaf{height:10px}",
        "#p{flex-direction:row}#b{height:auto;max-height:5px;align-self:last baseline}#leaf{height:10px}",
        "#p{flex-direction:row}#b{height:60px;flex-direction:column-reverse;align-self:last baseline}#leaf{height:10px}",
        "#p{flex-direction:row}#a{height:1000000px;align-self:last baseline}#b{height:1000000px;align-self:last baseline}#leaf{height:1px}",
    ] {let errors=compile(&input(css)).expect_err(css);assert_eq!(errors[0].code,"unsupported-target-semantics");}
}

#[test]
fn last_group_record_growth_is_bounded_per_authored_participant() {
    let mut request=CompileInput{html:"<main>".into(),css:"section{flex-direction:row;height:1px}div{height:1px;align-self:last baseline}".into(),width:240.,height:160.};
    for _ in 0..128 {request.html.push_str("<section><div></div></section>");}request.html.push_str("</main>");
    let aligned=compile(&request).unwrap();
    request.css=request.css.replace("last baseline","flex-start");let unaligned=compile(&request).unwrap();
    assert_eq!(decoded(&aligned).len()-decoded(&unaligned).len(),128*8);
    assert_eq!(aligned.source_map.len(),257);
}
