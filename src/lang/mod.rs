pub mod ast;
pub mod gen;
pub mod parser;

use std::path::PathBuf;

pub struct XicCfg {
    pub ext_paths: Vec<String>,
    pub crate_name: String,
    pub root_path: PathBuf,
    pub out_dir: PathBuf,
    pub optim: usize,
    pub verbose: usize,
}
