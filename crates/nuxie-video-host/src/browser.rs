//! Browser-managed A/V playback with bounded RGBA copies for the renderer.
//! The host supplies CORS-authorized, retained asset URLs or embedded bytes.
use nuxie_runtime::video::playback::{DecoderAction, Playback};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement};

pub use crate::scene::Frame;
pub struct BrowserPlayer {
    video: HtmlVideoElement,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    generation: u64,
    capture_generation: Rc<Cell<u64>>,
    last_capture_pts: Rc<Cell<Option<f64>>>,
    opened: bool,
    disposed: bool,
    ended: bool,
    capture: Rc<RefCell<Option<Result<Frame, JsValue>>>>,
    capture_pending: Rc<Cell<bool>>,
    capture_callback: Option<Closure<dyn FnMut(f64, JsValue)>>,
    capture_request: Option<f64>,
    max_frame_bytes: usize,
    object_url: Option<String>,
    play_blocked: Rc<Cell<bool>>,
    play_attempt: Rc<Cell<u64>>,
}
fn capture_frame(
    video: &HtmlVideoElement,
    canvas: &HtmlCanvasElement,
    context: &CanvasRenderingContext2d,
    generation: u64,
    pts: f64,
    max_bytes: usize,
) -> Result<Frame, JsValue> {
    let (width, height) = (video.video_width(), video.video_height());
    let bytes = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4));
    if bytes.is_none_or(|n| n == 0 || n > max_bytes) {
        return Err(JsValue::from_str("video frame exceeds budget"));
    }
    if canvas.width() != width {
        canvas.set_width(width);
    }
    if canvas.height() != height {
        canvas.set_height(height);
    }
    context.draw_image_with_html_video_element(video, 0.0, 0.0)?;
    let rgba = context
        .get_image_data(0.0, 0.0, width.into(), height.into())?
        .data()
        .0;
    Ok(Frame {
        generation,
        pts,
        width,
        height,
        rgba,
    })
}

