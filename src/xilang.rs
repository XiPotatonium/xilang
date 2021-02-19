extern crate clap;

mod ir;
mod vm;

use clap::{App, Arg};

use std::path::PathBuf;

use vm::mem::Mem;
use vm::VMCfg;
use vm::loader::load;

fn main() {
    let cfg = {
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

        VMCfg {
            entry: PathBuf::from(entry),
            ext_paths: ext_paths
                .split(';')
                .map(|x| x.to_owned())
                .collect::<Vec<String>>(),
            diagnose: matches.is_present("diagnose"),
        }
    };

    let mut m = Mem::new();

    load(&cfg, &mut m);
}
