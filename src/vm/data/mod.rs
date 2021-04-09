mod field;
mod method;
mod module;
mod ty;

pub use self::field::VMField;
pub use self::method::{VMMethod, VMMethodILImpl, VMMethodImpl, VMMethodNativeImpl};
pub use self::module::{VMILModule, VMMemberRef, VMModule};
pub use self::ty::VMType;

#[derive(PartialEq, Eq)]
pub enum VMBuiltinType {
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
    Obj(*const VMType),
    Array(Box<VMBuiltinType>),
    /// to be filled
    Unk,
}
