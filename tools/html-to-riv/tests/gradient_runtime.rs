use nuxie_html_to_riv::{compile, CompileInput};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use nuxie_runtime::source::{artboard::Artboard, css_linear_gradient::{CssLinearGradient,CssGradientDirection}};

#[test]
fn gradient_installation_is_atomic_and_resizes_original_and_clone() {
    // Public CSS admission is still gated; inject only the checked experimental paint.
    let output=compile(&CompileInput {
        html:"<div id='box'></div>".into(),
        css:"#box{width:60%;height:100px}".into(),
        width:390.,height:320.,..Default::default()
    }).unwrap();
    let id=output.source_map.iter().find(|n|n.id=="box").unwrap().object_id;
    let mut factory=PersistentFactory::new(RecordingFactory::new());
    let file=File::import(&output.riv,RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),None,None,None).unwrap();
    let original=file.with_file(File::artboard_default).unwrap();
    let paint=CssLinearGradient::new(CssGradientDirection::Degrees(90.),vec![0xffff0000,0xff0000ff],vec![None,None]).unwrap();
    assert!(Artboard::set_css_linear_gradients_occurrence(&original.core_handle(),&[(id,paint.clone())]));
    original.update_pass(true);
    let cloned=original.instance().unwrap();
    for targets in [vec![(0,paint.clone())],vec![(u32::MAX,paint.clone())],vec![(id,paint.clone()),(id,paint.clone())]] {
        assert!(!Artboard::set_css_linear_gradients_occurrence(&original.core_handle(),&targets));
    }
    for instance in [&original,&cloned] {
        for (width,expected) in [(240.,144.),(390.,234.),(768.,460.8),(240.,144.)] {
            instance.set_size(width,320.);instance.update_pass(true);
            let start=factory.borrow().stream().len();
            let mut renderer=factory.borrow().make_renderer();
            assert!(instance.try_draw(&mut renderer));
            let stream=factory.borrow().stream()[start..].to_string();
            let line=stream.lines().find(|l|l.starts_with("makePremultipliedLinearGradient")).expect("paint recorded");
            assert!(line.contains("start=(0,50)"),"{line}");
            let end=line.split("end=(").nth(1).unwrap().split(',').next().unwrap().parse::<f32>().unwrap();
            assert!((end-expected).abs()<0.001,"{line}");
        }
    }
    assert!(Artboard::set_css_linear_gradients_occurrence(&original.core_handle(),&[]));
    original.update_pass(true);
    let start=factory.borrow().stream().len();
    let mut renderer = factory.borrow().make_renderer();
    assert!(original.try_draw(&mut renderer));
    assert!(!factory.borrow().stream()[start..].contains("makePremultipliedLinearGradient"));
    let start=factory.borrow().stream().len();
    let mut renderer = factory.borrow().make_renderer();
    assert!(cloned.try_draw(&mut renderer));
    assert!(factory.borrow().stream()[start..].contains("makePremultipliedLinearGradient"));
}

