; FARD Seed VM v2 — Linux x86_64
; Clean register discipline, no function call overhead in dispatch loop
;
; Register allocation during dispatch:
;   r12 = current frame ptr
;   r13 = code base (fn code bytes)
;   r14 = locals ptr (VAL_SIZE * n)
;   r15 = vstack top ptr (grows up)
;   rbp = fn descriptor ptr
;
; Frame layout (FRAME_SIZE=64 bytes):
;   [0]  fn_desc ptr   (8)
;   [8]  locals ptr    (8)
;   [16] pc            (8)
;   [24] ret_slot      (8)  (-1 = no return)
;   [32] vstack_base   (8)
;   [40] vstack_top    (8)
;   [48] pad           (16)
;
; Value layout (VAL_SIZE=16 bytes):
;   [0]  tag   (8)
;   [8]  i64   (8)

bits 64
default rel

; Syscalls
SYS_READ    equ 0
SYS_WRITE   equ 1
SYS_OPEN    equ 2
SYS_CLOSE   equ 3
SYS_MMAP    equ 9
SYS_EXIT    equ 60

; mmap
PROT_RW     equ 3
MAP_ANON    equ 0x22

; Value tags
TAG_NULL    equ 0
TAG_BOOL    equ 1
TAG_INT     equ 2
TAG_FLOAT   equ 3
TAG_STR     equ 4
TAG_CLOS    equ 7

; Sizes
VAL_SIZE    equ 16
FRAME_SIZE  equ 64
MAX_FRAMES  equ 4096
MAX_GLOBALS equ 256
HEAP_SIZE   equ 0x4000000   ; 64MB

; Opcodes
OP_NOP          equ 0x00
OP_CONST        equ 0x01
OP_LOAD         equ 0x02
OP_STORE        equ 0x03
OP_GLOBAL_LOAD  equ 0x04
OP_GLOBAL_STORE equ 0x05
OP_ADD          equ 0x10
OP_SUB          equ 0x11
OP_MUL          equ 0x12
OP_DIV          equ 0x13
OP_MOD          equ 0x14
OP_NEG          equ 0x15
OP_EQ           equ 0x20
OP_NE           equ 0x21
OP_LT           equ 0x22
OP_LE           equ 0x23
OP_GT           equ 0x24
OP_GE           equ 0x25
OP_AND          equ 0x30
OP_OR           equ 0x31
OP_NOT          equ 0x32
OP_JUMP         equ 0x40
OP_JUMP_IF      equ 0x41
OP_JUMP_IFNOT   equ 0x42
OP_RETURN       equ 0x43
OP_MAKE_CLOSURE equ 0x60
OP_CALL         equ 0x61

section .bss
heap_base:      resq 1
heap_ptr:       resq 1
bc_buf:         resb 0x1000000      ; 16MB bytecode buffer
n_consts:       resq 1
consts_ptr:     resq 1              ; ptr to array of VAL_SIZE entries
n_names:        resq 1
names_ptr:      resq 1              ; ptr to array of ptrs
n_fns:          resq 1
fns_ptr:        resq 1              ; ptr to array of 24-byte fn descs
entry_fn:       resq 1
frame_top:      resq 1
call_stack:     resb FRAME_SIZE * MAX_FRAMES
n_globals:      resq 1
glob_keys:      resq MAX_GLOBALS    ; name_idx stored as u64
glob_vals:      resb VAL_SIZE * MAX_GLOBALS
null_val_buf:   resb VAL_SIZE

section .data
msg_usage:      db "Usage: fardvm <file.fardc>", 10
msg_usage_len   equ $ - msg_usage
msg_err_open:   db "Error: open failed", 10
msg_err_open_len equ $ - msg_err_open
msg_nl:         db 10
msg_true:       db "true", 10
msg_false:      db "false", 10
msg_null_str:   db "null", 10

section .text
global _start

; ── Macros ────────────────────────────────────────────────────────────────────
; VPUSH val_ptr(rax) — push value onto current frame vstack
; Clobbers: rcx
%macro VPUSH 0
    mov rcx, [r12 + 40]        ; vstack_top
    mov rdx, [rax]
    mov [rcx], rdx
    mov rdx, [rax + 8]
    mov [rcx + 8], rdx
    add qword [r12 + 40], VAL_SIZE
