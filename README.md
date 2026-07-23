# FARD

FARD is a deterministic, content-addressed scripting language. Every execution produces a SHA-256 receipt committing to source code, imports, inputs, intermediate steps, and final result. Two runs of the same program on the same inputs produce the same digest — on any machine, at any time.

Traceability is not a feature. It is an invariant of execution.

**Version:** v1.7.0

---

## Install

    curl -sf https://raw.githubusercontent.com/mauludsadiq/FARD/main/install.sh | sh

Installs fardrun to /usr/local/bin. Detects macOS arm64/x86_64 and Linux x86_64 automatically. Or build from source:

    git clone https://github.com/mauludsadiq/FARD.git && cd FARD
    cargo build --release --bin fardrun

---

## Testing

309 test suites passing across 316 total.

    for f in tests/test_*.fard; do fardrun test --program "$f"; done

---

## Language

### Values

    42               // Int (64-bit signed)
    3.14             // Float (64-bit IEEE 754)
    true             // Bool
    null             // Unit
    "hello"          // Text
    "hello ${name}"  // Interpolated string
    [1, 2, 3]        // List
    { x: 1, y: 2 }  // Record

### Functions

    fn add(a, b) { a + b }
    let double = fn(x) { x * 2 }
    fn make_adder(n) { fn(x) { x + n } }

    list.map(xs, fn(x) { x * 2 })
    list.fold(xs, 0, fn(acc, x) { acc + x })

### Bindings and Control Flow

    let x = 42 in x + 1

    if x > 0 then "positive" else "non-positive"

    match type.of(x) {
      "int"  => str.from(x),
      "text" => x,
      _      => "other"
    }

### While — Hash-Chained Iteration

while produces a cryptographic certificate of the entire computation.

    let result = while { n: 0, acc: 0 }
      fn(s) { s.n < 10 }
      fn(s) { { n: s.n + 1, acc: s.acc + s.n } }

    result.value      // { n: 10, acc: 45 }
    result.chain_hex  // sha256 of full computation history
    result.steps      // 10

### Imports

    import("std/math")               as math
    import("./mylib")                as mylib
    import("packages/web-core/app")  as app

---

## Standard Library

65 modules. All verified against fardrun v1.7.0.

**Data** — str, list, rec, map, set, option, result

**Numbers** — math, float, int, bigint, bits, linalg

**Encoding** — json, csv, bytes, base64, codec, re

**Crypto** — hash (SHA-256), crypto (HMAC, AES, Ed25519), uuid

**I/O** — io, fs, path, env, process

**Network** — net (TCP), http, ws (WebSocket)

**Concurrency** — promise, chan, mutex, cell

**Storage** — sqlite

**Time** — datetime, time

**Metaprogramming** — eval, ast

**Observability** — trace, witness, artifact

**Interop** — ffi (dynamic libraries), compress, wasm

Full reference: FARD-EBNF.md

---

## Packages

199 packages covering the full production spine, scientific computing, media, and developer tooling. 70+ packages have verified test suites.

### Web

    packages/fard-web/      HTTP routing, middleware, request/response, OpenAPI spec generation
    packages/web-core/      Core HTTP primitives
    packages/web-router/    Path parameter matching, grouped routers
    packages/http-client/   Sync and async HTTP client
    packages/http-server/   Sync and async HTTP server
    packages/websocket/     WebSocket client and server

### Auth and Security

    packages/auth-core/     JWT (HS256), password hashing, sessions, RBAC
    packages/oauth2/        OAuth2 client flows — auth code, client credentials, token refresh
    packages/hmac-sign/     HMAC request signing, webhook verification, timestamped signatures
    packages/crypto-extra/  Random tokens, password hashing, Merkle trees, AES encrypt/decrypt
    packages/jwt/           JWT encode, verify, expiry checking

### Data

    packages/frame/         Typed columnar DataFrame — filter, group, join, sort, aggregate
    packages/csv-extra/     CSV parse with headers, filter, select, column, group
    packages/csv-stream/    Lazy CSV streaming — map, filter, fold, column
    packages/diff/          Line diff, format, patch, change count
    packages/xml/           XML element construction, rendering, attribute handling
    packages/json-schema/   Schema definition and validation
    packages/toml/          TOML parse and encode
    packages/yaml/          YAML parse and encode

### Scientific

    packages/tensor/        N-dimensional arrays — zeros, ones, eye, reshape, index
    packages/tensor-core/   Matrix ops, autograd, neural network layers, SGD
    packages/stats/         Sum, mean, min, max, variance, std dev, median
    packages/plot/          SVG chart generation — line, bar, scatter, histogram

### Storage

    packages/sqlite/        SQLite key-value store
    packages/kv/            JSON-backed persistent key-value store
    packages/cache/         LRU cache with TTL eviction
    packages/queue/         Persistent FIFO queue with dequeue and peek

### Messaging

    packages/pubsub/        In-process pub/sub with history and subscriber tracking
    packages/smtp/          Email composition, RFC 2822 formatting, curl/sendmail send

