extern crate pest;
#[macro_use]
extern crate pest_derive;

mod ir;

pub use self::ir::bc_serde;
pub use self::ir::blob;
pub use self::ir::flag;
pub use self::ir::inst;
pub use self::ir::ir_file;
pub use self::ir::path;
pub use self::ir::text_serde;

pub static CCTOR_NAME: &'static str = ".cctor";
pub static CTOR_NAME: &'static str = ".ctor";
