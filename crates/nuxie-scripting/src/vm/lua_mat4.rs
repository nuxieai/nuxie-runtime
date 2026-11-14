//! Rive's single-precision, column-major `Mat4` Luau userdata.
//!
//! `lua_mat4_safe.rs` binds this module's matrix math on the safe VM.

#[path = "lua_mat4_safe.rs"]
mod safe;
pub(super) use safe::install_mat4_global;

const IDENTITY: [f32; 16] = [
    1.0, 0.0, 0.0, 0.0, // column 0
    0.0, 1.0, 0.0, 0.0, // column 1
    0.0, 0.0, 1.0, 0.0, // column 2
    0.0, 0.0, 0.0, 1.0, // column 3
];

#[derive(Clone, Copy)]
#[repr(transparent)]
struct ScriptedMat4 {
    values: [f32; 16],
}

/// One column of a column-major matrix.
type Column = [f32; 4];

/// The matrix's four columns, in storage order.
fn columns(values: [f32; 16]) -> [Column; 4] {
    let [
        a0,
        a1,
        a2,
        a3,
        b0,
        b1,
        b2,
        b3,
        c0,
        c1,
        c2,
        c3,
        d0,
        d1,
        d2,
        d3,
    ] = values;
    [
        [a0, a1, a2, a3],
        [b0, b1, b2, b3],
        [c0, c1, c2, c3],
        [d0, d1, d2, d3],
    ]
}

fn from_columns(columns: [Column; 4]) -> [f32; 16] {
    let [
        [a0, a1, a2, a3],
        [b0, b1, b2, b3],
        [c0, c1, c2, c3],
        [d0, d1, d2, d3],
    ] = columns;
    [
        a0, a1, a2, a3, b0, b1, b2, b3, c0, c1, c2, c3, d0, d1, d2, d3,
    ]
}

/// A column whose entry in each row is `combine` of the four columns'
/// entries in that row.
fn by_row(columns: [Column; 4], combine: impl Fn(f32, f32, f32, f32) -> f32) -> Column {
    let [
        [a0, a1, a2, a3],
        [b0, b1, b2, b3],
        [c0, c1, c2, c3],
        [d0, d1, d2, d3],
    ] = columns;
    [
        combine(a0, b0, c0, d0),
        combine(a1, b1, c1, d1),
        combine(a2, b2, c2, d2),
        combine(a3, b3, c3, d3),
    ]
}

/// The matrix with rows and columns exchanged.
fn transpose(values: [f32; 16]) -> [f32; 16] {
    let [
        [a0, a1, a2, a3],
        [b0, b1, b2, b3],
        [c0, c1, c2, c3],
        [d0, d1, d2, d3],
    ] = columns(values);
    from_columns([
        [a0, b0, c0, d0],
        [a1, b1, c1, d1],
        [a2, b2, c2, d2],
        [a3, b3, c3, d3],
    ])
}

impl ScriptedMat4 {
    fn transform_vec4(&self, x: f32, y: f32, z: f32, w: f32) -> [f32; 4] {
        by_row(columns(self.values), |a, b, c, d| {
            a * x + b * y + c * z + d * w
        })
    }
}

/// Storage index by `mNN` name (row, then column), in storage order.
const ELEMENT_NAMES: [&[u8]; 16] = [
    b"m11", b"m21", b"m31", b"m41", // column 1
    b"m12", b"m22", b"m32", b"m42", // column 2
    b"m13", b"m23", b"m33", b"m43", // column 3
    b"m14", b"m24", b"m34", b"m44", // column 4
];

