//! ViewModel-property transition comparator ownership.
//!
//! Mirrors pinned C++
//! `src/animation/transition_property_viewmodel_comparator.cpp`.

use super::TransitionConditionOp;
use nuxie_binary::{RuntimeFile, RuntimeObject};

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionPropertyViewModelComparator<'a> {
    comparator: &'a RuntimeObject,
    bindable: &'a RuntimeObject,
}

impl<'a> RuntimeTransitionPropertyViewModelComparator<'a> {
    pub(super) fn from_object(
        file: &'a RuntimeFile,
        comparator: &'a RuntimeObject,
    ) -> Option<Self> {
        if comparator.type_name != "TransitionPropertyViewModelComparator" {
            return None;
        }
        Some(Self {
            comparator,
            bindable: file.latest_bindable_property_for_object(comparator)?,
        })
    }

    pub(super) fn bindable(self) -> &'a RuntimeObject {
        debug_assert_eq!(
            self.comparator.type_name,
            "TransitionPropertyViewModelComparator"
        );
        self.bindable
    }
}

pub(super) fn compare_view_model_integer_pair(
    op: TransitionConditionOp,
    left: u64,
    right: u64,
) -> bool {
    // Pinned C++ resolves two BindablePropertyInteger comparands to
    // ComparisonShape::Uint32. Only equality and inequality are implemented
    // for that shape; ordering operations return false.
    op.compare_u32_equal_only(left as u32, right as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_pairs_keep_uint_precision_and_cpp_operation_support() {
        let left = 0x0100_0001;
        let right = 0x0100_0000;
        assert!(!compare_view_model_integer_pair(
            TransitionConditionOp::Equal,
            left,
            right,
        ));
        assert!(compare_view_model_integer_pair(
            TransitionConditionOp::NotEqual,
            left,
            right,
        ));
        assert!(!compare_view_model_integer_pair(
            TransitionConditionOp::LessThan,
            left,
            right,
        ));
        assert!(!compare_view_model_integer_pair(
            TransitionConditionOp::GreaterThan,
            left,
            right,
        ));
        assert!(compare_view_model_integer_pair(
            TransitionConditionOp::Equal,
            u64::from(u32::MAX) + 1,
            0,
        ));
    }
}
