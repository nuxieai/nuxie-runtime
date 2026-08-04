//! `Lexeme::to_string` (`Ast/src/Lexer.cpp`).
//!
//! `Lexeme::Type` is a newtype over `i32`, so its named values are associated
//! consts used here as constant match patterns (`Type::Eof`), not glob-imported
//! enum variants. `Lexeme::data` is a C `union`, so the payload arms read the
//! active member through `unsafe { self.data.<member> }` rather than matching it.

use crate::functions::find_confusable::find_confusable;
use crate::records::lexeme::{Lexeme, Type};
use alloc::format;
use alloc::string::String;
use core::ffi::CStr;

#[allow(non_upper_case_globals)]
const kReserved: [&str; 21] = [
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in", "local",
    "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

impl Lexeme {
    #[allow(non_snake_case)]
    pub fn to_string(&self) -> String {
        // The active union member is selected by `self.r#type`; reading the
        // matching member is sound for these arms.
        let data_str = |fallback: &str, wrap: &dyn Fn(&str) -> String| -> String {
            let ptr = unsafe { self.data.data };
            if !ptr.is_null() {
                let s = unsafe {
                    core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                        ptr as *const u8,
                        self.length as usize,
                    ))
                };
                wrap(s)
            } else {
                String::from(fallback)
            }
        };

        let name_str = |fallback: &str| -> String {
            let ptr = unsafe { self.data.name };
            if !ptr.is_null() {
                let s = unsafe { CStr::from_ptr(ptr).to_string_lossy() };
                format!("'{}'", s)
            } else {
                String::from(fallback)
            }
        };

        match self.r#type {
            Type::Eof => String::from("<eof>"),
            Type::Equal => String::from("'=='"),
            Type::LessEqual => String::from("'<='"),
            Type::GreaterEqual => String::from("'>='"),
            Type::NotEqual => String::from("'~='"),
            Type::Dot2 => String::from("'..'"),
            Type::Dot3 => String::from("'...'"),
            Type::SkinnyArrow => String::from("'->'"),
            Type::DoubleColon => String::from("'::'"),
            Type::FloorDiv => String::from("'//'"),
            Type::AddAssign => String::from("'+='"),
            Type::SubAssign => String::from("'-='"),
            Type::MulAssign => String::from("'*='"),
            Type::DivAssign => String::from("'/='"),
            Type::FloorDivAssign => String::from("'//='"),
            Type::ModAssign => String::from("'%='"),
            Type::PowAssign => String::from("'^='"),
            Type::ConcatAssign => String::from("'..='"),

            Type::RawString | Type::QuotedString => data_str("string", &|s| format!("\"{}\"", s)),
            Type::InterpStringBegin => data_str("the beginning of an interpolated string", &|s| {
                format!("`{}{{", s)
            }),
            Type::InterpStringMid => data_str("the middle of an interpolated string", &|s| {
                format!("}}{}{{", s)
            }),
            Type::InterpStringEnd => data_str("the end of an interpolated string", &|s| {
                format!("}}{}`", s)
            }),
            Type::InterpStringSimple => data_str("interpolated string", &|s| format!("`{}`", s)),
            Type::Number => data_str("number", &|s| format!("'{}'", s)),

            Type::Name => name_str("identifier"),
            Type::Comment => String::from("comment"),
            Type::Attribute => name_str("attribute"),

            Type::AttributeOpen => String::from("'@['"),
            Type::BrokenString => String::from("malformed string"),
            Type::BrokenComment => String::from("unfinished comment"),
            Type::BrokenInterpDoubleBrace => {
                String::from("'{{', which is invalid (did you mean '\\{'?)")
            }

            Type::BrokenUnicode => {
                let cp = unsafe { self.data.codepoint };
                if cp != 0 {
                    let confusable_ptr = find_confusable(cp);
                    if !confusable_ptr.is_null() {
                        let confusable =
                            unsafe { CStr::from_ptr(confusable_ptr).to_string_lossy() };
                        format!(
                            "Unicode character U+{:x} (did you mean '{}'?)",
                            cp, confusable
                        )
                    } else {
                        format!("Unicode character U+{:x}", cp)
                    }
                } else {
                    String::from("invalid UTF-8 sequence")
                }
            }

            _ => {
                let type_val = self.r#type.0;
                if type_val < Type::Char_END.0 {
                    format!("'{}'", type_val as u8 as char)
                } else if type_val >= Type::Reserved_BEGIN.0 && type_val < Type::Reserved_END.0 {
                    let index = (type_val - Type::Reserved_BEGIN.0) as usize;
                    format!("'{}'", kReserved[index])
                } else {
                    String::from("<unknown>")
                }
            }
        }
    }
}
