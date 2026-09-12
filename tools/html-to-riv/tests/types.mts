import {createCompiler, LANGUAGE_VERSION} from '../js/index.mjs';
import type {Compiler, CompileInput, DesignDocument, CompileResult, Diagnostic, SourceNode} from '../js/index.mjs';

const input: CompileInput = {html:'<div id="box"></div>',css:'#box{background:red;}',width:240,height:160};
const document: DesignDocument = {...input,languageVersion:LANGUAGE_VERSION};
const bytes = new Uint8Array(8);
const fromBytes: Promise<Compiler> = createCompiler(bytes);
const fromModule: Promise<Compiler> = createCompiler(new WebAssembly.Module(bytes));
void fromBytes; void fromModule;
function verify(compiler: Compiler) {
 const result: CompileResult = compiler.compile(document);
 if (result.ok) {
  const bytes: Uint8Array = result.riv;
  const map: SourceNode[] = result.sourceMap;
  const version: typeof LANGUAGE_VERSION = result.languageVersion;
  for (const node of map) { const identity: [string,string,number]=[node.id,node.path,node.object_id];void identity; }
  // @ts-expect-error success is ordinary Rive output, no policy contract
  result.runtimeRequirements;
  // @ts-expect-error diagnostics require the failure branch
  result.diagnostics;
  void bytes;void version;
 } else {
  const diagnostics: Diagnostic[] = result.diagnostics;
  for (const item of diagnostics) { const fields:string[]=[item.code,item.source,item.message];void fields; }
  // @ts-expect-error a rejected compile exposes no output bytes
  result.riv;
 }
 // @ts-expect-error old language contract is incompatible
 compiler.compile({...input,languageVersion:'nuxie-html-v1'});
 compiler.compile({...document,assets:{}});
 // @ts-expect-error viewport values must be numbers
 compiler.compile({...document,width:'240'});
 // @ts-expect-error a versioned document is required
 compiler.compile(input);
 // @ts-expect-error compiler's language version is read-only
 compiler.languageVersion=LANGUAGE_VERSION;
}
void verify;
