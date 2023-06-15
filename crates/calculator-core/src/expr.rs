use crate::{
    eval::{Environment, Eval, EvalError, EvalResult},
    parse::{Parse, ParseError, ParseResult},
    token::{Token, TokenStream},
};
use num::{traits::Pow, BigInt, Zero};

#[derive(Debug, Clone)]
pub enum Expr {
    Int(BigInt),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Paren(Box<Expr>),
    Variable(String),
    Call(String, Vec<Expr>),
}

impl Eval for Expr {
    type Output = BigInt;

    fn eval(self, env: &mut Environment) -> EvalResult<Self::Output> {
        Ok(match self {
            Expr::Int(n) => n,
            Expr::Binary(lhs, BinaryOp::Assign, rhs) => {
                let r = rhs.eval(env)?;
                match *lhs {
                    Expr::Variable(ident) => {
                        env.set_variable(ident, r.clone());
                        r
                    }
                    _ => Err(EvalError::UnableToAssign)?,
                }
            }
            Expr::Binary(lhs, op, rhs) => {
                let (l, r) = (lhs.eval(env)?, rhs.eval(env)?);
                match op {
                    BinaryOp::Add => l + r,
                    BinaryOp::Sub => l - r,
                    BinaryOp::Mul => l * r,
                    BinaryOp::Div | BinaryOp::Rem if r.is_zero() => Err(EvalError::DevideByZero)?,
                    BinaryOp::Div => l / r,
                    BinaryOp::Rem => l % r,
                    BinaryOp::Pow => {
                        if let Some(r) = r.to_biguint() {
                            l.pow(r)
                        } else {
                            Err(EvalError::NegativePower)?
                        }
                    }
                    BinaryOp::Assign => Err(EvalError::Unimplemented)?,
                }
            }
            Expr::Unary(op, expr) => match op {
                UnaryOp::Plus => expr.eval(env)?,
                UnaryOp::Minus => -expr.eval(env)?,
            },
            Expr::Paren(expr) => expr.eval(env)?,
            Expr::Variable(ident) => env.get_variable(&ident).cloned()?,
            Expr::Call(s, args) if s.as_str() == "pow" => {
                if args.len() == 2 {
                    let mut it = args.into_iter();
                    let (l, r) = (it.next().unwrap().eval(env)?, it.next().unwrap().eval(env)?);
                    if let Some(r) = r.to_biguint() {
                        l.pow(r)
                    } else {
                        Err(EvalError::NegativePower)?
                    }
                } else {
                    Err(EvalError::InvalidArgumentLength)?
                }
            }
            Expr::Call(_, _) => Err(EvalError::Unimplemented)?,
        })
    }
}

impl Parse for BigInt {
    fn parse(input: &mut TokenStream) -> ParseResult<Self> {
        Ok(match input.peek()? {
            Token::NumLit(_) => match input.consume()? {
                Token::NumLit(s) => s.parse()?,
                _ => unreachable!(),
            },
            _ => Err(ParseError::ExpectedNum)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Pow,
    Assign,
}

impl Parse for BinaryOp {
    fn parse(input: &mut TokenStream) -> ParseResult<Self> {
        let op = Self::peek(input)?;
        input.consume()?;
        Ok(op)
    }
}

impl BinaryOp {
    pub fn peek(input: &TokenStream) -> ParseResult<Self> {
        Ok(match input.peek()? {
            Token::Plus => Self::Add,
            Token::Minus => Self::Sub,
            Token::Ast => Self::Mul,
            Token::Slash => Self::Div,
            Token::Percent => Self::Rem,
            Token::AstAst => Self::Pow,
            Token::Equal => Self::Assign,
            _ => Err(ParseError::ExpectedBinary)?,
        })
    }
    pub fn precedence(&self) -> Precedence {
        match self {
            Self::Add | Self::Sub => Precedence::Additive,
            Self::Mul | Self::Div | Self::Rem => Precedence::Multiplicative,
            Self::Pow => Precedence::Exponent,
            Self::Assign => Precedence::Assign,
        }
    }
    pub fn peek_precedence(input: &TokenStream) -> Option<Precedence> {
        match Self::peek(input) {
            Ok(op) => Some(op.precedence()),
            Err(_) => None,
        }
    }
    pub fn is_right(&self) -> bool {
        matches!(self, Self::Pow | Self::Assign)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Any,
    Assign,
    Additive,
    Multiplicative,
    Exponent,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Plus,
    Minus,
}

impl Parse for UnaryOp {
    fn parse(input: &mut TokenStream) -> ParseResult<Self> {
        let op = match input.peek()? {
            Token::Plus => UnaryOp::Plus,
            Token::Minus => UnaryOp::Minus,
            _ => Err(ParseError::ExpectedUnary)?,
        };
        input.consume()?;
        Ok(op)
    }
}

impl Parse for Expr {
    fn parse(input: &mut TokenStream) -> ParseResult<Self> {
        parse_expr(input)
    }
}

fn parse_expr(input: &mut TokenStream) -> ParseResult<Expr> {
    let lhs = parse_unary(input)?;
    parse_rexpr(input, lhs, Precedence::Any)
}

fn parse_rexpr(input: &mut TokenStream, mut lhs: Expr, base: Precedence) -> ParseResult<Expr> {
    while let Some(precedence) = BinaryOp::peek_precedence(input) {
        if precedence < base {
            break;
        }
        let op = input.parse::<BinaryOp>()?;
        let precedence = op.precedence();
        let mut rhs = parse_unary(input)?;
        while let Some(next) = BinaryOp::peek_precedence(input) {
            if next > precedence || next == precedence && op.is_right() {
                rhs = parse_rexpr(input, rhs, next)?;
            } else {
                break;
            }
        }
        lhs = Expr::Binary(Box::new(lhs), op, Box::new(rhs));
    }
    Ok(lhs)
}

fn parse_unary(input: &mut TokenStream) -> ParseResult<Expr> {
    let token = input.peek()?;
    Ok(match token {
        Token::Plus | Token::Minus => Expr::Unary(input.parse()?, Box::new(parse_unary(input)?)),
        Token::NumLit(_) => Expr::Int(input.parse()?),
        Token::LParen => {
            input.consume()?;
            let expr = input.parse()?;
            if matches!(input.consume()?, Token::RParen) {
                Expr::Paren(Box::new(expr))
            } else {
                Err(ParseError::ExpectedRParen)?
            }
        }
        Token::VarLit(_) => {
            let ident = match input.consume() {
                Ok(Token::VarLit(lit)) => lit.to_string(),
                _ => unreachable!(),
            };
            let token = input.peek();
            if matches!(token, Ok(Token::LParen)) {
                input.consume()?;
                let mut args = vec![];
                loop {
                    match parse_expr(input) {
                        Ok(expr) => {
                            args.push(expr);
                            match input.consume()? {
                                Token::Comma => {}
                                Token::RParen => break,
                                _ => Err(ParseError::ExpectedRParen)?,
                            }
                        }
                        Err(_) => match input.consume()? {
                            Token::RParen => break,
                            _ => Err(ParseError::ExpectedRParen)?,
                        },
                    }
                }
                Expr::Call(ident, args)
            } else {
                Expr::Variable(ident)
            }
        }
        _ => Err(ParseError::ExpectedUnary)?,
    })
}
