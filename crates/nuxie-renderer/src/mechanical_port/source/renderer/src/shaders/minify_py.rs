/*
 * Mechanical translation of the complete pinned source
 * renderer/src/shaders/minify.py.
 * Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
 *
 * This Phase-1 owner intentionally retains the Python tool's source-shaped
 * lexer/parser, global accounting/rename state, ordering, command-line surface,
 * and file modes. The executable Makefile translation invokes the same batch
 * command surface and missing-output recovery behavior.
 */

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

const PARSER_DESCRIPTION: &str = r#"
Process a batch of .glsl files. Minify and export them to C++ strings.

Performs the following transformations:
    * Strip comments.
    * Strip whitespace.
    * Strip unused #defines.
    * Rename stpq and rgba swizzles to xyzw.
    * Rename variables.
        - No new name begins with the '_' character, so internal code can begin names with '_'
          without fear of renaming collisions.
        - GLSL keywords and builtins are not renamed.
        - Tokens beginning with '@' have their new name exported to a header file.
        - Tokens beginning with '$' are not renamed, with the exception of removing the leading '$'.

"file.glsl" gets exported to:
    * "outdir/file.exports.h", with #defines for the rewritten names of each identifier that began
      with '@' in the original shader.
    * "outdir/file.glsl.cpp", with a global const char file_glsl[] in the rive::glsl
      namespace that contains the minified shader code. This variable is intentionally declared as a
      global in order to generate a link error if the user includes the string more than once in the
      build process.
    * "outdir/file.minified.glsl" for offline compiling, with all variables renamed except for
      exported #defines names (since the offline compiling process will set those defines).
"#;

#[derive(Clone, Debug, Default)]
pub struct Args {
    pub files: Vec<PathBuf>,
    pub outdir: PathBuf,
    pub human_readable: bool,
    pub ply_path: Option<String>,
    pub msvc: bool,
}

static ARGS: OnceLock<Args> = OnceLock::new();
static PYTHON_SYS_PATH: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn args() -> &'static Args {
    ARGS.get()
        .expect("argument parser must run before source operations")
}

// Python argparse accepts both long and short spellings used by the Makefile.
// Unknown/missing options are errors, preserving argparse's fail-closed route.
pub fn parse_args<I>(arguments: I) -> Result<Args, MinifyError>
where
    I: IntoIterator<Item = String>,
{
    let mut args = arguments.into_iter();
    let _program = args.next();
    let mut parsed = Args::default();
    let mut positional = Vec::new();

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "-H" | "--human-readable" => parsed.human_readable = true,
            "-m" | "--msvc" => parsed.msvc = true,
            "-o" | "--outdir" => {
                parsed.outdir = PathBuf::from(args.next().ok_or_else(|| {
                    MinifyError::new("argument -o/--outdir: expected one argument")
                })?);
            }
            "-p" | "--ply-path" => {
                parsed.ply_path = Some(args.next().ok_or_else(|| {
                    MinifyError::new("argument -p/--ply-path: expected one argument")
                })?);
            }
            option if option.starts_with("--outdir=") => {
                parsed.outdir = PathBuf::from(&option["--outdir=".len()..]);
            }
            option if option.starts_with("--ply-path=") => {
                parsed.ply_path = Some(option["--ply-path=".len()..].to_string());
            }
            option if option.starts_with('-') => {
                return Err(MinifyError::new(format!(
                    "unrecognized arguments: {option}"
                )));
            }
            file => positional.push(PathBuf::from(file)),
        }
    }

    if positional.is_empty() {
        return Err(MinifyError::new(
            "the following arguments are required: files",
        ));
    }
    if parsed.outdir.as_os_str().is_empty() {
        return Err(MinifyError::new(
            "the following arguments are required: -o/--outdir",
        ));
    }
    parsed.files = positional;
    Ok(parsed)
}

fn configure_ply_path(ply_path: Option<&str>) {
    if let Some(path) = ply_path {
        // --ply-path was specified, so add it to the sys path so we can locate
        // the module. If it was not specified we assume that it is already
        // reachable via the path.
        //
        // Convert posix path to windows.
        let mut converted_path = path.to_string();
        if cfg!(windows) && path.len() >= 2 && &path[..2] == "/c" {
            converted_path = format!(r"C:\{}", &path[2..]);
            println!("Using ply path:{converted_path}");
        }
        PYTHON_SYS_PATH
            .get_or_init(|| Mutex::new(Vec::new()))
            .lock()
            .expect("sys.path lock")
            .push(converted_path);
    }
}

#[derive(Debug)]
pub struct MinifyError {
    message: String,
}

impl MinifyError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for MinifyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for MinifyError {}

pub const TOKENS: &[&str] = &[
    "DEFINE",
    "IFDEF",
    "DEFINED_ID",
    "TOKEN_PASTE",
    "DIRECTIVE",
    "LINE_COMMENT",
    "BLOCK_COMMENT",
    "WHITESPACE",
    "OP",
    "FLOAT",
    "HEX",
    "INT",
    "ID",
    "UNKNOWN",
];

static EXPORTED_SWITCHES: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static ALL_ID_COUNTS: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();
static ALL_ID_REFERENCE_COUNTS: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();
// Python dictionaries retain insertion order. Keep a parallel order list so
// equal-count identifiers remain stable under the source's reverse-count sort.
static ALL_ID_ORDER: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static USED_NEW_NAMES: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static NEW_NAMES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
static DEFAULT_EXPORTS: OnceLock<Arc<Mutex<HashSet<String>>>> = OnceLock::new();
static UPPER_CASE_NAME_GENERATOR: OnceLock<Mutex<NameGenerator>> = OnceLock::new();
static GENERAL_NAME_GENERATOR: OnceLock<Mutex<NameGenerator>> = OnceLock::new();

fn exported_switches() -> &'static Mutex<HashSet<String>> {
    EXPORTED_SWITCHES.get_or_init(|| Mutex::new(HashSet::new()))
}

fn all_id_counts() -> &'static Mutex<HashMap<String, usize>> {
    ALL_ID_COUNTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn all_id_reference_counts() -> &'static Mutex<HashMap<String, usize>> {
    ALL_ID_REFERENCE_COUNTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn all_id_order() -> &'static Mutex<Vec<String>> {
    ALL_ID_ORDER.get_or_init(|| Mutex::new(Vec::new()))
}

fn used_new_names() -> &'static Mutex<HashSet<String>> {
    USED_NEW_NAMES.get_or_init(|| Mutex::new(HashSet::new()))
}

fn new_names() -> &'static Mutex<HashMap<String, String>> {
    NEW_NAMES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn default_exports() -> Arc<Mutex<HashSet<String>>> {
    DEFAULT_EXPORTS
        .get_or_init(|| Arc::new(Mutex::new(HashSet::new())))
        .clone()
}

// tracks which exported identifiers (identifiers whose @name begins with '@')
// are used as switches inside an #ifdef, #if defined(), etc.
fn note_exported_switch(name: &str) {
    if name.as_bytes()[0] == b'@' {
        exported_switches()
            .lock()
            .expect("exported switch lock")
            .insert(name.to_string());
    }
}

