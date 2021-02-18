use super::class::Class;
use super::member::{Method, Var};
use super::module_mgr::ModuleMgr;
use crate::ir::flag::Flag;
use crate::ir::ty::RValType;

use std::cell::RefCell;
use std::collections::HashMap;

pub struct Locals {
    pub locals: Vec<Var>,
    pub sym_tbl: Vec<HashMap<String, usize>>,
}

impl Locals {
    pub fn new() -> Locals {
        Locals {
            locals: Vec::new(),
            sym_tbl: Vec::new(),
        }
    }

    pub fn push(&mut self) {
        self.sym_tbl.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.sym_tbl.pop().expect("Cannot pop empty stack");
    }

    pub fn add(&mut self, id: &str, ty: RValType, flag: Flag, initialized: bool) -> u16 {
        let offset = self.size();
        let var = Var::new(id, flag, ty, offset, initialized);
        self.sym_tbl
            .last_mut()
            .unwrap()
            .insert(id.to_owned(), self.locals.len());
        self.locals.push(var);
        offset
    }

    pub fn get(&self, id: &str) -> Option<&Var> {
        for frame in self.sym_tbl.iter().rev() {
            if let Some(ret) = frame.get(id) {
                return Some(&self.locals[*ret]);
            }
        }
        None
    }

    pub fn get_idx(&self, id: &str) -> Option<usize> {
        for frame in self.sym_tbl.iter().rev() {
            if let Some(ret) = frame.get(id) {
                return Some(*ret);
            }
        }
        None
    }

    pub fn contains_key(&self, id: &str) -> bool {
        for frame in self.sym_tbl.iter().rev() {
            if frame.contains_key(id) {
                return true;
            }
        }
        false
    }

    pub fn size(&self) -> u16 {
        self.locals.len() as u16
    }
}

pub struct CodeGenCtx<'mgr> {
    pub mgr: &'mgr ModuleMgr,
    pub class: &'mgr Class,
    pub method: &'mgr Method,
    pub locals: RefCell<Locals>,
}
