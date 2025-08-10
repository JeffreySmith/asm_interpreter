# asm_interpreter
An initial poc of a faux asm interpreter for a puzzle game idea.

It is sort of vaguely inspired by the 6502, but has very few limitation in order to keep things simple since the idea is that even non-programmers could eventually understand this.

It has 8 general purpose registers, r0 through r8. It has an 'A' register as an accumulator (the resuts of all math instructions is put into 'A'), and an F flag (not yet doing anything) for flags.

There are currently 256 "slots" in memory, which will likely be decreased later, in which any integer (positive or negative that fits into a 64 signed int) or string value can be stored. Again, this is to keep it very simple. This also applies to all of the registers.

Numbers with decimals are intentionally not implemented. Numbers when divided will be truncated. Only whole numbers are valid.

At the moment, instructions can be either in lowercase or uppercase, but not mixed case. This will eventually change. Labels, however, are case sensitive, as well as string comparisons.

Equality comparisons use only one '=' instead of the normal convention of using two. This isn't a huge problem since '=' isn't used anywhere else in the instruction set.

Comments can be on their own line or inline. Anything after a ';' will be ignored by the interpreter.

Work is on going, so things will likely change.

The following instructions exist for this faux cpu:
```
DEFINE .constant value ; Constant must be a string of some kind. These are only evaluated once, and cannot be changed while the program is running.
SET dest, value ; value can be an int or string
STORE register, memory_address
LOAD memory_address, register
CLEAR dest
MOV src, dest

ADD left, right ; this can be used on ints or strings
SUB left, right ; this can only be used on ints
MUL left, right ; this can be used on ints and strings. string*int results in a duplicated string
DIV numerator, denominator ; works on both ints and strings. All int division is trunctated, and dividing a string by another string gives you the difference in their length
INC dest ; ++ operator for some dest, either a register or a memory address
DEC dest ; same as above, but --

AND left, right
OR left, right
XOR left, right
NOT src 

JMP label (left comparison right) ; this can be '=', '<','<=','>','>='. Example: R3=100. This is how branching can be achieved
CALL label
RET ; returns from the function
HALT ; end the program
```

An example program:
```
DEFINE .name "my_name"
define .age 100

START:
    SET %0xFF, 10000
    LOAD %0xFF, R1
    CLEAR R4
    SET A, 1
    SET %0, 1
    set %1, 2
    SET %2, 3
    SET %3, 4
    SET %4, 5

MAIN:
    SUB 100, .age
    SET %11, "abc"
    SET %12, 'h'
    STORE A, %10
    ADD %100, %200
    ;ADD R4, 1
    ;MOV A, R4
    call add_a_r4
    SET %201, .add
    jmp END r4=5
    jmp MAIN ; also a comment
    not 10

add_a_r4:
    INC R4
    ADD A, %R4
    ret

END:
    PUSH 100
    PUSH 200
    LOAD %R2, R3;load from the address contained in R2
    POP
    POP r7
    push "abcdefg"
    SET r6, "abc"
    jmp IS_FINE .name="my_name"
    halt
IS_FINE:
    SET %0xFF, "True"
```
