use super::super::data_converter_string_remove_zeros::DataConverterStringRemoveZeros;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum StringTemplateSegmentKind {
    #[default]
    Literal,
    Name,
    Position,
    Next,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StringTemplateSegment {
    pub kind: StringTemplateSegmentKind,
    pub text: String,
    pub name: String,
    pub index: u32,
    pub spec: String,
}

impl StringTemplateSegment {
    pub fn is_placeholder(&self) -> bool {
        self.kind != StringTemplateSegmentKind::Literal
    }
}

fn name_start(c: u8) -> bool {
    c.is_ascii_alphabetic() || c == b'_'
}

fn starts_with(text: &[u8], index: usize, prefix: &[u8]) -> bool {
    text.get(index..)
        .is_some_and(|tail| tail.starts_with(prefix))
}

#[derive(Default)]
struct NumberSpec {
    left_align: bool,
    plus: bool,
    space: bool,
    zero_pad: bool,
    alternate: bool,
    thousands: bool,
    width: Option<usize>,
    precision: Option<usize>,
    conversion: u8,
}

fn read_two_digits(spec: &[u8], index: &mut usize, value: &mut Option<usize>) -> bool {
    let start = *index;
    let mut result = 0;
    while *index < spec.len() && spec[*index].is_ascii_digit() && *index - start < 2 {
        result = result * 10 + usize::from(spec[*index] - b'0');
        *index += 1;
    }
    if *index > start {
        *value = Some(result);
    }
    *index >= spec.len() || !spec[*index].is_ascii_digit()
}

fn integer_conversion(c: u8) -> bool {
    matches!(c, b'd' | b'i' | b'u' | b'x' | b'X' | b'o')
}

fn parse_number_spec(spec: &str) -> Option<NumberSpec> {
    let spec = spec.as_bytes();
    let mut out = NumberSpec::default();
    let mut index = usize::from(spec.first() == Some(&b'%'));
    while index < spec.len() {
        match spec[index] {
            b'-' => out.left_align = true,
            b'+' => out.plus = true,
            b' ' => out.space = true,
            b'0' => out.zero_pad = true,
            b'#' => out.alternate = true,
            b',' => out.thousands = true,
            _ => break,
        }
        index += 1;
    }
    if !read_two_digits(spec, &mut index, &mut out.width) {
        return None;
    }
    if spec.get(index) == Some(&b'.') {
        index += 1;
        out.precision = Some(0);
        if !read_two_digits(spec, &mut index, &mut out.precision) {
            return None;
        }
    }
    if index + 1 != spec.len() {
        return None;
    }
    let c = spec[index];
    if !integer_conversion(c) && !matches!(c, b'f' | b'F' | b'e' | b'E' | b'g' | b'G') {
        return None;
    }
    out.conversion = c;
    let decimal = matches!(c, b'd' | b'i' | b'u');
    if out.alternate && decimal {
        return None;
    }
    if out.thousands && (out.zero_pad || !(decimal || matches!(c, b'f' | b'F' | b'g' | b'G'))) {
        return None;
    }
    Some(out)
}

// The validated printf subset is implemented with Rust's correctly rounded
// floating-point formatting. This preserves the pure-Rust runtime boundary;
// the value is promoted to double just as at the upstream snprintf call.
fn scientific(value: f64, precision: usize) -> (String, i32) {
    let output = format!("{value:.precision$e}");
    let (mantissa, exponent) = output.split_once('e').expect("scientific format exponent");
    (
        mantissa.to_owned(),
        exponent
            .parse()
            .expect("scientific format integer exponent"),
    )
}

fn decimal_point(output: &mut String) {
    if !output.contains('.') {
        output.push('.');
    }
}

fn trim_fraction(output: &mut String) {
    if output.contains('.') {
        while output.ends_with('0') {
            output.pop();
        }
        if output.ends_with('.') {
            output.pop();
        }
    }
}

fn float_digits(value: f64, spec: &NumberSpec) -> String {
    let precision = spec.precision.unwrap_or(6);
    match spec.conversion {
        b'f' | b'F' => {
            let mut output = format!("{value:.precision$}");
            if spec.alternate {
                decimal_point(&mut output);
            }
            output
        }
        b'e' | b'E' => {
            let (mut mantissa, exponent) = scientific(value, precision);
            if spec.alternate {
                decimal_point(&mut mantissa);
            }
            let marker = spec.conversion as char;
            format!("{mantissa}{marker}{exponent:+03}")
        }
        _ => {
            let precision = precision.max(1);
            // printf chooses fixed/exponent notation using the exponent after
            // rounding to the requested number of significant digits.
            let (mut mantissa, exponent) = scientific(value, precision - 1);
            if exponent < -4 || exponent >= precision as i32 {
                if spec.alternate {
                    decimal_point(&mut mantissa);
                } else {
                    trim_fraction(&mut mantissa);
                }
                let marker = if spec.conversion == b'G' { 'E' } else { 'e' };
                format!("{mantissa}{marker}{exponent:+03}")
            } else {
                let places = (precision as i32 - exponent - 1) as usize;
                let mut output = format!("{value:.places$}");
                if spec.alternate {
                    decimal_point(&mut output);
                } else {
                    trim_fraction(&mut output);
                }
                output
            }
        }
    }
}

fn print_number(value: f32, spec: &NumberSpec) -> String {
    let integer = integer_conversion(spec.conversion);
    let mut prefix = String::new();
    let mut digits;
    if integer {
        let rounded = f64::from(value).round().clamp(-9.0e18, 9.0e18);
        let signed = matches!(spec.conversion, b'd' | b'i');
        let magnitude = if signed {
            let number = rounded as i64;
            if number < 0 {
                prefix.push('-');
            } else if spec.plus {
                prefix.push('+');
            } else if spec.space {
                prefix.push(' ');
            }
            number.unsigned_abs()
        } else {
            rounded.max(0.0) as u64
        };
        digits = match spec.conversion {
            b'x' => format!("{magnitude:x}"),
            b'X' => format!("{magnitude:X}"),
            b'o' => format!("{magnitude:o}"),
            _ => magnitude.to_string(),
        };
        if magnitude == 0 && spec.precision == Some(0) {
            digits.clear();
        }
        if let Some(precision) = spec.precision {
            if digits.len() < precision {
                digits = "0".repeat(precision - digits.len()) + &digits;
            }
        }
        if spec.alternate {
            match spec.conversion {
                b'x' if magnitude != 0 => prefix.push_str("0x"),
                b'X' if magnitude != 0 => prefix.push_str("0X"),
                b'o' if !digits.starts_with('0') => digits.insert(0, '0'),
                _ => {}
            }
        }
    } else {
        if value.is_sign_negative() {
            prefix.push('-');
        } else if spec.plus {
            prefix.push('+');
        } else if spec.space {
            prefix.push(' ');
        }
        digits = float_digits(f64::from(value).abs(), spec);
    }
    let width = if spec.thousands {
        0
    } else {
        spec.width.unwrap_or(0)
    };
    let padding = width.saturating_sub(prefix.len() + digits.len());
    if spec.left_align {
        prefix + &digits + &" ".repeat(padding)
    } else if spec.zero_pad && !(integer && spec.precision.is_some()) {
        prefix + &"0".repeat(padding) + &digits
    } else {
        " ".repeat(padding) + &prefix + &digits
    }
}

pub struct StringTemplateFormat;

impl StringTemplateFormat {
    pub fn is_valid_delimiter(delimiter: &str) -> bool {
        !delimiter.is_empty()
            && delimiter.len() <= 4
            && delimiter.bytes().all(|c| {
                matches!(c, b'!'..=b'/' | b';'..=b'@' | b'['..=b'`' | b'{'..=b'~')
                    && c != b'\\'
                    && c != b'_'
            })
    }

    pub fn parse(
        template_text: &str,
        open: &str,
        close: &str,
        segments: &mut Vec<StringTemplateSegment>,
    ) {
        segments.clear();
        if !Self::is_valid_delimiter(open) || !Self::is_valid_delimiter(close) {
            if !template_text.is_empty() {
                segments.push(StringTemplateSegment {
                    text: template_text.to_owned(),
                    ..Default::default()
                });
            }
            return;
        }
        let text = template_text.as_bytes();
        let mut literal = Vec::new();
        let mut next_index = 0u32;
        let mut close_missing_from = None;
        let mut index = 0;
        while index < text.len() {
            let c = text[index];
            if c == b'\\' {
                if starts_with(text, index + 1, open.as_bytes()) {
                    literal.extend_from_slice(open.as_bytes());
                    index += 1 + open.len();
                    continue;
                }
                if text.get(index + 1) == Some(&b'\\') {
                    literal.push(b'\\');
                    index += 2;
                    continue;
                }
            }
            if starts_with(text, index, open.as_bytes()) {
                let key_start = index + open.len();
                let mut key_end = key_start;
                let mut kind = StringTemplateSegmentKind::Next;
                let mut position = 0u32;
                if text.get(key_end).is_some_and(|c| name_start(*c)) {
                    kind = StringTemplateSegmentKind::Name;
                    while text
                        .get(key_end)
                        .is_some_and(|c| name_start(*c) || c.is_ascii_digit())
                    {
                        key_end += 1;
                    }
                } else if text.get(key_end).is_some_and(u8::is_ascii_digit) {
                    kind = StringTemplateSegmentKind::Position;
                    while text.get(key_end).is_some_and(u8::is_ascii_digit) {
                        if position < 100_000_000 {
                            position = position * 10 + u32::from(text[key_end] - b'0');
                        }
                        key_end += 1;
                    }
                }
                let mut spec_start = None;
                let mut close_start = None;
                if starts_with(text, key_end, close.as_bytes()) {
                    close_start = Some(key_end);
                } else if text.get(key_end) == Some(&b':') {
                    let start = key_end + 1;
                    spec_start = Some(start);
                    if close_missing_from.is_none_or(|missing| start < missing) {
                        close_start = template_text[start..]
                            .find(close)
                            .map(|offset| start + offset);
                        if close_start.is_none() {
                            close_missing_from = Some(start);
                        }
                    }
                }
                if let Some(close_start) = close_start {
                    if !literal.is_empty() {
                        segments.push(StringTemplateSegment {
                            text: String::from_utf8(std::mem::take(&mut literal))
                                .expect("literal preserves input UTF-8"),
                            ..Default::default()
                        });
                    }
                    let end = close_start + close.len();
                    let mut placeholder = StringTemplateSegment {
                        kind,
                        text: template_text[index..end].to_owned(),
                        ..Default::default()
                    };
                    match kind {
                        StringTemplateSegmentKind::Name => {
                            placeholder.name = template_text[key_start..key_end].to_owned()
                        }
                        StringTemplateSegmentKind::Position => placeholder.index = position,
                        _ => {
                            placeholder.index = next_index;
                            next_index = next_index.wrapping_add(1);
                        }
                    }
                    if let Some(start) = spec_start {
                        placeholder.spec = template_text[start..close_start].to_owned();
                    }
                    segments.push(placeholder);
                    index = end;
                    continue;
                }
            }
            literal.push(c);
            index += 1;
        }
        if !literal.is_empty() {
            segments.push(StringTemplateSegment {
                text: String::from_utf8(literal).expect("literal preserves input UTF-8"),
                ..Default::default()
            });
        }
    }

    pub fn is_valid_number_spec(spec: &str) -> bool {
        parse_number_spec(spec).is_some()
    }

    pub fn format_number(value: f32, spec: &str) -> String {
        if !value.is_finite() {
            return if value.is_nan() {
                "nan"
            } else if value < 0.0 {
                "-inf"
            } else {
                "inf"
            }
            .to_owned();
        }
        let Some(number_spec) = parse_number_spec(spec) else {
            let output =
                DataConverterStringRemoveZeros::remove_zeros(format!("{:.6}", f64::from(value)));
            return if output == "-0" {
                "0".to_owned()
            } else {
                output
            };
        };
        let mut output = print_number(value, &number_spec);
        if number_spec.thousands {
            output = Self::format_thousands(&output);
            let width = number_spec.width.unwrap_or(0);
            if output.len() < width {
                let padding = " ".repeat(width - output.len());
                output = if number_spec.left_align {
                    output + &padding
                } else {
                    padding + &output
                };
            }
        }
        output
    }

    pub fn format_thousands(number: &str) -> String {
        let (integer, fraction) = number
            .find('.')
            .map_or((number, ""), |dot| (&number[..dot], &number[dot..]));
        let mut integer = integer.to_owned();
        let mut position = integer.len() as isize - 3;
        while position > 0 && integer.as_bytes()[position as usize - 1].is_ascii_digit() {
            integer.insert(position as usize, ',');
            position -= 3;
        }
        integer + fraction
    }
}
