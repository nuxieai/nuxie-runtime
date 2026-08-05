#![cfg(feature = "luau")]

use nuxie_scripting::vm::ScriptVm;

fn rive_vm() -> ScriptVm {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    vm
}

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "expected {actual} to be within {tolerance} of {expected}"
    );
}

fn eval_error(source: &str) -> String {
    rive_vm().eval::<()>(source).unwrap_err().to_string()
}

#[test]
fn buffer_writef16_and_readf16_round_trip() {
    let vm = rive_vm();

    let result: f64 = vm
        .eval(
            r#"
            local b = buffer.create(2)
            buffer.writef16(b, 0, 1.5)
            return buffer.readf16(b, 0)
            "#,
        )
        .unwrap();

    assert_eq!(result, 1.5);
}

#[test]
fn buffer_readf16_zero() {
    let result: f64 = rive_vm()
        .eval(
            r#"
            local b = buffer.create(2)
            buffer.writef16(b, 0, 0)
            return buffer.readf16(b, 0)
            "#,
        )
        .unwrap();
    assert_eq!(result, 0.0);
}

#[test]
fn buffer_readf16_negative_zero() {
    let result: f64 = rive_vm()
        .eval(
            r#"
            local b = buffer.create(2)
            buffer.writef16(b, 0, -0.0)
            return buffer.readf16(b, 0)
            "#,
        )
        .unwrap();
    assert_eq!(result, 0.0);
}

#[test]
fn buffer_writef16_and_readf16_negative_value() {
    let result: f64 = rive_vm()
        .eval(
            r#"
            local b = buffer.create(2)
            buffer.writef16(b, 0, -3.25)
            return buffer.readf16(b, 0)
            "#,
        )
        .unwrap();
    assert_eq!(result, -3.25);
}

#[test]
fn buffer_writef16_and_readf16_small_denormal() {
    let result: f64 = rive_vm()
        .eval(
            r#"
            local b = buffer.create(2)
            buffer.writef16(b, 0, 5.96046448e-8)
            return buffer.readf16(b, 0)
            "#,
        )
        .unwrap();
    assert_close(result, 5.960_464_48e-8, 1e-9);
}

#[test]
fn buffer_writef16_infinity() {
    let result: f64 = rive_vm()
        .eval(
            r#"
            local b = buffer.create(2)
            buffer.writef16(b, 0, math.huge)
            return buffer.readf16(b, 0)
            "#,
        )
        .unwrap();
    assert_eq!(result, f64::INFINITY);
}

#[test]
fn buffer_writef16_negative_infinity() {
    let result: f64 = rive_vm()
        .eval(
            r#"
            local b = buffer.create(2)
            buffer.writef16(b, 0, -math.huge)
            return buffer.readf16(b, 0)
            "#,
        )
        .unwrap();
    assert_eq!(result, f64::NEG_INFINITY);
}

#[test]
fn buffer_writef16_overflow_to_infinity() {
    let result: f64 = rive_vm()
        .eval(
            r#"
            local b = buffer.create(2)
            buffer.writef16(b, 0, 100000)
            return buffer.readf16(b, 0)
            "#,
        )
        .unwrap();
    assert_eq!(result, f64::INFINITY);
}

#[test]
fn buffer_readf16_multiple_offsets() {
    let result: (f64, f64, f64) = rive_vm()
        .eval(
            r#"
            local b = buffer.create(6)
            buffer.writef16(b, 0, 1.0)
            buffer.writef16(b, 2, 2.0)
            buffer.writef16(b, 4, 3.0)
            return buffer.readf16(b, 0), buffer.readf16(b, 2),
                buffer.readf16(b, 4)
            "#,
        )
        .unwrap();
    assert_eq!(result, (1.0, 2.0, 3.0));
}

#[test]
fn buffer_readf16_out_of_bounds() {
    let error = eval_error(
        r#"
        local b = buffer.create(1)
        return buffer.readf16(b, 0)
        "#,
    );
    assert!(error.contains("out of bounds"), "{error}");
}

#[test]
fn buffer_writef16_out_of_bounds() {
    let error = eval_error(
        r#"
        local b = buffer.create(1)
        buffer.writef16(b, 0, 1.0)
        "#,
    );
    assert!(error.contains("out of bounds"), "{error}");
}

