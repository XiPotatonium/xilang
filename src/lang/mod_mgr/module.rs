use core::panic;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use xir::attrib::*;
use xir::file::IrFile;
use xir::inst::Inst;
use xir::tok::{to_tok, TokTag};
use xir::util::path::{IModPath, ModPath};
use xir::{CCTOR_NAME, CTOR_NAME};

use super::super::gen::{gen, Builder, CodeGenCtx, MethodBuilder, RValType, ValType};
use super::super::parser;
use super::super::XicCfg;
use super::ModRef;
use super::{super::ast::AST, external::load_external_crate};
use super::{Arg, Class, Field, Locals, Method, ModMgr};

// use macro to avoid borrow mutable self twice, SB rust
macro_rules! declare_method {
    ($class: expr, $builder: expr, $id: expr, $flag: expr, $impl_flag: expr, $ret_ty: expr, $ps: expr) => {{
        let idx = $builder
            .borrow_mut()
            .add_method($id, &$ps, &$ret_ty, $flag, $impl_flag);

        let method = Box::new(Method {
            ret_ty: $ret_ty,
            ps_ty: $ps,
            flag: $flag.clone(),
            impl_flag: $impl_flag.clone(),
            idx,
        });
        // let sig = format!("{}{}", $id, method.descriptor());

        if let Some(old_method) = $class.methods.insert($id.to_owned(), method) {
            // TODO: use expect_none once it becomes stable
            panic!(
                "Duplicated method {}{} in class {}",
                $id,
                old_method.descriptor(),
                $class.name
            );
        }
        idx
    }};
}

pub struct Module {
    pub mod_path: ModPath,
    pub sub_mods: HashSet<String>,
    /// key: class_name
    pub classes: HashMap<String, Rc<RefCell<Class>>>,

    /// Vec<Box<AST::Class>>
    class_asts: Option<Vec<Box<AST>>>,
    is_dir_mod: bool,
    pub use_map: HashMap<String, ModPath>,

    pub builder: RefCell<Builder>,
}

