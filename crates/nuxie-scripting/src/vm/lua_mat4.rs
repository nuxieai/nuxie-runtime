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

impl ScriptedMat4 {
    fn transform_vec4(&self, x: f32, y: f32, z: f32, w: f32) -> [f32; 4] {
        std::array::from_fn(|row| {
            self.values[row] * x
                + self.values[4 + row] * y
                + self.values[8 + row] * z
                + self.values[12 + row] * w
        })
    }
}

fn matrix_index(name: &[u8]) -> Option<usize> {
    if let [b'm', row, column] = name {
        let row = row.wrapping_sub(b'0');
        let column = column.wrapping_sub(b'0');
        if (1..=4).contains(&row) && (1..=4).contains(&column) {
            return Some(usize::from(column - 1) * 4 + usize::from(row - 1));
        }
    }

    // Mirror C strtol's useful subset for the upstream 1-2 byte fallback:
    // leading ASCII whitespace and an optional sign, then decimal digits.
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
        .then_some(value - 1)
}

fn multiply(lhs: [f32; 16], rhs: [f32; 16]) -> [f32; 16] {
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

fn multiply_affine(lhs: [f32; 16], rhs: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0; 16];
    for column in 0..3 {
        for row in 0..4 {
            output[column * 4 + row] = lhs[row] * rhs[column * 4]
                + lhs[4 + row] * rhs[column * 4 + 1]
                + lhs[8 + row] * rhs[column * 4 + 2];
        }
    }
    for row in 0..4 {
        output[12 + row] =
            lhs[row] * rhs[12] + lhs[4 + row] * rhs[13] + lhs[8 + row] * rhs[14] + lhs[12 + row];
    }
    output
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
