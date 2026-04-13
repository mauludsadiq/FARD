# FARD — ISO EBNF Grammar

**13 Apr 2026 · v1.7.0 · updated for media decoders, transforms, and integration packages**

This document covers the **fardrun** production dialect. All stdlib module contents
verified directly from source — not inferred from prior documentation.

-----

## Lexical

```ebnf
alpha           = "A" | … | "Z" | "a" | … | "z" ;
digit           = "0" | … | "9" ;

ident_start     = alpha | "_" ;
ident_continue  = alpha | digit | "_" ;
ident           = ident_start , { ident_continue } ;

integer         = digit , { digit } ;
(* Multi-digit integer cannot have leading "0".
   Negative integers are handled by unary "-" at the expression level. *)

float           = digit , { digit } , "." , digit , { digit } , [ sci_exp ] ;
sci_exp         = ( "e" | "E" ) , [ "+" | "-" ] , digit , { digit } ;
(* Float literals lex to Tok::Float, evaluate to Val::Float(f64).
   Scientific notation is supported: 1.5e10, 2.3E-4.
   Use float.* builtins for arithmetic. Do not use == on float results. *)

escape          = "\\n" | "\\t" | "\\\"" | "\\\\" ;
(* Only these four escapes are valid. "\r" produces "bad escape: \r". *)

string_char     = ? any char except " and \ ? | escape ;
string          = "\"" , { string_char | interp_expr } , "\"" ;
interp_expr     = "${" , expr , "}" ;
(* String interpolation is a native token (Tok::StrInterp).
   "${expr}" is valid inside any double-quoted string. *)

backtick_string = "`" , { ? any char except ` ? } , "`" ;
(* Backtick strings have NO escape processing — all characters literal.
   Useful for raw strings containing backslashes or double quotes. *)

whitespace      = { " " | "\t" | "\r" | "\n" } ;
comment         = "//" , { ? any char except newline ? } , ( "\n" | ? EOF ? )
                | "#"  , { ? any char except newline ? } , ( "\n" | ? EOF ? ) ;

(* Two-char operators — lexed in this order: *)
sym2            = "!=" | "==" | "<=" | ">=" | "&&" | "->" | "=>" | "|>" ;
(* NOTE: "!=" IS implemented — use it directly. Earlier docs were wrong. *)
(* "||" is lexed as a dedicated Tok::OrOr token, not Tok::Sym("||") *)

sym3            = "..." ;

(* Single-char symbols: *)
sym1            = "(" | ")" | "{" | "}" | "[" | "]" | "," | ":" | "."
                | "+" | "-" | "*" | "/" | "=" | "%" | "|" | "<" | ">"
                | "?" | "!" ;
(* "%" — modulo operator, valid at the mul_expr level *)
(* "!" — unary logical not, valid at the unary_expr level *)
(* "|" — single pipe char, distinct from "|>" pipe operator *)

(* Keywords — exactly as registered in source: *)
keyword         = "let" | "in" | "fn" | "if" | "then" | "else"
                | "import" | "as" | "export" | "match" | "using"
                | "test" | "while" | "return"
                | "true" | "false" | "null" ;
(* "test", "while", "return" are reserved keywords in the lexer.
   "int" is effectively reserved as an import alias —
   import("std/int") as int is rejected. Use any other alias. *)
```

-----

## Module (Top-Level)

```ebnf
module          = { module_item } ;

(* parse_module dispatch order — literal from source:
   1) "test"    → test_item
   2) "a"       → type_decl  (type declaration)
   3) "import"  → import_item
   4) "artifact"→ artifact_item
   5) "export"  → export_item
   6) "fn"      → fn_item
   7) "let"     → try parse_expr first (handles let…in);
                  if that fails, backtrack and parse let_item
   8) anything else → expr
*)

module_item     = test_item
                | type_decl
                | import_item
                | artifact_item
                | export_item
                | fn_item
                | let_item
                | expr ;

test_item       = "test" , string , "{" , fn_block_inner , "}" ;

(* Type declarations — two forms: *)
type_decl       = "a" , ident , "is" , ( record_type_body | sum_type_body ) ;

record_type_body = "{" , { type_field , [ "," ] } , "}" ;
type_field       = ident , ":" , ident ;

sum_type_body    = variant , { "or" , variant } ;
variant          = ident , [ "(" , { type_field , [ "," ] } , ")" ] ;

(* Examples:
   a Point is { x: Int, y: Int }
   a Shape is Circle(r: Int) or Rect(w: Int, h: Int) *)

import_item     = "import" , "(" , string , ")" , "as" , ident ;
(* "as alias" is mandatory. Path must be a string literal. *)

artifact_item   = "artifact" , ident , "=" , sha256_string ;
sha256_string   = string ;
(* run_id string MUST start with "sha256:" or parse fails. *)

export_item     = "export" , "{" , ident , { "," , ident } , [ "," ] , "}" ;

(* Module-level let binds IDENT or destructuring pattern *)
let_item        = "let" , ( ident | obj_pat | list_pat ) , "=" , expr ;

fn_item         = "fn" , ident , "(" , [ fn_param , { "," , fn_param } ] , ")"
                , [ "->" , type ]
                , "{" , fn_block_body ;

fn_param        = pat , [ ":" , type ] ;

(* fn_block_body and fn_block_inner:
   Zero or more "let" bindings, then a tail expression, then "}" (for body).
   
   Three forms of binding are supported inside a block:
   1. let x = e         — block-local binding, no "in"
   2. let x = e in body — inline let-in expression, terminates the binding sequence
   3. let x = e | next  — sequencing: binds x, then continues with next

   Additionally "expr | expr" at tail position acts as sequencing:
   desugars to "let _ = lhs in rhs".

   CONSTRAINT: "let" may NOT appear as the direct body of an if/else
   branch — extract to a named helper function. *)

fn_block_body   = fn_block_inner , "}" ;
fn_block_inner  = { let_binding } , seq_expr ;

let_binding     = "let" , ( ident | obj_pat | list_pat ) , "=" , expr ,
                  ( "in" , expr               (* terminates — inline let-in *)
                  | "|" , fn_block_inner      (* sequencing continuation *)
                  | (* nothing — block binding, continues *)
                  ) ;

seq_expr        = expr , { "|" , expr } ;
(* Single "|" (not "||") between expressions acts as sequencing:
   "e1 | e2" desugars to "let _ = e1 in e2" *)
```

-----

## Expressions

