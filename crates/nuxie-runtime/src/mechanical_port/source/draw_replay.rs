//! Retained draws for artboards whose content did not change since their
//! last draw.
//!
//! An artboard's draw is a pure function of its content: when no component
//! notified the artboard of a change since the last draw (its content
//! revision stands) and no property of its graph was written (its arena's
//! write epoch stands), the renderer receives the same calls with the same
//! arguments and the same render paths and paints. The first draw after the
//! content settles draws in full, the next records its calls through a
//! recording renderer, and later draws replay them into the renderer without
//! walking the draw order, as long as both stand.
//!
//! Replay needs the very resources a call received. A call only hands over
//! references, so render paths and paints shared through `Rc` register their
//! addresses here when created; a recorded call resolves its references to
//! those handles. A call whose resource is not registered, or a call the
//! recording does not keep (images, meshes, layer masks, stroke clips, or a
//! query of the renderer's state), makes the recording unusable: that content
//! keeps drawing in full.
//!
//! A draw also sets state on shared resources between its calls: a render
//! path's fill rule before each paint draws it, a render paint's blend before
//! a text or layout paint draws. Those setters go through `set_fill_rule` and
//! `set_blend`, which every recording in progress keeps in order with its
//! calls.

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
    sync::atomic::{AtomicUsize, Ordering},
};

use nuxie_render_api::{
    BlendMode, ColorInt, FillRule, ImageMeshInstancesHandle, ImageSampler, LayerMaskMode, Mat2D,
    RenderBuffer, RenderImage, RenderPaint, RenderPath, Renderer, StrokeParams,
};

/// A render path shared between its owner and the recordings that use it.
pub(crate) type RenderPathHandle = Rc<RefCell<Box<dyn RenderPath>>>;
/// A render paint shared between its owner and the recordings that use it.
pub(crate) type RenderPaintHandle = Rc<RefCell<Box<dyn RenderPaint>>>;

/// Resources by address. Addresses are process-local, so the multiplicative
/// mix serves as well as SipHash at a fraction of a recorded call's cost.
type Registry<T> = HashMap<
    usize,
    Weak<RefCell<Box<T>>>,
    std::hash::BuildHasherDefault<crate::mechanical_port::source::core::CoreHandleHasher>,
>;

thread_local! {
    static PATHS: RefCell<Registry<dyn RenderPath>> = RefCell::new(Registry::default());
    static PAINTS: RefCell<Registry<dyn RenderPaint>> = RefCell::new(Registry::default());
    // The calls of the recordings in progress, innermost last; None once a
    // recording can't be replayed.
    static RECORDINGS: RefCell<Vec<Option<Vec<DrawCmd>>>> = const { RefCell::new(Vec::new()) };
}

/// Recordings in progress on any thread. Draws mostly run with none, so the
/// setters read this before reaching their thread's recordings.
static ACTIVE_RECORDINGS: AtomicUsize = AtomicUsize::new(0);

/// Keep `cmd` in every recording in progress: a recording renderer that
/// draws into another forwards its calls, but a setter reaches no renderer.
#[inline]
fn note(cmd: impl FnOnce() -> Option<DrawCmd>) {
    if ACTIVE_RECORDINGS.load(Ordering::Relaxed) != 0 {
        note_in_recordings(cmd);
    }
}

#[cold]
fn note_in_recordings(cmd: impl FnOnce() -> Option<DrawCmd>) {
    RECORDINGS.with(|recordings| {
        let mut recordings = recordings.borrow_mut();
        if recordings.iter().all(Option::is_none) {
            return;
        }
        let cmd = cmd();
        for slot in recordings.iter_mut() {
            if let Some(cmds) = slot.as_mut() {
                match &cmd {
                    Some(cmd) => cmds.push(cmd.clone()),
                    None => *slot = None,
                }
            }
        }
    });
}

