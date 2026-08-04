/// `Luau::detail::countTrailingZeroes` from `DenseHash2.h`.
///
/// The caller must supply a non-zero word.
pub fn count_trailing_zeroes(word: u64) -> usize {
    debug_assert_ne!(word, 0);
    word.trailing_zeros() as usize
}
