use std::collections::HashMap;
use std::rc::Rc;

use super::{VMClass, VMField, VMMethod};

pub struct VMModule {
    pub class_map: HashMap<u32, usize>,
    pub classes: Vec<Rc<VMClass>>,
    pub methods: Vec<Rc<VMMethod>>,
    pub fields: Vec<Rc<VMField>>,
}
