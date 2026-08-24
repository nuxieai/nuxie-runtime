// Malicious JPEG files can cause operations in the idct to overflow.
// One example is tests/crashtest/images/imagetestsuite/b0b8914cc5f7a6eff409f16d8cc236c5.jpg
// That's why wrapping operators are needed.

// Note: we have many values that are straight from a reference.
// Do not warn on them or try to automatically change them.
#![allow(clippy::excessive_precision)]
// Note: consistency for unrolled, scaled offset loops
#![allow(clippy::erasing_op)]
#![allow(clippy::identity_op)]
use crate::parser::Dimensions;
use core::num::Wrapping;

pub(crate) fn choose_idct_size(full_size: Dimensions, requested_size: Dimensions) -> usize {
    fn scaled(len: u16, scale: usize) -> u16 {
        ((len as u32 * scale as u32 - 1) / 8 + 1) as u16
    }

    for &scale in &[1, 2, 4] {
        if scaled(full_size.width, scale) >= requested_size.width
            || scaled(full_size.height, scale) >= requested_size.height
        {
            return scale;
        }
    }

    8
}

#[test]
fn test_choose_idct_size() {
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 200,
                height: 200
            }
        ),
        1
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 500,
                height: 500
            }
        ),
        1
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 684,
                height: 456
            }
        ),
        1
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 999,
                height: 456
            }
        ),
        1
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 684,
                height: 999
            }
        ),
        1
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 500,
                height: 333
            },
            Dimensions {
                width: 63,
                height: 42
            }
        ),
        1
    );

    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 685,
                height: 999
            }
        ),
        2
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 1000,
                height: 1000
            }
        ),
        2
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 1400,
                height: 1400
            }
        ),
        4
    );

    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 5472,
                height: 3648
            }
        ),
        8
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 16384,
                height: 16384
            }
        ),
        8
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 1,
                height: 1
            },
            Dimensions {
                width: 65535,
                height: 65535
            }
        ),
        8
    );
    assert_eq!(
        choose_idct_size(
            Dimensions {
                width: 5472,
                height: 3648
            },
            Dimensions {
                width: 16384,
                height: 16384
            }
        ),
        8
    );
}

pub(crate) fn dequantize_and_idct_block(
    scale_h: usize,
    scale_v: usize,
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    match (scale_h, scale_v) {
        #[cfg(feature = "rive_v9f")]
        (16, 16) => {
            rive_v9f_idct_16x16(coefficients, quantization_table, output_linestride, output)
        }
        #[cfg(feature = "rive_v9f")]
        (16, 8) => rive_v9f_idct_16x8(coefficients, quantization_table, output_linestride, output),
        #[cfg(feature = "rive_v9f")]
        (8, 16) => rive_v9f_idct_8x16(coefficients, quantization_table, output_linestride, output),
        #[cfg(feature = "rive_v9f")]
        (8, 8) => rive_v9f_idct_8x8(coefficients, quantization_table, output_linestride, output),
        #[cfg(not(feature = "rive_v9f"))]
        (8, 8) => dequantize_and_idct_block_8x8(
            coefficients,
            quantization_table,
            output_linestride,
            output,
        ),
        (4, 4) => dequantize_and_idct_block_4x4(
            coefficients,
            quantization_table,
            output_linestride,
            output,
        ),
        (2, 2) => dequantize_and_idct_block_2x2(
            coefficients,
            quantization_table,
            output_linestride,
            output,
        ),
        (1, 1) => dequantize_and_idct_block_1x1(
            coefficients,
            quantization_table,
            output_linestride,
            output,
        ),
        _ => panic!("Unsupported IDCT scale {scale_h}x{scale_v}/8"),
    }
}

