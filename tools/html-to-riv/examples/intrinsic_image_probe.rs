//! Investigate existing intrinsic image measurement without widening CSS admission.
use nuxie_html_to_riv::{compile, Asset, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use nuxie_runtime::source::layout_component::LayoutComponent;

fn main() {
    let output_path = std::env::args().nth(1).expect("NEW_OUTPUT.json");
    assert!(!std::path::Path::new(&output_path).exists(), "Refusing to overwrite evidence");
    let oracle: serde_json::Value = serde_json::from_str(include_str!("../tests/assets/intrinsic-image-initial-oracle.json")).unwrap();
    let oracle: serde_json::Value = if let Some(path) = std::env::args().nth(2) {
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
    } else { oracle };
    let mut rows = Vec::new();
    for case in oracle["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let width_auto = !name.ends_with("percent-width") && !name.ends_with("auto-natural-ratio");
        let height_auto = !name.ends_with("fixed-height") && !name.ends_with("constrained-height");
        let temporary = format!("#photo{{{}{}}}", if width_auto { "width:10px;" } else { "" }, if height_auto { "height:10px;" } else { "" });
        let image_bytes = match case["imageAsset"].as_str() {
            Some("intrinsic-wide.png") => include_bytes!("../tests/assets/intrinsic-wide.png").as_slice(),
            Some("intrinsic-tall.png") => include_bytes!("../tests/assets/intrinsic-tall.png").as_slice(),
            None => include_bytes!("../tests/assets/quadrants.png").as_slice(),
            Some(name) => panic!("Unknown image fixture: {name}"),
        };
        let reader = png::Decoder::new(std::io::Cursor::new(image_bytes)).read_info().unwrap();
        let natural_pair = [reader.info().width, reader.info().height];
        let result = compile(&CompileInput {
            html: case["html"].as_str().unwrap().into(), css: format!("{}{temporary}",case["css"].as_str().unwrap()),
            width:390.,height:320.,assets:[("photo".into(), Asset::Image {bytes:image_bytes.to_vec()})].into(),..Default::default()
        }).unwrap();
        let mut factory=PersistentFactory::new(RecordingFactory::new());
        let file=File::import(&result.riv,RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),None,None,None).unwrap();
        let original=file.with_file(File::artboard_default).unwrap();
        for id in &result.runtime_requirements.layout_content_box {
            let target=original.with_artboard(|a|a.objects()[*id as usize].clone()).unwrap();
            assert!(LayoutComponent::set_css_content_box_occurrence(&target,true));
        }
        let node=result.source_map.iter().find(|n|n.id=="photo").unwrap();
        let target=original.with_artboard(|a|a.objects()[node.object_id as usize].clone()).unwrap();
        let style=target.with(|o|o.as_layout_component().unwrap().style_handle().unwrap()).unwrap();
        let set=|key,bytes:Vec<u8>| {
            let mut reader=nuxie_runtime::source::core::binary_reader::BinaryReader::new(&bytes);
            assert_eq!(style.with_mut(|o|o.deserialize(key,&mut reader)),Some(true));
            assert!(!reader.has_error() && reader.reached_end());
        };
        if width_auto {set(607,vec![3]);set(655,vec![2]);}
        if height_auto {set(608,vec![3]);set(656,vec![2]);}
        set(606,vec![1]);
        let bare=name.ends_with("both-auto-bare-ratio");
        let pair=if bare {[3,2]} else {natural_pair};
        set(524,(pair[0] as f32/pair[1] as f32).to_le_bytes().to_vec());
        assert!(LayoutComponent::set_css_ratio_content_box_occurrence(&target,!bare));
        assert!(LayoutComponent::set_css_ratio_pair_occurrence(&target,Some(pair)));
        LayoutComponent::sync_style_occurrence(&target);
        LayoutComponent::mark_layout_node_dirty_occurrence(&target,true);
        LayoutComponent::mark_layout_style_dirty_occurrence(&target);
        for (instance_index,instance) in [original.clone(),original.instance().unwrap()].into_iter().enumerate() {
            for width in [240.,390.,768.,240.] {
                instance.set_size(width,320.);instance.update_pass(true);
                let reference=case["viewports"].as_array().unwrap().iter().find(|v|v["width"].as_f64()==Some(width as f64)).unwrap();
                let mut errors=Vec::new();
                for node in &result.source_map {
                    let owner=instance.with_artboard(|a|a.objects()[node.object_id as usize].clone()).unwrap();
                    let actual=owner.with(|o| {
                        let bounds=o.as_layout_component().unwrap().layout_bounds();let t=o.as_world_transform_component().unwrap().world_transform();
                        [t[4],t[5],bounds.width(),bounds.height()]
                    }).unwrap();
                    for (i,axis) in ["x","y","width","height"].iter().enumerate() {
                        let expected=reference["boxes"][&node.id][axis].as_f64().unwrap() as f32;
                        if !actual[i].is_finite() || (actual[i]-expected).abs()>0.1 {errors.push(serde_json::json!({"node":node.id,"axis":axis,"actual":actual[i],"expected":expected}));}
                    }
                }
                rows.push(serde_json::json!({"name":name,"instance":instance_index,"width":width,"errors":errors}));
            }
        }
    }
    let failed=rows.iter().filter(|r|!r["errors"].as_array().unwrap().is_empty()).count();
    std::fs::write(output_path,serde_json::to_string_pretty(&serde_json::json!({"status":"runtime experiment only; compiler admission unchanged","comparisons":rows.len(),"failed":failed,"rows":rows})).unwrap()).unwrap();
    println!("{} comparisons, {failed} failed",rows.len());
}
