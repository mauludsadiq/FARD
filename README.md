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

## Self-hosting

Goal: eliminate Rust entirely. FARD compiles to native machine code with no
foreign runtime. See ROADMAP.md for the full plan.

Current status: Stage 5 in progress — native x86_64 backend working.

Rust host: 13,632 lines
FARD implementation: 3,500+ lines
Native backend: working for integer programs

### Pipeline

    source -> fardlex2 -> fardparse -> fard_lower -> fard_codegen -> fard_elf -> native ELF

All stages written in FARD. Native ELF runs without VM or interpreter.

### Language pipeline

| File | Lines | Description |
|---|---|---|
| apps/fardlex2.fard | 141 | Lexer |
| apps/fardparse.fard | 501 | Recursive descent parser |
| apps/fardeval.fard | 466 | Tree-walking evaluator |
| apps/fard_lower.fard | 417 | AST to IR lowering |
| apps/fard_ir_interp.fard | 360 | Iterative IR interpreter |
| apps/fard_emit.fard | 446 | IR to bytecode emitter |
| apps/fard_bc_interp.fard | 399 | Bytecode interpreter |
| apps/fard_x86.fard | 415 | x86_64 instruction emitter |
| apps/fard_codegen.fard | 550 | IR to native code generator |
| apps/fard_elf.fard | 380 | Linux x86_64 ELF writer |
| apps/fardc.fard | 75 | Compiler driver |

### Verified results

| Test | Result | Backend |
|---|---|---|
| add(3,4) | 7 | IR interpreter |
| fib(10) | 55 | IR interpreter |
| add(3,4) | 7 | Seed VM (bytecode) |
| fib(10) | 55 | Seed VM (bytecode, under 1s) |
| add(3,4) | 7 | Native ELF, no VM |
| fib(10) | 55 | Native ELF, no VM, 0.5s incl Docker |

### Toolchain

| File | Lines |
|---|---|
| apps/farddoc.fard | 227 |
| apps/fardfmt.fard | 171 |
| apps/fardregistry.fard | 138 |
| apps/fard-build.fard | 183 |
| apps/fardbundle.fard | 159 |
| apps/fardlock.fard | 160 |
| apps/fardcheck.fard | 240 |

### Bootstrap seed VM

bootstrap/vm.asm is 1068 lines of x86_64 assembly. Executes FARD bytecode
on Linux x86_64 with no libc dependency. Temporary bootstrap artifact
displaced by the native backend.

### Roadmap

| Stage | Description | Status |
|---|---|---|
| 0 | Lexer, parser, evaluator, toolchain | done |
| 1 | IR + lowering + IR interpreter | done |
| 2 | Bytecode + emitter + interpreter | done |
| 3 | Seed VM in x86_64 assembly | done |
| 4 | Self-hosting compiler driver | done |
| 5 | Native x86_64 ELF backend | in progress |
| 6 | FARD-native production compiler | planned |

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
"hello ${name}" // Interpolated string
[1, 2, 3]       // List
{ x: 1, y: 2 }  // Record
```

### Functions

```
fn add(a, b) { a + b }
let double = fn(x) { x * 2 }
fn make_adder(n) { fn(x) { x + n } }

// Named arguments
greet(name: "Alice", greeting: "Hello")

// Default arguments
fn greet(name, greeting = "Hello") { str.concat(greeting, str.concat(", ", name)) }
greet("Alice", null)  // -> "Hello, Alice"
```

### Bindings and Control Flow

```
let x = 42
let result = let x = 10 in let y = 20 in x + y

if x > 0 then "positive" else "non-positive"

match type.of(x) {
  "int"  => str.from_int(x),
  "text" => x,
  _      => "other"
}
```

### Collections

```
// List comprehensions
let squares      = [x * x for x in [1, 2, 3, 4, 5]]
let even_squares = [x * x for x in [1, 2, 3, 4, 5] if x % 2 == 0]

// Pipe operator
list.range(1, 11) |> list.filter(x => x % 2 == 0) |> list.map(x => x * x)