impl Module {
    /// Create a module from directory
    pub fn from_xi(
        mod_path: ModPath,
        rel_dir: &Path,
        fpath: &Path,
        is_dir_mod: bool,
        mod_tbl: &mut HashMap<String, Box<ModRef>>,
        cfg: &XicCfg,
    ) {
        let output_dir = cfg.out_dir.join(rel_dir);
        let mod_self_name = mod_path.get_self_name().unwrap();

        let builder = Builder::new(mod_path.as_str());

        // Parse source file
        let ast = parser::peg_parse(fpath).unwrap();

        if cfg.verbose >= 2 {
            // save ast to .json file
            let mut f =
                fs::File::create(output_dir.join(format!("{}.ast.json", mod_self_name))).unwrap();
            write!(f, "{}", ast).unwrap();
        }

        if let AST::File(mods, exts, uses, classes) = *ast {
            if mod_path.len() == 1 {
                // load external modules specified in root module
                let mut exts_map: HashMap<String, Option<&PathBuf>> = HashMap::new();
                for ext in exts.iter() {
                    if let Some(_) = exts_map.insert(ext.to_owned(), None) {
                        panic!("Declaring duplicated external module {}", ext);
                    }
                }
                for ext_path in cfg.ext_paths.iter() {
                    let file = IrFile::from_binary(Box::new(fs::File::open(ext_path).unwrap()));
                    let ext_mod_name = file.mod_name();
                    // only import declared external modules
                    if let Some(imported) = exts_map.get_mut(ext_mod_name) {
                        if let Some(old_path) = imported {
                            panic!(
                                "Ambiguous external module {}: {} or {}?",
                                ext_mod_name,
                                old_path.display(),
                                ext_path.display()
                            );
                        } else {
                            *imported = Some(ext_path);
                        }
                    }
                    load_external_crate(mod_tbl, ext_path.parent().unwrap(), file);
                }
            } else if exts.len() != 0 {
                println!("Warning: {} is not root mod. External mod specified in this file won't take effect", fpath.display());
            }

            let mut use_map: HashMap<String, ModPath> = HashMap::new();
            // process sub mods
            let mut sub_mods = HashSet::new();
            let sub_mod_rel_dir = if mod_path.len() == 1 || is_dir_mod {
                // this is the root mod in this dir
                // search sub modules in this directory
                rel_dir.to_owned()
            } else {
                // this is a normal file mod
                // search sub modules in directory dir/mod_name
                rel_dir.join(mod_self_name)
            };
            // input_root/sub_mod_rel_dir
            let sub_mod_input_dir = cfg.root_dir.join(&sub_mod_rel_dir);
            // output_root/sub_mod_rel_dir
            let sub_mod_output_dir = cfg.out_dir.join(&sub_mod_rel_dir);

            for sub_mod_name in mods.into_iter() {
                if !sub_mods.insert(sub_mod_name.clone()) {
                    panic!(
                        "Sub-module {} is defined multiple times in {}",
                        sub_mod_name,
                        mod_path.as_str()
                    );
                }

                let mut sub_mod_path = mod_path.clone();
                sub_mod_path.push(&sub_mod_name);

                // sub mods will be use by default
                if use_map.contains_key(&sub_mod_name) {
                    panic!(
                        "Ambiguous id {}. Both a sub module and a using token",
                        sub_mod_name
                    );
                } else {
                    use_map.insert(sub_mod_name.clone(), sub_mod_path.clone());
                }

                // possible locations
                // * input_root/sub_mod_rel_dir/sub_mod_name.xi
                // * input_root/sub_mod_rel_dir/sub_mod_name/mod.xi
                // * input_root/sub_mod_rel_dir/sub_mod_name.xir
                // * input_root/sub_mod_rel_dir/sub_mod_name/sub_mod_name.xir

                // input_root/sub_mod_rel_dir/sub_mod_name.xi
                let sub_mod_fpath = sub_mod_input_dir.join(format!("{}.xi", sub_mod_name));
                // input_root/sub_mod_rel_dir/sub_mod_name/mod.xi
                let sub_mod_dpath = sub_mod_input_dir.join(format!("{}\\mod.xi", &sub_mod_name));

                if sub_mod_fpath.is_file() && sub_mod_dpath.is_file() {
                    panic!(
                        "Ambiguous sub-module {} in {}. {} or {}?",
                        sub_mod_name,
                        mod_path.as_str(),
                        sub_mod_dpath.display(),
                        sub_mod_fpath.display()
                    );
                } else if sub_mod_dpath.is_file() {
                    // prepare output dir for sub dir mod
                    // output_dir/sub_mod_rel_dir/sub_mod_name
                    let sub_mod_out_dir = sub_mod_output_dir.join(&sub_mod_name);
                    if !sub_mod_out_dir.exists() {
                        fs::create_dir_all(sub_mod_out_dir).unwrap();
                    } else if !sub_mod_out_dir.is_dir() {
                        panic!("Path {} is not a directory", sub_mod_out_dir.display());
                    }

                    let sub_mod_rel_dir = sub_mod_rel_dir.join(&sub_mod_name);
                    Module::from_xi(
                        sub_mod_path,
                        &sub_mod_rel_dir,
                        &sub_mod_dpath,
                        true,
                        mod_tbl,
                        cfg,
                    );
                } else if sub_mod_fpath.is_file() {
                    Module::from_xi(
                        sub_mod_path,
                        &sub_mod_rel_dir,
                        &sub_mod_fpath,
                        false,
                        mod_tbl,
                        cfg,
                    );
                } else {
                    panic!(
                        "Cannot find sub-module {} in {} (Consider create {} or {})",
                        sub_mod_name,
                        mod_path.as_str(),
                        sub_mod_dpath.display(),
                        sub_mod_fpath.display()
                    );
                }
            }

            // generate all classes
            let mut class_map = HashMap::new();
            for class in classes.iter() {
                if let AST::Class(id, flag, _, _, _, _) = class.as_ref() {
                    if sub_mods.contains(id) {
                        panic!(
                            "Ambiguous name {} in module {}. Both a sub-module and a class",
                            id, mod_self_name
                        );
                    }

                    class_map.insert(
                        id.to_owned(),
                        Rc::new(RefCell::new(Class::new(id.to_owned(), 0, flag.clone()))),
                    );
                } else {
                    unreachable!();
                }
            }

            // process uses
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

                    todo!("TODO: Validate use path");
                } else {
                    unreachable!();
                }
            }

