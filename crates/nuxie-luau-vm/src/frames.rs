//! The frame stack (C++ `CallInfo` array) of one thread, and what code
//! outside the call protocol sees of it.
//!
//! **A frameless native leaves no record until something looks**
//! (docs/luau-safe-vm-call-path.md, section 4.1). A plain native called
//! by `CALL` runs without a frame: the call only stores its callee slot
//! in `Core::frameless`. The list's methods are visible to the call
//! protocol's modules only (`crate::protocol`); every other reader gets an
//! [`Observed`] view from `Core::frames_observed`, which first pushes the
//! real frame a pending frameless native stands for. An observer therefore
//! never sees a frameless native: it sees exactly the frame a framed
//! native has, so error positions, argument-error names, `debug.info`,
//! tracebacks and `getfenv` levels are computed on the same frame shape
//! as for any call.

use crate::vm::{Core, Frame};

#[derive(Default)]
pub(crate) struct Frames {
    list: Vec<Frame>,
}

impl Frames {
    #[inline]
    pub(in crate::protocol) fn len(&self) -> usize {
        self.list.len()
    }

    #[inline]
    pub(in crate::protocol) fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// The innermost frame.
    #[inline]
    pub(in crate::protocol) fn last(&self) -> Option<&Frame> {
        self.list.last()
    }

    #[inline]
    pub(in crate::protocol) fn last_mut(&mut self) -> Option<&mut Frame> {
        self.list.last_mut()
    }

    /// Frame `index`, counted from the outermost.
    #[inline]
    pub(in crate::protocol) fn get(&self, index: usize) -> Option<&Frame> {
        self.list.get(index)
    }

    #[inline]
    pub(in crate::protocol) fn get_mut(&mut self, index: usize) -> Option<&mut Frame> {
        self.list.get_mut(index)
    }

    pub(in crate::protocol) fn first_mut(&mut self) -> Option<&mut Frame> {
        self.list.first_mut()
    }

    /// Outermost first.
    pub(in crate::protocol) fn iter(&self) -> impl Iterator<Item = &Frame> {
        self.list.iter()
    }

    #[inline]
    pub(in crate::protocol) fn push(&mut self, frame: Frame) {
        self.list.push(frame);
    }

    /// Removes the innermost frame.
    #[inline]
    pub(in crate::protocol) fn pop(&mut self) -> Option<Frame> {
        self.list.pop()
    }

    /// Keeps the outermost `len` frames.
    pub(in crate::protocol) fn truncate(&mut self, len: usize) {
        self.list.truncate(len);
    }
}

/// A thread's frames as an observer outside the call protocol sees them:
/// no frameless native is pending (`Core::frames_observed` pushed it), so
/// every call in progress has its frame. Read-only; derefs to the `Core`
/// it was taken from, so an observer can read the heap beside it.
pub(crate) struct Observed<'a> {
    core: &'a Core,
    frames: &'a Frames,
    /// The innermost frame's pc (`Core::pc` for the running thread).
    pc: usize,
}

impl<'a> Observed<'a> {
    #[inline]
    pub(in crate::protocol) fn new(core: &'a Core, frames: &'a Frames, pc: usize) -> Self {
        Observed { core, frames, pc }
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.frames.len()
    }

    /// Frame `index`, counted from the outermost.
    #[inline]
    pub(crate) fn get(&self, index: usize) -> Option<&'a Frame> {
        self.frames.get(index)
    }

    /// The innermost frame.
    #[inline]
    pub(crate) fn last(&self) -> Option<&'a Frame> {
        self.frames.last()
    }

    /// Frame `level` calls below the innermost (0 is the innermost), with
    /// its index.
    #[inline]
    pub(crate) fn at_level(&self, level: usize) -> Option<(usize, &'a Frame)> {
        let index = self.len().checked_sub(1)?.checked_sub(level)?;
        Some((index, self.get(index)?))
    }

    /// The pc of frame `index`: the running pc for the innermost, its
    /// `savedpc` for the others.
    pub(crate) fn pc_of(&self, index: usize) -> usize {
        if index.saturating_add(1) == self.len() {
            self.pc
        } else {
            self.get(index).map_or(0, |f| f.savedpc)
        }
    }
}

impl std::ops::Deref for Observed<'_> {
    type Target = Core;
    #[inline]
    fn deref(&self) -> &Core {
        self.core
    }
}
