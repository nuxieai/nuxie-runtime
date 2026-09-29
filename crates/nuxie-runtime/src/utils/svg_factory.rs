//! Translation of upstream include/utils/svg_factory.hpp and utils/svg_factory.cpp.
use nuxie_render_api::*;
use std::{any::Any, fmt::Write, rc::Rc};

// C++ defaultfloat uses significant digits, switching to scientific notation
// below exponent -4 or at the precision, and removes trailing zeroes.
pub(super) fn format_float(value: f32, precision: i32) -> String {
    if !value.is_finite() {
        if value.is_nan() && value.is_sign_negative() {
            return "-nan".into();
        }
        return value.to_string().to_lowercase();
    }
    // A negative printf/defaultfloat precision uses the default six digits;
    // zero precision is treated as one significant digit.
    let precision = if precision < 0 { 6 } else { precision.max(1) } as usize;
    let scientific = format!("{:.*e}", precision - 1, f64::from(value));
    let (mantissa, exponent) = scientific.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    let trim = |s: String| {
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_owned()
        } else {
            s
        }
    };
    if exponent < -4 || exponent >= precision as i32 {
        format!(
            "{}e{}{:02}",
            trim(mantissa.to_owned()),
            if exponent < 0 { '-' } else { '+' },
            exponent.unsigned_abs()
        )
    } else {
        trim(format!(
            "{:.*}",
            (precision as i32 - exponent - 1).max(0) as usize,
            f64::from(value)
        ))
    }
}

#[cfg(test)]
mod formatting_tests {
    use super::format_float;

    #[test]
    fn defaultfloat_precision_and_special_values() {
        assert_eq!(format_float(1.234567, -1), "1.23457");
        assert_eq!(format_float(1.234567, 0), "1");
        assert_eq!(format_float(12.34567, 4), "12.35");
        assert_eq!(format_float(0.00001, 4), "1e-05");
        assert_eq!(format_float(10000.0, 4), "1e+04");
        assert_eq!(format_float(-0.0, 4), "-0");
        assert_eq!(format_float(f32::from_bits(0xffc00000), 4), "-nan");
        assert_eq!(format_float(f32::INFINITY, 4), "inf");
    }
}

pub struct SVGRenderPath {
    raw_path: RawPath,
    fill_rule: FillRule,
}
impl Default for SVGRenderPath {
    fn default() -> Self {
        Self::new(RawPath::default(), FillRule::NonZero)
    }
}
impl SVGRenderPath {
    pub fn new(raw_path: RawPath, fill_rule: FillRule) -> Self {
        Self {
            raw_path,
            fill_rule,
        }
    }
    pub fn get_fill_rule(&self) -> FillRule {
        self.fill_rule
    }
    pub fn to_svg_d(&self, precision: i32) -> String {
        let mut out = String::new();
        let mut point = 0;
        let verbs = self.raw_path.verbs();
        for (index, verb) in verbs.iter().enumerate() {
            let (letter, count) = match verb {
                PathVerb::Move => ('M', 1),
                PathVerb::Line => ('L', 1),
                PathVerb::Quad => ('Q', 2),
                PathVerb::Cubic => ('C', 3),
                PathVerb::Close => ('Z', 0),
            };
            out.push(letter);
            for offset in 0..count {
                if offset != 0 {
                    out.push(' ');
                }
                let p = self.raw_path.points()[point + offset];
                write!(
                    out,
                    "{} {}",
                    format_float(p.x, precision),
                    format_float(p.y, precision)
                )
                .unwrap();
            }
            if *verb == PathVerb::Move
                && (index + 1 == verbs.len() || verbs[index + 1] == PathVerb::Move)
            {
                out.push('Z');
            }
            point += count;
        }
        out
    }
}
impl RenderPath for SVGRenderPath {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn rewind(&mut self) {
        self.raw_path.rewind();
    }
    fn fill_rule(&mut self, value: FillRule) {
        self.fill_rule = value;
    }
    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        self.raw_path.add_path_with_transform(
            &path
                .as_any()
                .downcast_ref::<Self>()
                .expect("SVG path")
                .raw_path,
            transform,
        );
    }
    fn add_render_path_self(&mut self, transform: Mat2D) {
        let source = self.raw_path.clone();
        self.raw_path.add_path_with_transform(&source, transform);
    }
    fn add_raw_path(&mut self, path: &RawPath) {
        self.raw_path.add_path(path, Mat2D::IDENTITY);
    }
    fn move_to(&mut self, x: f32, y: f32) {
        self.raw_path.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.raw_path.inject_implicit_move_if_needed_for_owner();
        self.raw_path.line(Vec2D::new(x, y));
    }
    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.raw_path.inject_implicit_move_if_needed_for_owner();
        self.raw_path
            .cubic(Vec2D::new(ox, oy), Vec2D::new(ix, iy), Vec2D::new(x, y));
    }
    fn close(&mut self) {
        self.raw_path.close();
    }
}

