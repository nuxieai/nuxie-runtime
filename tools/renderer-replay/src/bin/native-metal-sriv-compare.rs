//! Same-input SRIV cache validation on a real Metal device. No C++ oracle claim.
#[cfg(target_os = "macos")]
mod apple {
    use nuxie_render_api::{
        serialized_replay::{replay_serialized_commands, SerializedReplayHooks},
        *,
    };
    use nuxie_renderer::{
        NativeMetalContextOptions, NativeMetalFactory, NativeMetalFrame, RenderMode,
        ShaderCompilationMode,
    };
    use nuxie_sriv::{parse_sriv, OpKind, Value};
    use objc2_metal::MTLDevice;
    use pixel_compare::{compare, RgbaImage, Tolerance};
    use sha2::{Digest, Sha256};
    use std::{
        cell::RefCell,
        error::Error,
        fs,
        path::{Path, PathBuf},
        rc::Rc,
    };
    type Result<T> = std::result::Result<T, Box<dyn Error>>;

    struct Options {
        cached: PathBuf,
        fresh: PathBuf,
        output: PathBuf,
        mode: RenderMode,
        frames: usize,
        comparison: String,
    }
    fn options() -> Result<Options> {
        let mut args = std::env::args().skip(1);
        let (mut cached, mut fresh, mut output) = (None, None, None);
        let mut mode = RenderMode::RasterOrdering;
        let mut frames = 164;
        let mut comparison = None;
        while let Some(key) = args.next() {
            let value = args.next().ok_or("option requires value")?;
            match key.as_str() {
                "--cached" => cached = Some(PathBuf::from(value)),
                "--fresh" => fresh = Some(PathBuf::from(value)),
                "--output" => output = Some(PathBuf::from(value)),
                "--frames" => frames = value.parse()?,
                "--comparison" => {
                    if !matches!(
                        value.as_str(),
                        "cache-rebuild" | "upstream-local" | "repeatability"
                    ) {
                        return Err(
                            "--comparison must be cache-rebuild, upstream-local, or repeatability"
                                .into(),
                        );
                    }
                    comparison = Some(value);
                }
                "--mode" => {
                    mode = match value.as_str() {
                        "raster-ordering" => RenderMode::RasterOrdering,
                        "clockwise-atomic" => RenderMode::ClockwiseAtomic,
                        _ => return Err("unsupported Metal mode".into()),
                    }
                }
                _ => return Err(format!("unknown option {key}").into()),
            }
        }
        if frames == 0 {
            return Err("positive frame count required".into());
        }
        Ok(Options {
            cached: cached.ok_or("--cached required")?,
            fresh: fresh.ok_or("--fresh required")?,
            output: output.ok_or("--output required")?,
            mode,
            frames,
            comparison: comparison.ok_or("--comparison required")?,
        })
    }
    fn input(path: &Path, frames: usize) -> Result<(Vec<u8>, u32, u32)> {
        let mut bytes = fs::read(path)?;
        let stream = parse_sriv(&bytes)?;
        let mut size = None;
        let mut drawn = std::collections::BTreeSet::new();
        for op in &stream.operations {
            match op.kind {
                OpKind::FrameSize => {
                    let n = |i: usize| match op.fields[i].value {
                        Value::Uint(v) => u32::try_from(v).ok(),
                        _ => None,
                    };
                    let current = (n(0).ok_or("invalid width")?, n(1).ok_or("invalid height")?);
                    if size.is_some_and(|s| s != current) {
                        return Err("changing frame sizes unsupported".into());
                    }
                    size = Some(current);
                }
                OpKind::DrawPath => {
                    drawn.insert(op.frame);
                }
                OpKind::MakeRenderPath
                | OpKind::MakeRenderPaint
                | OpKind::MakeLinearGradient
                | OpKind::MakeRadialGradient
                | OpKind::AddRawPath
                | OpKind::Rewind
                | OpKind::FillRule
                | OpKind::Style
                | OpKind::Color
                | OpKind::Thickness
                | OpKind::Join
                | OpKind::Cap
                | OpKind::Feather
                | OpKind::BlendMode
                | OpKind::Shader
                | OpKind::Save
                | OpKind::Restore
                | OpKind::Transform
                | OpKind::ClipPath
                | OpKind::Frame
                | OpKind::ModulateOpacity
                | OpKind::ModulateColor => {}
                _ => {
                    return Err(format!(
                        "unsupported operation {}: this bounded harness must not silently drop it",
                        op.kind
                    )
                    .into());
                }
            }
        }
        if drawn.len() != frames || drawn.iter().copied().ne(0..frames) {
            return Err(
                format!("expected {frames} consecutive nonempty frames; got {drawn:?}").into(),
            );
        }
        let (w, h) = size.ok_or("missing frameSize")?;
        if w == 0 || h == 0 {
            return Err("empty dimensions".into());
        }
        // Flush the last frame while replay's resource maps still own all paths/paints.
        // Input was validated before appending; an already terminated frame stays empty.
        bytes.push(OpKind::Frame as u8);
        Ok((bytes, w, h))
    }
    struct State {
        factory: PersistentFactory<NativeMetalFactory>,
        frame: Option<NativeMetalFrame>,
        error: Option<String>,
        width: u32,
        height: u32,
        mode: RenderMode,
        output: PathBuf,
        cache: PathBuf,
        fresh: bool,
        records: Vec<serde_json::Value>,
        differing_frames: usize,
        max_delta: u8,
        different_pixels: u64,
    }
    impl State {
        fn call(&mut self, f: impl FnOnce(&mut NativeMetalFrame)) {
            if self.error.is_some() {
                return;
            }
            if self.frame.is_none() {
                let frame = self.factory.borrow().begin_frame_for_benchmark(0, true);
                match frame {
                    Ok(frame) => self.frame = Some(frame),
                    Err(e) => {
                        self.error = Some(e.to_string());
                        return;
                    }
                }
            }
            f(self.frame.as_mut().unwrap());
        }
        fn finish(&mut self) -> Result<()> {
            let Some(frame) = self.frame.take() else {
                return Ok(());
            };
            let index = self.records.len();
            let output = frame.finish_for_benchmark()?;
            if output.execution_inventory.mode != self.mode {
                return Err("actual Metal mode differs from requested mode".into());
            }
            if output.execution_inventory.draw_calls == 0 {
                return Err(format!("frame{index}: no physical Metal draws").into());
            }
            let nonblank = output
                .pixels
                .chunks_exact(4)
                .filter(|p| *p != [0, 0, 0, 0])
                .count();
            if nonblank == 0 {
                return Err(format!("frame{index}: blank GPU readback").into());
            }
            let image = RgbaImage::new(self.width, self.height, output.pixels)?;
            let file = self.cache.join(format!("{index:04}.png"));
            let mut different_pixels = 0;
            let mut max_channel_delta = 0;
            if self.fresh {
                let expected = RgbaImage::read_png(&file)?;
                let report = compare(&expected, &image, Tolerance::EXACT)?;
                different_pixels = report.different_pixels;
                max_channel_delta = report.max_channel_delta;
                self.max_delta = self.max_delta.max(max_channel_delta);
                self.different_pixels += different_pixels;
                if !report.within_tolerance {
                    if self.differing_frames == 0 {
                        expected.write_png(
                            self.output
                                .join(format!("first-mismatch-{index:04}-cached.png")),
                        )?;
                        image.write_png(
                            self.output
                                .join(format!("first-mismatch-{index:04}-fresh.png")),
                        )?;
                        let delta = expected
                            .pixels
                            .chunks_exact(4)
                            .zip(image.pixels.chunks_exact(4))
                            .flat_map(|(a, b)| {
                                let d = a.iter().zip(b).map(|(a, b)| a.abs_diff(*b)).max().unwrap();
                                [d, 0, 0, 255]
                            })
                            .collect::<Vec<_>>();
                        RgbaImage::new(self.width, self.height, delta)?.write_png(
                            self.output
                                .join(format!("first-mismatch-{index:04}-delta.png")),
                        )?;
                    }
                    self.differing_frames += 1;
                }
                fs::remove_file(file)?;
            } else {
                image.write_png(&file)?;
            }
            if index == 0 {
                image.write_png(self.output.join(if self.fresh {
                    "sample-fresh.png"
                } else {
                    "sample-cached.png"
                }))?;
            }
            println!(
                "lane={} frame={index} nonblank_pixels={nonblank} gpu_draw_calls={} different_pixels={different_pixels} max_channel_delta={max_channel_delta}",
                if self.fresh { "fresh" } else { "cached" },
                output.execution_inventory.draw_calls
            );
            self.records.push(serde_json::json!({"frame":index,"nonblank_pixels":nonblank,"gpu_draw_calls":output.execution_inventory.draw_calls,"different_pixels":different_pixels,"max_channel_delta":max_channel_delta}));
            Ok(())
        }
    }
    struct FrameProxy(Rc<RefCell<State>>);
    impl Renderer for FrameProxy {
        fn save(&mut self) {
            self.0.borrow_mut().call(|f| f.save());
        }
        fn restore(&mut self) {
            self.0.borrow_mut().call(|f| f.restore());
        }
        fn transform(&mut self, m: Mat2D) {
            self.0.borrow_mut().call(|f| f.transform(m));
        }
        fn draw_path(&mut self, p: &dyn RenderPath, c: &dyn RenderPaint) {
            self.0.borrow_mut().call(|f| f.draw_path(p, c));
        }
        fn clip_path(&mut self, p: &dyn RenderPath) {
            self.0.borrow_mut().call(|f| f.clip_path(p));
        }
        fn draw_image(
            &mut self,
            i: Option<&dyn RenderImage>,
            s: ImageSampler,
            b: BlendMode,
            o: f32,
        ) {
            self.0.borrow_mut().call(|f| f.draw_image(i, s, b, o));
        }
        fn draw_image_mesh(
            &mut self,
            i: Option<&dyn RenderImage>,
            s: ImageSampler,
            v: Option<&dyn RenderBuffer>,
            uv: Option<&dyn RenderBuffer>,
            indices: Option<&dyn RenderBuffer>,
            vc: u32,
            ic: u32,
            b: BlendMode,
            o: f32,
        ) {
            self.0
                .borrow_mut()
                .call(|f| f.draw_image_mesh(i, s, v, uv, indices, vc, ic, b, o));
        }
        fn modulate_opacity(&mut self, o: f32) {
            self.0.borrow_mut().call(|f| f.modulate_opacity(o));
        }
        fn modulate_color(&mut self, c: ColorInt, r: bool) {
            self.0.borrow_mut().call(|f| f.modulate_color(c, r));
        }
    }
    fn replay(
        bytes: &[u8],
        width: u32,
        height: u32,
        options: &Options,
        fresh: bool,
    ) -> Result<(String, u64, State)> {
        let mut native = NativeMetalFactory::new_with_mode_and_context_options(
            width,
            height,
            options.mode,
            NativeMetalContextOptions {
                shader_compilation_mode: ShaderCompilationMode::AlwaysSynchronous,
                disable_framebuffer_reads: false,
                ..Default::default()
            },
        )?;
        // Upstream supports an infinite budget specifically to remove adaptive
        // wall-clock decisions. Keep both lanes on the same triangulation policy.
        native.use_deterministic_validation_thresholds();
        let adapter = native.adapter_name();
        let device_id = native.retained_metal_device().registryID();
        let mut factory = PersistentFactory::new(native);
        let state = Rc::new(RefCell::new(State {
            factory: factory.clone(),
            frame: None,
            error: None,
            width,
            height,
            mode: options.mode,
            output: options.output.clone(),
            cache: options.output.join("frame-cache"),
            fresh,
            records: vec![],
            differing_frames: 0,
            max_delta: 0,
            different_pixels: 0,
        }));
        let mut renderer = FrameProxy(state.clone());
        let hook = state.clone();
        let mut hooks = SerializedReplayHooks {
            on_frame: Some(Box::new(move || {
                let mut s = hook.borrow_mut();
                if s.error.is_none() {
                    if let Err(e) = s.finish() {
                        s.error = Some(e.to_string());
                    }
                }
            })),
            on_frame_size: Some(Box::new(move |w, h| assert_eq!((w, h), (width, height)))),
            ..Default::default()
        };
        if !replay_serialized_commands(bytes, &mut factory, &mut renderer, &mut hooks) {
            return Err("SRIV replay rejected validated input".into());
        }
        drop(hooks);
        drop(renderer);
        let mut state = Rc::try_unwrap(state)
            .map_err(|_| "frame state still retained")?
            .into_inner();
        if let Some(error) = state.error.take() {
            return Err(error.into());
        }
        if state.frame.is_some() {
            return Err("final frame did not flush inside replay".into());
        }
        if state.records.len() != options.frames {
            return Err(format!(
                "executed {} frames, expected {}",
                state.records.len(),
                options.frames
            )
            .into());
        }
        Ok((adapter, device_id, state))
    }

