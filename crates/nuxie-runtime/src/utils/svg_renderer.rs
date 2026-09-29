//! Translation of upstream include/utils/svg_renderer.hpp and utils/svg_renderer.cpp.

use super::svg_factory::{SVGRenderImage, SVGRenderPaint, SVGRenderPath, format_float};
use crate::source::math::mat2d::Mat2D as SourceMat2D;
use nuxie_render_api::{
    BlendMode, FillRule, ImageSampler, Mat2D, RenderBuffer, RenderImage, RenderPaint, RenderPath,
    Renderer, StrokeCap, StrokeJoin,
};
use std::{collections::HashMap, fmt::Write, sync::Once};

const EPSILON: f32 = 1e-6;

struct Item {
    body: String,
    blend: BlendMode,
    is_draw: bool,
    clip_open_id: i32,
    single_clip_id: i32,
    clip_open_len: usize,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            body: String::new(),
            blend: BlendMode::SrcOver,
            is_draw: false,
            clip_open_id: -1,
            single_clip_id: -1,
            clip_open_len: 0,
        }
    }
}

struct StackEntry {
    matrix: SourceMat2D,
    opacity: f32,
    open_clip_groups: i32,
    items: Vec<Item>,
}

impl Default for StackEntry {
    fn default() -> Self {
        Self {
            matrix: SourceMat2D::identity(),
            opacity: 1.0,
            open_clip_groups: 0,
            items: Vec::new(),
        }
    }
}

pub struct SVGRenderer {
    float_precision: i32,
    stack: Vec<StackEntry>,
    defs: String,
    clip_def_ids: HashMap<String, i32>,
    clip_id_counter: i32,
    gradient_id_counter: i32,
}

impl Default for SVGRenderer {
    fn default() -> Self {
        Self::new(Self::K_DEFAULT_FLOAT_PRECISION)
    }
}

impl SVGRenderer {
    pub const K_DEFAULT_FLOAT_PRECISION: i32 = 4;

    pub fn new(float_precision: i32) -> Self {
        Self {
            float_precision,
            stack: vec![StackEntry::default()],
            defs: String::new(),
            clip_def_ids: HashMap::new(),
            clip_id_counter: 0,
            gradient_id_counter: 0,
        }
    }

    fn write_transform(out: &mut String, matrix: SourceMat2D, precision: i32) {
        let m = matrix.values();
        let identity_linear = (m[0] - 1.0).abs() < EPSILON
            && m[1].abs() < EPSILON
            && m[2].abs() < EPSILON
            && (m[3] - 1.0).abs() < EPSILON;
        let zero_translation = m[4].abs() < EPSILON && m[5].abs() < EPSILON;
        if identity_linear && zero_translation {
            return;
        }
        if identity_linear {
            write!(
                out,
                " transform=\"translate({} {})\"",
                format_float(m[4], precision),
                format_float(m[5], precision)
            )
            .unwrap();
        } else {
            write!(
                out,
                " transform=\"matrix({} {} {} {} {} {})\"",
                format_float(m[0], precision),
                format_float(m[1], precision),
                format_float(m[2], precision),
                format_float(m[3], precision),
                format_float(m[4], precision),
                format_float(m[5], precision)
            )
            .unwrap();
        }
    }

    fn emit_paint_attributes(
        &mut self,
        out: &mut String,
        paint: &SVGRenderPaint,
        path: &SVGRenderPath,
        extra_opacity: f32,
    ) {
        let color = paint.get_color();
        let r = (color >> 16) & 255;
        let g = (color >> 8) & 255;
        let b = color & 255;
        let alpha = (((color >> 24) & 255) as f32 / 255.0) * extra_opacity;
        let shader = paint.get_svg_shader();
        if paint.is_stroke() {
            out.push_str(" fill=\"none\"");
            if let Some(shader) = shader {
                let id = format!("grad{}", self.gradient_id_counter);
                self.gradient_id_counter += 1;
                shader.emit_defs(&mut self.defs, &id, self.float_precision);
                write!(out, " stroke=\"url(#{id})\"").unwrap();
            } else {
                write!(out, " stroke=\"#{r:02x}{g:02x}{b:02x}\"").unwrap();
            }
            if alpha < 1.0 - EPSILON {
                write!(
                    out,
                    " stroke-opacity=\"{}\"",
                    format_float(alpha, self.float_precision)
                )
                .unwrap();
            }
            write!(
                out,
                " stroke-width=\"{}\"",
                format_float(paint.get_thickness(), self.float_precision)
            )
            .unwrap();
            match paint.get_join() {
                StrokeJoin::Miter => {}
                StrokeJoin::Round => out.push_str(" stroke-linejoin=\"round\""),
                StrokeJoin::Bevel => out.push_str(" stroke-linejoin=\"bevel\""),
            }
            match paint.get_cap() {
                StrokeCap::Butt => {}
                StrokeCap::Round => out.push_str(" stroke-linecap=\"round\""),
                StrokeCap::Square => out.push_str(" stroke-linecap=\"square\""),
            }
        } else {
            if let Some(shader) = shader {
                let id = format!("grad{}", self.gradient_id_counter);
                self.gradient_id_counter += 1;
                shader.emit_defs(&mut self.defs, &id, self.float_precision);
                write!(out, " fill=\"url(#{id})\"").unwrap();
            } else if r != 0 || g != 0 || b != 0 {
                write!(out, " fill=\"#{r:02x}{g:02x}{b:02x}\"").unwrap();
            }
            if alpha < 1.0 - EPSILON {
                write!(
                    out,
                    " fill-opacity=\"{}\"",
                    format_float(alpha, self.float_precision)
                )
                .unwrap();
            }
            if path.get_fill_rule() == FillRule::EvenOdd {
                out.push_str(" fill-rule=\"evenodd\"");
            }
        }
    }