// Record spread and computed keys
let updated = { ...defaults, color: "red", [dynamic_key]: value }
```

### While (Hash-Chained Iteration)

`while` produces a cryptographic certificate of the entire computation — every state transition is hashed into the chain.

```
let result = while {n: 0, acc: 0}
  fn(s) { s.n < 10 }
  fn(s) { {n: s.n + 1, acc: s.acc + s.n} }

result.value      // {n: 10, acc: 45}
result.chain_hex  // sha256 of full computation history
```

### Ergonomics

```
// Safe navigation and null-coalescing
let host = config?.db?.host ?? "localhost"

// Error propagation
fn pipeline(x) { let a = div(x, 2)? let b = div(a, 2)? { t: "ok", v: b } }

// Lambdas
list.map(xs, x => x * 2)
```

### Imports

```
import("std/math")  as math
import("./mylib")   as mylib
import("pkg:greet") as greet
```

-----

## Standard Library

65 modules. Selected highlights:

**Data** — `std/str`, `std/list`, `std/rec`, `std/map`, `std/set`, `std/option`, `std/result`

**Numbers** — `std/math`, `std/float`, `std/int`, `std/bigint`, `std/bits`, `std/linalg`

**Encoding** — `std/json`, `std/csv`, `std/bytes`, `std/base64`, `std/codec`, `std/re`

**Crypto** — `std/hash` (SHA-256), `std/crypto` (HMAC, AES, Ed25519), `std/uuid`

**I/O** — `std/io`, `std/fs`, `std/path`, `std/env`, `std/process`

**Network** — `std/net` (TCP server + client), `std/http`, `std/ws` (WebSocket)

**Concurrency** — `std/promise`, `std/chan`, `std/mutex`, `std/cell`

**Storage** — `std/sqlite`

**Time** — `std/datetime`, `std/time`

**Metaprogramming** — `std/eval`, `std/ast`

**Observability** — `std/trace`, `std/witness`, `std/artifact`

**Interop** — `std/ffi` (dynamic library loading), `std/compress`, `std/wasm`

Full API reference: `spec/fardlang_grammar_v0.5.txt`

-----

## Media Stack

A complete pure-FARD media pipeline. Every encoder, decoder, and transform is written in FARD with no native dependencies. Every export emits a cryptographic receipt alongside the output file.

### Image

```
import("packages/image-core/src/draw")       as draw
import("packages/image-core/src/export")     as image_export
import("packages/image-core/src/png_decode") as png_decode
import("packages/image-core/src/transform")  as transform

let raster  = draw.gradient(256, 256)
image_export.write_png("out/gradient.png", "gradient", raster)

let decoded = png_decode.decode(io.read_file("in/photo.png").ok)
let resized = transform.resize(decoded.ok, 128, 128)
let gray    = transform.grayscale(resized)
```

**Encode:** PPM (P6), PNG (8-bit RGB — pure-FARD CRC-32, Adler-32, zlib stored blocks, IHDR/IDAT/IEND)

**Decode:** PPM, PNG (filter types 0, 1, 2)

**Transform:** `resize`, `crop`, `flip_h`, `flip_v`, `composite`, `brightness`, `grayscale`

### Audio

```
import("packages/audio-core/src/synth")      as synth
import("packages/audio-core/src/export")     as audio_export
import("packages/audio-core/src/wav_decode") as wav_decode
import("packages/audio-core/src/transform")  as transform

let samples = synth.sine(440.0, 2.0, 44100)
audio_export.write_wav("out/tone.wav", "tone_440", 44100, 1, samples)

let decoded = wav_decode.decode(io.read_file("in/sound.wav").ok)
let trimmed = transform.trim(decoded.ok.samples, 44100, 500, 1500)
let mixed   = transform.mix(trimmed, other_samples)
```

**Encode:** WAV (PCM S16LE, RIFF/fmt/data)

**Decode:** WAV (PCM S16LE and 8-bit)

**Transform:** `gain`, `trim`, `mix`, `fade_in`, `fade_out`, `mono_to_stereo`, `stereo_to_mono`

### Video

```
import("packages/video-core/src/timeline") as timeline
import("packages/video-core/src/export")   as video_export

