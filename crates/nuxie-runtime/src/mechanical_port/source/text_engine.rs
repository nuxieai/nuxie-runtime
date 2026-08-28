use std::any::Any;
use std::rc::Rc;

use crate::mechanical_port::source::math::raw_path::RawPath;
use crate::mechanical_port::source::math::vec2d::Vec2D;
pub use crate::mechanical_port::source::renderer::is_white_space;
use crate::mechanical_port::source::shapes::paint::color::ColorInt;
use crate::mechanical_port::source::text::glyph_lookup::GlyphLookup;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSizing {
    AutoWidth,
    AutoHeight,
    Fixed,
    Unknown(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextOverflow {
    Visible,
    Hidden,
    Clipped,
    Ellipsis,
    Fit,
    FitFontSize,
    Unknown(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextOrigin {
    Top,
    Baseline,
    Unknown(u32),
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextTrimTop(pub u8);

impl TextTrimTop {
    pub const None: Self = Self(0);
    pub const Cap: Self = Self(1);
    pub const Ex: Self = Self(2);
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextTrimBottom(pub u8);

impl TextTrimBottom {
    pub const None: Self = Self(0);
    pub const Alphabetic: Self = Self(1);
    pub const Text: Self = Self(2);
}

pub const TEXT_TRIM_TOP_SHIFT: u32 = 0;
pub const TEXT_TRIM_BOTTOM_SHIFT: u32 = 8;
pub const TEXT_TRIM_FIELD_MASK: u32 = 0xff;

pub fn text_trim_top(packed: u32) -> TextTrimTop {
    TextTrimTop(((packed >> TEXT_TRIM_TOP_SHIFT) & TEXT_TRIM_FIELD_MASK) as u8)
}

pub fn text_trim_bottom(packed: u32) -> TextTrimBottom {
    TextTrimBottom(((packed >> TEXT_TRIM_BOTTOM_SHIFT) & TEXT_TRIM_FIELD_MASK) as u8)
}

pub fn pack_text_vertical_trim(top: TextTrimTop, bottom: TextTrimBottom) -> u32 {
    ((top.0 as u32) << TEXT_TRIM_TOP_SHIFT) | ((bottom.0 as u32) << TEXT_TRIM_BOTTOM_SHIFT)
}

pub type Unichar = u32;
pub type GlyphId = u16;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDirection {
    Ltr = 0,
    Rtl = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Unknown(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWrap {
    Wrap,
    NoWrap,
    Unknown(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalTextAlign {
    Top,
    Bottom,
    Middle,
    Unknown(u32),
}

#[derive(Clone, Debug)]
pub struct GlyphLine {
    pub start_run_index: u32,
    pub start_glyph_index: u32,
    pub end_run_index: u32,
    pub end_glyph_index: u32,
    pub start_x: f32,
    pub top: f32,
    pub baseline: f32,
    pub bottom: f32,
}

impl PartialEq for GlyphLine {
    fn eq(&self, other: &Self) -> bool {
        self.start_run_index == other.start_run_index
            && self.start_glyph_index == other.start_glyph_index
            && self.end_run_index == other.end_run_index
            && self.end_glyph_index == other.end_glyph_index
    }
}

impl Default for GlyphLine {
    fn default() -> Self {
        Self {
            start_run_index: 0,
            start_glyph_index: 0,
            end_run_index: 0,
            end_glyph_index: 0,
            start_x: 0.0,
            top: 0.0,
            baseline: 0.0,
            bottom: 0.0,
        }
    }
}

impl GlyphLine {
    pub fn at(run: u32, index: u32) -> Self {
        Self {
            start_run_index: run,
            start_glyph_index: index,
            end_run_index: run,
            end_glyph_index: index,
            start_x: 0.0,
            top: 0.0,
            baseline: 0.0,
            bottom: 0.0,
        }
    }

    pub fn empty(&self) -> bool {
        self.start_run_index == self.end_run_index && self.start_glyph_index == self.end_glyph_index
    }
}

#[derive(Clone)]
pub struct Paragraph {
    pub runs: Vec<GlyphRun>,
    pub level: u8,
}

impl Paragraph {
    pub fn base_direction(&self) -> TextDirection {
        if self.level & 1 != 0 {
            TextDirection::Rtl
        } else {
            TextDirection::Ltr
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LineMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub cap_height: f32,
    pub x_height: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Axis {
    pub tag: u32,
    pub min: f32,
    pub def: f32,
    pub max: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Coord {
    pub axis: u32,
    pub value: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Feature {
    pub tag: u32,
    pub value: u32,
}

// Rust spellings for the nested C++ Font::Coord and Font::Feature records.
pub type FontCoord = Coord;
pub type FontFeature = Feature;

#[derive(Clone, Copy, Debug)]
pub struct GradientStop {
    pub offset: f32,
    pub color: ColorInt,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorGlyphPaintType {
    Solid,
    LinearGradient,
    RadialGradient,
    SweepGradient,
    Image,
}

#[derive(Clone)]
pub struct ColorGlyphLayer {
    pub path: RawPath,
    pub paint_type: ColorGlyphPaintType,
    pub color: ColorInt,
    pub use_foreground: bool,
    pub stops: Vec<GradientStop>,
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub r0: f32,
    pub r1: f32,
    pub start_angle: f32,
    pub end_angle: f32,
    pub image_bytes: Vec<u8>,
    pub image_width: u32,
    pub image_height: u32,
    pub image_bearing_x: f32,
    pub image_bearing_y: f32,
    pub image_extent_x: f32,
    pub image_extent_y: f32,
}

impl Default for ColorGlyphLayer {
    fn default() -> Self {
        Self {
            path: RawPath::default(),
            paint_type: ColorGlyphPaintType::Solid,
            color: 0xff000000,
            use_foreground: false,
            stops: Vec::new(),
            x0: 0.0,
            y0: 0.0,
            x1: 0.0,
            y1: 0.0,
            r0: 0.0,
            r1: 0.0,
            start_angle: 0.0,
            end_angle: 0.0,
            image_bytes: Vec::new(),
            image_width: 0,
            image_height: 0,
            image_bearing_x: 0.0,
            image_bearing_y: 0.0,
            image_extent_x: 0.0,
            image_extent_y: 0.0,
        }
    }
}

pub struct FontBase {
    line_metrics: LineMetrics,
}

impl FontBase {
    pub fn new(line_metrics: LineMetrics) -> Self {
        Self { line_metrics }
    }

    pub fn line_metrics(&self) -> &LineMetrics {
        &self.line_metrics
    }
}

pub trait Font: Any {
    fn as_any(&self) -> &dyn Any;
    fn base(&self) -> &FontBase;
    fn get_axis_count(&self) -> u16;
    fn get_axis(&self, index: u16) -> Axis;
    fn get_axis_value(&self, axis_tag: u32) -> f32;
    fn get_weight(&self) -> u16;
    fn is_italic(&self) -> bool;
    fn features(&self) -> Vec<u32>;
    fn has_glyph(&self, value: Unichar) -> bool;
    fn get_feature_value(&self, feature_tag: u32) -> u32;
    fn with_options(&self, variable_axes: &[Coord], features: &[Feature]) -> FontRef;
    fn get_path(&self, glyph: GlyphId) -> RawPath;
    fn on_shape_text(
        &self,
        text: &[Unichar],
        runs: &[TextRun],
        text_direction_flag: i32,
    ) -> Vec<Paragraph>;

    fn line_metrics(&self) -> &LineMetrics {
        self.base().line_metrics()
    }

    fn ascent(&self, size: f32) -> f32 {
        self.line_metrics().ascent * size
    }

    fn descent(&self, size: f32) -> f32 {
        self.line_metrics().descent * size
    }

    fn cap_height(&self, size: f32) -> f32 {
        self.line_metrics().cap_height * size
    }

    fn x_height(&self, size: f32) -> f32 {
        self.line_metrics().x_height * size
    }

    fn make_at_coords(&self, coords: &[Coord]) -> FontRef {
        self.with_options(coords, &[])
    }

    fn make_at_coord(&self, coord: Coord) -> FontRef {
        self.make_at_coords(&[coord])
    }

    fn has_color_glyphs(&self) -> bool {
        false
    }

    fn is_color_glyph(&self, _glyph: GlyphId) -> bool {
        false
    }

    fn get_color_layers(
        &self,
        _glyph: GlyphId,
        _out: &mut Vec<ColorGlyphLayer>,
        _foreground: ColorInt,
    ) -> usize {
        0
    }

    fn shape_text(
        &self,
        text: &[Unichar],
        runs: &[TextRun],
        text_direction_flag: i32,
    ) -> Vec<Paragraph> {
        debug_assert!({
            let count: usize = runs
                .iter()
                .map(|run| {
                    assert!(run.unichar_count > 0);
                    run.unichar_count as usize
                })
                .sum();
            count <= text.len()
        });

        let mut paragraphs = self.on_shape_text(text, runs, text_direction_flag);
        let mut want_white_space = false;
        let reserve_size = text.len() / 4;
        let mut breaks = Vec::with_capacity(reserve_size);
        let mut joiners = Vec::with_capacity(reserve_size);
        let mut last_run: Option<(usize, usize)> = None;

        for paragraph_index in 0..paragraphs.len() {
            for run_index in 0..paragraphs[paragraph_index].runs.len() {
                if let Some((previous_paragraph, previous_run)) = last_run {
                    let previous = &mut paragraphs[previous_paragraph].runs[previous_run];
                    previous.breaks =
                        std::mem::replace(&mut breaks, Vec::with_capacity(reserve_size));
                    previous.joiners =
                        std::mem::replace(&mut joiners, Vec::with_capacity(reserve_size));
                }

                let glyph_run = &mut paragraphs[paragraph_index].runs[run_index];
                for (glyph_index, offset) in glyph_run.text_indices.iter().copied().enumerate() {
                    let unicode = text[offset as usize];
                    if unicode == u32::from(b'\n') || unicode == 0x2028 {
                        breaks.push(glyph_index as u32);
                        breaks.push(glyph_index as u32);
                    }
                    if unicode == 0x2060 {
                        joiners.push(offset);
                    }
                    if want_white_space == is_white_space(unicode) {
                        breaks.push(glyph_index as u32);
                        want_white_space = !want_white_space;
                    }
                }

                last_run = Some((paragraph_index, run_index));
            }
        }

        if let Some((paragraph_index, run_index)) = last_run {
            let last_run = &mut paragraphs[paragraph_index].runs[run_index];
            if want_white_space {
                breaks.push(last_run.glyphs.len() as u32);
            } else {
                let last_break = breaks.last().copied().unwrap_or(0);
                breaks.push(last_break);
                breaks.push(last_run.glyphs.len() as u32);
            }
            last_run.breaks = breaks;
            last_run.joiners = joiners;
        }

        debug_assert!(
            paragraphs
                .iter()
                .all(
                    |paragraph| paragraph.runs.iter().all(|run| !run.glyphs.is_empty()
                        && run.glyphs.len() == run.text_indices.len()
                        && run.glyphs.len() + 1 == run.xpos.len())
                )
        );
        paragraphs
    }
}

pub type FontRef = Rc<dyn Font>;
pub type FallbackProc = fn(Unichar, u32, &dyn Font) -> Option<FontRef>;
thread_local! {
    // A host RawText occurrence may provide a scoped fallback chain without
    // racing another thread's process-level fallback provider.
    static HOST_FALLBACK_PROC: std::cell::Cell<Option<FallbackProc>> = const { std::cell::Cell::new(None) };
}
pub fn with_host_fallback_proc<R>(callback: FallbackProc, work: impl FnOnce() -> R) -> R {
    struct Restore(Option<FallbackProc>);
    impl Drop for Restore {
        fn drop(&mut self) {
            HOST_FALLBACK_PROC.with(|slot| slot.set(self.0));
        }
    }
    let previous = HOST_FALLBACK_PROC.with(|slot| slot.replace(Some(callback)));
    let _restore = Restore(previous);
    work()
}
static G_FALLBACK_PROC: std::sync::Mutex<Option<FallbackProc>> = std::sync::Mutex::new(None);
static G_FALLBACK_PROC_ENABLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);
pub const REGULAR_WEIGHT: u32 = 400;

pub fn set_fallback_proc(value: Option<FallbackProc>) {
    *G_FALLBACK_PROC
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
}

pub fn fallback_proc() -> Option<FallbackProc> {
    if let Some(callback) = HOST_FALLBACK_PROC.with(|slot| slot.get()) {
        return Some(callback);
    }
    *G_FALLBACK_PROC
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn set_fallback_proc_enabled(value: bool) {
    G_FALLBACK_PROC_ENABLED.store(value, std::sync::atomic::Ordering::Relaxed);
}

pub fn fallback_proc_enabled() -> bool {
    G_FALLBACK_PROC_ENABLED.load(std::sync::atomic::Ordering::Relaxed)
}

#[derive(Clone)]
pub struct TextRun {
    pub font: Option<FontRef>,
    pub size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub unichar_count: u32,
    pub script: u32,
    pub style_id: u16,
    pub level: u8,
}

#[derive(Clone)]
pub struct GlyphRun {
    pub font: Option<FontRef>,
    pub size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub glyphs: Vec<GlyphId>,
    pub text_indices: Vec<u32>,
    pub advances: Vec<f32>,
    pub xpos: Vec<f32>,
    pub offsets: Vec<Vec2D>,
    pub breaks: Vec<u32>,
    pub style_id: u16,
    pub level: u8,
    pub joiners: Vec<u32>,
}

impl GlyphRun {
    pub fn new(glyph_count: usize) -> Self {
        Self {
            font: None,
            size: 0.0,
            line_height: 0.0,
            letter_spacing: 0.0,
            glyphs: vec![0; glyph_count],
            text_indices: vec![0; glyph_count],
            advances: vec![0.0; glyph_count],
            xpos: vec![0.0; glyph_count + 1],
            offsets: vec![Vec2D::default(); glyph_count],
            breaks: Vec::new(),
            style_id: 0,
            level: 0,
            joiners: Vec::new(),
        }
    }

    pub fn from_arrays(
        glyphs: Vec<GlyphId>,
        text_indices: Vec<u32>,
        advances: Vec<f32>,
        xpos: Vec<f32>,
        offsets: Vec<Vec2D>,
    ) -> Self {
        Self {
            font: None,
            size: 0.0,
            line_height: 0.0,
            letter_spacing: 0.0,
            glyphs,
            text_indices,
            advances,
            xpos,
            offsets,
            breaks: Vec::new(),
            style_id: 0,
            level: 0,
            joiners: Vec::new(),
        }
    }

    pub fn dir(&self) -> TextDirection {
        if self.level & 1 != 0 {
            TextDirection::Rtl
        } else {
            TextDirection::Ltr
        }
    }
}

pub struct OrderedLine {
    start_logical: Option<usize>,
    end_logical: Option<usize>,
    start_glyph_index: u32,
    end_glyph_index: u32,
    runs: Vec<OrderedRun>,
    glyph_line: GlyphLine,
    y: f32,
}

#[derive(Clone)]
struct OrderedRun {
    run: GlyphRun,
    logical_index: Option<usize>,
}

impl OrderedLine {
    pub fn new(
        paragraph: &Paragraph,
        line: &GlyphLine,
        line_width: f32,
        want_ellipsis: bool,
        is_ellipsis_line_last: bool,
        ellipsis_run: &mut GlyphRun,
        y: f32,
    ) -> Self {
        let mut result = Self {
            start_logical: None,
            end_logical: None,
            start_glyph_index: line.start_glyph_index,
            end_glyph_index: line.end_glyph_index,
            runs: Vec::new(),
            glyph_line: line.clone(),
            y,
        };
        let mut logical_runs = Vec::new();

        if !want_ellipsis
            || !result.build_ellipsis_runs(
                &mut logical_runs,
                paragraph,
                line,
                line_width,
                is_ellipsis_line_last,
                ellipsis_run,
            )
        {
            for i in line.start_run_index..line.end_run_index + 1 {
                logical_runs.push(OrderedRun {
                    run: paragraph.runs[i as usize].clone(),
                    logical_index: Some(i as usize),
                });
            }
            if !logical_runs.is_empty() {
                result.start_logical = logical_runs[0].logical_index;
                result.end_logical = logical_runs[logical_runs.len() - 1].logical_index;
            }
        }

        let mut max_level = 0;
        for run in &logical_runs {
            let level = run.run.level;
            if level > max_level {
                max_level = level;
            }
        }
        for new_level in (1..=max_level).rev() {
            let mut start = logical_runs.len() as i32 - 1;
            while start >= 0 {
                if logical_runs[start as usize].run.level >= new_level {
                    let mut count = 1;
                    while start > 0 && logical_runs[start as usize - 1].run.level >= new_level {
                        start -= 1;
                        count += 1;
                    }
                    reverse_runs(&mut logical_runs[start as usize..start as usize + count]);
                }
                start -= 1;
            }
        }
        result.runs = logical_runs;
        result
    }

    pub fn start_logical(&self) -> Option<usize> {
        self.start_logical
    }

    pub fn end_logical(&self) -> Option<usize> {
        self.end_logical
    }

    pub fn runs(&self) -> impl ExactSizeIterator<Item = &GlyphRun> {
        self.runs.iter().map(|run| &run.run)
    }

    pub fn begin(&self) -> GlyphItr<'_> {
        let mut iterator = GlyphItr {
            line: self,
            run_index: 0,
            glyph_index: self.start_glyph_index(0),
        };
        iterator.try_advance_run();
        iterator
    }

    pub fn end(&self) -> GlyphItr<'_> {
        let run_index = if self.runs.is_empty() {
            0
        } else {
            self.runs.len() - 1
        };
        GlyphItr {
            line: self,
            run_index,
            glyph_index: self.end_glyph_index(run_index),
        }
    }

    pub fn glyph_line(&self) -> &GlyphLine {
        &self.glyph_line
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn bottom(&self) -> f32 {
        self.y - self.glyph_line().baseline + self.glyph_line().bottom
    }

    pub fn first_code_point_index(&self, glyph_lookup: &GlyphLookup) -> u32 {
        let index = self.begin();
        let glyph_index = index.glyph_index();
        let run = index.run();
        let mut first = run.text_indices[glyph_index as usize];
        if run.dir() == TextDirection::Rtl {
            first += glyph_lookup.count(run.text_indices[glyph_index as usize]);
        }
        first.min(glyph_lookup.last_code_point_index().wrapping_sub(1))
    }

    pub fn last_code_point_index(&self, glyph_lookup: &GlyphLookup) -> u32 {
        let mut index = self.begin();
        let mut last_index = index;
        while index != self.end() {
            last_index = index;
            index.advance();
        }
        let glyph_index = last_index.glyph_index();
        let run = last_index.run();
        let mut last = run.text_indices[glyph_index as usize];
        if run.dir() == TextDirection::Ltr {
            last += glyph_lookup.count(run.text_indices[glyph_index as usize]);
        }
        last.min(glyph_lookup.last_code_point_index().wrapping_sub(1))
    }

    pub fn contains_code_point_index(
        &self,
        glyph_lookup: &GlyphLookup,
        code_point_index: u32,
    ) -> bool {
        code_point_index >= self.first_code_point_index(glyph_lookup)
            && code_point_index <= self.last_code_point_index(glyph_lookup)
    }

    pub fn last_run(&self) -> &GlyphRun {
        &self.runs[self.runs.len() - 1].run
    }

    pub fn start_glyph_index(&self, run_index: usize) -> u32 {
        let ordered = &self.runs[run_index];
        let run = &ordered.run;
        match run.dir() {
            TextDirection::Ltr => {
                if self.start_logical == ordered.logical_index {
                    self.start_glyph_index
                } else {
                    0
                }
            }
            TextDirection::Rtl => (if self.end_logical == ordered.logical_index {
                self.end_glyph_index
            } else {
                run.glyphs.len() as u32
            })
            .wrapping_sub(1),
        }
    }

    pub fn end_glyph_index(&self, run_index: usize) -> u32 {
        let ordered = &self.runs[run_index];
        let run = &ordered.run;
        match run.dir() {
            TextDirection::Ltr => {
                if self.end_logical == ordered.logical_index {
                    self.end_glyph_index
                } else {
                    run.glyphs.len() as u32
                }
            }
            TextDirection::Rtl => (if self.start_logical == ordered.logical_index {
                self.start_glyph_index
            } else {
                0
            })
            .wrapping_sub(1),
        }
    }

    fn build_ellipsis_runs(
        &mut self,
        logical_runs: &mut Vec<OrderedRun>,
        paragraph: &Paragraph,
        line: &GlyphLine,
        line_width: f32,
        is_ellipsis_line_last: bool,
        stored_ellipsis_run: &mut GlyphRun,
    ) -> bool {
        let mut x = 0.0;
        let glyph_runs = &paragraph.runs;
        let mut start_glyph_index = line.start_glyph_index;
        if is_ellipsis_line_last {
            let mut fits = true;
            'measured: for i in line.start_run_index..line.end_run_index + 1 {
                let run = &glyph_runs[i as usize];
                let end_glyph_index = if i == line.end_run_index {
                    line.end_glyph_index
                } else {
                    run.glyphs.len() as u32
                };
                for j in start_glyph_index..end_glyph_index {
                    x += run.advances[j as usize];
                    if x > line_width {
                        fits = false;
                        break 'measured;
                    }
                }
                start_glyph_index = 0;
            }
            if fits {
                return false;
            }
        }

        let ellipsis_code_points = vec![b'.' as Unichar, b'.' as Unichar, b'.' as Unichar];
        let mut ellipsis_font: Option<FontRef> = None;
        let mut ellipsis_font_size = 0.0;
        let mut ellipsis_run = GlyphRun::new(0);
        let mut ellipsis_width = 0.0;
        let mut ellipsis_overflowed = false;
        start_glyph_index = line.start_glyph_index;
        x = 0.0;

        for i in line.start_run_index..line.end_run_index + 1 {
            let run = &glyph_runs[i as usize];
            let fonts_differ = match (&run.font, &ellipsis_font) {
                (Some(left), Some(right)) => !Rc::ptr_eq(left, right),
                (None, None) => false,
                _ => true,
            };
            if fonts_differ && run.size != ellipsis_font_size {
                ellipsis_font = run.font.clone();
                ellipsis_font_size = run.size;
                let ellipsis_runs = [TextRun {
                    font: ellipsis_font.clone(),
                    size: ellipsis_font_size,
                    line_height: run.line_height,
                    letter_spacing: run.letter_spacing,
                    unichar_count: ellipsis_code_points.len() as u32,
                    script: 0,
                    style_id: run.style_id,
                    level: 0,
                }];
                let next_shape = ellipsis_font.as_ref().unwrap().shape_text(
                    &ellipsis_code_points,
                    &ellipsis_runs,
                    -1,
                );
                let next_run = &next_shape[0].runs[0];
                let mut next_width = 0.0;
                for advance in &next_run.advances {
                    next_width += *advance;
                }
                if ellipsis_run.font.is_none() || x + next_width <= line_width {
                    ellipsis_width = next_width;
                    ellipsis_run = next_run.clone();
                }
            }

            let end_glyph_index = if i == line.end_run_index {
                line.end_glyph_index
            } else {
                run.glyphs.len() as u32
            };
            for j in start_glyph_index..end_glyph_index {
                let advance = run.advances[j as usize];
                if x + advance + ellipsis_width > line_width {
                    self.end_glyph_index = j;
                    ellipsis_overflowed = true;
                    break;
                }
                x += advance;
            }
            start_glyph_index = 0;
            logical_runs.push(OrderedRun {
                run: run.clone(),
                logical_index: Some(i as usize),
            });
            self.end_logical = Some(i as usize);

            if ellipsis_overflowed && ellipsis_run.font.is_some() {
                *stored_ellipsis_run = ellipsis_run;
                logical_runs.push(OrderedRun {
                    run: stored_ellipsis_run.clone(),
                    logical_index: None,
                });
                break;
            }
        }

        if !ellipsis_overflowed && ellipsis_run.font.is_some() {
            *stored_ellipsis_run = ellipsis_run;
            logical_runs.push(OrderedRun {
                run: stored_ellipsis_run.clone(),
                logical_index: None,
            });
        }
        self.start_logical = logical_runs[0].logical_index;
        true
    }
}

fn reverse_runs(runs: &mut [OrderedRun]) {
    let half_count = runs.len() / 2;
    let final_index = runs.len() - 1;
    for index in 0..half_count {
        let tie_index = final_index - index;
        runs.swap(index, tie_index);
    }
}

#[derive(Clone, Copy)]
pub struct GlyphItr<'a> {
    line: &'a OrderedLine,
    run_index: usize,
    glyph_index: u32,
}

impl PartialEq for GlyphItr<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.run_index == other.run_index && self.glyph_index == other.glyph_index
    }
}

impl GlyphItr<'_> {
    pub fn try_advance_run(&mut self) {
        loop {
            if self.glyph_index == self.line.end_glyph_index(self.run_index)
                && self.run_index + 1 < self.line.runs.len()
            {
                self.run_index += 1;
                self.glyph_index = self.line.start_glyph_index(self.run_index);
            } else {
                break;
            }
        }
    }

    pub fn advance(&mut self) {
        let run = self.run();
        self.glyph_index = if run.dir() == TextDirection::Ltr {
            self.glyph_index + 1
        } else {
            self.glyph_index.wrapping_sub(1)
        };
        self.try_advance_run();
    }

    pub fn run(&self) -> &GlyphRun {
        &self.line.runs[self.run_index].run
    }

    pub fn glyph_index(&self) -> u32 {
        self.glyph_index
    }
}

pub struct GlyphItrIterator<'a> {
    current: GlyphItr<'a>,
    end: GlyphItr<'a>,
}

impl<'a> Iterator for GlyphItrIterator<'a> {
    type Item = (&'a GlyphRun, u32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            return None;
        }
        let run = self.current.run();
        let glyph_index = self.current.glyph_index();
        self.current.advance();
        Some((run, glyph_index))
    }
}

impl<'a> IntoIterator for &'a OrderedLine {
    type Item = (&'a GlyphRun, u32);
    type IntoIter = GlyphItrIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        GlyphItrIterator {
            current: self.begin(),
            end: self.end(),
        }
    }
}
