--! attacks: malformed bytecode, a chunk with one string whose length claims 2^32-1 bytes; expect: ok; within: 20s; harness: bytecode; bytecode: hex 06 03 01 ffffffff0f 41
return {}
