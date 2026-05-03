# FARD — ISO EBNF Grammar

**2 May 2026 · v1.7.0 · fully verified against fardrun source and runtime**

This document covers the **fardrun** production dialect. All grammar rules and stdlib
entries verified directly by running test programs against fardrun v1.7.0.

-----

## Lexical

```ebnf
alpha           = "A" | ... | "Z" | "a" | ... | "z" ;
digit           = "0" | ... | "9" ;

ident_start     = alpha | "_" ;
ident_continue  = alpha | digit | "_" ;
ident           = ident_start , { ident_continue } ;

integer         = digit , { digit } ;
(* Multi-digit integer cannot have leading "0" — ERROR_PARSE leading zero integer literal. *)
(* Negative integers are handled by unary "-" at the expression level. *)

float           = digit , { digit } , "." , digit , { digit } , [ sci_exp ] ;
sci_exp         = ( "e" | "E" ) , [ "+" | "-" ] , digit , { digit } ;
(* Float literals evaluate to Val::Float(f64). Scientific notation supported: 1.5e10, 2.3E-4. *)
(* Int+float arithmetic is automatically promoted: 1 + 0.5 == 1.5. *)

escape          = "\\n" | "\\t" | "\\\"" | "\\\\" ;
(* Only these four escape sequences are valid. "\r" produces: bad escape: \r *)

string_char     = ? any char except " and \ ? | escape ;
string          = "\"" , { string_char | interp_expr } , "\"" ;
interp_expr     = "${" , expr , "}" ;
(* String interpolation: "${expr}" embeds any expression inside a double-quoted string. *)

backtick_string = "`" , { ? any char except backtick ? } , "`" ;
(* Backtick strings have NO escape processing. All chars are literal. *)

whitespace      = { " " | "\t" | "\r" | "\n" } ;
comment         = "//" , { ? any char except newline ? } , ( "\n" | ? EOF ? )
                | "#"  , { ? any char except newline ? } , ( "\n" | ? EOF ? ) ;
(* Both // and # introduce line comments. Both verified working. *)

sym2            = "!=" | "==" | "<=" | ">=" | "&&" | "->" | "=>" | "|>" ;
(* "||" is a dedicated token. "!=" IS implemented — use it directly. *)

sym3            = "..." ;

sym1            = "(" | ")" | "{" | "}" | "[" | "]" | "," | ":" | "."
                | "+" | "-" | "*" | "/" | "=" | "%" | "|" | "<" | ">"
                | "?" | "!" ;
(* "%" modulo. "!" unary not. "|" sequencing, distinct from "|>" *)

keyword         = "let" | "in" | "fn" | "if" | "then" | "else"
                | "import" | "as" | "export" | "match" | "using"
                | "test" | "while" | "return"
                | "true" | "false" | "null" ;
(* Keywords ARE valid record keys: { ok: true, if: "allowed", return: 42 } *)
(* Keywords ARE valid bind names in patterns (except true/false/null). *)
(* "int" is NOT reserved — import("std/int") as int works fine. *)
```

-----

## Module (Top-Level)

```ebnf
module          = { module_item } ;

(* parse_module dispatch order:
   1) "test"     -> test_item
   2) "a"        -> type_decl
   3) "import"   -> import_item
   4) "artifact" -> artifact_item
   5) "export"   -> export_item
   6) "fn"       -> fn_item
   7) "let"      -> try parse_expr first; if fails, parse let_item
   8) anything else -> expr
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

type_decl       = "a" , ident , "is" , ( record_type_body | sum_type_body ) ;
record_type_body = "{" , { type_field , [ "," ] } , "}" ;
type_field       = ident , ":" , ident ;
sum_type_body    = variant , { "or" , variant } ;
variant          = ident , [ "(" , { type_field , [ "," ] } , ")" ] ;
(* Type declarations are parsed but NOT enforced at runtime. *)
(* a Point is { x: Int, y: Int } *)
(* a Shape is Circle(r: Int) or Rect(w: Int, h: Int) *)

import_item     = "import" , "(" , string , ")" , "as" , ident ;
(* "as alias" is mandatory. Path resolved relative to importing file, extensionless. *)

artifact_item   = "artifact" , ident , "=" , string ;
(* string MUST start with "sha256:". Validated against receipts/ at runtime. *)

export_item     = "export" , "{" , ident , { "," , ident } , [ "," ] , "}" ;

let_item        = "let" , ( ident | obj_pat | list_pat ) , "=" , expr ;
(* Module-level let — no "in" required. *)
(* CONSTRAINT: list_pat at module level always fails — use let-in form. *)
(* CONSTRAINT: obj_pat with ...rest at module level — rest is silently unbound. *)

fn_item         = "fn" , ident , "(" , [ fn_param , { "," , fn_param } ] , ")"
                , [ "->" , type ]
                , "{" , fn_block_body ;

fn_param        = pat , [ ":" , type ] , [ "=" , expr ] ;
(* "= expr" default parameter syntax is PARSED but NOT implemented at runtime. *)
(* Calling with fewer args than declared always causes arity mismatch. Do not use. *)
```