let video = timeline.test_pattern_video(160, 120, 24, 1, 10)
video_export.write_rawvid("out/test.fvid", "test_pattern", video)
```

Native format is `FARDVID1` — a deterministic binary container (8-byte magic + u32le header + packed RGB frames). Transcode contracts describe downstream conversion to MP4 (H.264/AAC) or WebM (VP9/Opus) via ffmpeg.

**Modules:** `frame`, `rawvid` (encode), `rawvid_decode`, `timeline`, `mux`, `avbundle`, `mp4_manifest`, `webm_manifest`, `transcode_contract`, `transcode_pipeline`, `gifbridge`

### PDF

```
import("packages/pdf_v0/src/write_pdf")    as write_pdf
import("packages/pdf_patch/src/patch_pdf") as patch_pdf

let pdf     = write_pdf.build_pdf([model.make_bbox(10, 10, 100, 50)])
let patched = patch_pdf.patch_pdf(existing_pdf_text, 10, 10, 100, 50)
```

Pure-FARD PDF-1.4 generation and annotation. `write_pdf` builds complete documents with filled rectangles. `patch_pdf` injects overlay content streams into existing PDFs. `highlight` finds text spans and produces overlay plans.

-----

## Integration Packages

Pure-FARD implementations of every integration a programmer expects. No native dependencies.

|Package        |What it does                                                               |
|---------------|---------------------------------------------------------------------------|
|`postgres-core`|PostgreSQL wire protocol v3 — connect, exec, query, row parsing            |
|`ws-core`      |WebSocket client — RFC 6455 framing, handshake, send/recv                  |
|`xlsx-core`    |Excel workbook writer — OOXML with ZIP container                           |
|`avro-core`    |Apache Avro OCF encoder/decoder with schema inference                      |
|`parquet-core` |Apache Parquet writer with Thrift compact metadata                         |
|`duckdb-core`  |In-memory analytical query engine — filter, project, group, aggregate, join|
|`wasm-core`    |WASM binary decoder and stack machine interpreter                          |
|`watch-core`   |Poll-based filesystem watcher                                              |

-----

## Package Registry

164 packages. Semver ranges supported. SHA-256 verified and locally cached.

```toml
[deps]
greet  = "greet@1.6.0"
jwt    = "jwt@^1.6.0"
stream = "stream@~1.6.0"
```

```bash
fardrun install --manifest fard.toml
fardrun search jwt
```

Categories: data science, HTTP, auth, storage, build/CI, infrastructure, deployment, utilities.

Registry: `https://github.com/mauludsadiq/FARD/releases/latest/download/registry.json`

-----

## Cryptographic Witnessing

Every run produces a receipt. Receipts chain. Chains verify.

```
import("std/witness") as w
w.self_digest()   // -> "sha256:e60cb9e82ac28f..."

// Bind a prior run by digest
artifact step1 = "sha256:689dede5..."
step1.output

// Proof-carrying code
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

Oracle boundaries — `std/http`, `std/datetime.now`, `std/io.read_stdin`, `std/uuid.v4`, `std/ffi.call` — are explicitly marked and recorded in the trace so runs remain auditable even when interacting with the outside world.

### Distributed Receipt Convergence

The Inherit-Cert CRDT is a Min-Register Map. After one round of merge, all replicas converge on the canonical RunID for each effect.

```bash
curl -X POST http://registry/crdt/propose \
  -d '{"effect_kind":"http_get","req_hex":"...","run_id":"sha256:aaa..."}'
curl http://registry/crdt/state
```

-----

## Self-Hosting

FARD owns its intermediate representation and first compiler stage, implemented entirely in FARD.

The pipeline — source text → parse → lower → typecheck → HM type inference → eval → result — runs fully in FARD. Rust is no longer required for execution or type checking of core functional programs.

Verified bootstrap: FARD’s evaluator executes FARD’s evaluator executing FARD’s evaluator. `fib(10) = 55` via self-hosted recursive evaluation. `int.add(10, 32) = 42` via 2-layer bootstrap.

Key packages: `fard_parse`, `fir` (IR), `fard_lower`, `fard_eval`, `fard_type`, `fard_hm` (Algorithm W).

-----

## Verifiable Build System

```toml
[[step]]
name    = "compile"
program = "steps/compile.fard"
out     = "build/compile"