```ebnf
expr            = using_expr
                | match_expr
                | let_expr
                | while_expr
                | return_expr
                | if_expr
                | infix_expr ;

using_expr      = "using" , pat , "=" , expr , "in" , expr ;

match_expr      = "match" , expr , match_arms ;
match_arms      = "{" , [ match_arm , { "," , match_arm } , [ "," ] ] , "}" ;
match_arm       = pat , [ "if" , expr ] , "=>" , expr ;

(* Expression let — binds a pattern, requires "in" *)
let_expr        = "let" , pat , "=" , expr , "in" , expr ;

(* while takes three bare expressions: init state, condition fn, step fn *)
while_expr      = "while" , expr , expr , expr ;

return_expr     = "return" , expr ;
(* "return" is only valid inside a fn body *)

(* if/then/else — both branches required.
   The "then" branch MAY be a { } block if the token after "{" is
   "let", "return", or "}" — the parser disambiguates from record literals. *)
if_expr         = "if" , expr , "then" , ( block_expr | expr ) , "else" , expr ;
block_expr      = "{" , fn_block_inner , "}" ;

(* Infix operators — precedence-climbing, all left-associative:
   Prec 1 (lowest): || ??  (null-coalescing: x ?? default returns x if x != null, else default)
   Prec 2:          &&
   Prec 3:          == != < > <= >=
   Prec 4:          + -
   Prec 5 (highest):* / %
*)
infix_expr      = unary_expr , { infix_op , unary_expr } ;
infix_op        = "||" | "??" | "&&"
                | "==" | "!=" | "<" | ">" | "<=" | ">="
                | "+" | "-"
                | "*" | "/" | "%" ;

(* Both "-" and "!" are unary prefix operators *)
unary_expr      = { "-" | "!" } , pipe_expr ;

(* Pipe inserts lhs as first arg: x |> f(a, b) → f(x, a, b) *)
pipe_expr       = postfix_expr , { "|>" , postfix_expr } ;

(* Postfix: "?", ".field", call "(…)", index "[…]"
   Index "[expr]" is NOT parsed when:
   - the base expression is a literal (Int, Float, Bool, Str, Null, List, Rec)
   - there is a newline between the base expression and "["
   This resolves the [[...]] fn-tail ambiguity — [[...]] now works correctly. *)
postfix_expr    = primary_expr , { postfix_op } ;

postfix_op      = "?" "."           (* safe navigation: e?.f -> null if e==null, else e.f *)
                | "?"
                | "." , ident
                | "(" , arg_list , ")"
                | "[" , expr , "]" ;

(* Call arguments may be positional or named. Cannot mix in one call. *)
arg_list        = [ pos_args | named_args ] ;
pos_args        = expr , { "," , expr } , [ "," ] ;
named_args      = named_arg , { "," , named_arg } , [ "," ] ;
named_arg       = ident , ":" , expr ;

primary_expr    = lambda_expr
                | multi_lambda
                | anon_fn_expr
                | interp_string
                | backtick_string
                | float
                | literal
                | ident
                | "(" , expr , ")"
                | list_lit
                | rec_lit ;

lambda_expr     = ident , "=>" , expr ;
multi_lambda    = "(" , [ ident , { "," , ident } ] , ")" , "=>" , expr ;
anon_fn_expr    = "fn" , "(" , [ pat , { "," , pat } ] , ")" ,
                  "{" , fn_block_inner , "}" ;

interp_string   = "\"" , { string_char | "${" , expr , "}" } , "\"" ;
backtick_string = "`" , { ? any char except backtick ? } , "`" ;

literal         = integer | "true" | "false" | "null" ;

list_lit        = "[" , [ expr , { "," , expr } , [ "," ] ] , "]" ;

rec_lit         = "{" , [ rec_spread | rec_kv , { "," , rec_kv } , [ "," ] ] , "}" ;
rec_spread      = "..." , postfix_expr , { "," , rec_kv } ;
(* { ...base, key: val } merges base with overrides. Last write wins. No import needed. *)
rec_kv          = rec_key , ":" , expr
                | "[" , expr , "]" , ":" , expr ;  (* computed key: { [k]: v } *)
rec_key         = ident | keyword | string ;  (* static keys only; use [expr] form for dynamic keys *)
(* Keywords are valid record keys: { ok: true, if: "allowed", return: 42 } *)
```

-----

## Patterns

```ebnf
pat             = "true" | "false" | "null"
                | "_"                          (* wildcard — Tok::Ident("_"), no binding *)
                | integer | string
                | obj_pat | list_pat
                | bind_name ;

bind_name       = ident | keyword ;
(* Keywords are valid bind names except true/false/null.
   Duplicate bind names within a single pattern are a parse error
   (pat_reject_duplicate_binds is called after parsing). *)

obj_pat         = "{" , [ obj_pat_body ] , "}" ;
obj_pat_body    = { obj_field , "," } , obj_field , [ "," ]    (* exact fields *)
                | { obj_field , "," } , obj_field , "," , "..." , ident  (* with rest *)
                | "..." , ident ;                              (* rest only *)
obj_field       = ident , ":" , pat ;

(* NOTE: rest capture uses "..." ident directly — NOT ". ident" as in old docs *)

list_pat        = "[" , [ list_pat_body ] , "]" ;
list_pat_body   = pat , { "," , pat } , [ "," , "..." , ident ]
                | "..." , ident ;
```

-----

## Types

```ebnf
type            = builtin_type
                | list_type
                | rec_type
                | func_type
                | named_type
                | "(" , type , ")" ;

builtin_type    = "Int" | "String" | "Bool" | "Unit" | "Dynamic" ;
(* These five are matched by name before falling through to named_type *)

list_type       = "List" , "<" , type , ">" ;

rec_type        = "Rec" , "{" , [ rec_type_field , { "," , rec_type_field } ] , "}" ;
rec_type_field  = ident , ":" , type ;
(* NOTE: Rec uses BRACES not angle brackets: Rec { x: Int, y: Int } *)

func_type       = "Func" , "(" , [ type , { "," , type } ] , ")" , "->" , type ;

(* Any other ident — optionally followed by "<" type_args ">" *)
named_type      = ident , [ "<" , type , { "," , type } , ">" ] ;
(* Examples: MyType, Option<Int>, Result<Text, Int> *)

(* Types are optional everywhere — FARD is dynamically typed at runtime.
   Type annotations on fn params and return use ":" and "->" syntax.
   No "Tuple", "Option", or "Result" built-in type constructors exist —
   use named_type for those. *)
```

-----

## Runtime Value Types (fardrun `Val`)

