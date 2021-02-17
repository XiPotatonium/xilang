use crate::ir::class::class_file::{Attribute, ClassFile, Constant, IrField, IrMethod};
use crate::ir::class::CODE_ATTR_NAME;
use crate::ir::flag::Flag;
use crate::ir::inst::Inst;
use crate::ir::ty::RValType;
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
    utf8_map: HashMap<String, u16>,
    // utf8 idx -> String idx
    str_map: HashMap<u16, u16>,
    // utf8 idx -> Class idx
    class_map: HashMap<u16, u16>,
    // (Class idx, NameAndType idx) -> Field idx
    field_map: HashMap<(u16, u16), u16>,
    // (Class idx, NameAndType idx) -> Field idx
    method_map: HashMap<(u16, u16), u16>,
    // (utf8 name idx, utf8 ty idx) -> NameAndType idx
    name_and_type_map: HashMap<(u16, u16), u16>,
    // value -> idx
    int_map: HashMap<i32, u16>,
    pub class_file: ClassFile,

    codes: Vec<MethodBuilder>,
}

impl ClassBuilder {
    pub fn new(class_name: &str, flag: &Flag) -> ClassBuilder {
        let mut ret = ClassBuilder {
            utf8_map: HashMap::new(),
            str_map: HashMap::new(),
            class_map: HashMap::new(),
            field_map: HashMap::new(),
            method_map: HashMap::new(),
            name_and_type_map: HashMap::new(),
            int_map: HashMap::new(),
            class_file: ClassFile::new(flag.flag),
            codes: Vec::new(),
        };

        ret.class_file.this_class = ret.add_const_class(class_name);

        ret
    }

    /// Add a field of this class
    pub fn add_field(&mut self, name: &str, ty: &str, flag: &Flag) -> usize {
        let name_index = self.add_const_utf8(name);
        let descriptor_index = self.add_const_utf8(ty);
        self.class_file.fields.push(IrField {
            access_flags: flag.flag,
            name_index,
            descriptor_index,
            attributes: vec![],
        });
        self.class_file.fields.len() - 1
    }

    /// Add a field of this class
    pub fn add_method(&mut self, name: &str, ty: &str, flag: &Flag) -> usize {
        let name_index = self.add_const_utf8(name);
        let descriptor_index = self.add_const_utf8(ty);
        self.class_file.methods.push(IrMethod {
            access_flags: flag.flag,
            name_index,
            descriptor_index,
            attributes: vec![],
        });
        self.codes.push(MethodBuilder::new());
        self.class_file.methods.len() - 1
    }

    /// Post-Process
    ///
    /// Fill all jump instructions, concat all basic blocks
    ///
    pub fn done(&mut self, method_idx: usize, max_stack: u16) {
        let code_attr_name_idx = self.add_const_utf8(CODE_ATTR_NAME);
        let ir_method = &mut self.class_file.methods[method_idx];
        let method_builder = &mut self.codes[method_idx];
        // fill jump instructions

        // concat basic blocks
        let mut codes: Vec<Inst> = Vec::new();
        for bb in method_builder.codes.iter_mut() {
            codes.append(&mut bb.insts);
        }
        ir_method.attributes.push(Attribute::Code(
            code_attr_name_idx,
            max_stack,
            codes,
            vec![],
            vec![],
        ));
    }
}

// Const values
impl ClassBuilder {
    pub fn add_const_string(&mut self, v: &str) -> u16 {
        let utf8 = self.add_const_utf8(v);
        if let Some(ret) = self.str_map.get(&utf8) {
            *ret
        } else {
            self.class_file.constant_pool.push(Constant::String(utf8));
            let ret = self.class_file.constant_pool.len() as u16;
            self.str_map.insert(utf8, ret);
            ret
        }
    }

