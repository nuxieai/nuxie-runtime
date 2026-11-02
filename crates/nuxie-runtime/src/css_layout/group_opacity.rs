//! A CSS element that fades draws as one picture (docs/CSS_LAYOUT.md, 4.4):
//! a hosted CSS artboard, a CSS box that clips both axes, and a Screen, whose
//! flights draw outside its picture. Rive has no save layer, so opacity
//! multiplies down the tree and each paint takes it alone. CSS says an element
//! and its descendants composite as one, so the element draws whole into a
//! canvas and the canvas goes down at the fade, once. Rive-made content keeps
//! upstream's per-paint fade.
use super::{
    keys::FADE_OUTSIDE,
    record::{LayoutCssStyle, find_in},
};
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
    shapes::shape::Shape,
};
use crate::video::Video;
use nuxie_render_api::{BlendMode, DeferredCanvasHostHandle, ImageSampler};
use std::{collections::HashMap, rc::Rc};

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

/// What a box's draw did with its run.
pub(crate) enum BoxDrawn {
    /// No picture: the run draws entry by entry, from the box's own paint.
    Entries,
    /// The whole run is in the picture: the range moves to its end, the box.
    Whole(RuntimeDrawableOccurrence),
    /// In a Screen's draw, the picture left out what draws otherwise: a
    /// picture never holds a marked descendant. `layer` is the box's.
    LeftOut {
        end: RuntimeDrawableOccurrence,
        layer: ScreenLayer,
    },
}

/// Draws a CSS box that clips both axes, in a run that holds what it clips,
/// from `proxy`, its own paint, as one picture while its effective fade is
/// below 1 and it can take one. `Entries` leaves the run to draw as usual,
/// where a box that holds its fade puts it back as a scope
/// (`LayoutComponent::draw_proxy`). `plain` is false under a draw visitor,
/// whose draws a picture would change. In a Screen's draw, `pass`, the
/// picture holds what answers as the box does and leaves the rest out
/// (`past_box`).
pub(crate) fn draw_box(
    root: &CoreHandle,
    renderer: &mut Renderer,
    proxy: &RuntimeDrawableOccurrence,
    plain: bool,
    pass: Option<&ScreenPass>,
) -> BoxDrawn {
    let Some(owner) = proxy.with_proxy(DrawableProxy::owner_handle) else {
        return BoxDrawn::Entries;
    };
    let Some((whole, own, held, has_canvas)) = owner
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
        .flatten()
    else {
        return BoxDrawn::Entries;
    };
    let effective = effective_fade(own, renderer);
    let layer = pass.map(|pass| pass.list.layer(proxy));
    let hold = pass.zip(layer).map(|(pass, layer)| pass.hold(layer));
    let drew = (whole && plain && effective > 0.0 && effective < 1.0)
        .then(|| box_run(proxy, &owner))
        .flatten()
        .filter(|run| !run.lone)
        .and_then(|run| draw_box_picture(root, renderer, proxy, &owner, run.end, held, hold));
    if drew.is_none() && has_canvas {
        owner.with_mut(|object| {
            if let Some(layout) = object.as_layout_component_mut() {
                layout.group_picture.release();
            }
        });
    }
    match (drew, layer) {
        (None, _) => BoxDrawn::Entries,
        (Some((end, true)), Some(layer)) => BoxDrawn::LeftOut { end, layer },
        (Some((end, _)), _) => BoxDrawn::Whole(end),
    }
}

/// Where the range goes after `drawn`, a box's draw from `proxy`: past the
/// run when it is all in the picture, and otherwise None, to go on entry by
/// entry. A picture that left out what draws otherwise moves on as well in a
/// pass that draws into a picture, which then left it out too, since that
/// pass draws none of it. A pass on the renderer re-enters the box to draw
/// what it left out: its own paints skipped, its clip saved and its held
/// fade on as a scope, and from there what answers as the box does is
/// skipped, to the box, which restores the clip (`ScreenPass::step`).
pub(crate) fn past_box(
    drawn: BoxDrawn,
    pass: Option<&mut ScreenPass>,
    proxy: &RuntimeDrawableOccurrence,
    renderer: &mut Renderer,
) -> Option<RuntimeDrawableOccurrence> {
    match (drawn, pass) {
        (BoxDrawn::Entries, _) => None,
        (BoxDrawn::Whole(end), _) | (BoxDrawn::LeftOut { end, .. }, None) => Some(end),
        (BoxDrawn::LeftOut { end, .. }, Some(pass))
            if matches!(pass.kind, ScreenPassKind::Hold(_)) =>
        {
            pass.left_out = true;
            Some(end)
        }
        (BoxDrawn::LeftOut { end, layer }, Some(pass)) => {
            save_box_clip(proxy, renderer);
            pass.reentered.push((end, layer));
            None
        }
    }
}

