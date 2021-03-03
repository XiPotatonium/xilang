use crate::ir::blob::IrBlob;
use crate::ir::flag::*;
use crate::ir::inst::Inst;
use crate::ir::ir_file::*;

use std::collections::HashMap;
use std::mem;

use super::basic_block::{BasicBlock, LLCursor, LinkedList};
use super::{fn_descriptor, RValType};

pub struct MethodBuilder {
    bb: LinkedList<BasicBlock>,
    cur_bb: LLCursor<BasicBlock>,
}

impl MethodBuilder {
    pub fn new() -> MethodBuilder {
        let mut bb = LinkedList::new();
        bb.push_back(BasicBlock::new());
        let cur_bb = bb.cursor_back_mut();

        MethodBuilder { bb, cur_bb }
    }

    pub fn insert_after_cur(&mut self) -> LLCursor<BasicBlock> {
        self.bb
            .insert_after_cursor(&mut self.cur_bb, BasicBlock::new())
    }

    pub fn set_cur_bb(&mut self, cur_bb: LLCursor<BasicBlock>) -> LLCursor<BasicBlock> {
        let mut cur_bb = cur_bb;
        mem::swap(&mut cur_bb, &mut self.cur_bb);
        cur_bb
    }

    pub fn cur_bb_last_is_branch(&self) -> bool {
        if let Some(inst) = self.cur_bb.as_ref().unwrap().insts.last() {
            match inst {
                Inst::BEq(_)
                | Inst::BGe(_)
                | Inst::BGt(_)
                | Inst::BLe(_)
                | Inst::BLt(_)
                | Inst::Br(_)
                | Inst::BrFalse(_)
                | Inst::BrTrue(_)
                | Inst::Ret => true,
                _ => false,
            }
        } else {
            false
        }
    }
}

impl MethodBuilder {
    pub fn add_inst(&mut self, inst: Inst) {
        self.cur_bb.as_mut().unwrap().push(inst);
    }

    pub fn add_brfalse(&mut self, target: LLCursor<BasicBlock>) {
        let cur_bb = self.cur_bb.as_mut().unwrap();
        cur_bb.push(Inst::BrFalse(0));
        if let Some(_) = cur_bb.target {
            unreachable!();
        } else {
            cur_bb.target = Some(target);
        }
    }

    pub fn add_br(&mut self, target: LLCursor<BasicBlock>) {
        let cur_bb = self.cur_bb.as_mut().unwrap();
        cur_bb.push(Inst::Br(0));
        if let Some(_) = cur_bb.target {
            unreachable!();
        } else {
            cur_bb.target = Some(target);
        }
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
                    Inst::LdCI4S(value as i8)
                } else {
                    Inst::LdCI4(value)
                }
            }
        });
    }
}

pub struct Builder {
    // use const map to avoid redeclaration
    mod_name_idx: u32,
    mod_tbl_idx: u32,
    /// <name> -> tbl idx
    modref_map: HashMap<u32, u32>,

    /// <name> -> tbl idx
    type_map: HashMap<u32, u32>,
    /// <mod>, <name> -> tbl idx
    typeref_map: HashMap<(u32, u32), u32>,

    /// <class> <name> <desc> -> tbl idx
    member_map: HashMap<(u32, u32, u32), u32>,
    /// <class> <name> <des> -> tbl idx
    memberref_map: HashMap<(u32, u32, u32), u32>,

    /// str -> str idx
    str_map: HashMap<String, u32>,
    /// descriptor -> blobl idx
    blob_map: HashMap<String, u32>,

    pub file: IrFile,
}

impl Builder {
    pub fn new(name: &str) -> Builder {
        let mut builder = Builder {
            mod_name_idx: 0,
            mod_tbl_idx: 0,
            modref_map: HashMap::new(),

            type_map: HashMap::new(),
            typeref_map: HashMap::new(),

            member_map: HashMap::new(),
            memberref_map: HashMap::new(),

            str_map: HashMap::new(),
            blob_map: HashMap::new(),

            file: IrFile::new(),
        };
        let name = builder.add_const_str(name);
        builder.file.mod_tbl.push(IrMod {
            name,
            entrypoint: 0,
        });
        builder.mod_name_idx = name;
        builder.mod_tbl_idx = builder.file.mod_tbl.len() as u32 | TBL_MOD_TAG;
        builder
    }

