//! Literal Luau producers from scripting color/input/listener/converter/condition,
//! renderer and text unit tests. No Wasm guest engine is involved.
use crate::scripted_source::ScriptVmSourceTestExt as _;
use anyhow::{Context, Result};
use luaur_rt::{Function, Table, Value};
use luaur_vm::{
    enums::lua_gc_op::lua_GCOp,
    functions::{lua_gc::lua_gc, lua_gettop::lua_gettop},
};
use nuxie_render_api::{PersistentFactory, SerializingFactory};
use nuxie_runtime::{
    RuntimeFactoryHandle,
    source::{
        math::vec2d::Vec2D, pointer_button::PointerButton,
        viewmodel::runtime::viewmodel_instance_runtime::ViewModelInstanceRuntime,
    },
};
use nuxie_scripting::vm::ScriptVm;
use std::path::Path;

pub(crate) fn supports(id: &str) -> bool {
    matches!(
        id,
        "script_color_data_test"
            | "script_input_color_trigger"
            | "scripted_listener_action"
            | "listener_action_inputs"
            | "script_string_converter"
            | "scripted_data_converter_bound_input"
            | "scripted_transition_condition"
            | "scripted_oval"
            | "scripted_animated_oval"
            | "scripted_text_wrapped"
            | "scripted_text_bidi"
            | "scripted_text_glyphs"
    )
}
fn lua_error(error: impl std::fmt::Display) -> anyhow::Error {
    anyhow::anyhow!("{error}")
}
fn standalone(id: &str, runtime: &Path) -> Result<Vec<u8>> {
    let vm = ScriptVm::new();
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    vm.install_render_factory(&mut factory).map_err(lua_error)?;
    vm.install_rive_globals().map_err(lua_error)?;
    let source = match id {
        "scripted_oval" => OVAL,
        "scripted_animated_oval" => ANIMATED,
        "scripted_text_wrapped" => WRAPPED,
        "scripted_text_bidi" => BIDI,
        "scripted_text_glyphs" => GLYPHS,
        _ => anyhow::bail!("unknown standalone {id}"),
    };
    if id.starts_with("scripted_text_") {
        let globals = vm.lua().globals();
        let readonly = globals.is_readonly();
        globals.set_readonly(false);
        let loaded = (|| -> Result<()> {
            let font: Table = globals.get("Font").map_err(lua_error)?;
            let decode: Function = font.get("decode").map_err(lua_error)?;
            for (name, path) in [
                ("latin", "assets/fonts/Inter_18pt-Regular.ttf"),
                ("arabic", "assets/IBMPlexSansArabic-Regular.ttf"),
            ] {
                let bytes = std::fs::read(runtime.join("tests/unit_tests").join(path))?;
                let value: Value = decode
                    .call(vm.lua().create_buffer(bytes).map_err(lua_error)?)
                    .map_err(lua_error)?;
                anyhow::ensure!(
                    matches!(value, Value::UserData(_)),
                    "font decode failed: {name}"
                );
                globals.set(name, value).map_err(lua_error)?;
            }
            Ok(())
        })();
        globals.set_readonly(readonly);
        loaded?;
    }
    vm.run_source_bytecode::<()>("test_source", source)
        .map_err(lua_error)?;
    let table: Table = vm
        .eval("return { draw = function(self, renderer) return render(renderer) end }")
        .map_err(lua_error)?;
    if id == "scripted_animated_oval" {
        let advance: Function = vm.lua().globals().get("advance").map_err(lua_error)?;
        let state = vm.lua().current_thread().state();
        for i in 0..1000 {
            let top = unsafe { lua_gettop(state) };
            if i != 0 {
                factory.borrow_mut().add_frame();
            }
            anyhow::ensure!(
                advance.call::<bool>(1.0_f32 / 60.0).map_err(lua_error)?,
                "advance returned false"
            );
            let mut renderer = factory.borrow().make_renderer();
            anyhow::ensure!(
                vm.upstream_test_call_draw_with_balance(&table, &mut factory, &mut renderer)
                    .map_err(lua_error)?,
                "unbalanced renderer"
            );
            anyhow::ensure!(top == unsafe { lua_gettop(state) }, "Lua stack changed");
            lua_gc(state, lua_GCOp::LUA_GCCOLLECT as i32, 0).map_err(|error| {
                anyhow::anyhow!("Luau collection failed: status {}", error.getStatus())
            })?;
        }
    } else {
        let mut renderer = factory.borrow().make_renderer();
        anyhow::ensure!(
            vm.upstream_test_call_draw_with_balance(&table, &mut factory, &mut renderer)
                .map_err(lua_error)?,
            "unbalanced renderer"
        );
    }
    let bytes = factory.borrow().bytes().to_vec();
    Ok(bytes)
}

