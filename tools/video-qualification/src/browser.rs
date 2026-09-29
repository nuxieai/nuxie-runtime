use nuxie_render_api::PersistentFactory;
#[cfg(feature = "webgpu")]
use nuxie_renderer::NativeWebGpuFactory as ProofFactory;
use nuxie_renderer::RenderMode;
#[cfg(not(feature = "webgpu"))]
use nuxie_renderer::WebGl2Factory as ProofFactory;
use nuxie_runtime::{
    File, RuntimeFactoryHandle,
    source::{artboard::RuntimeArtboardInstanceHandle, core::CoreHandle},
    video::{Video, playback::*},
};
use nuxie_video_host::{
    browser::{BrowserScenePlayer, Frame, FramePixels},
    pool::ManagedPlayer,
    sync::RegisteredSynchronizationGroup,
};
use wasm_bindgen::prelude::*;

fn error(e: impl std::fmt::Debug) -> JsValue {
    JsValue::from_str(&format!("{e:?}"))
}

/// Per-video upload state: WebGPU copies each decoded frame's canvas into reused
/// textures on the GPU; WebGL2 uploads bytes.
#[cfg(feature = "webgpu")]
type FrameTextures = nuxie_renderer::ExternalImageTextures;
#[cfg(not(feature = "webgpu"))]
type FrameTextures = ();

/// Upload a decoded frame the way the product does on this backend. Returns
/// the image and the decoded top-left pixel, an oracle independent of the
/// renderer.
fn upload_frame(
    factory: &PersistentFactory<ProofFactory>,
    textures: &mut FrameTextures,
    frame: &Frame,
) -> Result<(std::rc::Rc<dyn nuxie_render_api::RenderImage>, Vec<u8>), JsValue> {
    #[cfg(feature = "webgpu")]
    if let FramePixels::Canvas(decoded) = &frame.pixels {
        let pixel = first_pixel(decoded.source())?;
        let image = factory
            .borrow()
            .copy_external_image(textures, decoded.source(), frame.width, frame.height)
            .map_err(error)?;
        return Ok((image, pixel));
    }
    let _ = textures;
    let rgba = match &frame.pixels {
        FramePixels::Rgba(rgba) => std::borrow::Cow::Borrowed(rgba.as_slice()),
        FramePixels::Canvas(decoded) => std::borrow::Cow::Owned(decoded.read_rgba()?),
    };
    let image = factory
        .borrow()
        .upload_canonical_rgba8_premul_srgb(frame.width, frame.height, frame.width * 4, &rgba)
        .map_err(error)?;
    Ok((std::rc::Rc::from(image), rgba[..4].to_vec()))
}

