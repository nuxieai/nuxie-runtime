//! Number to string (C++ `luai_num2str`) and string to number (C++
//! `luaO_str2d`).
//!
//! `num2str` prints the shortest digits that round-trip (C++ uses
//! Schubfach; Rust's `{:e}` formatting produces the same shortest, closest
//! digits) in Luau's layout: fixed notation when the decimal point falls
//! within `-5 ..= 21` digits of the first digit, otherwise `d.ddde+XX`.

/// C++ `luai_num2str`.
pub(crate) fn num2str(n: f64) -> String {
    if n.is_nan() {
        return "nan".to_owned();
    }
    if n.is_infinite() {
        return if n < 0.0 {
            "-inf".to_owned()
        } else {
            "inf".to_owned()
        };
    }
    let mut out = String::new();
    if n.is_sign_negative() {
        out.push('-');
    }
    if n == 0.0 {
        out.push('0');
        return out;
    }
    // `{:e}` of the magnitude: "d[.ddd]e[-]x", shortest round-trip digits.
    let sci = format!("{:e}", n.abs());
    let (mantissa, exponent) = sci.split_once('e').unwrap_or((sci.as_str(), "0"));
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let digits = digits.trim_end_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let declen = i32::try_from(digits.len()).unwrap_or(17);
    // Position of the decimal point relative to the first digit.
    let dot = exponent.saturating_add(1);

    if (-5..=21).contains(&dot) {
        if dot <= 0 {
            out.push_str("0.");
            for _ in 0..dot.unsigned_abs() {
                out.push('0');
            }
            out.push_str(digits);
        } else if dot >= declen {
            out.push_str(digits);
            for _ in 0..dot.saturating_sub(declen) {
                out.push('0');
            }
        } else {
            let split = usize::try_from(dot).unwrap_or(0);
            let (int, frac) = digits.split_at_checked(split).unwrap_or((digits, ""));
            out.push_str(int);
            out.push('.');
            out.push_str(frac);
        }
    } else {
        let mut chars = digits.chars();
        if let Some(first) = chars.next() {
            out.push(first);
        }
        let rest = chars.as_str();
        if !rest.is_empty() {
            out.push('.');
            out.push_str(rest);
        }
        let e = dot.saturating_sub(1);
        out.push('e');
        out.push(if e < 0 { '-' } else { '+' });
        let e = e.unsigned_abs();
        if e >= 100 {
            out.push_str(&e.to_string());
        } else {
            out.push_str(&format!("{e:02}"));
        }
    }
    out
}

#[inline]
fn is_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// C++ `luaO_str2d`: `strtod` semantics (leading space, sign, decimal or
/// hexadecimal, `inf`/`nan`), then only trailing space. Like C++, the
/// string ends at its first NUL.
pub(crate) fn str2d(bytes: &[u8]) -> Option<f64> {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let s = bytes.get(..end)?;
    let mut i = s.iter().position(|&c| !is_space(c)).unwrap_or(s.len());
    let start = i;
    let negative = match s.get(i) {
        Some(b'-') => {
            i = i.saturating_add(1);
            true
        }
        Some(b'+') => {
            i = i.saturating_add(1);
            false
        }
        _ => false,
    };
    let rest = s.get(i..)?;
    let (value, used) = if let Some((v, n)) = parse_hex(rest) {
        (v, n)
    } else if let Some((v, n)) = parse_special(rest) {
        (v, n)
    } else {
        parse_decimal(s, start, i)?
    };
    let after = i.saturating_add(used);
    if s.get(after..)?.iter().all(|&c| is_space(c)) {
        Some(if negative { -value } else { value })
    } else {
        None
    }
}

fn parse_special(s: &[u8]) -> Option<(f64, usize)> {
    let lower: Vec<u8> = s.iter().take(8).map(u8::to_ascii_lowercase).collect();
    if lower.starts_with(b"infinity") {
        return Some((f64::INFINITY, 8));
    }
    if lower.starts_with(b"inf") {
        return Some((f64::INFINITY, 3));
    }
    if lower.starts_with(b"nan") {
        // `nan(chars)` is accepted by strtod.
        if s.get(3) == Some(&b'(') {
            let close = s
                .iter()
                .skip(4)
                .position(|&c| !(c.is_ascii_alphanumeric() || c == b'_'));
            if let Some(len) = close {
                let at = len.saturating_add(4);
                if s.get(at) == Some(&b')') {
                    return Some((f64::NAN, at.saturating_add(1)));
                }
            }
        }
        return Some((f64::NAN, 3));
    }
    None
}

/// A decimal number starting at `s[digits_at]` (after the sign); returns
/// the value and how many bytes after the sign it used. `start` is where
/// the sign (if any) begins, so the slice handed to Rust's parser includes
/// it only for rounding-neutral reasons (it does not).
fn parse_decimal(s: &[u8], _start: usize, digits_at: usize) -> Option<(f64, usize)> {
    let t = s.get(digits_at..)?;
    let mut i = 0usize;
    let mut mantissa_digits = 0usize;
    while t.get(i).is_some_and(u8::is_ascii_digit) {
        i = i.saturating_add(1);
        mantissa_digits = mantissa_digits.saturating_add(1);
    }
    if t.get(i) == Some(&b'.') {
        i = i.saturating_add(1);
        while t.get(i).is_some_and(u8::is_ascii_digit) {
            i = i.saturating_add(1);
            mantissa_digits = mantissa_digits.saturating_add(1);
        }
    }
    if mantissa_digits == 0 {
        return None;
    }
    if matches!(t.get(i), Some(b'e' | b'E')) {
        let mut j = i.saturating_add(1);
        if matches!(t.get(j), Some(b'+' | b'-')) {
            j = j.saturating_add(1);
        }
        if t.get(j).is_some_and(u8::is_ascii_digit) {
            while t.get(j).is_some_and(u8::is_ascii_digit) {
                j = j.saturating_add(1);
            }
            i = j;
        }
    }
    let text = std::str::from_utf8(t.get(..i)?).ok()?;
    let value: f64 = text.parse().ok()?;
    Some((value, i))
}

