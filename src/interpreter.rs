use crate::Value;
use crate::ast::{Comparison, Instruction, Operand, Statement};
use std::convert::TryInto;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::{collections::HashMap, num::ParseIntError, sync::Arc, sync::RwLock};

const RAM_SLOTS: usize = 256;
const DISPLAY_PIXELS: usize = 16 * 16;
const ACC: &str = "a";

const REGISTERS: [&str; 10] = ["a", "f", "r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7"];

pub struct Interpreter {
    display: Arc<RwLock<Vec<(i32, i32, i32)>>>,
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
    pub fn check(&mut self, statements: &[Statement]) {
        for (i, statement) in statements.iter().enumerate() {
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

    pub fn execute(&mut self, stmt: &Statement) {
        println!("{}", self.pc.load(Ordering::SeqCst));
        println!("{stmt:?}");
    }

    fn execute_set(&mut self, value: Value, dest: &Operand) -> Result<(), String> {
        match dest {
            Operand::Register(name) => self.set_register(name, value),
            Operand::Memory(address) => self.set_address(address, value),
            _ => Err(format!("Invalid operand for set: {dest:?}")),
        }
    }

    fn execute_load(&mut self, memory_address: &Operand, register: &Operand) -> Result<(), String> {
        if let (Operand::Memory(address), Operand::Register(name)) = (memory_address, register) {
            let value = self.get_address(address)?;
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
        label: Statement,
        comparison: Option<Comparison>,
    ) -> Result<(), String> {
        let name = match label {
            Statement::Label(name) => Ok(name),
            _ => Err(format!("Invalid statement type '{label:?}")),
        }?;

        let idx = self
            .labels
            .get(&name)
            .ok_or_else(|| format!("Label '{name}' does not exist"))?;

        if let Some(comparison) = comparison {
            let left = self
                .get_operand_value(&comparison.left)
                .ok_or_else(|| format!("Operand {:?} not found", comparison.left))?;
            let right = self
                .get_operand_value(&comparison.right)
                .ok_or_else(|| format!("Operand {:?} not found", comparison.right))?;

            let result = Value::compare(&left, &right, &comparison.equality)?;
            if !result {
                return Ok(());
            }
        }
        self.pc.store(*idx, Ordering::SeqCst);
        Ok(())
    }

    fn execute_call(&mut self, label: Statement) -> Result<(), String> {
        let name = match label {
            Statement::Label(name) => Ok(name),
            _ => Err(format!("Invalid statement type '{label:?}'")),
        }?;
        let idx = self
            .labels
            .get(&name)
            .ok_or_else(|| format!("Label '{name}' does not exist"))?;
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

        if let Some(addr) = call_stack.pop() {
            self.pc.store(addr, Ordering::SeqCst);
            Ok(())
        } else {
            Err("Cannot return from a function since the call stack is empty".to_string())
        }
    }

    fn execute_halt(&mut self) -> bool {
        if self.running.load(Ordering::SeqCst) {
            self.running.store(false, Ordering::SeqCst);
            return true;
        }
        false
    }

    fn get_operand_value(&self, operand: &Operand) -> Option<Value> {
        match operand {
            Operand::Number(num) => convert_string_to_num(num).ok().map(Value::Number),
            Operand::Constant(name) => self.constants.get(name).cloned(),
            Operand::Character(c) => Some(Value::String(c.clone())),
            Operand::String(s) => Some(Value::String(s.clone())),
            Operand::Memory(s) => self.get_address(s).ok(),
            Operand::Register(r) => self.get_register(r).ok(),
            Operand::Identifier(_) => None,
        }
    }

    fn set_operand_value(&mut self, operand: &Operand, value: &Value) -> Result<(), String> {
        match operand {
            Operand::Memory(address) => self.set_address(address, value.clone()),
            Operand::Register(register) => self.set_register(register, value.clone()),
            Operand::Constant(s) => Err(format!("Cannot change constant {s}")),
            Operand::Identifier(i) => Err(format!("Cannot set identifier {i}")),

            Operand::Number(_) | Operand::Character(_) | Operand::String(_) => {
                Err("Cannot set operand, invalid type".to_string())
            }
        }
    }

    fn get_address(&self, address: &str) -> Result<Value, String> {
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
        match convert_string_to_num(address) {
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
        match registers.get(register) {
            Some(r) => Ok(r.clone()),
            None => Err(format!("Register '{register}' does not exist")),
        }
    }
    fn set_register(&self, register: &str, value: Value) -> Result<(), String> {
        let mut registers = self
            .registers
            .write()
            .map_err(|e| format!("Memory lock Poisoned: {e}"))?;
        let exists = registers.get(register);
        if exists.is_some() {
            registers.insert(register.to_string(), value);
            Ok(())
        } else {
            Err(format!("Register '{register}' does not exist"))
        }
    }
}

fn resolve_operand_compile(operand: &Operand, constants: &HashMap<String, Value>) -> Option<Value> {
    match operand {
        Operand::Number(num) => convert_string_to_num(num).ok().map(Value::Number),
        Operand::Constant(name) => constants.get(name).cloned(),
        Operand::Character(c) => Some(Value::String(c.clone())),
        Operand::String(s) => Some(Value::String(s.clone())),
        Operand::Register(_) | Operand::Memory(_) | Operand::Identifier(_) => None,
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
    fn test_set_and_get_register() {
        let mut interpreter = Interpreter::new();
        set_reg(&mut interpreter, "r2", Value::Number(42));

        let v = get_reg(&interpreter, "r2");
        assert_eq!(v, Some(Value::Number(42)));

        set_reg(&mut interpreter, "r2", Value::String("abc".to_string()));
        let v = get_reg(&interpreter, "r2");
        assert_eq!(v, Some(Value::String("abc".to_string())));
    }
    #[test]
    fn test_set_and_get_memory() {
        let mut interpreter = Interpreter::new();
        set_mem(&mut interpreter, "100", Value::String("abc".to_string()));
        let v = get_mem(&interpreter, 100);
        assert_eq!(v, Some(Value::String("abc".to_string())));

        set_mem(&mut interpreter, "100", Value::Number(100));
        let v = get_mem(&interpreter, 100);
        assert_eq!(v, Some(Value::Number(100)));
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
        set_mem(&mut interpreter, "0xff", Value::Number(-50));
        set_mem(&mut interpreter, "250", Value::Number(100));

        interpreter
            .execute_add(
                &ast::Operand::Memory("250".to_string()),
                &ast::Operand::Memory("0xFF".to_string()),
            )
            .unwrap();
        assert_eq!(get_reg(&interpreter, "a"), Some(Value::Number(50)));
    }
    #[test]
    fn test_execute_add_memory_strings() {
        let mut interpreter = Interpreter::new();
        set_mem(
            &mut interpreter,
            "0xff",
            Value::String("hello,".to_string()),
        );
        set_mem(&mut interpreter, "250", Value::String(" there".to_string()));

        interpreter
            .execute_add(
                &ast::Operand::Memory("0xFF".to_string()),
                &ast::Operand::Memory("250".to_string()),
            )
            .unwrap();
        assert_eq!(
            get_reg(&interpreter, "a"),
            Some(Value::String("hello, there".to_string()))
        );
    }
}