/// A box's run in the sorted list, which the sort keeps whole for a box that
/// clips: its proxy, its content, then the box, whose draw restores the clip.
struct BoxRun {
    end: RuntimeDrawableOccurrence,
    /// Its content is one image and nothing else, so it keeps its per-paint
    /// fade. A drawable that won't draw is nothing, and so is a paintless Shape.
    lone: bool,
}

fn box_run(proxy: &RuntimeDrawableOccurrence, owner: &CoreHandle) -> Option<BoxRun> {
    let end = RuntimeDrawableOccurrence::Authored(owner.clone());
    let mut content = LoneContent::default();
    let mut current = proxy.with(Drawable::prev_drawable).flatten();
    while let Some(drawable) = current {
        if drawable == end {
            return Some(BoxRun {
                end: drawable,
                lone: content.is_lone_picture(),
            });
        }
        current = drawable.with(Drawable::prev_drawable).flatten();
        if drawable.is_clip_start() || drawable.is_clip_end() || !drawable.will_draw() {
            continue;
        }
        content.add(&drawable);
    }
    None
}

/// Draws a box's run from `proxy` to `end` into its picture: its own paints
/// at the strength its children see, its clip, its content and the clip's
/// restore. The picture goes down at `held`, under whatever scope `renderer`
/// holds. In a Screen's draw `hold` is the box's pass, which leaves out of
/// the content what answers otherwise. Answers `end`, and whether anything
/// was left out.
fn draw_box_picture(
    root: &CoreHandle,
    renderer: &mut Renderer,
    proxy: &RuntimeDrawableOccurrence,
    owner: &CoreHandle,
    end: RuntimeDrawableOccurrence,
    held: f32,
    hold: Option<ScreenPass>,
) -> Option<(RuntimeDrawableOccurrence, bool)> {
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
    let mut left_out = false;
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
            left_out = Artboard::draw_drawable_range_holding(
                root,
                canvas,
                content,
                Some(end.clone()),
                hold,
            );
            end.draw(canvas);
        },
    );
    owner.with_mut(|object| {
        if let Some(layout) = object.as_layout_component_mut() {
            layout.group_picture = picture;
        }
    });
    drew.map(|()| (end, left_out))
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

/// Whether a Screen's draw reads its fade as 0: any own opacity at or below
/// 0, an easing's overshoot included (`held_screen_fade`). The visibility
/// walk asks this, so it reads the zero the draw does.
pub(crate) fn screen_fade_is_zero(artboard: &Artboard) -> bool {
    held_screen_fade(artboard) == Some(0.0)
}

/// Where an entry of a Screen's draw list draws while the Screen fades. The
/// draw and the visibility walk both read `ScreenList`, so they can't
/// disagree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScreenLayer {
    /// Mark 0 below the Screen's first flight: in its picture. Hidden at its
    /// zero.
    Picture,
    /// Mark 1 below the first flight, and mark 0 after it: outside the
    /// picture, under a scope at the Screen's fade. Hidden at its zero.
    Scope,
    /// A slot holding a flight and all it hosts, mark 1 at or above the first
    /// flight, and mark 2: outside the picture, at its own strength. Visible
    /// at the Screen's zero.
    OwnStrength,
}

/// An element's mark, its record's `fadeOutside`: where a fading Screen draws
/// it and what it holds (docs/live-moves.md). A mark applies to the element
/// and all under it, and a mark under a marked element adds nothing.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mark {
    /// 0, the default: in the Screen's picture.
    Inside,
    /// 1: outside it, at the Screen's fade below its first flight and at its
    /// own strength above it.
    Outside,
    /// 2: outside it, at its own strength.
    OwnStrength,
}

