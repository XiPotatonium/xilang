use core::panic;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::rc::{Rc, Weak};

use crate::ir::flag::*;
use crate::ir::inst::Inst;
use crate::ir::path::{IModPath, ModPath};
use crate::ir::ty::{fn_descriptor, RValType};
use crate::ir::CLINIT_METHOD_NAME;

use super::super::ast::ast::AST;
use super::super::parser::peg_parser;
use super::builder::{Builder, MethodBuilder};
use super::class::Class;
use super::field::Field;
use super::gen::{gen, CodeGenCtx, ValType};
use super::method::Method;
use super::var::{Arg, Locals};
use super::xi_crate::Crate;

// use macro to avoid borrow mutable self twice, SB rust
macro_rules! declare_method {
    ($class: expr, $builder: expr, $id: expr, $flag: expr, $ret_ty: expr, $ps: expr) => {
        let method_idx =
            $builder
                .borrow_mut()
                .add_method($id, &fn_descriptor(&$ret_ty, &$ps), $flag);

        if let Some(_) = $class.methods.insert(
            $id.to_owned(),
            Box::new(Method {
                ret_ty: $ret_ty,
                ps_ty: $ps,
                ps_flag: vec![],
                flag: $flag.clone(),
                method_idx,
            }),
        ) {
            // TODO: use expect_none once it becomes stable
            panic!("Duplicated method {} in class {}", $id, $class.fullname);
        }
    };
}

pub struct Module {
    pub mod_path: ModPath,
    sub_mods: HashMap<String, Rc<Module>>,
    classes: HashMap<String, Rc<RefCell<Class>>>,
    /// Vec<Box<AST::Class>>
    class_asts: Vec<Box<AST>>,
    is_crate_root: bool,
    is_dir_mod: bool,
    use_map: HashMap<String, ModPath>,

    pub builder: RefCell<Builder>,
}

