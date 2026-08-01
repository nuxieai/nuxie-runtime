// Shared compatibility facade for renderer bindings plus the Lua Gradient
// binding, whose upstream row is outside owner-split cluster C4.
use std::rc::Rc;

use luaur_rt::{Lua, Result, Table, UserData, Vector as LuaVector};
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
    for stop in stops.sequence_values::<Table>() {
        let stop = stop?;
        positions.push(stop.get("position")?);
        colors.push(stop.get("color")?);
    }
    Ok((colors, positions))
}
