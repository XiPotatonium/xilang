extern crate pest;
extern crate pest_derive;

mod ir;

pub use self::ir::blob;
pub use self::ir::file;
pub use self::ir::flag;
pub use self::ir::inst;
pub use self::ir::tok;
pub use self::ir::util;

pub static CCTOR_NAME: &'static str = ".cctor";
pub static CTOR_NAME: &'static str = ".ctor";
