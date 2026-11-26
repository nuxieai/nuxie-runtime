// Shared compatibility facade for renderer bindings plus the Lua Gradient
// binding, whose upstream row is outside owner-split cluster C4.
use std::rc::Rc;

use crate::luau::{Lua, Result, Table, UserData, Value, Vector as LuaVector};
use nuxie_render_api::{ColorInt, RenderShader};

use super::host_memory_budget::{HostMemoryCharge, bytes_of, grown_capacity};
use super::non_finite::report_non_finite;

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
                    let non_finite = non_finite_gradient(
                        &[
                            ("from", from.x()),
                            ("from", from.y()),
                            ("to", to.x()),
                            ("to", to.y()),
                        ],
                        &positions,
                    );
                    let shader = bindings.with_factory(|factory| {
                        Ok(if non_finite.is_some() {
                            factory.make_linear_gradient(
                                0.0,
                                0.0,
                                1.0,
                                0.0,
                                &TRANSPARENT_COLORS,
                                &TRANSPARENT_POSITIONS,
                            )
                        } else {
                            factory.make_linear_gradient(
                                from.x(),
                                from.y(),
                                to.x(),
                                to.y(),
                                &colors,
                                &positions,
                            )
                        })
                    })?;
                    if let Some(what) = non_finite {
                        report_non_finite(lua, "Gradient.linear", "draws nothing", &what);
                    }
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
                    let non_finite = non_finite_gradient(
                        &[
                            ("center", center.x()),
                            ("center", center.y()),
                            ("radius", radius),
                        ],
                        &positions,
                    );
                    let shader = bindings.with_factory(|factory| {
                        Ok(if non_finite.is_some() {
                            factory.make_radial_gradient(
                                0.0,
                                0.0,
                                1.0,
                                &TRANSPARENT_COLORS,
                                &TRANSPARENT_POSITIONS,
                            )
                        } else {
                            factory.make_radial_gradient(
                                center.x(),
                                center.y(),
                                radius,
                                &colors,
                                &positions,
                            )
                        })
                    })?;
                    if let Some(what) = non_finite {
                        report_non_finite(lua, "Gradient.radial", "draws nothing", &what);
                    }
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

/// The stops of the gradient that stands in for one made with a number
/// that is not finite: a definition every factory accepts, which shows
/// nothing wherever it is drawn.
const TRANSPARENT_COLORS: [ColorInt; 2] = [0, 0];
const TRANSPARENT_POSITIONS: [f32; 2] = [0.0, 1.0];

/// Names the first number a gradient was made from that is not finite:
/// one of `values`, or a stop's position.
fn non_finite_gradient(values: &[(&str, f32)], positions: &[f32]) -> Option<String> {
    if let Some((name, value)) = values.iter().find(|(_, value)| !value.is_finite()) {
        return Some(format!("its {name} ({value})"));
    }
    positions
        .iter()
        .position(|position| !position.is_finite())
        .map(|index| format!("its stop {} position ({})", index + 1, positions[index]))
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
    let mut charge = HostMemoryCharge::for_lua(lua, gradient_bytes(capacity))?;
    let mut colors = Vec::with_capacity(capacity);
    let mut positions = Vec::with_capacity(capacity);
    for index in 1_i64.. {
        let Value::Table(stop) = stops.raw_get::<Value>(index)? else {
            break;
        };
        // A stop's `__index` may have added stops past the length read
        // above: what the vectors grow to is charged before they grow.
        charge.reserve_to(gradient_bytes(grown_capacity(
            colors.capacity(),
            colors.len(),
            1,
        )))?;
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
    use crate::luau::AnyUserData;
    use crate::vm::ScriptVm;
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