impl Module {
    /// Create a module from directory
    pub fn new(
        mod_path: ModPath,
        path: &Path,
        is_crate_root: bool,
        is_dir_mod: bool,
        class_tbl: &mut HashMap<String, Weak<RefCell<Class>>>,
        show_ast: bool,
    ) -> Rc<Module> {
        let dir = path.parent().unwrap();

        let mut builder = Builder::new();
        builder.set_mod(mod_path.get_self_name().unwrap());
        if is_crate_root {
            builder.set_crate(mod_path.get_self_name().unwrap());
        }

        let ast = peg_parser::parse(path).unwrap();

        if show_ast {
            // save ast to .json file
            let mut f = path.to_owned();
            f.set_extension("ast.json");
            let mut f = fs::File::create(f).unwrap();
            write!(f, "{}", ast).unwrap();
        }

        if let AST::File(mods, uses, classes) = *ast {
            // process uses
            let mut use_map: HashMap<String, ModPath> = HashMap::new();
            for use_ast in uses.iter() {
                if let AST::Use(raw_path, as_id) = use_ast.as_ref() {
                    let (path_has_crate, path_super_count, can_path) = raw_path.canonicalize();
                    let use_path = if path_has_crate {
                        let mut use_path = ModPath::new();
                        use_path.push(mod_path.get_root_name().unwrap());
                        for seg in can_path.range(1, can_path.len()).iter() {
                            use_path.push(seg);
                        }
                        use_path
                    } else if path_super_count != 0 {
                        let mut root_path = mod_path.get_super();
                        for _ in (0..path_super_count).into_iter() {
                            root_path.to_super();
                        }
                        let mut use_path = root_path.to_owned();
                        for seg in can_path.range(path_super_count, can_path.len()).iter() {
                            use_path.push(seg);
                        }
                        use_path
                    } else {
                        can_path
                    };

                    let as_id = if let Some(as_id) = as_id {
                        as_id.to_owned()
                    } else {
                        use_path.get_self_name().unwrap().to_owned()
                    };

                    if use_map.contains_key(&as_id) {
                        panic!("Duplicated use as {}", as_id);
                    } else {
                        use_map.insert(as_id, use_path);
                    }
                } else {
                    unreachable!();
                }
            }

            // generate all classes
            let mut class_map: HashMap<String, Rc<RefCell<Class>>> = HashMap::new();
            for class in classes.iter() {
                if let AST::Class(id, flag, _, _, _) = class.as_ref() {
                    let idx = builder.add_class(id, flag);
                    let class_fullname = format!("{}/{}", mod_path.as_str(), id);
                    let class = Rc::new(RefCell::new(Class::new(class_fullname.clone(), idx)));
                    class_tbl.insert(class_fullname, Rc::downgrade(&class));
                    class_map.insert(id.to_owned(), class);
                } else {
                    unreachable!();
                }
            }

            // process sub mods
            let mut sub_modules: HashMap<String, Rc<Module>> = HashMap::new();
            for sub_mod_name in mods.into_iter() {
                if sub_modules.contains_key(&sub_mod_name) {
                    panic!(
                        "Sub-module {} is defined multiple times in {}",
                        sub_mod_name,
                        mod_path.as_str()
                    );
                }

                let dir = if is_crate_root || is_dir_mod {
                    // this is the root mod in this dir
                    // search sub modules in this directory
                    dir.to_owned()
                } else {
                    // this is a normal file mod
                    // search sub modules in directory dir/mod_name
                    dir.join(mod_path.get_self_name().unwrap())
                };

                let mut sub_mod_dir_path = dir.join(&sub_mod_name);
                let has_sub_dir_mod = sub_mod_dir_path.is_dir() && {
                    sub_mod_dir_path.push("mod.xi");
                    sub_mod_dir_path.is_file()
                };
                let sub_mod_fpath = dir.join(format!("{}.xi", sub_mod_name));
                let has_sub_mod = sub_mod_fpath.is_file();

                let mut sub_mod_path = mod_path.clone();
                sub_mod_path.push(&sub_mod_name);
                if has_sub_dir_mod && has_sub_mod {
                    panic!(
                        "Ambiguous sub-module {} in {}. {} or {}?",
                        sub_mod_name,
                        mod_path.as_str(),
                        sub_mod_dir_path.to_str().unwrap(),
                        sub_mod_fpath.to_str().unwrap()
                    );
                } else if has_sub_dir_mod {
                    sub_modules.insert(
                        sub_mod_name,
                        Module::new(
                            sub_mod_path,
                            &sub_mod_dir_path,
                            false,
                            true,
                            class_tbl,
                            show_ast,
                        ),
                    );
                } else if has_sub_mod {
                    sub_modules.insert(
                        sub_mod_name,
                        Module::new(
                            sub_mod_path,
                            &sub_mod_fpath,
                            false,
                            false,
                            class_tbl,
                            show_ast,
                        ),
                    );
                } else {
                    panic!(
                        "Cannot find sub-module {} in {}",
                        sub_mod_name,
                        mod_path.as_str()
                    );
                }
            }

            Rc::new(Module {
                sub_mods: sub_modules,
                mod_path,
                classes: class_map,
                class_asts: classes,
                is_crate_root,
                is_dir_mod,
                use_map,

                builder: RefCell::new(builder),
            })
        } else {
            unreachable!();
        }
    }

    pub fn name(&self) -> &str {
        self.mod_path.get_self_name().unwrap()
    }

    /// Display module and its sub-modules
    pub fn tree(&self, depth: usize) {
        if depth > 0 {
            print!("{}+---", "|   ".repeat(depth - 1));
        }
        println!("{}", self.name());
        for (_, sub) in self.sub_mods.iter() {
            sub.tree(depth + 1);
        }
    }

