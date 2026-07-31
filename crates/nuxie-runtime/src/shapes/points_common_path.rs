pub(crate) fn is_clockwise(path_flags: u64) -> bool {
    // `ShapePathFlags::isCounterClockwise` is bit 1 in the generated flags.
    path_flags & (1 << 1) == 0
}

#[cfg(test)]
mod tests {
    use super::is_clockwise;

    #[test]
    fn authored_direction_uses_the_generated_counter_clockwise_bit() {
        assert!(is_clockwise(0));
        assert!(!is_clockwise(1 << 1));
        assert!(is_clockwise(1));
    }
}