fn matrix_index(name: &[u8]) -> Option<usize> {
    if let Some(index) = ELEMENT_NAMES.iter().position(|element| *element == name) {
        return Some(index);
    }

    // Mirror C strtol's useful subset for the upstream 1-2 byte fallback:
    // one leading ASCII whitespace or `+`, then decimal digits. A `-` makes
    // every value negative, so it never names an element.
    let digits = match name {
        [lead, digit] if lead.is_ascii_whitespace() || *lead == b'+' => std::slice::from_ref(digit),
        _ => name,
    };
    if !(1..=2).contains(&digits.len()) || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let value: usize = std::str::from_utf8(digits).ok()?.parse().ok()?;
    if (1..=16).contains(&value) {
        value.checked_sub(1)
    } else {
        None
    }
}

fn multiply(lhs: [f32; 16], rhs: [f32; 16]) -> [f32; 16] {
    let lhs = columns(lhs);
    from_columns(
        columns(rhs).map(|[x, y, z, w]| by_row(lhs, |a, b, c, d| a * x + b * y + c * z + d * w)),
    )
}

fn multiply_affine(lhs: [f32; 16], rhs: [f32; 16]) -> [f32; 16] {
    let lhs = columns(lhs);
    let [r0, r1, r2, [x, y, z, _]] = columns(rhs);
    let [o0, o1, o2] =
        [r0, r1, r2].map(|[x, y, z, _]| by_row(lhs, |a, b, c, _| a * x + b * y + c * z));
    let o3 = by_row(lhs, |a, b, c, d| a * x + b * y + c * z + d);
    from_columns([o0, o1, o2, o3])
}

