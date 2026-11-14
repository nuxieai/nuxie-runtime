use crate::luau::{Buffer, Error, Lua, Result, Table};

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

    /// [`Self::byte_size`] as a byte count.
    const fn width(self) -> usize {
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
                strided_copy(
                    Strided {
                        buffer: &destination,
                        offset: destination_offset,
                        stride: destination_stride,
                    },
                    Strided {
                        buffer: &source,
                        offset: source_offset,
                        stride: source_stride,
                    },
                    element_size,
                    count,
                )
            },
        )?,
    )?;
    buffer.set(
        "convert",
        lua.create_function(|_, arguments: ConvertArguments| convert(arguments))?,
    )?;

    Ok(())
}

/// One side of a strided operation, as the script passed it.
struct Strided<'a> {
    buffer: &'a Buffer,
    offset: i32,
    stride: i32,
}

/// `buffer.stridedcopy`, as the pinned C++ extension
/// (`src/lua/lua_buffer_ext.cpp`): the same argument checks and errors, then
/// `count` element copies, each reading what earlier ones wrote when source
/// and destination are one buffer. The work is bounded by the buffers: a
/// zero stride needs a zero element size (`stride >= elementSize`), and then
/// every copy is empty; otherwise the bounds check limits `count` to the
/// buffer length. Within one buffer the copies run on a local copy of it,
/// written back once ([`Target`]).
fn strided_copy(
    destination: Strided<'_>,
    source: Strided<'_>,
    element_size: i32,
    count: i32,
) -> Result<()> {
    if element_size < 0 {
        return Err(Error::runtime("elementSize must be non-negative"));
    }
    if count < 0 {
        return Err(Error::runtime("count must be non-negative"));
    }
    if count == 0 {
        return Ok(());
    }
    if source.stride < element_size || destination.stride < element_size {
        return Err(Error::runtime("stride must be >= elementSize"));
    }
    let source_span = strided_span(&source, i64::from(element_size), count)?;
    let destination_span = strided_span(&destination, i64::from(element_size), count)?;
    if element_size == 0 {
        // C++ runs `count` empty copies.
        return Ok(());
    }
    let element_size = to_index(element_size)?;
    let mut target = Target::new(source.buffer, destination.buffer);
    let positions = positions(source_span.start, source.stride)?
        .zip(positions(destination_span.start, destination.stride)?)
        .take(to_index(count)?);
    for (from, to) in positions {
        let element = bytes_at(target.readable(), from, element_size)?.to_vec();
        target.write(to, &element)?;
    }
    target.finish(destination_span)
}

/// Where an operation reads and writes while it runs.
enum Target<'a> {
    /// Source and destination are one buffer: a local copy of it, read and
    /// written in order (reads see earlier writes), written back once.
    Shared {
        bytes: Vec<u8>,
        buffer: &'a Buffer,
    },
    /// Two buffers: reads from a copy of the source, writes straight to the
    /// destination (nothing written can be read back).
    Apart {
        source: Vec<u8>,
        destination: &'a Buffer,
    },
}

impl<'a> Target<'a> {
    fn new(source: &Buffer, destination: &'a Buffer) -> Self {
        if source == destination {
            Self::Shared {
                bytes: destination.to_vec(),
                buffer: destination,
            }
        } else {
            Self::Apart {
                source: source.to_vec(),
                destination,
            }
        }
    }

    /// The bytes reads come from.
    fn readable(&self) -> &[u8] {
        match self {
            Self::Shared { bytes, .. } => bytes,
            Self::Apart { source, .. } => source,
        }
    }

    fn write(&mut self, at: usize, written: &[u8]) -> Result<()> {
        match self {
            Self::Shared { bytes, .. } => {
                bytes_at_mut(bytes, at, written.len())?.copy_from_slice(written);
            }
            Self::Apart { destination, .. } => {
                // In range: the bounds check covered every element.
                destination.write_bytes(at, written);
            }
        }
        Ok(())
    }

