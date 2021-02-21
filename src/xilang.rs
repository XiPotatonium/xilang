extern crate clap;

mod ir;
mod vm;

use clap::{App, Arg};

use std::fs;

use vm::mem::SharedMem;
use vm::VMCfg;
use vm::{executor::TExecutor, loader::load};

fn main() {
    let (entry, cfg) = {
        let matches = App::new("xilang")
            .version("0.1.0")
            .author("Xi")
            .about("Hello world! This is xilang")
            .arg(
                Arg::with_name("entry")
                    .help("Entry module of executable")
                    .required(true)
                    .index(1),
            )
            .arg(
                Arg::with_name("ext")
                    .help("External module paths")
                    .short("i")
                    .long("import")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("diagnose")
                    .short("d")
                    .long("diagnose")
                    .help("Run diagnose or not")
                    .takes_value(false),
            )
            .get_matches();

        let entry = matches.value_of("entry").unwrap();
        let ext_paths = matches.value_of("ext").unwrap_or("");

        let ext_paths = if ext_paths.len() == 0 {
            Vec::new()
        } else {
            ext_paths
                .split(';')
                .map(|x| fs::canonicalize(x).unwrap())
                .collect()
        };

        let entry = fs::canonicalize(entry).unwrap();
        let entry_root = entry.parent().unwrap().to_owned();

        (
            entry,
            VMCfg {
                entry_root,
                ext_paths,
                diagnose: matches.is_present("diagnose"),
            },
        )
    };

    let mut m = SharedMem::new();

    let (static_inits, entry) = load(entry, &mut m, &cfg);

    // static inits
    for static_init in static_inits.iter() {
        let mut executor = TExecutor::new();
        executor.run(&static_init, &mut m);
    }

    let mut executor = TExecutor::new();
    executor.run(&entry, &mut m);
}