impl BrowserPlayer {
    pub fn open(source: &str, generation: u64, max_frame_bytes: usize) -> Result<Self, JsValue> {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or_else(|| JsValue::from_str("video requires a document host"))?;
        let video: HtmlVideoElement = document.create_element("video")?.dyn_into()?;
        video.set_cross_origin(Some("anonymous"));
        video.set_attribute("playsinline", "")?;
        video.set_preload("auto");
        video.set_muted(true);
        video.set_src(source);
        let canvas: HtmlCanvasElement = document.create_element("canvas")?.dyn_into()?;
        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("video canvas unavailable"))?
            .dyn_into()?;
        let play_blocked = Rc::new(Cell::new(false));
        let mut player = Self {
            video,
            canvas,
            context,
            generation,
            capture_generation: Rc::new(Cell::new(generation)),
            last_capture_pts: Rc::new(Cell::new(None)),
            opened: false,
            disposed: false,
            ended: false,
            capture: Rc::new(RefCell::new(None)),
            capture_pending: Rc::new(Cell::new(false)),
            capture_callback: None,
            capture_request: None,
            max_frame_bytes,
            object_url: None,
            play_blocked,
            play_attempt: Rc::new(Cell::new(0)),
        };
        player.arm_capture()?;
        Ok(player)
    }
    fn cancel_capture(&mut self) {
        if let Some(id) = self.capture_request.take() {
            if let Ok(cancel) =
                js_sys::Reflect::get(&self.video, &JsValue::from_str("cancelVideoFrameCallback"))
            {
                if let Some(cancel) = cancel.dyn_ref::<js_sys::Function>() {
                    let _ = cancel.call1(&self.video, &JsValue::from_f64(id));
                }
            }
        }
        self.capture_pending.set(false);
        self.capture_callback = None;
        self.capture.borrow_mut().take();
    }
    fn arm_capture(&mut self) -> Result<(), JsValue> {
        if self.disposed || self.capture_pending.get() {
            return Ok(());
        }
        let request =
            js_sys::Reflect::get(&self.video, &JsValue::from_str("requestVideoFrameCallback"))?
                .dyn_into::<js_sys::Function>()
                .map_err(|_| JsValue::from_str("video host requires requestVideoFrameCallback"))?;
        let video = self.video.clone();
        let canvas = self.canvas.clone();
        let context = self.context.clone();
        let generation = self.capture_generation.clone();
        let last_capture_pts = self.last_capture_pts.clone();
        let max_bytes = self.max_frame_bytes;
        let capture = self.capture.clone();
        let pending = self.capture_pending.clone();
        // Unwinding releases local buffers/borrows before this callback can run
        // again. No capture borrow crosses a JS call; the completed frame is
        // committed only at the end, and pending is cleared before fallible work.
        let callback =
            Closure::wrap_assert_unwind_safe(Box::new(move |_now: f64, metadata: JsValue| {
                pending.set(false);
                if video.seeking() || video.ready_state() < 2 {
                    return;
                }
                let frame = (|| {
                    let pts = js_sys::Reflect::get(&metadata, &JsValue::from_str("mediaTime"))?
                        .as_f64()
                        .filter(|n| n.is_finite() && *n >= 0.0)
                        .ok_or_else(|| JsValue::from_str("invalid video frame timestamp"))?;
                    let frame =
                        capture_frame(&video, &canvas, &context, generation.get(), pts, max_bytes)?;
                    last_capture_pts.set(Some(pts));
                    Ok(frame)
                })();
                *capture.borrow_mut() = Some(frame);
            }) as Box<dyn FnMut(f64, JsValue)>);
        let id = request
            .call1(&self.video, callback.as_ref())?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("invalid video frame callback identifier"))?;
        self.capture_request = Some(id);
        self.capture_callback = Some(callback);
        self.capture_pending.set(true);
        Ok(())
    }
    pub fn open_embedded(
        bytes: &[u8],
        content_type: &str,
        generation: u64,
        max_frame_bytes: usize,
    ) -> Result<Self, JsValue> {
        let parts = js_sys::Array::new();
        parts.push(&js_sys::Uint8Array::from(bytes));
        let options = web_sys::BlobPropertyBag::new();
        options.set_type(content_type);
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options)?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)?;
        match Self::open(&url, generation, max_frame_bytes) {
            Ok(mut player) => {
                player.object_url = Some(url);
                Ok(player)
            }
            Err(error) => {
                let _ = web_sys::Url::revoke_object_url(&url);
                Err(error)
            }
        }
    }
    pub fn apply(&mut self, action: DecoderAction) -> Result<(), JsValue> {
        if self.disposed {
            return Err(JsValue::from_str("video disposed"));
        }
        match action {
            DecoderAction::Play => {
                self.play_blocked.set(false);
                self.play_attempt
                    .set(self.play_attempt.get().wrapping_add(1));
                let attempt = self.play_attempt.get();
                let current_attempt = self.play_attempt.clone();
                let promise = self.video.play()?;
                let blocked = self.play_blocked.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if wasm_bindgen_futures::JsFuture::from(promise).await.is_err()
                        && current_attempt.get() == attempt
                    {
                        blocked.set(true);
                    }
                });
            }
            DecoderAction::Pause => {
                self.play_attempt
                    .set(self.play_attempt.get().wrapping_add(1));
                self.play_blocked.set(false);
                self.video.pause()?;
            }
            DecoderAction::Seek {
                seconds,
                generation,
            } => {
                if !seconds.is_finite() || seconds < 0.0 {
                    return Err(JsValue::from_str("invalid seek"));
                }
                self.play_attempt
                    .set(self.play_attempt.get().wrapping_add(1));
                self.play_blocked.set(false);
                self.generation = generation;
                self.capture_generation.set(generation);
                self.ended = false;
                // A paused seek to the same media clock need not produce a new
                // compositor frame. Preserve the first callback while opening;
                // after presentation, recapture only its known pixel timestamp.
                if self.video.paused()
                    && !self.video.seeking()
                    && (self.video.current_time() - seconds).abs() < f64::EPSILON
                    && self
                        .last_capture_pts
                        .get()
                        .is_none_or(|pts| (pts - seconds).abs() <= 0.05)
                {
                    if let Some(pts) = self
                        .last_capture_pts
                        .get()
                        .filter(|pts| (pts - seconds).abs() <= 0.05)
                    {
                        if self.video.ready_state() >= 2 {
                            self.cancel_capture();
                            let frame = capture_frame(
                                &self.video,
                                &self.canvas,
                                &self.context,
                                generation,
                                pts,
                                self.max_frame_bytes,
                            );
                            *self.capture.borrow_mut() = Some(frame);
                        }
                    } else if let Some(Ok(frame)) = self.capture.borrow_mut().as_mut() {
                        frame.generation = generation;
                    }
                    self.arm_capture()?;
                    return Ok(());
                }
                self.last_capture_pts.set(None);
                self.cancel_capture();
                self.video.set_current_time(seconds);
                self.arm_capture()?;
            }
            DecoderAction::Rate(rate) => {
                if !rate.is_finite() || rate <= 0.0 {
                    return Err(JsValue::from_str("invalid rate"));
                }
                self.video.set_playback_rate(f64::from(rate));
            }
            DecoderAction::Volume(volume) => {
                if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
                    return Err(JsValue::from_str("invalid volume"));
                }
                self.video.set_muted(volume == 0.0);
                self.video.set_volume(f64::from(volume));
            }
            DecoderAction::Dispose => self.close(),
        }
        Ok(())
    }
    pub fn tick(&mut self, playback: &mut Playback) -> Result<Option<Frame>, JsValue> {
        for action in playback.drain_actions() {
            self.apply(action)?;
        }
        if self.disposed {
            return Ok(None);
        }
        if self.video.error().is_some() {
            playback.failed(self.generation);
            self.close();
            return Err(JsValue::from_str("browser media decode failed"));
        }
        if self.play_blocked.replace(false) {
            playback.observed_play_blocked(self.generation);
        }
        if !self.opened && self.video.ready_state() >= 1 && self.video.duration().is_finite() {
            self.opened = true;
            for action in playback.opened(self.generation, self.video.duration()) {
                self.apply(action)?;
            }
        }
        if self.video.ended() && !self.ended {
            self.ended = true;
            for action in playback.ended(self.generation) {
                self.apply(action)?;
            }
        }
        if self.video.seeking() || self.video.ready_state() < 2 {
            return Ok(None);
        }
        if !self.video.paused() {
            if self.video.ready_state() < 3 {
                playback.observed_buffering(self.generation);
            } else {
                playback.observed_playing(self.generation);
            }
        }
        let captured = self.capture.borrow_mut().take();
        self.arm_capture()?;
        match captured {
            Some(Ok(frame)) => Ok(Some(frame)),
            Some(Err(error)) => {
                playback.failed(self.generation);
                self.close();
                Err(error)
            }
            None => Ok(None),
        }
    }

    pub fn close(&mut self) {
        if self.disposed {
            return;
        }
        self.disposed = true;
        self.cancel_capture();
        self.play_attempt
            .set(self.play_attempt.get().wrapping_add(1));
        self.play_blocked.set(false);
        let _ = self.video.pause();
        let _ = self.video.remove_attribute("src");
        self.video.load();
        if let Some(url) = self.object_url.take() {
            let _ = web_sys::Url::revoke_object_url(&url);
        }
    }
}
impl Drop for BrowserPlayer {
    fn drop(&mut self) {
        self.close();
    }
}