/// Read one pixel of a decoded frame's canvas. A one-pixel draw keeps the oracle's
/// readback small beside the GPU copy it checks.
#[cfg(feature = "webgpu")]
fn first_pixel(frame: &JsValue) -> Result<Vec<u8>, JsValue> {
    use wasm_bindgen::JsCast;
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| error("oracle needs a document"))?;
    let canvas: web_sys::HtmlCanvasElement = document.create_element("canvas")?.dyn_into()?;
    canvas.set_width(1);
    canvas.set_height(1);
    let context: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")?
        .ok_or_else(|| error("oracle canvas unavailable"))?
        .dyn_into()?;
    let arguments = [frame.clone()]
        .into_iter()
        .chain([0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0].map(JsValue::from_f64))
        .collect::<js_sys::Array>();
    js_sys::Reflect::get(&context, &JsValue::from_str("drawImage"))?
        .dyn_into::<js_sys::Function>()?
        .apply(&context, &arguments)?;
    Ok(context.get_image_data(0.0, 0.0, 1.0, 1.0)?.data().0)
}
#[wasm_bindgen]
pub struct BrowserVideoProof {
    factory: PersistentFactory<ProofFactory>,
    textures: FrameTextures,
    artboard: RuntimeArtboardInstanceHandle,
    video: CoreHandle,
    player: BrowserScenePlayer,
    reclaimed: bool,
    red: bool,
    blue: bool,
    loops: usize,
    rates_observed: u8,
    frames: u32,
    last_pts: f64,
    last_decoded_pixel: [u8; 4],
}
#[wasm_bindgen]
impl BrowserVideoProof {
    #[wasm_bindgen(constructor)]
    pub fn new(
        canvas: web_sys::HtmlCanvasElement,
        source: &str,
    ) -> Result<BrowserVideoProof, JsValue> {
        #[cfg(not(feature = "webgpu"))]
        let backend = ProofFactory::new(canvas, 64, 32).map_err(error)?;
        #[cfg(feature = "webgpu")]
        let backend = {
            let _ = canvas;
            ProofFactory::new(64, 32).map_err(error)?
        };
        let mut factory = PersistentFactory::new(backend);
        let embedded = source == "embedded" || source == "embedded-sync";
        let bytes: Option<&[u8]> = embedded.then_some(if source == "embedded-sync" {
            &include_bytes!("../../../fixtures/video/red-blue-sync.mp4")[..]
        } else {
            &include_bytes!("../../../fixtures/video/red-blue-audio.mp4")[..]
        });
        let file = File::import(
            &super::video_scene_with_media(bytes),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .ok_or_else(|| error("scene import failed"))?;
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        artboard.update_pass(true);
        let video = artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                    .cloned()
            })
            .unwrap();
        video
            .with_downcast_mut::<Video, _>(|v| {
                v.playback = Playback::new(PlaybackSettings {
                    autoplay: true,
                    muted: true,
                    ..Default::default()
                })
            })
            .unwrap();
        let mut player = if embedded {
            BrowserScenePlayer::open_embedded(video.clone(), 1024 * 1024, 5.0, true)?
        } else {
            BrowserScenePlayer::open(video.clone(), source, 1024 * 1024, 5.0, true)?
        };
        for mode in [
            nuxie_runtime::video::resources::Allocation::Hardware,
            nuxie_runtime::video::resources::Allocation::Software,
        ] {
            assert!(
                player
                    .tick(mode, 0.0, |_| panic!("unsupported mode uploaded"))
                    .is_err()
            );
        }
        assert!(!player.owns_decoder());
        Ok(Self {
            factory,
            textures: FrameTextures::default(),
            artboard,
            video,
            player,
            reclaimed: false,
            red: false,
            blue: false,
            loops: 0,
            rates_observed: 0,
            frames: 0,
            last_pts: 0.0,
            last_decoded_pixel: [0; 4],
        })
    }
    fn render_tick(&mut self) -> Result<(), JsValue> {
        let now = web_sys::window().unwrap().performance().unwrap().now() / 1000.0;
        use nuxie_runtime::video::resources::Allocation;
        let mut decoded = None;
        let status = self
            .player
            .tick(Allocation::PlatformManaged, now, |frame| {
                let (image, pixel) = upload_frame(&self.factory, &mut self.textures, frame)?;
                decoded = Some((frame.pts, pixel));
                let view = nuxie_render_api::Factory::make_gpu_canvas_image_view(
                    &mut *self.factory.borrow_mut(),
                    image.clone(),
                )
                .expect("decoded frame must be available to GPU image sampling");
                assert!(std::rc::Rc::ptr_eq(&image, &view));
                Ok(view)
            })?;
        if status.presented {
            let (pts, pixel) = decoded.unwrap();
            let mut render = self
                .factory
                .borrow()
                .begin_frame(0xff000000, RenderMode::Msaa)
                .map_err(error)?;
            self.artboard.draw(&mut render);
            self.last_pts = pts;
            self.last_decoded_pixel.copy_from_slice(&pixel);
            #[cfg(not(feature = "webgpu"))]
            {
                // WebGL readPixels returns bottom-row-first bytes. Normalize
                // only this readback oracle; scene/image coordinates stay
                // unchanged and the other backends already return top row.
                let raw = render.finish().map_err(error)?;
                let pixels: Vec<u8> = raw.rchunks_exact(64 * 4).flatten().copied().collect();
                super::verify_composition(&pixels).map_err(error)?;
                let pixel = &pixels[(16 * 64 + 32) * 4..(16 * 64 + 32) * 4 + 4];
                self.verify_pixel(pixel[0], pixel[2])?;
            }
            #[cfg(feature = "webgpu")]
            render.finish_present().map_err(error)?;
            self.frames += 1;
        }
        Ok(())
    }
    pub fn tick(&mut self) -> Result<bool, JsValue> {
        let now = web_sys::window().unwrap().performance().unwrap().now() / 1000.0;
        use nuxie_runtime::video::resources::Allocation;
        if self.red && !self.reclaimed {
            self.player.tick(Allocation::Poster, now, |_| {
                panic!("denied decoder uploaded")
            })?;
            self.reclaimed = true;
            self.video
                .with_downcast_mut::<Video, _>(|v| {
                    v.playback.enqueue(Command::LoopRange {
                        start: 1.3,
                        end: 1.7,
                    })?;
                    v.playback.enqueue(Command::Rate(0.5))?;
                    v.playback.enqueue(Command::Loop(true))?;
                    v.playback.enqueue(Command::Seek(1.3))
                })
                .unwrap()
                .map_err(error)?;
        }
        self.render_tick()?;
        // Sample the platform clock, independently of the requested settings.
        if let Some(clock) = self.player.clock().filter(|clock| clock.playing) {
            if (clock.rate - 0.5).abs() < 0.01 {
                self.rates_observed |= 1;
            }
            if (clock.rate - 1.5).abs() < 0.01 {
                self.rates_observed |= 2;
            }
        }
        self.video
            .with_downcast_mut::<Video, _>(|v| {
                while let Some(event) = v.playback.pop_event() {
                    if event == PlaybackEvent::Looped {
                        self.loops += 1;
                        if self.loops == 1 {
                            v.playback.enqueue(Command::Rate(1.5)).unwrap();
                        }
                        if self.loops == 2 {
                            v.playback.enqueue(Command::Rate(1.0)).unwrap();
                            v.playback.enqueue(Command::Loop(false)).unwrap();
                        }
                    }
                }
            })
            .unwrap();
        let ended = self
            .video
            .with_downcast::<Video, _>(|v| v.playback.state() == PlaybackState::Ended)
            .unwrap();
        if ended {
            if !(self.red
                && self.blue
                && self.frames >= 2
                && self.loops == 2
                && self.rates_observed == 3)
            {
                return Err(error((
                    "incomplete proof",
                    self.red,
                    self.blue,
                    self.frames,
                )));
            }
            self.video
                .with_downcast_mut::<Video, _>(|v| v.playback.enqueue(Command::Dispose))
                .unwrap()
                .map_err(error)?;
            self.player.tick(Allocation::PlatformManaged, now, |_| {
                panic!("disposed decoder uploaded")
            })?;
        }
        Ok(ended)
    }
    pub fn caption(&self) -> String {
        self.video
            .with_downcast::<Video, _>(|v| {
                if v.has_video_frame() {
                    v.caption_text()
                } else {
                    String::new()
                }
            })
            .unwrap_or_default()
    }
    pub fn verify_pixel(&mut self, red: u8, blue: u8) -> Result<(), JsValue> {
        if self.last_pts < 0.9 {
            if !(red > 200 && blue < 30) {
                return Err(error(("red pixel mismatch", self.last_pts, red, blue)));
            }
            if self.caption() != "Red scene" {
                return Err(error("red caption mismatch"));
            }
            self.red = true;
        }
        if self.last_pts > 1.1 {
            if !(blue > 200 && red < 30) {
                return Err(error(("blue pixel mismatch", self.last_pts, red, blue)));
            }
            let expected_caption = if self.last_pts < 2.0 {
                "Blue scene"
            } else {
                ""
            };
            if self.caption() != expected_caption {
                return Err(error("blue caption mismatch"));
            }
            self.blue = true;
        }
        Ok(())
    }
    pub fn verify_composition(&self, pixels: &[u8]) -> Result<(), JsValue> {
        super::verify_composition(pixels).map_err(error)
    }
    pub fn decoded_pixel(&self) -> Vec<u8> {
        self.last_decoded_pixel.to_vec()
    }
    pub fn frames(&self) -> u32 {
        self.frames
    }
    pub fn endpoint_duration(&self) -> f64 {
        self.video
            .with_downcast::<Video, _>(|v| v.playback.duration().unwrap_or(0.0))
            .unwrap()
    }
    pub fn endpoint_scrub(&mut self, progress: f64) -> Result<(), JsValue> {
        self.video
            .with_downcast_mut::<Video, _>(|v| {
                v.playback
                    .scrub(progress, 0.0, v.playback.duration().unwrap())
            })
            .unwrap()
            .map(|_| ())
            .map_err(error)
    }
    pub fn endpoint_tick(&mut self) -> Result<bool, JsValue> {
        self.render_tick()?;
        Ok(self
            .video
            .with_downcast::<Video, _>(|v| {
                v.playback
                    .request_status()
                    .is_some_and(|r| r.state == RequestState::Settled)
            })
            .unwrap())
    }
    pub fn endpoint_pts(&self) -> f64 {
        self.last_pts
    }
}

