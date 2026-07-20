#![cfg(feature = "luau")]

use luaur_rt::Buffer as LuaBuffer;
use nuxie_scripting::vm::ScriptVm;

fn rive_vm() -> ScriptVm {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    vm
}

fn buffer_words(buffer: LuaBuffer) -> [u32; 16] {
    let bytes = buffer.to_vec();
    std::array::from_fn(|index| {
        u32::from_ne_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
    })
}

#[test]
fn mat4_identity_exposes_column_major_fields_and_indices() {
    let vm = rive_vm();

    let values: (f64, f64, f64, f64, f64, f64) = vm
        .eval(
            r#"
            local m = Mat4.identity()
            return m.m11, m.m22, m.m33, m.m44, m.m12, m[6]
            "#,
        )
        .unwrap();

    assert_eq!(values, (1.0, 1.0, 1.0, 1.0, 0.0, 1.0));
}

#[test]
fn mat4_instances_share_one_metatable_and_methods_are_namecall_only() {
    let vm = rive_vm();

    let values: (bool, bool, String, f64) = vm
        .eval(
            r#"
            local a = Mat4.identity()
            local b = Mat4.fromScale(2)
            local propertyOk = pcall(function()
                return a.invert
            end)
            local methodOk, inverse = pcall(function()
                return a:invert()
            end)
            return rawequal(getmetatable(a), getmetatable(b)), propertyOk,
                typeof(inverse), inverse.m11
            "#,
        )
        .unwrap();

    assert_eq!(values, (true, false, "userdata".to_owned(), 1.0));
}

#[test]
fn mat4_values_are_mutable_and_column_major() {
    let vm = rive_vm();

    let values: (f64, f64, f64, f64, f64, f64) = vm
        .eval(
            r#"
            local m = Mat4.values(
                1, 2, 3, 4,
                5, 6, 7, 8,
                9, 10, 11, 12,
                13, 14, 15, 16)
            m.m21 = 20
            m[16] = 160
            return m.m11, m.m21, m.m31, m.m41, m.m14, m.m44
            "#,
        )
        .unwrap();

    assert_eq!(values, (1.0, 20.0, 3.0, 4.0, 13.0, 160.0));
}

#[test]
fn mat4_translation_transforms_a_point() {
    let vm = rive_vm();

    let values: (f64, f64, f64) = vm
        .eval(
            r#"
            local m = Mat4.fromTranslation(10, 20, 30)
            local v = m:transformPoint(1, 2, 3)
            return v.x, v.y, v.z
            "#,
        )
        .unwrap();

    assert_eq!(values, (11.0, 22.0, 33.0));
}

#[test]
fn mat4_look_at_builds_a_right_handed_view_matrix() {
    let vm = rive_vm();

    let values: (f64, f64, f64, f64, f64, f64) = vm
        .eval(
            r#"
            local view = Mat4.lookAt(
                Vector.xyz(0, 0, 5),
                Vector.origin(),
                Vector.xyz(0, 1, 0))
            local v = view:transformPoint(0, 0, 0)
            return v.x, v.y, v.z, view.m11, view.m22, view.m33
            "#,
        )
        .unwrap();

    assert_eq!(values, (0.0, 0.0, -5.0, 1.0, 1.0, 1.0));

    let side: (f64, f64, f64) = vm
        .eval(
            r#"
            local view = Mat4.lookAt(
                Vector.xyz(5, 0, 0),
                Vector.origin(),
                Vector.xyz(0, 1, 0))
            local v = view:transformPoint(1, 0, 0)
            return v.x, v.y, v.z
            "#,
        )
        .unwrap();
    assert_eq!(side, (0.0, 0.0, -4.0));
}

#[test]
fn mat4_ortho_maps_depth_to_zero_through_one() {
    let vm = rive_vm();

    let values: (f64, f64, f64, f64, f64, f64) = vm
        .eval(
            r#"
            local proj = Mat4.ortho(-2, 2, -1, 1, 0, 10)
            local near = proj:transformPoint(2, 1, 0)
            local far = proj:transformPoint(-2, -1, -10)
            return near.x, near.y, near.z, far.x, far.y, far.z
            "#,
        )
        .unwrap();

    assert_eq!(values, (1.0, 1.0, 0.0, -1.0, -1.0, 1.0));
}

#[test]
fn mat4_ortho_times_look_at_maps_a_world_point() {
    let vm = rive_vm();

    let values: (f64, f64, f64) = vm
        .eval(
            r#"
            local view = Mat4.lookAt(
                Vector.xyz(0, 0, 5),
                Vector.origin(),
                Vector.xyz(0, 1, 0))
            local projection = Mat4.ortho(-4, 4, -4, 4, 5, 15)
            local viewProjection = Mat4.multiply(Mat4.identity(), projection, view)
            local point = viewProjection:transformPoint(2, -2, -5)
            return point.x, point.y, point.z
            "#,
        )
        .unwrap();

    assert_eq!(values, (0.5, -0.5, 0.5));
}

