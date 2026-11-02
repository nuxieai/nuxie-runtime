//! A CSS element that fades draws as one picture (docs/CSS_LAYOUT.md, 4.4):
//! a hosted CSS artboard, a CSS box that clips both axes, and a Screen, whose
//! flights draw outside its picture. Rive has no save layer, so opacity
//! multiplies down the tree and each paint takes it alone. CSS says an element
//! and its descendants composite as one, so the element draws whole into a
//! canvas and the canvas goes down at the fade, once. Rive-made content keeps
//! upstream's per-paint fade.
use crate::live_move::records::LiveSlot;
use crate::source::{
    artboard::Artboard,
    artboard_component_list::ArtboardComponentList,
    bitmap_cache::BitmapCache,
    core::{CoreHandle, CoreObject},
    drawable::{BoundsFidelity, Drawable, DrawableProxy, RuntimeDrawableOccurrence},
    generated::shapes::image_base::ImageBase,
    layout_component::LayoutComponent,
    math::{aabb::Aabb, mat2d::Mat2D},
    offscreen_raster::{
        CachedRaster, CanvasContentScope, K_SHRINK_DRAWS, RasterPlan, hold_raster_allocation,
    },
    renderer::Renderer,
};
use crate::video::Video;
use nuxie_render_api::{BlendMode, DeferredCanvasHostHandle, ImageSampler};

/// Canvases round up to multiples of this.
const CANVAS_STEP: u32 = 64;
/// Device pixels kept around a box for the smoothing of a clip's own edge.
const GUARD_PIXELS: f32 = 2.0;

/// An element's held picture: its canvas, and how many draws in a row have
/// needed less than it.
#[derive(Default)]
pub(crate) struct GroupPicture {
    raster: CachedRaster,
    smaller_draws: u8,
}

impl GroupPicture {
    pub(crate) fn release(&mut self) {
        self.raster.release();
        self.smaller_draws = 0;
    }

    fn holds_canvas(&self) -> bool {
        self.raster.canvas.is_some()
    }

    /// Draws `content` into this picture and composites it at `held`. `area`
    /// is the box `content` can reach, mapped by `place` into the renderer's
    /// current space, with `guard` device pixels around it. None leaves
    /// `renderer` untouched and the fade still to apply.
    fn draw(
        &mut self,
        host: &DeferredCanvasHostHandle,
        renderer: &mut Renderer,
        (place, area, guard): (Mat2D, Aabb, f32),
        held: f32,
        content: impl FnOnce(&mut Renderer),
    ) -> Option<()> {
        let [xx, xy, yx, yy, tx, ty] = renderer.current_transform()?.0;
        let transform = Mat2D::new(xx, xy, yx, yy, tx, ty);
        let mut inverse = Mat2D::default();
        if !transform.invert(&mut inverse) {
            return None;
        }
        if host.borrow().composite_drops_clips() {
            return None;
        }
        let (left, top, need) = device_pixels((transform * place).map_bounding_box(area), guard)?;
        let size = self.fit(need, host)?;
        let canvas = self.raster.canvas.clone()?;
        let image = host.borrow_mut().content_canvas_image(&canvas)?;
        let mut plan = RasterPlan {
            bounds: Aabb::new(left, top, left + need.0 as f32, top + need.1 as f32),
            raster_scale: 1.0,
            width_px: need.0,
            height_px: need.1,
            ..RasterPlan::default()
        };
        hold_raster_allocation(size.0, size.1, &mut plan);
        {
            let mut scope = CanvasContentScope::new(Some(host), Some(&canvas), &plan, 0);
            let canvas_renderer = scope.renderer()?;
            canvas_renderer.transform(nuxie_render_api::Mat2D(*transform.values()));
            content(canvas_renderer);
        }
        // One texel to one device pixel, so nothing resamples. The renderer's
        // own scope applies on top of `held`.
        renderer.save();
        renderer.transform(nuxie_render_api::Mat2D(*inverse.values()));
        renderer.translate(left, top);
        renderer.draw_image(
            Some(image.as_ref()),
            ImageSampler::LINEAR_CLAMP,
            BlendMode::SrcOver,
            held,
        );
        renderer.restore();
        Some(())
    }

    /// Keeps or makes the canvas `need` takes, and answers its size. None
    /// when `need` is past the renderer's real limit or no canvas comes.
    fn fit(&mut self, need: (u32, u32), host: &DeferredCanvasHostHandle) -> Option<(u32, u32)> {
        let limit = host.borrow().max_canvas_dim();
        if need.0 > limit || need.1 > limit {
            return None;
        }
        let have = (self.raster.width_px, self.raster.height_px);
        let size = next_canvas_size(need, have, &mut self.smaller_draws, limit);
        if self.raster.canvas.is_none() || size != have {
            self.raster.canvas = host.borrow_mut().make_content_canvas(size.0, size.1);
            (self.raster.width_px, self.raster.height_px) = size;
        }
        Some(size)
    }
}

