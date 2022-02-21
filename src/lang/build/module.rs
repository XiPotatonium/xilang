use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::ptr::NonNull;

use super::super::ast::AST;
use super::super::parser;
use super::super::sym::Module;
use super::super::XiCfg;
use super::{ClassBuilder, FileLoader, FuncBuilder, TypeLinkContext};
use core::util::{IItemPath, ItemPathBuf};

static CACHE_DIRNAME: &str = ".xicache";

pub struct ModuleBuilder {
    pub fpath: PathBuf,
    pub sym: NonNull<Module>,
    use_map: HashMap<String, ItemPathBuf>,
    pub classes: Vec<Box<ClassBuilder>>,
    /// no overload
    pub funcs: Vec<Box<FuncBuilder>>,
}

impl ModuleBuilder {
    pub fn load(
        mod_path: ItemPathBuf,
        fpath: PathBuf,
        loader: &mut FileLoader,
        cfg: &XiCfg,
    ) -> Box<Module> {
        let mut this_mod = Box::new(Module {
            path: mod_path.clone(),
            sub_mods: HashMap::new(),
            use_map: HashMap::new(),
            classes: HashMap::new(),
            funcs: HashMap::new(),
        });
        let this_mod_nonnull =
            NonNull::new(this_mod.as_ref() as *const Module as *mut Module).unwrap();
        let mut module_builder = Box::new(ModuleBuilder {
            sym: this_mod_nonnull,
            fpath,
            use_map: HashMap::new(),
            classes: Vec::new(),
            funcs: Vec::new(),
        });

        // Parse source file
        let ast = parser::parse(&module_builder.fpath).unwrap();

        if cfg.dump_ast {
            let mut cache_dir = module_builder.fpath.parent().unwrap().to_owned();
            cache_dir.push(CACHE_DIRNAME);
            fs::create_dir_all(&cache_dir).unwrap();
            // save ast to .json file
            let mut f = fs::File::create(&cache_dir.join(format!(
                "{}.ast.json",
                module_builder.fpath.file_stem().unwrap().to_str().unwrap()
            )))
            .unwrap();
            write!(f, "{}", ast).unwrap();
        }

        if let AST::File(_, module_declares, uses, items) = *ast {
            // load all sub modules
            for sub_mod_name in module_declares.into_iter() {
                if this_mod.sub_mods.contains_key(&sub_mod_name) {
                    panic!(
                        "Sub-module {} is defined multiple times in {}",
                        sub_mod_name,
                        module_builder.fpath.display()
                    );
                }

                let mut sub_mod_path = mod_path.clone();
                sub_mod_path.push(&sub_mod_name);

                // add sub-module to use map
                module_builder
                    .use_map
                    .insert(sub_mod_name.to_owned(), sub_mod_path.clone());

                // locate sub module
                let mut input_dir = cfg.entry_path.parent().unwrap().to_owned();
                for seg_id in mod_path.iter().skip(1) {
                    input_dir.push(seg_id);
                }
                let fpath1 = input_dir.with_extension("xi"); // entry/.../mod_name.xi
                let fpath2 = input_dir.join("mod.xi"); // entry/.../mod_name/mod.xi
                if fpath1.is_file() && fpath2.is_file() {
                    panic!(
                        "Ambiguous module {}. {} or {}?",
                        mod_path,
                        fpath1.display(),
                        fpath2.display()
                    );
                }
                let sub_mod_fpath = if fpath1.is_file() {
                    fpath1
                } else if fpath2.is_file() {
                    fpath2
                } else {
                    panic!("Cannot find module {}", mod_path,);
                };

                this_mod.sub_mods.insert(
                    sub_mod_name.to_owned(),
                    Self::load(sub_mod_path, sub_mod_fpath, loader, cfg),
                );
            }

            // process uses
            for use_stmt in uses.iter() {
                if let AST::Use(path, alias) = use_stmt.as_ref() {
                    let alias = if let Some(alias) = alias {
                        alias.clone()
                    } else {
                        path.to_string()
                    };
                    if this_mod.sub_mods.contains_key(&alias) {
                        panic!(
                            "{} is a sub-module but also a used symbol in file {}",
                            alias,
                            module_builder.fpath.display()
                        );
                    } else if this_mod.use_map.contains_key(&alias) {
                        panic!(
                            "{} is ambiguous in used symbols of file {}",
                            alias,
                            module_builder.fpath.display()
                        );
                    }
                    module_builder.use_map.insert(alias, path.clone());
                } else {
                    unreachable!()
                }
            }

            // declare all items
            for item in items.into_iter() {
                match *item {
                    AST::Class(class_ast) => {
                        let class_name = class_ast.name.clone();
                        module_builder.check_item_ambiguity(&class_name).unwrap();
                        let mut class_path = this_mod.path.clone();
                        class_path.push(&class_name);
                        this_mod.classes.insert(
                            class_name,
                            ClassBuilder::load(class_path, module_builder.as_mut(), class_ast),
                        );
                    }
                    AST::Func(func_ast) => {
                        let func_name = func_ast.name.clone();
                        module_builder.check_item_ambiguity(&func_name).unwrap();
                        let mut func_path = this_mod.path.clone();
                        func_path.push(&func_name);
                        this_mod.funcs.insert(
                            func_name,
                            FuncBuilder::load_func(func_path, module_builder.as_mut(), func_ast),
                        );
                    }
                    _ => unreachable!(),
                };
            }

            loader.builders.push(module_builder);
            loader
                .module_map
                .insert(mod_path.to_string(), this_mod_nonnull);

            this_mod
        } else {
            unreachable!()
        }
    }

    fn check_item_ambiguity(&self, name: &str) -> Result<(), String> {
        let sym = unsafe { self.sym.as_ref() };
        if sym.sub_mods.contains_key(name) {
            Err(format!("Already exists a sub module named {}", name))
        } else if sym.classes.contains_key(name) {
            Err(format!("Already exists a class named {}", name))
        } else if sym.funcs.contains_key(name) {
            Err(format!("Already exists a function named {}", name))
        } else {
            Ok(())
        }
    }

    pub fn link_type(&mut self, ctx: TypeLinkContext) {
        let mut ctx = ctx;
        for (alias, path) in self.use_map.iter() {
            unsafe { self.sym.as_mut() }
                .use_map
                .insert(alias.clone(), ctx.resolve(path));
        }
        for func_builder in self.funcs.iter_mut() {
            func_builder.link_type(&ctx);
        }
        for class_builder in self.classes.iter_mut() {
            ctx.class = class_builder.sym.as_ptr();
            class_builder.link_type(&ctx);
        }
    }

    pub fn code_gen(&mut self, cfg: &XiCfg) {
        for class_builder in self.classes.iter_mut() {
            class_builder.code_gen(cfg);
        }
    }

    pub fn dump(&self, _: &XiCfg) {
        unimplemented!();
    }
}
