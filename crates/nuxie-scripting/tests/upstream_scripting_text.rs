//! Literal native-Luau cases from runtime/scripting/scripting_text_test.cpp at db31ea87.
#![cfg(all(feature = "luau", feature = "upstream-test-seams"))]
use luaur_rt::{Function, Table, Value};
use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_scripting::vm::ScriptVm;
use nuxie_sriv::{compare_sriv, parse_sriv};
use std::path::PathBuf;
mod support;
use support::ScriptVmSourceTestExt as _;

#[test]
fn text_append_preserves_upstream_nul_and_permissive_decoder_behavior() {
    let _test = TextTest::new(
        r#"
        local text = Text.new()
        text:append("a\0" .. string.char(255), {font = latin})
        assert(text.length == 1)
        local line = text:line(1)
        text:append("\0" .. string.char(255), {font = latin})
        assert(text.length == 1)
        assert(not pcall(function() return line.width end))
        text:clear()
        text:append("\0" .. string.char(255), {font = latin})
        assert(text.length == 0)
        assert(not text.isEmpty)
        text:clear()
        -- UTF::NextUTF8 accepts complete overlong sequences, masking payload
        -- bytes without Unicode validation. C0 AF represents one slash.
        text:append(string.char(192, 175), {font = latin})
        assert(text.length == 1)
        local ordinary = Text.new()
        ordinary:append("/", {font = latin})
        assert(text:line(1):glyph(1).id == ordinary:line(1):glyph(1).id)
    "#,
        true,
    );
}

fn asset(path: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    std::fs::read(PathBuf::from(root).join("tests/unit_tests").join(path)).unwrap()
}

struct TextTest {
    vm: ScriptVm,
    factory: PersistentFactory<SerializingFactory>,
}
impl TextTest {
    fn new(source: &str, seed_fonts: bool) -> Self {
        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(SerializingFactory::new());
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        let globals = vm.lua().globals();
        let readonly = globals.is_readonly();
        globals.set_readonly(false);
        if seed_fonts {
            let font: Table = globals.get("Font").unwrap();
            let decode: Function = font.get("decode").unwrap();
            for (name, path) in [
                ("latin", "assets/fonts/Inter_18pt-Regular.ttf"),
                ("arabic", "assets/IBMPlexSansArabic-Regular.ttf"),
            ] {
                let value: Value = decode
                    .call(vm.lua().create_buffer(asset(path)).unwrap())
                    .unwrap();
                assert!(matches!(value, Value::UserData(_)));
                globals.set(name, value).unwrap();
            }
        } else {
            globals
                .set(
                    "fontBytes",
                    vm.lua()
                        .create_buffer(asset("assets/fonts/Inter_18pt-Regular.ttf"))
                        .unwrap(),
                )
                .unwrap();
        }
        globals.set_readonly(readonly);
        vm.eval::<()>(source).unwrap();
        Self { vm, factory }
    }

    fn draw_with(&mut self, silver: &str) {
        let table: Table = self
            .vm
            .eval("return { draw = function(self, renderer) return render(renderer) end }")
            .unwrap();
        let mut renderer = self.factory.borrow().make_renderer();
        assert!(
            self.vm
                .upstream_test_call_draw_with_balance(&table, &mut self.factory, &mut renderer)
                .unwrap()
        );
        let expected = parse_sriv(&asset(&format!("silvers/{silver}.sriv"))).unwrap();
        let bytes = self.factory.borrow().bytes().to_vec();
        let actual = parse_sriv(&bytes).unwrap();
        compare_sriv(&expected, &actual)
            .unwrap_or_else(|difference| panic!("{silver}: {difference}"));
    }
}

#[test]
fn text_lays_out_and_draws() {
    let mut test = TextTest::new(
        r#"
        local text = Text.new()
        local paint = Paint.with({ color = 0xFF202020 })
        text:append('Hello ', { font = latin, size = 24, paint = paint })
        text:append('world', { font = latin, size = 24, paint = paint })
        assert(text.length == 11)
        assert(not text.isEmpty)
        assert(text.lineCount == 1)
        local min, max = text:bounds()
        assert(min.x == 0 and max.x > 100, 'width ' .. max.x)
        assert(text.sizing == 'autoWidth' and text.align == 'left')
        text.sizing = 'autoHeight'
        text.maxWidth = 60
        assert(text.lineCount > 1, 'lines ' .. text.lineCount)
        function render(renderer: Renderer)
            text:draw(renderer)
        end
    "#,
        true,
    );
    test.draw_with("scripted_text_wrapped");
}

