use crate::type_aliases::op_constness::OpConstness;

#[derive(Debug, Default)]
pub struct SccpState {
    pub op_constness: OpConstness,
}
