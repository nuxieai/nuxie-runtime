--! attacks: malformed bytecode, a chunk claiming 2^32-1 strings then ending; expect: ok; within: 20s; harness: bytecode; bytecode: hex 06 03 ffffffff0f
return {}
