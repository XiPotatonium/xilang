use super::class::Class;
use super::member::{Locals, Method};
use super::module_mgr::ModuleMgr;

use std::cell::RefCell;

pub struct CodeGenCtx<'mgr> {
    pub mgr: &'mgr ModuleMgr,
    pub class: &'mgr Class,
    pub method: &'mgr Method,
    pub locals: RefCell<Locals>,
}
