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
