mod method;

pub use method::{
    InnerParamType, InnerRetType, MethodSigFlag, MethodSigFlagTag, ParamType, RetType,
};

use super::bc_serde::{IDeserializer, ISerializable};
use super::file::IrFile;
use super::text_serde::IrFmt;
use super::tok::fmt_tok;

use std::convert::TryFrom;
use std::fmt;

/// II.23.1.16 Element types used in signature
enum EleType {
    Void,
    Boolean,
    Char,
    I1,
    U1,
    I4,
    U4,
    I8,
    U8,
    R4,
    R8,
    String,
    ByRef,
    Class,
    SZArray,
    I,
    U,
    Object,
}

impl TryFrom<u8> for EleType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            ELEMENT_TYPE_VOID => Self::Void,
            ELEMENT_TYPE_BOOLEAN => Self::Boolean,
            ELEMENT_TYPE_CHAR => Self::Char,
            ELEMENT_TYPE_I1 => Self::I1,
            ELEMENT_TYPE_U1 => Self::U1,
            ELEMENT_TYPE_I4 => Self::I4,
            ELEMENT_TYPE_U4 => Self::U4,
            ELEMENT_TYPE_I8 => Self::I8,
            ELEMENT_TYPE_U8 => Self::U8,
            ELEMENT_TYPE_R4 => Self::R4,
            ELEMENT_TYPE_R8 => Self::R8,
            ELEMENT_TYPE_STRING => Self::String,
            ELEMENT_TYPE_BYREF => Self::ByRef,
            ELEMENT_TYPE_CLASS => Self::Class,
            ELEMENT_TYPE_SZARRAY => Self::SZArray,
            ELEMENT_TYPE_I => Self::I,
            ELEMENT_TYPE_U => Self::U,
            ELEMENT_TYPE_OBJECT => Self::Object,
            _ => return Err("Invalid value for EleType"),
        })
    }
}

const ELEMENT_TYPE_VOID: u8 = 0x01;
const ELEMENT_TYPE_BOOLEAN: u8 = 0x02;
const ELEMENT_TYPE_CHAR: u8 = 0x03;
const ELEMENT_TYPE_I1: u8 = 0x04;
const ELEMENT_TYPE_U1: u8 = 0x05;
const ELEMENT_TYPE_I4: u8 = 0x08;
const ELEMENT_TYPE_U4: u8 = 0x09;
const ELEMENT_TYPE_I8: u8 = 0x0A;
const ELEMENT_TYPE_U8: u8 = 0x0B;
const ELEMENT_TYPE_R4: u8 = 0x0C;
const ELEMENT_TYPE_R8: u8 = 0x0D;
const ELEMENT_TYPE_STRING: u8 = 0x0E;
const ELEMENT_TYPE_BYREF: u8 = 0x10;
const ELEMENT_TYPE_CLASS: u8 = 0x12;
const ELEMENT_TYPE_I: u8 = 0x18;
const ELEMENT_TYPE_U: u8 = 0x19;
const ELEMENT_TYPE_OBJECT: u8 = 0x1C;
const ELEMENT_TYPE_SZARRAY: u8 = 0x1D;

/// II.23.2.12
pub enum TypeSig {
    Boolean,
    Char,
    I1,
    U1,
    I4,
    U4,
    I8,
    U8,
    R4,
    R8,
    I,
    U,
    /// No CustomMod for now
    SZArray(Box<TypeSig>),
    /// typedef or typeref or typespec
    Class(u32),
    String,
}

/// II.23.2.13
/*
pub struct ArrayShapeSig {
    pub rank: u32,
    pub sizes: Vec<u32>,
    // no lower bound, default to be 0
}
*/
// TypeSpec:
//  0x0F(PTR) CustomMod* (VOID|Type)
//  | 0x1B(FNPTR) (MethodDefSig|MethodRefSig)
//  | 0x14(ARRAY) Type ArrayShape
//  | 0x1D(SZARRAY) CustomMod* Type
//  | 0x15(GENERICINST)

/// II.23.2.14
pub enum TypeSpecSig {
    SZArray(TypeSig),
}

pub enum InnerLocalVarType {
    // currently no CustomMod or Constraint
    Default(TypeSig),
    ByRef(TypeSig),
    // currently no TYPEBYREF
}

