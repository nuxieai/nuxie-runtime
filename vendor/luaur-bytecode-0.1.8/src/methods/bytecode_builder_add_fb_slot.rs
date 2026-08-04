use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_feedback_type::LuauFeedbackType;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn add_fb_slot(&mut self, t: LuauFeedbackType) -> u32 {
        LUAU_ASSERT!(t == LuauFeedbackType::LFT_CALLTARGET);
        self.fb_slots.push(self.get_instruction_count() as u32);
        (self.fb_slots.len() - 1) as u32
    }
}
