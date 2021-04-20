extern crate pest;
extern crate pest_derive;

mod ir;

pub use self::ir::attrib;
pub use self::ir::blob;
pub use self::ir::tok;
pub use self::ir::util;

pub use self::ir::code;
pub use self::ir::file;
pub use self::ir::inst;
pub use self::ir::member;
pub use self::ir::module;
pub use self::ir::stand_alone_sig;
pub use self::ir::ty;

pub use ir::inst::Inst;
pub use ir::member::{Field, ImplMap, MemberRef, MethodDef};
pub use ir::module::{Mod, ModRef};
pub use ir::param::Param;
pub use ir::ty::{TypeDef, TypeRef};

pub static CCTOR_NAME: &'static str = ".cctor";
pub static CTOR_NAME: &'static str = ".ctor";
