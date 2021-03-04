use std::fs;
use std::path::Path;

use super::super::blob::*;
use super::super::ir_file::*;

use pest::error::Error;
use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "ir/text_serde/ir.pest"]
struct IRParser;

pub fn parse(path: &Path) -> Result<IrFile, Error<Rule>> {
    let code = fs::read_to_string(path).unwrap();

    let file = IRParser::parse(Rule::File, &code)?.next().unwrap();

    for sub in file.into_inner() {
        match sub.as_rule() {
            Rule::EOI => break,
            _ => unreachable!(),
        };
    }

    unimplemented!();
}
