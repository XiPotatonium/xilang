use core::{CCTOR_NAME, CTOR_NAME, STRING_CLASS_NAME};
use std::ptr::NonNull;

use super::super::ast::{ASTType, MethodFlag, MethodFlags, AST};
use super::super::sym::{Field, Method, Param, RValType, Struct};
use super::super::util::{IItemPath, ItemPathBuf};
use super::super::XiCfg;
use super::{MethodBuilder, ModuleBuilder};

pub struct StructBuilder {
    parent: NonNull<ModuleBuilder>,
    sym: NonNull<Struct>,
    ast: Box<AST>,
    pub methods: Vec<Box<MethodBuilder>>,
}

impl StructBuilder {
    pub fn new(
        parent: NonNull<ModuleBuilder>,
        strukt: NonNull<Struct>,
        ast: Box<AST>,
    ) -> StructBuilder {
        StructBuilder {
            parent,
            sym: strukt,
            ast,
            methods: vec![],
        }
    }

    /// declare method according to ast
    fn declare_method(
        &self,
        strukt_mut: &mut Struct,
        ast: Option<NonNull<AST>>,
    ) -> Box<MethodBuilder> {
        let (name, flags, ps, ret) = match ast {
            Some(ast) => {
                match unsafe { ast.as_ref() } {
                    AST::Block(_) => (
                        CCTOR_NAME,
                        MethodFlags::from(
                            u16::from(MethodFlag::Public) | u16::from(MethodFlag::Static),
                        ),
                        None,
                        RValType::Void,
                    ), // cctor
                    AST::Method(method) => {
                        if !method.custom_attribs.is_empty() {
                            unimplemented!()
                        }

                        (
                            method.name.as_str(),
                            method.flags,
                            Some(&method.ps),
                            self.get_rval_type(&method.ret),
                        )
                    }
                    _ => unreachable!(),
                }
            }
            None => {
                // default ctor
                (
                    CTOR_NAME,
                    MethodFlags::from(u16::from(MethodFlag::Public)),
                    None,
                    RValType::Void,
                )
            }
        };

        let ps = if let Some(ps) = ps {
            ps.iter()
                .map(|p| {
                    if let AST::Param(id, ty) = p.as_ref() {
                        Param {
                            id: id.to_owned(),
                            ty: self.get_rval_type(ty),
                        }
                    } else {
                        unreachable!();
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        let method = Box::new(Method {
            parent: NonNull::new(strukt_mut as *mut Struct).unwrap(),
            name: name.to_owned(),
            ret,
            ps,
            flags,
        });

        let ret = Box::new(MethodBuilder::new(
            NonNull::new(self as *const StructBuilder as *mut StructBuilder).unwrap(),
            NonNull::new(method.as_ref() as *const Method as *mut Method).unwrap(),
            ast,
        ));

        if strukt_mut.methods.contains_key(name) {
            // check duplication
            panic!("Duplicated method {}", method);
        } else {
            strukt_mut.methods.insert(name.to_owned(), method);
        }
        ret
    }

    /// declare methods and fields
    pub fn member_pass(&mut self) {
        let strukt_ast = if let AST::Struct(ast) = self.ast.as_ref() {
            ast
        } else {
            unreachable!()
        };

        let mut strukt_mut = unsafe { self.sym.as_mut() };

        // declare fields
        for field in strukt_ast.fields.iter() {
            if let AST::Field(field_ast) = field.as_ref() {
                // Field will have default initialization
                let ty = self.get_rval_type(field_ast.ty.as_ref());

                let field = Box::new(Field {
                    parent: self.sym,
                    name: field_ast.name.clone(),
                    flags: field_ast.flags,
                    ty,
                });

                if strukt_mut
                    .fields
                    .insert(field.name.to_owned(), field)
                    .is_some()
                {
                    // TODO: use expect_none once it becomes stable
                    panic!(
                        "Dulicated field {} in class {}",
                        field_ast.name, strukt_mut.path
                    );
                }
            }
        }

        // Add static init
        match strukt_ast.cctor.as_ref() {
            AST::Block(_) => {
                self.methods.push(self.declare_method(
                    &mut strukt_mut,
                    NonNull::new(strukt_ast.cctor.as_ref() as *const AST as *mut AST),
                ));
            }
            AST::None => (),
            _ => unreachable!("Parser error"),
        };

        // Add default object creator
        self.methods
            .push(self.declare_method(&mut strukt_mut, None));

        for method_ast in strukt_ast.methods.iter() {
            self.methods.push(self.declare_method(
                &mut strukt_mut,
                NonNull::new(method_ast.as_ref() as *const AST as *mut AST),
            ));
        }
    }

    pub fn code_gen(&mut self, cfg: &XiCfg) {
        for method_builder in self.methods.iter_mut() {
            method_builder.code_gen(cfg);
        }
    }

    pub fn dump(&self, cfg: &XiCfg) {
        let mut p = cfg.out_dir.clone();
        // output at {out_dir}/{mod_path_except_root}/{class_name}.xibc
        for seg in unsafe { self.sym.as_ref() }.path.iter().skip(1) {
            p.push(seg);
        }

        // dump byte code
        p.set_extension("xibc");
        // let buf = self.file.borrow().file.to_binary();
        // let mut f = fs::File::create(&p).unwrap();
        // f.write_all(&buf).unwrap();
    }
}

impl StructBuilder {
    /// item must exist
    pub fn resolve_struct_type(&self, path: &ItemPathBuf) -> NonNull<Struct> {
        let (has_crate, super_cnt, canonicalized_path) = path.canonicalize();
        let struct_id = canonicalized_path.get_self().unwrap();
        let mod_path = canonicalized_path.get_super();
        let module_builder = unsafe { self.parent.as_ref() };
        let module = unsafe { module_builder.module.as_ref() };
        let crate_builder = unsafe { module_builder.parent.as_ref() };
        if mod_path.len() == 0 {
            // this mod
            // might be a struct in this module
            if struct_id == "Self" {
                // refer to this struct
                self.sym
            } else if let Some(ty) = module.structs.get(struct_id) {
                // refer to another struct in this module
                NonNull::new(ty.as_ref() as *const Struct as *mut Struct).unwrap()
            } else {
                panic!("No struct {} in mod {}", struct_id, module.fullname());
            }
        } else {
            let m = if has_crate {
                // crate::...
                let mut m = ItemPathBuf::new();
                m.push(&crate_builder.krate.crate_name);
                for seg_id in mod_path.iter().skip(1) {
                    m.push(seg_id);
                }
                m
            } else if super_cnt != 0 {
                // super::...
                let mut m = module.mod_path.as_slice();
                for _ in (0..super_cnt).into_iter() {
                    m.to_super();
                }
                let mut m = m.to_owned();
                for seg_id in mod_path.iter().skip(super_cnt) {
                    m.push(seg_id);
                }
                m
            } else {
                let mut mod_path_iter = mod_path.iter();
                let r = mod_path_iter.next().unwrap();
                if let Some(m) = module_builder.use_map.get(r) {
                    // try use map
                    let mut m = m.clone();
                    for seg_id in mod_path_iter {
                        m.push(seg_id);
                    }
                    m
                } else if module.sub_mods.contains(r) {
                    // try sub modules
                    let mut m = module.mod_path.clone();
                    m.push(r);
                    for seg_id in mod_path_iter {
                        m.push(seg_id);
                    }
                    m
                } else {
                    panic!("Cannot resolve path {}", path);
                }
            };

            if let Some(m) = crate_builder.krate.mod_tbl.get(m.as_str()) {
                if let Some(ty) = m.structs.get(struct_id) {
                    NonNull::new(ty.as_ref() as *const Struct as *mut Struct).unwrap()
                } else {
                    panic!("Struct {} not found", struct_id);
                }
            } else {
                panic!("Module {} not found", m.as_str());
            }
        }
    }

    pub fn get_rval_type(&self, ast: &ASTType) -> RValType {
        match ast {
            ASTType::I32 => RValType::I32,
            ASTType::F64 => RValType::F64,
            ASTType::Bool => RValType::Bool,
            ASTType::None => RValType::Void,
            ASTType::Char => RValType::Char,
            ASTType::String => {
                let ty = self.resolve_struct_type(&ItemPathBuf::from_ir_path(STRING_CLASS_NAME));
                RValType::StructRef(ty)
            }
            ASTType::Tuple(_) => unimplemented!(),
            ASTType::UsrType(class_path) => {
                let ty = self.resolve_struct_type(class_path);
                RValType::StructRef(ty)
            }
            ASTType::Arr(dtype) => RValType::Array(Box::new(self.get_rval_type(dtype))),
        }
    }
}
