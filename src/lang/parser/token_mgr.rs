use super::lexer::Lexer;
use super::token::Token;

use std::io::Read;
use std::ops::Index;

pub struct TokenMgr {
    _token_buf: Vec<Token>,
}

impl TokenMgr {
    pub fn new<R: Read>(l: Lexer<R>) -> TokenMgr {
        let mut v: Vec<Token> = Vec::new();
        for t in l {
            v.push(t);
        }
        TokenMgr { _token_buf: v }
    }

    pub fn len(&self) -> usize {
        self._token_buf.len()
    }
}

impl Index<usize> for TokenMgr {
    type Output = Token;
    fn index(&self, idx: usize) -> &Self::Output {
        self._token_buf.index(idx)
    }
}
