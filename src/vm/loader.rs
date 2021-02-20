use super::mem::Mem;
use super::VMCfg;

use crate::ir::ir_file::*;

use std::fs;

pub fn load(cfg: &VMCfg, mem: &mut Mem) {
    let f = fs::File::open(&cfg.entry).unwrap();
    let f = IrFile::from_binary(Box::new(f));

    let const_pool_lnk: Vec<u64> = Vec::new();
}