/// The canvas after a draw that needs `need`, holding `have`. The first
/// allocation is exact. A later growth takes half again on each axis, so a
/// growing element allocates a few times and not every frame. A canvas larger
/// than its need shrinks after `K_SHRINK_DRAWS` smaller draws. Upstream's
/// `decide_raster_resize` keeps its own cap, so a Screen-sized picture sizes
/// here against the renderer's limit (`limit`) instead.
fn next_canvas_size(
    need: (u32, u32),
    have: (u32, u32),
    smaller_draws: &mut u8,
    limit: u32,
) -> (u32, u32) {
    let exact = (rounded(need.0, limit), rounded(need.1, limit));
    if have.0 == 0 || have.1 == 0 {
        *smaller_draws = 0;
        return exact;
    }
    if need.0 > have.0 || need.1 > have.1 {
        *smaller_draws = 0;
        let grown =
            |need: u32, have: u32| rounded(need.saturating_mul(3).div_ceil(2), limit).max(have);
        return (grown(need.0, have.0), grown(need.1, have.1));
    }
    if exact.0 < have.0 || exact.1 < have.1 {
        *smaller_draws += 1;
        if *smaller_draws < K_SHRINK_DRAWS {
            return have;
        }
        *smaller_draws = 0;
        return exact;
    }
    *smaller_draws = 0;
    have
}

/// Up to the next multiple of `CANVAS_STEP`, stopping at the limit.
fn rounded(pixels: u32, limit: u32) -> u32 {
    pixels
        .div_ceil(CANVAS_STEP)
        .saturating_mul(CANVAS_STEP)
        .min(limit)
}

/// A device box rounded out to whole pixels with `guard` pixels around it:
/// its corner, and its size. A fractional corner would resample the picture.
fn device_pixels(area: Aabb, guard: f32) -> Option<(f32, f32, (u32, u32))> {
    let left = area.left().floor() - guard;
    let top = area.top().floor() - guard;
    let width = area.right().ceil() + guard - left;
    let height = area.bottom().ceil() + guard - top;
    (left.is_finite() && top.is_finite() && width.is_finite() && height.is_finite()).then_some((
        left,
        top,
        (width as u32, height as u32),
    ))
}

/// A host's fade, split: what a hosted artboard's children see, and what its
/// draw applies. Strictly between 0 and 1 a CSS artboard holds the fade for
/// its picture. Zero reaches the children, so every "is it hidden" check keeps
/// its answer, and a Rive-made artboard hands the whole fade down.
pub(crate) fn split_host_fade(css_mode: bool, value: f32) -> (f32, f32) {
    if css_mode && value > 0.0 && value < 1.0 {
        (1.0, value)
    } else {
        (value, 1.0)
    }
}

/// An element's fade as it draws here: its own, times the scope it draws
/// under. A canvas's renderer starts at 1, so an element inside a picture
/// doesn't count that picture's fade.
pub(crate) fn effective_fade(own: f32, renderer: &Renderer) -> f32 {
    own * renderer.current_modulated_opacity().unwrap_or(1.0)
}

/// Runs `draw` with `hosted`'s held fade back on as a scope: today's fade, paint
/// by paint, and one a child with a box reads as its own effective fade.
pub(crate) fn under_held_fade<R>(
    hosted: Option<&CoreHandle>,
    renderer: &mut Renderer,
    draw: impl FnOnce(&mut Renderer) -> R,
) -> R {
    let held = hosted
        .and_then(|hosted| hosted.with_downcast::<Artboard, _>(Artboard::held_fade))
        .unwrap_or(1.0);
    if held >= 1.0 {
        return draw(renderer);
    }
    renderer.save();
    renderer.modulate_opacity(held);
    let result = draw(renderer);
    renderer.restore();
    result
}

/// Draws a hosted artboard: as one picture while its effective fade is below
/// 1 and it can take one, and otherwise through `draw`, its usual draw, with
/// any held fade back on as a scope. `plain` is false under a draw visitor or a
/// mesh bypass, whose draws a picture would change.
pub(crate) fn draw_hosted(
    hosted: &CoreHandle,
    renderer: &mut Renderer,
    plain: bool,
    draw: impl FnOnce(&mut Renderer),
) {
    if let Some(held) = hosted
        .with_downcast::<Artboard, _>(held_screen_fade)
        .flatten()
    {
        return draw_screen(hosted, renderer, held, plain, draw);
    }
    let read = hosted.with_downcast::<Artboard, _>(|artboard| {
        let fades = artboard.base.base.is_css_mode() && artboard.child_opacity() != 0.0;
        (fades, artboard.held_fade())
    });
    let Some((true, held)) = read else {
        return draw(renderer);
    };
    let effective = effective_fade(held, renderer);
    if effective >= 1.0 {
        release(hosted);
        return draw(renderer);
    }
    if plain
        && effective > 0.0
        && !has_enabled_cache(hosted)
        && !is_lone_picture(hosted)
        && draw_picture(hosted, renderer, held)
    {
        return;
    }
    release(hosted);
    under_held_fade(Some(hosted), renderer, draw);
}

fn draw_picture(hosted: &CoreHandle, renderer: &mut Renderer, held: f32) -> bool {
    let Some(host) = canvas_host(hosted) else {
        return false;
    };
    let Term::Area(area) = box_term(hosted) else {
        return false;
    };
    let Some(place) = hosted.with_downcast::<Artboard, _>(Artboard::content_transform) else {
        return false;
    };
    // Out of the artboard for the draw: the renderer can reenter its owner.
    let Some(mut picture) = hosted
        .with_downcast_mut::<Artboard, _>(|artboard| std::mem::take(&mut artboard.group_picture))
    else {
        return false;
    };
    let drew = picture
        .draw(
            &host,
            renderer,
            (place, area, GUARD_PIXELS),
            held,
            |canvas| {
                Artboard::draw_content_with_visitor_handle(hosted, canvas, None);
            },
        )
        .is_some();
    hosted.with_downcast_mut::<Artboard, _>(|artboard| artboard.group_picture = picture);
    drew
}

