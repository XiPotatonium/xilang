use super::flag::*;
use super::inst::Inst;
use super::ir_file::*;

use std::fmt;

impl IrFile {
    fn get_str(&self, idx: u32) -> &str {
        match &self[idx] {
            Constant::Class(name_idx) => self.get_str(*name_idx),
            Constant::Utf8(s) => s,
            _ => unimplemented!(),
        }
    }

    fn get_ir_str(&self, idx: u32) -> String {
        match &self[idx] {
            // TODO restore escape chars
            Constant::String(utf8_idx) => format!("\"{}\"", self.get_str(*utf8_idx)),
            Constant::Fieldref(class_idx, name_and_ty) => format!(
                "{}::{}",
                self.get_ir_str(*class_idx),
                self.get_ir_str(*name_and_ty)
            ),
            Constant::Methodref(class_idx, name_and_ty) => format!(
                "{}::{}",
                self.get_ir_str(*class_idx),
                self.get_ir_str(*name_and_ty)
            ),
            Constant::Class(class_idx) => format!("{}", self.get_str(*class_idx)),
            Constant::NameAndType(name, ty) => {
                format!("{}: {}", self.get_str(*name), self.get_str(*ty))
            }
            _ => unimplemented!(),
        }
    }

    pub fn from_text(text: &str) -> IrFile {
        unimplemented!();
    }

    pub fn write_field(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
        field_i: usize,
    ) -> fmt::Result {
        let field = &self.field_tbl[field_i - 1];
        let flag = FieldFlag::new(field.flag);
        write!(
            f,
            "\n{}.field {} {} {}",
            " ".repeat(indent * 2),
            flag,
            self.get_str(field.name),
            self.get_str(field.descriptor)
        )
    }

    pub fn write_method(
        &self,
        f: &mut fmt::Formatter<'_>,
        indent: usize,
        method_i: usize,
        is_entrypoint: bool,
    ) -> fmt::Result {
        let method = &self.method_tbl[method_i - 1];
        let code = &self.codes[method_i - 1];
        let flag = MethodFlag::new(method.flag);
        write!(
            f,
            "\n\n{}.method {} {} {}\n",
            " ".repeat(indent * 4),
            flag,
            self.get_str(method.name),
            self.get_str(method.descriptor)
        )?;

        write!(f, "{}.locals\t{}", " ".repeat(indent * 8), method.locals)?;
        if is_entrypoint {
            write!(f, "\n{}.entrypoint", " ".repeat(indent * 8))?;
        }

        for inst in code.iter() {
            write!(f, "\n{}", " ".repeat(indent * 8))?;
            inst.fmt(f, self)?;
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

        let mut entrypoint = 0usize;
        if let Some(c) = self.crate_tbl.first() {
            write!(f, ".crate {}\n", self.get_str(c.name))?;
            entrypoint = c.entrypoint as usize;
        }

        if let Some(m) = self.mod_tbl.first() {
            write!(f, ".mod {}\n", self.get_str(m.name))?;
        }

        let mut field_lim = if let Some(c0) = self.class_tbl.first() {
            c0.fields as usize
        } else {
            self.field_tbl.len()
        };
        let mut method_lim = if let Some(c0) = self.class_tbl.first() {
            c0.methods as usize
        } else {
            self.method_tbl.len()
        };
        let mut field_i = 1;
        let mut method_i = 1;

        while field_i < field_lim {
            self.write_field(f, 0, field_i)?;
            field_i += 1;
        }

        while method_i < method_lim {
            self.write_method(f, 0, method_i, method_i == entrypoint)?;
            method_i += 1;
        }

        for class_i in (0..self.class_tbl.len()).into_iter() {
            if class_i + 1 >= self.class_tbl.len() {
                // last class
                field_lim = self.field_tbl.len();
                method_lim = self.method_tbl.len();
            }

            let class = &self.class_tbl[class_i];
            let flag = TypeFlag::new(class.flag);
            write!(f, "\n\n\n.class {} {}", flag, self.get_str(class.name))?;

            while field_i < field_lim {
                self.write_field(f, 1, field_i)?;
                field_i += 1;
            }

            while method_i < method_lim {
                self.write_method(f, 1, method_i, method_i == entrypoint)?;
                method_i += 1;
            }
        }

        Ok(())
    }
}

impl Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, c: &IrFile) -> fmt::Result {
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
            Inst::LdCS(num) => write!(f, "ldc.i4.s {}", num),
            Inst::LdC(num) => write!(f, "ldc.i4 {}", num),

            Inst::Dup => write!(f, "dup"),
            Inst::Pop => write!(f, "pop"),

            Inst::Call(idx) => write!(f, "call {}", c.get_ir_str(*idx)),
            Inst::Ret => write!(f, "ret"),

            Inst::Add => write!(f, "add"),

            Inst::CallVirt(idx) => write!(f, "callvirt {}", c.get_ir_str(*idx)),
            Inst::New(idx) => write!(f, "new {}", c.get_ir_str(*idx)),
            Inst::LdFld(idx) => write!(f, "ldfld {}", c.get_ir_str(*idx)),
            Inst::StFld(idx) => write!(f, "stfld {}", c.get_ir_str(*idx)),
            Inst::LdSFld(idx) => write!(f, "ldsfld {}", c.get_ir_str(*idx)),
            Inst::StSFld(idx) => write!(f, "stsfld {}", c.get_ir_str(*idx)),
        }
    }
}
