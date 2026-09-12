//! Authored automatic physical margins, separate from alignment helper margins.
use super::{Diagnostic, Direction, unsupported};

#[derive(Clone, Copy, Default)]
pub(super) struct Margins(pub [bool; 4]); // left, top, right, bottom
impl Margins {
    pub fn apply(&mut self, name: &str, text: &str, parent: Self, source: &str) -> Result<(), Diagnostic> {
        let text = crate::css_whitespace::trim(text).to_ascii_lowercase();
        let side = match name { "margin-left" => Some(0), "margin-top" => Some(1), "margin-right" => Some(2), "margin-bottom" => Some(3), _ => None };
        if let Some(side) = side {
            self.0[side] = value(&text, parent.0[side], source)?;
        } else if text == "inherit" {
            *self = parent;
        } else if matches!(text.as_str(), "initial" | "unset") {
            *self = Self::default();
        } else {
            let tokens: Vec<_> = crate::css_whitespace::words(&text).collect();
            if !(1..=4).contains(&tokens.len()) || tokens.iter().any(|v| matches!(*v, "inherit" | "initial" | "unset")) {
                return Err(unsupported(source, "margin requires one to four auto/zero values or one CSS-wide keyword"));
            }
            let sides: Vec<_> = tokens.iter().map(|v| value(v, false, source)).collect::<Result<_, _>>()?;
            // CSS shorthand order is top/right/bottom/left, internal order is physical.
            let top = sides[0]; let right = *sides.get(1).unwrap_or(&top);
            let bottom = *sides.get(2).unwrap_or(&top); let left = *sides.get(3).unwrap_or(&right);
            self.0 = [left, top, right, bottom];
        }
        Ok(())
    }
    pub fn cross(self, direction: Direction) -> bool {
        if direction.is_row() { self.vertical() } else { self.0[0] || self.0[2] }
    }
    pub fn main(self, direction: Direction) -> bool {
        self.cross(if direction.is_row() { Direction::Column } else { Direction::Row })
    }
    pub fn vertical(self) -> bool { self.0[1] || self.0[3] }
    pub fn any(self) -> bool { self.0.into_iter().any(|v| v) }
}
fn value(text: &str, inherited: bool, source: &str) -> Result<bool, Diagnostic> {
    match text {
        "auto" => Ok(true), "inherit" => Ok(inherited), "initial" | "unset" => Ok(false),
        _ => {
            // Nonzero/percentage margins are separate backlog work. Parse zero
            // lengths through the shared grammar, without admitting auto sizes.
            if matches!(super::size(text, source)?, super::SpecifiedSize::Pixels(0.) | super::SpecifiedSize::Em(0.) | super::SpecifiedSize::Rem(0.)) { Ok(false) }
            else { Err(unsupported(source, "Margins currently admit auto and zero lengths; nonzero and percentage margins require separate validation")) }
        }
    }
}
