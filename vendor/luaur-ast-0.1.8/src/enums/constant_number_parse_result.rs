#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConstantNumberParseResult {
    Ok,
    Imprecise,
    Malformed,
    BinOverflow,
    HexOverflow,
    IntOverflow,
}