#[test]
fn text_orders_bidi_runs_visually() {
    let mut test = TextTest::new(
        r#"
        local text = Text.new()
        text:append('abc مرحبا def', { font = arabic, size = 20, paint = Paint.new() })
        assert(text.lineCount == 1)
        local line = text:line(1)
        assert(line.direction == 'ltr')
        local glyphs = line:glyphs()
        assert(#glyphs == line.glyphCount)
        local iterated = 0
        for _, glyph in line do
            iterated += 1
        end
        assert(iterated == #glyphs)
        local lastX = -math.huge
        local lastRtlIndex = math.huge
        local runs = {}
        for _, glyph in glyphs do
            assert(glyph.x >= lastX, 'glyphs must advance left to right')
            lastX = glyph.x
            if glyph.direction == 'rtl' then
                assert(glyph.textIndex < lastRtlIndex, 'rtl glyphs read right to left')
                lastRtlIndex = glyph.textIndex
            end
            if runs[#runs] ~= glyph.direction then
                runs[#runs + 1] = glyph.direction
            end
        end
        assert(#runs == 3 and runs[1] == 'ltr' and runs[2] == 'rtl' and runs[3] == 'ltr',
            table.concat(runs, ','))
        assert(glyphs[1].textIndex == 1)
        function render(renderer: Renderer)
            text:draw(renderer)
        end
    "#,
        true,
    );
    test.draw_with("scripted_text_bidi");
}

#[test]
fn text_resolves_start_and_end_against_the_paragraph_direction() {
    let _test = TextTest::new(
        r#"
        local text = Text.new()
        text:append('مرحبا', { font = arabic, size = 20, paint = Paint.new() })
        text.sizing = 'fixed'
        text.maxWidth = 300
        text.maxHeight = 100
        text.align = 'start'
        assert(text.align == 'start')
        assert(text:line(1).direction == 'rtl')
        assert(text:line(1).x > 100, 'rtl start sits on the right')
        text.align = 'end'
        assert(text:line(1).x == 0, 'rtl end sits on the left')
        text.direction = 'ltr'
        assert(text:line(1).x > 100, 'forced ltr moves end to the right')
        text.align = 'left'
        assert(text.align == 'left' and text:line(1).x == 0)
    "#,
        true,
    );
}

#[test]
fn text_carets_and_hit_tests_agree() {
    let _test = TextTest::new(
        r#"
        local text = Text.new()
        text:append('Hello', { font = latin, size = 20, paint = Paint.new() })
        local x1, top, bottom = text:caret(1)
        assert(x1 == 0 and top == 0 and bottom > top)
        local x4 = text:caret(4)
        assert(x4 > x1)
        assert(text:hitTest(Vector.xy(x4 + 1, (top + bottom) / 2)) == 4)
        assert(text:hitTest(Vector.xy(-10, 0)) == 1)
        local rects = text:selectionRects(1, 3)
        assert(#rects == 2 and rects[1].min.x == 0 and rects[2].max.x <= x4)
        assert(text:line(1).firstIndex == 1 and text:line(1).lastIndex == 5)
    "#,
        true,
    );
}

#[test]
fn text_lines_go_stale_when_the_text_changes() {
    let _test = TextTest::new(
        r#"
        local text = Text.new()
        text:append('Hello', { font = latin, size = 20 })
        local line = text:line(1)
        local glyph = line:glyph(1)
        text.maxWidth = 0
        assert(line.width > 0, 'setting the same value keeps the lines')
        text:append(' world', { font = latin, size = 20 })
        local ok, message = pcall(function() return line.width end)
        assert(not ok and message:find('fetch it again'), message)
        ok = pcall(function() return glyph.advance end)
        assert(not ok)
        assert(text:line(1).width > 0)
    "#,
        true,
    );
}

#[test]
fn cleared_text_reads_as_empty() {
    let _test = TextTest::new(
        r#"
        local text = Text.new()
        text:append('Hello world', { font = latin, size = 20 })
        assert(text.lineCount == 1)
        text:clear()
        assert(text.isEmpty and text.length == 0)
        assert(text.lineCount == 0 and #text:lines() == 0)
        assert(text:hitTest(Vector.xy(10, 5)) == 1)
        assert(text:caret(1) == nil)
        assert(#text:selectionRects(1, 3) == 0)
        local min, max = text:bounds()
        assert(min.x == 0 and max.x == 0 and max.y == 0)
        text:append('again', { font = latin, size = 20 })
        assert(text.lineCount == 1)
    "#,
        true,
    );
}

#[test]
fn font_exposes_metrics_options_and_outlines() {
    let _test = TextTest::new(
        r#"
        assert(latin.ascent < 0 and latin.descent > 0)
        assert(latin.capHeight < 0 and latin.xHeight < 0)
        assert(latin.weight == 400 and not latin.isItalic)
        assert(math.abs(latin:lineHeight(10) - (latin.descent - latin.ascent) * 10) < 1e-5)
        assert(latin:hasGlyph(65) and not latin:hasGlyph(0x0627))
        assert(arabic:hasGlyph(0x0627))
        assert(type(latin.axes) == 'table' and type(latin.features) == 'table')
        assert(#latin.features > 0 and #latin.features[1] == 4)
        local variant = latin:withOptions({ wght = 700 }, { liga = 0 })
        assert(variant.ascent == latin.ascent)
        local text = Text.new()
        text:append('A', { font = latin, size = 10 })
        local glyph = text:line(1):glyph(1)
        assert(#latin:glyphPath(glyph.id) > 0)
        assert(#glyph:path() > 0)
        assert(glyph.transform[1] == 10)
        assert(glyph.font.ascent == latin.ascent)
        assert(not glyph.isColor)
        assert(Font.decode(buffer.create(16)) == nil)
    "#,
        true,
    );
}

#[test]
fn font_tag_tables_reject_keys_that_are_not_strings() {
    let _test = TextTest::new(
        r#"
        local ok, message = pcall(function()
            return latin:withOptions({ [1] = 700 })
        end)
        assert(not ok and message:find('four letter strings'), message)
        ok, message = pcall(function()
            return latin:withOptions({ weight = 700 })
        end)
        assert(not ok and message:find('four letter tag'), message)
        assert(latin:withOptions({ wght = 700 }) ~= nil)
    "#,
        true,
    );
}

#[test]
fn font_decodes_from_bytes() {
    let _test = TextTest::new(
        r#"
        local font = Font.decode(fontBytes)
        assert(font and font.ascent < 0)
    "#,
        false,
    );
}

#[test]
fn glyphs_expose_their_placed_outline() {
    let mut test = TextTest::new(
        r#"
        local text = Text.new()
        local paint = Paint.with({ color = 0xFF0000FF })
        text:append('ab', { font = latin, size = 30, paint = paint })
        text:append('ج', { font = arabic, size = 30, paint = paint })
        function render(renderer: Renderer)
            for _, line in text:lines() do
                for _, glyph in line do
                    renderer:drawPath(glyph:path(), paint)
                    renderer:save()
                    renderer:transform(Mat2D.withTranslation(0, 40) * glyph.transform)
                    renderer:drawPath(glyph.font:glyphPath(glyph.id), paint)
                    renderer:restore()
                end
            end
        end
    "#,
        true,
    );
    test.draw_with("scripted_text_glyphs");
}

// Source-review regressions for lua_font.cpp and lua_text.cpp, beyond the
// ten literal upstream cases above.
#[test]
fn text_binding_preserves_source_error_and_unsigned_boundaries() {
    let _test = TextTest::new(
        r#"
        local text = Text.new()
        text:append('A', { font = latin, foregroundColor = -1 })
        local line = text:line(1)
        local glyph = line:glyph(1)
        assert(latin:hasGlyph(-1) == latin:hasGlyph(0xffffffff))
        assert(#latin:glyphPath(-1) == #latin:glyphPath(0xffffffff))
        for _, entry in {
            { latin, 'Font' }, { text, 'Text' },
            { line, 'TextLine' }, { glyph, 'Glyph' },
        } do
            local ok, message = pcall(function() return entry[1].missingField end)
            assert(not ok and message:find('is not a valid index of ' .. entry[2]), message)
        end
        for _, entry in {
            { latin, 'hasGlyph' }, { text, 'append' },
            { line, 'glyph' }, { glyph, 'path' },
        } do
            assert(not pcall(function() return entry[1][entry[2]] end))
        end
        local ok, message = pcall(function()
            return latin:withOptions({ [1] = 'not a number' })
        end)
        assert(not ok and message:find('four letter strings'), message)
        text:append('B', { font = latin })
        for _, value in { line, glyph } do
            ok, message = pcall(function() return value.missingField end)
            assert(not ok and message:find('fetch it again'), message)
        end
    "#,
        true,
    );
}