fn release(hosted: &CoreHandle) {
    hosted.with_downcast_mut::<Artboard, _>(|artboard| artboard.group_picture.release());
}

/// The canvases `artboard` draws through, when its factory has them.
fn canvas_host(artboard: &CoreHandle) -> Option<DeferredCanvasHostHandle> {
    artboard
        .with_downcast::<Artboard, _>(Artboard::factory)
        .flatten()?
        .with_factory_mut(|factory| factory.canvas_content_host())
}

/// Whether an element holds its own fade out of what it and its children draw
/// at, to apply it to its picture or as a scope at its draw. Two do. A CSS box
/// does while it clips both axes at an own fade strictly between 0 and 1, in a
/// run that holds what it clips; zero reaches its children, so every "is it
/// hidden" check keeps its answer. A Screen does at an own fade below 1, zero
/// included, because its flights still draw at its zero. Any other artboard
/// hands its own opacity down as before. Read by the writer of render
/// opacity, the draw and a flight's pose, so the three can't disagree.
pub(crate) fn holds_fade(element: &dyn CoreObject) -> bool {
    match element.as_artboard() {
        Some(artboard) => screen_holds_fade(artboard),
        None => element.as_layout_component().is_some_and(box_holds_fade),
    }
}

fn box_holds_fade(layout: &LayoutComponent) -> bool {
    let own = layout.opacity();
    own > 0.0
        && own < 1.0
        && layout.is_css_mode()
        && !layout.is_artboard()
        && layout.clipped_axes() == (true, true)
        && run_holds_subtree(layout)
}

/// Whether a clipping CSS box's run, its proxy, its content and then the box,
/// holds what it clips. The CSS sort keeps it whole, and runs only where the
/// box's artboard has no draw rules. Draw rules keep Rive's order, which can
/// move the box's draw or its content out of its run, so a box there keeps
/// Rive's per-paint fade and takes no picture. Draw targets are file objects,
/// set at load, so this moves with no write. The box reaches its artboard as
/// its clip's save does, and nothing that asks holds the artboard's borrow.
fn run_holds_subtree(layout: &LayoutComponent) -> bool {
    layout
        .with_artboard(|artboard| !artboard.has_draw_rules())
        .unwrap_or(false)
}

/// The fade a box's draw applies itself: its own while it holds it, and 1
/// when its children carry it.
pub(crate) fn held_box_fade(layout: &LayoutComponent) -> f32 {
    if box_holds_fade(layout) {
        layout.opacity()
    } else {
        1.0
    }
}

/// The fades the boxes above `component` hold out of its render opacity,
/// multiplied, up to its artboard: what that number would carry if none of
/// them drew as a picture. A flight's pose reads it (decisions 68 and 69).
pub(crate) fn held_above(component: &CoreHandle) -> f32 {
    let mut fade = 1.0;
    let mut parent = component
        .with(|object| object.component_parent_handle())
        .flatten();
    while let Some(current) = parent {
        let Some((held, next)) = current
            .with(|object| {
                if object.as_artboard().is_some() {
                    return None;
                }
                let held = object.as_layout_component().map_or(1.0, held_box_fade);
                Some((held, object.component_parent_handle()))
            })
            .flatten()
        else {
            break;
        };
        fade *= held;
        parent = next;
    }
    fade
}

/// Draws a CSS box that clips both axes, in a run that holds what it clips,
/// from `proxy`, its own paint, as one picture while its effective fade is
/// below 1 and it can take one, and answers the box itself, which ends its
/// run, so the range moves past it. None leaves the run to draw as usual,
/// where a box that holds its fade puts it back as a scope
/// (`LayoutComponent::draw_proxy`). `plain` is false under a draw visitor,
/// whose draws a picture would change.
pub(crate) fn draw_box(
    root: &CoreHandle,
    renderer: &mut Renderer,
    proxy: &RuntimeDrawableOccurrence,
    plain: bool,
) -> Option<RuntimeDrawableOccurrence> {
    let owner = proxy.with_proxy(DrawableProxy::owner_handle)?;
    let (whole, own, held, has_canvas) = owner
        .with(|object| {
            let layout = object.as_layout_component()?;
            (layout.is_css_mode() && !layout.is_artboard()).then(|| {
                (
                    layout.clipped_axes() == (true, true) && run_holds_subtree(layout),
                    layout.opacity(),
                    held_box_fade(layout),
                    layout.group_picture.holds_canvas(),
                )
            })
        })
        .flatten()?;
    let effective = effective_fade(own, renderer);
    let drew = (whole && plain && effective > 0.0 && effective < 1.0)
        .then(|| box_run(proxy, &owner))
        .flatten()
        .filter(|run| !run.lone)
        .and_then(|run| draw_box_picture(root, renderer, proxy, &owner, run.end, held));
    if drew.is_none() && has_canvas {
        owner.with_mut(|object| {
            if let Some(layout) = object.as_layout_component_mut() {
                layout.group_picture.release();
            }
        });
    }
    drew
}

