use crate::ast::{Comparison, ComparisonOp, Instruction, Operand, Statement};
use std::convert::TryInto;
use std::{collections::HashMap, num::ParseIntError};

pub const RAM_BYTES: usize = 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Number(i64),
    String(String),
}

impl Value {
    pub fn compare(&self, other: &Value, op: &ComparisonOp) -> Result<bool, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(op.compare(a.cmp(b))),
            (Value::String(a), Value::String(b)) => Ok(op.compare(a.cmp(b))),
            _ => Err(format!("Invalid comparison between {self:?} and {other:?}")),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Number(0)
    }
}

pub struct Interpreter {
    display: Vec<(i32, i32, i32)>,
    pub registers: HashMap<String, Value>,
    pub memory: Vec<Value>,
    stack: Vec<Value>,
    pub labels: HashMap<String, usize>,
    pub constants: HashMap<String, Value>,
    pc: usize,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut registers: HashMap<String, Value> = HashMap::new();
        for reg in &["a", "f", "r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7"] {
            registers.insert(reg.to_string(), Value::default());
        }
        Interpreter {
            display: Vec::new(),
            registers,
            memory: vec![Value::Number(0); RAM_BYTES],
            stack: Vec::new(),
            labels: HashMap::new(),
            constants: HashMap::new(),
            pc: 0,
        }
    }
    pub fn check(&mut self, statements: &[Statement]) {
        for (i, statement) in statements.iter().enumerate() {
            match statement {
                Statement::CompileTime(instr) => match instr {
                    Instruction::Define { name, value } => {
                        let resolved = resolve_operand_compile(value, &self.constants);
                        if let Some(val) = resolved {
                            _ = self.constants.insert(name.clone(), val);
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
        println!("{}", self.pc);
        println!("{stmt:?}");
    }

    pub fn execute_set(&mut self, value: Value, dest: &Operand) -> Result<(), String> {
        match dest {
            Operand::Register(name) => {
                self.registers
                    .insert(name.clone(), value)
                    .ok_or_else(|| format!("Unable to set {name:?}"))?;
                Ok(())
            }
            Operand::Memory(address) => self.set_address(address, value),
            _ => Err(format!("Invalid operand for set: {dest:?}")),
        }
    }

    pub fn execute_load(
        &mut self,
        memory_address: &Operand,
        register: &Operand,
    ) -> Result<(), String> {
        if let (Operand::Memory(address), Operand::Register(register)) = (memory_address, register)
        {
            let value = self.get_address(address)?;
            self.registers
                .insert(register.clone(), value)
                .ok_or_else(|| format!("Unable to load into {register:?}"))?;
            Ok(())
        } else {
            Err("Invalid operands used in LOAD".to_string())
        }
    }

    pub fn execute_move(&mut self, src: &Operand, dest: &Operand) -> Result<(), String> {
        let value = self
            .get_operand_value(src)
            .ok_or_else(|| format!("Could not resolve value of operand '{src:?}'"))?;

        self.set_operand_value(dest, &value)?;
        Ok(())
    }

    pub fn execute_jump(
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
        self.pc = *idx;
        Ok(())
    }

    fn get_operand_value(&self, operand: &Operand) -> Option<Value> {
        match operand {
            Operand::Number(num) => convert_string_to_num(num).ok().map(Value::Number),
            Operand::Constant(name) => self.constants.get(name).cloned(),
            Operand::Character(c) => Some(Value::String(c.clone())),
            Operand::String(s) => Some(Value::String(s.clone())),
            Operand::Memory(s) => self.get_address(s).ok(),
            Operand::Register(r) => self.registers.get(r).cloned(),
            Operand::Identifier(_) => None,
        }
    }

    fn set_operand_value(&mut self, operand: &Operand, value: &Value) -> Result<(), String> {
        match operand {
            Operand::Memory(address) => self.set_address(address, value.clone()),
            Operand::Register(register) => {
                self.registers.insert(register.clone(), value.clone());
                Ok(())
            }
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
                if index >= RAM_BYTES {
                    return Err(format!("Address '{index}' out of range"));
                }

                let value = self.memory[index].clone();
                Ok(value)
            }
            Err(e) => Err(format!("Invalid address: {address:?} - {e}")),
        }
    }

    fn set_address(&mut self, address: &str, value: Value) -> Result<(), String> {
        match convert_string_to_num(address) {
            Ok(address) => {
                let index: usize = address
                    .try_into()
                    .map_err(|_| format!("Negative memory address {address}"))?;
                if index >= RAM_BYTES {
                    return Err(format!("Address '{index}' out of range"));
                }

                self.memory[index] = value;
                Ok(())
            }
            Err(e) => Err(format!("Invalid address: {address:?} - {e}")),
        }
    }

    /*fn compare_operand(&self, left: &Operand, right: &Operand) -> Result<ComparisonOp, String> {
        let left_val = self.get_operand_value(left);
        let right_val = self.get_operand_value(right);
        match (left_val, right_val) {
            (Some(left), Some(right)) => {
                Interpreter::compare_value(left, right).ok_or("No valid comparison".to_string())
            }
            (Some(_), None) => Err("Right operand not valid".to_string()),
            (None, Some(_)) => Err("Left operand not valid".to_string()),
            (None, None) => Err("No valid operands".to_string()),
        }
    }

    fn compare_value(left: Value, right: Value) -> Option<ComparisonOp> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Some(a.cmp(&b)),
            (Value::String(a), Value::String(b)) => Some(a.cmp(&b)),
            _ => None,
        }
    }
    */
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
