use super::Span;

fn accepts_mut_span(_: Span<'_, i32>) {}
fn accepts_const_span(_: Span<'_, i32>) {}

#[test]
fn wave_c4_span_002_const_and_containers() {
    let const_array = [1, 2, 3, 4];
    accepts_const_span(Span::new(&const_array));

    let array = [1, 2, 3, 4];
    accepts_mut_span(Span::new(&array));
    accepts_const_span(Span::new(&array));

    let values = Vec::<i32>::new();
    accepts_mut_span(Span::new(&values));
    accepts_const_span(Span::new(&values));
}

#[test]
fn wave_c4_span_003_can_iterate_span() {
    let array = [2, 4, 8, 16];

    let span = Span::new(&array);
    let mut expect = 2;
    for value in span.iter() {
        assert_eq!(*value, expect);
        expect *= 2;
    }
}
