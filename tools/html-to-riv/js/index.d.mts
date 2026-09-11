export const LANGUAGE_VERSION: 'nuxie-html-immutable-v1';
export interface CompileInput {
  html: string;
  css: string;
  width: number;
  height: number;
}
export interface DesignDocument extends CompileInput {
  languageVersion: typeof LANGUAGE_VERSION;
}
export interface SourceNode {
  id: string;
  path: string;
  object_id: number;
}
export interface Diagnostic {
  code: string;
  source: string;
  message: string;
}
export type CompileResult = {
  ok: true;
  languageVersion: typeof LANGUAGE_VERSION;
  riv: Uint8Array;
  sourceMap: SourceNode[];
} | {
  ok: false;
  diagnostics: Diagnostic[];
};
export interface Compiler {
  readonly languageVersion: typeof LANGUAGE_VERSION;
  compile(document: DesignDocument): CompileResult;
}
export function createCompiler(bytesOrModule: BufferSource | WebAssembly.Module): Promise<Compiler>;