/// A box's run in the sorted list, which the sort keeps whole for a box that
/// clips: its proxy, its content, then the box, whose draw restores the clip.
struct BoxRun {
    end: RuntimeDrawableOccurrence,
    /// Its content is one image and nothing else, so it keeps its per-paint
    /// fade. A drawable that won't draw is nothing.
    lone: bool,
}

fn box_run(proxy: &RuntimeDrawableOccurrence, owner: &CoreHandle) -> Option<BoxRun> {
    let end = RuntimeDrawableOccurrence::Authored(owner.clone());
    let mut drawn = 0;
    let mut image = false;
    let mut current = proxy.with(Drawable::prev_drawable).flatten();
    while let Some(drawable) = current {
        if drawable == end {
            return Some(BoxRun {
                end: drawable,
                lone: drawn == 1 && image,
            });
        }
        current = drawable.with(Drawable::prev_drawable).flatten();
        if drawable.is_clip_start() || drawable.is_clip_end() || !drawable.will_draw() {
            continue;
        }
        drawn += 1;
        image = is_image(&drawable);
    }
    None
}

/// Draws a box's run from `proxy` to `end` into its picture: its own paints
/// at the strength its children see, its clip, its content and the clip's
/// restore. The picture goes down at `held`, under whatever scope `renderer`
/// holds.
fn draw_box_picture(
    root: &CoreHandle,
    renderer: &mut Renderer,
    proxy: &RuntimeDrawableOccurrence,
    owner: &CoreHandle,
    end: RuntimeDrawableOccurrence,
    held: f32,
) -> Option<RuntimeDrawableOccurrence> {
    let host = canvas_host(root)?;
    let Term::Area(area) = owner
        .with_mut(|object| object.as_layout_component_mut().map(clipped_box))
        .flatten()?
    else {
        return None;
    };
    // Out of the box for the draw: its proxy and its draw borrow it.
    let mut picture = owner
        .with_mut(|object| {
            object
                .as_layout_component_mut()
                .map(|layout| std::mem::take(&mut layout.group_picture))
        })
        .flatten()?;
    let drew = picture.draw(
        &host,
        renderer,
        (Mat2D::default(), area, GUARD_PIXELS),
        held,
        |canvas| {
            owner.with_mut(|object| {
                if let Some(layout) = object.as_layout_component_mut() {
                    layout.draw_proxy_with(canvas, 1.0);
                }
            });
            let content = proxy.with(Drawable::prev_drawable).flatten();
            Artboard::draw_drawable_range_handle(root, canvas, content, Some(end.clone()));
            end.draw(canvas);
        },
    );
    owner.with_mut(|object| {
        if let Some(layout) = object.as_layout_component_mut() {
            layout.group_picture = picture;
        }
    });
    drew.map(|()| end)
}

/// A Screen: a CSS row of a list whose artboard clips and has no host of its
/// own, the page's `screens`, with no clip of its own: neither its `clip` nor
/// its record's overflow on either axis. Its box is the page's frame
/// (`screen_box`). A row anywhere else isn't one, so an ordinary list never
/// makes a surface-sized picture per row.
pub(crate) fn is_screen(artboard: &Artboard) -> bool {
    let layout = &artboard.base.base;
    layout.is_css_mode()
        && !artboard.clip()
        && layout.clipped_axes() == (false, false)
        && artboard.is_page_row()
}

fn screen_holds_fade(artboard: &Artboard) -> bool {
    artboard.base.base.opacity() < 1.0 && is_screen(artboard)
}

/// The fade a Screen's draw applies itself: its own while it holds it, and
/// 1 when its children carry it. None for anything but a Screen.
fn held_screen_fade(artboard: &Artboard) -> Option<f32> {
    is_screen(artboard).then(|| {
        if screen_holds_fade(artboard) {
            artboard.base.base.opacity().max(0.0)
        } else {
            1.0
        }
    })
}

/// Where an entry of a Screen's draw list draws while the Screen fades. The
/// draw and the visibility walk both ask `screen_layer`, so they can't
/// disagree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScreenLayer {
    /// Below the Screen's first flight: in its picture. Hidden at its zero.
    Picture,
    /// After the first flight: outside the picture, under a scope at the
    /// Screen's fade. Hidden at its zero.
    Scope,
    /// A slot holding a flight, and all it hosts: outside the picture, at its
    /// own strength. Visible at the Screen's zero.
    OwnStrength,
}

/// What `screen_layer` reads of a Screen's sorted draw list: the slots
/// holding a flight, and whether a walk of the list in draw order has passed
/// the first of them. The draw reads it once a frame, the walk on each ask.
#[derive(Clone)]
pub(crate) struct ScreenList {
    flights: Vec<CoreHandle>,
    past_first: bool,
}

impl ScreenList {
    fn of(screen: &Artboard) -> Self {
        let mut flights = Vec::new();
        let mut current = screen.first_drawable();
        while let Some(drawable) = current {
            current = drawable.with(Drawable::prev_drawable).flatten();
            flights.extend(holds_flight(&drawable));
        }
        Self {
            flights,
            past_first: false,
        }
    }
}