pub trait SVGRenderShader: RenderShader {
    fn emit_defs(&self, out: &mut String, id: &str, precision: i32);
}
#[derive(Clone)]
pub struct SVGLinearGradientShader {
    data: Rc<(f32, f32, f32, f32, Vec<ColorInt>, Vec<f32>)>,
}
#[derive(Clone)]
pub struct SVGRadialGradientShader {
    data: Rc<(f32, f32, f32, Vec<ColorInt>, Vec<f32>)>,
}
impl SVGLinearGradientShader {
    pub fn new(sx: f32, sy: f32, ex: f32, ey: f32, colors: &[ColorInt], stops: &[f32]) -> Self {
        assert_eq!(colors.len(), stops.len());
        Self {
            data: Rc::new((sx, sy, ex, ey, colors.to_vec(), stops.to_vec())),
        }
    }
}
impl SVGRadialGradientShader {
    pub fn new(cx: f32, cy: f32, radius: f32, colors: &[ColorInt], stops: &[f32]) -> Self {
        assert_eq!(colors.len(), stops.len());
        Self {
            data: Rc::new((cx, cy, radius, colors.to_vec(), stops.to_vec())),
        }
    }
}
macro_rules! shader_object {
    ($ty:ty) => {
        impl RenderShader for $ty {
            fn as_any(&self) -> &dyn Any {
                self
            }
            fn retain_shader(&self) -> Rc<dyn RenderShader> {
                Rc::new(self.clone())
            }
            fn shader_identity(&self) -> usize {
                Rc::as_ptr(&self.data) as usize
            }
        }
    };
}
shader_object!(SVGLinearGradientShader);
shader_object!(SVGRadialGradientShader);
fn emit_color_stop(out: &mut String, color: ColorInt, offset: f32, precision: i32) {
    writeln!(
        out,
        "<stop offset=\"{}\" stop-color=\"rgb({},{},{})\" stop-opacity=\"{}\"/>",
        format_float(offset, precision),
        (color >> 16) & 255,
        (color >> 8) & 255,
        color & 255,
        format_float((color >> 24) as f32 / 255.0, precision)
    )
    .unwrap();
}
impl SVGRenderShader for SVGLinearGradientShader {
    fn emit_defs(&self, out: &mut String, id: &str, precision: i32) {
        let (sx, sy, ex, ey, colors, stops) = &*self.data;
        writeln!(out,"<linearGradient id=\"{}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" gradientUnits=\"userSpaceOnUse\">",id,format_float(*sx,precision),format_float(*sy,precision),format_float(*ex,precision),format_float(*ey,precision)).unwrap();
        for (color, stop) in colors.iter().zip(stops) {
            emit_color_stop(out, *color, *stop, precision);
        }
        out.push_str("</linearGradient>\n");
    }
}
impl SVGRenderShader for SVGRadialGradientShader {
    fn emit_defs(&self, out: &mut String, id: &str, precision: i32) {
        let (cx, cy, radius, colors, stops) = &*self.data;
        writeln!(out,"<radialGradient id=\"{}\" cx=\"{}\" cy=\"{}\" r=\"{}\" gradientUnits=\"userSpaceOnUse\">",id,format_float(*cx,precision),format_float(*cy,precision),format_float(*radius,precision)).unwrap();
        for (color, stop) in colors.iter().zip(stops) {
            emit_color_stop(out, *color, *stop, precision);
        }
        out.push_str("</radialGradient>\n");
    }
}

pub struct SVGRenderPaint {
    is_stroke: bool,
    color: ColorInt,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    blend_mode: BlendMode,
    shader: Option<Rc<dyn RenderShader>>,
}
impl Default for SVGRenderPaint {
    fn default() -> Self {
        Self {
            is_stroke: false,
            color: 0xff000000,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            blend_mode: BlendMode::SrcOver,
            shader: None,
        }
    }
}
impl SVGRenderPaint {
    pub fn is_stroke(&self) -> bool {
        self.is_stroke
    }
    pub fn get_color(&self) -> ColorInt {
        self.color
    }
    pub fn get_thickness(&self) -> f32 {
        self.thickness
    }
    pub fn get_join(&self) -> StrokeJoin {
        self.join
    }
    pub fn get_cap(&self) -> StrokeCap {
        self.cap
    }
    pub fn get_blend_mode(&self) -> BlendMode {
        self.blend_mode
    }
    pub fn get_svg_shader(&self) -> Option<&dyn SVGRenderShader> {
        self.shader.as_ref().map(|shader| {
            if let Some(shader) = shader.as_any().downcast_ref::<SVGLinearGradientShader>() {
                shader as &dyn SVGRenderShader
            } else {
                shader
                    .as_any()
                    .downcast_ref::<SVGRadialGradientShader>()
                    .expect("SVG shader") as &dyn SVGRenderShader
            }
        })
    }
}
impl RenderPaint for SVGRenderPaint {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn style(&mut self, value: RenderPaintStyle) {
        self.is_stroke = value == RenderPaintStyle::Stroke;
    }
    fn color(&mut self, value: ColorInt) {
        self.color = value;
    }
    fn thickness(&mut self, value: f32) {
        self.thickness = value;
    }
    fn join(&mut self, value: StrokeJoin) {
        self.join = value;
    }
    fn cap(&mut self, value: StrokeCap) {
        self.cap = value;
    }
    fn blend_mode(&mut self, value: BlendMode) {
        self.blend_mode = value;
    }
    fn shader(&mut self, value: Option<&dyn RenderShader>) {
        self.shader = value.map(RenderShader::retain_shader);
    }
    fn invalidate_stroke(&mut self) {}
    fn feather(&mut self, _value: f32) {}
}