%endmacro

; VPOP → rax points to value on vstack (still valid until next push)
%macro VPOP 0
    sub qword [r12 + 40], VAL_SIZE
    mov rax, [r12 + 40]
%endmacro

; ── Entry ─────────────────────────────────────────────────────────────────────
_start:
    ; Check argc >= 2
    mov rax, [rsp]
    cmp rax, 2
    jl .usage

    ; Init heap
    xor rdi, rdi
    mov rsi, HEAP_SIZE
    mov rdx, PROT_RW
    mov r10, MAP_ANON
    mov r8, -1
    xor r9, r9
    mov rax, SYS_MMAP
    syscall
    cmp rax, -1
    je .err_open
    mov [heap_base], rax
    mov [heap_ptr], rax

    ; Open file
    mov rdi, [rsp + 16]
    xor rsi, rsi
    xor rdx, rdx
    mov rax, SYS_OPEN
    syscall
    cmp rax, 0
    jl .err_open
    mov r8, rax                 ; fd

    ; Read into bc_buf
    mov rdi, r8
    lea rsi, [bc_buf]
    mov rdx, 0x1000000
    mov rax, SYS_READ
    syscall
    cmp rax, 0
    jl .err_open

    ; Close
    mov rdi, r8
    mov rax, SYS_CLOSE
    syscall

    ; Parse header
    call parse_bc

    ; Run
    call vm_run                 ; result val ptr in rax

    ; Print result
    call print_val

    ; Exit 0
    xor rdi, rdi
    mov rax, SYS_EXIT
    syscall

.usage:
    mov rdi, 2
    lea rsi, [msg_usage]
    mov rdx, msg_usage_len
    mov rax, SYS_WRITE
    syscall
    mov rdi, 1
    mov rax, SYS_EXIT
    syscall

.err_open:
    mov rdi, 2
    lea rsi, [msg_err_open]
    mov rdx, msg_err_open_len
    mov rax, SYS_WRITE
    syscall
    mov rdi, 1
    mov rax, SYS_EXIT
    syscall

; ── heap_alloc(rdi=size) → rax ────────────────────────────────────────────────
heap_alloc:
    mov rax, [heap_ptr]
    add [heap_ptr], rdi
    ret

; ── parse_bc ──────────────────────────────────────────────────────────────────
parse_bc:
    lea r13, [bc_buf]
    add r13, 8                  ; skip magic + version

    ; Const pool
    mov eax, [r13]
    mov [n_consts], rax
    add r13, 4

    mov rdi, rax
    imul rdi, VAL_SIZE
    call heap_alloc
    mov [consts_ptr], rax
    mov rbx, rax                ; rbx = consts array

    mov rcx, [n_consts]
    xor r9, r9
.pc_loop:
    test rcx, rcx
    jz .pc_done
    dec rcx
    mov rax, r9
    imul rax, VAL_SIZE
    add rax, rbx                ; rax = slot ptr

    movzx edx, byte [r13]
    inc r13
    mov [rax], rdx              ; tag

    cmp dl, TAG_NULL
    je .pc_null
    cmp dl, TAG_BOOL
    je .pc_bool
    cmp dl, TAG_INT
    je .pc_int
    cmp dl, TAG_FLOAT
    je .pc_float
    ; STR
    mov [rax + 8], r13
    mov ecx, [r13]
    add r13, 4
    add r13, rcx
    mov rcx, [n_consts]         ; restore
    inc r9
    jmp .pc_loop

.pc_null:
    mov qword [rax + 8], 0
    inc r9
    jmp .pc_loop
.pc_bool:
    movzx edx, byte [r13]
    inc r13
    mov [rax + 8], rdx
    inc r9
    jmp .pc_loop
.pc_int:
    mov rdx, [r13]
    add r13, 8
    mov [rax + 8], rdx
    inc r9
    jmp .pc_loop
.pc_float:
    mov rdx, [r13]
    add r13, 8
    mov [rax + 8], rdx
    inc r9
    jmp .pc_loop

.pc_done:
    ; Name pool
    mov eax, [r13]
    mov [n_names], rax
    add r13, 4

    mov rdi, rax
    imul rdi, 8
    call heap_alloc
    mov [names_ptr], rax
    mov rbx, rax

    mov rcx, [n_names]
    xor r9, r9