pub(crate) fn run(case: &crate::Case, runtime: &Path) -> Result<Vec<u8>> {
    anyhow::ensure!(
        supports(&case.id),
        "unsupported scripted misc producer {}",
        case.id
    );
    if matches!(
        case.id.as_str(),
        "scripted_oval"
            | "scripted_animated_oval"
            | "scripted_text_wrapped"
            | "scripted_text_bidi"
            | "scripted_text_glyphs"
    ) {
        anyhow::ensure!(
            case.source == "inline-script" && case.artboard == "cpp-test-defined",
            "standalone producer source/artboard mismatch"
        );
        return standalone(&case.id, runtime);
    }
    let (source, board, mode, initial) = match case.id.as_str() {
        "script_color_data_test" => ("script_color_data_test.riv", None, 0, 0.0),
        "script_input_color_trigger" => ("script_inputs_test_1.riv", Some("Artboard"), 1, 0.1),
        "scripted_listener_action" => ("scripted_listener_action.riv", None, 2, 0.1),
        "listener_action_inputs" => ("listener_action_inputs.riv", None, 1, 0.016),
        "script_string_converter" => (
            "script_string_converter_test.riv",
            Some("Converter"),
            1,
            0.1,
        ),
        "scripted_data_converter_bound_input" => {
            ("scripted_data_converter_bound_input.riv", None, 0, 0.1)
        }
        "scripted_transition_condition" => ("scripted_transition_condition.riv", None, 2, 0.1),
        _ => unreachable!(),
    };
    anyhow::ensure!(
        case.source == source && case.artboard == board.unwrap_or("default"),
        "scripted producer metadata differs: {}",
        case.id
    );
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).context("retained factory")?;
    let file = crate::scripting::import_file(
        &std::fs::read(runtime.join("tests/unit_tests/assets").join(source))?,
        retained,
    )?;
    let artboard = file
        .with_file(|f| match board {
            Some(name) => f.artboard_named(name),
            None => f.artboard_default(),
        })
        .context("artboard")?;
    let (width, height, id) =
        artboard.with_artboard(|a| (a.width(), a.height(), a.base.view_model_id()));
    factory.borrow_mut().frame_size(width as u32, height as u32);
    let machine = artboard.state_machine_at(0).context("state machine")?;
    let root = file
        .with_file_mut(|f| match mode {
            0 => f.create_default_view_model_instance_for_artboard(artboard.core_handle()),
            1 if id != u32::MAX => f.create_view_model_instance_at(id as usize, 0),
            _ => f.create_view_model_instance_for_artboard(artboard.core_handle()),
        })
        .context("authored VMI creation")?;
    machine.with_instance_mut(|m| m.bind_view_model_instance(root.clone()));
    let vmi = ViewModelInstanceRuntime::new(root);
    // Listener inputs creates its renderer before the first advance; all other
    // producers create it after their initialization advance.
    let mut renderer = if case.id == "listener_action_inputs" {
        Some(factory.borrow().make_renderer())
    } else {
        None
    };
    machine.advance_and_apply(initial);
    let condition_properties = if case.id == "scripted_transition_condition" {
        Some([
            vmi.property_boolean("timelineBool")
                .context("timelineBool")?,
            vmi.property_boolean("anyStateBool")
                .context("anyStateBool")?,
        ])
    } else {
        None
    };
    let renderer = renderer.get_or_insert_with(|| factory.borrow().make_renderer());
    if case.id == "script_color_data_test" {
        machine.advance_and_apply(0.016);
    }
    artboard.draw(renderer);
    match case.id.as_str() {
        "script_color_data_test" => {}
        "script_input_color_trigger" => {
            let trigger = vmi.property_trigger("Trigger").context("Trigger")?;
            for _ in 0..30 {
                trigger.trigger();
                factory.borrow_mut().add_frame();
                machine.advance_and_apply(0.016);
                artboard.draw(renderer);
            }
        }
        "scripted_listener_action" => {
            factory.borrow_mut().add_frame();
            for (x, id) in [(200.0, 1), (300.0, 2), (400.0, 3)] {
                machine.with_instance_mut(|m| {
                    m.pointer_down(Vec2D::new(x, 20.0), id, PointerButton::Primary)
                });
                machine.with_instance_mut(|m| {
                    m.pointer_up(Vec2D::new(x, 20.0), id, PointerButton::Primary)
                });
                machine.advance_and_apply(0.016);
                artboard.draw(renderer);
            }
        }
        "listener_action_inputs" => {
            let position =
                artboard.with_artboard(|a| Vec2D::new(a.width() / 2.0, a.height() / 2.0));
            machine.with_instance_mut(|m| m.pointer_down(position, 3, PointerButton::Primary));
            let position =
                artboard.with_artboard(|a| Vec2D::new(a.width() / 2.0, a.height() / 2.0));
            machine.with_instance_mut(|m| m.pointer_up(position, 3, PointerButton::Primary));
            factory.borrow_mut().add_frame();
            machine.advance_and_apply(0.016);
            artboard.draw(renderer);
        }
        "script_string_converter" => {
            for (name, value) in [
                ("Field1", "H#e%l&l*o"),
                ("Field2", "____one two three___"),
                ("Field3", "  **This uses a string converter@@. "),
                ("Field4", "It strips special characters like *&^%$#@!)()"),
            ] {
                vmi.property_string(name)
                    .with_context(|| format!("string {name}"))?
                    .set_value(value);
                factory.borrow_mut().add_frame();
                machine.advance_and_apply(0.016);
                artboard.draw(renderer);
            }
        }
        "scripted_data_converter_bound_input" => {
            factory.borrow_mut().add_frame();
            machine.advance_and_apply(0.1);
            artboard.draw(renderer);
        }
        "scripted_transition_condition" => {
            for property in condition_properties.context("condition properties")? {
                factory.borrow_mut().add_frame();
                property.set_value(true);
                machine.advance_and_apply(0.016);
                artboard.draw(renderer);
            }
        }
        _ => unreachable!(),
    }
    let bytes = factory.borrow().bytes().to_vec();
    Ok(bytes)
}
const OVAL: &str = r###"function addOval(path: Path, x: number, y: number, width: number, height: number)
	local c: number = 0.5519150244935105707435627
	local unit: { Vector } = {
		Vector.xy(1, 0),
		Vector.xy(1, c),
		Vector.xy(c, 1), -- quadrant 1 ( 4:30)
		Vector.xy(0, 1),
		Vector.xy(-c, 1),
		Vector.xy(-1, c), -- quadrant 2 ( 7:30)
		Vector.xy(-1, 0),
		Vector.xy(-1, -c),
		Vector.xy(-c, -1), -- quadrant 3 (10:30)
		Vector.xy(0, -1),
		Vector.xy(c, -1),
		Vector.xy(1, -c), -- quadrant 4 ( 1:30)
		Vector.xy(1, 0),
	}

	local dx: number = x - width / 2
	local dy: number = y - height / 2
	local sx: number = width * 0.5
	local sy: number = height * 0.5

	local map = function(p: Vector): Vector
		return Vector.xy(p.x * sx + dx, p.y * sy + dy)
	end
	path:moveTo(map(unit[1]))
	for i = 1, 12, 3 do
		path:cubicTo(map(unit[i + 1]), map(unit[i + 2]), map(unit[i + 3]))
	end
	path:close()
