#[allow(non_snake_case)]
pub(crate) unsafe fn writestring(s: *const core::ffi::c_char, l: usize) {
    use std::io::Write;

    let buf = core::slice::from_raw_parts(s as *const u8, l);
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(buf);
    let _ = stdout.flush();
}