#[test]
fn buffer_stridedcopy_basic() {
    let vm = rive_vm();

    let result: (f64, f64, f64) = vm
        .eval(
            r#"
            local src = buffer.create(24)
            buffer.writef32(src, 0, 10.0)
            buffer.writef32(src, 8, 20.0)
            buffer.writef32(src, 16, 30.0)
            local dst = buffer.create(12)
            buffer.stridedcopy(dst, 0, 4, src, 0, 8, 4, 3)
            return buffer.readf32(dst, 0), buffer.readf32(dst, 4),
                buffer.readf32(dst, 8)
            "#,
        )
        .unwrap();

    assert_eq!(result, (10.0, 20.0, 30.0));
}

#[test]
fn buffer_stridedcopy_interleave() {
    let result: (f64, f64, f64, f64, f64, f64) = rive_vm()
        .eval(
            r#"
            local pos = buffer.create(12)
            buffer.writef32(pos, 0, 1.0)
            buffer.writef32(pos, 4, 2.0)
            buffer.writef32(pos, 8, 3.0)
            local col = buffer.create(12)
            buffer.writef32(col, 0, 4.0)
            buffer.writef32(col, 4, 5.0)
            buffer.writef32(col, 8, 6.0)
            local interleaved = buffer.create(24)
            buffer.stridedcopy(interleaved, 0, 8, pos, 0, 4, 4, 3)
            buffer.stridedcopy(interleaved, 4, 8, col, 0, 4, 4, 3)
            return buffer.readf32(interleaved, 0),
                buffer.readf32(interleaved, 4), buffer.readf32(interleaved, 8),
                buffer.readf32(interleaved, 12), buffer.readf32(interleaved, 16),
                buffer.readf32(interleaved, 20)
            "#,
        )
        .unwrap();
    assert_eq!(result, (1.0, 4.0, 2.0, 5.0, 3.0, 6.0));
}

#[test]
fn buffer_stridedcopy_zero_count() {
    let result: f64 = rive_vm()
        .eval(
            r#"
            local b = buffer.create(4)
            buffer.writef32(b, 0, 99.0)
            buffer.stridedcopy(b, 0, 4, b, 0, 4, 4, 0)
            return buffer.readf32(b, 0)
            "#,
        )
        .unwrap();
    assert_eq!(result, 99.0);
}

#[test]
fn buffer_stridedcopy_multi_byte_element() {
    let result: (f64, f64, f64, f64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(24)
            buffer.writef32(src, 0, 1.0)
            buffer.writef32(src, 4, 2.0)
            buffer.writef32(src, 12, 3.0)
            buffer.writef32(src, 16, 4.0)
            local dst = buffer.create(16)
            buffer.stridedcopy(dst, 0, 8, src, 0, 12, 8, 2)
            return buffer.readf32(dst, 0), buffer.readf32(dst, 4),
                buffer.readf32(dst, 8), buffer.readf32(dst, 12)
            "#,
        )
        .unwrap();
    assert_eq!(result, (1.0, 2.0, 3.0, 4.0));
}

#[test]
fn buffer_stridedcopy_out_of_bounds_src() {
    let error = eval_error(
        r#"
        local src = buffer.create(4)
        local dst = buffer.create(8)
        buffer.stridedcopy(dst, 0, 4, src, 0, 4, 4, 2)
        "#,
    );
    assert!(error.contains("out of bounds"), "{error}");
}

#[test]
fn buffer_stridedcopy_out_of_bounds_dst() {
    let error = eval_error(
        r#"
        local src = buffer.create(8)
        local dst = buffer.create(4)
        buffer.stridedcopy(dst, 0, 4, src, 0, 4, 4, 2)
        "#,
    );
    assert!(error.contains("out of bounds"), "{error}");
}

#[test]
fn buffer_convert_f32_to_f16() {
    let vm = rive_vm();

    let result: (f64, f64, f64) = vm
        .eval(
            r#"
            local src = buffer.create(12)
            buffer.writef32(src, 0, 1.0)
            buffer.writef32(src, 4, 0.5)
            buffer.writef32(src, 8, -2.0)
            local dst = buffer.create(6)
            buffer.convert(dst, 0, 'f16', src, 0, 'f32', 3)
            return buffer.readf16(dst, 0), buffer.readf16(dst, 2),
                buffer.readf16(dst, 4)
            "#,
        )
        .unwrap();

    assert_eq!(result, (1.0, 0.5, -2.0));
}