-----

## Function Bodies

```ebnf
fn_block_body   = fn_block_inner , "}" ;
fn_block_inner  = { let_binding } , seq_expr ;

let_binding     = "let" , ( ident | obj_pat | list_pat ) , "=" , expr ,
                  ( "in" , expr
                  | "|" , fn_block_inner
                  | (* nothing — block binding *)
                  ) ;

seq_expr        = expr , { "|" , expr } ;
(* "|" between expressions is sequencing: e1 | e2  ==  let _ = e1 in e2 *)
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
(* Verified: using x = 42 in x + 1  =>  43 *)

match_expr      = "match" , expr , "{" , [ match_arm , { "," , match_arm } , [ "," ] ] , "}" ;
match_arm       = pat , [ "if" , expr ] , "=>" , expr ;
(* Guards: match x { n if n > 3 => "big", _ => "small" } *)
(* Nested obj patterns: match p { { x: 0, y } => y, { x, y } => x + y } *)

let_expr        = "let" , pat , "=" , expr , "in" , expr ;
(* Requires "in". Chain: let a = 1 in let b = 2 in a + b *)

while_expr      = "while" , expr , expr , expr ;
(* Three bare expressions: init_state, condition_fn, step_fn. *)
(* Returns: { chain_hex: string, steps: int, value: final_state } *)
(* Example: while 0 fn(s) { s < 5 } fn(s) { s + 1 } *)

return_expr     = "return" , expr ;
(* Exits the IMMEDIATELY ENCLOSING fn {} body only. *)
(* Inside an anonymous fn passed to list.fold, return exits the closure *)
(* not the outer named function. Use recursion for early exit from outer fns. *)

if_expr         = "if" , expr , "then" , ( block_expr | expr ) , "else" , expr ;
block_expr      = "{" , fn_block_inner , "}" ;
(* "then { }" blocks supported when first token is "let", "return", or "}". *)
(* "then { k: v }" is parsed as a record literal — use "then ({ k: v })" if needed. *)

infix_expr      = unary_expr , { infix_op , unary_expr } ;
infix_op        = "||" | "??" | "&&"
                | "==" | "!=" | "<" | ">" | "<=" | ">="
                | "+" | "-"
                | "*" | "/" | "%" ;
(* Precedence low to high: || ??  /  &&  /  == != < > <= >=  /  + -  /  * / % *)
(* "??" is null-coalescing: x ?? default returns x if x != null, else default. *)
(* "+" does NOT concatenate strings — use str.concat([a, b]) instead. *)

unary_expr      = { "-" | "!" } , pipe_expr ;

pipe_expr       = postfix_expr , { "|>" , postfix_expr } ;
(* "|>" inserts lhs as FIRST argument: "hello" |> str.split(" ")  =>  str.split("hello", " ") *)

postfix_expr    = primary_expr , { postfix_op } ;
postfix_op      = "?" , "."            (* safe nav: null?.f => null, rec?.f => rec.f *)
                | "?"                  (* result unwrap — see note below *)
                | "." , ident          (* field access *)
                | "(" , arg_list , ")"
                | "[" , expr , "]" ;   (* index: list[i] or record["key"] *)
(* "?" unwraps {t:"ok",v:val} => val. On {t:"err",...} propagates error. *)
(* On any other value: QMARK_EXPECT_RESULT error. *)
(* "[expr]" NOT parsed when there is a newline between base and "[". *)

arg_list        = [ pos_args | named_args ] ;
pos_args        = expr , { "," , expr } , [ "," ] ;
named_args      = named_arg , { "," , named_arg } , [ "," ] ;
named_arg       = ident , ":" , expr ;
(* Named args reorder by parameter name. Cannot mix positional and named. *)
(* Verified working in v1.7.0: add(y: 3, x: 7) == 10 *)

primary_expr    = lambda_expr
                | multi_lambda
                | anon_fn_expr
                | string
                | backtick_string
                | float
                | integer
                | "true" | "false" | "null"
                | ident
                | "(" , expr , ")"
                | list_lit
                | rec_lit ;

lambda_expr     = ident , "=>" , expr ;
(* x => x * 2. Verified: f(5) => 10 *)

multi_lambda    = "(" , [ ident , { "," , ident } ] , ")" , "=>" , expr ;
(* (x, y) => x + y. Verified: f(3, 4) => 7 *)

anon_fn_expr    = "fn" , "(" , [ pat , { "," , pat } ] , ")" ,
                  "{" , fn_block_inner , "}" ;

list_lit        = "[" , [ expr , { "," , expr } , [ "," ] ] , "]" ;

rec_lit         = "{" , [ rec_content ] , "}" ;
rec_content     = rec_spread
                | rec_kv , { "," , rec_kv } , [ "," ] ;
rec_spread      = "..." , postfix_expr , { "," , rec_kv } ;
(* { ...base, key: val } merges base with overrides. Last write wins. *)
(* Verified: let base = {x:1,y:2} in {...base, z:3} => {x:1,y:2,z:3} *)
rec_kv          = rec_key , ":" , expr
                | "[" , expr , "]" , ":" , expr ;
(* Computed keys: { [k]: v } — dynamic key evaluated at runtime. *)
(* Verified: let k = "foo" in { [k]: 42 } => {"foo": 42} *)
rec_key         = ident | keyword | string ;
```

