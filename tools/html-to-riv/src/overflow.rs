//! Preserve specified overflow values until both axes can be computed together.
//! Runtime admission is separate: an auto result cannot silently become a clip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Value {
    Visible,
    Clip,
    Hidden,
    Auto,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Overflow {
    pub x: Value,
    pub y: Value,
}

impl Default for Overflow {
    fn default() -> Self {
        Self::uniform(Value::Visible)
    }
}

impl Overflow {
    pub(crate) const fn uniform(value: Value) -> Self {
        Self { x: value, y: value }
    }

    pub(crate) fn computed(self) -> Self {
        fn scrollable(value: Value) -> bool {
            matches!(value, Value::Hidden | Value::Auto)
        }
        fn axis(value: Value, other: Value) -> Value {
            if !scrollable(other) { return value; }
            match value {
                Value::Visible => Value::Auto,
                Value::Clip => Value::Hidden,
                value => value,
            }
        }
        // Each conversion uses the specified opposite axis, not a partially
        // updated result. Keep specified values intact for later declarations.
        Self { x: axis(self.x, self.y), y: axis(self.y, self.x) }
    }

    pub(crate) fn axis_clip(self) -> Option<crate::OverflowAxis> {
        match (self.computed().x, self.computed().y) {
            (Value::Clip, Value::Visible) => Some(crate::OverflowAxis::X),
            (Value::Visible, Value::Clip) => Some(crate::OverflowAxis::Y),
            _ => None,
        }
    }

    /// Whether the current non-scrolling runtime can use its ordinary full clip.
    /// Auto is deliberately excluded even though browsers may clip its pixels.
    pub(crate) fn clips_both(self) -> bool {
        let value = self.computed();
        matches!(value.x, Value::Clip | Value::Hidden)
            && matches!(value.y, Value::Clip | Value::Hidden)
    }
}

#[cfg(test)]
mod tests {
    use super::{Overflow, Value::*};

    #[test]
    fn axis_pairs_compute_without_losing_specified_values() {
        let cases = [
            ((Visible, Visible), (Visible, Visible), false),
            ((Visible, Clip), (Visible, Clip), false),
            ((Clip, Visible), (Clip, Visible), false),
            ((Clip, Clip), (Clip, Clip), true),
            ((Hidden, Hidden), (Hidden, Hidden), true),
            ((Hidden, Clip), (Hidden, Hidden), true),
            ((Clip, Hidden), (Hidden, Hidden), true),
            ((Visible, Hidden), (Auto, Hidden), false),
            ((Hidden, Visible), (Hidden, Auto), false),
        ];
        for ((x,y),(cx,cy),clips) in cases {
            let specified = Overflow {x,y};
            assert_eq!(specified.computed(), Overflow {x:cx,y:cy});
            assert_eq!(specified.clips_both(), clips);
            assert_eq!(specified, Overflow {x,y});
        }
        // Overriding the scrollable opposite axis must restore visible, rather
        // than retaining auto from a previous computation.
        let mut specified = Overflow {x:Visible,y:Hidden};
        assert_eq!(specified.computed().x, Auto);
        specified.y = Clip;
        assert_eq!(specified.computed().x, Visible);
    }
}
