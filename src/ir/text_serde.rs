use super::class_file::*;
use super::flag::*;
use super::inst::Inst;

use std::fmt;

impl ClassFile {
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

    pub fn from_text(text: &str) -> ClassFile {
        unimplemented!();
    }
}

impl fmt::Display for ClassFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ".version: {}.{}\n",
            self.major_version, self.minor_version
        )?;
        let flag = TypeFlag::new(self.access_flags);
        write!(f, ".class {} {}", flag, self.get_str(self.this_class))?;

        for field in self.fields.iter() {
            let flag = FieldFlag::new(field.access_flags);
            write!(
                f,
                "\n\n    .field {} {} {}",
                flag,
                self.get_str(field.name_index),
                self.get_str(field.descriptor_index)
            )?;
        }

        for method in self.methods.iter() {
            let flag = MethodFlag::new(method.access_flags);
            write!(
                f,
                "\n\n    .method {} {} {}\n",
                flag,
                self.get_str(method.name_index),
                self.get_str(method.descriptor_index)
            )?;

            write!(f, "        .locals\t{}", method.locals)?;

            for inst in method.insts.iter() {
                write!(f, "\n        ")?;
                inst.fmt(f, self)?;
            }
        }

        Ok(())
    }
}

impl Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, c: &ClassFile) -> fmt::Result {
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