|Variant                       |Description                                                          |
|------------------------------|---------------------------------------------------------------------|
|`Unit`                        |unit / null                                                          |
|`Bool(bool)`                  |true / false                                                         |
|`Int(i64)`                    |64-bit signed integer                                                |
|`Float(f64)`                  |Produced only by `json.decode` on JSON floats or `std/math` constants|
|`Text(String)`                |UTF-8 text (note: field is `Text`, not `Str`)                        |
|`Bytes(Vec<u8>)`              |Raw bytes — produced by `std/bytes` operations. Not used for floats. |
|`List(Vec<Val>)`              |Ordered list                                                         |
|`Record(BTreeMap<String,Val>)`|Record with sorted string keys                                       |
|`Err { code, data }`          |Error value with string code and data payload                        |
|`Func`                        |Closure — params + body + captured env                               |
|`Builtin`                     |Native stdlib function pointer                                       |
|`BoundMethod(receiver, fn)`   |Method bound to a receiver value                                     |
|`Chan`                        |Channel for concurrent communication (`std/chan`)                    |
|`Mtx`                         |Mutex (`std/mutex`)                                                  |
|`Big(BigInt)`                 |Arbitrary-precision integer (`std/bigint`)                           |
|`Promise`                     |Async promise (`std/promise`)                                        |

**Float representation:** `Val::Float(f64)` is used consistently. Float literals, JSON-decoded floats, and `std/float` results all produce `Val::Float`. Int+float arithmetic is automatically promoted. Note: `==` on floats uses exact IEEE 754 bit comparison — `0.1 + 0.2 == 0.3` is `true` because both sides have the same bit pattern, but accumulated floating-point errors across many operations may produce unexpected results. Use tolerance checks for computed values.

**`Val::Text` not `Val::Str`:** The field is named `Text` in the current source,
not `Str` as in earlier versions. The `type_name()` method returns `"text"` for
text values.

-----

## Stdlib Modules — Complete Reference

All modules verified from source. Only `std/artifact.ref` and `std/artifact.derive`
remain as `Unimplemented` — all other registered builtins are functional.

### std/str

`len`, `trim`, `split_lines`, `lower`, `to_lower`, `toLower`, `upper`, `to_upper`, `toUpper`, `concat`, `split`,
`contains`, `starts_with`, `ends_with`, `replace`, `slice`, `format`,
`from_int`, `from_float`, `from`, `join`, `pad_left`, `pad_right`, `repeat`,
`index_of`, `chars`

> **Name note:** `lower` and `upper` are the correct names. `to_lower`/`to_upper`
> do not exist and will produce a field-not-found error at runtime.

### std/list

`len`, `range`, `repeat`, `concat`, `group_by`, `fold`, `map`, `filter`,
`get`, `head`, `tail`, `last`, `append`, `zip`, `reverse`, `flatten`, `set`,
`any`, `all`, `find`, `find_index`, `take`, `drop`, `flat_map`, `par_map`,
`zip_with`, `chunk`, `sort_by`, `sort_by_int_key`, `sort_int`,
`dedupe`, `dedupe_sorted_int`, `hist_int`

### std/rec

`empty`, `keys`, `values`, `has`, `get`, `getOr`, `getOrErr`, `set`,
`remove`, `merge`, `select`, `rename`, `update`

> **`rec.get` behavior:** returns `Unit` (null) for missing keys — silent, no error.
> Use `rec.has(r, key)` before `rec.get` when key presence is uncertain.
> Use `rec.getOr(r, key, default)` for safe access with a fallback value.
> Use `rec.getOrErr(r, key)` to get an error value on missing key.

### std/map

`get`, `set`, `keys`, `values`, `has`, `delete`, `entries`, `new`, `from_entries`

> `std/record` — aliases `std/rec`. Both work identically. `import("std/record") as rec` is valid.

### std/set

`new`, `add`, `remove`, `has`, `union`, `intersect`, `diff`, `to_list`,
`from_list`, `size`

### std/int

`add`, `eq`, `parse`, `pow`, `to_hex`, `to_bin`, `mul`, `div`, `sub`,
`abs`, `min`, `max`, `to_text`, `to_string`, `from_text`, `neg`, `clamp`, `mod`,
`lt`, `gt`, `le`, `ge`, `to_str_padded`

> **Note:** `import("std/int") as int` works correctly. `int` is not a reserved alias.

### std/float

`from_int`, `to_int`, `from_text`, `to_text`, `parse`, `add`, `sub`, `mul`, `div`,
`exp`, `ln`, `sqrt`, `pow`, `abs`, `neg`, `floor`, `ceil`, `round`,
`lt`, `gt`, `le`, `ge`, `eq`, `nan`, `inf`, `is_nan`, `is_finite`, `min`, `max`

### std/math

`abs`, `min`, `max`, `pow`, `sqrt`, `floor`, `ceil`, `round`,
`log`, `log2`, `log10`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `exp`

Constants: `pi`, `e`, `inf` (registered as `Val::Float` values, not functions)

### std/bigint

`from_int`, `from_str`, `to_str`, `add`, `sub`, `mul`, `div`, `mod`,
`pow`, `eq`, `lt`, `gt`

### std/bits

`band`, `bor`, `bxor`, `bnot`, `bshl`, `bshr`, `popcount`

> `std/bits` is used internally by `image-core/src/crc32` for CRC-32 computation.
> `bits.band(n, 4294967295)` is the canonical u32 mask. `bits.bshr(n, k)` is logical right shift.
> All bit operations accept `Int` and return `Int`.

### std/json

`encode`, `decode`, `canonicalize`

> `encode_pretty` was present in earlier versions — verify before using.

### std/bytes

`concat`, `to_str`, `to_string`, `from_string`, `len`, `get`, `of_list`, `to_list`,
`of_str`, `merkle_root`, `eq`, `to_hex`, `to_base64`, `from_hex`, `from_base64`, `slice`

### std/codec

`base64url_encode`, `base64url_encode_hex`, `base64url_decode`,
`hex_encode`, `hex_decode`

### std/base64

`encode`, `decode`

> Returns `Val::Bytes`. Use `bytes.to_str()` to convert to text.

### std/csv

`parse`, `encode`

### std/hash

`sha256_text`, `sha256_bytes`

Returns hex string prefixed `sha256:`.

### std/crypto

`hmac_sha256(key_hex_or_bytes, msg)`, `ed25519_verify(pk_hex, msg_hex, sig_hex)`,
`sha512`, `aes_encrypt`, `aes_decrypt`, `merkle_root`

> `ed25519_verify` is fully functional. Backed by `ed25519-dalek` directly in fardrun.

### std/uuid

`v4`, `validate`

### std/rand

`uuid_v4`

### std/random

`uuid_v4`, `int`, `float`

> Non-deterministic oracle — results recorded in trace. `int` returns random int, `float` returns random float in [0,1].

### std/avro

> Implemented via `packages/avro-core`. Import by path — see Integration Packages.

### std/duckdb

> Implemented via `packages/duckdb-core`. Import by path — see Integration Packages.

### std/parquet

> Implemented via `packages/parquet-core`. Import by path — see Integration Packages.

### std/postgres

