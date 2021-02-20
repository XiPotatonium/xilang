use crate::ir::flag::*;
use crate::ir::inst::Inst;
use crate::ir::ir_file::*;
use crate::ir::util::linkedlist::LinkedList;

use std::collections::HashMap;

struct BasicBlock {
    insts: Vec<Inst>,
}

pub struct MethodBuilder {
    codes: LinkedList<BasicBlock>,
    size: u16,
}

impl MethodBuilder {
    pub fn new() -> MethodBuilder {
        let mut ret = MethodBuilder {
            codes: LinkedList::new(),
            size: 0,
        };

        // there is a default bb
        ret.push_bb();

        ret
    }

    fn push_bb(&mut self) {
        self.codes.push_back(BasicBlock { insts: Vec::new() });
    }
}

impl MethodBuilder {
    pub fn add_inst(&mut self, inst: Inst) {
        self.size += inst.size();
        self.codes.back_mut().unwrap().insts.push(inst);
    }

    pub fn add_inst_stloc(&mut self, local_offset: u16) {
        self.add_inst(match local_offset {
            0 => Inst::StLoc0,
            1 => Inst::StLoc1,
            2 => Inst::StLoc2,
            3 => Inst::StLoc3,
            _ => {
                if local_offset >= u8::MIN as u16 && local_offset <= u8::MAX as u16 {
                    Inst::StLocS(local_offset as u8)
                } else {
                    Inst::StLoc(local_offset)
                }
            }
        });
    }

    pub fn add_inst_ldloc(&mut self, local_offset: u16) {
        self.add_inst(match local_offset {
            0 => Inst::LdLoc0,
            1 => Inst::LdLoc1,
            2 => Inst::LdLoc2,
            3 => Inst::LdLoc3,
            _ => {
                if local_offset >= u8::MIN as u16 && local_offset <= u8::MAX as u16 {
                    Inst::LdLocS(local_offset as u8)
                } else {
                    Inst::LdLoc(local_offset)
                }
            }
        });
    }

    pub fn add_inst_ldarg(&mut self, arg_offset: u16) {
        self.add_inst(match arg_offset {
            0 => Inst::LdArg0,
            1 => Inst::LdArg1,
            2 => Inst::LdArg2,
            3 => Inst::LdArg3,
            _ => {
                if arg_offset >= u8::MIN as u16 && arg_offset <= u8::MAX as u16 {
                    Inst::LdArgS(arg_offset as u8)
                } else {
                    unimplemented!("ldarg is not implemeneted");
                }
            }
        });
    }

    pub fn add_inst_starg(&mut self, arg_offset: u16) {
        self.add_inst(
            if arg_offset >= u8::MIN as u16 && arg_offset <= u8::MAX as u16 {
                Inst::StArgS(arg_offset as u8)
            } else {
                unimplemented!("ldarg is not implemeneted");
            },
        );
    }

    /// Push an int value to the stack
    pub fn add_inst_ldc(&mut self, value: i32) {
        self.add_inst(match value {
            -1 => Inst::LdCM1,
            0 => Inst::LdC0,
            1 => Inst::LdC1,
            2 => Inst::LdC2,
            3 => Inst::LdC3,
            4 => Inst::LdC4,
            5 => Inst::LdC5,
            6 => Inst::LdC6,
            7 => Inst::LdC7,
            8 => Inst::LdC8,
            _ => {
                if value >= i8::MIN as i32 && value <= i8::MAX as i32 {
                    Inst::LdCS(value as i8)
                } else {
                    Inst::LdC(value)
                }
            }
        });
    }
}

pub struct Builder {
    // use const map to avoid redeclaration
    // str -> utf8 idx
    utf8_map: HashMap<String, u32>,
    // utf8 idx -> String idx
    str_map: HashMap<u32, u32>,
    // <mod, name> -> Class idx
    class_map: HashMap<(u32, u32), u32>,
    mod_map: HashMap<u32, u32>,
    // (Class idx, NameAndType idx) -> Field idx
    field_map: HashMap<(u32, u32), u32>,
    // (Class idx, NameAndType idx) -> Field idx
    method_map: HashMap<(u32, u32), u32>,
    // (utf8 name idx, utf8 ty idx) -> NameAndType idx
    name_and_type_map: HashMap<(u32, u32), u32>,

