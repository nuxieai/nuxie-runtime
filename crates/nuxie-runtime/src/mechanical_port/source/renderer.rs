use std::rc::Rc;

use crate::mechanical_port::source::{
    layout::{Alignment, Fit},
    math::{aabb::Aabb, mat2d::Mat2D, path_types::PathVerb, raw_path::RawPath},
};

pub use nuxie_render_api::{
    BlendMode, ColorInt, FillRule, ImageFilter, ImageSampler, ImageWrap, RenderBufferFlags,
    RenderBufferType, RenderPaintStyle, StrokeCap, StrokeJoin,
};

/// The renderer-facing virtual owners are the existing `nuxie_render_api`
/// traits. These aliases name trait objects; they do not introduce a second
/// renderer identity or ownership model.
pub type RenderBuffer = dyn nuxie_render_api::RenderBuffer;
pub type RenderImage = dyn nuxie_render_api::RenderImage;
pub type RenderPaint = dyn nuxie_render_api::RenderPaint;
pub type RenderPath = dyn nuxie_render_api::RenderPath;
pub type Renderer = dyn nuxie_render_api::Renderer;
pub type RenderShader = dyn nuxie_render_api::RenderShader;

/// Shared renderer-owned image occurrence used at the runtime/host seam.
pub type RenderImageRef = Rc<RenderImage>;

pub fn to_render_raw_path(path: &RawPath) -> nuxie_render_api::RawPath {
    let mut result = nuxie_render_api::RawPath::new();
    let mut point_index = 0;
    for verb in path.verbs() {
        match verb {
            PathVerb::Move => {
                let point = path.points()[point_index];
                point_index += 1;
                result.move_to(point.x, point.y);
            }
            PathVerb::Line => {
                let point = path.points()[point_index];
                point_index += 1;
                result.line_to(point.x, point.y);
            }
            PathVerb::Quad => {
                let control = path.points()[point_index];
                let point = path.points()[point_index + 1];
                point_index += 2;
                result.quad_to(control.x, control.y, point.x, point.y);
            }
            PathVerb::Cubic => {
                let out = path.points()[point_index];
                let incoming = path.points()[point_index + 1];
                let point = path.points()[point_index + 2];
                point_index += 3;
                result.cubic_to(out.x, out.y, incoming.x, incoming.y, point.x, point.y);
            }
            PathVerb::Close => result.close(),
        }
    }
    result
}

/// Preserve every verb and control point across the approved renderer/VM DTO seam.
pub fn from_render_raw_path(path: &nuxie_render_api::RawPath) -> RawPath {
    let mut result = RawPath::default();
    let mut points = path.points().iter();
    for verb in path.verbs() {
        match verb {
            nuxie_render_api::PathVerb::Move => {
                let p = points.next().unwrap();
                result.move_to(p.x, p.y);
            }
            nuxie_render_api::PathVerb::Line => {
                let p = points.next().unwrap();
                result.line_to(p.x, p.y);
            }
            nuxie_render_api::PathVerb::Quad => {
                let c = points.next().unwrap();
                let p = points.next().unwrap();
                result.quad_to(c.x, c.y, p.x, p.y);
            }
            nuxie_render_api::PathVerb::Cubic => {
                let a = points.next().unwrap();
                let b = points.next().unwrap();
                let p = points.next().unwrap();
                result.cubic_to(a.x, a.y, b.x, b.y, p.x, p.y);
            }
            nuxie_render_api::PathVerb::Close => result.close(),
        }
    }
    result
}

/// Compute the exact pinned alignment matrix in the mechanical math domain.
/// Renderer call sites convert its six scalar lanes to
/// `nuxie_render_api::Mat2D` at the call boundary.
pub fn compute_alignment(
    fit: Fit,
    alignment: Alignment,
    frame: &Aabb,
    content: &Aabb,
    scale_factor: f32,
) -> Mat2D {
    let content_width = content.width();
    let content_height = content.height();
    let x = -content.left() - content_width * 0.5 - alignment.x() * content_width * 0.5;
    let y = -content.top() - content_height * 0.5 - alignment.y() * content_height * 0.5;

    let (scale_x, scale_y) = match fit {
        Fit::Fill => (
            frame.width() / content_width,
            frame.height() / content_height,
        ),
        Fit::Contain => {
            let scale = (frame.width() / content_width).min(frame.height() / content_height);
            (scale, scale)
        }
        Fit::Cover => {
            let scale = (frame.width() / content_width).max(frame.height() / content_height);
            (scale, scale)
        }
        Fit::FitHeight => {
            let scale = frame.height() / content_height;
            (scale, scale)
        }
        Fit::FitWidth => {
            let scale = frame.width() / content_width;
            (scale, scale)
        }
        Fit::Layout => (scale_factor, scale_factor),
        Fit::None => (1.0, 1.0),
        Fit::ScaleDown => {
            let scale = (frame.width() / content_width)
                .min(frame.height() / content_height)
                .min(1.0);
            (scale, scale)
        }
    };

    let translation = Mat2D::from_translate(
        frame.left() + frame.width() * 0.5 + alignment.x() * frame.width() * 0.5,
        frame.top() + frame.height() * 0.5 + alignment.y() * frame.height() * 0.5,
    );
    translation * Mat2D::from_scale(scale_x, scale_y) * Mat2D::from_translate(x, y)
}

pub fn is_white_space(character: u32) -> bool {
    character <= u32::from(b' ') || character == 0x2028 || character == 0x200b
}