#[cfg(feature = "rive_v9f")]
fn rive_v9f_idct_8x8(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    const CONST_BITS: u32 = 13;
    const PASS1_BITS: u32 = 2;
    let dequantize =
        |index: usize| i64::from(coefficients[index]) * i64::from(quantization_table[index]);
    let mut workspace = [0_i64; 64];

    for column in 0..8 {
        if (1..8).all(|row| coefficients[column + row * 8] == 0) {
            let dc = dequantize(column) << PASS1_BITS;
            for row in 0..8 {
                workspace[column + row * 8] = dc;
            }
            continue;
        }

        let mut z2 = (dequantize(column) << CONST_BITS) + (1 << (CONST_BITS - PASS1_BITS - 1));
        let mut z3 = dequantize(column + 4 * 8) << CONST_BITS;
        let mut tmp0 = z2 + z3;
        let mut tmp1 = z2 - z3;
        z2 = dequantize(column + 2 * 8);
        z3 = dequantize(column + 6 * 8);
        let mut z1 = (z2 + z3) * 4433;
        let tmp2 = z1 + z2 * 6270;
        let tmp3 = z1 - z3 * 15137;
        let tmp10 = tmp0 + tmp2;
        let tmp13 = tmp0 - tmp2;
        let tmp11 = tmp1 + tmp3;
        let tmp12 = tmp1 - tmp3;

        tmp0 = dequantize(column + 7 * 8);
        tmp1 = dequantize(column + 5 * 8);
        let mut tmp2 = dequantize(column + 3 * 8);
        let mut tmp3 = dequantize(column + 8);
        z2 = tmp0 + tmp2;
        z3 = tmp1 + tmp3;
        z1 = (z2 + z3) * 9633;
        z2 = z2 * -16069 + z1;
        z3 = z3 * -3196 + z1;
        z1 = (tmp0 + tmp3) * -7373;
        tmp0 = tmp0 * 2446 + z1 + z2;
        tmp3 = tmp3 * 12299 + z1 + z3;
        z1 = (tmp1 + tmp2) * -20995;
        tmp1 = tmp1 * 16819 + z1 + z3;
        tmp2 = tmp2 * 25172 + z1 + z2;

        let shift = CONST_BITS - PASS1_BITS;
        workspace[column] = (tmp10 + tmp3) >> shift;
        workspace[column + 7 * 8] = (tmp10 - tmp3) >> shift;
        workspace[column + 8] = (tmp11 + tmp2) >> shift;
        workspace[column + 6 * 8] = (tmp11 - tmp2) >> shift;
        workspace[column + 2 * 8] = (tmp12 + tmp1) >> shift;
        workspace[column + 5 * 8] = (tmp12 - tmp1) >> shift;
        workspace[column + 3 * 8] = (tmp13 + tmp0) >> shift;
        workspace[column + 4 * 8] = (tmp13 - tmp0) >> shift;
    }

    for row in 0..8 {
        let values = &workspace[row * 8..row * 8 + 8];
        let mut z2 = values[0] + ((128 << (PASS1_BITS + 3)) + (1 << (PASS1_BITS + 2)));
        let row_start = row * output_linestride;
        if values[1..].iter().all(|value| *value == 0) {
            let value = (z2 >> (PASS1_BITS + 3)).clamp(0, 255) as u8;
            output[row_start..row_start + 8].fill(value);
            continue;
        }

        let mut z3 = values[4];
        let mut tmp0 = (z2 + z3) << CONST_BITS;
        let mut tmp1 = (z2 - z3) << CONST_BITS;
        z2 = values[2];
        z3 = values[6];
        let mut z1 = (z2 + z3) * 4433;
        let tmp2 = z1 + z2 * 6270;
        let tmp3 = z1 - z3 * 15137;
        let tmp10 = tmp0 + tmp2;
        let tmp13 = tmp0 - tmp2;
        let tmp11 = tmp1 + tmp3;
        let tmp12 = tmp1 - tmp3;

        tmp0 = values[7];
        tmp1 = values[5];
        let mut tmp2 = values[3];
        let mut tmp3 = values[1];
        z2 = tmp0 + tmp2;
        z3 = tmp1 + tmp3;
        z1 = (z2 + z3) * 9633;
        z2 = z2 * -16069 + z1;
        z3 = z3 * -3196 + z1;
        z1 = (tmp0 + tmp3) * -7373;
        tmp0 = tmp0 * 2446 + z1 + z2;
        tmp3 = tmp3 * 12299 + z1 + z3;
        z1 = (tmp1 + tmp2) * -20995;
        tmp1 = tmp1 * 16819 + z1 + z3;
        tmp2 = tmp2 * 25172 + z1 + z2;

        let shift = CONST_BITS + PASS1_BITS + 3;
        let mut store = |column: usize, value: i64| {
            output[row_start + column] = (value >> shift).clamp(0, 255) as u8;
        };
        store(0, tmp10 + tmp3);
        store(7, tmp10 - tmp3);
        store(1, tmp11 + tmp2);
        store(6, tmp11 - tmp2);
        store(2, tmp12 + tmp1);
        store(5, tmp12 - tmp1);
        store(3, tmp13 + tmp0);
        store(4, tmp13 - tmp0);
    }
}