    pub fn add_class(&mut self, name: &str, flag: &TypeFlag) -> u32 {
        let name = self.add_const_str(name);
        self.file.class_tbl.push(IrClass {
            name,
            flag: flag.flag,
            fields: (self.file.field_tbl.len() + 1) as u32,
            methods: (self.file.method_tbl.len() + 1) as u32,
        });
        let ret = self.file.class_tbl.len() as u32 | TBL_CLASS_TAG;
        self.type_map.insert(name, ret);
        ret
    }

    /// Add a field of this class
    ///
    /// Field parent is the newly added class or none if no class has been added
    pub fn add_field(&mut self, name: &str, ty: &RValType, flag: &FieldFlag) -> u32 {
        let name = self.add_const_str(name);
        let sig = self.add_const_ty_blob(ty);
        self.file.field_tbl.push(IrField {
            name,
            signature: sig,
            flag: flag.flag,
        });
        let ret = self.file.field_tbl.len() as u32 | TBL_FIELD_TAG;
        self.member_map.insert(
            (self.file.class_tbl.len() as u32 | TBL_CLASS_TAG, name, sig),
            ret,
        );
        ret
    }

    /// Add a method of this class
    ///
    /// Method parent is the newly added class or none if no class has been added
    pub fn add_method(
        &mut self,
        name: &str,
        ps: &Vec<RValType>,
        ret_ty: &RValType,
        flag: &MethodFlag,
    ) -> u32 {
        let name = self.add_const_str(name);
        let sig = self.add_const_fn_blob(ps, ret_ty);
        self.file.method_tbl.push(IrMethod {
            flag: flag.flag,
            name,
            signature: sig,
            locals: 0,
        });
        let ret = self.file.method_tbl.len() as u32 | TBL_METHOD_TAG;
        self.member_map.insert(
            (self.file.class_tbl.len() as u32 | TBL_CLASS_TAG, name, sig),
            ret,
        );
        ret
    }

    /// Post-Process
    ///
    /// Fill all jump instructions, concat all basic blocks
    ///
    pub fn done(&mut self, m: &mut MethodBuilder, method_idx: u32, locals: u16, fold_br: bool) {
        let ir_method = &mut self.file.method_tbl[((method_idx & !TBL_TAG_MASK) - 1) as usize];

        if fold_br {
            unimplemented!("Fold branch operation is not implemented");
            // ceq, brfalse -> bne
        }

        let mut offset = 0;
        for bb in m.bb.iter_mut() {
            bb.offset = offset;
            offset += bb.size as i32;
        }

        // fill jump instructions
        for bb in m.bb.iter_mut() {
            if let Some(target) = &bb.target {
                let offset = target.as_ref().unwrap().offset - (bb.size as i32 + bb.offset);
                match bb.insts.last_mut().unwrap() {
                    Inst::Br(offset_) => *offset_ = offset,
                    Inst::BrFalse(offset_) => *offset_ = offset,
                    Inst::BrTrue(offset_) => *offset_ = offset,
                    Inst::BEq(offset_) => *offset_ = offset,
                    Inst::BGe(offset_) => *offset_ = offset,
                    Inst::BGt(offset_) => *offset_ = offset,
                    Inst::BLe(offset_) => *offset_ = offset,
                    Inst::BLt(offset_) => *offset_ = offset,
                    _ => {}
                }
            }
        }

        // concat basic blocks
        let mut code: Vec<Inst> = Vec::new();
        for bb in m.bb.iter_mut() {
            code.append(&mut bb.insts);
        }
        ir_method.locals = locals;
        self.file.codes.push(code);
    }
}