> Implemented via `packages/postgres-core`. Import by path — see Integration Packages.

### std/wasm

> Implemented via `packages/wasm-core`. Import by path — see Integration Packages.

### std/watch

> Implemented via `packages/watch-core`. Import by path — see Integration Packages.

### std/ws

> Implemented via `packages/ws-core`. Import by path — see Integration Packages.

### std/xlsx

> Implemented via `packages/xlsx-core`. Import by path — see Integration Packages.

### std/datetime

`now`, `format`, `parse`, `add`, `diff`, `field`

### std/time

`now`, `parse`, `format`, `add`, `sub`

`Duration` record: `{ ms, sec, min }` — each is a builtin function.

### std/io

`read_file`, `write_file`, `append_file`, `read_lines`, `file_exists`,
`delete_file`, `read_stdin`, `read_stdin_lines`, `list_dir`, `make_dir`

### std/fs

`read_text`, `write_text`, `exists`, `read_dir`, `stat`, `delete`, `make_dir`

### std/path

`base`, `dir`, `ext`, `isAbs`, `join`, `joinAll`, `normalize`

### std/env

`get`, `args`

### std/process

`spawn`, `exit`

### std/http

`get`, `post`, `request`

### std/net

`serve(port, handler_fn)` — blocking HTTP server.

`connect(host, port) -> { ok: conn_id } | { err: text }` — opens a TCP connection. Returns an integer connection handle.

`send(conn_id, bytes) -> { ok: null } | { err: text }` — writes bytes to an open connection. Accepts `Bytes` or `List(Int)`.

`recv(conn_id, max_bytes) -> { ok: Bytes } | { err: text }` — reads up to `max_bytes` from an open connection.

`close_conn(conn_id) -> Unit` — closes and removes the connection.

> TCP client primitives are used by `postgres-core` and `ws-core`. Connection handles are integers assigned sequentially per process.

### std/promise

`spawn`, `await`, `spawn_ordered`

> `spawn_ordered(fns)` spawns a list of zero-argument functions concurrently and joins in spawn order. Returns a list of results with guaranteed ordering — same digest across runs.

### std/chan

`new`, `send`, `recv`, `try_recv`, `close`, `is_closed`

### std/mutex

`new`, `lock`, `unlock`, `with_lock`

### std/sqlite

`open`, `exec`, `query`, `close`

> `sqlite.open(path)` — returns a db handle record. Use `":memory:"` for in-memory.
> `sqlite.exec(db, sql)` — execute DDL/DML, returns db handle.
> `sqlite.query(db, sql)` — returns list of records.

### std/menv

`new`, `set`, `get`, `has`, `child`, `call_eval`, `apply_closure`

> Mutable environment used for self-hosting bootstrap. `menv.set` returns `Unit`.
> `menv.call_eval(body, env, eval_fn)` — evaluates FIR body natively.

### std/async

`sleep`, `spawn`, `await`, `all`, `resolved`, `rejected`, `yield`, `race`, `timeout`

> Stub implementation — operations complete synchronously in current runtime.

### std/cell

`new`, `get`, `set`

### std/option

`none`/`None`, `some`/`Some`, `is_none`/`isNone`, `is_some`/`isSome`,
`from_nullable`/`fromNullable`, `to_nullable`/`toNullable`,
`map`, `and_then`/`andThen`, `unwrap_or`/`unwrapOr`,
`unwrap_or_else`/`unwrapOrElse`, `to_result`/`toResult`

> Both snake_case and camelCase aliases are registered for all functions.

### std/result

`ok`, `err`, `and_then`/`andThen`, `unwrap_ok`, `unwrap_err`, `unwrap`,
`unwrap_or`, `is_ok`, `is_err`, `map`, `map_err`, `or_else`

### std/null

`isNull`, `coalesce`, `guardNotNull`

### std/type

`of`

### std/cast

`float`, `int`, `text`

> **`cast.text` converts integers to Unicode characters** — `cast.text(65)` returns `"A"`, not `"65"`.
> Use `str.from(n)` to convert a number to its string representation.
> `cast.text` is correct for Unicode codepoint conversion only.

### std/re

`is_match`, `find`, `find_all`, `split`, `replace`

### std/eval

`eval`

### std/ast

`parse`

### std/ffi

`open`/`load`, `call`, `call_pure`, `call_checked`, `call_str`, `close`

> `call_checked` calls the function twice and verifies identical results (determinism check). Emits `ffi_checked` trace event on success. `call` emits `ffi_oracle` boundary event.

### std/compress

`gzip`, `gunzip`

> Note: registered as `gzip`/`gunzip`, not the previously documented
> `gzip_compress`/`gzip_decompress`. `zstd` is not present.

### std/graph

`of`, `ancestors`, `leaves`, `to_dot`

> Note: registered API differs from previously documented
> (`add_node`, `add_edge`, `bfs`, `dfs`, `shortest_path`, `topo_sort` are not present).

### std/linalg

`zeros`, `eye`, `dot`, `norm`, `vec_add`, `vec_sub`, `vec_scale`,
`matvec`, `matmul`, `mat_add`, `mat_scale`, `transpose`, `eigh`

### std/flow

`id`, `pipe`, `tap`

### std/grow

`append`, `merge`, `unfold_tree`, `unfold`

### std/sembit

`partition`

### std/artifact

`import`, `emit`

> `ref` and `derive` are registered as `Unimplemented("std/trace.ref")` /
> `Unimplemented("std/trace.derive")` — the only two remaining unimplemented
> builtins in the entire stdlib.

### std/witness

`self_digest`, `deps`, `verify`, `verify_chain`

### std/trace

`emit`, `info`, `warn`, `error`, `span`

### std/png

`red_1x1`

### std/cli

`args`

-----

## Media Stack Modules

The following workspace packages ship with FARD. Import by relative path — they are not in the package registry.

### media-core/src/types

`image_spec(width, height, encoding)` — returns `{ width, height, encoding }` spec record used by raster values.
`video_spec(width, height, fps_num, fps_den, encoding)` — returns `{ width, height, fps_num, fps_den, encoding }` spec record used by video values.

### media-core/src/bytes

`concat(a, b)`, `concat_many(list)` — byte concatenation.
`u32le(n)` — 4-byte little-endian unsigned int.
`u16le(n)` — 2-byte little-endian unsigned int.
`i16le(n)` — 2-byte little-endian signed int.
`ascii_bytes(str)` — encodes an ASCII string to bytes.

All functions return `Bytes`.

### media-core/src/artifact

`write_receipt(path, payload_bytes, meta)` — writes `path.receipt.json` containing the SHA-256 digest of `payload_bytes` merged with `meta`. Returns the enriched metadata record. Used by all media exporters.

### image-core/src/color

