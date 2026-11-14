use std::{error::Error, fmt};

use nuxie_luau_vm::bytecode_format::{Opcode, op, tag};

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

/// The bytecode versions the VM runs. The safe VM loads 3 through
/// 11 and rejects 12 (cost model) and 13 (double-precision vector
/// constants), so the product rejects them here, before loading, rather than
/// admitting what the VM then refuses. Nothing the product uses emits them.
fn admitted_bytecode_versions() -> (u8, u8) {
    (
        nuxie_luau_vm::BYTECODE_VERSION_MIN,
        nuxie_luau_vm::BYTECODE_VERSION_MAX,
    )
}

fn unsupported_version_note(version: u8) -> &'static str {
    match version {
        12 => " (v12, the cost model, is not run by the safe Luau VM)",
        13 => " (v13, double-precision vector constants, is not run by the safe Luau VM)",
        _ => "",
    }
}

pub fn validate_luau_bytecode(bytecode: &[u8]) -> Result<(), BytecodeValidationError> {
    let mut cursor = Cursor::new(bytecode);
    let version = cursor.read_u8("bytecode version")?;

    if version == 0 {
        return Err(BytecodeValidationError::new(
            "bytecode version 0 contains compiler diagnostics, not executable bytecode",
        ));
    }

    let (min_version, max_version) = admitted_bytecode_versions();
    if !(min_version..=max_version).contains(&version) {
        return Err(BytecodeValidationError::new(format!(
            "bytecode version {version} is outside supported range {min_version}..={max_version}{}",
            unsupported_version_note(version)
        )));
    }

    let mut types_version = 0;
    if version >= 4 {
        types_version = cursor.read_u8("bytecode type version")?;
        let min_type_version = tag::TYPE_VERSION_MIN;
        let max_type_version = tag::TYPE_VERSION_MAX;
        if !(min_type_version..=max_type_version).contains(&types_version) {
            return Err(BytecodeValidationError::new(format!(
                "bytecode type version {types_version} is outside supported range {min_type_version}..={max_type_version}"
            )));
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
                    // A u8 plus two cannot overflow.
                    let expected = usize::from(num_params).saturating_add(2);
                    if type_size != expected {
                        return Err(BytecodeValidationError::new(format!(
                            "v1 typeinfo size {type_size} does not match numparams {num_params}"
                        )));
                    }
                    let header_matches = matches!(
                        typeinfo,
                        [kind, params, ..] if *kind == tag::TYPE_FUNCTION && *params == num_params
                    );
                    if !header_matches {
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
        // One interval per `2^linegaplog2` instructions, rounded up; none
        // for an empty proto. A shift of the word size or more is rejected.
        let intervals = match size_code.checked_sub(1) {
            None => 0,
            Some(last_pc) => last_pc
                .checked_shr(u32::from(line_gap_log2))
                .ok_or_else(|| {
                    BytecodeValidationError::new(format!(
                        "linegaplog2 {line_gap_log2} is too large"
                    ))
                })?
                .checked_add(1)
                .ok_or_else(|| BytecodeValidationError::new("abslineinfo size overflow"))?,
        };
        cursor.skip(size_code, "lineinfo byte deltas")?;
        cursor.skip(
            intervals
                .checked_mul(4)
                .ok_or_else(|| BytecodeValidationError::new("abslineinfo size overflow"))?,
            "absolute lineinfo",
        )?;
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

    // v11 call feedback: the `LuauCallFeedback` flag was on in every device
    // configuration, so v11 is always admitted, as the VM admits it.
    if version >= 11 {
        let feedback_slots = cursor.read_count("feedback slot count", 1)?;
        for _ in 0..feedback_slots {
            let slot_type = cursor.read_u8("feedback slot type")?;
            if slot_type != tag::FEEDBACK_CALL_TARGET {
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
    let constant_tag = cursor.read_u8("constant tag")?;
    match constant_tag {
        tag::CONSTANT_NIL => Ok(ConstantKind::Nil),
        tag::CONSTANT_BOOLEAN => {
            cursor.read_u8("boolean constant")?;
            Ok(ConstantKind::Boolean)
        }
        tag::CONSTANT_NUMBER => {
            cursor.skip(8, "number constant")?;
            Ok(ConstantKind::Number)
        }
        tag::CONSTANT_VECTOR => {
            cursor.skip(16, "vector constant")?;
            Ok(ConstantKind::Vector)
        }
        tag::CONSTANT_STRING => {
            cursor.read_string_id(string_count, "string constant", true)?;
            Ok(ConstantKind::String)
        }
        tag::CONSTANT_IMPORT => {
            cursor.skip(4, "import constant")?;
            Ok(ConstantKind::Import)
        }
        tag::CONSTANT_TABLE => {
            let keys = cursor.read_count("table constant key count", 1)?;
            for _ in 0..keys {
                let key = cursor.read_count("table constant key", 0)?;
                validate_constant_index(key, size_k, "table constant key")?;
            }
            Ok(ConstantKind::Table)
        }
        tag::CONSTANT_TABLE_WITH_CONSTANTS => {
            let keys = cursor.read_count("table-with-constants key count", 5)?;
            for _ in 0..keys {
                let key = cursor.read_count("table-with-constants key", 0)?;
                validate_constant_index(key, size_k, "table-with-constants key")?;
                let constant_idx = cursor.read_i32("table-with-constants value")?;
                // A negative value index means "no value"; any other is checked.
                if let Ok(constant_idx) = usize::try_from(constant_idx) {
                    validate_constant_index(constant_idx, size_k, "table-with-constants value")?;
                }
            }
            Ok(ConstantKind::Table)
        }
        tag::CONSTANT_CLOSURE => {
            let fid = cursor.read_count("closure proto id", 0)?;
            validate_prior_proto(fid, proto_count, proto_index, "closure proto id")?;
            Ok(ConstantKind::Closure)
        }
        tag::CONSTANT_CLASS_SHAPE => {
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
        tag::CONSTANT_INTEGER => {
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
    // `rest` starts at instruction `pc`; each step splits one whole
    // instruction (its opcode word and any operand words) off the front.
    let mut rest = code;
    let mut pc = 0usize;
    while let Some(&word) = rest.first() {
        let opcode_byte = (word & 0xff) as u8;
        let Some(opcode) = Opcode::new(opcode_byte) else {
            return Err(BytecodeValidationError::new(format!(
                "invalid opcode {opcode_byte} at instruction {pc}"
            )));
        };
        let op_len = opcode.length();
        let Some((instruction, after)) = rest.split_at_checked(op_len) else {
            return Err(BytecodeValidationError::new(format!(
                "instruction {pc} ({opcode:?}) extends past code stream"
            )));
        };

        if matches!(
            opcode.number(),
            op::GETTABLEKS | op::SETTABLEKS | op::NAMECALL
        ) {
            // These opcodes are two words long: the key is the aux word.
            if let [_, aux, ..] = instruction {
                validate_string_constant_key(*aux as usize, constants)?;
            }
        }

        rest = after;
        // `op_len` words were in `code`, so the sum is at most its length.
        pc = pc.saturating_add(op_len);
    }
    Ok(())
}

/// A direct-access key names an existing constant, and that constant is a
/// string.
fn validate_string_constant_key(
    key: usize,
    constants: &[ConstantKind],
) -> Result<(), BytecodeValidationError> {
    validate_constant_index(key, constants.len(), "userdata direct-access key")?;
    if constants.get(key) != Some(&ConstantKind::String) {
        return Err(BytecodeValidationError::new(format!(
            "userdata direct-access key constant {key} is not a string"
        )));
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

/// The one reader of the bytecode: every read takes a whole field off the
/// front of the bytes not yet read, or fails with the field's name.
struct Cursor<'a> {
    data: &'a [u8],
    rest: &'a [u8],
}

/// Shift of each 7-bit group of a LEB128 varint, lowest group first.
const VAR_U32_SHIFTS: [u32; 5] = [0, 7, 14, 21, 28];
const VAR_U64_SHIFTS: [u32; 10] = [0, 7, 14, 21, 28, 35, 42, 49, 56, 63];

impl<'a> Cursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, rest: data }
    }

    /// Bytes read so far.
    fn offset(&self) -> usize {
        // `rest` is a suffix of `data`.
        self.data.len().saturating_sub(self.rest.len())
    }

    fn remaining(&self) -> usize {
        self.rest.len()
    }

    fn read_array<const N: usize>(&mut self, what: &str) -> Result<[u8; N], BytecodeValidationError> {
        let Some((bytes, rest)) = self.rest.split_first_chunk::<N>() else {
            return Err(self.eof(N, what));
        };
        self.rest = rest;
        Ok(*bytes)
    }

    fn read_u8(&mut self, what: &str) -> Result<u8, BytecodeValidationError> {
        let [byte] = self.read_array(what)?;
        Ok(byte)
    }

    fn read_u32(&mut self, what: &str) -> Result<u32, BytecodeValidationError> {
        Ok(u32::from_le_bytes(self.read_array(what)?))
    }

    fn read_i32(&mut self, what: &str) -> Result<i32, BytecodeValidationError> {
        Ok(i32::from_le_bytes(self.read_array(what)?))
    }

    fn read_var_u32(&mut self, what: &str) -> Result<u32, BytecodeValidationError> {
        let mut result = 0u32;
        for (byte_index, shift) in VAR_U32_SHIFTS.into_iter().enumerate() {
            let byte = self.read_u8(what)?;
            let payload = u32::from(byte & 0x7f);
            if byte_index == 4 && payload > 0x0f {
                return Err(BytecodeValidationError::new(format!(
                    "{what} varint overflows u32 at offset {}",
                    self.offset()
                )));
            }
            // Every shift is below 32, and the last group's payload is at
            // most four bits, so no bit is shifted out.
            result |= payload.checked_shl(shift).unwrap_or(0);
            if byte & 0x80 == 0 {
                return Ok(result);
            }
        }
        Err(BytecodeValidationError::new(format!(
            "{what} varint exceeds 5 bytes at offset {}",
            self.offset()
        )))
    }

    fn read_var_u64(&mut self, what: &str) -> Result<u64, BytecodeValidationError> {
        let mut result = 0u64;
        for (byte_index, shift) in VAR_U64_SHIFTS.into_iter().enumerate() {
            let byte = self.read_u8(what)?;
            let payload = u64::from(byte & 0x7f);
            if byte_index == 9 && payload > 0x01 {
                return Err(BytecodeValidationError::new(format!(
                    "{what} varint overflows u64 at offset {}",
                    self.offset()
                )));
            }
            // Every shift is below 64, and the last group's payload is at
            // most one bit.
            result |= payload.checked_shl(shift).unwrap_or(0);
            if byte & 0x80 == 0 {
                return Ok(result);
            }
        }
        Err(BytecodeValidationError::new(format!(
            "{what} varint exceeds 10 bytes at offset {}",
            self.offset()
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

    fn eof(&self, len: usize, what: &str) -> BytecodeValidationError {
        BytecodeValidationError::new(format!(
            "unexpected EOF reading {what}: need {len} bytes at offset {}, only {} remain",
            self.offset(),
            self.remaining()
        ))
    }

    fn peek(&self, len: usize, what: &str) -> Result<&'a [u8], BytecodeValidationError> {
        self.rest.get(..len).ok_or_else(|| self.eof(len, what))
    }

    fn skip(&mut self, len: usize, what: &str) -> Result<(), BytecodeValidationError> {
        let Some((_, rest)) = self.rest.split_at_checked(len) else {
            return Err(self.eof(len, what));
        };
        self.rest = rest;
        Ok(())
    }
}

/// The host compiler (`luaur-common`'s tables) and the device tables
/// (`nuxie-luau-vm`'s) must not drift: entry by entry, every opcode number,
/// name and length and every tag the validator reads agree.
#[cfg(all(test, feature = "compiler"))]
mod compiler_table_tests {
    use luaur_common::enums::{
        luau_bytecode_tag::{
            LBC_CONSTANT__COUNT, LBC_CONSTANT_BOOLEAN, LBC_CONSTANT_CLASS_SHAPE,
            LBC_CONSTANT_CLOSURE, LBC_CONSTANT_IMPORT, LBC_CONSTANT_INTEGER, LBC_CONSTANT_NIL,
            LBC_CONSTANT_NUMBER, LBC_CONSTANT_STRING, LBC_CONSTANT_TABLE,
            LBC_CONSTANT_TABLE_WITH_CONSTANTS, LBC_CONSTANT_VECTOR, LBC_CONSTANT_VECTORD,
            LBC_TYPE_VERSION_MAX, LBC_TYPE_VERSION_MIN,
        },
        luau_bytecode_type::LBC_TYPE_FUNCTION,
        luau_feedback_type::LuauFeedbackType,
        luau_opcode::LuauOpcode,
    };
    use luaur_common::functions::get_op_length::getOpLength;
    use nuxie_luau_vm::bytecode_format::{Opcode, op, tag};

    #[test]
    fn vm_opcodes_match_the_compiler_entry_by_entry() {
        assert_eq!(op::COUNT, LuauOpcode::LOP__COUNT as u8);
        for byte in 0..=u8::MAX {
            let vm = Opcode::new(byte);
            if byte >= LuauOpcode::LOP__COUNT as u8 {
                assert!(vm.is_none(), "byte {byte} is past the compiler's opcodes");
                continue;
            }
            let vm = vm.unwrap_or_else(|| panic!("the VM has no opcode {byte}"));
            let compiler = LuauOpcode::from(byte);
            assert_eq!(vm.number(), compiler as u8);
            assert_eq!(vm.name(), format!("{compiler:?}"), "opcode {byte}");
            assert_eq!(format!("{vm:?}"), format!("{compiler:?}"), "opcode {byte}");
            assert_eq!(vm.length(), getOpLength(compiler) as usize, "length of {compiler:?}");
        }
    }

    #[test]
    fn vm_tags_match_the_compiler() {
        let constants = [
            (tag::CONSTANT_NIL, LBC_CONSTANT_NIL),
            (tag::CONSTANT_BOOLEAN, LBC_CONSTANT_BOOLEAN),
            (tag::CONSTANT_NUMBER, LBC_CONSTANT_NUMBER),
            (tag::CONSTANT_STRING, LBC_CONSTANT_STRING),
            (tag::CONSTANT_IMPORT, LBC_CONSTANT_IMPORT),
            (tag::CONSTANT_TABLE, LBC_CONSTANT_TABLE),
            (tag::CONSTANT_CLOSURE, LBC_CONSTANT_CLOSURE),
            (tag::CONSTANT_VECTOR, LBC_CONSTANT_VECTOR),
            (tag::CONSTANT_TABLE_WITH_CONSTANTS, LBC_CONSTANT_TABLE_WITH_CONSTANTS),
            (tag::CONSTANT_INTEGER, LBC_CONSTANT_INTEGER),
            (tag::CONSTANT_CLASS_SHAPE, LBC_CONSTANT_CLASS_SHAPE),
            (tag::CONSTANT_VECTORD, LBC_CONSTANT_VECTORD),
        ];
        assert_eq!(constants.len(), LBC_CONSTANT__COUNT.0 as usize, "every constant kind is listed");
        for (vm, compiler) in constants {
            assert_eq!(u32::from(vm), compiler.0, "constant tag {compiler:?}");
        }
        assert_eq!(u32::from(tag::TYPE_VERSION_MIN), LBC_TYPE_VERSION_MIN.0);
        assert_eq!(u32::from(tag::TYPE_VERSION_MAX), LBC_TYPE_VERSION_MAX.0);
        assert_eq!(u16::from(tag::TYPE_FUNCTION), LBC_TYPE_FUNCTION.0);
        assert_eq!(tag::FEEDBACK_CALL_TARGET, LuauFeedbackType::LFT_CALLTARGET as u8);
    }
}
