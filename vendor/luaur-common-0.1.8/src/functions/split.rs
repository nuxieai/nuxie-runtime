extern crate alloc;

use alloc::vec::Vec;

/// Generated skeleton item.
/// Node: `cxx:Function:Luau.Common:Common/src/StringUtils.cpp:100:split`
/// Source: `Common/src/StringUtils.cpp`
pub fn split(mut s: &str, delimiter: char) -> Vec<&str> {
    let mut result = Vec::new();

    while !s.is_empty() {
        if let Some(index) = s.find(delimiter) {
            result.push(&s[..index]);
            s = &s[index + 1..];
        } else {
            result.push(s);
            break;
        }
    }

    result
}