fn invert(matrix: [f32; 16]) -> Option<[f32; 16]> {
    let m = matrix;
    let mut inverse = [0.0; 16];

    inverse[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
        + m[9] * m[7] * m[14]
        + m[13] * m[6] * m[11]
        - m[13] * m[7] * m[10];
    inverse[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
        - m[8] * m[7] * m[14]
        - m[12] * m[6] * m[11]
        + m[12] * m[7] * m[10];
    inverse[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
        + m[8] * m[7] * m[13]
        + m[12] * m[5] * m[11]
        - m[12] * m[7] * m[9];
    inverse[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
        - m[8] * m[6] * m[13]
        - m[12] * m[5] * m[10]
        + m[12] * m[6] * m[9];
    inverse[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
        - m[9] * m[3] * m[14]
        - m[13] * m[2] * m[11]
        + m[13] * m[3] * m[10];
    inverse[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
        + m[8] * m[3] * m[14]
        + m[12] * m[2] * m[11]
        - m[12] * m[3] * m[10];
    inverse[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
        - m[8] * m[3] * m[13]
        - m[12] * m[1] * m[11]
        + m[12] * m[3] * m[9];
    inverse[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
        + m[8] * m[2] * m[13]
        + m[12] * m[1] * m[10]
        - m[12] * m[2] * m[9];
    inverse[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
        + m[5] * m[3] * m[14]
        + m[13] * m[2] * m[7]
        - m[13] * m[3] * m[6];
    inverse[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
        - m[4] * m[3] * m[14]
        - m[12] * m[2] * m[7]
        + m[12] * m[3] * m[6];
    inverse[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
        + m[4] * m[3] * m[13]
        + m[12] * m[1] * m[7]
        - m[12] * m[3] * m[5];
    inverse[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
        - m[4] * m[2] * m[13]
        - m[12] * m[1] * m[6]
        + m[12] * m[2] * m[5];
    inverse[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
        - m[5] * m[3] * m[10]
        - m[9] * m[2] * m[7]
        + m[9] * m[3] * m[6];
    inverse[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
        + m[4] * m[3] * m[10]
        + m[8] * m[2] * m[7]
        - m[8] * m[3] * m[6];
    inverse[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
        - m[4] * m[3] * m[9]
        - m[8] * m[1] * m[7]
        + m[8] * m[3] * m[5];
    inverse[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
        + m[4] * m[2] * m[9]
        + m[8] * m[1] * m[6]
        - m[8] * m[2] * m[5];

    let determinant =
        m[0] * inverse[0] + m[1] * inverse[4] + m[2] * inverse[8] + m[3] * inverse[12];
    if determinant == 0.0 {
        return None;
    }
    let inverse_determinant = 1.0 / determinant;
    for value in &mut inverse {
        *value *= inverse_determinant;
    }
    Some(inverse)
}

fn invert_affine(m: [f32; 16]) -> Option<[f32; 16]> {
    let cofactor_00 = m[5] * m[10] - m[6] * m[9];
    let cofactor_10 = m[6] * m[8] - m[4] * m[10];
    let cofactor_20 = m[4] * m[9] - m[5] * m[8];
    let determinant = m[0] * cofactor_00 + m[1] * cofactor_10 + m[2] * cofactor_20;
    if determinant == 0.0 {
        return None;
    }
    let inverse_determinant = 1.0 / determinant;
    let cofactor_01 = m[2] * m[9] - m[1] * m[10];
    let cofactor_02 = m[1] * m[6] - m[2] * m[5];
    let cofactor_11 = m[0] * m[10] - m[2] * m[8];
    let cofactor_12 = m[2] * m[4] - m[0] * m[6];
    let cofactor_21 = m[1] * m[8] - m[0] * m[9];
    let cofactor_22 = m[0] * m[5] - m[1] * m[4];

    let r0_0 = cofactor_00 * inverse_determinant;
    let r0_1 = cofactor_10 * inverse_determinant;
    let r0_2 = cofactor_20 * inverse_determinant;
    let r1_0 = cofactor_01 * inverse_determinant;
    let r1_1 = cofactor_11 * inverse_determinant;
    let r1_2 = cofactor_21 * inverse_determinant;
    let r2_0 = cofactor_02 * inverse_determinant;
    let r2_1 = cofactor_12 * inverse_determinant;
    let r2_2 = cofactor_22 * inverse_determinant;

    let translation_x = m[12];
    let translation_y = m[13];
    let translation_z = m[14];
    let inverse_x = -(r0_0 * translation_x + r0_1 * translation_y + r0_2 * translation_z);
    let inverse_y = -(r1_0 * translation_x + r1_1 * translation_y + r1_2 * translation_z);
    let inverse_z = -(r2_0 * translation_x + r2_1 * translation_y + r2_2 * translation_z);

    Some([
        r0_0, r1_0, r2_0, 0.0, r0_1, r1_1, r2_1, 0.0, r0_2, r1_2, r2_2, 0.0, inverse_x, inverse_y,
        inverse_z, 1.0,
    ])
}

#[cfg(test)]
mod shape_tests {
    //! The destructured matrix math against the index arithmetic it
    //! replaced (kept here as the reference), bit for bit.
    #![allow(clippy::indexing_slicing, clippy::arithmetic_side_effects)]

    use super::*;

    /// The old resolver, except that its `then_some(value - 1)` (which
    /// underflowed for the name "0" before being discarded) is lazy here.
    fn reference_index(name: &[u8]) -> Option<usize> {
        if let [b'm', row, column] = name {
            let row = row.wrapping_sub(b'0');
            let column = column.wrapping_sub(b'0');
            if (1..=4).contains(&row) && (1..=4).contains(&column) {
                return Some(usize::from(column - 1) * 4 + usize::from(row - 1));
            }
        }
        if !(1..=2).contains(&name.len()) {
            return None;
        }
        let mut cursor = 0;
        while cursor < name.len() && name[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let negative = if cursor < name.len() && matches!(name[cursor], b'+' | b'-') {
            let negative = name[cursor] == b'-';
            cursor += 1;
            negative
        } else {
            false
        };
        let first_digit = cursor;
        let mut value = 0_usize;
        while cursor < name.len() && name[cursor].is_ascii_digit() {
            value = value * 10 + usize::from(name[cursor] - b'0');
            cursor += 1;
        }
        (cursor > first_digit && cursor == name.len() && !negative && (1..=16).contains(&value))
            .then(|| value - 1)
    }

    fn reference_multiply(lhs: [f32; 16], rhs: [f32; 16]) -> [f32; 16] {
        let mut output = [0.0; 16];
        for column in 0..4 {
            for row in 0..4 {
                output[column * 4 + row] = lhs[row] * rhs[column * 4]
                    + lhs[4 + row] * rhs[column * 4 + 1]
                    + lhs[8 + row] * rhs[column * 4 + 2]
                    + lhs[12 + row] * rhs[column * 4 + 3];
            }
        }
        output
    }

    fn reference_multiply_affine(lhs: [f32; 16], rhs: [f32; 16]) -> [f32; 16] {
        let mut output = [0.0; 16];
        for column in 0..3 {
            for row in 0..4 {
                output[column * 4 + row] = lhs[row] * rhs[column * 4]
                    + lhs[4 + row] * rhs[column * 4 + 1]
                    + lhs[8 + row] * rhs[column * 4 + 2];
            }
        }
        for row in 0..4 {
            output[12 + row] = lhs[row] * rhs[12]
                + lhs[4 + row] * rhs[13]
                + lhs[8 + row] * rhs[14]
                + lhs[12 + row];
        }
        output
    }

    fn reference_transpose(matrix: [f32; 16]) -> [f32; 16] {
        let mut values = [0.0; 16];
        for row in 0..4 {
            for column in 0..4 {
                values[row * 4 + column] = matrix[column * 4 + row];
            }
        }
        values
    }

    fn reference_transform(values: [f32; 16], v: [f32; 4]) -> [f32; 4] {
        std::array::from_fn(|row| {
            values[row] * v[0]
                + values[4 + row] * v[1]
                + values[8 + row] * v[2]
                + values[12 + row] * v[3]
        })
    }

    #[test]
    fn element_names_resolve_as_the_index_arithmetic_did() {
        let mut names: Vec<Vec<u8>> = vec![Vec::new()];
        for a in 0..=255_u8 {
            names.push(vec![a]);
            for b in 0..=255_u8 {
                names.push(vec![a, b]);
                names.push(vec![b'm', a, b]);
            }
        }
        for name in &names {
            assert_eq!(matrix_index(name), reference_index(name), "{name:?}");
        }
    }

    /// Values whose products and sums round differently under any change
    /// of order or fusing, plus signed zeros, infinities and NaN.
    fn samples() -> Vec<[f32; 16]> {
        let mut state = 0x9e37_79b9_u32;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            f32::from_bits(0x3c00_0000 | (state & 0x03ff_ffff))
                * if state & 1 == 0 { 1.0 } else { -3.0 }
        };
        let mut samples: Vec<[f32; 16]> =
            (0..64).map(|_| std::array::from_fn(|_| next())).collect();
        let special = [
            0.0,
            -0.0,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NAN,
            f32::MIN_POSITIVE,
            1.0e30,
        ];
        samples.push(std::array::from_fn(|i| special[i % special.len()]));
        samples.push(IDENTITY);
        samples
    }

    #[test]
    fn destructured_math_is_bit_exact_to_the_indexed_loops() {
        let bits = |values: [f32; 16]| values.map(f32::to_bits);
        let samples = samples();
        for lhs in &samples {
            assert_eq!(bits(transpose(*lhs)), bits(reference_transpose(*lhs)));
            let v = [lhs[3], lhs[7], -lhs[11], lhs[15]];
            assert_eq!(
                ScriptedMat4 { values: *lhs }
                    .transform_vec4(v[0], v[1], v[2], v[3])
                    .map(f32::to_bits),
                reference_transform(*lhs, v).map(f32::to_bits)
            );
            for rhs in &samples {
                assert_eq!(
                    bits(multiply(*lhs, *rhs)),
                    bits(reference_multiply(*lhs, *rhs))
                );
                assert_eq!(
                    bits(multiply_affine(*lhs, *rhs)),
                    bits(reference_multiply_affine(*lhs, *rhs))
                );
            }
        }
    }
}
