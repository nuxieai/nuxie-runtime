#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BcOpKind {
    None,
    /// To reference a immediate value
    Imm,
    /// To reference a result of a previous instruction
    Inst,
    /// To reference a basic block in control flow
    Block,
    /// Phi operand
    Phi,
    /// Projection of multireturn call or variadic arguments
    Proj,
    /// To reference a VM register
    VmReg,
    /// To reference a VM constant
    VmConst,
    /// To reference a VM upvalue
    VmUpvalue,
    /// To reference a VM upvalue
    VmProto,
}
