use std::convert::TryFrom;
use std::fmt;

use super::super::bc_serde::{IDeserializer, ISerializable};
use super::super::file::IrFile;
use super::super::text_serde::IrFmt;
use super::TypeSig;
use super::{EleType, ELEMENT_TYPE_BYREF, ELEMENT_TYPE_VOID};

const METHOD_SIG_DEFAULT_FLAG: u8 = 0x0;
// c: 0x1
// stdcall: 0x2
// thiscall: 0x3
// fastcall: 0x4

// vararg: 0x5

// generic: 0x10
const METHOD_SIG_HASTHIS_FLAG: u8 = 0x20;
// explicit this: 0x40

pub enum MethodSigFlagTag {
    HasThis,
    Default,
}

#[derive(Clone)]
pub struct MethodSigFlag {
    pub flag: u8,
}

pub struct ParamType {
    // currently no CustomMod
    pub ty: InnerParamType,
}

pub enum InnerParamType {
    Default(TypeSig),
    ByRef(TypeSig),
    // currently no TYPEBYREF
}

pub struct RetType {
    // currently no CustomMod
    pub ty: InnerRetType,
}

pub enum InnerRetType {
    Default(TypeSig),
    ByRef(TypeSig),
    // currently no TYPEBYREF
    Void,
}

impl TryFrom<u8> for MethodSigFlagTag {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            METHOD_SIG_DEFAULT_FLAG => Ok(Self::Default),
            METHOD_SIG_HASTHIS_FLAG => Ok(Self::HasThis),
            _ => Err("Invalid value for MethodSigFlag"),
        }
    }
}

impl From<MethodSigFlagTag> for u8 {
    fn from(value: MethodSigFlagTag) -> Self {
        match value {
            MethodSigFlagTag::Default => METHOD_SIG_DEFAULT_FLAG,
            MethodSigFlagTag::HasThis => METHOD_SIG_HASTHIS_FLAG,
        }
    }
}

impl MethodSigFlag {
    pub fn from(flag: u8) -> MethodSigFlag {
        MethodSigFlag { flag }
    }

    pub fn new(flag: MethodSigFlagTag) -> MethodSigFlag {
        MethodSigFlag {
            flag: u8::from(flag),
        }
    }

    pub fn set_flag(&mut self, flag: MethodSigFlagTag) {
        self.flag |= u8::from(flag);
    }

    pub fn has_flag(&self, flag: MethodSigFlagTag) -> bool {
        (self.flag & u8::from(flag)) != 0
    }
}

impl IrFmt for ParamType {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        match &self.ty {
            InnerParamType::Default(ty) => ty.fmt(f, ctx),
            InnerParamType::ByRef(ty) => {
                write!(f, "&")?;
                ty.fmt(f, ctx)
            }
        }
    }
}

impl ISerializable for ParamType {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match &self.ty {
            InnerParamType::Default(ty) => ty.serialize(buf),
            InnerParamType::ByRef(ty) => {
                ELEMENT_TYPE_BYREF.serialize(buf);
                ty.serialize(buf);
            }
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        // no ambiguouty, TypeSig will not stated with ELEMENT_TYPE_BYREF
        let ty = if let Ok(EleType::ByRef) = EleType::try_from(buf.peek_byte()) {
            u8::deserialize(buf);
            InnerParamType::ByRef(TypeSig::deserialize(buf))
        } else {
            InnerParamType::Default(TypeSig::deserialize(buf))
        };
        ParamType { ty }
    }
}

impl IrFmt for RetType {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        match &self.ty {
            InnerRetType::Default(ty) => ty.fmt(f, ctx),
            InnerRetType::ByRef(ty) => {
                write!(f, "&")?;
                ty.fmt(f, ctx)
            }
            InnerRetType::Void => write!(f, "void"),
        }
    }
}

impl ISerializable for RetType {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match &self.ty {
            InnerRetType::Default(ty) => ty.serialize(buf),
            InnerRetType::ByRef(ty) => {
                ELEMENT_TYPE_BYREF.serialize(buf);
                ty.serialize(buf);
            }
            InnerRetType::Void => ELEMENT_TYPE_VOID.serialize(buf),
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        // no ambiguouty, TypeSig will not stated with ELEMENT_TYPE_BYREF or ELEMENT_TYPE_VOID
        let peek = EleType::try_from(buf.peek_byte());
        let ty = if let Ok(EleType::ByRef) = peek {
            u8::deserialize(buf);
            InnerRetType::ByRef(TypeSig::deserialize(buf))
        } else if let Ok(EleType::Void) = peek {
            u8::deserialize(buf);
            InnerRetType::Void
        } else {
            InnerRetType::Default(TypeSig::deserialize(buf))
        };
        RetType { ty }
    }
}
