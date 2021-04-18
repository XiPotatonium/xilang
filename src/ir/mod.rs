#[macro_use]
mod bc_serde;
mod text_serde;

pub mod attrib;
pub mod blob;
pub mod tok;
pub mod util;

pub mod code;
pub mod file;
pub mod inst;
pub mod member;
pub mod module;
pub mod param;
pub mod stand_alone_sig;
pub mod ty;

pub static CCTOR_NAME: &'static str = ".cctor";
pub static CTOR_NAME: &'static str = ".ctor";
