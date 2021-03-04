mod ir_parser;

use super::flag::*;
use super::inst::Inst;
use super::ir_file::*;

use std::fmt;

impl IrFile {
    pub fn write_field(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
        field_i: usize,
    ) -> fmt::Result {
        let field = &self.field_tbl[field_i];
        let flag = FieldFlag::new(field.flag);
        write!(
            f,
            "\n{}.field {} {} {}",
            " ".repeat(indent * 4),
            flag,
            self.get_str(field.name),
            self.get_blob_repr(field.signature)
        )
    }

    pub fn write_method(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
        method_i: usize,
        is_entrypoint: bool,
    ) -> fmt::Result {
        let method = &self.method_tbl[method_i];
        let code = &self.codes[method_i];
        let flag = MethodFlag::new(method.flag);
        write!(
            f,
            "\n\n{}.method {} {} {}\n",
            " ".repeat(indent * 4),
            flag,
            self.get_str(method.name),
            self.get_blob_repr(method.signature)
        )?;

        write!(f, "{}.locals\t{}", " ".repeat(indent * 8), method.locals)?;
        if is_entrypoint {
            write!(f, "\n{}.entrypoint", " ".repeat(indent * 8))?;
        }

        let mut offset = 0;
        for inst in code.iter() {
            write!(f, "\n{}", " ".repeat(indent * 8))?;
            inst.fmt(f, self, offset)?;
            offset += inst.size();
        }

        Ok(())
    }
}

impl fmt::Display for IrFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".version {}.{}\n",
            self.major_version, self.minor_version
        )?;
        let entrypoint = if let Some(m) = self.mod_tbl.first() {
            write!(f, ".mod {}", self.get_str(m.name))?;
            m.entrypoint & !TBL_TAG_MASK
        } else {
            0
        };

        let (mut field_i, mut method_i) = if let Some(c0) = self.class_tbl.first() {
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

        for (class_i, class) in self.class_tbl.iter().enumerate() {
            let (field_lim, method_lim) = if class_i + 1 >= self.class_tbl.len() {
                // last class
                (self.field_tbl.len(), self.method_tbl.len())
            } else {
                let next_class = &self.class_tbl[class_i + 1];
                (
                    next_class.fields as usize - 1,
                    next_class.methods as usize - 1,
                )
            };

            let flag = TypeFlag::new(class.flag);
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

            Inst::Call(idx) => write!(f, "call {}", c.get_tbl_entry_repr(*idx)),
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

            Inst::CallVirt(idx) => write!(f, "callvirt {}", c.get_tbl_entry_repr(*idx)),
            Inst::NewObj(idx) => write!(f, "newobj {}", c.get_tbl_entry_repr(*idx)),
            Inst::LdFld(idx) => write!(f, "ldfld {}", c.get_tbl_entry_repr(*idx)),
            Inst::StFld(idx) => write!(f, "stfld {}", c.get_tbl_entry_repr(*idx)),
            Inst::LdSFld(idx) => write!(f, "ldsfld {}", c.get_tbl_entry_repr(*idx)),
            Inst::StSFld(idx) => write!(f, "stsfld {}", c.get_tbl_entry_repr(*idx)),
        }
    }
}
