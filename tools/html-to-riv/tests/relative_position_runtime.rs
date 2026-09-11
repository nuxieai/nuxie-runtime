use nuxie_html_to_riv::{compile, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{core::binary_reader::BinaryReader, layout_component::LayoutComponent};
use nuxie_runtime::{CoreHandle, File, RuntimeFactoryHandle};

fn inject(style: &CoreHandle, key: u16, bytes: &[u8]) {
    let mut reader = BinaryReader::new(bytes);
    assert_eq!(style.with_mut(|o| o.deserialize(key, &mut reader)), Some(true));
    assert!(reader.reached_end() && !reader.has_error());
}

// Inject existing wire offsets before public CSS syntax is admitted. This tests
// Rive import, occurrence policy, actual layout, clone ownership and resizing.
#[test]
fn relative_offsets_match_chrome_after_import_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/relative-position-oracle.json")).unwrap();
    let mut failures = Vec::new();
    for case in oracle["cases"].as_array().unwrap() {
        let css = case["css"].as_str().unwrap().rsplit_once("#card{").unwrap().0;
        let output = compile(&CompileInput { html: case["html"].as_str().unwrap().into(), css: css.into(), width:390.,height:320.,..Default::default() }).unwrap();
        let mut factory=PersistentFactory::new(RecordingFactory::new());
        let file=File::import(&output.riv,RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),None,None,None).unwrap();
        let original=file.with_file(File::artboard_default).unwrap();
        for id in &output.runtime_requirements.layout_content_box {
            let target=original.with_artboard(|a|a.objects()[*id as usize].clone()).unwrap();
            assert!(LayoutComponent::set_css_content_box_occurrence(&target,true));
        }
        let node=output.source_map.iter().find(|n|n.id=="card").unwrap();
        let target=original.with_artboard(|a|a.objects()[node.object_id as usize].clone()).unwrap();
        let style=target.with(|o|o.as_layout_component().unwrap().style_handle().unwrap()).unwrap();
        let name=case["name"].as_str().unwrap();
        let mode=name.split(if name.contains("-intrinsic-"){ "-intrinsic-" }else{"-definite-"}).nth(1).unwrap();
        // left, right, top, bottom; wire units: px=1, percent=2, auto=3.
        let mut offsets=[(0.0_f32,3_u8);4];
        match mode {
            "baseline"|"auto"|"static-offset"=>{},
            "left"=>offsets[0]=(15.,1), "right"=>offsets[1]=(12.,1),
            "top"=>offsets[2]=(-9.,1), "bottom"=>offsets[3]=(7.,1),
            "opposing"=>offsets=[(15.,1),(90.,1),(11.,1),(80.,1)],
            "percentage"=>{offsets[0]=(10.,2);offsets[2]=(10.,2)},
            "end-percentage"=>{offsets[1]=(10.,2);offsets[3]=(10.,2)},
            "units"=>{offsets[0]=(20.,1);offsets[2]=(-16.,1)},
            "inset"=>offsets=[(20.,1),(10.,1),(5.,1),(15.,1)],
            _=>panic!("unknown mode {mode}")
        }
        inject(&style,597,&[1]);
        for (i,(value,unit)) in offsets.into_iter().enumerate() {
            inject(&style,516+i as u16,&value.to_le_bytes());
            inject(&style,621+i as u16,&[unit]);
        }
        assert!(LayoutComponent::set_css_relative_position_occurrence(&target,true));
        assert!(!LayoutComponent::set_css_relative_position_occurrence(&style,true));
        LayoutComponent::sync_style_occurrence(&target);
        LayoutComponent::mark_layout_node_dirty_occurrence(&target,true);
        LayoutComponent::mark_layout_style_dirty_occurrence(&target);
        for (instance_index,instance) in [original.clone(),original.instance().unwrap()].into_iter().enumerate() {
            for width in [240.,390.,768.,240.] {
                instance.set_size(width,320.);instance.update_pass(true);
                let positioned=instance.with_artboard(|a|a.objects()[node.object_id as usize].clone()).unwrap();
                assert!(positioned.with(|o|o.as_layout_component().unwrap().taffy_style().css_relative_position).unwrap(), "{name} instance{instance_index} lost policy");
                let reference=case["viewports"].as_array().unwrap().iter().find(|v|v["width"].as_f64()==Some(width as f64)).unwrap();
                for source in &output.source_map {
                    let owner=instance.with_artboard(|a|a.objects()[source.object_id as usize].clone()).unwrap();
                    let actual=owner.with(|o|{let l=o.as_layout_component().unwrap().layout();let t=o.as_world_transform_component().unwrap().world_transform();[t[4],t[5],l.width(),l.height()]}).unwrap();
                    for (i,axis) in ["x","y","width","height"].into_iter().enumerate() {
                        let expected=reference["boxes"][&source.id][axis].as_f64().unwrap() as f32;
                        if !actual[i].is_finite() || (actual[i]-expected).abs()>0.1 { failures.push(format!("{name} instance{instance_index} width{width} {}.{axis}: {} vs {expected}",source.id,actual[i])); }
                    }
                }
            }
        }
        if name=="relative-position-row-border-box-intrinsic-percentage" {
            let retained=original.instance().unwrap();
            assert!(LayoutComponent::set_css_relative_position_occurrence(&target,false));
            for (instance,enabled) in [(original.clone(),false),(retained,true)] {
                instance.set_size(240.,320.);instance.update_pass(true);
                let owner=instance.with_artboard(|a|a.objects()[node.object_id as usize].clone()).unwrap();
                assert_eq!(owner.with(|o|o.as_layout_component().unwrap().taffy_style().css_relative_position).unwrap(), enabled);
                let actual=owner.with(|o|o.as_world_transform_component().unwrap().world_transform()[5]).unwrap();
                // Runtime layout passes already resolve this fixture even with
                // the policy disabled. Test ownership, not an engine-only failure.
                if (actual-27.).abs()>0.1 {failures.push(format!("cleared/retained y {actual} vs 27"));}
            }
        }
    }
    assert!(failures.is_empty(),"{} mismatches:\n{}",failures.len(),failures.join("\n"));
}
