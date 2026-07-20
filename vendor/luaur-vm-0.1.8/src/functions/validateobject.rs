use crate::functions::validateobjref::validateobjref;
use crate::functions::validateref::validateref;
use crate::macros::obj_2_gco::obj2gco;
use crate::records::global_state::global_State;
use crate::records::luau_object::LuauObject;

#[allow(non_snake_case)]
pub fn validateobject(g: *mut global_State, inst: *mut LuauObject) {
    unsafe {
        let obj = obj2gco!(inst);
        validateobjref(g, obj, obj2gco!((*inst).lclass));
        let numberofmembers = (*inst).numberofmembers;
        let members = (*inst).members;
        for i in 0..numberofmembers as usize {
            let member_ptr = members.add(i);
            validateref(g, obj, member_ptr);
        }
    }
}