.np_loop:
    test rcx, rcx
    jz .np_done
    dec rcx
    mov [rbx + r9 * 8], r13
    mov eax, [r13]
    add r13, 4
    add r13, rax
    inc r9
    jmp .np_loop
.np_done:

    ; Function table
    mov eax, [r13]
    mov [n_fns], rax
    add r13, 4

    mov rdi, rax
    imul rdi, 24
    call heap_alloc
    mov [fns_ptr], rax
    mov rbx, rax

    mov rcx, [n_fns]
    xor r9, r9
.fn_loop:
    test rcx, rcx
    jz .fn_done
    dec rcx
    mov rax, r9
    imul rax, 24
    add rax, rbx
    mov edx, [r13]
    mov [rax], edx              ; name_idx
    mov edx, [r13 + 4]
    mov [rax + 4], edx          ; arity
    mov edx, [r13 + 8]
    mov [rax + 8], edx          ; locals
    mov edx, [r13 + 12]
    mov [rax + 12], edx         ; n_bytes
    lea rdx, [r13 + 16]
    mov [rax + 16], rdx         ; code ptr
    add r13, 16
    mov edx, [rax + 12]
    add r13, rdx
    inc r9
    jmp .fn_loop
.fn_done:

    mov eax, [r13]
    mov [entry_fn], rax
    ret

; ── vm_run → rax (result val ptr) ────────────────────────────────────────────
vm_run:
    mov qword [frame_top], -1
    mov qword [n_globals], 0

    ; Push entry frame
    mov rax, [entry_fn]
    call push_frame

    ; Load frame registers
    call load_frame_regs

.dispatch:
    ; Check pc vs code size
    mov rax, [r12 + 16]         ; pc
    mov ecx, [rbp + 12]         ; n_bytes
    cmp rax, rcx
    jge .pop_and_cont

    ; Fetch opcode
    mov al, [r13 + rax]
    movzx eax, al
    mov rcx, [r12 + 16]
    inc rcx
    mov [r12 + 16], rcx         ; pc++

    ; Dispatch
    cmp al, OP_NOP
    je .dispatch
    cmp al, OP_CONST
    je .do_const
    cmp al, OP_LOAD
    je .do_load
    cmp al, OP_STORE
    je .do_store
    cmp al, OP_GLOBAL_LOAD
    je .do_global_load
    cmp al, OP_GLOBAL_STORE
    je .do_global_store
    cmp al, OP_ADD
    je .do_add
    cmp al, OP_SUB
    je .do_sub
    cmp al, OP_MUL
    je .do_mul
    cmp al, OP_DIV
    je .do_div
    cmp al, OP_MOD
    je .do_mod
    cmp al, OP_NEG
    je .do_neg
    cmp al, OP_EQ
    je .do_eq
    cmp al, OP_NE
    je .do_ne
    cmp al, OP_LT
    je .do_lt
    cmp al, OP_LE
    je .do_le
    cmp al, OP_GT
    je .do_gt
    cmp al, OP_GE
    je .do_ge
    cmp al, OP_AND
    je .do_and
    cmp al, OP_OR
    je .do_or
    cmp al, OP_NOT
    je .do_not
    cmp al, OP_JUMP
    je .do_jump
    cmp al, OP_JUMP_IF
    je .do_jump_if
    cmp al, OP_JUMP_IFNOT
    je .do_jump_ifnot
    cmp al, OP_RETURN
    je .do_return
    cmp al, OP_MAKE_CLOSURE
    je .do_make_closure
    cmp al, OP_CALL
    je .do_call
    jmp .dispatch

; ── CONST ─────────────────────────────────────────────────────────────────────
.do_const:
    mov rcx, [r12 + 16]
    mov eax, [r13 + rcx]        ; idx u32
    add rcx, 4
    mov [r12 + 16], rcx
    ; Get const ptr
    mov rcx, [consts_ptr]
    imul rax, VAL_SIZE
    add rax, rcx
    VPUSH
    jmp .dispatch

