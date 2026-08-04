pub(crate) fn printableStringConstant(str: &[u8]) -> bool {
    for &b in str {
        if b < b' ' {
            return false;
        }
    }
    true
}