    /// Writes a shared buffer's local copy back over `range`, the bytes the
    /// operation may have changed.
    fn finish(self, range: std::ops::Range<usize>) -> Result<()> {
        let Self::Shared { bytes, buffer } = self else {
            return Ok(());
        };
        let start = range.start;
        let changed = bytes
            .get(range)
            .ok_or_else(|| Error::runtime(OUT_OF_BOUNDS))?;
        buffer.write_bytes(start, changed);
        Ok(())
    }
}

type ConvertArguments = (
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
);

/// Most scalar conversions `buffer.convert` repeats, with both strides zero,
/// while looking for the destination bytes to repeat (see [`convert`]).
const MAX_REPEATED_CONVERSIONS: usize = 1 << 22;

/// `buffer.convert`, as the pinned C++ extension: the same argument checks
/// and errors, then `count` elements of `components` scalars, each read and
/// written in order (reading what earlier ones wrote when source and
/// destination are one buffer). The C++ SIMD fast paths for packed f32/f16
/// are not reproduced; this follows the scalar path, as before.
///
/// The work is bounded by the buffers: a nonzero stride on either side
/// limits `count` to that buffer's length. With both strides zero, every
/// element converts the same scalars: apart, a repeat changes nothing; in
/// one buffer, a repeat can feed the next. Repeats stop when the destination
/// bytes return to a checkpoint (Brent's power-of-two checkpoints: a fixed
/// point or a cycle), and the rest of `count` is taken modulo the cycle, so
/// the result is the one `count` repeats give. A repeat that has not come
/// back within [`MAX_REPEATED_CONVERSIONS`] scalars is an error instead of a
/// hang (in one buffer only: between two, one conversion is the result).
/// See [`Target`] for where reads and writes go.
fn convert(
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
    ): ConvertArguments,
) -> Result<()> {
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
    let source_span = i64::from(components).saturating_mul(i64::from(source_element_size));
    let destination_span =
        i64::from(components).saturating_mul(i64::from(destination_element_size));
    let destination_stride = destination_stride.map_or(destination_span, i64::from);
    let source_stride = source_stride.map_or(source_span, i64::from);
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
    let source_range = checked_strided_range(
        i64::from(source_offset),
        source_stride,
        source_span,
        i64::from(count),
        source.len(),
    )?;
    let destination_range = checked_strided_range(
        i64::from(destination_offset),
        destination_stride,
        destination_span,
        i64::from(count),
        destination.len(),
    )?;

    let packed = source_stride == source_span && destination_stride == destination_span;
    if source_format == destination_format && packed {
        // One copy of the whole run (both ranges are `count * span` long).
        let bytes = source.to_vec();
        let run = bytes
            .get(source_range)
            .ok_or_else(|| Error::runtime(OUT_OF_BOUNDS))?;
        destination.write_bytes(destination_range.start, run);
        return Ok(());
    }

    let elements = Elements {
        source_format,
        destination_format,
        components: to_index(components)?,
        source_stride: usize::try_from(source_stride).map_err(|_| Error::runtime(OUT_OF_BOUNDS))?,
        destination_stride: usize::try_from(destination_stride)
            .map_err(|_| Error::runtime(OUT_OF_BOUNDS))?,
    };
    let count = to_index(count)?;
    let mut target = Target::new(&source, &destination);
    if elements.source_stride == 0 && elements.destination_stride == 0 {
        let element = Element {
            source: source_range.start,
            destination: destination_range.start,
        };
        match &mut target {
            // Every repeat reads the same source and writes the same bytes.
            Target::Apart { .. } => elements.convert(element, &mut target)?,
            Target::Shared { bytes, .. } => {
                repeat_until_settled(&elements, element, count, bytes, &destination_range)?;
            }
        }
    } else {
        let starts = positions(source_range.start, elements.source_stride)?
            .zip(positions(destination_range.start, elements.destination_stride)?)
            .take(count);
        for (from, to) in starts {
            let element = Element {
                source: from,
                destination: to,
            };
            elements.convert(element, &mut target)?;
        }
    }
    target.finish(destination_range)
}

/// What every element of one `buffer.convert` call shares.
struct Elements {
    source_format: BufferFormat,
    destination_format: BufferFormat,
    components: usize,
    source_stride: usize,
    destination_stride: usize,
}