// Literal arithmetic owner: rive-app_libjpeg_v9f/jidctint.c,
// jpeg_idct_16x8().  This is the source's 8-point column kernel followed by
// its 16-point row kernel; it is selected for horizontal-only subsampling.
#[cfg(feature = "rive_v9f")]
fn rive_v9f_idct_16x8(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    const CONST_BITS: u32 = 13;
    const PASS1_BITS: u32 = 2;
    let dequantize =
        |index: usize| i64::from(coefficients[index]) * i64::from(quantization_table[index]);
    let mut workspace = [0_i64; 64];

    for column in 0..8 {
        if (1..8).all(|row| coefficients[column + row * 8] == 0) {
            let dc = dequantize(column) << PASS1_BITS;
            for row in 0..8 {
                workspace[column + row * 8] = dc;
            }
            continue;
        }

        let mut z2 = (dequantize(column) << CONST_BITS) + (1 << (CONST_BITS - PASS1_BITS - 1));
        let mut z3 = dequantize(column + 4 * 8) << CONST_BITS;
        let mut tmp0 = z2 + z3;
        let mut tmp1 = z2 - z3;
        z2 = dequantize(column + 2 * 8);
        z3 = dequantize(column + 6 * 8);
        let mut z1 = (z2 + z3) * 4433;
        let tmp2 = z1 + z2 * 6270;
        let tmp3 = z1 - z3 * 15137;
        let tmp10 = tmp0 + tmp2;
        let tmp13 = tmp0 - tmp2;
        let tmp11 = tmp1 + tmp3;
        let tmp12 = tmp1 - tmp3;

        tmp0 = dequantize(column + 7 * 8);
        tmp1 = dequantize(column + 5 * 8);
        let mut tmp2 = dequantize(column + 3 * 8);
        let mut tmp3 = dequantize(column + 8);
        z2 = tmp0 + tmp2;
        z3 = tmp1 + tmp3;
        z1 = (z2 + z3) * 9633;
        z2 = z2 * -16069 + z1;
        z3 = z3 * -3196 + z1;
        z1 = (tmp0 + tmp3) * -7373;
        tmp0 = tmp0 * 2446 + z1 + z2;
        tmp3 = tmp3 * 12299 + z1 + z3;
        z1 = (tmp1 + tmp2) * -20995;
        tmp1 = tmp1 * 16819 + z1 + z3;
        tmp2 = tmp2 * 25172 + z1 + z2;

        let shift = CONST_BITS - PASS1_BITS;
        workspace[column] = (tmp10 + tmp3) >> shift;
        workspace[column + 7 * 8] = (tmp10 - tmp3) >> shift;
        workspace[column + 8] = (tmp11 + tmp2) >> shift;
        workspace[column + 6 * 8] = (tmp11 - tmp2) >> shift;
        workspace[column + 2 * 8] = (tmp12 + tmp1) >> shift;
        workspace[column + 5 * 8] = (tmp12 - tmp1) >> shift;
        workspace[column + 3 * 8] = (tmp13 + tmp0) >> shift;
        workspace[column + 4 * 8] = (tmp13 - tmp0) >> shift;
    }

    for row in 0..8 {
        let values = &workspace[row * 8..row * 8 + 8];
        let mut tmp0 =
            (values[0] + ((128 << (PASS1_BITS + 3)) + (1 << (PASS1_BITS + 2)))) << CONST_BITS;
        let mut z1 = values[4];
        let tmp1 = z1 * 10703;
        let tmp2 = z1 * 4433;
        let tmp10 = tmp0 + tmp1;
        let tmp11_base = tmp0 - tmp1;
        let tmp12_base = tmp0 + tmp2;
        let tmp13_base = tmp0 - tmp2;

        z1 = values[2];
        let mut z2 = values[6];
        let mut z3 = z1 - z2;
        let z4 = z3 * 2260;
        z3 *= 11363;
        tmp0 = z3 + z2 * 20995;
        let tmp1 = z4 + z1 * 7373;
        let tmp2 = z3 - z1 * 4926;
        let tmp3 = z4 - z2 * 4176;

        let tmp20 = tmp10 + tmp0;
        let tmp27 = tmp10 - tmp0;
        let tmp21 = tmp12_base + tmp1;
        let tmp26 = tmp12_base - tmp1;
        let tmp22 = tmp13_base + tmp2;
        let tmp25 = tmp13_base - tmp2;
        let tmp23 = tmp11_base + tmp3;
        let tmp24 = tmp11_base - tmp3;

        z1 = values[1];
        z2 = values[3];
        z3 = values[5];
        let mut z4 = values[7];
        let mut tmp11 = z1 + z3;
        let mut tmp1 = (z1 + z2) * 11086;
        let mut tmp2 = tmp11 * 10217;
        let mut tmp3 = (z1 + z4) * 8956;
        let mut tmp10 = (z1 - z4) * 7350;
        tmp11 *= 5461;
        let mut tmp12 = (z1 - z2) * 3363;
        tmp0 = tmp1 + tmp2 + tmp3 - z1 * 18730;
        let tmp13 = tmp10 + tmp11 + tmp12 - z1 * 15038;
        z1 = (z2 + z3) * 1136;
        tmp1 += z1 + z2 * 589;
        tmp2 += z1 - z3 * 9222;
        z1 = (z3 - z2) * 11529;
        tmp11 += z1 - z3 * 6278;
        tmp12 += z1 + z2 * 16154;
        z2 += z4;
        z1 = z2 * -5461;
        tmp1 += z1;
        tmp3 += z1 + z4 * 8728;
        z2 *= -10217;
        tmp10 += z2 + z4 * 25733;
        tmp12 += z2;
        z2 = (z3 + z4) * -11086;
        tmp2 += z2;
        tmp3 += z2;
        z4 = (z4 - z3) * 3363;
        tmp10 += z4;
        tmp11 += z4;

        let shift = CONST_BITS + PASS1_BITS + 3;
        let row_start = row * output_linestride;
        let mut store = |column: usize, value: i64| {
            output[row_start + column] = (value >> shift).clamp(0, 255) as u8;
        };
        store(0, tmp20 + tmp0);
        store(15, tmp20 - tmp0);
        store(1, tmp21 + tmp1);
        store(14, tmp21 - tmp1);
        store(2, tmp22 + tmp2);
        store(13, tmp22 - tmp2);
        store(3, tmp23 + tmp3);
        store(12, tmp23 - tmp3);
        store(4, tmp24 + tmp10);
        store(11, tmp24 - tmp10);
        store(5, tmp25 + tmp11);
        store(10, tmp25 - tmp11);
        store(6, tmp26 + tmp12);
        store(9, tmp26 - tmp12);
        store(7, tmp27 + tmp13);
        store(8, tmp27 - tmp13);
    }
}

