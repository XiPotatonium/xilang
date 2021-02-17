use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use super::super::ast::ast::AST;
use super::class_builder::ClassBuilder;
use super::ctx::CodeGenCtx;
use super::gen::gen;
use super::member::{Field, Locals, Method};
use super::module_mgr::ModuleMgr;
use crate::ir::flag::{Flag, FlagTag};
use crate::ir::ty::{fn_descriptor, VarType};
use crate::ir::CLINIT_METHOD_NAME;

pub struct Class {
    pub path: Vec<String>,
    pub descriptor: String,
    ast_fields: Vec<Box<AST>>,
    ast_methods: Vec<Box<AST>>,
    // static init
    ast_init: Box<AST>,
    pub fields: RefCell<HashMap<String, Box<Field>>>,
    // overload is not allowed
    pub methods: RefCell<HashMap<String, Box<Method>>>,
    pub builder: RefCell<ClassBuilder>,
}

impl Class {
    pub fn new(module_path: &Vec<String>, ast: Box<AST>) -> Class {
        let mut class_path = module_path.to_owned();

        if let AST::Class(id, flag, methods, fields, init) = *ast {
            class_path.push(id);
            let fullname = class_path.join("/");
            Class {
                descriptor: format!("L{};", fullname),
                ast_fields: fields,
                ast_methods: methods,
                ast_init: init,
                fields: RefCell::new(HashMap::new()),
                methods: RefCell::new(HashMap::new()),
                builder: RefCell::new(ClassBuilder::new(&fullname, &flag)),
                path: class_path,
            }
        } else {
            unreachable!("Parser error");
        }
    }

    pub fn fullname(&self) -> String {
        self.path.join("/")
    }

    pub fn get_type(&self, ast: &Box<AST>, mgr: &ModuleMgr) -> VarType {
        match ast.as_ref() {
            AST::TypeI32 => VarType::Int,
            AST::TypeF64 => VarType::Double,
            AST::TypeBool => VarType::Boolean,
            AST::None => VarType::Void,
            AST::TypeTuple(types) => {
                unimplemented!();
            }
            AST::TypeClass(class_name) => {
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
                        return VarType::Class(class_des);
                    }
                }

                // Search in global
                let class_des = format!("L{};", class_name.join("/"));
                if mgr.class_table.contains_key(&class_des) {
                    VarType::Class(class_des)
                } else {
                    panic!("Class {} not found", class_des);
                }
            }
            AST::TypeArr(dtype, _) => VarType::Array(Box::new(self.get_type(dtype, mgr))),
            _ => unreachable!(),
        }
    }

    fn declare_method(&self, id: &str, flag: &Flag, ret_ty: VarType, ps: Vec<VarType>) {
        let method_idx =
            self.builder
                .borrow_mut()
                .add_method(id, &fn_descriptor(&ret_ty, &ps), flag);

        if let Some(_) = self.methods.borrow_mut().insert(
            id.to_owned(),
            Box::new(Method {
                ret_ty,
                ps_ty: ps,
                flag: flag.clone(),
                method_idx,
            }),
        ) {
            // TODO: use expect_none once it becomes stable
            panic!("Duplicated method {} in class {}", id, self.path.join("::"));
        }
    }

    pub fn member_pass(&self, mgr: &ModuleMgr) {
        // Add static init
        match self.ast_init.as_ref() {
            AST::Block(_) => {
                let ret_ty = VarType::Void;
                let ps: Vec<VarType> = vec![];
                let mut flag = Flag::default();
                flag.set(FlagTag::Static);

                self.declare_method(CLINIT_METHOD_NAME, &flag, ret_ty, ps);
            }
            AST::None => (),
            _ => unreachable!("Parser error"),
        };

        for method in self.ast_methods.iter() {
            if let AST::Func(id, flag, ty, ps, _) = method.as_ref() {
                let ps = ps
                    .iter()
                    .map(|p| {
                        if let AST::Param(_, _, ty) = p.as_ref() {
                            self.get_type(ty, mgr)
                        } else {
                            unreachable!();
                        }
                    })
                    .collect();
                let ret_ty = self.get_type(ty, mgr);
                self.declare_method(id, flag, ret_ty, ps);
            }
        }

        for field in self.ast_fields.iter() {
            if let AST::Field(id, flag, ty) = field.as_ref() {
                // Field will have default initialization
                let field = Box::new(Field::new(id, *flag, self.get_type(ty, mgr)));

                // Build Field in class file
                self.builder
                    .borrow_mut()
                    .add_field(id, &field.ty.descriptor(), flag);

                if let Some(_) = self.fields.borrow_mut().insert(id.clone(), field) {
                    // TODO: use expect_none once it becomes stable
                    panic!("Dulicated field {} in class {}", id, self.path.join("::"));
                }
            }
        }
    }

    fn code_gen_method(&self, mgr: &ModuleMgr, m: &Method, ps: &Vec<Box<AST>>, block: &Box<AST>) {
        // Create symbol table, put args into locals
        let mut locals = Locals::new();
        {
            locals.push();
            for (p, ty) in ps.iter().zip(m.ps_ty.iter()) {
                if let AST::Field(id, flag, _) = p.as_ref() {
                    // args will be initialized by caller
                    locals.add(id, ty.clone(), *flag, true);
                } else {
                    unreachable!("Parser error");
                }
            }
        }

        let ctx = CodeGenCtx {
            mgr,
            class: self,
            locals: RefCell::new(locals),
            method: m,
        };
        let mut ret = VarType::Void;
        if let AST::Block(stmts) = block.as_ref() {
            for stmt in stmts.iter() {
                ret = gen(&ctx, stmt);
            }
        } else {
            unreachable!("Parser error")
        }

        // Check type equivalent
        if ret != m.ret_ty {
            panic!();
        }

        // TODO add return instruction

        {
            let mut local_mut = ctx.locals.borrow_mut();
            local_mut.pop();
            assert_eq!(
                local_mut.sym_tbl.len(),
                0,
                "Symbol table is not empty after generation"
            );

            self.builder
                .borrow_mut()
                .done(ctx.method.method_idx, local_mut.size);
        }
    }

    /// Code Generation
    ///
    /// There is no default value for fields
    pub fn code_gen(&self, mgr: &ModuleMgr) {
        let ms = self.methods.borrow();
        // gen static init
        match self.ast_init.as_ref() {
            AST::Block(_) => {
                let m = ms.get(CLINIT_METHOD_NAME).unwrap();
                let ps: Vec<Box<AST>> = vec![];
                self.code_gen_method(mgr, m, &ps, &self.ast_init);
            }
            AST::None => (),
            _ => unreachable!("Parser error"),
        };

        for method_ast in self.ast_methods.iter() {
            if let AST::Func(id, _, _, ps, block) = method_ast.as_ref() {
                let m = ms.get(id).unwrap();
                self.code_gen_method(mgr, m, ps, block);
            } else {
                unreachable!("Parser error");
            }
        }
    }

    pub fn dump(&self, dir: &Path) {
        let module_name = &self.path[self.path.len() - 2];
        let class_name = &self.path[self.path.len() - 1];

        let ir = self.builder.borrow().class_file.to_text();
        let mut f =
            fs::File::create(dir.join(format!("{}.{}.xir", module_name, class_name))).unwrap();
        f.write_all(ir.as_bytes()).unwrap();

        let buf = self.builder.borrow().class_file.to_binary();
        let mut f =
            fs::File::create(dir.join(format!("{}.{}.xibc", module_name, class_name))).unwrap();
        f.write_all(&buf).unwrap();
    }
}
