//! Inline port of the upstream Luau lexer unit tests (`luau/tests/Lexer.test.cpp`,
//! `TEST_SUITE("LexerTests")`). Each C++ `TEST_CASE` becomes a Rust `#[test]`.
//! These are collocated unit tests for the hand-ported lexer; they exercise only
//! the lexer (no Analysis dependency) and run under `cargo test -p luau-ast`.
//!
//! API mapping vs C++:
//! - `Lexer lexer(input, size, table)` -> `Lexer::new(ptr, size, &mut names, Position{0,0})`
//! - `lexeme.type` -> `lexeme.r#type` (a `Type(i32)` newtype; char tokens are `Type(c)`)
//! - `lexeme.location` -> `lexeme.location` (`Location{begin,end}`)
//! - `lexeme.getBlockDepth()` -> `lexeme.getBlock_depth()` (translated name)
//! - `lexeme.data`/`getLength()` -> `lexeme.data.data`/`get_length()`
//! - `lexeme.name` -> `lexeme.data.name` (union arm, read as a C string)

#![cfg(test)]
#![allow(non_snake_case)]

use core::ffi::{c_char, CStr};

use crate::enums::type_lexer::Type;
use crate::records::allocator::Allocator;
use crate::records::ast_name_table::AstNameTable;
use crate::records::lexeme::{Lexeme, QuoteStyle};
use crate::records::lexer::Lexer;
use crate::records::location::Location;
use crate::records::position::Position;

/// Set up a lexer over `input` (NUL-terminated like `std::string::c_str()`, with
/// `size` excluding the terminator) and run `f`. The backing buffer and name
/// table outlive the closure, so token data/name pointers stay valid inside it.
fn with_lexer<R>(input: &[u8], f: impl FnOnce(&mut Lexer) -> R) -> R {
    let mut buf = input.to_vec();
    buf.push(0);
    let size = input.len();
    let mut allocator = Allocator::allocator();
    let mut names = AstNameTable::new(&mut allocator);
    let mut lexer = Lexer::new(
        buf.as_ptr() as *const c_char,
        size,
        &mut names,
        Position { line: 0, column: 0 },
    );
    f(&mut lexer)
}

fn loc(bl: u32, bc: u32, el: u32, ec: u32) -> Location {
    Location {
        begin: Position {
            line: bl,
            column: bc,
        },
        end: Position {
            line: el,
            column: ec,
        },
    }
}

/// A single-character token's type (`':'`, `'{'`, ...): the char's code point.
fn ch(c: u8) -> Type {
    Type(c as i32)
}

/// The interned name of a `Name`/reserved lexeme (`Lexeme::name` in C++).
fn name_str(lx: &Lexeme) -> String {
    let ptr = unsafe { lx.data.name };
    if ptr.is_null() {
        return String::new();
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

/// The raw payload bytes of a data-bearing lexeme (`std::string(lexeme.data,
/// lexeme.getLength())` in C++).
fn data_bytes(lx: &Lexeme) -> Vec<u8> {
    let len = lx.get_length() as usize;
    let ptr = unsafe { lx.data.data } as *const u8;
    unsafe { core::slice::from_raw_parts(ptr, len) }.to_vec()
}

#[test]
fn broken_string_works() {
    with_lexer(b"[[", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::BrokenString);
        assert_eq!(lexeme.location, loc(0, 0, 0, 2));
    });
}

#[test]
fn broken_comment() {
    with_lexer(b"--[[  ", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::BrokenComment);
        assert_eq!(lexeme.location, loc(0, 0, 0, 6));
    });
}

