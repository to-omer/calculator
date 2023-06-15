use crate::token::{tokens, TokenStream};
use num::bigint::ParseBigIntError;
use thiserror::Error;

pub trait Parse: Sized {
    fn parse(input: &mut TokenStream) -> ParseResult<Self>;
}

pub fn parse_from_str<T: Parse>(input: &str) -> ParseResult<T> {
    let tokens = tokens(input)?;
    let mut stream = TokenStream::new(&tokens);
    let t = stream.parse()?;
    stream.eof()?;
    Ok(t)
}

#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("expected one of `+-`")]
    ExpectedUnary,
    #[error("expected one of `+-*/%`")]
    ExpectedBinary,
    #[error("expected digits")]
    ExpectedNum,
    #[error("expected `)`")]
    ExpectedRParen,
    #[error("unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("unexpected token")]
    UnexpectedToken,
    #[error("unexpected integer literal")]
    ParseBigIntError(#[from] ParseBigIntError),
}

pub type ParseResult<T> = Result<T, ParseError>;