impl crate::scene::Decoder for BrowserPlayer {
    type Error = JsValue;
    fn close(&mut self) -> Result<(), Self::Error> {
        BrowserPlayer::close(self);
        Ok(())
    }
    fn clock(&self) -> Option<crate::scene::MediaClock> {
        if self.disposed || self.video.seeking() || self.video.ready_state() < 2 {
            return None;
        }
        let seconds = self.video.current_time();
        if !seconds.is_finite() || seconds < 0.0 {
            return None;
        }
        let playing = !self.video.paused() && !self.video.ended();
        Some(crate::scene::MediaClock {
            generation: self.generation,
            seconds,
            rate: if playing {
                self.video.playback_rate()
            } else {
                0.0
            },
            playing,
        })
    }

    fn apply(&mut self, action: DecoderAction) -> Result<(), Self::Error> {
        BrowserPlayer::apply(self, action)
    }
    fn tick(&mut self, playback: &mut Playback) -> Result<Option<Frame>, Self::Error> {
        BrowserPlayer::tick(self, playback)
    }
}

/// Retained source plus managed scene occurrence. Blob URLs and media elements
/// are recreated only after readmission; encoded source bytes remain shared.
pub struct BrowserScenePlayer {
    scene: crate::scene::ScenePlayer<BrowserPlayer>,
    source: BrowserSource,
    max_frame_bytes: usize,
}
enum BrowserSource {
    External(String),
    Embedded(std::sync::Arc<[u8]>, String),
}
impl BrowserScenePlayer {
    pub fn clock(&self) -> Option<crate::scene::MediaClock> {
        self.scene.clock()
    }

