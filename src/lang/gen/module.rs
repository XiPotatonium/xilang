use lazy_static::lazy_static;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};

use crate::ir::flag::*;
use crate::ir::inst::Inst;
use crate::ir::ty::{fn_descriptor, RValType};
use crate::ir::CLINIT_METHOD_NAME;

use super::super::ast::ast::AST;
use super::super::parser::peg_parser;
use super::builder::{Builder, MethodBuilder};
use super::class::Class;
use super::field::Field;
use super::gen::{gen, ValType ,CodeGenCtx};
use super::method::Method;
use super::module_mgr::ModuleMgr;
use super::var::{Arg, Locals};

// use macro to avoid borrow mutable self twice, SB rust
macro_rules! declare_method {
    ($class: expr, $builder: expr, $id: expr, $flag: expr, $ret_ty: expr, $ps: expr) => {
        let method_idx = $builder.borrow_mut().add_method(
            $class.idx,
            $id,
            &fn_descriptor(&$ret_ty, &$ps),
            $flag,
        );

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
    pub name: String,
    pub fullname: String,
    sub_modules: HashMap<String, Rc<Module>>,
    classes: HashMap<String, Rc<RefCell<Class>>>,
    /// Vec<Box<AST::File>>
    asts: Vec<Box<AST>>,
    from_dir: bool,

    pub builder: RefCell<Builder>,
}

lazy_static! {
    // Same as identifier
    static ref NAME_RULE : regex::Regex = regex::Regex::new(r"^[_a-zA-Z][_a-zA-Z0-9]*").unwrap();
}

fn check_module_name_validity(name: &str) -> bool {
    NAME_RULE.is_match(name)
}

fn parse(
    module_path: &Vec<String>,
    paths: &Vec<PathBuf>,
    builder: &mut Builder,
    class_tbl: &mut HashMap<String, Weak<RefCell<Class>>>,
    show_ast: bool,
) -> (Vec<Box<AST>>, HashMap<String, Rc<RefCell<Class>>>) {
    let mut class_map: HashMap<String, Rc<RefCell<Class>>> = HashMap::new();
    let mut asts: Vec<Box<AST>> = Vec::new();
    for path in paths.iter() {
        let ast = peg_parser::parse(path).unwrap();

        if show_ast {
            // save ast to .json file
            let mut f = PathBuf::from(path);
            f.set_extension("ast.json");
            let mut f = fs::File::create(f).unwrap();
            write!(f, "{}", ast);
        }

        if let AST::File(_, _, classes) = ast.as_ref() {
            for class in classes {
                if let AST::Class(id, flag, _, _, _) = class.as_ref() {
                    let idx = builder.add_class(id, flag);
                    let class_fullname = format!("{}/{}", module_path.join("/"), id);
                    let class = Rc::new(RefCell::new(Class::new(class_fullname.clone(), idx)));
                    class_tbl.insert(class_fullname, Rc::downgrade(&class));
                    class_map.insert(id.to_owned(), class);
                } else {
                    unreachable!();
                }
            }
        } else {
            unreachable!();
        }

        asts.push(ast);
    }

    (asts, class_map)
}

impl Module {
    /// Create a module from files
    fn new(
        module_path: &Vec<String>,
        paths: &Vec<PathBuf>,
        class_tbl: &mut HashMap<String, Weak<RefCell<Class>>>,
        save_json: bool,
    ) -> Rc<Module> {
        let fullname = module_path.join("/");
        let mut builder = Builder::new(&fullname);
        let (asts, classes) = parse(&module_path, paths, &mut builder, class_tbl, save_json);
        Rc::new(Module {
            sub_modules: HashMap::new(),
            name: module_path.last().unwrap().clone(),
            fullname,
            asts: asts,
            classes: classes,
            from_dir: false,

            builder: RefCell::new(builder),
        })
    }

    /// Create a module from directory
    pub fn new_dir(
        module_path: Vec<String>,
        dir: &Path,
        class_tbl: &mut HashMap<String, Weak<RefCell<Class>>>,
        show_ast: bool,
    ) -> Option<Rc<Module>> {
        let mut files: Vec<PathBuf> = Vec::new();
        let mut leaf_sub_mods: HashMap<String, Vec<PathBuf>> = HashMap::new();
        let mut non_leaf_sub_mods: HashMap<String, PathBuf> = HashMap::new();

        // This is a non-leaf (directory) module. Find all Mod\.(.*\.)?xi
        for entry in dir.read_dir().unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            let file_name = entry.file_name().into_string().unwrap();
            let file_ty = entry.file_type().unwrap();

            if file_ty.is_dir() && check_module_name_validity(&file_name) {
                // might be a non-leaf sub-module
                // filename is sub module name
                if non_leaf_sub_mods.contains_key(&file_name) {
                    unreachable!();
                } else {
                    non_leaf_sub_mods.insert(String::from(file_name), entry_path);
                }
            } else if file_ty.is_file() && file_name.ends_with(".xi") {
                // find a .xi file
                if file_name.starts_with("Mod.") {
                    // current module file detected
                    files.push(entry_path);
                } else {
                    // leaf sub-module find
                    // TODO: use split_once() if it becomes stable
                    let module_name = file_name.split('.').next().unwrap();
                    if check_module_name_validity(module_name) {
                        if leaf_sub_mods.contains_key(module_name) {
                            leaf_sub_mods.get_mut(module_name).unwrap().push(entry_path);
                        } else {
                            leaf_sub_mods.insert(String::from(module_name), vec![entry_path]);
                        }
                    }
                }
            }
        }

        let fullname = module_path.join("/");
        let mut builder = Builder::new(&fullname);
        let (asts, classes) = parse(&module_path, &files, &mut builder, class_tbl, show_ast);
        let mut sub_modules: HashMap<String, Rc<Module>> = HashMap::new();

        for (leaf_sub_mod_name, file_paths) in leaf_sub_mods.iter() {
            let mut sub_module_path = module_path.to_vec();
            sub_module_path.push(String::from(leaf_sub_mod_name));
            let sub = Module::new(&sub_module_path, file_paths, class_tbl, show_ast);
            if sub_modules.contains_key(leaf_sub_mod_name) {
                panic!(
                    "Duplicate module {} in {}",
                    leaf_sub_mod_name,
                    dir.to_str().unwrap()
                );
            } else {
                sub_modules.insert(String::from(leaf_sub_mod_name), sub);
            }
        }

        for (non_leaf_sub_mod_name, dir_path) in non_leaf_sub_mods.iter() {
            let mut sub_module_path = module_path.to_vec();
            sub_module_path.push(String::from(non_leaf_sub_mod_name));
            let sub = Module::new_dir(sub_module_path, dir_path, class_tbl, show_ast);
            match sub {
                Some(sub) => {
                    if sub_modules.contains_key(non_leaf_sub_mod_name) {
                        panic!(
                            "Duplicate module {} in {}",
                            non_leaf_sub_mod_name,
                            dir.to_str().unwrap()
                        );
                    } else {
                        sub_modules.insert(non_leaf_sub_mod_name.to_owned(), sub);
                    }
                }
                None => continue,
            }
        }

        if classes.len() == 0 && sub_modules.len() == 0 {
            return None;
        }

        Some(Rc::new(Module {
            sub_modules,
            name: module_path.last().unwrap().clone(),
            fullname,
            classes,
            asts,
            from_dir: true,

            builder: RefCell::new(builder),
        }))
    }

    /// Display module and its sub-modules
    pub fn tree(&self, depth: usize) {
        if depth > 0 {
            print!("{}+---", "|   ".repeat(depth - 1));
        }
        println!("{}", self.name);
        for (_, sub) in self.sub_modules.iter() {
            sub.tree(depth + 1);
        }
    }

    pub fn dump(&self, dir: &PathBuf) {
        // dump in this dir
        let mut f = fs::File::create(dir.join(format!("{}.xir", self.name))).unwrap();
        write!(f, "{}", self.builder.borrow().file);

        let buf = self.builder.borrow().file.to_binary();
        let mut f = fs::File::create(dir.join(format!("{}.xibc", self.name))).unwrap();
        f.write_all(&buf).unwrap();

        for sub in self.sub_modules.values() {
            if sub.from_dir {
                // create a new sub dir
                let mut sub_dir = dir.to_owned();
                sub_dir.push(&sub.name);
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
    fn process_use_path(&self, mgr: &ModuleMgr, uses: &Vec<Box<AST>>) -> HashMap<String, String> {
        let mut use_map: HashMap<String, String> = HashMap::new();
        for use_ast in uses.iter() {
            if let AST::Use(raw_paths, as_id) = use_ast.as_ref() {
                let mut paths: Vec<&str> = Vec::new();
                let mut raw_paths_iter = raw_paths.iter();
                let first_path_seg = raw_paths_iter.next().unwrap();
                if first_path_seg == "super" {
                    unimplemented!();
                } else if first_path_seg == "crate" {
                    paths.push(&mgr.root.name)
                } else {
                    paths.push(first_path_seg);
                }

                for seg in raw_paths_iter {
                    paths.push(seg);
                }

                let as_id = if let Some(as_id) = as_id {
                    as_id.to_owned()
                } else {
                    (*paths.last().unwrap()).to_owned()
                };

                if use_map.contains_key(&as_id) {
                    panic!("Duplicated use as {}", as_id);
                } else {
                    use_map.insert(as_id, paths.join("::"));
                }
            } else {
                unreachable!();
            }
        }
        use_map
    }

    pub fn get_ty(
        &self,
        ast: &Box<AST>,
        mgr: &ModuleMgr,
        use_map: &HashMap<String, String>,
    ) -> RValType {
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
                    let class_fullname = format!("{}/{}", self.fullname, class_name[0]);
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
            AST::TypeArr(dtype, _) => RValType::Array(Box::new(self.get_ty(dtype, mgr, use_map))),
            _ => unreachable!(),
        }
    }
}

// member pass
impl Module {
    pub fn member_pass(&self, mgr: &ModuleMgr) {
        for file in self.asts.iter() {
            if let AST::File(_, uses, classes) = file.as_ref() {
                let use_map = self.process_use_path(mgr, uses);

                for class in classes.iter() {
                    if let AST::Class(id, _, ast_methods, ast_fields, static_init) = class.as_ref()
                    {
                        let mut class_mut = self.classes.get(id).unwrap().borrow_mut();
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
                                            self.get_ty(ty, mgr, &use_map)
                                        } else {
                                            unreachable!();
                                        }
                                    })
                                    .collect();
                                let ret_ty = self.get_ty(ty, mgr, &use_map);
                                declare_method!(class_mut, self.builder, id, flag, ret_ty, ps);
                            }
                        }

                        for field in ast_fields.iter() {
                            if let AST::Field(id, flag, ty) = field.as_ref() {
                                // Field will have default initialization
                                let field = Box::new(Field::new(
                                    id,
                                    *flag,
                                    self.get_ty(ty, mgr, &use_map),
                                ));

                                // Build Field in class file
                                self.builder.borrow_mut().add_field(
                                    class_mut.idx,
                                    id,
                                    &field.ty.descriptor(),
                                    flag,
                                );

                                if !flag.is(FieldFlagTag::Static) {
                                    // non-static field
                                    class_mut.non_static_fields.push(id.to_owned());
                                }
                                if let Some(_) = class_mut.fields.insert(id.to_owned(), field) {
                                    // TODO: use expect_none once it becomes stable
                                    panic!(
                                        "Dulicated field {} in class {}",
                                        id, class_mut.fullname
                                    );
                                }
                            }
                        }
                    } else {
                        unreachable!();
                    }
                }
            } else {
                unreachable!();
            }
        }

        // recursive
        for sub in self.sub_modules.values() {
            sub.member_pass(mgr);
        }
    }
}