#[cfg(feature = "rive_v9f")]
fn rive_v9f_idct_16x16(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    const CONST_BITS: u32 = 13;
    const PASS1_BITS: u32 = 2;
    const FIX_0_541196100: i64 = 4433;
    const FIX_0_899976223: i64 = 7373;
    const FIX_2_562915447: i64 = 20995;
    let dequantize =
        |index: usize| i64::from(coefficients[index]) * i64::from(quantization_table[index]);
    let mut workspace = [0_i64; 8 * 16];

    for column in 0..8 {
        let mut tmp0 = dequantize(column) << CONST_BITS;
        tmp0 += 1 << (CONST_BITS - PASS1_BITS - 1);

        let mut z1 = dequantize(column + 8 * 4);
        let tmp1 = z1 * 10703;
        let tmp2 = z1 * FIX_0_541196100;
        let tmp10 = tmp0 + tmp1;
        let tmp11 = tmp0 - tmp1;
        let tmp12 = tmp0 + tmp2;
        let tmp13 = tmp0 - tmp2;

        z1 = dequantize(column + 8 * 2);
        let mut z2 = dequantize(column + 8 * 6);
        let mut z3 = z1 - z2;
        let z4 = z3 * 2260;
        z3 *= 11363;
        tmp0 = z3 + z2 * FIX_2_562915447;
        let tmp1 = z4 + z1 * FIX_0_899976223;
        let tmp2 = z3 - z1 * 4926;
        let tmp3 = z4 - z2 * 4176;

        let tmp20 = tmp10 + tmp0;
        let tmp27 = tmp10 - tmp0;
        let tmp21 = tmp12 + tmp1;
        let tmp26 = tmp12 - tmp1;
        let tmp22 = tmp13 + tmp2;
        let tmp25 = tmp13 - tmp2;
        let tmp23 = tmp11 + tmp3;
        let tmp24 = tmp11 - tmp3;

        z1 = dequantize(column + 8);
        z2 = dequantize(column + 8 * 3);
        z3 = dequantize(column + 8 * 5);
        let mut z4 = dequantize(column + 8 * 7);
        let mut tmp11 = z1 + z3;
        let mut tmp1 = (z1 + z2) * 11086;
        let mut tmp2 = tmp11 * 10217;
        let mut tmp3 = (z1 + z4) * 8956;
        let mut tmp10 = (z1 - z4) * 7350;
        tmp11 *= 5461;
        let mut tmp12 = (z1 - z2) * 3363;
        tmp0 = tmp1 + tmp2 + tmp3 - z1 * 18730;
        let tmp13 = tmp10 + tmp11 + tmp12 - z1 * 15038;
        z1 = (z2 + z3) * 1136;
        tmp1 += z1 + z2 * 589;
        tmp2 += z1 - z3 * 9222;
        z1 = (z3 - z2) * 11529;
        tmp11 += z1 - z3 * 6278;
        tmp12 += z1 + z2 * 16154;
        z2 += z4;
        z1 = z2 * -5461;
        tmp1 += z1;
        tmp3 += z1 + z4 * 8728;
        z2 *= -10217;
        tmp10 += z2 + z4 * 25733;
        tmp12 += z2;
        z2 = (z3 + z4) * -11086;
        tmp2 += z2;
        tmp3 += z2;
        z4 = (z4 - z3) * 3363;
        tmp10 += z4;
        tmp11 += z4;

        let shift = CONST_BITS - PASS1_BITS;
        workspace[column + 8 * 0] = (tmp20 + tmp0) >> shift;
        workspace[column + 8 * 15] = (tmp20 - tmp0) >> shift;
        workspace[column + 8 * 1] = (tmp21 + tmp1) >> shift;
        workspace[column + 8 * 14] = (tmp21 - tmp1) >> shift;
        workspace[column + 8 * 2] = (tmp22 + tmp2) >> shift;
        workspace[column + 8 * 13] = (tmp22 - tmp2) >> shift;
        workspace[column + 8 * 3] = (tmp23 + tmp3) >> shift;
        workspace[column + 8 * 12] = (tmp23 - tmp3) >> shift;
        workspace[column + 8 * 4] = (tmp24 + tmp10) >> shift;
        workspace[column + 8 * 11] = (tmp24 - tmp10) >> shift;
        workspace[column + 8 * 5] = (tmp25 + tmp11) >> shift;
        workspace[column + 8 * 10] = (tmp25 - tmp11) >> shift;
        workspace[column + 8 * 6] = (tmp26 + tmp12) >> shift;
        workspace[column + 8 * 9] = (tmp26 - tmp12) >> shift;
        workspace[column + 8 * 7] = (tmp27 + tmp13) >> shift;
        workspace[column + 8 * 8] = (tmp27 - tmp13) >> shift;
    }

    for row in 0..16 {
        let values = &workspace[row * 8..row * 8 + 8];
        let mut tmp0 =
            (values[0] + ((128 << (PASS1_BITS + 3)) + (1 << (PASS1_BITS + 2)))) << CONST_BITS;
        let mut z1 = values[4];
        let tmp1 = z1 * 10703;
        let tmp2 = z1 * FIX_0_541196100;
        let tmp10 = tmp0 + tmp1;
        let tmp11_base = tmp0 - tmp1;
        let tmp12_base = tmp0 + tmp2;
        let tmp13_base = tmp0 - tmp2;

        z1 = values[2];
        let mut z2 = values[6];
        let mut z3 = z1 - z2;
        let z4 = z3 * 2260;
        z3 *= 11363;
        tmp0 = z3 + z2 * FIX_2_562915447;
        let tmp1 = z4 + z1 * FIX_0_899976223;
        let tmp2 = z3 - z1 * 4926;
        let tmp3 = z4 - z2 * 4176;

        let tmp20 = tmp10 + tmp0;
        let tmp27 = tmp10 - tmp0;
        let tmp21 = tmp12_base + tmp1;
        let tmp26 = tmp12_base - tmp1;
        let tmp22 = tmp13_base + tmp2;
        let tmp25 = tmp13_base - tmp2;
        let tmp23 = tmp11_base + tmp3;
        let tmp24 = tmp11_base - tmp3;

        z1 = values[1];
        z2 = values[3];
        z3 = values[5];
        let mut z4 = values[7];
        let mut tmp11 = z1 + z3;
        let mut tmp1 = (z1 + z2) * 11086;
        let mut tmp2 = tmp11 * 10217;
        let mut tmp3 = (z1 + z4) * 8956;
        let mut tmp10 = (z1 - z4) * 7350;
        tmp11 *= 5461;
        let mut tmp12 = (z1 - z2) * 3363;
        tmp0 = tmp1 + tmp2 + tmp3 - z1 * 18730;
        let tmp13 = tmp10 + tmp11 + tmp12 - z1 * 15038;
        z1 = (z2 + z3) * 1136;
        tmp1 += z1 + z2 * 589;
        tmp2 += z1 - z3 * 9222;
        z1 = (z3 - z2) * 11529;
        tmp11 += z1 - z3 * 6278;
        tmp12 += z1 + z2 * 16154;
        z2 += z4;
        z1 = z2 * -5461;
        tmp1 += z1;
        tmp3 += z1 + z4 * 8728;
        z2 *= -10217;
        tmp10 += z2 + z4 * 25733;
        tmp12 += z2;
        z2 = (z3 + z4) * -11086;
        tmp2 += z2;
        tmp3 += z2;
        z4 = (z4 - z3) * 3363;
        tmp10 += z4;
        tmp11 += z4;

        let shift = CONST_BITS + PASS1_BITS + 3;
        let row_start = row * output_linestride;
        let mut store = |column: usize, value: i64| {
            output[row_start + column] = (value >> shift).clamp(0, 255) as u8;
        };
        store(0, tmp20 + tmp0);
        store(15, tmp20 - tmp0);
        store(1, tmp21 + tmp1);
        store(14, tmp21 - tmp1);
        store(2, tmp22 + tmp2);
        store(13, tmp22 - tmp2);
        store(3, tmp23 + tmp3);
        store(12, tmp23 - tmp3);
        store(4, tmp24 + tmp10);
        store(11, tmp24 - tmp10);
        store(5, tmp25 + tmp11);
        store(10, tmp25 - tmp11);
        store(6, tmp26 + tmp12);
        store(9, tmp26 - tmp12);
        store(7, tmp27 + tmp13);
        store(8, tmp27 - tmp13);
    }
}