`clamp8(x) -> Int` — clamps integer to [0, 255].
`rgb(r, g, b) -> Record` — `{ r: Int, g: Int, b: Int }` with values clamped via `clamp8`.

### image-core/src/raster

`make(width, height, pixel_fn) -> Record`

Calls `pixel_fn(x, y)` for every position in `[0, width) × [0, height)`, row-major. Returns:

```ebnf
raster_value    = "{" , "spec" , ":" , image_spec_record , "," ,
                  "pixels" , ":" , pixel_list , "}" ;
image_spec_rec  = "{" , "width" , ":" , integer , "," ,
                  "height" , ":" , integer , "," ,
                  "encoding" , ":" , string , "}" ;
pixel_list      = "[" , { rgb_record , [ "," ] } , "]" ;
rgb_record      = "{" , "r" , ":" , integer , "," ,
                  "g" , ":" , integer , "," ,
                  "b" , ":" , integer , "}" ;
```

### image-core/src/draw

`gradient(width, height) -> raster_value` — produces an RGB gradient raster. R increases left-to-right, G increases top-to-bottom, B is fixed at 128.

### image-core/src/ppm

`ppm_bytes(width, height, pixels) -> Bytes` — P6 binary PPM encoder. Header: `"P6\n{width} {height}\n255\n"` followed by raw RGB bytes.

`ppm_header(width, height) -> Bytes`, `ppm_pixel_bytes(pixels) -> Bytes` also exported.

### image-core/src/png

`png_bytes(width, height, pixels) -> Bytes` — complete PNG encoder. Encoding: 8-bit RGB (colour type 2).

Chunk layout:

```
PNG signature   8 bytes: 137 80 78 71 13 10 26 10
IHDR            width u32be, height u32be, bit_depth=8, colour_type=2,
                compression=0, filter=0, interlace=0
IDAT            zlib level-0 stored blocks over filter-0 scanlines
IEND            empty body
```

Each chunk: `u32be(data_len) || tag_bytes || data || u32be(crc32(tag || data))`.
CRC-32 computed by `image-core/src/crc32` using `std/bits`.
IDAT payload produced by `image-core/src/zlib0`.

`be32(n)`, `chunk(tag, data)`, `scanlines(width, height, pixels)` also exported.

### image-core/src/zlib0

`zlib_bytes(bs) -> Bytes` — zlib level-0 (no compression). Handles input of any size via multi-block framing.

Format:

```
CMF=0x78, FLG=0x01
{ stored_block }+         one block per 65535 bytes of input
adler32_be(full_input)    4-byte big-endian Adler-32
```

Each stored block: `flag(1 byte) || u16le(len) || u16le(65535 - len) || data`. Final block flag = 1, all preceding blocks flag = 0.

`stored_block(flag, bs)`, `zlib_blocks(bs)`, `u16le_bytes(n)` also exported.

### image-core/src/adler32

`adler32_int(bs) -> Int` — Adler-32 checksum. `a` starts at 1, `b` at 0; both reduced mod 65521 per byte. Result: `b * 65536 + a`.
`adler32_be_bytes(bs) -> Bytes` — 4-byte big-endian encoding of the checksum.

`mod65521(n)`, `adler32_parts(bs)` also exported.

### image-core/src/crc32

`crc32_bytes(bs) -> Int` — CRC-32 (polynomial 0xEDB88320). Initial value `0xFFFFFFFF`, final XOR `0xFFFFFFFF`. Implemented with `std/bits`: `bits.bxor`, `bits.band`, `bits.bshr`.
`crc32_be_bytes(bs) -> Bytes` — 4-byte big-endian encoding.

`xor_byte(a, b)`, `xor_u32(a, b)` also exported.

### audio-core/src/pcm

`encode_pcm_s16le(samples) -> Bytes` — encodes `List(Float)` in [-1.0, 1.0] to 16-bit signed little-endian PCM. Each sample: `floor(clamp_unit(s) * 32767.0)` cast to `Int`, then encoded as `i16le`.

### audio-core/src/synth

`sine(freq, duration_s, sample_rate) -> List(Float)` — generates `floor(duration_s * sample_rate)` float samples of a sine wave at `freq` Hz.

### audio-core/src/export

`write_wav(path, title, sample_rate, channels, samples)` — encodes samples to PCM S16LE, wraps in WAV container, emits file via `artifact_std.emit`, writes `.receipt.json`. Receipt includes `artifact_type`, `encoding`, `sample_rate`, `channels`, `byte_len`, `duration_seconds`.

### video-core/src/frame

`make(index, pts_num, pts_den, raster) -> Record` — `{ index, pts_num, pts_den, raster }`.
`test_pattern(width, height, index) -> Record` — gradient raster frame at the given index.

### video-core/src/rawvid

`rawvid_bytes(width, height, fps_num, fps_den, frames) -> Bytes` — encodes to FARDVID1 binary format.

```ebnf
rawvid_file     = magic , u32le , u32le , u32le , u32le , u32le , { frame_rgb } ;
(* magic:      ascii "FARDVID1"  (8 bytes)                        *)
(* u32le[0]:   width                                              *)
(* u32le[1]:   height                                             *)
(* u32le[2]:   fps_num                                            *)
(* u32le[3]:   fps_den                                            *)
(* u32le[4]:   frame_count                                        *)
(* frame_rgb:  width * height * 3 bytes, row-major packed RGB     *)
(* total header size: 28 bytes                                    *)
```

`frame_rgb_bytes(frame) -> Bytes` also exported.

### video-core/src/rawvid_decode

`header(bs) -> Record` — parses bytes 8–27: `{ width, height, fps_num, fps_den, frames }`.
`frame_byte_len(h) -> Int` — `h.width * h.height * 3`.
`frame_offset(h, index) -> Int` — `28 + index * frame_byte_len(h)`.
`frame_bytes(bs, index) -> Bytes` — extracts raw RGB bytes for frame at `index`.

`u32le_at(bs, off)` also exported.

### video-core/src/timeline

`make(width, height, fps_num, fps_den, frames) -> Record` — `{ spec: video_spec(...), frames: List }`.
`test_pattern_video(width, height, fps_num, fps_den, count) -> Record` — builds a video of `count` gradient frames.

### video-core/src/mux

`av_pair(video, audio) -> Record` — `{ video, audio }`.
`with_audio(video, wav_artifact) -> Record` — `{ video, audio: wav_artifact }`.

### video-core/src/avbundle

`make(title, video_artifact, audio_artifact) -> Record` — `{ artifact_type: "video/x-fard-avbundle", title, video, audio }`.

### video-core/src/mp4_manifest

`make(title, video_artifact, audio_artifact) -> Record` — MP4 transcode manifest. Container: `mp4`. Codecs: H.264 (video), AAC (audio). Carries full path, dimension, fps, frame count, and duration metadata from the source artifacts.

