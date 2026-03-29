# FARD

FARD is a deterministic, content-addressed scripting language where every execution produces a cryptographic receipt.

Each run emits a SHA-256 digest committing to the source code, imported modules, inputs, intermediate computation steps, and final result.

Two executions of the same program on the same inputs produce the same digest.

Traceability is not a feature — it is an invariant of execution.

**Version:** v1.6.0 — [Releases](https://github.com/mauludsadiq/FARD/releases)

-----

## What FARD Is

FARD is a deterministic scripting language with a functional core, controlled mutable state, and a content-addressed execution runtime.

It provides:

- cryptographic witness receipts on every run
- 77 standard library modules
- 242 built-in primitives
- a 13-binary toolchain
- native FFI via dynamic library loading
- a WebAssembly compilation target
- an LSP server with go-to-definition and find-references
- a SQLite-backed receipt registry with CRDT replication
- a content-addressed package manager with 164 packages and semver ranges
- a web playground (playground/index.jsx)
- a doc generator (farddoc)
- a verifiable build system (fard-build)
- distributed receipt convergence via Inherit-Cert CRDT

FARD turns program execution itself into a cryptographic artifact.

Programs written in FARD do not merely return values — they return values together with verifiable evidence of how those values were computed. Every receipt binds the result to the exact source, imports, inputs, and execution history that produced it. Any execution can therefore be independently verified on another machine.

-----

## Install

```bash
# macOS (Apple Silicon)
curl -L https://github.com/mauludsadiq/FARD/releases/latest/download/fard-macos-aarch64.tar.gz | tar xz
sudo mv fard-macos-aarch64/fard* /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/mauludsadiq/FARD/releases/latest/download/fard-macos-x86_64.tar.gz | tar xz
sudo mv fard-macos-x86_64/fard* /usr/local/bin/

# Linux
curl -L https://github.com/mauludsadiq/FARD/releases/latest/download/fard-linux-x86_64.tar.gz | tar xz
sudo mv fard-linux-x86_64/fard* /usr/local/bin/
```

Or build from source:

```bash
git clone https://github.com/mauludsadiq/FARD.git
cd FARD
cargo build --release
```

-----

## Quick Start

```bash
fardrun new my-project
cd my-project
fardrun run --program main.fard --out ./out
cat out/result.json
```

-----

## Language

### Values

```
42              // Int   (64-bit signed)
3.14            // Float (64-bit IEEE 754)
true            // Bool
null            // Unit
"hello"         // Text
[1, 2, 3]       // List
{ x: 1, y: 2 }  // Record
```

String interpolation:

```
let name = "world"
"hello ${name}"
```

### Functions

```
fn add(a, b) { a + b }

let double = fn(x) { x * 2 }

fn make_adder(n) { fn(x) { x + n } }
let add5 = make_adder(5)
add5(10)   // -> 15
```

Named arguments:

```
fn greet(name, greeting) { str.concat(greeting, str.concat(" ", name)) }
greet(name: "Alice", greeting: "Hello")
```

Default arguments:

```
fn greet(name, greeting = "Hello") { str.concat(greeting, str.concat(", ", name)) }
greet("Alice", null)     // uses default -> "Hello, Alice"
greet("Bob", "Hi")       // explicit override -> "Hi, Bob"
```

### Let Bindings

```
let x = 42
let y = x * 2
let result = let x = 10 in let y = 20 in x + y
```

### Conditionals and Pattern Matching

```
if x > 0 then "positive" else "non-positive"

match type.of(x) {
  "int"  => str.from_int(x),
  "text" => x,
  _      => "other"
}
```

### For and List Comprehensions

```
let doubled      = for x in [1, 2, 3, 4, 5] do x * 2
let squares      = [x * x for x in [1, 2, 3, 4, 5]]
let even_squares = [x * x for x in [1, 2, 3, 4, 5] if x % 2 == 0]
```

### Recursion

Tail-recursive functions are optimised.

```
fn factorial(n) { if n <= 1 then 1 else n * factorial(n - 1) }
fn sum_to(n, acc) { if n == 0 then acc else sum_to(n - 1, acc + n) }
sum_to(1000000, 0)
```

### While (Hash-Chained Iteration)

`while` is a hash-chained ledger of every state transition, producing a cryptographic certificate of the entire computation.

```
let result = while {n: 0, acc: 0}
  fn(s) { s.n < 10 }
  fn(s) { {n: s.n + 1, acc: s.acc + s.n} }

result.value      // {n: 10, acc: 45}
result.chain_hex  // sha256 of the full computation history
```

### Mutable Cells

```
import("std/cell") as cell

let counter = cell.new(0)
let _       = cell.set(counter, cell.get(counter) + 1)
cell.get(counter)   // -> 1
```

### Imports

```
import("std/math")   as math
import("std/list")   as list
import("./mylib")    as mylib
import("pkg:greet")  as greet
```

-----

## Standard Library (53 Modules)

### Core Data

**std/str** — `len`, `concat`, `join`, `split`, `slice`, `upper`, `lower`, `trim`, `contains`, `starts_with`, `ends_with`, `pad_left`, `pad_right`, `repeat`, `index_of`, `chars`, `replace`, `from_int`, `from_float`

**std/list** — `map`, `filter`, `fold`, `any`, `all`, `find`, `find_index`, `flat_map`, `take`, `drop`, `zip_with`, `chunk`, `sort_by`, `par_map`, `len`, `range`, `reverse`, `concat`, `group_by`

**std/map** — `new`, `get`, `set`, `has`, `delete`, `keys`, `values`, `entries`

**std/set** — `new`, `add`, `remove`, `has`, `union`, `intersect`, `diff`, `to_list`, `from_list`, `size`

**std/rec** / **std/record** — `get`, `set`, `has`, `keys`, `merge`, `delete`

**std/option** — `some`, `none`, `is_some`, `is_none`, `unwrap`, `unwrap_or`, `map`, `and_then`, `from_nullable`, `to_nullable`

**std/result** — `ok`, `err`, `is_ok`, `is_err`, `unwrap`, `unwrap_or`, `map`, `map_err`, `and_then`, `or_else`

**std/type** — `of`, `is_int`, `is_float`, `is_bool`, `is_text`, `is_list`, `is_record`, `is_null`, `is_fn`

**std/cast** — `int`, `float`, `text`

**std/null** — `is_null`, `coalesce`

### Numbers

**std/math** — `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `log`, `log2`, `log10`, `sqrt`, `pow`, `abs`, `floor`, `ceil`, `round`, `pi`, `e`

**std/float** — `add`, `sub`, `mul`, `div`, `sqrt`, `abs`, `ln`, `pow`, `neg`, `le`, `gt`, `from_int`, `to_text`, `to_str_fixed`, `is_nan`, `is_inf`

**std/int** — `to_str_padded`

**std/bigint** — Arbitrary-precision integers: `from_int`, `from_str`, `to_str`, `add`, `sub`, `mul`, `div`, `mod`, `pow`, `eq`, `lt`, `gt`

**std/bits** — `band`, `bor`, `bxor`, `bnot`, `bshl`, `bshr`

**std/linalg** — `zeros`, `eye`, `dot`, `norm`, `vec_add`, `vec_sub`, `vec_scale`, `transpose`, `matmul`, `relu`, `softmax`, `argmax`, `eigh`

**std/uuid** — `v4`, `validate`

### Text and Encoding

**std/re** — `is_match`, `find`, `find_all`, `split`, `replace`

**std/json** — `encode`, `decode`, `canonicalize`

**std/base64** — `encode`, `decode`

**std/codec** — `base64url_encode`, `base64url_decode`, `hex_encode`, `hex_decode`

**std/csv** — `parse`, `encode`

**std/bytes** — `concat`, `len`, `get`, `of_list`, `to_list`, `of_str`, `to_str`

### Cryptography and Hashing

**std/hash** — `sha256_bytes`, `sha256_text`

**std/crypto** — `hmac_sha256`, `aes_encrypt`, `aes_decrypt`, `pbkdf2`, `ed25519_sign`, `ed25519_verify`

### I/O and System

**std/io** — `read_file`, `write_file`, `append_file`, `read_lines`, `read_stdin`, `read_stdin_lines`, `file_exists`, `delete_file`, `list_dir`, `make_dir`

**std/fs** — `read`, `write`, `exists`, `stat`, `list`

**std/path** — `join`, `base`, `dir`, `ext`, `isAbs`, `normalize`

**std/env** — `get`, `args`

**std/process** — `spawn`, `exit`

**std/http** — `get`, `post`, `request`

**std/net** — `serve(port, handler_fn)`

### Time

**std/datetime** — `now`, `format`, `parse`, `add`, `diff`, `field`

**std/time** — `now_ms`, `sleep_ms`

### Concurrency

**std/promise** — `spawn`, `await`

**std/chan** — `new`, `send`, `recv`, `try_recv`, `close`

**std/mutex** — `new`, `lock`, `unlock`, `with_lock`

**std/cell** — `new`, `get`, `set`

### Compression

**std/compress** — `gzip_compress`, `gzip_decompress`, `zstd_compress`, `zstd_decompress`

### Metaprogramming

**std/eval** — `eval(source_text)`

**std/ast** — `parse(source_text)`

### Tracing and Observability

**std/trace** — `info`, `warn`, `error`, `span`

**std/witness** — `verify`, `verify_chain`, `self_digest`

**std/artifact** — bind a prior verified run by RunID

### Interoperability

**std/ffi** — `load`, `call`, `call_pure`, `call_str`, `close`

**std/png** — `red_1x1`

**std/cli** — command-line argument parsing

### Domain-Specific

**std/graph** — `new`, `add_node`, `add_edge`, `bfs`, `dfs`, `shortest_path`, `topo_sort`

**std/sembit** — semantic bitfield partitioning

**std/grow** — `append`, `merge`, `unfold`, `unfold_tree`

**std/flow** — `id`, `pipe`, `tap`

-----

## Package Manager

Packages are versioned, SHA-256 verified, and cached locally. Semver ranges are supported.

```toml
name = "my-project"
version = "1.0.0"
entry = "main.fard"

[deps]
greet  = "greet@1.6.0"
jwt    = "jwt@^1.6.0"
stream = "stream@~1.6.0"
stats  = "stats@>=1.0.0"
```

```bash
fardrun install --manifest fard.toml
fardrun search jwt
fardrun search
```

```
import("pkg:greet") as greet
greet.hello("world")
```

Registry: `https://github.com/mauludsadiq/FARD/releases/latest/download/registry.json`

### Available Packages (58)

| Package | Category | Description |
|---|---|---|
| `tensor@1.6.0` | Data Science | N-dimensional arrays, activations, matmul |
| `frame@1.6.0` | Data Science | Typed columnar DataFrame |
| `plot@1.6.0` | Data Science | SVG chart generation |
| `stats@1.6.0` | Data Science | Descriptive statistics |
| `table@1.6.0` | Data Science | In-memory tabular data |
| `stream@1.6.0` | Data Science | Lazy sequences |
| `csv-stream@1.6.0` | Data Science | Streaming CSV for large files |
| `parse@1.6.0` | Data Science | Parser combinators |
| `fard-web@1.6.0` | HTTP | Full web framework |
| `http-client@1.6.0` | HTTP | HTTP client with retry and JSON helpers |
| `http-server@1.6.0` | HTTP | HTTP server with routing |
| `http-middleware@1.6.0` | HTTP | CORS, auth, logging, rate-limit middleware |
| `async@1.6.0` | Async | par_map, all, race, retry, worker pool, pipeline |
| `jwt@1.6.0` | Auth | JWT encode/decode/verify (HS256) |
| `hmac-sign@1.6.0` | Auth | HMAC-SHA256 request signing |
| `oauth2@1.6.0` | Auth | OAuth2 client flows |
| `kv@1.6.0` | Storage | Persistent key-value store |
| `sqlite@1.6.0` | Storage | SQLite client via FFI |
| `s3@1.6.0` | Storage | S3-compatible object storage |
| `toml@1.6.0` | Data/Text | TOML parsing and generation |
| `yaml@1.6.0` | Data/Text | YAML parsing and generation |
| `template@1.6.0` | Data/Text | Jinja2-style string templating |
| `json-schema@1.6.0` | Data/Text | JSON schema validation |
| `csv-extra@1.6.0` | Data/Text | Enhanced CSV |
| `diff@1.6.0` | Data/Text | Text diffing |
| `xml@1.6.0` | Data/Text | XML parsing and generation |
| `markdown@1.6.0` | Data/Text | Markdown to HTML |
| `fard-test@1.6.0` | Dev Tools | Assertion library |
| `fard-bench@1.6.0` | Dev Tools | Microbenchmarking with witnessed results |
| `fard-mock@1.6.0` | Dev Tools | Mock HTTP server for testing |
| `fard-lint@1.6.0` | Dev Tools | Custom lint rules |
| `fard-check@1.6.0` | Dev Tools | Runtime type and schema validation |
| `semver@1.6.0` | Build/CI | Semantic versioning |
| `glob@1.6.0` | Build/CI | File glob matching |
| `shell@1.6.0` | Build/CI | Safe shell command composition |
| `env-config@1.6.0` | Build/CI | Structured config from environment and files |
| `fard-ci@1.6.0` | Build/CI | CI pipeline primitives with witnessed steps |
| `logger@1.6.0` | Infrastructure | Structured logging |
| `cache@1.6.0` | Infrastructure | In-memory TTL cache |
| `config@1.6.0` | Infrastructure | Layered configuration |
| `queue@1.6.0` | Infrastructure | Persistent FIFO queue |
| `pubsub@1.6.0` | Infrastructure | Publish/subscribe event bus |
| `rate-limiter@1.6.0` | Infrastructure | Token bucket rate limiter |
| `websocket@1.6.0` | Protocols | WebSocket frame encoding |
| `smtp@1.6.0` | Protocols | Email composition and SMTP |
| `uuid@1.6.0` | Utilities | UUID generation |
| `base64@1.6.0` | Utilities | Base64 encode/decode/url |
| `crypto-extra@1.6.0` | Utilities | SHA-256, HMAC, key derivation |
| `regex-extra@1.6.0` | Utilities | Glob matching, validators |
| `fard-fmt-extra@1.6.0` | Utilities | Number, byte, duration formatting |
| `fard-deploy@1.6.0` | Deployment | SSH, rsync, Docker, systemd |
| `fard-watch@1.6.0` | Deployment | Poll-based file watching |
| `fard-notebook@1.6.0` | Literate | Literate programming with HTML export |

-----

## Concurrency

```
import("std/promise") as promise
import("std/chan")     as chan
import("std/list")    as list

let p1 = promise.spawn(fn() { expensive_a() })
let p2 = promise.spawn(fn() { expensive_b() })
let a  = promise.await(p1)
let b  = promise.await(p2)

list.par_map([1, 2, 3, 4, 5], fn(x) { x * x })

let c = chan.new()
chan.send(c, 42)
chan.recv(c)   // -> {t: "some", v: 42}
```

-----

## Cryptographic Witnessing

### Self-Digest

```
import("std/witness") as w
w.self_digest()   // -> "sha256:e60cb9e82ac28f..."
```

### Artifact Binding

```
artifact step1 = "sha256:689dede5..."
step1.output
```

### Proof-Carrying Code

```bash
fardverify prove --out ./out --spec spec.json
```

```json
{
  "obligations": [
    {"type": "no_errors"},
    {"type": "has_child_receipts", "min": 3},
    {"type": "result_field", "field": "sum", "min": 1400, "max": 1400}
  ]
}
```

### Distributed Receipt Convergence (Inherit-Cert CRDT)

The Inherit-Cert CRDT is a Min-Register Map satisfying all four semilattice laws. After one round of merge, all replicas converge on the canonical (lexicographic minimum) RunID for each effect.

```bash
curl -X POST http://registry/crdt/propose \
  -d '{"effect_kind":"http_get","req_hex":"...","run_id":"sha256:aaa..."}'
curl http://registry/crdt/state
```

-----

## FFI

```
import("std/ffi") as ffi

let lib    = ffi.load("/usr/lib/libm.dylib")
let result = ffi.call(lib.ok, "abs", [-42])
result.ok   // -> 42

let r2 = ffi.call_pure(lib.ok, "abs", [-7])
```

Type mapping: `Int` -> `i64`, `Float` -> `f64`, `Text` -> `char*`, `Bool` -> `0/1`

-----

## WebAssembly

```bash
fardwasm main.fard --out main.wat
fardwasm main.fard --target wasi --out main.wasm
```

-----

## Verifiable Build System

```toml
[build]
name = "my-project"
version = "1.0.0"

[[step]]
name = "compile"
program = "steps/compile.fard"
out = "build/compile"

[[step]]
name = "test"
program = "steps/test.fard"
out = "build/test"
depends_on = ["compile"]
```

```bash
fard-build --config fard.build.toml --out build/
fard-build --verify --out build/
```

Each step produces a cryptographic receipt. The `build.receipt.json` chains all step digests. Any change to any step breaks the chain.

-----

## Documentation Generation

```bash
farddoc --package packages/stats --out docs/ --format html
farddoc --out docs/
farddoc --program main.fard --format md
```

Doc comment syntax:

```
/// Returns the sum of all elements in a list.
/// xs: List(Int|Float)
fn sum(xs) { ... }
```

-----

## CLI

### fardrun

```bash
fardrun new my-project
fardrun run --program main.fard --out ./out
fardrun test --program math.fard
fardrun repl
fardrun install --manifest fard.toml
fardrun search jwt
```

Output: `result.json`, `error.json`, `trace.ndjson`, `module_graph.json`, `digests.json`

### fardverify

```bash
fardverify trace  --out ./out
fardverify chain  --out ./out --registry ./registry
fardverify prove  --out ./out --spec spec.json
fardverify bundle --out ./out
```

### fard-build

```bash
fard-build --config fard.build.toml --out build/
fard-build --verify --out build/
fard-build --step test
```

### fardregistry

```bash
fardregistry --port 7370 --db receipts.db
# CRDT routes: GET /crdt/state  POST /crdt/propose  POST /crdt/merge
```

-----

## VS Code Extension

```bash
code --install-extension editors/vscode/fard-language-0.1.0.vsix
```

Syntax highlighting, inline diagnostics, dot-completion, hover docs, go-to-definition (F12), find-all-references (Shift+F12).

-----

## Binaries

| Binary | Purpose |
|---|---|
| `fardrun` | Runtime: run, test, repl, new, install, search, publish |
| `fardfmt` | Canonical formatter |
| `fardcheck` | HM-style type checker |
| `fardwasm` | FARD to WAT/WASM compiler |
| `fardregistry` | Receipt registry server with CRDT routes |
| `fardlock` | Lockfile generation and enforcement |
| `fardbundle` | Bundle build, verify, and run |
| `fardverify` | Trace, chain, proof, and bundle verification |
| `fardpkg` | Package management |
| `fard-lsp` | Language Server Protocol |
| `fardc` | Compiler frontend and canonicalizer |
| `farddoc` | Documentation generator |
| `fard-build` | Verifiable build system |

-----

## Self-Hosting (v1.6.0)

FARD now owns its intermediate representation and first compiler stage,
implemented entirely in FARD.

**FARD-native Parser** (`packages/fard_parse/parse.fard`)
Parses FARD source text into AST nodes. Handles:
- Identifiers and variables
- Integer and text literals
- Binary operators with correct precedence (* before +/-)
- Left associativity for + and -
- Parenthesized expressions: (x + y) * z
- Multi-parameter function signatures
- Comparison operators: ==, !=, <, <=, >, >=
- Boolean logic: and, or, not (with correct precedence: cmp < logic < if)
- Compound predicates for control flow

**FIR v1** — FARD Intermediate Representation (`packages/fir/fir.fard`)
A minimal IR with constructors for literals, variables, let bindings,
function definitions, calls, and modules.

**AST→FIR Lowering** (`packages/fard_lower/lower.fard`)
Lowers FARD AST nodes to FIR. Handles: int, text, var, let, fn,
binary operators (+, -, *, /), and module assembly.

**FIR Evaluator** (`packages/fard_eval/eval.fard`)
Executes FIR directly in FARD. Supports:
- Literals (int, text, bool)
- Variables and let bindings
- Closures with environment capture
- User-defined function calls
- Builtin dispatch (int.add/sub/mul/div)
- Comparison operators (==, !=, <, <=, >, >=)
- Short-circuit boolean evaluation (and, or, not)
- Control flow (if/then/else)
- Module evaluation

**AST Type Checker** (`packages/fard_type/typecheck.fard`)
Static validation before evaluation. Handles:
- Type environment (tenv) with let and def_fn binding
- Builtin typing rules (int.*, comparisons)
- Function type collapse: fn_type(params, ret)
- Typed call nodes with arity checking
- Recursive def_fn via self-binding in type env
- Structured type error propagation

**Hindley-Milner Type Inference** (`packages/fard_hm/hm.fard`)
Algorithm W implementation in FARD. Includes:
- Type variables and substitution
- Occurs check and unification
- Let generalization and let-rec binding
- Full multi-arg support via curried tfun folding
- Inference for literals, variables, functions, multi-arg calls
- Module-level HM with threaded type environment
- Error locations with line/col via token_pos_to_line_col
- Integrated into fardrun as `--hm-types` gate
- Forward reference resolution via preregister_defs pre-pass
- Mutual recursion support
- Graceful error node handling (fresh tvar, no crash)
- All 4 core self-hosting packages pass --hm-types (M4)

**End-to-end pipeline:** source text → parse → lower → typecheck → infer → eval → result, fully in FARD.
Rust is no longer required for execution or type checking of core functional programs.
This is a complete compiler frontend written in FARD.

**Self-host bootstrap (v1.6.0):** FARD's evaluator executes FARD's evaluator.

Infrastructure added:
- `Val::MutEnv` — arc-shared mutable environment for bootstrap cycles
- `std/menv` — module: new/set/get/has/child/call_eval/apply_closure
- `fir_val_to_expr` — converts FIR record values to native Rust Expr
- `apply_record_closure` — executes FARD closures stored as records
- `get_field` pattern — handles `int.add` style dotted match patterns

Verified self-hosted execution:
- lit_int, lit_bool, var, call_builtin, if_node, eval_args

The self-hosted evaluator is now the execution substrate for the
self-hosting pipeline. FARD evaluates FARD.

-----

## Determinism

Given identical source code, imports, and inputs, FARD guarantees identical results, identical execution traces, and identical execution digests across machines, operating systems, and time.

Oracle boundaries — `std/http`, `std/datetime.now`, `std/io.read_stdin`, `std/uuid.v4`, `std/ffi.call` — are explicitly marked. Their observed values are recorded in the execution trace so runs remain auditable even when interacting with the outside world.

-----

## Architecture

```
Layer 5  Execution ABI v0        bundle -> witness bytes
Layer 4  Registry Semantics v0   content-addressed receipt storage
Layer 3  Composition Semantics   executions link by verified RunID
Layer 2  Artifact Semantics      same (program, input, deps) -> same RunID
Layer 1  Value Core v0           same value -> same bytes -> same hash
```

The entire system reduces to one primitive:

```
CID(bytes) = "sha256:" || hex(SHA256(bytes))
```

-----

## Self-Verifying

313 tests across 36 files, all written in pure FARD:

```bash
for f in tests/test_*.fard; do fardrun test --program "$f"; done
```

-----

## Specifications

| Document | Contents |
|---|---|
| `spec/fard_spec_stack_v0_final.md` | Trust stack specification (frozen) |
| `spec/fardlang_grammar_v0.5.txt` | Surface language grammar |
| `SPEC.md` | Stdlib surface spec (generated) |
| `ANNOUNCEMENT.md` | Release announcement (generated) |

-----

## License

MUI
