//! A hosted CSS element that fades draws as one picture
//! (docs/CSS_LAYOUT.md, 4.4). Rive has no save layer, so opacity multiplies
//! down the tree and each paint takes it alone. CSS says an element and its
//! descendants composite as one, so a CSS artboard draws whole into a canvas
//! and the canvas goes down at the fade, once. Rive-made content keeps
//! upstream's per-paint fade.
use crate::source::{
    artboard::Artboard,
    bitmap_cache::BitmapCache,
    core::CoreHandle,
    drawable::{BoundsFidelity, Drawable, RuntimeDrawableOccurrence},
    generated::shapes::image_base::ImageBase,
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

/// A hosted artboard's held picture: its canvas, and how many draws in a row
/// have needed less than it.
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

    /// Draws `hosted` into this picture and composites it at `held`. None
    /// leaves `renderer` untouched and the fade still to apply.
    fn draw(&mut self, hosted: &CoreHandle, renderer: &mut Renderer, held: f32) -> Option<()> {
        let [xx, xy, yx, yy, tx, ty] = renderer.current_transform()?.0;
        let transform = Mat2D::new(xx, xy, yx, yy, tx, ty);
        let mut inverse = Mat2D::default();
        if !transform.invert(&mut inverse) {
            return None;
        }
        let factory = hosted
            .with_downcast::<Artboard, _>(Artboard::factory)
            .flatten()?;
        let host = factory.with_factory_mut(|factory| factory.canvas_content_host())?;
        if host.borrow().composite_drops_clips() {
            return None;
        }
        let Term::Area(area) = box_term(hosted) else {
            return None;
        };
        let content = hosted.with_downcast::<Artboard, _>(Artboard::content_transform)?;
        let (left, top, need) = device_pixels((transform * content).map_bounding_box(area))?;
        let size = self.fit(need, &host)?;
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
            let mut scope = CanvasContentScope::new(Some(&host), Some(&canvas), &plan, 0);
            let content = scope.renderer()?;
            content.transform(nuxie_render_api::Mat2D(*transform.values()));
            Artboard::draw_content_with_visitor_handle(hosted, content, None);
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

/// A device box rounded out to whole pixels with its guard: its corner, and
/// its size. A fractional corner would resample the picture.
fn device_pixels(area: Aabb) -> Option<(f32, f32, (u32, u32))> {
    let left = area.left().floor() - GUARD_PIXELS;
    let top = area.top().floor() - GUARD_PIXELS;
    let width = area.right().ceil() + GUARD_PIXELS - left;
    let height = area.bottom().ceil() + GUARD_PIXELS - top;
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
    // Out of the artboard for the draw: the renderer can reenter its owner.
    let Some(mut picture) = hosted
        .with_downcast_mut::<Artboard, _>(|artboard| std::mem::take(&mut artboard.group_picture))
    else {
        return false;
    };
    let drew = picture.draw(hosted, renderer, held).is_some();
    hosted.with_downcast_mut::<Artboard, _>(|artboard| artboard.group_picture = picture);
    drew
}

fn release(hosted: &CoreHandle) {
    hosted.with_downcast_mut::<Artboard, _>(|artboard| artboard.group_picture.release());
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
/// that's Hero's image copy. A video draws as an image but plays live, so it
/// isn't one. Read at the draw, where the sorted list is current.
fn is_lone_picture(artboard: &CoreHandle) -> bool {
    let Some(list) = artboard.with_downcast::<Artboard, _>(DrawList::of) else {
        return false;
    };
    list.count == 1
        && list
            .first
            .and_then(|drawable| drawable.authored_handle())
            .is_some_and(|image| {
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
        if !holds_nothing && layout.clipped_axes() != (true, true) {
            return None;
        }
        let mut painted = Aabb::default();
        let fidelity = layout.painted_world_bounds(&mut painted);
        let own = Term::painted(fidelity, painted);
        if holds_nothing {
            return Some(own);
        }
        let paths = layout.render_paths_handle()?;
        let clip = paths.borrow().clip.raw_path().bounds();
        Some(own.joined(Term::of(clip)))
    })
    .flatten()
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
