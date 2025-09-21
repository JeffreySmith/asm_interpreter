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

use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Statement {
    Label(String),
    CompileTime(Instruction),
    Instruction(Instruction),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Instruction {
    Define {
        name: String,
        value: Operand,
    },
    Set {
        value: Operand,
        dest: Operand,
    },
    Load {
        src: Operand,
        dest: Operand,
    },
    Store {
        value: Operand,
        dest: Operand,
    },
    Clear {
        target: Operand,
    },

    Add {
        left: Operand,
        right: Operand,
    },
    Sub {
        left: Operand,
        right: Operand,
    },
    Mul {
        left: Operand,
        right: Operand,
    },
    Div {
        left: Operand,
        right: Operand,
    },
    Inc {
        dest: Operand,
    },
    Dec {
        dest: Operand,
    },

    Mov {
        src: Operand,
        dest: Operand,
    },
    Push {
        src: Operand,
    },
    Pop {
        dest: Option<Operand>,
    },

    Jmp {
        target: Operand,
        comparison: Option<Comparison>,
    },
    Call {
        target: Operand,
    },

    And {
        left: Operand,
        right: Operand,
    },
    Or {
        left: Operand,
        right: Operand,
    },
    Xor {
        left: Operand,
        right: Operand,
    },
    Not {
        op: Operand,
    },

    Ret,
    Halt,
}

impl AsRef<Instruction> for Instruction {
    fn as_ref(&self) -> &Instruction {
        self
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::Define { name, value } => write!(f, "DEFINE {name} {value}"),
            Instruction::Set { value, dest } => write!(f, "SET {value}, {dest}"),
            Instruction::Load { src, dest } => write!(f, "LOAD {src}, {dest}"),
            Instruction::Store { value, dest } => write!(f, "STORE {value}, {dest}"),
            Instruction::Clear { target } => write!(f, "CLEAR {target}"),
            Instruction::Add { left, right } => write!(f, "ADD {left}, {right}"),
            Instruction::Sub { left, right } => write!(f, "SUB {left}, {right}"),
            Instruction::Mul { left, right } => write!(f, "MUL {left}, {right}"),
            Instruction::Div { left, right } => write!(f, "DIV {left}, {right}"),
            Instruction::Inc { dest } => write!(f, "INC {dest}"),
            Instruction::Dec { dest } => write!(f, "DEC {dest}"),
            Instruction::Mov { src, dest } => write!(f, "MOV {src}, {dest}"),
            Instruction::Push { src } => write!(f, "PUSH {src}"),
            Instruction::Pop { dest } => {
                if let Some(op) = dest {
                    write!(f, "POP {op}")
                } else {
                    write!(f, "POP")
                }
            }
            Instruction::Jmp { target, comparison } => {
                if let Some(comp) = comparison {
                    write!(f, "JMP {target} {comp}")
                } else {
                    write!(f, "JMP {target}")
                }
            }
            Instruction::Call { target } => write!(f, "CALL {target}"),
            Instruction::And { left, right } => write!(f, "AND {left}, {right}"),
            Instruction::Or { left, right } => write!(f, "OR {left}, {right}"),
            Instruction::Xor { left, right } => write!(f, "XOR {left}, {right}"),
            Instruction::Not { op } => write!(f, "NOT {op}"),
            Instruction::Ret => write!(f, "RET"),
            Instruction::Halt => write!(f, "HALT"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Operand {
    Register(String),
    Memory(String),
    #[allow(dead_code)]
    IndirectMemory(String),
    Number(String),
    Identifier(String),
    Constant(String),
    Character(String),
    String(String),
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Register(s)
            | Operand::Memory(s)
            | Operand::IndirectMemory(s)
            | Operand::Number(s)
            | Operand::Identifier(s)
            | Operand::Constant(s)
            | Operand::Character(s)
            | Operand::String(s) => write!(f, "{s}"),
        }
    }
}

impl AsRef<Operand> for Operand {
    fn as_ref(&self) -> &Operand {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Comparison {
    pub left: Operand,
    pub equality: ComparisonOp,
    pub right: Operand,
}

impl fmt::Display for Comparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.equality, self.right)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum ComparisonOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
impl ComparisonOp {
    pub fn compare(&self, ordering: Ordering) -> bool {
        match self {
            ComparisonOp::Eq => ordering == Ordering::Equal,
            ComparisonOp::Ne => ordering != Ordering::Equal,
            ComparisonOp::Lt => ordering == Ordering::Less,
            ComparisonOp::Le => ordering != Ordering::Greater,
            ComparisonOp::Gt => ordering == Ordering::Greater,
            ComparisonOp::Ge => ordering != Ordering::Less,
        }
    }
}
impl fmt::Display for ComparisonOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOp::Eq => write!(f, "="),
            ComparisonOp::Ne => write!(f, "!="),
            ComparisonOp::Lt => write!(f, "<"),
            ComparisonOp::Le => write!(f, "<="),
            ComparisonOp::Gt => write!(f, ">"),
            ComparisonOp::Ge => write!(f, ">="),
        }
    }
}