fn parse_id(name: &str, exports: &Arc<Mutex<HashSet<String>>>, is_reference: bool) {
    all_id_counts()
        .lock()
        .expect("identifier count lock")
        .entry(name.to_string())
        .and_modify(|count| *count += 1)
        .or_insert_with(|| {
            all_id_order()
                .lock()
                .expect("identifier order lock")
                .push(name.to_string());
            1
        });
    if is_reference {
        all_id_reference_counts()
            .lock()
            .expect("identifier reference count lock")
            .entry(name.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
    // identifiers that begin with '@' get exported to C++ through #defines.
    if name.as_bytes()[0] == b'@' {
        exports
            .lock()
            .expect("lexer export lock")
            .insert(name.to_string());
    }
}

#[derive(Debug)]
pub struct Token {
    pub r#type: String,
    pub value: String,
    pub define_id: Option<String>,
    pub define_arglist: Option<Box<Minifier>>,
    pub define_val: Option<Box<Minifier>>,
    pub ifdef_tag: Option<String>,
    pub ifdef_id: Option<String>,
    pub defined_id: Option<String>,
    pub directive_val: Option<Box<Minifier>>,
}

impl Token {
    fn new(token_type: &str, value: &str) -> Self {
        Self {
            r#type: token_type.to_string(),
            value: value.to_string(),
            define_id: None,
            define_arglist: None,
            define_val: None,
            ifdef_tag: None,
            ifdef_id: None,
            defined_id: None,
            directive_val: None,
        }
    }
}

#[derive(Clone)]
pub struct Lexer {
    pub exports: Arc<Mutex<HashSet<String>>>,
    pub lineno: usize,
}

fn count_newlines(value: &str) -> usize {
    value.chars().filter(|character| *character == '\n').count()
}

fn anchored_match(pattern: &str, input: &str) -> Result<Option<String>, MinifyError> {
    let regex = Regex::new(pattern).map_err(|error| MinifyError::new(error.to_string()))?;
    Ok(regex
        .find(input)
        .filter(|matched| matched.start() == 0)
        .map(|matched| matched.as_str().to_string()))
}

// Rust's `regex` deliberately does not implement the negative look-ahead used
// by PLY in DEFINE, DIRECTIVE, and BLOCK_COMMENT.  Match those three rules with
// the same left-to-right stopping conditions instead of feeding their Python
// expressions to a different regex language.
fn directive_body_len(input: &str) -> usize {
    let bytes = input.as_bytes();
    let mut offset = 0;
    while offset < bytes.len() {
        if bytes[offset] == b'\\' && bytes.get(offset + 1) == Some(&b'\n') {
            offset += 2;
            continue;
        }
        if bytes[offset] == b'\n'
            || (bytes[offset] == b'/' && matches!(bytes.get(offset + 1), Some(b'/') | Some(b'*')))
        {
            break;
        }
        offset += 1;
    }
    offset
}

fn match_directive(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    if bytes.first() != Some(&b'#') {
        return None;
    }
    let mut offset = 1;
    while matches!(bytes.get(offset), Some(b' ' | b'\t')) {
        offset += 1;
    }
    offset += directive_body_len(&input[offset..]);
    Some(input[..offset].to_string())
}

fn identifier_len(input: &str) -> usize {
    let bytes = input.as_bytes();
    let mut offset = usize::from(matches!(bytes.first(), Some(b'@' | b'$')));
    if !matches!(bytes.get(offset), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_')) {
        return 0;
    }
    offset += 1;
    while matches!(
        bytes.get(offset),
        Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
    ) {
        offset += 1;
    }
    offset
}

fn match_define(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    if bytes.first() != Some(&b'#') {
        return None;
    }
    let mut offset = 1;
    while matches!(bytes.get(offset), Some(b' ' | b'\t')) {
        offset += 1;
    }
    if !input[offset..].starts_with("define") {
        return None;
    }
    offset += "define".len();
    let whitespace_start = offset;
    while matches!(bytes.get(offset), Some(b' ' | b'\t')) {
        offset += 1;
    }
    if offset == whitespace_start {
        return None;
    }
    let id_len = identifier_len(&input[offset..]);
    if id_len == 0 {
        return None;
    }
    offset += id_len;
    if bytes.get(offset) == Some(&b'(') {
        if let Some(close) = input[offset + 1..].find(')') {
            offset += close + 2;
        }
    }
    offset += directive_body_len(&input[offset..]);
    Some(input[..offset].to_string())
}

fn match_block_comment(input: &str) -> Option<String> {
    input
        .strip_prefix("/*")?
        .find("*/")
        .map(|end| input[..end + 4].to_string())
}

// lexing functions used by PLY.
fn t_DEFINE(mut tok: Token, lexer: &mut Lexer) -> Result<Token, MinifyError> {
    let after_hash = tok.value[1..].trim_start_matches([' ', '\t']);
    let after_define = after_hash
        .strip_prefix("define")
        .ok_or_else(|| MinifyError::new("define token did not match its source rule"))?
        .trim_start_matches([' ', '\t']);
    let id_len = identifier_len(after_define);
    if id_len == 0 {
        return Err(MinifyError::new(
            "define token did not contain an identifier",
        ));
    }
    tok.define_id = Some(after_define[..id_len].to_string());
    let remainder = &after_define[id_len..];
    let (arglist, val) = if let Some(close) = remainder
        .starts_with('(')
        .then(|| remainder.find(')'))
        .flatten()
    {
        (
            Some(remainder[..=close].to_string()),
            Some(remainder[close + 1..].to_string()).filter(|value| !value.is_empty()),
        )
    } else {
        (
            None,
            Some(remainder.to_string()).filter(|value| !value.is_empty()),
        )
    };
    tok.define_arglist = arglist
        .map(|value| Minifier::new_with_exports(&value, "", lexer.exports.clone()))
        .transpose()?
        .map(Box::new);
    tok.define_val = val
        .map(|value| Minifier::new_with_exports(&value, "", lexer.exports.clone()))
        .transpose()?
        .map(Box::new);
    parse_id(tok.define_id.as_deref().unwrap(), &lexer.exports, false);
    lexer.lineno += count_newlines(&tok.value);
    Ok(tok)
}

fn t_IFDEF(mut tok: Token, lexer: &mut Lexer) -> Result<Token, MinifyError> {
    let pattern = r"\#[ \t]*(?P<tag>ifn?def)[ \t]+(?P<ifdef_id>[\@\$]?[A-Za-z_][A-Za-z0-9_]*)";
    let captures = Regex::new(pattern)
        .map_err(|error| MinifyError::new(error.to_string()))?
        .captures(&tok.value)
        .ok_or_else(|| MinifyError::new("ifdef token did not match its source rule"))?;
    tok.ifdef_tag = Some(captures["tag"].to_string());
    tok.ifdef_id = Some(captures["ifdef_id"].to_string());
    note_exported_switch(tok.ifdef_id.as_deref().unwrap());
    parse_id(tok.ifdef_id.as_deref().unwrap(), &lexer.exports, true);
    Ok(tok)
}

fn t_DEFINED_ID(mut tok: Token, lexer: &mut Lexer) -> Result<Token, MinifyError> {
    let pattern = r"defined\((?P<defined_id>[\@\$]?[A-Za-z_][A-Za-z0-9_]*)\)";
    let captures = Regex::new(pattern)
        .map_err(|error| MinifyError::new(error.to_string()))?
        .captures(&tok.value)
        .ok_or_else(|| MinifyError::new("defined-id token did not match its source rule"))?;
    tok.defined_id = Some(captures["defined_id"].to_string());
    note_exported_switch(tok.defined_id.as_deref().unwrap());
    parse_id(tok.defined_id.as_deref().unwrap(), &lexer.exports, true);
    Ok(tok)
}

fn t_TOKEN_PASTE(tok: Token, _lexer: &mut Lexer) -> Result<Token, MinifyError> {
    Ok(tok)
}

fn t_DIRECTIVE(mut tok: Token, lexer: &mut Lexer) -> Result<Token, MinifyError> {
    let val = Some(tok.value[1..].trim_start_matches([' ', '\t']).to_string())
        .filter(|value| !value.is_empty());
    tok.directive_val = val
        .map(|value| Minifier::new_with_exports(&value, "", lexer.exports.clone()))
        .transpose()?
        .map(Box::new);
    lexer.lineno += count_newlines(&tok.value);
    Ok(tok)
}

fn t_LINE_COMMENT(tok: Token, lexer: &mut Lexer) -> Result<Token, MinifyError> {
    lexer.lineno += count_newlines(&tok.value);
    Ok(tok)
}

fn t_BLOCK_COMMENT(tok: Token, lexer: &mut Lexer) -> Result<Token, MinifyError> {
    lexer.lineno += count_newlines(&tok.value);
    Ok(tok)
}

fn t_WHITESPACE(tok: Token, lexer: &mut Lexer) -> Result<Token, MinifyError> {
    lexer.lineno += count_newlines(&tok.value);
    Ok(tok)
}

fn t_OP(tok: Token, _lexer: &mut Lexer) -> Result<Token, MinifyError> {
    Ok(tok)
}

fn t_FLOAT(tok: Token, _lexer: &mut Lexer) -> Result<Token, MinifyError> {
    Ok(tok)
}

fn t_HEX(tok: Token, _lexer: &mut Lexer) -> Result<Token, MinifyError> {
    Ok(tok)
}

fn t_INT(tok: Token, _lexer: &mut Lexer) -> Result<Token, MinifyError> {
    Ok(tok)
}

fn t_ID(tok: Token, lexer: &mut Lexer) -> Result<Token, MinifyError> {
    parse_id(&tok.value, &lexer.exports, true);
    Ok(tok)
}

fn t_UNKNOWN(tok: Token, _lexer: &mut Lexer) -> Result<Token, MinifyError> {
    Ok(tok)
}

fn t_error(tok: &Token, lexer: &Lexer) -> Result<(), MinifyError> {
    Err(MinifyError::new(format!(
        "Illegal character '{}' at line {}",
        tok.value.chars().next().unwrap_or('\0'),
        lexer.lineno
    )))
}

fn lex_code(code: &str, exports: Arc<Mutex<HashSet<String>>>) -> Result<Vec<Token>, MinifyError> {
    let mut lexer = Lexer { exports, lineno: 1 };
    let mut offset = 0;
    let mut tokens = Vec::new();
    let rules: &[(&str, &str)] = &[
        (
            "DEFINE",
            r"\#[ \t]*define[ \t]+(?P<id>[\@\$]?[A-Za-z_][A-Za-z0-9_]*)(?P<arglist>\((\n|[^\)])*\))?(?P<val>(((\\\n|.)(?!\/[\/\*]))*))?",
        ),
        (
            "IFDEF",
            r"\#[ \t]*(?P<tag>ifn?def)[ \t]+(?P<ifdef_id>[\@\$]?[A-Za-z_][A-Za-z0-9_]*)",
        ),
        (
            "DEFINED_ID",
            r"defined\((?P<defined_id>[\@\$]?[A-Za-z_][A-Za-z0-9_]*)\)",
        ),
        ("TOKEN_PASTE", r"\#\#"),
        ("DIRECTIVE", r"\#[ \t]*(?P<val>(((\\\n|.)(?!\/[\/\*]))*))"),
        ("LINE_COMMENT", r"//(\\\n|.)*"),
        ("BLOCK_COMMENT", r"\/\*(\*(?!\/)|[^*])*\*\/"),
        ("WHITESPACE", r"(\s|\\\r?\n)+"),
        ("OP", r"[~!%^&*()=+/\[\]{}?:<>.,|;-]"),
        (
            "FLOAT",
            r"([0-9]*\.[0-9]+|[0-9]+\.)([eE][+\-]?[0-9]+)?|([0-9]+[eE][+\-]?[0-9]+)",
        ),
        ("HEX", r"0x[0-9a-fA-F]+u?"),
        ("INT", r"[0-9]+u?"),
        ("ID", r"[\@\$]?[A-Za-z_][A-Za-z0-9_]*"),
        ("UNKNOWN", r"."),
    ];

    while offset < code.len() {
        let remaining = &code[offset..];
        let mut matched = None;
        for (token_type, pattern) in rules {
            let value = match *token_type {
                "DEFINE" => match_define(remaining),
                "DIRECTIVE" => match_directive(remaining),
                "BLOCK_COMMENT" => match_block_comment(remaining),
                _ => anchored_match(pattern, remaining)?,
            };
            if let Some(value) = value {
                matched = Some(((*token_type).to_string(), value));
                break;
            }
        }
        let (token_type, value) = match matched {
            Some(value) => value,
            None => {
                let unknown = Token::new("", remaining);
                t_error(&unknown, &lexer)?;
                break;
            }
        };
        let token_len = value.len();
        let token = Token::new(&token_type, &value);
        let token = match token_type.as_str() {
            "DEFINE" => t_DEFINE(token, &mut lexer)?,
            "IFDEF" => t_IFDEF(token, &mut lexer)?,
            "DEFINED_ID" => t_DEFINED_ID(token, &mut lexer)?,
            "TOKEN_PASTE" => t_TOKEN_PASTE(token, &mut lexer)?,
            "DIRECTIVE" => t_DIRECTIVE(token, &mut lexer)?,
            "LINE_COMMENT" => t_LINE_COMMENT(token, &mut lexer)?,
            "BLOCK_COMMENT" => t_BLOCK_COMMENT(token, &mut lexer)?,
            "WHITESPACE" => t_WHITESPACE(token, &mut lexer)?,
            "OP" => t_OP(token, &mut lexer)?,
            "FLOAT" => t_FLOAT(token, &mut lexer)?,
            "HEX" => t_HEX(token, &mut lexer)?,
            "INT" => t_INT(token, &mut lexer)?,
            "ID" => t_ID(token, &mut lexer)?,
            "UNKNOWN" => t_UNKNOWN(token, &mut lexer)?,
            _ => unreachable!("PLY token rule table is exhaustive"),
        };
        tokens.push(token);
        offset += token_len;
    }
    Ok(tokens)
}

// identifier names that cannot be renamed
pub const GLSL_RESERVED: &[&str] = &[
    "EmitStreamVertex",
    "EmitVertex",
    "EmitVertex",
    "EndPrimitive",
    "EndPrimitive",
    "EndStreamPrimitive",
    "abs",
    "abs",
    "abs",
    "acos",
    "acosh",
    "all",
    "allInvocations",
    "allInvocationsEqual",
    "any",
    "anyInvocation",
    "asin",
    "asinh",
    "atan",
    "atan",
    "atanh",
    "atomicAdd",
    "atomicAdd",
    "atomicAnd",
    "atomicAnd",
    "atomicCompSwap",
    "atomicCompSwap",
    "atomicCounter",
    "atomicCounterAdd",
    "atomicCounterAnd",
    "atomicCounterCompSwap",
    "atomicCounterDecrement",
    "atomicCounterExchange",
    "atomicCounterIncrement",
    "atomicCounterMax",
    "atomicCounterMin",
    "atomicCounterOr",
    "atomicCounterSubtract",
    "atomicCounterXor",
    "atomicExchange",
    "atomicExchange",
    "atomicMax",
    "atomicMax",
    "atomicMin",
    "atomicMin",
    "atomicOr",
    "atomicOr",
    "atomicXor",
    "atomicXor",
    "barrier",
    "barrier",
    "beginFragmentShaderOrderingINTEL",
    "beginInvocationInterlockARB",
    "beginInvocationInterlockNV",
    "binding",
    "bitCount",
    "bitCount",
    "bitfieldExtract",
    "bitfieldExtract",
    "bitfieldInsert",
    "bitfieldInsert",
    "bitfieldReverse",
    "bitfieldReverse",
    "bool",
    "break",
    "bvec2",
    "bvec3",
    "bvec4",
    "case",
    "ceil",
    "ceil",
    "centroid",
    "clamp",
    "clamp",
    "clamp",
    "clamp",
    "clamp",
    "clamp",
    "clamp",
    "clamp",
    "coherent",
    "const",
    "continue",
    "cos",
    "cosh",
    "cross",
    "cross",
    "dFdx",
    "dFdx",
    "dFdxCoarse",
    "dFdxFine",
    "dFdy",
    "dFdy",
    "dFdyCoarse",
    "dFdyFine",
    "default",
    "degrees",
    "determinant",
    "determinant",
    "determinant",
    "discard",
    "distance",
    "distance",
    "do",
    "dot",
    "dot",
    "else",
    "endInvocationInterlockARB",
    "endInvocationInterlockNV",
    "equal",
    "equal",
    "equal",
    "equal",
    "exp",
    "exp2",
    "faceforward",
    "faceforward",
    "false",
    "findLSB",
    "findLSB",
    "findMSB",
    "findMSB",
    "flat",
    "float",
    "floatBitsToInt",
    "floatBitsToUint",
    "floor",
    "floor",
    "fma",
    "fma",
    "fma",
    "for",
    "fract",
    "fract",
    "frexp",
    "frexp",
    "ftransform",
    "fwidth",
    "fwidth",
    "fwidthCoarse",
    "fwidthFine",
    "greaterThan",
    "greaterThan",
    "greaterThan",
    "greaterThanEqual",
    "greaterThanEqual",
    "greaterThanEqual",
    "groupMemoryBarrier",
    "highp",
    "if",
    "iimage2D",
    "image2D",
    "imageAtomicAdd",
    "imageAtomicAdd",
    "imageAtomicAdd",
    "imageAtomicAdd",
    "imageAtomicAnd",
    "imageAtomicAnd",
    "imageAtomicAnd",
    "imageAtomicAnd",
    "imageAtomicCompSwap",
    "imageAtomicCompSwap",
    "imageAtomicCompSwap",
    "imageAtomicCompSwap",
    "imageAtomicExchange",
    "imageAtomicExchange",
    "imageAtomicExchange",
    "imageAtomicExchange",
    "imageAtomicExchange",
    "imageAtomicMax",
    "imageAtomicMax",
    "imageAtomicMax",
    "imageAtomicMax",
    "imageAtomicMin",
    "imageAtomicMin",
    "imageAtomicMin",
    "imageAtomicMin",
    "imageAtomicOr",
    "imageAtomicOr",
    "imageAtomicOr",
    "imageAtomicOr",
    "imageAtomicXor",
    "imageAtomicXor",
    "imageAtomicXor",
    "imageAtomicXor",
    "imageLoad",
    "imageLoad",
    "imageLoad",
    "imageLoad",
    "imageLoad",
    "imageLoad",
    "imageLoad",
    "imageLoad",
    "imageSamples",
    "imageSamples",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageSize",
    "imageStore",
    "imageStore",
    "imageStore",
    "imageStore",
    "imageStore",
    "imageStore",
    "imageStore",
    "imageStore",
    "imageStore",
    "imulExtended",
    "in",
    "inout",
    "int",
    "intBitsToFloat",
    "interpolateAtCentroid",
    "interpolateAtCentroid",
    "interpolateAtCentroid",
    "interpolateAtCentroid",
    "interpolateAtCentroid",
    "interpolateAtCentroid",
    "interpolateAtCentroid",
    "interpolateAtCentroid",
    "interpolateAtOffset",
    "interpolateAtOffset",
    "interpolateAtOffset",
    "interpolateAtOffset",
    "interpolateAtOffset",
    "interpolateAtOffset",
    "interpolateAtOffset",
    "interpolateAtSample",
    "interpolateAtSample",
    "interpolateAtSample",
    "interpolateAtSample",
    "interpolateAtSample",
    "interpolateAtSample",
    "interpolateAtSample",
    "interpolateAtSample",
    "invariant",
    "inverse",
    "inverse",
    "inverse",
    "inversesqrt",
    "inversesqrt",
    "isampler2D",
    "isampler2DArray",
    "isampler3D",
    "isamplerCube",
    "isinf",
    "isinf",
    "isnan",
    "isnan",
    "ivec2",
    "ivec3",
    "ivec4",
    "layout",
    "ldexp",
    "ldexp",
    "length",
    "length",
    "lessThan",
    "lessThan",
    "lessThan",
    "lessThanEqual",
    "lessThanEqual",
    "lessThanEqual",
    "location",
    "log",
    "log2",
    "lowp",
    "main",
    "mat2",
    "mat2x2",
    "mat2x3",
    "mat2x4",
    "mat3",
    "mat3x2",
    "mat3x3",
    "mat3x4",
    "mat4",
    "mat4x2",
    "mat4x3",
    "mat4x4",
    "matrixCompMult",
    "matrixCompMult",
    "matrixCompMult",
    "matrixCompMult",
    "matrixCompMult",
    "matrixCompMult",
    "matrixCompMult",
    "matrixCompMult",
    "matrixCompMult",
    "max",
    "max",
    "max",
    "max",
    "max",
    "max",
    "max",
    "max",
    "mediump",
    "memoryBarrier",
    "memoryBarrierAtomicCounter",
    "memoryBarrierBuffer",
    "memoryBarrierImage",
    "memoryBarrierShared",
    "min",
    "min",
    "min",
    "min",
    "min",
    "min",
    "min",
    "min",
    "mix",
    "mix",
    "mix",
    "mix",
    "mix",
    "mix",
    "mix",
    "mix",
    "mix",
    "mod",
    "mod",
    "mod",
    "mod",
    "modf",
    "modf",
    "noise1",
    "noise2",
    "noise3",
    "noise4",
    "noperspective",
    "normalize",
    "normalize",
    "not",
    "notEqual",
    "notEqual",
    "notEqual",
    "notEqual",
    "out",
    "outerProduct",
    "outerProduct",
    "outerProduct",
    "outerProduct",
    "outerProduct",
    "outerProduct",
    "outerProduct",
    "outerProduct",
    "outerProduct",
    "packDouble2x32",
    "packHalf2x16",
    "packSnorm2x16",
    "packSnorm4x8",
    "packUnorm2x16",
    "packUnorm4x8",
    "pixelLocalLoadANGLE",
    "pixelLocalStoreANGLE",
    "pow",
    "precision",
    "r16f",
    "r32f",
    "r32i",
    "r32ui",
    "radians",
    "reflect",
    "reflect",
    "refract",
    "refract",
    "return",
    "rg16f",
    "rgb_2_yuv",
    "rgba8",
    "rgba8i",
    "rgba8ui",
    "round",
    "round",
    "roundEven",
    "roundEven",
    "sampler2D",
    "sampler2DArray",
    "sampler2DArrayShadow",
    "sampler2DShadow",
    "sampler3D",
    "samplerCube",
    "samplerCubeShadow",
    "shadow1D",
    "shadow1DLod",
    "shadow1DProj",
    "shadow1DProj",
    "shadow1DProjLod",
    "shadow2D",
    "shadow2D",
    "shadow2DEXT",
    "shadow2DLod",
    "shadow2DProj",
    "shadow2DProj",
    "shadow2DProjEXT",
    "shadow2DProjLod",
    "sign",
    "sign",
    "sign",
    "sin",
    "sinh",
    "smooth",
    "smoothstep",
    "smoothstep",
    "smoothstep",
    "smoothstep",
    "sqrt",
    "sqrt",
    "std140",
    "std430",
    "step",
    "step",
    "step",
    "step",
    "struct",
    "subpassLoad",
    "subpassLoad",
    "switch",
    "tan",
    "tanh",
    "texelFetch",
    "texelFetch",
    "texelFetch",
    "texelFetch",
    "texelFetch",
    "texelFetch",
    "texelFetch",
    "texelFetch",
    "texelFetch",
    "texelFetch",
    "texelFetch",
    "texelFetch",
    "texelFetch",
    "texelFetchOffset",
    "texelFetchOffset",
    "texelFetchOffset",
    "texelFetchOffset",
    "texelFetchOffset",
    "texelFetchOffset",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture",
    "texture1D",
    "texture1D",
    "texture1DLod",
    "texture1DProj",
    "texture1DProj",
    "texture1DProj",
    "texture1DProj",
    "texture1DProjLod",
    "texture1DProjLod",
    "texture2D",
    "texture2D",
    "texture2D",
    "texture2DGradEXT",
    "texture2DLod",
    "texture2DLod",
    "texture2DLodEXT",
    "texture2DProj",
    "texture2DProj",
    "texture2DProj",
    "texture2DProj",
    "texture2DProj",
    "texture2DProj",
    "texture2DProjGradEXT",
    "texture2DProjGradEXT",
    "texture2DProjLod",
    "texture2DProjLod",
    "texture2DProjLod",
    "texture2DProjLod",
    "texture2DProjLodEXT",
    "texture2DProjLodEXT",
    "texture2DRect",
    "texture2DRectProj",
    "texture2DRectProj",
    "texture3D",
    "texture3D",
    "texture3D",
    "texture3D",
    "texture3DLod",
    "texture3DLod",
    "texture3DProj",
    "texture3DProj",
    "texture3DProj",
    "texture3DProj",
    "texture3DProjLod",
    "texture3DProjLod",
    "textureCube",
    "textureCube",
    "textureCubeGradEXT",
    "textureCubeLod",
    "textureCubeLod",
    "textureCubeLodEXT",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGather",
    "textureGatherOffset",
    "textureGatherOffset",
    "textureGatherOffset",
    "textureGatherOffset",
    "textureGatherOffset",
    "textureGatherOffset",
    "textureGatherOffset",
    "textureGatherOffset",
    "textureGatherOffset",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGatherOffsets",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGrad",
    "textureGradOffset",
    "textureGradOffset",
    "textureGradOffset",
    "textureGradOffset",
    "textureGradOffset",
    "textureGradOffset",
    "textureGradOffset",
    "textureGradOffset",
    "textureGradOffset",
    "textureGradOffset",
    "textureGradOffset",
    "textureLod",
    "textureLod",
    "textureLod",
    "textureLod",
    "textureLod",
    "textureLod",
    "textureLod",
    "textureLod",
    "textureLod",
    "textureLod",
    "textureLod",
    "textureLodOffset",
    "textureLodOffset",
    "textureLodOffset",
    "textureLodOffset",
    "textureLodOffset",
    "textureLodOffset",
    "textureLodOffset",
    "textureLodOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureOffset",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProj",
    "textureProjGrad",
    "textureProjGrad",
    "textureProjGrad",
    "textureProjGrad",
    "textureProjGrad",
    "textureProjGrad",
    "textureProjGrad",
    "textureProjGrad",
    "textureProjGrad",
    "textureProjGrad",
    "textureProjGradOffset",
    "textureProjGradOffset",
    "textureProjGradOffset",
    "textureProjGradOffset",
    "textureProjGradOffset",
    "textureProjGradOffset",
    "textureProjGradOffset",
    "textureProjGradOffset",
    "textureProjGradOffset",
    "textureProjGradOffset",
    "textureProjLod",
    "textureProjLod",
    "textureProjLod",
    "textureProjLod",
    "textureProjLod",
    "textureProjLod",
    "textureProjLod",
    "textureProjLodOffset",
    "textureProjLodOffset",
    "textureProjLodOffset",
    "textureProjLodOffset",
    "textureProjLodOffset",
    "textureProjLodOffset",
    "textureProjLodOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureProjOffset",
    "textureQueryLevels",
    "textureQueryLevels",
    "textureQueryLevels",
    "textureQueryLevels",
    "textureQueryLevels",
    "textureQueryLevels",
    "textureQueryLevels",
    "textureQueryLevels",
    "textureQueryLevels",
    "textureQueryLevels",
    "textureQueryLevels",
    "textureQueryLevels",
    "textureQueryLevels",
    "textureQueryLod",
    "textureQueryLod",
    "textureQueryLod",
    "textureQueryLod",
    "textureQueryLod",
    "textureQueryLod",
    "textureQueryLod",
    "textureQueryLod",
    "textureQueryLod",
    "textureQueryLod",
    "textureQueryLod",
    "textureQueryLod",
    "textureQueryLod",
    "textureSamples",
    "textureSamples",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureSize",
    "textureVideoWEBGL",
    "transpose",
    "transpose",
    "transpose",
    "transpose",
    "transpose",
    "transpose",
    "transpose",
    "transpose",
    "transpose",
    "true",
    "trunc",
    "trunc",
    "uaddCarry",
    "uimage2D",
    "uint",
    "uintBitsToFloat",
    "umulExtended",
    "uniform",
    "unpackDouble2x32",
    "unpackHalf2x16",
    "unpackSnorm2x16",
    "unpackSnorm4x8",
    "unpackUnorm2x16",
    "usampler2D",
    "usampler2DArray",
    "usampler3D",
    "usamplerCube",
    "usubBorrow",
    "uvec2",
    "uvec3",
    "uvec4",
    "vec2",
    "vec3",
    "vec4",
    "void",
    "volatile",
    "while",
    "yuv_2_rgb",
    "__pixel_localEXT",
    "__pixel_local_inEXT",
    "__pixel_local_outEXT",
    "set",
    "texture2D",
    "utexture2D",
    "sampler",
    "subpassInput",
    "subpassInputMS",
    "usubpassInput",
    "input_attachment_index",
    "readonly",
    "buffer",
    "unpackUnorm4x8",
    "defined",
    "elif",
    "extension",
    "enable",
    "require",
    "endif",
    "undef",
    "pragma",
    "__VERSION__",
    "constant_id",
    "blend_support_all_equations",
    "blend_support_multiply",
    "blend_support_screen",
    "blend_support_overlay",
    "blend_support_darken",
    "blend_support_lighten",
    "blend_support_colordodge",
    "blend_support_colorburn",
    "blend_support_hardlight",
    "blend_support_softlight",
    "blend_support_difference",
    "blend_support_exclusion",
    "rgb10_a2",
];

fn glsl_reserved() -> &'static HashSet<&'static str> {
    static RESERVED: OnceLock<HashSet<&'static str>> = OnceLock::new();
    RESERVED.get_or_init(|| GLSL_RESERVED.iter().copied().collect())
}

// rgba and stpq get rewritten to xyzw, so we only need to check xyzw here.
const XYZW_PATTERN: &str = r"^[xyzw]{1,4}$";

// HLSL registers base names can't be overwritten by macro arguments if token pasting
// (e.g. t##IDX).
const HLSL_REGISTER_BASE_NAMES: &[&str] = &["t", "s", "u", "b"];
const HLSL_REGISTER_PATTERN: &str = r"^[tsub]\d+$";

// can we rename to or from 'name'?
fn is_xyzw(name: &str) -> bool {
    let length = name.len();
    (1..=4).contains(&length) && name.bytes().all(|character| b"xyzw".contains(&character))
}

fn is_hlsl_register(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some('t' | 's' | 'u' | 'b'))
        && chars.next().is_some()
        && chars.all(|character| character.is_ascii_digit())
}

fn is_reserved_keyword(name: &str) -> bool {
    glsl_reserved().contains(name)
        || is_xyzw(name)
        || HLSL_REGISTER_BASE_NAMES.contains(&name)
        || is_hlsl_register(name)
        || name.starts_with('$')
        || name.starts_with("gl_")
        || name.starts_with("GL_")
        || name.starts_with("__pixel_local")
        || name.ends_with("ANGLE")
}

fn remove_leading_annotation(name: &str) -> String {
    if name.as_bytes()[0] == b'@' {
        // A leading '@' indicates identifier names that should be exported.
        // Rename '@my_var' to 'EXPORTED_my_var' to enforce that '@my_var' and
        // 'my_var' are not interchangeable.
        return format!("EXPORTED_{}", &name[1..]);
    }
    if name.as_bytes()[0] == b'$' {
        // A leading '$' indicates identifier names that should not be renamed.
        return name[1..].to_string();
    }
    name.to_string()
}

// Generates new identifier names to rewrite our variables.
#[derive(Clone, Debug)]
pub struct NameGenerator {
    pub first_letter_chars: String,
    pub additional_letter_chars: String,
    pub name_index: usize,
}

impl NameGenerator {
    fn new(first_letter_chars: &str, additional_letter_chars: &str) -> Self {
        Self {
            first_letter_chars: first_letter_chars.to_string(),
            additional_letter_chars: additional_letter_chars.to_string(),
            name_index: 0,
        }
    }

    fn next_name(&mut self) -> String {
        let mut i = self.name_index;
        // Generate the first character from 'self.first_letter_chars'.
        let bytes = self.first_letter_chars.as_bytes();
        let mut name = (bytes[i % bytes.len()] as char).to_string();
        i /= bytes.len();
        while i > 0 {
            // Generate the remaining characters from 'self.additional_letter_chars'.
            let additional = self.additional_letter_chars.as_bytes();
            name.push(additional[i % additional.len()] as char);
            i /= additional.len();
        }
        self.name_index += 1;
        name
    }
}

// Exported variables only use upper case letters in their names. HLSL semantics
// are not case sensitive and may also assign special meaning to numbers.
fn upper_case_name_generator() -> &'static Mutex<NameGenerator> {
    UPPER_CASE_NAME_GENERATOR.get_or_init(|| {
        Mutex::new(NameGenerator::new(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ_",
        ))
    })
}