; ── LOAD ──────────────────────────────────────────────────────────────────────
.do_load:
    mov rcx, [r12 + 16]
    movzx eax, byte [r13 + rcx]
    inc rcx
    mov [r12 + 16], rcx
    ; locals[slot]
    imul rax, VAL_SIZE
    add rax, r14
    VPUSH
    jmp .dispatch

; ── STORE ─────────────────────────────────────────────────────────────────────
.do_store:
    mov rcx, [r12 + 16]
    movzx eax, byte [r13 + rcx]
    inc rcx
    mov [r12 + 16], rcx
    imul rax, VAL_SIZE
    add rax, r14                ; rax = dst slot ptr in locals
    mov rbx, rax                ; save dst
    VPOP                        ; rax = src val ptr (vstack top)
    mov rdx, [rax]
    mov [rbx], rdx
    mov rdx, [rax + 8]
    mov [rbx + 8], rdx
    jmp .dispatch

; ── GLOBAL_LOAD ───────────────────────────────────────────────────────────────
.do_global_load:
    mov rcx, [r12 + 16]
    mov eax, [r13 + rcx]
    add rcx, 4
    mov [r12 + 16], rcx
    ; Search globals
    mov rdi, rax
    call glob_get
    VPUSH
    jmp .dispatch

; ── GLOBAL_STORE ──────────────────────────────────────────────────────────────
.do_global_store:
    mov rcx, [r12 + 16]
    mov eax, [r13 + rcx]
    add rcx, 4
    mov [r12 + 16], rcx
    mov rdi, rax
    VPOP
    mov rsi, rax                ; val ptr
    call glob_set
    jmp .dispatch

; ── ADD ───────────────────────────────────────────────────────────────────────
.do_add:
    VPOP
    mov rcx, [rax + 8]         ; b
    VPOP
    mov rax, [rax + 8]         ; a
    add rax, rcx
    call alloc_int
    VPUSH
    jmp .dispatch

; ── SUB ───────────────────────────────────────────────────────────────────────
.do_sub:
    VPOP
    mov rcx, [rax + 8]
    VPOP
    mov rax, [rax + 8]
    sub rax, rcx
    call alloc_int
    VPUSH
    jmp .dispatch

; ── MUL ───────────────────────────────────────────────────────────────────────
.do_mul:
    VPOP
    mov rcx, [rax + 8]         ; b
    VPOP
    mov rax, [rax + 8]         ; a
    imul rax, rcx
    call alloc_int
    VPUSH
    jmp .dispatch

; ── DIV ───────────────────────────────────────────────────────────────────────
.do_div:
    VPOP
    mov rcx, [rax + 8]
    VPOP
    mov rax, [rax + 8]
    cqo
    idiv rcx
    call alloc_int
    VPUSH
    jmp .dispatch

; ── MOD ───────────────────────────────────────────────────────────────────────
.do_mod:
    VPOP
    mov rcx, [rax + 8]
    VPOP
    mov rax, [rax + 8]
    cqo
    idiv rcx
    mov rax, rdx
    call alloc_int
    VPUSH
    jmp .dispatch

; ── NEG ───────────────────────────────────────────────────────────────────────
.do_neg:
    VPOP
    mov rax, [rax + 8]
    neg rax
    call alloc_int
    VPUSH
    jmp .dispatch

; ── Comparisons ───────────────────────────────────────────────────────────────
.do_eq:
    VPOP
    mov rcx, [rax + 8]
    VPOP
    mov rax, [rax + 8]
    cmp rax, rcx
    sete al
    movzx rax, al
    call alloc_bool
    VPUSH
    jmp .dispatch

.do_ne:
    VPOP
    mov rcx, [rax + 8]
    VPOP
    mov rax, [rax + 8]
    cmp rax, rcx
    setne al
    movzx rax, al
    call alloc_bool
    VPUSH
    jmp .dispatch

.do_lt:
    VPOP
    mov rcx, [rax + 8]
    VPOP
    mov rax, [rax + 8]
    cmp rax, rcx
    setl al
    movzx rax, al
    call alloc_bool
    VPUSH
    jmp .dispatch

.do_le:
    VPOP
    mov rcx, [rax + 8]
    VPOP
    mov rax, [rax + 8]
    cmp rax, rcx
    setle al
    movzx rax, al
    call alloc_bool
    VPUSH
    jmp .dispatch

