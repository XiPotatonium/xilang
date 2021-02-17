use super::super::flag::Flag;
use super::super::inst::Inst;
use super::class_file::*;

impl ClassFile {
    fn get_str(&self, idx: u16) -> &str {
        match &self[idx] {
            Constant::Class(name_idx) => self.get_str(*name_idx),
            Constant::Utf8(s) => s,
            _ => unimplemented!(),
        }
    }

    fn get_ir_str(&self, idx: u16) -> String {
        match &self[idx] {
            Constant::Integer(v) => format!("{}", v),
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

    fn get_inst_str(&self, inst: &Inst) -> String {
        match inst {
            Inst::IConstM1 => String::from("iconst_m1"),
            Inst::IConst0 => String::from("iconst_0"),
            Inst::IConst1 => String::from("iconst_1"),
            Inst::IConst2 => String::from("iconst_2"),
            Inst::IConst3 => String::from("iconst_3"),
            Inst::IConst4 => String::from("iconst_4"),
            Inst::IConst5 => String::from("iconst_5"),
            Inst::BIPush(v) => format!("bipush {}", v),
            Inst::LdC(idx) => self.get_ir_str(*idx),
            Inst::ILoad(offset) => format!("iload {}", offset),
            Inst::ALoad(offset) => format!("aload {}", offset),
            Inst::ILoad0 => String::from("iload_0"),
            Inst::ILoad1 => String::from("iload_1"),
            Inst::ILoad2 => String::from("iload_2"),
            Inst::ILoad3 => String::from("iload_3"),
            Inst::ALoad0 => String::from("aload_0"),
            Inst::ALoad1 => String::from("aload_1"),
            Inst::ALoad2 => String::from("aload_2"),
            Inst::ALoad3 => String::from("aload_3"),
            Inst::IStore(offset) => format!("istore {}", offset),
            Inst::AStore(offset) => format!("astore {}", offset),
            Inst::IStore0 => String::from("istore_0"),
            Inst::IStore1 => String::from("istore_1"),
            Inst::IStore2 => String::from("istore_2"),
            Inst::IStore3 => String::from("istore_3"),
            Inst::AStore0 => String::from("astore_0"),
            Inst::AStore1 => String::from("astore_1"),
            Inst::AStore2 => String::from("astore_2"),
            Inst::AStore3 => String::from("astore_3"),
            Inst::Pop => String::from("pop"),
            Inst::Pop2 => String::from("pop2"),
            Inst::IAdd => String::from("iadd"),
            Inst::Return => String::from("return"),
            Inst::GetStatic(idx) => format!("getstatic {}", self.get_ir_str(*idx)),
            Inst::PutStatic(idx) => format!("putstatic {}", self.get_ir_str(*idx)),
            Inst::GetField(idx) => format!("getfield {}", self.get_ir_str(*idx)),
            Inst::PutField(idx) => format!("putfield {}", self.get_ir_str(*idx)),
            Inst::InvokeVirtual(idx) => format!("invokevirtual {}", self.get_ir_str(*idx)),
            Inst::InvokeSpecial(idx) => format!("invokespecial {}", self.get_ir_str(*idx)),
            Inst::InvokeStatic(idx) => format!("invokestatic {}", self.get_ir_str(*idx)),
            Inst::New(idx) => format!("new {}", self.get_ir_str(*idx)),
            Inst::ArrayLength => String::from("arraylength"),
        }
    }

    pub fn to_text(&self) -> String {
        let mut ret = format!(
            "Class version: {}.{}\n\n",
            self.major_version, self.minor_version
        );
        let flag = Flag::new(self.access_flags);
        ret.push_str(&format!(
            "{} class {} ",
            flag,
            self.get_str(self.this_class)
        ));

        if self.interfaces.len() != 0 {
            ret.push_str("implementes ");
            for _ in self.interfaces.iter() {
                unimplemented!();
            }
        }

        ret.push_str("{\n");

        for field in self.fields.iter() {
            let flag = Flag::new(field.access_flags);
            ret.push_str(&format!(
                "    {} {} {}\n\n",
                flag,
                self.get_str(field.name_index),
                self.get_str(field.descriptor_index)
            ));
        }

        for method in self.methods.iter() {
            let flag = Flag::new(method.access_flags);
            ret.push_str(&format!(
                "    {} {} {}\n",
                flag,
                self.get_str(method.name_index),
                self.get_str(method.descriptor_index)
            ));

            ret.push_str(&format!("        Locals stack: {}\n", method.locals_stack));

            for inst in method.insts.iter() {
                ret.push_str("\n        ");
                ret.push_str(&self.get_inst_str(inst));
            }

            ret.push_str("\n\n");
        }

        ret.push_str("}\n");

        ret
    }

    pub fn from_text(text: &str) -> ClassFile {
        unimplemented!();
    }
}
