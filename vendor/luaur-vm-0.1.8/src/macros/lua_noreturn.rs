#[allow(non_snake_case)]
macro_rules! LUA_NORETURN {
    ($declaration:item) => {
        #[noreturn]
        $declaration
    };
}

pub(crate) use LUA_NORETURN;