// code gen
impl Module {
    fn code_gen_method(
        &self,
        mgr: &ModuleMgr,
        use_map: &HashMap<String, String>,
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
            mgr,
            module: self,
            class,
            locals: RefCell::new(Locals::new()),
            method: m,
            args_map,
            use_map,
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

    pub fn code_gen(&self, mgr: &ModuleMgr) {
        for file in self.asts.iter() {
            if let AST::File(_, uses, classes) = file.as_ref() {
                let use_map = self.process_use_path(mgr, uses);

                for class in classes.iter() {
                    if let AST::Class(id, _, ast_methods, _, ast_init) = class.as_ref() {
                        let class_ref = self.classes.get(id).unwrap().borrow();
                        // gen static init
                        match ast_init.as_ref() {
                            AST::Block(_) => {
                                let m = class_ref.methods.get(CLINIT_METHOD_NAME).unwrap();
                                let ps: Vec<Box<AST>> = vec![];
                                self.code_gen_method(mgr, &use_map, &class_ref, m, &ps, ast_init);
                            }
                            AST::None => (),
                            _ => unreachable!("Parser error"),
                        };

                        for method_ast in ast_methods.iter() {
                            if let AST::Method(id, _, _, ps, block) = method_ast.as_ref() {
                                let m = class_ref.methods.get(id).unwrap();
                                self.code_gen_method(mgr, &use_map, &class_ref, m, ps, block);
                            } else {
                                unreachable!("Parser error");
                            }
                        }
                    } else {
                        unreachable!();
                    }
                }
            } else {
                unreachable!();
            }
        }

        // recursive
        for sub in self.sub_modules.values() {
            sub.code_gen(mgr);
        }
    }
}
