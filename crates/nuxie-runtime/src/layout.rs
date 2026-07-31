/// Direct value owner for pinned C++ `Alignment` (`src/layout.cpp:5-21`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Alignment {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Alignment {
    pub(crate) const TOP_LEFT: Self = Self::new(-1.0, -1.0);
    pub(crate) const TOP_CENTER: Self = Self::new(0.0, -1.0);
    pub(crate) const TOP_RIGHT: Self = Self::new(1.0, -1.0);
    pub(crate) const CENTER_LEFT: Self = Self::new(-1.0, 0.0);
    pub(crate) const CENTER: Self = Self::new(0.0, 0.0);
    pub(crate) const CENTER_RIGHT: Self = Self::new(1.0, 0.0);
    pub(crate) const BOTTOM_LEFT: Self = Self::new(-1.0, 1.0);
    pub(crate) const BOTTOM_CENTER: Self = Self::new(0.0, 1.0);
    pub(crate) const BOTTOM_RIGHT: Self = Self::new(1.0, 1.0);

    pub(crate) const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[cfg(test)]
mod tests {
    use super::Alignment;

    #[test]
    fn named_alignments_match_cpp_values() {
        assert_eq!(
            [
                Alignment::TOP_LEFT,
                Alignment::TOP_CENTER,
                Alignment::TOP_RIGHT,
                Alignment::CENTER_LEFT,
                Alignment::CENTER,
                Alignment::CENTER_RIGHT,
                Alignment::BOTTOM_LEFT,
                Alignment::BOTTOM_CENTER,
                Alignment::BOTTOM_RIGHT,
            ],
            [
                Alignment::new(-1.0, -1.0),
                Alignment::new(0.0, -1.0),
                Alignment::new(1.0, -1.0),
                Alignment::new(-1.0, 0.0),
                Alignment::new(0.0, 0.0),
                Alignment::new(1.0, 0.0),
                Alignment::new(-1.0, 1.0),
                Alignment::new(0.0, 1.0),
                Alignment::new(1.0, 1.0),
            ]
        );
    }
}