#[wasm_bindgen]
pub struct BrowserSyncProof {
    leader: BrowserVideoProof,
    follower: BrowserVideoProof,
    group: std::rc::Rc<RegisteredSynchronizationGroup>,
    follower_generation: u64,
    started_follower: bool,
    corrections: u32,
    maximum_drift: f64,
    settled_peak: f64,
    settled_since: Option<f64>,
}
#[wasm_bindgen]
impl BrowserSyncProof {
    #[wasm_bindgen(constructor)]
    pub fn new(leader: BrowserVideoProof, follower: BrowserVideoProof) -> Result<Self, JsValue> {
        follower
            .video
            .with_downcast_mut::<Video, _>(|v| v.playback.enqueue(Command::Pause))
            .unwrap()
            .map_err(error)?;
        let group = RegisteredSynchronizationGroup::new(
            leader.video.clone(),
            vec![follower.video.clone()],
            0.06,
            0.25,
            std::rc::Rc::new(std::cell::Cell::new(true)),
        )
        .map_err(error)?;
        Ok(Self {
            leader,
            follower,
            group,
            follower_generation: 0,
            started_follower: false,
            corrections: 0,
            maximum_drift: 0.0,
            settled_peak: 0.0,
            settled_since: None,
        })
    }
    pub fn tick(&mut self) -> Result<bool, JsValue> {
        let now = web_sys::window().unwrap().performance().unwrap().now() / 1000.0;
        let drift = self
            .leader
            .player
            .clock()
            .zip(self.follower.player.clock())
            .filter(|(leader, follower)| leader.playing && follower.playing)
            .map(|(leader, follower)| leader.seconds - follower.seconds);
        self.leader.render_tick()?;
        self.follower.render_tick()?;
        if let Some(failure) = self.group.take_error() {
            return Err(error(failure));
        }
        let generation = self
            .follower
            .video
            .with_downcast::<Video, _>(|v| v.playback.generation())
            .unwrap();
        if generation != self.follower_generation {
            self.corrections += 1;
            self.follower_generation = generation;
        }
        if !self.started_follower
            && self
                .leader
                .player
                .clock()
                .is_some_and(|c| c.playing && c.seconds >= 0.4)
        {
            self.follower
                .video
                .with_downcast_mut::<Video, _>(|v| v.playback.enqueue(Command::Play))
                .unwrap()
                .map_err(error)?;
            self.started_follower = true;
        }
        if let Some(drift) = drift {
            self.maximum_drift = self.maximum_drift.max(drift.abs());
            if self.corrections > 0 && drift.abs() <= 0.06 {
                self.settled_since.get_or_insert(now);
                self.settled_peak = self.settled_peak.max(drift.abs());
            } else {
                self.settled_since = None;
                self.settled_peak = 0.0;
            }
        } else {
            self.settled_since = None;
            self.settled_peak = 0.0;
        }
        if self.leader.blue
            && self.follower.blue
            && self.settled_since.is_some_and(|s| now - s >= 0.3)
        {
            if self.maximum_drift <= 0.25
                || self.corrections == 0
                || self.leader.frames < 3
                || self.follower.frames < 3
            {
                return Err(error("incomplete synchronization proof"));
            }
            self.group.command(Command::Dispose).map_err(error)?;
            for proof in [&mut self.leader, &mut self.follower] {
                proof.player.tick(
                    nuxie_runtime::video::resources::Allocation::PlatformManaged,
                    now,
                    |_| panic!("disposed sync member uploaded"),
                )?;
                if proof.player.owns_decoder() {
                    return Err(error("sync decoder leaked"));
                }
            }
            return Ok(true);
        }
        Ok(false)
    }
    pub fn frames(&self, follower: bool) -> u32 {
        if follower {
            self.follower.frames
        } else {
            self.leader.frames
        }
    }
    pub fn verify_frame(&mut self, follower: bool, pixels: &[u8]) -> Result<(), JsValue> {
        let proof = if follower {
            &mut self.follower
        } else {
            &mut self.leader
        };
        proof.verify_composition(pixels)?;
        let offset = (16 * 64 + 32) * 4;
        proof.verify_pixel(pixels[offset], pixels[offset + 2])
    }
    pub fn caption(&self) -> String {
        self.follower.caption()
    }
    pub fn metrics(&self) -> String {
        format!(
            "injected drift={:.6}s; corrections={}; settled peak={:.6}s for >=300ms; frames={}/{}; owned decoders={}",
            self.maximum_drift,
            self.corrections,
            self.settled_peak,
            self.leader.frames,
            self.follower.frames,
            usize::from(self.leader.player.owns_decoder())
                + usize::from(self.follower.player.owns_decoder())
        )
    }
}

