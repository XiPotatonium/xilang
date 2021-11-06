pub mod ast;
pub mod build;
pub mod parser;
pub mod sym;
mod util;

use std::path::PathBuf;

pub struct XicCfg {
    /// path to module root file (.xibc file)
    pub ext_paths: Vec<PathBuf>,
    pub crate_name: String,
    pub root_dir: PathBuf,
    pub root_path: PathBuf,
    pub out_dir: PathBuf,
    pub optim: usize,
    pub verbose: usize,
}