pub enum IrSig {
    Method(MethodSigFlag, Vec<ParamType>, RetType),
    /// II.23.2.4
    Field(TypeSig),
    /// II.23.2.6
    LocalVar(Vec<InnerLocalVarType>),
    /// II.23.2.14
    TypeSpec(TypeSpecSig),
}

impl IrFmt for TypeSig {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        match self {
            TypeSig::Boolean => write!(f, "bool"),
            TypeSig::Char => write!(f, "char"),
            TypeSig::I1 => write!(f, "i1"),
            TypeSig::U1 => write!(f, "u1"),
            TypeSig::I4 => write!(f, "i4"),
            TypeSig::U4 => write!(f, "u4"),
            TypeSig::I8 => write!(f, "i8"),
            TypeSig::U8 => write!(f, "u8"),
            TypeSig::R4 => write!(f, "r4"),
            TypeSig::R8 => write!(f, "r8"),
            TypeSig::I => write!(f, "i"),
            TypeSig::U => write!(f, "u"),
            TypeSig::Class(t) => fmt_tok(*t, f, ctx),
            TypeSig::SZArray(ty) => {
                ty.fmt(f, ctx)?;
                write!(f, "[]")
            }
            TypeSig::String => write!(f, "string"),
        }
    }
}

impl ISerializable for TypeSig {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            TypeSig::Boolean => ELEMENT_TYPE_BOOLEAN.serialize(buf),
            TypeSig::Char => ELEMENT_TYPE_CHAR.serialize(buf),
            TypeSig::I1 => ELEMENT_TYPE_I1.serialize(buf),
            TypeSig::U1 => ELEMENT_TYPE_U1.serialize(buf),
            TypeSig::I4 => ELEMENT_TYPE_I4.serialize(buf),
            TypeSig::U4 => ELEMENT_TYPE_U4.serialize(buf),
            TypeSig::I8 => ELEMENT_TYPE_I8.serialize(buf),
            TypeSig::U8 => ELEMENT_TYPE_U8.serialize(buf),
            TypeSig::R4 => ELEMENT_TYPE_R4.serialize(buf),
            TypeSig::R8 => ELEMENT_TYPE_R8.serialize(buf),
            TypeSig::I => ELEMENT_TYPE_I.serialize(buf),
            TypeSig::U => ELEMENT_TYPE_U.serialize(buf),
            TypeSig::Class(t) => {
                ELEMENT_TYPE_CLASS.serialize(buf);
                t.serialize(buf);
            }
            TypeSig::SZArray(ty) => {
                ELEMENT_TYPE_SZARRAY.serialize(buf);
                ty.serialize(buf);
            }
            TypeSig::String => ELEMENT_TYPE_STRING.serialize(buf),
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let code = u8::deserialize(buf);
        match code {
            ELEMENT_TYPE_BOOLEAN => TypeSig::Boolean,
            ELEMENT_TYPE_CHAR => TypeSig::Char,
            ELEMENT_TYPE_I1 => TypeSig::I1,
            ELEMENT_TYPE_U1 => TypeSig::U1,
            ELEMENT_TYPE_I4 => TypeSig::I4,
            ELEMENT_TYPE_U4 => TypeSig::U4,
            ELEMENT_TYPE_I8 => TypeSig::I8,
            ELEMENT_TYPE_U8 => TypeSig::U8,
            ELEMENT_TYPE_R4 => TypeSig::R4,
            ELEMENT_TYPE_R8 => TypeSig::R8,
            ELEMENT_TYPE_I => TypeSig::I,
            ELEMENT_TYPE_U => TypeSig::U,
            ELEMENT_TYPE_CLASS => TypeSig::Class(u32::deserialize(buf)),
            ELEMENT_TYPE_SZARRAY => TypeSig::SZArray(Box::new(TypeSig::deserialize(buf))),
            ELEMENT_TYPE_STRING => TypeSig::String,
            _ => panic!("Cannot recognize TypeSig with code {:0X}", code),
        }
    }
}

