use std::rc::Rc;

use super::{VMClasse, VMConstant, VMField, VMMethod};

pub struct VMModule {
    pub constant_map: Vec<VMConstant>,

    pub classes: Vec<Rc<VMClasse>>,
    pub methods: Vec<Rc<VMMethod>>,
    pub fields: Vec<Rc<VMField>>,
}
