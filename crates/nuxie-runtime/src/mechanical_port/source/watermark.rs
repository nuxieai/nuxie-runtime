//! Mechanical translation of rive/watermark.hpp and src/watermark.cpp.

use super::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard::RuntimeArtboardInstanceHandle,
    file::File,
    layout::{Alignment, Fit},
    math::aabb::Aabb,
    renderer::{RenderPaint, RenderPaintStyle, RenderPath, Renderer, compute_alignment},
};

const WATERMARK_MAX_SECONDS: f32 = 10.0;
const WATERMARK_CLOCK_TOLERANCE_SECONDS: f32 = 0.004;
const WATERMARK_MAX_FRAME_SECONDS: f32 = 0.25;
const WATERMARK_BACKDROP_COLOR: u32 = 0xff000000;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn now_micros() -> i64 {
    // Only differences are consumed; an arbitrary steady epoch is equivalent
    // to C++ steady_clock::time_since_epoch().
    static EPOCH: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    EPOCH
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_micros() as i64
}

#[cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    feature = "js-host-seed"
))]
fn now_micros() -> i64 {
    use wasm_bindgen::{JsCast, JsValue};

    // Same browser/worker host boundary used by the existing random seed,
    // but retain integer microseconds rather than reducing to a seed.
    let global = js_sys::global();
    let performance = js_sys::Reflect::get(&global, &JsValue::from_str("performance"))
        .unwrap_or_else(|error| wasm_bindgen::throw_val(error));
    let now = js_sys::Reflect::get(&performance, &JsValue::from_str("now"))
        .unwrap_or_else(|error| wasm_bindgen::throw_val(error))
        .dyn_into::<js_sys::Function>()
        .unwrap_or_else(|error| wasm_bindgen::throw_val(error));
    let milliseconds = now
        .call0(&performance)
        .unwrap_or_else(|error| wasm_bindgen::throw_val(error))
        .as_f64()
        .unwrap_or_else(|| wasm_bindgen::throw_str("performance.now() did not return a number"));
    (milliseconds * 1_000.0) as i64
}

#[cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    not(feature = "js-host-seed")
))]
thread_local! {
    static HOST_CLOCK: std::cell::Cell<Option<fn() -> i64>> = const { std::cell::Cell::new(None) };
}

/// Install the monotonic microsecond clock for an import-free Wasm embedder.
///
/// Required before constructing a watermarked artboard, including under
/// deterministic mode: upstream samples the clock during construction. The
/// callback must use a consistent monotonic epoch for the entire lifetime of
/// every watermark. This explicit boundary keeps cold publisher modules free
/// of JS imports without silently disabling the playback clock budget.
#[cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    not(feature = "js-host-seed")
))]
pub fn set_watermark_clock(clock: fn() -> i64) {
    HOST_CLOCK.with(|host_clock| host_clock.set(Some(clock)));
}

#[cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    not(feature = "js-host-seed")
))]
fn now_micros() -> i64 {
    let clock = HOST_CLOCK.with(std::cell::Cell::get).expect(
        "import-free Wasm watermark playback requires set_watermark_clock before construction",
    );
    clock()
}

/// A pre-roll that owns each frame until the frame after its final advance.
/// An instance not driven by a state machine never starts its pre-roll.
pub struct Watermark {
    backdrop_path: Option<Box<RenderPath>>,
    backdrop_paint: Option<Box<RenderPaint>>,
    // Rust drops fields in declaration order: the state machine must be gone
    // before the retained artboard, the reverse of the C++ declarations.
    state_machine: RuntimeStateMachineInstanceHandle,
    artboard: RuntimeArtboardInstanceHandle,
    last_time_micros: i64,
    wall_seconds: f32,
    backdrop_bounds: Aabb,
    elapsed_seconds: f32,
    started: bool,
    finished: bool,
}

