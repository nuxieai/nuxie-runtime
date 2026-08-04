use crate::enums::dump_flags::DumpFlags;
use crate::enums::r#type::Type;
use crate::records::bytecode_encoder::BytecodeEncoder;
use crate::records::class_shape::ClassShape;
use crate::records::constant::Constant;
use crate::records::constant_key::ConstantKey;
use crate::records::constant_key_hash::ConstantKeyHash;
use crate::records::debug_local_bytecode_builder::DebugLocal;
use crate::records::debug_upval::DebugUpval;
use crate::records::function::Function;
use crate::records::jump::Jump;
use crate::records::string_ref::StringRef;
use crate::records::string_ref_hash::StringRefHash;
use crate::records::table_shape::TableShape;
use crate::records::table_shape_hash::TableShapeHash;
use crate::records::typed_local_bytecode_builder::TypedLocal;
use crate::records::typed_upval::TypedUpval;
use crate::records::userdata_type::UserdataType;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_table::DenseEqDefault;
use luaur_common::type_aliases::dense_hash_default::DenseHashDefault;

#[derive(Debug, Clone)]
pub struct BytecodeBuilder {
    pub(crate) functions: Vec<Function>,
    pub(crate) current_function: u32,
    pub(crate) main_function: u32,

    pub(crate) total_instruction_count: usize,
    pub(crate) insns: Vec<u32>,
    pub(crate) lines: Vec<i32>,
    pub(crate) constants: Vec<Constant>,
    pub(crate) protos: Vec<u32>,
    pub(crate) jumps: Vec<Jump>,

    pub(crate) table_shapes: Vec<TableShape>,
    pub(crate) class_shapes: Vec<ClassShape>,

    pub(crate) fb_slots: Vec<u32>,

    pub(crate) has_long_jumps: bool,

    pub(crate) constant_map:
        DenseHashMap<ConstantKey, i32, DenseHashDefault<ConstantKey>, DenseEqDefault<ConstantKey>>,
    pub(crate) table_shape_map:
        DenseHashMap<TableShape, i32, DenseHashDefault<TableShape>, DenseEqDefault<TableShape>>,
    pub(crate) proto_map: DenseHashMap<u32, i16>,

    pub(crate) debug_line: i32,

    pub(crate) debug_locals: Vec<DebugLocal>,
    pub(crate) debug_upvals: Vec<DebugUpval>,

    pub(crate) typed_locals: Vec<TypedLocal>,
    pub(crate) typed_upvals: Vec<TypedUpval>,

    pub(crate) userdata_types: Vec<UserdataType>,

    pub(crate) string_table:
        DenseHashMap<StringRef, u32, DenseHashDefault<StringRef>, DenseEqDefault<StringRef>>,
    pub(crate) debug_strings: Vec<StringRef>,

    pub(crate) debug_remarks: Vec<(u32, u32)>,
    pub(crate) debug_remark_buffer: String,

    pub(crate) encoder: Option<*mut dyn BytecodeEncoder>,
    pub(crate) bytecode: String,

    pub(crate) dump_flags: u32,
    pub(crate) dump_source: Vec<String>,
    pub(crate) dump_remarks: Vec<(i32, String)>,

    pub(crate) temp_type_info: String,

    pub(crate) dump_function_ptr: Option<fn(&BytecodeBuilder, &mut Vec<i32>) -> String>,
}

impl BytecodeBuilder {
    pub fn get_string_hash(key: StringRef) -> u32 {
        crate::methods::bytecode_builder_get_string_hash::bytecode_builder_get_string_hash(key)
    }

    pub const DUMP_CODE: u32 = DumpFlags::Dump_Code as u32;
    pub const DUMP_LINES: u32 = DumpFlags::Dump_Lines as u32;
    pub const DUMP_SOURCE: u32 = DumpFlags::Dump_Source as u32;
    pub const DUMP_LOCALS: u32 = DumpFlags::Dump_Locals as u32;
    pub const DUMP_REMARKS: u32 = DumpFlags::Dump_Remarks as u32;
    pub const DUMP_TYPES: u32 = DumpFlags::Dump_Types as u32;
    pub const DUMP_CONSTANTS: u32 = DumpFlags::Dump_Constants as u32;
}

struct DummyEncoder;

impl BytecodeEncoder for DummyEncoder {
    fn encode(&mut self, _data: &mut [u32]) {}
}

impl Default for ConstantKey {
    fn default() -> Self {
        Self {
            r#type: Type::Type_Nil,
            value: 0,
            extra: 0,
        }
    }
}

impl Default for BytecodeBuilder {
    fn default() -> Self {
        Self {
            functions: Vec::new(),
            current_function: !0u32,
            main_function: !0u32,
            total_instruction_count: 0,
            insns: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
            protos: Vec::new(),
            jumps: Vec::new(),
            table_shapes: Vec::new(),
            class_shapes: Vec::new(),
            fb_slots: Vec::new(),
            has_long_jumps: false,
            constant_map: DenseHashMap::new(ConstantKey {
                r#type: Type::Type_Nil,
                value: 0,
                extra: 0,
            }),
            table_shape_map: DenseHashMap::new(TableShape {
                keys: [0; 32],
                constants: [-1; 32],
                length: 0,
                hasConstants: false,
            }),
            proto_map: DenseHashMap::new(!0u32),
            debug_line: 0,
            debug_locals: Vec::new(),
            debug_upvals: Vec::new(),
            typed_locals: Vec::new(),
            typed_upvals: Vec::new(),
            userdata_types: Vec::new(),
            string_table: DenseHashMap::new(StringRef::default()),
            debug_strings: Vec::new(),
            debug_remarks: Vec::new(),
            debug_remark_buffer: String::new(),
            encoder: None,
            bytecode: String::new(),
            dump_flags: 0,
            dump_source: Vec::new(),
            dump_remarks: Vec::new(),
            temp_type_info: String::new(),
            dump_function_ptr: None,
        }
    }
}