#[test]
fn buffer_convert_f16_to_f32() {
    let result: (f64, f64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(4)
            buffer.writef16(src, 0, 3.5)
            buffer.writef16(src, 2, -1.25)
            local dst = buffer.create(8)
            buffer.convert(dst, 0, 'f32', src, 0, 'f16', 2)
            return buffer.readf32(dst, 0), buffer.readf32(dst, 4)
            "#,
        )
        .unwrap();
    assert_eq!(result, (3.5, -1.25));
}

#[test]
fn buffer_convert_u8norm_to_f32() {
    let result: (f64, f64, f64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(3)
            buffer.writeu8(src, 0, 0)
            buffer.writeu8(src, 1, 128)
            buffer.writeu8(src, 2, 255)
            local dst = buffer.create(12)
            buffer.convert(dst, 0, 'f32', src, 0, 'u8norm', 3)
            return buffer.readf32(dst, 0), buffer.readf32(dst, 4),
                buffer.readf32(dst, 8)
            "#,
        )
        .unwrap();
    assert_eq!(result.0, 0.0);
    assert_close(result.1, 128.0 / 255.0, 0.001);
    assert_eq!(result.2, 1.0);
}

#[test]
fn buffer_convert_f32_to_u8norm() {
    let result: (i64, i64, i64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(12)
            buffer.writef32(src, 0, 0.0)
            buffer.writef32(src, 4, 0.5)
            buffer.writef32(src, 8, 1.0)
            local dst = buffer.create(3)
            buffer.convert(dst, 0, 'u8norm', src, 0, 'f32', 3)
            return buffer.readu8(dst, 0), buffer.readu8(dst, 1),
                buffer.readu8(dst, 2)
            "#,
        )
        .unwrap();
    assert_eq!(result, (0, 128, 255));
}

#[test]
fn buffer_convert_u8norm_clamps_out_of_range_f32() {
    let result: (i64, i64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(8)
            buffer.writef32(src, 0, -0.5)
            buffer.writef32(src, 4, 1.5)
            local dst = buffer.create(2)
            buffer.convert(dst, 0, 'u8norm', src, 0, 'f32', 2)
            return buffer.readu8(dst, 0), buffer.readu8(dst, 1)
            "#,
        )
        .unwrap();
    assert_eq!(result, (0, 255));
}

#[test]
fn buffer_convert_u16norm_to_f32() {
    let result: (f64, f64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(4)
            buffer.writeu16(src, 0, 0)
            buffer.writeu16(src, 2, 65535)
            local dst = buffer.create(8)
            buffer.convert(dst, 0, 'f32', src, 0, 'u16norm', 2)
            return buffer.readf32(dst, 0), buffer.readf32(dst, 4)
            "#,
        )
        .unwrap();
    assert_eq!(result, (0.0, 1.0));
}

#[test]
fn buffer_convert_same_format_is_memcpy() {
    let result: (f64, f64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(8)
            buffer.writef32(src, 0, 42.0)
            buffer.writef32(src, 4, 99.0)
            local dst = buffer.create(8)
            buffer.convert(dst, 0, 'f32', src, 0, 'f32', 2)
            return buffer.readf32(dst, 0), buffer.readf32(dst, 4)
            "#,
        )
        .unwrap();
    assert_eq!(result, (42.0, 99.0));
}

#[test]
fn buffer_convert_zero_count() {
    let result: f64 = rive_vm()
        .eval(
            r#"
            local src = buffer.create(4)
            local dst = buffer.create(4)
            buffer.writef32(dst, 0, 77.0)
            buffer.convert(dst, 0, 'f16', src, 0, 'f32', 0)
            return buffer.readf32(dst, 0)
            "#,
        )
        .unwrap();
    assert_eq!(result, 77.0);
}

