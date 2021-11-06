extern crate pest;
extern crate pest_derive;

mod ir;

pub use ir::flags;

pub const CTOR_NAME: &str = "<init>";
pub const CCTOR_NAME: &str = "<clinit>";
