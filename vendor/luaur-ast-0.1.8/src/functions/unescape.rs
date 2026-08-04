#[allow(non_snake_case)]
pub(crate) fn unescape(ch: char) -> char {
    match ch {
        'a' => '\x07', // \a
        'b' => '\x08', // \b
        'f' => '\x0c', // \f
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        'v' => '\x0b', // \v
        _ => ch,
    }
}