-----

## Patterns

```ebnf
pat             = "true" | "false" | "null"
                | "_"
                | integer | string
                | obj_pat | list_pat
                | bind_name ;

bind_name       = ident | keyword ;
(* Keywords valid as bind names except true/false/null. *)
(* Duplicate bind names in a single pattern are a parse error. *)

obj_pat         = "{" , [ obj_pat_body ] , "}" ;
obj_pat_body    = { obj_field , "," } , obj_field , [ "," ]
                | { obj_field , "," } , obj_field , "," , "..." , ident
                | "..." , ident ;
obj_field       = ident , ":" , pat
                | ident ;
(* Shorthand { name } binds field "name" to variable "name". *)
(* Rest capture { x, ...rest } works in let-in form. *)
(* CONSTRAINT: { x, ...rest } at module level — rest is silently unbound. *)

list_pat        = "[" , [ list_pat_body ] , "]" ;
list_pat_body   = pat , { "," , pat } , [ "," , "..." , ident ]
                | "..." , ident ;
(* CONSTRAINT: list_pat in let_item (module level) always fails ERROR_PARSE. *)
(* Use let-in form only: let [a, b, ...rest] = expr in body *)
(* Verified: let [a, b, ...rest] = [1,2,3,4] in rest => [3,4] *)
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

list_type       = "List" , "<" , type , ">" ;

rec_type        = "Rec" , "{" , [ rec_type_field , { "," , rec_type_field } ] , "}" ;
rec_type_field  = ident , ":" , type ;
(* NOTE: Rec uses BRACES not angle brackets: Rec { x: Int, y: Int } *)

func_type       = "Func" , "(" , [ type , { "," , type } ] , ")" , "->" , type ;

named_type      = ident , [ "<" , type , { "," , type } , ">" ] ;
(* Examples: MyType, Option<Int>, Result<Text, Int> *)

(* Type annotations are optional everywhere. Parsed but NOT enforced at runtime. *)
```

-----

## Runtime Value Types

| Variant | type.of() | Description |
|---|---|---|
| `Unit` | `"unit"` | null |
| `Bool(bool)` | `"bool"` | true / false |
| `Int(i64)` | `"int"` | 64-bit signed integer |
| `Float(f64)` | `"float"` | 64-bit float |
| `Text(String)` | `"text"` | UTF-8 text — NOT "str" or "string" |
| `Bytes(Vec<u8>)` | `"bytes"` | Raw bytes |
| `List(Vec<Val>)` | `"list"` | Ordered list |
| `Record(BTreeMap<String,Val>)` | `"record"` | Sorted string keys |
| `Err { code, data }` | `"err"` | Error value |
| `Func` | `"func"` | Closure |
| `Builtin` | `"builtin"` | Native stdlib function |
| `BoundMethod` | `"function"` | Method bound to receiver |
| `Chan` | — | Channel (std/chan) |
| `Mtx` | — | Mutex (std/mutex) |
| `Big(BigInt)` | — | Arbitrary-precision integer (std/bigint) |
| `Promise` | — | Async promise (std/promise) |

