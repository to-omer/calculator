use crate::expr::Expr;
use num::BigInt;
use std::{collections::HashMap, fmt::Display};
use thiserror::Error;

pub trait Eval {
    type Output: Display;
    fn eval(self, env: &mut Environment) -> EvalResult<Self::Output>;
}

#[derive(Debug, Clone, Error)]
pub enum EvalError {
    #[error("devide by zero")]
    DevideByZero,
    #[error("negative power")]
    NegativePower,
    #[error("unimplemented")]
    Unimplemented,
    #[error("invalid argument length")]
    InvalidArgumentLength,
    #[error("undefined variable")]
    UndefinedVariable,
    #[error("undefined function")]
    UndefinedFunction,
    #[error("unable to assign")]
    UnableToAssign,
}

pub type EvalResult<T> = Result<T, EvalError>;

#[derive(Debug, Default)]
pub struct Environment {
    variables: HashMap<String, BigInt>,
    functions: HashMap<String, Function>,
}

impl Environment {
    pub fn get_variable(&self, ident: &str) -> EvalResult<&BigInt> {
        self.variables
            .get(ident)
            .ok_or_else(|| EvalError::UndefinedVariable)
    }
    pub fn set_variable(&mut self, ident: String, expr: BigInt) {
        *self.variables.entry(ident).or_default() = expr;
    }
    pub fn call(&self, ident: &str, _args: Vec<Expr>) -> EvalResult<&Function> {
        self.functions
            .get(ident)
            .ok_or_else(|| EvalError::UndefinedFunction)
    }
}

pub type Function = (); // TODO
