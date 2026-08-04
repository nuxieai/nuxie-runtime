use crate::functions::is_stat_last::is_stat_last;
use crate::records::allocator::Allocator;
use crate::records::ast_array::AstArray;
use crate::records::ast_stat::AstStat;
use crate::records::ast_stat_block::AstStatBlock;
use crate::records::cst_node::CstNode;
use crate::records::lexeme::Lexeme;
use crate::records::location::Location;
use crate::records::parser::Parser;
use crate::records::position::Position;
use crate::records::temp_vector::TempVector;

impl Parser {
    pub fn parse_block_no_scope(&mut self) -> *mut AstStatBlock {
        let mut body = TempVector::new(&mut self.scratch_stat);

        let prev_position = self.lexer.previous_location().end;

        while !self.block_follow(self.lexer.current()) {
            let old_recursion_count = self.recursion_counter;

            self.increment_recursion_counter("block");

            let stat = self.parse_stat();

            self.recursion_counter = old_recursion_count;

            if self.lexer.current().r#type == crate::records::lexeme::Type::Semicolon {
                self.next_lexeme();
                unsafe {
                    (*stat).has_semicolon = true;
                }
                unsafe {
                    (*stat).base.location.end = self.lexer.previous_location().end;
                }
            }

            body.push_back(stat);

            if is_stat_last(stat) {
                break;
            }
        }

        let location = Location::new(prev_position, self.lexer.current().location.begin);

        let node = unsafe {
            (*self.allocator).alloc(AstStatBlock::new(
                location,
                self.copy_temp_vector_t(&body),
                true,
            ))
        };

        // C++ parseBlockNoScope creates NO CstNode for a block (there is no
        // CstStatBlock in the Luau parser at all). The port invented one, which gave
        // the module/root block a spurious CST entry (do/while/etc. blocks get their
        // own CstStatDo/... from their statement parser). Removed.

        node as *mut AstStatBlock
    }
}