**Key notes:**
- `type.of(null)` returns `"unit"` — NOT `"null"`
- `type.of("hello")` returns `"text"` — NOT `"str"` or `"string"`
- `cast.text(65)` returns `"A"` (Unicode codepoint) — NOT number-to-string
- Use `str.from(65)` for `"65"` (number to string representation)
- Int + Float arithmetic auto-promotes: `1 + 0.5 == 1.5`

-----

## Stdlib Modules — Complete Reference

### std/str

`len`, `trim`, `split_lines`, `lower`, `to_lower`, `toLower`, `upper`, `to_upper`, `toUpper`,
`concat`, `split`, `contains`, `starts_with`, `ends_with`, `replace`, `slice`, `format`,
`from_int`, `from_float`, `from`, `join`, `pad_left`, `pad_right`, `repeat`,
`index_of`, `chars`

**Verified:**
- `str.concat(a, b)` and `str.concat([a, b, c])` both work
- `str.pad_left("hi", 5, "0")` => `"000hi"`
- `str.pad_right("hi", 5, ".")` => `"hi..."`
- `str.repeat("ab", 3)` => `"ababab"`
- `str.index_of("hello", "ll")` => `2`
- `str.join(["a","b","c"], "-")` => `"a-b-c"`
- `str.chars("hi")` => `["h","i"]`
- `str.lower`, `str.to_lower`, `str.toLower` all work (same for upper)
- `"+"` does NOT concatenate strings — always use `str.concat`

### std/list

`len`, `range`, `repeat`, `concat`, `group_by`, `fold`, `map`, `filter`,
`get`, `head`, `tail`, `last`, `append`, `zip`, `reverse`, `flatten`, `set`,
`any`, `all`, `find`, `find_index`, `take`, `drop`, `flat_map`, `par_map`,
`zip_with`, `chunk`, `sort_by`, `sort_by_int_key`, `sort_int`,
`dedupe`, `dedupe_sorted_int`, `hist_int`, `build`

**Verified:**
- `list.range(0, 5)` => `[0,1,2,3,4]`
- `list.zip([1,2,3], ["a","b","c"])` => `[[1,"a"],[2,"b"],[3,"c"]]`
- `list.find([1,2,3], fn(x) { x == 2 })` => `{"some": 2}` — access with `.some`
- `list.sort_int([3,1,2])` => `[1,2,3]`
- `list.chunk([1,2,3,4,5], 2)` => `[[1,2],[3,4],[5]]`
- `list.flat_map([1,2,3], fn(x) { [x, x*10] })` => `[1,10,2,20,3,30]`
- `list.concat` takes ONE argument — a list of lists
- `list.build(n, fn(i) { expr })` => list of n items where item[i] = fn(i)

### std/rec

`empty`, `keys`, `values`, `has`, `get`, `getOr`, `getOrErr`, `set`,
`remove`, `merge`, `select`, `rename`, `update`

**Verified:**
- `rec.get(r, "missing")` returns `null` — silent, no error
- `rec.getOr(r, "missing", default)` returns default
- `rec.remove(r, "key")` — use `remove` not `delete`
- `rec.merge({x:1}, {y:2})` => `{x:1,y:2}`
- `rec.select(r, ["a","c"])` => record with only those keys

> `std/record` aliases `std/rec`. Both work identically.

### std/map

`get`, `set`, `keys`, `values`, `has`, `delete`, `entries`, `new`, `from_entries`

**Verified:**
- `map.new()` creates empty map
- `map.set(m, "x", 42)` returns new map
- `map.get(m, "x")` => `42`
- `map.keys(m)` => list of keys

### std/set

`new`, `add`, `remove`, `has`, `union`, `intersect`, `diff`, `to_list`, `from_list`, `size`

**Verified:**
- `set.from_list([1,2,3,2,1])` deduplicates => size 3
- `set.to_list(s)` => `[1,2,3]`

### std/int