// Literal arithmetic owner: rive-app_libjpeg_v9f/jidctint.c,
// jpeg_idct_8x16().  This is the source's 16-point column kernel followed by
// its 8-point row kernel; it is selected for vertical-only subsampling.
#[cfg(feature = "rive_v9f")]
fn rive_v9f_idct_8x16(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    const CONST_BITS: u32 = 13;
    const PASS1_BITS: u32 = 2;
    let dequantize =
        |index: usize| i64::from(coefficients[index]) * i64::from(quantization_table[index]);
    let mut workspace = [0_i64; 8 * 16];

    for column in 0..8 {
        let mut tmp0 = dequantize(column) << CONST_BITS;
        tmp0 += 1 << (CONST_BITS - PASS1_BITS - 1);

        let mut z1 = dequantize(column + 8 * 4);
        let tmp1 = z1 * 10703;
        let tmp2 = z1 * 4433;
        let tmp10 = tmp0 + tmp1;
        let tmp11 = tmp0 - tmp1;
        let tmp12 = tmp0 + tmp2;
        let tmp13 = tmp0 - tmp2;

        z1 = dequantize(column + 8 * 2);
        let mut z2 = dequantize(column + 8 * 6);
        let mut z3 = z1 - z2;
        let z4 = z3 * 2260;
        z3 *= 11363;
        tmp0 = z3 + z2 * 20995;
        let tmp1 = z4 + z1 * 7373;
        let tmp2 = z3 - z1 * 4926;
        let tmp3 = z4 - z2 * 4176;

        let tmp20 = tmp10 + tmp0;
        let tmp27 = tmp10 - tmp0;
        let tmp21 = tmp12 + tmp1;
        let tmp26 = tmp12 - tmp1;
        let tmp22 = tmp13 + tmp2;
        let tmp25 = tmp13 - tmp2;
        let tmp23 = tmp11 + tmp3;
        let tmp24 = tmp11 - tmp3;

        z1 = dequantize(column + 8);
        z2 = dequantize(column + 8 * 3);
        z3 = dequantize(column + 8 * 5);
        let mut z4 = dequantize(column + 8 * 7);
        let mut tmp11 = z1 + z3;
        let mut tmp1 = (z1 + z2) * 11086;
        let mut tmp2 = tmp11 * 10217;
        let mut tmp3 = (z1 + z4) * 8956;
        let mut tmp10 = (z1 - z4) * 7350;
        tmp11 *= 5461;
        let mut tmp12 = (z1 - z2) * 3363;
        tmp0 = tmp1 + tmp2 + tmp3 - z1 * 18730;
        let tmp13 = tmp10 + tmp11 + tmp12 - z1 * 15038;
        z1 = (z2 + z3) * 1136;
        tmp1 += z1 + z2 * 589;
        tmp2 += z1 - z3 * 9222;
        z1 = (z3 - z2) * 11529;
        tmp11 += z1 - z3 * 6278;
        tmp12 += z1 + z2 * 16154;
        z2 += z4;
        z1 = z2 * -5461;
        tmp1 += z1;
        tmp3 += z1 + z4 * 8728;
        z2 *= -10217;
        tmp10 += z2 + z4 * 25733;
        tmp12 += z2;
        z2 = (z3 + z4) * -11086;
        tmp2 += z2;
        tmp3 += z2;
        z4 = (z4 - z3) * 3363;
        tmp10 += z4;
        tmp11 += z4;

        let shift = CONST_BITS - PASS1_BITS;
        workspace[column + 8 * 0] = (tmp20 + tmp0) >> shift;
        workspace[column + 8 * 15] = (tmp20 - tmp0) >> shift;
        workspace[column + 8 * 1] = (tmp21 + tmp1) >> shift;
        workspace[column + 8 * 14] = (tmp21 - tmp1) >> shift;
        workspace[column + 8 * 2] = (tmp22 + tmp2) >> shift;
        workspace[column + 8 * 13] = (tmp22 - tmp2) >> shift;
        workspace[column + 8 * 3] = (tmp23 + tmp3) >> shift;
        workspace[column + 8 * 12] = (tmp23 - tmp3) >> shift;
        workspace[column + 8 * 4] = (tmp24 + tmp10) >> shift;
        workspace[column + 8 * 11] = (tmp24 - tmp10) >> shift;
        workspace[column + 8 * 5] = (tmp25 + tmp11) >> shift;
        workspace[column + 8 * 10] = (tmp25 - tmp11) >> shift;
        workspace[column + 8 * 6] = (tmp26 + tmp12) >> shift;
        workspace[column + 8 * 9] = (tmp26 - tmp12) >> shift;
        workspace[column + 8 * 7] = (tmp27 + tmp13) >> shift;
        workspace[column + 8 * 8] = (tmp27 - tmp13) >> shift;
    }

    for row in 0..16 {
        let values = &workspace[row * 8..row * 8 + 8];
        let mut z2 = values[0] + ((128 << (PASS1_BITS + 3)) + (1 << (PASS1_BITS + 2)));
        let row_start = row * output_linestride;
        if values[1..].iter().all(|value| *value == 0) {
            let value = (z2 >> (PASS1_BITS + 3)).clamp(0, 255) as u8;
            output[row_start..row_start + 8].fill(value);
            continue;
        }

        let mut z3 = values[4];
        let mut tmp0 = (z2 + z3) << CONST_BITS;
        let mut tmp1 = (z2 - z3) << CONST_BITS;
        z2 = values[2];
        z3 = values[6];
        let mut z1 = (z2 + z3) * 4433;
        let tmp2 = z1 + z2 * 6270;
        let tmp3 = z1 - z3 * 15137;
        let tmp10 = tmp0 + tmp2;
        let tmp13 = tmp0 - tmp2;
        let tmp11 = tmp1 + tmp3;
        let tmp12 = tmp1 - tmp3;

        tmp0 = values[7];
        tmp1 = values[5];
        let mut tmp2 = values[3];
        let mut tmp3 = values[1];
        z2 = tmp0 + tmp2;
        z3 = tmp1 + tmp3;
        z1 = (z2 + z3) * 9633;
        z2 = z2 * -16069 + z1;
        z3 = z3 * -3196 + z1;
        z1 = (tmp0 + tmp3) * -7373;
        tmp0 = tmp0 * 2446 + z1 + z2;
        tmp3 = tmp3 * 12299 + z1 + z3;
        z1 = (tmp1 + tmp2) * -20995;
        tmp1 = tmp1 * 16819 + z1 + z3;
        tmp2 = tmp2 * 25172 + z1 + z2;

        let shift = CONST_BITS + PASS1_BITS + 3;
        let mut store = |column: usize, value: i64| {
            output[row_start + column] = (value >> shift).clamp(0, 255) as u8;
        };
        store(0, tmp10 + tmp3);
        store(7, tmp10 - tmp3);
        store(1, tmp11 + tmp2);
        store(6, tmp11 - tmp2);
        store(2, tmp12 + tmp1);
        store(5, tmp12 - tmp1);
        store(3, tmp13 + tmp0);
        store(4, tmp13 - tmp0);
    }
}

