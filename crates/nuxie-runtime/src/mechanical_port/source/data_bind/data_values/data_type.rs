#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DataType {
    None = 0,
    String = 1,
    Number = 2,
    Boolean = 3,
    Color = 4,
    List = 5,
    Enum = 6,
    Trigger = 7,
    ViewModel = 8,
    Integer = 9,
    SymbolListIndex = 10,
    AssetImage = 11,
    Artboard = 12,
    AssetFont = 13,
    AssetBlob = 14,
    Input = 99,
    Any = 100,
}
