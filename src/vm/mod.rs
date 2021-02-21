pub mod executor;
pub mod loader;
pub mod mem;

mod data;

use std::path::PathBuf;

pub struct VMCfg {
    pub entry_root: PathBuf,
    pub ext_paths: Vec<PathBuf>,
    pub diagnose: bool,
}
