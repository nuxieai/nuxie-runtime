use nuxie_html_to_riv::{compile, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};

fn input(direction:&str)->CompileInput {
    CompileInput {html:"<div id='root'><div id='a'></div><div id='b'></div></div>".into(),css:format!("#root{{width:100%;height:100%;padding:10px;gap:8px;flex-direction:{direction};align-items:flex-start}}#a{{width:40px;height:20px}}#b{{width:60px;height:30px}}"),width:390.,height:200.,..Default::default()}
}
#[test]
fn reverse_directions_resize_without_reordering_source_objects() {
    for (direction,forward,row) in [("row-reverse","row",true),("column-reverse","column",false)] {
        let output=compile(&input(direction)).unwrap();
        let normal=compile(&input(forward)).unwrap();
        assert_eq!(serde_json::to_value(&output.source_map).unwrap(),serde_json::to_value(&normal.source_map).unwrap());
        let mut factory=PersistentFactory::new(RecordingFactory::new());
        let file=File::import(&output.riv,RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),None,None,None).unwrap();
        let artboard=file.with_file(File::artboard_default).unwrap();
        for (width,height) in [(240.,180.),(390.,260.),(768.,400.)] {
            artboard.set_size(width,height);artboard.update_pass(true);
            for (id,expected) in [("a",if row {(width-50.,10.)}else{(10.,height-30.)}),("b",if row {(width-118.,10.)}else{(10.,height-68.)})] {
                let node=output.source_map.iter().find(|n|n.id==id).unwrap();
                let object=artboard.with_artboard(|a|a.objects()[node.object_id as usize].clone()).unwrap();
                let actual=object.with(|o|{let t=o.as_world_transform_component().unwrap().world_transform();(t[4],t[5])}).unwrap();
                assert!((actual.0-expected.0).abs()<0.01&&(actual.1-expected.1).abs()<0.01,"{direction} {width}x{height} {id}: {actual:?} vs {expected:?}");
            }
        }
    }
}
#[test]
fn reverse_direction_cascade_and_substitution_preserve_semantics() {
    for direction in ["row-reverse","column-reverse"] {
        let direct=compile(&input(direction)).unwrap();
        let mut substituted=input(direction);substituted.css=substituted.css.replace(&format!("flex-direction:{direction}"),&format!("--direction:{direction};flex-direction:var(--direction)"));
        assert!(direct.riv==compile(&substituted).unwrap().riv);
        let mut inherited=input(direction);inherited.css.push_str("#a{flex-direction:inherit}");
        let mut explicit=input(direction);explicit.css.push_str(&format!("#a{{flex-direction:{direction}}}"));
        assert!(compile(&inherited).unwrap().riv==compile(&explicit).unwrap().riv);
        let mut reset=input(direction);reset.css.push_str("#root{flex-direction:unset}");
        assert!(compile(&reset).unwrap().riv==compile(&input("row")).unwrap().riv);
    }
}
