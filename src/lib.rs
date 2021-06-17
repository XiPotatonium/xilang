extern crate pest;
extern crate pest_derive;

mod ir;

pub use ir::attrib;
pub use ir::sig;
pub use ir::tok;
pub use ir::util;

pub use ir::code;
pub use ir::file;
pub use ir::generic;
pub use ir::inst;
pub use ir::member;
pub use ir::module;
pub use ir::stand_alone_sig;
pub use ir::ty;

pub use ir::inst::Inst;
pub use ir::member::{Field, ImplMap, MemberRef, MethodDef};
pub use ir::module::{Mod, ModRef};
pub use ir::param::Param;
pub use ir::ty::{TypeDef, TypeRef};

pub static CCTOR_NAME: &'static str = ".cctor";
pub static CTOR_NAME: &'static str = ".ctor";