            let this_mod = Module {
                mod_path,
                sub_mods,
                classes: class_map,
                class_asts: Some(classes),
                is_dir_mod,
                use_map,

                builder: RefCell::new(builder),
            };
            mod_tbl.insert(
                this_mod.fullname().to_owned(),
                Box::new(ModRef::Mod(this_mod)),
            );
        } else {
            unreachable!();
        }
    }

    pub fn name(&self) -> &str {
        self.mod_path.get_self_name().unwrap()
    }

    pub fn fullname(&self) -> &str {
        self.mod_path.as_str()
    }

    pub fn is_root(&self) -> bool {
        self.mod_path.len() == 1
    }

    pub fn dump(&self, mgr: &ModMgr, out_dir: &Path) {
        let mut p = out_dir.join(format!("{}.xir", self.name()));

        // dump xir
        let mut f = fs::File::create(&p).unwrap();
        write!(f, "{}", self.builder.borrow().file).unwrap();

        p.set_extension("xibc");
        let buf = self.builder.borrow().file.to_binary();
        let mut f = fs::File::create(&p).unwrap();
        f.write_all(&buf).unwrap();

        let out_dir = if self.is_dir_mod {
            p.set_file_name(self.name());
            if !p.exists() {
                fs::create_dir(&p).unwrap();
            } else if !p.is_dir() {
                panic!("{} already exists but it is not a directory", p.display());
            }
            &p
        } else {
            out_dir
        };

        for sub in self.sub_mods.iter() {
            let mut sub_mod_path = self.mod_path.clone();
            sub_mod_path.push(sub);
            let sub = mgr.mod_tbl.get(sub_mod_path.as_str()).unwrap().expect_mod();
            sub.dump(mgr, out_dir);
        }
    }
}

impl Module {
    pub fn get_ty(&self, ast: &Box<AST>, c: &ModMgr) -> RValType {
        match ast.as_ref() {
            AST::TypeI32 => RValType::I32,
            AST::TypeF64 => RValType::F64,
            AST::TypeBool => RValType::Bool,
            AST::None => RValType::Void,
            AST::TypeTuple(_) => {
                unimplemented!();
            }
            AST::Path(class_path) => {
                let (has_crate, super_cnt, class_path) = class_path.canonicalize();
                let class_id = class_path.get_self_name().unwrap();
                let mod_path = class_path.get_super();
                if mod_path.len() == 0 {
                    // this mod
                    // might be a class in this module
                    if self.classes.contains_key(class_id) {
                        RValType::Obj(self.fullname().to_owned(), class_id.to_owned())
                    } else {
                        panic!("No class {} in mod {}", class_id, self.fullname());
                    }
                } else {
                    let m = if has_crate {
                        // crate::...
                        let mut m = ModPath::new();
                        m.push(&c.name);
                        for seg in mod_path.iter().skip(1) {
                            m.push(seg);
                        }
                        m
                    } else if super_cnt != 0 {
                        // super::...
                        let mut m = self.mod_path.as_slice();
                        for _ in (0..super_cnt).into_iter() {
                            m.to_super();
                        }
                        let mut m = m.to_owned();
                        for seg in mod_path.iter().skip(super_cnt) {
                            m.push(seg);
                        }
                        m
                    } else {
                        let mut mod_path_iter = mod_path.iter();
                        let r = mod_path_iter.next().unwrap();
                        if let Some(m) = self.use_map.get(r) {
                            let mut m = m.clone();
                            for seg in mod_path_iter {
                                m.push(seg);
                            }
                            m
                        } else {
                            mod_path.to_owned()
                        }
                    };

                    if let Some(m) = c.mod_tbl.get(m.as_str()) {
                        if let Some(_) = m.get_class(class_id) {
                            RValType::Obj(m.fullname().to_owned(), class_id.to_owned())
                        } else {
                            panic!("Class {} not found", class_id);
                        }
                    } else {
                        panic!("Module {} not found", m.as_str());
                    }
                }
            }
            AST::TypeArr(dtype, _) => RValType::Array(Box::new(self.get_ty(dtype, c))),
            _ => unreachable!(),
        }
    }
}