    pub fn dump(&self, dir: &Path) {
        let fstem = self.name();

        // dump in this dir
        let mut f = fs::File::create(dir.join(format!("{}.xir", fstem))).unwrap();
        write!(f, "{}", self.builder.borrow().file).unwrap();

        let buf = self.builder.borrow().file.to_binary();
        let mut f = fs::File::create(dir.join(format!("{}.xibc", fstem))).unwrap();
        f.write_all(&buf).unwrap();

        for sub in self.sub_mods.values() {
            if sub.is_dir_mod {
                // create a new sub dir
                let mut sub_dir = dir.to_owned();
                sub_dir.push(sub.name());
                if !sub_dir.exists() {
                    fs::create_dir(&sub_dir).unwrap();
                } else if !sub_dir.is_dir() {
                    panic!(
                        "{} already exists but it is not a directory",
                        sub_dir.to_str().unwrap()
                    );
                }
                sub.dump(&sub_dir);
            } else {
                sub.dump(&dir);
            }
        }
    }
}

impl Module {
    pub fn get_ty(&self, ast: &Box<AST>, c: &Crate) -> RValType {
        match ast.as_ref() {
            AST::TypeI32 => RValType::I32,
            AST::TypeF64 => RValType::F64,
            AST::TypeBool => RValType::Bool,
            AST::None => RValType::Void,
            AST::TypeTuple(types) => {
                unimplemented!();
            }
            AST::TypeClass(class_path) => {
                // TODO: use
                // Search in this module and global
                if class_path.len() == 0 {
                    panic!("Parser error");
                } else if class_path.len() == 1 {
                    // might be a class in this module
                    let class_fullname = format!(
                        "{}/{}",
                        self.mod_path.as_str(),
                        class_path.get_self_name().unwrap()
                    );
                    if c.class_tbl.contains_key(&class_fullname) {
                        return RValType::Obj(class_fullname);
                    }
                }

                // Search in global
                let class_fullname = class_path.as_str().to_owned();
                if c.class_tbl.contains_key(&class_fullname) {
                    RValType::Obj(class_fullname)
                } else {
                    panic!("Class {} not found", class_fullname);
                }
            }
            AST::TypeArr(dtype, _) => RValType::Array(Box::new(self.get_ty(dtype, c))),
            _ => unreachable!(),
        }
    }
}

// member pass
impl Module {
    pub fn member_pass(&self, c: &Crate) {
        for class in self.class_asts.iter() {
            if let AST::Class(class_id, _, ast_methods, ast_fields, static_init) = class.as_ref() {
                let mut class_mut = self.classes.get(class_id).unwrap().borrow_mut();
                // Add static init
                match static_init.as_ref() {
                    AST::Block(_) => {
                        let ret_ty = RValType::Void;
                        let ps: Vec<RValType> = vec![];
                        let mut flag = MethodFlag::default();
                        flag.set(MethodFlagTag::Static);
                        declare_method!(
                            class_mut,
                            self.builder,
                            CLINIT_METHOD_NAME,
                            &flag,
                            ret_ty,
                            ps
                        );
                    }
                    AST::None => (),
                    _ => unreachable!("Parser error"),
                };

                for method in ast_methods.iter() {
                    if let AST::Method(id, flag, ty, ps, _) = method.as_ref() {
                        let ps = ps
                            .iter()
                            .map(|p| {
                                if let AST::Param(_, _, ty) = p.as_ref() {
                                    self.get_ty(ty, c)
                                } else {
                                    unreachable!();
                                }
                            })
                            .collect();
                        let ret_ty = self.get_ty(ty, c);
                        declare_method!(class_mut, self.builder, id, flag, ret_ty, ps);
                    }
                }

                for field in ast_fields.iter() {
                    if let AST::Field(id, flag, ty) = field.as_ref() {
                        // Field will have default initialization
                        let field = Box::new(Field::new(id, *flag, self.get_ty(ty, c)));

                        // Build Field in class file
                        self.builder
                            .borrow_mut()
                            .add_field(id, &field.ty.descriptor(), flag);

                        if !flag.is(FieldFlagTag::Static) {
                            // non-static field
                            class_mut.non_static_fields.push(id.to_owned());
                        }
                        if let Some(_) = class_mut.fields.insert(id.to_owned(), field) {
                            // TODO: use expect_none once it becomes stable
                            panic!("Dulicated field {} in class {}", id, class_mut.fullname);
                        }
                    }
                }

                if self.is_crate_root && class_id == "Program" {
                    if let Some(m) = class_mut.methods.get("main") {
                        if let RValType::Void = m.ret_ty {
                            if m.ps_ty.len() == 0
                                && m.flag.is(MethodFlagTag::Pub)
                                && m.flag.is(MethodFlagTag::Static)
                            {
                                // pub Program::main()
                                self.builder.borrow_mut().file.crate_tbl[0].entrypoint =
                                    m.method_idx;
                            }
                        }
                    }
                }
            } else {
                unreachable!();
            }
        }

        // recursive
        for sub in self.sub_mods.values() {
            sub.member_pass(c);
        }
    }
}

