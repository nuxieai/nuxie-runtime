use crate::functions::format_append::formatAppend;
use alloc::string::String;

pub fn escape(s: &str, escape_for_interp_string: bool) -> String {
    let mut r = String::with_capacity(s.len() + 50);

    for &c in s.as_bytes() {
        if c >= b' ' && c != b'\\' && c != b'\'' && c != b'\"' && c != b'`' && c != b'{' {
            r.push(c as char);
        } else {
            r.push('\\');

            if escape_for_interp_string && (c == b'`' || c == b'{') {
                r.push(c as char);
                continue;
            }

            match c {
                7 => r.push('a'),  // \a
                8 => r.push('b'),  // \b
                12 => r.push('f'), // \f
                10 => r.push('n'), // \n
                13 => r.push('r'), // \r
                9 => r.push('t'),  // \t
                11 => r.push('v'), // \v
                b'\'' => r.push('\''),
                b'\"' => r.push('\"'),
                b'\\' => r.push('\\'),
                // Upstream emits `%03u` (zero-padded, width 3) via `formatAppend`.
                _ => formatAppend(&mut r, format_args!("{:03}", c)),
            }
        }
    }

    r
}