    pub fn open(
        video: nuxie_runtime::source::core::CoreHandle,
        source: &str,
        max_frame_bytes: usize,
        timeout: f64,
        optional: bool,
    ) -> Result<Self, JsValue> {
        Ok(Self {
            scene: crate::scene::ScenePlayer::new(video, timeout, optional)
                .map_err(|e| JsValue::from_str(&format!("{e:?}")))?,
            source: BrowserSource::External(source.into()),
            max_frame_bytes,
        })
    }
    pub fn open_embedded(
        video: nuxie_runtime::source::core::CoreHandle,
        max_frame_bytes: usize,
        timeout: f64,
        optional: bool,
    ) -> Result<Self, JsValue> {
        use nuxie_runtime::video::{Video, VideoAsset};
        let asset = video
            .with_downcast::<Video, _>(|v| v.asset())
            .flatten()
            .ok_or_else(|| JsValue::from_str("missing video asset"))?;
        let bytes = asset
            .with_downcast::<VideoAsset, _>(|a| a.encoded_bytes())
            .flatten()
            .ok_or_else(|| JsValue::from_str("missing embedded video bytes"))?;
        let mime = asset
            .with_downcast::<VideoAsset, _>(|a| a.content_type.clone())
            .ok_or_else(|| JsValue::from_str("missing video content type"))?;
        let mut result = Self::open(video, "", max_frame_bytes, timeout, optional)?;
        result.source = BrowserSource::Embedded(bytes, mime);
        Ok(result)
    }
    /// Call from lifecycle callbacks even when frame updates are stopped.
    pub fn set_suspended(
        &mut self,
        reason: nuxie_runtime::video::playback::SuspensionReason,
        suspended: bool,
    ) -> Result<(), JsValue> {
        self.scene
            .set_suspended(reason, suspended)
            .map_err(|error| JsValue::from_str(&format!("{error:?}")))
    }
    pub fn replace_source(
        &mut self,
        source: &str,
        settings: nuxie_runtime::video::playback::PlaybackSettings,
    ) -> Result<u64, JsValue> {
        if source.is_empty() {
            return Err(JsValue::from_str("missing replacement video source"));
        }
        let generation = self
            .scene
            .replace_source(settings)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        self.source = BrowserSource::External(source.into());
        Ok(generation)
    }
    pub fn tick(
        &mut self,
        allocation: nuxie_runtime::video::resources::Allocation,
        now: f64,
        upload: impl FnMut(&Frame) -> Result<Rc<dyn nuxie_render_api::RenderImage>, JsValue>,
    ) -> Result<crate::scene::SceneStatus, JsValue> {
        if !matches!(
            allocation,
            nuxie_runtime::video::resources::Allocation::PlatformManaged
                | nuxie_runtime::video::resources::Allocation::Poster
        ) {
            return Err(JsValue::from_str(
                "HTMLVideoElement requires platform-managed decoding",
            ));
        }
        self.scene
            .tick(
                allocation,
                now,
                |generation, _| match &self.source {
                    BrowserSource::External(url) => {
                        BrowserPlayer::open(url, generation, self.max_frame_bytes)
                    }
                    BrowserSource::Embedded(bytes, mime) => {
                        BrowserPlayer::open_embedded(bytes, mime, generation, self.max_frame_bytes)
                    }
                },
                upload,
            )
            .map_err(|error| match error {
                crate::scene::SceneError::Decoder(error) => error,
                other => JsValue::from_str(&format!("{other:?}")),
            })
    }
}

impl crate::pool::ManagedPlayer for BrowserScenePlayer {
    type Error = JsValue;
    fn allocation(&self) -> nuxie_runtime::video::resources::Allocation {
        self.scene.allocation()
    }
    fn owns_decoder(&self) -> bool {
        self.scene.owns_decoder()
    }
    fn can_decode(&self) -> bool {
        self.scene.can_decode()
    }
    fn reclaim(&mut self) -> Result<(), Self::Error> {
        self.scene
            .reclaim()
            .map_err(|error| JsValue::from_str(&format!("{error:?}")))
    }
}
