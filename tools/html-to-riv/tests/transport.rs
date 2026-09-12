//! CLI transport agrees with the library and emits no runtime policy sidecar.
use nuxie_html_to_riv::{CompileInput, compile};
use std::{fs, process::Command, sync::atomic::{AtomicUsize, Ordering}};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(std::path::PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("immutable-transport-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir(&path).unwrap(); Self(path)
    }
}
impl Drop for Temp { fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); } }

#[test]
fn cli_bytes_and_source_map_equal_library_without_requirements() {
    let temp=Temp::new();
    let input = CompileInput { assets: Default::default(), html:"<div id=box></div>".into(), css:"#box{width:100px;height:40px;background-color:red;}".into(), width:240.,height:160. };
    let expected=compile(&input).unwrap();
    fs::write(temp.0.join("input.json"), serde_json::json!({"html":input.html,"css":input.css,"width":input.width,"height":input.height}).to_string()).unwrap();
    let result=Command::new(env!("CARGO_BIN_EXE_html-to-riv")).arg(temp.0.join("input.json")).arg(temp.0.join("scene.riv")).output().unwrap();
    assert!(result.status.success(), "{}",String::from_utf8_lossy(&result.stderr));
    assert_eq!(fs::read(temp.0.join("scene.riv")).unwrap(),expected.riv);
    let map:serde_json::Value=serde_json::from_slice(&fs::read(temp.0.join("scene.map.json")).unwrap()).unwrap();
    assert_eq!(map,serde_json::to_value(expected.source_map).unwrap());
    assert!(!temp.0.join("scene.requirements.json").exists());
}

#[test]
fn malformed_and_unknown_input_fields_do_not_publish() {
    for raw in ["{",r#"{"html":"","css":"","width":240,"height":160,"assets":[]}"#,r#"{"html":"","css":"","width":240,"height":160,"runtimeRequirements":{}}"#] {
        let temp=Temp::new();fs::write(temp.0.join("input.json"),raw).unwrap();
        let result=Command::new(env!("CARGO_BIN_EXE_html-to-riv")).arg(temp.0.join("input.json")).arg(temp.0.join("scene.riv")).output().unwrap();
        assert!(!result.status.success());
        let diagnostics:serde_json::Value=serde_json::from_slice(&result.stderr).unwrap();
        assert_eq!(diagnostics[0]["code"],"invalid-request");
        assert!(!temp.0.join("scene.riv").exists());assert!(!temp.0.join("scene.map.json").exists());
    }
}

#[test]
fn positional_requests_cannot_replace_existing_named_object_output() {
    let temp=Temp::new();
    let good=CompileInput { assets: Default::default(),html:"<div id=box></div>".into(),css:"#box{width:20px;height:10px;background:teal}".into(),width:240.,height:160.};
    let request=temp.0.join("input.json");let output=temp.0.join("scene.riv");
    let invoke=||Command::new(env!("CARGO_BIN_EXE_html-to-riv")).arg(&request).arg(&output).output().unwrap();
    fs::write(&request,serde_json::to_vec(&good).unwrap()).unwrap();
    assert!(invoke().status.success());
    let expected_riv=fs::read(&output).unwrap();let expected_map=fs::read(output.with_extension("map.json")).unwrap();
    for bad in [serde_json::json!([good.html,good.css,good.width,good.height]),serde_json::json!([good.html,good.css,good.width,good.height,{}])] {
        fs::write(&request,serde_json::to_vec(&bad).unwrap()).unwrap();
        let result=invoke();assert!(!result.status.success(),"positional JSON must not be a request");
        let diagnostics:serde_json::Value=serde_json::from_slice(&result.stderr).unwrap();
        assert_eq!(diagnostics[0]["code"],"invalid-request");
        assert_eq!(fs::read(&output).unwrap(),expected_riv);
        assert_eq!(fs::read(output.with_extension("map.json")).unwrap(),expected_map);
    }
    fs::write(&request,serde_json::to_vec(&good).unwrap()).unwrap();
    assert!(invoke().status.success());
    assert_eq!(fs::read(&output).unwrap(),expected_riv);
    assert_eq!(fs::read(output.with_extension("map.json")).unwrap(),expected_map);
}
