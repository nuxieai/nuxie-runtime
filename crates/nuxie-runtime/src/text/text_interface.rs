//! Text/TextInput dispatch ported from `src/text/text_interface.cpp`.

pub(crate) fn is_text_interface(type_name: Option<&str>) -> bool {
    matches!(type_name, Some("Text" | "TextInput"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_text_interface_dispatch_is_exact() {
        assert!(is_text_interface(Some("Text")));
        assert!(is_text_interface(Some("TextInput")));
        assert!(!is_text_interface(Some("RawText")));
        assert!(!is_text_interface(None));
    }
}