/*
impl IrFmt for ArrayShapeSig {
    fn fmt(&self, f: &mut fmt::Formatter, _: &IrFile) -> fmt::Result {
        write!(f, "[")?;
        for i in 0..(self.rank as usize) {
            if i != 0 {
                write!(f, ",")?;
            }
            if i < self.sizes.len() {
                write!(f, "{}", self.sizes[i])?;
            }
        }
        write!(f, "]")
    }
}

impl ISerializable for ArrayShapeSig {
    fn serialize(&self, buf: &mut Vec<u8>) {
        self.rank.serialize(buf);
        self.sizes.serialize(buf);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let rank = u32::deserialize(buf);
        let sizes = Vec::deserialize(buf);
        ArrayShapeSig { rank, sizes }
    }
}
*/

impl IrFmt for InnerLocalVarType {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        match self {
            InnerLocalVarType::Default(ty) => ty.fmt(f, ctx),
            InnerLocalVarType::ByRef(ty) => {
                write!(f, "&")?;
                ty.fmt(f, ctx)
            }
        }
    }
}

impl_vec_serde!(InnerLocalVarType);

impl ISerializable for InnerLocalVarType {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            InnerLocalVarType::Default(ty) => {
                ty.serialize(buf);
            }
            InnerLocalVarType::ByRef(ty) => {
                ELEMENT_TYPE_BYREF.serialize(buf);
                ty.serialize(buf);
            }
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        // no ambiguouty, TypeSig will not stated with ELEMENT_TYPE_BYREF
        if let Ok(EleType::ByRef) = EleType::try_from(buf.peek_byte()) {
            u8::deserialize(buf);
            InnerLocalVarType::ByRef(TypeSig::deserialize(buf))
        } else {
            InnerLocalVarType::Default(TypeSig::deserialize(buf))
        }
    }
}

impl IrFmt for IrSig {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        match self {
            Self::Method(flag, ps, ret) => {
                if flag.has_flag(MethodSigFlagTag::HasThis) {
                    write!(f, "instance ")?;
                }
                write!(f, "(")?;
                for (i, p) in ps.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    p.fmt(f, ctx)?;
                }
                write!(f, ") -> ")?;
                ret.fmt(f, ctx)
            }
            Self::Field(t) => t.fmt(f, ctx),
            Self::LocalVar(vars) => {
                write!(f, "(")?;
                for (i, t) in vars.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    t.fmt(f, ctx)?;
                    write!(f, " v{}", i)?;
                }
                write!(f, ")")
            }
            Self::TypeSpec(spec) => match spec {
                TypeSpecSig::SZArray(ty) => {
                    ty.fmt(f, ctx)?;
                    write!(f, "[]")
                }
            },
        }
    }
}

impl ISerializable for IrSig {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            IrSig::Method(flag, ps, ret) => {
                flag.flag.serialize(buf);
                // param count
                (ps.len() as u32).serialize(buf);
                ret.serialize(buf);
                for p in ps.iter() {
                    p.serialize(buf);
                }
            }
            IrSig::Field(t) => {
                0x06u8.serialize(buf);
                t.serialize(buf);
            }
            IrSig::LocalVar(vars) => {
                0x07u8.serialize(buf);
                vars.serialize(buf);
            }
            IrSig::TypeSpec(typespec) => match typespec {
                TypeSpecSig::SZArray(ty) => {
                    ELEMENT_TYPE_SZARRAY.serialize(buf);
                    ty.serialize(buf);
                }
            },
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let code = u8::deserialize(buf);
        match code {
            0x06u8 => IrSig::Field(TypeSig::deserialize(buf)),
            0x07u8 => IrSig::LocalVar(Vec::deserialize(buf)),
            _ => {
                if let Ok(ele_ty) = EleType::try_from(code) {
                    match ele_ty {
                        EleType::SZArray => {
                            return IrSig::TypeSpec(TypeSpecSig::SZArray(TypeSig::deserialize(
                                buf,
                            )));
                        }
                        _ => {}
                    }
                }
                // TODO: check flag validity
                let method_sig_flag = MethodSigFlag::from(code);

                let param_count = u32::deserialize(buf);
                let ret = RetType::deserialize(buf);
                let ps = (0..param_count)
                    .into_iter()
                    .map(|_| ParamType::deserialize(buf))
                    .collect();

                IrSig::Method(method_sig_flag, ps, ret)
            }
        }
    }
}
