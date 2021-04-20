use super::Inst;
use crate::file::IrFile;
use crate::tok::fmt_tok;

use std::fmt;

impl Inst {
    pub fn fmt(&self, f: &mut fmt::Formatter<'_>, c: &IrFile, i: usize) -> fmt::Result {
        write!(f, "IL_{:0>4X}:  ", i)?;
        match self {
            Inst::Nop => write!(f, "nop"),

            Inst::LdArg0 => write!(f, "ldarg.0"),
            Inst::LdArg1 => write!(f, "ldarg.1"),
            Inst::LdArg2 => write!(f, "ldarg.2"),
            Inst::LdArg3 => write!(f, "ldarg.3"),
            Inst::LdArgS(idx) => write!(f, "ldarg.s {}", idx),

            Inst::StArgS(idx) => write!(f, "starg.s {}", idx),

            Inst::LdLoc0 => write!(f, "ldloc.0"),
            Inst::LdLoc1 => write!(f, "ldloc.1"),
            Inst::LdLoc2 => write!(f, "ldloc.2"),
            Inst::LdLoc3 => write!(f, "ldloc.3"),
            Inst::LdLocS(idx) => write!(f, "ldloc.s {}", idx),
            Inst::LdLoc(idx) => write!(f, "ldloc {}", idx),
            Inst::StLoc0 => write!(f, "stloc.0"),
            Inst::StLoc1 => write!(f, "stloc.1"),
            Inst::StLoc2 => write!(f, "stloc.2"),
            Inst::StLoc3 => write!(f, "stloc.3"),
            Inst::StLocS(idx) => write!(f, "stloc.s {}", idx),
            Inst::StLoc(idx) => write!(f, "stloc {}", idx),

            Inst::LdNull => write!(f, "ldnull"),
            Inst::LdCM1 => write!(f, "ldc.i4.m1"),
            Inst::LdC0 => write!(f, "ldc.i4.0"),
            Inst::LdC1 => write!(f, "ldc.i4.1"),
            Inst::LdC2 => write!(f, "ldc.i4.2"),
            Inst::LdC3 => write!(f, "ldc.i4.3"),
            Inst::LdC4 => write!(f, "ldc.i4.4"),
            Inst::LdC5 => write!(f, "ldc.i4.5"),
            Inst::LdC6 => write!(f, "ldc.i4.6"),
            Inst::LdC7 => write!(f, "ldc.i4.7"),
            Inst::LdC8 => write!(f, "ldc.i4.8"),
            Inst::LdCI4S(num) => write!(f, "ldc.i4.s {}", num),
            Inst::LdCI4(num) => write!(f, "ldc.i4 {}", num),

            Inst::Dup => write!(f, "dup"),
            Inst::Pop => write!(f, "pop"),

            Inst::Call(tok) => {
                write!(f, "call ")?;
                fmt_tok(*tok, f, c)
            }
            Inst::Ret => write!(f, "ret"),

            Inst::Br(offset) => write!(f, "br IL_{:0>4X}", (i + self.size()) as i32 + offset),
            Inst::BrFalse(offset) => {
                write!(f, "brfalse IL_{:0>4X}", (i + self.size()) as i32 + offset)
            }
            Inst::BrTrue(offset) => {
                write!(f, "brtrue IL_{:0>4X}", (i + self.size()) as i32 + offset)
            }
            Inst::BEq(offset) => write!(f, "beq IL_{:0>4X}", (i + self.size()) as i32 + offset),
            Inst::BGe(offset) => write!(f, "bge IL_{:0>4X}", (i + self.size()) as i32 + offset),
            Inst::BGt(offset) => write!(f, "bgt IL_{:0>4X}", (i + self.size()) as i32 + offset),
            Inst::BLe(offset) => write!(f, "ble IL_{:0>4X}", (i + self.size()) as i32 + offset),
            Inst::BLt(offset) => write!(f, "blt IL_{:0>4X}", (i + self.size()) as i32 + offset),

            Inst::CEq => write!(f, "ceq"),
            Inst::CGt => write!(f, "cgt"),
            Inst::CLt => write!(f, "clt"),

            Inst::Add => write!(f, "add"),
            Inst::Sub => write!(f, "sub"),
            Inst::Mul => write!(f, "mul"),
            Inst::Div => write!(f, "div"),
            Inst::Rem => write!(f, "rem"),

            Inst::Neg => write!(f, "neg"),

            Inst::CallVirt(tok) => {
                write!(f, "callvirt ")?;
                fmt_tok(*tok, f, c)
            }
            Inst::NewObj(tok) => {
                write!(f, "newobj ")?;
                fmt_tok(*tok, f, c)
            }
            Inst::LdFld(tok) => {
                write!(f, "ldfld ")?;
                fmt_tok(*tok, f, c)
            }
            Inst::StFld(tok) => {
                write!(f, "stfld ")?;
                fmt_tok(*tok, f, c)
            }
            Inst::LdSFld(tok) => {
                write!(f, "ldsfld ")?;
                fmt_tok(*tok, f, c)
            }
            Inst::StSFld(tok) => {
                write!(f, "stsfld ")?;
                fmt_tok(*tok, f, c)
            }
        }
    }
}
