//! Computed wrapping declarations, independent of contextual scene admission.
use super::{unsupported, Diagnostic};
use crate::css_whitespace;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Wrap { #[default] NoWrap, Wrap, Reverse }
impl Wrap {
    pub(super) fn is_wrapped(self) -> bool { self != Self::NoWrap }
    pub(super) fn wire(self) -> u32 {
        match self { Self::NoWrap => 0, Self::Wrap => 1, Self::Reverse => 2 }
    }
}
pub(super) fn computed_wrap(text: &str, parent: Wrap, source: &str) -> Result<Wrap, Diagnostic> {
    match css_whitespace::trim(text).to_ascii_lowercase().as_str() {
        "nowrap" | "initial" | "unset" => Ok(Wrap::NoWrap),
        "wrap" => Ok(Wrap::Wrap), "wrap-reverse" => Ok(Wrap::Reverse),
        "inherit" => Ok(parent),
        _ => Err(unsupported(source, "flex-wrap admits nowrap, wrap, wrap-reverse, inherit, initial and unset")),
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Content { #[default] Normal, Stretch, FlexStart, Center, FlexEnd, Start, End }
impl Content {
    /// Fraction along the directed cross axis; logical start/end assume the
    /// compiler's admitted horizontal writing mode and left-to-right direction.
    pub(super) fn fraction(self, reverse_cross: bool) -> Option<f32> {
        match self {
            Self::Normal | Self::Stretch => None,
            Self::FlexStart => Some(0.), Self::Center => Some(0.5), Self::FlexEnd => Some(1.),
            Self::Start => Some(if reverse_cross { 1. } else { 0. }),
            Self::End => Some(if reverse_cross { 0. } else { 1. }),
        }
    }
}
pub(super) fn computed_content(text: &str, parent: Content, source: &str) -> Result<Content, Diagnostic> {
    match css_whitespace::trim(text).to_ascii_lowercase().as_str() {
        "normal" | "initial" | "unset" => Ok(Content::Normal),
        "stretch" => Ok(Content::Stretch), "flex-start" => Ok(Content::FlexStart),
        "center" => Ok(Content::Center), "flex-end" => Ok(Content::FlexEnd),
        "start" => Ok(Content::Start), "end" => Ok(Content::End), "inherit" => Ok(parent),
        _ => Err(unsupported(source, "align-content admits normal, stretch, flex-start, center, flex-end, start, end, inherit, initial and unset; other distributions and overflow alignment need qualification")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn computed_noninherited_resets_and_explicit_inheritance() {
        for parent in [Wrap::NoWrap, Wrap::Wrap, Wrap::Reverse] {
            assert_eq!(computed_wrap("inherit", parent, "test").unwrap(), parent);
            for reset in ["initial", "unset"] { assert_eq!(computed_wrap(reset, parent, "test").unwrap(), Wrap::default()); }
        }
        for parent in [Content::Normal, Content::Stretch, Content::FlexStart, Content::Center, Content::FlexEnd, Content::Start, Content::End] {
            assert_eq!(computed_content("inherit", parent, "test").unwrap(), parent);
            for reset in ["initial", "unset"] { assert_eq!(computed_content(reset, parent, "test").unwrap(), Content::default()); }
        }
        assert_eq!(computed_wrap(" \tWrAp-ReVeRsE\n", Wrap::NoWrap, "test").unwrap(), Wrap::Reverse);
        assert_eq!(computed_content(" CENTER ", Content::Normal, "test").unwrap(), Content::Center);
        for text in ["", "wrap wrap", "12px", "revert", "unknown"] { assert!(computed_wrap(text, Wrap::Wrap, "test").is_err()); }
        for text in ["", "center end", "space-between", "safe center", "baseline", "unknown"] { assert!(computed_content(text, Content::Center, "test").is_err()); }
    }
    #[test]
    fn wire_and_directed_cross_axis_fractions() {
        for (wrap, wire, wrapped) in [(Wrap::NoWrap, 0, false), (Wrap::Wrap, 1, true), (Wrap::Reverse, 2, true)] {
            assert_eq!(wrap.wire(), wire); assert_eq!(wrap.is_wrapped(), wrapped);
        }
        for reverse in [false, true] {
            assert_eq!(Content::Normal.fraction(reverse), None); assert_eq!(Content::Stretch.fraction(reverse), None);
            assert_eq!(Content::FlexStart.fraction(reverse), Some(0.)); assert_eq!(Content::Center.fraction(reverse), Some(0.5)); assert_eq!(Content::FlexEnd.fraction(reverse), Some(1.));
            assert_eq!(Content::Start.fraction(reverse), Some(if reverse { 1. } else { 0. }));
            assert_eq!(Content::End.fraction(reverse), Some(if reverse { 0. } else { 1. }));
        }
    }
}