// code gen
impl Module {
    fn code_gen_method(
        &self,
        c: &Crate,
        class: &Class,
        m: &Method,
        ps: &Vec<Box<AST>>,
        block: &Box<AST>,
    ) {
        let mut args_map: HashMap<String, Arg> = HashMap::new();
        if !m.flag.is(MethodFlagTag::Static) {
            // non-static method param "self"
            args_map.insert(
                String::from("self"),
                Arg::new(
                    Default::default(),
                    RValType::Obj(class.fullname.clone()),
                    args_map.len() as u16,
                ),
            );
        }
        for (p, ty) in ps.iter().zip(m.ps_ty.iter()) {
            if let AST::Param(id, flag, _) = p.as_ref() {
                // args will be initialized by caller
                args_map.insert(
                    id.to_owned(),
                    Arg::new(*flag, ty.clone(), args_map.len() as u16),
                );
            } else {
                unreachable!("Parser error");
            }
        }

        let ctx = CodeGenCtx {
            mgr: c,
            module: self,
            class,
            locals: RefCell::new(Locals::new()),
            method: m,
            args_map,
            method_builder: RefCell::new(MethodBuilder::new()),
        };
        let ret = gen(&ctx, block);

        // Check type equivalent
        match &ret {
            ValType::RVal(rval_ty) => {
                if rval_ty != &m.ret_ty {
                    panic!("Expect return {} but return {}", m.ret_ty, rval_ty);
                }
                // Add return instruction
                ctx.method_builder.borrow_mut().add_inst(Inst::Ret);
            }
            ValType::Ret(ret_ty) => {
                if ret_ty != &m.ret_ty {
                    panic!("Expect return {} but return {}", m.ret_ty, ret_ty);
                }
            }
            _ => unreachable!(),
        }

        ctx.done();
    }

    pub fn code_gen(&self, c: &Crate) {
        for class in self.class_asts.iter() {
            if let AST::Class(id, _, ast_methods, _, ast_init) = class.as_ref() {
                let class_ref = self.classes.get(id).unwrap().borrow();
                // gen static init
                match ast_init.as_ref() {
                    AST::Block(_) => {
                        let m = class_ref.methods.get(CLINIT_METHOD_NAME).unwrap();
                        let ps: Vec<Box<AST>> = vec![];
                        self.code_gen_method(c, &class_ref, m, &ps, ast_init);
                    }
                    AST::None => (),
                    _ => unreachable!("Parser error"),
                };

                for method_ast in ast_methods.iter() {
                    if let AST::Method(id, _, _, ps, block) = method_ast.as_ref() {
                        let m = class_ref.methods.get(id).unwrap();
                        self.code_gen_method(c, &class_ref, m, ps, block);
                    } else {
                        unreachable!("Parser error");
                    }
                }
            } else {
                unreachable!();
            }
        }

        // recursive
        for sub in self.sub_mods.values() {
            sub.code_gen(c);
        }
    }
}