### video-core/src/webm_manifest

`make(title, video_artifact, audio_artifact) -> Record` — WebM transcode manifest. Container: `webm`. Codecs: VP9 (video), Opus (audio). Same metadata shape as `mp4_manifest`.

### video-core/src/transcode_contract

`make(kind, input_artifact, output_artifact_type, container, video_codec, audio_codec) -> Record` — base transcode contract. `bridge_state` is `"contract_only"`.

`rawvid_wav_to_mp4(input_artifact) -> Record` — contract targeting `video/mp4`, H.264/AAC.
`rawvid_wav_to_webm(input_artifact) -> Record` — contract targeting `video/webm`, VP9/Opus.

Contract record fields: `artifact_type` (`"video/x-fard-transcode-contract"`), `kind`, `input_artifact`, `input_artifact_type`, `input_path`, `output_artifact_type`, `container`, `video_codec`, `audio_codec`, `bridge_state`.

### video-core/src/transcode_pipeline

`rawvid_frames_to_mp4(rawvid_bytes, wav_path, frames_dir, output_path)` — materialises PPM frame sequence to `frames_dir`, then calls `transcode_exec.transcode_mp4` with ffmpeg pattern `frames_dir/frame_%06d.ppm`.
`rawvid_frames_to_webm(rawvid_bytes, wav_path, frames_dir, output_path)` — same, targeting WebM.

### video-core/src/gifbridge

`gif_stub(video_artifact) -> Record` — `{ artifact_type: "image/gif", source_artifact_type, source_path, bridge: "rawvid_to_gif_stub" }`. Stub pointing at a rawvid source for downstream GIF conversion.

### video-core/src/export

`write_rawvid(path, title, video)` — encodes video to FARDVID1, emits via `artifact_std.emit`, writes `.receipt.json`. Receipt includes `artifact_type`, `encoding`, `width`, `height`, `fps_num`, `fps_den`, `frames`, `title`, `byte_len`.

### pdf_v0/src/model

```ebnf
bbox_record     = "{" , "x0" , ":" , number , "," , "y0" , ":" , number , ","
                  "x1" , ":" , number , "," , "y1" , ":" , number , "}" ;

match_record    = "{" , "page" , ":" , integer , "," , "match_id" , ":" , string , ","
                  "text" , ":" , string , "," , "span_ids" , ":" , list , ","
                  "bbox" , ":" , bbox_record , "}" ;

overlay_item    = "{" , "page" , ":" , integer , "," , "type" , ":" , string , ","
                  "x0" , ":" , number , "," , "y0" , ":" , number , ","
                  "x1" , ":" , number , "," , "y1" , ":" , number , ","
                  "fill_rgb" , ":" , "[" , integer , "," , integer , "," , integer , "]" , ","
                  "opacity_mode" , ":" , string , "}" ;
```

`make_bbox(x0, y0, x1, y1)`, `make_match(page, id, text, span_ids, bbox)`, `make_overlay_item(page, kind, bbox, fill_rgb)`, `make_matches_doc(query, matches)`, `make_overlay_plan(items)`.

### pdf_v0/src/write_pdf

`build_pdf(rects) -> Text` — generates a complete PDF-1.4 document. Structure: catalog (obj 1) → pages (obj 2) → page (obj 3, MediaBox 200×200) → content stream (obj 4). Content stream contains `rg` colour ops and `re f` fill ops for each rect. Returns the full PDF as a text string.

### pdf_v0/src/highlight

`find_matches(spans, query) -> List` — folds over a list of span records `{ text, page, bbox }`, returning match records for any span whose text contains `query`.
`make_overlay(matches) -> List` — maps matches to yellow (`[255, 235, 59]`) `highlight_rect` overlay items.

### pdf_patch/src/patch_pdf

`patch_pdf(pdf_text, x0, y0, x1, y1) -> Text` — injects a yellow rectangle overlay into an existing PDF. Appends a new content stream object (obj 5) with a `re f` drawing op, updates the page `/Contents` from `4 0 R` to `[4 0 R 5 0 R]`, and replaces the trailing `%%EOF` marker. Returns the patched PDF as text.

-----

## Media Decoders

### audio-core/src/wav_decode

`decode(bs) -> { ok: { header, samples } } | { err: text }`

Parses a WAV file from `Bytes`. Locates `fmt ` and `data` chunks by scanning the RIFF container. Returns:

- `header` — `{ channels, sample_rate, bits_per_sample }`
- `samples` — `List(Int)`, PCM values. 16-bit: signed integers in [-32768, 32767]. 8-bit: unsigned offset by -128.

Exports: `u16le_at`, `u32le_at`, `i16le_at`, `find_chunk`, `decode`.

### image-core/src/ppm_decode

`decode(bs) -> { ok: { width, height, maxval, pixels } } | { err: text }`

Parses a P6 binary PPM file from `Bytes`. Skips whitespace and `#` comment lines between tokens. Returns `pixels` as `List({ r, g, b })`.

Exports: `read_token`, `decode`.

> `width`, `height`, `maxval` are `Int` — parsed via `int.parse(...).v`.

### image-core/src/png_decode

`decode(bs) -> { ok: { width, height, pixels } } | { err: text }`

Parses an 8-bit RGB PNG file from `Bytes`. Validates the 8-byte signature. Decompresses IDAT using a pure-FARD zlib stored-block decompressor. Unfilters scanlines (filter types 0=None, 1=Sub, 2=Up). Returns `pixels` as `List({ r, g, b })`.

Exports: `be32_at`, `valid_signature`, `parse_chunks`, `parse_ihdr`, `zlib_decompress`, `unfilter_row`, `decode_pixels`, `decode`.

-----

## Media Transforms

### image-core/src/transform

All functions take a raster value and return a new raster value.

`resize(src, new_width, new_height)` — nearest-neighbour resampling.

`crop(src, x0, y0, w, h)` — extract sub-rectangle at `(x0, y0)` with dimensions `w × h`.

`flip_h(src)` — mirror horizontally.

`flip_v(src)` — mirror vertically.

`composite(dst, src, dx, dy)` — overlay `src` onto `dst` at offset `(dx, dy)`. Pixels outside `src` bounds taken from `dst`.

`brightness(src, factor)` — scale each channel by `factor` (Float). Channels clamped to [0, 255].

`grayscale(src)` — convert to grayscale using luminance weights: R×0.299 + G×0.587 + B×0.114.

### audio-core/src/transform

All functions operate on sample lists. Samples may be `Float` (synth output) or `Int` (decoded PCM).

`gain(samples, factor)` — scale all samples by `factor` (Float). Integer samples clamped to [-32768, 32767].

