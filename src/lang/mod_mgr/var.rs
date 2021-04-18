use xir::attrib::*;

use super::super::gen::RValType;

use std::collections::HashMap;

pub struct Var {
    pub id: String,
    pub flag: LocalAttrib,
    pub ty: RValType,
    /// index in Local var table
    pub idx: u16,
    pub initialized: bool,
}

impl Var {
    pub fn new(id: &str, flag: LocalAttrib, ty: RValType, idx: u16, initialized: bool) -> Var {
        Var {
            id: id.to_owned(),
            flag,
            ty,
            idx,
            initialized,
        }
    }
}

pub struct Arg {
    pub attrib: ParamAttrib,
    pub ty: RValType,
    pub offset: u16,
}

impl Arg {
    pub fn new(attrib: ParamAttrib, ty: RValType, offset: u16) -> Arg {
        Arg { attrib, ty, offset }
    }
}

pub struct Locals {
    pub locals: Vec<Var>,
    /// map from id to index of local
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

    pub fn add(&mut self, id: &str, ty: RValType, flag: LocalAttrib, initialized: bool) -> u16 {
        let idx = self.locals.len();
        let var = Var::new(id, flag, ty, idx as u16, initialized);
        self.sym_tbl.last_mut().unwrap().insert(id.to_owned(), idx);
        self.locals.push(var);
        idx as u16
    }

    pub fn get(&self, id: &str) -> Option<&Var> {
        for frame in self.sym_tbl.iter().rev() {
            if let Some(ret) = frame.get(id) {
                return Some(&self.locals[*ret]);
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
}