.do_gt:
    VPOP
    mov rcx, [rax + 8]
    VPOP
    mov rax, [rax + 8]
    cmp rax, rcx
    setg al
    movzx rax, al
    call alloc_bool
    VPUSH
    jmp .dispatch

.do_ge:
    VPOP
    mov rcx, [rax + 8]
    VPOP
    mov rax, [rax + 8]
    cmp rax, rcx
    setge al
    movzx rax, al
    call alloc_bool
    VPUSH
    jmp .dispatch

; ── Logic ─────────────────────────────────────────────────────────────────────
.do_and:
    VPOP
    mov rcx, [rax + 8]
    VPOP
    mov rax, [rax + 8]
    and rax, rcx
    call alloc_bool
    VPUSH
    jmp .dispatch

.do_or:
    VPOP
    mov rcx, [rax + 8]
    VPOP
    mov rax, [rax + 8]
    or rax, rcx
    call alloc_bool
    VPUSH
    jmp .dispatch

.do_not:
    VPOP
    mov rax, [rax + 8]
    xor rax, 1
    call alloc_bool
    VPUSH
    jmp .dispatch

; ── JUMP ──────────────────────────────────────────────────────────────────────
.do_jump:
    mov rcx, [r12 + 16]
    movsx rax, word [r13 + rcx]
    mov [r12 + 16], rax
    jmp .dispatch

; ── JUMP_IF ───────────────────────────────────────────────────────────────────
.do_jump_if:
    mov rcx, [r12 + 16]
    movsx rdx, word [r13 + rcx]
    add rcx, 2
    VPOP
    cmp qword [rax + 8], 0
    je .ji_false
    mov [r12 + 16], rdx
    jmp .dispatch
.ji_false:
    mov [r12 + 16], rcx
    jmp .dispatch

; ── JUMP_IFNOT ────────────────────────────────────────────────────────────────
.do_jump_ifnot:
    mov rcx, [r12 + 16]
    movsx rdx, word [r13 + rcx]
    add rcx, 2
    VPOP
    cmp qword [rax + 8], 0
    jne .jin_false
    mov [r12 + 16], rdx
    jmp .dispatch
.jin_false:
    mov [r12 + 16], rcx
    jmp .dispatch

; ── RETURN ────────────────────────────────────────────────────────────────────
.do_return:
    VPOP
    mov rbx, rax                ; save return val ptr
    mov rcx, [r12 + 24]         ; ret_slot

    ; Pop frame
    dec qword [frame_top]
    cmp qword [frame_top], -1
    je .vm_done

    ; Restore caller frame
    call load_frame_regs

    ; Write return value to caller's local slot
    cmp rcx, 0
    jl .dispatch                ; ret_slot < 0, discard
    mov rax, rcx
    imul rax, VAL_SIZE
    add rax, r14                ; dst in caller locals
    mov rdx, [rbx]
    mov [rax], rdx
    mov rdx, [rbx + 8]
    mov [rax + 8], rdx
    jmp .dispatch

.vm_done:
    mov rax, rbx
    ret

; ── MAKE_CLOSURE ──────────────────────────────────────────────────────────────
.do_make_closure:
    mov rcx, [r12 + 16]
    mov eax, [r13 + rcx]
    add rcx, 4
    mov [r12 + 16], rcx
    ; Allocate closure val
    push rax
    mov rdi, VAL_SIZE
    call heap_alloc
    pop rcx
    mov qword [rax], TAG_CLOS
    mov [rax + 8], rcx
    VPUSH
    jmp .dispatch

; ── CALL ──────────────────────────────────────────────────────────────────────
.do_call:
    mov rcx, [r12 + 16]
    movzx ebx, byte [r13 + rcx] ; n_args
    inc rcx
    ; Read STORE dst slot (STORE opcode + slot = 2 bytes after CALL+nargs)
    movzx esi, byte [r13 + rcx + 1]  ; dst_slot
    add rcx, 2                  ; skip STORE opcode + slot
    mov [r12 + 16], rcx         ; advance caller PC past CALL+nargs+STORE+slot

    ; Pop args into temp buffer on heap
    mov rdi, rbx
    imul rdi, VAL_SIZE
    push rbx
    push rsi
    call heap_alloc
    mov r11, rax                ; r11 = args buffer

    ; Pop args in reverse order (last arg first from stack)
    pop rsi                     ; dst_slot
    pop rbx                     ; n_args
    mov rcx, rbx