`trim(samples, sample_rate, start_ms, end_ms)` — extract the slice from `start_ms` to `end_ms`. Uses `floor(ms * sample_rate / 1000)` for sample-accurate boundaries.

`mix(a, b)` — sum two sample lists element-wise. Output length is `max(len(a), len(b))`. Missing samples treated as 0.

`fade_in(samples, fade_samples)` — ramp gain from 0 to 1 over the first `fade_samples` samples.

`fade_out(samples, fade_samples)` — ramp gain from 1 to 0 over the last `fade_samples` samples.

`mono_to_stereo(samples)` — duplicate each sample into an interleaved stereo pair.

`stereo_to_mono(samples)` — average interleaved stereo pairs into a mono list.

-----

## Integration Packages

All packages import by relative path. No native dependencies.

### packages/postgres-core

PostgreSQL wire protocol v3 client, pure FARD.

**conn.fard** — public API:

`connect(host, port, user, password, database) -> { ok: conn_id } | { err: text }` — opens a TCP connection, sends startup message, handles cleartext auth.

`exec(conn_id, sql) -> { ok: null } | { err: text }` — executes DDL or DML, reads until `ReadyForQuery`.

`query(conn_id, sql) -> { ok: List(Record) } | { err: text }` — executes a query, parses `RowDescription` and `DataRow` messages, returns rows as records keyed by column name.

`close(conn_id)` — sends `Terminate` message and closes TCP connection.

**wire.fard** — `be32`, `be16`, `be32_at`, `be16_at`, `cstr`, `msg` — big-endian encoding helpers for the PG wire protocol.

**auth.fard** — `startup_msg`, `password_msg`, `auth_type` — startup and authentication message builders.

**query.fard** — `simple_query`, `parse_msg`, `parse_all`, `parse_data_row`, `parse_row_description`, `msgs_to_rows` — message parsing and row assembly.

### packages/ws-core

WebSocket client, pure FARD (RFC 6455).

**conn.fard** — `connect(host, port, path)`, `send_text`, `send_binary`, `recv`, `close` — full client lifecycle. Auto-responds to ping frames with pong.

**frame.fard** — `encode(op, payload, mask)`, `decode(bs, off)` — frame encoder/decoder. Supports 7-bit, 16-bit, and 64-bit payload length fields. Opcodes: 1=text, 2=binary, 8=close, 9=ping, 10=pong.

**handshake.fard** — `upgrade_request`, `handshake_ok`, `split_http` — HTTP upgrade request builder and response validator.

### packages/watch-core

Poll-based filesystem watcher, pure FARD.

**watch.fard** — `watch(paths, poll_ms, max_iters, handler)`, `watch_dir(path, poll_ms, max_iters, handler)` — calls `handler({ added, removed, changed })` whenever the snapshot changes. Pass `max_iters: -1` for infinite polling.

**stat.fard** — `snapshot(paths) -> Record` — maps each path to its current byte length. `snapshot_dir(path)` — lists a directory and returns `{ path, name }` records.

**diff.fard** — `diff(before, after) -> { added, removed, changed }`, `any_change(d) -> Bool` — snapshot comparison.

### packages/xlsx-core

Excel workbook writer (OOXML), pure FARD.

**write.fard** — `build_xlsx(sheets) -> Bytes`, `write_xlsx(path, sheets)` — builds a complete `.xlsx` file. `sheets` is `List({ name, rows })` where `rows` is `List(List(value))`. Values may be `Int`, `Float`, or `Text`.

**sheet.fard** — `sheet_xml(rows)`, `col_letter(n)`, `cell_ref(col, row)` — worksheet XML generation. Numeric values use `<c>` with `<v>`, strings use `inlineStr`.

**zip.fard** — `build(files) -> Bytes` — minimal ZIP writer (store method, no compression). `files` is `List({ name, data })`.

**xml.fard** — `escape`, `attr`, `tag`, `self_closing`, `declaration` — XML helpers.

**workbook.fard** — `workbook_xml`, `rels_xml`, `content_types_xml`, `root_rels_xml` — OOXML relationship and manifest XML.

### packages/avro-core

Apache Avro OCF encoder/decoder, pure FARD.

**container.fard** — `write_ocf(records) -> Bytes` — writes an Avro Object Container File. Infers schema from the first record. Encodes all records in a single block with a fixed 16-byte sync marker.

**encode.fard** — `varint(n)`, `encode_string(s)`, `encode_long(n)`, `encode_boolean(b)`, `encode_record(record, fields)` — Avro binary encoding. Uses zigzag LEB128 for integers.

**decode.fard** — `decode_varint(bs, off)`, `decode_string(bs, off)`, `decode_long(bs, off)`, `decode_boolean(bs, off)`, `decode_record(bs, off, fields)` — Avro binary decoding. All return `{ value, next_off }`.

**schema.fard** — `infer(records) -> schema_record` — infers an Avro record schema from a list of FARD records. Type mapping: `Int -> long`, `Float -> double`, `Bool -> boolean`, `Text -> string`.

### packages/parquet-core

Apache Parquet writer, pure FARD.

**write.fard** — `build_parquet(records) -> Bytes`, `write_parquet(path, records)` — writes a valid Parquet file with one row group. Magic: `PAR1`. Metadata encoded with Thrift compact protocol.

**thrift.fard** — `uvarint`, `varint`, `field_header`, `i32_field`, `i64_field`, `string_field`, `binary_field`, `list_header`, `stop` — Thrift compact protocol encoder.

**schema.fard** — `infer_columns(records)` — infers column types. Type mapping: `Bool -> BOOLEAN (0)`, `Int -> INT64 (2)`, `Float -> DOUBLE (4)`, other -> `BYTE_ARRAY (5)`. Keys sorted lexicographically (BTreeMap order).

**encode.fard** — `encode_plain(v)`, `encode_column(values)` — plain encoding. INT64: 8-byte LE. DOUBLE: 8-byte LE (integer cast). BYTE_ARRAY: 4-byte LE length prefix + UTF-8 bytes.

### packages/duckdb-core

In-memory analytical query engine, pure FARD.

**query.fard** — `run(plan) -> List(Record)` — executes a query plan. Plan fields: `from` (rows), `where` (expr), `select` (list of `{ name, expr }`), `group_by` (col names), `aggs` (list of `{ name, fn_name, col }`), `order_by` (col name), `limit` (int). All fields optional.

**expr.fard** — `eval(row, expr)`, `filter(rows, pred)`, `project(rows, selects)` — expression evaluator. Expr formats: `"col_name"` (field lookup), `{ "lit": value }` (literal), `{ "col": "name" }` (explicit field), `{ "op": "==", "left": e, "right": e }` (binary op). Use `{ "lit": v }` for literal values in predicates to distinguish from field names.

