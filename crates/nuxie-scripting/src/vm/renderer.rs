// Shared compatibility facade for renderer bindings plus the Lua Gradient
// binding, whose upstream row is outside owner-split cluster C4.
use std::rc::Rc;

use crate::luau::{Lua, Result, Table, UserData, Value, Vector as LuaVector};
use nuxie_render_api::{ColorInt, RenderShader};

use super::host_memory_budget::{HostMemoryCharge, bytes_of};

pub(super) use super::lua_path::call_path_effect_update;
pub(crate) use super::lua_renderer_library::RendererBindings;

impl RendererBindings {
    pub(super) fn install_gradient_global(&self, lua: &Lua) -> Result<()> {
        let table = lua.create_table();

        let bindings = self.clone();
        table.set(
            "linear",
            lua.create_function_lent(
                move |lua, loan, (from, to, stops): (LuaVector, LuaVector, Table)| {
                    let (colors, positions, charge) = gradient_stops(lua, loan, stops)?;
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
                    lua.create_userdata(ScriptedGradient {
                        shader: Rc::from(shader),
                        _charge: Rc::new(charge),
                    })
                },
            )?,
        )?;

        let bindings = self.clone();
        table.set(
            "radial",
            lua.create_function_lent(
                move |lua, loan, (center, radius, stops): (LuaVector, f32, Table)| {
                    let (colors, positions, charge) = gradient_stops(lua, loan, stops)?;
                    let shader = bindings.with_factory(|factory| {
                        Ok(factory.make_radial_gradient(
                            center.x(),
                            center.y(),
                            radius,
                            &colors,
                            &positions,
                        ))
                    })?;
                    lua.create_userdata(ScriptedGradient {
                        shader: Rc::from(shader),
                        _charge: Rc::new(charge),
                    })
                },
            )?,
        )?;

        table.set_readonly(true);
        lua.globals().set("Gradient", table)?;
        Ok(())
    }
}

/// A gradient and the charge for the stops its shader keeps, shared by every
/// paint the gradient is set on.
#[derive(Clone)]
pub(super) struct ScriptedGradient {
    pub(super) shader: Rc<dyn RenderShader>,
    /// Kept for its refund on drop.
    _charge: Rc<HostMemoryCharge>,
}

impl UserData for ScriptedGradient {}

/// What a shader of `stops` stops keeps: a color and a position per stop.
fn gradient_bytes(stops: usize) -> usize {
    bytes_of::<ColorInt>(stops).saturating_add(bytes_of::<f32>(stops))
}

/// The stops of a gradient, charged before they are read; an `__index` a
/// stop runs gets the loan of the call that made the gradient.
fn gradient_stops(
    lua: &Lua,
    mut loan: crate::luau::Loan<'_>,
    stops: Table,
) -> Result<(Vec<ColorInt>, Vec<f32>, HostMemoryCharge)> {
    let capacity = stops.raw_len();
    let charge = HostMemoryCharge::for_lua(lua, gradient_bytes(capacity))?;
    let mut colors = Vec::with_capacity(capacity);
    let mut positions = Vec::with_capacity(capacity);
    for index in 1_i64.. {
        let Value::Table(stop) = stops.raw_get::<Value>(index)? else {
            break;
        };
        positions.push(stop.get_with(crate::luau::reborrow_loan(&mut loan), "position")?);
        let color = stop.get_with::<Value>(crate::luau::reborrow_loan(&mut loan), "color")?;
        colors.push(super::lua_color::required_unsigned(
            &stop.lua(),
            Some(&color),
            "gradient stop color",
        )?);
    }
    Ok((colors, positions, charge))
}

#[cfg(all(test, feature = "compiler"))]
mod upstream_gradient_tests {
    use super::ScriptedGradient;
    use crate::vm::ScriptVm;
    use crate::luau::AnyUserData;
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
