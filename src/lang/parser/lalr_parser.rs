use super::super::ast::ast::AST;

use std::fs;
use std::io::{Read};

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(grammar, "/lang/parser/grammar.rs");

pub fn lalr_parse(file: fs::File) -> Result<Box<AST>, &'static str> {
    let mut text = String::new();
    let mut file = file;
    file.read_to_string(&mut text).unwrap();
    Ok(grammar::FileParser::new().parse(&text).unwrap())
}