#[test]
fn mat4_scale_rotation_and_perspective_constructors_match_rive_layout() {
    let vm = rive_vm();

    let values: (f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64) = vm
        .eval(
            r#"
            local uniform = Mat4.fromScale(2)
            local partial = Mat4.fromScale(2, 3)
            local rx = Mat4.fromRotationX(math.rad(90))
            local ry = Mat4.fromRotationY(math.rad(90))
            local rz = Mat4.fromRotationZ(math.rad(90))
            local p = Mat4.perspective(math.rad(90), 1, 5, 25)
            local reverse = Mat4.perspectiveReverseZ(math.rad(90), 1, 5)
            return uniform.m11, uniform.m22, uniform.m33,
                partial.m11, partial.m22, partial.m33,
                rx.m32, ry.m13, rz.m21,
                p.m43, reverse.m34
            "#,
        )
        .unwrap();

    assert_eq!(
        values,
        (2.0, 2.0, 2.0, 2.0, 3.0, 2.0, 1.0, 1.0, 1.0, -1.0, 5.0)
    );
}

#[test]
fn mat4_constructors_use_luau_numeric_string_coercion() {
    let vm = rive_vm();

    let values: (f64, f64, f64) = vm
        .eval(
            r#"
            local scale = Mat4.fromScale("0x2", "0x3")
            return scale.m11, scale.m22, scale.m33
            "#,
        )
        .unwrap();

    assert_eq!(values, (2.0, 3.0, 2.0));
}

#[test]
fn mat4_transpose_vec4_and_equality_match_upstream() {
    let vm = rive_vm();

    let values: (f64, f64, f64, f64, f64, f64, bool, bool) = vm
        .eval(
            r#"
            local m = Mat4.values(
                1, 2, 3, 4, 5, 6, 7, 8,
                9, 10, 11, 12, 13, 14, 15, 16)
            local transposed = m:transpose()
            local translated = Mat4.fromTranslation(10, 20, 30)
            local x, y, z, w = translated:transformVec4(1, 2, 3, 1)
            local same = Mat4.identity()
            local other = Mat4.identity()
            local equalBefore = same == other
            other.m11 = 2
            return transposed.m12, transposed.m21, x, y, z, w,
                equalBefore, same == other
            "#,
        )
        .unwrap();

    assert_eq!(values, (2.0, 5.0, 11.0, 22.0, 33.0, 1.0, true, false));
}

#[test]
fn mat4_multiply_composes_and_static_variants_write_in_place() {
    let vm = rive_vm();

    let values: (f64, f64, f64, f64, f64, f64, f64, bool) = vm
        .eval(
            r#"
            local translation = Mat4.fromTranslation(10, 0, 0)
            local scale = Mat4.fromScale(2, 2, 2)
            local composed = translation * scale
            local point = composed:transformPoint(1, 1, 1)

            local a = Mat4.fromTranslation(3, -1, 5) * Mat4.fromRotationY(0.7)
            local b = Mat4.fromScale(2, 0.5, 1) * Mat4.fromRotationZ(-0.3)
            local slow = Mat4.identity()
            local fast = Mat4.identity()
            local returned = Mat4.multiply(slow, a, b)
            Mat4.multiplyAffine(fast, a, b)
            local difference = 0
            for index = 1, 16 do
                difference += math.abs(slow[index] - fast[index])
            end
            return point.x, point.y, point.z, difference,
                fast.m41, fast.m42, fast.m43, rawequal(returned, slow)
            "#,
        )
        .unwrap();

    assert_eq!(values, (12.0, 2.0, 2.0, 0.0, 0.0, 0.0, 0.0, true));
}

#[test]
fn mat4_inverse_variants_round_trip_and_preserve_outputs_on_singular_input() {
    let vm = rive_vm();

    let values: (f64, f64, bool, f64, bool, f64, bool, bool) = vm
        .eval(
            r#"
            local matrix = Mat4.fromTranslation(3, -4, 5)
                * Mat4.fromRotationY(0.4)
                * Mat4.fromScale(2, 2, 2)
            local inverse = matrix:invert()
            local affineInverse = matrix:invertAffine()
            local identity = matrix * inverse
            local affineIdentity = matrix * affineInverse

            local staticOutput = Mat4.identity()
            local staticOk = Mat4.invert(staticOutput, matrix)
            local staticAffineOutput = Mat4.identity()
            local staticAffineOk = Mat4.invertAffine(staticAffineOutput, matrix)

            local singular = Mat4.fromScale(2, 0, 1)
            local untouched = Mat4.fromTranslation(9, 8, 7)
            local singularOk = Mat4.invert(untouched, singular)
            return math.abs(identity.m11 - 1) + math.abs(identity.m22 - 1)
                    + math.abs(identity.m33 - 1) + math.abs(identity.m44 - 1),
                math.abs(affineIdentity.m11 - 1) + math.abs(affineIdentity.m22 - 1)
                    + math.abs(affineIdentity.m33 - 1) + math.abs(affineIdentity.m44 - 1),
                staticOk, staticOutput.m14, staticAffineOk, staticAffineOutput.m14,
                singularOk, untouched.m14 == 9 and singular:invert() == nil
            "#,
        )
        .unwrap();

    assert!(values.0 < 1e-5, "general inverse error: {}", values.0);
    assert!(values.1 < 1e-5, "affine inverse error: {}", values.1);
    assert!(values.2);
    assert!(values.3.is_finite());
    assert!(values.4);
    assert!(values.5.is_finite());
    assert!(!values.6);
    assert!(values.7);
}

