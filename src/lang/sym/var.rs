use std::collections::HashMap;

use super::RValType;

pub struct Var {
    pub id: String,
    pub ty: RValType,
    /// index in Local var table
    pub idx: u16,
    pub initialized: bool,
}

impl Var {
    pub fn new(id: &str, ty: RValType, idx: u16, initialized: bool) -> Var {
        Var {
            id: id.to_owned(),
            ty,
            idx,
            initialized,
        }
    }
}

pub struct Locals {
    pub local_lst: Vec<Var>,
    /// map from id to index of local
    pub sym_tbl: Vec<HashMap<String, usize>>,
}

impl Locals {
    pub fn new() -> Locals {
        Locals {
            local_lst: Vec::new(),
            sym_tbl: Vec::new(),
        }
    }

    pub fn push(&mut self) {
        self.sym_tbl.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.sym_tbl.pop().expect("Cannot pop empty stack");
    }

    pub fn add(&mut self, id: &str, ty: RValType, initialized: bool) -> u16 {
        let idx = self.local_lst.len();
        let var = Var::new(id, ty, idx as u16, initialized);
        self.sym_tbl.last_mut().unwrap().insert(id.to_owned(), idx);
        self.local_lst.push(var);
        idx as u16
    }

    pub fn add_tmp(&mut self, ty: RValType, initialized: bool) -> u16 {
        let idx = self.local_lst.len();
        let id = format!("${}", idx);
        let var = Var::new(&id, ty, idx as u16, initialized);
        self.sym_tbl.last_mut().unwrap().insert(id, idx);
        self.local_lst.push(var);
        idx as u16
    }

    pub fn get(&self, id: &str) -> Option<&Var> {
        for frame in self.sym_tbl.iter().rev() {
            if let Some(ret) = frame.get(id) {
                return Some(&self.local_lst[*ret]);
            }
        }
        None
    }
}