/// Where one element starts on each side.
#[derive(Clone, Copy)]
struct Element {
    source: usize,
    destination: usize,
}

impl Elements {
    /// Converts one element's scalars in order.
    fn convert(&self, element: Element, target: &mut Target<'_>) -> Result<()> {
        for (from, to) in self.scalars(element)? {
            let value = read_element(target.readable(), from, self.source_format)?;
            encode(self.destination_format, value, |bytes| target.write(to, bytes))?;
        }
        Ok(())
    }

    /// [`Self::convert`] within one buffer's local copy.
    fn convert_in_place(&self, element: Element, bytes: &mut [u8]) -> Result<()> {
        for (from, to) in self.scalars(element)? {
            let value = read_element(bytes, from, self.source_format)?;
            write_element(bytes, to, self.destination_format, value)?;
        }
        Ok(())
    }

    /// Where each scalar of `element` is read and written.
    fn scalars(&self, element: Element) -> Result<impl Iterator<Item = (usize, usize)>> {
        Ok(positions(element.source, self.source_format.width())?
            .zip(positions(element.destination, self.destination_format.width())?)
            .take(self.components))
    }
}

/// `count` conversions of the same element (both strides zero) within one
/// buffer's local copy; see [`convert`].
fn repeat_until_settled(
    elements: &Elements,
    element: Element,
    count: usize,
    target: &mut [u8],
    destination: &std::ops::Range<usize>,
) -> Result<()> {
    let region = |target: &[u8]| -> Result<Vec<u8>> {
        target
            .get(destination.clone())
            .map(<[u8]>::to_vec)
            .ok_or_else(|| Error::runtime(OUT_OF_BOUNDS))
    };
    // At least two repeats, whatever the element's size: the first is the
    // conversion itself and the second shows a fixed point.
    let budget = MAX_REPEATED_CONVERSIONS
        .checked_div(elements.components)
        .unwrap_or(0)
        .max(2);
    let mut checkpoint = region(target)?;
    let mut since_checkpoint = 0usize;
    let mut power = 1usize;
    let mut remaining = count;
    for _ in 0..budget {
        if remaining == 0 {
            return Ok(());
        }
        elements.convert_in_place(element, target)?;
        remaining = remaining.saturating_sub(1);
        since_checkpoint = since_checkpoint.saturating_add(1);
        if target.get(destination.clone()) == Some(checkpoint.as_slice()) {
            // The bytes repeat every `since_checkpoint` conversions from
            // here on: only the remainder changes the result.
            for _ in 0..remaining.checked_rem(since_checkpoint).unwrap_or(0) {
                elements.convert_in_place(element, target)?;
            }
            return Ok(());
        }
        if since_checkpoint == power {
            checkpoint = region(target)?;
            since_checkpoint = 0;
            power = power.saturating_mul(2);
        }
    }
    if remaining == 0 {
        return Ok(());
    }
    Err(Error::runtime(format!(
        "buffer.convert with zero strides did not settle within {MAX_REPEATED_CONVERSIONS} conversions"
    )))
}

/// `offset`, `offset + stride`, ...: the element starts of one side.
fn positions(offset: usize, stride: impl TryInto<usize>) -> Result<impl Iterator<Item = usize>> {
    let stride = stride
        .try_into()
        .map_err(|_| Error::runtime(OUT_OF_BOUNDS))?;
    Ok(std::iter::successors(Some(offset), move |at: &usize| {
        at.checked_add(stride)
    }))
}

/// The byte range one side of a strided copy covers, after the bounds check.
fn strided_span(side: &Strided<'_>, element_size: i64, count: i32) -> Result<std::ops::Range<usize>> {
    checked_strided_range(
        i64::from(side.offset),
        i64::from(side.stride),
        element_size,
        i64::from(count),
        side.buffer.len(),
    )
}

fn to_index(value: i32) -> Result<usize> {
    usize::try_from(value).map_err(|_| Error::runtime(OUT_OF_BOUNDS))
}

fn bytes_at(bytes: &[u8], at: usize, len: usize) -> Result<&[u8]> {
    bytes
        .get(at..)
        .and_then(|rest| rest.get(..len))
        .ok_or_else(|| Error::runtime(OUT_OF_BOUNDS))
}

