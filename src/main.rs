#![deny(clippy::pedantic)]
use crate::interpreter::Value;
use interpreter::Interpreter;
mod ast;
mod ast_builder;
mod interpreter;

fn main() {
    let source = r#"
DEFINE .name "Jeffrey"
define .age 33
START:
    SET %0xFF, 1
    LOAD %0xFF, R1
    CLEAR R4

MAIN:
    SUB 100, .age
    ADD %100, %200
    ADD r4, 1
    jmp END r4=106;befg
    jmp MAIN ; also a comment
    not 10
    ;this is a comment to help, maybe?
END:
    LOAD %R2, R3;load from the address contained in R2
    POP
    POP r7
    HALT
"#;
    let result = ast_builder::parse_program(source);
    if let Ok(program) = result {
        let mut interpreter = Interpreter::new();
        interpreter.check(&program);
        for (key, value) in &interpreter.constants {
            println!("{key} : {value:?}");
        }
        for (name, position) in &interpreter.labels {
            println!("{name} : {position}");
        }
        let result =
            interpreter.execute_set(Value::Number(100), &ast::Operand::Memory("100".to_string()));
        match result {
            Ok(()) => println!("Value is: {:?}", interpreter.memory[100]),
            Err(e) => eprintln!("Error: {e}"),
        }
        let result = interpreter.execute_set(
            Value::Number(-50),
            &ast::Operand::Register("r4".to_string()),
        );
        match result {
            Ok(()) => println!("Value is: {:?}", interpreter.registers.get("r4")),
            Err(e) => eprintln!("Error: {e}"),
        }
        println!(
            "Value of r2 is: '{:?}' before move",
            interpreter.registers.get("r2")
        );
        let result = interpreter.execute_move(
            &ast::Operand::Register("r4".to_string()),
            &ast::Operand::Register("r2".to_string()),
        );
        match result {
            Ok(()) => println!("Value is: {:?}", interpreter.registers.get("r2")),
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}
