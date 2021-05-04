mod field;
mod method;
mod module;
mod ty;

use xir::blob::EleType;
use xir::file::IrFile;
use xir::tok::{get_tok_tag, TokTag};
use xir::ty::ResolutionScope;

use std::mem::size_of;

pub use self::field::Field;
pub use self::method::{
    method_sig, method_sig_from_ir, Local, MethodDesc, MethodILImpl, MethodImpl, MethodNativeImpl,
    Param,
};
pub use self::module::{ILModule, MemberRef, Module};
pub use self::ty::Type;

pub const REF_SIZE: usize = size_of::<*mut u8>();

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
                    TokTag::TypeDef => ctx.types[idx].as_ref() as *const Type,
                    TokTag::TypeRef => ctx.typerefs[idx],
                    _ => unreachable!(),
                })
            }
        }
    }

    pub fn byte_size(&self) -> usize {
        match self {
            BuiltinType::Void => panic!("Void type has no byte size"),
            BuiltinType::Bool => size_of::<i8>(),
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
            BuiltinType::ByRef(_) => REF_SIZE,
            BuiltinType::Array(_) => unimplemented!(),
            BuiltinType::Class(_) => unreachable!(),
            BuiltinType::Unk => unreachable!(),
        }
    }
}

pub fn builtin_ty_sig(ty: &BuiltinType, str_pool: &Vec<String>) -> String {
    match ty {
        BuiltinType::Void => String::from("V"),
        BuiltinType::Bool => String::from("Z"),
        BuiltinType::Char => String::from("C"),
        BuiltinType::U1 => String::from("B"),
        BuiltinType::I1 => String::from("b"),
        BuiltinType::U2 => String::from("S"),
        BuiltinType::I2 => String::from("s"),
        BuiltinType::U4 => String::from("I"),
        BuiltinType::I4 => String::from("i"),
        BuiltinType::U8 => String::from("L"),
        BuiltinType::I8 => String::from("l"),
        BuiltinType::UNative => String::from("N"),
        BuiltinType::INative => String::from("n"),
        BuiltinType::R4 => String::from("F"),
        BuiltinType::R8 => String::from("D"),
        BuiltinType::ByRef(inner) => format!("O{};", builtin_ty_sig(inner, str_pool)),
        BuiltinType::Array(inner) => format!("[{}", builtin_ty_sig(inner, str_pool)),
        BuiltinType::Class(ty) => unsafe { ty.as_ref().unwrap().fullname(str_pool) },
        BuiltinType::Unk => unreachable!(),
    }
}

pub fn ir_ty_sig(ty: &EleType, ctx: &IrFile) -> String {
    match ty {
        EleType::Void => String::from("V"),
        EleType::Boolean => String::from("Z"),
        EleType::Char => String::from("C"),
        EleType::I1 => String::from("b"),
        EleType::U1 => String::from("B"),
        EleType::I2 => String::from("s"),
        EleType::U2 => String::from("S"),
        EleType::I4 => String::from("i"),
        EleType::U4 => String::from("I"),
        EleType::I8 => String::from("l"),
        EleType::U8 => String::from("L"),
        EleType::R4 => String::from("F"),
        EleType::R8 => String::from("D"),
        EleType::ByRef(inner) => format!("O{};", ir_ty_sig(inner, ctx)),
        EleType::Class(tok) => {
            let (tag, idx) = get_tok_tag(*tok);
            let idx = idx as usize - 1;
            match tag {
                TokTag::TypeDef => format!(
                    "{}::{}",
                    ctx.mod_name(),
                    ctx.get_str(ctx.typedef_tbl[idx].name)
                ),
                TokTag::TypeRef => {
                    let (scope_tag, parent_idx) = ctx.typeref_tbl[idx].get_parent();
                    format!(
                        "{}::{}",
                        ctx.get_str(match scope_tag {
                            ResolutionScope::Mod => ctx.mod_tbl[parent_idx].name,
                            ResolutionScope::ModRef => ctx.modref_tbl[parent_idx].name,
                            ResolutionScope::TypeRef => unimplemented!(),
                        }),
                        ctx.get_str(ctx.typeref_tbl[idx].name)
                    )
                }
                _ => unreachable!(),
            }
        }
    }
}