end

function render(renderer: Renderer): ()
	local path: Path = Path.new()
	local paint: Paint = Paint.with({color=0xFFFF0000, feather=20})

	addOval(path, 600, 500, 100, 180)
	renderer:drawPath(path, paint)
end
"###;
const ANIMATED: &str = r###"function addOval(path: Path, x: number, y: number, width: number, height: number)
	local c: number = 0.5519150244935105707435627
	local unit: { Vector } = {
		Vector.xy(1, 0),
		Vector.xy(1, c),
		Vector.xy(c, 1), -- quadrant 1 ( 4:30)
		Vector.xy(0, 1),
		Vector.xy(-c, 1),
		Vector.xy(-1, c), -- quadrant 2 ( 7:30)
		Vector.xy(-1, 0),
		Vector.xy(-1, -c),
		Vector.xy(-c, -1), -- quadrant 3 (10:30)
		Vector.xy(0, -1),
		Vector.xy(c, -1),
		Vector.xy(1, -c), -- quadrant 4 ( 1:30)
		Vector.xy(1, 0),
	}

	local dx: number = x - width / 2
	local dy: number = y - height / 2
	local sx: number = width * 0.5
	local sy: number = height * 0.5

	local map = function(p: Vector): Vector
		return Vector.xy(p.x * sx + dx, p.y * sy + dy)
	end
	path:moveTo(map(unit[1]))
	for i = 1, 12, 3 do
		path:cubicTo(map(unit[i + 1]), map(unit[i + 2]), map(unit[i + 3]))
	end
	path:close()
end

local path: Path = Path.new()
local rotation:number = 0
local paint: Paint = Paint.with({color=0xFFFF0000, feather=20})

function advance(seconds:number): boolean
    path:reset()
    addOval(path, 600, 500, 100, 180)
    rotation += seconds
    return true
end

function render(renderer: Renderer): ()
	renderer:save()
    renderer:transform(Mat2D.withRotation(rotation))
	renderer:drawPath(path, paint)
     renderer:restore()
end
"###;
const WRAPPED: &str = r###"
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
    "###;
const BIDI: &str = r###"
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
    "###;
const GLYPHS: &str = r###"
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
    "###;