#[test]
fn buffer_convert_u8_to_u16() {
    let result: (i64, i64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(2)
            buffer.writeu8(src, 0, 200)
            buffer.writeu8(src, 1, 0)
            local dst = buffer.create(4)
            buffer.convert(dst, 0, 'u16', src, 0, 'u8', 2)
            return buffer.readu16(dst, 0), buffer.readu16(dst, 2)
            "#,
        )
        .unwrap();
    assert_eq!(result, (200, 0));
}

#[test]
fn buffer_convert_out_of_bounds_src() {
    let error = eval_error(
        r#"
        local src = buffer.create(2)
        local dst = buffer.create(8)
        buffer.convert(dst, 0, 'f32', src, 0, 'f32', 1)
        "#,
    );
    assert!(error.contains("out of bounds"), "{error}");
}

#[test]
fn buffer_convert_out_of_bounds_dst() {
    let error = eval_error(
        r#"
        local src = buffer.create(8)
        local dst = buffer.create(2)
        buffer.convert(dst, 0, 'f32', src, 0, 'f32', 1)
        "#,
    );
    assert!(error.contains("out of bounds"), "{error}");
}

#[test]
fn buffer_convert_unknown_format() {
    let error = eval_error(
        r#"
        local src = buffer.create(4)
        local dst = buffer.create(4)
        buffer.convert(dst, 0, 'rgb8', src, 0, 'f32', 1)
        "#,
    );
    assert!(error.contains("unknown buffer format"), "{error}");
}

#[test]
fn buffer_convert_i8norm_round_trip() {
    let result: (f64, f64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(8)
            buffer.writef32(src, 0, -1.0)
            buffer.writef32(src, 4, 0.5)
            local mid = buffer.create(2)
            buffer.convert(mid, 0, 'i8norm', src, 0, 'f32', 2)
            local dst = buffer.create(8)
            buffer.convert(dst, 0, 'f32', mid, 0, 'i8norm', 2)
            return buffer.readf32(dst, 0), buffer.readf32(dst, 4)
            "#,
        )
        .unwrap();
    assert_close(result.0, -1.0, 0.01);
    assert_close(result.1, 0.5, 0.01);
}

#[test]
fn buffer_convert_i16norm_round_trip() {
    let result: (f64, f64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(8)
            buffer.writef32(src, 0, -0.75)
            buffer.writef32(src, 4, 0.25)
            local mid = buffer.create(4)
            buffer.convert(mid, 0, 'i16norm', src, 0, 'f32', 2)
            local dst = buffer.create(8)
            buffer.convert(dst, 0, 'f32', mid, 0, 'i16norm', 2)
            return buffer.readf32(dst, 0), buffer.readf32(dst, 4)
            "#,
        )
        .unwrap();
    assert_close(result.0, -0.75, 0.001);
    assert_close(result.1, 0.25, 0.001);
}

#[test]
fn buffer_convert_f32_to_f16_with_components_2() {
    let result: (f64, f64, f64, f64, f64, f64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(24)
            buffer.writef32(src, 0, 1.0)
            buffer.writef32(src, 4, 2.0)
            buffer.writef32(src, 8, 3.0)
            buffer.writef32(src, 12, 4.0)
            buffer.writef32(src, 16, 5.0)
            buffer.writef32(src, 20, 6.0)
            local dst = buffer.create(12)
            buffer.convert(dst, 0, 'f16', src, 0, 'f32', 3, 2)
            return buffer.readf16(dst, 0), buffer.readf16(dst, 2),
                buffer.readf16(dst, 4), buffer.readf16(dst, 6),
                buffer.readf16(dst, 8), buffer.readf16(dst, 10)
            "#,
        )
        .unwrap();
    assert_eq!(result, (1.0, 2.0, 3.0, 4.0, 5.0, 6.0));
}

#[test]
fn buffer_convert_strided_f32_to_f16() {
    let result: (f64, f64, f64, f64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(32)
            buffer.writef32(src, 0, 100.0)
            buffer.writef32(src, 4, 200.0)
            buffer.writef32(src, 8, 0.5)
            buffer.writef32(src, 12, 0.75)
            buffer.writef32(src, 16, 300.0)
            buffer.writef32(src, 20, 400.0)
            buffer.writef32(src, 24, 0.25)
            buffer.writef32(src, 28, 1.0)
            local dst = buffer.create(8)
            buffer.convert(dst, 0, 'f16', src, 8, 'f32', 2, 2, 4, 16)
            return buffer.readf16(dst, 0), buffer.readf16(dst, 2),
                buffer.readf16(dst, 4), buffer.readf16(dst, 6)
            "#,
        )
        .unwrap();
    assert_eq!(result, (0.5, 0.75, 0.25, 1.0));
}

