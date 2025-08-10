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

#[derive(Debug, Clone)]
pub enum Statement {
    Label(String),
    CompileTime(Instruction),
    Instruction(Instruction),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum Operand {
    Register(String),
    Memory(String),
    IndirectMemory(String),
    Number(String),
    Identifier(String),
    Constant(String),
    Character(String),
    String(String),
}

#[derive(Debug, Clone)]
pub struct Comparison {
    pub left: Operand,
    pub equality: ComparisonOp,
    pub right: Operand,
}
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
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