pub fn dequantize_and_idct_block_8x8(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    #[cfg(not(feature = "platform_independent"))]
    if let Some(idct) = crate::arch::get_dequantize_and_idct_block_8x8() {
        #[allow(unsafe_code)]
        unsafe {
            return idct(coefficients, quantization_table, output_linestride, output);
        }
    }

    let output = output.chunks_mut(output_linestride);
    dequantize_and_idct_block_8x8_inner(coefficients, quantization_table, output)
}

// This is based on stb_image's 'stbi__idct_block'.
fn dequantize_and_idct_block_8x8_inner<'a, I>(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output: I,
) where
    I: IntoIterator<Item = &'a mut [u8]>,
    I::IntoIter: ExactSizeIterator<Item = &'a mut [u8]>,
{
    let output = output.into_iter();
    debug_assert!(
        output.len() >= 8,
        "Output iterator has the wrong length: {}",
        output.len()
    );

    let mut temp = [Wrapping(0); 64];

    // columns
    for i in 0..8 {
        if coefficients[i + 8] == 0
            && coefficients[i + 16] == 0
            && coefficients[i + 24] == 0
            && coefficients[i + 32] == 0
            && coefficients[i + 40] == 0
            && coefficients[i + 48] == 0
            && coefficients[i + 56] == 0
        {
            let dcterm = dequantize(coefficients[i], quantization_table[i]) << 2;
            temp[i] = dcterm;
            temp[i + 8] = dcterm;
            temp[i + 16] = dcterm;
            temp[i + 24] = dcterm;
            temp[i + 32] = dcterm;
            temp[i + 40] = dcterm;
            temp[i + 48] = dcterm;
            temp[i + 56] = dcterm;
        } else {
            let s0 = dequantize(coefficients[i], quantization_table[i]);
            let s1 = dequantize(coefficients[i + 8], quantization_table[i + 8]);
            let s2 = dequantize(coefficients[i + 16], quantization_table[i + 16]);
            let s3 = dequantize(coefficients[i + 24], quantization_table[i + 24]);
            let s4 = dequantize(coefficients[i + 32], quantization_table[i + 32]);
            let s5 = dequantize(coefficients[i + 40], quantization_table[i + 40]);
            let s6 = dequantize(coefficients[i + 48], quantization_table[i + 48]);
            let s7 = dequantize(coefficients[i + 56], quantization_table[i + 56]);

            let Kernel {
                xs: [x0, x1, x2, x3],
                ts: [t0, t1, t2, t3],
            } = kernel(
                [s0, s1, s2, s3, s4, s5, s6, s7],
                // constants scaled things up by 1<<12; let's bring them back
                // down, but keep 2 extra bits of precision
                512,
            );

            temp[i] = (x0 + t3) >> 10;
            temp[i + 56] = (x0 - t3) >> 10;
            temp[i + 8] = (x1 + t2) >> 10;
            temp[i + 48] = (x1 - t2) >> 10;
            temp[i + 16] = (x2 + t1) >> 10;
            temp[i + 40] = (x2 - t1) >> 10;
            temp[i + 24] = (x3 + t0) >> 10;
            temp[i + 32] = (x3 - t0) >> 10;
        }
    }

    for (chunk, output_chunk) in temp.chunks_exact(8).zip(output) {
        let chunk = <&[_; 8]>::try_from(chunk).unwrap();

        // constants scaled things up by 1<<12, plus we had 1<<2 from first
        // loop, plus horizontal and vertical each scale by sqrt(8) so together
        // we've got an extra 1<<3, so 1<<17 total we need to remove.
        // so we want to round that, which means adding 0.5 * 1<<17,
        // aka 65536. Also, we'll end up with -128 to 127 that we want
        // to encode as 0..255 by adding 128, so we'll add that before the shift
        const X_SCALE: i32 = 65536 + (128 << 17);

        // eliminate downstream bounds checks
        let output_chunk = &mut output_chunk[..8];

        // TODO When the minimum rust version supports it
        // let [s0, rest @ ..] = chunk;
        let (s0, rest) = chunk.split_first().unwrap();
        if *rest == [Wrapping(0); 7] {
            let dcterm = stbi_clamp((stbi_fsh(*s0) + Wrapping(X_SCALE)) >> 17);
            output_chunk[0] = dcterm;
            output_chunk[1] = dcterm;
            output_chunk[2] = dcterm;
            output_chunk[3] = dcterm;
            output_chunk[4] = dcterm;
            output_chunk[5] = dcterm;
            output_chunk[6] = dcterm;
            output_chunk[7] = dcterm;
        } else {
            let Kernel {
                xs: [x0, x1, x2, x3],
                ts: [t0, t1, t2, t3],
            } = kernel(*chunk, X_SCALE);

            output_chunk[0] = stbi_clamp((x0 + t3) >> 17);
            output_chunk[7] = stbi_clamp((x0 - t3) >> 17);
            output_chunk[1] = stbi_clamp((x1 + t2) >> 17);
            output_chunk[6] = stbi_clamp((x1 - t2) >> 17);
            output_chunk[2] = stbi_clamp((x2 + t1) >> 17);
            output_chunk[5] = stbi_clamp((x2 - t1) >> 17);
            output_chunk[3] = stbi_clamp((x3 + t0) >> 17);
            output_chunk[4] = stbi_clamp((x3 - t0) >> 17);
        }
    }
}

