// Declarative macros must use the stable `macro_rules!`, not the unstable
// `macro` keyword (decl-macro 2.0). Expands, in item position, to the per-class
// `CstClassIndex` accessor the C++ `LUAU_CST_RTTI(Class)` macro injects.
#[macro_export]
macro_rules! LUAU_CST_RTTI {
    ($Class:ty) => {
        #[allow(non_snake_case)]
        pub(crate) fn CstClassIndex() -> i32 {
            <$crate::records::cst_rtti::CstRtti<$Class>>::value
        }
    };
}
