use crate::records::ast_array::AstArray;
use crate::records::ast_attr::AstAttr;
use crate::records::ast_stat::AstStat;
use crate::records::parser::Parser;

impl Parser {
    #[allow(non_snake_case)]
    pub fn parseLocal_DEPRECATED(&mut self, attributes: &AstArray<*mut AstAttr>) -> *mut AstStat {
        // C++: `Location start = lexer.current().location; if (attributes.size > 0)
        // start = attributes.data[0]->location;` — when attributes are present the
        // statement begins at the attribute, so a `local function` start location
        // includes the leading `@native`/`@checked`. The port ignored attributes.
        let mut start = self.lexer.current().location;
        if attributes.size > 0 {
            start = unsafe { (**attributes.data.add(0)).base.location };
        }
        self.parse_local(
            start,
            self.lexer.current().location.begin,
            attributes,
            false,
        )
    }
}