    fn wants_blend_wrap(entry: &StackEntry) -> bool {
        let mut first_blend = BlendMode::SrcOver;
        let mut count = 0;
        for item in &entry.items {
            if !item.is_draw {
                continue;
            }
            if count == 0 {
                first_blend = item.blend;
            } else if item.blend != first_blend {
                return false;
            }
            count += 1;
        }
        count > 1 && first_blend != BlendMode::SrcOver
    }

    fn write_entry(dst: &mut String, entry: &StackEntry) {
        if entry.items.is_empty() {
            return;
        }
        let wrap = Self::wants_blend_wrap(entry);
        if wrap {
            let first = entry.items.iter().find(|item| item.is_draw).unwrap();
            writeln!(dst, "<g{}>", blend_style(first.blend)).unwrap();
        }
        for (i, item) in entry.items.iter().enumerate() {
            if !item.is_draw || wrap || item.blend == BlendMode::SrcOver {
                let merge_prev = item.single_clip_id >= 0
                    && i > 0
                    && entry.items[i - 1].single_clip_id == item.single_clip_id;
                let merge_next = item.single_clip_id >= 0
                    && i + 1 < entry.items.len()
                    && entry.items[i + 1].single_clip_id == item.single_clip_id;
                let begin = if merge_prev { item.clip_open_len } else { 0 };
                let end = item.body.len() - if merge_next { "</g>\n".len() } else { 0 };
                dst.push_str(&item.body[begin..end]);
            } else {
                let mut body = item.body.clone();
                let style = blend_style(item.blend);
                if let Some(close) = body.rfind("/>") {
                    if !style.is_empty() {
                        body.insert_str(close, &style);
                    }
                }
                dst.push_str(&body);
            }
        }
        if wrap {
            dst.push_str("</g>\n");
        }
    }

    pub fn finalize(&self, width: i32, height: i32) -> String {
        let mut svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\" viewBox=\"0 0 {width} {height}\" width=\"{width}\" height=\"{height}\">\n"
        );
        if !self.defs.is_empty() {
            write!(svg, "<defs>\n{}</defs>\n", self.defs).unwrap();
        }
        if let Some(root) = self.stack.first() {
            Self::write_entry(&mut svg, root);
        }
        svg.push_str("</svg>\n");
        svg
    }
}

impl Renderer for SVGRenderer {
    fn save(&mut self) {
        let top = self.stack.last().unwrap();
        let next = StackEntry {
            matrix: top.matrix,
            opacity: top.opacity,
            ..Default::default()
        };
        self.stack.push(next);
    }

    fn restore(&mut self) {
        if self.stack.len() <= 1 {
            return;
        }
        let mut entry = self.stack.pop().unwrap();
        for _ in 0..entry.open_clip_groups {
            entry.items.push(Item {
                body: "</g>\n".into(),
                ..Default::default()
            });
        }
        let mut body = String::new();
        Self::write_entry(&mut body, &entry);
        let mut passthrough = Item {
            body,
            ..Default::default()
        };
        if entry.open_clip_groups >= 1
            && !entry.items.is_empty()
            && entry.items[0].clip_open_id >= 0
            && !Self::wants_blend_wrap(&entry)
        {
            passthrough.single_clip_id = entry.items[0].clip_open_id;
            passthrough.clip_open_len = entry.items[0].body.len();
        }
        self.stack.last_mut().unwrap().items.push(passthrough);
    }