`add`, `eq`, `parse`, `pow`, `to_hex`, `to_bin`, `mul`, `div`, `sub`,
`abs`, `min`, `max`, `to_text`, `to_string`, `from_text`, `neg`, `clamp`, `mod`,
`lt`, `gt`, `le`, `ge`, `to_str_padded`

**Verified:**
- `int.parse("42")` returns `{t: "ok", v: 42}` — access with `.v` not `.ok`
- `int.parse("42")?` => `42` (unwrap with `?`)
- `int.parse("abc")?` propagates error

### std/float

`from_int`, `to_int`, `from_text`, `to_text`, `parse`, `add`, `sub`, `mul`, `div`,
`exp`, `ln`, `sqrt`, `pow`, `abs`, `neg`, `floor`, `ceil`, `round`,
`lt`, `gt`, `le`, `ge`, `eq`, `nan`, `inf`, `is_nan`, `is_finite`, `min`, `max`

### std/math

`abs`, `min`, `max`, `pow`, `sqrt`, `floor`, `ceil`, `round`,
`log`, `log2`, `log10`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `exp`

Constants: `pi`, `e`, `inf` (Val::Float values, not functions)

### std/bigint

`from_int`, `from_str`, `to_str`, `add`, `sub`, `mul`, `div`, `mod`, `pow`, `eq`, `lt`, `gt`

### std/bits

`band`, `bor`, `bxor`, `bnot`, `bshl`, `bshr`, `popcount`

> All bit operations accept Int and return Int. `bits.band(n, 4294967295)` is the canonical u32 mask.

### std/json

`encode`, `decode`, `canonicalize`

**Verified:**
- `json.encode({a: 1, b: [1,2,3]})` => `'{"a":1,"b":[1,2,3]}'`
- `json.decode('{"x":42}')` => `{x: 42}`

### std/bytes

`concat`, `to_str`, `to_string`, `from_string`, `len`, `get`, `of_list`, `to_list`,
`of_str`, `merkle_root`, `eq`, `to_hex`, `to_base64`, `from_hex`, `from_base64`, `slice`

**Verified:**
- `bytes.of_str("hello")` => Bytes of length 5
- `bytes.to_hex(bytes.of_str("hi"))` => `"6869"`

### std/codec

`base64url_encode`, `base64url_encode_hex`, `base64url_decode`, `hex_encode`, `hex_decode`

### std/base64

`encode`, `decode`

> Returns `Val::Bytes`. Use `bytes.to_str()` to convert to text.

### std/csv

`parse`, `encode`

> CSV values are type-inferred — numbers come back as Int or Float, not Text.

### std/hash

`sha256_text`, `sha256_bytes`

**Verified:**
- `hash.sha256_text("hello")` => `"sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"`

### std/crypto

`hmac_sha256(key, msg)`, `ed25519_verify(pk_hex, msg_hex, sig_hex)`,
`sha512`, `aes_encrypt`, `aes_decrypt`, `merkle_root`

### std/uuid

`v4`, `validate`

### std/rand / std/random

`uuid_v4`, `int`, `float`

### std/datetime

`now`, `format`, `parse`, `add`, `diff`, `field`

### std/time

`now`, `parse`, `format`, `add`, `sub`

Duration record: `{ ms, sec, min }` — each is a builtin function.

### std/io

`read_file`, `write_file`, `append_file`, `read_lines`, `file_exists`,
`delete_file`, `read_stdin`, `read_stdin_lines`, `list_dir`, `make_dir`

> `io.read_file` returns `{ ok: text }` on success — no `err` field. Accessing `.err` crashes.

### std/fs

`read_text`, `write_text`, `write_bytes`, `exists`, `read_dir`, `stat`, `delete`, `make_dir`

> `write_bytes(path, bytes)` writes a Bytes value. Sandbox-restricted to relative paths.

### std/path

`base`, `dir`, `ext`, `isAbs`, `join`, `joinAll`, `normalize`

### std/env

`get`, `args`

### std/process

`spawn`, `exit`, `capture`

> `capture(cmd, args)` => `{ ok, exit_code, stdout, stderr }`

### std/http

`get`, `post`, `request`

### std/net

`serve(port, handler_fn)`, `connect(host, port)`, `send(conn_id, bytes)`,
`recv(conn_id, max_bytes)`, `close_conn(conn_id)`

### std/promise