struct Kernel {
    xs: [Wrapping<i32>; 4],
    ts: [Wrapping<i32>; 4],
}

#[inline]
fn kernel_x([s0, s2, s4, s6]: [Wrapping<i32>; 4], x_scale: i32) -> [Wrapping<i32>; 4] {
    // Even `chunk` indicies
    let (t2, t3);
    {
        let p2 = s2;
        let p3 = s6;

        let p1 = (p2 + p3) * stbi_f2f(0.5411961);
        t2 = p1 + p3 * stbi_f2f(-1.847759065);
        t3 = p1 + p2 * stbi_f2f(0.765366865);
    }

    let (t0, t1);
    {
        let p2 = s0;
        let p3 = s4;

        t0 = stbi_fsh(p2 + p3);
        t1 = stbi_fsh(p2 - p3);
    }

    let x0 = t0 + t3;
    let x3 = t0 - t3;
    let x1 = t1 + t2;
    let x2 = t1 - t2;

    let x_scale = Wrapping(x_scale);

    [x0 + x_scale, x1 + x_scale, x2 + x_scale, x3 + x_scale]
}

#[inline]
fn kernel_t([s1, s3, s5, s7]: [Wrapping<i32>; 4]) -> [Wrapping<i32>; 4] {
    // Odd `chunk` indicies
    let mut t0 = s7;
    let mut t1 = s5;
    let mut t2 = s3;
    let mut t3 = s1;

    let p3 = t0 + t2;
    let p4 = t1 + t3;
    let p1 = t0 + t3;
    let p2 = t1 + t2;
    let p5 = (p3 + p4) * stbi_f2f(1.175875602);

    t0 *= stbi_f2f(0.298631336);
    t1 *= stbi_f2f(2.053119869);
    t2 *= stbi_f2f(3.072711026);
    t3 *= stbi_f2f(1.501321110);

    let p1 = p5 + p1 * stbi_f2f(-0.899976223);
    let p2 = p5 + p2 * stbi_f2f(-2.562915447);
    let p3 = p3 * stbi_f2f(-1.961570560);
    let p4 = p4 * stbi_f2f(-0.390180644);

    t3 += p1 + p4;
    t2 += p2 + p3;
    t1 += p2 + p4;
    t0 += p1 + p3;

    [t0, t1, t2, t3]
}

#[inline]
fn kernel([s0, s1, s2, s3, s4, s5, s6, s7]: [Wrapping<i32>; 8], x_scale: i32) -> Kernel {
    Kernel {
        xs: kernel_x([s0, s2, s4, s6], x_scale),
        ts: kernel_t([s1, s3, s5, s7]),
    }
}

#[inline(always)]
fn dequantize(c: i16, q: u16) -> Wrapping<i32> {
    Wrapping(i32::from(c) * i32::from(q))
}

// 4x4 and 2x2 IDCT based on Rakesh Dugad and Narendra Ahuja: "A Fast Scheme for Image Size Change in the Compressed Domain" (2001).
// http://sylvana.net/jpegcrop/jidctred/
fn dequantize_and_idct_block_4x4(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    debug_assert_eq!(coefficients.len(), 64);
    let mut temp = [Wrapping(0i32); 4 * 4];

    const CONST_BITS: usize = 12;
    const PASS1_BITS: usize = 2;
    const FINAL_BITS: usize = CONST_BITS + PASS1_BITS + 3;

    // columns
    for i in 0..4 {
        let s0 = Wrapping(coefficients[i + 8 * 0] as i32 * quantization_table[i + 8 * 0] as i32);
        let s1 = Wrapping(coefficients[i + 8 * 1] as i32 * quantization_table[i + 8 * 1] as i32);
        let s2 = Wrapping(coefficients[i + 8 * 2] as i32 * quantization_table[i + 8 * 2] as i32);
        let s3 = Wrapping(coefficients[i + 8 * 3] as i32 * quantization_table[i + 8 * 3] as i32);

        let x0 = (s0 + s2) << PASS1_BITS;
        let x2 = (s0 - s2) << PASS1_BITS;

        let p1 = (s1 + s3) * stbi_f2f(0.541196100);
        let t0 = (p1 + s3 * stbi_f2f(-1.847759065) + Wrapping(512)) >> (CONST_BITS - PASS1_BITS);
        let t2 = (p1 + s1 * stbi_f2f(0.765366865) + Wrapping(512)) >> (CONST_BITS - PASS1_BITS);

        temp[i + 4 * 0] = x0 + t2;
        temp[i + 4 * 3] = x0 - t2;
        temp[i + 4 * 1] = x2 + t0;
        temp[i + 4 * 2] = x2 - t0;
    }

    for i in 0..4 {
        let s0 = temp[i * 4 + 0];
        let s1 = temp[i * 4 + 1];
        let s2 = temp[i * 4 + 2];
        let s3 = temp[i * 4 + 3];

        let x0 = (s0 + s2) << CONST_BITS;
        let x2 = (s0 - s2) << CONST_BITS;

        let p1 = (s1 + s3) * stbi_f2f(0.541196100);
        let t0 = p1 + s3 * stbi_f2f(-1.847759065);
        let t2 = p1 + s1 * stbi_f2f(0.765366865);

        // constants scaled things up by 1<<12, plus we had 1<<2 from first
        // loop, plus horizontal and vertical each scale by sqrt(8) so together
        // we've got an extra 1<<3, so 1<<17 total we need to remove.
        // so we want to round that, which means adding 0.5 * 1<<17,
        // aka 65536. Also, we'll end up with -128 to 127 that we want
        // to encode as 0..255 by adding 128, so we'll add that before the shift
        let x0 = x0 + Wrapping(1 << (FINAL_BITS - 1)) + Wrapping(128 << FINAL_BITS);
        let x2 = x2 + Wrapping(1 << (FINAL_BITS - 1)) + Wrapping(128 << FINAL_BITS);

        let output = &mut output[i * output_linestride..][..4];
        output[0] = stbi_clamp((x0 + t2) >> FINAL_BITS);
        output[3] = stbi_clamp((x0 - t2) >> FINAL_BITS);
        output[1] = stbi_clamp((x2 + t0) >> FINAL_BITS);
        output[2] = stbi_clamp((x2 - t0) >> FINAL_BITS);
    }
}

