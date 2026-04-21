# FARD Self-Hosting Roadmap

Goal: eliminate Rust entirely. fardrun becomes a native binary produced
by FARD itself. No interpreter. No host language. No foreign runtime.

## Stage 0 — Bootstrap Foundation (COMPLETE)

The Rust host still executes everything, but the language components
are written in FARD.

| Component | File | Lines | Status |
|---|---|---|---|
| Lexer | apps/fardlex2.fard | 141 | done |
| Parser | apps/fardparse.fard | 500 | done |
| Evaluator (core) | apps/fardeval.fard | 466 | partial |
| Toolchain (7 apps) | apps/fard*.fard | 1308 | done |

The Rust host (src/bin/fardrun.rs, 13614 lines) breaks down as:

| Section | Lines | Disposition |
|---|---|---|
| CLI, module loader, tracer | ~2233 | replace with FARD |
| Lexer and Parser | ~2270 | already replaced |
| Evaluator core | ~1678 | replace with FARD |
| Stdlib builtins | ~7430 | thin syscall layer only |

## Stage 1 — Canonical FARD IR

Define a backend-neutral intermediate representation in FARD.
Written as FARD data structures. Serializable. Deterministic.

Instruction families: const, load, store, global, rec, list, get,
set, arith, cmp, logic, branch, jump, call, return, closure,
import, effect (syscalls).

Deliverables:
- spec/fard_ir.md
- apps/fard_lower.fard (AST to IR)
- apps/fard_ir_interp.fard (IR interpreter, replaces fardeval)

## Stage 2 — Deterministic Bytecode

Compact binary encoding of the IR. Register-based machine model.
Fixed opcode table. Canonical constant pool. Symbol and module tables.

Format: magic, version, const_pool, symbol_table, import_table,
function_table, entry_point.

Deliverables:
- spec/fard_bytecode.md
- apps/fard_emit.fard (IR to bytecode)
- apps/fard_disasm.fard (disassembler)
- apps/fard_bc_interp.fard (bytecode interpreter in FARD)

## Stage 3 — Tiny Bootstrap VM

Smallest possible executor for FARD bytecode. Written in assembly
(x86_64) as a temporary bootstrap artifact. Not the destination.

Responsibilities: load bytecode, bump allocator, opcode dispatch,
minimal syscalls (read, write, open, close, exit, mmap). Nothing else.

Target: Linux x86_64 first. macOS arm64 second.
Size target: under 2000 lines of assembly.

Deliverables:
- bootstrap/vm.s (seed VM)
- bootstrap/Makefile
- Test: fib(10) bytecode runs correctly on seed VM

## Stage 4 — Self-Hosting Compiler in FARD

FARD front-end plus IR plus bytecode emitter all in FARD, running
on the Stage 3 VM. The compiler compiles itself.

Pipeline: source -> fardlex2 -> fardparse -> fard_lower -> fard_emit -> bytecode

Deliverables:
- apps/fardc.fard (compiler driver)
- Self-hosting test: fardc.fard compiles itself, output matches

## Stage 5 — Native Code Backend

FARD IR to machine code. First target: Linux x86_64 ELF.

Subsystems: instruction selection, linear scan register allocation,
System V AMD64 ABI calling convention, ELF emitter (sections,
relocations, symbol table, entry point).

Deliverables:
- apps/fard_x86.fard (x86_64 instruction emitter)
- apps/fard_elf.fard (ELF binary writer)
- apps/fard_regalloc.fard (linear scan register allocator)
- Test: hello.fard compiles to native ELF, runs without VM

Second target: macOS arm64 Mach-O after Linux x86_64 is stable.

## Stage 6 — FARD-Native Production Compiler

Final state. No Rust anywhere in the execution chain.

    fardrun build app.fard -o app
    fardrun run app.fard

The fardrun binary is itself a FARD-compiled native executable.
The compiler is written in FARD and compiled by itself.
The stdlib is implemented in FARD with a minimal syscall layer.

Trust boundary (the irreducible seed):
- Program entry (_start)
- Bump allocator (mmap syscall)
- Syscall bridge (read, write, open, close, exit, mmap, spawn)
- ELF/Mach-O loader support

Everything else is FARD.

## What Gets Eliminated

| Currently in Rust | Replaced by |
|---|---|
| Lexer | fardlex2.fard (done) |
| Parser | fardparse.fard (done) |
| Evaluator | fard_bc_interp.fard (Stage 2) |
| CLI and module loader | fardc.fard (Stage 4) |
| Stdlib I/O builtins | Syscall layer (Stage 3) |
| fardrun binary | FARD-compiled native binary (Stage 6) |

The Rust host is not patched or shrunk. It is displaced entirely.

## Current Priority

Stage 1: Define the FARD IR.

The IR specification drives everything downstream. Getting it right
is the most important decision in the roadmap.

Next actions:
1. Survey AST node types from fardparse output
2. Define IR instruction set (target 20-30 opcodes)
3. Write spec/fard_ir.md
4. Implement apps/fard_lower.fard
5. Implement apps/fard_ir_interp.fard