// Don't begin new names with the '_' character. Internal code can begin names
// with '_' without fear of renaming collisions.
fn general_name_generator() -> &'static Mutex<NameGenerator> {
    GENERAL_NAME_GENERATOR.get_or_init(|| {
        Mutex::new(NameGenerator::new(
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "_0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
        ))
    })
}

fn generate_new_name(force_upper_case: bool) -> String {
    let mut generator = if force_upper_case {
        upper_case_name_generator()
            .lock()
            .expect("upper generator lock")
    } else {
        general_name_generator()
            .lock()
            .expect("general generator lock")
    };
    loop {
        let name = generator.next_name();
        if !is_reserved_keyword(&name)
            && !used_new_names()
                .lock()
                .expect("used-name lock")
                .contains(&name)
        {
            used_new_names()
                .lock()
                .expect("used-name lock")
                .insert(name.clone());
            return name;
        }
    }
}

// mapping from original identifiers to new names.
fn generate_new_names() {
    let counts = all_id_counts()
        .lock()
        .expect("identifier count lock")
        .clone();
    let order = all_id_order()
        .lock()
        .expect("identifier order lock")
        .clone();
    let mut ordered = order;
    // Python sorted(..., key=lambda x:x[1], reverse=True) is stable.
    ordered.sort_by_key(|name| std::cmp::Reverse(*counts.get(name).unwrap_or(&0)));
    let mut generated = new_names().lock().expect("new-name lock");
    for name in ordered {
        let reserved = is_reserved_keyword(&name);
        let generated_name = if args().human_readable || reserved {
            remove_leading_annotation(&name)
        } else {
            // HLSL semantics are not case sensitive and can assign special
            // meaning to numbers. Make all exported names upper case with no
            // numbers.
            generate_new_name(name.as_bytes()[0] == b'@')
        };
        generated.insert(name, generated_name);
    }
}

