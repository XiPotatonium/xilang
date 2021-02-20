pub mod ast;
pub mod parser;
pub mod xi_crate;

mod gen;

use std::path::PathBuf;

pub struct XicCfg {
    pub ext_paths: Vec<String>,
    pub crate_name: String,
    pub root_dir: PathBuf,
    pub root_path: PathBuf,
    pub out_dir: PathBuf,
    pub optim: usize,
    pub verbose: usize,
}
