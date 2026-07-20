use crate::enums::feedback_vector_slot_kind::FeedbackVectorSlotKind;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FeedbackVectorSlot {
    pub kind: FeedbackVectorSlotKind,
    pub data: FeedbackVectorSlotData,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
#[repr(C)]
pub union FeedbackVectorSlotData {
    pub call_target: FeedbackVectorSlotCallTarget,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct FeedbackVectorSlotCallTarget {
    pub pc: u32,
    pub proto: u32,
    pub hits: u32,
}

impl Default for FeedbackVectorSlot {
    fn default() -> Self {
        Self {
            kind: FeedbackVectorSlotKind::CALL_TARGET,
            data: FeedbackVectorSlotData {
                call_target: FeedbackVectorSlotCallTarget::default(),
            },
        }
    }
}

impl Default for FeedbackVectorSlotData {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

impl core::fmt::Debug for FeedbackVectorSlotData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FeedbackVectorSlotData")
            .field("call_target", unsafe { &self.call_target })
            .finish()
    }
}