// Const values
impl Builder {
    pub fn add_const_str(&mut self, v: &str) -> u32 {
        if let Some(ret) = self.str_map.get(v) {
            *ret
        } else {
            let ret = self.file.str_heap.len() as u32;
            self.file.str_heap.push(v.to_owned());
            self.str_map.insert(v.to_owned(), ret);
            ret
        }
    }

    fn to_blob(&mut self, ty: &RValType) -> IrBlob {
        match ty {
            RValType::Bool => IrBlob::Bool,
            RValType::U8 => IrBlob::U8,
            RValType::Char => IrBlob::Char,
            RValType::I32 => IrBlob::I32,
            RValType::F64 => IrBlob::F64,
            RValType::Void => IrBlob::Void,
            RValType::Never => unreachable!(),
            RValType::Obj(mod_name, name) => IrBlob::Obj(self.add_const_class(mod_name, name)),
            RValType::Array(inner) => IrBlob::Array(self.add_const_ty_blob(&inner)),
        }
    }

    pub fn add_const_ty_blob(&mut self, ty: &RValType) -> u32 {
        let desc = ty.descriptor();
        if let Some(ret) = self.blob_map.get(&desc) {
            *ret
        } else {
            let ty = self.to_blob(ty);
            let ret = self.file.blob_heap.len() as u32;
            self.file.blob_heap.push(ty);
            self.blob_map.insert(desc, ret);
            ret
        }
    }

    pub fn add_const_fn_blob(&mut self, ps: &Vec<RValType>, ret_ty: &RValType) -> u32 {
        let desc = fn_descriptor(ret_ty, ps);
        if let Some(ret) = self.blob_map.get(&desc) {
            *ret
        } else {
            let ps: Vec<u32> = ps.iter().map(|p| self.add_const_ty_blob(p)).collect();
            let ret_ty = self.add_const_ty_blob(ret_ty);
            let ret = self.file.blob_heap.len() as u32;
            self.file.blob_heap.push(IrBlob::Func(ps, ret_ty));
            self.blob_map.insert(desc, ret);
            ret
        }
    }

    pub fn add_const_mod(&mut self, name: &str) -> u32 {
        let name = self.add_const_str(name);
        if name == self.mod_name_idx {
            // this module
            self.mod_tbl_idx
        } else if let Some(ret) = self.modref_map.get(&name) {
            *ret
        } else {
            self.file.modref_tbl.push(IrModRef { name });
            let ret = (self.file.modref_tbl.len() as u32) | TBL_MODREF_TAG;
            self.modref_map.insert(name, ret);
            ret
        }
    }

    pub fn add_const_class(&mut self, mod_name: &str, name: &str) -> u32 {
        let parent = self.add_const_mod(mod_name);
        let name = self.add_const_str(name);
        if parent == self.mod_tbl_idx {
            // class in this module
            *self.type_map.get(&name).unwrap()
        } else if let Some(ret) = self.typeref_map.get(&(parent, name)) {
            *ret
        } else {
            self.file.classref_tbl.push(IrClassRef { parent, name });
            let ret = self.file.classref_tbl.len() as u32 | TBL_CLASSREF_TAG;
            self.typeref_map.insert((parent, name), ret);
            ret
        }
    }

    pub fn add_const_member(
        &mut self,
        mod_name: &str,
        class_name: &str,
        name: &str,
        sig: u32,
    ) -> u32 {
        let parent = self.add_const_class(mod_name, class_name);
        let name = self.add_const_str(name);

        if parent & TBL_TAG_MASK == TBL_CLASS_TAG {
            // class in this module
            *self.member_map.get(&(parent, name, sig)).unwrap()
        } else if let Some(ret) = self.memberref_map.get(&(parent, name, sig)) {
            *ret
        } else {
            self.file.memberref_tbl.push(IrMemberRef {
                parent,
                name,
                signature: sig,
            });
            let ret = self.file.memberref_tbl.len() as u32 | TBL_MEMBERREF_TAG;
            self.memberref_map.insert((parent, name, sig), ret);
            ret
        }
    }
}
