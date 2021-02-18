use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use super::super::ast::ast::AST;
use super::class_builder::ClassBuilder;
use super::ctx::{CodeGenCtx, Locals};
use super::gen::{gen, ValType};
use super::member::{Field, Method};
use super::module_mgr::ModuleMgr;
use crate::ir::flag::{Flag, FlagTag};
use crate::ir::inst::Inst;
use crate::ir::ty::{fn_descriptor, RValType};
use crate::ir::CLINIT_METHOD_NAME;

pub struct Class {
    pub path: Vec<String>,
    pub fullname: String,
    ast_fields: Option<Vec<Box<AST>>>,
    ast_methods: Option<Vec<Box<AST>>>,
    // static init
    ast_init: Option<Box<AST>>,
    pub non_static_fields: Vec<String>,
    pub fields: HashMap<String, Box<Field>>,
    // overload is not allowed
    pub methods: HashMap<String, Box<Method>>,
    pub builder: RefCell<ClassBuilder>,
}

// use macro to avoid borrow mutable self twice, SB rust
macro_rules! declare_method {
    ($self: ident, $id: expr, $flag: expr, $ret_ty: expr, $ps: expr) => {
        let method_idx =
            $self
                .builder
                .borrow_mut()
                .add_method($id, &fn_descriptor(&$ret_ty, &$ps), $flag);

        if let Some(_) = $self.methods.insert(
            $id.to_owned(),
            Box::new(Method {
                ret_ty: $ret_ty,
                ps_ty: $ps,
                flag: $flag.clone(),
                method_idx,
            }),
        ) {
            // TODO: use expect_none once it becomes stable
            panic!(
                "Duplicated method {} in class {}",
                $id,
                $self.path.join("::")
            );
        }
    };
}

impl Class {
    pub fn new(module_path: &Vec<String>, ast: Box<AST>) -> Class {
        let mut class_path = module_path.to_owned();

        if let AST::Class(id, flag, methods, fields, init) = *ast {
            class_path.push(id);
            let fullname = class_path.join("/");
            let builder = RefCell::new(ClassBuilder::new(&fullname, &flag));
            Class {
                fullname,
                ast_fields: Some(fields),
                ast_methods: Some(methods),
                ast_init: Some(init),
                non_static_fields: Vec::new(),
                fields: HashMap::new(),
                methods: HashMap::new(),
                builder,
                path: class_path,
            }
        } else {
            unreachable!("Parser error");
        }
    }

    pub fn descriptor(&self) -> String {
        format!("L{};", self.fullname)
    }

    pub fn get_type(&self, ast: &Box<AST>, mgr: &ModuleMgr) -> RValType {
        match ast.as_ref() {
            AST::TypeI32 => RValType::I32,
            AST::TypeF64 => RValType::F64,
            AST::TypeBool => RValType::Bool,
            AST::None => RValType::Void,
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
                    let class_fullname = format!(
                        "{}/{}",
                        self.path[..self.path.len() - 1].join("/"),
                        class_name[0]
                    );
                    if mgr.class_table.contains_key(&class_fullname) {
                        return RValType::Obj(class_fullname);
                    }
                }

                // Search in global
                let class_fullname = class_name.join("/");
                if mgr.class_table.contains_key(&class_fullname) {
                    RValType::Obj(class_fullname)
                } else {
                    panic!("Class {} not found", class_fullname);
                }
            }
            AST::TypeArr(dtype, _) => RValType::Array(Box::new(self.get_type(dtype, mgr))),
            _ => unreachable!(),
        }
    }

    pub fn member_pass(&mut self, mgr: &ModuleMgr) {
        // Add static init
        match self.ast_init.as_ref().unwrap().as_ref() {
            AST::Block(_) => {
                let ret_ty = RValType::Void;
                let ps: Vec<RValType> = vec![];
                let mut flag = Flag::default();
                flag.set(FlagTag::Static);
                declare_method!(self, CLINIT_METHOD_NAME, &flag, ret_ty, ps);
            }
            AST::None => (),
            _ => unreachable!("Parser error"),
        };

        for method in self.ast_methods.as_ref().unwrap().iter() {
            if let AST::Method(id, flag, ty, ps, _) = method.as_ref() {
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
                declare_method!(self, id, flag, ret_ty, ps);
            }
        }

        for field in self.ast_fields.as_ref().unwrap().iter() {
            if let AST::Field(id, flag, ty) = field.as_ref() {
                // Field will have default initialization
                let field = Box::new(Field::new(id, *flag, self.get_type(ty, mgr)));

                // Build Field in class file
                self.builder
                    .borrow_mut()
                    .add_field(id, &field.ty.descriptor(), flag);

                if !flag.is(FlagTag::Static) {
                    // non-static field
                    self.non_static_fields.push(id.to_owned());
                }
                if let Some(_) = self.fields.insert(id.to_owned(), field) {
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
            if !m.flag.is(FlagTag::Static) {
                // non-static method variable "self"
                locals.add(
                    "self",
                    RValType::Obj(self.fullname.clone()),
                    Default::default(),
                    true,
                );
            }
            for (p, ty) in ps.iter().zip(m.ps_ty.iter()) {
                if let AST::Param(id, flag, _) = p.as_ref() {
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
        let mut ret = ValType::RVal(RValType::Void);
        if let AST::Block(stmts) = block.as_ref() {
            for stmt in stmts.iter() {
                ret = gen(&ctx, stmt);
            }

            // Check type equivalent
            match &ret {
                ValType::RVal(rval_ty) => {
                    if rval_ty != &m.ret_ty {
                        panic!("Expect return {} but return {}", m.ret_ty, rval_ty);
                    }
                    // Add return instruction
                    ctx.class
                        .builder
                        .borrow_mut()
                        .add_inst(ctx.method.method_idx, Inst::Ret);
                }
                ValType::Ret(ret_ty) => {
                    if ret_ty != &m.ret_ty {
                        panic!("Expect return {} but return {}", m.ret_ty, ret_ty);
                    }
                }
                _ => unreachable!(),
            }
        } else {
            unreachable!("Parser error")
        }

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
                .done(ctx.method.method_idx, local_mut.size());
        }
    }

    /// Code Generation
    ///
    /// There is no default value for fields
    pub fn code_gen(&self, mgr: &ModuleMgr) {
        // gen static init
        match self.ast_init.as_ref().unwrap().as_ref() {
            AST::Block(_) => {
                let m = self.methods.get(CLINIT_METHOD_NAME).unwrap();
                let ps: Vec<Box<AST>> = vec![];
                self.code_gen_method(mgr, m, &ps, &self.ast_init.as_ref().unwrap());
            }
            AST::None => (),
            _ => unreachable!("Parser error"),
        };

        for method_ast in self.ast_methods.as_ref().unwrap().iter() {
            if let AST::Method(id, _, _, ps, block) = method_ast.as_ref() {
                let m = self.methods.get(id).unwrap();
                self.code_gen_method(mgr, m, ps, block);
            } else {
                unreachable!("Parser error");
            }
        }
    }

    pub fn dump(&self, dir: &Path) {
        let module_name = &self.path[self.path.len() - 2];
        let class_name = &self.path[self.path.len() - 1];

        let mut f =
            fs::File::create(dir.join(format!("{}.{}.xir", module_name, class_name))).unwrap();
        write!(f, "{}", self.builder.borrow().class_file);

        let buf = self.builder.borrow().class_file.to_binary();
        let mut f =
            fs::File::create(dir.join(format!("{}.{}.xibc", module_name, class_name))).unwrap();
        f.write_all(&buf).unwrap();
    }
}