impl Watermark {
    pub fn new(
        artboard: RuntimeArtboardInstanceHandle,
        state_machine: RuntimeStateMachineInstanceHandle,
    ) -> Self {
        let last_time_micros = now_micros();
        // The caller applies the host origin; fit this content from its own 0,0.
        artboard.with_artboard_mut(|artboard| artboard.base.set_frame_origin(false));
        Self {
            state_machine,
            artboard,
            last_time_micros,
            wall_seconds: 0.0,
            backdrop_paint: None,
            backdrop_path: None,
            backdrop_bounds: Aabb::default(),
            elapsed_seconds: 0.0,
            started: false,
            finished: false,
        }
    }

    pub fn is_playing(&self) -> bool {
        self.started
    }

    pub fn elapsed_seconds(&self) -> f32 {
        self.elapsed_seconds
    }

    fn clamp_elapsed(&mut self, elapsed_seconds: f32) -> f32 {
        if elapsed_seconds < 0.0 {
            return 0.0;
        }
        if File::deterministic_mode() {
            return elapsed_seconds;
        }
        let now = now_micros();
        let mut wall_seconds = (now - self.last_time_micros) as f32 / 1_000_000.0;
        self.last_time_micros = now;
        if wall_seconds < 0.0 {
            wall_seconds = 0.0;
        }
        // Credit, as well as spend, is capped: idle time cannot be banked to
        // fund a burst of advances that skips the pre-roll.
        self.wall_seconds += if wall_seconds > WATERMARK_MAX_FRAME_SECONDS {
            WATERMARK_MAX_FRAME_SECONDS
        } else {
            wall_seconds
        };
        let mut budget =
            self.wall_seconds + WATERMARK_CLOCK_TOLERANCE_SECONDS - self.elapsed_seconds;
        if budget < 0.0 {
            budget = 0.0;
        }
        let seconds = if elapsed_seconds > budget {
            budget
        } else {
            elapsed_seconds
        };
        if seconds > WATERMARK_MAX_FRAME_SECONDS {
            WATERMARK_MAX_FRAME_SECONDS
        } else {
            seconds
        }
    }

    pub fn advance(&mut self, elapsed_seconds: f32) -> bool {
        if self.finished {
            return false;
        }
        self.started = true;
        let seconds = self.clamp_elapsed(elapsed_seconds);
        // The host entry point counts the host file's pending async work;
        // that must not keep its watermark running.
        let more = self.state_machine.advance_and_apply_view_models(seconds, true);
        self.elapsed_seconds += seconds;
        self.finished = !more || self.elapsed_seconds >= WATERMARK_MAX_SECONDS;
        // The finishing frame is still drawn. Only the next call hands over.
        true
    }

    fn draw_backdrop(&mut self, renderer: &mut Renderer, host_bounds: &Aabb) {
        let Some(factory) = self
            .artboard
            .with_artboard(|artboard| artboard.base.factory())
        else {
            return;
        };
        if self.backdrop_paint.is_none() {
            let mut paint = factory.with_factory_mut(|factory| factory.make_render_paint());
            paint.style(RenderPaintStyle::Fill);
            paint.color(WATERMARK_BACKDROP_COLOR);
            self.backdrop_paint = Some(paint);
        }
        if self.backdrop_path.is_none() || self.backdrop_bounds != *host_bounds {
            self.backdrop_path = Some(factory.with_factory_mut(|factory| {
                factory.make_render_path_from_aabb(nuxie_render_api::Aabb::new(
                    host_bounds.min_x,
                    host_bounds.min_y,
                    host_bounds.max_x,
                    host_bounds.max_y,
                ))
            }));
            self.backdrop_bounds = *host_bounds;
        }
        if let (Some(paint), Some(path)) = (&self.backdrop_paint, &self.backdrop_path) {
            renderer.draw_path(path.as_ref(), paint.as_ref());
        }
    }

    pub fn draw(&mut self, renderer: &mut Renderer, host_bounds: &Aabb) {
        self.draw_backdrop(renderer, host_bounds);
        renderer.save();
        let bounds = self
            .artboard
            .with_artboard(|artboard| artboard.base.bounds());
        let alignment =
            compute_alignment(Fit::Contain, Alignment::CENTER, host_bounds, &bounds, 1.0);
        renderer.transform(nuxie_render_api::Mat2D(*alignment.values()));
        // The caller already incremented the global frame id.
        self.artboard.draw_internal(renderer);
        renderer.restore();
    }
}
