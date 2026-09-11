use nuxie_html_to_riv::{compile, Asset, CompileInput};
fn input()->CompileInput {
    CompileInput{html:"<div id='word'>Second</div>".into(),css:"#word{display:block;width:48px;font-family:Inter;font-size:16px;line-height:24px}".into(),width:240.,height:200.,assets:[("inter".into(),Asset::Font{family:"Inter".into(),weight:400,bytes:include_bytes!("assets/Inter-Regular.ttf").to_vec()})].into(),..Default::default()}
}
#[test]
fn normal_text_requires_an_explicit_occurrence_wrap_policy() {
    let output=compile(&input()).unwrap();
    let manifest=serde_json::to_value(&output.runtime_requirements).unwrap();
    assert!(manifest["capabilities"].as_array().unwrap().iter().any(|v|v=="text-css-normal-wrap-v1"));
    assert!(manifest["text_policies"].as_array().unwrap().iter().any(|v|v["policy"]=="css-normal-wrap-v1"));
}

#[test]
fn normal_wrap_manifest_rejects_missing_capability_and_bad_targets() {
    use nuxie_html_to_riv::{RuntimeCapability, RuntimeRequirements};
    let output=compile(&input()).unwrap();
    let requirements=&output.runtime_requirements;
    let supported=requirements.capabilities.iter().copied().collect::<Vec<_>>();
    requirements.ensure_supported(&supported).unwrap();
    let without=supported.iter().copied().filter(|c|*c!=RuntimeCapability::TextCssNormalWrapV1).collect::<Vec<_>>();
    assert_eq!(requirements.ensure_supported(&without).unwrap_err().code,"missing-runtime-capability");
    assert_eq!(requirements.ensure_text_targets(|_|false).unwrap_err().code,"invalid-text-policy-target");
    let original=serde_json::to_value(requirements).unwrap();
    for kind in ["version","missing-policy","missing-capability","duplicate"] {
        let mut value=original.clone();
        match kind {
            "version"=>value["version"]=1.into(),
            "missing-policy"=>value["text_policies"]=serde_json::json!([]),
            "missing-capability"=>value["capabilities"]=serde_json::json!([]),
            _=>{let policy=value["text_policies"][0].clone();value["text_policies"].as_array_mut().unwrap().push(policy);},
        }
        let r:RuntimeRequirements=serde_json::from_value(value).unwrap();
        assert_eq!(r.ensure_supported(&supported).unwrap_err().code,"invalid-text-policies","{kind}");
    }
}
