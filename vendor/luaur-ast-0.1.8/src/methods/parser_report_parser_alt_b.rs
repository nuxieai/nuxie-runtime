use crate::records::location::Location;
use crate::records::parser::Parser;

impl Parser {
    #[allow(non_snake_case)]
    pub fn report_location_c_char_item(
        &mut self,
        location: Location,
        format: core::fmt::Arguments<'_>,
    ) {
        self.report(location, format);
    }
}

#[allow(non_snake_case)]
pub fn parser_report(parser: &mut Parser, location: Location, format: core::fmt::Arguments<'_>) {
    parser.report_location_c_char_item(location, format);
}
