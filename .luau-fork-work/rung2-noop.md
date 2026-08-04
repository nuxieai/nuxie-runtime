# Luau fork rung 2 no-op rows

Diff: `91caa731..86d2a9dc`

| C++ symbol / row | Verification | Rust disposition |
| --- | --- | --- |
| `BytecodeBuilder::~BytecodeBuilder` (H36) | The C++ change adds only `virtual ~BytecodeBuilder() = default`; it adds no destructor body or engine cleanup. | No payload. Rust already performs default field destruction and has no C++ virtual-destructor analogue. |
| `BytecodeBuilder` private-to-protected access policy (H38) | The hunk changes only the C++ access label. The Rust builder state in `src/records/bytecode_builder.rs` is already crate-visible through `pub(crate)` fields. | No payload. Rust has no inheritance/protected access equivalent; changing field visibility would not mirror runtime behavior. |
| `BytecodeBuilder::dumpConstant` virtual qualifier (H39) | The C++ method body and all calls are unchanged; only virtual dispatch is added for subclasses. | No payload. The Rust twin remains a plain inherent method because luaur has no C++ subclass/virtual dispatch analogue. |

Verified against:

`git -C /Users/levi/dev/oss/luigi-rosso-luau diff 91caa731..86d2a9dc -- Bytecode/include/Luau/BytecodeBuilder.h Bytecode/src/BytecodeBuilder.cpp`