#[test]
fn gradient_reference_records_original_and_clone_resize_frames() {
    use nuxie_runtime::source::css_linear_gradient::CssGradientPosition;
    let oracle:serde_json::Value=serde_json::from_str(include_str!("assets/linear-gradient-initial-oracle.json")).unwrap();
    assert_eq!(oracle["browser"],"153.0.8010.12");
    let recording=std::env::var_os("NUXIE_GRADIENT_RECORD_DIR").map(std::path::PathBuf::from);
    if let Some(out)=&recording {std::fs::create_dir(out).unwrap();}
    let mut records=Vec::new();
    for case in oracle["cases"].as_array().unwrap() {
        let name=case["name"].as_str().unwrap();let authored=case["css"].as_str().unwrap();
        let start=authored.find("background:linear-gradient(").unwrap();let end=start+authored[start..].find(';').unwrap()+1;
        let css=format!("{}{}",&authored[..start],&authored[end..]);
        let output=compile(&CompileInput {html:case["html"].as_str().unwrap().into(),css:css.clone(),width:390.,height:320.,..Default::default()}).unwrap();
        let mut factory=PersistentFactory::new(RecordingFactory::new());
        let file=File::import(&output.riv,RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),None,None,None).unwrap();
        let original=file.with_file(File::artboard_default).unwrap();
        let id=output.source_map.iter().find(|n|n.id=="gradient").unwrap().object_id;
        let spec=&case["diagnosticGradient"];
        let direction=if let Some(degrees)=spec["direction"]["degrees"].as_f64() {CssGradientDirection::Degrees(degrees as f32)}
            else {CssGradientDirection::Corner {right:spec["direction"]["corner"][0].as_bool().unwrap(),bottom:spec["direction"]["corner"][1].as_bool().unwrap()}};
        let colors=spec["colors"].as_array().unwrap().iter().map(|v|v.as_u64().unwrap() as u32).collect();
        let positions=spec["positions"].as_array().unwrap().iter().map(|v|if v.is_null(){None}else if let Some(p)=v["percent"].as_f64(){Some(CssGradientPosition::Percent(p as f32))}else{Some(CssGradientPosition::Pixels(v["pixels"].as_f64().unwrap() as f32))}).collect();
        let paint=CssLinearGradient::new(direction,colors,positions).unwrap();
        assert!(Artboard::set_css_linear_gradients_occurrence(&original.core_handle(),&[(id,paint)]));
        original.with_artboard_mut(|a|a.set_css_paint_order(true));
        let pixel_ids=output.runtime_requirements.layout_pixel_bounds.iter().copied().chain(std::iter::once(id)).collect::<Vec<_>>();
        for id in &pixel_ids {original.with_artboard(|a|a.objects()[*id as usize].clone()).unwrap().with_mut(|o|o.as_layout_component_mut().unwrap().set_css_pixel_bounds(true)).unwrap();}
        original.update_pass(true);let cloned=original.instance().unwrap();let mut views=Vec::new();
        for (instance_index,instance) in [&original,&cloned].iter().enumerate() {
            for (step,width) in [240.,390.,768.,240.].into_iter().enumerate() {
                let frame=instance_index*4+step;instance.set_size(width,320.);instance.update_pass(true);
                factory.borrow_mut().frame_size(width as u32,320);factory.borrow_mut().clear_color(0xffffffff);factory.borrow_mut().add_sample(frame as f32);
                let mut renderer=factory.borrow().make_renderer();assert!(instance.try_draw(&mut renderer));factory.borrow_mut().add_frame();
                let reference=case["viewports"].as_array().unwrap().iter().find(|v|v["width"].as_f64()==Some(width as f64)).unwrap();
                let mut bounds=serde_json::Map::new();
                for node in &output.source_map {
                    let owner=instance.with_artboard(|a|a.objects()[node.object_id as usize].clone()).unwrap();
                    let values=owner.with(|o| {let l=o.as_layout_component().unwrap().layout();let t=o.as_world_transform_component().unwrap().world_transform();[t[4],t[5],l.width(),l.height()]}).unwrap();
                    for (i,axis) in ["x","y","width","height"].iter().enumerate(){assert!((values[i] as f64-reference["boxes"][&node.id][axis].as_f64().unwrap()).abs()<0.1,"{name}/{width}/{axis}");}
                    bounds.insert(node.id.clone(),serde_json::json!({"x":values[0],"y":values[1],"width":values[2],"height":values[3]}));
                }
                if let Some(out)=&recording {let path=out.join(format!("{name}-{frame}.stream"));std::fs::write(&path,factory.borrow().stream()).unwrap();views.push(serde_json::json!({"instance":instance_index,"frame":frame,"width":width,"height":320,"stream":path,"bounds":bounds}));}
            }
        }
        if let Some(out)=&recording {std::fs::write(out.join(format!("{name}.riv")),&output.riv).unwrap();records.push(serde_json::json!({"name":name,"html":case["html"],"css":authored,"compilerRequestCss":css,"injectedGradient":spec,"injectedPixelBounds":pixel_ids,"runtimeRequirements":output.runtime_requirements,"views":views,"font":false,"image":false}));}
    }
    if let Some(out)=&recording {std::fs::write(out.join("lifecycle.json"),serde_json::to_vec_pretty(&serde_json::json!({"kind":"gradient-diagnostic","browser":oracle["browser"],"cases":records})).unwrap()).unwrap();}
}
