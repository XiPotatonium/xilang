use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use super::ast::ast::AST;
use super::class_builder::ClassBuilder;
use super::module_mgr::ModuleMgr;
use crate::ir::flag::{Flag, FlagTag};
use crate::ir::var::VarType;

struct Var {
    flag: Flag,
    ty: VarType,
    offset: usize,
}

struct Func {
    is_static: bool,
    ret_ty: VarType,
    ps_ty: Vec<VarType>,
    locals: Vec<Var>,
}

pub struct Class {
    pub path: Vec<String>,
    pub descriptor: String,
    ast_fields: Vec<Box<AST>>,
    ast_methods: Vec<Box<AST>>,
    fields: RefCell<HashMap<String, Box<Var>>>,
    // overload is not allowed
    methods: RefCell<HashMap<String, Box<Func>>>,
    builder: RefCell<ClassBuilder>,
}

impl Class {
    pub fn new(module_path: &Vec<String>, ast: Box<AST>) -> Class {
        let mut class_path = module_path.to_owned();

        if let AST::Class(id, methods, fields) = *ast {
            class_path.push(id);
            Class {
                descriptor: format!("L{};", class_path.join("/")),
                path: class_path,
                ast_fields: fields,
                ast_methods: methods,
                fields: RefCell::new(HashMap::new()),
                methods: RefCell::new(HashMap::new()),
                builder: RefCell::new(ClassBuilder::new()),
            }
        } else {
            panic!("Parser error");
        }
    }

    fn get_type(&self, ast: &Box<AST>, mgr: &ModuleMgr) -> VarType {
        match ast.as_ref() {
            AST::I32Type => VarType::I32,
            AST::F64Type => VarType::F64,
            AST::BoolType => VarType::Bool,
            AST::None => VarType::Void,
            AST::TupleType(_) => unimplemented!(),
            AST::ClassType(class_name) => {
                // TODO: use
                // Search in this module and global
                if class_name.len() == 0 {
                    panic!("Parser error");
                } else if class_name.len() == 1 {
                    // might be a class in this module
                    let class_des = format!(
                        "L{}/{};",
                        self.path[..self.path.len() - 1].join("/"),
                        class_name[0]
                    );
                    if mgr.class_table.contains_key(&class_des) {
                        return VarType::Obj(class_des);
                    }
                }

                // Search in global
                let class_des = format!("L{};", class_name.join("/"));
                if mgr.class_table.contains_key(&class_des) {
                    VarType::Obj(class_des)
                } else {
                    panic!("Class {} not found", class_des);
                }
            }
            AST::ArrType(dtype, _) => VarType::Arr(Box::new(self.get_type(dtype, mgr))),
            _ => unreachable!(),
        }
    }

    pub fn member_pass(&self, mgr: &ModuleMgr) {
        let mut methods_mut = self.methods.borrow_mut();
        for method in self.ast_methods.iter() {
            if let AST::Func(id, ty, ps, _) = method.as_ref() {
                let mut ps_: Vec<VarType> = Vec::new();
                let mut has_self = false;
                if ps.len() > 0 {
                    if let AST::Field(_, ty, _) = ps[0].as_ref() {
                        if let AST::None = ty.as_ref() {
                            // first param is "self"
                            has_self = true;
                        }
                    }
                }
                for p in ps.iter().skip(if has_self { 1 } else { 0 }) {
                    if let AST::Field(_, ty, _) = p.as_ref() {
                        ps_.push(self.get_type(ty, mgr));
                    }
                }
                if let Some(_) = methods_mut.insert(
                    id.clone(),
                    Box::new(Func {
                        ret_ty: self.get_type(ty, mgr),
                        ps_ty: ps_,
                        locals: Vec::new(),
                        is_static: !has_self,
                    }),
                ) {
                    // TODO: use expect_none once it becomes stable
                    panic!("Dulicated method {} in class {}", id, self.path.join("::"));
                }
            }
        }

        // FIXME: specify misc data size
        let mut obj_offset: usize = 0;
        let mut static_offset: usize = 0;
        let mut fields_mut = self.fields.borrow_mut();
        for field in self.ast_fields.iter() {
            if let AST::Field(id, ty, flag) = field.as_ref() {
                let is_static = flag.is(FlagTag::Static);
                let field = Box::new(Var {
                    flag: *flag,
                    ty: self.get_type(ty, mgr),
                    offset: if is_static { static_offset } else { obj_offset },
                });

                // currently no padding nor alignment
                if is_static {
                    static_offset += field.ty.size();
                } else {
                    obj_offset += field.ty.size();
                }

                if let Some(_) = fields_mut.insert(id.clone(), field) {
                    // TODO: use expect_none once it becomes stable
                    panic!("Dulicated field {} in class {}", id, self.path.join("::"));
                }
            }
        }
    }

    /// Code Generation
    ///
    /// There is no default value for fields
    pub fn code_gen(&self, mgr: &ModuleMgr) {
        unimplemented!();
    }

    pub fn dump(&self, dir: &Path) {
        let mut buf: Vec<u8> = Vec::new();
        self.builder.borrow().serialize(&mut buf);

        let module_name = &self.path[self.path.len() - 2];
        let class_name = &self.path[self.path.len() - 1];

        let path = dir.join(format!("{}.{}.xibc", module_name, class_name));

        let mut f = fs::File::create(path).unwrap();
        f.write_all(&buf).unwrap();
    }
}
