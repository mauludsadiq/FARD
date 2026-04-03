# FARD — ISO EBNF Grammar

**31 Mar 2026 · v1.6.0 · updated for data pipeline domain**

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
(* Float literals lex to Tok::Float, evaluate to Val::Bytes (8-byte LE f64).
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

|Variant                       |Description                                                             |
|------------------------------|------------------------------------------------------------------------|
|`Unit`                        |unit / null                                                             |
|`Bool(bool)`                  |true / false                                                            |
|`Int(i64)`                    |64-bit signed integer                                                   |
|`Float(f64)`                  |Produced only by `json.decode` on JSON floats or `std/math` constants   |
|`Text(String)`                |UTF-8 text (note: field is `Text`, not `Str`)                           |
|`Bytes(Vec<u8>)`              |Raw bytes — produced by `std/bytes` operations. Not used for floats.                  |
|`List(Vec<Val>)`              |Ordered list                                                            |
|`Record(BTreeMap<String,Val>)`|Record with sorted string keys                                          |
|`Err { code, data }`          |Error value with string code and data payload                           |
|`Func`                        |Closure — params + body + captured env                                  |
|`Builtin`                     |Native stdlib function pointer                                          |
|`BoundMethod(receiver, fn)`   |Method bound to a receiver value                                        |
|`Chan`                        |Channel for concurrent communication (`std/chan`)                       |
|`Mtx`                         |Mutex (`std/mutex`)                                                     |
|`Big(BigInt)`                 |Arbitrary-precision integer (`std/bigint`)                              |
|`Promise`                     |Async promise (`std/promise`)                                           |

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

> `std/record` — now aliases `std/rec`. Both work identically.

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
`from_int`, `to_int`, `from_text`, `to_text`, `add`, `sub`, `mul`, `div`,
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

`serve`

### std/promise

`spawn`, `await`, `spawn_ordered`

> `spawn_ordered(fns)` spawns a list of zero-argument functions concurrently and joins in spawn order. Returns a list of results with guaranteed ordering — same digest across runs.

### std/chan

`new`, `send`, `recv`, `try_recv`, `close`, `is_closed`

### std/mutex

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


`new`, `lock`, `unlock`, `with_lock`

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
-----

*Audited against fardrun v1.6.0. Canonical source: `src/bin/fardrun.rs`.*