    fn positive_control(
        options: &Options,
        width: u32,
        height: u32,
        device_id: u64,
    ) -> Result<serde_json::Value> {
        fn sentinel(width: u32, height: u32, color: u32) -> Vec<u8> {
            let mut factory = SerializingFactory::new();
            factory.frame_size(width, height);
            let mut renderer = factory.make_renderer();
            let mut raw = RawPath::new();
            let right = (width as f32).min(24.0);
            let bottom = (height as f32).min(24.0);
            raw.move_to(0.0, 0.0);
            raw.line_to(right, 0.0);
            raw.line_to(right, bottom);
            raw.line_to(0.0, bottom);
            raw.close();
            let path = factory.make_render_path(raw, FillRule::NonZero);
            let mut paint = factory.make_render_paint();
            paint.color(color);
            renderer.draw_path(path.as_ref(), paint.as_ref());
            let mut bytes = factory.bytes().to_vec();
            bytes.push(OpKind::Frame as u8);
            bytes
        }
        let output = options.output.join("positive-control");
        fs::create_dir(&output)?;
        fs::create_dir(output.join("frame-cache"))?;
        let control = Options {
            cached: PathBuf::new(),
            fresh: PathBuf::new(),
            output: output.clone(),
            mode: options.mode,
            frames: 1,
            comparison: options.comparison.clone(),
        };
        let (_, a, cached) = replay(
            &sentinel(width, height, 0xff00ff00),
            width,
            height,
            &control,
            false,
        )?;
        let (_, b, fresh) = replay(
            &sentinel(width, height, 0xffff0000),
            width,
            height,
            &control,
            true,
        )?;
        fs::remove_dir(output.join("frame-cache"))?;
        if a != device_id || b != device_id {
            return Err("positive control changed physical Metal device".into());
        }
        if fresh.differing_frames != 1 || fresh.different_pixels == 0 || fresh.max_delta == 0 {
            return Err("visible color sentinel was not detected by GPU replay/comparison".into());
        }
        let report = serde_json::json!({"expected_difference":true,"different_pixels":fresh.different_pixels,"max_channel_delta":fresh.max_delta,"cached":cached.records,"fresh":fresh.records});
        fs::write(
            output.join("report.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        Ok(report)
    }
    pub fn run() -> Result<()> {
        let options = options()?;
        let (cached, w, h) = input(&options.cached, options.frames)?;
        let (fresh, fw, fh) = input(&options.fresh, options.frames)?;
        if (w, h) != (fw, fh) {
            return Err("cached/fresh dimensions differ".into());
        }
        if options.comparison == "repeatability" && cached != fresh {
            return Err("repeatability requires byte-identical inputs".into());
        }
        if options.comparison != "repeatability" && cached == fresh {
            return Err(
                "comparison inputs are identical; no changed-state control was supplied".into(),
            );
        }
        let description = match options.comparison.as_str() {
            "repeatability" => "Byte-identical SRIV replay repeated on the same Metal device and mode",
            "cache-rebuild" => "Rust CSS replay cached versus forced-fresh local paths",
            "upstream-local" => {
                "Rust CSS replay cached versus nine pinned upstream local-path substitutions; not whole-upstream rendering"
            }
            _ => unreachable!(),
        };
        let identity = |path: &Path, bytes: &[u8]| serde_json::json!({"path":path,"bytes":bytes.len()-1,"sha256":format!("{:x}",Sha256::digest(&bytes[..bytes.len()-1]))});
        let cached_identity = identity(&options.cached, &cached);
        let candidate_identity = identity(&options.fresh, &fresh);
        fs::create_dir_all(&options.output)?;
        if fs::read_dir(&options.output)?.next().is_some() {
            return Err("comparison requires an empty output directory".into());
        }
        let cache = options.output.join("frame-cache");
        fs::create_dir(&cache)?;
        let (adapter, device_id, cached_result) = replay(&cached, w, h, &options, false)?;
        let (fresh_adapter, fresh_device, fresh_result) = replay(&fresh, w, h, &options, true)?;
        if adapter != fresh_adapter || device_id != fresh_device {
            return Err("cached/fresh physical adapters differ".into());
        }
        fs::remove_dir(&cache)?;
        let positive_control = positive_control(&options, w, h, device_id)?;
        let report = serde_json::json!({"triangulation_policy":"infinite-budget","positive_control":positive_control,"comparison":options.comparison,"description":description,"not_cpp_renderer_oracle":true,"inputs":{"cached":cached_identity,"candidate":candidate_identity},"adapter":adapter,"metal_registry_id":device_id,"mode":format!("{:?}",options.mode),"width":w,"height":h,"clear_rgba":0,"expected_frames":options.frames,"cached_frames":cached_result.records,"fresh_frames":fresh_result.records,"differing_frames":fresh_result.differing_frames,"different_pixels":fresh_result.different_pixels,"max_channel_delta":fresh_result.max_delta,"exact":fresh_result.differing_frames==0});
        fs::write(
            options.output.join("report.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        println!(
            "adapter={adapter} metal_registry_id={device_id} mode={:?} size={w}x{h} frames={} differing_frames={} different_pixels={} max_channel_delta={}",
            options.mode,
            options.frames,
            fresh_result.differing_frames,
            fresh_result.different_pixels,
            fresh_result.max_delta
        );
        if fresh_result.differing_frames != 0 {
            return Err("GPU pixels differ; inspect report.json and first-mismatch PNGs".into());
        }
        Ok(())
    }
}
#[cfg(target_os = "macos")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    objc2::rc::autoreleasepool(|_| apple::run())
}
#[cfg(not(target_os = "macos"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("native-metal-sriv-compare requires a real macOS Metal device".into())
}
