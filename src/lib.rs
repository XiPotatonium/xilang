extern crate pest;
extern crate pest_derive;

mod ir;

pub use ir::constant::Constant;
pub use ir::flags;
pub use ir::inst::Instruction;
pub use ir::{ClassFile, Field, Method};

pub const CTOR_NAME: &str = "<init>";
pub const CCTOR_NAME: &str = "<clinit>";

pub const STRING_CLASS_NAME: &str = "java/lang/String";
pub const OBJECT_CLASS_NAME: &str = "java/lang/Object";
