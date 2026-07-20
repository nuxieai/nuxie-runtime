#[allow(non_snake_case)]
macro_rules! l_noret {
    ($declaration:item) => {
        #[noreturn]
        $declaration
    };
}

pub(crate) use l_noret;
