pub fn posrelat(pos: i32, len: usize) -> i32 {
    // relative string position: negative means back from end
    let pos = if pos < 0 { pos + len as i32 + 1 } else { pos };
    if pos >= 0 {
        pos
    } else {
        0
    }
}
