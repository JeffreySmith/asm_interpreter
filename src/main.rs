#![deny(clippy::pedantic)]
use crate::value::Value;
use interpreter::Interpreter;
mod ast;
mod ast_builder;
mod interpreter;
mod value;

fn main() {
    let source = r#"
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
    define .add "constant abc"
    SET %201, .add
    jmp END r4=5
    jmp MAIN ; also a comment
    not 10
    ;this is a comment to help, maybe?

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
    NOT -100
    STORE A, %15
    SET %16, "previous should be NOT -100"
    AND 2,6
    STORE A, %17
    AND 2,6
"#;
    let mut interpreter = Interpreter::new();
    let result = interpreter.parse(source);

    match result {
        Ok(()) => {
            interpreter.run();
        }
        Err(e) => eprintln!("Failed to parse: {e:?}"),
    }
}
