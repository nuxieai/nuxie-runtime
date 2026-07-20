use core::mem;

pub fn variant_fn_move() {
    // This C++ function is a low-level placement-new move used to implement
    // Variant alternative moves via raw memory operations.
    //
    // In this Rust port, moves are naturally handled by Rust assignment and
    // enum variant moves, so this function is intentionally a no-op.
    let _ = mem::size_of::<()>(); // keep a side-effect-free use of `core`
}
