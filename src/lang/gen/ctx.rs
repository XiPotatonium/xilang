use super::class::Class;
use super::member::Method;
use super::module_mgr::ModuleMgr;
use crate::ir::flag::*;
use crate::ir::ty::RValType;

use std::cell::RefCell;
use std::collections::HashMap;

pub struct Var {
    pub id: String,
    pub flag: LocalFlag,
    pub ty: RValType,
    pub offset: u16,
    pub initialized: bool,
}

impl Var {
    pub fn new(id: &str, flag: LocalFlag, ty: RValType, offset: u16, initialized: bool) -> Var {
        Var {
            id: id.to_owned(),
            flag,
            ty,
            offset,
            initialized,
        }
    }
}

pub struct Arg {
    pub flag: ParamFlag,
    pub ty: RValType,
    pub offset: u16,
}

impl Arg {
    pub fn new(flag: ParamFlag, ty: RValType, offset: u16) -> Arg {
        Arg { flag, ty, offset }
    }
}

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

    pub fn add(&mut self, id: &str, ty: RValType, flag: LocalFlag, initialized: bool) -> u16 {
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
    pub args_map: HashMap<String, Arg>,
}