/// Where `entry`, the next entry of a Screen's list in draw order, draws:
/// a slot holding a flight at its own strength, what comes after the first
/// of them under the Screen's scope, and what comes before in its picture.
/// With no flight, everything is before the first.
pub(crate) fn screen_layer(
    list: &mut ScreenList,
    entry: &RuntimeDrawableOccurrence,
) -> ScreenLayer {
    if entry
        .authored_handle()
        .is_some_and(|slot| list.flights.contains(&slot))
    {
        list.past_first = true;
        ScreenLayer::OwnStrength
    } else if list.past_first {
        ScreenLayer::Scope
    } else {
        ScreenLayer::Picture
    }
}

/// Where `element`, on the Screen `screen`, draws: where the first entry of
/// the sorted list that draws for it, it or anything under it, draws. The
/// visibility walk asks while the Screen reads 0, so it reads the list live.
pub(crate) fn screen_layer_of(screen: &CoreHandle, element: &CoreHandle) -> ScreenLayer {
    let Some((mut list, first)) = screen
        .with_downcast::<Artboard, _>(|screen| (ScreenList::of(screen), screen.first_drawable()))
    else {
        return ScreenLayer::Picture;
    };
    let mut current = first;
    while let Some(entry) = current {
        current = entry.with(Drawable::prev_drawable).flatten();
        let layer = screen_layer(&mut list, &entry);
        if draws_for(&entry, element) {
            return layer;
        }
    }
    if list.past_first {
        ScreenLayer::Scope
    } else {
        ScreenLayer::Picture
    }
}

/// Whether `entry` draws for `element`: it is the element, or the own paint
/// of a box that is, or sits under it on the element's artboard.
fn draws_for(entry: &RuntimeDrawableOccurrence, element: &CoreHandle) -> bool {
    let mut node = entry
        .authored_handle()
        .or_else(|| entry.with_proxy(DrawableProxy::owner_handle));
    while let Some(current) = node {
        if current == *element {
            return true;
        }
        node = current
            .with(|object| {
                if object.as_artboard().is_some() {
                    None
                } else {
                    object.component_parent_handle()
                }
            })
            .flatten();
    }
    false
}

/// A slot holding a flight: a nested artboard with a flight's record, role
/// 1, and a unit in it.
fn holds_flight(entry: &RuntimeDrawableOccurrence) -> Option<CoreHandle> {
    let slot = entry.authored_handle()?;
    let flies = slot
        .with(|object| {
            let nested = object.as_nested_artboard()?;
            nested.artboard_instance_default()?;
            Some(nested.children().iter().any(|child| {
                child
                    .with_downcast::<LiveSlot, _>(|record| record.values.role == 1)
                    .unwrap_or(false)
            }))
        })
        .flatten()
        .unwrap_or(false);
    flies.then_some(slot)
}

/// Which of a Screen's draws a draw of its content is.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ScreenPassKind {
    /// Pass one, into the picture: what answers `Picture`.
    Picture,
    /// Pass two, on the renderer: the rest, the Screen's own paints left out.
    Outside,
    /// No picture: every entry and the Screen's own paints under a scope at
    /// its fade, but a flight at its own strength.
    Fallback,
}

/// One of a Screen's draws, which `draw_screen` sets on the Screen and the
/// draw of its content takes (`Artboard::draw_content_with_visitor_handle`).
pub(crate) struct ScreenPass {
    kind: ScreenPassKind,
    fade: f32,
    list: ScreenList,
}

/// What a Screen's draw does with an entry its prelude would draw.
#[derive(Clone, Copy)]
pub(crate) enum ScreenStep {
    /// Draw it as usual.
    Draw,
    /// Leave it out.
    Skip,
    /// A box's own paint left in the picture: save its clip with its held
    /// fade on, for what draws outside under it, and paint nothing.
    Clip,
    /// Draw it as one entry under a scope at this fade.
    Scope(f32),
}

impl ScreenPass {
    /// The scope the Screen's own paints take, or None to leave them out.
    pub(crate) fn own_paints(&self) -> Option<f32> {
        match self.kind {
            ScreenPassKind::Picture => Some(1.0),
            ScreenPassKind::Outside => None,
            ScreenPassKind::Fallback => Some(self.fade),
        }
    }

    /// Where `entry`, the next entry in draw order, draws. Every entry is
    /// read, whether it draws or not, so the first flight counts where the
    /// list has it.
    pub(crate) fn layer(&mut self, entry: &RuntimeDrawableOccurrence) -> ScreenLayer {
        screen_layer(&mut self.list, entry)
    }

    /// What this draw does with `entry`, at `layer`. What restores a save made
    /// before it, a clip's end or a box, draws whatever its layer, since its
    /// opening drew in this draw too.
    pub(crate) fn step(&self, layer: ScreenLayer, entry: &RuntimeDrawableOccurrence) -> ScreenStep {
        match (self.kind, layer) {
            (ScreenPassKind::Picture, ScreenLayer::Picture) => ScreenStep::Draw,
            (ScreenPassKind::Picture, _) => ScreenStep::Skip,
            (_, ScreenLayer::OwnStrength) => ScreenStep::Draw,
            _ if closes_bracket(entry) => ScreenStep::Draw,
            (ScreenPassKind::Outside, ScreenLayer::Picture) => {
                if box_of(entry).is_some() {
                    ScreenStep::Clip
                } else {
                    ScreenStep::Skip
                }
            }
            _ => ScreenStep::Scope(self.fade),
        }
    }
}