    pub file: IrFile,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            utf8_map: HashMap::new(),
            str_map: HashMap::new(),
            class_map: HashMap::new(),
            mod_map: HashMap::new(),
            field_map: HashMap::new(),
            method_map: HashMap::new(),
            name_and_type_map: HashMap::new(),
            file: IrFile::new(),
        }
    }

    pub fn set_crate(&mut self, name: &str) {
        let name = self.add_const_utf8(name);
        self.file.crate_tbl.push(IrCrate {
            name,
            entrypoint: 0,
        });
    }

    pub fn set_mod(&mut self, name: &str) {
        let name = self.add_const_utf8(name);
        self.file.mod_tbl.push(IrMod { name });
    }

    pub fn add_class(&mut self, name: &str, flag: &TypeFlag) -> u32 {
        let name = self.add_const_utf8(name);
        self.file.class_tbl.push(IrClass {
            name,
            flag: flag.flag,
            fields: 0,
            methods: 0,
        });
        self.file.class_tbl.len() as u32
    }

    /// Add a field of this class
    pub fn add_field(&mut self, name: &str, ty: &str, flag: &FieldFlag) -> u32 {
        let name = self.add_const_utf8(name);
        let descriptor = self.add_const_utf8(ty);
        self.file.field_tbl.push(IrField {
            name,
            descriptor,
            flag: flag.flag,
        });
        self.file.field_tbl.len() as u32
    }

    /// Add a field of this class
    pub fn add_method(&mut self, name: &str, ty: &str, flag: &MethodFlag) -> u32 {
        let name = self.add_const_utf8(name);
        let descriptor = self.add_const_utf8(ty);
        self.file.method_tbl.push(IrMethod {
            flag: flag.flag,
            name,
            descriptor,
            locals: 0,
        });
        self.file.method_tbl.len() as u32
    }

    /// Post-Process
    ///
    /// Fill all jump instructions, concat all basic blocks
    ///
    pub fn done(&mut self, m: &mut MethodBuilder, method_idx: u32, locals_stack: u16) {
        let ir_method = &mut self.file.method_tbl[(method_idx - 1) as usize];
        // fill jump instructions

        // concat basic blocks
        let mut code: Vec<Inst> = Vec::new();
        for bb in m.codes.iter_mut() {
            code.append(&mut bb.insts);
        }
        ir_method.locals = locals_stack;
        self.file.codes.push(code);
    }
}

// Const values
impl Builder {
    pub fn add_const_string(&mut self, v: &str) -> u32 {
        let utf8 = self.add_const_utf8(v);
        if let Some(ret) = self.str_map.get(&utf8) {
            *ret
        } else {
            self.file.constant_pool.push(Constant::String(utf8));
            let ret = self.file.constant_pool.len() as u32;
            self.str_map.insert(utf8, ret);
            ret
        }
    }

    pub fn add_const_utf8(&mut self, v: &str) -> u32 {
        if let Some(ret) = self.utf8_map.get(v) {
            *ret
        } else {
            self.file
                .constant_pool
                .push(Constant::Utf8(String::from(v)));
            let ret = self.file.constant_pool.len() as u32;
            self.utf8_map.insert(String::from(v), ret);
            ret
        }
    }

    pub fn add_const_mod(&mut self, name: &str) -> u32 {
        let name = self.add_const_utf8(name);
        if let Some(ret) = self.mod_map.get(&name) {
            *ret
        } else {
            self.file.constant_pool.push(Constant::Mod(name));
            let ret = self.file.constant_pool.len() as u32;
            self.mod_map.insert(name, ret);
            ret
        }
    }

    pub fn add_const_class(&mut self, mod_name: &str, name: &str) -> u32 {
        let mod_idx = self.add_const_mod(mod_name);
        let name = self.add_const_utf8(name);
        if let Some(ret) = self.class_map.get(&(mod_idx, name)) {
            *ret
        } else {
            self.file.constant_pool.push(Constant::Class(mod_idx, name));
            let ret = self.file.constant_pool.len() as u32;
            self.class_map.insert((mod_idx, name), ret);
            ret
        }
    }

    fn add_const_name_and_type(&mut self, name: &str, ty: &str) -> u32 {
        let name = self.add_const_utf8(name);
        let ty = self.add_const_utf8(ty);

        if let Some(ret) = self.name_and_type_map.get(&(name, ty)) {
            *ret
        } else {
            self.file
                .constant_pool
                .push(Constant::NameAndType(name, ty));
            let ret = self.file.constant_pool.len() as u32;
            self.name_and_type_map.insert((name, ty), ret);
            ret
        }
    }

    pub fn add_const_fieldref(
        &mut self,
        mod_name: &str,
        class_name: &str,
        name: &str,
        ty: &str,
    ) -> u32 {
        let class = self.add_const_class(mod_name, class_name);
        let name_and_type = self.add_const_name_and_type(name, ty);

        if let Some(ret) = self.field_map.get(&(class, name_and_type)) {
            *ret
        } else {
            self.file
                .constant_pool
                .push(Constant::Fieldref(class, name_and_type));
            let ret = self.file.constant_pool.len() as u32;
            self.field_map.insert((class, name_and_type), ret);
            ret
        }
    }

    pub fn add_const_methodref(
        &mut self,
        mod_name: &str,
        class_name: &str,
        name: &str,
        ty: &str,
    ) -> u32 {
        let class = self.add_const_class(mod_name, class_name);
        let name_and_type = self.add_const_name_and_type(name, ty);

        if let Some(ret) = self.method_map.get(&(class, name_and_type)) {
            *ret
        } else {
            self.file
                .constant_pool
                .push(Constant::Methodref(class, name_and_type));
            let ret = self.file.constant_pool.len() as u32;
            self.method_map.insert((class, name_and_type), ret);
            ret
        }
    }
}
