use crate::records::position::Position;
use alloc::string::String;

// C++ `StringWriter : Writer`. `Writer` is an abstract base (a Rust trait), so
// `StringWriter` *implements* it rather than embedding it — the `impl Writer for
// StringWriter` lives with the PrettyPrinter methods; the inherent methods below
// are the overrides.
#[derive(Debug, Clone)]
pub struct StringWriter {
    pub(crate) ss: String,
    pub(crate) pos: Position,
    pub(crate) last_char: char,
}

impl StringWriter {
    pub(crate) fn str(&self) -> &String {
        &self.ss
    }

    pub(crate) fn advance(&mut self, newPos: &Position) {
        while self.pos.line < newPos.line {
            self.newline();
        }

        if self.pos.column < newPos.column {
            let count = (newPos.column - self.pos.column) as usize;
            self.write(&" ".repeat(count));
        }
    }

    pub(crate) fn maybe_space(&mut self, newPos: &Position, reserve: i32) {
        if self.pos.column + (reserve as u32) < newPos.column {
            self.space();
        }
    }

    pub(crate) fn newline(&mut self) {
        self.ss.push('\n');
        self.pos.column = 0;
        self.pos.line += 1;
        self.last_char = '\n';
    }

    pub(crate) fn space(&mut self) {
        self.ss.push(' ');
        self.pos.column += 1;
        self.last_char = ' ';
    }

    pub(crate) fn write_multiline(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }

        self.ss.push_str(s);
        self.last_char = s.chars().last().unwrap_or('\0');

        let mut index = 0;
        let mut numLines = 0;
        let bytes = s.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'\n' {
                numLines += 1;
                index = i + 1;
            }
        }

        self.pos.line += numLines as u32;
        if numLines > 0 {
            self.pos.column = (s.len() - index) as u32;
        } else {
            self.pos.column += s.len() as u32;
        }
    }

    pub(crate) fn write(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }

        self.ss.push_str(s);
        self.pos.column += s.len() as u32;
        self.last_char = s.chars().last().unwrap_or('\0');
    }

    pub(crate) fn write_char(&mut self, c: char) {
        self.ss.push(c);
        self.pos.column += 1;
        self.last_char = c;
    }

    pub(crate) fn identifier(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }

        if crate::records::string_writer::is_identifier_char(self.last_char) {
            self.space();
        }

        self.write(s);
    }

    pub(crate) fn keyword(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }

        if crate::records::string_writer::is_identifier_char(self.last_char) {
            self.space();
        }

        self.write(s);
    }

    pub(crate) fn symbol(&mut self, s: &str) {
        self.write(s);
    }

    pub(crate) fn literal(&mut self, s: &str) {
        if s.is_empty() {
            return;
        } else if crate::records::string_writer::is_identifier_char(self.last_char)
            && s.chars().next().map_or(false, |c| c.is_ascii_digit())
        {
            self.space();
        }

        self.write(s);
    }

    pub(crate) fn string(&mut self, s: &str) {
        let mut quote = '\'';
        if s.contains('\'') {
            quote = '\"';
        }

        self.write_char(quote);
        self.write(&luaur_common::functions::escape::escape(s, false));
        self.write_char(quote);
    }

    pub(crate) fn source_string(
        &mut self,
        s: &str,
        quote_style: crate::enums::quote_style_cst::QuoteStyle,
        block_depth: u32,
    ) {
        use crate::enums::quote_style_cst::QuoteStyle;
        if quote_style == QuoteStyle::QuotedRaw {
            let blocks = "=".repeat(block_depth as usize);
            self.write_char('[');
            self.write(&blocks);
            self.write_char('[');
            self.write_multiline(s);
            self.write_char(']');
            self.write(&blocks);
            self.write_char(']');
        } else {
            debug_assert!(block_depth == 0);

            let quote = match quote_style {
                QuoteStyle::QuotedDouble => '"',
                QuoteStyle::QuotedSingle => '\'',
                QuoteStyle::QuotedInterp => '`',
                _ => {
                    debug_assert!(false, "Unhandled quote type");
                    '"'
                }
            };

            self.write_char(quote);
            self.write_multiline(s);
            self.write_char(quote);
        }
    }
}

#[inline]
fn is_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
