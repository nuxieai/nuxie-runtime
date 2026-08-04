use crate::records::bc_block::BcBlock;
use crate::records::bc_imm::BcImm;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bc_phi::BcPhi;
use crate::records::bc_proj::BcProj;
use crate::records::bc_vm_const::BcVmConst;
use crate::records::debug_local_bytecode_graph::DebugLocal;
use crate::records::table_shape::TableShape;
use crate::records::typed_local_bytecode_graph::TypedLocal;
use crate::type_aliases::reg_map::RegMap;
use alloc::string::String;
use alloc::vec::Vec;
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;

pub type VmConst = BcVmConst;

#[derive(Debug, Clone)]
pub struct BcFunction {
    pub maxstacksize: u8,
    pub numparams: u8,
    pub nups: u8,
    pub is_vararg: bool,
    pub flags: u8,

    pub blocks: Vec<BcBlock>,
    pub instructions: Vec<BcInst>,
    pub constants: Vec<VmConst>,
    pub immediates: Vec<BcImm>,
    pub phis: Vec<BcPhi>,
    pub projections: Vec<BcProj>,
    pub table_shapes: Vec<TableShape>,

    pub entry_block: BcOp,
    pub exit_block: BcOp,

    pub type_info: String,
    pub upvalue_types: Vec<LuauBytecodeType>,
    pub local_types: Vec<TypedLocal>,
    pub protos: Vec<u32>,

    pub debugname: String,
    pub linedefined: u32,
    pub upvalue_names: Vec<String>,
    pub locals: Vec<DebugLocal<'static>>,

    pub regs: RegMap,
}

impl Default for BcFunction {
    fn default() -> Self {
        Self {
            maxstacksize: 0,
            numparams: 0,
            nups: 0,
            is_vararg: false,
            flags: 0,
            blocks: Vec::new(),
            instructions: Vec::new(),
            constants: Vec::new(),
            immediates: Vec::new(),
            phis: Vec::new(),
            projections: Vec::new(),
            table_shapes: Vec::new(),
            entry_block: BcOp::new(),
            exit_block: BcOp::new(),
            type_info: String::new(),
            upvalue_types: Vec::new(),
            local_types: Vec::new(),
            protos: Vec::new(),
            debugname: String::new(),
            linedefined: 0,
            upvalue_names: Vec::new(),
            locals: Vec::new(),
            regs: RegMap::default(),
        }
    }
}
