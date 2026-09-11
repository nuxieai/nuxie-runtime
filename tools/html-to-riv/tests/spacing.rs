//! Public distribution semantics; native qualification is recorded separately.
use nuxie_html_to_riv::{compile, CompileInput, CompileOutput};
use nuxie_schema::{definition_by_type_key, FieldKind};
use serde_json::{Value,json};
use std::collections::BTreeMap;
fn input(css:&str)->CompileInput { CompileInput { html:"<div id=p><div id=a></div><div id=b></div><div id=c></div></div>".into(),css:css.into(),width:240.,height:160. } }
fn scene(css:&str)->CompileOutput { compile(&input(css)).unwrap() }
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
fn distribution_weights_and_direction_are_ordinary_file_properties() {
    for direction in ["row","row-reverse","column","column-reverse"] {
        for mode in ["space-around","space-evenly"] {
            let css=format!("#p{{flex-direction:{direction};justify-content:{mode}}}#a{{order:2}}");
            let output=scene(&css);let records=decoded(&output);
            let parent=output.source_map[0].object_id;
            let row=direction.starts_with("row");
            assert_eq!(records[parent as usize+2].1["layoutAlignmentType"],json!(if row {2} else {6}));
            let weights:Vec<_>=records.iter().filter_map(|(_,v)|v.get(if row {"fractionalWidth"} else {"fractionalHeight"})).collect();
            let edge=if mode=="space-around" {0.5} else {1.};
            assert_eq!(weights,vec![&json!(edge),&json!(1.),&json!(1.),&json!(edge)]);
            assert_eq!(output.source_map.iter().map(|n|n.id.as_str()).collect::<Vec<_>>(),["p","a","b","c"]);
            let mut ordered:Vec<_>=output.source_map.iter().skip(1).collect();ordered.sort_by_key(|n|n.object_id);
            assert_eq!(ordered.iter().map(|n|n.id.as_str()).collect::<Vec<_>>(),if direction.ends_with("reverse") {vec!["b","c","a"]} else {vec!["a","c","b"]});
            assert_eq!(output,scene(&css));
        }
    }
}
#[test]
fn css_wide_values_variables_and_selectors_preserve_authored_semantics() {
    for value in ["normal","flex-start","initial","unset"] {assert_eq!(scene(""),scene(&format!("div{{justify-content:{value}}}")));}
    assert_eq!(scene("#p{justify-content:space-around}#a{justify-content:inherit}"),scene("#p,#a{justify-content:space-around}"));
    assert_eq!(scene("#p{--mode:SPACE-EVENLY;justify-content:var(--mode)}#p>div:nth-child(2){background:red}#a+div{height:20px}"),scene("#p{justify-content:space-evenly}#b{background:red;height:20px}"));
    assert_eq!(scene("#p{justify-content:space-around!important;justify-content:space-evenly}"),scene("#p{justify-content:space-around}"));
}
#[test]
fn baseline_helpers_do_not_receive_distribution_shares() {
    for mode in ["space-around","space-evenly"] {
        let output=scene(&format!("#p{{flex-direction:row;justify-content:{mode}}}#a,#b{{height:20px;align-self:baseline}}#c{{height:30px;align-self:last baseline}}"));
        let records=decoded(&output);
        assert_eq!(records.iter().filter(|(_,v)|v.contains_key("fractionalWidth")).count(),4);
        assert_eq!(records.iter().filter(|(k,_)|k=="TranslationConstraint").count(),4);
    }
}
#[test]
fn nonempty_spaced_column_baseline_metrics_remain_explicitly_unresolved() {
    for baseline in ["baseline","last baseline"] {
        let mut request=input(&format!("#p{{flex-direction:row}}#a{{height:60px;justify-content:space-evenly;align-self:{baseline}}}#leaf{{height:10px}}"));
        request.html="<div id=p><div id=a><div id=leaf></div></div></div>".into();
        let errors=compile(&request).unwrap_err();assert_eq!(errors[0].code,"unsupported-target-semantics");
        // Empty baseline boxes have no descendant-position expression.
        request.html="<div id=p><div id=a></div></div>".into();assert!(compile(&request).is_ok());
    }
}
#[test]
fn unsupported_values_reject_even_in_losing_or_unmatched_declarations() {
    for value in ["center","space-between","safe space-around","unsafe space-evenly","auto","space-around extra","revert"] {
        for css in [format!("#never{{justify-content:{value}}}"),format!("#p{{justify-content:{value};justify-content:normal}}"),format!("#p{{--j:{value};justify-content:var(--j)}}")] {assert!(compile(&input(&css)).is_err(),"{css}");}
    }
}
#[test]
fn empty_containers_add_no_spacers_and_authored_limit_bounds_helpers() {
    let mut request=input("div{justify-content:space-evenly}");request.html="<div></div>".into();
    assert_eq!(decoded(&compile(&request).unwrap()).iter().filter(|(_,v)|v.contains_key("fractionalHeight")).count(),0);
    request.html=format!("<div>{}</div>","<div></div>".repeat(8191));
    let output=compile(&request).unwrap();assert_eq!(output.source_map.len(),8192);
    assert_eq!(decoded(&output).iter().filter(|(_,v)|v.contains_key("fractionalHeight")).count(),8192);
    request.html.push_str("<div></div>");assert_eq!(compile(&request).unwrap_err()[0].code,"object-limit");
}
