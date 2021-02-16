use super::super::flag::Flag;
use super::super::inst::Inst;
use super::class_file::*;

impl ClassFile {
    fn get_str(&self, idx: u16) -> &str {
        match &self[idx] {
            Constant::Class(name_idx) => {
                if let Constant::Utf8(s) = &self[*name_idx] {
                    s
                } else {
                    unreachable!();
                }
            }
            _ => unimplemented!(),
        }
    }

    fn get_inst_str(&self, inst: &Inst) -> String {
        unimplemented!();
    }

    pub fn to_text(&self) -> String {
        let mut ret = format!(
            "Class version: {}.{}",
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

            for attr in method.attributes.iter() {
                match attr {
                    Attribute::Code(max_stack, insts, _, _) => {
                        ret.push_str(&format!("        Max stack: {}", max_stack));

                        for inst in insts.iter() {
                            ret.push_str("\n        ");
                            ret.push_str(&self.get_inst_str(inst));
                        }
                    }
                    _ => unimplemented!(),
                }
            }

            ret.push_str("\n\n");
        }

        ret.push_str("}\n");

        ret
    }

    pub fn to_binary(&self) -> Vec<u8> {
        unimplemented!();
    }

    pub fn from_text(text: &str) -> ClassFile {
        unimplemented!();
    }

    pub fn from_binary(bin: &[u8]) -> ClassFile {
        unimplemented!();
    }
}
