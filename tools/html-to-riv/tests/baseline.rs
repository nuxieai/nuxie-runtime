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
        "#p{flex-direction:row}#a{height:auto;align-self:baseline}",
        "#p{flex-direction:row;height:100px}#a{height:50%;max-height:60px;align-self:baseline}",
        "#p{flex-direction:row;height:100px}#a{height:50%;align-self:baseline}#b{height:60px;align-self:baseline}#leaf{height:10px}",
        "#p{flex-direction:row}#b{height:0;align-self:baseline}#leaf{height:10px}",
        "#p{flex-direction:row}#b{height:60px;flex-direction:column-reverse;align-self:baseline}#leaf{height:10px}",
        "#p{flex-direction:row}#a{height:1000000px;align-self:baseline}#b{height:1000000px;align-self:baseline}#leaf{height:1px}",
    ] {let errors=compile(&input(css)).expect_err(css);assert_eq!(errors[0].code,"unsupported-target-semantics");}
    for value in ["last baseline","safe baseline","unsafe baseline","safe first baseline","first","first baseline extra"] {assert!(compile(&input(&format!("#never{{align-self:{value}}}"))).is_err());}
}
