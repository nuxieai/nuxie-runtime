# Differential corpus for nuxie-luau-vm

Each `*.luau` file is one case. `make luau-vm-diff` compiles it with the
vendored compiler under three profiles (all flags on at -O1 and -O2, all flags
off at -O1), runs the same bytecode on the vendored VM and on
`nuxie-luau-vm`, and compares returned values, `print` output and the escaping
error. Add a case by adding a file.

Cases must be deterministic on both VMs: never print an object address and
never depend on the iteration order of a table's hash part (sort keys first).
The harness provides `print` and `loadstring(source [, name])`.

After the run the harness checks that the corpus reached every opcode the
loader admits.
