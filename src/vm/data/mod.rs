mod field;
mod method;
mod module;
mod ty;

use xir::file::IrFile;
use xir::sig::{self, TypeSig};
use xir::tok::{get_tok_tag, TokTag};
use xir::ty::ResolutionScope;

use super::util::ptr::NonNull;

use std::mem::size_of;

pub use self::field::Field;
pub use self::method::{
    method_str_desc, method_str_desc_from_ir, Local, MethodDesc, MethodILImpl, MethodImpl,
    MethodNativeImpl, MethodRuntimeImpl, Param,
};
pub use self::module::{ILModule, MemberRef, Module};
pub use self::ty::{Type, TypeInitState};

pub const REF_SIZE: usize = size_of::<*mut u8>();
pub const BOOL_SIZE: usize = size_of::<i8>();
pub const CHAR_SIZE: usize = size_of::<u16>();
pub const U1_SIZE: usize = size_of::<u8>();
pub const I1_SIZE: usize = size_of::<i8>();
pub const U4_SIZE: usize = size_of::<u32>();
pub const I4_SIZE: usize = size_of::<i32>();
pub const U8_SIZE: usize = size_of::<u64>();
pub const I8_SIZE: usize = size_of::<i64>();
pub const UNATIVE_SIZE: usize = size_of::<usize>();
pub const INATIVE_SIZE: usize = size_of::<isize>();
pub const R4_SIZE: usize = size_of::<f32>();
pub const R8_SIZE: usize = size_of::<f64>();

pub struct TypedAddr {
    pub ty: NonNull<Type>,
    pub addr: *mut u8,
}

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
    String,
    ByRef(Box<BuiltinType>),
    SZArray(Box<BuiltinType>),
    Value(NonNull<Type>),
    Class(NonNull<Type>),
    /// is_class, type, args
    GenericInst(bool, NonNull<Type>, Vec<BuiltinType>),
    /// to be filled
    Unk,
}

/// tok: TypeDefOrRefOrSpec
fn query_type_from_mod(tok: u32, ctx: &ILModule) -> NonNull<Type> {
    let (tag, idx) = get_tok_tag(tok);
    let idx = idx as usize - 1;
    match tag {
        TokTag::TypeDef => {
            NonNull::new(ctx.types[idx].as_ref() as *const Type as *mut Type).unwrap()
        }
        TokTag::TypeRef => ctx.typerefs[idx],
        TokTag::TypeSpec => unimplemented!(),
        _ => unreachable!(),
    }
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
            TypeSig::SZArray(ele_ty) => {
                BuiltinType::SZArray(Box::new(Self::from_type_sig(ele_ty, ctx)))
            }
            TypeSig::String => BuiltinType::String,
            TypeSig::ValueType(tok) => BuiltinType::Value(query_type_from_mod(*tok, ctx)),
            TypeSig::Class(tok) => BuiltinType::Class(query_type_from_mod(*tok, ctx)),
            TypeSig::GenericInst(is_class, tok, args) => {
                let ty = query_type_from_mod(*tok, ctx);
                BuiltinType::GenericInst(
                    *is_class,
                    ty,
                    args.iter()
                        .map(|arg| Self::from_type_sig(arg, ctx))
                        .collect(),
                )
            }
        }
    }

    /// must be called after ty info has been initialized
    pub fn byte_size(&self) -> usize {
        match self {
            BuiltinType::Void => panic!("Void type has no byte size"),
            BuiltinType::Bool => BOOL_SIZE,
            BuiltinType::Char => CHAR_SIZE,
            BuiltinType::U1 => U1_SIZE,
            BuiltinType::I1 => I1_SIZE,
            BuiltinType::U4 => U4_SIZE,
            BuiltinType::I4 => I4_SIZE,
            BuiltinType::U8 => U8_SIZE,
            BuiltinType::I8 => I8_SIZE,
            BuiltinType::UNative => UNATIVE_SIZE,
            BuiltinType::INative => INATIVE_SIZE,
            BuiltinType::R4 => R4_SIZE,
            BuiltinType::R8 => R8_SIZE,
            BuiltinType::String
            | BuiltinType::ByRef(_)
            | BuiltinType::SZArray(_)
            | BuiltinType::Class(_) => REF_SIZE,
            BuiltinType::Value(ty) => unsafe { ty.as_ref() }.basic_instance_size,
            BuiltinType::GenericInst(is_class, ty, _) => {
                if *is_class {
                    REF_SIZE
                } else {
                    unsafe { ty.as_ref() }.basic_instance_size
                }
            }
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
        BuiltinType::String => String::from("Ostd/String;"),
        BuiltinType::ByRef(inner) => format!("&{}", builtin_ty_str_desc(inner, str_pool)),
        BuiltinType::SZArray(inner) => format!("[{}", builtin_ty_str_desc(inner, str_pool)),
        BuiltinType::Class(ty) => {
            format!("O{};", unsafe { ty.as_ref().fullname(str_pool) })
        }
        BuiltinType::Value(ty) => {
            format!("o{};", unsafe { ty.as_ref().fullname(str_pool) })
        }
        BuiltinType::GenericInst(_, _, _) => todo!(),
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
        TypeSig::SZArray(_) => unimplemented!(),
        TypeSig::String => String::from("Ostd/String;"),
        TypeSig::Class(tok) => {
            let (tag, idx) = get_tok_tag(*tok);
            let idx = idx as usize - 1;
            match tag {
                TokTag::TypeDef => format!(
                    "O{}/{};",
                    ctx.mod_name(),
                    ctx.get_str(ctx.typedef_tbl[idx].name)
                ),
                TokTag::TypeRef => {
                    let (scope_tag, parent_idx) = ctx.typeref_tbl[idx].get_parent();
                    format!(
                        "O{}/{};",
                        ctx.get_str(match scope_tag {
                            ResolutionScope::Mod => ctx.mod_tbl[parent_idx].name,
                            ResolutionScope::ModRef => ctx.modref_tbl[parent_idx].name,
                            ResolutionScope::TypeRef => unimplemented!(),
                        }),
                        ctx.get_str(ctx.typeref_tbl[idx].name)
                    )
                }
                TokTag::TypeSpec => unimplemented!(),
                _ => unreachable!(),
            }
        }
        TypeSig::ValueType(tok) => {
            let (tag, idx) = get_tok_tag(*tok);
            let idx = idx as usize - 1;
            match tag {
                TokTag::TypeDef => format!(
                    "o{}/{};",
                    ctx.mod_name(),
                    ctx.get_str(ctx.typedef_tbl[idx].name)
                ),
                TokTag::TypeRef => {
                    let (scope_tag, parent_idx) = ctx.typeref_tbl[idx].get_parent();
                    format!(
                        "o{}/{};",
                        ctx.get_str(match scope_tag {
                            ResolutionScope::Mod => ctx.mod_tbl[parent_idx].name,
                            ResolutionScope::ModRef => ctx.modref_tbl[parent_idx].name,
                            ResolutionScope::TypeRef => unimplemented!(),
                        }),
                        ctx.get_str(ctx.typeref_tbl[idx].name)
                    )
                }
                TokTag::TypeSpec => unimplemented!(),
                _ => unreachable!(),
            }
        }
        TypeSig::GenericInst(_, _, _) => {
            todo!()
        }
    }
}

pub fn param_sig_str_desc(p: &sig::ParamType, ctx: &IrFile) -> String {
    match &p.ty {
        sig::InnerParamType::Default(ty) => type_sig_str_desc(ty, ctx),
        sig::InnerParamType::ByRef(ty) => format!("&{}", type_sig_str_desc(ty, ctx)),
    }
}
