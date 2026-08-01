// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/math/lua_mat2d.cpp
use luaur_rt::{
    Error, Lua, MultiValue, Result, UserData, UserDataFields, UserDataMethods, Value,
    Vector as LuaVector,
};
use nuxie_render_api::Mat2D;

#[derive(Clone, Copy)]
pub(super) struct ScriptedMat2D(pub(super) Mat2D);

impl UserData for ScriptedMat2D {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        for (name, index) in [
            ("xx", 0usize),
            ("yx", 1),
            ("xy", 2),
            ("yy", 3),
            ("tx", 4),
            ("ty", 5),
        ] {
            fields.add_field_method_get(name, move |_, this| Ok(this.0.0[index]));
            fields.add_field_method_set(name, move |_, this, value: f32| {
                this.0.0[index] = value;
                Ok(())
            });
        }
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method("__mul", |lua, this, rhs: Value| {
            Ok(match rhs {
                Value::Vector(vector) => {
                    let point = this
                        .0
                        .transform_point(nuxie_render_api::Vec2D::new(vector.x(), vector.y()));
                    Value::Vector(LuaVector::new(point.x, point.y, 0.0))
                }
                Value::UserData(rhs) => {
                    let rhs = rhs.borrow::<ScriptedMat2D>()?;
                    Value::UserData(
                        lua.create_userdata(ScriptedMat2D(multiply_mat2d(this.0, rhs.0)))?,
                    )
                }
                _ => return Err(Error::runtime("Mat2D can multiply a Vector or Mat2D")),
            })
        });
    }
}

fn multiply_mat2d(lhs: Mat2D, rhs: Mat2D) -> Mat2D {
    let a = lhs.0;
    let b = rhs.0;
    Mat2D([
        a[0].mul_add(b[0], a[2] * b[1]),
        a[1].mul_add(b[0], a[3] * b[1]),
        a[0].mul_add(b[2], a[2] * b[3]),
        a[1].mul_add(b[2], a[3] * b[3]),
        a[0].mul_add(b[4], a[2] * b[5]) + a[4],
        a[1].mul_add(b[4], a[3] * b[5]) + a[5],
    ])
}

pub(super) fn install_mat2d_global(lua: &Lua) -> Result<()> {
    let table = lua.create_table();
    table.set(
        "identity",
        lua.create_function(|lua, ()| lua.create_userdata(ScriptedMat2D(Mat2D::IDENTITY)))?,
    )?;
    table.set(
        "values",
        lua.create_function(
            |lua, (xx, yx, xy, yy, tx, ty): (f32, f32, f32, f32, f32, f32)| {
                lua.create_userdata(ScriptedMat2D(Mat2D([xx, yx, xy, yy, tx, ty])))
            },
        )?,
    )?;
    table.set(
        "withTranslation",
        lua.create_function(|lua, args: MultiValue| {
            let (x, y) = vec_or_numbers(&args)?;
            lua.create_userdata(ScriptedMat2D(Mat2D([1.0, 0.0, 0.0, 1.0, x, y])))
        })?,
    )?;
    table.set(
        "withScale",
        lua.create_function(|lua, args: MultiValue| {
            let (x, y) = vec_or_numbers_or_uniform(&args)?;
            lua.create_userdata(ScriptedMat2D(Mat2D([x, 0.0, 0.0, y, 0.0, 0.0])))
        })?,
    )?;
    table.set(
        "withScaleAndTranslation",
        lua.create_function(|lua, args: MultiValue| {
            let (sx, sy, tx, ty) = scale_translation_args(&args)?;
            lua.create_userdata(ScriptedMat2D(Mat2D([sx, 0.0, 0.0, sy, tx, ty])))
        })?,
    )?;
    table.set(
        "withRotation",
        lua.create_function(|lua, radians: f32| {
            let c = radians.cos();
            let s = radians.sin();
            lua.create_userdata(ScriptedMat2D(Mat2D([c, s, -s, c, 0.0, 0.0])))
        })?,
    )?;
    table.set_readonly(true);
    lua.globals().set("Mat2D", table)?;
    Ok(())
}

fn vec_or_numbers(args: &MultiValue) -> Result<(f32, f32)> {
    match args.front() {
        Some(Value::Vector(value)) => Ok((value.x(), value.y())),
        Some(Value::Integer(x)) => Ok((*x as f32, number_arg(args.get(1), "y")?)),
        Some(Value::Number(x)) => Ok((*x as f32, number_arg(args.get(1), "y")?)),
        _ => Err(Error::runtime("expected Vector or x/y numbers")),
    }
}

fn vec_or_numbers_or_uniform(args: &MultiValue) -> Result<(f32, f32)> {
    match args.front() {
        Some(Value::Vector(value)) => Ok((value.x(), value.y())),
        Some(Value::Integer(x)) => Ok((
            *x as f32,
            number_arg(args.get(1), "scaleY").unwrap_or(*x as f32),
        )),
        Some(Value::Number(x)) => Ok((
            *x as f32,
            number_arg(args.get(1), "scaleY").unwrap_or(*x as f32),
        )),
        _ => Err(Error::runtime("expected Vector or scale numbers")),
    }
}

fn scale_translation_args(args: &MultiValue) -> Result<(f32, f32, f32, f32)> {
    match (args.front(), args.get(1)) {
        (Some(Value::Vector(scale)), Some(Value::Vector(translation))) => {
            Ok((scale.x(), scale.y(), translation.x(), translation.y()))
        }
        _ => Ok((
            number_arg(args.front(), "scaleX")?,
            number_arg(args.get(1), "scaleY")?,
            number_arg(args.get(2), "translationX")?,
            number_arg(args.get(3), "translationY")?,
        )),
    }
}

pub(super) fn number_arg(value: Option<&Value>, name: &str) -> Result<f32> {
    match value {
        Some(Value::Integer(value)) => Ok(*value as f32),
        Some(Value::Number(value)) => Ok(*value as f32),
        _ => Err(Error::runtime(format!("expected numeric {name}"))),
    }
}

#[cfg(test)]
mod matrix_tests {
    use super::*;

    #[test]
    fn matrix_multiplication_matches_cpp_contraction_order() {
        let matrix = Mat2D([0.8660254, 0.5, -0.5, 0.8660254, 12.124355, 7.0]);
        let result = multiply_mat2d(matrix, matrix);

        assert_eq!(
            result.0.map(f32::to_bits),
            [
                0x3eff_ffff,
                0x3f5d_b3d7,
                0xbf5d_b3d7,
                0x3f00_0000,
                0x4198_feae,
                0x4198_feae,
            ]
        );
    }
}
