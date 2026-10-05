//! Upstream tests/unit_tests/runtime/layout_container_alignment_test.cpp.
use super::*;
#[test]
fn container_alignment_resolves_onto_row_and_column_axes() {
    use LayoutAlignmentType as A;
    use LayoutCrossAlign as X;
    use LayoutMainDistribute as M;
    let rows = [
        (A::TopLeft, M::Start, X::Start, M::Start, X::Start),
        (A::TopCenter, M::Center, X::Start, M::Start, X::Center),
        (A::TopRight, M::End, X::Start, M::Start, X::End),
        (A::CenterLeft, M::Start, X::Center, M::Center, X::Start),
        (A::Center, M::Center, X::Center, M::Center, X::Center),
        (A::CenterRight, M::End, X::Center, M::Center, X::End),
        (A::BottomLeft, M::Start, X::End, M::End, X::Start),
        (A::BottomCenter, M::Center, X::End, M::End, X::Center),
        (A::BottomRight, M::End, X::End, M::End, X::End),
        (
            A::SpaceBetweenStart,
            M::SpaceBetween,
            X::Start,
            M::SpaceBetween,
            X::Start,
        ),
        (
            A::SpaceBetweenCenter,
            M::SpaceBetween,
            X::Center,
            M::SpaceBetween,
            X::Center,
        ),
        (
            A::SpaceBetweenEnd,
            M::SpaceBetween,
            X::End,
            M::SpaceBetween,
            X::End,
        ),
    ];
    for (kind, row_main, row_cross, column_main, column_cross) in rows {
        let row = container_alignment(kind, true);
        assert_eq!(row.main, row_main);
        assert_eq!(row.cross, row_cross);
        let column = container_alignment(kind, false);
        assert_eq!(column.main, column_main);
        assert_eq!(column.cross, column_cross);
    }
}
