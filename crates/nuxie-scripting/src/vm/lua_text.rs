//! src/lua/renderer/lua_text.cpp and include/rive/lua/scripted_text.hpp at db31ea87.
//! Retained layout plus epoch-checked indices replace interior C++ pointers.
use super::{
    host_memory_budget::HostMemoryCharge, lua_font::ScriptedFont, lua_mat2d::ScriptedMat2D,
    lua_paint::ScriptedPaint, lua_path::ScriptedPath, lua_renderer::ScriptedRenderer,
    lua_renderer_library::RendererBindings,
};
use crate::luau::{
    AnyUserData, Error, Lua, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods,
    Value, Vector,
};
use nuxie_runtime::{
    RuntimeFactoryHandle,
    source::{
        math::{raw_path::RawPath, vec2d::Vec2D},
        renderer::to_render_raw_path,
        shapes::paint::shape_paint::RuntimeRenderPaintHandle,
        text::{
            cursor::{Cursor, CursorPosition},
            raw_text::RawText,
        },
        text_engine::{
            GlyphRenderKind, GlyphRun, OrderedLine, TextDirection, TextOrigin, TextOverflow,
            TextSizing, TextWordBreak, TextWrap,
        },
    },
};
use std::{cell::RefCell, rc::Rc};

/// Host bytes one appended input byte is charged: a code point (4 bytes; a
/// byte is at most one code point) and the shaped glyph data it may become
/// (glyph id, text index, advance, x position, offset: about 20 bytes).
const TEXT_BYTES_PER_INPUT_BYTE: usize = 24;
/// Host bytes one append's `TextRun` is charged.
const TEXT_RUN_BYTES: usize = 64;
/// Host bytes one new style (a distinct paint and colour) is charged: the
/// style, its paint handle and its retained render path.
const TEXT_STYLE_BYTES: usize = 256;
/// Styles one Text may hold: a glyph run names its style with a `u16`.
const MAX_TEXT_STYLES: usize = 1 << 16;

const STALE: &str = "Text changed since this was fetched, fetch it again.";

fn stale() -> Error {
    Error::runtime(STALE)
}

/// A Text's layout and the epoch its handles are checked against.
///
/// `RawText::revision` is a `u32` that wraps, so after 2^32 changes a handle
/// fetched before them would pass a revision check against a different
/// layout. The epoch counts every revision change observed, in a `u64`; it
/// is brought up to date on every access, and one binding call changes the
/// revision a few times at most, never 2^32.
struct TextLayout {
    text: RawText,
    /// `text.revision()` when last observed.
    seen_revision: u32,
    epoch: u64,
    /// Host bytes this Text is charged: the most its content has needed
    /// since it was created, since a clear keeps the capacity it grew to.
    charge: HostMemoryCharge,
    /// Bytes the content appended since the last clear needs.
    content_bytes: usize,
}

impl TextLayout {
    fn new(text: RawText) -> Self {
        Self {
            seen_revision: text.revision(),
            text,
            epoch: 0,
            charge: HostMemoryCharge::default(),
            content_bytes: 0,
        }
    }

    fn sync(&mut self) {
        let revision = self.text.revision();
        self.epoch = advance_epoch(self.epoch, self.seen_revision, revision);
        self.seen_revision = revision;
    }

    fn epoch(&mut self) -> u64 {
        self.sync();
        self.epoch
    }

    fn text(&mut self) -> &mut RawText {
        self.sync();
        &mut self.text
    }

    /// The text, if nothing changed since `epoch`.
    fn fresh(&mut self, epoch: u64) -> Result<&mut RawText> {
        if self.epoch() != epoch {
            return Err(stale());
        }
        Ok(&mut self.text)
    }
}

/// The epoch after the revision moved from `seen` to `revision`: the
/// changes in between, counted modulo 2^32 as the revision wraps.
fn advance_epoch(epoch: u64, seen: u32, revision: u32) -> u64 {
    epoch.wrapping_add(u64::from(revision.wrapping_sub(seen)))
}

