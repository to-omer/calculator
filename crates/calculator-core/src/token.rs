use crate::parse::{Parse, ParseError, ParseResult};
use unicode_ident::{is_xid_continue, is_xid_start};

#[derive(Debug, Clone, Copy)]
pub enum Token<'s> {
    NumLit(&'s str),
    VarLit(&'s str),
    Plus,
    Minus,
    Ast,
    AstAst,
    Slash,
    Percent,
    LParen,
    RParen,
    Comma,
    Equal,
}

pub fn tokens(mut s: &str) -> ParseResult<Vec<Token>> {
    macro_rules! symbol_arm {
        ($token:expr) => {{
            let (_, s1) = s.split_at(1);
            s = s1;
            $token
        }};
    }
    let mut buffer = Vec::new();
    while !s.is_empty() {
        let token = match s.chars().next().unwrap() {
            '0'..='9' => {
                let pos = s
                    .chars()
                    .position(|c| !c.is_ascii_digit())
                    .unwrap_or(s.len());
                let (lit, spos) = s.split_at(pos);
                s = spos;
                Token::NumLit(lit)
            }
            '+' => symbol_arm!(Token::Plus),
            '-' => symbol_arm!(Token::Minus),
            '*' => {
                if s.starts_with("**") {
                    let (_, s2) = s.split_at(2);
                    s = s2;
                    Token::AstAst
                } else {
                    symbol_arm!(Token::Ast)
                }
            }
            '/' => symbol_arm!(Token::Slash),
            '%' => symbol_arm!(Token::Percent),
            '(' => symbol_arm!(Token::LParen),
            ')' => symbol_arm!(Token::RParen),
            ',' => symbol_arm!(Token::Comma),
            '=' => symbol_arm!(Token::Equal),
            c if c.is_ascii_whitespace() => {
                let (_, s1) = s.split_at(1);
                s = s1;
                continue;
            }
            c if is_xid_start(c) => {
                let slen = c.len_utf8();
                let (_, s1) = s.split_at(slen);
                let pos = s1
                    .chars()
                    .position(|c| !is_xid_continue(c))
                    .unwrap_or(s1.len())
                    + slen;
                let (lit, spos) = s.split_at(pos);
                s = spos;
                Token::VarLit(lit)
            }
            _ => Err(ParseError::UnexpectedToken)?,
        };
        buffer.push(token);
    }
    Ok(buffer)
}

#[derive(Debug)]
pub struct TokenStream<'a> {
    tokens: &'a [Token<'a>],
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: &'a [Token<'a>]) -> Self {
        Self { tokens }
    }
    pub fn parse<T: Parse>(&mut self) -> ParseResult<T> {
        <T as Parse>::parse(self)
    }
    pub fn peek(&self) -> ParseResult<&Token> {
        self.tokens.first().ok_or(ParseError::UnexpectedEndOfInput)
    }
    pub fn consume(&mut self) -> ParseResult<Token> {
        if !self.tokens.is_empty() {
            let (first, res) = self.tokens.split_at(1);
            self.tokens = res;
            Ok(unsafe { *first.get_unchecked(0) })
        } else {
            Err(ParseError::UnexpectedEndOfInput)
        }
    }
    pub fn eof(&self) -> ParseResult<()> {
        if self.tokens.is_empty() {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken)
        }
    }
}