// member pass
impl Module {
    pub fn member_pass(&self, c: &ModMgr) {
        if let Some(class_asts) = &self.class_asts {
            for class in class_asts.iter() {
                if let AST::Class(class_id, class_flag, _, ast_methods, ast_fields, static_init) =
                    class.as_ref()
                {
                    let mut class_mut = self.classes.get(class_id).unwrap().borrow_mut();
                    class_mut.idx = self.builder.borrow_mut().add_class(class_id, class_flag);

                    for field in ast_fields.iter() {
                        if let AST::Field(id, flag, _, ty) = field.as_ref() {
                            // Field will have default initialization
                            let ty = self.get_ty(ty, c);

                            // Build Field in class file
                            let idx = self.builder.borrow_mut().add_field(id, &ty, flag);

                            let field = Box::new(Field::new(*flag, ty, idx));

                            if !flag.is(FieldAttribFlag::Static) {
                                // non-static field
                                class_mut.non_static_fields.push(id.to_owned());
                            }
                            if let Some(_) = class_mut.fields.insert(id.to_owned(), field) {
                                // TODO: use expect_none once it becomes stable
                                panic!("Dulicated field {} in class {}", id, class_mut.name);
                            }
                        }
                    }

                    // Add static init
                    match static_init.as_ref() {
                        AST::Block(_) => {
                            let ret_ty = RValType::Void;
                            let ps: Vec<RValType> = vec![];
                            let flag = MethodAttrib::from(
                                u16::from(MethodAttribFlag::Pub)
                                    | u16::from(MethodAttribFlag::Static)
                                    | u16::from(MethodAttribFlag::RTSpecialName),
                            );
                            let impl_flag = MethodImplAttrib::new(
                                MethodImplAttribCodeTypeFlag::IL,
                                MethodImplAttribManagedFlag::Managed,
                            );
                            declare_method!(
                                class_mut,
                                self.builder,
                                CCTOR_NAME,
                                &flag,
                                &impl_flag,
                                ret_ty,
                                ps
                            );
                        }
                        AST::None => (),
                        _ => unreachable!("Parser error"),
                    };

                    // Add default object creator
                    {
                        let ret_ty = RValType::Void;
                        let mut ps: Vec<RValType> = vec![RValType::Obj(
                            self.fullname().to_owned(),
                            class_mut.name.clone(),
                        )];
                        for f in class_mut.non_static_fields.iter() {
                            ps.push(class_mut.fields.get(f).unwrap().ty.clone());
                        }
                        let flag = MethodAttrib::from(
                            u16::from(MethodAttribFlag::Pub)
                                | u16::from(MethodAttribFlag::Static)
                                | u16::from(MethodAttribFlag::RTSpecialName),
                        );
                        let impl_flag = MethodImplAttrib::new(
                            MethodImplAttribCodeTypeFlag::IL,
                            MethodImplAttribManagedFlag::Managed,
                        );
                        declare_method!(
                            class_mut,
                            self.builder,
                            CTOR_NAME,
                            &flag,
                            &impl_flag,
                            ret_ty,
                            ps
                        );
                    }

                    for method in ast_methods.iter() {
                        if let AST::Method(id, flag, attrs, ty, ps, _) = method.as_ref() {
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
                            let mut impl_flag = MethodImplAttrib::new(
                                MethodImplAttribCodeTypeFlag::IL,
                                MethodImplAttribManagedFlag::Managed,
                            );
                            for attr in attrs.iter() {
                                if let AST::CustomAttr(id, args) = attr.as_ref() {
                                    if id == "Dllimport" {
                                        // TODO: use real attribute object
                                        // Currently it's adhoc
                                        assert_eq!(
                                            args.len(),
                                            1,
                                            "Invalid arg for Dllimport attribute"
                                        );
                                        if let AST::String(_) = args[0].as_ref() {
                                            impl_flag
                                                .set_code_ty(MethodImplAttribCodeTypeFlag::Native);
                                            impl_flag.set_managed(
                                                MethodImplAttribManagedFlag::Unmanaged,
                                            );
                                        } else {
                                            panic!("Invalid arg for Dllimport attribute");
                                        }
                                    } else {
                                        panic!("Unrecognizable custom attribute {}", id);
                                    }
                                } else {
                                    unreachable!();
                                }
                            }

                            let method_idx = declare_method!(
                                class_mut,
                                self.builder,
                                id,
                                flag,
                                &impl_flag,
                                ret_ty,
                                ps
                            );
                            for (id, args) in attrs.iter().map(|attr| {
                                if let AST::CustomAttr(id, args) = attr.as_ref() {
                                    (id, args)
                                } else {
                                    unreachable!()
                                }
                            }) {
                                if id == "Dllimport" {
                                    // TODO: use real attribute object
                                    // Currently it's adhoc
                                    if let AST::String(v) = args[0].as_ref() {
                                        let pinvoke_attrib = PInvokeAttrib::new(
                                            PInvokeAttribCharsetFlag::Ansi,
                                            PInvokeAttribCallConvFlag::CDecl,
                                        );
                                        self.builder.borrow_mut().add_extern_fn(
                                            v,
                                            id,
                                            &pinvoke_attrib,
                                            method_idx,
                                        );
                                    } else {
                                        unreachable!();
                                    }
                                } else {
                                    unreachable!();
                                }
                            }
                        }
                    }

                    if self.is_root() && class_id == "Program" {
                        if let Some(m) = class_mut.methods.get("main") {
                            if let RValType::Void = m.ret_ty {
                                if m.ps_ty.len() == 0
                                    && m.flag.is(MethodAttribFlag::Pub)
                                    && m.flag.is(MethodAttribFlag::Static)
                                {
                                    // pub Program::main()
                                    self.builder.borrow_mut().file.mod_tbl[0].entrypoint = m.idx;
                                }
                            }
                        }
                    }
                } else {
                    unreachable!();
                }
            }
        }
    }
}

