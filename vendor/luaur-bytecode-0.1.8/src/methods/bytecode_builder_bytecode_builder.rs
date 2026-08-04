use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::bytecode_encoder::BytecodeEncoder;
use crate::records::constant_key::ConstantKey;
use crate::records::string_ref::StringRef;
use crate::records::table_shape::TableShape;
use alloc::string::String;
use alloc::vec::Vec;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl BytecodeBuilder {
    pub fn new(encoder: Option<*mut dyn BytecodeEncoder>) -> Self {
        let constant_map = DenseHashMap::new(ConstantKey {
            r#type: crate::enums::r#type::Type::Type_Nil,
            value: !0u64,
            extra: 0,
        });

        let table_shape_map = DenseHashMap::new(TableShape::default());

        let proto_map = DenseHashMap::new(!0u32);

        let string_table = DenseHashMap::new(StringRef {
            data: core::ptr::null(),
            length: 0,
        });

        let mut result = BytecodeBuilder {
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
            constant_map,
            table_shape_map,
            proto_map,
            debug_line: 0,
            debug_locals: Vec::new(),
            debug_upvals: Vec::new(),
            typed_locals: Vec::new(),
            typed_upvals: Vec::new(),
            userdata_types: Vec::new(),
            string_table,
            debug_strings: Vec::new(),
            debug_remarks: Vec::new(),
            debug_remark_buffer: String::new(),
            encoder,
            bytecode: String::new(),
            dump_flags: 0,
            dump_source: Vec::new(),
            dump_remarks: Vec::new(),
            temp_type_info: String::new(),
            dump_function_ptr: None,
        };

        // preallocate some buffers that are very likely to grow anyway; this works around std::vector's inefficient growth policy for small arrays
        result.insns.reserve(32);
        result.lines.reserve(32);
        result.constants.reserve(16);
        result.protos.reserve(16);
        result.functions.reserve(8);

        // LUAU_ASSERT(stringTable.find(StringRef{"", 0}) == nullptr);
        let empty_key = StringRef {
            data: b"\0".as_ptr() as *const i8,
            length: 0,
        };
        LUAU_ASSERT!(result.string_table.find(&empty_key).is_none());

        result
    }
}
