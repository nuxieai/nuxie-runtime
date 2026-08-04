pub fn three_way<T: PartialOrd>(a: &T, b: &T) -> i32 {
    i32::from(a > b) - i32::from(a < b)
}
