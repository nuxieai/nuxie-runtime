const encoder = new TextEncoder();
const decoder = new TextDecoder('utf-8', {fatal:true});
const MAX_REQUEST_BYTES = 192 * 1024 * 1024;
export const LANGUAGE_VERSION = 'nuxie-html-immutable-v1';
const failure = (code, message, source='request') => ({ok:false, diagnostics:[{code,source,message}]});

/** Instantiate an isolated compiler. Output contains ordinary Rive bytes only. */
export async function createCompiler(bytesOrModule) {
  const loaded = await WebAssembly.instantiate(bytesOrModule, {});
  const wasm = (loaded instanceof WebAssembly.Instance ? loaded : loaded.instance).exports;
  const names = ['abi_version','request_alloc','compile','metadata_ptr','metadata_len','riv_ptr','riv_len','reset'];
  if (!(wasm.memory instanceof WebAssembly.Memory) || names.some(n => typeof wasm[`html_compiler_${n}`] !== 'function') || wasm.html_compiler_abi_version() !== 2) {
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
        if (Object.keys(input).some(k => !['html','css','width','height'].includes(k))) return failure('invalid-request','Unknown input field; assets and runtime policies are not supported');
        if (typeof input.html !== 'string' || typeof input.css !== 'string' || typeof input.width !== 'number' || typeof input.height !== 'number' || !Number.isFinite(input.width) || !Number.isFinite(input.height)) return failure('invalid-request','Expected html/css strings and finite width/height numbers');
        request = encoder.encode(JSON.stringify({languageVersion,input}));
      } catch (error) { return failure('invalid-request',String(error)); }
      if (request.byteLength > MAX_REQUEST_BYTES) return failure('input-limit','Serialized request exceeds 192 MiB');
      try {
        const ptr = wasm.html_compiler_request_alloc(request.length) >>> 0;
        if (!ptr) return failure('allocation-failed','Cannot allocate compiler request');
        // Allocation/compile may grow memory. Copy outputs before reset releases
        // Rust-owned buffers; the host never supplies free pointers or lengths.
        new Uint8Array(wasm.memory.buffer,ptr,request.length).set(request);
        const status = wasm.html_compiler_compile();
        const result = JSON.parse(decoder.decode(copy(wasm.html_compiler_metadata_ptr(),wasm.html_compiler_metadata_len())));
        if ((status === 0) !== (result.ok === true) || (status !== 0 && status !== 1)) throw new Error('Invalid compiler response status');
        if (!result.ok) return result;
        if (result.languageVersion !== LANGUAGE_VERSION || !Array.isArray(result.sourceMap) || Object.keys(result).some(k => !['ok','languageVersion','sourceMap'].includes(k))) throw new Error('Invalid immutable compiler metadata');
        return {...result,riv:copy(wasm.html_compiler_riv_ptr(),wasm.html_compiler_riv_len())};
      } finally { wasm.html_compiler_reset(); }
    },
  });
}
