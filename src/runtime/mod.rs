use crate::lang::build::FileLoader;

mod interp;
mod mem;

pub fn exec(load: &FileLoader) {
    println!("This is xi runtime");
    if let Some(entry) = load.crates[0].funcs.get("main") {
    } else {
        panic!("entry not found");
    }
}