**agg.fard** — `sum`, `count`, `avg`, `min_val`, `max_val`, `group_by(rows, key_cols)`, `aggregate(groups, aggs)` — aggregation. `fn_name` values: `"sum"`, `"count"`, `"avg"`, `"min"`, `"max"`.

**join.fard** — `inner_join(left, right, left_key, right_key)`, `left_join(...)`, `cross_join(left, right)` — relational joins. Matching rows merged via `rec.merge`.

### packages/wasm-core

WASM binary format decoder and stack machine interpreter, pure FARD.

**decode.fard** — `parse(bs) -> { ok: { sections } } | { err: text }` — validates magic/version and parses all sections. `parse_sections(bs)` returns `List({ id, name, size, body })`. `parse_type_section(body)` returns function type signatures. `parse_export_section(body)` returns `List({ name, kind, index })`.

`leb128_u(bs, off) -> { value, next_off }`, `leb128_s(bs, off) -> { value, next_off }` — LEB128 unsigned and signed decode.

**encode.fard** — `leb128_u(n)`, `leb128_s(n)` — LEB128 encode. `section(id, body)`, `vec(items)`, `functype(params, results)` — module structure helpers. Instructions: `i32_const`, `i64_const`, `i32_add`, `i32_sub`, `i32_mul`, `local_get`, `local_set`, `end_op`, `return_op`. `build_module(type_sec, func_sec, export_sec, code_sec)` — assembles a complete WASM binary.

**interp.fard** — `call(wasm_bytes, fn_name, args) -> { ok: value } | { err: text }` — executes an exported function by name. Supported opcodes: `i32.const` (65), `i64.const` (66), `local.get` (32), `local.set` (33), `i32.add` (106), `i32.sub` (107), `i32.mul` (108), `return` (15), `end` (11).

`exec_body(body, locals) -> value` — executes a raw code body with initial local values.

-----

## Known Parser Constraints

1. **`let` inside `if/else` branches works in both forms.** `if c then let x = e in body` and `if c then { let x = e\n body }` both work correctly.
1. **`[[…]]` as fn tail is FIXED.** The postfix parser now checks for newlines and
   literal bases before treating `[` as an index operator. `[[a, b], [c, d]]` works
   correctly as a tail expression.
1. **`then { }` blocks ARE supported** when the first token inside `{` is `let`,
   `return`, or `}` (empty block). `then { k: v }` is still parsed as a record literal.
1. **`!=` IS implemented** — lexed as a two-char operator. Use it directly.
1. **`\r` not a valid escape.** Only `\n \t \" \\` accepted.
1. **`str.lower`, `str.to_lower`, `str.toLower` all work.** All three aliases exist for case conversion. Same for upper.
1. **`list.find` returns `{some: value}` or `{none: unit}`.** Check with `rec.has(r, "some")` or access `.some` directly. There is no `.data` or `.value` field.
1. **`list.concat` takes one argument** — a list of lists.
1. **`rec.remove` not `rec.delete`.**
1. **`int` as import alias works.** `import("std/int") as int` is valid. Previously documented as reserved — this was incorrect.
1. **Destructuring in `let`** — `let { a, b } = expr` works at top-level and in fn bodies. Shorthand `{ name }` without `: pat` binds to variable `name`. List destructuring `let [a, b] = list` is also supported.
1. **Float literals are `Val::Float`.** `1.5` produces `Val::Float(1.5)`. Int+float arithmetic is automatically promoted: `1 + 0.5 == 1.5`. The previous documentation claiming float literals produce `Val::Bytes` was incorrect.
1. **`std/compress` uses `gzip`/`gunzip`**, not `gzip_compress`/`gzip_decompress`.
1. **`std/graph` uses `of`/`ancestors`/`leaves`/`to_dot`**, not the previously documented API.
1. **`Val` field is `Text` not `Str`.** The runtime type name is `"text"`, returned by `type.of()`.
1. **Computed record keys `[expr]: val`** — dynamic keys evaluated at runtime. Works in both top-level and fn bodies. Can be combined with spread: `{ ...base, [key]: val }`.
1. **`str.from(v)` converts any scalar to string.** `str.from(42)` gives `"42"`. **`cast.text(42)` gives `"*"` (Unicode codepoint 42) — never use it for number-to-string conversion.**
1. **`float + int` is automatically promoted.** `1 + 0.5 == 1.5` works without explicit casting. `cast.float` is no longer needed for mixed arithmetic.
1. **`list.find` returns `{some: value}` or `{none: unit}`.** Access value with `.some`, not `.data` or `.value`.
1. **`menv.set` returns `Unit`.** Never use it in value position. Always `let _ = menv.set(...)`.
1. **`and`/`or` are not keywords.** Use `&&` and `||` instead.
1. **`&&` and `||` are fully implemented** with short-circuit evaluation. `if x > 0 && x < 10` works correctly. Previously documented as broken — now fixed.
1. **CSV values are type-inferred.** `csv.parse_csv` calls `int.parse` and `float.parse` on each cell. Numbers come back as `Int` or `Float`, not `Text`.
1. **SQLite exec is per-connection.** `sqlite.open` on a file path creates a new connection each call. Use a single `db` handle throughout a pipeline.
1. **`raster.make` takes a `pixel_fn(x, y)`**, not a pre-built pixel list. The function is called for each `(x, y)` position row-major. To build a raster from an existing list, wrap it: `fn(x, y) { list.get(pixels, y * width + x) }`.
1. **`audio-core/src/pcm` no longer exports `f32_to_pcm_s16`.** The function was inlined. Only `encode_pcm_s16le` is exported.
1. **All media exporters use `artifact_std.emit`**, not `io.write_file`. Output files are content-addressed and appear in the execution trace as oracle boundary events.
1. **`int.parse` returns `{ t: "ok", v: n }`**, not `{ ok: n }`. Access the value with `.v`, not `.ok`.
1. **`io.read_file` returns `{ ok: text }`** on success with no `err` field. Accessing `.err` on a successful result crashes — check for `.ok` instead.
1. **`duckdb-core` expr literals must use `{ "lit": value }`** to distinguish from field name lookups. A bare string like `"eng"` in an expr is treated as a field name, not a string value.
1. **`audio-core/src/transform` operates on float or int samples.** `gain` accepts both. `fade_in`/`fade_out` use float multiplication. `stereo_to_mono` uses integer division — apply after PCM conversion for best results.
1. **`png_decode` supports filter types 0, 1, and 2 only.** Filter type 3 (Average) and 4 (Paeth) are not yet implemented — unfiltered scanlines are returned as-is for unsupported filter types.

-----

*Audited against fardrun v1.7.0. Canonical sources: `src/bin/fardrun.rs`, `packages/`.*