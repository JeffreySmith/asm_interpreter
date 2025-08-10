use crate::ast::{Comparison, Instruction, Operand, Statement};
use crate::{Value, ast_builder};
use std::convert::TryInto;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::{collections::HashMap, num::ParseIntError, sync::Arc, sync::RwLock};

const RAM_SLOTS: usize = 256;
const DISPLAY_PIXELS: usize = 16 * 16;
const ACC: &str = "a";

const REGISTERS: [&str; 10] = ["a", "f", "r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7"];

pub struct Interpreter {
    pub display: Arc<RwLock<Vec<(i32, i32, i32)>>>,
    pub registers: Arc<RwLock<HashMap<String, Value>>>,
    pub memory: Arc<RwLock<Vec<Value>>>,
    pub stack: Arc<RwLock<Vec<Value>>>,
    labels: HashMap<String, usize>,
    constants: HashMap<String, Value>,
    statements: Vec<Statement>,
    pub pc: AtomicUsize,
    pub call_stack: Arc<RwLock<Vec<usize>>>,
    pub running: AtomicBool,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut registers: HashMap<String, Value> = HashMap::new();
        for reg in REGISTERS {
            registers.insert((*reg).to_string(), Value::default());
        }

        Interpreter {
            display: Arc::new(RwLock::new(vec![(0, 0, 0); DISPLAY_PIXELS])),
            registers: Arc::new(RwLock::new(registers)),
            memory: Arc::new(RwLock::new(vec![Value::Number(0); RAM_SLOTS])),
            stack: Arc::new(RwLock::new(Vec::new())),
            labels: HashMap::new(),
            constants: HashMap::new(),
            statements: Vec::new(),
            pc: AtomicUsize::new(0),
            call_stack: Arc::new(RwLock::new(Vec::new())),
            running: AtomicBool::new(true),
        }
    }
    pub fn parse(&mut self, contents: &str) -> Result<(), ast_builder::ParseError> {
        self.statements = ast_builder::parse_program(contents)?;
        self.compile();
        Ok(())
    }
    fn compile(&mut self) {
        for (i, statement) in self.statements.iter().enumerate() {
            match statement {
                Statement::CompileTime(instr) => match instr {
                    Instruction::Define { name, value } => {
                        let resolved = resolve_operand_compile(value, &self.constants);
                        if let Some(val) = resolved {
                            self.constants.insert(name.clone(), val);
                        }
                    }
                    _ => unimplemented!(),
                },
                Statement::Label(name) => _ = self.labels.insert(name.clone(), i),
                Statement::Instruction(_) => {}
            }
        }
    }

    pub fn run(&mut self) {
        while self.running.load(Ordering::SeqCst) {
            let result = self.step();
            if let Err(e) = result {
                eprintln!("Error occured: {e}");
                self.running.store(false, Ordering::SeqCst);
                break;
            }
            let pc = self.pc.load(Ordering::SeqCst);
            if pc < self.statements.len() {
                println!(
                    "Current instruction: {:#?}",
                    self.statements[self.pc.load(Ordering::SeqCst)]
                );
            }
        }
        let stack = self
            .stack
            .read()
            .map_err(|e| format!("Memory lock poisoned: {e}"))
            .unwrap();
        let registers = self
            .registers
            .read()
            .map_err(|e| format!("Memory lock poisoned: {e}"))
            .unwrap();
        let memory = self
            .memory
            .read()
            .map_err(|e| format!("Memory lock poisoned: {e}"))
            .unwrap();
        let call_stack = self
            .call_stack
            .read()
            .map_err(|e| format!("Memory lock poisoned: {e}"))
            .unwrap();
        println!("Stack: {stack:#?}");
        println!("\n\nRegisters: {registers:#?}");
        println!("\n\nMemory: {memory:?}");
        println!("\n\nCall stack: {call_stack:#?}");
    }

    pub fn step(&mut self) -> Result<(), String> {
        let pc = self.pc.load(Ordering::SeqCst);
        if pc >= self.statements.len() {
            self.execute_halt();
            return Ok(());
        }
        let mut increment_pc = true;
        match &self.statements[pc].clone() {
            Statement::Label(_) | Statement::CompileTime(_) => {}
            Statement::Instruction(instruction) => match instruction {
                Instruction::Define { name: _, value: _ } => {}
                Instruction::Set { value, dest } => {
                    let val = self
                        .get_operand_value(value)
                        .ok_or_else(|| "No value found".to_string())?;
                    self.execute_set(val, dest)?;
                }
                Instruction::Load { src, dest } => self.execute_load(src, dest)?,
                Instruction::Clear { target } => self.execute_clear(target)?,
                Instruction::Mov { src, dest } => self.execute_move(src, dest)?,
                Instruction::Add { left, right } => self.execute_add(left, right)?,
                Instruction::Sub { left, right } => self.execute_sub(left, right)?,
                Instruction::Mul { left, right } => self.execute_mul(left, right)?,
                Instruction::Div { left, right } => self.execute_div(left, right)?,
                Instruction::Inc { dest } => self.execute_inc(dest)?,
                Instruction::Dec { dest } => self.execute_dec(dest)?,
                Instruction::And { left, right } => self.execute_and(left, right)?,
                Instruction::Or { left, right } => self.execute_or(left, right)?,
                Instruction::Xor { left, right } => self.execute_xor(left, right)?,
                Instruction::Not { op } => self.execute_not(op)?,
                Instruction::Jmp { target, comparison } => {
                    if let Operand::Identifier(label) = target {
                        self.execute_jump(label, comparison.as_ref())?;
                        increment_pc = false;
                    } else {
                        return Err(format!("Invalid target '{target:?}'"));
                    }
                }
                Instruction::Call { target } => {
                    if let Operand::Identifier(label) = target {
                        self.execute_call(label)?;
                        increment_pc = false;
                    } else {
                        return Err(format!("Invalid target '{target:?}"));
                    }
                }
                Instruction::Ret => {
                    self.execute_ret()?;
                    increment_pc = false;
                }
                Instruction::Halt => {
                    self.execute_halt();
                    increment_pc = false;
                }
                Instruction::Push { src } => self.execute_push(src)?,
                Instruction::Pop { dest } => self.execute_pop(dest.as_ref())?,
                Instruction::Store { value, dest } => self.execute_store(value, dest)?,
            },
        }

        if increment_pc {
            self.pc.store(pc + 1, Ordering::SeqCst);
        }
        Ok(())
    }

    fn execute_set(&mut self, value: Value, dest: &Operand) -> Result<(), String> {
        match dest {
            Operand::Register(name) => self.set_register(name, value),
            Operand::Memory(address) => self.set_address(address, value),
            _ => Err(format!("Invalid operand for set: {dest:?}")),
        }
    }

    fn execute_load(&mut self, src: &Operand, register: &Operand) -> Result<(), String> {
        let value = match src {
            Operand::Memory(address) | Operand::Number(address) => self.get_address(address),
            Operand::Register(reg) => self.get_register(reg),
            _ => Err(format!("Invalid src for LOAD '{src:#?}'")),
        }?;
        if let Operand::Register(name) = register {
            self.set_register(name, value)
        } else {
            Err("Invalid operands used in LOAD".to_string())
        }
    }

    fn execute_clear(&mut self, dest: &Operand) -> Result<(), String> {
        self.set_operand_value(dest, &Value::default())
    }

    fn execute_move(&mut self, src: &Operand, dest: &Operand) -> Result<(), String> {
        let value = self
            .get_operand_value(src)
            .ok_or_else(|| format!("Could not resolve value of operand '{src:?}'"))?;

        self.set_operand_value(dest, &value)?;
        Ok(())
    }

    fn execute_add(&mut self, left: &Operand, right: &Operand) -> Result<(), String> {
        let left_val = self
            .get_operand_value(left)
            .ok_or_else(|| format!("Could not resolve value of operand '{left:?}'"))?;

        let right_val = self
            .get_operand_value(right)
            .ok_or_else(|| format!("Could not resolve value of operand '{right:?}'"))?;
        let value = Value::add(&left_val, &right_val)?;
        self.set_operand_value(&Operand::Register(ACC.to_string()), &value)
    }
    fn execute_sub(&mut self, left: &Operand, right: &Operand) -> Result<(), String> {
        let left_val = self
            .get_operand_value(left)
            .ok_or_else(|| format!("Could not resolve value of operand '{left:?}'"))?;

        let right_val = self
            .get_operand_value(right)
            .ok_or_else(|| format!("Could not resolve value of operand '{right:?}'"))?;
        let value = Value::sub(&left_val, &right_val)?;
        self.set_operand_value(&Operand::Register(ACC.to_string()), &value)
    }
    fn execute_mul(&mut self, left: &Operand, right: &Operand) -> Result<(), String> {
        let left_val = self
            .get_operand_value(left)
            .ok_or_else(|| format!("Could not resolve value of operand '{left:?}'"))?;

        let right_val = self
            .get_operand_value(right)
            .ok_or_else(|| format!("Could not resolve value of operand '{right:?}'"))?;
        let value = Value::mul(&left_val, &right_val)?;
        self.set_operand_value(&Operand::Register(ACC.to_string()), &value)
    }
    fn execute_div(&mut self, left: &Operand, right: &Operand) -> Result<(), String> {
        let left_val = self
            .get_operand_value(left)
            .ok_or_else(|| format!("Could not resolve value of operand '{left:?}'"))?;

        let right_val = self
            .get_operand_value(right)
            .ok_or_else(|| format!("Could not resolve value of operand '{right:?}'"))?;
        let value = Value::div(&left_val, &right_val)?;
        self.set_operand_value(&Operand::Register(ACC.to_string()), &value)
    }
    fn execute_inc(&mut self, dest: &Operand) -> Result<(), String> {
        let value = self
            .get_operand_value(dest)
            .ok_or_else(|| format!("Could not resolve value of operand '{dest:?}'"))?;

        let result = Value::add(&value, &Value::Number(1))?;
        self.set_operand_value(dest, &result)
    }
    fn execute_dec(&mut self, dest: &Operand) -> Result<(), String> {
        let value = self
            .get_operand_value(dest)
            .ok_or_else(|| format!("Could not resolve value of operand '{dest:?}'"))?;

        let result = Value::sub(&value, &Value::Number(1))?;
        self.set_operand_value(dest, &result)
    }

    fn execute_and(&mut self, left: &Operand, right: &Operand) -> Result<(), String> {
        let left_val = self
            .get_operand_value(left)
            .ok_or_else(|| format!("Could not resolve value of operand '{left:?}'"))?;

        let right_val = self
            .get_operand_value(right)
            .ok_or_else(|| format!("Could not resolve value of operand '{right:?}'"))?;
        let value = Value::and(&left_val, &right_val)?;
        self.set_operand_value(&Operand::Register(ACC.to_string()), &value)
    }

    fn execute_or(&mut self, left: &Operand, right: &Operand) -> Result<(), String> {
        let left_val = self
            .get_operand_value(left)
            .ok_or_else(|| format!("Could not resolve value of operand '{left:?}'"))?;

        let right_val = self
            .get_operand_value(right)
            .ok_or_else(|| format!("Could not resolve value of operand '{right:?}'"))?;
        let value = Value::or(&left_val, &right_val)?;
        self.set_operand_value(&Operand::Register(ACC.to_string()), &value)
    }
    fn execute_xor(&mut self, left: &Operand, right: &Operand) -> Result<(), String> {
        let left_val = self
            .get_operand_value(left)
            .ok_or_else(|| format!("Could not resolve value of operand '{left:?}'"))?;

        let right_val = self
            .get_operand_value(right)
            .ok_or_else(|| format!("Could not resolve value of operand '{right:?}'"))?;
        let value = Value::xor(&left_val, &right_val)?;
        self.set_operand_value(&Operand::Register(ACC.to_string()), &value)
    }
    fn execute_not(&mut self, src: &Operand) -> Result<(), String> {
        let val = self
            .get_operand_value(src)
            .ok_or_else(|| format!("Could not resolve value of operand '{src:?}'"))?;
        let value = val.not()?;
        self.set_operand_value(&Operand::Register(ACC.to_string()), &value)
    }
    fn execute_jump(
        &mut self,
        label: &String,
        comparison: Option<&Comparison>,
    ) -> Result<(), String> {
        let idx = self
            .labels
            .get(label)
            .ok_or_else(|| format!("Label '{label}' does not exist"))?;

        if let Some(comparison) = comparison {
            let left = self
                .get_operand_value(&comparison.left)
                .ok_or_else(|| format!("Operand {:?} not found", comparison.left))?;
            let right = self
                .get_operand_value(&comparison.right)
                .ok_or_else(|| format!("Operand {:?} not found", comparison.right))?;
            println!("{left:?} = {right:?} ?");
            let result = Value::compare(&left, &right, &comparison.equality)?;
            if !result {
                let idx = self.pc.load(Ordering::SeqCst);
                self.pc.store(idx + 1, Ordering::SeqCst);
                return Ok(());
            }
        }
        self.pc.store(*idx, Ordering::SeqCst);
        Ok(())
    }

    fn execute_call(&mut self, label: &String) -> Result<(), String> {
        let idx = self
            .labels
            .get(label)
            .ok_or_else(|| format!("Label '{label}' does not exist"))?;
        let mut call_stack = self
            .call_stack
            .write()
            .map_err(|e| format!("Memory lock poisoned: {e}"))?;
        call_stack.push(self.pc.load(Ordering::SeqCst));
        self.pc.store(*idx, Ordering::SeqCst);
        Ok(())
    }

    fn execute_ret(&mut self) -> Result<(), String> {
        let mut call_stack = self
            .call_stack
            .write()
            .map_err(|e| format!("Memory lock poisoned: {e}"))?;

        println!("Call stack: {call_stack:?}");
        if let Some(addr) = call_stack.pop() {
            // return one more than where we started so we don't just call the "function"
            // infinitely
            self.pc.store(addr + 1, Ordering::SeqCst);
            Ok(())
        } else {
            Err("Cannot return from a function since the call stack is empty".to_string())
        }
    }

    fn execute_halt(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            self.running.store(false, Ordering::SeqCst);
        }
    }

    fn execute_push(&mut self, src: &Operand) -> Result<(), String> {
        let val = self.get_operand_value(src).unwrap_or_default();

        self.stack
            .write()
            .map_err(|e| format!("Memory lock Poisoned: {e}"))?
            .push(val);

        Ok(())
    }

    fn execute_pop(&mut self, dest: Option<&Operand>) -> Result<(), String> {
        let val = {
            let mut stack = self
                .stack
                .write()
                .map_err(|e| format!("Memory lock poisoned: {e}"))?;

            stack.pop()
        }
        .ok_or_else(|| "Stack is empty, cannot pop".to_string())?;

        if let Some(dest) = dest {
            self.set_operand_value(dest, &val)?;
        }
        Ok(())
    }

    fn execute_store(
        &mut self,
        register: &Operand,
        memory_address: &Operand,
    ) -> Result<(), String> {
        let val = match register {
            Operand::Register(name) => self
                .get_operand_value(register)
                .ok_or_else(|| format!("Invalid register '{name}'"))?,
            _ => return Err("Store must be 'STORE register, memory address'".to_string()),
        };
        self.set_operand_value(memory_address, &val)?;
        Ok(())
    }

    fn get_operand_value(&self, operand: &Operand) -> Option<Value> {
        match operand {
            Operand::Number(num) => convert_string_to_num(num).ok().map(Value::Number),
            Operand::Constant(name) => self.constants.get(name).cloned(),
            Operand::Character(c) => Some(Value::String(c.clone())),
            Operand::String(s) => Some(Value::String(s.clone())),
            Operand::Memory(s) => self.get_address(s).ok(),
            Operand::Register(r) | Operand::IndirectMemory(r) => self.get_register(r).ok(),
            Operand::Identifier(_) => None,
        }
    }

    fn set_operand_value(&mut self, operand: &Operand, value: &Value) -> Result<(), String> {
        match operand {
            Operand::Memory(address) => self.set_address(address, value.clone()),
            Operand::Register(register) | Operand::IndirectMemory(register) => {
                self.set_register(register, value.clone())
            }
            Operand::Constant(s) => Err(format!("Cannot change constant {s}")),
            Operand::Identifier(i) => Err(format!("Cannot set identifier {i}")),

            Operand::Number(_) | Operand::Character(_) | Operand::String(_) => {
                Err("Cannot set operand, invalid type".to_string())
            }
        }
    }

    fn get_address(&self, addr: &str) -> Result<Value, String> {
        let address = addr
            .strip_prefix('%')
            .ok_or_else(|| format!("Invalid address '{addr}'"))?;
        match convert_string_to_num(address) {
            Ok(address) => {
                let index: usize = address
                    .try_into()
                    .map_err(|_| format!("Negative memory address {address}"))?;
                if index >= RAM_SLOTS {
                    return Err(format!("Address '{index}' out of range"));
                }
                let memory = self
                    .memory
                    .read()
                    .map_err(|e| format!("Memory lock poisoned: {e}"))?;
                let value = memory[index].clone();
                Ok(value)
            }
            Err(e) => Err(format!("Invalid address: {address:?} - {e}")),
        }
    }

    fn set_address(&self, address: &str, value: Value) -> Result<(), String> {
        let mut address = address.strip_prefix('%').unwrap_or(address).to_string();
        let reg_address = match self.get_register(&address) {
            Ok(Value::Number(_)) => Some(self.get_register(&address)?),
            _ => None,
        };
        if let Some(Value::Number(new_address)) = reg_address {
            address = new_address.to_string();
        }

        match convert_string_to_num(&address) {
            Ok(address) => {
                let index: usize = address
                    .try_into()
                    .map_err(|_| format!("Negative memory address {address}"))?;
                if index >= RAM_SLOTS {
                    return Err(format!("Address '{index}' out of range"));
                }
                let mut memory = self
                    .memory
                    .write()
                    .map_err(|e| format!("Memory lock poisoned: {e}"))?;

                memory[index] = value;
                Ok(())
            }
            Err(e) => Err(format!("Invalid address: {address:?} - {e}")),
        }
    }
    fn get_register(&self, register: &str) -> Result<Value, String> {
        let registers = self
            .registers
            .read()
            .map_err(|e| format!("Memory lock Poisoned: {e}"))?;
        let reg = register.to_lowercase();
        if let Some(r) = reg.strip_prefix("%") {
            match registers.get(r) {
                Some(Value::Number(addr)) => self.get_address(&addr.to_string()),
                Some(_) => Err(format!(
                    "Register '{r}' does not hold a numeric value for indirect access",
                )),
                _ => Err(format!("'{r}' invalid")),
            }
        } else {
            match registers.get(&reg) {
                Some(r) => Ok(r.clone()),
                None => Err(format!("Register '{register}' does not exist")),
            }
        }
    }
    fn set_register(&self, register: &str, value: Value) -> Result<(), String> {
        let mut registers = self
            .registers
            .write()
            .map_err(|e| format!("Memory lock Poisoned: {e}"))?;
        let reg = register.to_lowercase();
        let exists = registers.get(&reg);
        if exists.is_some() {
            registers.insert(reg, value);
            Ok(())
        } else {
            Err(format!("Register '{reg}' does not exist"))
        }
    }
}

