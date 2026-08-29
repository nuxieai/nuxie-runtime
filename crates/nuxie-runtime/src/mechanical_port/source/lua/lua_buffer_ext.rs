use crate::mechanical_port::source::lua::rive_lua_libs::LuaState;
fn half_to_float(h: u16) -> f32 {
    let sign = ((h >> 15) as u32) << 31;
    let mut exp = ((h >> 10) & 0x1f) as u32;
    let mut mant = (h & 0x3ff) as u32;
    if exp == 0 {
        if mant == 0 {
            return f32::from_bits(sign);
        }
        exp = 1;
        while mant & 0x400 == 0 {
            mant <<= 1;
            exp = exp.wrapping_sub(1);
        }
        mant &= 0x3ff;
        return f32::from_bits(sign | ((exp + 127 - 15) << 23) | (mant << 13));
    }
    if exp == 31 {
        return f32::from_bits(sign | 0x7f800000 | (mant << 13));
    }
    f32::from_bits(sign | ((exp + 127 - 15) << 23) | (mant << 13))
}
fn float_to_half(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let mut exp = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    let mut mant = bits & 0x7fffff;
    if exp <= 0 {
        if exp < -10 {
            return sign;
        }
        mant |= 0x800000;
        let shift = (1 - exp) as u32;
        let round = mant & ((1u32 << (shift + 13)) - 1);
        mant >>= shift + 13;
        if round > (1 << (shift + 12)) || (round == (1 << (shift + 12)) && mant & 1 != 0) {
            mant += 1;
        }
        return sign | mant as u16;
    }
    if exp == 0xff - 127 + 15 {
        return if mant == 0 {
            sign | 0x7c00
        } else {
            sign | 0x7c00 | (mant >> 13) as u16
        };
    }
    let round = mant & 0x1fff;
    mant >>= 13;
    if round > 0x1000 || (round == 0x1000 && mant & 1 != 0) {
        mant += 1;
        if mant >= 0x400 {
            mant = 0;
            exp += 1;
        }
    }
    if exp >= 31 {
        return sign | 0x7c00;
    }
    sign | ((exp as u16) << 10) | mant as u16
}
fn read_f16(s: &mut LuaState) -> i32 {
    let buffer = s.check_buffer(1).to_vec();
    let offset = s.check_integer(2);
    if offset < 0 || offset as usize + 2 > buffer.len() {
        return s.error("buffer access out of bounds");
    }
    s.push_number(half_to_float(u16::from_ne_bytes(
        buffer[offset as usize..offset as usize + 2]
            .try_into()
            .unwrap(),
    )) as f64);
    1
}
fn write_f16(s: &mut LuaState) -> i32 {
    let buffer_len = s.check_buffer(1).len();
    let offset = s.check_integer(2);
    let value = s.check_number(3) as f32;
    if offset < 0 || offset as usize + 2 > buffer_len {
        return s.error("buffer access out of bounds");
    }
    let buffer = s.check_buffer_mut(1);
    buffer[offset as usize..offset as usize + 2]
        .copy_from_slice(&float_to_half(value).to_ne_bytes());
    0
}
fn strided_copy(s: &mut LuaState) -> i32 {
    let dst_len = s.check_buffer(1).len();
    let dst_off = s.check_integer(2);
    let dst_stride = s.check_integer(3);
    let src = s.check_buffer(4).to_vec();
    let src_off = s.check_integer(5);
    let src_stride = s.check_integer(6);
    let size = s.check_integer(7);
    let count = s.check_integer(8);
    if size < 0 {
        return s.error("elementSize must be non-negative");
    }
    if count < 0 {
        return s.error("count must be non-negative");
    }
    if count == 0 {
        return 0;
    }
    if src_stride < size || dst_stride < size {
        return s.error("stride must be >= elementSize");
    }
    let src_end = src_off as i64 + (count - 1) as i64 * src_stride as i64 + size as i64;
    let dst_end = dst_off as i64 + (count - 1) as i64 * dst_stride as i64 + size as i64;
    if src_off < 0 || src_end > src.len() as i64 || dst_off < 0 || dst_end > dst_len as i64 {
        return s.error("buffer access out of bounds");
    }
    let dst = s.check_buffer_mut(1);
    for i in 0..count as usize {
        let so = src_off as usize + i * src_stride as usize;
        let d = dst_off as usize + i * dst_stride as usize;
        dst[d..d + size as usize].copy_from_slice(&src[so..so + size as usize]);
    }
    0
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    F16,
    F32,
    U8,
    U8Norm,
    I8Norm,
    U16,
    U16Norm,
    I16Norm,
    U32,
}
impl Format {
    fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "f16" => Self::F16,
            "f32" => Self::F32,
            "u8" => Self::U8,
            "u8norm" => Self::U8Norm,
            "i8norm" => Self::I8Norm,
            "u16" => Self::U16,
            "u16norm" => Self::U16Norm,
            "i16norm" => Self::I16Norm,
            "u32" => Self::U32,
            _ => return None,
        })
    }
    fn size(self) -> usize {
        match self {
            Self::F16 | Self::U16 | Self::U16Norm | Self::I16Norm => 2,
            Self::F32 | Self::U32 => 4,
            _ => 1,
        }
    }
}
fn read(p: &[u8], f: Format) -> f64 {
    match f {
        Format::F16 => half_to_float(u16::from_ne_bytes(p[..2].try_into().unwrap())) as f64,
        Format::F32 => f32::from_ne_bytes(p[..4].try_into().unwrap()) as f64,
        Format::U8 => p[0] as f64,
        Format::U8Norm => p[0] as f64 / 255.0,
        Format::I8Norm => (p[0] as i8 as f64 / 127.0).max(-1.0),
        Format::U16 => u16::from_ne_bytes(p[..2].try_into().unwrap()) as f64,
        Format::U16Norm => u16::from_ne_bytes(p[..2].try_into().unwrap()) as f64 / 65535.0,
        Format::I16Norm => {
            (i16::from_ne_bytes(p[..2].try_into().unwrap()) as f64 / 32767.0).max(-1.0)
        }
        Format::U32 => u32::from_ne_bytes(p[..4].try_into().unwrap()) as f64,
    }
}
fn write(p: &mut [u8], f: Format, v: f64) {
    match f {
        Format::F16 => p[..2].copy_from_slice(&float_to_half(v as f32).to_ne_bytes()),
        Format::F32 => p[..4].copy_from_slice(&(v as f32).to_ne_bytes()),
        Format::U8 => p[0] = v as u32 as u8,
        Format::U8Norm => p[0] = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        Format::I8Norm => {
            p[0] = (v.clamp(-1.0, 1.0) * 127.0 + if v >= 0.0 { 0.5 } else { -0.5 }) as i8 as u8
        }
        Format::U16 => p[..2].copy_from_slice(&(v as u32 as u16).to_ne_bytes()),
        Format::U16Norm => {
            p[..2].copy_from_slice(&((v.clamp(0.0, 1.0) * 65535.0 + 0.5) as u16).to_ne_bytes())
        }
        Format::I16Norm => p[..2].copy_from_slice(
            &((v.clamp(-1.0, 1.0) * 32767.0 + if v >= 0.0 { 0.5 } else { -0.5 }) as i16)
                .to_ne_bytes(),
        ),
        Format::U32 => p[..4].copy_from_slice(&(v as u32).to_ne_bytes()),
    }
}
fn convert(s: &mut LuaState) -> i32 {
    let dst_len = s.check_buffer(1).len();
    let dst_off = s.check_integer(2);
    let dst_format_name = s.check_string(3);
    let dst_format = Format::parse(&dst_format_name)
        .unwrap_or_else(|| s.error(format!("unknown buffer format '{dst_format_name}'")));
    let src = s.check_buffer(4).to_vec();
    let src_off = s.check_integer(5);
    let src_format_name = s.check_string(6);
    let src_format = Format::parse(&src_format_name)
        .unwrap_or_else(|| s.error(format!("unknown buffer format '{src_format_name}'")));
    let count = s.check_integer(7);
    let components = s.opt_integer(8, 1);
    let ds = dst_format.size() as i32;
    let ss = src_format.size() as i32;
    let dst_stride = s.opt_integer(9, components * ds);
    let src_stride = s.opt_integer(10, components * ss);
    if count < 0 {
        return s.error("count must be non-negative");
    }
    if components < 1 {
        return s.error("components must be at least 1");
    }
    if count == 0 {
        return 0;
    }
    if src_stride < 0 || dst_stride < 0 {
        return s.error("stride must be non-negative");
    }
    let src_span = components * ss;
    let dst_span = components * ds;
    if src_stride > 0 && src_stride < src_span {
        return s.error("srcStride must be >= components * element size");
    }
    if dst_stride > 0 && dst_stride < dst_span {
        return s.error("dstStride must be >= components * element size");
    }
    let src_end = src_off as i64 + (count - 1) as i64 * src_stride as i64 + src_span as i64;
    let dst_end = dst_off as i64 + (count - 1) as i64 * dst_stride as i64 + dst_span as i64;
    if src_off < 0 || src_end > src.len() as i64 || dst_off < 0 || dst_end > dst_len as i64 {
        return s.error("buffer access out of bounds");
    }
    let dst = s.check_buffer_mut(1);
    if src_format == dst_format && src_stride == src_span && dst_stride == dst_span {
        let bytes = count as usize * components as usize * ss as usize;
        dst[dst_off as usize..dst_off as usize + bytes]
            .copy_from_slice(&src[src_off as usize..src_off as usize + bytes]);
        return 0;
    }
    for i in 0..count as usize {
        for c in 0..components as usize {
            let so = src_off as usize + i * src_stride as usize + c * ss as usize;
            let d = dst_off as usize + i * dst_stride as usize + c * ds as usize;
            let value = read(&src[so..], src_format);
            write(&mut dst[d..], dst_format, value);
        }
    }
    0
}
pub fn luaopen_rive_buffer_ext(s: &mut LuaState) -> i32 {
    s.get_global("buffer");
    for (name, f) in [
        ("readf16", read_f16 as LuaFunction),
        ("writef16", write_f16),
        ("stridedcopy", strided_copy),
        ("convert", convert),
    ] {
        s.push_function_named(f, &format!("buffer.{name}"));
        s.set_field(-2, name);
    }
    s.pop(1);
    0
}