`spawn`, `await`, `spawn_ordered`

> `spawn_ordered(fns)` spawns concurrently, joins in spawn order.

### std/chan

`new`, `send`, `recv`, `try_recv`, `close`, `is_closed`

### std/mutex

`new`, `lock`, `unlock`, `with_lock`

### std/sqlite

`open`, `exec`, `query`, `close`

> `sqlite.open(":memory:")` for in-memory. `exec` returns db handle. `query` returns list of records.

### std/menv

`new`, `set`, `get`, `has`, `child`, `call_eval`, `apply_closure`

> `menv.set` returns Unit — always `let _ = menv.set(...)`.

### std/async

`sleep`, `spawn`, `await`, `all`, `resolved`, `rejected`, `yield`, `race`, `timeout`

### std/cell

`new`, `get`, `set`

### std/option

`none`/`None`, `some`/`Some`, `is_none`/`isNone`, `is_some`/`isSome`,
`from_nullable`/`fromNullable`, `to_nullable`/`toNullable`,
`map`, `and_then`/`andThen`, `unwrap_or`/`unwrapOr`,
`unwrap_or_else`/`unwrapOrElse`, `to_result`/`toResult`

### std/result

`ok`, `err`, `and_then`/`andThen`, `unwrap_ok`, `unwrap_err`, `unwrap`,
`unwrap_or`, `is_ok`, `is_err`, `map`, `map_err`, `or_else`

### std/null

`isNull`, `coalesce`, `guardNotNull`

### std/type

`of`

**Verified type.of return values:**
- `type.of(42)` => `"int"`
- `type.of("hi")` => `"text"` (NOT "str")
- `type.of([1,2])` => `"list"`
- `type.of({a:1})` => `"record"`
- `type.of(null)` => `"unit"` (NOT "null")
- `type.of(true)` => `"bool"`

### std/cast

`float`, `int`, `text`

**CRITICAL:**
- `cast.text(65)` => `"A"` (Unicode codepoint 65) — NOT `"65"`
- `cast.text` is for Unicode codepoint conversion ONLY
- Use `str.from(n)` for number-to-string: `str.from(65)` => `"65"`
- `cast.int("42")` => `42`
- `cast.float("3.14")` => `3.14`

### std/re

`is_match`, `find`, `find_all`, `split`, `replace`

### std/eval

`eval`

### std/ast

`parse`

### std/ffi

`open`/`load`, `call`, `call_pure`, `call_checked`, `call_str`, `close`

### std/compress

`gzip`, `gunzip`

> Registered as `gzip`/`gunzip` — NOT `gzip_compress`/`gzip_decompress`.

### std/graph

`of`, `ancestors`, `leaves`, `to_dot`

### std/linalg

`zeros`, `eye`, `dot`, `norm`, `vec_add`, `vec_sub`, `vec_scale`,
`matvec`, `matmul`, `mat_add`, `mat_scale`, `transpose`, `eigh`

### std/flow

`id`, `pipe`, `tap`

### std/artifact

`import`, `emit`

> `ref` and `derive` are Unimplemented — the only two unimplemented builtins.

### std/witness

`self_digest`, `deps`, `verify`, `verify_chain`

### std/trace

`emit`, `info`, `warn`, `error`, `span`

### std/png

`red_1x1`, `encode`, `encode_rgb`, `encode_rgba`, `encode_palette`

### std/cli

`args`

-----

## Media Stack Modules

Import by relative path — not in the package registry.

### media-core/src/types

`image_spec(width, height, encoding)`, `video_spec(width, height, fps_num, fps_den, encoding)`

### media-core/src/bytes

`concat(a, b)`, `concat_many(list)`, `u32le(n)`, `u16le(n)`, `i16le(n)`, `ascii_bytes(str)`

### media-core/src/artifact

`write_receipt(path, payload_bytes, meta)`

### image-core/src/color

`clamp8(x)`, `rgb(r, g, b)`

### image-core/src/raster

`make(width, height, pixel_fn)`

> `pixel_fn(x, y)` called for every position row-major. Returns `{spec, pixels}`.
> To build from existing list: `fn(x, y) { list.get(pixels, y * width + x) }`

### image-core/src/draw

`gradient(width, height)`

### image-core/src/ppm

`ppm_bytes(width, height, pixels)`, `ppm_header(width, height)`, `ppm_pixel_bytes(pixels)`

