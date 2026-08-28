//! Correspondence owner for pinned `rive/refcnt.hpp`.
//!
//! The C++ owner supplies intrusive `RefCnt`/`rcp` lifetime management. The
//! Rust port deliberately exposes no equivalent raw-pointer API: each live
//! owner selects its truthful native representation explicitly (`Box` for a
//! unique renderer resource, `Rc` for shared immutable identity, or
//! `Rc<RefCell<_>>` for shared single-threaded mutable identity). This keeps
//! allocation, cloning, mutation, and destruction structural and prevents an
//! intrusive compatibility layer from manufacturing aliasing references.
