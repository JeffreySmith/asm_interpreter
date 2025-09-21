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

use thiserror::Error;

use crate::ast_builder;
use crate::value::Value;

#[derive(Debug, Error)]
pub enum InterpreterError {
    #[error("Failed to parse program: {0}")]
    ParseError(#[from] Box<pest::error::Error<ast_builder::Rule>>),

    #[error("Invalid operand: {0}")]
    InvalidOperand(String),

    #[error("Invalid register: {0}")]
    InvalidRegister(String),

    #[error("Invalid memory address: {0}")]
    InvalidMemoryAddress(String),

    #[error("Division by zero in {0}/{1}")]
    DivisionByZero(i64, i64),

    #[error("Operation not supported on these types: {0:?}")]
    TypeMismatch(Box<(Value, Value)>),

    #[error("Label not found: {0}")]
    LabelNotFound(String),

    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Cannot set a constant: {0}")]
    CannotSetConstant(String),

    #[error("Cannot set an identifier: {0}")]
    CannotSetIdentifier(String),

    #[error("Memory lock poisoned: {0}")]
    LockPoisoned(String),

    #[error("Other error: {0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum ValueError {
    #[error("Type mismatch: {0:?} and {1:?}")]
    TypeMismatch(Value, Value),

    #[error("Division by zero: {0}/{1}")]
    DivisionByZero(i64, i64),

    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}
