use crate::type_aliases::thread_context_provider::ThreadContextProvider;

pub fn thread_context_provider() -> &'static mut ThreadContextProvider {
    static mut HANDLER: ThreadContextProvider = {
        extern "C" fn default_provider() -> *mut crate::records::thread_context::ThreadContext {
            core::ptr::null_mut()
        }
        default_provider
    };

    // Safety: In the original C++, this is a function-local static.
    // While C++11 guarantees thread-safe initialization for statics,
    // it does not guarantee thread-safe access to the object itself.
    // Luau's TimeTrace usage of this global provider is typically
    // configured once at startup or in a single-threaded context.
    unsafe { &mut HANDLER }
}
