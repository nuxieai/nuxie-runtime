use crate::macros::markobject::markobject;
use crate::macros::markvalue::markvalue;
use crate::records::gc_object::GCObject;
use crate::records::global_state::global_State;
use crate::records::luau_class::LuauClass;

#[allow(non_snake_case)]
pub(crate) unsafe fn traverseclass(g: *mut global_State, classobject: *mut LuauClass) {
    let classobject = &*classobject;

    markobject!(g, classobject.name);
    markobject!(g, classobject.memberstooffset);

    for i in 0..classobject.numberofallmembers {
        markobject!(g, *classobject.offsettomember.add(i as usize));
    }

    for i in 0..(classobject.numberofallmembers - classobject.numberofinstancemembers) {
        markvalue!(g, classobject.staticmembers.add(i as usize));
    }

    markobject!(g, classobject.metatable);

    if !classobject.instancemetatable.is_null() {
        markobject!(g, classobject.instancemetatable);
    }
}
