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

use crate::call::{Saved, Wait};
use crate::vm::{Core, Frame};

/// What a frame needs beyond its 32 bytes, on the pending side stack
/// (docs/luau-safe-vm-call-path.md, 4.3): what it waits for, and the
/// C-call counters a counted call restores when it returns. Pushed by a
/// metamethod call, a native's request, a counted call and a yield; never
/// by `CALL` of a Lua function from Lua.
#[derive(Default)]
pub(crate) struct Pending {
    pub(crate) wait: Wait,
    pub(crate) restore: Option<Saved>,
}

/// The frame stack and its pending side stack: one entry per frame with
/// [`Frame::PENDING`] set, in frame order, so the innermost flagged
/// frame's entry is the last one.
#[derive(Default)]
pub(crate) struct Frames {
    list: Vec<Frame>,
    pending: Vec<Pending>,
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

    /// Removes the innermost frame and its pending entry, if it has one.
    pub(in crate::protocol) fn pop(&mut self) -> Option<(Frame, Option<Pending>)> {
        let frame = self.list.pop()?;
        let pending = if frame.has(Frame::PENDING) { self.pending.pop() } else { None };
        Some((frame, pending))
    }

    /// Keeps the outermost `len` frames, and their pending entries.
    pub(in crate::protocol) fn truncate(&mut self, len: usize) {
        let flagged = self
            .list
            .get(len..)
            .map_or(0, |gone| gone.iter().filter(|f| f.has(Frame::PENDING)).count());
        self.list.truncate(len);
        self.pending.truncate(self.pending.len().saturating_sub(flagged));
    }

    /// The innermost frame's pending entry, if it has one.
    #[inline]
    pub(in crate::protocol) fn top_pending(&mut self) -> Option<&mut Pending> {
        if self.list.last()?.has(Frame::PENDING) {
            self.pending.last_mut()
        } else {
            None
        }
    }

    /// The innermost frame's pending entry, made if it has none.
    pub(in crate::protocol) fn top_pending_or_insert(&mut self) -> Option<&mut Pending> {
        let frame = self.list.last_mut()?;
        if !frame.has(Frame::PENDING) {
            frame.set(Frame::PENDING);
            self.pending.push(Pending::default());
        }
        self.pending.last_mut()
    }

    /// The frames with a pending entry, innermost first, with their
    /// indices.
    pub(in crate::protocol) fn pending_rev(&self) -> impl Iterator<Item = (usize, &Pending)> {
        let flagged = self
            .list
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, f)| f.has(Frame::PENDING))
            .map(|(i, _)| i);
        flagged.zip(self.pending.iter().rev())
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
            self.get(index).map_or(0, Frame::savedpc)
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
