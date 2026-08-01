use luaur_rt::{Buffer, Error, Lua, Result, Table};

const OUT_OF_BOUNDS: &str = "buffer access out of bounds";

#[derive(Clone, Copy, PartialEq, Eq)]
enum BufferFormat {
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

impl BufferFormat {
    fn parse(name: &str) -> Result<Self> {
        match name {
            "f16" => Ok(Self::F16),
            "f32" => Ok(Self::F32),
            "u8" => Ok(Self::U8),
            "u8norm" => Ok(Self::U8Norm),
            "i8norm" => Ok(Self::I8Norm),
            "u16" => Ok(Self::U16),
            "u16norm" => Ok(Self::U16Norm),
            "i16norm" => Ok(Self::I16Norm),
            "u32" => Ok(Self::U32),
            _ => Err(Error::runtime(format!("unknown buffer format '{name}'"))),
        }
    }

    const fn byte_size(self) -> i32 {
        match self {
            Self::F16 | Self::U16 | Self::U16Norm | Self::I16Norm => 2,
            Self::F32 | Self::U32 => 4,
            Self::U8 | Self::U8Norm | Self::I8Norm => 1,
        }
    }
}

pub(super) fn install_buffer_extensions(lua: &Lua) -> Result<()> {
    let buffer: Table = lua.globals().get("buffer")?;

    buffer.set(
        "readf16",
        lua.create_function(|_, (buffer, offset): (Buffer, i32)| {
            let offset = checked_range(offset, buffer.len(), 2)?;
            let bits = u16::from_ne_bytes(buffer.read_bytes(offset));
            Ok(f64::from(half_to_float(bits)))
        })?,
    )?;
    buffer.set(
        "writef16",
        lua.create_function(|_, (buffer, offset, value): (Buffer, i32, f64)| {
            let offset = checked_range(offset, buffer.len(), 2)?;
            buffer.write_bytes(offset, &float_to_half(value as f32).to_ne_bytes());
            Ok(())
        })?,
    )?;
    buffer.set(
        "stridedcopy",
        lua.create_function(
            |_,
             (
                destination,
                destination_offset,
                destination_stride,
                source,
                source_offset,
                source_stride,
                element_size,
                count,
            ): (Buffer, i32, i32, Buffer, i32, i32, i32, i32)| {
                if element_size < 0 {
                    return Err(Error::runtime("elementSize must be non-negative"));
                }
                if count < 0 {
                    return Err(Error::runtime("count must be non-negative"));
                }
                if count == 0 {
                    return Ok(());
                }
                if source_stride < element_size || destination_stride < element_size {
                    return Err(Error::runtime("stride must be >= elementSize"));
                }

                let source_offset = checked_strided_range(
                    i64::from(source_offset),
                    i64::from(source_stride),
                    i64::from(element_size),
                    i64::from(count),
                    source.len(),
                )?;
                let destination_offset = checked_strided_range(
                    i64::from(destination_offset),
                    i64::from(destination_stride),
                    i64::from(element_size),
                    i64::from(count),
                    destination.len(),
                )?;
                let source_stride = source_stride as usize;
                let destination_stride = destination_stride as usize;
                let element_size = element_size as usize;
                let source_snapshot = (source != destination).then(|| source.to_vec());

                for index in 0..count as usize {
                    let source_start = source_offset + index * source_stride;
                    let destination_start = destination_offset + index * destination_stride;
                    if let Some(bytes) = source_snapshot.as_ref() {
                        destination.write_bytes(
                            destination_start,
                            &bytes[source_start..source_start + element_size],
                        );
                    } else {
                        // Preserve the C++ loop's live reads when both handles point
                        // at the same buffer.
                        let bytes = source.to_vec();
                        destination.write_bytes(
                            destination_start,
                            &bytes[source_start..source_start + element_size],
                        );
                    }
                }
                Ok(())
            },
        )?,
    )?;
    buffer.set(
        "convert",
        lua.create_function(
            |_,
             (
                destination,
                destination_offset,
                destination_format,
                source,
                source_offset,
                source_format,
                count,
                components,
                destination_stride,
                source_stride,
            ): (
                Buffer,
                i32,
                String,
                Buffer,
                i32,
                String,
                i32,
                Option<i32>,
                Option<i32>,
                Option<i32>,
            )| {
                let destination_format = BufferFormat::parse(&destination_format)?;
                let source_format = BufferFormat::parse(&source_format)?;
                let components = components.unwrap_or(1);

                if count < 0 {
                    return Err(Error::runtime("count must be non-negative"));
                }
                if components < 1 {
                    return Err(Error::runtime("components must be at least 1"));
                }
                if count == 0 {
                    return Ok(());
                }

                let source_element_size = source_format.byte_size();
                let destination_element_size = destination_format.byte_size();
                let source_span = i64::from(components) * i64::from(source_element_size);
                let destination_span = i64::from(components) * i64::from(destination_element_size);
                let destination_stride = destination_stride
                    .map(i64::from)
                    .unwrap_or(destination_span);
                let source_stride = source_stride.map(i64::from).unwrap_or(source_span);
                if source_stride < 0 || destination_stride < 0 {
                    return Err(Error::runtime("stride must be non-negative"));
                }

                if source_stride > 0 && source_stride < source_span {
                    return Err(Error::runtime(
                        "srcStride must be >= components * element size",
                    ));
                }
                if destination_stride > 0 && destination_stride < destination_span {
                    return Err(Error::runtime(
                        "dstStride must be >= components * element size",
                    ));
                }

                let source_offset = checked_strided_range(
                    i64::from(source_offset),
                    source_stride,
                    source_span,
                    i64::from(count),
                    source.len(),
                )?;
                let destination_offset = checked_strided_range(
                    i64::from(destination_offset),
                    destination_stride,
                    destination_span,
                    i64::from(count),
                    destination.len(),
                )?;
                let source_stride =
                    usize::try_from(source_stride).map_err(|_| Error::runtime(OUT_OF_BOUNDS))?;
                let destination_stride = usize::try_from(destination_stride)
                    .map_err(|_| Error::runtime(OUT_OF_BOUNDS))?;
                let source_span =
                    usize::try_from(source_span).map_err(|_| Error::runtime(OUT_OF_BOUNDS))?;
                let destination_span =
                    usize::try_from(destination_span).map_err(|_| Error::runtime(OUT_OF_BOUNDS))?;
                let source_element_size = source_element_size as usize;
                let destination_element_size = destination_element_size as usize;
                let components = components as usize;

                let packed = source_stride == source_span && destination_stride == destination_span;
                if source_format == destination_format && packed {
                    let byte_count = (count as usize)
                        .checked_mul(source_span)
                        .ok_or_else(|| Error::runtime(OUT_OF_BOUNDS))?;
                    let source_bytes = source.to_vec();
                    destination.write_bytes(
                        destination_offset,
                        &source_bytes[source_offset..source_offset + byte_count],
                    );
                    return Ok(());
                }

                let source_snapshot = (source != destination).then(|| source.to_vec());
                for index in 0..count as usize {
                    for component in 0..components {
                        let source_start =
                            source_offset + index * source_stride + component * source_element_size;
                        let value = if let Some(bytes) = source_snapshot.as_ref() {
                            read_element(
                                &bytes[source_start..source_start + source_element_size],
                                source_format,
                            )
                        } else {
                            // General conversion reads and writes scalars in order in
                            // the pinned C++ implementation, including for aliases.
                            let bytes = source.to_vec();
                            read_element(
                                &bytes[source_start..source_start + source_element_size],
                                source_format,
                            )
                        };
                        let destination_start = destination_offset
                            + index * destination_stride
                            + component * destination_element_size;
                        let encoded = write_element(destination_format, value);
                        destination.write_bytes(destination_start, encoded.as_slice());
                    }
                }
                Ok(())
            },
        )?,
    )?;

    Ok(())
}

fn read_element(bytes: &[u8], format: BufferFormat) -> f64 {
    match format {
        BufferFormat::F16 => f64::from(half_to_float(u16::from_ne_bytes(
            bytes.try_into().expect("f16 byte width"),
        ))),
        BufferFormat::F32 => f64::from(f32::from_ne_bytes(
            bytes.try_into().expect("f32 byte width"),
        )),
        BufferFormat::U8 => f64::from(bytes[0]),
        BufferFormat::U8Norm => f64::from(bytes[0]) / 255.0,
        BufferFormat::I8Norm => (f64::from(bytes[0] as i8) / 127.0).max(-1.0),
        BufferFormat::U16 => f64::from(u16::from_ne_bytes(
            bytes.try_into().expect("u16 byte width"),
        )),
        BufferFormat::U16Norm => {
            f64::from(u16::from_ne_bytes(
                bytes.try_into().expect("u16norm byte width"),
            )) / 65_535.0
        }
        BufferFormat::I16Norm => (f64::from(i16::from_ne_bytes(
            bytes.try_into().expect("i16norm byte width"),
        )) / 32_767.0)
            .max(-1.0),
        BufferFormat::U32 => f64::from(u32::from_ne_bytes(
            bytes.try_into().expect("u32 byte width"),
        )),
    }
}

struct EncodedElement {
    bytes: [u8; 4],
    len: usize,
}

impl EncodedElement {
    fn from_byte(byte: u8) -> Self {
        Self {
            bytes: [byte, 0, 0, 0],
            len: 1,
        }
    }

