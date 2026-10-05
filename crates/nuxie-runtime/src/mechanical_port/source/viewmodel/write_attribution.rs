//! `include/rive/viewmodel/write_attribution.hpp` and
//! `src/viewmodel/write_attribution.cpp`: tools-only writer attribution.
use std::{
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum WriteSourceKind {
    #[default]
    luau,
    wasm,
    layer,
    listener,
    dataBind,
}

/// Non-owning source identities. Core objects use their stable arena slot
/// address; Lua states and native machine borrows use their address. These
/// tokens are never dereferenced and do not retain the attributed object.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WriteSource {
    pub kind: WriteSourceKind,
    pub object: usize,
    pub machine: Option<usize>,
}

// C++'s process-wide switch is atomic at Rust's shared-thread boundary.
static ENABLED: AtomicBool = AtomicBool::new(false);
thread_local! {
    static SOURCES: RefCell<Vec<WriteSource>> = const { RefCell::new(Vec::new()) };
}

pub struct WriteAttribution;
impl WriteAttribution {
    pub fn enabled() -> bool {
        ENABLED.load(Ordering::Relaxed)
    }
    pub fn enable(enabled: bool) {
        ENABLED.store(enabled, Ordering::Relaxed);
    }
    /// Snapshot the current thread's stack without holding a borrow during
    /// debugger callbacks that may themselves enter writer scopes.
    pub fn sources() -> Vec<WriteSource> {
        SOURCES.with(|sources| sources.borrow().clone())
    }
    /// At a frame boundary, where no scope is open.
    pub fn reset() {
        SOURCES.with(|sources| sources.borrow_mut().clear());
    }
}

pub struct WriteAttributionScope {
    pushed: bool,
    depth: usize,
    // A thread-local scope must be destroyed on its originating thread.
    _thread: PhantomData<Rc<()>>,
}
impl WriteAttributionScope {
    pub fn new(kind: WriteSourceKind, object: usize, machine: Option<usize>) -> Self {
        let pushed = WriteAttribution::enabled();
        let depth = SOURCES.with(|sources| {
            let mut sources = sources.borrow_mut();
            let depth = sources.len();
            if pushed {
                sources.push(WriteSource {
                    kind,
                    object,
                    machine,
                });
            }
            depth
        });
        Self {
            pushed,
            depth,
            _thread: PhantomData,
        }
    }
}
impl Drop for WriteAttributionScope {
    fn drop(&mut self) {
        if self.pushed {
            // Restore depth, not one pop: errors in a guest can bypass inner
            // destructors. The enclosing call bounds that abandoned state.
            SOURCES.with(|sources| {
                sources
                    .borrow_mut()
                    .resize(self.depth, WriteSource::default())
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writer_scopes_preserve_order_identity_and_restore_call_depth() {
        WriteAttribution::enable(false);
        WriteAttribution::reset();
        let disabled = WriteAttributionScope::new(WriteSourceKind::layer, 1, Some(2));
        assert!(WriteAttribution::sources().is_empty());
        drop(disabled);
        WriteAttribution::enable(true);
        {
            let _layer = WriteAttributionScope::new(WriteSourceKind::layer, 1, Some(2));
            {
                let _call = WriteAttributionScope::new(WriteSourceKind::luau, 3, None);
                let setter = WriteAttributionScope::new(WriteSourceKind::dataBind, 4, None);
                assert_eq!(
                    WriteAttribution::sources(),
                    vec![
                        WriteSource {
                            kind: WriteSourceKind::layer,
                            object: 1,
                            machine: Some(2)
                        },
                        WriteSource {
                            kind: WriteSourceKind::luau,
                            object: 3,
                            machine: None
                        },
                        WriteSource {
                            kind: WriteSourceKind::dataBind,
                            object: 4,
                            machine: None
                        },
                    ]
                );
                // Model the upstream longjmp case: the call owns restoration.
                std::mem::forget(setter);
            }
            assert_eq!(WriteAttribution::sources().len(), 1);
            std::thread::spawn(|| assert!(WriteAttribution::sources().is_empty()))
                .join()
                .unwrap();
            WriteAttribution::enable(false);
        }
        assert!(WriteAttribution::sources().is_empty());
        WriteAttribution::reset();
    }
}
