// Appended only to a frozen compiler copy. Direct primitive descriptors are not CSS proof.
pub(crate) fn validation_fixedpaint_recipe(input: &str, out: &std::path::Path) -> Result<(), String> {
    #[derive(serde::Deserialize)]
    struct R { left:i32, top:i32, right:i32, bottom:i32, color:u32 }
    #[derive(serde::Deserialize)]
    struct Q { width:u32, height:u32, rects:Vec<R> }
    let q:Q=serde_json::from_str(input).map_err(|e|e.to_string())?;
    if out.exists() {return Err("fresh output required".into());}
    let build=||->Result<_,Diagnostic>{
        let mut base=vec![Record::new("Backboard"),Record::new("Artboard")];
        base[1].set("width",Value::Float(q.width as f32))?;
        base[1].set("height",Value::Float(q.height as f32))?;
        let mut fill=Record::new("Fill");fill.set("parentId",Value::Uint(0))?;
        let mut color=Record::new("SolidColor");color.set("parentId",Value::Uint(1))?;color.set("colorValue",Value::Color(0xffffffff))?;
        base.extend([fill,color]);
        let mut rects=Vec::new();
        for r in &q.rects {
            let geometry=base.len() as u32-1;
            let mut owner=Record::new("LayoutComponent");owner.set("parentId",Value::Uint(0))?;owner.set("styleId",Value::Uint(geometry+1))?;
            base.extend([owner,Record::new("LayoutComponentStyle")]);
            rects.push(fixed_paint::Rect{geometry,left:r.left,top:r.top,right:r.right,bottom:r.bottom,color:r.color});
        }
        let folded=fixed_paint::Folded::new(&base,&rects)?;
        folded.validate(&base,&rects)?;
        Ok((wire::encode(folded.records())?,base.len(),folded.records().len()))
    };
    let (bytes,start,end)=build().map_err(|e|format!("{e:?}"))?;
    std::fs::create_dir_all(out).map_err(|e|e.to_string())?;
    std::fs::write(out.join("scene.riv"),bytes).map_err(|e|e.to_string())?;
    std::fs::write(out.join("binding.json"),serde_json::to_vec_pretty(&serde_json::json!({"scope":"direct fixed_paint primitive only; no CSS source proof","start":start,"end":end,"descriptors":q.rects.iter().enumerate().map(|(i,r)|serde_json::json!({"geometry":2*i+3,"left":r.left,"top":r.top,"right":r.right,"bottom":r.bottom,"color":r.color})).collect::<Vec<_>>()})).unwrap()).map_err(|e|e.to_string())?;
    Ok(())
}
