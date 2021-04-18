use super::bc_serde::{IDeserializer, ISerializable};
use super::file::IrFile;
use super::text_serde::IrFmt;
use super::tok::fmt_tok;

use std::convert::TryFrom;
use std::fmt;

/// II.23.1.16 Element types used in signature
pub enum EleType {
    Void,
    Boolean,
    Char,
    I1,
    U1,
    I2,
    U2,
    I4,
    U4,
    I8,
    U8,
    R4,
    R8,
    ByRef(Box<EleType>),
    // TypeDef or TypeRef token
    Class(u32),
}

const METHOD_SIG_DEFAULT_FLAG: u8 = 0x0;
const METHOD_SIG_HASTHIS_FLAG: u8 = 0x20;

pub enum MethodSigFlagTag {
    HasThis,
    Default,
}

#[derive(Clone)]
pub struct MethodSigFlag {
    pub flag: u8,
}

pub enum IrSig {
    Method(MethodSigFlag, Vec<EleType>, EleType),
    Field(EleType),
    LocalVar(Vec<EleType>),
}

impl IrFmt for EleType {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        match self {
            EleType::Void => write!(f, "void"),
            EleType::Boolean => write!(f, "bool"),
            EleType::Char => write!(f, "char"),
            EleType::I1 => write!(f, "i1"),
            EleType::U1 => write!(f, "u1"),
            EleType::I2 => write!(f, "i2"),
            EleType::U2 => write!(f, "u2"),
            EleType::I4 => write!(f, "i4"),
            EleType::U4 => write!(f, "u4"),
            EleType::I8 => write!(f, "i8"),
            EleType::U8 => write!(f, "u8"),
            EleType::R4 => write!(f, "r4"),
            EleType::R8 => write!(f, "r8"),
            EleType::ByRef(t) => {
                if let EleType::Class(_) = t.as_ref() {
                } else {
                    unimplemented!();
                }
                t.fmt(f, ctx)
            }
            EleType::Class(t) => fmt_tok(*t, f, ctx),
        }
    }
}

impl ISerializable for EleType {
    fn serialize(&self, buf: &mut Vec<u8>) {
        match self {
            EleType::Void => 0x01u8.serialize(buf),
            EleType::Boolean => 0x02u8.serialize(buf),
            EleType::Char => 0x03u8.serialize(buf),
            EleType::I1 => 0x04u8.serialize(buf),
            EleType::U1 => 0x05u8.serialize(buf),
            EleType::I2 => 0x06u8.serialize(buf),
            EleType::U2 => 0x07u8.serialize(buf),
            EleType::I4 => 0x08u8.serialize(buf),
            EleType::U4 => 0x09u8.serialize(buf),
            EleType::I8 => 0x0Au8.serialize(buf),
            EleType::U8 => 0x0Bu8.serialize(buf),
            EleType::R4 => 0x0Cu8.serialize(buf),
            EleType::R8 => 0x0Du8.serialize(buf),
            EleType::ByRef(t) => {
                0x10u8.serialize(buf);
                t.serialize(buf);
            }
            EleType::Class(t) => {
                0x12u8.serialize(buf);
                t.serialize(buf);
            }
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let code = u8::deserialize(buf);
        match code {
            0x01 => EleType::Void,
            0x02 => EleType::Boolean,
            0x03 => EleType::Char,
            0x04 => EleType::I1,
            0x05 => EleType::U1,
            0x06 => EleType::I2,
            0x07 => EleType::U2,
            0x08 => EleType::I4,
            0x09 => EleType::U4,
            0x0A => EleType::I8,
            0x0B => EleType::U8,
            0x0C => EleType::R4,
            0x0D => EleType::R8,
            0x10 => EleType::ByRef(Box::new(EleType::deserialize(buf))),
            0x12 => EleType::Class(u32::deserialize(buf)),
            _ => panic!("Cannot recognize EleType with code {:0X}", code),
        }
    }
}

impl_vec_serde!(EleType);

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
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> Self {
        let code = u8::deserialize(buf);
        match code {
            0x06u8 => IrSig::Field(EleType::deserialize(buf)),
            0x07u8 => IrSig::LocalVar(Vec::deserialize(buf)),
            _ => {
                // TODO: check flag validity
                let param_count = u32::deserialize(buf);
                let ret = EleType::deserialize(buf);
                let ps = (0..param_count)
                    .into_iter()
                    .map(|_| EleType::deserialize(buf))
                    .collect();

                IrSig::Method(MethodSigFlag::from(code), ps, ret)
            }
        }
    }
}