#[wasm_bindgen]
pub struct BrowserVideoBenchmark {
    factory: PersistentFactory<ProofFactory>,
    textures: FrameTextures,
    artboard: RuntimeArtboardInstanceHandle,
    player: BrowserScenePlayer,
    decoded: Vec<u8>,
}
#[wasm_bindgen]
impl BrowserVideoBenchmark {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: web_sys::HtmlCanvasElement, source: &str) -> Result<Self, JsValue> {
        #[cfg(not(feature = "webgpu"))]
        let backend = ProofFactory::new(canvas, 1280, 720).map_err(error)?;
        #[cfg(feature = "webgpu")]
        let backend = {
            let _ = canvas;
            ProofFactory::new(1280, 720).map_err(error)?
        };
        let mut factory = PersistentFactory::new(backend);
        let file = File::import(
            &super::video_scene_with_dimensions(None, 1280, 720),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .ok_or_else(|| error("benchmark scene import failed"))?;
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        artboard.update_pass(true);
        let video = artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                    .cloned()
            })
            .unwrap();
        video
            .with_downcast_mut::<Video, _>(|v| {
                v.playback = Playback::new(PlaybackSettings {
                    autoplay: true,
                    looping: true,
                    muted: true,
                    ..Default::default()
                })
            })
            .unwrap();
        let player = BrowserScenePlayer::open(video, source, 8 * 1024 * 1024, 5.0, true)?;
        Ok(Self {
            factory,
            textures: FrameTextures::default(),
            artboard,
            player,
            decoded: vec![],
        })
    }
    pub fn tick(&mut self) -> Result<bool, JsValue> {
        let now = web_sys::window().unwrap().performance().unwrap().now() / 1000.0;
        let status = self.player.tick(
            nuxie_runtime::video::resources::Allocation::PlatformManaged,
            now,
            |frame| {
                if (frame.width, frame.height) != (1280, 720) {
                    return Err(error("wrong benchmark resolution"));
                }
                let (image, pixel) = upload_frame(&self.factory, &mut self.textures, frame)?;
                self.decoded = pixel;
                Ok(image)
            },
        )?;
        if status.presented {
            self.artboard.update_pass(true);
            let mut render = self
                .factory
                .borrow()
                .begin_frame(0xff000000, RenderMode::Msaa)
                .map_err(error)?;
            nuxie_render_api::Renderer::scale(&mut render, 20.0, 22.5);
            self.artboard.draw(&mut render);
            #[cfg(not(feature = "webgpu"))]
            {
                let raw = render.finish().map_err(error)?;
                let pixels: Vec<u8> = raw.rchunks_exact(1280 * 4).flatten().copied().collect();
                self.verify_pixels(&pixels)?;
            }
            #[cfg(feature = "webgpu")]
            render.finish_present().map_err(error)?;
        }
        Ok(status.presented)
    }
    pub fn verify_pixels(&self, pixels: &[u8]) -> Result<(), JsValue> {
        if pixels.len() != 1280 * 720 * 4 {
            return Err(error("wrong benchmark output extent"));
        }
        let mut sampled = Vec::with_capacity(64 * 32 * 4);
        for y in 0..32 {
            for x in 0..64 {
                let offset = (((y as f64 + 0.5) * 22.5) as usize * 1280 + x * 20 + 10) * 4;
                sampled.extend_from_slice(&pixels[offset..offset + 4]);
            }
        }
        super::verify_composition(&sampled).map_err(error)?;
        let center = &sampled[(16 * 64 + 32) * 4..][..4];
        if self.decoded.len() != 4
            || center[0].abs_diff(self.decoded[0]) >= 8
            || center[2].abs_diff(self.decoded[2]) >= 8
        {
            return Err(error("benchmark decoded/rendered color mismatch"));
        }
        Ok(())
    }
    pub fn decoded_pixel(&self) -> Vec<u8> {
        self.decoded.clone()
    }
    pub fn close(&mut self) -> Result<(), JsValue> {
        self.player.reclaim()?;
        if self.player.owns_decoder() {
            return Err(error("benchmark decoder retained"));
        }
        Ok(())
    }
}