[[step]]
name       = "test"
program    = "steps/test.fard"
depends_on = ["compile"]
```

```bash
fard-build --config fard.build.toml --out build/
fard-build --verify --out build/
```

Each step produces a receipt. `build.receipt.json` chains all step digests. Any change to any step breaks the chain.

-----

## Toolchain

|Binary        |Purpose                                                |
|--------------|-------------------------------------------------------|
|`fardrun`     |Runtime: run, test, repl, new, install, search, publish|
|`fardfmt`     |Canonical formatter                                    |
|`fardcheck`   |HM-style type checker                                  |
|`fardwasm`    |FARD → WAT/WASM compiler                               |
|`fardregistry`|Receipt registry server — self-hosted in FARD          |
|`fardlock`    |Lockfile generation and enforcement — self-hosted       |
|`fardbundle`  |Bundle build, verify, and run — self-hosted in FARD    |
|`fardcheck`   |Type and style checker — self-hosted in FARD           |
|`fardverify`  |Trace, chain, proof, and bundle verification           |
|`fardpkg`     |Package management                                     |
|`fard-lsp`    |Language Server Protocol                               |
|`fardc`       |Compiler frontend and canonicalizer                    |
|`farddoc`     |Documentation generator — self-hosted in FARD           |
|`fard-build`  |Verifiable build system — self-hosted in FARD          |

```bash
fardrun run --program main.fard --out ./out   # produces result.json, trace.ndjson, digests.json
fardverify trace --out ./out
fardverify prove --out ./out --spec spec.json
fardregistry --port 7370 --db receipts.db
```

VS Code: `code --install-extension editors/vscode/fard-language-0.1.0.vsix`

Syntax highlighting, dot-completion, hover docs, go-to-definition (F12), find-all-references (Shift+F12).

-----

## Architecture

```
Layer 5  Execution ABI v0        bundle -> witness bytes
Layer 4  Registry Semantics v0   content-addressed receipt storage
Layer 3  Composition Semantics   executions link by verified RunID
Layer 2  Artifact Semantics      same (program, input, deps) -> same RunID
Layer 1  Value Core v0           same value -> same bytes -> same hash
```

```
CID(bytes) = "sha256:" || hex(SHA256(bytes))
```

-----

## Testing

All tests written in pure FARD:

```bash
for f in tests/test_*.fard;               do fardrun test --program "$f"; done
for f in tests/image_core/test_*.fard;    do fardrun test --program "$f"; done
for f in tests/audio_core/test_*.fard;    do fardrun test --program "$f"; done
for f in tests/video_core/test_*.fard;    do fardrun test --program "$f"; done
for f in tests/pdf_v0/test_*.fard;        do fardrun test --program "$f"; done
for f in tests/postgres_core/test_*.fard; do fardrun test --program "$f"; done
for f in tests/ws_core/test_*.fard;       do fardrun test --program "$f"; done
for f in tests/xlsx_core/test_*.fard;     do fardrun test --program "$f"; done
for f in tests/avro_core/test_*.fard;     do fardrun test --program "$f"; done
for f in tests/parquet_core/test_*.fard;  do fardrun test --program "$f"; done
for f in tests/duckdb_core/test_*.fard;   do fardrun test --program "$f"; done
for f in tests/wasm_core/test_*.fard;     do fardrun test --program "$f"; done
for f in tests/watch_core/test_*.fard;    do fardrun test --program "$f"; done
```

-----

## Specifications

|Document                          |Contents                                          |
|----------------------------------|--------------------------------------------------|
|`spec/fard_spec_stack_v0_final.md`|Trust stack specification (frozen)                |
|`spec/fardlang_grammar_v0.5.txt`  |Surface language grammar and full stdlib reference|
|`SPEC.md`                         |Stdlib surface spec (generated)                   |

-----

## License

MUI