    pub fn add_const_utf8(&mut self, v: &str) -> u16 {
        if let Some(ret) = self.utf8_map.get(v) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::Utf8(String::from(v)));
            let ret = self.class_file.constant_pool.len() as u16;
            self.utf8_map.insert(String::from(v), ret);
            ret
        }
    }

    pub fn add_const_class(&mut self, class_name: &str) -> u16 {
        let class_name_idx = self.add_const_utf8(class_name);
        if let Some(ret) = self.class_map.get(&class_name_idx) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::Class(class_name_idx));
            let ret = self.class_file.constant_pool.len() as u16;
            self.class_map.insert(class_name_idx, ret);
            ret
        }
    }

    fn add_const_name_and_type(&mut self, name: &str, ty: &str) -> u16 {
        let name = self.add_const_utf8(name);
        let ty = self.add_const_utf8(ty);

        if let Some(ret) = self.name_and_type_map.get(&(name, ty)) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::NameAndType(name, ty));
            let ret = self.class_file.constant_pool.len() as u16;
            self.name_and_type_map.insert((name, ty), ret);
            ret
        }
    }

    pub fn add_const_fieldref(&mut self, class_name: &str, name: &str, ty: &str) -> u16 {
        let class = self.add_const_class(class_name);
        let name_and_type = self.add_const_name_and_type(name, ty);

        if let Some(ret) = self.field_map.get(&(class, name_and_type)) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::Fieldref(class, name_and_type));
            let ret = self.class_file.constant_pool.len() as u16;
            self.field_map.insert((class, name_and_type), ret);
            ret
        }
    }

    pub fn add_const_methodref(&mut self, class_name: &str, name: &str, ty: &str) -> u16 {
        let class = self.add_const_class(class_name);
        let name_and_type = self.add_const_name_and_type(name, ty);

        if let Some(ret) = self.method_map.get(&(class, name_and_type)) {
            *ret
        } else {
            self.class_file
                .constant_pool
                .push(Constant::Methodref(class, name_and_type));
            let ret = self.class_file.constant_pool.len() as u16;
            self.method_map.insert((class, name_and_type), ret);
            ret
        }
    }

    fn add_const_i(&mut self, val: i32) -> u16 {
        if let Some(ret) = self.int_map.get(&val) {
            *ret
        } else {
            self.class_file.constant_pool.push(Constant::Integer(val));
            let ret = self.class_file.constant_pool.len() as u16;
            self.int_map.insert(val, ret);
            ret
        }
    }
}

/// Wrap load/store/iinc
///
/// TODO: use concat_idents! once it becomes stable
/*
macro_rules! wrap_wide {
    ($inst: path, $arg: expr) => {
        {
            let val = $arg as u16;
            match val {
                0 => inst0(val as u8),
                1 => inst1(val as u8),
                2 => inst2(val as u8),
                3 => inst3(val as u8),
                _ => {
                    if val >= u8::MIN as u16 && val <= u8::MAX as u16 {
                        $inst(val as u8)
                    } else {
                        Inst::Wide(Box::new($inst((val >> 8) as u8)), (val % (1u16 << 8)) as u8)
                    }
                }
            }
        }
    };
}
*/

// instructions
impl ClassBuilder {
    pub fn add_inst(&mut self, method_idx: usize, inst: Inst) {
        self.codes[method_idx].push(inst);
    }

    pub fn add_inst_store(&mut self, method_idx: usize, local_ty: &RValType, local_offset: u16) {
        self.codes[method_idx].push(match local_ty {
            RValType::Int => match local_offset {
                0 => Inst::IStore0,
                1 => Inst::IStore1,
                2 => Inst::IStore2,
                3 => Inst::IStore3,
                _ => Inst::IStore(local_offset),
            },
            RValType::Class(_) => match local_offset {
                0 => Inst::AStore0,
                1 => Inst::AStore1,
                2 => Inst::AStore2,
                3 => Inst::AStore3,
                _ => Inst::AStore(local_offset),
            },
            _ => unimplemented!(),
        });
    }

    pub fn add_inst_load(&mut self, method_idx: usize, local_ty: &RValType, local_offset: u16) {
        self.codes[method_idx].push(match local_ty {
            RValType::Int => match local_offset {
                0 => Inst::ILoad0,
                1 => Inst::ILoad1,
                2 => Inst::ILoad2,
                3 => Inst::ILoad3,
                _ => Inst::ILoad(local_offset),
            },
            RValType::Class(_) => match local_offset {
                0 => Inst::ALoad0,
                1 => Inst::ALoad1,
                2 => Inst::ALoad2,
                3 => Inst::ALoad3,
                _ => Inst::ALoad(local_offset),
            },
            _ => unimplemented!(),
        });
    }

    /// Push an int value to the stack
    pub fn add_inst_pushi(&mut self, method_idx: usize, value: i32) {
        let inst = match value {
            -1 => Inst::IConstM1,
            0 => Inst::IConst0,
            1 => Inst::IConst1,
            2 => Inst::IConst2,
            3 => Inst::IConst3,
            4 => Inst::IConst4,
            5 => Inst::IConst5,
            _ => {
                if value >= i8::MIN as i32 && value <= i8::MAX as i32 {
                    Inst::BIPush(value as i8)
                } else if value >= i16::MIN as i32 && value >= i16::MAX as i32 {
                    unimplemented!("SIPush is not implemented")
                } else {
                    let i_const_idx = self.add_const_i(value);
                    Inst::LdC(i_const_idx)
                }
            }
        };
        self.codes[method_idx].push(inst);
    }
}