.pop_args:
    test rcx, rcx
    jz .pop_fn
    dec rcx
    VPOP                        ; rax = val ptr
    mov rdx, rcx
    imul rdx, VAL_SIZE
    mov r9, [rax]
    mov [r11 + rdx], r9
    mov r9, [rax + 8]
    mov [r11 + rdx + 8], r9
    jmp .pop_args

.pop_fn:
    VPOP                        ; rax = closure val
    cmp qword [rax], TAG_CLOS
    jne .dispatch               ; not a closure, skip

    mov rdx, [rax + 8]          ; fn_index
    push rsi                    ; dst_slot
    push r11                    ; args buffer
    push rbx                    ; n_args
    mov rax, rdx
    call push_frame             ; clobbers rbx, r12 updated
    call load_frame_regs        ; r12, r13, r14, rbp updated for new frame

    pop rcx                     ; n_args
    pop r11                     ; args buffer
    pop rdx                     ; dst_slot
    mov [r12 + 24], rdx         ; set ret_slot in new frame

    ; Copy args into new frame's locals (r14 = new locals)
    xor rbx, rbx
.copy_args:
    cmp rbx, rcx
    jge .call_done
    mov rax, rbx
    imul rax, VAL_SIZE
    mov r9, [r11 + rax]
    mov [r14 + rax], r9
    mov r9, [r11 + rax + 8]
    mov [r14 + rax + 8], r9
    inc rbx
    jmp .copy_args
.call_done:
    jmp .dispatch

.pop_and_cont:
    ; PC past end of function — implicit return null
    mov rax, rcx                ; save
    lea rbx, [null_val_buf]
    mov qword [rbx], TAG_NULL
    mov qword [rbx + 8], 0
    mov rcx, [r12 + 24]
    dec qword [frame_top]
    cmp qword [frame_top], -1
    je .vm_done2
    call load_frame_regs
    cmp rcx, 0
    jl .dispatch
    mov rax, rcx
    imul rax, VAL_SIZE
    add rax, r14
    mov qword [rax], TAG_NULL
    mov qword [rax + 8], 0
    jmp .dispatch
.vm_done2:
    lea rax, [null_val_buf]
    ret

; ── load_frame_regs ───────────────────────────────────────────────────────────
; Load r12, r13, r14, r15, rbp from current frame
load_frame_regs:
    mov rax, [frame_top]
    imul rax, FRAME_SIZE
    lea r12, [call_stack]
    add r12, rax                ; r12 = frame ptr
    mov rbp, [r12]              ; fn descriptor
    mov r13, [rbp + 16]         ; code ptr
    mov r14, [r12 + 8]          ; locals ptr
    ret

; ── push_frame(fn_index in rax) ───────────────────────────────────────────────
push_frame:
    push rax
    ; Get fn descriptor
    mov rbx, [fns_ptr]
    imul rax, 24
    add rax, rbx
    mov rbx, rax                ; rbx = fn desc

    ; Increment frame_top
    inc qword [frame_top]
    mov rdi, [frame_top]
    imul rdi, FRAME_SIZE
    lea r12, [call_stack]
    add r12, rdi                ; r12 = new frame

    mov [r12], rbx              ; fn_desc ptr

    ; Allocate locals
    mov eax, [rbx + 8]          ; n_locals
    imul rax, VAL_SIZE
    mov rdi, rax
    call heap_alloc
    mov [r12 + 8], rax          ; locals_ptr

    ; Zero locals
    mov rdi, rax
    mov ecx, [rbx + 8]
    imul rcx, VAL_SIZE
    xor rax, rax
    push rcx
    push rdi
    mov rdx, rcx
    shr rcx, 3
    rep stosq
    pop rdi
    pop rcx

    ; PC = 0, ret_slot = -1
    mov qword [r12 + 16], 0
    mov qword [r12 + 24], -1

    ; Allocate vstack
    mov rdi, VAL_SIZE * 256
    call heap_alloc
    mov [r12 + 32], rax
    mov [r12 + 40], rax         ; vstack_top = base (empty)

    pop rax
    ret