// code gen
impl Module {
    fn code_gen_method(
        &self,
        c: &ModMgr,
        cfg: &XicCfg,
        class: &Class,
        m: &Method,
        ps: &Vec<Box<AST>>,
        block: &Box<AST>,
    ) {
        let mut args_map: HashMap<String, Arg> = HashMap::new();
        if !m.flag.is(MethodAttribFlag::Static) {
            // non-static method param "self"
            args_map.insert(
                String::from("self"),
                Arg::new(
                    ParamAttrib::from(0),
                    RValType::Obj(self.fullname().to_owned(), class.name.clone()),
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
            cfg,
            module: self,
            class,
            locals: RefCell::new(Locals::new()),
            method: m,
            args_map,
            method_builder: RefCell::new(MethodBuilder::new()),
            loop_ctx: RefCell::new(vec![]),
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

    pub fn code_gen(&self, c: &ModMgr, cfg: &XicCfg) {
        if let Some(class_asts) = &self.class_asts {
            for class in class_asts.iter() {
                if let AST::Class(id, _, _, ast_methods, _, ast_init) = class.as_ref() {
                    let class_ref = self.classes.get(id).unwrap().borrow();
                    // gen static init
                    match ast_init.as_ref() {
                        AST::Block(_) => {
                            let m = class_ref.methods.get(CCTOR_NAME).unwrap();
                            let ps: Vec<Box<AST>> = vec![];
                            self.code_gen_method(c, cfg, &class_ref, m, &ps, ast_init);
                        }
                        AST::None => (),
                        _ => unreachable!("Parser error"),
                    };

                    // gen default creator
                    // ldarg.0
                    // dup
                    // ...
                    // dup
                    // ldarg.1
                    // stfld <field0>
                    // ldarg.2
                    // stfld <field1>
                    // ...
                    {
                        let m = class_ref.methods.get(CTOR_NAME).unwrap();
                        let mut method_builder = MethodBuilder::new();
                        if m.ps_ty.len() == 1 {
                            // no field
                        } else {
                            method_builder.add_inst_ldarg(0);
                            for _ in (2..m.ps_ty.len()).into_iter() {
                                method_builder.add_inst(Inst::Dup);
                            }
                            for (i, f_id) in (1..m.ps_ty.len())
                                .into_iter()
                                .zip(class_ref.non_static_fields.iter())
                            {
                                method_builder.add_inst_ldarg(i as u16);
                                method_builder.add_inst(Inst::StFld(to_tok(
                                    class_ref.fields.get(f_id).unwrap().idx,
                                    TokTag::Field,
                                )));
                            }
                        }
                        method_builder.add_inst(Inst::Ret);
                        self.builder.borrow_mut().done(
                            &mut method_builder,
                            m.idx,
                            0,
                            cfg.optim >= 1,
                        );
                    }

                    for method_ast in ast_methods.iter() {
                        if let AST::Method(id, _, _, _, ps, block) = method_ast.as_ref() {
                            if let AST::None = block.as_ref() {
                                // extern function
                            } else {
                                let m = class_ref.methods.get(id).unwrap();
                                self.code_gen_method(c, cfg, &class_ref, m, ps, block);
                            }
                        } else {
                            unreachable!("Parser error");
                        }
                    }
                } else {
                    unreachable!();
                }
            }
        }
    }
}