/// The box whose own paint `entry` is: its proxy.
fn box_of(entry: &RuntimeDrawableOccurrence) -> Option<CoreHandle> {
    let owner = entry.with_proxy(DrawableProxy::owner_handle)?;
    owner
        .with(|object| object.as_layout_component().is_some() && object.as_artboard().is_none())
        .unwrap_or(false)
        .then_some(owner)
}

/// Whether `entry` restores a save an earlier entry made: a clip's end, or a
/// box, whose draw restores its proxy's clip.
fn closes_bracket(entry: &RuntimeDrawableOccurrence) -> bool {
    entry.is_clip_end()
        || entry.authored_handle().is_some_and(|handle| {
            handle
                .with(|object| {
                    object.as_layout_component().is_some() && object.as_artboard().is_none()
                })
                .unwrap_or(false)
        })
}

/// Whether `entry` is a box's own paint.
pub(crate) fn is_box_proxy(entry: &RuntimeDrawableOccurrence) -> bool {
    box_of(entry).is_some()
}

/// A box's own paint left in a Screen's picture, in the pass that draws what
/// is outside it: nothing of its own, which the picture holds, but its clip
/// saved with its held fade on, so what draws outside under it is clipped and
/// faded as in place. The box's `draw` restores it.
pub(crate) fn save_box_clip(proxy: &RuntimeDrawableOccurrence, renderer: &mut Renderer) {
    let Some(owner) = box_of(proxy) else {
        return;
    };
    owner.with_mut(|object| {
        if let Some(layout) = object.as_layout_component_mut() {
            let held = held_box_fade(layout);
            layout.draw_proxy_scoped(renderer, None, held);
        }
    });
}

/// A box's own paint after a Screen's first flight, one entry under a scope
/// at the Screen's fade `fade`. A box that takes its picture draws its whole
/// run into it, and the composite takes the scope; answers the box, which
/// ends the run. One that doesn't wraps its own paints in the scope and saves
/// its clip, and each of its children is an entry of its own.
pub(crate) fn draw_box_in_scope(
    root: &CoreHandle,
    renderer: &mut Renderer,
    proxy: &RuntimeDrawableOccurrence,
    fade: f32,
    plain: bool,
) -> Option<RuntimeDrawableOccurrence> {
    renderer.save();
    renderer.modulate_opacity(fade);
    let end = draw_box(root, renderer, proxy, plain);
    renderer.restore();
    if end.is_none()
        && let Some(owner) = box_of(proxy)
    {
        owner.with_mut(|object| {
            if let Some(layout) = object.as_layout_component_mut() {
                let held = held_box_fade(layout);
                layout.draw_proxy_scoped(renderer, Some(fade), held);
            }
        });
    }
    end
}

/// Draws a Screen at its fade `held` (rule 7). What draws below its first
/// flight goes into one picture, composited at `held`. Then the renderer
/// draws the rest: what comes after the first flight under a scope at
/// `held`, and its flights at their own strength. At an effective fade of 0
/// the picture's draw doesn't run and no canvas is made, while the rest still
/// draws. With no picture every entry but a flight takes the scope, the
/// Screen's own paints too, so its paints fade one by one as today.
fn draw_screen(
    row: &CoreHandle,
    renderer: &mut Renderer,
    held: f32,
    plain: bool,
    draw: impl FnOnce(&mut Renderer),
) {
    let effective = effective_fade(held, renderer);
    if effective >= 1.0 {
        release(row);
        return draw(renderer);
    }
    let Some(list) = row.with_downcast::<Artboard, _>(ScreenList::of) else {
        return draw(renderer);
    };
    let pictured = plain
        && effective > 0.0
        && !has_enabled_cache(row)
        && !is_lone_picture(row)
        && draw_screen_picture(row, renderer, held, list.clone());
    if !pictured {
        release(row);
    }
    if pictured && list.flights.is_empty() {
        return;
    }
    let kind = if pictured || effective <= 0.0 {
        ScreenPassKind::Outside
    } else {
        ScreenPassKind::Fallback
    };
    set_screen_pass(
        row,
        Some(ScreenPass {
            kind,
            fade: held,
            list,
        }),
    );
    draw(renderer);
    // A draw through a bitmap cache's kept raster never reached the content.
    set_screen_pass(row, None);
}

/// Draws what lies below a Screen's first flight into its picture,
/// composited at `held`. False, having drawn nothing, when it can't take one.
fn draw_screen_picture(
    row: &CoreHandle,
    renderer: &mut Renderer,
    held: f32,
    list: ScreenList,
) -> bool {
    let Some(host) = canvas_host(row) else {
        return false;
    };
    let Some((place, area)) = screen_box(row) else {
        return false;
    };
    // Out of the artboard for the draw: the renderer can reenter its owner.
    let Some(mut picture) = row
        .with_downcast_mut::<Artboard, _>(|artboard| std::mem::take(&mut artboard.group_picture))
    else {
        return false;
    };
    let drew = picture
        .draw(&host, renderer, (place, area, 0.0), held, |canvas| {
            set_screen_pass(
                row,
                Some(ScreenPass {
                    kind: ScreenPassKind::Picture,
                    fade: held,
                    list,
                }),
            );
            Artboard::draw_content_with_visitor_handle(row, canvas, None);
            set_screen_pass(row, None);
        })
        .is_some();
    row.with_downcast_mut::<Artboard, _>(|artboard| artboard.group_picture = picture);
    drew
}