impl Mark {
    /// A value between two marks reads as the nearer, and one that isn't a
    /// number as 0.
    fn of(value: f32) -> Self {
        if value >= 1.5 {
            Self::OwnStrength
        } else if value >= 0.5 {
            Self::Outside
        } else {
            Self::Inside
        }
    }

    /// Where an element with this mark draws, before the first flight or
    /// past it.
    fn layer(self, past_first: bool) -> ScreenLayer {
        match (self, past_first) {
            (Self::OwnStrength, _) | (Self::Outside, true) => ScreenLayer::OwnStrength,
            (Self::Outside, false) | (Self::Inside, true) => ScreenLayer::Scope,
            (Self::Inside, false) => ScreenLayer::Picture,
        }
    }
}

/// The marks a Screen's elements draw by, each read once: the outermost mark
/// on the element or above it on its artboard.
#[derive(Default)]
struct Marks(HashMap<CoreHandle, Mark>);

impl Marks {
    fn of(&mut self, node: &CoreHandle) -> Mark {
        let mut above = Mark::Inside;
        let mut unread = Vec::new();
        let mut current = Some(node.clone());
        while let Some(handle) = current {
            if let Some(&mark) = self.0.get(&handle) {
                above = mark;
                break;
            }
            let Some((own, parent)) = handle
                .with(|object| {
                    object
                        .as_artboard()
                        .is_none()
                        .then(|| (own_mark(object), object.component_parent_handle()))
                })
                .flatten()
            else {
                break;
            };
            unread.push((handle, own));
            current = parent;
        }
        for (handle, own) in unread.into_iter().rev() {
            if above == Mark::Inside {
                above = own;
            }
            self.0.insert(handle, above);
        }
        above
    }
}

/// The mark on an element's own record: a box's, or for a slot the one
/// among its children.
fn own_mark(object: &dyn CoreObject) -> Mark {
    let record = match object.as_layout_component() {
        Some(layout) => layout.css_record(),
        None => object
            .as_container_component()
            .and_then(|container| find_in(container.children())),
    };
    record
        .and_then(|record| {
            record
                .with_downcast::<LayoutCssStyle, _>(|record| record.number(FADE_OUTSIDE))
                .flatten()
        })
        .map_or(Mark::Inside, Mark::of)
}

/// Where each entry of a Screen's sorted draw list draws, from one read of
/// the list in draw order: a slot holding a flight at its own strength, and
/// anything else by its mark and whether the first flight is past
/// (`Mark::layer`). With no flight, everything is before the first. A mask
/// run stays whole, since a mask keeps one content canvas: everything in it
/// answers as the run's start does, by the marks on what the mask sits on
/// and above it, so a mark inside the run doesn't count. No flight sits in
/// a run. The draw reads the list once a frame, the walk on each ask, live,
/// because Android asks after the scene step and before drawing.
#[derive(Clone)]
pub(crate) struct ScreenList {
    layers: Rc<HashMap<RuntimeDrawableOccurrence, ScreenLayer>>,
    /// Some entry draws outside the picture.
    outside: bool,
    /// The list holds a flight.
    flight: bool,
}

impl ScreenList {
    fn of(screen: &CoreHandle) -> Option<Self> {
        let mut current = screen.with_downcast::<Artboard, _>(Artboard::first_drawable)?;
        let mut marks = Marks::default();
        let mut layers = HashMap::new();
        let (mut past_first, mut outside) = (false, false);
        // The answer of the mask run being read, and how many runs deep.
        let mut run: Option<(ScreenLayer, usize)> = None;
        while let Some(entry) = current {
            current = entry.with(Drawable::prev_drawable).flatten();
            let layer = if holds_flight(&entry).is_some() {
                past_first = true;
                ScreenLayer::OwnStrength
            } else if let Some((layer, _)) = run {
                layer
            } else {
                entry_node(&entry)
                    .map_or(Mark::Inside, |node| marks.of(&node))
                    .layer(past_first)
            };
            if entry.is_mask_start() {
                run = Some((layer, run.map_or(1, |(_, depth)| depth + 1)));
            } else if entry.is_mask_end() {
                run = run
                    .filter(|&(_, depth)| depth > 1)
                    .map(|(layer, depth)| (layer, depth - 1));
            }
            outside |= layer != ScreenLayer::Picture;
            layers.insert(entry, layer);
        }
        Some(Self {
            layers: Rc::new(layers),
            outside,
            flight: past_first,
        })
    }