type Layout = Rc<RefCell<TextLayout>>;
pub(super) struct ScriptedText {
    layout: Layout,
}
#[derive(Clone)]
struct ScriptedTextLine {
    layout: Layout,
    epoch: u64,
    index: u32,
}
struct ScriptedGlyph {
    layout: Layout,
    epoch: u64,
    line: u32,
    run: usize,
    glyph: u32,
    position: Vec2D,
}
struct ScriptedGlyphCursor {
    line: ScriptedTextLine,
    run: usize,
    glyph: u32,
    x: f32,
}
impl UserData for ScriptedGlyphCursor {}

const SIZING: &[&str] = &["autoWidth", "autoHeight", "fixed"];
const OVERFLOW: &[&str] = &["visible", "hidden", "clipped", "ellipsis"];
const ALIGN: &[&str] = &["left", "right", "center", "start", "end"];
const WRAP: &[&str] = &["wrap", "noWrap"];
const WORD_BREAK: &[&str] = &["breakWord", "normal", "breakAll"];
const ORIGIN: &[&str] = &["top", "baseline"];
const DIRECTION: &[&str] = &["auto", "ltr", "rtl"];
fn enum_index(value: &str, names: &[&str], kind: &str) -> Result<i32> {
    names
        .iter()
        .position(|name| *name == value)
        .map(|i| i as i32)
        .ok_or_else(|| Error::runtime(format!("'{value}' is not a valid {kind}")))
}
/// The name at a layout's enum index; a file can store any index.
fn enum_name(names: &'static [&'static str], index: i32, kind: &str) -> Result<&'static str> {
    usize::try_from(index)
        .ok()
        .and_then(|index| names.get(index).copied())
        .ok_or_else(|| Error::runtime(format!("Text has an invalid {kind} value")))
}
/// A zero-based index as the one-based index a script sees.
fn one_based(index: u32) -> Result<u32> {
    index.checked_add(1).ok_or_else(stale)
}

