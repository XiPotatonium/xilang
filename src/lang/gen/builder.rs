use xir::blob::Blob;
use xir::file::*;
use xir::flag::*;
use xir::inst::Inst;
use xir::tok::{to_tok, TokTag};

use std::collections::HashMap;

use super::{fn_descriptor, MethodBuilder, RValType};

#[derive(Clone, Hash, PartialEq, Eq)]
struct FieldOrMethod {
    /// index into typedef tbl
    parent: u32,
    /// index into str heap
    name: u32,
    /// index into blob heap
    sig: u32,
}

pub struct Builder {
    // use const map to avoid redeclaration
    mod_name_idx: u32,
    mod_tbl_idx: u32, // always 1

    /// Name -> TblIdx
    modref_map: HashMap<u32, u32>,

    /// Name -> TblIdx
    type_map: HashMap<u32, u32>,
    /// TypeRef -> TblIdx
    typeref_map: HashMap<IrTypeRef, u32>,

    /// FieldOrMethod -> IdxIntoFieldTbl
    field_map: HashMap<FieldOrMethod, u32>,
    method_map: HashMap<FieldOrMethod, u32>,
    /// MemberRef -> TblIdx
    memberref_map: HashMap<IrMemberRef, u32>,

    /// str -> StrHeapIdx
    str_map: HashMap<String, u32>,
    /// str -> UsrStrHeapIdx
    usr_str_map: HashMap<String, u32>,
    /// descriptor -> BlobHeapIdx
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

            field_map: HashMap::new(),
            method_map: HashMap::new(),
            memberref_map: HashMap::new(),

            str_map: HashMap::new(),
            usr_str_map: HashMap::new(),
            blob_map: HashMap::new(),

            file: IrFile::new(),
        };
        let name = builder.add_const_str(name);
        builder.file.mod_tbl.push(IrMod {
            name,
            entrypoint: 0,
        });
        builder.mod_name_idx = name;
        builder.mod_tbl_idx = builder.file.mod_tbl.len() as u32;
        builder
    }

    pub fn add_class(&mut self, name: &str, flag: &TypeFlag) -> u32 {
        let name = self.add_const_str(name);
        self.file.typedef_tbl.push(IrTypeDef {
            name,
            flag: flag.flag,
            fields: (self.file.field_tbl.len() + 1) as u32,
            methods: (self.file.method_tbl.len() + 1) as u32,
        });
        let ret = self.file.typedef_tbl.len() as u32;
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
            sig,
            flag: flag.flag,
        });
        let ret = self.file.field_tbl.len() as u32;
        let info = FieldOrMethod {
            parent: self.file.typedef_tbl.len() as u32,
            name,
            sig,
        };
        // TODO: expect none
        self.field_map.insert(info, ret);
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
            body: 0,
            sig,
        });
        let ret = self.file.method_tbl.len() as u32;
        let info = FieldOrMethod {
            parent: self.file.typedef_tbl.len() as u32,
            name,
            sig,
        };
        self.method_map.insert(info, ret);
        ret
    }

    /// Post-Process
    ///
    /// Fill all jump instructions, concat all basic blocks
    ///
    pub fn done(&mut self, m: &mut MethodBuilder, method_idx: u32, local: u16, fold_br: bool) {
        let ir_method = &mut self.file.method_tbl[method_idx as usize - 1];

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

        self.file.codes.push(CorILMethod::new(0, local, code));
        ir_method.body = self.file.codes.len() as u32;
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

    fn to_blob(&mut self, ty: &RValType) -> Blob {
        match ty {
            RValType::Bool => Blob::Bool,
            RValType::U8 => Blob::U8,
            RValType::Char => Blob::Char,
            RValType::I32 => Blob::I32,
            RValType::F64 => Blob::F64,
            RValType::Void => Blob::Void,
            RValType::Never => unreachable!(),
            RValType::Obj(mod_name, name) => {
                let (class_idx, class_tag) = self.add_const_class(mod_name, name);
                let tok = match class_tag {
                    TokTag::TypeDef => to_tok(class_idx, TokTag::TypeDef),
                    TokTag::TypeRef => to_tok(class_idx, TokTag::TypeRef),
                    _ => unreachable!(),
                };
                Blob::Obj(tok)
            }
            RValType::Array(inner) => Blob::Array(self.add_const_ty_blob(&inner)),
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
            self.file.blob_heap.push(Blob::Func(ps, ret_ty));
            self.blob_map.insert(desc, ret);
            ret
        }
    }

    pub fn add_const_mod(&mut self, name: &str) -> (u32, TokTag) {
        let name = self.add_const_str(name);
        if name == self.mod_name_idx {
            // this module
            (self.mod_tbl_idx, TokTag::Mod)
        } else if let Some(ret) = self.modref_map.get(&name) {
            (*ret, TokTag::ModRef)
        } else {
            self.file.modref_tbl.push(IrModRef { name });
            let ret = self.file.modref_tbl.len() as u32;
            self.modref_map.insert(name, ret);
            (ret, TokTag::ModRef)
        }
    }

    pub fn add_const_class(&mut self, mod_name: &str, name: &str) -> (u32, TokTag) {
        let (parent_idx, parent_tag) = self.add_const_mod(mod_name);
        let name = self.add_const_str(name);
        match parent_tag {
            TokTag::Mod => {
                // class in this module
                (*self.type_map.get(&name).unwrap(), TokTag::TypeDef)
            }
            TokTag::ModRef => {
                let typeref = IrTypeRef {
                    parent: get_typeref_parent(parent_idx, ResolutionScope::ModRef),
                    name,
                };
                if let Some(ret) = self.typeref_map.get(&typeref) {
                    (*ret, TokTag::TypeRef)
                } else {
                    self.file.typeref_tbl.push(typeref.clone());
                    let ret = self.file.typeref_tbl.len() as u32;
                    self.typeref_map.insert(typeref, ret);
                    (ret, TokTag::TypeRef)
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn add_const_member(
        &mut self,
        mod_name: &str,
        class_name: &str,
        name: &str,
        sig: u32,
    ) -> (u32, TokTag) {
        let (parent_idx, parent_tag) = self.add_const_class(mod_name, class_name);
        let name = self.add_const_str(name);

        match parent_tag {
            TokTag::TypeDef => {
                // class in this module
                let info = FieldOrMethod {
                    parent: parent_idx,
                    name,
                    sig,
                };
                if let Some(f) = self.field_map.get(&info) {
                    (*f, TokTag::Field)
                } else if let Some(m) = self.method_map.get(&info) {
                    (*m, TokTag::MethodDef)
                } else {
                    unreachable!()
                }
            }
            TokTag::TypeRef => {
                let parent_tagged_idx = to_memberref_parent(parent_idx, MemberRefParent::TypeRef);
                let memberref = IrMemberRef {
                    parent: parent_tagged_idx,
                    name,
                    sig,
                };
                if let Some(ret) = self.memberref_map.get(&memberref) {
                    (*ret, TokTag::MemberRef)
                } else {
                    self.file.memberref_tbl.push(memberref.clone());
                    let ret = self.file.memberref_tbl.len() as u32;
                    self.memberref_map.insert(memberref, ret);
                    (ret, TokTag::MemberRef)
                }
            }
            _ => unreachable!(),
        }
    }
}