    /// Where `entry` draws. One the read didn't meet, which a draw's
    /// callback has linked in since, draws in the picture.
    fn layer(&self, entry: &RuntimeDrawableOccurrence) -> ScreenLayer {
        self.layers
            .get(entry)
            .copied()
            .unwrap_or(ScreenLayer::Picture)
    }

    /// Each entry of `screen`'s list and where it draws, in draw order. The
    /// map's own order changes from one read to the next.
    #[cfg(any(test, feature = "testing"))]
    fn fingerprint(&self, screen: &CoreHandle) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let mut current = screen
            .with_downcast::<Artboard, _>(Artboard::first_drawable)
            .flatten();
        while let Some(entry) = current {
            current = entry.with(Drawable::prev_drawable).flatten();
            entry.hash(&mut hasher);
            (self.layer(&entry) as u8).hash(&mut hasher);
        }
        hasher.finish()
    }
}

/// The element an entry draws for: itself, or a proxy's owner, which is a
/// box for its own paint and a mask for its run's markers.
fn entry_node(entry: &RuntimeDrawableOccurrence) -> Option<CoreHandle> {
    entry
        .authored_handle()
        .or_else(|| entry.with_proxy(DrawableProxy::owner_handle))
}

/// Where `element`, on the Screen `screen`, draws: where the first entry of
/// the sorted list that draws for it, it or anything under it, draws. One
/// no entry draws for reads by its mark, past the first flight when there
/// is one. The visibility walk asks while the Screen reads 0.
pub(crate) fn screen_layer_of(screen: &CoreHandle, element: &CoreHandle) -> ScreenLayer {
    let Some(list) = ScreenList::of(screen) else {
        return ScreenLayer::Picture;
    };
    let mut current = screen
        .with_downcast::<Artboard, _>(Artboard::first_drawable)
        .flatten();
    while let Some(entry) = current {
        current = entry.with(Drawable::prev_drawable).flatten();
        if draws_for(&entry, element) {
            return list.layer(&entry);
        }
    }
    Marks::default().of(element).layer(list.flight)
}

/// Whether `entry` draws for `element`: it is the element, or the own paint
/// of a box that is, or sits under it on the element's artboard.
fn draws_for(entry: &RuntimeDrawableOccurrence, element: &CoreHandle) -> bool {
    let mut node = entry_node(entry);
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
    /// Into a picture: only what answers this layer. Pass one is the
    /// Screen's picture, at `Picture`. In a Screen's draw a box's picture
    /// holds what answers as the box does.
    Hold(ScreenLayer),
    /// Pass two, on the renderer: the rest, the Screen's own paints left out.
    Outside,
    /// No picture: every entry and the Screen's own paints under a scope at
    /// its fade, but what draws at its own strength.
    Fallback,
}

/// One of a Screen's draws, which `draw_screen` sets on the Screen and the
/// draw of its content takes (`Artboard::draw_content_with_visitor_handle`),
/// or a box's picture inside one (`draw_box`).
pub(crate) struct ScreenPass {
    kind: ScreenPassKind,
    fade: f32,
    /// The Screen's effective fade is 0: what rides it draws nothing and makes
    /// no canvas (ruling U12).
    gone: bool,
    list: ScreenList,
    /// The boxes this pass re-entered after their pictures went down,
    /// innermost last: each box's own entry, which ends its run, and the
    /// layer it answers.
    reentered: Vec<(RuntimeDrawableOccurrence, ScreenLayer)>,
    /// A picture this pass draws into left something out.
    left_out: bool,
}

/// What a Screen's draw does with an entry its prelude would draw.
#[derive(Clone, Copy)]
pub(crate) enum ScreenStep {
    /// Draw it as usual.
    Draw,
    /// Leave it out.
    Skip,
    /// A box's own paint left in a picture: save its clip with its held fade
    /// on, for what draws outside under it, and paint nothing.
    Clip,
    /// Draw it as one entry under a scope at this fade.
    Scope(f32),
}

impl ScreenPass {
    fn new(kind: ScreenPassKind, fade: f32, gone: bool, list: ScreenList) -> Self {
        Self {
            kind,
            fade,
            gone,
            list,
            reentered: Vec::new(),
            left_out: false,
        }
    }

