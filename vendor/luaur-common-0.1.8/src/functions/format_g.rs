//! C `printf("%.*g", precision, value)` equivalent.
//!
//! `%g` prints `precision` significant digits, strips trailing zeros (and a
//! trailing `.`), and switches to `%e`-style exponential form when the decimal
//! exponent is `< -4` or `>= precision`. Rust's `{:.*}` is fixed-point and `{:e}`
//! is always exponential, so neither matches `%g` directly — this reproduces it.
//!
//! Used by bytecode/IR disassembly which must be byte-identical to the C++
//! reference output (`%.17g` for numbers, `%.9g` for vector components).

use alloc::format;
use alloc::string::String;

pub fn format_g(v: f64, precision: i32) -> String {
    let precision = precision.max(1);

    if v == 0.0 {
        return if v.is_sign_negative() {
            String::from("-0")
        } else {
            String::from("0")
        };
    }

    if v.is_nan() {
        return String::from("nan");
    }
    if v.is_infinite() {
        return String::from(if v < 0.0 { "-inf" } else { "inf" });
    }

    // `{:.*e}` yields `d.dddde SXX`; recover the decimal exponent.
    let sci = format!("{:.*e}", (precision - 1) as usize, v);
    let exp: i32 = sci[sci.find('e').map(|i| i + 1).unwrap_or(sci.len())..]
        .parse()
        .unwrap_or(0);

    if exp >= -4 && exp < precision {
        let frac = (precision - 1 - exp).max(0) as usize;
        let mut s = format!("{:.*}", frac, v);
        if s.contains('.') {
            while s.ends_with('0') {
                s.pop();
            }
            if s.ends_with('.') {
                s.pop();
            }
        }
        s
    } else {
        let (mant, exp_part) = sci.split_once('e').unwrap_or((sci.as_str(), "0"));
        let mut m = String::from(mant);
        if m.contains('.') {
            while m.ends_with('0') {
                m.pop();
            }
            if m.ends_with('.') {
                m.pop();
            }
        }
        let e: i32 = exp_part.parse().unwrap_or(0);
        format!("{}e{}{:02}", m, if e < 0 { "-" } else { "+" }, e.abs())
    }
}
