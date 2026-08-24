// Direct safe-Rust ports of the complete pinned
// `tests/unit_tests/runtime/span_test.cpp` denominator. Rust slices are the
// intentional language-native owner for upstream `rive::Span`.

#[test]
fn span_basics_direct_port() {
    let array = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut span: &[i32] = &[];
    assert!(span.is_empty());
    assert_eq!(span.len(), 0);
    assert_eq!(std::mem::size_of_val(span), 0);
    assert_eq!(span.as_ptr_range().start, span.as_ptr_range().end);

    span = &array[..4];
    assert!(!span.is_empty());
    assert_eq!(span.as_ptr(), array.as_ptr());
    assert_eq!(span.len(), 4);
    assert_eq!(std::mem::size_of_val(span), 4 * std::mem::size_of::<i32>());
    assert_eq!(span.iter().count(), 4);
    assert_eq!(span.iter().copied().sum::<i32>(), 0 + 1 + 2 + 3);

    let mut subset = &span[1..3];
    assert!(!subset.is_empty());
    assert_eq!(subset.as_ptr(), array[1..].as_ptr());
    assert_eq!(subset.len(), 2);

    subset = &subset[1..1];
    assert!(subset.is_empty());
    assert_eq!(subset.len(), 0);
}

fn accepts_mut_span(_: &mut [i32]) {}
fn accepts_const_span(_: &[i32]) {}

#[test]
fn span_const_and_containers_direct_port() {
    let const_array = [1, 2, 3, 4];
    accepts_const_span(&const_array);

    let mut array = [1, 2, 3, 4];
    accepts_mut_span(&mut array);
    accepts_const_span(&array);

    let mut values = Vec::<i32>::new();
    accepts_mut_span(&mut values);
    accepts_const_span(&values);
}

#[test]
fn can_iterate_span_direct_port() {
    let array = [2, 4, 8, 16];
    let mut expected = 2;
    for value in array {
        assert_eq!(value, expected);
        expected *= 2;
    }
}
