mod class;
mod field;
mod method;
mod module;

pub use self::class::VMClass;
pub use self::field::VMField;
pub use self::method::VMMethod;
pub use self::module::VMModule;

use std::rc::Rc;

pub enum VMType {
    Void,
    Bool,
    Char,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    UNative,
    INative,
    F32,
    F64,
    Obj(Rc<VMClass>),
    Array(Box<VMType>),
    /// to be filled
    Unk,
}
