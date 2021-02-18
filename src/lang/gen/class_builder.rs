use crate::ir::class_file::{ClassFile, Constant, IrField, IrMethod};
use crate::ir::flag::*;
use crate::ir::inst::Inst;
use crate::ir::util::linkedlist::LinkedList;

use std::collections::HashMap;

struct BasicBlock {
    insts: Vec<Inst>,
}

struct MethodBuilder {
    codes: LinkedList<BasicBlock>,
    size: u16,
}

impl MethodBuilder {
    fn new() -> MethodBuilder {
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

    fn push(&mut self, inst: Inst) {
        self.size += inst.size();
        self.codes.back_mut().unwrap().insts.push(inst);
    }
}

pub struct ClassBuilder {
    // use const map to avoid redeclaration
    // str -> utf8 idx
    utf8_map: HashMap<String, u32>,
    // utf8 idx -> String idx
    str_map: HashMap<u32, u32>,
    // utf8 idx -> Class idx
    class_map: HashMap<u32, u32>,
    // (Class idx, NameAndType idx) -> Field idx
    field_map: HashMap<(u32, u32), u32>,
    // (Class idx, NameAndType idx) -> Field idx
    method_map: HashMap<(u32, u32), u32>,
    // (utf8 name idx, utf8 ty idx) -> NameAndType idx
    name_and_type_map: HashMap<(u32, u32), u32>,
    pub class_file: ClassFile,

    codes: Vec<MethodBuilder>,
}

impl ClassBuilder {
    pub fn new(class_name: &str, flag: &TypeFlag) -> ClassBuilder {
        let mut ret = ClassBuilder {
            utf8_map: HashMap::new(),
            str_map: HashMap::new(),
            class_map: HashMap::new(),
            field_map: HashMap::new(),
            method_map: HashMap::new(),
            name_and_type_map: HashMap::new(),
            class_file: ClassFile::new(flag.flag),
            codes: Vec::new(),
        };

        ret.class_file.this_class = ret.add_const_class(class_name);

        ret
    }

    /// Add a field of this class
    pub fn add_field(&mut self, name: &str, ty: &str, flag: &FieldFlag) -> usize {
        let name_index = self.add_const_utf8(name);
        let descriptor_index = self.add_const_utf8(ty);
        self.class_file.fields.push(IrField {
            access_flags: flag.flag,
            name_index,
            descriptor_index,
        });
        self.class_file.fields.len() - 1
    }

    /// Add a field of this class
    pub fn add_method(&mut self, name: &str, ty: &str, flag: &MethodFlag) -> usize {
        let name_index = self.add_const_utf8(name);
        let descriptor_index = self.add_const_utf8(ty);
        self.class_file.methods.push(IrMethod {
            access_flags: flag.flag,
            name_index,
            descriptor_index,
            locals: 0,
            insts: vec![],
            exception: vec![],
        });
        self.codes.push(MethodBuilder::new());
        self.class_file.methods.len() - 1
    }

    /// Post-Process
    ///
    /// Fill all jump instructions, concat all basic blocks
    ///
    pub fn done(&mut self, method_idx: usize, locals_stack: u16) {
        let ir_method = &mut self.class_file.methods[method_idx];
        let method_builder = &mut self.codes[method_idx];
        // fill jump instructions

        // concat basic blocks
        let mut codes: Vec<Inst> = Vec::new();
        for bb in method_builder.codes.iter_mut() {
            codes.append(&mut bb.insts);
        }
        ir_method.locals = locals_stack;
        ir_method.insts = codes;
    }
}

// Const values
impl ClassBuilder {
    pub fn add_const_string(&mut self, v: &str) -> u32 {
        let utf8 = self.add_const_utf8(v);
        if let Some(ret) = self.str_map.get(&utf8) {
            *ret
        } else {
            self.class_file.constant_pool.push(Constant::String(utf8));
            let ret = self.class_file.constant_pool.len() as u32;
            self.str_map.insert(utf8, ret);
            ret
        }
    }

    pub fn add_const_utf8(&mut self, v: &str) -> u32 {
        if let Some(ret) = self.utf8_map.get(v) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::Utf8(String::from(v)));
            let ret = self.class_file.constant_pool.len() as u32;
            self.utf8_map.insert(String::from(v), ret);
            ret
        }
    }

    pub fn add_const_class(&mut self, class_name: &str) -> u32 {
        let class_name_idx = self.add_const_utf8(class_name);
        if let Some(ret) = self.class_map.get(&class_name_idx) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::Class(class_name_idx));
            let ret = self.class_file.constant_pool.len() as u32;
            self.class_map.insert(class_name_idx, ret);
            ret
        }
    }

    fn add_const_name_and_type(&mut self, name: &str, ty: &str) -> u32 {
        let name = self.add_const_utf8(name);
        let ty = self.add_const_utf8(ty);

        if let Some(ret) = self.name_and_type_map.get(&(name, ty)) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::NameAndType(name, ty));
            let ret = self.class_file.constant_pool.len() as u32;
            self.name_and_type_map.insert((name, ty), ret);
            ret
        }
    }

    pub fn add_const_fieldref(&mut self, class_name: &str, name: &str, ty: &str) -> u32 {
        let class = self.add_const_class(class_name);
        let name_and_type = self.add_const_name_and_type(name, ty);

        if let Some(ret) = self.field_map.get(&(class, name_and_type)) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::Fieldref(class, name_and_type));
            let ret = self.class_file.constant_pool.len() as u32;
            self.field_map.insert((class, name_and_type), ret);
            ret
        }
    }

    pub fn add_const_methodref(&mut self, class_name: &str, name: &str, ty: &str) -> u32 {
        let class = self.add_const_class(class_name);
        let name_and_type = self.add_const_name_and_type(name, ty);

        if let Some(ret) = self.method_map.get(&(class, name_and_type)) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::Methodref(class, name_and_type));
            let ret = self.class_file.constant_pool.len() as u32;
            self.method_map.insert((class, name_and_type), ret);
            ret
        }
    }
}

// instructions
impl ClassBuilder {
    pub fn add_inst(&mut self, method_idx: usize, inst: Inst) {
        self.codes[method_idx].push(inst);
    }

    pub fn add_inst_stloc(&mut self, method_idx: usize, local_offset: u16) {
        self.codes[method_idx].push(match local_offset {
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

    pub fn add_inst_ldloc(&mut self, method_idx: usize, local_offset: u16) {
        self.codes[method_idx].push(match local_offset {
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

    pub fn add_inst_ldarg(&mut self, method_idx: usize, arg_offset: u16) {
        self.codes[method_idx].push(match arg_offset {
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

    pub fn add_inst_starg(&mut self, method_idx: usize, arg_offset: u16) {
        self.codes[method_idx].push(
            if arg_offset >= u8::MIN as u16 && arg_offset <= u8::MAX as u16 {
                Inst::StArgS(arg_offset as u8)
            } else {
                unimplemented!("ldarg is not implemeneted");
            },
        );
    }

    /// Push an int value to the stack
    pub fn add_inst_ldc(&mut self, method_idx: usize, value: i32) {
        let inst = match value {
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
        };
        self.codes[method_idx].push(inst);
    }
}