#[test]
fn broken_comment_kept() {
    with_lexer(b"--[[  ", |lexer| {
        lexer.set_skip_comments(true);
        assert_eq!(lexer.next().r#type, Type::BrokenComment);
    });
}

#[test]
fn comment_skipped() {
    with_lexer(b"--  ", |lexer| {
        lexer.set_skip_comments(true);
        assert_eq!(lexer.next().r#type, Type::Eof);
    });
}

#[test]
fn multiline_comment_with_lexeme_in_and_after() {
    with_lexer(b"--[[ function \n]] end", |lexer| {
        let comment = *lexer.next();
        let end = *lexer.next();
        assert_eq!(comment.r#type, Type::BlockComment);
        assert_eq!(comment.location, loc(0, 0, 1, 2));
        assert_eq!(end.r#type, Type::ReservedEnd);
        assert_eq!(end.location, loc(1, 3, 1, 6));
    });
}

#[test]
fn test_broken_escape_tolerant() {
    let input = br"'\3729472897292378'";
    with_lexer(input, |lexer| {
        let item = *lexer.next();
        assert_eq!(item.r#type, Type::QuotedString);
        assert_eq!(item.location, loc(0, 0, 0, input.len() as u32));
    });
}

#[test]
fn test_big_delimiters() {
    with_lexer(b"--[===[\n\n\n\n]===]", |lexer| {
        let item = *lexer.next();
        assert_eq!(item.r#type, Type::BlockComment);
        assert_eq!(item.location, loc(0, 0, 4, 5));
    });
}

#[test]
fn lookahead() {
    with_lexer(b"foo --[[ comment ]] bar : nil end", |lexer| {
        lexer.set_skip_comments(true);
        lexer.next(); // must call next() before reading data at least once

        let cur = *lexer.current();
        assert_eq!(cur.r#type, Type::Name);
        assert_eq!(name_str(&cur), "foo");
        let la = lexer.lookahead();
        assert_eq!(la.r#type, Type::Name);
        assert_eq!(name_str(&la), "bar");

        lexer.next();
        let cur = *lexer.current();
        assert_eq!(cur.r#type, Type::Name);
        assert_eq!(name_str(&cur), "bar");
        assert_eq!(lexer.lookahead().r#type, ch(b':'));

        lexer.next();
        assert_eq!(lexer.current().r#type, ch(b':'));
        assert_eq!(lexer.lookahead().r#type, Type::ReservedNil);

        lexer.next();
        assert_eq!(lexer.current().r#type, Type::ReservedNil);
        assert_eq!(lexer.lookahead().r#type, Type::ReservedEnd);

        lexer.next();
        assert_eq!(lexer.current().r#type, Type::ReservedEnd);
        assert_eq!(lexer.lookahead().r#type, Type::Eof);

        lexer.next();
        assert_eq!(lexer.current().r#type, Type::Eof);
        assert_eq!(lexer.lookahead().r#type, Type::Eof);
    });
}

#[test]
fn string_interpolation_basic() {
    with_lexer(br#"`foo {"bar"}`"#, |lexer| {
        assert_eq!(lexer.next().r#type, Type::InterpStringBegin);
        assert_eq!(lexer.next().r#type, Type::QuotedString);
        let interp_end = *lexer.next();
        assert_eq!(interp_end.r#type, Type::InterpStringEnd);
        // The InterpStringEnd should start with }, not `.
        assert_eq!(interp_end.location.begin.column, 11);
    });
}

#[test]
fn string_interpolation_full() {
    with_lexer(br#"`foo {"bar"} {"baz"} end`"#, |lexer| {
        let interp_begin = *lexer.next();
        assert_eq!(interp_begin.r#type, Type::InterpStringBegin);
        assert_eq!(interp_begin.to_string(), "`foo {");

        let quote1 = *lexer.next();
        assert_eq!(quote1.r#type, Type::QuotedString);
        assert_eq!(quote1.to_string(), "\"bar\"");

        let interp_mid = *lexer.next();
        assert_eq!(interp_mid.r#type, Type::InterpStringMid);
        assert_eq!(interp_mid.to_string(), "} {");
        assert_eq!(interp_mid.location.begin.column, 11);

        let quote2 = *lexer.next();
        assert_eq!(quote2.r#type, Type::QuotedString);
        assert_eq!(quote2.to_string(), "\"baz\"");

        let interp_end = *lexer.next();
        assert_eq!(interp_end.r#type, Type::InterpStringEnd);
        assert_eq!(interp_end.to_string(), "} end`");
        assert_eq!(interp_end.location.begin.column, 19);
    });
}

#[test]
fn string_interpolation_double_brace() {
    with_lexer(br#"`foo{{bad}}bar`"#, |lexer| {
        let broken_interp_begin = *lexer.next();
        assert_eq!(broken_interp_begin.r#type, Type::BrokenInterpDoubleBrace);
        assert_eq!(data_bytes(&broken_interp_begin), b"foo");

        assert_eq!(lexer.next().r#type, Type::Name);

        let interp_end = *lexer.next();
        assert_eq!(interp_end.r#type, Type::InterpStringEnd);
        assert_eq!(data_bytes(&interp_end), b"}bar");
    });
}

#[test]
fn string_interpolation_double_but_unmatched_brace() {
    with_lexer(br#"`{{oops}`, 1"#, |lexer| {
        assert_eq!(lexer.next().r#type, Type::BrokenInterpDoubleBrace);
        assert_eq!(lexer.next().r#type, Type::Name);
        assert_eq!(lexer.next().r#type, Type::InterpStringEnd);
        assert_eq!(lexer.next().r#type, ch(b','));
        assert_eq!(lexer.next().r#type, Type::Number);
    });
}

#[test]
fn string_interpolation_unmatched_brace() {
    let input =
        b"{\n        `hello {\"world\"}\n    } -- this might be incorrectly parsed as a string";
    with_lexer(input, |lexer| {
        assert_eq!(lexer.next().r#type, ch(b'{'));
        assert_eq!(lexer.next().r#type, Type::InterpStringBegin);
        assert_eq!(lexer.next().r#type, Type::QuotedString);
        assert_eq!(lexer.next().r#type, Type::BrokenString);
        assert_eq!(lexer.next().r#type, ch(b'}'));
    });
}

#[test]
fn string_interpolation_with_unicode_escape() {
    with_lexer(br"`\u{1F41B}`", |lexer| {
        assert_eq!(lexer.next().r#type, Type::InterpStringSimple);
        assert_eq!(lexer.next().r#type, Type::Eof);
    });
}

#[test]
fn single_quoted_string() {
    with_lexer(b"'test'", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::QuotedString);
        assert_eq!(lexeme.get_quote_style(), QuoteStyle::Single);
    });
}

#[test]
fn double_quoted_string() {
    with_lexer(b"\"test\"", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::QuotedString);
        assert_eq!(lexeme.get_quote_style(), QuoteStyle::Double);
    });
}

#[test]
fn lexer_determines_string_block_depth_0() {
    with_lexer(b"[[ test ]]", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::RawString);
        assert_eq!(lexeme.getBlock_depth(), 0);
    });
}

#[test]
fn lexer_determines_string_block_depth_0_multiline_1() {
    with_lexer(b"[[ test\n    ]]", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::RawString);
        assert_eq!(lexeme.getBlock_depth(), 0);
    });
}

#[test]
fn lexer_determines_string_block_depth_0_multiline_2() {
    with_lexer(b"[[\n    test\n    ]]", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::RawString);
        assert_eq!(lexeme.getBlock_depth(), 0);
    });
}

#[test]
fn lexer_determines_string_block_depth_0_multiline_3() {
    with_lexer(b"[[\n    test ]]", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::RawString);
        assert_eq!(lexeme.getBlock_depth(), 0);
    });
}

#[test]
fn lexer_determines_string_block_depth_1() {
    with_lexer(b"[=[[%s]]=]", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::RawString);
        assert_eq!(lexeme.getBlock_depth(), 1);
    });
}

#[test]
fn lexer_determines_string_block_depth_2() {
    with_lexer(b"[==[ test ]==]", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::RawString);
        assert_eq!(lexeme.getBlock_depth(), 2);
    });
}

#[test]
fn lexer_determines_string_block_depth_2_multiline_1() {
    with_lexer(b"[==[ test\n    ]==]", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::RawString);
        assert_eq!(lexeme.getBlock_depth(), 2);
    });
}

#[test]
fn lexer_determines_string_block_depth_2_multiline_2() {
    with_lexer(b"[==[\n    test\n    ]==]", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::RawString);
        assert_eq!(lexeme.getBlock_depth(), 2);
    });
}

#[test]
fn lexer_determines_string_block_depth_2_multiline_3() {
    with_lexer(b"[==[\n\n    test ]==]", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::RawString);
        assert_eq!(lexeme.getBlock_depth(), 2);
    });
}

#[test]
fn lexer_determines_comment_block_depth_0() {
    with_lexer(b"--[[ test ]]", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::BlockComment);
        assert_eq!(lexeme.getBlock_depth(), 0);
    });
}

// C++ duplicates the name `lexer_determines_string_block_depth_1`/`_2` for these
// two comment cases; renamed here to keep Rust test names unique and accurate.
// Node: `cxx:Test:Luau.UnitTest:tests/Lexer.test.cpp:406:lexer_determines_string_block_depth_1`
#[test]
fn lexer_determines_comment_block_depth_1() {
    with_lexer("--[=[ μέλλον ]=]".as_bytes(), |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::BlockComment);
        assert_eq!(lexeme.getBlock_depth(), 1);
    });
}

// Node: `cxx:Test:Luau.UnitTest:tests/Lexer.test.cpp:418:lexer_determines_string_block_depth_2`
#[test]
fn lexer_determines_comment_block_depth_2() {
    with_lexer(b"--[==[ test ]==]", |lexer| {
        let lexeme = *lexer.next();
        assert_eq!(lexeme.r#type, Type::BlockComment);
        assert_eq!(lexeme.getBlock_depth(), 2);
    });
}
