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
- 53 standard library modules
- 242 built-in primitives
- an 11-binary toolchain
- native FFI via dynamic library loading
- a WebAssembly compilation target
- an LSP server with VS Code extension
- a SQLite-backed receipt registry
- a content-addressed package manager with 58 packages

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
// for..in..do sugar for list.map
let doubled = for x in [1, 2, 3, 4, 5] do x * 2

// list comprehension
let squares = [x * x for x in [1, 2, 3, 4, 5]]

// with filter
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

Packages are versioned, SHA-256 verified, and cached locally.

```toml
name = "my-project"
version = "1.0.0"
entry = "main.fard"

[deps]
greet = "greet@1.6.0"
```

```bash
fardrun install --manifest fard.toml
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
| `fard-web@1.6.0` | HTTP | Full web framework — routing, path params, validation, OpenAPI |
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

## Metaprogramming

```
import("std/eval") as e
e.eval("fn double(n) { n * 2 }\ndouble(21)")   // -> 42

import("std/ast") as ast
let nodes = ast.parse("1 + 2")
nodes[0].t    // -> "bin"
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

### Chain Verification

```
import("std/witness") as w
let r = w.verify_chain("sha256:47912fef...")
r.t      // "ok"
r.depth  // depth of the verified chain
```

### Distributed Verification

```bash
export FARD_REGISTRY_URL=http://registry.example.com:7370
fardrun run --program main.fard --out ./out
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

## CLI

### fardrun

```bash
fardrun new my-project
fardrun run --program main.fard --out ./out
fardrun test --program math.fard
fardrun repl
fardrun install --manifest fard.toml
```

Output: `result.json`, `error.json`, `trace.ndjson`, `module_graph.json`, `digests.json`

### fardrun repl

```
FARD v1.6.0 REPL
  :quit / :q      exit
  :help           show commands
  :reset          clear environment
  :vars           show defined names
  :time <expr>    time an expression

fard> let x = 42
fard> x * 2
84
fard> :time list.range(0, 100000)
time: 4.2ms
```

### fardrun test

```
test "basic"       { gcd(12, 8) == 4 }
test "commutative" { gcd(8, 12) == gcd(12, 8) }
```

### fardfmt

```bash
fardfmt main.fard
fardfmt --check main.fard
```

### fardcheck

```bash
fardcheck main.fard
# ok -- 47 items checked, 0 errors
```

### fardlock

```bash
fardlock gen-toml --manifest fard.toml --out fard.lock.json
fardrun run --program main.fard --lockfile fard.lock.json --enforce-lockfile
```

### fardbundle

```bash
fardbundle build  --root . --entry main.fard --out ./bundle
fardbundle verify --bundle bundle/bundle.json --out ./out
fardbundle run    --bundle bundle/bundle.json --out ./out
```

### fardregistry

```bash
fardregistry --port 7370 --db receipts.db --seed receipts/
```

-----

## VS Code Extension

```bash
code --install-extension editors/vscode/fard-language-0.1.0.vsix
```

Syntax highlighting, inline diagnostics, dot-completion for all 53 stdlib modules, hover documentation, import path completion.

```json
{ "fard.lspPath": "/usr/local/bin/fard-lsp" }
```

-----

## Binaries

| Binary | Purpose |
|---|---|
| `fardrun` | Runtime: `run`, `test`, `repl`, `new`, `install`, `publish` |
| `fardfmt` | Canonical formatter |
| `fardcheck` | HM-style type checker |
| `fardwasm` | FARD to WAT/WASM compiler |
| `fardregistry` | SQLite-backed receipt registry server |
| `fardlock` | Lockfile generation and enforcement |
| `fardbundle` | Bundle build, verify, and run |
| `fardverify` | Trace, artifact, and bundle verification |
| `fardpkg` | Package management |
| `fard-lsp` | Language Server Protocol |
| `fardc` | Compiler frontend and canonicalizer |

-----

## Error Messages

```
Error: unbound var: str -- did you forget to import? Try: import("std/str") as str
Error: no member 'mpa' -- did you mean 'map'?
Error: arity mismatch: expected 2 args, got 3
```

| Code | Meaning |
|---|---|
| `ERROR_PARSE` | Syntax error |
| `ERROR_RUNTIME` | Runtime failure |
| `ERROR_DIV_ZERO` | Division by zero |
| `ERROR_PAT_MISMATCH` | Pattern match failed |
| `ERROR_ARITY` | Wrong number of arguments |
| `ERROR_BADARG` | Wrong argument type for a builtin |
| `ERROR_IO` | File or network I/O failure |
| `ERROR_LOCK` | Lockfile enforcement failure |
| `ERROR_FFI` | Foreign function interface error |

-----

## Determinism

Given identical source code, imports, and inputs, FARD guarantees identical results, identical execution traces, and identical execution digests across machines, operating systems, and time.

Oracle boundaries — `std/http`, `std/datetime.now`, `std/io.read_stdin`, `std/uuid.v4`, `std/ffi.call` — are explicitly marked. Their observed values are recorded in the execution trace so runs remain auditable even when interacting with the outside world.

`std/ffi.call_pure` declares a foreign call deterministic and includes its result in the witness hash chain.

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
