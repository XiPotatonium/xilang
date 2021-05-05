mod field;
mod method;
mod module;
mod ty;

use xir::file::IrFile;
use xir::sig::{self, TypeSig};
use xir::tok::{get_tok_tag, TokTag};
use xir::ty::ResolutionScope;

use std::mem::size_of;

pub use self::field::Field;
pub use self::method::{
    method_str_desc, method_str_desc_from_ir, Local, MethodDesc, MethodILImpl, MethodImpl,
    MethodNativeImpl, Param,
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
    pub fn from_param(param: &sig::ParamType, ctx: &ILModule) -> BuiltinType {
        match &param.ty {
            sig::InnerParamType::Default(ty) => Self::from_type_sig(ty, ctx),
            sig::InnerParamType::ByRef(ty) => {
                BuiltinType::ByRef(Box::new(Self::from_type_sig(ty, ctx)))
            }
        }
    }

    pub fn from_ret(ret: &sig::RetType, ctx: &ILModule) -> BuiltinType {
        match &ret.ty {
            sig::InnerRetType::Default(ty) => Self::from_type_sig(ty, ctx),
            sig::InnerRetType::ByRef(ty) => {
                BuiltinType::ByRef(Box::new(Self::from_type_sig(ty, ctx)))
            }
            sig::InnerRetType::Void => BuiltinType::Void,
        }
    }

    pub fn from_local(var: &sig::InnerLocalVarType, ctx: &ILModule) -> BuiltinType {
        match var {
            sig::InnerLocalVarType::Default(ty) => Self::from_type_sig(ty, ctx),
            sig::InnerLocalVarType::ByRef(ty) => {
                BuiltinType::ByRef(Box::new(Self::from_type_sig(ty, ctx)))
            }
        }
    }

    pub fn from_type_sig(ty: &TypeSig, ctx: &ILModule) -> BuiltinType {
        match ty {
            TypeSig::Boolean => BuiltinType::Bool,
            TypeSig::Char => BuiltinType::Char,
            TypeSig::I1 => BuiltinType::I1,
            TypeSig::U1 => BuiltinType::U1,
            TypeSig::I4 => BuiltinType::I4,
            TypeSig::U4 => BuiltinType::U4,
            TypeSig::I8 => unimplemented!(),
            TypeSig::U8 => unimplemented!(),
            TypeSig::R4 => BuiltinType::R4,
            TypeSig::R8 => BuiltinType::R8,
            TypeSig::I => BuiltinType::INative,
            TypeSig::U => BuiltinType::UNative,
            TypeSig::Array(_, _) => unimplemented!(),
            TypeSig::String => unimplemented!(),
            TypeSig::Class(tok) => {
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
            BuiltinType::U4 => size_of::<u32>(),
            BuiltinType::I4 => size_of::<i32>(),
            BuiltinType::U8 => size_of::<u64>(),
            BuiltinType::I8 => size_of::<i64>(),
            BuiltinType::UNative => size_of::<usize>(),
            BuiltinType::INative => size_of::<isize>(),
            BuiltinType::R4 => size_of::<f32>(),
            BuiltinType::R8 => size_of::<f64>(),
            BuiltinType::ByRef(_) | BuiltinType::Array(_) | BuiltinType::Class(_) => REF_SIZE,
            BuiltinType::Unk => unreachable!(),
        }
    }
}

pub fn builtin_ty_str_desc(ty: &BuiltinType, str_pool: &Vec<String>) -> String {
    match ty {
        BuiltinType::Void => String::from("V"),
        BuiltinType::Bool => String::from("Z"),
        BuiltinType::Char => String::from("C"),
        BuiltinType::U1 => String::from("B"),
        BuiltinType::I1 => String::from("b"),
        BuiltinType::U4 => String::from("I"),
        BuiltinType::I4 => String::from("i"),
        BuiltinType::U8 => String::from("L"),
        BuiltinType::I8 => String::from("l"),
        BuiltinType::UNative => String::from("N"),
        BuiltinType::INative => String::from("n"),
        BuiltinType::R4 => String::from("F"),
        BuiltinType::R8 => String::from("D"),
        BuiltinType::ByRef(inner) => format!("&{}", builtin_ty_str_desc(inner, str_pool)),
        BuiltinType::Array(inner) => format!("[{}", builtin_ty_str_desc(inner, str_pool)),
        BuiltinType::Class(ty) => {
            format!("O{};", unsafe { ty.as_ref().unwrap().fullname(str_pool) })
        }
        BuiltinType::Unk => unreachable!(),
    }
}

fn type_sig_str_desc(ty: &TypeSig, ctx: &IrFile) -> String {
    match ty {
        TypeSig::Boolean => String::from("Z"),
        TypeSig::Char => String::from("C"),
        TypeSig::I1 => String::from("b"),
        TypeSig::U1 => String::from("B"),
        TypeSig::I4 => String::from("i"),
        TypeSig::U4 => String::from("I"),
        TypeSig::I8 => String::from("l"),
        TypeSig::U8 => String::from("L"),
        TypeSig::R4 => String::from("F"),
        TypeSig::R8 => String::from("D"),
        TypeSig::I => String::from("n"),
        TypeSig::U => String::from("N"),
        TypeSig::Array(_, _) => unimplemented!(),
        TypeSig::String => unimplemented!(),
        TypeSig::Class(tok) => {
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

pub fn param_sig_str_desc(p: &sig::ParamType, ctx: &IrFile) -> String {
    match &p.ty {
        sig::InnerParamType::Default(ty) => type_sig_str_desc(ty, ctx),
        sig::InnerParamType::ByRef(ty) => format!("&{}", type_sig_str_desc(ty, ctx)),
    }
}
