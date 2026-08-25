// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/math/lua_mat2d.cpp
use luaur_rt::{
    AnyUserData, Error, Lua, MultiValue, Result, UserData, UserDataFields, UserDataMethods, Value,
    Vector as LuaVector,
};
use nuxie_render_api::Mat2D;

#[derive(Clone, Copy)]
pub(super) struct ScriptedMat2D(pub(super) Mat2D);

impl UserData for ScriptedMat2D {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        for (name, index) in [
            ("xx", 0usize),
            ("xy", 1),
            ("yx", 2),
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
        for (lua_index, matrix_index) in (1i64..=6).zip(0usize..6) {
            fields.add_field_index_method_get(lua_index, move |_, this| Ok(this.0.0[matrix_index]));
            fields.add_field_index_method_set(lua_index, move |_, this, value: f32| {
                this.0.0[matrix_index] = value;
                Ok(())
            });
        }
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("invert", |lua, this, ()| {
            this.0.invert()
                .map(|matrix| lua.create_userdata(ScriptedMat2D(matrix)))
                .transpose()
        });
        methods.add_method("isIdentity", |_, this, ()| Ok(this.0 == Mat2D::IDENTITY));
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
        methods.add_meta_method("__eq", |_, this, rhs: AnyUserData| {
            Ok(this.0 == rhs.borrow::<ScriptedMat2D>()?.0)
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
            |lua, (xx, xy, yx, yy, tx, ty): (f32, f32, f32, f32, f32, f32)| {
                lua.create_userdata(ScriptedMat2D(Mat2D([xx, xy, yx, yy, tx, ty])))
            },
        )?,
    )?;
    table.set(
        "invert",
        lua.create_function(|_, (output, input): (AnyUserData, AnyUserData)| {
            let input = input.borrow::<ScriptedMat2D>()?.0;
            let Some(inverse) = input.invert() else {
                return Ok(false);
            };
            output.borrow_mut::<ScriptedMat2D>()?.0 = inverse;
            Ok(true)
        })?,
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

    #[test]
    fn matrix_inverse_reaches_pinned_render_mat2d_owner() {
        let matrix = Mat2D([
            f32::from_bits(0x26cd_29b3),
            f32::from_bits(0x2533_fdc2),
            f32::from_bits(0xd01a_d4bb),
            f32::from_bits(0xce87_d5a9),
            0.0,
            0.0,
        ]);
        assert_eq!(matrix.determinant().to_bits(), 0xa7ee_c560);
        assert_eq!(
            matrix.invert().expect("pinned determinant is nonzero").0.map(f32::to_bits),
            [
                0x6611_a2d3,
                0x3cc0_fa97,
                0xe7a6_00cd,
                0xbe5b_f782,
                0x8000_0000,
                0x8000_0000,
            ],
        );
    }
}

#[cfg(all(test, feature = "compiler"))]
mod upstream_tests {
    use luaur_rt::Table;

    use super::*;

    fn mat2d_lua() -> Lua {
        let lua = Lua::new();
        super::super::lua_vec2d::install_vector_global(&lua).expect("Vector global installs");
        install_mat2d_global(&lua).expect("Mat2D global installs");
        lua
    }

    #[test]
    fn mat2d_can_be_constructed_direct_port() {
        let lua = mat2d_lua();
        let result: Table = lua
            .load(
                r#"
                local mat = Mat2D.values(1, 2, 3, 4, 5, 6)
                return {
                    tx = mat.tx,
                    ty = mat.ty,
                    xx = mat.xx,
                    xy = mat.xy,
                    yx = mat.yx,
                    yy = mat.yy,
                }
                "#,
            )
            .eval()
            .expect("pinned Mat2D constructor script runs");
        assert_eq!(result.get::<f64>("tx").unwrap(), 5.0);
        assert_eq!(result.get::<f64>("ty").unwrap(), 6.0);
        assert_eq!(result.get::<f64>("xx").unwrap(), 1.0);
        assert_eq!(result.get::<f64>("xy").unwrap(), 2.0);
        assert_eq!(result.get::<f64>("yx").unwrap(), 3.0);
        assert_eq!(result.get::<f64>("yy").unwrap(), 4.0);
    }

    #[test]
    fn mat2d_static_methods_work_direct_port() {
        let lua = mat2d_lua();
        let result: Table = lua
            .load(
                r#"
                return {
                    translationX = Mat2D.withTranslation(10, 20).tx,
                    translationY = Mat2D.withTranslation(10, 20).ty,
                    vectorTranslationX = Mat2D.withTranslation(Vector.xy(10, 20)).tx,
                    vectorTranslationY = Mat2D.withTranslation(Vector.xy(10, 20)).ty,
                    vectorScaleX = Mat2D.withScale(Vector.xy(10, 20)).xx,
                    vectorScaleY = Mat2D.withScale(Vector.xy(10, 20)).yy,
                    pairScaleX = Mat2D.withScale(10, 20).xx,
                    pairScaleY = Mat2D.withScale(10, 20).yy,
                    uniformScaleY = Mat2D.withScale(3).yy,
                    uniformScaleX = Mat2D.withScale(3).xx,
                    vectorScaleTranslationX = Mat2D.withScaleAndTranslation(Vector.xy(2, 3), Vector.xy(10, 20)).xx,
                    vectorScaleTranslationY = Mat2D.withScaleAndTranslation(Vector.xy(2, 3), Vector.xy(10, 20)).yy,
                    vectorScaleTranslationTx = Mat2D.withScaleAndTranslation(Vector.xy(2, 3), Vector.xy(10, 20)).tx,
                    vectorScaleTranslationTy = Mat2D.withScaleAndTranslation(Vector.xy(2, 3), Vector.xy(10, 20)).ty,
                    numberScaleTranslationX = Mat2D.withScaleAndTranslation(2, 3, 10, 20).xx,
                    numberScaleTranslationY = Mat2D.withScaleAndTranslation(2, 3, 10, 20).yy,
                    numberScaleTranslationTx = Mat2D.withScaleAndTranslation(2, 3, 10, 20).tx,
                    numberScaleTranslationTy = Mat2D.withScaleAndTranslation(2, 3, 10, 20).ty,
                }
                "#,
            )
            .eval()
            .expect("pinned Mat2D static-method script runs");
        for (field, expected) in [
            ("translationX", 10.0),
            ("translationY", 20.0),
            ("vectorTranslationX", 10.0),
            ("vectorTranslationY", 20.0),
            ("vectorScaleX", 10.0),
            ("vectorScaleY", 20.0),
            ("pairScaleX", 10.0),
            ("pairScaleY", 20.0),
            ("uniformScaleY", 3.0),
            ("uniformScaleX", 3.0),
            ("vectorScaleTranslationX", 2.0),
            ("vectorScaleTranslationY", 3.0),
            ("vectorScaleTranslationTx", 10.0),
            ("vectorScaleTranslationTy", 20.0),
            ("numberScaleTranslationX", 2.0),
            ("numberScaleTranslationY", 3.0),
            ("numberScaleTranslationTx", 10.0),
            ("numberScaleTranslationTy", 20.0),
        ] {
            assert_eq!(result.get::<f64>(field).unwrap(), expected, "{field}");
        }
    }

    #[test]
    fn mat2d_methods_work_direct_port() {
        let lua = mat2d_lua();
        assert_eq!(
            lua.load("return Mat2D.identity():invert().xx")
                .eval::<f64>()
                .expect("invert identity"),
            1.0
        );
        assert!(
            lua.load("return Mat2D.values(0,0,0,0,0,0):invert()")
                .eval::<Value>()
                .expect("invert singular")
                .is_nil()
        );
        assert!(
            !lua.load("return Mat2D.values(0,0,0,0,0,0):isIdentity()")
                .eval::<bool>()
                .expect("zero identity query")
        );
        assert!(
            lua.load("return Mat2D.values(1,0,0,1,0,0):isIdentity()")
                .eval::<bool>()
                .expect("literal identity query")
        );
        assert!(
            lua.load("return Mat2D.identity():isIdentity()")
                .eval::<bool>()
                .expect("identity query")
        );
        assert!(
            lua.load(
                r#"
                local mat = Mat2D.identity()
                mat.tx = 23
                local result = Mat2D.identity()
                if not Mat2D.invert(result, mat) then return false end
                return result == Mat2D.values(1,0,0,1,-23,0)
                "#,
            )
            .eval::<bool>()
            .expect("static invert")
        );
    }

    #[test]
    fn mat2d_meta_methods_work_direct_port() {
        let lua = mat2d_lua();
        for (source, expected) in [
            (
                "return Mat2D.identity() == Mat2D.values(1,2,3,4,5,6)",
                false,
            ),
            ("return Mat2D.identity() ~= Mat2D.values(1,2,3,4,5,6)", true),
            ("return Mat2D.identity() == Mat2D.values(1,0,0,1,0,0)", true),
            (
                "return Mat2D.identity() ~= Mat2D.values(1,0,0,1,0,0)",
                false,
            ),
        ] {
            assert_eq!(lua.load(source).eval::<bool>().unwrap(), expected);
        }
        assert_eq!(
            lua.load("return (Mat2D.withScale(2) * Vector.xy(1,1)).x")
                .eval::<f64>()
                .unwrap(),
            2.0
        );
        assert_eq!(
            lua.load("return (Mat2D.withScale(2) * Mat2D.withScale(2)).xx")
                .eval::<f64>()
                .unwrap(),
            4.0
        );
    }

    #[test]
    fn mat2d_setters_work_direct_port() {
        let lua = mat2d_lua();
        for (field, index, value, expected) in [
            ("xx", 1, 23, "Mat2D.values(23,0,0,1,0,0)"),
            ("xy", 2, 23, "Mat2D.values(1,23,0,1,0,0)"),
            ("yx", 3, 24, "Mat2D.values(1,0,24,1,0,0)"),
            ("yy", 4, 25, "Mat2D.values(1,0,0,25,0,0)"),
            ("tx", 5, 26, "Mat2D.values(1,0,0,1,26,0)"),
            ("ty", 6, 27, "Mat2D.values(1,0,0,1,0,27)"),
        ] {
            let named = format!(
                "local mat = Mat2D.identity(); mat.{field} = {value}; return mat == {expected}"
            );
            assert!(lua.load(&named).eval::<bool>().unwrap(), "named {field}");
            let indexed = format!(
                "local mat = Mat2D.identity(); mat[{index}] = {value}; return mat == {expected}"
            );
            assert!(
                lua.load(&indexed).eval::<bool>().unwrap(),
                "indexed {index}"
            );
        }
    }
}
