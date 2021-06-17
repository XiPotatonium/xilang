use xir::attrib::*;
use xir::code::CorILMethod;
use xir::file::IrFile;
use xir::inst::Inst;
use xir::member::{
    to_implmap_member, to_memberref_parent, Field, ImplMap, MemberForwarded, MemberRef,
    MemberRefParent, MethodDef,
};
use xir::module::{Mod, ModRef};
use xir::sig::{self, IrSig, MethodSigFlag, MethodSigFlagTag, TypeSig};
use xir::stand_alone_sig::IrStandAloneSig;
use xir::tok::{to_tok, TokTag};
use xir::ty::{get_typeref_parent, ResolutionScope, TypeDef, TypeDefOrRef, TypeRef};

use std::collections::HashMap;

use super::super::mod_mgr::{Param, Var};
use super::{MethodBuilder, RValType};

#[derive(Clone, Hash, PartialEq, Eq)]
struct FieldOrMethod {
    /// index into typedef tbl
    parent: u32,
    /// index into str heap
    name: u32,
    /// index into blob heap
    sig: u32,
}

#[derive(Clone, Hash, PartialEq, Eq)]
struct ImplMapInfo {
    /// index into modref tbl, external mod
    scope: u32,
    /// index into str heap, external fn name
    name: u32,
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
    typeref_map: HashMap<TypeRef, u32>,

    /// FieldOrMethod -> IdxIntoFieldTbl
    field_map: HashMap<FieldOrMethod, u32>,
    method_map: HashMap<FieldOrMethod, u32>,
    /// MemberRef -> TblIdx
    memberref_map: HashMap<MemberRef, u32>,

    /// ImplMap -> TblIdx
    implmap_map: HashMap<ImplMapInfo, u32>,

    /// str -> StrHeapIdx
    str_map: HashMap<String, u32>,
    /// str -> UsrStrHeapIdx
    usr_str_map: HashMap<String, u32>,

    /// descriptor -> blob head index
    member_sig_map: HashMap<String, u32>,

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

            implmap_map: HashMap::new(),

            str_map: HashMap::new(),
            usr_str_map: HashMap::new(),

            member_sig_map: HashMap::new(),