    /// The pass a box's picture draws its run with in this draw: what
    /// answers `layer`, the box's.
    fn hold(&self, layer: ScreenLayer) -> Self {
        Self::new(
            ScreenPassKind::Hold(layer),
            self.fade,
            self.gone,
            self.list.clone(),
        )
    }

    /// Whether a picture this pass drew into left something out.
    pub(crate) fn left_out(&self) -> bool {
        self.left_out
    }

    /// The scope the Screen's own paints take, or None to leave them out.
    pub(crate) fn own_paints(&self) -> Option<f32> {
        match self.kind {
            ScreenPassKind::Hold(_) => Some(1.0),
            ScreenPassKind::Outside => None,
            ScreenPassKind::Fallback => Some(self.fade),
        }
    }

    /// Where `entry`, the next entry in draw order, draws. Every entry is
    /// read, whether it draws or not, so a re-entered box's run ends at the
    /// box even when the box doesn't draw.
    pub(crate) fn layer(&mut self, entry: &RuntimeDrawableOccurrence) -> ScreenLayer {
        if self.reentered.last().is_some_and(|(end, _)| end == entry) {
            self.reentered.pop();
        }
        self.list.layer(entry)
    }

    /// What this draw does with `entry`, at `layer`. What restores a save
    /// made before it, a clip's end or a box, draws whatever its layer,
    /// since its opening drew in this draw too. What a picture already holds
    /// leaves only its boxes' clips: in pass two, what answers `Picture`, and
    /// in a re-entered box's run, what answers as the box does.
    pub(crate) fn step(
        &mut self,
        layer: ScreenLayer,
        entry: &RuntimeDrawableOccurrence,
    ) -> ScreenStep {
        if let ScreenPassKind::Hold(held) = self.kind {
            if layer == held {
                return ScreenStep::Draw;
            }
            self.left_out = true;
            return ScreenStep::Skip;
        }
        let in_a_picture = self
            .reentered
            .last()
            .is_some_and(|&(_, held)| held == layer)
            || match (self.kind, layer) {
                (ScreenPassKind::Outside, ScreenLayer::Picture) => true,
                // What rides the Screen's fade at 0 draws nothing and makes
                // no canvas (ruling U12).
                (_, ScreenLayer::Scope) => self.gone,
                _ => false,
            };
        if closes_bracket(entry) {
            ScreenStep::Draw
        } else if in_a_picture {
            if box_of(entry).is_some() {
                ScreenStep::Clip
            } else {
                ScreenStep::Skip
            }
        } else if layer == ScreenLayer::OwnStrength {
            ScreenStep::Draw
        } else {
            ScreenStep::Scope(self.fade)
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

/// A box's own paint left in a picture, in a pass that draws what is outside
/// it: nothing of its own, which the picture holds, but its clip saved with
/// its held fade on, so what draws outside under it is clipped and faded as
/// in place. The box's `draw` restores it.
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

/// A box's own paint in a Screen's pass two, one entry under a scope at the
/// Screen's fade `fade`. A box that takes its picture draws its run into it,
/// and the composite takes the scope; answers the box, which ends the run,
/// or, when the picture left out a marked descendant, re-enters the box
/// (`past_box`). One that doesn't wraps its own paints in the scope and saves
/// its clip, and each of its children is an entry of its own.
pub(crate) fn draw_box_in_scope(
    root: &CoreHandle,
    renderer: &mut Renderer,
    proxy: &RuntimeDrawableOccurrence,
    fade: f32,
    plain: bool,
    pass: Option<&mut ScreenPass>,
) -> Option<RuntimeDrawableOccurrence> {
    renderer.save();
    renderer.modulate_opacity(fade);
    let drawn = draw_box(root, renderer, proxy, plain, pass.as_deref());
    renderer.restore();
    if let BoxDrawn::Entries = drawn {
        if let Some(owner) = box_of(proxy) {
            owner.with_mut(|object| {
                if let Some(layout) = object.as_layout_component_mut() {
                    let held = held_box_fade(layout);
                    layout.draw_proxy_scoped(renderer, Some(fade), held);
                }
            });
        }
        return None;
    }
    past_box(drawn, pass, proxy, renderer)
}

/// Draws a Screen at its fade `held` (rule 7). What answers `Picture` goes
/// into one picture, composited at `held`. Then the renderer draws the rest:
/// what answers `Scope` under a scope at `held`, and what answers
/// `OwnStrength`, its flights and what is marked to, at its own strength. At
/// an effective fade of 0 the draw is that second pass alone, straight onto
/// the renderer, and what rides the fade is left out, so no canvas is made
/// for the Screen. With no picture every entry but what draws at its own
/// strength takes the scope, the Screen's own paints too, so its paints fade
/// one by one as today. Below 1 a plain draw passes the Screen's bitmap
/// cache, whose raster would keep one frame's split across frames, so a
/// cached Screen fades as an uncached one does. At 1 the cache serves it.
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
    let Some(list) = ScreenList::of(row) else {
        return draw(renderer);
    };
    #[cfg(any(test, feature = "testing"))]
    let _seen = SplitSeen::check(row, &list);
    let pictured = plain
        && effective > 0.0
        && !is_lone_picture(row)
        && draw_screen_picture(row, renderer, held, list.clone());
    if !pictured {
        release(row);
    }
    if pictured && !list.outside {
        return;
    }
    let kind = if pictured || effective <= 0.0 {
        ScreenPassKind::Outside
    } else {
        ScreenPassKind::Fallback
    };
    set_screen_pass(
        row,
        Some(ScreenPass::new(kind, held, effective <= 0.0, list)),
    );
    if plain {
        // `draw`, a plain draw's, less the bitmap cache, whose raster would
        // hold this pass (`Artboard::draw_internal_with_visitor_handle`).
        Artboard::draw_content_with_visitor_handle(row, renderer, None);
    } else {
        draw(renderer);
    }
    // The content draw takes the pass. This keeps one it missed from the next
    // draw.
    set_screen_pass(row, None);
}

/// A test build's check that a Screen's split reports its changes (design,
/// "The split keeps Rive's two rules"): where each entry of its list draws,
/// from the last draw that read it, against the content revision a kept
/// result keys on. An input that moves the split and moves no revision fails
/// the draw, in place of a stale result nobody looks at.
#[cfg(any(test, feature = "testing"))]
struct SplitSeen<'a> {
    row: &'a CoreHandle,
    fingerprint: u64,
}

#[cfg(any(test, feature = "testing"))]
impl<'a> SplitSeen<'a> {
    fn check(row: &'a CoreHandle, list: &ScreenList) -> Self {
        let fingerprint = list.fingerprint(row);
        let seen = row
            .with_downcast::<Artboard, _>(|artboard| artboard.split_seen.get())
            .flatten();
        if let Some((seen_fingerprint, seen_revision)) = seen {
            assert!(
                seen_fingerprint == fingerprint || seen_revision != content_revision(row),
                "a Screen's split changed with no report: its entries draw elsewhere than on \
                 its last draw, at content revision {seen_revision}, which hasn't moved"
            );
        }
        Self { row, fingerprint }
    }
}

/// The revision is read after the draw, so a change the draw itself reports
/// doesn't disarm the next comparison.
#[cfg(any(test, feature = "testing"))]
impl Drop for SplitSeen<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        let revision = content_revision(self.row);
        self.row.with_downcast::<Artboard, _>(|artboard| {
            artboard.split_seen.set(Some((self.fingerprint, revision)));
        });
    }
}

