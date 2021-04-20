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
pub use self::ir::param;
pub use self::ir::stand_alone_sig;
pub use self::ir::ty;

pub static CCTOR_NAME: &'static str = ".cctor";
pub static CTOR_NAME: &'static str = ".ctor";
