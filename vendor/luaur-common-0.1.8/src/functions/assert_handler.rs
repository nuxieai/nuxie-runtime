use crate::type_aliases::assert_handler::AssertHandler;

pub fn assert_handler() -> &'static mut AssertHandler {
    static mut HANDLER: AssertHandler = None;

    // Safety: In the original C++, this is a function-local static.
    // While C++11 guarantees thread-safe initialization for statics,
    // it does not guarantee thread-safe access to the object itself.
    // Luau's Common.h usage of this global handler is typically
    // configured once at startup or in a single-threaded test context.
    unsafe { &mut HANDLER }
}
