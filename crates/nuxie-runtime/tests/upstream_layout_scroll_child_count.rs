//! New runtime/layout_scroll_test.cpp regression at 955d6a05.
#![cfg(feature = "testing")]
#[path = "support/layout_955.rs"]
mod support;
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags, constraints::scrolling::scroll_constraint::ScrollConstraint,
    generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
};
use support::*;
#[test]
fn scroll_constraint_constrains_each_layout_child_once_per_update() {
    let file = read_file("layout/layout_scroll_vertical.riv");
    let artboard = file.with_file(File::artboard).expect("source artboard");
    let scroll = artboard
        .with_downcast::<Artboard, _>(|a| a.find_all_handles::<ScrollConstraint>()[0].clone())
        .unwrap();
    let advance = || {
        Artboard::advance_handle(
            &artboard,
            0.0,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        )
    };
    advance();
    number(
        &scroll,
        ScrollConstraintBase::SCROLL_OFFSET_Y_PROPERTY_KEY,
        -100.0,
    );
    advance();
    scroll
        .with_downcast::<ScrollConstraint, _>(|s| {
            assert!(s.scroll_children().len() > 1);
            assert_eq!(
                s.child_constraint_applied_count(),
                s.scroll_children().len() as i32
            );
        })
        .unwrap();
}