#[derive(Clone)]
pub struct SVGRenderImage {
    data: Rc<(Vec<u8>, u32, u32)>,
}
impl SVGRenderImage {
    pub fn new(bytes: &[u8], width: u32, height: u32) -> Self {
        Self {
            data: Rc::new((bytes.to_vec(), width, height)),
        }
    }
    pub fn to_data_uri(&self) -> String {
        let bytes = &self.data.0;
        let mut mime = "application/octet-stream";
        if bytes.len() >= 8 {
            if bytes[0] == 0x89 && bytes[1] == b'P' {
                mime = "image/png";
            } else if bytes[0] == 0xff && bytes[1] == 0xd8 {
                mime = "image/jpeg";
            } else if &bytes[..4] == b"RIFF" {
                mime = "image/webp";
            }
        }
        let mut out = format!("data:{mime};base64,");
        const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        for chunk in bytes.chunks(3) {
            let n = ((chunk[0] as u32) << 16)
                | ((chunk.get(1).copied().unwrap_or(0) as u32) << 8)
                | chunk.get(2).copied().unwrap_or(0) as u32;
            out.push(TABLE[((n >> 18) & 63) as usize] as char);
            out.push(TABLE[((n >> 12) & 63) as usize] as char);
            out.push(if chunk.len() > 1 {
                TABLE[((n >> 6) & 63) as usize] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                TABLE[(n & 63) as usize] as char
            } else {
                '='
            });
        }
        out
    }
}
impl RenderImage for SVGRenderImage {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn retain_image(&self) -> Rc<dyn RenderImage> {
        Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Rc::as_ptr(&self.data) as usize
    }
    fn width(&self) -> u32 {
        self.data.1
    }
    fn height(&self) -> u32 {
        self.data.2
    }
}
pub struct SVGRenderBuffer {
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
}
impl SVGRenderBuffer {
    pub fn new(buffer_type: RenderBufferType, flags: RenderBufferFlags, size: usize) -> Self {
        Self {
            buffer_type,
            flags,
            bytes: vec![0; size],
        }
    }
}
impl RenderBuffer for SVGRenderBuffer {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn buffer_type(&self) -> RenderBufferType {
        self.buffer_type
    }
    fn flags(&self) -> RenderBufferFlags {
        self.flags
    }
    fn size_in_bytes(&self) -> usize {
        self.bytes.len()
    }
    fn map_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
    fn unmap(&mut self) {}
}
pub type DecodeImageSize = fn(&[u8], &mut u32, &mut u32) -> bool;
#[derive(Default)]
pub struct SVGFactory {
    decode_image_size: Option<DecodeImageSize>,
}
impl SVGFactory {
    pub fn new(decode_image_size: Option<DecodeImageSize>) -> Self {
        Self { decode_image_size }
    }
}
impl Factory for SVGFactory {
    fn make_render_buffer(
        &mut self,
        ty: RenderBufferType,
        flags: RenderBufferFlags,
        size: usize,
    ) -> Box<dyn RenderBuffer> {
        Box::new(SVGRenderBuffer::new(ty, flags, size))
    }
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        Box::new(SVGLinearGradientShader::new(sx, sy, ex, ey, colors, stops))
    }
    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        Box::new(SVGRadialGradientShader::new(cx, cy, radius, colors, stops))
    }
    fn make_render_path(&mut self, path: RawPath, rule: FillRule) -> Box<dyn RenderPath> {
        Box::new(SVGRenderPath::new(path, rule))
    }
    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        Box::new(SVGRenderPath::default())
    }
    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        Box::new(SVGRenderPaint::default())
    }
    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        let (mut width, mut height) = (0, 0);
        if !self
            .decode_image_size
            .is_some_and(|decode| decode(data, &mut width, &mut height))
        {
            return Err(ImageDecodeError);
        }
        Ok(Box::new(SVGRenderImage::new(data, width, height)))
    }
}