#[test]
fn mat4_buffer_write_is_column_major_and_checks_the_full_range() {
    let vm = rive_vm();

    let values: (f64, f64, f64, bool, bool, bool, bool) = vm
        .eval(
            r#"
            local matrix = Mat4.values(
                1, 2, 3, 4, 5, 6, 7, 8,
                9, 10, 11, 12, 13, 14, 15, 16)
            local bytes = buffer.create(80)
            matrix:writeToBuffer(bytes, 16)
            local negative = pcall(function()
                matrix:writeToBuffer(bytes, -1)
            end)
            local tooShort = pcall(function()
                matrix:writeToBuffer(buffer.create(63), 0)
            end)
            local unknownRead = pcall(function()
                return matrix.m55
            end)
            local unknownWrite = pcall(function()
                matrix.m55 = 1
            end)
            return buffer.readf32(bytes, 16),
                buffer.readf32(bytes, 16 + 4 * 4),
                buffer.readf32(bytes, 16 + 15 * 4),
                negative, tooShort, unknownRead, unknownWrite
            "#,
        )
        .unwrap();

    assert_eq!(values, (1.0, 5.0, 16.0, false, false, false, false));

    let error: String = vm
        .eval(
            r#"
            local ok, message = pcall(function()
                Mat4.identity():writeToBuffer(buffer.create(64), 1)
            end)
            return message
            "#,
        )
        .unwrap();
    assert!(
        error.contains("Mat4:writeToBuffer offset out of range"),
        "got: {error}"
    );
}

#[test]
fn mat4_nontrivial_f32_results_are_bit_exact_to_cpp_rive() {
    let vm = rive_vm();

    let (view, projection, combined, inverse): (LuaBuffer, LuaBuffer, LuaBuffer, LuaBuffer) = vm
        .eval(
            r#"
            local function words(matrix)
                local bytes = buffer.create(64)
                matrix:writeToBuffer(bytes, 0)
                return bytes
            end
            local view = Mat4.lookAt(
                Vector.xyz(3.25, -2.5, 7.75),
                Vector.xyz(-1.5, 4.25, 0.5),
                Vector.xyz(0.1, 1.0, 0.2))
            local projection = Mat4.ortho(-3.25, 7.5, -2.75, 8.125, 0.35, 91.0)
            local combined = projection * view
            return words(view), words(projection), words(combined), words(combined:invert())
            "#,
        )
        .unwrap();

    assert_eq!(
        buffer_words(view),
        [
            0x3f58_7788,
            0x3ea0_538a,
            0x3edd_6080,
            0x0000_0000,
            0x3cb5_3a63,
            0x3f49_e5ae,
            0xbf1d_4b4e,
            0x0000_0000,
            0xbf08_8ce4,
            0x3f07_73da,
            0x3f28_f211,
            0x0000_0000,
            0x3fb8_7400,
            0xc049_62d9,
            0xc100_e4d1,
            0x3f80_0000,
        ]
    );
    assert_eq!(
        buffer_words(projection),
        [
            0x3e3e_82fa,
            0x0000_0000,
            0x0000_0000,
            0x0000_0000,
            0x0000_0000,
            0x3e3c_5264,
            0x0000_0000,
            0x0000_0000,
            0x0000_0000,
            0x0000_0000,
            0xbc34_bd36,
            0x0000_0000,
            0xbeca_6b2a,
            0xbefd_0eb6,
            0xbb7d_08e5,
            0x3f80_0000,
        ]
    );
    assert_eq!(
        buffer_words(combined),
        [
            0x3e21_1777,
            0x3d6b_e1e5,
            0xbb9c_4b79,
            0x0000_0000,
            0x3b86_de0e,
            0x3e14_85a6,
            0x3bde_1a69,
            0x0000_0000,
            0xbdcb_3cdc,
            0x3dc7_4958,
            0xbbee_8e25,
            0x0000_0000,
            0xbe02_4d66,
            0xbf89_5662,
            0x3dae_1807,
            0x3f80_0000,
        ]
    );
    assert_eq!(
        buffer_words(inverse),
        [
            0x4091_704f,
            0x3df3_867b,
            0xc037_7d52,
            0x8000_0000,
            0x3fd9_f190,
            0x4089_3a1d,
            0x4038_217e,
            0x8000_0000,
            0xc21c_c7a4,
            0x425e_cadc,
            0xc26f_4bab,
            0x8000_0000,
            0x40b7_9611,
            0xbdf2_8229,
            0x40f9_d74d,
            0x3f7f_ffff,
        ]
    );
}
