# FARD

FARD is a deterministic, content-addressed scripting language. Every execution produces a SHA-256 receipt committing to source code, imports, inputs, intermediate steps, and final result. Two runs of the same program on the same inputs produce the same digest — on any machine, at any time.

Traceability is not a feature. It is an invariant of execution.

**Version:** v1.7.0 — [Releases](https://github.com/mauludsadiq/FARD/releases)

```bash
curl -sf https://raw.githubusercontent.com/mauludsadiq/FARD/main/install.sh | sh
```

Installs `fardrun` to `/usr/local/bin`. Detects macOS arm64/x86_64 and Linux x86_64 automatically. Or build from source:

```bash
git clone https://github.com/mauludsadiq/FARD.git && cd FARD
cargo build --release --bin fardrun
```

-----

## Built with FARD

**Qasim** — cryptographically verifiable financial state engine. Ingests signed fills, instruments, corporate actions, and multi-source price feeds. Computes recency-weighted consensus prices, Greeks, Monte Carlo risk, and unified NAV across public, private, and derivatives books. Every output is traceable:

```
curl state_at/ACCT-123/1775710735 -> sha256:6f73405b...
curl state_at/ACCT-123/1775710735 -> sha256:6f73405b...
```

120 files. 4,930 lines. 135 tests. Pure FARD. https://github.com/mauludsadiq/Qasim-in-FARD

-----

## Language

### Values

```fard
42               // Int   (64-bit signed)
3.14             // Float (64-bit IEEE 754)
true             // Bool
null             // Unit
"hello"          // Text
"hello ${name}"  // Interpolated string
[1, 2, 3]        // List
{ x: 1, y: 2 }  // Record
```

### Functions

```fard
fn add(a, b) { a + b }
let double = fn(x) { x * 2 }
fn make_adder(n) { fn(x) { x + n } }

// Named arguments
greet(name: "Alice", greeting: "Hello")

// Lambdas
list.map(xs, x => x * 2)
list.fold(xs, 0, fn(acc, x) { acc + x })
```

### Bindings and Control Flow

```fard
let x = 42
let result = let x = 10 in let y = 20 in x + y

if x > 0 then "positive" else "non-positive"

match type.of(x) {
  "int"  => str.from(x),
  "text" => x,
  _      => "other"
}
```

### Collections

```fard
// Pipe operator
list.range(1, 11)
  |> list.filter(fn(x) { x % 2 == 0 })
  |> list.map(fn(x) { x * x })

// Record spread and computed keys
let updated = { ...defaults, color: "red", [dynamic_key]: value }

// Destructuring
let { name, age } = user
let [first, ...rest] = items in first
```

### While — Hash-Chained Iteration

`while` produces a cryptographic certificate of the entire computation. Every state transition is hashed into a chain.

```fard
let result = while { n: 0, acc: 0 }
  fn(s) { s.n < 10 }
  fn(s) { { n: s.n + 1, acc: s.acc + s.n } }

result.value      // { n: 10, acc: 45 }
result.chain_hex  // sha256 of full computation history
result.steps      // 10
```

### Ergonomics

```fard
// Safe navigation and null-coalescing
let host = config?.db?.host ?? "localhost"

// Result unwrap with propagation
let value = int.parse("42")?   // -> 42

// String concatenation
str.concat(["Hello, ", name, "!"])
```

### Imports

```fard
import("std/math")               as math
import("./mylib")                as mylib
import("packages/web-core/app")  as app
```

-----

## Standard Library

65 modules. All verified against fardrun v1.7.0.

**Data** — `str`, `list`, `rec`, `map`, `set`, `option`, `result`

**Numbers** — `math`, `float`, `int`, `bigint`, `bits`, `linalg`

**Encoding** — `json`, `csv`, `bytes`, `base64`, `codec`, `re`

**Crypto** — `hash` (SHA-256), `crypto` (HMAC, AES, Ed25519), `uuid`

**I/O** — `io`, `fs`, `path`, `env`, `process`

**Network** — `net` (TCP), `http`, `ws` (WebSocket)

**Concurrency** — `promise`, `chan`, `mutex`, `cell`

**Storage** — `sqlite`

**Time** — `datetime`, `time`

**Metaprogramming** — `eval`, `ast`

**Observability** — `trace`, `witness`, `artifact`

**Interop** — `ffi` (dynamic libraries), `compress`, `wasm`

Full reference: `FARD-EBNF.md`

-----

## Application Packages

21 pure-FARD packages covering the full production spine and developer tooling. Import by relative path.

| Package | Contents |
|---|---|
| `packages/web-core/` | HTTP routing, middleware, request/response, app builder |
| `packages/auth-core/` | JWT (HS256), password hashing, sessions, RBAC |
| `packages/orm-core/` | Schema definitions, query builder, SQLite model layer |
| `packages/test-core/` | Assertions, suite runner, property testing |
| `packages/error-core/` | Rich errors, HTTP status mapping, result chaining |
| `packages/cli-core/` | Argument parsing, subcommands, help text |
| `packages/log-core/` | Structured JSON logging, levels, trace IDs |
| `packages/config-core/` | Env config loading, validation, typed getters, profiles |
| `packages/metrics-core/` | Counters, gauges, histograms, Prometheus export |
| `packages/dataframe-core/` | Columnar data, filter, group, aggregate, sort |
| `packages/template-core/` | HTML/text templating with escaping and layouts |
| `packages/queue-core/` | Durable jobs, retry, dead-letter queue |
| `packages/deploy-core/` | Dockerfile/compose generation, deployment manifests |
| `packages/cache-core/` | LRU cache, TTL, get_or_set, prefix invalidation |
| `packages/email-core/` | MIME construction, multipart, attachments, templates |
| `packages/migration-core/` | SQLite migrations, version table, checksum, rollback |
| `packages/secret-core/` | Env secret loading, redaction, sealing, safe_dump |
| `packages/bench-core/` | Deterministic benchmarks, timing envelopes, comparison |
| `packages/docsite-core/` | Package doc extraction, markdown/HTML generation |
| `packages/debug-core/` | Snapshots, step traces, diffs, breakpoints, watch |
| `packages/lsp-core/` | JSON-RPC, diagnostics, completions, hover, capabilities |

### Full-Stack Example

```fard
import("packages/web-core/app")       as app
import("packages/web-core/response")  as res
import("packages/auth-core/jwt")      as jwt
import("packages/config-core/config") as cfg

let config = cfg.from_env(cfg.new({ port: 8080 }), [
  { key: "port", env_var: "PORT", default: 8080 }
])

let server =
  app.get(app.new(), "/hello/:name", fn(req) {
    res.json_ok({ hello: req.params.name })
  })

app.run(server, cfg.get_int(config, "port", 8080))
```

### Application Path

```
CLI (cli-core) -> config (config-core) -> web routes (web-core)
  -> auth (auth-core) -> database (orm-core) -> migration (migration-core)
  -> templates (template-core) -> jobs (queue-core) -> email (email-core)
  -> cache (cache-core) -> metrics (metrics-core) -> secrets (secret-core) -> deploy (deploy-core)
```

-----

## Media Stack

A complete pure-FARD media pipeline. Every encoder, decoder, and transform is written in FARD with no native dependencies. Every export emits a cryptographic receipt.

### Image

```fard
import("packages/image-core/src/draw")      as draw
import("packages/image-core/src/export")    as image_export
import("packages/image-core/src/transform") as transform

let raster  = draw.gradient(256, 256)
image_export.write_png("out/gradient.png", "gradient", raster)
let gray = transform.grayscale(transform.resize(raster, 128, 128))
```

**Encode:** PPM, PNG (pure-FARD CRC-32, Adler-32, zlib, IHDR/IDAT/IEND)
**Decode:** PPM, PNG (filter types 0, 1, 2)
**Transform:** `resize`, `crop`, `flip_h`, `flip_v`, `composite`, `brightness`, `grayscale`

### Audio

```fard
import("packages/audio-core/src/synth")  as synth
import("packages/audio-core/src/export") as audio_export

let samples = synth.sine(440.0, 2.0, 44100)
audio_export.write_wav("out/tone.wav", "tone", 44100, 1, samples)
```

**Encode:** WAV (PCM S16LE)  **Decode:** WAV (PCM S16LE, 8-bit)
**Transform:** `gain`, `trim`, `mix`, `fade_in`, `fade_out`, `mono_to_stereo`, `stereo_to_mono`

### Video

Native format: `FARDVID1` — deterministic binary container (8-byte magic + u32le header + packed RGB frames). Transcode contracts describe downstream conversion to MP4 or WebM via ffmpeg.

**Modules:** `frame`, `rawvid`, `rawvid_decode`, `timeline`, `mux`, `avbundle`, `mp4_manifest`, `webm_manifest`, `transcode_contract`, `transcode_pipeline`, `gifbridge`

### PDF

Pure-FARD PDF-1.4 generation and annotation. `write_pdf` builds complete documents. `patch_pdf` injects overlay content streams into existing PDFs.

-----

## Integration Packages

| Package | What it does |
|---|---|
| `postgres-core` | PostgreSQL wire protocol v3 — connect, exec, query, row parsing |
| `ws-core` | WebSocket client — RFC 6455 framing, handshake, send/recv |
| `xlsx-core` | Excel workbook writer — OOXML with ZIP container |
| `avro-core` | Apache Avro OCF encoder/decoder with schema inference |
| `parquet-core` | Apache Parquet writer with Thrift compact metadata |
| `duckdb-core` | In-memory analytical query engine — filter, project, group, join |
| `wasm-core` | WASM binary decoder and stack machine interpreter |
| `watch-core` | Poll-based filesystem watcher |

-----

## Self-Hosting

FARD compiles itself to native x86_64 ELF. The entire compiler pipeline is written in FARD.

### Pipeline

```
source -> fardlex -> fardparse -> fard_lower -> fard_codegen -> fard_elf -> native ELF
fard_obj (relocatable objects) + fard_link (linker) -> single monolithic ELF
```

### Roadmap

| Stage | Description | Status |
|---|---|---|
| 0 | Lexer, parser, evaluator, toolchain | complete |
| 1 | IR + lowering + IR interpreter | complete |
| 2 | Bytecode + emitter + interpreter | complete |
| 3 | Seed VM in x86_64 assembly (1068 lines, no libc) | complete |
| 4 | Self-hosting compiler driver | complete |
| 5 | Native x86_64 ELF backend | complete |
| 6 | All pipeline components compile to native ELF | complete |
| 7 | Native linker — cross-module calls resolved | **complete** — math.add(10,32)=42 via linked ELF |

### Compiler Components

| File | Description |
|---|---|
| `apps/fardlex.fard` | Lexer (26 fns, 66KB native ELF) |
| `apps/fardparse.fard` | Recursive descent parser (43 fns, 148KB native ELF) |
| `apps/fard_lower.fard` | AST to IR lowering (67 fns, 88KB native ELF) |
| `apps/fard_codegen.fard` | IR to x86_64 code generator (85 fns, 282KB native ELF) |
| `apps/fard_elf.fard` | Linux ELF writer (40 fns, 52KB native ELF) |
| `apps/fard_obj.fard` | Relocatable object format |
| `apps/fard_link.fard` | Native linker — resolves cross-module calls |

### Verified Native Results

| Program | Result | Notes |
|---|---|---|
| `1 + 2` | `3` | Full pipeline: lex->parse->IR->codegen->ELF |
| `fib(10)` | `55` | Native ELF, no VM |
| `list.fold([1,2,3,4], 0, add)` | `10` | Native ELF, closures |
| `str.len(str.slice(s, 6, 11))` | `5` | Native stdlib |
| `list.len(list.build(5, fn(i){i*2}))` | `5` | Native list.build |
| `math.add(10, 32)` | `42` | **Two linked objects**, no Rust at runtime |

-----

## Cryptographic Witnessing

Every run produces a receipt. Receipts chain. Chains verify.

```fard
import("std/witness") as w
w.self_digest()   // -> "sha256:e60cb9e82ac28f..."

// Bind a prior verified run by digest
artifact step1 = "sha256:689dede5..."
step1.output
```

Oracle boundaries — `http`, `datetime.now`, `io.read_stdin`, `uuid.v4`, `ffi.call` — are explicitly recorded in the trace so runs remain auditable even when interacting with the outside world.

-----

## Toolchain

| Binary | Purpose |
|---|---|
| `fardrun` | Runtime: run, test, repl, new, install, search, publish |
| `fardfmt` | Canonical formatter |
| `fardcheck` | Type checker |
| `fardverify` | Trace, chain, and proof verification |
| `fardregistry` | Receipt registry server |
| `fardlock` | Lockfile generation and enforcement |
| `fardbundle` | Bundle build, verify, and run |
| `fard-build` | Verifiable build system |
| `fard-lsp` | Language Server Protocol |
| `fardc` | Compiler frontend |
| `farddoc` | Documentation generator |

```bash
fardrun run --program main.fard --out ./out   # produces result.json, trace.ndjson, digests.json
fardverify trace --out ./out
fardverify prove --out ./out --spec spec.json
```

VS Code: `code --install-extension editors/vscode/fard-language-0.1.0.vsix`

Syntax highlighting, dot-completion, hover docs, go-to-definition, find-all-references.

-----

## Package Registry

All 21 packages are imported by relative path — no registry required:

-----

## Testing

**231 test suites passing** across core language, stdlib, and application packages.

```bash
for f in tests/test_*.fard; do fardrun test --program "$f"; done
```

-----

## Specifications

| Document | Contents |
|---|---|
| `FARD-EBNF.md` | Full grammar and stdlib reference, verified against v1.7.0 |
| `spec/fard_spec_stack_v0_final.md` | Trust stack specification (frozen) |
| `SPEC.md` | Stdlib surface spec (generated) |

-----

## License

MUI
