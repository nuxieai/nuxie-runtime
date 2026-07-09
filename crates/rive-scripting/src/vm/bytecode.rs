use std::{error::Error, fmt};

use luaur_common::{
    FFlag,
    enums::{
        luau_bytecode_tag::{
            LBC_CONSTANT_BOOLEAN, LBC_CONSTANT_CLASS_SHAPE, LBC_CONSTANT_CLOSURE,
            LBC_CONSTANT_IMPORT, LBC_CONSTANT_INTEGER, LBC_CONSTANT_NIL, LBC_CONSTANT_NUMBER,
            LBC_CONSTANT_STRING, LBC_CONSTANT_TABLE, LBC_CONSTANT_TABLE_WITH_CONSTANTS,
            LBC_CONSTANT_VECTOR, LBC_TYPE_VERSION_MAX, LBC_TYPE_VERSION_MIN, LBC_VERSION_MAX,
            LBC_VERSION_MIN,
        },
        luau_bytecode_type::LBC_TYPE_FUNCTION,
        luau_feedback_type::LuauFeedbackType,
        luau_opcode::LuauOpcode,
    },
    functions::get_op_length::getOpLength,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BytecodeValidationError {
    message: String,
}

impl BytecodeValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for BytecodeValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for BytecodeValidationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstantKind {
    Nil,
    Boolean,
    Number,
    String,
    Import,
    Table,
    Closure,
    Vector,
    Integer,
    ClassShape,
}

pub fn validate_luau_bytecode(bytecode: &[u8]) -> Result<(), BytecodeValidationError> {
    let mut cursor = Cursor::new(bytecode);
    let version = cursor.read_u8("bytecode version")?;

    if version == 0 {
        return Ok(());
    }

    let min_version = LBC_VERSION_MIN.0 as u8;
    let max_version = LBC_VERSION_MAX.0 as u8;
    if !(min_version..=max_version).contains(&version) {
        return Ok(());
    }

    let mut types_version = 0;
    if version >= 4 {
        types_version = cursor.read_u8("bytecode type version")?;
        let min_type_version = LBC_TYPE_VERSION_MIN.0 as u8;
        let max_type_version = LBC_TYPE_VERSION_MAX.0 as u8;
        if !(min_type_version..=max_type_version).contains(&types_version) {
            return Ok(());
        }
    }

    let string_count = cursor.read_count("string count", 1)?;
    for _ in 0..string_count {
        let length = cursor.read_count("string length", 1)?;
        cursor.skip(length, "string bytes")?;
    }

    if types_version == 3 {
        loop {
            let index = cursor.read_u8("userdata type remap index")?;
            if index == 0 {
                break;
            }
            cursor.read_string_id(string_count, "userdata type remap name", true)?;
        }
    }

    let proto_count = cursor.read_count("proto count", 1)?;
    if proto_count == 0 {
        return Err(BytecodeValidationError::new(
            "proto table is empty; main proto would be out of bounds",
        ));
    }

    for proto_index in 0..proto_count {
        validate_proto(
            &mut cursor,
            version,
            types_version,
            string_count,
            proto_count,
            proto_index,
        )?;
    }

    let main_id = cursor.read_count("main proto id", 0)?;
    if main_id >= proto_count {
        return Err(BytecodeValidationError::new(format!(
            "main proto id {main_id} out of bounds for {proto_count} protos"
        )));
    }

    Ok(())
}

fn validate_proto(
    cursor: &mut Cursor<'_>,
    version: u8,
    types_version: u8,
    string_count: usize,
    proto_count: usize,
    proto_index: usize,
) -> Result<(), BytecodeValidationError> {
    cursor.read_u8("proto maxstacksize")?;
    let num_params = cursor.read_u8("proto numparams")?;
    let nups = cursor.read_u8("proto nups")?;
    cursor.read_u8("proto is_vararg")?;

    if version >= 4 {
        cursor.read_u8("proto flags")?;
        match types_version {
            1 => {
                let type_size = cursor.read_count("typeinfo size", 1)?;
                if type_size != 0 {
                    if type_size < 2 {
                        return Err(BytecodeValidationError::new(
                            "v1 typeinfo is too short for function header",
                        ));
                    }
                    let typeinfo = cursor.peek(type_size, "typeinfo bytes")?;
                    let expected = 2usize + usize::from(num_params);
                    if type_size != expected {
                        return Err(BytecodeValidationError::new(format!(
                            "v1 typeinfo size {type_size} does not match numparams {num_params}"
                        )));
                    }
                    if typeinfo[0] != LBC_TYPE_FUNCTION.0 as u8 || typeinfo[1] != num_params {
                        return Err(BytecodeValidationError::new(
                            "v1 typeinfo function header mismatch",
                        ));
                    }
                    cursor.skip(type_size, "typeinfo bytes")?;
                }
            }
            2 | 3 => {
                let type_size = cursor.read_count("typeinfo size", 1)?;
                cursor.skip(type_size, "typeinfo bytes")?;
            }
            _ => {}
        }
    }

    let size_code = cursor.read_count("instruction count", 4)?;
    let mut code = Vec::with_capacity(size_code);
    for _ in 0..size_code {
        code.push(cursor.read_u32("instruction")?);
    }

    let size_k = cursor.read_count("constant count", 1)?;
    let mut constants = Vec::with_capacity(size_k);
    for _ in 0..size_k {
        constants.push(validate_constant(
            cursor,
            string_count,
            proto_count,
            proto_index,
            size_k,
            &constants,
        )?);
    }

    validate_instruction_stream(&code, &constants)?;

    let size_p = cursor.read_count("child proto count", 1)?;
    for _ in 0..size_p {
        let fid = cursor.read_count("child proto id", 0)?;
        validate_prior_proto(fid, proto_count, proto_index, "child proto id")?;
    }

    cursor.read_var_u32("line defined")?;
    cursor.read_string_id(string_count, "debug name", false)?;

    let lineinfo = cursor.read_u8("lineinfo flag")?;
    if lineinfo != 0 {
        let line_gap_log2 = cursor.read_u8("linegaplog2")?;
        let intervals = if size_code == 0 {
            0
        } else {
            if usize::from(line_gap_log2) >= usize::BITS as usize {
                return Err(BytecodeValidationError::new(format!(
                    "linegaplog2 {line_gap_log2} is too large"
                )));
            }
            ((size_code - 1) >> usize::from(line_gap_log2)) + 1
        };
        let abs_offset = size_code
            .checked_add(3)
            .ok_or_else(|| BytecodeValidationError::new("lineinfo size overflow"))?
            & !3;
        cursor.skip(size_code, "lineinfo byte deltas")?;
        cursor.skip(
            intervals
                .checked_mul(4)
                .ok_or_else(|| BytecodeValidationError::new("abslineinfo size overflow"))?,
            "absolute lineinfo",
        )?;
        let _ = abs_offset;
    }

    let debuginfo = cursor.read_u8("debuginfo flag")?;
    if debuginfo != 0 {
        let size_locvars = cursor.read_count("local variable count", 1)?;
        for _ in 0..size_locvars {
            cursor.read_string_id(string_count, "local variable name", false)?;
            cursor.read_var_u32("local variable start pc")?;
            cursor.read_var_u32("local variable end pc")?;
            cursor.read_u8("local variable register")?;
        }

        let size_upvalues = cursor.read_count("upvalue name count", 1)?;
        if size_upvalues != usize::from(nups) {
            return Err(BytecodeValidationError::new(format!(
                "upvalue name count {size_upvalues} does not match nups {nups}"
            )));
        }
        for _ in 0..size_upvalues {
            cursor.read_string_id(string_count, "upvalue name", false)?;
        }
    }

    if version >= 11 {
        if !FFlag::LuauCallFeedback.get() {
            return Err(BytecodeValidationError::new(
                "bytecode v11 requires LuauCallFeedback flag",
            ));
        }
        let feedback_slots = cursor.read_count("feedback slot count", 1)?;
        for _ in 0..feedback_slots {
            let slot_type = cursor.read_u8("feedback slot type")?;
            if slot_type != LuauFeedbackType::LFT_CALLTARGET as u8 {
                return Err(BytecodeValidationError::new(format!(
                    "unsupported feedback slot type {slot_type}"
                )));
            }
            cursor.read_var_u32("feedback call target pc")?;
        }
    }

    Ok(())
}

fn validate_constant(
    cursor: &mut Cursor<'_>,
    string_count: usize,
    proto_count: usize,
    proto_index: usize,
    size_k: usize,
    previous_constants: &[ConstantKind],
) -> Result<ConstantKind, BytecodeValidationError> {
    let tag = cursor.read_u8("constant tag")?;
    match tag {
        tag if tag == LBC_CONSTANT_NIL.0 as u8 => Ok(ConstantKind::Nil),
        tag if tag == LBC_CONSTANT_BOOLEAN.0 as u8 => {
            cursor.read_u8("boolean constant")?;
            Ok(ConstantKind::Boolean)
        }
        tag if tag == LBC_CONSTANT_NUMBER.0 as u8 => {
            cursor.skip(8, "number constant")?;
            Ok(ConstantKind::Number)
        }
        tag if tag == LBC_CONSTANT_VECTOR.0 as u8 => {
            cursor.skip(16, "vector constant")?;
            Ok(ConstantKind::Vector)
        }
        tag if tag == LBC_CONSTANT_STRING.0 as u8 => {
            cursor.read_string_id(string_count, "string constant", true)?;
            Ok(ConstantKind::String)
        }
        tag if tag == LBC_CONSTANT_IMPORT.0 as u8 => {
            cursor.skip(4, "import constant")?;
            Ok(ConstantKind::Import)
        }
        tag if tag == LBC_CONSTANT_TABLE.0 as u8 => {
            let keys = cursor.read_count("table constant key count", 1)?;
            for _ in 0..keys {
                let key = cursor.read_count("table constant key", 0)?;
                validate_constant_index(key, size_k, "table constant key")?;
            }
            Ok(ConstantKind::Table)
        }
        tag if tag == LBC_CONSTANT_TABLE_WITH_CONSTANTS.0 as u8 => {
            let keys = cursor.read_count("table-with-constants key count", 5)?;
            for _ in 0..keys {
                let key = cursor.read_count("table-with-constants key", 0)?;
                validate_constant_index(key, size_k, "table-with-constants key")?;
                let constant_idx = cursor.read_i32("table-with-constants value")?;
                if constant_idx >= 0 {
                    validate_constant_index(
                        constant_idx as usize,
                        size_k,
                        "table-with-constants value",
                    )?;
                }
            }
            Ok(ConstantKind::Table)
        }
        tag if tag == LBC_CONSTANT_CLOSURE.0 as u8 => {
            let fid = cursor.read_count("closure proto id", 0)?;
            validate_prior_proto(fid, proto_count, proto_index, "closure proto id")?;
            Ok(ConstantKind::Closure)
        }
        tag if tag == LBC_CONSTANT_CLASS_SHAPE.0 as u8 => {
            let class_name = cursor.read_count("class-shape class name constant", 0)?;
            validate_prior_string_constant(
                class_name,
                previous_constants,
                "class-shape class name",
            )?;
            let num_properties = cursor.read_count("class-shape property count", 1)?;
            let num_methods = cursor.read_count("class-shape method count", 1)?;
            let num_members = num_properties
                .checked_add(num_methods)
                .ok_or_else(|| BytecodeValidationError::new("class-shape member count overflow"))?;
            cursor.ensure_min_remaining(num_members, 1, "class-shape members")?;
            for _ in 0..num_members {
                let member = cursor.read_count("class-shape member name constant", 0)?;
                validate_prior_string_constant(
                    member,
                    previous_constants,
                    "class-shape member name",
                )?;
            }
            Ok(ConstantKind::ClassShape)
        }
        tag if tag == LBC_CONSTANT_INTEGER.0 as u8 => {
            cursor.read_u8("integer constant sign")?;
            cursor.read_var_u64("integer constant magnitude")?;
            Ok(ConstantKind::Integer)
        }
        other => Err(BytecodeValidationError::new(format!(
            "unknown constant tag {other}"
        ))),
    }
}

fn validate_instruction_stream(
    code: &[u32],
    constants: &[ConstantKind],
) -> Result<(), BytecodeValidationError> {
    let mut pc = 0usize;
    while pc < code.len() {
        let opcode_byte = (code[pc] & 0xff) as u8;
        if opcode_byte >= LuauOpcode::LOP__COUNT as u8 {
            return Err(BytecodeValidationError::new(format!(
                "invalid opcode {opcode_byte} at instruction {pc}"
            )));
        }
        let opcode = LuauOpcode::from(opcode_byte);
        let op_len = getOpLength(opcode) as usize;
        if op_len == 0 || pc + op_len > code.len() {
            return Err(BytecodeValidationError::new(format!(
                "instruction {pc} ({opcode:?}) extends past code stream"
            )));
        }

        if FFlag::LuauUdataDirectAccess6.get()
            && matches!(
                opcode,
                LuauOpcode::LOP_GETTABLEKS | LuauOpcode::LOP_SETTABLEKS | LuauOpcode::LOP_NAMECALL
            )
        {
            let key = code[pc + 1] as usize;
            validate_constant_index(key, constants.len(), "userdata direct-access key")?;
            if constants[key] != ConstantKind::String {
                return Err(BytecodeValidationError::new(format!(
                    "userdata direct-access key constant {key} is not a string"
                )));
            }
        }

        pc += op_len;
    }
    Ok(())
}

fn validate_constant_index(
    index: usize,
    len: usize,
    what: &str,
) -> Result<(), BytecodeValidationError> {
    if index >= len {
        return Err(BytecodeValidationError::new(format!(
            "{what} {index} out of bounds for {len} constants"
        )));
    }
    Ok(())
}

fn validate_prior_string_constant(
    index: usize,
    constants: &[ConstantKind],
    what: &str,
) -> Result<(), BytecodeValidationError> {
    let Some(kind) = constants.get(index) else {
        return Err(BytecodeValidationError::new(format!(
            "{what} {index} is not a prior constant"
        )));
    };
    if *kind != ConstantKind::String {
        return Err(BytecodeValidationError::new(format!(
            "{what} {index} is {kind:?}, not string"
        )));
    }
    Ok(())
}

fn validate_prior_proto(
    fid: usize,
    proto_count: usize,
    current_index: usize,
    what: &str,
) -> Result<(), BytecodeValidationError> {
    if fid >= proto_count {
        return Err(BytecodeValidationError::new(format!(
            "{what} {fid} out of bounds for {proto_count} protos"
        )));
    }
    if fid >= current_index {
        return Err(BytecodeValidationError::new(format!(
            "{what} {fid} is not loaded before proto {current_index}"
        )));
    }
    Ok(())
}

struct Cursor<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.offset)
    }

    fn read_u8(&mut self, what: &str) -> Result<u8, BytecodeValidationError> {
        let bytes = self.read_exact(1, what)?;
        Ok(bytes[0])
    }

    fn read_u32(&mut self, what: &str) -> Result<u32, BytecodeValidationError> {
        let bytes = self.read_exact(4, what)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_i32(&mut self, what: &str) -> Result<i32, BytecodeValidationError> {
        let bytes = self.read_exact(4, what)?;
        Ok(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_var_u32(&mut self, what: &str) -> Result<u32, BytecodeValidationError> {
        let mut result = 0u32;
        for byte_index in 0..5 {
            let byte = self.read_u8(what)?;
            let payload = u32::from(byte & 0x7f);
            if byte_index == 4 && payload > 0x0f {
                return Err(BytecodeValidationError::new(format!(
                    "{what} varint overflows u32 at offset {}",
                    self.offset
                )));
            }
            result |= payload << (byte_index * 7);
            if byte & 0x80 == 0 {
                return Ok(result);
            }
        }
        Err(BytecodeValidationError::new(format!(
            "{what} varint exceeds 5 bytes at offset {}",
            self.offset
        )))
    }

    fn read_var_u64(&mut self, what: &str) -> Result<u64, BytecodeValidationError> {
        let mut result = 0u64;
        for byte_index in 0..10 {
            let byte = self.read_u8(what)?;
            let payload = u64::from(byte & 0x7f);
            if byte_index == 9 && payload > 0x01 {
                return Err(BytecodeValidationError::new(format!(
                    "{what} varint overflows u64 at offset {}",
                    self.offset
                )));
            }
            result |= payload << (byte_index * 7);
            if byte & 0x80 == 0 {
                return Ok(result);
            }
        }
        Err(BytecodeValidationError::new(format!(
            "{what} varint exceeds 10 bytes at offset {}",
            self.offset
        )))
    }

    fn read_count(
        &mut self,
        what: &str,
        min_width: usize,
    ) -> Result<usize, BytecodeValidationError> {
        let count = self.read_var_u32(what)?;
        let count = usize::try_from(count)
            .map_err(|_| BytecodeValidationError::new(format!("{what} does not fit in usize")))?;
        self.ensure_min_remaining(count, min_width, what)?;
        Ok(count)
    }

    fn read_string_id(
        &mut self,
        string_count: usize,
        what: &str,
        nonzero: bool,
    ) -> Result<(), BytecodeValidationError> {
        let string_id = self.read_count(what, 0)?;
        if string_id == 0 {
            if nonzero {
                return Err(BytecodeValidationError::new(format!(
                    "{what} cannot be string id 0"
                )));
            }
            return Ok(());
        }
        if string_id > string_count {
            return Err(BytecodeValidationError::new(format!(
                "{what} string id {string_id} out of bounds for {string_count} strings"
            )));
        }
        Ok(())
    }

    fn ensure_min_remaining(
        &self,
        count: usize,
        min_width: usize,
        what: &str,
    ) -> Result<(), BytecodeValidationError> {
        let min_bytes = count.checked_mul(min_width).ok_or_else(|| {
            BytecodeValidationError::new(format!("{what} minimum byte count overflow"))
        })?;
        if min_bytes > self.remaining() {
            return Err(BytecodeValidationError::new(format!(
                "{what} count {count} requires at least {min_bytes} bytes, only {} remain",
                self.remaining()
            )));
        }
        Ok(())
    }

    fn peek(&self, len: usize, what: &str) -> Result<&'a [u8], BytecodeValidationError> {
        let end = self.offset.checked_add(len).ok_or_else(|| {
            BytecodeValidationError::new(format!("{what} length overflows offset"))
        })?;
        self.data.get(self.offset..end).ok_or_else(|| {
            BytecodeValidationError::new(format!(
                "unexpected EOF reading {what}: need {len} bytes at offset {}, only {} remain",
                self.offset,
                self.remaining()
            ))
        })
    }

    fn skip(&mut self, len: usize, what: &str) -> Result<(), BytecodeValidationError> {
        self.read_exact(len, what)?;
        Ok(())
    }

    fn read_exact(&mut self, len: usize, what: &str) -> Result<&'a [u8], BytecodeValidationError> {
        let bytes = self.peek(len, what)?;
        self.offset += len;
        Ok(bytes)
    }
}