            file: IrFile::new(),
        };
        let name = builder.add_const_str(name);
        builder.file.mod_tbl.push(Mod {
            name,
            entrypoint: 0,
        });
        builder.mod_name_idx = name;
        builder.mod_tbl_idx = builder.file.mod_tbl.len() as u32;
        builder
    }

    pub fn add_class(&mut self, name: &str, flag: &TypeAttrib) -> u32 {
        let name = self.add_const_str(name);
        self.file.typedef_tbl.push(TypeDef {
            flag: flag.attrib,
            name,
            extends: 0,
            fields: (self.file.field_tbl.len() + 1) as u32,
            methods: (self.file.method_tbl.len() + 1) as u32,
        });
        let ret = self.file.typedef_tbl.len() as u32;
        self.type_map.insert(name, ret);
        ret
    }

    /// extends_idx and extends_idx_tag may be acquired from Builder.add_const_class
    pub fn set_class_extends(
        &mut self,
        class_idx: u32,
        extends_idx: u32,
        extends_idx_tag: TypeDefOrRef,
    ) {
        let old_extends =
            self.file.typedef_tbl[class_idx as usize - 1].set_extends(extends_idx, extends_idx_tag);
        assert_eq!(old_extends, None, "Overriding old extends");
    }

    /// Add a field of this class
    ///
    /// Field parent is the newly added class or none if no class has been added
    pub fn add_field(&mut self, name: &str, ty: &RValType, flag: &FieldAttrib) -> u32 {
        let name = self.add_const_str(name);
        let sig = self.add_field_sig(ty);
        self.file.field_tbl.push(Field {
            name,
            sig,
            flag: flag.attrib,
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
        ps: &Vec<Param>,
        ret: &RValType,
        flag: &MethodAttrib,
        impl_flag: &MethodImplAttrib,
    ) -> u32 {
        let name = self.add_const_str(name);

        let sig = self.add_method_sig(!flag.is(MethodAttribFlag::Static), ps, ret);

        self.file.method_tbl.push(MethodDef {
            name,
            body: 0,
            sig,
            flag: flag.attrib,
            impl_flag: impl_flag.attrib,
            param_list: self.file.param_tbl.len() as u32 + 1,
        });
        for (i, p) in ps.iter().enumerate() {
            let name = self.add_const_str(&p.id);
            self.file.param_tbl.push(xir::Param {
                flag: p.attrib.attrib,
                name,
                sequence: i as u16 + 1,
            });
        }
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
    pub fn done(
        &mut self,
        m: &mut MethodBuilder,
        method_idx: u32,
        locals: &Vec<Var>,
        fold_br: bool,
    ) {
        // store local var info
        let locals_sig = if locals.len() == 0 {
            // no locals
            0
        } else {
            // silly rust
            let mut locals_ty = Vec::new();
            for v in locals.iter() {
                locals_ty.push(self.to_local_sig(v));
            }
            self.file.blob_heap.push(IrSig::LocalVar(locals_ty));
            self.file.stand_alone_sig_tbl.push(IrStandAloneSig {
                sig: self.file.blob_heap.len() as u32 - 1,
            });
            self.file.stand_alone_sig_tbl.len() as u32
        };

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

        self.file.codes.push(CorILMethod::new(0, locals_sig, code));
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

    pub fn add_const_usr_str(&mut self, v: &str) -> u32 {
        if let Some(ret) = self.usr_str_map.get(v) {
            *ret
        } else {
            let ret = self.file.usr_str_heap.len() as u32;
            self.file.usr_str_heap.push(v.to_owned());
            self.usr_str_map.insert(v.to_owned(), ret);
            ret
        }
    }

    fn to_param(&mut self, param: &Param) -> sig::ParamType {
        sig::ParamType {
            ty: sig::InnerParamType::Default(self.to_sig_ty(&param.ty)),
        }
    }

    fn to_ret(&mut self, ret_ty: &RValType) -> sig::RetType {
        sig::RetType {
            ty: if let RValType::Void = ret_ty {
                sig::InnerRetType::Void
            } else {
                sig::InnerRetType::Default(self.to_sig_ty(ret_ty))
            },
        }
    }

    fn to_local_sig(&mut self, var: &Var) -> sig::InnerLocalVarType {
        sig::InnerLocalVarType::Default(self.to_sig_ty(&var.ty))
    }

    fn to_sig_ty(&mut self, ty: &RValType) -> TypeSig {
        match ty {
            RValType::Bool => TypeSig::Boolean,
            RValType::U8 => TypeSig::U1,
            RValType::Char => TypeSig::Char,
            RValType::I32 => TypeSig::I4,
            RValType::F64 => TypeSig::R8,
            RValType::Never => unreachable!(),
            RValType::String => TypeSig::String,
            RValType::Class(ty) => {
                let ty_ref = unsafe { ty.as_ref() };
                let (class_idx, class_tag) = self.add_const_class(ty_ref.modname(), &ty_ref.name);
                TypeSig::Class(to_tok(class_idx, class_tag.to_tok_tag()))
            }
            RValType::Value(ty) => {
                let ty_ref = unsafe { ty.as_ref() };
                let (class_idx, class_tag) = self.add_const_class(ty_ref.modname(), &ty_ref.name);
                TypeSig::ValueType(to_tok(class_idx, class_tag.to_tok_tag()))
            }
            RValType::GenericInst(is_class, ty, args) => {
                let ty_ref = unsafe { ty.as_ref() };
                let (class_idx, class_tag) = self.add_const_class(ty_ref.modname(), &ty_ref.name);
                TypeSig::GenericInst(
                    *is_class,
                    to_tok(class_idx, class_tag.to_tok_tag()),
                    args.iter().map(|arg| self.to_sig_ty(arg)).collect(),
                )
            }
            RValType::ByRef(_) => unimplemented!(),
            RValType::Array(ele_ty) => TypeSig::SZArray(Box::new(self.to_sig_ty(ele_ty))),
            RValType::Void => unreachable!(),
        }
    }

    pub fn add_field_sig(&mut self, ty: &RValType) -> u32 {
        let desc = ty.descriptor();
        if let Some(ret) = self.member_sig_map.get(&desc) {
            *ret
        } else {
            let ty = self.to_sig_ty(ty);
            let ret = self.file.blob_heap.len() as u32;
            self.file.blob_heap.push(IrSig::Field(ty));
            self.member_sig_map.insert(desc, ret);
            ret
        }
    }

    pub fn add_method_sig(&mut self, is_instance: bool, ps: &Vec<Param>, ret_ty: &RValType) -> u32 {
        let desc = format!(
            "{}({}){}",
            if is_instance { "instance " } else { "" },
            ps.iter().map(|t| format!("{}", t.ty)).collect::<String>(),
            ret_ty
        );
        if let Some(ret) = self.member_sig_map.get(&desc) {
            *ret
        } else {
            let ps = ps.iter().map(|p| self.to_param(&p)).collect();
            let ret_ty = self.to_ret(ret_ty);
            let flag = if is_instance {
                MethodSigFlag::new(MethodSigFlagTag::HasThis)
            } else {
                MethodSigFlag::new(MethodSigFlagTag::Default)
            };
            let ret = self.file.blob_heap.len() as u32;
            self.file.blob_heap.push(IrSig::Method(flag, ps, ret_ty));
            self.member_sig_map.insert(desc, ret);
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
            self.file.modref_tbl.push(ModRef { name });
            let ret = self.file.modref_tbl.len() as u32;
            self.modref_map.insert(name, ret);
            (ret, TokTag::ModRef)
        }
    }

    pub fn add_const_class(&mut self, mod_name: &str, name: &str) -> (u32, TypeDefOrRef) {
        let (parent_idx, parent_tag) = self.add_const_mod(mod_name);
        let name = self.add_const_str(name);
        match parent_tag {
            TokTag::Mod => {
                // class in this module
                (*self.type_map.get(&name).unwrap(), TypeDefOrRef::TypeDef)
            }
            TokTag::ModRef => {
                let typeref = TypeRef {
                    parent: get_typeref_parent(parent_idx, ResolutionScope::ModRef),
                    name,
                };
                if let Some(ret) = self.typeref_map.get(&typeref) {
                    (*ret, TypeDefOrRef::TypeRef)
                } else {
                    self.file.typeref_tbl.push(typeref.clone());
                    let ret = self.file.typeref_tbl.len() as u32;
                    self.typeref_map.insert(typeref, ret);
                    (ret, TypeDefOrRef::TypeRef)
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn add_const_member(
        &mut self,
        mod_name: &str,
        class_name: &str,
        member_name: &str,
        sig: u32,
    ) -> (u32, TokTag) {
        let (parent_idx, parent_tag) = self.add_const_class(mod_name, class_name);
        let name = self.add_const_str(member_name);

        match parent_tag {
            TypeDefOrRef::TypeDef => {
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
            TypeDefOrRef::TypeRef => {
                let parent_tagged_idx = to_memberref_parent(parent_idx, MemberRefParent::TypeRef);
                let memberref = MemberRef {
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
            TypeDefOrRef::TypeSpec => unimplemented!(),
        }
    }

    /// mod_name: external module name
    ///
    /// name: external function name
    ///
    /// forwarded: forwarded method idx, index into methoddef tbl
    ///
    /// return: index into ImplMap tbl
    pub fn add_extern_fn(
        &mut self,
        mod_name: &str,
        name: &str,
        attrib: &PInvokeAttrib,
        forwarded: u32,
    ) -> u32 {
        let (scope, scope_tag) = self.add_const_mod(mod_name);
        assert_eq!(scope_tag, TokTag::ModRef);
        let name = self.add_const_str(name);
        let implmap_info = ImplMapInfo { scope, name };
        if let Some(idx) = self.implmap_map.get(&implmap_info) {
            let idx = *idx;
            if self.file.implmap_tbl[idx as usize - 1].flag != attrib.attrib {
                panic!("");
            }
            idx
        } else {
            self.file.implmap_tbl.push(ImplMap {
                member: to_implmap_member(forwarded, MemberForwarded::MethodDef),
                name,
                scope,
                flag: attrib.attrib,
            });
            let ret = self.file.implmap_tbl.len() as u32;
            self.implmap_map.insert(implmap_info, ret);
            ret
        }
    }
}
