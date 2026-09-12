//! Numeric spelling must survive public compilation before ordinary fields are emitted.
//! Exact field assertions are authoring tests, not browser/native qualification.
use nuxie_html_to_riv::{CompileInput,CompileOutput,compile};
use nuxie_schema::{FieldKind,definition_by_type_key};
use serde_json::{Value,json};
use std::collections::BTreeMap;

// Reuse the schema-based decoding pattern from spacing.rs/baseline.rs. Keep
// float values as exact f64 conversions so JSON display cannot hide f32 bits.
fn uint(bytes:&[u8],i:&mut usize)->u32 {let mut result=0;for shift in (0..35).step_by(7){let b=bytes[*i];*i+=1;result|=((b&127)as u32)<<shift;if b<128{return result}}panic!("bad varuint")}
fn decoded(output:&CompileOutput)->Vec<(String,BTreeMap<String,Value>)> {
    let bytes=&output.riv;assert_eq!(&bytes[..7],b"RIVE\x07\x03\x00");let mut i=7;let mut fields=0;
    while uint(bytes,&mut i)!=0{fields+=1}i+=((fields+3)/4)*4;
    let mut result=Vec::new();
    while i<bytes.len(){let definition=definition_by_type_key(uint(bytes,&mut i)as u16).unwrap();let mut values=BTreeMap::new();loop{let key=uint(bytes,&mut i)as u16;if key==0{break}let property=definition.property_by_key_in_hierarchy(key).unwrap();let value=match property.runtime_type{
        FieldKind::Uint=>json!(uint(bytes,&mut i)),FieldKind::Double=>{let value=f32::from_le_bytes(bytes[i..i+4].try_into().unwrap());i+=4;assert!(value.is_finite());json!(f64::from(value))},FieldKind::Color=>{let value=u32::from_le_bytes(bytes[i..i+4].try_into().unwrap());i+=4;json!(value)},FieldKind::String|FieldKind::Bytes=>{let len=uint(bytes,&mut i)as usize;let value=&bytes[i..i+len];i+=len;json!(String::from_utf8_lossy(value))},FieldKind::Bool=>{let value=bytes[i]!=0;i+=1;json!(value)},other=>panic!("unexpected {other:?}")};values.insert(property.name.to_owned(),value);}result.push((definition.name.to_owned(),values));}
    result
}
fn input(case:&Value,width:f32)->CompileInput {CompileInput{html:case["html"].as_str().unwrap().into(),css:case["css"].as_str().unwrap().into(),width,height:160.}}
fn check_group(group:&str) {
    let cases:Vec<Value>=serde_json::from_str(include_str!("../validation/public-numeric-token-cases.json")).unwrap();
    let mut failures=Vec::new();let mut checked=0;
    for case in cases.iter().filter(|c|c["group"]==group) {
        checked+=1;
        for viewport in [240.,768.] {
            let request=input(case,viewport);
            let output=match compile(&request){Ok(value)=>value,Err(errors)=>{failures.push(format!("{}: compile {errors:?}",case["name"]));continue}};
            assert_eq!(output,compile(&request).unwrap(),"{} deterministic",case["name"]);
            let records=decoded(&output);assert_eq!(records[0].0,"Backboard");
            for expected in case["expectedFields"].as_array().unwrap() {
                let id=output.source_map.iter().find(|n|n.id==expected["node"].as_str().unwrap()).unwrap().object_id;
                let index=match expected["record"].as_str().unwrap(){
                    "component"=>{assert_eq!(records[id as usize+1].0,"LayoutComponent");id as usize+1},
                    "style"=>{assert_eq!(records[id as usize+2].0,"LayoutComponentStyle");id as usize+2},
                    "paint"=>{let fill=records.iter().position(|(kind,v)|kind=="Fill"&&v.get("parentId")==Some(&json!(id))).unwrap();records.iter().position(|(kind,v)|kind=="SolidColor"&&v.get("parentId")==Some(&json!(fill-1))).unwrap()},
                    other=>panic!("unexpected record {other}")};
                let property=expected["property"].as_str().unwrap();let actual=&records[index].1[property];
                if let Some(decimal)=expected["decimal"].as_str() {
                    let want=decimal.parse::<f32>().unwrap();let observed=actual.as_f64().unwrap()as f32;
                    if want.to_bits()!=observed.to_bits(){failures.push(format!("{} @ {viewport}: {}.{} expected {} ({:08x}), got {} ({:08x})",case["name"],expected["node"],property,f64::from(want),want.to_bits(),f64::from(observed),observed.to_bits()));}
                } else if actual.as_u64()!=expected["integer"].as_u64(){failures.push(format!("{} @ {viewport}: {}.{} expected {}, got {actual}",case["name"],expected["node"],property,expected["integer"]));}
            }
        }
    }
    assert!(checked>0);assert!(failures.is_empty(),"{}",failures.join("\n"));
}
#[test]fn literal_inline_point_percentage_and_numeric_spellings(){check_group("ordinary")}
#[test]fn variable_alias_inheritance_and_fallback_keep_exact_values(){check_group("variables")}
#[test]fn font_relative_lengths_compute_from_unrounded_coefficients(){check_group("font")}
#[test]fn padding_minmax_and_content_box_lowering_keep_exact_coefficients(){check_group("bounds")}
#[test]fn color_numbers_percentages_angles_and_alpha_keep_byte_boundaries(){check_group("color")}
#[test]
fn token_categories_and_invalid_integer_lexemes_cannot_be_repaired_by_serialization(){
    let cases:Vec<Value>=serde_json::from_str(include_str!("../validation/public-numeric-token-rejections.json")).unwrap();let mut failures=Vec::new();
    for case in cases {match compile(&input(&case,240.)){Ok(_)=>failures.push(format!("{} unexpectedly compiled: {}",case["name"],case["css"])),Err(errors)=>{assert!(!errors.is_empty());assert!(errors.iter().all(|e|!e.code.is_empty()&&!e.source.is_empty()&&!e.message.is_empty()));}}}
    assert!(failures.is_empty(),"{}",failures.join("\n"));
}
#[test]
fn exact_signed_integer_order_still_preserves_large_values(){
    let request=CompileInput{html:"<div id=p><div id=a></div><div id=b></div><div id=c></div></div>".into(),css:"#p{flex-direction:row-reverse;--n:16777217}#a{order:var(--n)}#b{order:16777216}#c{order:-2147483648}".into(),width:240.,height:160.};
    let output=compile(&request).unwrap();let mut nodes:Vec<_>=output.source_map.iter().filter(|n|n.id!="p").collect();nodes.sort_by_key(|n|n.object_id);assert_eq!(nodes.iter().map(|n|n.id.as_str()).collect::<Vec<_>>(),["c","b","a"]);
}
