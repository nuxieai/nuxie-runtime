use nuxie_html_to_riv::{compile, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{core::binary_reader::BinaryReader, layout_component::LayoutComponent};
use nuxie_runtime::{CoreHandle, File, RuntimeFactoryHandle};
use nuxie_runtime::source::artboard::Artboard;

fn inject(style: &CoreHandle, key: u16, bytes: &[u8]) {
    let mut reader = BinaryReader::new(bytes);
    assert_eq!(style.with_mut(|o| o.deserialize(key, &mut reader)), Some(true));
    assert!(reader.reached_end() && !reader.has_error());
}

// Diagnostic only: public absolute syntax remains rejected. Compile equivalent
// relative declarations, then switch the card using the existing runtime wire.
#[test]
fn absolute_wire_matches_chrome_after_clone_and_resize() {
    let oracle: serde_json::Value = serde_json::from_str(include_str!("assets/absolute-position-oracle.json")).unwrap();
    let containing: serde_json::Value = serde_json::from_str(include_str!("assets/absolute-position-containing-block-oracle.json")).unwrap();
    let mut failures = Vec::new();
    for case in oracle["cases"].as_array().unwrap().iter().chain(containing["cases"].as_array().unwrap()) {
        let name=case["name"].as_str().unwrap();
        let output=compile(&CompileInput { html:case["html"].as_str().unwrap().into(),css:case["css"].as_str().unwrap().replace("position:absolute","position:relative"),width:390.,height:320.,..Default::default() }).unwrap();
        let mut factory=PersistentFactory::new(RecordingFactory::new());
        let file=File::import(&output.riv,RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),None,None,None).unwrap();
        let original=file.with_file(File::artboard_default).unwrap();
        let node=output.source_map.iter().find(|n|n.id=="card").unwrap();
        let target=original.with_artboard(|a|a.objects()[node.object_id as usize].clone()).unwrap();
        let style=target.with(|o|o.as_layout_component().unwrap().style_handle().unwrap()).unwrap();
        inject(&style,597,&[2]);
        if let Some(ids) = case["absoluteIds"].as_array() {
            for id in ids {
                let source = output.source_map.iter().find(|n| n.id == id.as_str().unwrap()).unwrap();
                let owner = original.with_artboard(|a| a.objects()[source.object_id as usize].clone()).unwrap();
                let node_style = owner.with(|o| o.as_layout_component().unwrap().style_handle().unwrap()).unwrap();
                inject(&node_style,597,&[2]);
                LayoutComponent::sync_style_occurrence(&owner);
                LayoutComponent::mark_layout_node_dirty_occurrence(&owner,true);
                LayoutComponent::mark_layout_style_dirty_occurrence(&owner);
            }
        }
        let absolute_ids: Vec<_> = case["absoluteIds"].as_array().map(|ids| ids.iter().map(|id|
            output.source_map.iter().find(|n|n.id==id.as_str().unwrap()).unwrap().object_id).collect())
            .unwrap_or_else(|| vec![node.object_id]);
        LayoutComponent::sync_style_occurrence(&target);
        let root = original.with_artboard(|a| a.objects()[0].clone()).unwrap();
        assert!(Artboard::set_css_absolute_position_policy_occurrence(&root,
            &output.runtime_requirements.layout_positioned, &absolute_ids));
        let mut duplicate = absolute_ids.clone(); duplicate.push(absolute_ids[0]);
        for (positioned, absolute) in [
            (vec![0], absolute_ids.clone()),
            (vec![u32::MAX], absolute_ids.clone()),
            (output.runtime_requirements.layout_positioned.clone(), duplicate),
            (output.runtime_requirements.layout_positioned.clone(), vec![]),
            (output.runtime_requirements.layout_positioned.clone(), vec![u32::MAX]),
            (vec![], absolute_ids.clone()),
        ] {
            assert!(!Artboard::set_css_absolute_position_policy_occurrence(&root, &positioned, &absolute));
        }
        LayoutComponent::sync_style_occurrence(&target);
        LayoutComponent::mark_layout_node_dirty_occurrence(&target,true);
        LayoutComponent::mark_layout_style_dirty_occurrence(&target);
        for (instance_index,instance) in [original.clone(),original.instance().unwrap()].into_iter().enumerate() {
            for width in [240.,390.,768.,240.] {
                instance.set_size(width,320.);instance.update_pass(true);
                let reference=case["viewports"].as_array().unwrap().iter().find(|v|v["width"].as_f64()==Some(width as f64)).unwrap();
                for source in &output.source_map {
                    let owner=instance.with_artboard(|a|a.objects()[source.object_id as usize].clone()).unwrap();
                    let actual=owner.with(|o|{let l=o.as_layout_component().unwrap().layout();let t=o.as_world_transform_component().unwrap().world_transform();[t[4],t[5],l.width(),l.height()]}).unwrap();
                    for (i,axis) in ["x","y","width","height"].into_iter().enumerate() {
                        let expected=reference["boxes"][&source.id][axis].as_f64().unwrap() as f32;
                        if !actual[i].is_finite() || (actual[i]-expected).abs()>0.1 {failures.push(format!("{name} instance{instance_index} width{width} {}.{axis}: {} vs {expected}",source.id,actual[i]));}
                    }
                }
            }
        }
        if name == "absolute-position-row-static-ancestor" {
            let retained = original.instance().unwrap();
            // Clearing a host policy must invalidate the solved containing block;
            // reinstalling it must restore responsive geometry without touching clones.
            for enabled in [false, true, false, true] {
                let (positioned, absolute) = if enabled {
                    (output.runtime_requirements.layout_positioned.as_slice(), absolute_ids.as_slice())
                } else {
                    (&[][..], &[][..])
                };
                assert!(Artboard::set_css_absolute_position_policy_occurrence(&root, positioned, absolute));
                for (instance, active) in [(original.clone(), enabled), (retained.clone(), true)] {
                    for width in [390.0_f32, 768.0, 240.0] {
                        instance.set_size(width, 320.0);
                        instance.update_pass(true);
                        let owner = instance.with_artboard(|a| a.objects()[node.object_id as usize].clone()).unwrap();
                        let actual = owner.with(|o| o.as_world_transform_component().unwrap().world_transform()[4]).unwrap();
                        let expected = if active { width * 0.1 } else { 32.0 };
                        assert!((actual - expected).abs() < 0.1,
                            "installer enabled={enabled}, active={active}, width={width}, actual={actual}, expected={expected}");
                    }
                }
            }
            assert!(!LayoutComponent::set_css_positioned_occurrence(&style, Some(true)));
            for policy in [None, Some(true), None, Some(true)] {
                assert!(LayoutComponent::set_css_positioned_occurrence(&target, policy));
                for (instance, enabled) in [(original.clone(), policy.is_some()), (retained.clone(), true)] {
                    for width in [390.0_f32, 768.0, 240.0] {
                        instance.set_size(width, 320.0);
                        instance.update_pass(true);
                        let owner = instance.with_artboard(|a| a.objects()[node.object_id as usize].clone()).unwrap();
                        let actual = owner.with(|o| o.as_world_transform_component().unwrap().world_transform()[4]).unwrap();
                        let expected = if enabled { width * 0.1 } else { 32.0 };
                        assert!((actual - expected).abs() < 0.1, "policy={policy:?}, retained={}, width={width}, actual={actual}, expected={expected}", enabled && policy.is_none());
                    }
                }
            }
            let wrapper_id = output.source_map.iter().find(|n| n.id == "wrapper").unwrap().object_id;
            let wrapper = original.with_artboard(|a| a.objects()[wrapper_id as usize].clone()).unwrap();
            for positioned in [true, false, true, false] {
                assert!(LayoutComponent::set_css_positioned_occurrence(&wrapper, Some(positioned)));
                original.set_size(390.0, 320.0);
                original.update_pass(true);
                let actual = target.with(|o| o.as_world_transform_component().unwrap().world_transform()[4]).unwrap();
                assert!((actual - if positioned { 32.0 } else { 39.0 }).abs() < 0.1);
            }
        }
    }
    assert!(failures.is_empty(),"{} mismatches:\n{}",failures.len(),failures.join("\n"));
}
