const encoder = new TextEncoder();
const decoder = new TextDecoder('utf-8', {fatal:true});
const MAX_REQUEST_BYTES = 192 * 1024 * 1024;
export const LANGUAGE_VERSION = 'nuxie-html-v1';

const failure = (code, message, source='request') => ({ok:false, diagnostics:[{code,source,message}]});

/** Instantiate the compiler once per editor worker or publishing worker. */
export async function createCompiler(bytesOrModule) {
  const loaded = await WebAssembly.instantiate(bytesOrModule, {});
  const wasm = (loaded instanceof WebAssembly.Instance ? loaded : loaded.instance).exports;
  const names = ['abi_version','request_alloc','compile','metadata_ptr','metadata_len','riv_ptr','riv_len','reset'];
  if (!(wasm.memory instanceof WebAssembly.Memory) || names.some(n => typeof wasm[`html_compiler_${n}`] !== 'function') || wasm.html_compiler_abi_version() !== 1) {
    throw new Error('Incompatible HTML compiler WebAssembly ABI');
  }
  function copy(ptr,len) {
    ptr >>>= 0; len >>>= 0;
    if (ptr + len > wasm.memory.buffer.byteLength) throw new Error('Invalid compiler response buffer');
    return new Uint8Array(wasm.memory.buffer,ptr,len).slice();
  }
  return Object.freeze({
    languageVersion: LANGUAGE_VERSION,
    compile(document) {
      let request;
      try {
        if (!document || typeof document !== 'object' || Array.isArray(document)) return failure('invalid-request','Expected a versioned design document');
        const {languageVersion, ...input} = document;
        if (languageVersion !== LANGUAGE_VERSION) return failure('unsupported-language-version',`Expected ${LANGUAGE_VERSION}`,'languageVersion');
        if (input.assets && typeof input.assets === 'object' && !Array.isArray(input.assets)) {
          input.assets = Object.fromEntries(Object.entries(input.assets).map(([key,asset]) => [key,
            asset?.bytes instanceof Uint8Array ? {...asset,bytes:Array.from(asset.bytes)} : asset,
          ]));
        }
        // JSON arrays and Uint8Array both work for asset bytes. Rive output is
        // returned as binary rather than expanding it back into a JSON array.
        request = encoder.encode(JSON.stringify({languageVersion,input}, (_,value) => value instanceof Uint8Array ? Array.from(value) : value));
      } catch (error) { return failure('invalid-request',String(error)); }
      if (request.byteLength > MAX_REQUEST_BYTES) return failure('input-limit','Serialized request exceeds 192 MiB');
      try {
        const ptr = wasm.html_compiler_request_alloc(request.length) >>> 0;
        if (!ptr) return failure('allocation-failed','Cannot allocate compiler request');
        // Allocation and compilation can grow memory. Always acquire a fresh
        // view and copy output before resetting owned WASM buffers.
        new Uint8Array(wasm.memory.buffer,ptr,request.length).set(request);
        const status = wasm.html_compiler_compile();
        const result = JSON.parse(decoder.decode(copy(wasm.html_compiler_metadata_ptr(),wasm.html_compiler_metadata_len())));
        if ((status === 0) !== (result.ok === true) || (status !== 0 && status !== 1)) throw new Error('Invalid compiler response status');
        if (!result.ok) return result;
        return {...result,riv:copy(wasm.html_compiler_riv_ptr(),wasm.html_compiler_riv_len())};
      } finally { wasm.html_compiler_reset(); }
    },
  });
}