    fn transform(&mut self, transform: Mat2D) {
        let [a, b, c, d, e, f] = transform.0;
        self.stack.last_mut().unwrap().matrix *= SourceMat2D::new(a, b, c, d, e, f);
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.stack.last_mut().unwrap().opacity *= opacity;
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        let path = path
            .as_any()
            .downcast_ref::<SVGRenderPath>()
            .expect("SVGRenderPath");
        let paint = paint
            .as_any()
            .downcast_ref::<SVGRenderPaint>()
            .expect("SVGRenderPaint");
        let d = path.to_svg_d(self.float_precision);
        if d.is_empty() {
            return;
        }
        let top = self.stack.last().unwrap();
        let (matrix, opacity) = (top.matrix, top.opacity);
        let mut body = String::from("<path");
        Self::write_transform(&mut body, matrix, self.float_precision);
        self.emit_paint_attributes(&mut body, paint, path, opacity);
        writeln!(body, " d=\"{d}\"/>").unwrap();
        self.stack.last_mut().unwrap().items.push(Item {
            body,
            blend: paint.get_blend_mode(),
            is_draw: true,
            ..Default::default()
        });
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        let path = path
            .as_any()
            .downcast_ref::<SVGRenderPath>()
            .expect("SVGRenderPath");
        let d = path.to_svg_d(self.float_precision);
        if d.is_empty() {
            return;
        }
        let top = self.stack.last_mut().unwrap();
        let mut def_path = String::from("<path");
        Self::write_transform(&mut def_path, top.matrix, self.float_precision);
        write!(def_path, " d=\"{d}\"").unwrap();
        if path.get_fill_rule() == FillRule::EvenOdd {
            def_path.push_str(" clip-rule=\"evenodd\"");
        }
        def_path.push_str("/>");
        let clip_id = if let Some(id) = self.clip_def_ids.get(&def_path) {
            *id
        } else {
            let id = self.clip_id_counter;
            self.clip_id_counter += 1;
            writeln!(self.defs, "<clipPath id=\"clip{id}\">{def_path}</clipPath>").unwrap();
            self.clip_def_ids.insert(def_path, id);
            id
        };
        top.items.push(Item {
            body: format!("<g clip-path=\"url(#clip{clip_id})\">\n"),
            clip_open_id: clip_id,
            ..Default::default()
        });
        top.open_clip_groups += 1;
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        _: ImageSampler,
        blend: BlendMode,
        opacity: f32,
    ) {
        let image = image.expect("SVGRenderImage");
        let svg_image = image
            .as_any()
            .downcast_ref::<SVGRenderImage>()
            .expect("SVGRenderImage");
        let top = self.stack.last_mut().unwrap();
        let alpha = top.opacity * opacity;
        let mut body = format!(
            "<image width=\"{}\" height=\"{}\" href=\"{}\"",
            image.width(),
            image.height(),
            svg_image.to_data_uri()
        );
        Self::write_transform(&mut body, top.matrix, self.float_precision);
        if alpha < 1.0 - EPSILON {
            write!(
                body,
                " opacity=\"{}\"",
                format_float(alpha, self.float_precision)
            )
            .unwrap();
        }
        body.push_str("/>\n");
        top.items.push(Item {
            body,
            blend,
            is_draw: true,
            ..Default::default()
        });
    }

    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: u32,
        _: u32,
        _: BlendMode,
        _: f32,
    ) {
        static WARNED: Once = Once::new();
        WARNED.call_once(|| {
            eprintln!(
                "SVGRenderer: drawImageMesh is not supported in SVG output and will be skipped."
            )
        });
    }
}

fn blend_style(mode: BlendMode) -> String {
    let css = match mode {
        BlendMode::SrcOver => return String::new(),
        BlendMode::Screen => "screen",
        BlendMode::Overlay => "overlay",
        BlendMode::Darken => "darken",
        BlendMode::Lighten => "lighten",
        BlendMode::ColorDodge => "color-dodge",
        BlendMode::ColorBurn => "color-burn",
        BlendMode::HardLight => "hard-light",
        BlendMode::SoftLight => "soft-light",
        BlendMode::Difference => "difference",
        BlendMode::Exclusion => "exclusion",
        BlendMode::Multiply => "multiply",
        BlendMode::Hue => "hue",
        BlendMode::Saturation => "saturation",
        BlendMode::Color => "color",
        BlendMode::Luminosity => "luminosity",
    };
    format!(" style=\"mix-blend-mode:{css}\"")
}