fn bytes_at_mut(bytes: &mut [u8], at: usize, len: usize) -> Result<&mut [u8]> {
    bytes
        .get_mut(at..)
        .and_then(|rest| rest.get_mut(..len))
        .ok_or_else(|| Error::runtime(OUT_OF_BOUNDS))
}

fn array_at<const N: usize>(bytes: &[u8], at: usize) -> Result<[u8; N]> {
    bytes
        .get(at..)
        .and_then(<[u8]>::first_chunk::<N>)
        .copied()
        .ok_or_else(|| Error::runtime(OUT_OF_BOUNDS))
}

fn read_element(bytes: &[u8], at: usize, format: BufferFormat) -> Result<f64> {
    Ok(match format {
        BufferFormat::F16 => f64::from(half_to_float(u16::from_ne_bytes(array_at(bytes, at)?))),
        BufferFormat::F32 => f64::from(f32::from_ne_bytes(array_at(bytes, at)?)),
        BufferFormat::U8 => f64::from(u8::from_ne_bytes(array_at(bytes, at)?)),
        BufferFormat::U8Norm => f64::from(u8::from_ne_bytes(array_at(bytes, at)?)) / 255.0,
        BufferFormat::I8Norm => {
            (f64::from(i8::from_ne_bytes(array_at(bytes, at)?)) / 127.0).max(-1.0)
        }
        BufferFormat::U16 => f64::from(u16::from_ne_bytes(array_at(bytes, at)?)),
        BufferFormat::U16Norm => f64::from(u16::from_ne_bytes(array_at(bytes, at)?)) / 65_535.0,
        BufferFormat::I16Norm => {
            (f64::from(i16::from_ne_bytes(array_at(bytes, at)?)) / 32_767.0).max(-1.0)
        }
        BufferFormat::U32 => f64::from(u32::from_ne_bytes(array_at(bytes, at)?)),
    })
}

fn write_element(target: &mut [u8], at: usize, format: BufferFormat, value: f64) -> Result<()> {
    encode(format, value, |bytes| {
        bytes_at_mut(target, at, bytes.len())?.copy_from_slice(bytes);
        Ok(())
    })
}

/// `value` in `format`'s bytes, handed to `put`.
fn encode(
    format: BufferFormat,
    value: f64,
    put: impl FnOnce(&[u8]) -> Result<()>,
) -> Result<()> {
    // Pinned C++ float-to-integer casts are undefined for NaN and values outside
    // the destination range. Follow the workspace policy in docs/PORTING.md
    // section 3.3: Rust casts deliberately saturate (and map NaN to zero).
    match format {
        BufferFormat::F16 => put(&float_to_half(value as f32).to_ne_bytes()),
        BufferFormat::F32 => put(&(value as f32).to_ne_bytes()),
        BufferFormat::U8 => put(&[(value as u32) as u8]),
        BufferFormat::U8Norm => put(&[(value.clamp(0.0, 1.0) * 255.0 + 0.5) as u8]),
        BufferFormat::I8Norm => {
            let value = value.clamp(-1.0, 1.0);
            let rounded = value * 127.0 + if value >= 0.0 { 0.5 } else { -0.5 };
            put(&(rounded as i8).to_ne_bytes())
        }
        BufferFormat::U16 => put(&((value as u32) as u16).to_ne_bytes()),
        BufferFormat::U16Norm => {
            put(&((value.clamp(0.0, 1.0) * 65_535.0 + 0.5) as u16).to_ne_bytes())
        }
        BufferFormat::I16Norm => {
            let value = value.clamp(-1.0, 1.0);
            let rounded = value * 32_767.0 + if value >= 0.0 { 0.5 } else { -0.5 };
            put(&(rounded as i16).to_ne_bytes())
        }
        BufferFormat::U32 => put(&(value as u32).to_ne_bytes()),
    }
}