fn vector(p: Vec2D) -> Vector {
    Vector::new(p.x, p.y, 0.0)
}
fn push_path(lua: &Lua, path: RawPath) -> Result<AnyUserData> {
    lua.create_userdata(ScriptedPath::charged(lua, to_render_raw_path(&path))?)
}
fn direction(value: TextDirection) -> &'static str {
    match value {
        TextDirection::Ltr => "ltr",
        TextDirection::Rtl => "rtl",
    }
}
fn line_at(text: &mut RawText, index: u32) -> Result<&OrderedLine> {
    text.ordered_lines().get(index as usize).ok_or_else(stale)
}
fn run_at(line: &OrderedLine, run: usize) -> Result<&GlyphRun> {
    line.runs().nth(run).ok_or_else(stale)
}
/// One glyph's entry in a run's per-glyph arrays.
fn glyph_value<T: Copy>(values: &[T], glyph: u32) -> Result<T> {
    values.get(glyph as usize).copied().ok_or_else(stale)
}
fn same_paint(a: Option<&RuntimeRenderPaintHandle>, b: Option<&RuntimeRenderPaintHandle>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
/// Refuses an append that would add a style past the `u16` a glyph run
/// names it with. `RawText::append` reuses a style with the same paint and
/// colour, so only a full Text is searched, and only for a new style.
fn check_style_room(
    text: &RawText,
    paint: Option<&RuntimeRenderPaintHandle>,
    foreground: u32,
) -> Result<()> {
    if text.style_count() < MAX_TEXT_STYLES {
        return Ok(());
    }
    let existing = (0..=u16::MAX).any(|style| {
        text.style_foreground_color(style) == foreground
            && same_paint(text.style_paint(style).as_ref(), paint)
    });
    if existing {
        Ok(())
    } else {
        Err(Error::runtime(format!(
            "Text may hold at most {MAX_TEXT_STYLES} styles"
        )))
    }
}
impl ScriptedText {
    fn line(&self, index: u32) -> ScriptedTextLine {
        ScriptedTextLine {
            layout: self.layout.clone(),
            epoch: self.layout.borrow_mut().epoch(),
            index,
        }
    }
    fn line_count(&self) -> usize {
        self.layout.borrow_mut().text().ordered_lines().len()
    }
}
impl UserData for ScriptedText {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("sizing", |_, t| match t.layout.borrow_mut().text().sizing() {
            TextSizing::AutoWidth => Ok("autoWidth"),
            TextSizing::AutoHeight => Ok("autoHeight"),
            TextSizing::Fixed => Ok("fixed"),
            TextSizing::Unknown(_) => Err(Error::runtime("Text has an invalid sizing value")),
        });
        fields.add_field_method_get("overflow", |_, t| {
            match t.layout.borrow_mut().text().overflow() {
                TextOverflow::Visible => Ok("visible"),
                TextOverflow::Hidden => Ok("hidden"),
                TextOverflow::Clipped => Ok("clipped"),
                TextOverflow::Ellipsis => Ok("ellipsis"),
                TextOverflow::Fit | TextOverflow::FitFontSize | TextOverflow::Unknown(_) => {
                    Err(Error::runtime("Text has an invalid overflow value"))
                }
            }
        });
        fields.add_field_method_get("align", |_, t| {
            enum_name(ALIGN, t.layout.borrow_mut().text().align_index(), "align")
        });
        fields.add_field_method_get("wrap", |_, t| match t.layout.borrow_mut().text().wrap() {
            TextWrap::Wrap => Ok("wrap"),
            TextWrap::NoWrap => Ok("noWrap"),
            TextWrap::Unknown(_) => Err(Error::runtime("Text has an invalid wrap value")),
        });
        fields.add_field_method_get("wordBreak", |_, t| {
            match t.layout.borrow_mut().text().word_break() {
                TextWordBreak::BreakWord => Ok("breakWord"),
                TextWordBreak::Normal => Ok("normal"),
                TextWordBreak::BreakAll => Ok("breakAll"),
                TextWordBreak::Unknown(_) => {
                    Err(Error::runtime("Text has an invalid wordBreak value"))
                }
            }
        });
        fields.add_field_method_get("origin", |_, t| match t.layout.borrow_mut().text().origin() {
            TextOrigin::Top => Ok("top"),
            TextOrigin::Baseline => Ok("baseline"),
            TextOrigin::Unknown(_) => Err(Error::runtime("Text has an invalid origin value")),
        });
        fields.add_field_method_get("direction", |_, t| {
            enum_name(
                DIRECTION,
                t.layout.borrow_mut().text().direction_index(),
                "direction",
            )
        });
        fields.add_field_method_get("maxWidth", |_, t| {
            Ok(t.layout.borrow_mut().text().max_width())
        });
        fields.add_field_method_get("maxHeight", |_, t| {
            Ok(t.layout.borrow_mut().text().max_height())
        });
        fields.add_field_method_get("paragraphSpacing", |_, t| {
            Ok(t.layout.borrow_mut().text().paragraph_spacing())
        });
        fields.add_field_method_get("length", |_, t| Ok(t.layout.borrow_mut().text().length()));
        fields.add_field_method_get("isEmpty", |_, t| Ok(t.layout.borrow_mut().text().empty()));
        fields.add_field_method_get("lineCount", |_, t| Ok(t.line_count()));
        fields.add_field_method_set("sizing", |_, t, v: String| {
            t.layout
                .borrow_mut()
                .text()
                .set_sizing_index(enum_index(&v, SIZING, "TextSizing")?);
            Ok(())
        });
        fields.add_field_method_set("overflow", |_, t, v: String| {
            t.layout
                .borrow_mut()
                .text()
                .set_overflow_index(enum_index(&v, OVERFLOW, "TextOverflow")?);
            Ok(())
        });
        fields.add_field_method_set("align", |_, t, v: String| {
            t.layout
                .borrow_mut()
                .text()
                .set_align_index(enum_index(&v, ALIGN, "TextAlign")?);
            Ok(())
        });
        fields.add_field_method_set("wrap", |_, t, v: String| {
            t.layout
                .borrow_mut()
                .text()
                .set_wrap_index(enum_index(&v, WRAP, "TextWrap")?);
            Ok(())
        });
        fields.add_field_method_set("wordBreak", |_, t, v: String| {
            t.layout.borrow_mut().text().set_word_break_index(enum_index(
                &v,
                WORD_BREAK,
                "TextWordBreak",
            )?);
            Ok(())
        });
        fields.add_field_method_set("origin", |_, t, v: String| {
            t.layout
                .borrow_mut()
                .text()
                .set_origin_index(enum_index(&v, ORIGIN, "TextOrigin")?);
            Ok(())
        });
        fields.add_field_method_set("direction", |_, t, v: String| {
            t.layout
                .borrow_mut()
                .text()
                .set_direction_index(enum_index(&v, DIRECTION, "TextDirection")?);
            Ok(())
        });
        fields.add_field_method_set("maxWidth", |_, t, v: f32| {
            t.layout.borrow_mut().text().set_max_width(v);
            Ok(())
        });
        fields.add_field_method_set("maxHeight", |_, t, v: f32| {
            t.layout.borrow_mut().text().set_max_height(v);
            Ok(())
        });
        fields.add_field_method_set("paragraphSpacing", |_, t, v: f32| {
            t.layout.borrow_mut().text().set_paragraph_spacing(v);
            Ok(())
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "append",
            |lua, t, (chars, style): (crate::luau::LuaString, Table)| {
                let bytes = chars.as_bytes();
                let font: Option<AnyUserData> = style.raw_get("font")?;
                let font = font.ok_or_else(|| Error::runtime("Text:append needs a font."))?;
                let font = font.borrow::<ScriptedFont>()?.native()?;
                let paint: Option<AnyUserData> = style.raw_get("paint")?;
                let paint = paint
                    .map(|p| p.borrow::<ScriptedPaint>().map(|p| p.render_paint.clone()))
                    .transpose()?;
                let foreground: Value = style.raw_get("foregroundColor")?;
                let foreground = if matches!(foreground, Value::Nil) {
                    0xff000000
                } else {
                    super::lua_color::required_unsigned(lua, Some(&foreground), "foregroundColor")?
                };
                let size = style.raw_get::<Option<f32>>("size")?.unwrap_or(16.0);
                let line_height = style.raw_get::<Option<f32>>("lineHeight")?.unwrap_or(-1.0);
                let letter_spacing = style
                    .raw_get::<Option<f32>>("letterSpacing")?
                    .unwrap_or(0.0);
                let mut layout = t.layout.borrow_mut();
                let layout = &mut *layout;
                check_style_room(layout.text(), paint.as_ref(), foreground)?;
                // Charged for a new style up front; the content keeps only
                // what it used, the charge the most it has needed.
                let appended = bytes
                    .len()
                    .saturating_mul(TEXT_BYTES_PER_INPUT_BYTE)
                    .saturating_add(TEXT_RUN_BYTES);
                layout.charge.attach(lua)?;
                layout.charge.reserve_to(
                    layout
                        .content_bytes
                        .saturating_add(appended)
                        .saturating_add(TEXT_STYLE_BYTES),
                )?;
                let styles = layout.text.style_count();
                layout.text().append(
                    &bytes,
                    paint,
                    font,
                    size,
                    line_height,
                    letter_spacing,
                    foreground,
                );
                let new_style = if layout.text.style_count() > styles {
                    TEXT_STYLE_BYTES
                } else {
                    0
                };
                layout.content_bytes = layout
                    .content_bytes
                    .saturating_add(appended)
                    .saturating_add(new_style);
                Ok(())
            },
        );
        methods.add_method("clear", |_, t, ()| {
            let mut layout = t.layout.borrow_mut();
            layout.text().clear();
            layout.content_bytes = 0;
            Ok(())
        });
        methods.add_method_lent(
            "draw",
            |_, t, loan, (renderer, paint): (AnyUserData, Option<AnyUserData>)| {
                let paint = paint
                    .map(|p| p.borrow::<ScriptedPaint>().map(|p| p.render_paint.clone()))
                    .transpose()?;
                renderer
                    .borrow::<ScriptedRenderer>()?
                    .with_renderer_mut(loan, |renderer| {
                        t.layout.borrow_mut().text().render(renderer, paint);
                        Ok(())
                    })
            },
        );
        methods.add_method("bounds", |_, t, ()| {
            let b = t.layout.borrow_mut().text().bounds();
            Ok((
                vector(Vec2D::new(b.min_x, b.min_y)),
                vector(Vec2D::new(b.max_x, b.max_y)),
            ))
        });
        methods.add_method("line", |lua, t, index: i32| {
            let count = t.line_count();
            let line = usize::try_from(index)
                .ok()
                .and_then(|index| index.checked_sub(1))
                .filter(|line| *line < count)
                .and_then(|line| u32::try_from(line).ok())
                .ok_or_else(|| Error::runtime(format!("line {index} is out of range")))?;
            push_line(lua, t.line(line))
        });
        methods.add_method("lines", |lua, t, ()| {
            let count = t.line_count();
            let lines = lua.create_table();
            for i in (0..count).map_while(|i| u32::try_from(i).ok()) {
                lines.raw_push(push_line(lua, t.line(i))?)?;
            }
            Ok(lines)
        });
        methods.add_method("hitTest", |_, t, p: Vector| {
            let mut layout = t.layout.borrow_mut();
            let view = layout.text().layout_view();
            one_based(
                CursorPosition::from_translation(Vec2D::new(p.x(), p.y()), &view)
                    .code_point_index(),
            )
        });
        methods.add_method("caret", |_, t, index: i32| {
            let mut layout = t.layout.borrow_mut();
            let view = layout.text().layout_view();
            let p = CursorPosition::at_index(index.saturating_sub(1).max(0) as u32, &view)
                .visual_position(&view);
            Ok(if p.found() {
                MultiValue::from_vec(vec![
                    Value::Number(p.x() as f64),
                    Value::Number(p.top() as f64),
                    Value::Number(p.bottom() as f64),
                ])
            } else {
                MultiValue::new()
            })
        });
        methods.add_method("selectionRects", |lua, t, (from, to): (i32, i32)| {
            // The rectangles are collected and the layout released before any
            // VM object is made.
            let rects = {
                let mut layout = t.layout.borrow_mut();
                let view = layout.text().layout_view();
                let cursor = Cursor::new(
                    CursorPosition::at_index(from.saturating_sub(1).max(0) as u32, &view),
                    CursorPosition::at_index(to.saturating_sub(1).max(0) as u32, &view),
                );
                let mut rects = Vec::new();
                cursor.selection_rects(&mut rects, &view);
                rects
            };
            let table = lua.create_table();
            for r in rects {
                let row = lua.create_table();
                row.set("min", vector(Vec2D::new(r.min_x, r.min_y)))?;
                row.set("max", vector(Vec2D::new(r.max_x, r.max_y)))?;
                table.raw_push(row)?;
            }
            Ok(table)
        });
    }
}

impl ScriptedTextLine {
    fn with<R>(&self, f: impl FnOnce(&mut RawText) -> Result<R>) -> Result<R> {
        let mut layout = self.layout.borrow_mut();
        f(layout.fresh(self.epoch)?)
    }
    fn glyphs(&self) -> Result<Vec<ScriptedGlyph>> {
        self.with(|text| {
            let line = line_at(text, self.index)?;
            let mut itr = line.begin();
            let end = line.end();
            let mut x = line.start_x();
            let mut result = Vec::new();
            while itr != end {
                let run = itr.run();
                let glyph = itr.glyph_index();
                let index = line
                    .runs()
                    .position(|candidate| std::ptr::eq(candidate, run))
                    .ok_or_else(stale)?;
                let offset = glyph_value(&run.offsets, glyph)?;
                result.push(ScriptedGlyph {
                    layout: self.layout.clone(),
                    epoch: self.epoch,
                    line: self.index,
                    run: index,
                    glyph,
                    position: Vec2D::new(x + offset.x, line.y() + offset.y),
                });
                x += glyph_value(&run.advances, glyph)?;
                itr.advance();
            }
            Ok(result)
        })
    }
}
impl UserData for ScriptedTextLine {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("index", |_, l| l.with(|_| one_based(l.index)));
        fields.add_field_method_get("x", |_, l| {
            l.with(|t| Ok(line_at(t, l.index)?.start_x()))
        });
        fields.add_field_method_get("top", |_, l| l.with(|t| Ok(t.line_top(l.index))));
        fields.add_field_method_get("baseline", |_, l| {
            l.with(|t| Ok(line_at(t, l.index)?.y()))
        });
        fields.add_field_method_get("bottom", |_, l| {
            l.with(|t| Ok(line_at(t, l.index)?.bottom()))
        });
        fields.add_field_method_get("width", |_, l| {
            l.with(|t| {
                let mut width = 0.0;
                t.for_each_glyph(l.index, |r, i, _| {
                    width += r.advances.get(i as usize).copied().unwrap_or(0.0);
                    true
                });
                Ok(width)
            })
        });
        fields.add_field_method_get("direction", |_, l| {
            l.with(|t| Ok(direction(t.line_direction(l.index))))
        });
        fields.add_field_method_get("firstIndex", |_, l| {
            l.with(|t| {
                let view = t.layout_view();
                let line = view.ordered_lines().get(l.index as usize).ok_or_else(stale)?;
                one_based(line.first_code_point_index(view.glyph_lookup()))
            })
        });
        fields.add_field_method_get("lastIndex", |_, l| {
            l.with(|t| {
                let view = t.layout_view();
                let line = view.ordered_lines().get(l.index as usize).ok_or_else(stale)?;
                one_based(line.last_code_point_index(view.glyph_lookup()))
            })
        });
        fields.add_field_method_get("glyphCount", |_, l| {
            l.with(|t| Ok(t.line_glyph_count(l.index)))
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("glyph", |lua, l, index: i32| {
            let glyph = l
                .glyphs()?
                .into_iter()
                .nth(index.wrapping_sub(1) as usize)
                .ok_or_else(|| Error::runtime(format!("glyph {index} is out of range")))?;
            push_glyph(lua, glyph)
        });
        methods.add_method("glyphs", |lua, l, ()| {
            let table = lua.create_table();
            for g in l.glyphs()? {
                table.raw_push(push_glyph(lua, g)?)?;
            }
            Ok(table)
        });
        methods.add_meta_method("__iter", |lua, l, ()| {
            let (glyph, x) = l.with(|t| {
                let line = line_at(t, l.index)?;
                Ok((line.start_glyph_index(0), line.start_x()))
            })?;
            let mut cursor = ScriptedGlyphCursor {
                line: l.clone(),
                run: 0,
                glyph,
                x,
            };
            cursor.skip_empty_runs()?;
            let next = lua.create_function(|lua, (cursor, index): (AnyUserData, i32)| {
                let glyph = cursor.borrow_mut::<ScriptedGlyphCursor>()?.next()?;
                match glyph {
                    Some(g) => Ok(MultiValue::from_vec(vec![
                        Value::Integer(i64::from(index).saturating_add(1)),
                        Value::UserData(push_glyph(lua, g)?),
                    ])),
                    None => Ok(MultiValue::from_vec(vec![Value::Nil])),
                }
            })?;
            Ok((next, lua.create_userdata(cursor)?, 0))
        });
    }
}
impl ScriptedGlyphCursor {
    fn skip_empty_runs(&mut self) -> Result<()> {
        let mut layout = self.line.layout.borrow_mut();
        let line = line_at(layout.fresh(self.line.epoch)?, self.line.index)?;
        let last_run = line.runs().len().saturating_sub(1);
        while self.glyph == line.end_glyph_index(self.run) && self.run < last_run {
            self.run = self.run.saturating_add(1);
            self.glyph = line.start_glyph_index(self.run);
        }
        Ok(())
    }
    fn next(&mut self) -> Result<Option<ScriptedGlyph>> {
        let result = {
            let mut layout = self.line.layout.borrow_mut();
            let line = line_at(layout.fresh(self.line.epoch)?, self.line.index)?;
            let last_run = line.runs().len().saturating_sub(1);
            if self.run >= last_run && self.glyph == line.end_glyph_index(self.run) {
                return Ok(None);
            }
            let run = run_at(line, self.run)?;
            let offset = glyph_value(&run.offsets, self.glyph)?;
            let glyph = ScriptedGlyph {
                layout: self.line.layout.clone(),
                epoch: self.line.epoch,
                line: self.line.index,
                run: self.run,
                glyph: self.glyph,
                position: Vec2D::new(self.x + offset.x, line.y() + offset.y),
            };
            self.x += glyph_value(&run.advances, self.glyph)?;
            self.glyph = if run.dir() == TextDirection::Ltr {
                self.glyph.wrapping_add(1)
            } else {
                self.glyph.wrapping_sub(1)
            };
            glyph
        };
        self.skip_empty_runs()?;
        Ok(Some(result))
    }
}
impl ScriptedGlyph {
    fn with<R>(&self, f: impl FnOnce(&GlyphRun) -> Result<R>) -> Result<R> {
        let mut layout = self.layout.borrow_mut();
        let line = line_at(layout.fresh(self.epoch)?, self.line)?;
        f(run_at(line, self.run)?)
    }
}
/// The font a shaped run was shaped with.
fn run_font(run: &GlyphRun) -> Result<&nuxie_runtime::source::text_engine::FontRef> {
    run.font
        .as_ref()
        .ok_or_else(|| Error::runtime("glyph has no font"))
}
impl UserData for ScriptedGlyph {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, g| g.with(|r| glyph_value(&r.glyphs, g.glyph)));
        fields.add_field_method_get("font", |lua, g| {
            super::lua_font::push_font(lua, g.with(|r| run_font(r).cloned())?)
        });
        fields.add_field_method_get("size", |_, g| g.with(|r| Ok(r.size)));
        fields.add_field_method_get("x", |_, g| g.with(|_| Ok(g.position.x)));
        fields.add_field_method_get("y", |_, g| g.with(|_| Ok(g.position.y)));
        fields.add_field_method_get("advance", |_, g| {
            g.with(|r| glyph_value(&r.advances, g.glyph))
        });
        fields.add_field_method_get("textIndex", |_, g| {
            g.with(|r| one_based(glyph_value(&r.text_indices, g.glyph)?))
        });
        fields.add_field_method_get("direction", |_, g| g.with(|r| Ok(direction(r.dir()))));
        fields.add_field_method_get("isColor", |_, g| {
            g.with(|r| Ok(r.glyph_render_kind(g.glyph as usize) == GlyphRenderKind::Color))
        });
        fields.add_field_method_get("transform", |lua, g| {
            lua.create_userdata(ScriptedMat2D(nuxie_render_api::Mat2D(
                *g.with(|r| Ok(RawText::glyph_transform(r, g.position)))?
                    .values(),
            )))
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("path", |lua, g, ()| {
            let path = g.with(|r| {
                let mut path = RawPath::default();
                if r.glyph_render_kind(g.glyph as usize) == GlyphRenderKind::Suppressed {
                    return Ok(path);
                }
                path.add_path(
                    &RawText::glyph_path(
                        run_font(r)?.as_ref(),
                        glyph_value(&r.glyphs, g.glyph)?,
                    ),
                    Some(&RawText::glyph_transform(r, g.position)),
                );
                Ok(path)
            })?;
            push_path(lua, path)
        });
    }
}
pub(super) fn register(lua: &Lua) -> Result<()> {
    let table = lua.create_table();
    table.set(
        "new",
        lua.create_function(|lua, ()| {
            let factory = RendererBindings::for_lua(lua)
                .ok_or_else(|| Error::runtime("missing renderer factory"))?
                .with_factory(|factory| {
                    RuntimeFactoryHandle::from_factory(factory)
                        .ok_or_else(|| Error::runtime("retained factory required"))
                })?;
            let userdata = lua.create_userdata(ScriptedText {
                layout: Rc::new(RefCell::new(TextLayout::new(RawText::new(factory)))),
            })?;
            super::lua_font::source_metatable(
                lua,
                &userdata,
                "Text",
                &[
                    "sizing",
                    "overflow",
                    "align",
                    "wrap",
                    "wordBreak",
                    "origin",
                    "direction",
                    "maxWidth",
                    "maxHeight",
                    "paragraphSpacing",
                    "length",
                    "isEmpty",
                    "lineCount",
                ],
                &[
                    "append",
                    "clear",
                    "draw",
                    "bounds",
                    "line",
                    "lines",
                    "hitTest",
                    "caret",
                    "selectionRects",
                ],
                Some(&[
                    "sizing",
                    "overflow",
                    "align",
                    "wrap",
                    "wordBreak",
                    "origin",
                    "direction",
                    "maxWidth",
                    "maxHeight",
                    "paragraphSpacing",
                ]),
                |u| {
                    let _ = u.borrow::<ScriptedText>()?;
                    Ok(())
                },
            )?;
            Ok(userdata)
        })?,
    )?;
    lua.globals().set("Text", table)
}

fn push_line(lua: &Lua, line: ScriptedTextLine) -> Result<AnyUserData> {
    let userdata = lua.create_userdata(line)?;
    super::lua_font::source_metatable(
        lua,
        &userdata,
        "TextLine",
        &[
            "index",
            "x",
            "top",
            "baseline",
            "bottom",
            "width",
            "direction",
            "firstIndex",
            "lastIndex",
            "glyphCount",
        ],
        &["glyph", "glyphs"],
        None,
        |u| u.borrow::<ScriptedTextLine>()?.with(|_| Ok(())),
    )?;
    Ok(userdata)
}
fn push_glyph(lua: &Lua, glyph: ScriptedGlyph) -> Result<AnyUserData> {
    let userdata = lua.create_userdata(glyph)?;
    super::lua_font::source_metatable(
        lua,
        &userdata,
        "Glyph",
        &[
            "id",
            "font",
            "size",
            "x",
            "y",
            "advance",
            "textIndex",
            "direction",
            "isColor",
            "transform",
        ],
        &["path"],
        None,
        |u| u.borrow::<ScriptedGlyph>()?.with(|_| Ok(())),
    )?;
    Ok(userdata)
}

#[cfg(all(test, feature = "compiler"))]
mod host_cluster_tests {
    use super::*;
    use crate::vm::ScriptVm;
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::source::{
        text::font_hb::{HbFont, HostGlyphPicture, HostPictureScope},
        text_engine::TextRun,
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn scripted_host_cluster_reports_color_once_and_suppresses_other_paths() {
        let base = HbFont::decode(include_bytes!(
            "../../../../fixtures/fonts/ibm-plex-sans-arabic.ttf"
        ))
        .expect("fixture font");
        // B + acute does not compose; its second glyph has real outline ink.
        let text = [u32::from('B'), 0x301];
        let run = TextRun {
            font: Some(base.clone()),
            size: 20.0,
            line_height: -1.0,
            letter_spacing: 0.0,
            unichar_count: 2,
            script: 0,
            style_id: 0,
            level: 0,
            shape_at_size: false,
        };
        let shaped = base.shape_text(&text, &[run], 0);
        let glyphs = &shaped[0].runs[0];
        assert_eq!(glyphs.glyphs.len(), 2);
        assert_eq!(glyphs.text_indices, [0, 0]);
        assert!(!base.get_path(glyphs.glyphs[1]).empty());
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let host = base
            .as_any()
            .downcast_ref::<HbFont>()
            .unwrap()
            .with_host_pictures(
                Arc::new(move |_, cluster| {
                    assert_eq!(cluster, text);
                    count.fetch_add(1, Ordering::SeqCst);
                    Some(HostGlyphPicture {
                        png: vec![
                            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0,
                            1, 0, 0, 0, 1, 8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 11, 73, 68,
                            65, 84, 120, 156, 99, 248, 15, 4, 0, 9, 251, 3, 253, 251, 94, 107, 43,
                            0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
                        ],
                        width: 1,
                        height: 1,
                        bearing_x: 0.0,
                        bearing_y: -1.0,
                        extent_x: 1.0,
                        extent_y: 1.0,
                    })
                }),
                HostPictureScope::Cluster,
            );
        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        let globals = vm.lua().globals();
        let readonly = globals.is_readonly();
        globals.set_readonly(false);
        globals
            .set(
                "hostFont",
                super::super::lua_font::push_font(vm.lua(), host).unwrap(),
            )
            .unwrap();
        globals.set_readonly(readonly);
        vm.eval::<()>(
            r#"
            local text = Text.new()
            text:append("B́", {font = hostFont, size = 20})
            local line = text:line(1)
            assert(line.glyphCount == 2)
            local first = line:glyph(1)
            local second = line:glyph(2)
            -- Query the ancillary glyph first to exercise call-order independence.
            assert(not second.isColor)
            assert(#second:path() == 0)
            assert(first.isColor)
            assert(#first:path() > 0)
            assert(not second.isColor and #second:path() == 0)
            "#,
        )
        .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[cfg(test)]
mod epoch_tests {
    use super::advance_epoch;

    // A handle fetched at revision 5 was accepted again once the `u32`
    // revision wrapped back to 5 (2^32 changes later), against a layout it
    // no longer indexes. The epoch does not come back.
    #[test]
    fn a_wrapped_revision_does_not_make_a_stale_handle_fresh() {
        let fetched = 7;
        let mut epoch = fetched;
        let mut seen = 5_u32;
        for revision in [u32::MAX, 4, 5] {
            epoch = advance_epoch(epoch, seen, revision);
            seen = revision;
        }
        assert_eq!(seen, 5, "the revision is back where the handle saw it");
        assert_eq!(epoch, fetched + (1 << 32));
        assert_eq!(advance_epoch(9, 3, 3), 9, "no change keeps handles fresh");
    }
}