const RGBA_STPQ_PATTERN: &str = r"^([rgba]{1,4}|[stpq]{1,4})$";
const RGBA_STPQ_REMAP: &[(char, char)] = &[
    ('r', 'x'),
    ('g', 'y'),
    ('b', 'z'),
    ('a', 'w'),
    ('s', 'x'),
    ('t', 'y'),
    ('p', 'z'),
    ('q', 'w'),
];

fn rgba_stpq_match(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=4).contains(&bytes.len())
        && (bytes.iter().all(|character| b"rgba".contains(character))
            || bytes.iter().all(|character| b"stpq".contains(character)))
}

fn rgba_stpq_remap(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            RGBA_STPQ_REMAP
                .iter()
                .find(|(source, _)| *source == character)
                .map(|(_, target)| *target)
                .unwrap_or(character)
        })
        .collect()
}

// minifies a single GLSL file.
#[derive(Debug)]
pub struct Minifier {
    pub tokens: Vec<Token>,
    pub exports: Arc<Mutex<HashSet<String>>>,
    pub basename: String,
}

impl Minifier {
    pub fn new(code: &str, basename: &str) -> Result<Self, MinifyError> {
        Self::new_with_exports(code, basename, default_exports())
    }

    fn new_with_exports(
        code: &str,
        basename: &str,
        exports: Arc<Mutex<HashSet<String>>>,
    ) -> Result<Self, MinifyError> {
        // parse tokens.
        let tokens = lex_code(code, exports.clone())?;
        Ok(Self {
            tokens,
            exports,
            basename: basename.to_string(),
        })
    }

