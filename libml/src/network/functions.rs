use serde::{Deserialize, Serialize};
use strum::EnumString;

use super::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Combinator {
    Add,
    Mul,
}

impl Combinator {
    pub fn combine(&self, a: Value, b: Value) -> Value {
        match self {
            Combinator::Add => a + b,
            Combinator::Mul => a * b,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Activator {
    Binary,
    ReLU,
    Tanh,
}

impl Activator {
    pub fn activate(&self, value: Value) -> Value {
        match self {
            Activator::Binary => if value > 0.5 { 1.0 } else { 0.0 },
            Activator::ReLU => value.max(0.0),
            Activator::Tanh => value.tanh(),
        }
    }
}
