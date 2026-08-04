#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuoteStyle {
    QuotedSingle,
    QuotedDouble,
    QuotedRaw,
    QuotedInterp,
}