    fn from_two(bytes: [u8; 2]) -> Self {
        Self {
            bytes: [bytes[0], bytes[1], 0, 0],
            len: 2,
        }
    }

    fn from_four(bytes: [u8; 4]) -> Self {
        Self { bytes, len: 4 }
    }

    fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

fn write_element(format: BufferFormat, value: f64) -> EncodedElement {
    // Pinned C++ float-to-integer casts are undefined for NaN and values outside
    // the destination range. Follow the workspace policy in docs/PORTING.md
    // section 3.3: Rust casts deliberately saturate (and map NaN to zero).
    match format {
        BufferFormat::F16 => EncodedElement::from_two(float_to_half(value as f32).to_ne_bytes()),
        BufferFormat::F32 => EncodedElement::from_four((value as f32).to_ne_bytes()),
        BufferFormat::U8 => EncodedElement::from_byte((value as u32) as u8),
        BufferFormat::U8Norm => {
            EncodedElement::from_byte((value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8)
        }
        BufferFormat::I8Norm => {
            let value = value.clamp(-1.0, 1.0);
            EncodedElement::from_byte(
                (value * 127.0 + if value >= 0.0 { 0.5 } else { -0.5 }) as i8 as u8,
            )
        }
        BufferFormat::U16 => EncodedElement::from_two(((value as u32) as u16).to_ne_bytes()),
        BufferFormat::U16Norm => EncodedElement::from_two(
            ((value.clamp(0.0, 1.0) * 65_535.0 + 0.5) as u16).to_ne_bytes(),
        ),
        BufferFormat::I16Norm => {
            let value = value.clamp(-1.0, 1.0);
            let rounded = value * 32_767.0 + if value >= 0.0 { 0.5 } else { -0.5 };
            EncodedElement::from_two((rounded as i16).to_ne_bytes())
        }
        BufferFormat::U32 => EncodedElement::from_four((value as u32).to_ne_bytes()),
    }
}

fn checked_strided_range(
    offset: i64,
    stride: i64,
    element_size: i64,
    count: i64,
    buffer_len: usize,
) -> Result<usize> {
    let end = (count - 1)
        .checked_mul(stride)
        .and_then(|tail| offset.checked_add(tail))
        .and_then(|last| last.checked_add(element_size))
        .ok_or_else(|| Error::runtime(OUT_OF_BOUNDS))?;
    if offset < 0 || end > buffer_len as i64 {
        return Err(Error::runtime(OUT_OF_BOUNDS));
    }
    usize::try_from(offset).map_err(|_| Error::runtime(OUT_OF_BOUNDS))
}

fn checked_range(offset: i32, buffer_len: usize, access_size: usize) -> Result<usize> {
    let offset = usize::try_from(offset).map_err(|_| Error::runtime(OUT_OF_BOUNDS))?;
    if !matches!(
        offset.checked_add(access_size),
        Some(end) if end <= buffer_len
    ) {
        return Err(Error::runtime(OUT_OF_BOUNDS));
    }
    Ok(offset)
}

fn half_to_float(half: u16) -> f32 {
    let sign = u32::from(half >> 15) << 31;
    let mut exponent = u32::from((half >> 10) & 0x1f);
    let mut mantissa = u32::from(half & 0x03ff);

    let bits = if exponent == 0 {
        if mantissa == 0 {
            sign
        } else {
            exponent = 1;
            while mantissa & 0x0400 == 0 {
                mantissa <<= 1;
                exponent = exponent.wrapping_sub(1);
            }
            mantissa &= 0x03ff;
            sign | (exponent.wrapping_add(127 - 15) << 23) | (mantissa << 13)
        }
    } else if exponent == 31 {
        sign | 0x7f80_0000 | (mantissa << 13)
    } else {
        sign | (exponent.wrapping_add(127 - 15) << 23) | (mantissa << 13)
    };

    f32::from_bits(bits)
}

fn float_to_half(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let mut exponent = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    let mut mantissa = bits & 0x007f_ffff;

    if exponent <= 0 {
        if exponent < -10 {
            return sign;
        }
        mantissa |= 0x0080_0000;
        let shift = (1 - exponent) as u32;
        let round = mantissa & ((1 << (shift + 13)) - 1);
        mantissa >>= shift + 13;
        if round > 1 << (shift + 12) || (round == 1 << (shift + 12) && mantissa & 1 != 0) {
            mantissa += 1;
        }
        return sign | mantissa as u16;
    }

    if exponent == 0xff - 127 + 15 {
        if mantissa == 0 {
            return sign | 0x7c00;
        }
        return sign | 0x7c00 | (mantissa >> 13) as u16;
    }

    let round = mantissa & 0x1fff;
    mantissa >>= 13;
    if round > 0x1000 || (round == 0x1000 && mantissa & 1 != 0) {
        mantissa += 1;
        if mantissa >= 0x0400 {
            mantissa = 0;
            exponent += 1;
        }
    }
    if exponent >= 31 {
        return sign | 0x7c00;
    }
    sign | ((exponent as u16) << 10) | mantissa as u16
}
