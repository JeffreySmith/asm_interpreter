/*BSD 3-Clause License

Copyright (c) 2025, Jeffrey Smith

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its
   contributors may be used to endorse or promote products derived from
   this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

#![deny(clippy::pedantic)]

use crate::ast::{Comparison, ComparisonOp, Instruction, Operand, Statement};
use crate::ast_builder;
use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "asm.pest"]
pub struct ASMParser;

fn next_operand<'a, I>(inner: &mut I) -> Operand
where
    I: Iterator<Item = pest::iterators::Pair<'a, Rule>>,
{
    operand_from_pair(inner.next().expect("Expected operand")).expect("")
}

fn operand_from_pair(pair: Pair<Rule>) -> Result<Operand, String> {
    match pair.as_rule() {
        Rule::OPERAND => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or("Couldn't find any more operands")?;
            operand_from_pair(inner)
        }
        Rule::REGISTER => Ok(Operand::Register(pair.as_str().to_string())),
        Rule::MEMORYADDRESS => Ok(Operand::Memory(pair.as_str().to_string())),
        Rule::INDIRECTADDRESS => {
            let name = pair.as_str();
            let parsed_name = name
                .strip_prefix('%')
                .ok_or_else(|| format!("'{name}' is not a valid indirect address"))?;
            Ok(Operand::Register(parsed_name.to_string()))
        }
        Rule::NUMBER => Ok(Operand::Number(pair.as_str().to_string())),
        Rule::IDENTIFIER => Ok(Operand::Identifier(pair.as_str().to_string())),
        Rule::STRING => Ok(Operand::String(pair.as_str().to_string())),
        Rule::CHARACTER => Ok(Operand::Character(pair.as_str().to_string())),
        Rule::CONSTANT => Ok(Operand::Constant(pair.as_str().to_string())),
        _ => Err(format!("Unknown operand type: {:?}", pair.as_rule())),
    }
}
fn comparison_from_pair(pair: Pair<Rule>) -> Result<Comparison, String> {
    let mut inner = pair.into_inner();
    let left = operand_from_pair(inner.next().ok_or("Expected left for equality")?)?;

    let op_pair = inner.next().ok_or("Expected equality")?;
    let equality = match op_pair.as_str() {
        "=" => ComparisonOp::Eq,
        "!=" => ComparisonOp::Ne,
        "<" => ComparisonOp::Lt,
        "<=" => ComparisonOp::Le,
        ">" => ComparisonOp::Gt,
        ">=" => ComparisonOp::Ge,
        _ => panic!("Unknown comparison"),
    };

    let right = operand_from_pair(inner.next().ok_or("Expected right for equality")?)?;

    Ok(Comparison {
        left,
        equality,
        right,
    })
}

#[allow(clippy::too_many_lines)]
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
            let dest = inner.next().map(|pair| operand_from_pair(pair).ok());
            if let Some(Some(_)) = dest {
                Statement::Instruction(Instruction::Pop {
                    dest: dest.unwrap(),
                })
            } else {
                Statement::Instruction(Instruction::Pop { dest: None })
            }
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
            if let Some(Ok(_)) = comparison {
                Statement::Instruction(Instruction::Jmp {
                    target,
                    comparison: comparison.unwrap().ok(),
                })
            } else {
                Statement::Instruction(Instruction::Jmp {
                    target,
                    comparison: None,
                })
            }
        }
        Rule::CALL => {
            let target = next_operand(&mut inner);
            Statement::Instruction(Instruction::Call { target })
        }
        Rule::RET => Statement::Instruction(Instruction::Ret),
        Rule::HALT => Statement::Instruction(Instruction::Halt),
        Rule::STORE => {
            let value = next_operand(&mut inner);
            let dest = next_operand(&mut inner);
            Statement::Instruction(Instruction::Store { value, dest })
        }
        _ => unimplemented!(),
    }
}

pub fn parse_program<T: AsRef<str>>(
    contents: T,
) -> Result<Vec<Statement>, Box<pest::error::Error<ast_builder::Rule>>> {
    let mut statements: Vec<Statement> = Vec::new();
    let parse_results = ASMParser::parse(Rule::program, contents.as_ref());
    match parse_results {
        Ok(pairs) => {
            for pair in pairs {
                if pair.as_rule() == Rule::EOI {
                    continue;
                }
                let statement = statement_from_pair(&pair);
                statements.push(statement);
            }
        }
        Err(e) => return Err(Box::new(e)),
    }
    Ok(statements)
}