fn set_screen_pass(row: &CoreHandle, pass: Option<ScreenPass>) {
    row.with_downcast::<Artboard, _>(|artboard| artboard.screen_pass.set(pass));
}

/// A Screen's box: the page's frame, which clips at the surface, mapped into
/// the Screen's space by the transform its list just drew it under. It takes
/// no guard pixels, so a Screen at the renderer's limit keeps its picture.
fn screen_box(row: &CoreHandle) -> Option<(Mat2D, Aabb)> {
    let (list, page, instance) = row
        .with_downcast::<Artboard, _>(|artboard| (artboard.host(), artboard.parent_artboard()))
        .map(|(list, page)| (list, page, row.runtime_artboard_instance()))?;
    let (list, page, instance) = (list?, page?, instance?);
    let area = page.with_downcast::<Artboard, _>(frame_clip).flatten()?;
    let drawn = list
        .with_downcast::<ArtboardComponentList, _>(|list| {
            list.draw_transform_for_index(list.index_of_artboard_instance(&instance))
        })
        .flatten()?;
    let mut place = Mat2D::default();
    drawn.invert(&mut place).then_some((place, area))
}

/// An enabled bitmap cache is one picture already: it composites under the
/// fallback scope.
fn has_enabled_cache(artboard: &CoreHandle) -> bool {
    artboard
        .with_downcast::<Artboard, _>(Artboard::bitmap_cache)
        .flatten()
        .and_then(|cache| cache.with_downcast::<BitmapCache, _>(BitmapCache::cache_enabled))
        .unwrap_or(false)
}

/// Own paints plus one image, with nothing else, keep their per-paint fade:
/// that's Hero's image copy. Read at the draw, where the sorted list is
/// current.
fn is_lone_picture(artboard: &CoreHandle) -> bool {
    let Some(list) = artboard.with_downcast::<Artboard, _>(DrawList::of) else {
        return false;
    };
    list.count == 1 && list.first.as_ref().is_some_and(is_image)
}

/// An image, which a video isn't: a video draws as an image but plays live.
fn is_image(drawable: &RuntimeDrawableOccurrence) -> bool {
    drawable.authored_handle().is_some_and(|image| {
        image.is_type_of(ImageBase::TYPE_KEY) && !image.is_type_of(Video::TYPE_KEY)
    })
}

/// An artboard's draw list with clip operations set aside.
struct DrawList {
    first: Option<RuntimeDrawableOccurrence>,
    last: Option<RuntimeDrawableOccurrence>,
    count: usize,
}

impl DrawList {
    fn of(artboard: &Artboard) -> Self {
        let mut list = Self {
            first: None,
            last: None,
            count: 0,
        };
        let mut current = artboard.first_drawable();
        while let Some(drawable) = current {
            current = drawable.with(Drawable::prev_drawable).flatten();
            if drawable.is_clip_start() || drawable.is_clip_end() {
                continue;
            }
            list.count += 1;
            list.first.get_or_insert_with(|| drawable.clone());
            list.last = Some(drawable);
        }
        list
    }
}

/// Part of a picture's box: nothing drawn, an area, or unknown, which only a
/// frame clip can answer for.
#[derive(Clone, Copy)]
enum Term {
    Empty,
    Area(Aabb),
    Unknown,
}

impl Term {
    fn of(area: Aabb) -> Self {
        if area.is_empty_or_nan() {
            Self::Empty
        } else {
            Self::Area(area)
        }
    }

    /// A painted bound counts only when exact: a picture cuts off anything
    /// its box misses.
    fn painted(fidelity: BoundsFidelity, area: Aabb) -> Self {
        match fidelity {
            BoundsFidelity::Exact => Self::of(area),
            BoundsFidelity::Approximate => Self::Unknown,
            BoundsFidelity::None => Self::Empty,
        }
    }

    fn joined(self, other: Self) -> Self {
        match (self, other) {
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::Empty, term) | (term, Self::Empty) => term,
            (Self::Area(mut area), Self::Area(other)) => {
                area.expand(other);
                Self::Area(area)
            }
        }
    }
}

/// What an artboard's draw can reach, in the space its content draws in: its
/// frame clip around its own paints and its content. The box holds every pixel
/// the vectors would draw, so it comes only from clips the draw installs and
/// from own paints with an exact bound, and never from bounding what the
/// content paints.
fn box_term(artboard: &CoreHandle) -> Term {
    let read = artboard.with_downcast::<Artboard, _>(|artboard| {
        (
            frame_clip(artboard),
            own_paints(artboard),
            record_clip(artboard),
            DrawList::of(artboard),
            artboard.has_draw_rules(),
        )
    });
    let Some((frame, own, record, list, has_rules)) = read else {
        return Term::Unknown;
    };
    let content = record.unwrap_or_else(|| listed_content(&list, has_rules));
    match (frame, own.joined(content)) {
        (Some(frame), Term::Area(area)) => Term::of(Aabb::new(
            area.left().max(frame.left()),
            area.top().max(frame.top()),
            area.right().min(frame.right()),
            area.bottom().min(frame.bottom()),
        )),
        (Some(frame), Term::Unknown) => Term::of(frame),
        (Some(_), Term::Empty) => Term::Empty,
        (None, term) => term,
    }
}