/// Renders each decoded frame twice, through the WebGPU external copy and
/// through the CPU readback it replaces (Canvas2D bytes, premultiplied, then
/// uploaded), so a page can capture both and compare them pixel by pixel.
/// Each image is drawn alone at 1:1 over a transparent target, so the capture
/// holds its premultiplied colors, its alpha and its orientation.
#[cfg(feature = "webgpu")]
#[wasm_bindgen]
pub struct BrowserUploadParity {
    factory: PersistentFactory<ProofFactory>,
    textures: FrameTextures,
    _artboard: RuntimeArtboardInstanceHandle,
    player: BrowserScenePlayer,
    gpu: Option<std::rc::Rc<dyn nuxie_render_api::RenderImage>>,
    cpu: Option<std::rc::Rc<dyn nuxie_render_api::RenderImage>>,
}

#[cfg(feature = "webgpu")]
#[wasm_bindgen]
impl BrowserUploadParity {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: web_sys::HtmlCanvasElement, source: &str) -> Result<Self, JsValue> {
        let _ = canvas;
        let mut factory = PersistentFactory::new(ProofFactory::new(64, 32).map_err(error)?);
        let file = File::import(
            &super::video_scene_with_dimensions(None, 64, 32),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .ok_or_else(|| error("parity scene import failed"))?;
        let artboard = file.with_file(|f| f.artboard_default()).unwrap();
        artboard.update_pass(true);
        let video = artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.core_type() == Some(Video::TYPE_KEY))
                    .cloned()
            })
            .unwrap();
        video
            .with_downcast_mut::<Video, _>(|v| {
                v.playback = Playback::new(PlaybackSettings {
                    autoplay: true,
                    muted: true,
                    ..Default::default()
                })
            })
            .unwrap();
        let player = BrowserScenePlayer::open(video, source, 64 * 32 * 4, 5.0, true)?;
        Ok(Self {
            factory,
            textures: FrameTextures::default(),
            _artboard: artboard,
            player,
            gpu: None,
            cpu: None,
        })
    }

    /// Advance playback; true when a new frame was uploaded both ways.
    pub fn tick(&mut self) -> Result<bool, JsValue> {
        let now = web_sys::window().unwrap().performance().unwrap().now() / 1000.0;
        let mut uploaded = None;
        let status = self.player.tick(
            nuxie_runtime::video::resources::Allocation::PlatformManaged,
            now,
            |frame| {
                let FramePixels::Canvas(decoded) = &frame.pixels else {
                    return Err(error("parity needs a browser canvas frame"));
                };
                let gpu = self
                    .factory
                    .borrow()
                    .copy_external_image(
                        &mut self.textures,
                        decoded.source(),
                        frame.width,
                        frame.height,
                    )
                    .map_err(error)?;
                // The byte path the GPU copy replaces: straight-alpha Canvas2D
                // bytes, premultiplied on the CPU, then uploaded.
                let mut rgba = decoded.read_rgba()?;
                for pixel in rgba.chunks_exact_mut(4) {
                    let alpha = u16::from(pixel[3]);
                    for color in &mut pixel[..3] {
                        *color = ((u16::from(*color) * alpha + 127) / 255) as u8;
                    }
                }
                let cpu = self
                    .factory
                    .borrow()
                    .upload_canonical_rgba8_premul_srgb(
                        frame.width,
                        frame.height,
                        frame.width * 4,
                        &rgba,
                    )
                    .map_err(error)?;
                uploaded = Some((gpu.clone(), std::rc::Rc::from(cpu)));
                Ok(gpu)
            },
        )?;
        if let Some((gpu, cpu)) = uploaded.filter(|_| status.presented) {
            self.gpu = Some(gpu);
            self.cpu = Some(cpu);
            return Ok(true);
        }
        Ok(false)
    }

    /// Present the latest frame's GPU-copied (`true`) or CPU-uploaded image.
    pub fn draw(&mut self, gpu: bool) -> Result<(), JsValue> {
        let image = (if gpu { &self.gpu } else { &self.cpu })
            .clone()
            .ok_or_else(|| error("no uploaded frame"))?;
        let mut render = self
            .factory
            .borrow()
            .begin_frame(0x00000000, RenderMode::Msaa)
            .map_err(error)?;
        nuxie_render_api::Renderer::draw_image(
            &mut render,
            Some(&*image),
            nuxie_render_api::ImageSampler::default(),
            nuxie_render_api::BlendMode::SrcOver,
            1.0,
        );
        render.finish_present().map_err(error)
    }

    pub fn close(&mut self) -> Result<(), JsValue> {
        self.gpu = None;
        self.cpu = None;
        self.player.reclaim()
    }
}