fn hex_digit(c: u8) -> Option<u64> {
    char::from(c).to_digit(16).map(u64::from)
}

/// A hexadecimal number (`0x` prefix, optional fraction and binary
/// exponent), as strtod reads it.
fn parse_hex(s: &[u8]) -> Option<(f64, usize)> {
    if !(s.first() == Some(&b'0') && matches!(s.get(1), Some(b'x' | b'X'))) {
        return None;
    }
    let mut i = 2usize;
    let mut mantissa: u64 = 0;
    let mut exponent: i64 = 0;
    let mut any = false;
    let mut sticky = false;
    let mut push = |d: u64, mantissa: &mut u64, exponent: &mut i64, fraction: bool| {
        if *mantissa >> 60 == 0 {
            *mantissa = (*mantissa << 4) | d;
            if fraction {
                *exponent = exponent.saturating_sub(4);
            }
        } else {
            sticky |= d != 0;
            if !fraction {
                *exponent = exponent.saturating_add(4);
            }
        }
    };
    while let Some(d) = s.get(i).copied().and_then(hex_digit) {
        push(d, &mut mantissa, &mut exponent, false);
        any = true;
        i = i.saturating_add(1);
    }
    if s.get(i) == Some(&b'.') {
        let mut j = i.saturating_add(1);
        let mut frac_any = false;
        while let Some(d) = s.get(j).copied().and_then(hex_digit) {
            push(d, &mut mantissa, &mut exponent, true);
            frac_any = true;
            j = j.saturating_add(1);
        }
        if any || frac_any {
            any = true;
            i = j;
        }
    }
    if !any {
        // strtod reads "0" and stops at the 'x'; C++ then retries with
        // strtoul, which also stops there, so the trailing check fails.
        return None;
    }
    if matches!(s.get(i), Some(b'p' | b'P')) {
        let mut j = i.saturating_add(1);
        let negative = match s.get(j) {
            Some(b'-') => {
                j = j.saturating_add(1);
                true
            }
            Some(b'+') => {
                j = j.saturating_add(1);
                false
            }
            _ => false,
        };
        if s.get(j).is_some_and(u8::is_ascii_digit) {
            let mut e: i64 = 0;
            while let Some(d) = s.get(j).filter(|c| c.is_ascii_digit()) {
                e = e
                    .saturating_mul(10)
                    .saturating_add(i64::from(d.saturating_sub(b'0')));
                j = j.saturating_add(1);
            }
            exponent = if negative {
                exponent.saturating_sub(e)
            } else {
                exponent.saturating_add(e)
            };
            i = j;
        }
    }
    if sticky {
        mantissa |= 1;
    }
    let value =
        (mantissa as f64) * 2f64.powi(i32::try_from(exponent.clamp(-2200, 2200)).unwrap_or(0));
    Some((value, i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_like_luau() {
        let cases: &[(f64, &str)] = &[
            (0.0, "0"),
            (-0.0, "-0"),
            (1.0, "1"),
            (-1.5, "-1.5"),
            (100.0, "100"),
            (0.1, "0.1"),
            (1e-5, "0.00001"),
            (1e-6, "0.000001"),
            (1e-7, "1e-07"),
            (1.25e-7, "1.25e-07"),
            (1e20, "100000000000000000000"),
            (1e21, "1e+21"),
            (1e22, "1e+22"),
            (123456789012.0, "123456789012"),
            (1.5e300, "1.5e+300"),
            (f64::INFINITY, "inf"),
            (f64::NEG_INFINITY, "-inf"),
            (2f64.powi(53), "9007199254740992"),
            (0.30000000000000004, "0.30000000000000004"),
        ];
        for &(n, want) in cases {
            assert_eq!(num2str(n), want, "{n:?}");
        }
    }

    #[test]
    fn parses_like_strtod() {
        assert_eq!(str2d(b" 10 "), Some(10.0));
        assert_eq!(str2d(b"0x10"), Some(16.0));
        assert_eq!(str2d(b"-0x1p4"), Some(-16.0));
        assert_eq!(str2d(b"1e5"), Some(1e5));
        assert_eq!(str2d(b"1e"), None);
        assert_eq!(str2d(b".5"), Some(0.5));
        assert_eq!(str2d(b"5."), Some(5.0));
        assert_eq!(str2d(b""), None);
        assert_eq!(str2d(b"0x"), None);
        assert_eq!(str2d(b"inf"), Some(f64::INFINITY));
        assert!(str2d(b"nan").is_some_and(f64::is_nan));
        assert_eq!(str2d(b"1 2"), None);
        assert_eq!(str2d(b"abc"), None);
    }
}