fn dequantize_and_idct_block_2x2(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    output_linestride: usize,
    output: &mut [u8],
) {
    debug_assert_eq!(coefficients.len(), 64);

    const SCALE_BITS: usize = 3;

    // Column 0
    let s00 = Wrapping(coefficients[8 * 0] as i32 * quantization_table[8 * 0] as i32);
    let s10 = Wrapping(coefficients[8 * 1] as i32 * quantization_table[8 * 1] as i32);

    let x0 = s00 + s10;
    let x2 = s00 - s10;

    // Column 1
    let s01 = Wrapping(coefficients[8 * 0 + 1] as i32 * quantization_table[8 * 0 + 1] as i32);
    let s11 = Wrapping(coefficients[8 * 1 + 1] as i32 * quantization_table[8 * 1 + 1] as i32);

    let x1 = s01 + s11;
    let x3 = s01 - s11;

    let x0 = x0 + Wrapping(1 << (SCALE_BITS - 1)) + Wrapping(128 << SCALE_BITS);
    let x2 = x2 + Wrapping(1 << (SCALE_BITS - 1)) + Wrapping(128 << SCALE_BITS);

    // Row 0
    output[0] = stbi_clamp((x0 + x1) >> SCALE_BITS);
    output[1] = stbi_clamp((x0 - x1) >> SCALE_BITS);

    // Row 1
    output[output_linestride + 0] = stbi_clamp((x2 + x3) >> SCALE_BITS);
    output[output_linestride + 1] = stbi_clamp((x2 - x3) >> SCALE_BITS);
}

fn dequantize_and_idct_block_1x1(
    coefficients: &[i16; 64],
    quantization_table: &[u16; 64],
    _output_linestride: usize,
    output: &mut [u8],
) {
    debug_assert_eq!(coefficients.len(), 64);

    let s0 = (Wrapping(coefficients[0] as i32 * quantization_table[0] as i32) + Wrapping(128 * 8))
        / Wrapping(8);
    output[0] = stbi_clamp(s0);
}

// take a -128..127 value and stbi__clamp it and convert to 0..255
fn stbi_clamp(x: Wrapping<i32>) -> u8 {
    x.0.max(0).min(255) as u8
}

fn stbi_f2f(x: f32) -> Wrapping<i32> {
    Wrapping((x * 4096.0 + 0.5) as i32)
}

fn stbi_fsh(x: Wrapping<i32>) -> Wrapping<i32> {
    x << 12
}

#[test]
fn test_dequantize_and_idct_block_8x8() {
    #[rustfmt::skip]
    let coefficients: [i16; 8 * 8] = [
        -14, -39, 58, -2, 3, 3, 0, 1,
        11, 27, 4, -3, 3, 0, 1, 0,
        -6, -13, -9, -1, -2, -1, 0, 0,
        -4, 0, -1, -2, 0, 0, 0, 0,
        3, 0, 0, 0, 0, 0, 0, 0,
        -3, -2, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0
    ];

    #[rustfmt::skip]
    let quantization_table: [u16; 8 * 8] = [
        8, 6, 5, 8, 12, 20, 26, 31,
        6, 6, 7, 10, 13, 29, 30, 28,
        7, 7, 8, 12, 20, 29, 35, 28,
        7, 9, 11, 15, 26, 44, 40, 31,
        9, 11, 19, 28, 34, 55, 52, 39,
        12, 18, 28, 32, 41, 52, 57, 46,
        25, 32, 39, 44, 52, 61, 60, 51,
        36, 46, 48, 49, 56, 50, 52, 50
    ];
    let output_linestride: usize = 8;
    let mut output = [0u8; 8 * 8];
    dequantize_and_idct_block_8x8(
        &coefficients,
        &quantization_table,
        output_linestride,
        &mut output,
    );
    #[rustfmt::skip]
    let expected_output = [
        118, 92, 110, 83, 77, 93, 144, 198,
        172, 116, 114, 87, 78, 93, 146, 191,
        194, 107, 91, 76, 71, 93, 160, 198,
        196, 100, 80, 74, 67, 92, 174, 209,
        182, 104, 88, 81, 68, 89, 178, 206,
        105, 64, 59, 59, 63, 94, 183, 201,
        35, 27, 28, 37, 72, 121, 203, 204,
        37, 45, 41, 47, 98, 154, 223, 208
    ];
    for i in 0..64 {
        assert!((output[i] as i16 - expected_output[i] as i16).abs() <= 1);
    }
}

#[test]
fn test_dequantize_and_idct_block_8x8_all_zero() {
    let mut output = [0u8; 8 * 8];
    dequantize_and_idct_block_8x8(&[0; 8 * 8], &[666; 8 * 8], 8, &mut output);
    assert_eq!(&output[..], &[128; 8 * 8][..]);
}

#[test]
fn test_dequantize_and_idct_block_8x8_saturated() {
    // Arch-specific IDCT implementations need not handle i16::MAX values.
    #[cfg(not(feature = "platform_independent"))]
    if crate::arch::get_dequantize_and_idct_block_8x8().is_some() {
        return;
    }
    let mut output = [0u8; 8 * 8];
    dequantize_and_idct_block_8x8(&[i16::MAX; 8 * 8], &[u16::MAX; 8 * 8], 8, &mut output);
    #[rustfmt::skip]
    let expected = [
        0, 0, 0, 255, 255, 0, 0, 255,
        0, 0, 215, 0, 0, 255, 255, 0,
        255, 255, 255, 255, 255, 0, 0, 255,
        0, 0, 255, 0, 255, 0, 255, 255,
        0, 0, 255, 255, 0, 255, 0, 0,
        255, 255, 0, 255, 255, 255, 170, 0,
        0, 255, 0, 0, 0, 0, 0, 255,
        255, 255, 0, 255, 0, 255, 0, 0
    ];
    assert_eq!(&output[..], &expected[..]);
}
