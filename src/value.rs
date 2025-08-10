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

use crate::ast::ComparisonOp;

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
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{a}{b}"))),
            _ => Err("Cannot add Number and String".to_string()),
        }
    }
    // For strings, when we subtract a positive number, it removes characters from the front.
    // Negative numbers remove them from the end. This "kind of" works like Python's list indexing
    pub fn sub(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            (Value::String(a), Value::Number(b)) => {
                let len = a.chars().count();
                if *b >= 0 {
                    Ok(Value::String(
                        a.chars().skip(b.unsigned_abs() as usize).collect(),
                    ))
                } else {
                    let abs_len = b.unsigned_abs() as usize;
                    if abs_len >= len {
                        Ok(Value::String(String::new()))
                    } else {
                        Ok(Value::String(a.chars().take(len - abs_len).collect()))
                    }
                }
            }
            _ => Err("Invalid subtraction".to_string()),
        }
    }
    // Multiplying a string and a number returns a string concatenated with itself
    // the number of times it's being multiplied by.
    // If you multiply it by number less than or equal to zero, you will get an empty string
    pub fn mul(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            (Value::String(a), Value::Number(b)) => {
                let mut new_string = String::new();
                for _ in 0..*b {
                    new_string += a;
                }
                Ok(Value::String(new_string))
            }
            _ => Err("Invalid multiplication between number and string".to_string()),
        }
    }
    // Dividing two strings gives us the size difference between the two strings. Division between
    // two numbers is always truncated since it will always be integer division
    pub fn div(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                if *b > 0 {
                    Ok(Value::Number(a / b))
                } else {
                    Err(format!("Division denominator is zero in {a}/{b}"))
                }
            }
            (Value::String(a), Value::String(b)) => {
                let len_left = a.chars().count() as i64;
                let len_right = b.chars().count() as i64;
                Ok(Value::Number(len_left - len_right))
            }
            _ => Err("Invalid division between number and string".to_string()),
        }
    }
    pub fn and(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => {
                println!("A: {a}, B: {b}");
                Ok(Value::Number(a & b))
            }
            _ => Err("Invalid 'and'. Both arguments must be numbers ".to_string()),
        }
    }
    pub fn or(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a | b)),
            _ => Err("Invalid 'and'. Both arguments must be numbers.".to_string()),
        }
    }
    pub fn xor(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a ^ b)),
            _ => Err("Invalid 'xor'. Both arguments must be numbers.".to_string()),
        }
    }
    pub fn not(&self) -> Result<Value, String> {
        match self {
            Value::Number(a) => Ok(Value::Number(!a)),
            Value::String(a) => Err(format!("Invalid 'not' for '{a:?}'. Must be a number")),
        }
    }
}
impl Default for Value {
    fn default() -> Self {
        Value::Number(0)
    }
}
