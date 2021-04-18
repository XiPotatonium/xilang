mod field;
mod method;
mod module;
mod ty;

use xir::blob::EleType;
use xir::tok::{get_tok_tag, TokTag};

use std::mem::size_of;

pub use self::field::VMField;
pub use self::method::{VMMethod, VMMethodILImpl, VMMethodImpl, VMMethodNativeImpl};
pub use self::module::{VMILModule, VMMemberRef, VMModule};
pub use self::ty::VMType;

/// VM representation of IrSig
#[derive(PartialEq, Eq)]
pub enum VMBuiltinType {
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
    ByRef(Box<VMBuiltinType>),
    Array(Box<VMBuiltinType>),
    Class(*const VMType),
    /// to be filled
    Unk,
}

impl VMBuiltinType {
    pub fn from_ir_ele_ty(ir_ty: &EleType, ctx: &VMILModule) -> VMBuiltinType {
        match ir_ty {
            EleType::Void => VMBuiltinType::Void,
            EleType::Boolean => VMBuiltinType::Bool,
            EleType::Char => VMBuiltinType::Char,
            EleType::I1 => VMBuiltinType::I1,
            EleType::U1 => VMBuiltinType::U1,
            EleType::I2 => VMBuiltinType::I2,
            EleType::U2 => VMBuiltinType::U2,
            EleType::I4 => VMBuiltinType::I4,
            EleType::U4 => VMBuiltinType::U4,
            EleType::I8 => unimplemented!(),
            EleType::U8 => unimplemented!(),
            EleType::R4 => VMBuiltinType::R4,
            EleType::R8 => VMBuiltinType::R8,
            EleType::ByRef(t) => {
                VMBuiltinType::ByRef(Box::new(VMBuiltinType::from_ir_ele_ty(t, ctx)))
            }
            EleType::Class(tok) => {
                // tok is TypeRef or TypeDef
                let (tag, idx) = get_tok_tag(*tok);
                let idx = idx as usize - 1;
                VMBuiltinType::Class(match tag {
                    TokTag::TypeDef => ctx.classes[idx].as_ref() as *const VMType,
                    TokTag::TypeRef => ctx.classref[idx],
                    _ => unreachable!(),
                })
            }
        }
    }

    pub fn heap_size(&self) -> usize {
        match self {
            VMBuiltinType::Void => panic!("Void type has no heap size"),
            VMBuiltinType::Bool => size_of::<i32>(),
            VMBuiltinType::Char => size_of::<u16>(),
            VMBuiltinType::U1 => size_of::<u8>(),
            VMBuiltinType::I1 => size_of::<i8>(),
            VMBuiltinType::U2 => size_of::<u16>(),
            VMBuiltinType::I2 => size_of::<i16>(),
            VMBuiltinType::U4 => size_of::<u32>(),
            VMBuiltinType::I4 => size_of::<i32>(),
            VMBuiltinType::U8 => size_of::<u64>(),
            VMBuiltinType::I8 => size_of::<i64>(),
            VMBuiltinType::UNative => size_of::<usize>(),
            VMBuiltinType::INative => size_of::<isize>(),
            VMBuiltinType::R4 => size_of::<f32>(),
            VMBuiltinType::R8 => size_of::<f64>(),
            VMBuiltinType::ByRef(_) => size_of::<usize>(),
            VMBuiltinType::Array(_) => unimplemented!(),
            VMBuiltinType::Class(_) => unreachable!(),
            VMBuiltinType::Unk => unreachable!(),
        }
    }
}
