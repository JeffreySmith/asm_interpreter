#![deny(clippy::pedantic)]

use crate::ast::{Comparison, ComparisonOp, Instruction, Operand, Statement};
use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct ASMParser;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken,
    Invalid,
}

fn next_operand<'a, I>(inner: &mut I) -> Operand
where
    I: Iterator<Item = pest::iterators::Pair<'a, Rule>>,
{
    operand_from_pair(inner.next().expect("Expected operand"))
}

fn operand_from_pair(pair: Pair<Rule>) -> Operand {
    match pair.as_rule() {
        Rule::OPERAND => {
            let inner = pair.into_inner().next().expect("OPERAND should exist");
            operand_from_pair(inner)
        }
        Rule::REGISTER => Operand::Register(pair.as_str().to_string()),
        Rule::MEMORYADDRESS => Operand::Memory(pair.as_str().to_string()),
        Rule::NUMBER => Operand::Number(pair.as_str().to_string()),
        Rule::IDENTIFIER => Operand::Identifier(pair.as_str().to_string()),
        Rule::STRING => Operand::String(pair.as_str().to_string()),
        Rule::CHARACTER => Operand::Character(pair.as_str().to_string()),
        Rule::CONSTANT => Operand::Constant(pair.as_str().to_string()),
        _ => {
            panic!("Unknown operand type: {:?}", pair.as_rule())
        }
    }
}
fn comparison_from_pair(pair: Pair<Rule>) -> Comparison {
    let mut inner = pair.into_inner();
    let left = operand_from_pair(inner.next().expect("Expected left for equality"));

    let op_pair = inner.next().expect("Missing comparison");
    let equality = match op_pair.as_str() {
        "=" => ComparisonOp::Eq,
        "!=" => ComparisonOp::Ne,
        "<" => ComparisonOp::Lt,
        "<=" => ComparisonOp::Le,
        ">" => ComparisonOp::Gt,
        ">=" => ComparisonOp::Ge,
        _ => panic!("Unknown comparison"),
    };

    let right = operand_from_pair(inner.next().expect("Expected right for equality"));

    Comparison {
        left,
        equality,
        right,
    }
}

pub fn statement_from_pair(pair: &Pair<Rule>) -> Statement {
    let mut inner = pair.clone().into_inner();
    match pair.as_rule() {
        Rule::LABEL => Statement::Label(pair.as_str().trim_end_matches(':').to_string()),
        Rule::DEFINE => {
            let name = inner.next().unwrap().as_str().to_string();
            let value = next_operand(&mut inner);
            Statement::CompileTime(Instruction::Define { name, value })
        }
        Rule::SET => {
            let dest = next_operand(&mut inner);
            let value = next_operand(&mut inner);
            Statement::Instruction(Instruction::Set { value, dest })
        }
        Rule::LOAD => {
            let src = next_operand(&mut inner);
            let dest = next_operand(&mut inner);
            Statement::Instruction(Instruction::Load { src, dest })
        }
        Rule::CLEAR => {
            let target = next_operand(&mut inner);
            Statement::Instruction(Instruction::Clear { target })
        }

        Rule::MOVE => {
            let src = next_operand(&mut inner);
            let dest = next_operand(&mut inner);
            Statement::Instruction(Instruction::Mov { src, dest })
        }

        Rule::OPPUSH => {
            let src = next_operand(&mut inner);
            Statement::Instruction(Instruction::Push { src })
        }
        Rule::OPPOP => {
            let dest = inner.next().map(|pair| operand_from_pair(pair));
            Statement::Instruction(Instruction::Pop { dest })
        }

        Rule::ADD => {
            let left = next_operand(&mut inner);
            let right = next_operand(&mut inner);
            Statement::Instruction(Instruction::Add { left, right })
        }
        Rule::SUB => {
            let left = next_operand(&mut inner);
            let right = next_operand(&mut inner);
            Statement::Instruction(Instruction::Sub { left, right })
        }
        Rule::MUL => {
            let left = next_operand(&mut inner);
            let right = next_operand(&mut inner);
            Statement::Instruction(Instruction::Mul { left, right })
        }
        Rule::DIV => {
            let left = next_operand(&mut inner);
            let right = next_operand(&mut inner);
            Statement::Instruction(Instruction::Div { left, right })
        }
        Rule::INC => {
            let dest = next_operand(&mut inner);
            Statement::Instruction(Instruction::Inc { dest })
        }
        Rule::DEC => {
            let dest = next_operand(&mut inner);
            Statement::Instruction(Instruction::Dec { dest })
        }
        Rule::AND => {
            let left = next_operand(&mut inner);
            let right = next_operand(&mut inner);
            Statement::Instruction(Instruction::And { left, right })
        }
        Rule::OR => {
            let left = next_operand(&mut inner);
            let right = next_operand(&mut inner);
            Statement::Instruction(Instruction::Or { left, right })
        }
        Rule::XOR => {
            let left = next_operand(&mut inner);
            let right = next_operand(&mut inner);
            Statement::Instruction(Instruction::Xor { left, right })
        }
        Rule::NOT => {
            let op = next_operand(&mut inner);
            Statement::Instruction(Instruction::Not { op })
        }

        Rule::JUMP => {
            let target = next_operand(&mut inner);
            let comparison = inner.next().map(comparison_from_pair);
            Statement::Instruction(Instruction::Jmp { target, comparison })
        }
        Rule::CALL => {
            let target = next_operand(&mut inner);
            Statement::Instruction(Instruction::Call { target })
        }
        Rule::RET => Statement::Instruction(Instruction::Ret),
        Rule::HALT => Statement::Instruction(Instruction::Halt),

        _ => unimplemented!(),
    }
}

pub fn parse_program(contents: &str) -> Result<Vec<Statement>, ParseError> {
    let mut statements: Vec<Statement> = Vec::new();
    let parse_results = ASMParser::parse(Rule::program, contents);
    if let Ok(pairs) = parse_results {
        for pair in pairs {
            if pair.as_rule() == Rule::EOI {
                continue;
            }
            let statement = statement_from_pair(&pair);
            statements.push(statement);
        }
    } else {
        return Err(ParseError::Invalid);
    }
    Ok(statements)
}
