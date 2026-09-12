# Original values in the production computation path

The production computed-style path now resolves declarations into a transient pair: the existing native declaration and optional original selected tokens. Direct declarations preserve text before ordinary_value normalization; variable declarations use the shared dual-stream resolver. The native declaration still carries its name, source, importance and normalized text, and existing typed property computation consumes exactly that declaration.

Validation remains in its existing order, including substituted losing declarations. Font size is still computed before font-relative dimensions regardless of declaration order. Original metadata is not retained in Style or mistaken for computed ideal bounds; it is available for the next typed-scalar integration and then dropped with the transient declarations.

Three focused tests verify direct and winning-variable normalization pairs; frozen inherited aliases, fallback choice and unavailable provenance; and production font ordering plus strict losing-declaration diagnostics. Full163 Rust and35 Node tests pass, together with native/WASM builds, TypeScript and immutable-source checks. All482 prior public outputs,79 padding/provenance controls and48 private flex candidate files/maps remain exact. Evidence is bound in declaration-provenance-receipt.json.

This closes the connection from original selected tokens into actual property computation. It does not yet attach scalar bounds to computed fields or qualify public flex. Next work must construct the typed scalar metadata while each native field is computed, including shorthand replacement and computed inheritance, then supply the descriptor/model prerequisites. No runtime or renderer modifications are involved.
