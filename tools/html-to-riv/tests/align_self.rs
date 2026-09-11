use nuxie_html_to_riv::{compile,AlignSelf,CompileInput,RuntimeCapability,RuntimeRequirements};

#[test]
fn alignment_cascade_resets_and_substitution_publish_the_correct_contract() {
 let compile_css=|css:&str|compile(&CompileInput{html:"<div id=root><div id=a></div></div>".into(),css:css.into(),width:390.,height:320.,..Default::default()});
 for value in ["auto","flex-start","center","flex-end","stretch","baseline"] {
  let direct=compile_css(&format!("#a{{align-self:{value}}}")).unwrap();
  assert_eq!(direct.runtime_requirements,compile_css(&format!("#a{{--v:{value};align-self:var(--v)}}")).unwrap().runtime_requirements);
 }
 let inherited=compile_css("#root{align-self:center}#a{align-self:inherit}").unwrap();
 assert_eq!(inherited.runtime_requirements.layout_align_self.len(),2);
 assert!(inherited.runtime_requirements.layout_align_self.iter().all(|entry|entry.alignment==AlignSelf::Center));
 assert_eq!(compile_css("#root{align-self:center}").unwrap().runtime_requirements.layout_align_self.len(),1,"not implicitly inherited");
 for reset in ["initial","unset","var(--missing)","var(--bad)"] {
  let output=compile_css(&format!("#a{{--bad:7px;align-self:center;align-self:{reset}}}")).unwrap();
  assert!(output.runtime_requirements.layout_align_self.is_empty(),"{reset}");
 }
 for invalid in ["normal","self-start","safe center","last baseline","calc(1)","var(--unsupported)"] {
  assert!(compile_css(&format!("#a{{--unsupported:safe center;align-self:{invalid}}}")).is_err(),"{invalid} must not be silently reset");
 }
 let painted=compile_css("#root{align-self:center}#a{background:red}").unwrap();
 assert_eq!(painted.runtime_requirements.version,6,"later paint requirements must not downgrade version");
 assert!(!painted.runtime_requirements.layout_pixel_bounds.is_empty());
}

#[test]
fn alignment_manifest_versions_capabilities_and_targets_are_checked() {
 let valid=serde_json::json!({"version":6,"capabilities":["layout-css-align-self-v1"],"layout_align_self":[{"object_id":7,"alignment":"center"}]});
 let requirements:RuntimeRequirements=serde_json::from_value(valid.clone()).unwrap();
 assert_eq!(requirements.ensure_supported(&[]).unwrap_err().code,"missing-runtime-capability");
 requirements.ensure_supported(&[RuntimeCapability::LayoutCssAlignSelfV1]).unwrap();
 requirements.ensure_layout_targets(|id|id==7).unwrap();
 assert_eq!(requirements.ensure_layout_targets(|_|false).unwrap_err().code,"invalid-layout-align-self-target");
 for version in 1..=5 {
  let mut value=valid.clone();value["version"]=version.into();
  assert_eq!(serde_json::from_value::<RuntimeRequirements>(value).unwrap().ensure_supported(&[RuntimeCapability::LayoutCssAlignSelfV1]).unwrap_err().code,"invalid-layout-align-self");
 }
 for (field,value) in [("capabilities",serde_json::json!([])),("layout_align_self",serde_json::json!([])),("layout_align_self",serde_json::json!([{"object_id":7,"alignment":"center"},{"object_id":7,"alignment":"stretch"}]))] {
  let mut bad=valid.clone();bad[field]=value;
  assert_eq!(serde_json::from_value::<RuntimeRequirements>(bad).unwrap().ensure_supported(&[RuntimeCapability::LayoutCssAlignSelfV1]).unwrap_err().code,"invalid-layout-align-self");
 }
 let mut invalid=valid.clone();invalid["layout_align_self"][0]["alignment"]="space-between".into();assert!(serde_json::from_value::<RuntimeRequirements>(invalid).is_err());
 let mut unknown=valid;unknown["layout_align_self"][0]["unknown"]=true.into();assert!(serde_json::from_value::<RuntimeRequirements>(unknown).is_err());
}
