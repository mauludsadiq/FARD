# FARD IR Specification

Version: 0.1
Status: Draft

## Overview

The FARD IR is a backend-neutral intermediate representation.
It is produced by lowering the AST from fardparse.
It is consumed by the bytecode emitter and the IR interpreter.

All IR values are records. All IR programs are lists of functions.

## Value Types

    int       64-bit signed integer
    float     64-bit IEEE 754 double
    bool      true | false
    null      unit value
    str       UTF-8 string (immutable)
    list      ordered sequence of values
    rec       string-keyed record of values
    closure   function + captured environment
    ref       mutable cell (for future use)

## IR Program Structure

An IR program is a record:

    {
      fns:     [fn_def, ...],    // all functions, index = fn_id
      globals: [global, ...],    // top-level bindings
      entry:   int               // index of entry function in fns
    }

## Function Definition

    {
      name:    str,              // debug name
      arity:   int,              // number of parameters
      locals:  int,              // number of local slots (includes params)
      consts:  [value, ...],     // constant pool for this function
      code:    [instr, ...]      // instruction sequence
    }

Locals 0..arity-1 are parameters.
Locals arity..locals-1 are temporaries.

## Instruction Format

Each instruction is a record { op: str, ... } with op-specific fields.

## Instructions

### Constants

    { op: "const", dst: int, val: any }
    Load a literal value into local dst.

    { op: "const_pool", dst: int, idx: int }
    Load consts[idx] into local dst.

### Local Access

    { op: "load", dst: int, src: int }
    Copy local src to local dst.

    { op: "store", dst: int, src: int }
    Copy local src to local dst (alias for load, explicit write intent).

### Global Access

    { op: "global_load", dst: int, name: str }
    Load global binding name into local dst.

    { op: "global_store", name: str, src: int }
    Store local src into global binding name.

### Arithmetic

    { op: "add", dst: int, a: int, b: int }
    { op: "sub", dst: int, a: int, b: int }
    { op: "mul", dst: int, a: int, b: int }
    { op: "div", dst: int, a: int, b: int }
    { op: "mod", dst: int, a: int, b: int }
    { op: "neg", dst: int, src: int }

### Comparison

    { op: "eq",  dst: int, a: int, b: int }
    { op: "ne",  dst: int, a: int, b: int }
    { op: "lt",  dst: int, a: int, b: int }
    { op: "le",  dst: int, a: int, b: int }
    { op: "gt",  dst: int, a: int, b: int }
    { op: "ge",  dst: int, a: int, b: int }

### Logic

    { op: "and", dst: int, a: int, b: int }
    { op: "or",  dst: int, a: int, b: int }
    { op: "not", dst: int, src: int }

### Control Flow

    { op: "jump", target: int }
    Unconditional jump to instruction index target.

    { op: "branch", cond: int, then_t: int, else_t: int }
    If local cond is true jump to then_t else jump to else_t.

    { op: "return", src: int }
    Return local src from current function.

### Aggregate Construction

    { op: "make_list", dst: int, items: [int, ...] }
    Construct a list from locals items into dst.

    { op: "make_rec", dst: int, fields: [{key: str, src: int}, ...] }
    Construct a record from locals into dst.

### Aggregate Access

    { op: "get_field", dst: int, obj: int, field: str }
    Load field from record in local obj into dst.

    { op: "get_index", dst: int, obj: int, idx: int }
    Load list[idx_local] from local obj into dst.

### Function and Closure

    { op: "make_closure", dst: int, fn_id: int, captures: [int, ...] }
    Create a closure over function fn_id capturing locals into dst.

    { op: "call", dst: int, fn_r: int, args: [int, ...] }
    Call closure in local fn_r with args locals, result into dst.

    { op: "call_global", dst: int, name: str, args: [int, ...] }
    Call a named global function directly.

### Module and Import

    { op: "import", dst: int, path: str, alias: str }
    Load module at path into local dst under alias.

    { op: "get_method", dst: int, obj: int, method: str }
    Dispatch method on obj (type-based), result closure into dst.

### Effects (I/O and Syscalls)

    { op: "effect", dst: int, name: str, args: [int, ...] }
    Invoke a named effect (syscall or stdlib primitive).

    Effect names (initial set):
      fs.read        read file to string
      fs.write       write string to file
      fs.exists      check file existence
      io.print       write to stdout
      io.read_line   read line from stdin
      proc.exit      terminate process
      proc.spawn     spawn subprocess

## Lowering Map (AST -> IR)

| AST node    | IR instructions |
|-------------|-----------------|
| int/float/bool/null/str | const |
| var         | global_load or load |
| let         | eval val, store to local, eval body |
| if          | eval cond, branch, then block, else block |
| bin (+,-,*,/,%) | arith |
| bin (==,!=,<,>,<=,>=) | cmp |
| bin (&&,||) | and/or |
| bin (|>)    | rewrite as call |
| unary (!)   | not |
| unary (-)   | neg |
| list        | make_list |
| rec         | make_rec |
| get         | get_field or get_method |
| index       | get_index |
| fn          | make_closure |
| call        | call or call_global |
| match       | series of branches |
| fn_item     | define function, global_store |
| let_item    | eval, global_store |
| import      | import |

## Register Allocation Model

The IR uses an infinite virtual register set (locals).
The lowering pass assigns locals greedily.
The bytecode emitter maps locals to a compact register file.

## Calling Convention (IR level)

Arguments are passed in locals 0..arity-1 of the callee frame.
Return value is in the src of the return instruction.
Closures carry their captured environment as an implicit first slot.

## Constant Pool

Each function has a local constant pool (consts array).
Complex constants (long strings, nested records) use const_pool.
Literals fitting in one word use const directly.

## Next Steps

1. Implement apps/fard_lower.fard — AST to IR lowering
2. Implement apps/fard_ir_interp.fard — IR interpreter
3. Validate IR against fardparse output for all FARD apps
4. Define bytecode encoding in spec/fard_bytecode.md