/// Keep `cmd` in recording `slot` alone.
#[inline]
fn note_in(slot: usize, cmd: impl FnOnce() -> Option<DrawCmd>) {
    RECORDINGS.with(|recordings| {
        if let Some(entry) = recordings.borrow_mut().get_mut(slot)
            && let Some(cmds) = entry.as_mut()
        {
            match cmd() {
                Some(cmd) => cmds.push(cmd),
                None => *entry = None,
            }
        }
    });
}

/// The draw does something a recording can't keep (a paint image applied to
/// a paint, drawing through another renderer): the recording in progress
/// can't be replayed.
#[inline]
pub(crate) fn unreplayable() {
    note(|| None);
}

/// RenderPath::fillRule during a draw, kept by the recording in progress.
#[inline]
pub(crate) fn set_fill_rule(path: &mut dyn RenderPath, rule: FillRule) {
    path.fill_rule(rule);
    note(|| Some(DrawCmd::FillRule(resolve_path(path)?, rule)));
}

/// RenderPaint::blendMode and additiveness during a draw, kept by the
/// recording in progress.
#[inline]
pub(crate) fn set_blend(paint: &mut dyn RenderPaint, mode: BlendMode, additiveness: f32) {
    paint.blend_mode(mode);
    paint.additiveness(additiveness);
    note(|| Some(DrawCmd::Blend(resolve_paint(paint)?, mode, additiveness)));
}

/// The address a call's reference to `object` carries. Zero-sized objects
/// share one address, so they have none here.
fn address<T: ?Sized>(object: &T) -> Option<usize> {
    (std::mem::size_of_val(object) != 0).then(|| (object as *const T).cast::<()>() as usize)
}

/// Register `path` so recorded calls can resolve references to it.
pub(crate) fn register_path(path: &RenderPathHandle) {
    let Some(key) = address(&**path.borrow()) else {
        return;
    };
    PATHS.with(|paths| {
        let mut paths = paths.borrow_mut();
        if paths.len() > 1024 && paths.len().is_power_of_two() {
            paths.retain(|_, path| path.strong_count() > 0);
        }
        paths.insert(key, Rc::downgrade(path));
    });
}

/// Register `paint` so recorded calls can resolve references to it.
pub(crate) fn register_paint(paint: &RenderPaintHandle) {
    let Some(key) = address(&**paint.borrow()) else {
        return;
    };
    PAINTS.with(|paints| {
        let mut paints = paints.borrow_mut();
        if paints.len() > 1024 && paints.len().is_power_of_two() {
            paints.retain(|_, paint| paint.strong_count() > 0);
        }
        paints.insert(key, Rc::downgrade(paint));
    });
}

/// The live handle a call's reference `path` points into, when registered.
fn resolve_path(path: &dyn RenderPath) -> Option<RenderPathHandle> {
    let key = address(path)?;
    PATHS.with(|paths| paths.borrow().get(&key).and_then(Weak::upgrade))
}

fn resolve_paint(paint: &dyn RenderPaint) -> Option<RenderPaintHandle> {
    let key = address(paint)?;
    PAINTS.with(|paints| paints.borrow().get(&key).and_then(Weak::upgrade))
}

/// One renderer call a recording keeps, with its arguments.
#[derive(Clone)]
pub(crate) enum DrawCmd {
    Save,
    Restore,
    Transform(Mat2D),
    Translate(f32, f32),
    Scale(f32, f32),
    Rotate(f32),
    DrawPath(RenderPathHandle, RenderPaintHandle),
    ClipPath(RenderPathHandle),
    ModulateOpacity(f32),
    ModulateColor(ColorInt, bool),
    FillRule(RenderPathHandle, FillRule),
    Blend(RenderPaintHandle, BlendMode, f32),
}

