#[allow(non_snake_case)]
macro_rules! VM_CONTINUE {
    ($op:expr) => {
        return $op
    };
}

pub(crate) use VM_CONTINUE;
