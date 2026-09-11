//! Retained-resource completed-frame latency, including wait/readback, not GPU time.
#[cfg(not(all(feature = "native-metal", target_os = "macos")))]
fn main() { eprintln!("gradient-benchmark requires macOS and --features native-metal"); std::process::exit(2); }

#[cfg(all(feature = "native-metal", target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use nuxie_render_api::{Factory, Renderer, RawPath, FillRule, RenderPaintStyle, BlendMode};
    use nuxie_renderer::{NativeMetalFactory, NativeMetalContextOptions, ShaderCompilationMode, RenderMode};
    use serde_json::json;
    use std::{fs, path::PathBuf, time::Instant};
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 9 { return Err("usage: gradient-benchmark NEW_OUTPUT rust-metal|rust-metal-atomic|rust-metal-clockwise ordinary|css|tiled WIDTH HEIGHT STOPS WARMUPS SAMPLES distinct|shared".into()); }
    let out = PathBuf::from(&args[0]);
    let backend = args[1].as_str(); let kind = args[2].as_str();
    let width: u32 = args[3].parse()?; let height: u32 = args[4].parse()?;
    let count: usize = args[5].parse()?; let warmups: usize = args[6].parse()?; let samples: usize = args[7].parse()?;
    let shared = match args[8].as_str() { "shared" => true, "distinct" => false, _ => return Err("unknown sharing mode".into()) };
    if !matches!(backend, "rust-metal"|"rust-metal-atomic"|"rust-metal-clockwise") || !matches!(kind,"ordinary"|"css"|"tiled")
        || !(2..=256).contains(&count) || width < 48 || height < 48 || width > 2048 || height > 2048
        || samples == 0 || samples > 10000 || warmups > 10000 { return Err("invalid benchmark configuration".into()); }
    fs::create_dir(&out)?;
    let started = Instant::now();
    let options = NativeMetalContextOptions { shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous, disable_framebuffer_reads:false, ..Default::default() };
    let mut factory = if backend == "rust-metal" { NativeMetalFactory::new_with_context_options(width,height,options)? }
        else { NativeMetalFactory::new_with_mode_and_context_options(width,height,
            if backend=="rust-metal-atomic" {RenderMode::Atomics} else {RenderMode::ClockwiseAtomic},options)? };
    let factory_ms = started.elapsed().as_secs_f64()*1000.;
    let resource_start = Instant::now();
    let stops: Vec<f32> = (0..count).map(|i|i as f32/(count-1) as f32).collect();
    let mut resources = Vec::new();
    let mut specifications = Vec::new();
    for index in 0..12 {
        let left = (index%4) as f32*width as f32/4.; let top=(index/4) as f32*height as f32/3.;
        let w=width as f32/4.; let h=height as f32/3.;
        let span = if kind=="tiled" { w/2. } else { w };
        let colors: Vec<u32> = stops.iter().map(|t|0xff000000|(((255.*(1.-t)).round() as u32)<<16)
            |((if shared {0} else {index*19})<<8)|(255.*t).round() as u32).collect();
        let shader = match kind {
            "ordinary" => factory.make_linear_gradient(left,top,left+span,top,&colors,&stops),
            "css" => factory.make_premultiplied_linear_gradient(left,top,left+span,top,&colors,&stops).ok_or("CSS unavailable")?,
            _ => factory.make_tiled_premultiplied_linear_gradient(left,top,left+span,top,[left,top,span,h],&colors,&stops).ok_or("tiled CSS unavailable")?,
        };
        let mut paint=factory.make_render_paint(); paint.style(RenderPaintStyle::Fill);paint.color(0xffffffff);
        paint.blend_mode(BlendMode::SrcOver);paint.shader(Some(shader.as_ref()));
        let mut path=RawPath::default();path.move_to(left,top);path.line_to(left+w,top);path.line_to(left+w,top+h);path.line_to(left,top+h);path.close();
        resources.push((shader,paint,factory.make_render_path(path,FillRule::NonZero)));
        specifications.push((left,top,w,h,span,colors));
    }
    let resource_ms=resource_start.elapsed().as_secs_f64()*1000.;
    let mut raw=Vec::new(); let mut baseline: Option<Vec<u8>>=None; let mut max_error=0u8; let mut analytic_samples=0usize;
    for iteration in 0..1+warmups+samples {
        let start=Instant::now();let mut frame=factory.begin_frame_for_benchmark(0xffffffff,true)?;
        for (_,paint,path) in &resources {frame.draw_path(path.as_ref(),paint.as_ref());}
        let before_finish=start.elapsed().as_secs_f64()*1000.;
        let finish_start=Instant::now();let result=frame.finish_for_benchmark()?;
        let finish_ms=finish_start.elapsed().as_secs_f64()*1000.;let total_ms=start.elapsed().as_secs_f64()*1000.;
        let phase=if iteration==0 {"cold"} else if iteration<=warmups {"warmup"} else {"measured"};
        raw.push(json!({"iteration":iteration,"phase":phase,"totalMs":total_ms,"beginAndDrawMs":before_finish,"finishWaitReadbackMs":finish_ms,
            "backendWork":{"queueSubmissions":result.backend_work.queue_submissions,"allFieldsDebug":format!("{:?}",result.backend_work)},
            "executionInventory":{"drawCalls":result.execution_inventory.draw_calls,"logicalFlushes":result.execution_inventory.logical_flushes,
                "gradientTexture":result.execution_inventory.gradient_texture,"allFieldsDebug":format!("{:?}",result.execution_inventory)}}));
        // Correctness validation, allocations and PNG encoding are outside timers.
        if let Some(first)=&baseline { if *first!=result.pixels {return Err(format!("pixels changed in retained frame {iteration}").into());} }
        else {
            for (left,top,w,h,span,colors) in &specifications {
                for y in ((*top as u32+3)..((top+h) as u32-3)).step_by(13) {
                    for x in ((*left as u32+3)..((left+w) as u32-3)).step_by(13) {
                        let position=x as f32+0.5-left;
                        let position=if kind=="tiled" {position.rem_euclid(*span)} else {position};
                        if kind=="tiled" && position.min(span-position)<2. {continue;}
                        let t=(position/span).clamp(0.,1.); let low=((t*(count-1) as f32).floor() as usize).min(count-2);
                        let u=(t-stops[low])/(stops[low+1]-stops[low]);
                        for (channel,shift) in [16,8,0].into_iter().enumerate() {
                            let a=(colors[low]>>shift&255) as f32;let b=(colors[low+1]>>shift&255) as f32;
                            let expected=(a+(b-a)*u).round() as u8;
                            let actual=result.pixels[((y*width+x)*4) as usize+channel];max_error=max_error.max(actual.abs_diff(expected));
                        }
                        analytic_samples+=1;
                    }
                }
            }
            pixel_compare::RgbaImage::new(width,height,result.pixels.clone())?.write_png(out.join("first.png"))?;
            if max_error>2 {return Err(format!("analytic color mismatch {max_error}").into());}
            baseline=Some(result.pixels.clone());
        }
        if iteration==warmups+samples {pixel_compare::RgbaImage::new(width,height,result.pixels)?.write_png(out.join("last.png"))?;}
    }
    let mut times:Vec<f64>=raw.iter().filter(|r|r["phase"]=="measured").map(|r|r["totalMs"].as_f64().unwrap()).collect();
    times.sort_by(f64::total_cmp);let median=(times[(times.len()-1)/2]+times[times.len()/2])/2.;
    let report=json!({"status":"measured","metric":"persistent completed-frame wall latency including GPU completion wait and readback; not GPU-only time or presentation throughput",
        "config":{"backend":backend,"rendererMode":format!("{:?}",factory.render_mode()),"cssEquivalenceEligible":backend!="rust-metal-clockwise","kind":kind,"width":width,"height":height,"stops":count,"gradients":12,"sharedStops":shared,"warmups":warmups,"samples":samples},
        "environment":{"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"adapter":factory.adapter_name(),"parallelism":std::thread::available_parallelism().map(|n|n.get()).ok()},
        "factorySetupMs":factory_ms,"retainedResourceCreationMs":resource_ms,"raw":raw,"medianMs":median,"minMs":times[0],"maxMs":times[times.len()-1],
        "validation":{"analyticSamples":analytic_samples,"maxChannelError":max_error,"allFramesByteIdentical":true},
        "limitations":["synchronous completion and readback are included","backend_work fields not populated by this backend are not measured zero work","no resource rebuilding or resize in timed frames","opaque synthetic tiles only","no universal performance threshold"]});
    fs::write(out.join("receipt.json"),serde_json::to_vec_pretty(&report)?)?;
    println!("{}",json!({"status":"measured","medianMs":median,"output":out}));
    Ok(())
}