impl PartialEq for DrawCmd {
    fn eq(&self, other: &Self) -> bool {
        use DrawCmd::*;
        match (self, other) {
            (Save, Save) | (Restore, Restore) => true,
            (Transform(a), Transform(b)) => a.0 == b.0,
            (Translate(ax, ay), Translate(bx, by)) | (Scale(ax, ay), Scale(bx, by)) => {
                ax.to_bits() == bx.to_bits() && ay.to_bits() == by.to_bits()
            }
            (Rotate(a), Rotate(b)) | (ModulateOpacity(a), ModulateOpacity(b)) => {
                a.to_bits() == b.to_bits()
            }
            (DrawPath(ap, aq), DrawPath(bp, bq)) => Rc::ptr_eq(ap, bp) && Rc::ptr_eq(aq, bq),
            (ClipPath(a), ClipPath(b)) => Rc::ptr_eq(a, b),
            (ModulateColor(ac, ar), ModulateColor(bc, br)) => ac == bc && ar == br,
            (FillRule(ap, ar), FillRule(bp, br)) => Rc::ptr_eq(ap, bp) && ar == br,
            (Blend(ap, am, aa), Blend(bp, bm, ba)) => {
                Rc::ptr_eq(ap, bp) && am == bm && aa.to_bits() == ba.to_bits()
            }
            _ => false,
        }
    }
}

/// Issue `cmds` to `renderer`, as the recorded draw issued them.
pub(crate) fn replay(cmds: &[DrawCmd], renderer: &mut dyn Renderer) {
    for cmd in cmds {
        match cmd {
            DrawCmd::Save => renderer.save(),
            DrawCmd::Restore => renderer.restore(),
            DrawCmd::Transform(transform) => renderer.transform(*transform),
            DrawCmd::Translate(x, y) => renderer.translate(*x, *y),
            DrawCmd::Scale(x, y) => renderer.scale(*x, *y),
            DrawCmd::Rotate(radians) => renderer.rotate(*radians),
            DrawCmd::DrawPath(path, paint) => {
                renderer.draw_path(&**path.borrow(), &**paint.borrow());
            }
            DrawCmd::ClipPath(path) => renderer.clip_path(&**path.borrow()),
            DrawCmd::ModulateOpacity(opacity) => renderer.modulate_opacity(*opacity),
            DrawCmd::ModulateColor(color, replace) => renderer.modulate_color(*color, *replace),
            // Through the setters, so a recording around the replay keeps them.
            DrawCmd::FillRule(path, rule) => set_fill_rule(&mut **path.borrow_mut(), *rule),
            DrawCmd::Blend(paint, mode, additiveness) => {
                set_blend(&mut **paint.borrow_mut(), *mode, *additiveness);
            }
        }
    }
}

/// A renderer that forwards every call to `inner` and records the calls a
/// replay can issue again. Any other call ends the recording (`cmds` None).
pub(crate) struct RecordingRenderer<'a> {
    inner: &'a mut dyn Renderer,
    // This recording's slot in RECORDINGS, which draw code also writes.
    slot: usize,
    finished: bool,
    // The draw read the renderer's state, which a replay can't reproduce.
    queried: std::cell::Cell<bool>,
}

impl<'a> RecordingRenderer<'a> {
    pub(crate) fn new(inner: &'a mut dyn Renderer) -> Self {
        ACTIVE_RECORDINGS.fetch_add(1, Ordering::Relaxed);
        let slot = RECORDINGS.with(|recordings| {
            let mut recordings = recordings.borrow_mut();
            recordings.push(Some(Vec::new()));
            recordings.len() - 1
        });
        Self {
            inner,
            slot,
            finished: false,
            queried: std::cell::Cell::new(false),
        }
    }

    /// The recorded calls, unless a call could not be recorded.
    pub(crate) fn finish(mut self) -> Option<Vec<DrawCmd>> {
        self.finished = true;
        ACTIVE_RECORDINGS.fetch_sub(1, Ordering::Relaxed);
        let cmds = RECORDINGS.with(|recordings| {
            let mut recordings = recordings.borrow_mut();
            debug_assert_eq!(recordings.len(), self.slot + 1, "recordings end innermost first");
            recordings.pop().flatten()
        });
        cmds.filter(|_| !self.queried.get())
    }

