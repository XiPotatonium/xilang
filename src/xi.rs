extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate clap;
extern crate regex;

extern crate core;

mod lang;
mod runtime;

use lang::build::FileLoader;
use lang::XiCfg;

use clap::{App, Arg};

use std::fs;
use std::time::SystemTime;

fn main() {
    let cfg = {
        let matches = App::new("xi")
            .version("0.1.0")
            .author("shwu")
            .about("Hello world! This is xi")
            .arg(Arg::new("entry").help("Entry file").required(true))
            .arg(
                Arg::new("compile")
                    .short('c')
                    .long("compile")
                    .help("Do not run. Only generate byte code in cache."),
            )
            .arg(Arg::new("ast").long("ast").help("Dump .ast.json in cache"))
            .get_matches();

        let entry_path = matches.value_of("entry").unwrap();
        let entry_path = fs::canonicalize(entry_path).unwrap();

        XiCfg {
            entry_path,
            dump_ast: matches.is_present("ast"),
            compile: matches.is_present("compile"),
        }
    };

    let start_time = SystemTime::now();
    let mut loader = FileLoader::load(&cfg);
    println!(
        "Parsing finished in {} seconds",
        SystemTime::now()
            .duration_since(start_time)
            .unwrap()
            .as_secs_f32()
    );

    if cfg.compile {
        let start_time = SystemTime::now();
        loader.build(&cfg);
        println!(
            "Build finished in {} seconds",
            SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_secs_f32()
        );

        let start_time = SystemTime::now();
        loader.dump(&cfg);
        println!(
            "Dump finished in {} seconds",
            SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_secs_f32()
        );
    }

    runtime::exec(&loader);
}
