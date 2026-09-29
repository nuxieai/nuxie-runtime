//! JNI owner for the bundled platform MediaPlayer/ImageReader adapter. Each
//! decoded frame arrives in the decoder's hardware buffer, which the host's
//! retained Vulkan factory imports and converts on the GPU. Java never calls
//! into a borrowed runtime occurrence.
use jni::{
    JNIEnv, JavaVM,
    objects::{GlobalRef, JObject, JString, JValue},
};
use nuxie_runtime::video::playback::{Command, DecoderAction, Playback};
use std::{cell::RefCell, ffi::c_void, marker::PhantomData, ptr::NonNull, rc::Rc};

#[derive(Debug)]
pub enum AndroidError {
    Jni(jni::errors::Error),
    Media(String),
    Disposed,
    InvalidFrame,
    InvalidCommand,
    UnsupportedDecodeMode,
}
impl From<jni::errors::Error> for AndroidError {
    fn from(error: jni::errors::Error) -> Self {
        Self::Jni(error)
    }
}
pub use crate::scene::{Frame, FramePixels};

/// An Android frame in the decoder's hardware buffer. Pass its buffer, crop,
/// turns and color to `NativeVulkanFactory::import_hardware_buffer`, which
/// converts it on the GPU before returning. Dropping the frame returns the
/// buffer to the decoder.
pub struct HardwareBufferFrame {
    vm: JavaVM,
    frame: GlobalRef,
    buffer: NonNull<c_void>,
    /// The picture's left, top, right and bottom edges in buffer pixels.
    pub crop: [u32; 4],
    /// Clockwise rotation from buffer to display: 0, 90, 180 or 270.
    pub rotation_degrees: u32,
    /// The Y'CbCr matrix the decoder tags its output with: 1 BT.601, 2 BT.709,
    /// 3 BT.2020, numbered as in `NuxVideoHardwareBufferFrame`.
    pub color_matrix: u32,
    /// 1 limited range, 2 full range.
    pub color_range: u32,
}

impl HardwareBufferFrame {
    /// The `AHardwareBuffer*`, valid while this frame lives.
    pub fn buffer(&self) -> NonNull<c_void> {
        self.buffer
    }

    /// Clockwise quarter turns from buffer to display.
    pub fn quarter_turns(&self) -> u32 {
        self.rotation_degrees / 90
    }
}

impl Drop for HardwareBufferFrame {
    fn drop(&mut self) {
        if let Ok(mut env) = self.vm.attach_current_thread() {
            if env
                .call_method(self.frame.as_obj(), "close", "()V", &[])
                .is_err()
            {
                // Closing an image of a closed reader is harmless; do not
                // leave the exception pending for unrelated calls.
                let _ = env.exception_clear();
            }
        }
    }
}

