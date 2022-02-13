pub mod ast;
pub mod build;
pub mod parser;
pub mod sym;
mod util;

use std::path::PathBuf;

pub struct XiCfg {
    pub crate_name: String,
    pub root_dir: PathBuf,
    pub root_path: PathBuf,
    pub out_dir: PathBuf,
    pub dump_ast: bool,
    pub compile: bool,
}
