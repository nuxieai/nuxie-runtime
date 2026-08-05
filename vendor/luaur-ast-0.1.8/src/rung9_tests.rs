use crate::records::allocator::Allocator;
use crate::records::ast_name_table::AstNameTable;
use crate::records::lexeme::Type;
use crate::records::parse_options::ParseOptions;
use crate::records::parser::Parser;

#[test]
fn parser_parse_captures_lexemes_independently_of_comment_locations() {
    let source = "-- note\nlocal value = 42";
    let mut allocator = Allocator::allocator();
    let mut names = AstNameTable::new(&mut allocator);
    let result = Parser::parse(
        source,
        source.len(),
        &mut names,
        &mut allocator,
        ParseOptions::default(),
    );

    assert!(result.errors.is_empty());
    assert!(result.comment_locations.is_empty());
    assert_eq!(
        result.lexemes.first().map(|lexeme| lexeme.r#type),
        Some(Type::Comment)
    );
    assert!(result
        .lexemes
        .iter()
        .any(|lexeme| lexeme.r#type == Type::ReservedLocal));
    assert_eq!(
        result.lexemes.last().map(|lexeme| lexeme.r#type),
        Some(Type::Eof)
    );
}

#[test]
fn parser_parse_fatal_result_defaults_lexemes_empty() {
    let source = "local =\n".repeat(200);
    let mut allocator = Allocator::allocator();
    let mut names = AstNameTable::new(&mut allocator);
    let result = Parser::parse(
        &source,
        source.len(),
        &mut names,
        &mut allocator,
        ParseOptions::default(),
    );

    assert!(result.root.is_null());
    assert!(!result.errors.is_empty());
    assert!(result.lexemes.is_empty());
}