/// `AHardwareBuffer_fromHardwareBuffer` is API 26 while the runtime targets
/// API 23, so it is looked up in the NDK's public `libandroid.so` when needed
/// rather than linked. A default-scope `dlsym` does not see it from inside an
/// app's linker namespace.
fn hardware_buffer_from_java(env: &JNIEnv<'_>, buffer: &JObject<'_>) -> Option<NonNull<c_void>> {
    type FromHardwareBuffer =
        unsafe extern "C" fn(*mut jni::sys::JNIEnv, jni::sys::jobject) -> *mut c_void;
    unsafe extern "C" {
        fn dlopen(file: *const std::ffi::c_char, mode: std::ffi::c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const std::ffi::c_char) -> *mut c_void;
    }
    const RTLD_NOW: std::ffi::c_int = 2;
    let library = unsafe { dlopen(c"libandroid.so".as_ptr(), RTLD_NOW) };
    if library.is_null() {
        return None;
    }
    let address = unsafe { dlsym(library, c"AHardwareBuffer_fromHardwareBuffer".as_ptr()) };
    if address.is_null() {
        return None;
    }
    let from_hardware_buffer: FromHardwareBuffer = unsafe { std::mem::transmute(address) };
    NonNull::new(unsafe { from_hardware_buffer(env.get_raw(), buffer.as_raw()) })
}
/// The platform's observed decoder identity. Classification is unavailable
/// before API 29 or when MediaPlayer's identity cannot be matched exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeAcceleration {
    Unknown,
    Hardware,
    Software,
}
#[derive(Debug, Clone)]
pub struct DecoderInfo {
    pub name: String,
    pub acceleration: DecodeAcceleration,
    /// Advertised capability only. Never treat this as measured available capacity.
    pub advertised_max_instances: Option<u32>,
}
pub struct AndroidPlayer {
    vm: JavaVM,
    object: GlobalRef,
    generation: u64,
    ready: bool,
    ended: bool,
    disposed: bool,
    clock_error: RefCell<Option<AndroidError>>,
    max_frame_bytes: usize,
    _thread: PhantomData<Rc<()>>,
}
impl AndroidPlayer {
    /// Call from an app JNI entry point so the app class loader can resolve
    /// ai.nuxie.runtime.VideoPlayer. Subsequent calls may attach the same VM.
    pub fn open(
        env: &mut JNIEnv<'_>,
        context: &JObject<'_>,
        source: &str,
        generation: u64,
        max_frame_bytes: usize,
        audio_policy: u32,
    ) -> Result<Self, AndroidError> {
        let budget = i32::try_from(max_frame_bytes).map_err(|_| AndroidError::InvalidFrame)?;
        let source = env.new_string(source)?;
        let object = env.new_object(
            "ai/nuxie/runtime/VideoPlayer",
            "(Landroid/content/Context;Ljava/lang/String;JII)V",
            &[
                JValue::Object(context),
                JValue::Object(&source),
                JValue::Long(generation as i64),
                JValue::Int(budget),
                JValue::Int(audio_policy as i32),
            ],
        )?;
        let object = env.new_global_ref(object)?;
        Ok(Self {
            vm: env.get_java_vm()?,
            object,
            generation,
            ready: false,
            ended: false,
            disposed: false,
            clock_error: RefCell::new(None),
            max_frame_bytes,
            _thread: PhantomData,
        })
    }
    pub fn decoder_info(&self) -> Result<Option<DecoderInfo>, AndroidError> {
        if self.disposed {
            return Ok(None);
        }
        let mut env = self.vm.attach_current_thread()?;
        let result = env.with_local_frame(8, |env| -> Result<_, AndroidError> {
            let info = env
                .call_method(
                    self.object.as_obj(),
                    "decoderInfo",
                    "()Lai/nuxie/runtime/VideoPlayer$DecoderInfo;",
                    &[],
                )?
                .l()?;
            if info.is_null() {
                return Ok(None);
            }
            let name = JString::from(env.get_field(&info, "name", "Ljava/lang/String;")?.l()?);
            let name = env.get_string(&name)?.into();
            let acceleration = match env.get_field(&info, "acceleration", "I")?.i()? {
                1 => DecodeAcceleration::Hardware,
                2 => DecodeAcceleration::Software,
                _ => DecodeAcceleration::Unknown,
            };
            let instances = env.get_field(&info, "advertisedMaxInstances", "I")?.i()?;
            Ok(Some(DecoderInfo {
                name,
                acceleration,
                advertised_max_instances: (instances > 0).then_some(instances as u32),
            }))
        });
        if result.is_err() && env.exception_check()? {
            env.exception_clear()?;
        }
        result
    }
    pub fn clock(&self) -> Option<crate::scene::MediaClock> {
        if self.disposed {
            return None;
        }
        let result = (|| -> Result<_, AndroidError> {
            let mut env = self.vm.attach_current_thread()?;
            let result = env.with_local_frame(8, |env| -> Result<_, AndroidError> {
                let clock = env
                    .call_method(
                        self.object.as_obj(),
                        "clock",
                        "()Lai/nuxie/runtime/VideoPlayer$Clock;",
                        &[],
                    )?
                    .l()?;
                if clock.is_null() {
                    return Ok(None);
                }
                Ok(Some(crate::scene::MediaClock {
                    generation: env.get_field(&clock, "generation", "J")?.j()? as u64,
                    seconds: env.get_field(&clock, "seconds", "D")?.d()?,
                    rate: env.get_field(&clock, "rate", "D")?.d()?,
                    playing: env.get_field(&clock, "playing", "Z")?.z()?,
                }))
            });
            // Retain the error for the normal failed/close path, but never leave
            // a pending JNI exception to poison unrelated calls on this thread.
            if result.is_err() && env.exception_check()? {
                env.exception_clear()?;
            }
            result
        })();
        match result {
            Ok(clock) => clock,
            Err(error) => {
                *self.clock_error.borrow_mut() = Some(error);
                None
            }
        }
    }
    pub fn apply(&mut self, action: DecoderAction) -> Result<(), AndroidError> {
        if self.disposed {
            return Err(AndroidError::Disposed);
        }
        let (code, value, token) = match action {
            DecoderAction::Play => (0, 0.0, 0),
            DecoderAction::Pause => (1, 0.0, 0),
            DecoderAction::Seek {
                seconds,
                generation,
            } if seconds.is_finite() && seconds >= 0.0 => {
                self.generation = generation;
                self.ended = false;
                (2, seconds, generation)
            }
            DecoderAction::Rate(rate) if rate.is_finite() && rate > 0.0 => (3, rate.into(), 0),
            DecoderAction::Volume(volume)
                if volume.is_finite() && (0.0..=1.0).contains(&volume) =>
            {
                (4, volume.into(), 0)
            }
            DecoderAction::Dispose => {
                self.close()?;
                return Ok(());
            }
            _ => return Err(AndroidError::InvalidCommand),
        };
        let mut env = self.vm.attach_current_thread()?;
        env.call_method(
            self.object.as_obj(),
            "action",
            "(IDJ)V",
            &[
                JValue::Int(code),
                JValue::Double(value),
                JValue::Long(token as i64),
            ],
        )?;
        Ok(())
    }
    fn observations(
        &mut self,
        playback: &mut Playback,
    ) -> Result<(Vec<DecoderAction>, Option<Frame>), AndroidError> {
        let mut env = self.vm.attach_current_thread()?;
        env.with_local_frame(24, |env| -> Result<_, AndroidError> {
            let failure = env
                .call_method(self.object.as_obj(), "failure", "()Ljava/lang/String;", &[])?
                .l()?;
            if !failure.is_null() {
                let message: String = env.get_string(&JString::from(failure))?.into();
                return Err(AndroidError::Media(message));
            }
            let interruption_ended = env
                .call_method(self.object.as_obj(), "takeInterruptionEnded", "()Z", &[])?
                .z()?;
            let interrupted = env
                .call_method(self.object.as_obj(), "interrupted", "()Z", &[])?
                .z()?;
            let mut actions = Vec::new();
            if env
                .call_method(self.object.as_obj(), "takePermanentLoss", "()Z", &[])?
                .z()?
            {
                playback
                    .enqueue(Command::Pause)
                    .map_err(|_| AndroidError::InvalidCommand)?;
                actions.extend(playback.drain_actions());
            }
            actions.extend(crate::audio::reconcile_interruption(
                playback,
                interrupted,
                interruption_ended,
            ));
            if env
                .call_method(self.object.as_obj(), "takePlayBlocked", "()Z", &[])?
                .z()?
            {
                playback.observed_play_blocked(self.generation);
            }
            if !self.ready
                && env
                    .call_method(self.object.as_obj(), "ready", "()Z", &[])?
                    .z()?
            {
                self.ready = true;
                let duration = env
                    .call_method(self.object.as_obj(), "duration", "()D", &[])?
                    .d()?;
                actions.extend(playback.opened(self.generation, duration));
            }
            if env
                .call_method(self.object.as_obj(), "playing", "()Z", &[])?
                .z()?
            {
                playback.observed_playing(self.generation);
            }
            if !self.ended
                && env
                    .call_method(self.object.as_obj(), "ended", "()Z", &[])?
                    .z()?
            {
                self.ended = true;
                actions.extend(playback.ended(self.generation));
            }
            let frame = env
                .call_method(
                    self.object.as_obj(),
                    "takeFrame",
                    "()Lai/nuxie/runtime/VideoPlayer$Frame;",
                    &[],
                )?
                .l()?;
            if frame.is_null() {
                return Ok((actions, None));
            }
            let generation = env.get_field(&frame, "generation", "J")?.j()? as u64;
            let pts = env.get_field(&frame, "seconds", "D")?.d()?;
            let edge = |env: &mut JNIEnv<'_>, name: &str| -> Result<u32, AndroidError> {
                let value = env.get_field(&frame, name, "I")?.i()?;
                u32::try_from(value).map_err(|_| AndroidError::InvalidFrame)
            };
            let crop = [
                edge(env, "cropLeft")?,
                edge(env, "cropTop")?,
                edge(env, "cropRight")?,
                edge(env, "cropBottom")?,
            ];
            let rotation_degrees = edge(env, "rotationDegrees")?;
            let color_matrix = edge(env, "colorMatrix")?;
            let color_range = edge(env, "colorRange")?;
            let [left, top, right, bottom] = crop;
            let (width, height) = match rotation_degrees {
                0 | 180 => (right.saturating_sub(left), bottom.saturating_sub(top)),
                90 | 270 => (bottom.saturating_sub(top), right.saturating_sub(left)),
                _ => return Err(AndroidError::InvalidFrame),
            };
            let bytes = (width as usize)
                .checked_mul(height as usize)
                .and_then(|n| n.checked_mul(4));
            if bytes.is_none_or(|n| n == 0 || n > self.max_frame_bytes)
                || !pts.is_finite()
                || pts < 0.0
            {
                let _ = env.call_method(&frame, "close", "()V", &[]);
                return Err(AndroidError::InvalidFrame);
            }
            let buffer_object = env
                .get_field(&frame, "buffer", "Landroid/hardware/HardwareBuffer;")?
                .l()?;
            let global = env.new_global_ref(&frame)?;
            let Some(buffer) = hardware_buffer_from_java(env, &buffer_object) else {
                let _ = env.call_method(&frame, "close", "()V", &[]);
                return Err(AndroidError::Media(
                    "AHardwareBuffer_fromHardwareBuffer (Android 8) is unavailable".into(),
                ));
            };
            Ok((
                actions,
                Some(Frame {
                    generation,
                    pts,
                    width,
                    height,
                    pixels: FramePixels::HardwareBuffer(HardwareBufferFrame {
                        vm: env.get_java_vm()?,
                        frame: global,
                        buffer,
                        crop,
                        rotation_degrees,
                        color_matrix,
                        color_range,
                    }),
                }),
            ))
        })
    }
    pub fn tick(&mut self, playback: &mut Playback) -> Result<Option<Frame>, AndroidError> {
        let clock_error = self.clock_error.borrow_mut().take();
        if let Some(error) = clock_error {
            playback.failed(playback.generation());
            self.close()?;
            return Err(error);
        }
        for action in playback.drain_actions() {
            self.apply(action)?;
        }
        if self.disposed {
            return Ok(None);
        }
        let (actions, frame) = match self.observations(playback) {
            Ok(value) => value,
            Err(error) => {
                playback.failed(self.generation);
                let _ = self.close();
                return Err(error);
            }
        };
        for action in actions {
            self.apply(action)?;
        }
        Ok(frame.filter(|f| f.generation == playback.generation()))
    }
    pub fn close(&mut self) -> Result<(), AndroidError> {
        if self.disposed {
            return Ok(());
        }
        let mut env = self.vm.attach_current_thread()?;
        if let Err(error) = env.call_method(self.object.as_obj(), "close", "()V", &[]) {
            // Return a Rust error without poisoning the next JNI retry with a
            // pending Java exception. Ownership remains until close succeeds.
            if env.exception_check().unwrap_or(false) {
                env.exception_clear()?;
            }
            return Err(AndroidError::Jni(error));
        }
        self.disposed = true;
        Ok(())
    }
}
impl Drop for AndroidPlayer {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

impl crate::scene::Decoder for AndroidPlayer {
    type Error = AndroidError;
    fn close(&mut self) -> Result<(), Self::Error> {
        AndroidPlayer::close(self)
    }
    fn clock(&self) -> Option<crate::scene::MediaClock> {
        AndroidPlayer::clock(self)
    }
    fn apply(&mut self, action: DecoderAction) -> Result<(), Self::Error> {
        AndroidPlayer::apply(self, action)
    }
    fn tick(&mut self, playback: &mut Playback) -> Result<Option<Frame>, Self::Error> {
        AndroidPlayer::tick(self, playback)
    }
}

/// Managed scene occurrence. Call tick from an app JNI entry point so reopening
/// uses the app class loader. The retained context should be ApplicationContext.
pub struct AndroidScenePlayer {
    scene: crate::scene::ScenePlayer<AndroidPlayer>,
    context: GlobalRef,
    source: String,
    max_frame_bytes: usize,
    audio_policy: u32,
    source_lease: Option<crate::source::EmbeddedFile>,
}
impl AndroidScenePlayer {
    pub fn decoder_info(&self) -> Result<Option<DecoderInfo>, AndroidError> {
        self.scene
            .decoder()
            .map(AndroidPlayer::decoder_info)
            .transpose()
            .map(Option::flatten)
    }

