use crate::enums::quote_style_cst::QuoteStyle;
use crate::records::position::Position;

pub trait Writer {
    fn advance(&mut self, pos: &Position);
    fn newline(&mut self);
    fn space(&mut self);
    fn maybe_space(&mut self, new_pos: &Position, reserve: i32);
    fn write(&mut self, s: &str);
    fn write_multiline(&mut self, s: &str);
    fn identifier(&mut self, name: &str);
    fn keyword(&mut self, s: &str);
    fn symbol(&mut self, s: &str);
    fn literal(&mut self, s: &str);
    fn string(&mut self, s: &str);
    fn source_string(&mut self, s: &str, quote_style: QuoteStyle, block_depth: u32);
}
