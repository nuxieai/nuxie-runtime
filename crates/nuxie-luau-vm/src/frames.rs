//! The frame stack (C++ `CallInfo` array) of one thread.
//!
//! A native that neither calls Lua nor yields runs **frameless**: `CALL`
//! fills the reserved [`Frames::native`] frame instead of pushing one
//! (docs/luau-safe-vm-embedding-core.md, section 2.1). Every view of the
//! stack here (`len`, `get`, `last`, `iter`, ...) includes that frame while
//! it is active, at the top, exactly where a pushed frame would be, so error
//! positions, `debug.info`, `getfenv`, tracebacks, the collector's roots and
//! the argument-error names see the native as they would see a real frame.
//! The list itself is private: code outside this module cannot walk the
//! frames without going through these views.

use crate::vm::Frame;

#[derive(Default)]
pub(crate) struct Frames {
    list: Vec<Frame>,
    /// The frame of the native running frameless, part of every view while
    /// `native_active`. Its fields other than the ones `enter_native` sets
    /// stay at their defaults: it is taken (reset) whenever it is pushed.
    native: Frame,
    native_active: bool,
}

impl Frames {
    /// The number of frames, a frameless native's included.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.list.len().saturating_add(usize::from(self.native_active))
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.list.is_empty() && !self.native_active
    }

    /// The innermost frame.
    #[inline]
    pub(crate) fn last(&self) -> Option<&Frame> {
        if self.native_active {
            Some(&self.native)
        } else {
            self.list.last()
        }
    }

    #[inline]
    pub(crate) fn last_mut(&mut self) -> Option<&mut Frame> {
        if self.native_active {
            Some(&mut self.native)
        } else {
            self.list.last_mut()
        }
    }

    /// Frame `index`, counted from the outermost.
    #[inline]
    pub(crate) fn get(&self, index: usize) -> Option<&Frame> {
        match self.list.get(index) {
            Some(frame) => Some(frame),
            None if self.native_active && index == self.list.len() => Some(&self.native),
            None => None,
        }
    }

    #[inline]
    pub(crate) fn get_mut(&mut self, index: usize) -> Option<&mut Frame> {
        let len = self.list.len();
        match self.list.get_mut(index) {
            Some(frame) => Some(frame),
            None if self.native_active && index == len => Some(&mut self.native),
            None => None,
        }
    }

    pub(crate) fn first_mut(&mut self) -> Option<&mut Frame> {
        self.get_mut(0)
    }

    /// Outermost first.
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Frame> {
        self.list
            .iter()
            .chain(self.native_active.then_some(&self.native))
    }

    /// Pushes `frame` above every frame, a frameless native's included
    /// (which becomes a real frame first).
    #[inline]
    pub(crate) fn push(&mut self, frame: Frame) {
        self.materialize();
        self.list.push(frame);
    }

    /// Removes the innermost frame.
    #[inline]
    pub(crate) fn pop(&mut self) -> Option<Frame> {
        if self.native_active {
            self.native_active = false;
            Some(std::mem::take(&mut self.native))
        } else {
            self.list.pop()
        }
    }

    /// Keeps the outermost `len` frames.
    pub(crate) fn truncate(&mut self, len: usize) {
        if len <= self.list.len() {
            if self.native_active {
                self.native_active = false;
                self.native = Frame::default();
            }
            self.list.truncate(len);
        }
    }

    /// Starts a frameless native: the reserved frame becomes the innermost
    /// one. Only the fields a native frame sets are written; the rest are
    /// at their defaults (the frame is reset whenever it is pushed, and a
    /// frameless native does not change them).
    #[inline]
    pub(crate) fn enter_native(
        &mut self,
        func: usize,
        nargs: usize,
        nresults: i32,
        closure: crate::value::FnId,
        top: usize,
    ) {
        let frame = &mut self.native;
        frame.func = func;
        frame.base = func.saturating_add(1);
        frame.top = top;
        frame.nresults = nresults;
        frame.closure = closure;
        frame.nargs = nargs;
        self.native_active = true;
    }

    /// Ends a frameless native that returned.
    #[inline]
    pub(crate) fn leave_native(&mut self) {
        self.native_active = false;
    }

    /// Turns a frameless native's frame into a real one, so that it
    /// survives the native's return (an error raised by the native unwinds
    /// through it like any frame).
    #[inline]
    pub(crate) fn materialize(&mut self) {
        if self.native_active {
            self.native_active = false;
            self.list.push(std::mem::take(&mut self.native));
        }
    }
}
