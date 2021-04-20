mod field;
mod method;
mod module;
mod ty;

use xir::blob::EleType;
use xir::tok::{get_tok_tag, TokTag};

use std::mem::size_of;

pub use self::field::Field;
pub use self::method::{Method, MethodILImpl, MethodImpl, MethodNativeImpl, Param};
pub use self::module::{ILModule, MemberRef, Module};
pub use self::ty::Type;

/// VM representation of IrSig
#[derive(PartialEq, Eq)]
pub enum BuiltinType {
    Void,
    Bool,
    Char,
    U1,
    I1,
    U2,
    I2,
    U4,
    I4,
    U8,
    I8,
    UNative,
    INative,
    R4,
    R8,
    ByRef(Box<BuiltinType>),
    Array(Box<BuiltinType>),
    Class(*const Type),
    /// to be filled
    Unk,
}

impl BuiltinType {
    pub fn from_ir_ele_ty(ir_ty: &EleType, ctx: &ILModule) -> BuiltinType {
        match ir_ty {
            EleType::Void => BuiltinType::Void,
            EleType::Boolean => BuiltinType::Bool,
            EleType::Char => BuiltinType::Char,
            EleType::I1 => BuiltinType::I1,
            EleType::U1 => BuiltinType::U1,
            EleType::I2 => BuiltinType::I2,
            EleType::U2 => BuiltinType::U2,
            EleType::I4 => BuiltinType::I4,
            EleType::U4 => BuiltinType::U4,
            EleType::I8 => unimplemented!(),
            EleType::U8 => unimplemented!(),
            EleType::R4 => BuiltinType::R4,
            EleType::R8 => BuiltinType::R8,
            EleType::ByRef(t) => BuiltinType::ByRef(Box::new(BuiltinType::from_ir_ele_ty(t, ctx))),
            EleType::Class(tok) => {
                // tok is TypeRef or TypeDef
                let (tag, idx) = get_tok_tag(*tok);
                let idx = idx as usize - 1;
                BuiltinType::Class(match tag {
                    TokTag::TypeDef => ctx.classes[idx].as_ref() as *const Type,
                    TokTag::TypeRef => ctx.classref[idx],
                    _ => unreachable!(),
                })
            }
        }
    }

    pub fn heap_size(&self) -> usize {
        match self {
            BuiltinType::Void => panic!("Void type has no heap size"),
            BuiltinType::Bool => size_of::<i32>(),
            BuiltinType::Char => size_of::<u16>(),
            BuiltinType::U1 => size_of::<u8>(),
            BuiltinType::I1 => size_of::<i8>(),
            BuiltinType::U2 => size_of::<u16>(),
            BuiltinType::I2 => size_of::<i16>(),
            BuiltinType::U4 => size_of::<u32>(),
            BuiltinType::I4 => size_of::<i32>(),
            BuiltinType::U8 => size_of::<u64>(),
            BuiltinType::I8 => size_of::<i64>(),
            BuiltinType::UNative => size_of::<usize>(),
            BuiltinType::INative => size_of::<isize>(),
            BuiltinType::R4 => size_of::<f32>(),
            BuiltinType::R8 => size_of::<f64>(),
            BuiltinType::ByRef(_) => size_of::<usize>(),
            BuiltinType::Array(_) => unimplemented!(),
            BuiltinType::Class(_) => unreachable!(),
            BuiltinType::Unk => unreachable!(),
        }
    }
}
