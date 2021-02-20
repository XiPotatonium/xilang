mod class;
mod field;
mod method;
mod module;

pub use self::class::VMClasse;
pub use self::field::VMField;
pub use self::method::VMMethod;
pub use self::module::VMModule;

use std::rc::Rc;

pub enum VMConstant {
    Utf8(u32),
    Class(Rc<VMClasse>),
    Fieldref(Rc<VMClasse>, Rc<VMField>),
    Methodref(Rc<VMClasse>, Rc<VMMethod>),
    Mod(Rc<VMModule>),
    None,
}
