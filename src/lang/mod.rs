pub mod ast;
pub mod build;
pub mod parser;
pub mod sym;

use std::path::PathBuf;

pub const SYS_NAME: &str = "sys";
pub const STRING_CLASS_NAME: &str = "sys/String";
pub const SYS_PATH: &str = "../../sys/mod.xi";

pub struct XiCfg {
    pub entry_path: PathBuf,
    pub dump_ast: bool,
    pub compile: bool,
    pub no_sys: bool,
}
