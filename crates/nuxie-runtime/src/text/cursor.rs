//! Editable cursor ownership ported from `src/text/cursor.cpp`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CursorPosition {
    pub(crate) line_index: Option<usize>,
    pub(crate) codepoint_index: usize,
}

impl CursorPosition {
    pub(crate) const fn unresolved(codepoint_index: usize) -> Self {
        Self {
            line_index: None,
            codepoint_index,
        }
    }

    pub(crate) const fn zero() -> Self {
        Self {
            line_index: Some(0),
            codepoint_index: 0,
        }
    }

    pub(crate) fn offset(self, offset: isize) -> Self {
        let codepoint_index = if offset < 0 {
            self.codepoint_index.saturating_sub(offset.unsigned_abs())
        } else {
            self.codepoint_index.saturating_add(offset as usize)
        };
        Self::unresolved(codepoint_index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Cursor {
    pub(crate) start: CursorPosition,
    pub(crate) end: CursorPosition,
}

impl Cursor {
    pub(crate) const fn collapsed(position: CursorPosition) -> Self {
        Self {
            start: position,
            end: position,
        }
    }

    pub(crate) const fn at_start() -> Self {
        Self::collapsed(CursorPosition::zero())
    }

    pub(crate) fn first(self) -> CursorPosition {
        if self.start.codepoint_index < self.end.codepoint_index {
            self.start
        } else {
            self.end
        }
    }

    pub(crate) fn last(self) -> CursorPosition {
        if self.start.codepoint_index < self.end.codepoint_index {
            self.end
        } else {
            self.start
        }
    }

    pub(crate) fn is_collapsed(self) -> bool {
        self.start == self.end
    }

    pub(crate) fn has_selection(self) -> bool {
        !self.is_collapsed()
    }

    pub(crate) fn contains(self, codepoint_index: usize) -> bool {
        codepoint_index >= self.first().codepoint_index
            && codepoint_index < self.last().codepoint_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_cursor_operators_and_saturating_subtract_are_ported() {
        let a = CursorPosition::unresolved(1);
        let b = CursorPosition::unresolved(4);
        assert!(a.codepoint_index < b.codepoint_index);
        assert_eq!(a.offset(-1).codepoint_index, 0);
        assert_eq!(a.offset(-2).codepoint_index, 0);
        let reversed = Cursor { start: b, end: a };
        assert_eq!(reversed.first(), a);
        assert_eq!(reversed.last(), b);
        assert!(reversed.contains(2));
        assert!(!reversed.contains(4));
    }
}
