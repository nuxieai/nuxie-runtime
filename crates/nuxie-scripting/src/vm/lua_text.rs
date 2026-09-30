//! src/lua/renderer/lua_text.cpp and include/rive/lua/scripted_text.hpp at db31ea87.
//! Retained layout plus revision-checked indices replace interior C++ pointers.
use super::{
    lua_font::ScriptedFont, lua_mat2d::ScriptedMat2D, lua_paint::ScriptedPaint,
    lua_path::ScriptedPath, lua_renderer::ScriptedRenderer, lua_renderer_library::RendererBindings,
};
use luaur_rt::{
    AnyUserData, Error, Lua, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods,
    Value, Vector,
};
use nuxie_runtime::{
    RuntimeFactoryHandle,
    source::{
        math::{raw_path::RawPath, vec2d::Vec2D},
        renderer::to_render_raw_path,
        text::{
            cursor::{Cursor, CursorPosition},
            raw_text::RawText,
        },
        text_engine::{
            GlyphRun, TextDirection, TextOrigin, TextOverflow, TextSizing, TextWordBreak, TextWrap,
        },
    },
};
use std::{cell::RefCell, rc::Rc};

type Layout = Rc<RefCell<RawText>>;
pub(super) struct ScriptedText {
    layout: Layout,
}
#[derive(Clone)]
struct ScriptedTextLine {
    layout: Layout,
    revision: u32,
    index: u32,
}
struct ScriptedGlyph {
    layout: Layout,
    revision: u32,
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
fn fresh(text: &RawText, revision: u32) -> Result<()> {
    if text.revision() != revision {
        Err(Error::runtime(
            "Text changed since this was fetched, fetch it again.",
        ))
    } else {
        Ok(())
    }
}

// Preserve the upstream UTF decoder's permissive, byte-oriented semantics.
// Reject only its asserted-invalid leads or reads beyond the provided bytes;
// never inspect a tail after the terminating NUL lead byte.
fn check_text_bytes(mut bytes: &[u8]) -> Result<()> {
    while let Some(&lead) = bytes.first().filter(|&&byte| byte != 0) {
        let count = if lead < 0x80 {
            1
        } else {
            lead.leading_ones() as usize
        };
        if !(1..=4).contains(&count) || lead & 0xc0 == 0x80 || count > bytes.len() {
            return Err(Error::runtime(
                "Text contains an invalid or truncated UTF-8 sequence.",
            ));
        }
        bytes = &bytes[count..];
    }
    Ok(())
}
fn vector(p: Vec2D) -> Vector {
    Vector::new(p.x, p.y, 0.0)
}
fn push_path(lua: &Lua, path: RawPath) -> Result<AnyUserData> {
    lua.create_userdata(ScriptedPath::from_render_raw_path(to_render_raw_path(
        &path,
    )))
}
fn direction(value: TextDirection) -> &'static str {
    DIRECTION[(value as i32 + 1) as usize]
}
impl ScriptedText {
    fn line(&self, index: u32) -> ScriptedTextLine {
        ScriptedTextLine {
            layout: self.layout.clone(),
            revision: self.layout.borrow().revision(),
            index,
        }
    }
}
impl UserData for ScriptedText {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("sizing", |_, t| match t.layout.borrow().sizing() {
            TextSizing::AutoWidth => Ok("autoWidth"),
            TextSizing::AutoHeight => Ok("autoHeight"),
            TextSizing::Fixed => Ok("fixed"),
            TextSizing::Unknown(_) => Err(Error::runtime("Text has an invalid sizing value")),
        });
        fields.add_field_method_get("overflow", |_, t| match t.layout.borrow().overflow() {
            TextOverflow::Visible => Ok("visible"),
            TextOverflow::Hidden => Ok("hidden"),
            TextOverflow::Clipped => Ok("clipped"),
            TextOverflow::Ellipsis => Ok("ellipsis"),
            TextOverflow::Fit | TextOverflow::FitFontSize | TextOverflow::Unknown(_) => {
                Err(Error::runtime("Text has an invalid overflow value"))
            }
        });
        fields.add_field_method_get("align", |_, t| {
            Ok(ALIGN[t.layout.borrow().align_index() as usize])
        });
        fields.add_field_method_get("wrap", |_, t| match t.layout.borrow().wrap() {
            TextWrap::Wrap => Ok("wrap"),
            TextWrap::NoWrap => Ok("noWrap"),
            TextWrap::Unknown(_) => Err(Error::runtime("Text has an invalid wrap value")),
        });
        fields.add_field_method_get("wordBreak", |_, t| match t.layout.borrow().word_break() {
            TextWordBreak::BreakWord => Ok("breakWord"),
            TextWordBreak::Normal => Ok("normal"),
            TextWordBreak::BreakAll => Ok("breakAll"),
            TextWordBreak::Unknown(_) => Err(Error::runtime("Text has an invalid wordBreak value")),
        });
        fields.add_field_method_get("origin", |_, t| match t.layout.borrow().origin() {
            TextOrigin::Top => Ok("top"),
            TextOrigin::Baseline => Ok("baseline"),
            TextOrigin::Unknown(_) => Err(Error::runtime("Text has an invalid origin value")),
        });
        fields.add_field_method_get("direction", |_, t| {
            Ok(DIRECTION[t.layout.borrow().direction_index() as usize])
        });
        fields.add_field_method_get("maxWidth", |_, t| Ok(t.layout.borrow().max_width()));
        fields.add_field_method_get("maxHeight", |_, t| Ok(t.layout.borrow().max_height()));
        fields.add_field_method_get("paragraphSpacing", |_, t| {
            Ok(t.layout.borrow().paragraph_spacing())
        });
        fields.add_field_method_get("length", |_, t| Ok(t.layout.borrow().length()));
        fields.add_field_method_get("isEmpty", |_, t| Ok(t.layout.borrow().empty()));
        fields.add_field_method_get("lineCount", |_, t| {
            Ok(t.layout.borrow_mut().ordered_lines().len())
        });
        fields.add_field_method_set("sizing", |_, t, v: String| {
            t.layout
                .borrow_mut()
                .set_sizing_index(enum_index(&v, SIZING, "TextSizing")?);
            Ok(())
        });
        fields.add_field_method_set("overflow", |_, t, v: String| {
            t.layout
                .borrow_mut()
                .set_overflow_index(enum_index(&v, OVERFLOW, "TextOverflow")?);
            Ok(())
        });
        fields.add_field_method_set("align", |_, t, v: String| {
            t.layout
                .borrow_mut()
                .set_align_index(enum_index(&v, ALIGN, "TextAlign")?);
            Ok(())
        });
        fields.add_field_method_set("wrap", |_, t, v: String| {
            t.layout
                .borrow_mut()
                .set_wrap_index(enum_index(&v, WRAP, "TextWrap")?);
            Ok(())
        });
        fields.add_field_method_set("wordBreak", |_, t, v: String| {
            t.layout.borrow_mut().set_word_break_index(enum_index(
                &v,
                WORD_BREAK,
                "TextWordBreak",
            )?);
            Ok(())
        });
        fields.add_field_method_set("origin", |_, t, v: String| {
            t.layout
                .borrow_mut()
                .set_origin_index(enum_index(&v, ORIGIN, "TextOrigin")?);
            Ok(())
        });
        fields.add_field_method_set("direction", |_, t, v: String| {
            t.layout
                .borrow_mut()
                .set_direction_index(enum_index(&v, DIRECTION, "TextDirection")?);
            Ok(())
        });
        fields.add_field_method_set("maxWidth", |_, t, v: f32| {
            t.layout.borrow_mut().set_max_width(v);
            Ok(())
        });
        fields.add_field_method_set("maxHeight", |_, t, v: f32| {
            t.layout.borrow_mut().set_max_height(v);
            Ok(())
        });
        fields.add_field_method_set("paragraphSpacing", |_, t, v: f32| {
            t.layout.borrow_mut().set_paragraph_spacing(v);
            Ok(())
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method(
            "append",
            |lua, t, (chars, style): (luaur_rt::LuaString, Table)| {
                let bytes = chars.as_bytes();
                check_text_bytes(&bytes)?;
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
                t.layout.borrow_mut().append(
                    &bytes,
                    paint,
                    font,
                    style.raw_get::<Option<f32>>("size")?.unwrap_or(16.0),
                    style.raw_get::<Option<f32>>("lineHeight")?.unwrap_or(-1.0),
                    style
                        .raw_get::<Option<f32>>("letterSpacing")?
                        .unwrap_or(0.0),
                    foreground,
                );
                Ok(())
            },
        );
        methods.add_method("clear", |_, t, ()| {
            t.layout.borrow_mut().clear();
            Ok(())
        });
        methods.add_method(
            "draw",
            |_, t, (renderer, paint): (AnyUserData, Option<AnyUserData>)| {
                let paint = paint
                    .map(|p| p.borrow::<ScriptedPaint>().map(|p| p.render_paint.clone()))
                    .transpose()?;
                renderer
                    .borrow::<ScriptedRenderer>()?
                    .with_renderer_mut(|renderer| {
                        t.layout.borrow_mut().render(renderer, paint);
                        Ok(())
                    })
            },
        );
        methods.add_method("bounds", |_, t, ()| {
            let b = t.layout.borrow_mut().bounds();
            Ok((
                vector(Vec2D::new(b.min_x, b.min_y)),
                vector(Vec2D::new(b.max_x, b.max_y)),
            ))
        });
        methods.add_method("line", |lua, t, index: i32| {
            let count = t.layout.borrow_mut().ordered_lines().len();
            if index < 1 || index as usize > count {
                return Err(Error::runtime(format!("line {index} is out of range")));
            }
            push_line(lua, t.line(index as u32 - 1))
        });
        methods.add_method("lines", |lua, t, ()| {
            let count = t.layout.borrow_mut().ordered_lines().len();
            let lines = lua.create_table();
            for i in 0..count {
                lines.raw_set(i + 1, push_line(lua, t.line(i as u32))?)?;
            }
            Ok(lines)
        });
        methods.add_method("hitTest", |_, t, p: Vector| {
            let mut text = t.layout.borrow_mut();
            let view = text.layout_view();
            Ok(
                CursorPosition::from_translation(Vec2D::new(p.x(), p.y()), &view)
                    .code_point_index()
                    + 1,
            )
        });
        methods.add_method("caret", |_, t, index: i32| {
            let mut text = t.layout.borrow_mut();
            let view = text.layout_view();
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
            let mut text = t.layout.borrow_mut();
            let view = text.layout_view();
            let cursor = Cursor::new(
                CursorPosition::at_index(from.saturating_sub(1).max(0) as u32, &view),
                CursorPosition::at_index(to.saturating_sub(1).max(0) as u32, &view),
            );
            let mut rects = Vec::new();
            cursor.selection_rects(&mut rects, &view);
            let table = lua.create_table();
            for (i, r) in rects.into_iter().enumerate() {
                let row = lua.create_table();
                row.set("min", vector(Vec2D::new(r.min_x, r.min_y)))?;
                row.set("max", vector(Vec2D::new(r.max_x, r.max_y)))?;
                table.raw_set(i + 1, row)?;
            }
            Ok(table)
        });
    }
}

impl ScriptedTextLine {
    fn with<R>(&self, f: impl FnOnce(&mut RawText) -> R) -> Result<R> {
        let mut text = self.layout.borrow_mut();
        fresh(&text, self.revision)?;
        Ok(f(&mut text))
    }
    fn glyphs(&self) -> Result<Vec<ScriptedGlyph>> {
        self.with(|text| {
            let line = &text.ordered_lines()[self.index as usize];
            let mut itr = line.begin();
            let end = line.end();
            let mut x = line.glyph_line().start_x;
            let mut result = Vec::new();
            while itr != end {
                let run = itr.run();
                let glyph = itr.glyph_index();
                let index = line
                    .runs()
                    .position(|candidate| std::ptr::eq(candidate, run))
                    .unwrap();
                let offset = run.offsets[glyph as usize];
                result.push(ScriptedGlyph {
                    layout: self.layout.clone(),
                    revision: self.revision,
                    line: self.index,
                    run: index,
                    glyph,
                    position: Vec2D::new(x + offset.x, line.y() + offset.y),
                });
                x += run.advances[glyph as usize];
                itr.advance();
            }
            result
        })
    }
}
impl UserData for ScriptedTextLine {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("index", |_, l| l.with(|_| l.index + 1));
        fields.add_field_method_get("x", |_, l| {
            l.with(|t| t.ordered_lines()[l.index as usize].glyph_line().start_x)
        });
        fields.add_field_method_get("top", |_, l| l.with(|t| t.line_top(l.index)));
        fields.add_field_method_get("baseline", |_, l| {
            l.with(|t| t.ordered_lines()[l.index as usize].y())
        });
        fields.add_field_method_get("bottom", |_, l| {
            l.with(|t| t.ordered_lines()[l.index as usize].bottom())
        });
        fields.add_field_method_get("width", |_, l| {
            l.with(|t| {
                let mut width = 0.0;
                t.for_each_glyph(l.index, |r, i, _| {
                    width += r.advances[i as usize];
                    true
                });
                width
            })
        });
        fields.add_field_method_get("direction", |_, l| {
            l.with(|t| direction(t.line_direction(l.index)))
        });
        fields.add_field_method_get("firstIndex", |_, l| {
            l.with(|t| {
                let view = t.layout_view();
                view.ordered_lines()[l.index as usize].first_code_point_index(view.glyph_lookup())
                    + 1
            })
        });
        fields.add_field_method_get("lastIndex", |_, l| {
            l.with(|t| {
                let view = t.layout_view();
                view.ordered_lines()[l.index as usize].last_code_point_index(view.glyph_lookup())
                    + 1
            })
        });
        fields.add_field_method_get("glyphCount", |_, l| l.with(|t| t.line_glyph_count(l.index)));
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
            for (i, g) in l.glyphs()?.into_iter().enumerate() {
                table.raw_set(i + 1, push_glyph(lua, g)?)?;
            }
            Ok(table)
        });
        methods.add_meta_method("__iter", |lua, l, ()| {
            let (glyph, x) = l.with(|t| {
                let line = &t.ordered_lines()[l.index as usize];
                (line.start_glyph_index(0), line.glyph_line().start_x)
            })?;
            let mut cursor = ScriptedGlyphCursor {
                line: l.clone(),
                run: 0,
                glyph,
                x,
            };
            cursor.skip_empty_runs()?;
            let next = lua.create_function(|lua, (cursor, index): (AnyUserData, i32)| {
                let mut c = cursor.borrow_mut::<ScriptedGlyphCursor>()?;
                let glyph = c.next()?;
                match glyph {
                    Some(g) => Ok(MultiValue::from_vec(vec![
                        Value::Integer(i64::from(index) + 1),
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
        let mut text = self.line.layout.borrow_mut();
        fresh(&text, self.line.revision)?;
        let line = &text.ordered_lines()[self.line.index as usize];
        while self.glyph == line.end_glyph_index(self.run) && self.run + 1 < line.runs().len() {
            self.run += 1;
            self.glyph = line.start_glyph_index(self.run);
        }
        Ok(())
    }
    fn next(&mut self) -> Result<Option<ScriptedGlyph>> {
        let result = {
            let mut text = self.line.layout.borrow_mut();
            fresh(&text, self.line.revision)?;
            let line = &text.ordered_lines()[self.line.index as usize];
            if self.run + 1 >= line.runs().len() && self.glyph == line.end_glyph_index(self.run) {
                return Ok(None);
            }
            let run = line.runs().nth(self.run).unwrap();
            let i = self.glyph as usize;
            let offset = run.offsets[i];
            let glyph = ScriptedGlyph {
                layout: self.line.layout.clone(),
                revision: self.line.revision,
                line: self.line.index,
                run: self.run,
                glyph: self.glyph,
                position: Vec2D::new(self.x + offset.x, line.y() + offset.y),
            };
            self.x += run.advances[i];
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
    fn with<R>(&self, f: impl FnOnce(&GlyphRun) -> R) -> Result<R> {
        let mut text = self.layout.borrow_mut();
        fresh(&text, self.revision)?;
        Ok(f(text.ordered_lines()[self.line as usize]
            .runs()
            .nth(self.run)
            .unwrap()))
    }
}
impl UserData for ScriptedGlyph {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, g| g.with(|r| r.glyphs[g.glyph as usize]));
        fields.add_field_method_get("font", |lua, g| {
            super::lua_font::push_font(lua, g.with(|r| r.font.clone().expect("glyph run font"))?)
        });
        fields.add_field_method_get("size", |_, g| g.with(|r| r.size));
        fields.add_field_method_get("x", |_, g| g.with(|_| g.position.x));
        fields.add_field_method_get("y", |_, g| g.with(|_| g.position.y));
        fields.add_field_method_get("advance", |_, g| g.with(|r| r.advances[g.glyph as usize]));
        fields.add_field_method_get("textIndex", |_, g| {
            g.with(|r| r.text_indices[g.glyph as usize] + 1)
        });
        fields.add_field_method_get("direction", |_, g| g.with(|r| direction(r.dir())));
        fields.add_field_method_get("isColor", |_, g| {
            g.with(|r| {
                r.font
                    .as_ref()
                    .expect("glyph run font")
                    .is_color_glyph(r.glyphs[g.glyph as usize])
            })
        });
        fields.add_field_method_get("transform", |lua, g| {
            lua.create_userdata(ScriptedMat2D(nuxie_render_api::Mat2D(
                *g.with(|r| RawText::glyph_transform(r, g.position))?
                    .values(),
            )))
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("path", |lua, g, ()| {
            let path = g.with(|r| {
                let mut path = RawPath::default();
                path.add_path(
                    &RawText::glyph_path(
                        r.font.as_ref().expect("glyph run font").as_ref(),
                        r.glyphs[g.glyph as usize],
                    ),
                    Some(&RawText::glyph_transform(r, g.position)),
                );
                path
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
                layout: Rc::new(RefCell::new(RawText::new(factory))),
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
        |u| u.borrow::<ScriptedTextLine>()?.with(|_| ()),
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
        |u| u.borrow::<ScriptedGlyph>()?.with(|_| ()),
    )?;
    Ok(userdata)
}
