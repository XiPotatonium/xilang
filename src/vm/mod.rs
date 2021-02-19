pub mod executor;
pub mod loader;
pub mod mem;

mod data;

use std::path::PathBuf;

pub struct VMCfg {
    pub entry: PathBuf,
    pub ext_paths: Vec<String>,
    pub diagnose: bool,
}
