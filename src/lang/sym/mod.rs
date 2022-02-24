mod class;
mod field;
mod func;
mod module;
mod ty;

use core::util::{IItemPath, ItemPath, ItemPathBuf};
use std::collections::HashMap;
use std::fmt;
use std::ptr::NonNull;

pub use self::module::Module;
pub use self::ty::{RValType, ValExpectation, ValType};
pub use class::Class;
pub use field::Field;
pub use func::{Func, Param};

use super::ast::ASTType;
use super::build::FileLoader;
use super::STRING_CLASS_NAME;

#[derive(Clone)]
pub enum Symbol {
    Module(NonNull<Module>),
    Class(NonNull<Class>),
    Field(NonNull<Field>),
    Func(NonNull<Func>),
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Func(method) => write!(f, "(Func){}", unsafe { method.as_ref() }),
            Self::Field(field) => write!(f, "(Field){}", unsafe { field.as_ref() }),
            Self::Class(class) => write!(f, "(Class){}", unsafe { class.as_ref() }),
            Self::Module(m) => write!(f, "(Mod){}", unsafe { m.as_ref() }),
            // Self::Local(n) => write!(f, "(Local){}", n),
            // Self::KwLSelf => write!(f, "(Arg)self"),
            // Self::Arg(n) => write!(f, "(Arg){}", n),
            // Self::ArrAcc(ele_ty) => write!(f, "(acc){}[]", ele_ty),
        }
    }
}

pub struct SymTblFrame {
    pub frame: HashMap<String, Symbol>,
}

pub struct SymTbl {
    pub type_link_ctx: TypeLinkContext,
    pub table: Vec<Box<SymTblFrame>>,
}

pub struct TypeLinkContext {
    pub loader: NonNull<FileLoader>,
    pub module: NonNull<Module>,
    pub class: *mut Class,
}

impl TypeLinkContext {
    pub fn resolve_rval_type(&self, ast: &ASTType) -> RValType {
        match ast {
            ASTType::I32 => RValType::I32,
            ASTType::F64 => RValType::F64,
            ASTType::Bool => RValType::Bool,
            ASTType::None => RValType::None,
            ASTType::Char => RValType::Char,
            ASTType::ISize => RValType::ISize,
            ASTType::USize => RValType::USize,
            ASTType::String => RValType::ClassRef(
                if let Symbol::Class(string_class) =
                    self.resolve(&ItemPathBuf::from_ir_path(STRING_CLASS_NAME))
                {
                    string_class
                } else {
                    unreachable!()
                },
            ),
            ASTType::Tuple(_) => unimplemented!(),
            ASTType::UsrType(class_path) => RValType::ClassRef(
                if let Symbol::Class(string_class) = self.resolve(class_path) {
                    string_class
                } else {
                    unreachable!()
                },
            ),
            ASTType::Arr(dtype) => RValType::Array(Box::new(self.resolve_rval_type(dtype))),
        }
    }

    /// Canonicalize path
    ///
    /// A valid canonicalized path: `("crate"? | "super"*) ~ Id*`
    ///
    /// Return:
    pub fn canonicalize_path(&self, path: ItemPath) -> ItemPathBuf {
        let mut canonicalized_path = ItemPathBuf::default();
        let mut is_super_prefix = true;
        let module = unsafe { self.module.as_ref() };
        for (i, id) in path.iter().enumerate() {
            if id == "crate" {
                if i == 0 {
                    canonicalized_path.push(module.path.get_root().unwrap());
                } else {
                    panic!(
                        "\"crate\" should be the first segment in path {} ({})",
                        path, canonicalized_path
                    );
                }
                is_super_prefix = false;
            } else if id == "super" {
                if is_super_prefix {
                    if i == 0 {
                        canonicalized_path = module.path.clone();
                    }
                    if let Err(msg) = canonicalized_path.pop() {
                        panic!("Invalid path {} ({}), {}", path, canonicalized_path, msg);
                    }
                } else {
                    panic!("Invalid path {}({})", path, canonicalized_path);
                }
            } else if id == "self" {
                is_super_prefix = false;
            } else {
                is_super_prefix = false;
                canonicalized_path.push(id);
            }
        }
        canonicalized_path
    }

    pub fn resolve(&self, path: &ItemPathBuf) -> Symbol {
        let canonicalized_path = self.canonicalize_path(path.as_slice());
        let this_module = unsafe { self.module.as_ref() };
        let loader = unsafe { self.loader.as_ref() };
        let root = path.get_root().unwrap();

        let mut scope = if root == "Self" {
            // refer to this struct
            if let Some(class_nonnull) = NonNull::new(self.class as *mut Class) {
                Symbol::Class(class_nonnull)
            } else {
                panic!("Invalid \"Self\" outside a class scope");
            }
        } else if root == "self" {
            Symbol::Module(self.module)
        } else if let Some(ty) = this_module.classes.get(root) {
            // refer to another class in this module
            Symbol::Class(NonNull::new(ty.as_ref() as *const Class as *mut Class).unwrap())
        } else if let Some(m) = this_module.use_map.get(root) {
            // try use map, sub-modules are used automatically
            m.clone()
        } else if let Some(root_module) = loader.module_map.get(root) {
            Symbol::Module(*root_module)
        } else {
            panic!("Invalid path {} ({})", path, canonicalized_path);
        };

        for seg in canonicalized_path.iter().skip(1) {
            match scope {
                Symbol::Module(module) => {
                    let module = unsafe { module.as_ref() };
                    if let Some(m) = module.sub_mods.get(seg) {
                        scope = Symbol::Module(
                            NonNull::new(m.as_ref() as *const Module as *mut Module).unwrap(),
                        )
                    } else if let Some(c) = module.classes.get(seg) {
                        scope = Symbol::Class(
                            NonNull::new(c.as_ref() as *const Class as *mut Class).unwrap(),
                        )
                    } else if let Some(f) = module.funcs.get(seg) {
                        scope = Symbol::Func(
                            NonNull::new(f.as_ref() as *const Func as *mut Func).unwrap(),
                        )
                    } else {
                        panic!();
                    }
                }
                Symbol::Class(class) => {
                    let class = unsafe { class.as_ref() };
                    if let Some(m) = class.methods.get(seg) {
                        scope = Symbol::Func(
                            NonNull::new(m.as_ref() as *const Func as *mut Func).unwrap(),
                        )
                    }
                    if let Some(f) = class.fields.get(seg) {
                        scope = Symbol::Field(
                            NonNull::new(f.as_ref() as *const Field as *mut Field).unwrap(),
                        )
                    } else {
                        panic!();
                    }
                }
                Symbol::Field(_) => panic!(),
                Symbol::Func(_) => panic!(),
            }
        }
        scope
    }
}
