//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:592:propagateall`
//! Source: `VM/src/lgc.cpp` (lgc.cpp:592-600, hand-ported)

use crate::functions::propagatemark::propagatemark;
use crate::records::global_state::global_State;

#[allow(non_snake_case)]
pub(crate) unsafe fn propagateall(g: *mut global_State) -> usize {
    let mut work: usize = 0;
    while !(*g).gray.is_null() {
        work += propagatemark(g);
    }
    work
}
