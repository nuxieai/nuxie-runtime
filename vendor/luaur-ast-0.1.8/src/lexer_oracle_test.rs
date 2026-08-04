//! Cross-check the hand-ported Luau lexer against an independent oracle: a token
//! dump produced by the real Luau C++ lexer (`luau/Ast/src/Lexer.cpp`) built
//! standalone. Both emit `<typeInt>|<bL>|<bC>|<eL>|<eC>|<payloadHex>` per token;
//! the golden files are the C++ oracle's output for the `testdata/*.luau`
//! fixtures.

#![cfg(test)]

use crate::records::allocator::Allocator;
use crate::records::ast_name_table::AstNameTable;
use crate::records::lexeme::Type;
use crate::records::lexer::Lexer;
use crate::records::position::Position;

fn is_data_type(t: Type) -> bool {
    t == Type::RawString
        || t == Type::QuotedString
        || t == Type::InterpStringBegin
        || t == Type::InterpStringMid
        || t == Type::InterpStringEnd
        || t == Type::InterpStringSimple
        || t == Type::BrokenInterpDoubleBrace
        || t == Type::Number
        || t == Type::Comment
        || t == Type::BlockComment
}

fn is_name_type(t: Type) -> bool {
    t == Type::Name || t == Type::Attribute || (t >= Type::Reserved_BEGIN && t < Type::Reserved_END)
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Tokenize `fixture` and produce the same `<type>|<bL>|<bC>|<eL>|<eC>|<hex>`
/// dump the C++ oracle emits.
fn dump(fixture: &[u8]) -> String {
    // Mirror std::string semantics: NUL terminator present at [size] but not
    // counted in buffer_size.
    let mut buf = fixture.to_vec();
    buf.push(0);
    let size = fixture.len();

    let mut allocator = Allocator::allocator();
    let mut names = AstNameTable::new(&mut allocator);
    let mut lexer = Lexer::new(
        buf.as_ptr() as *const core::ffi::c_char,
        size,
        &mut names,
        Position { line: 0, column: 0 },
    );

    let mut out = String::new();
    loop {
        let lex = *lexer.next();
        let t = lex.r#type;
        out.push_str(&format!(
            "{}|{}|{}|{}|{}|",
            t.0,
            lex.location.begin.line,
            lex.location.begin.column,
            lex.location.end.line,
            lex.location.end.column,
        ));

        if is_data_type(t) {
            let ptr = unsafe { lex.data.data } as *const u8;
            let len = lex.get_length() as usize;
            let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };
            out.push_str(&hex(bytes));
        } else if is_name_type(t) {
            let ptr = unsafe { lex.data.name };
            if !ptr.is_null() {
                let bytes = unsafe { core::ffi::CStr::from_ptr(ptr).to_bytes() };
                out.push_str(&hex(bytes));
            }
        }

        out.push('\n');

        if t == Type::Eof {
            break;
        }
    }

    out
}

#[test]
fn lexer_matches_cpp_oracle() {
    let golden = include_str!("testdata/lex_fixture.golden");
    assert_eq!(
        dump(include_bytes!("testdata/lex_fixture.luau")),
        golden,
        "lexer token dump diverged from C++ oracle (main fixture)"
    );
}

#[test]
fn lexer_matches_cpp_oracle_edge_cases() {
    let golden = include_str!("testdata/lex_edge.golden");
    assert_eq!(
        dump(include_bytes!("testdata/lex_edge.luau")),
        golden,
        "lexer token dump diverged from C++ oracle (edge fixture)"
    );
}
