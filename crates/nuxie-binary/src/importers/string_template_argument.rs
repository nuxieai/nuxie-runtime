//! Direct import owner for pinned C++
//! `src/data_bind/converters/string_template/string_template_argument.cpp`
//! (upstream c63a5ba04).

use super::*;

/// `StringTemplateArgument::import`: an argument is written right after its
/// template and the template's binds, so it needs a BackboardImporter whose
/// last converter is a DataConverterStringTemplate. Its Super is Core, which
/// always imports.
pub(super) fn imports_successfully(
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    definition.is_a("StringTemplateArgument").then(|| {
        context.latest(ImportStackKey::Backboard)
            && context.last_data_converter == Some("DataConverterStringTemplate")
    })
}