#[test]
fn buffer_convert_strided_u8norm_to_f32() {
    let result: (f64, f64, f64, f64, f64, f64, f64, f64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(16)
            buffer.writeu8(src, 0, 255)
            buffer.writeu8(src, 1, 0)
            buffer.writeu8(src, 2, 128)
            buffer.writeu8(src, 3, 255)
            buffer.writeu8(src, 8, 0)
            buffer.writeu8(src, 9, 255)
            buffer.writeu8(src, 10, 0)
            buffer.writeu8(src, 11, 128)
            local dst = buffer.create(32)
            buffer.convert(dst, 0, 'f32', src, 0, 'u8norm', 2, 4, 16, 8)
            return buffer.readf32(dst, 0), buffer.readf32(dst, 4),
                buffer.readf32(dst, 8), buffer.readf32(dst, 12),
                buffer.readf32(dst, 16), buffer.readf32(dst, 20),
                buffer.readf32(dst, 24), buffer.readf32(dst, 28)
            "#,
        )
        .unwrap();
    assert_eq!(result.0, 1.0);
    assert_eq!(result.1, 0.0);
    assert_close(result.2, 128.0 / 255.0, 0.01);
    assert_eq!(result.3, 1.0);
    assert_eq!(result.4, 0.0);
    assert_eq!(result.5, 1.0);
    assert_eq!(result.6, 0.0);
    assert_close(result.7, 128.0 / 255.0, 0.01);
}

#[test]
fn buffer_convert_components_1_is_default_behavior() {
    let result: (f64, f64, f64, f64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(8)
            buffer.writef32(src, 0, 1.5)
            buffer.writef32(src, 4, -2.5)
            local dst1 = buffer.create(4)
            local dst2 = buffer.create(4)
            buffer.convert(dst1, 0, 'f16', src, 0, 'f32', 2)
            buffer.convert(dst2, 0, 'f16', src, 0, 'f32', 2, 1)
            return buffer.readf16(dst1, 0), buffer.readf16(dst1, 2),
                buffer.readf16(dst2, 0), buffer.readf16(dst2, 2)
            "#,
        )
        .unwrap();
    assert_eq!(result, (1.5, -2.5, 1.5, -2.5));
}

#[test]
fn buffer_convert_rejects_overflowing_component_spans_without_panicking() {
    let result: (bool, String) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(4)
            local dst = buffer.create(4)
            local ok, message = pcall(function()
                buffer.convert(dst, 0, 'f32', src, 0, 'f32', 1, 2147483647)
            end)
            return ok, tostring(message)
            "#,
        )
        .unwrap();

    assert!(!result.0);
    assert!(result.1.contains("out of bounds"), "{}", result.1);
    assert!(!result.1.contains("rust panic"), "{}", result.1);
}

#[test]
fn buffer_convert_non_finite_float_to_integer_uses_rust_saturation_policy() {
    let result: (f64, f64, f64, i64, i64, i64) = rive_vm()
        .eval(
            r#"
            local src = buffer.create(12)
            buffer.writef32(src, 0, 0 / 0)
            buffer.writef32(src, 4, math.huge)
            buffer.writef32(src, 8, -math.huge)

            local raw = buffer.create(12)
            buffer.convert(raw, 0, 'u32', src, 0, 'f32', 3)
            local normalized = buffer.create(3)
            buffer.convert(normalized, 0, 'u8norm', src, 0, 'f32', 3)

            return buffer.readu32(raw, 0), buffer.readu32(raw, 4),
                buffer.readu32(raw, 8), buffer.readu8(normalized, 0),
                buffer.readu8(normalized, 1), buffer.readu8(normalized, 2)
            "#,
        )
        .unwrap();

    assert_eq!(result, (0.0, u32::MAX as f64, 0.0, 0, 255, 0));
}
mod support;
use support::ScriptVmSourceTestExt as _;
