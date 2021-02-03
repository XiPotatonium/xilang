use super::parser::token::TokenTag;

#[derive(Debug)]
pub enum LangErr {
    // expect, actual
    UnexpectedToken(TokenTag, TokenTag),
    Err(&'static str),
}
