use super::attrib::*;
use super::file::IrFile;
use super::inst::Inst;
use super::member::MemberForwarded;
use super::tok::fmt_tok;

use std::fmt;

pub trait IrFmt {
    fn fmt(&self, f: &mut fmt::Formatter, ctx: &IrFile) -> fmt::Result;
}

impl IrFile {
    pub fn write_field(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
        field_i: usize,
    ) -> fmt::Result {
        let field = &self.field_tbl[field_i];
        let flag = FieldAttrib::from(field.flag);
        write!(
            f,
            "\n{}.field {} {} ",
            " ".repeat(indent * 4),
            flag,
            self.get_str(field.name),
        )?;
        self.blob_heap[field.sig as usize].fmt(f, self)
    }

    pub fn write_method(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
        method_i: usize,
        is_entrypoint: bool,
    ) -> fmt::Result {
        let method = &self.method_tbl[method_i];
        let flag = MethodAttrib::from(method.flag);
        let impl_flag = MethodImplAttrib::from(method.impl_flag);

        write!(f, "\n\n{}.method {} ", " ".repeat(indent * 4), flag)?;

        if method.body == 0 {
            // this is an external method
            // See ECMA-335 II.15.5.2

            // O(N) search, might need optimization
            let implmap_info = self.implmap_tbl.iter().find(|&info| {
                let (m_tag, m_idx) = info.get_member();
                MemberForwarded::MethodDef == m_tag && m_idx as usize == method_i + 1
            });

            if let Some(implmap_info) = implmap_info {
                let flag = PInvokeAttrib::from(implmap_info.flag);
                write!(
                    f,
                    "pinvokeimpl(\"{}\" {}) ",
                    self.get_str(self.modref_tbl[implmap_info.scope as usize - 1].name),
                    flag
                )?;
            } else {
                panic!("No implmap found for method {}", self.get_str(method.name));
            }
        }

        write!(f, "{} ", self.get_str(method.name))?;
        self.blob_heap[method.sig as usize].fmt(f, self)?;
        // TODO: display param name and flag
        write!(f, " {}", impl_flag)?;

        if method.body != 0 {
            // has body
            let body = &self.codes[method.body as usize - 1];

            write!(
                f,
                " {{\n{}.maxstacks\t{}",
                " ".repeat(indent * 8),
                body.max_stack
            )?;
            if body.locals != 0 {
                write!(f, "\n{}.locals\t", " ".repeat(indent * 8))?;
                self.blob_heap[self.stand_alone_sig_tbl[body.locals as usize - 1].sig as usize]
                    .fmt(f, self)?;
            }
            if is_entrypoint {
                write!(f, "\n{}.entrypoint", " ".repeat(indent * 8))?;
            }

            let code: Vec<Inst> = body.to_insts();

            let mut offset = 0;
            for inst in code.iter() {
                write!(f, "\n{}", " ".repeat(indent * 8))?;
                inst.fmt(f, self, offset)?;
                offset += inst.size();
            }
            write!(f, "\n{}}}", " ".repeat(indent * 4))
        } else {
            write!(f, " {{}}\n")
        }
    }
}

impl fmt::Display for IrFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".version {}.{}\n.mod {}",
            self.major_version,
            self.minor_version,
            self.get_str(self.mod_tbl[0].name)
        )?;

        let entrypoint = self.mod_tbl[0].entrypoint;

        let (mut field_i, mut method_i) = if let Some(c0) = self.typedef_tbl.first() {
            (c0.fields as usize - 1, c0.methods as usize - 1)
        } else {
            (self.field_tbl.len(), self.method_tbl.len())
        };

        for i in (0..field_i).into_iter() {
            self.write_field(f, 0, i)?;
        }

        for i in (0..method_i).into_iter() {
            self.write_method(f, 0, i, i as u32 + 1 == entrypoint)?;
        }

        for (class_i, class) in self.typedef_tbl.iter().enumerate() {
            let (field_lim, method_lim) = if class_i + 1 >= self.typedef_tbl.len() {
                // last class
                (self.field_tbl.len(), self.method_tbl.len())
            } else {
                let next_class = &self.typedef_tbl[class_i + 1];
                (
                    next_class.fields as usize - 1,
                    next_class.methods as usize - 1,
                )
            };

            let flag = TypeAttrib::from(class.flag);
            write!(f, "\n\n\n.class {} {}", flag, self.get_str(class.name))?;

            while field_i < field_lim {
                self.write_field(f, 1, field_i)?;
                field_i += 1;
            }

            while method_i < method_lim {
                self.write_method(f, 1, method_i, method_i as u32 + 1 == entrypoint)?;
                method_i += 1;
            }
        }

        Ok(())
    }
}

impl Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, c: &IrFile, i: usize) -> fmt::Result {
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