### image-core/src/png

`png_bytes(width, height, pixels)`

### image-core/src/zlib0

`zlib_bytes(bs)`

### image-core/src/adler32

`adler32_int(bs)`, `adler32_be_bytes(bs)`

### image-core/src/crc32

`crc32_bytes(bs)`, `crc32_be_bytes(bs)`

### audio-core/src/pcm

`encode_pcm_s16le(samples)`

> Only `encode_pcm_s16le` is exported. `f32_to_pcm_s16` was inlined and removed.

### audio-core/src/synth

`sine(freq, duration_s, sample_rate)`

### audio-core/src/export

`write_wav(path, title, sample_rate, channels, samples)`

### video-core/src/frame

`make(index, pts_num, pts_den, raster)`, `test_pattern(width, height, index)`

### video-core/src/rawvid

`rawvid_bytes(width, height, fps_num, fps_den, frames)`, `frame_rgb_bytes(frame)`

### video-core/src/rawvid_decode

`header(bs)`, `frame_byte_len(h)`, `frame_offset(h, index)`, `frame_bytes(bs, index)`, `u32le_at(bs, off)`

### video-core/src/timeline

`make(width, height, fps_num, fps_den, frames)`, `test_pattern_video(width, height, fps_num, fps_den, count)`

### video-core/src/mux

`av_pair(video, audio)`, `with_audio(video, wav_artifact)`

### video-core/src/avbundle

`make(title, video_artifact, audio_artifact)`

### video-core/src/mp4_manifest

`make(title, video_artifact, audio_artifact)`

### video-core/src/webm_manifest

`make(title, video_artifact, audio_artifact)`

### video-core/src/transcode_contract

`make(kind, input_artifact, output_artifact_type, container, video_codec, audio_codec)`,
`rawvid_wav_to_mp4(input_artifact)`, `rawvid_wav_to_webm(input_artifact)`

### video-core/src/transcode_pipeline

`rawvid_frames_to_mp4(rawvid_bytes, wav_path, frames_dir, output_path)`,
`rawvid_frames_to_webm(rawvid_bytes, wav_path, frames_dir, output_path)`

### video-core/src/gifbridge

`gif_stub(video_artifact)`

### video-core/src/export

`write_rawvid(path, title, video)`

### pdf_v0/src/model

`make_bbox(x0,y0,x1,y1)`, `make_match(page,id,text,span_ids,bbox)`,
`make_overlay_item(page,kind,bbox,fill_rgb)`, `make_matches_doc(query,matches)`,
`make_overlay_plan(items)`

### pdf_v0/src/write_pdf

`build_pdf(rects)`

### pdf_v0/src/highlight

`find_matches(spans, query)`, `make_overlay(matches)`

### pdf_patch/src/patch_pdf

`patch_pdf(pdf_text, x0, y0, x1, y1)`

-----

## Media Decoders

### audio-core/src/wav_decode

`decode(bs)` => `{ ok: { header, samples } } | { err: text }`

> `header` = `{ channels, sample_rate, bits_per_sample }`. `samples` = `List(Int)`.

### image-core/src/ppm_decode

`decode(bs)` => `{ ok: { width, height, maxval, pixels } } | { err: text }`

> `width`, `height`, `maxval` are Int. `pixels` = `List({ r, g, b })`.

### image-core/src/png_decode

`decode(bs)` => `{ ok: { width, height, pixels } } | { err: text }`

> Filter types 0 (None), 1 (Sub), 2 (Up) supported. Types 3 and 4 not implemented.

-----

## Media Transforms

### image-core/src/transform

`resize(src, new_width, new_height)`, `crop(src, x0, y0, w, h)`,
`flip_h(src)`, `flip_v(src)`, `composite(dst, src, dx, dy)`,
`brightness(src, factor)`, `grayscale(src)`

### audio-core/src/transform

`gain(samples, factor)`, `trim(samples, sample_rate, start_ms, end_ms)`,
`mix(a, b)`, `fade_in(samples, fade_samples)`, `fade_out(samples, fade_samples)`,
`mono_to_stereo(samples)`, `stereo_to_mono(samples)`

-----

## Integration Packages

### packages/postgres-core

`connect(host, port, user, password, database)`, `exec(conn_id, sql)`,
`query(conn_id, sql)`, `close(conn_id)`

> `std/postgres` is an empty stub — import by path from packages/postgres-core.

