pub mod data;
pub mod exec;
mod heap;
pub mod loader;
mod native;
pub mod shared_mem;
mod stack;
mod util;

use std::path::PathBuf;

pub struct VMCfg {
    pub entry_root: PathBuf,
    /// All canonicalized path
    /// external module root file or dir
    pub ext_paths: Vec<PathBuf>,
    pub diagnose: bool,
}