### Utilities

    packages/semver/        Parse, compare, bump, sort, latest semantic versions
    packages/glob/          Glob pattern matching, directory filtering
    packages/regex-extra/   Pure FARD regex — glob, email, IP, URL, JSON validators
    packages/markdown/      Markdown to HTML — headings, bold, inline code, paragraphs
    packages/template/      Mustache-style template rendering
    packages/base64/        Base64 encode/decode including URL-safe variant
    packages/uuid/          UUID v4 generation, nil, is_nil, short_id, stamped
    packages/diff/          Line diff, format, patch, change count
    packages/rate-limiter/  Token bucket rate limiting, per-key limiting
    packages/stream/        Lazy sequences — map, filter, fold, take, drop, range
    packages/shell/         Shell command execution, capture, file existence
    packages/parse/         Parser combinator library — digit, alpha, literal, many, choice

### Developer Tooling

    packages/fard-web/      Full web framework
    packages/fard-bench/    Deterministic benchmarks
    packages/fard-check/    Type checker
    packages/fard-ci/       CI pipeline definitions
    packages/fard-lint/     Linter
    packages/fard-test/     Test runner and assertions
    packages/fard-deploy/   Deployment manifests
    packages/fard-watch/    File watcher
    packages/fard-notebook/ Interactive notebook

### Full-Stack Example

    import("packages/fard-web/main") as app
    import("packages/auth-core/jwt") as jwt

    let server = app.get(app.new(), "/hello/:name", fn(req) {
      app.json_ok({ hello: req.params.name })
    })

    app.run(server, 8080)

---

## Self-Hosting

FARD now evaluates itself. The menv bootstrap loads eval.fard through the native runtime, producing a self-hosted evaluator that correctly interprets all core language constructs.

Verified:

    lit_int, lit_bool, call_builtin int.add    pass
    var lookup, if_node, match_expr wildcard   pass
    let binding, call with user fn             pass
    fib(10) = 55 via self-hosted eval          pass
    Level-2: eval evaluates eval evaluates     pass

### menv Bootstrap

The bootstrap seeds a shared mutable environment with stdlib modules, then loads all definitions from eval.fard into it. Because all closures share the same Arc-backed MutEnv, forward references resolve correctly at call time.

### eval_fir

Direct FIR Val interpreter added to the Rust runtime. Evaluates FARD FIR records without converting to Expr first, eliminating the memory explosion that occurred with deeply nested FIR trees. Closures store __closure_env__ as an Arc reference — no copying.

### value.eq / value.ne

The parser now emits value.eq and value.ne for == and != operators. These use val_eq which handles Int, Float, Bool, Text, List, and Record equality. Previously, == was lowered to int.eq causing failures on non-integer comparisons.

### Compiler Pipeline

    source -> fardlex -> fardparse -> fard_lower -> fard_codegen -> fard_elf -> native ELF

Stage 8 is complete. The entire pipeline — lexer, parser, lowerer, code generator, ELF writer, linker — is implemented in pure FARD and compiles to native x86-64 ELF. Verified:

    add(10, 32) = 42        full pipeline: lex->parse->IR->codegen->ELF
    fib(10) = 55            native ELF, no VM
    adder(10)(32) = 42      closures with captures
    fib(35) = 9227465       seed VM, 29M calls, ~3x speedup over tree walker

### Seed VM

A zero-heap frame allocator in x86-64 assembly. All frame scratch memory — locals, vstack, args — allocated from a BSS stack. Heap used only for closures and the const pool. Verified fib(35) = 9227465.

---

## Cryptographic Witnessing

Every run produces a receipt. Receipts chain. Chains verify.

    import("std/witness") as w
    w.self_digest()   // -> "sha256:e60cb9e82ac28f..."

    // Bind a prior verified run by digest
    artifact step1 = "sha256:689dede5..."
    step1.output

Oracle boundaries — http, datetime.now, io.read_stdin, uuid.v4, ffi.call — are explicitly recorded in the trace so runs remain auditable even when interacting with the outside world.

---

## Toolchain

    fardrun       Runtime: run, test, repl, new, install, search, publish
    fardfmt       Canonical formatter
    fardcheck     Type checker
    fardverify    Trace, chain, and proof verification
    fardregistry  Receipt registry server
    fardlock      Lockfile generation and enforcement
    fardbundle    Bundle build, verify, and run
    fard-build    Verifiable build system
    fard-lsp      Language Server Protocol
    fardc         Compiler frontend
    farddoc       Documentation generator

    fardrun run --program main.fard --out ./out
    fardverify trace --out ./out
    fardrun run --program main.fard --out ./out --fard-eval

VS Code: code --install-extension editors/vscode/fard-language-0.1.0.vsix

---

## Specifications

    FARD-EBNF.md                         Full grammar and stdlib reference
    spec/fard_spec_stack_v0_final.md     Trust stack specification (frozen)
    SPEC.md                              Stdlib surface spec (generated)

---

## License

MUI