#[cfg(any(test, feature = "testing"))]
fn content_revision(row: &CoreHandle) -> u64 {
    row.artboard_dirty_handle()
        .map_or(0, |dirty| dirty.content_revision())
}

/// Draws what answers `Picture` on a Screen into its picture, composited at
/// `held`. False, having drawn nothing, when it can't take one.
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
                Some(ScreenPass::new(
                    ScreenPassKind::Hold(ScreenLayer::Picture),
                    held,
                    false,
                    list,
                )),
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

/// A hosted artboard's enabled bitmap cache is one picture already: it
/// composites under the fallback scope.
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
    artboard
        .with_downcast::<Artboard, _>(DrawList::of)
        .is_some_and(|list| list.content.is_lone_picture())
}

/// What the lone-picture checks read of a draw list or a box's run: what draws,
/// which a Shape with no paints doesn't, whatever `Shape::will_draw` says. A
/// file's picture draws one, its opacity and visibility root, before its image.
#[derive(Default)]
struct LoneContent {
    drawn: usize,
    image: bool,
}

impl LoneContent {
    fn add(&mut self, drawable: &RuntimeDrawableOccurrence) {
        if !is_paintless_shape(drawable) {
            self.drawn += 1;
            self.image = is_image(drawable);
        }
    }

    fn is_lone_picture(&self) -> bool {
        self.drawn == 1 && self.image
    }
}