/// The byte range `offset .. offset + (count - 1) * stride + element_size`,
/// or the C++ extension's out-of-bounds error when it leaves the buffer.
fn checked_strided_range(
    offset: i64,
    stride: i64,
    element_size: i64,
    count: i64,
    buffer_len: usize,
) -> Result<std::ops::Range<usize>> {
    let end = count
        .checked_sub(1)
        .and_then(|tail| tail.checked_mul(stride))
        .and_then(|tail| offset.checked_add(tail))
        .and_then(|last| last.checked_add(element_size))
        .ok_or_else(|| Error::runtime(OUT_OF_BOUNDS))?;
    if offset < 0 || end > i64::try_from(buffer_len).unwrap_or(i64::MAX) {
        return Err(Error::runtime(OUT_OF_BOUNDS));
    }
    let start = usize::try_from(offset).map_err(|_| Error::runtime(OUT_OF_BOUNDS))?;
    let end = usize::try_from(end).map_err(|_| Error::runtime(OUT_OF_BOUNDS))?;
    Ok(start..end)
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
    // The f32 exponent rebiased for f16 (127 - 15); in -112..=143.
    let mut exponent = (((bits >> 23) & 0xff) as i32).wrapping_sub(127 - 15);
    let mut mantissa = bits & 0x007f_ffff;

    if exponent <= 0 {
        if exponent < -10 {
            return sign;
        }
        mantissa |= 0x0080_0000;
        // Bits dropped from the 24-bit significand: 14..=24.
        let dropped = 1i32.wrapping_sub(exponent).wrapping_add(13) as u32;
        let half = 1u32.wrapping_shl(dropped.wrapping_sub(1));
        let round = mantissa & 1u32.wrapping_shl(dropped).wrapping_sub(1);
        mantissa = mantissa.wrapping_shr(dropped);
        if round > half || (round == half && mantissa & 1 != 0) {
            mantissa = mantissa.wrapping_add(1);
        }
        return sign | mantissa as u16;
    }

    if exponent == 0xff - 127 + 15 {
        // Infinity, or NaN with the payload's top bits.
        return sign | 0x7c00 | (mantissa >> 13) as u16;
    }

    let round = mantissa & 0x1fff;
    mantissa >>= 13;
    if round > 0x1000 || (round == 0x1000 && mantissa & 1 != 0) {
        mantissa = mantissa.wrapping_add(1);
        if mantissa >= 0x0400 {
            mantissa = 0;
            exponent = exponent.wrapping_add(1);
        }
    }
    if exponent >= 31 {
        return sign | 0x7c00;
    }
    sign | ((exponent as u16) << 10) | mantissa as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORMATS: [BufferFormat; 9] = [
        BufferFormat::F16,
        BufferFormat::F32,
        BufferFormat::U8,
        BufferFormat::U8Norm,
        BufferFormat::I8Norm,
        BufferFormat::U16,
        BufferFormat::U16Norm,
        BufferFormat::I16Norm,
        BufferFormat::U32,
    ];
    const NAMES: [&str; 9] = [
        "f16", "f32", "u8", "u8norm", "i8norm", "u16", "u16norm", "i16norm", "u32",
    ];

    /// `float_to_half` before lane E2a, as the oracle for its rewrite.
    #[allow(clippy::arithmetic_side_effects)]
    fn old_float_to_half(value: f32) -> u16 {
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

    /// The rewrite agrees with the old body on every 251st f32 bit pattern
    /// and on the boundary cases.
    #[test]
    fn float_to_half_is_unchanged() {
        let edges = [
            0.0f32, -0.0, 1.0, -1.0, 65504.0, 65520.0, 6.0e-8, 5.9e-8, 3.0e-8, 2.9e-8,
            6.1035156e-5, f32::INFINITY, f32::NEG_INFINITY, f32::NAN, f32::MIN_POSITIVE,
        ];
        for value in edges {
            assert_eq!(float_to_half(value), old_float_to_half(value), "{value}");
        }
        for bits in (0..=u32::MAX).step_by(251) {
            let value = f32::from_bits(bits);
            assert_eq!(float_to_half(value), old_float_to_half(value), "{bits:#x}");
        }
    }

    struct XorShift(u64);
    impl XorShift {
        fn next(&mut self, bound: usize) -> usize {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 % bound as u64) as usize
        }
    }

    /// The pinned C++ general path, element by element on plain memory
    /// (`one` when source and destination are the same buffer).
    #[allow(clippy::too_many_arguments)]
    fn reference_convert(
        destination: &mut Vec<u8>,
        source: Option<&[u8]>,
        (dst_offset, dst_format, dst_stride): (usize, BufferFormat, usize),
        (src_offset, src_format, src_stride): (usize, BufferFormat, usize),
        count: usize,
        components: usize,
    ) {
        for index in 0..count {
            for component in 0..components {
                let from = src_offset + index * src_stride + component * src_format.width();
                let to = dst_offset + index * dst_stride + component * dst_format.width();
                let value = read_element(source.unwrap_or(destination), from, src_format).unwrap();
                write_element(destination, to, dst_format, value).unwrap();
            }
        }
    }

    /// Random conversions with zero and nonzero strides, apart and in one
    /// buffer, at repeat counts up to 3,000: the result is the C++ loop's,
    /// including when the repeats are resolved through a detected cycle.
    #[test]
    fn convert_matches_the_element_loop() {
        let lua = Lua::new();
        let mut rng = XorShift(0xb0ff_e2a0_0000_0001);
        let mut runs = 0;
        for round in 0..6_000 {
            let len = 48;
            let (sf, df) = (rng.next(9), rng.next(9));
            let (src_format, dst_format) = (FORMATS[sf], FORMATS[df]);
            let components = 1 + rng.next(4);
            let aliased = round % 2 == 0;
            let zero = rng.next(3) == 0;
            let src_stride = if zero { 0 } else { components * src_format.width() + rng.next(3) };
            let dst_stride = if zero { 0 } else { components * dst_format.width() + rng.next(3) };
            let count = if zero { 1 + rng.next(3_000) } else { 1 + rng.next(4) };
            let src_offset = rng.next(8);
            let dst_offset = rng.next(8);
            let fits = |offset: usize, stride: usize, width: usize| {
                offset + (count - 1) * stride + components * width <= len
            };
            if !fits(src_offset, src_stride, src_format.width())
                || !fits(dst_offset, dst_stride, dst_format.width())
            {
                continue;
            }
            let initial: Vec<u8> = (0..len).map(|_| rng.next(256) as u8).collect();
            let other: Vec<u8> = (0..len).map(|_| rng.next(256) as u8).collect();

            let mut expected = initial.clone();
            let packed = src_stride == components * src_format.width()
                && dst_stride == components * dst_format.width();
            if sf == df && packed {
                // The fast path copies the run as one block (memmove), as
                // the Rust port always has; C++'s `memcpy` of an
                // overlapping run is undefined.
                let run = count * components * src_format.width();
                let block = expected[src_offset..src_offset + run].to_vec();
                let source = if aliased { &block } else { &other[src_offset..src_offset + run] };
                expected[dst_offset..dst_offset + run].copy_from_slice(source);
            } else {
            reference_convert(
                &mut expected,
                (!aliased).then_some(other.as_slice()),
                (dst_offset, dst_format, dst_stride),
                (src_offset, src_format, src_stride),
                count,
                components,
            );
            }

            let destination = lua.create_buffer(&initial).unwrap();
            let source = if aliased { destination.clone() } else { lua.create_buffer(&other).unwrap() };
            convert((
                destination.clone(),
                dst_offset as i32,
                NAMES[df].to_owned(),
                source,
                src_offset as i32,
                NAMES[sf].to_owned(),
                count as i32,
                Some(components as i32),
                Some(dst_stride as i32),
                Some(src_stride as i32),
            ))
            .unwrap();
            assert_eq!(
                destination.to_vec(),
                expected,
                "{} <- {} x{count} c{components} strides {dst_stride}/{src_stride} \
                 offsets {dst_offset}/{src_offset} aliased {aliased}",
                NAMES[df],
                NAMES[sf]
            );
            runs += 1;
        }
        assert!(runs > 3_000, "{runs}");
    }

    /// Strided copies, apart and in one buffer (live reads, overlapping
    /// elements), against the element loop.
    #[test]
    fn strided_copy_matches_the_element_loop() {
        let lua = Lua::new();
        let mut rng = XorShift(0x5717_e2a0_0000_0001);
        for round in 0..4_000 {
            let len = 64;
            let element = rng.next(6);
            let src_stride = element + rng.next(4);
            let dst_stride = element + rng.next(4);
            let count = 1 + rng.next(8);
            let (src_offset, dst_offset) = (rng.next(12), rng.next(12));
            let aliased = round % 2 == 0;
            if src_offset + (count - 1) * src_stride + element > len
                || dst_offset + (count - 1) * dst_stride + element > len
            {
                continue;
            }
            let initial: Vec<u8> = (0..len).map(|_| rng.next(256) as u8).collect();
            let other: Vec<u8> = (0..len).map(|_| rng.next(256) as u8).collect();
            let mut expected = initial.clone();
            for index in 0..count {
                let from = src_offset + index * src_stride;
                let to = dst_offset + index * dst_stride;
                let bytes = if aliased {
                    expected[from..from + element].to_vec()
                } else {
                    other[from..from + element].to_vec()
                };
                expected[to..to + element].copy_from_slice(&bytes);
            }
            let destination = lua.create_buffer(&initial).unwrap();
            let source = if aliased { destination.clone() } else { lua.create_buffer(&other).unwrap() };
            strided_copy(
                Strided { buffer: &destination, offset: dst_offset as i32, stride: dst_stride as i32 },
                Strided { buffer: &source, offset: src_offset as i32, stride: src_stride as i32 },
                element as i32,
                count as i32,
            )
            .unwrap();
            assert_eq!(destination.to_vec(), expected);
        }
    }

    /// An element larger than the repeat cap still converts once or twice:
    /// the cap bounds repeats, not the element.
    #[test]
    fn a_large_element_converts_whatever_the_repeat_cap() {
        let lua = Lua::new();
        let components = MAX_REPEATED_CONVERSIONS + 1;
        let buffer = lua.create_buffer(vec![7u8; components + 1]).unwrap();
        for count in [1, 2] {
            convert((
                buffer.clone(),
                1,
                "u8".to_owned(),
                buffer.clone(),
                0,
                "u8".to_owned(),
                count,
                Some(components as i32),
                Some(0),
                Some(0),
            ))
            .unwrap();
        }
    }

    /// A zero-stride conversion in one buffer that cycles: `count` repeats
    /// near `i32::MAX` resolve through the cycle, and match the loop at the
    /// same count modulo the period.
    #[test]
    fn a_cycling_repeat_resolves_modulo_its_period() {
        let lua = Lua::new();
        // Search a small cycling case (period > 1) with the element loop.
        let mut rng = XorShift(0xc1c1_e2a0_0000_0001);
        for _ in 0..20_000 {
            let (sf, df) = (rng.next(9), rng.next(9));
            let components = 1 + rng.next(4);
            let (so, dof) = (rng.next(6), rng.next(6));
            let initial: Vec<u8> = (0..32).map(|_| rng.next(256) as u8).collect();
            let step = |bytes: &mut Vec<u8>, n: usize| {
                reference_convert(bytes, None, (dof, FORMATS[df], 0), (so, FORMATS[sf], 0), n, components)
            };
            let (mut one, mut two) = (initial.clone(), initial.clone());
            step(&mut one, 400);
            step(&mut two, 401);
            if one == two {
                continue; // settled to a fixed point
            }
            // A cycle: `count = i32::MAX` against the loop at the same phase.
            let mut period = 1;
            let mut probe = one.clone();
            step(&mut probe, 1);
            while probe != one {
                step(&mut probe, 1);
                period += 1;
            }
            let big = i32::MAX as usize;
            let mut expected = initial.clone();
            step(&mut expected, 400 + (big - 400) % period);
            let buffer = lua.create_buffer(&initial).unwrap();
            convert((
                buffer.clone(),
                dof as i32,
                NAMES[df].to_owned(),
                buffer.clone(),
                so as i32,
                NAMES[sf].to_owned(),
                i32::MAX,
                Some(components as i32),
                Some(0),
                Some(0),
            ))
            .unwrap();
            assert_eq!(buffer.to_vec(), expected, "period {period}");
            return;
        }
        panic!("no cycling case found");
    }
}
