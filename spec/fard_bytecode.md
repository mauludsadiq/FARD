# FARD Bytecode Specification

Version: 0.1
Status: Draft

## Overview

FARD bytecode is a compact stack-based binary encoding of the FARD IR.
It is produced by fard_emit.fard from IR produced by fard_lower.fard.
It is executed by fard_bc_interp.fard (FARD) and bootstrap/vm.s (assembly).

Stack machine. All operands come from the operand stack.
All results push to the operand stack.

## File Format

    [0..3]   magic       0x46 0x41 0x52 0x44  ("FARD")
    [4..7]   version     0x00 0x00 0x00 0x01
    [8..11]  n_consts    u32 little-endian
    [12..]   const_pool  n_consts constant entries
             func_table  function entries
             entry       u32 index of entry function

## Constant Pool Entry

    [0]      tag         0x00=null 0x01=bool 0x02=int 0x03=float 0x04=str
    bool:    [1]         0x00=false 0x01=true
    int:     [1..8]      i64 little-endian
    float:   [1..8]      f64 little-endian
    str:     [1..4]      u32 length, then UTF-8 bytes

## Function Table Entry

    [0..3]   name_idx    u32 index into const_pool (str constant)
    [4..7]   arity       u32 number of parameters
    [8..11]  n_locals    u32 number of local slots (includes stack slots)
    [12..15] n_instrs    u32 number of instructions
    [16..]   instrs      instruction bytes

## Instruction Encoding

Each instruction is 1 byte opcode + optional operands.

### Opcode Table

    0x00  NOP                     no operation
    0x01  CONST     u32           push const_pool[idx]
    0x02  LOAD      u8            push locals[slot]
    0x03  STORE     u8            pop -> locals[slot]
    0x04  GLOBAL_LOAD  u32        push globals[name_pool[idx]]
    0x05  GLOBAL_STORE u32        pop -> globals[name_pool[idx]]
    0x06  POP                     discard top of stack
    0x07  DUP                     duplicate top of stack

    // Arithmetic (pop 2, push 1)
    0x10  ADD
    0x11  SUB
    0x12  MUL
    0x13  DIV
    0x14  MOD
    0x15  NEG                     pop 1, push negated

    // Comparison (pop 2, push bool)
    0x20  EQ
    0x21  NE
    0x22  LT
    0x23  LE
    0x24  GT
    0x25  GE

    // Logic (pop 2, push bool)
    0x30  AND
    0x31  OR
    0x32  NOT                     pop 1, push !val

    // Control flow
    0x40  JUMP      i16           relative jump (signed offset from next instr)
    0x41  JUMP_IF   i16           pop cond, jump if true
    0x42  JUMP_IFNOT i16          pop cond, jump if false
    0x43  RETURN                  pop return value, return from function

    // Aggregates
    0x50  MAKE_LIST  u8           pop n items (pushed left-to-right), push list
    0x51  MAKE_REC   u8           pop n (key,val) pairs, push record
    0x52  GET_FIELD  u32          pop obj, push obj[name_pool[idx]]
    0x53  GET_INDEX              pop idx, pop obj, push obj[idx]

    // Functions
    0x60  MAKE_CLOSURE u32        push closure over func_table[idx]
    0x61  CALL        u8          pop n args + fn, call fn(args), push result
    0x62  CALL_NATIVE u32         call native[name_pool[idx]] with n args on stack

    // Globals
    0x70  IMPORT      u32         import module name_pool[idx], push module record

## Stack Discipline

Before each instruction, stack state is well-defined.
Functions receive args on stack: arg0 pushed first, argN-1 pushed last.
CALL pops args right-to-left, then pops fn, then pushes result.
RETURN pops one value and returns it to caller.

## Name Pool

Separate from const_pool. Contains strings used as identifiers
(global names, field names, module paths). Indexed by u32.

    [0..3]   n_names     u32
    entries  same format as str constants (u32 len + UTF-8)

## Locals

Each function has a fixed-size locals array (n_locals slots).
LOAD/STORE access locals by slot index (u8 = max 256 locals).
Parameters occupy slots 0..arity-1.
Temporaries occupy slots arity..n_locals-1.

## Example

Source: fn add(a, b) { a + b }  call add(3, 4)

Function 0 "add" arity=2 locals=2:
    LOAD 0        // push a
    LOAD 1        // push b
    ADD           // push a+b
    RETURN        // return a+b

Entry function:
    MAKE_CLOSURE 0    // push closure for add
    GLOBAL_STORE 0    // globals["add"] = closure
    GLOBAL_LOAD  0    // push add
    CONST 1           // push 3
    CONST 2           // push 4
    CALL 2            // call add(3,4), push result
    RETURN            // return result

## Design Decisions

- Stack-based for seed VM simplicity (smaller dispatch loop)
- Relative jumps (i16) for position-independent code
- u8 slot index limits 256 locals per function (sufficient for FARD)
- u8 arg count limits 255 args per call (sufficient)
- Little-endian throughout
- No GC header — memory model deferred to Stage 3