; ── alloc_int(rax=value) → rax=val_ptr ────────────────────────────────────────
alloc_int:
    push rax
    mov rdi, VAL_SIZE
    call heap_alloc
    pop rcx
    mov qword [rax], TAG_INT
    mov [rax + 8], rcx
    ret

; ── alloc_bool(rax=0or1) → rax=val_ptr ───────────────────────────────────────
alloc_bool:
    push rax
    mov rdi, VAL_SIZE
    call heap_alloc
    pop rcx
    mov qword [rax], TAG_BOOL
    mov [rax + 8], rcx
    ret

; ── glob_get(rdi=name_idx) → rax=val_ptr ─────────────────────────────────────
glob_get:
    mov rcx, [n_globals]
    xor rbx, rbx
    lea r9, [glob_keys]
    lea r10, [glob_vals]
.gg_loop:
    cmp rbx, rcx
    jge .gg_miss
    cmp [r9 + rbx * 8], rdi
    je .gg_hit
    inc rbx
    jmp .gg_loop
.gg_hit:
    mov rax, rbx
    imul rax, VAL_SIZE
    add rax, r10
    ret
.gg_miss:
    lea rax, [null_val_buf]
    ret

; ── glob_set(rdi=name_idx, rsi=val_ptr) ──────────────────────────────────────
glob_set:
    mov rcx, [n_globals]
    xor rbx, rbx
    lea r9, [glob_keys]
    lea r10, [glob_vals]
.gs_loop:
    cmp rbx, rcx
    jge .gs_new
    cmp [r9 + rbx * 8], rdi
    je .gs_update
    inc rbx
    jmp .gs_loop
.gs_update:
    mov rax, rbx
    imul rax, VAL_SIZE
    add rax, r10
    mov rdx, [rsi]
    mov [rax], rdx
    mov rdx, [rsi + 8]
    mov [rax + 8], rdx
    ret
.gs_new:
    mov [r9 + rbx * 8], rdi
    mov rax, rbx
    imul rax, VAL_SIZE
    add rax, r10
    mov rdx, [rsi]
    mov [rax], rdx
    mov rdx, [rsi + 8]
    mov [rax + 8], rdx
    inc qword [n_globals]
    ret

; ── print_val(rax=val_ptr) ────────────────────────────────────────────────────
print_val:
    test rax, rax
    jz .pv_null
    mov rcx, [rax]
    cmp rcx, TAG_INT
    je .pv_int
    cmp rcx, TAG_BOOL
    je .pv_bool
    jmp .pv_null

.pv_int:
    mov rax, [rax + 8]
    call print_i64
    mov rdi, 1
    lea rsi, [msg_nl]
    mov rdx, 1
    mov rax, SYS_WRITE
    syscall
    ret

.pv_bool:
    cmp qword [rax + 8], 0
    je .pv_false
    mov rdi, 1
    lea rsi, [msg_true]
    mov rdx, 5
    mov rax, SYS_WRITE
    syscall
    ret
.pv_false:
    mov rdi, 1
    lea rsi, [msg_false]
    mov rdx, 6
    mov rax, SYS_WRITE
    syscall
    ret

.pv_null:
    mov rdi, 1
    lea rsi, [msg_null_str]
    mov rdx, 5
    mov rax, SYS_WRITE
    syscall
    ret

; ── print_i64(rax=value) ──────────────────────────────────────────────────────
print_i64:
    push rbp
    mov rbp, rsp
    sub rsp, 32
    test rax, rax
    jns .pi_pos
    push rax
    mov rdi, 1
    lea rsi, [pi_minus]
    mov rdx, 1
    mov rax, SYS_WRITE
    syscall
    pop rax
    neg rax
.pi_pos:
    lea rdi, [rbp - 22]
    mov byte [rbp - 2], 0
    mov rcx, 20
    mov rbx, 10
.pi_loop:
    xor rdx, rdx
    div rbx
    add dl, '0'
    dec rcx
    mov [rdi + rcx], dl
    test rax, rax
    jnz .pi_loop
    lea rsi, [rdi + rcx]
    mov rdx, 20
    sub rdx, rcx
    mov rdi, 1
    mov rax, SYS_WRITE
    syscall
    leave
    ret

section .data
pi_minus: db "-"
