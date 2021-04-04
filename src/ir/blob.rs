use crate::ir::file::{IrFile, IrFmt};
use crate::ir::tok::fmt_tok;
use std::fmt;

pub enum Blob {
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
    Obj(u32),
    Func(Vec<u32>, u32),
    Array(u32),
}

impl IrFmt for Blob {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result {
        match self {
            Self::Void => write!(f, "V"),
            Self::Bool => write!(f, "Z"),
            Self::Char => write!(f, "C"),
            Self::U8 => write!(f, "B"),
            Self::I8 => write!(f, "b"),
            Self::U16 => write!(f, "S"),
            Self::I16 => write!(f, "s"),
            Self::U32 => write!(f, "I"),
            Self::I32 => write!(f, "i"),
            Self::U64 => write!(f, "L"),
            Self::I64 => write!(f, "l"),
            Self::UNative => write!(f, "N"),
            Self::INative => write!(f, "n"),
            Self::F32 => write!(f, "F"),
            Self::F64 => write!(f, "f"),
            Self::Obj(tok) => {
                write!(f, "O")?;
                fmt_tok(*tok, f, ctx)?;
                write!(f, ";")
            }

            Self::Func(ps, ret_ty) => {
                write!(f, "(")?;
                for p in ps.iter() {
                    ctx.blob_heap[*p as usize].fmt(f, ctx)?;
                }
                write!(f, ")")?;
                ctx.blob_heap[*ret_ty as usize].fmt(f, ctx)
            }
            Blob::Array(inner) => {
                write!(f, "[")?;
                ctx.blob_heap[*inner as usize].fmt(f, ctx)
            }
        }
    }
}