/// The path the draw clips its frame to, in the space of the artboard's paths.
fn frame_clip(artboard: &Artboard) -> Option<Aabb> {
    if !artboard.clip() {
        return None;
    }
    let paths = artboard.base.base.render_paths_handle()?;
    let paths = paths.borrow();
    let clip = paths.frame_clip.as_ref().unwrap_or(&paths.local);
    Some(clip.raw_path().bounds())
}

/// The artboard's own paints, which sit outside its overflow clip. A Rive-made
/// artboard has no fixed set of them.
fn own_paints(artboard: &Artboard) -> Term {
    let layout = &artboard.base.base;
    let Some(geometry) = layout.css_paint_geometry() else {
        return Term::Unknown;
    };
    let origin = artboard.content_origin();
    let mut painted = Aabb::default();
    let fidelity = layout.css_painted_bounds(
        geometry.bounds.offset(-origin.x, -origin.y),
        &Mat2D::default(),
        &mut painted,
    );
    Term::painted(fidelity, painted)
}

/// The record's overflow clip, drawn before every drawable. A clip on one axis
/// is a band with no end on the other, so it counts as no clip.
fn record_clip(artboard: &Artboard) -> Option<Term> {
    let layout = &artboard.base.base;
    if !layout.is_css_mode() || layout.clipped_axes() != (true, true) {
        return None;
    }
    Some(layout.render_paths_handle().map_or(Term::Unknown, |paths| {
        Term::of(paths.borrow().clip.raw_path().bounds())
    }))
}

/// What a draw list reaches with no record clip above it.
fn listed_content(list: &DrawList, has_rules: bool) -> Term {
    let content = match list.count {
        0 => return Term::Empty,
        1 => one_hosted_component(list),
        _ => one_root_box(list, has_rules),
    };
    content.unwrap_or(Term::Unknown)
}

/// One component host: what its component reaches, placed as the host draws
/// it. A mesh host draws through its mesh, which no clip bounds.
fn one_hosted_component(list: &DrawList) -> Option<Term> {
    let host = list.first.as_ref()?.authored_handle()?;
    let (hosted, world) = host
        .with(|object| {
            let nested = object.as_nested_artboard()?;
            if nested.hosted_mesh().is_some() {
                return None;
            }
            let hosted = nested.artboard_instance_default()?.core_handle();
            Some((hosted, nested.world_transform()))
        })
        .flatten()?;
    Some(match box_term(&hosted) {
        Term::Area(area) => {
            let content = hosted.with_downcast::<Artboard, _>(Artboard::content_transform)?;
            Term::of((world * content).map_bounding_box(area))
        }
        term => term,
    })
}

/// One root CSS box: the list begins with its proxy, which paints it and opens
/// its clip, and ends with the box, which closes it. It clips both axes, or
/// nothing sits between the two. Draw rules can move a box past its proxy, and
/// a hidden box installs no clip.
fn one_root_box(list: &DrawList, has_rules: bool) -> Option<Term> {
    if has_rules {
        return None;
    }
    let root = list.last.as_ref()?.authored_handle()?;
    let proxy_owner = list
        .first
        .as_ref()?
        .with_proxy(|proxy| proxy.owner_handle())?;
    if proxy_owner != root {
        return None;
    }
    let holds_nothing = list.count == 2;
    root.with_mut(|object| {
        let layout = object.as_layout_component_mut()?;
        if !layout.is_css_mode() || layout.is_hidden() {
            return None;
        }
        if holds_nothing {
            return Some(box_paints(layout));
        }
        (layout.clipped_axes() == (true, true)).then(|| clipped_box(layout))
    })
    .flatten()
}

/// A box's own paints, in its artboard's space.
fn box_paints(layout: &mut LayoutComponent) -> Term {
    let mut painted = Aabb::default();
    let fidelity = layout.painted_world_bounds(&mut painted);
    Term::painted(fidelity, painted)
}

/// What a CSS box that clips both axes can reach, in its artboard's space:
/// its overflow clip, and its own paints, which sit outside that clip.
fn clipped_box(layout: &mut LayoutComponent) -> Term {
    let own = box_paints(layout);
    let Some(paths) = layout.render_paths_handle() else {
        return Term::Unknown;
    };
    let clip = paths.borrow().clip.raw_path().bounds();
    own.joined(Term::of(clip))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shrink_needs_every_smaller_draw_in_a_row() {
        let mut smaller = 0;
        for _ in 1..K_SHRINK_DRAWS {
            let size = next_canvas_size((100, 100), (512, 192), &mut smaller, 8192);
            assert_eq!(size, (512, 192));
        }
        // A draw that fits the canvas exactly starts the count again.
        assert_eq!(
            next_canvas_size((500, 190), (512, 192), &mut smaller, 8192),
            (512, 192)
        );
        assert_eq!(smaller, 0);
    }

    #[test]
    fn growth_stops_at_the_limit_and_never_shrinks_an_axis() {
        let mut smaller = 0;
        assert_eq!(
            next_canvas_size((2100, 100), (2048, 512), &mut smaller, 2660),
            (2660, 512)
        );
    }
}