fn resolve_operand_compile(operand: &Operand, constants: &HashMap<String, Value>) -> Option<Value> {
    match operand {
        Operand::Number(num) => convert_string_to_num(num).ok().map(Value::Number),
        Operand::Constant(name) => constants.get(name).cloned(),
        Operand::Character(c) => Some(Value::String(c.clone())),
        Operand::String(s) => Some(Value::String(s.clone())),
        Operand::Register(_)
        | Operand::Memory(_)
        | Operand::Identifier(_)
        | Operand::IndirectMemory(_) => None,
    }
}

fn convert_string_to_num(input: &str) -> Result<i64, ParseIntError> {
    let input = input.to_lowercase().trim().to_string();
    if let Some(hex) = input.strip_prefix("0x") {
        i64::from_str_radix(hex, 16)
    } else if let Some(binary) = input.strip_prefix("0b") {
        i64::from_str_radix(binary, 2)
    } else {
        input.parse::<i64>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast;

    fn set_reg(interpreter: &mut Interpreter, name: &str, value: Value) {
        interpreter
            .execute_set(value, &ast::Operand::Register(name.to_string()))
            .unwrap();
    }

    fn get_reg(interpreter: &Interpreter, name: &str) -> Option<Value> {
        interpreter.registers.read().unwrap().get(name).cloned()
    }

    fn set_mem(interpreter: &mut Interpreter, addr: &str, value: Value) {
        interpreter
            .execute_set(value, &ast::Operand::Memory(addr.to_string()))
            .unwrap();
    }

    fn get_mem(interpreter: &Interpreter, addr: usize) -> Option<Value> {
        interpreter.memory.read().unwrap().get(addr).cloned()
    }

    fn move_reg_to_reg(interpreter: &mut Interpreter, left: &str, right: &str) {
        interpreter
            .execute_move(
                &ast::Operand::Register(left.to_string()),
                &ast::Operand::Register(right.to_string()),
            )
            .unwrap();
    }
    fn move_reg_to_mem(interpreter: &mut Interpreter, left: &str, right: &str) {
        interpreter
            .execute_move(
                &ast::Operand::Register(left.to_string()),
                &ast::Operand::Memory(right.to_string()),
            )
            .unwrap();
    }

    fn clear_reg(interpreter: &mut Interpreter, reg: &str) {
        interpreter
            .execute_clear(&ast::Operand::Register(reg.to_string()))
            .unwrap();
    }
    fn clear_mem(interpreter: &mut Interpreter, addr: &str) {
        interpreter
            .execute_clear(&ast::Operand::Memory(addr.to_string()))
            .unwrap();
    }
    #[test]
    fn test_convert_string_to_num() {
        let cases = vec![
            ("42", Ok(42)),
            ("-42", Ok(-42)),
            ("0x2a", Ok(42)),
            ("0X2A", Ok(42)),
            ("0b101010", Ok(42)),
            ("0B101010", Ok(42)),
            ("abc", Err(())),
            ("0xg", Err(())),
        ];
        for (input, expected) in cases {
            match expected {
                Ok(val) => {
                    assert_eq!(convert_string_to_num(input).unwrap(), val, "input: {input}");
                }
                _ => assert!(convert_string_to_num(input).is_err(), "input: {input}"),
            }
        }
    }
    #[test]
    fn test_set_and_get_and_clear_register() {
        let mut interpreter = Interpreter::new();
        set_reg(&mut interpreter, "r2", Value::Number(42));

        let v = get_reg(&interpreter, "r2");
        assert_eq!(v, Some(Value::Number(42)));

        set_reg(&mut interpreter, "r2", Value::String("abc".to_string()));
        let v = get_reg(&interpreter, "r2");
        assert_eq!(v, Some(Value::String("abc".to_string())));

        clear_reg(&mut interpreter, "r2");
        assert_eq!(get_reg(&interpreter, "r2").unwrap(), Value::default());
    }
    #[test]
    fn test_set_and_get_and_clear_memory() {
        let mut interpreter = Interpreter::new();
        set_mem(&mut interpreter, "%100", Value::String("abc".to_string()));
        let v = get_mem(&interpreter, 100);
        assert_eq!(v, Some(Value::String("abc".to_string())));

        set_mem(&mut interpreter, "%100", Value::Number(100));
        let v = get_mem(&interpreter, 100);
        assert_eq!(v, Some(Value::Number(100)));

        clear_mem(&mut interpreter, "%100");
        assert_eq!(get_mem(&interpreter, 100).unwrap(), Value::default());
    }

    #[test]
    fn test_execute_move_registers() {
        let mut interpreter = Interpreter::new();
        set_reg(&mut interpreter, "r2", Value::Number(100));
        move_reg_to_reg(&mut interpreter, "r2", "r4");
        let v = get_reg(&interpreter, "r4");
        assert_eq!(v, Some(Value::Number(100)));

        set_reg(&mut interpreter, "r2", Value::String("abc".to_string()));
        move_reg_to_reg(&mut interpreter, "r2", "r4");
        let v = get_reg(&interpreter, "r4");
        assert_eq!(v, Some(Value::String("abc".to_string())));
    }
    #[test]
    fn test_execute_register_math() {
        let mut interpreter = Interpreter::new();
        set_reg(&mut interpreter, "r4", Value::Number(-50));
        set_reg(&mut interpreter, "r2", Value::Number(100));

        interpreter
            .execute_add(
                &ast::Operand::Register("r4".to_string()),
                &ast::Operand::Register("r2".to_string()),
            )
            .unwrap();
        assert_eq!(get_reg(&interpreter, "a"), Some(Value::Number(50)));

        interpreter
            .execute_sub(
                &ast::Operand::Register("r4".to_string()),
                &ast::Operand::Register("r2".to_string()),
            )
            .unwrap();
        assert_eq!(get_reg(&interpreter, "a"), Some(Value::Number(-150)));

        interpreter
            .execute_mul(
                &ast::Operand::Register("r4".to_string()),
                &ast::Operand::Register("r2".to_string()),
            )
            .unwrap();
        assert_eq!(get_reg(&interpreter, "a"), Some(Value::Number(-5000)));

        set_reg(&mut interpreter, "r4", Value::Number(100));
        set_reg(&mut interpreter, "r2", Value::Number(50));

        interpreter
            .execute_div(
                &ast::Operand::Register("r4".to_string()),
                &ast::Operand::Register("r2".to_string()),
            )
            .unwrap();
        assert_eq!(get_reg(&interpreter, "a"), Some(Value::Number(2)));

        set_reg(&mut interpreter, "r4", Value::Number(0b1111));
        set_reg(&mut interpreter, "r2", Value::Number(0b0010));

        interpreter
            .execute_and(
                &ast::Operand::Register("r4".to_string()),
                &ast::Operand::Register("r2".to_string()),
            )
            .unwrap();
        assert_eq!(get_reg(&interpreter, "a"), Some(Value::Number(2)));

        interpreter
            .execute_or(
                &ast::Operand::Register("r4".to_string()),
                &ast::Operand::Register("r2".to_string()),
            )
            .unwrap();
        assert_eq!(get_reg(&interpreter, "a"), Some(Value::Number(15)));

        interpreter
            .execute_xor(
                &ast::Operand::Register("r4".to_string()),
                &ast::Operand::Register("r2".to_string()),
            )
            .unwrap();
        assert_eq!(get_reg(&interpreter, "a"), Some(Value::Number(0b1101)));

        interpreter
            .execute_not(&ast::Operand::Register("r2".to_string()))
            .unwrap();
        assert_eq!(get_reg(&interpreter, "a"), Some(Value::Number(-3)));
    }
    #[test]
    fn test_execute_add_register_strings() {
        let mut interpreter = Interpreter::new();
        set_reg(&mut interpreter, "r4", Value::String("abc".to_string()));
        set_reg(&mut interpreter, "r2", Value::String("def".to_string()));

        interpreter
            .execute_add(
                &ast::Operand::Register("r4".to_string()),
                &ast::Operand::Register("r2".to_string()),
            )
            .unwrap();
        assert_eq!(
            get_reg(&interpreter, "a"),
            Some(Value::String("abcdef".to_string()))
        );
    }
    #[test]
    fn test_execute_add_memory_numbers() {
        let mut interpreter = Interpreter::new();
        set_mem(&mut interpreter, "%0xff", Value::Number(-50));
        set_mem(&mut interpreter, "%250", Value::Number(100));

        interpreter
            .execute_add(
                &ast::Operand::Memory("%250".to_string()),
                &ast::Operand::Memory("%0xFF".to_string()),
            )
            .unwrap();
        assert_eq!(get_reg(&interpreter, "a"), Some(Value::Number(50)));
    }
    #[test]
    fn test_execute_add_memory_strings() {
        let mut interpreter = Interpreter::new();
        set_mem(
            &mut interpreter,
            "%0xff",
            Value::String("hello,".to_string()),
        );
        set_mem(
            &mut interpreter,
            "%250",
            Value::String(" there".to_string()),
        );

        interpreter
            .execute_add(
                &ast::Operand::Memory("%0xFF".to_string()),
                &ast::Operand::Memory("%250".to_string()),
            )
            .unwrap();
        assert_eq!(
            get_reg(&interpreter, "a"),
            Some(Value::String("hello, there".to_string()))
        );
    }
    #[test]
    fn test_string_math_operators() {
        let mut interpreter = Interpreter::new();
        set_reg(&mut interpreter, "r1", Value::String("Hi".to_string()));
        interpreter
            .execute_mul(
                &ast::Operand::Register("r1".to_string()),
                &ast::Operand::Number("3".to_string()),
            )
            .unwrap();
        assert_eq!(
            get_reg(&interpreter, "a"),
            Some(Value::String("HiHiHi".to_string()))
        );

        interpreter
            .execute_div(
                &ast::Operand::Register("r1".to_string()),
                &ast::Operand::String("hello".to_string()),
            )
            .unwrap();
        assert_eq!(get_reg(&interpreter, "a"), Some(Value::Number(-3)));
    }
}
