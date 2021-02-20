use std::rc::Rc;

use super::{VMField, VMMethod};
use crate::ir::flag::TypeFlag;

pub struct VMClasse {
    flag: TypeFlag,

    methods: Vec<Rc<VMMethod>>,
    fields: Vec<Rc<VMField>>,
}