    // Strips unneeded code from the tokens. Called after all Minifiers have
    // been parsed.
    pub fn strip_tokens(&mut self) {
        assert!(!args().human_readable);

        // strip comments.
        self.tokens.retain(|tok| !tok.r#type.contains("COMMENT"));

        // strip unused defines.
        self.tokens.retain(|tok| {
            tok.r#type != "DEFINE"
                || all_id_reference_counts()
                    .lock()
                    .expect("reference count lock")
                    .get(tok.define_id.as_deref().unwrap())
                    .copied()
                    .unwrap_or(0)
                    > 0
        });

        // merge whitespace.
        let unmerged = std::mem::take(&mut self.tokens);
        for mut tok in unmerged {
            if tok.r#type == "DEFINE" {
                if let Some(arglist) = tok.define_arglist.as_mut() {
                    arglist.strip_tokens();
                }
                if let Some(value) = tok.define_val.as_mut() {
                    value.strip_tokens();
                }
            }
            if tok.r#type == "DIRECTIVE" {
                if let Some(value) = tok.directive_val.as_mut() {
                    value.strip_tokens();
                }
            }
            if tok.r#type == "WHITESPACE"
                && !self.tokens.is_empty()
                && self.tokens.last().unwrap().r#type == "WHITESPACE"
            {
                self.tokens.last_mut().unwrap().value.push_str(&tok.value);
            } else {
                self.tokens.push(tok);
            }
        }
    }

    // generates rewritten glsl from our tokens.
    pub fn emit_tokens_to_rewritten_glsl<W: Write>(
        &self,
        out: &mut W,
        preserve_exported_switches: bool,
        calling_token_type: Option<&str>,
    ) -> io::Result<bool> {
        // stand-in for a null token.
        let mut lasttoken = Token::new("", "");
        let mut lasttoken_needs_whitespace = false;
        let mut is_newline = true;

        for tok in &self.tokens {
            if tok.r#type == "WHITESPACE" {
                if args().human_readable {
                    out.write_all(tok.value.as_bytes())?;
                    is_newline = tok.value.chars().last().unwrap() == '\n';
                    lasttoken_needs_whitespace = false;
                }
                continue;
            }

            let is_directive = matches!(tok.r#type.as_str(), "DEFINE" | "IFDEF" | "DIRECTIVE");
            let needs_whitespace = matches!(
                tok.r#type.as_str(),
                "FLOAT" | "INT" | "HEX" | "ID" | "DEFINED_ID"
            );
            // Adding this calling_token_type != 'DEFINE' prevents us from
            // adding a new line to stringify macros.
            if is_directive && !is_newline && calling_token_type != Some("DEFINE") {
                out.write_all(b"\n")?;
            } else if needs_whitespace && lasttoken_needs_whitespace {
                out.write_all(b" ")?;
            }

            // is_newline will be false once we output the token (unless this
            // value otherwise gets updated).
            is_newline = false;

            match tok.r#type.as_str() {
                "ID" => {
                    if rgba_stpq_match(&tok.value)
                        && lasttoken.r#type == "OP"
                        && lasttoken.value == "."
                    {
                        // convert rgba and stpq to xyzw.
                        out.write_all(rgba_stpq_remap(&tok.value).as_bytes())?;
                    } else {
                        self.write_identifier(out, &tok.value, preserve_exported_switches)?;
                    }
                }
                "DEFINE" => {
                    out.write_all(b"#define ")?;
                    self.write_identifier(
                        out,
                        tok.define_id.as_deref().unwrap(),
                        preserve_exported_switches,
                    )?;
                    if let Some(arglist) = tok.define_arglist.as_ref() {
                        is_newline = arglist.emit_tokens_to_rewritten_glsl(
                            out,
                            preserve_exported_switches,
                            Some(&tok.r#type),
                        )?;
                        assert!(!is_newline);
                    }
                    if let Some(value) = tok.define_val.as_ref() {
                        out.write_all(b" ")?;
                        is_newline = value.emit_tokens_to_rewritten_glsl(
                            out,
                            preserve_exported_switches,
                            Some(&tok.r#type),
                        )?;
                    }
                }
                "IFDEF" => {
                    out.write_all(b"#")?;
                    out.write_all(tok.ifdef_tag.as_deref().unwrap().as_bytes())?;
                    out.write_all(b" ")?;
                    self.write_identifier(
                        out,
                        tok.ifdef_id.as_deref().unwrap(),
                        preserve_exported_switches,
                    )?;
                }
                "DEFINED_ID" => {
                    out.write_all(b"defined(")?;
                    self.write_identifier(
                        out,
                        tok.defined_id.as_deref().unwrap(),
                        preserve_exported_switches,
                    )?;
                    out.write_all(b")")?;
                }
                "DIRECTIVE" => {
                    out.write_all(b"#")?;
                    if let Some(value) = tok.directive_val.as_ref() {
                        is_newline = value.emit_tokens_to_rewritten_glsl(
                            out,
                            preserve_exported_switches,
                            Some(&tok.r#type),
                        )?;
                    }
                }
                _ => out.write_all(tok.value.as_bytes())?,
            }

            // Since we preserve whitespace in human-readable mode, the newline
            // after a preprocessor directive happens automatically unless
            // human-readable is false.
            if !args().human_readable && is_directive && !is_newline {
                out.write_all(b"\n")?;
                is_newline = true;
            }

            lasttoken = Token::new(&tok.r#type, &tok.value);
            lasttoken_needs_whitespace = needs_whitespace;
        }

        Ok(is_newline)
    }

    fn write_identifier<W: Write>(
        &self,
        out: &mut W,
        identifier: &str,
        preserve_exported_switches: bool,
    ) -> io::Result<()> {
        if preserve_exported_switches
            && exported_switches()
                .lock()
                .expect("exported switch lock")
                .contains(identifier)
        {
            assert_eq!(identifier.as_bytes()[0], b'@');
            out.write_all(identifier[1..].as_bytes())?;
        } else {
            let rewritten = new_names()
                .lock()
                .expect("new-name lock")
                .get(identifier)
                .unwrap()
                .clone();
            out.write_all(rewritten.as_bytes())?;
        }
        Ok(())
    }

    fn write_exports(&self, outdir: &Path) -> io::Result<()> {
        let output_path = outdir.join(format!("{}.exports.h", self.basename));
        println!("Exporting {} <- {}", output_path.display(), self.basename);
        let mut out = File::create(output_path)?;
        out.write_all(b"#pragma once\n\n")?;
        let mut exports: Vec<String> = self
            .exports
            .lock()
            .expect("lexer export lock")
            .iter()
            .cloned()
            .collect();
        exports.sort();
        for exp in exports {
            let rewritten = new_names()
                .lock()
                .expect("new-name lock")
                .get(&exp)
                .unwrap()
                .clone();
            writeln!(out, "#define GLSL_{} \"{}\"", &exp[1..], rewritten)?;
            writeln!(out, "#define GLSL_{}_raw {}", &exp[1..], rewritten)?;
        }
        Ok(())
    }

    fn write_embedded_glsl(&self, outdir: &Path) -> io::Result<()> {
        let output_path = outdir.join(format!("{}.hpp", self.basename));
        println!("Embedding {} <- {}", output_path.display(), self.basename);
        let mut out = File::create(output_path)?;
        out.write_all(b"#pragma once\n\n")?;

        out.write_all(format!("#include \"{}.exports.h\"\n\n", self.basename).as_bytes())?;

        // emit shader code.
        let (root, ext) = splitext(&self.basename);
        let cpp_name = if ext != ".glsl" {
            format!("{}_{}", root, ext.strip_prefix('.').unwrap_or(""))
        } else {
            root.to_string()
        };
        out.write_all(b"namespace rive {\n")?;
        out.write_all(b"namespace gpu {\n")?;
        out.write_all(b"namespace glsl {\n")?;
        if args().msvc {
            // MSVC cannot compile raw shaders as strings because of an internal
            // string length limit. There is, however, no limit on arrays, so
            // instead write it out as an array of individual characters.
            writeln!(out, "const char {}[] = {{", cpp_name)?;
            out.write_all(b"   ")?;

            let mut code_io = io::Cursor::new(Vec::<u8>::new());
            self.emit_tokens_to_rewritten_glsl(&mut code_io, false, None)?;
            let code = String::from_utf8(code_io.into_inner())
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;

            let mut out_ch_count = 0;
            for character in code.chars() {
                // use repr to escape the characters (but handle single quote
                // manually, since Python repr emits "'" for it).
                if character == '\'' {
                    out.write_all(b" '\\'',")?;
                } else {
                    write!(out, " {},", python_repr_char(character))?;
                }
                out_ch_count += 1;
                const CHAR_COUNT_PER_LINE: usize = 12;
                if out_ch_count % CHAR_COUNT_PER_LINE == 0 {
                    out.write_all(b"\n   ")?;
                }
            }
            // Null-terminate so the array is a valid C-string when passed to
            // APIs like ostream::operator<<(const char*), which read until '\0'.
            out.write_all(b" '\\0'\n};\n")?;
        } else {
            // For non-MSVC outputs just output as a raw string.
            write!(out, "const char {}[] = R\"===(", cpp_name)?;
            let is_newline = self.emit_tokens_to_rewritten_glsl(&mut out, false, None)?;
            if !is_newline {
                out.write_all(b"\n")?;
            }
            out.write_all(b")===\";\n")?;
        }
        out.write_all(b"} // namespace glsl\n")?;
        out.write_all(b"} // namespace gpu\n")?;
        out.write_all(b"} // namespace rive")?;
        Ok(())
    }

    fn write_offline_glsl(&self, outdir: &Path) -> io::Result<()> {
        let (root, ext) = splitext(&self.basename);
        let output_path = outdir.join(format!("{}.minified{}", root, ext));
        // Preserve the pinned source's visible f{output_path} print typo.
        println!("Minifying f{} <- {}", output_path.display(), self.basename);
        let mut out = File::create(output_path)?;
        self.emit_tokens_to_rewritten_glsl(&mut out, true, None)?;
        Ok(())
    }
}

fn splitext(basename: &str) -> (&str, &str) {
    // Source os.path.splitext behavior for the basename strings passed here.
    let dot = basename.rfind('.');
    match dot {
        Some(index) if index > 0 => (&basename[..index], &basename[index..]),
        _ => (basename, ""),
    }
}

fn python_repr_char(character: char) -> String {
    // Python repr(char), for the character classes emitted by the minifier.
    match character {
        '\n' => "'\\n'".to_string(),
        '\r' => "'\\r'".to_string(),
        '\t' => "'\\t'".to_string(),
        '\\' => "'\\\\'".to_string(),
        '\0' => "'\\x00'".to_string(),
        character if character.is_ascii_control() => format!("'\\x{:02x}'", character as u32),
        character => format!("'{character}'"),
    }
}

// parse all GLSL files before renaming. This keeps the renaming consistent across files.
pub fn run(arguments: impl IntoIterator<Item = String>) -> Result<(), MinifyError> {
    let parsed = parse_args(arguments)?;
    ARGS.set(parsed.clone())
        .map_err(|_| MinifyError::new("arguments were already parsed"))?;
    configure_ply_path(parsed.ply_path.as_deref());

    let mut minifiers = Vec::with_capacity(parsed.files.len());
    for file in &parsed.files {
        let code = fs::read_to_string(file).map_err(|error| MinifyError::new(error.to_string()))?;
        let basename = file
            .file_name()
            .ok_or_else(|| MinifyError::new("input path has no basename"))?
            .to_string_lossy()
            .to_string();
        minifiers.push(Minifier::new(&code, &basename)?);
    }
    generate_new_names();

    // minify all GLSL files.
    if !parsed.outdir.exists() {
        fs::create_dir_all(&parsed.outdir).map_err(|error| MinifyError::new(error.to_string()))?;
    }
    for mut minifier in minifiers {
        if !parsed.human_readable {
            minifier.strip_tokens();
        }
        minifier
            .write_exports(&parsed.outdir)
            .map_err(|error| MinifyError::new(error.to_string()))?;
        minifier
            .write_embedded_glsl(&parsed.outdir)
            .map_err(|error| MinifyError::new(error.to_string()))?;
        minifier
            .write_offline_glsl(&parsed.outdir)
            .map_err(|error| MinifyError::new(error.to_string()))?;
    }
    Ok(())
}

pub fn main() -> Result<(), MinifyError> {
    run(env::args())
}
