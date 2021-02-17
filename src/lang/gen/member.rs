use crate::ir::flag::Flag;
use crate::ir::ty::VarType;

use std::collections::HashMap;

pub struct Var {
    pub id: String,
    pub flag: Flag,
    pub ty: VarType,
    pub offset: u16,
    pub initialized: bool,
}

impl Var {
    pub fn new(id: &str, flag: Flag, ty: VarType, offset: u16, initialized: bool) -> Var {
        Var {
            id: id.to_owned(),
            flag,
            ty,
            offset,
            initialized,
        }
    }
}

pub struct Field {
    pub id: String,
    pub flag: Flag,
    pub ty: VarType,
}

impl Field {
    pub fn new(id: &str, flag: Flag, ty: VarType) -> Field {
        Field {
            id: id.to_owned(),
            flag,
            ty,
        }
    }
}

pub struct Locals {
    pub locals: Vec<Var>,
    pub size: u16,
    pub sym_tbl: Vec<HashMap<String, usize>>,
}

impl Locals {
    pub fn new() -> Locals {
        Locals {
            locals: Vec::new(),
            size: 0,
            sym_tbl: Vec::new(),
        }
    }

    pub fn push(&mut self) {
        self.sym_tbl.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.sym_tbl.pop().expect("Cannot pop empty stack");
    }

    pub fn add(&mut self, id: &str, ty: VarType, flag: Flag, initialized: bool) -> u16 {
        let var_size = ty.slot();
        let var = Var::new(id, flag, ty, self.size, initialized);
        let offset = self.size;
        self.sym_tbl
            .last_mut()
            .unwrap()
            .insert(id.to_owned(), self.locals.len());
        self.locals.push(var);
        self.size += var_size;
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
}

pub struct Method {
    pub flag: Flag,
    pub ret_ty: VarType,
    pub ps_ty: Vec<VarType>,
    // method idx in class file
    pub method_idx: usize,
}
