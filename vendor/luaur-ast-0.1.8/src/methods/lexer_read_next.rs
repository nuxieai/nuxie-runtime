//! `Lexeme Lexer::read_next()` — Ast/src/Lexer.cpp:719. The single-token dispatch.

use crate::enums::brace_type::BraceType;
use crate::functions::is_alpha::is_alpha;
use crate::functions::is_digit_lexer::is_digit;
use crate::records::lexeme::{Lexeme, Type};
use crate::records::lexer::Lexer;
use crate::records::location::Location;

impl Lexer {
    pub(crate) fn read_next(&mut self) -> Lexeme {
        let start = self.position();

        match self.peekch() {
            '\0' => Lexeme::new(Location::with_length(start, 0), Type::Eof),

            '-' => {
                if self.peekch_ahead(1) == '>' {
                    self.consume();
                    self.consume();
                    Lexeme::new(Location::with_length(start, 2), Type::SkinnyArrow)
                } else if self.peekch_ahead(1) == '=' {
                    self.consume();
                    self.consume();
                    Lexeme::new(Location::with_length(start, 2), Type::SubAssign)
                } else if self.peekch_ahead(1) == '-' {
                    self.read_comment_body()
                } else {
                    self.consume();
                    Lexeme::from_char(Location::with_length(start, 1), '-')
                }
            }

            '[' => {
                let sep = self.skip_long_separator();

                if sep >= 0 {
                    self.read_long_string(&start, sep, Type::RawString, Type::BrokenString)
                } else if sep == -1 {
                    Lexeme::from_char(Location::with_length(start, 1), '[')
                } else {
                    Lexeme::new(Location::new(start, self.position()), Type::BrokenString)
                }
            }

            '{' => {
                self.consume();

                if !self.brace_stack.is_empty() {
                    self.brace_stack.push(BraceType::Normal);
                }

                Lexeme::from_char(Location::with_length(start, 1), '{')
            }

            '}' => {
                self.consume();

                if self.brace_stack.is_empty() {
                    return Lexeme::from_char(Location::with_length(start, 1), '}');
                }

                let brace_stack_top = *self.brace_stack.last().unwrap();
                self.brace_stack.pop();

                if brace_stack_top != BraceType::InterpolatedString {
                    return Lexeme::from_char(Location::with_length(start, 1), '}');
                }

                self.read_interpolated_string_section(
                    start,
                    Type::InterpStringMid,
                    Type::InterpStringEnd,
                )
            }

            '=' => {
                self.consume();
                if self.peekch() == '=' {
                    self.consume();
                    Lexeme::new(Location::with_length(start, 2), Type::Equal)
                } else {
                    Lexeme::from_char(Location::with_length(start, 1), '=')
                }
            }

            '<' => {
                self.consume();
                if self.peekch() == '=' {
                    self.consume();
                    Lexeme::new(Location::with_length(start, 2), Type::LessEqual)
                } else {
                    Lexeme::from_char(Location::with_length(start, 1), '<')
                }
            }

            '>' => {
                self.consume();
                if self.peekch() == '=' {
                    self.consume();
                    Lexeme::new(Location::with_length(start, 2), Type::GreaterEqual)
                } else {
                    Lexeme::from_char(Location::with_length(start, 1), '>')
                }
            }

            '~' => {
                self.consume();
                if self.peekch() == '=' {
                    self.consume();
                    Lexeme::new(Location::with_length(start, 2), Type::NotEqual)
                } else {
                    Lexeme::from_char(Location::with_length(start, 1), '~')
                }
            }

            '"' | '\'' => self.read_quoted_string(),

            '`' => self.read_interpolated_string_begin(),

            '.' => {
                self.consume();

                if self.peekch() == '.' {
                    self.consume();

                    if self.peekch() == '.' {
                        self.consume();
                        Lexeme::new(Location::with_length(start, 3), Type::Dot3)
                    } else if self.peekch() == '=' {
                        self.consume();
                        Lexeme::new(Location::with_length(start, 3), Type::ConcatAssign)
                    } else {
                        Lexeme::new(Location::with_length(start, 2), Type::Dot2)
                    }
                } else if is_digit(self.peekch()) {
                    self.read_number(&start, self.offset - 1)
                } else {
                    Lexeme::from_char(Location::with_length(start, 1), '.')
                }
            }

            '+' => {
                self.consume();
                if self.peekch() == '=' {
                    self.consume();
                    Lexeme::new(Location::with_length(start, 2), Type::AddAssign)
                } else {
                    Lexeme::from_char(Location::with_length(start, 1), '+')
                }
            }

            '/' => {
                self.consume();

                let ch = self.peekch();

                if ch == '=' {
                    self.consume();
                    Lexeme::new(Location::with_length(start, 2), Type::DivAssign)
                } else if ch == '/' {
                    self.consume();

                    if self.peekch() == '=' {
                        self.consume();
                        Lexeme::new(Location::with_length(start, 3), Type::FloorDivAssign)
                    } else {
                        Lexeme::new(Location::with_length(start, 2), Type::FloorDiv)
                    }
                } else {
                    Lexeme::from_char(Location::with_length(start, 1), '/')
                }
            }

            '*' => {
                self.consume();
                if self.peekch() == '=' {
                    self.consume();
                    Lexeme::new(Location::with_length(start, 2), Type::MulAssign)
                } else {
                    Lexeme::from_char(Location::with_length(start, 1), '*')
                }
            }

            '%' => {
                self.consume();
                if self.peekch() == '=' {
                    self.consume();
                    Lexeme::new(Location::with_length(start, 2), Type::ModAssign)
                } else {
                    Lexeme::from_char(Location::with_length(start, 1), '%')
                }
            }

            '^' => {
                self.consume();
                if self.peekch() == '=' {
                    self.consume();
                    Lexeme::new(Location::with_length(start, 2), Type::PowAssign)
                } else {
                    Lexeme::from_char(Location::with_length(start, 1), '^')
                }
            }

            ':' => {
                self.consume();
                if self.peekch() == ':' {
                    self.consume();
                    Lexeme::new(Location::with_length(start, 2), Type::DoubleColon)
                } else {
                    Lexeme::from_char(Location::with_length(start, 1), ':')
                }
            }

            '(' | ')' | ']' | ';' | ',' | '#' | '?' | '&' | '|' => {
                let ch = self.peekch();
                self.consume();

                Lexeme::from_char(Location::with_length(start, 1), ch)
            }

            '@' => {
                if self.peekch_ahead(1) == '[' {
                    self.consume();
                    self.consume();

                    Lexeme::new(Location::with_length(start, 2), Type::AttributeOpen)
                } else {
                    // consume @ first
                    self.consume();

                    if is_alpha(self.peekch()) || self.peekch() == '_' {
                        let attribute = self.read_name();
                        Lexeme::with_name(
                            Location::new(start, self.position()),
                            Type::Attribute,
                            attribute.0.value,
                        )
                    } else {
                        Lexeme::with_name(
                            Location::new(start, self.position()),
                            Type::Attribute,
                            b"\0".as_ptr() as *const core::ffi::c_char,
                        )
                    }
                }
            }

            _ => {
                if is_digit(self.peekch()) {
                    self.read_number(&start, self.offset)
                } else if is_alpha(self.peekch()) || self.peekch() == '_' {
                    let name = self.read_name();
                    Lexeme::with_name(Location::new(start, self.position()), name.1, name.0.value)
                } else if ((self.peekch() as u8) & 0x80) != 0 {
                    self.read_utf_8_error()
                } else {
                    let ch = self.peekch();
                    self.consume();

                    Lexeme::from_char(Location::with_length(start, 1), ch)
                }
            }
        }
    }
}
