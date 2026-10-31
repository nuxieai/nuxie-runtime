// Shared compatibility facade for renderer bindings plus the Lua Gradient
// binding, whose upstream row is outside owner-split cluster C4.
use std::rc::Rc;

use luaur_rt::{Lua, Result, Table, UserData, Value, Vector as LuaVector};
use nuxie_render_api::{ColorInt, RenderShader};

pub(super) use super::lua_path::call_path_effect_update;
pub(crate) use super::lua_renderer_library::RendererBindings;

impl RendererBindings {
    pub(super) fn install_gradient_global(&self, lua: &Lua) -> Result<()> {
        let table = lua.create_table();

        let bindings = self.clone();
        table.set(
            "linear",
            lua.create_function(
                move |lua, (from, to, stops): (LuaVector, LuaVector, Table)| {
                    let (colors, positions) = gradient_stops(stops)?;
                    let shader = bindings.with_factory(|factory| {
                        Ok(factory.make_linear_gradient(
                            from.x(),
                            from.y(),
                            to.x(),
                            to.y(),
                            &colors,
                            &positions,
                        ))
                    })?;
                    lua.create_userdata(ScriptedGradient(Rc::from(shader)))
                },
            )?,
        )?;

        let bindings = self.clone();
        table.set(
            "radial",
            lua.create_function(
                move |lua, (center, radius, stops): (LuaVector, f32, Table)| {
                    let (colors, positions) = gradient_stops(stops)?;
                    let shader = bindings.with_factory(|factory| {
                        Ok(factory.make_radial_gradient(
                            center.x(),
                            center.y(),
                            radius,
                            &colors,
                            &positions,
                        ))
                    })?;
                    lua.create_userdata(ScriptedGradient(Rc::from(shader)))
                },
            )?,
        )?;

        table.set_readonly(true);
        lua.globals().set("Gradient", table)?;
        Ok(())
    }
}

pub(super) struct ScriptedGradient(pub(super) Rc<dyn RenderShader>);

impl UserData for ScriptedGradient {}

fn gradient_stops(stops: Table) -> Result<(Vec<ColorInt>, Vec<f32>)> {
    let mut colors = Vec::with_capacity(stops.raw_len());
    let mut positions = Vec::with_capacity(stops.raw_len());
    let mut index = 1_i64;
    loop {
        let Value::Table(stop) = stops.raw_get::<Value>(index)? else {
            break;
        };
        positions.push(stop.get("position")?);
        let color = stop.get::<Value>("color")?;
        colors.push(super::lua_color::required_unsigned(
            &stop.lua(),
            Some(&color),
            "gradient stop color",
        )?);
        index += 1;
    }
    Ok((colors, positions))
}

#[cfg(all(test, feature = "compiler"))]
mod upstream_gradient_tests {
    use super::ScriptedGradient;
    use crate::vm::ScriptVm;
    use luaur_rt::AnyUserData;
    use nuxie_render_api::{NullFactory, PersistentFactory};

    // tests/unit_tests/runtime/scripting/scripting_gradient_test.cpp, 36780f92.
    #[test]
    fn gradient_takes_more_stops_than_the_lua_c_stack_limit() {
        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(NullFactory::new());
        vm.install_render_factory(&mut factory).unwrap();
        vm.install_rive_globals().unwrap();
        let gradient: AnyUserData = vm
            .eval(
                r#"local stops = {}
for i = 1, 9000 do
  stops[i] = { position = i / 9000, color = Color.rgba(255, 0, 0, 255) }
end
return Gradient.linear(Vector.xy(), Vector.xy(10, 0), stops)"#,
            )
            .unwrap();
        assert!(gradient.is::<ScriptedGradient>());
    }
}