### packages/ws-core

`connect(host, port, path)`, `send_text`, `send_binary`, `recv`, `close`

### packages/watch-core

`watch(paths, poll_ms, max_iters, handler)`, `watch_dir(path, poll_ms, max_iters, handler)`

> `diff(before, after)` => `{ added, removed, changed }`. Pass `max_iters: -1` for infinite polling.

### packages/xlsx-core

`build_xlsx(sheets)`, `write_xlsx(path, sheets)`

> `sheets` is `List({ name, rows })` where `rows` is `List(List(value))`.

### packages/avro-core

`write_ocf(records)` — writes Avro Object Container File. Infers schema from first record.

### packages/parquet-core

`build_parquet(records)`, `write_parquet(path, records)`

### packages/duckdb-core

`run(plan)` — executes query plan

> `std/duckdb` is an empty stub — import by path from packages/duckdb-core.
> Expr literals must use `{ "lit": value }` to distinguish from field name lookups.

### packages/wasm-core

`parse(bs)`, `call(wasm_bytes, fn_name, args)`, `exec_body(body, locals)`

-----

## Known Constraints and Gotchas

1. **`+` does not concatenate strings** — `"a" + "b"` is a type error. Use `str.concat("a", "b")` or `str.concat(["a", "b", "c"])`. The list form is preferred for 3+ parts.

1. **`str.concat` accepts two strings OR a list of strings** — both forms verified working.

1. **Missing imports cause runtime failures** — no auto-import. Always explicitly import `std/list`, `std/str`, etc.

1. **List destructuring `let [a, b] = expr` fails at module level** — use `let [a, b] = expr in body` or use `list.get` at module level.

1. **Rest capture `...rest` only works in `let...in` form** — at module level, rest bindings are silently dropped and unbound.

1. **`return` exits the immediately enclosing `fn {}` body only** — inside closures passed to `list.fold` etc., `return x` returns from the closure, not the outer function. Use recursion for early exit.

1. **Default parameters are NOT implemented** — `fn f(x, y = 0)` parses but calling `f(1)` causes arity mismatch. Always pass all arguments.

1. **Named args work in v1.7.0** — `add(y: 3, x: 7)` works for all function types. Fixed regression from earlier v1.7.0 builds.

1. **`while` returns a structured result** — `{ chain_hex: string, steps: int, value: final_state }`. Access the result with `.value`.

1. **`type.of(null)` returns `"unit"`** — not `"null"`. Null is `Val::Unit` internally.

1. **`cast.text(n)` converts to Unicode codepoint** — `cast.text(65)` => `"A"`. Use `str.from(n)` for number-to-string.

1. **`int.parse` returns `{t: "ok", v: n}`** — access value with `.v`. Use `?` to unwrap: `int.parse("42")?` => `42`.

1. **`?` operator requires a result record** — unwraps `{t:"ok",v:val}`, propagates `{t:"err",...}`. On other values: `QMARK_EXPECT_RESULT` error.

1. **`list.find` returns `{some: value}` or `{none: unit}`** — access with `.some`.

1. **`list.concat` takes one argument** — a list of lists: `list.concat([[1,2],[3,4]])` => `[1,2,3,4]`.

1. **`rec.remove` not `rec.delete`** — correct function name is `remove`.

1. **`std/compress` uses `gzip`/`gunzip`** — not `gzip_compress`/`gzip_decompress`.

1. **`std/graph` uses `of`/`ancestors`/`leaves`/`to_dot`** — not add_node/add_edge/bfs.

1. **`std/duckdb` and `std/postgres` are empty stubs** — import from packages/ by path.

1. **`io.read_file` returns `{ok: text}`** on success — no `err` field. Accessing `.err` crashes.

1. **`menv.set` returns Unit** — always `let _ = menv.set(...)`.

1. **Parser limit on deep if/else chains with complex record expressions** — 4+ branch if/else with complex record literals can fail. Hoist sub-expressions into `let` bindings first.

1. **`[[...]]` as fn tail works correctly** — newline check before `[` resolves list-of-lists ambiguity.

1. **`raster.make` takes `pixel_fn(x, y)`** — not a pre-built pixel list.

1. **`duckdb-core` expr literals must use `{ "lit": value }`** — bare strings are treated as field name lookups.

-----

*Fully verified against fardrun v1.7.0 — 2 May 2026*