    #[inline]
    fn record(&mut self, cmd: impl FnOnce() -> Option<DrawCmd>) {
        note_in(self.slot, cmd);
    }

    #[inline]
    fn unrecordable(&mut self) {
        note_in(self.slot, || None);
    }
}

impl Drop for RecordingRenderer<'_> {
    fn drop(&mut self) {
        if !self.finished {
            ACTIVE_RECORDINGS.fetch_sub(1, Ordering::Relaxed);
            RECORDINGS.with(|recordings| {
                recordings.borrow_mut().truncate(self.slot);
            });
        }
    }
}

impl Renderer for RecordingRenderer<'_> {
    fn apply_layer_mask(
        &mut self,
        mask: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        mode: LayerMaskMode,
    ) {
        self.unrecordable();
        self.inner.apply_layer_mask(mask, sampler, mode);
    }

    fn draw_image_mesh_instanced(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        instances: Option<&ImageMeshInstancesHandle>,
    ) {
        self.unrecordable();
        self.inner.draw_image_mesh_instanced(
            image,
            sampler,
            vertices,
            uv_coords,
            indices,
            vertex_count,
            index_count,
            instances,
        );
    }

    fn save(&mut self) {
        self.record(|| Some(DrawCmd::Save));
        self.inner.save();
    }

    fn restore(&mut self) {
        self.record(|| Some(DrawCmd::Restore));
        self.inner.restore();
    }

    fn transform(&mut self, transform: Mat2D) {
        self.record(|| Some(DrawCmd::Transform(transform)));
        self.inner.transform(transform);
    }

    fn translate(&mut self, tx: f32, ty: f32) {
        self.record(|| Some(DrawCmd::Translate(tx, ty)));
        self.inner.translate(tx, ty);
    }

    fn scale(&mut self, sx: f32, sy: f32) {
        self.record(|| Some(DrawCmd::Scale(sx, sy)));
        self.inner.scale(sx, sy);
    }

    fn rotate(&mut self, radians: f32) {
        self.record(|| Some(DrawCmd::Rotate(radians)));
        self.inner.rotate(radians);
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        self.record(|| Some(DrawCmd::DrawPath(resolve_path(path)?, resolve_paint(paint)?)));
        self.inner.draw_path(path, paint);
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        self.record(|| Some(DrawCmd::ClipPath(resolve_path(path)?)));
        self.inner.clip_path(path);
    }

    fn clip_stroke(&mut self, path: &dyn RenderPath, params: &StrokeParams) {
        self.unrecordable();
        self.inner.clip_stroke(path, params);
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        self.unrecordable();
        self.inner.draw_image(image, sampler, blend_mode, opacity);
    }

    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        self.unrecordable();
        self.inner.draw_image_mesh(
            image,
            sampler,
            vertices,
            uv_coords,
            indices,
            vertex_count,
            index_count,
            blend_mode,
            opacity,
        );
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.record(|| Some(DrawCmd::ModulateOpacity(opacity)));
        self.inner.modulate_opacity(opacity);
    }

    fn modulate_color(&mut self, color: ColorInt, replace: bool) {
        self.record(|| Some(DrawCmd::ModulateColor(color, replace)));
        self.inner.modulate_color(color, replace);
    }

    fn draw_image_with_additiveness(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
        additiveness: f32,
    ) {
        self.unrecordable();
        self.inner
            .draw_image_with_additiveness(image, sampler, blend_mode, opacity, additiveness);
    }

    fn draw_image_mesh_with_additiveness(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
        additiveness: f32,
    ) {
        self.unrecordable();
        self.inner.draw_image_mesh_with_additiveness(
            image,
            sampler,
            vertices,
            uv_coords,
            indices,
            vertex_count,
            index_count,
            blend_mode,
            opacity,
            additiveness,
        );
    }

    fn current_transform(&self) -> Option<Mat2D> {
        self.queried.set(true);
        self.inner.current_transform()
    }

    fn current_modulated_opacity(&self) -> Option<f32> {
        self.queried.set(true);
        self.inner.current_modulated_opacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_render_api::{Factory, ImageSampler, NullFactory, NullRenderer};

    fn path() -> RenderPathHandle {
        let path: RenderPathHandle = Rc::new(RefCell::new(NullFactory::new().make_empty_render_path()));
        register_path(&path);
        path
    }

    fn paint() -> RenderPaintHandle {
        let paint: RenderPaintHandle = Rc::new(RefCell::new(NullFactory::new().make_render_paint()));
        register_paint(&paint);
        paint
    }

    #[test]
    fn a_replay_makes_the_recorded_calls() {
        let (path, paint) = (path(), paint());
        let mut null = NullRenderer::new();
        let mut recorder = RecordingRenderer::new(&mut null);
        recorder.save();
        recorder.transform(Mat2D([1.0, 0.0, 0.0, 1.0, 5.0, -0.0]));
        set_fill_rule(&mut **path.borrow_mut(), FillRule::EvenOdd);
        recorder.draw_path(&**path.borrow(), &**paint.borrow());
        recorder.restore();
        let cmds = recorder.finish().expect("every call is recordable");
        assert_eq!(cmds.len(), 5);

        let mut null = NullRenderer::new();
        let mut recorder = RecordingRenderer::new(&mut null);
        replay(&cmds, &mut recorder);
        assert!(recorder.finish().as_deref() == Some(&cmds[..]));
    }

    #[test]
    fn calls_a_replay_cannot_make_end_a_recording() {
        let path = path();
        let unregistered: Box<dyn RenderPath> = NullFactory::new().make_empty_render_path();
        let mut null = NullRenderer::new();

        let mut recorder = RecordingRenderer::new(&mut null);
        recorder.draw_path(&*unregistered, &**paint().borrow());
        assert!(recorder.finish().is_none(), "unregistered path");

        let mut recorder = RecordingRenderer::new(&mut null);
        recorder.clip_path(&**path.borrow());
        recorder.draw_image(None, ImageSampler::LINEAR_CLAMP, BlendMode::SrcOver, 1.0);
        assert!(recorder.finish().is_none(), "image");

        let mut recorder = RecordingRenderer::new(&mut null);
        recorder.save();
        let _ = recorder.current_transform();
        assert!(recorder.finish().is_none(), "renderer state read");

        let mut recorder = RecordingRenderer::new(&mut null);
        unreplayable();
        assert!(recorder.finish().is_none(), "draw code gave up");
    }

    #[test]
    fn nested_recordings_each_keep_their_own_calls() {
        let path = path();
        let mut null = NullRenderer::new();
        let mut outer = RecordingRenderer::new(&mut null);
        outer.save();
        let mut inner = RecordingRenderer::new(&mut outer);
        inner.translate(1.0, 2.0);
        set_fill_rule(&mut **path.borrow_mut(), FillRule::Clockwise);
        let inner_cmds = inner.finish().expect("recordable");
        outer.restore();
        let outer_cmds = outer.finish().expect("recordable");
        assert!(
            inner_cmds
                == [
                    DrawCmd::Translate(1.0, 2.0),
                    DrawCmd::FillRule(Rc::clone(&path), FillRule::Clockwise)
                ]
        );
        assert!(
            outer_cmds
                == [
                    DrawCmd::Save,
                    DrawCmd::Translate(1.0, 2.0),
                    DrawCmd::FillRule(Rc::clone(&path), FillRule::Clockwise),
                    DrawCmd::Restore
                ]
        );
    }

    #[test]
    fn an_abandoned_recording_leaves_nothing_in_progress() {
        let mut null = NullRenderer::new();
        drop(RecordingRenderer::new(&mut null));
        assert!(RECORDINGS.with(|recordings| recordings.borrow().is_empty()));
        // A setter outside any recording only sets.
        let path = path();
        set_fill_rule(&mut **path.borrow_mut(), FillRule::EvenOdd);
    }
}
