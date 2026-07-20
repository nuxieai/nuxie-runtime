use crate::macros::markobject::markobject;
use crate::macros::markvalue::markvalue;
use crate::records::global_state::global_State;
use crate::records::lua_t_value::TValue;
use crate::records::luau_object::LuauObject;

#[allow(non_snake_case)]
pub fn traverseobject(g: *mut global_State, classinst: *mut LuauObject) {
    unsafe {
        // markobject(g, classinst->lclass);
        markobject!(g, (*classinst).lclass);

        // for (int i = 0; i < classinst->numberofmembers; i++)
        //     markvalue(g, &classinst->members[i]);
        let numberofmembers = (*classinst).numberofmembers;
        let members = (*classinst).members;
        for i in 0..numberofmembers as usize {
            let member_ptr = members.add(i);
            markvalue!(g, member_ptr as *mut TValue);
        }
    }
}
