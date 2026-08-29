//! All pinned span cases, using the translated production Span.

use nuxie_runtime::source::span::{Span, make_span};

#[test]
fn span_basics_direct_port() {
    let array = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    let mut span = Span::<i32>::default();
    assert!(span.empty());
    assert_eq!(span.size(), 0);
    assert_eq!(span.size_bytes(), 0);
    assert_eq!(
        span.as_slice().as_ptr_range().start,
        span.as_slice().as_ptr_range().end
    );

    span = Span::new(&array[..4]);
    assert!(!span.empty());
    assert_eq!(span.data(), array.as_ptr());
    assert_eq!(span.size(), 4);
    assert_eq!(span.size_bytes(), 4 * std::mem::size_of::<i32>());
    assert_eq!(
        span.data().wrapping_add(span.size()),
        span.as_slice().as_ptr_range().end
    );
    assert_eq!(span.iter().count(), 4);
    assert_eq!(span.iter().copied().sum::<i32>(), 0 + 1 + 2 + 3);

    let mut subset = span.subset(1, 2);
    assert!(!subset.empty());
    assert_eq!(subset.data(), array[1..].as_ptr());
    assert_eq!(subset.size(), 2);

    subset = subset.subset(1, 0);
    assert!(subset.empty());
    assert_eq!(subset.size(), 0);
}

// Mutable slices are the Rust borrowing substrate; the production Span is a
// shared view, so mutable borrowing remains a language-level assertion.
fn accepts_mut_span(_: &mut [i32]) {}
fn accepts_const_span(_: Span<'_, i32>) {}

#[test]
fn span_const_and_containers_direct_port() {
    let const_array = [1, 2, 3, 4];
    accepts_const_span(Span::from(&const_array));

    let mut array = [1, 2, 3, 4];
    accepts_mut_span(&mut array);
    accepts_const_span(Span::from(&array));

    let mut values = Vec::<i32>::new();
    accepts_mut_span(&mut values);
    accepts_const_span(Span::new(&values));
}

#[test]
fn can_iterate_span_direct_port() {
    let array = [2, 4, 8, 16];
    let mut expected = 2;
    for value in make_span(&array).iter() {
        assert_eq!(*value, expected);
        expected *= 2;
    }
}
