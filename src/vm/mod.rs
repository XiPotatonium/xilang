pub mod exec;
pub mod loader;
pub mod mem;

mod data;
mod native;

use std::path::PathBuf;

pub struct VMCfg {
    pub entry_root: PathBuf,
    /// All canonicalized path
    /// dir or file
    pub ext_paths: Vec<PathBuf>,
    pub diagnose: bool,
}