/// A Shape with a paint counts as a draw whatever the paint's strength.
fn is_paintless_shape(drawable: &RuntimeDrawableOccurrence) -> bool {
    drawable.authored_handle().is_some_and(|handle| {
        handle
            .with_downcast::<Shape, _>(|shape| shape.paint_container.shape_paints().is_empty())
            .unwrap_or(false)
    })
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
    content: LoneContent,
}

impl DrawList {
    fn of(artboard: &Artboard) -> Self {
        let mut list = Self {
            first: None,
            last: None,
            count: 0,
            content: LoneContent::default(),
        };
        let mut current = artboard.first_drawable();
        while let Some(drawable) = current {
            current = drawable.with(Drawable::prev_drawable).flatten();
            if drawable.is_clip_start() || drawable.is_clip_end() {
                continue;
            }
            list.count += 1;
            list.content.add(&drawable);
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

    /// Ruling U13's check: in a test build, a Screen's draw fails when where
    /// its entries draw changed and nothing reported the change. A Screen
    /// held at 0.5 draws three times with nothing changed, which the check
    /// passes however the draw happens to read its list. Then its box's mark
    /// is written to 2 through the record's import read, which reports
    /// nothing, as an input that forgets to announce its change would: the
    /// box now draws outside the picture at its own strength, and the
    /// Screen's change serial, which moves with its content revision, hasn't
    /// moved. The next draw panics, saying the split changed "with no
    /// report". A build without the check draws it.
    #[test]
    fn a_split_change_with_no_report_fails_the_draw() {
        use crate::group_opacity_fixture::{
            Block, Item, Page, Unit, bind_page, box_record, screen_row, set_row_opacity,
        };
        use crate::source::{
            core::binary_reader::BinaryReader, generated::core_registry::CoreRegistry,
        };
        use crate::{File, RuntimeFactoryHandle};
        use nuxie_render_api::{NullRenderer, PersistentFactory, RecordingFactory};
        use std::panic::{AssertUnwindSafe, catch_unwind};

        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &Page::one(
                (300.0, 300.0),
                Unit {
                    background: Some(0xff3c_3c3c),
                    items: vec![Item::Block(Block {
                        name: Some("A"),
                        at: (40.0, 40.0),
                        size: (160.0, 160.0),
                        color: Some(0xffc8_c8c8),
                        ..Block::default()
                    })],
                    ..Unit::css(300.0, 300.0)
                },
            )
            .bytes(),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let page = file.with_file(File::artboard_default).unwrap();
        bind_page(&file, &page);
        let screen = screen_row(&page, "Screen");
        set_row_opacity(&screen, 0.5);
        for _ in 0..3 {
            page.advance_default(1.0 / 60.0);
        }
        let draw = || page.draw(&mut NullRenderer);
        for _ in 0..3 {
            draw();
        }
        let record = box_record(&screen, "A");
        let serial = || screen.with_artboard(|artboard| artboard.change_serial());
        let before = serial();
        let bytes = 2.0f32.to_le_bytes();
        assert_eq!(
            record.with_mut(|object| object.deserialize(FADE_OUTSIDE, &mut BinaryReader::new(&bytes))),
            Some(true),
            "the record reads the mark"
        );
        assert_eq!(
            CoreRegistry::get_double_handle(&record, i32::from(FADE_OUTSIDE)),
            Some(2.0),
            "the record holds the mark"
        );
        assert_eq!(serial(), before, "the import read reports nothing");
        let drawn = catch_unwind(AssertUnwindSafe(draw));
        let Err(panic) = drawn else {
            panic!("the draw after a split change with no report passed the check");
        };
        let message = panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| panic.downcast_ref::<&str>().copied())
            .unwrap_or_default();
        assert!(
            message.contains("with no report"),
            "the check fails the draw, not something else: {message:?}"
        );
    }
}
