# FARD Native Backend — x86_64 Linux ELF

## Overview

The native backend translates FARD IR directly to x86_64 machine code
and packages it as a Linux ELF executable. No VM. No interpreter.

## Files

- apps/fard_x86.fard  — x86_64 instruction emitter (bytes)
- apps/fard_elf.fard  — ELF binary writer
- apps/fard_codegen.fard — IR to x86_64 code generator

## Value Representation

Every FARD value is 16 bytes on the heap:
  [0..7]  tag   u64  (TAG_NULL=0, TAG_BOOL=1, TAG_INT=2, TAG_STR=4, TAG_CLOS=7)
  [8..15] payload i64 or ptr

This matches the seed VM layout exactly.

## Memory Model

- Heap: 64MB anonymous mmap at startup, bump allocator
- heap_base and heap_ptr in BSS
- alloc_val: allocates 16 bytes, returns ptr in rax

## Stack Frame Layout

Each FARD function uses a standard x86_64 stack frame:
  push rbp
  mov rbp, rsp
  sub rsp, N*16        ; N = number of IR locals

IR local slot i lives at [rbp - (i+1)*16].
The tag is at [rbp - (i+1)*16], payload at [rbp - (i+1)*16 + 8].

## Calling Convention

FARD closures are called as:
  rdi = closure ptr (tag=TAG_CLOS, payload=fn_ptr)
  rsi = arg0 val ptr
  rdx = arg1 val ptr
  rcx = arg2 val ptr
  r8  = arg3 val ptr
  r9  = arg4 val ptr
  [rsp+8..] = remaining args

Return value: rax = val ptr (heap allocated)

For direct (non-closure) calls to known functions:
  Same ABI but rdi is unused (no closure env yet)

## Register Usage During Codegen

  rax — scratch, return value
  rbx — scratch (callee-saved, saved/restored around calls)  
  rcx — scratch
  rdx — scratch
  rsi, rdi — args
  rbp — frame pointer (always)
  rsp — stack pointer (always)
  r10, r11 — scratch
  r12 — heap_ptr (callee-saved, kept across function)
  r13 — heap_base (callee-saved, kept across function)
  r14, r15 — scratch (callee-saved)

## Instruction Selection

IR instruction -> x86_64 sequence:

const dst val:
  mov rax, heap_ptr
  add heap_ptr, 16
  mov qword [rax], TAG_INT
  mov qword [rax+8], val
  mov [rbp - (dst+1)*16], rax
  mov qword [rbp - (dst+1)*16 + 8], val  ; inline for speed

load dst src:
  mov rax, [rbp - (src+1)*16]
  mov rcx, [rbp - (src+1)*16 + 8]
  mov [rbp - (dst+1)*16], rax
  mov [rbp - (dst+1)*16 + 8], rcx

add dst a b:
  mov rax, [rbp - (a+1)*16 + 8]   ; a.payload
  add rax, [rbp - (b+1)*16 + 8]   ; + b.payload
  ; alloc_int(rax) -> rax
  call alloc_int
  mov [rbp - (dst+1)*16], TAG_INT
  mov [rbp - (dst+1)*16 + 8], rax  ; ptr to result

Actually simpler: store value directly in local slot, not via heap:
  Store int values inline in local slots.
  Only heap-allocate when passing to/from functions.

## ELF Layout

  ELF header (64 bytes)
  Program header: LOAD (r-x) for text
  Program header: LOAD (rw-) for data+bss
  .text section: generated code + runtime helpers
  .data section: string constants
  .bss section: heap_ptr, heap_base, globals

## Runtime Helpers (linked into every binary)

  _start:        entry, calls main, calls sys_exit
  alloc_val:     bump allocator, returns ptr in rax
  alloc_int:     alloc_val + set tag=INT, payload=rax
  alloc_bool:    alloc_val + set tag=BOOL
  sys_write:     write(fd, buf, len)
  sys_exit:      exit(code)
  print_int:     print i64 to stdout
  print_val:     dispatch on tag, print value

## Phase 1 — Integer programs only

Support: const(int), load, store, global_load, global_store,
         add, sub, mul, div, mod, neg,
         eq, ne, lt, le, gt, ge,
         branch, jump, return,
         make_closure, call (direct fn calls only)

Phase 1 is enough to compile and run fib, add, fizzbuzz natively.

## Phase 2 — Add records, lists, strings

Support: make_rec, get_field, make_list, get_index,
         string constants, str operations

Phase 2 enables compiling fardc.fard itself natively.