    pub fn clock(&self) -> Option<crate::scene::MediaClock> {
        self.scene.clock()
    }

    pub fn open(
        env: &mut JNIEnv<'_>,
        context: &JObject<'_>,
        video: nuxie_runtime::source::core::CoreHandle,
        source: &str,
        max_frame_bytes: usize,
        timeout: f64,
        optional: bool,
    ) -> Result<Self, AndroidError> {
        let audio_policy = video
            .with_downcast::<nuxie_runtime::video::Video, _>(|v| v.playback.settings().audio_policy)
            .ok_or(AndroidError::InvalidCommand)?;
        let context = env
            .call_method(
                context,
                "getApplicationContext",
                "()Landroid/content/Context;",
                &[],
            )?
            .l()?;
        Ok(Self {
            scene: crate::scene::ScenePlayer::new(video, timeout, optional)
                .map_err(|_| AndroidError::InvalidCommand)?,
            context: env.new_global_ref(context)?,
            source: source.into(),
            max_frame_bytes,
            audio_policy,
            source_lease: None,
        })
    }
    pub fn open_embedded(
        env: &mut JNIEnv<'_>,
        context: &JObject<'_>,
        video: nuxie_runtime::source::core::CoreHandle,
        directory: &std::path::Path,
        max_source_bytes: usize,
        max_frame_bytes: usize,
        timeout: f64,
        optional: bool,
    ) -> Result<Self, AndroidError> {
        use nuxie_runtime::video::{Video, VideoAsset};
        let bytes = video
            .with_downcast::<Video, _>(|v| v.asset())
            .flatten()
            .and_then(|a| {
                a.with_downcast::<VideoAsset, _>(|a| a.encoded_bytes())
                    .flatten()
            })
            .ok_or(AndroidError::InvalidCommand)?;
        let lease = crate::source::EmbeddedFile::materialize(directory, &bytes, max_source_bytes)
            .map_err(|e| AndroidError::Media(e.to_string()))?;
        let mut result = Self::open(
            env,
            context,
            video,
            lease.path().to_str().ok_or(AndroidError::InvalidCommand)?,
            max_frame_bytes,
            timeout,
            optional,
        )?;
        result.source_lease = Some(lease);
        Ok(result)
    }
    /// Call from lifecycle callbacks even when frame updates are stopped.
    pub fn set_suspended(
        &mut self,
        reason: nuxie_runtime::video::playback::SuspensionReason,
        suspended: bool,
    ) -> Result<(), AndroidError> {
        self.scene
            .set_suspended(reason, suspended)
            .map_err(|error| match error {
                crate::scene::SceneError::Decoder(error) => error,
                _ => AndroidError::InvalidCommand,
            })
    }
    pub fn replace_source(
        &mut self,
        source: &str,
        settings: nuxie_runtime::video::playback::PlaybackSettings,
    ) -> Result<u64, AndroidError> {
        if source.is_empty() {
            return Err(AndroidError::InvalidCommand);
        }
        let generation = self
            .scene
            .replace_source(settings)
            .map_err(|_| AndroidError::InvalidCommand)?;
        self.source = source.into();
        self.audio_policy = settings.audio_policy;
        self.source_lease = None;
        Ok(generation)
    }
    pub fn tick(
        &mut self,
        env: &mut JNIEnv<'_>,
        allocation: nuxie_runtime::video::resources::Allocation,
        now: f64,
        upload: impl FnMut(&Frame) -> Result<Rc<dyn nuxie_render_api::RenderImage>, AndroidError>,
    ) -> Result<crate::scene::SceneStatus, AndroidError> {
        if !matches!(
            allocation,
            nuxie_runtime::video::resources::Allocation::PlatformManaged
                | nuxie_runtime::video::resources::Allocation::Poster
        ) {
            return Err(AndroidError::UnsupportedDecodeMode);
        }
        self.scene
            .tick(
                allocation,
                now,
                |generation, _| {
                    AndroidPlayer::open(
                        env,
                        self.context.as_obj(),
                        &self.source,
                        generation,
                        self.max_frame_bytes,
                        self.audio_policy,
                    )
                },
                upload,
            )
            .map_err(|e| match e {
                crate::scene::SceneError::Decoder(e) => e,
                _ => AndroidError::InvalidCommand,
            })
    }
}

impl crate::pool::ManagedPlayer for AndroidScenePlayer {
    type Error = AndroidError;
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
        self.scene.reclaim().map_err(|error| match error {
            crate::scene::SceneError::Decoder(error) => error,
            _ => AndroidError::InvalidCommand,
        })
    }
}
