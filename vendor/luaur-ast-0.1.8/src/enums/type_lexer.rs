//! `Lexeme::Type` (`Ast/include/Luau/Lexer.h`).
//!
//! Faithful port. In Luau the token type is a plain integer: values `1..255`
//! are literal character codes (so `'+'`, `'-'`, `'<'` are valid token types),
//! and the named multi-character tokens begin at `Char_END = 256`. A fieldless
//! Rust enum cannot represent the single-character values, so `Type` is a
//! newtype over `i32` (the C++ enum's underlying type) with associated consts.
//! `Type::Equal`-style paths still resolve, a single-char token is `Type(c)`,
//! and the derived `Ord` matches the C++ `<`/`>=` range checks against
//! `Char_END` / `Reserved_BEGIN`.

#[allow(non_camel_case_types)]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Type(pub i32);

#[allow(non_upper_case_globals)]
impl Type {
    pub const Eof: Type = Type(0);

    // 1..255 means actual character values
    pub const Semicolon: Type = Type(b';' as i32);
    pub const Less: Type = Type(b'<' as i32);
    pub const Greater: Type = Type(b'>' as i32);
    pub const Pipe: Type = Type(b'|' as i32);
    pub const Question: Type = Type(b'?' as i32);
    pub const Ampersand: Type = Type(b'&' as i32);
    pub const Char_OPEN: Type = Type(b'(' as i32);
    pub const Char_PIPE: Type = Type(b'|' as i32);
    pub const Char_AMPERSAND: Type = Type(b'&' as i32);
    pub const Char_COMMA: Type = Type(b',' as i32);
    pub const Colon: Type = Type(b':' as i32);
    pub const Comma: Type = Type(b',' as i32);
    pub const Operator: Type = Type(b'=' as i32);
    pub const Char_END: Type = Type(256);

    pub const Equal: Type = Type(257);
    pub const LessEqual: Type = Type(258);
    pub const GreaterEqual: Type = Type(259);
    pub const NotEqual: Type = Type(260);
    pub const Dot2: Type = Type(261);
    pub const Dot3: Type = Type(262);
    pub const SkinnyArrow: Type = Type(263);
    pub const DoubleColon: Type = Type(264);
    pub const FloorDiv: Type = Type(265);

    pub const InterpStringBegin: Type = Type(266);
    pub const InterpStringMid: Type = Type(267);
    pub const InterpStringEnd: Type = Type(268);
    // An interpolated string with no expressions (like `x`)
    pub const InterpStringSimple: Type = Type(269);

    pub const AddAssign: Type = Type(270);
    pub const SubAssign: Type = Type(271);
    pub const MulAssign: Type = Type(272);
    pub const DivAssign: Type = Type(273);
    pub const FloorDivAssign: Type = Type(274);
    pub const ModAssign: Type = Type(275);
    pub const PowAssign: Type = Type(276);
    pub const ConcatAssign: Type = Type(277);

    pub const RawString: Type = Type(278);
    pub const QuotedString: Type = Type(279);
    pub const Number: Type = Type(280);
    pub const Name: Type = Type(281);

    pub const Comment: Type = Type(282);
    pub const BlockComment: Type = Type(283);

    pub const Attribute: Type = Type(284);
    pub const AttributeOpen: Type = Type(285);

    pub const BrokenString: Type = Type(286);
    pub const BrokenComment: Type = Type(287);
    pub const BrokenUnicode: Type = Type(288);
    pub const BrokenInterpDoubleBrace: Type = Type(289);
    pub const Error: Type = Type(290);

    pub const Reserved_BEGIN: Type = Type(291);
    pub const ReservedAnd: Type = Type::Reserved_BEGIN; // = 291
    pub const ReservedBreak: Type = Type(292);
    pub const ReservedDo: Type = Type(293);
    pub const ReservedElse: Type = Type(294);
    pub const ReservedElseif: Type = Type(295);
    pub const ReservedEnd: Type = Type(296);
    pub const ReservedFalse: Type = Type(297);
    pub const ReservedFor: Type = Type(298);
    pub const ReservedFunction: Type = Type(299);
    pub const ReservedIf: Type = Type(300);
    pub const ReservedIn: Type = Type(301);
    pub const ReservedLocal: Type = Type(302);
    pub const ReservedNil: Type = Type(303);
    pub const ReservedNot: Type = Type(304);
    pub const ReservedOr: Type = Type(305);
    pub const ReservedRepeat: Type = Type(306);
    pub const ReservedReturn: Type = Type(307);
    pub const ReservedThen: Type = Type(308);
    pub const ReservedTrue: Type = Type(309);
    pub const ReservedUntil: Type = Type(310);
    pub const ReservedWhile: Type = Type(311);
    pub const Reserved_END: Type = Type(312);
}
