use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use super::ast::ast::AST;
use super::ast::gen::gen;
use super::class_builder::ClassBuilder;
use super::module_mgr::ModuleMgr;
use crate::ir::flag::{Flag, FlagTag};
use crate::ir::var::VarType;

pub struct Var {
    pub id: String,
    pub flag: Flag,
    pub ty: VarType,
    pub offset: usize,
    pub initialized: bool,
}

impl Var {
    pub fn new(id: &str, flag: Flag, ty: VarType, offset: usize, initialized: bool) -> Var {
        Var {
            id: id.to_owned(),
            flag,
            ty,
            offset,
            initialized,
        }
    }
}

pub struct Locals {
    pub locals: Vec<Var>,
    pub size: usize,
    pub sym_tbl: Vec<HashMap<String, usize>>,
}

impl Locals {
    fn new() -> Locals {
        Locals {
            locals: Vec::new(),
            size: 0,
            sym_tbl: Vec::new(),
        }
    }

    fn push(&mut self) {
        self.sym_tbl.push(HashMap::new());
    }

    fn pop(&mut self) {
        self.sym_tbl.pop().expect("Cannot pop empty stack");
    }

    fn add(&mut self, id: &str, ty: VarType, flag: Flag, initialized: bool) {
        let var_size = ty.slot();
        let var = Var::new(id, flag, ty, self.size, initialized);
        self.sym_tbl
            .last_mut()
            .unwrap()
            .insert(id.to_owned(), self.locals.len());
        self.locals.push(var);
        self.size += var_size;
    }
}

pub struct Func {
    pub is_static: bool,
    pub ret_ty: VarType,
    pub ps_ty: Vec<VarType>,
    pub locals: RefCell<Locals>,
}

pub struct Class {
    pub path: Vec<String>,
    pub descriptor: String,
    ast_fields: Vec<Box<AST>>,
    ast_methods: Vec<Box<AST>>,
    pub fields: RefCell<HashMap<String, Box<Var>>>,
    // overload is not allowed
    pub methods: RefCell<HashMap<String, Box<Func>>>,
    builder: RefCell<ClassBuilder>,
}

pub struct CodeGenCtx<'mgr> {
    pub mgr: &'mgr ModuleMgr,
    pub class: &'mgr Class,
    pub method: &'mgr Func,
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

    pub fn get_type(&self, ast: &Box<AST>, mgr: &ModuleMgr) -> VarType {
        match ast.as_ref() {
            AST::I32Type => VarType::I32,
            AST::F64Type => VarType::F64,
            AST::BoolType => VarType::Bool,
            AST::None => VarType::Void,
            AST::TupleType(types) => {
                let mut ret: Vec<VarType> = Vec::new();
                for ty in types.iter() {
                    ret.push(self.get_type(ty, mgr));
                }
                VarType::Tuple(ret)
            }
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
                        locals: RefCell::new(Locals::new()),
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
                // Field will have default initialization
                let field = Box::new(Var::new(
                    id,
                    *flag,
                    self.get_type(ty, mgr),
                    if is_static { static_offset } else { obj_offset },
                    true,
                ));

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
        let ms = self.methods.borrow();
        for method_ast in self.ast_methods.iter() {
            if let AST::Func(id, _, ps, block) = method_ast.as_ref() {
                let m = ms.get(id).unwrap();
                // Create symbol table, put args into locals
                {
                    let mut local_mut = m.locals.borrow_mut();
                    local_mut.push();
                    for (p, ty) in ps.iter().zip(m.ps_ty.iter()) {
                        if let AST::Field(id, _, flag) = p.as_ref() {
                            // args will be initialized by caller
                            local_mut.add(id, ty.clone(), *flag, true);
                        } else {
                            panic!("Parser error");
                        }
                    }
                }

                let ctx = CodeGenCtx {
                    mgr,
                    class: self,
                    method: m.as_ref(),
                };
                let mut ret = VarType::Void;
                if let AST::Block(stmts) = block.as_ref() {
                    for stmt in stmts.iter() {
                        ret = gen(&ctx, stmt);
                    }
                } else {
                    panic!("Parser error")
                }
                // Check type match

                {
                    let mut local_mut = m.locals.borrow_mut();
                    local_mut.pop();
                    assert_eq!(
                        local_mut.sym_tbl.len(),
                        0,
                        "Symbol table is not empty after generation"
                    );
                }
                unimplemented!("Return type check is not implemented");
            } else {
                panic!("Parser error");
            }
        }
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
